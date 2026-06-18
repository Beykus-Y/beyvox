use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

use livekit::{
    options::{TrackPublishOptions, VideoCodec, VideoEncoding, VideoEncoderBackend},
    track::{LocalTrack, LocalVideoTrack, TrackSource},
    webrtc::{
        prelude::RtcVideoSource,
        video_frame::{I420Buffer, VideoFrame, VideoRotation},
        video_source::native::NativeVideoSource,
        prelude::VideoResolution,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ScreenInfo {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

struct ScreenEngine {
    video_track: LocalVideoTrack,
    stop_flag: Arc<AtomicBool>,
    room: Arc<livekit::Room>,
}

static SCREEN_ENGINE: Mutex<Option<ScreenEngine>> = Mutex::new(None);

// ─── Public Tauri commands ───────────────────────────────────────────────────

#[tauri::command]
pub fn list_screens() -> Vec<ScreenInfo> {
    // Use screenshots crate for enumeration (cross-platform, lightweight)
    screenshots::Screen::all()
        .unwrap_or_default()
        .into_iter()
        .map(|s| ScreenInfo {
            id: s.display_info.id,
            name: format!("Экран {}", s.display_info.id),
            width: s.display_info.width,
            height: s.display_info.height,
        })
        .collect()
}

#[tauri::command]
pub async fn start_screen_share(
    screen_id: u32,
    quality: String,
    fps: u8,
) -> Result<(), String> {
    let room = crate::voice::get_room()
        .ok_or("Не подключён к голосовому каналу")?;

    let (out_w, out_h): (u32, u32) = match quality.as_str() {
        "360p" => (640, 360),
        "720p" => (1280, 720),
        _ => (1920, 1080),
    };
    let fps = fps.clamp(1, 60);

    let source = NativeVideoSource::new(
        VideoResolution { width: out_w, height: out_h },
        false,
    );

    let track = LocalVideoTrack::create_video_track(
        "screen",
        RtcVideoSource::Native(source.clone()),
    );

    eprintln!("[screen] starting share: {}x{} @{}fps quality={}", out_w, out_h, fps, quality);

    room.local_participant()
        .publish_track(
            LocalTrack::Video(track.clone()),
            TrackPublishOptions {
                source: TrackSource::Screenshare,
                video_codec: VideoCodec::H264,
                simulcast: true,
                // SDK default: 1 low-fps layer at half resolution (3fps, 200kbps) — достаточно
                simulcast_layers: None,
                video_encoding: Some(VideoEncoding {
                    max_bitrate: match quality.as_str() {
                        "360p" => 500_000,
                        "720p" => 1_500_000,
                        _      => 3_000_000,
                    },
                    max_framerate: fps as f64,
                }),
                // NVENC на Windows, Auto fallback на остальных (LibWebRTC логирует если недоступно)
                video_encoder: {
                    #[cfg(target_os = "windows")]
                    { VideoEncoderBackend::Nvenc }
                    #[cfg(not(target_os = "windows"))]
                    { VideoEncoderBackend::Auto }
                },
                ..Default::default()
            },
        )
        .await
        .map_err(|e| format!("publish screen track: {e}"))?;

    // Останавливаем предыдущий стрим
    let old = SCREEN_ENGINE.lock().unwrap().take();
    if let Some(old_engine) = old {
        old_engine.stop_flag.store(true, Ordering::Relaxed);
        let _ = old_engine.room.local_participant()
            .unpublish_track(&old_engine.video_track.sid()).await;
    }

    let stop_flag = Arc::new(AtomicBool::new(false));

    // Запускаем захват на выделенном OS-потоке (не tokio — это CPU-bound работа)
    #[cfg(target_os = "windows")]
    start_capture_dxgi(screen_id, out_w, out_h, fps, source, stop_flag.clone());

    #[cfg(not(target_os = "windows"))]
    start_capture_fallback(screen_id, out_w, out_h, fps, source, stop_flag.clone());

    *SCREEN_ENGINE.lock().unwrap() = Some(ScreenEngine {
        video_track: track,
        stop_flag,
        room,
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_screen_share() -> Result<(), String> {
    let engine = SCREEN_ENGINE.lock().unwrap().take();
    if let Some(e) = engine {
        e.stop_flag.store(true, Ordering::Relaxed);
        let _ = e.room.local_participant().unpublish_track(&e.video_track.sid()).await;
    }
    Ok(())
}

// ─── Windows: DXGI Desktop Duplication capture via windows-capture ───────────

#[cfg(target_os = "windows")]
fn start_capture_dxgi(
    screen_id: u32,
    out_w: u32,
    out_h: u32,
    fps: u8,
    source: NativeVideoSource,
    stop_flag: Arc<AtomicBool>,
) {
    use windows_capture::{
        capture::{Context, GraphicsCaptureApiHandler},
        frame::Frame,
        graphics_capture_api::InternalCaptureControl,
        monitor::Monitor,
        settings::{
            ColorFormat, CursorCaptureSettings, DirtyRegionSettings,
            DrawBorderSettings, MinimumUpdateIntervalSettings,
            SecondaryWindowSettings, Settings,
        },
    };

    // Находим монитор по индексу (screen_id начинается с 0 или 1 в зависимости от ОС)
    let monitor = Monitor::from_index(screen_id as usize)
        .or_else(|_| Monitor::primary())
        .unwrap_or_else(|_| {
            eprintln!("[screen] Failed to find monitor {screen_id}, falling back to primary");
            Monitor::primary().expect("No monitor found")
        });

    struct CaptureHandler {
        source: NativeVideoSource,
        out_w: u32,
        out_h: u32,
        stop_flag: Arc<AtomicBool>,
        interval: std::time::Duration,
        last_frame: std::time::Instant,
        nopadding_buf: Vec<u8>,
    }

    impl GraphicsCaptureApiHandler for CaptureHandler {
        type Flags = (NativeVideoSource, u32, u32, Arc<AtomicBool>, u8);
        type Error = Box<dyn std::error::Error + Send + Sync>;

        fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
            let (source, out_w, out_h, stop_flag, fps) = ctx.flags;
            Ok(Self {
                source,
                out_w,
                out_h,
                stop_flag,
                interval: std::time::Duration::from_millis(1000 / fps as u64),
                last_frame: std::time::Instant::now()
                    .checked_sub(std::time::Duration::from_secs(1))
                    .unwrap_or(std::time::Instant::now()),
                nopadding_buf: Vec::with_capacity((out_w * out_h * 4) as usize),
            })
        }

        fn on_frame_arrived(
            &mut self,
            frame: &mut Frame,
            capture_control: InternalCaptureControl,
        ) -> Result<(), Self::Error> {
            // Проверяем флаг остановки
            if self.stop_flag.load(Ordering::Relaxed) {
                capture_control.stop();
                return Ok(());
            }

            // Ограничиваем FPS
            let now = std::time::Instant::now();
            if now.duration_since(self.last_frame) < self.interval {
                return Ok(());
            }
            self.last_frame = now;

            let src_w = frame.width();
            let src_h = frame.height();
            if src_w == 0 || src_h == 0 {
                return Ok(());
            }

            // Получаем BGRA пиксели (без padding)
            let buffer = frame.buffer()?;
            let bgra = buffer.as_nopadding_buffer(&mut self.nopadding_buf);

            let i420 = bgra_to_i420(bgra, src_w, src_h, self.out_w, self.out_h);

            let video_frame = VideoFrame::new(VideoRotation::VideoRotation0, i420);
            self.source.capture_frame(&video_frame);

            Ok(())
        }

        fn on_closed(&mut self) -> Result<(), Self::Error> {
            eprintln!("[screen] DXGI capture session closed");
            Ok(())
        }
    }

    // Запускаем capture на отдельном потоке (free_threaded — не блокирует)
    let flags = (source, out_w, out_h, stop_flag, fps);

    std::thread::Builder::new()
        .name("screen-capture".into())
        .spawn(move || {
            let settings = Settings::new(
                monitor,
                CursorCaptureSettings::WithCursor,
                DrawBorderSettings::WithoutBorder,
                SecondaryWindowSettings::Default,
                MinimumUpdateIntervalSettings::Default,
                DirtyRegionSettings::Default,
                ColorFormat::Bgra8,
                flags,
            );

            if let Err(e) = CaptureHandler::start(settings) {
                eprintln!("[screen] DXGI capture error: {e}");
            }
        })
        .expect("Failed to spawn capture thread");
}

// ─── Fallback: screenshots crate (Linux / macOS) ────────────────────────────

#[cfg(not(target_os = "windows"))]
fn start_capture_fallback(
    screen_id: u32,
    out_w: u32,
    out_h: u32,
    fps: u8,
    source: NativeVideoSource,
    stop_flag: Arc<AtomicBool>,
) {
    std::thread::Builder::new()
        .name("screen-capture".into())
        .spawn(move || {
            let screens = screenshots::Screen::all().unwrap_or_default();
            let Some(screen) = screens.into_iter().find(|s| s.display_info.id == screen_id) else { return };

            let interval = std::time::Duration::from_millis(1000 / fps as u64);

            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }

                let start = std::time::Instant::now();

                let rgba_image = match screen.capture() {
                    Ok(img) => img,
                    Err(_) => {
                        std::thread::sleep(interval);
                        continue;
                    }
                };

                let img_w = rgba_image.width();
                let img_h = rgba_image.height();
                let raw = rgba_image.into_raw();

                let i420 = rgba_to_i420(&raw, img_w, img_h, out_w, out_h);

                let frame = VideoFrame::new(VideoRotation::VideoRotation0, i420);
                source.capture_frame(&frame);

                let elapsed = start.elapsed();
                if elapsed < interval {
                    std::thread::sleep(interval - elapsed);
                }
            }
        })
        .expect("Failed to spawn capture thread");
}

// ─── Color conversion: BGRA/RGBA → I420 via libwebrtc/libyuv (SIMD) ────────

// Nearest-neighbor downscale on 4-bytes-per-pixel data.
// Only called when capture resolution differs from the target output resolution.
fn scale_pixels(src: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let sw = src_w as usize;
    let dw = dst_w as usize;
    let dh = dst_h as usize;
    let x_ratio = src_w as f32 / dst_w as f32;
    let y_ratio = src_h as f32 / dst_h as f32;
    let mut out = vec![0u8; dw * dh * 4];
    for row in 0..dh {
        let sy = (row as f32 * y_ratio) as usize;
        for col in 0..dw {
            let sx = (col as f32 * x_ratio) as usize;
            let src_i = (sy * sw + sx) * 4;
            let dst_i = (row * dw + col) * 4;
            out[dst_i..dst_i + 4].copy_from_slice(&src[src_i..src_i + 4]);
        }
    }
    out
}

/// BGRA → I420 (Windows/DXGI).
/// libyuv ARGBToI420 treats input as [B, G, R, A] in memory — matches DXGI BGRA layout.
#[cfg(target_os = "windows")]
fn bgra_to_i420(bgra: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> I420Buffer {
    use libwebrtc::native::yuv_helper;
    let cw = (dst_w + 1) / 2;
    let src = if src_w == dst_w && src_h == dst_h {
        std::borrow::Cow::Borrowed(bgra)
    } else {
        std::borrow::Cow::Owned(scale_pixels(bgra, src_w, src_h, dst_w, dst_h))
    };
    let mut buf = I420Buffer::new(dst_w, dst_h);
    let (yp, up, vp) = buf.data_mut();
    yuv_helper::argb_to_i420(&src, dst_w * 4, yp, dst_w, up, cw, vp, cw, dst_w as i32, dst_h as i32);
    buf
}

/// RGBA → I420 (non-Windows fallback).
/// libyuv ABGRToI420 treats input as [R, G, B, A] in memory — matches screenshots crate RGBA.
fn rgba_to_i420(rgba: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> I420Buffer {
    use libwebrtc::native::yuv_helper;
    let cw = (dst_w + 1) / 2;
    let src = if src_w == dst_w && src_h == dst_h {
        std::borrow::Cow::Borrowed(rgba)
    } else {
        std::borrow::Cow::Owned(scale_pixels(rgba, src_w, src_h, dst_w, dst_h))
    };
    let mut buf = I420Buffer::new(dst_w, dst_h);
    let (yp, up, vp) = buf.data_mut();
    yuv_helper::abgr_to_i420(&src, dst_w * 4, yp, dst_w, up, cw, vp, cw, dst_w as i32, dst_h as i32);
    buf
}

