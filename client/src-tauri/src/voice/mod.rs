use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use futures_util::StreamExt;
use livekit::{
    options::TrackPublishOptions,
    track::{LocalAudioTrack, LocalTrack, TrackSource},
    webrtc::{
        audio_frame::AudioFrame,
        audio_source::native::NativeAudioSource,
        audio_stream::native::NativeAudioStream,
        prelude::{AudioSourceOptions, RtcAudioSource},
        video_frame::VideoFormatType,
        video_stream::native::NativeVideoStream,
    },
    Room, RoomEvent, RoomOptions,
};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

// cpal::Stream не Send — оборачиваем (Mutex гарантирует монопольный доступ)
struct SendStream(#[allow(dead_code)] cpal::Stream);
unsafe impl Send for SendStream {}
unsafe impl Sync for SendStream {}

// Микшер: participant_identity → очередь сэмплов f32
type Mixer = Arc<Mutex<HashMap<String, VecDeque<f32>>>>;

struct VoiceEngine {
    room: Arc<Room>,
    local_track: LocalAudioTrack,
    _capture: SendStream,
    _playback: Option<SendStream>,
    events_task: JoinHandle<()>,
}

static ENGINE: Mutex<Option<VoiceEngine>> = Mutex::new(None);
static DEAFENED: AtomicBool = AtomicBool::new(false);
// Громкость хранится как f32 побитово в AtomicU32 (нет AtomicF32 в std)
static MIC_GAIN: AtomicU32 = AtomicU32::new(1065353216); // 1.0f32.to_bits()
static PLAYBACK_GAIN: AtomicU32 = AtomicU32::new(1065353216);

// Mic test streams (отдельные от голосовых)
static TEST_CAPTURE: Mutex<Option<SendStream>> = Mutex::new(None);
static TEST_PLAYBACK: Mutex<Option<SendStream>> = Mutex::new(None);

// ─── Room accessor (для screen share модуля) ────────────────────────────────

pub fn get_room() -> Option<Arc<Room>> {
    ENGINE.lock().unwrap().as_ref().map(|e| e.room.clone())
}

// ─── Public Tauri commands ───────────────────────────────────────────────────

#[tauri::command]
pub async fn join_voice_channel(
    app: AppHandle,
    url: String,
    token: String,
    input_device: Option<String>,
    output_device: Option<String>,
) -> Result<(), String> {
    // Отключаем предыдущее соединение
    {
        let old = ENGINE.lock().unwrap().take();
        if let Some(e) = old {
            e.events_task.abort();
            let _ = e.room.close().await;
        }
    }
    DEAFENED.store(false, Ordering::Relaxed);

    let host = cpal::default_host();

    // Захват микрофона — нативный sample rate устройства
    let in_dev = find_device(&host, input_device, true)?;
    let (mic_tx, mic_rx) = mpsc::channel::<Vec<i16>>(8);
    let (capture, capture_rate) = start_capture(&in_dev, mic_tx)?;
    let samples_per_10ms = (capture_rate / 100) as usize;

    // LiveKit аудио источник — скармливаем данные в нативном rate, LiveKit сам ресемплирует
    // WebRTC APM выключен — обработкой занимаются VST-плагины
    let lk_source = NativeAudioSource::new(
        AudioSourceOptions {
            echo_cancellation: false,
            noise_suppression: false,
            auto_gain_control: false,
        },
        capture_rate,
        1,  // mono
        20, // 20ms внутренний буфер — минимальная задержка
    );

    let local_track = LocalAudioTrack::create_audio_track(
        "microphone",
        RtcAudioSource::Native(lk_source.clone()),
    );

    // Подключаемся к комнате
    let mut room_opts = RoomOptions::default();
    room_opts.auto_subscribe = true;
    let (room, initial_events) = Room::connect(&url, &token, room_opts)
        .await
        .map_err(|e| format!("LiveKit connect: {e}"))?;
    let room = Arc::new(room);

    // Публикуем трек микрофона
    room.local_participant()
        .publish_track(
            LocalTrack::Audio(local_track.clone()),
            TrackPublishOptions { source: TrackSource::Microphone, ..Default::default() },
        )
        .await
        .map_err(|e| format!("publish track: {e}"))?;

    // Задача: mic-frames → 10ms chunks → NativeAudioSource
    {
        let src = lk_source.clone();
        tokio::spawn(async move {
            let mut rx = mic_rx;
            let mut buf = Vec::<i16>::new();
            let max_buf = capture_rate as usize / 10; // 100ms cap
            while let Some(chunk) = rx.recv().await {
                buf.extend_from_slice(&chunk);
                if buf.len() > max_buf {
                    buf.drain(..buf.len() - max_buf);
                }
                while buf.len() >= samples_per_10ms {
                    let data: Vec<i16> = buf.drain(..samples_per_10ms).collect();
                    let frame = AudioFrame {
                        data: data.into(),
                        sample_rate: capture_rate,
                        num_channels: 1,
                        samples_per_channel: samples_per_10ms as u32,
                    };
                    let _ = src.capture_frame(&frame).await;
                }
            }
        });
    }

    // Воспроизведение — нативный sample rate устройства вывода
    let mixer: Mixer = Arc::new(Mutex::new(HashMap::new()));
    let out_result = find_device(&host, output_device, false)
        .and_then(|dev| start_playback(&dev, mixer.clone()));
    let (playback, playback_rate) = match out_result {
        Ok((s, r)) => (Some(s), r),
        Err(e) => {
            eprintln!("[voice] playback init failed: {e}");
            (None, 48000u32)
        }
    };

    // Задача: события комнаты (TrackSubscribed, speakers, disconnect)
    // Передаём initial_events из Room::connect — он содержит TrackSubscribed для участников,
    // уже находившихся в комнате до нашего подключения
    let events_task = tokio::spawn(handle_room_events(room.clone(), app, mixer, playback_rate, initial_events));

    *ENGINE.lock().unwrap() = Some(VoiceEngine {
        room,
        local_track,
        _capture: capture,
        _playback: playback,
        events_task,
    });

    Ok(())
}

#[tauri::command]
pub async fn leave_voice_channel() {
    let old = ENGINE.lock().unwrap().take();
    if let Some(e) = old {
        e.events_task.abort();
        let _ = e.room.close().await;
    }
}

#[tauri::command]
pub async fn set_muted(muted: bool) {
    if let Some(e) = ENGINE.lock().unwrap().as_ref() {
        if muted {
            e.local_track.mute();
        } else {
            e.local_track.unmute();
        }
    }
}

#[tauri::command]
pub async fn set_deafened(deafened: bool) {
    DEAFENED.store(deafened, Ordering::Relaxed);
}

#[tauri::command]
pub async fn start_mic_test(input_device: Option<String>) -> Result<(), String> {
    stop_mic_test();

    let host = cpal::default_host();
    let in_dev = find_device(&host, input_device, true)?;
    let out_dev =
        find_device(&host, None, false).unwrap_or_else(|_| host.default_output_device().unwrap());

    let mixer: Mixer = Arc::new(Mutex::new(HashMap::new()));
    let (tx, mut rx) = mpsc::channel::<Vec<i16>>(8);

    let (capture, capture_rate) = start_capture(&in_dev, tx)?;
    let (playback, _) = start_playback(&out_dev, mixer.clone())?;

    // Роутим захват прямо в микшер (loopback), cap 80ms
    tokio::spawn(async move {
        let max_test_buf = capture_rate as usize * 80 / 1000;
        while let Some(chunk) = rx.recv().await {
            let samples: Vec<f32> =
                chunk.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
            let mut guard = mixer.lock().unwrap();
            let buf = guard.entry("loopback".to_string()).or_default();
            buf.extend(samples);
            if buf.len() > max_test_buf {
                let excess = buf.len() - max_test_buf;
                buf.drain(..excess);
            }
        }
    });

    *TEST_CAPTURE.lock().unwrap() = Some(capture);
    *TEST_PLAYBACK.lock().unwrap() = Some(playback);
    Ok(())
}

#[tauri::command]
pub fn stop_mic_test() {
    *TEST_CAPTURE.lock().unwrap() = None;
    *TEST_PLAYBACK.lock().unwrap() = None;
}

#[tauri::command]
pub fn set_participant_volume(_user_id: String, _volume: f32) {
    // Per-participant volume управляется на стороне JS (participantVolumes в voice store)
    // Эта команда — заглушка для совместимости
}

#[tauri::command]
pub fn set_mic_volume(percent: u32) {
    let gain = (percent as f32 / 100.0).clamp(0.0, 2.0);
    MIC_GAIN.store(gain.to_bits(), Ordering::Relaxed);
}

#[tauri::command]
pub fn set_playback_volume(percent: u32) {
    let gain = (percent as f32 / 100.0).clamp(0.0, 2.0);
    PLAYBACK_GAIN.store(gain.to_bits(), Ordering::Relaxed);
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn find_device(
    host: &cpal::Host,
    name: Option<String>,
    is_input: bool,
) -> Result<cpal::Device, String> {
    if let Some(n) = name.filter(|s| !s.is_empty()) {
        let found = if is_input {
            host.input_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.name().ok().as_deref() == Some(&n))
        } else {
            host.output_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.name().ok().as_deref() == Some(&n))
        };
        return found.ok_or_else(|| format!("Device '{n}' not found"));
    }
    if is_input {
        host.default_input_device().ok_or_else(|| "No default input device".into())
    } else {
        host.default_output_device().ok_or_else(|| "No default output device".into())
    }
}

// Возвращает (поток, фактический sample_rate устройства)
fn start_capture(
    device: &cpal::Device,
    tx: mpsc::Sender<Vec<i16>>,
) -> Result<(SendStream, u32), String> {
    let supported = device.default_input_config().map_err(|e| e.to_string())?;
    let channels = supported.channels() as usize;
    let sample_rate = supported.sample_rate().0;
    let fmt = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();

    // Обновляем sample rate в цепочке эффектов до старта потока
    if let Ok(mut c) = crate::effects::get_chain().lock() {
        c.set_sample_rate(sample_rate as f32);
    }
    let chain = crate::effects::get_chain();

    // Общий хелпер: собирает моно f32, применяет эффекты, конвертирует в i16
    // (вызывается из замыкания потока — chain и tx захватываются по move)
    macro_rules! send_mono {
        ($chain:expr, $tx:expr, $mono:expr) => {{
            let mut mono = $mono;
            if let Ok(mut c) = $chain.try_lock() {
                c.process(&mut mono);
            }
            let samples: Vec<i16> = mono
                .iter()
                .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                .collect();
            let _ = $tx.try_send(samples);
        }};
    }

    let stream = match fmt {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                let gain = f32::from_bits(MIC_GAIN.load(Ordering::Relaxed));
                let mono: Vec<f32> = data
                    .chunks(channels)
                    .map(|f| f.iter().sum::<f32>() / channels as f32 * gain)
                    .collect();
                send_mono!(chain, tx, mono);
            },
            |e| eprintln!("[voice] capture error: {e}"),
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _| {
                let gain = f32::from_bits(MIC_GAIN.load(Ordering::Relaxed));
                let mono: Vec<f32> = data
                    .chunks(channels)
                    .map(|f| {
                        let sum: i32 = f.iter().map(|&s| s as i32).sum();
                        (sum / channels as i32) as f32 / i16::MAX as f32 * gain
                    })
                    .collect();
                send_mono!(chain, tx, mono);
            },
            |e| eprintln!("[voice] capture error: {e}"),
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _| {
                let gain = f32::from_bits(MIC_GAIN.load(Ordering::Relaxed));
                let mono: Vec<f32> = data
                    .chunks(channels)
                    .map(|f| {
                        let avg = f.iter().map(|&s| s as f32).sum::<f32>() / channels as f32;
                        (avg / u16::MAX as f32 * 2.0 - 1.0) * gain
                    })
                    .collect();
                send_mono!(chain, tx, mono);
            },
            |e| eprintln!("[voice] capture error: {e}"),
            None,
        ),
        _ => return Err(format!("Unsupported format: {fmt:?}")),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    Ok((SendStream(stream), sample_rate))
}

// Возвращает (поток, фактический sample_rate устройства)
fn start_playback(
    device: &cpal::Device,
    mixer: Mixer,
) -> Result<(SendStream, u32), String> {
    let supported = device.default_output_config().map_err(|e| e.to_string())?;
    let channels = supported.channels() as usize;
    let sample_rate = supported.sample_rate().0;
    let config: cpal::StreamConfig = supported.into();

    let stream = device
        .build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                if DEAFENED.load(Ordering::Relaxed) {
                    output.fill(0.0);
                    return;
                }
                let mut bufs = mixer.lock().unwrap();
                for frame in output.chunks_mut(channels) {
                    let gain = f32::from_bits(PLAYBACK_GAIN.load(Ordering::Relaxed));
                    let sample: f32 = (bufs
                        .values_mut()
                        .filter_map(|q| q.pop_front())
                        .sum::<f32>() * gain)
                        .clamp(-1.0, 1.0);
                    frame.fill(sample); // mono → все каналы (stereo/etc)
                }
            },
            |e| eprintln!("[voice] playback error: {e}"),
            None,
        )
        .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    Ok((SendStream(stream), sample_rate))
}

async fn handle_room_events(
    _room: Arc<Room>,
    app: AppHandle,
    mixer: Mixer,
    playback_rate: u32,
    mut events: tokio::sync::mpsc::UnboundedReceiver<RoomEvent>,
) {
    while let Some(event) = events.recv().await {
        match event {
            RoomEvent::TrackSubscribed { track, participant, .. } => {
                match track {
                    livekit::track::RemoteTrack::Audio(audio_track) => {
                        let pid = participant.identity().to_string();
                        let mixer = mixer.clone();
                        tokio::spawn(async move {
                            let mut stream = NativeAudioStream::new(
                                audio_track.rtc_track(),
                                playback_rate as i32,
                                1,
                            );
                            let max_remote_buf = playback_rate as usize * 80 / 1000;
                            while let Some(frame) = stream.next().await {
                                let samples: Vec<f32> = frame
                                    .data
                                    .iter()
                                    .map(|&s| s as f32 / i16::MAX as f32)
                                    .collect();
                                let mut guard = mixer.lock().unwrap();
                                let buf = guard.entry(pid.clone()).or_default();
                                buf.extend(samples);
                                if buf.len() > max_remote_buf {
                                    let excess = buf.len() - max_remote_buf;
                                    buf.drain(..excess);
                                }
                            }
                            mixer.lock().unwrap().remove(&pid);
                        });
                    }
                    livekit::track::RemoteTrack::Video(video_track) => {
                        let pid = participant.identity().to_string();
                        let app = app.clone();
                        tokio::spawn(async move {
                            let mut stream = NativeVideoStream::new(video_track.rtc_track());
                            let frame_interval = std::time::Duration::from_millis(33); // ~30 fps viewer
                            let mut last_emit = std::time::Instant::now()
                                .checked_sub(frame_interval)
                                .unwrap_or(std::time::Instant::now());
                            while let Some(frame) = stream.next().await {
                                let now = std::time::Instant::now();
                                if now.duration_since(last_emit) < frame_interval {
                                    continue;
                                }
                                last_emit = now;

                                let w = frame.buffer.width();
                                let h = frame.buffer.height();
                                if w == 0 || h == 0 {
                                    continue;
                                }
                                let mut rgba = vec![0u8; (w * h * 4) as usize];
                                frame.buffer.to_argb(
                                    VideoFormatType::RGBA,
                                    &mut rgba,
                                    w * 4,
                                    w as i32,
                                    h as i32,
                                );

                                // Payload: [u32 width LE][u32 height LE][RGBA bytes...]
                                // Зритель декодирует putImageData — no JPEG lossy compression
                                let mut payload = Vec::with_capacity(8 + rgba.len());
                                payload.extend_from_slice(&w.to_le_bytes());
                                payload.extend_from_slice(&h.to_le_bytes());
                                payload.extend_from_slice(&rgba);
                                let event_name = format!("screen://frame/{pid}");
                                let _ = app.emit(&event_name, payload);
                            }
                        });
                    }
                }
            }

            RoomEvent::ActiveSpeakersChanged { speakers } => {
                let ids: Vec<String> =
                    speakers.iter().map(|p| p.identity().to_string()).collect();
                let _ = app.emit("voice://active-speakers", ids);
            }

            RoomEvent::Disconnected { .. } => {
                let _ = app.emit("voice://disconnected", ());
                break;
            }

            _ => {}
        }
    }
}
