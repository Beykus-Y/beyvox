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
    // Some when using CaptureCore.dll — must join before calling dll.init() again
    join_handle: Option<std::thread::JoinHandle<()>>,
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

    // fps clamped to [1,60] above — nonlinear curve smooths bitrate between 30 and 60fps
    let effective_fps = if fps <= 30 { fps as f64 } else { 30.0 + (fps - 30) as f64 * 0.7 };
    let target_bitrate = ((0.165 * out_w as f64 * out_h as f64 * effective_fps) as u64)
        .clamp(500_000, 15_000_000);

    eprintln!(
        "[screen] starting share: {}x{} @{}fps (eff={:.1}) bitrate={}bps quality={}",
        out_w, out_h, fps, effective_fps, target_bitrate, quality
    );

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
                    max_bitrate: target_bitrate,
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

    // Останавливаем предыдущий стрим и ждём завершения DLL-потока до нового dll.init()
    let old = SCREEN_ENGINE.lock().unwrap().take();
    if let Some(old_engine) = old {
        old_engine.stop_flag.store(true, Ordering::Release);
        // Join DLL capture thread before re-entering dll.init() — avoids g_capture race
        if let Some(handle) = old_engine.join_handle {
            let _ = tokio::task::spawn_blocking(move || { let _ = handle.join(); }).await;
        }
        let _ = old_engine.room.local_participant()
            .unpublish_track(&old_engine.video_track.sid()).await;
    }

    let stop_flag = Arc::new(AtomicBool::new(false));

    // Запускаем захват на выделенном OS-потоке (не tokio — это CPU-bound работа)
    #[cfg(target_os = "windows")]
    let join_handle = start_capture_dxgi(screen_id, out_w, out_h, fps, source, stop_flag.clone());

    #[cfg(not(target_os = "windows"))]
    let join_handle: Option<std::thread::JoinHandle<()>> = {
        start_capture_fallback(screen_id, out_w, out_h, fps, source, stop_flag.clone());
        None
    };

    *SCREEN_ENGINE.lock().unwrap() = Some(ScreenEngine {
        video_track: track,
        stop_flag,
        room,
        join_handle,
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_screen_share() -> Result<(), String> {
    let engine = SCREEN_ENGINE.lock().unwrap().take();
    if let Some(e) = engine {
        e.stop_flag.store(true, Ordering::Release);
        if let Some(handle) = e.join_handle {
            let _ = tokio::task::spawn_blocking(move || { let _ = handle.join(); }).await;
        }
        let _ = e.room.local_participant().unpublish_track(&e.video_track.sid()).await;
    }
    Ok(())
}

// ─── Windows: DXGI Desktop Duplication capture via windows-capture ───────────

// ─── Windows: Dynamic DLL FFI Loader for CaptureCore.dll ─────────────────────

#[cfg(target_os = "windows")]
struct CaptureCoreDll {
    _lib: libloading::Library,
    detect_gpu: unsafe extern "C" fn(vendor_id: *mut u32) -> bool,
    init: unsafe extern "C" fn(monitor_index: std::os::raw::c_int, out_width: std::os::raw::c_int, out_height: std::os::raw::c_int) -> bool,
    acquire_frame: unsafe extern "C" fn(y_plane: *mut u8, u_plane: *mut u8, v_plane: *mut u8) -> bool,
    release: unsafe extern "C" fn(),
}

#[cfg(target_os = "windows")]
impl CaptureCoreDll {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let mut exe_path = std::env::current_exe()?;
        exe_path.pop();
        let dll_path = exe_path.join("CaptureCore.dll");
        
        let lib = if dll_path.exists() {
            unsafe { libloading::Library::new(dll_path)? }
        } else {
            // Fallback: LoadLibraryW bare name — searches exe dir first (Windows default)
            unsafe { libloading::Library::new("CaptureCore.dll")? }
        };

        unsafe {
            let detect_gpu = *lib.get(b"capture_core_detect_gpu")?;
            let init = *lib.get(b"capture_core_init")?;
            let acquire_frame = *lib.get(b"capture_core_acquire_frame")?;
            let release = *lib.get(b"capture_core_release")?;

            Ok(Self {
                _lib: lib,
                detect_gpu,
                init,
                acquire_frame,
                release,
            })
        }
    }
}

// ─── Windows: DXGI Desktop Duplication capture via C++ DLL with WGC fallback ─

#[cfg(target_os = "windows")]
fn start_capture_dxgi(
    screen_id: u32,
    out_w: u32,
    out_h: u32,
    fps: u8,
    source: NativeVideoSource,
    stop_flag: Arc<AtomicBool>,
) -> Option<std::thread::JoinHandle<()>> {
    // 1. Попытка запустить через нативный C++ пайплайн CaptureCore (DXGI Duplication + GPU conversion)
    match CaptureCoreDll::load() {
        Ok(dll) => {
            let mut vendor_id = 0u32;
            if unsafe { (dll.detect_gpu)(&mut vendor_id) } {
                eprintln!("[screen] Detected GPU Vendor ID: 0x{:X}", vendor_id);
            }

            if unsafe { (dll.init)(screen_id as i32, out_w as i32, out_h as i32) } {
                eprintln!("[screen] High-performance C++ DXGI capture initialized. Running GPU loop.");

                let source_clone = source.clone();
                let stop_flag_clone = stop_flag.clone();
                let handle = std::thread::Builder::new()
                    .name("screen-capture-dxgi-dll".into())
                    .spawn(move || {
                        let interval = std::time::Duration::from_millis(1000 / fps as u64);

                        loop {
                            if stop_flag_clone.load(Ordering::Acquire) {
                                break;
                            }

                            let start = std::time::Instant::now();

                            let mut buf = I420Buffer::new(out_w, out_h);
                            let (yp, up, vp) = buf.data_mut();

                            let success = unsafe {
                                (dll.acquire_frame)(yp.as_mut_ptr(), up.as_mut_ptr(), vp.as_mut_ptr())
                            };

                            if success {
                                let video_frame = VideoFrame::new(VideoRotation::VideoRotation0, buf);
                                source_clone.capture_frame(&video_frame);
                            } else {
                                std::thread::sleep(std::time::Duration::from_millis(2));
                                continue;
                            }

                            let elapsed = start.elapsed();
                            if elapsed < interval {
                                std::thread::sleep(interval - elapsed);
                            }
                        }

                        unsafe { (dll.release)() };
                        eprintln!("[screen] C++ DXGI capture session released.");
                    })
                    .expect("Failed to spawn DXGI C++ capture thread");

                return Some(handle);
            } else {
                eprintln!("[screen] C++ DXGI capture init failed. Falling back to WGC.");
            }
        }
        Err(e) => {
            eprintln!("[screen] Failed to load CaptureCore.dll ({e}). Falling back to WGC.");
        }
    }

    // 2. Фолбек на WGC (Windows Graphics Capture) при недоступности DXGI
    start_capture_wgc(screen_id, out_w, out_h, fps, source, stop_flag);
    None
}

#[cfg(target_os = "windows")]
fn start_capture_wgc(
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
            if self.stop_flag.load(Ordering::Relaxed) {
                capture_control.stop();
                return Ok(());
            }

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

            let buffer = frame.buffer()?;
            let bgra = buffer.as_nopadding_buffer(&mut self.nopadding_buf);

            let i420 = bgra_to_i420(bgra, src_w, src_h, self.out_w, self.out_h);

            let video_frame = VideoFrame::new(VideoRotation::VideoRotation0, i420);
            self.source.capture_frame(&video_frame);

            Ok(())
        }

        fn on_closed(&mut self) -> Result<(), Self::Error> {
            eprintln!("[screen] WGC capture session closed");
            Ok(())
        }
    }

    let flags = (source, out_w, out_h, stop_flag, fps);

    std::thread::Builder::new()
        .name("screen-capture-wgc".into())
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
                eprintln!("[screen] WGC capture error: {e}");
            }
        })
        .expect("Failed to spawn WGC capture thread");
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
#[cfg(not(target_os = "windows"))]
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

