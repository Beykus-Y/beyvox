use std::sync::{Arc, Mutex};

use livekit::{
    options::{TrackPublishOptions, VideoCodec},
    track::{LocalTrack, LocalVideoTrack, TrackSource},
    webrtc::{
        prelude::RtcVideoSource,
        video_frame::{I420Buffer, VideoFrame, VideoRotation},
        video_source::native::NativeVideoSource,
        prelude::VideoResolution,
    },
};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize, Clone)]
pub struct ScreenInfo {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

struct ScreenEngine {
    video_track: LocalVideoTrack,
    capture_task: JoinHandle<()>,
    room: Arc<livekit::Room>,
}

static SCREEN_ENGINE: Mutex<Option<ScreenEngine>> = Mutex::new(None);

// ─── Public Tauri commands ───────────────────────────────────────────────────

#[tauri::command]
pub fn list_screens() -> Vec<ScreenInfo> {
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
    let fps = fps.clamp(1, 60) as u64;

    let source = NativeVideoSource::new(
        VideoResolution { width: out_w, height: out_h },
        false,
    );

    let track = LocalVideoTrack::create_video_track(
        "screen",
        RtcVideoSource::Native(source.clone()),
    );

    room.local_participant()
        .publish_track(
            LocalTrack::Video(track.clone()),
            TrackPublishOptions {
                source: TrackSource::Screenshare,
                video_codec: VideoCodec::H264,
                simulcast: false,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| format!("publish screen track: {e}"))?;

    let capture_task = tokio::spawn(async move {
        let screens = screenshots::Screen::all().unwrap_or_default();
        let Some(screen) = screens.into_iter().find(|s| s.display_info.id == screen_id) else { return };

        let interval = std::time::Duration::from_millis(1000 / fps);
        loop {
            let start = std::time::Instant::now();

            let rgba_image = match screen.capture() {
                Ok(img) => img,
                Err(_) => {
                    tokio::time::sleep(interval).await;
                    continue;
                }
            };

            let img_w = rgba_image.width();
            let img_h = rgba_image.height();
            let raw = rgba_image.into_raw();

            let pixels = if img_w != out_w || img_h != out_h {
                scale_rgba_nearest(&raw, img_w, img_h, out_w, out_h)
            } else {
                raw
            };

            let i420 = rgba_to_i420(&pixels, out_w, out_h);
            let frame = VideoFrame::new(VideoRotation::VideoRotation0, i420);
            source.capture_frame(&frame);

            let elapsed = start.elapsed();
            if elapsed < interval {
                tokio::time::sleep(interval - elapsed).await;
            }
        }
    });

    // Останавливаем предыдущий стрим (без await под mutex)
    let old = SCREEN_ENGINE.lock().unwrap().take();
    if let Some(old_engine) = old {
        old_engine.capture_task.abort();
        let _ = old_engine.room.local_participant().unpublish_track(&old_engine.video_track.sid()).await;
    }

    *SCREEN_ENGINE.lock().unwrap() = Some(ScreenEngine {
        video_track: track,
        capture_task,
        room,
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_screen_share() -> Result<(), String> {
    let engine = SCREEN_ENGINE.lock().unwrap().take();
    if let Some(e) = engine {
        e.capture_task.abort();
        let _ = e.room.local_participant().unpublish_track(&e.video_track.sid()).await;
    }
    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn scale_rgba_nearest(src: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (dst_w * dst_h * 4) as usize];
    let x_ratio = src_w as f32 / dst_w as f32;
    let y_ratio = src_h as f32 / dst_h as f32;
    for y in 0..dst_h as usize {
        for x in 0..dst_w as usize {
            let sx = (x as f32 * x_ratio) as usize;
            let sy = (y as f32 * y_ratio) as usize;
            let si = (sy * src_w as usize + sx) * 4;
            let di = (y * dst_w as usize + x) * 4;
            out[di..di + 4].copy_from_slice(&src[si..si + 4]);
        }
    }
    out
}

fn rgba_to_i420(rgba: &[u8], width: u32, height: u32) -> I420Buffer {
    let w = width as usize;
    let h = height as usize;
    let mut buf = I420Buffer::new(width, height);
    let (y_plane, u_plane, v_plane) = buf.data_mut();
    let chroma_w = (w + 1) / 2;

    for row in 0..h {
        for col in 0..w {
            let i = (row * w + col) * 4;
            let r = rgba[i] as f32;
            let g = rgba[i + 1] as f32;
            let b = rgba[i + 2] as f32;

            y_plane[row * w + col] = (0.257 * r + 0.504 * g + 0.098 * b + 16.5) as u8;

            if row % 2 == 0 && col % 2 == 0 {
                let uv_i = (row / 2) * chroma_w + col / 2;
                u_plane[uv_i] = (-0.148 * r - 0.291 * g + 0.439 * b + 128.5) as u8;
                v_plane[uv_i] = (0.439 * r - 0.368 * g - 0.071 * b + 128.5) as u8;
            }
        }
    }

    buf
}

pub fn encode_jpeg(rgba: &[u8], width: u32, height: u32, quality: u8) -> Result<Vec<u8>, String> {
    use image::{ImageBuffer, Rgba, DynamicImage};
    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba.to_vec())
        .ok_or("bad image dimensions")?;
    let rgb_img = DynamicImage::ImageRgba8(img).to_rgb8();
    let mut out = Vec::new();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(
        std::io::Cursor::new(&mut out),
        quality,
    );
    enc.encode(rgb_img.as_raw(), width, height, image::ColorType::Rgb8)
        .map_err(|e| e.to_string())?;
    Ok(out)
}
