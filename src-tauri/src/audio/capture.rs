use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

/// Shared audio buffer that can be placed in Tauri State (Send + Sync).
/// The cpal::Stream is NOT stored here — it lives on the thread that creates it.
pub struct AudioBuffer {
    pub data: Arc<Mutex<Vec<f32>>>,
    pub sample_rate: Mutex<u32>,
    pub is_recording: Mutex<bool>,
    /// Current RMS amplitude as f32 bits stored in AtomicU32 (lock-free for audio callback).
    pub amplitude: Arc<AtomicU32>,
}

impl AudioBuffer {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Mutex::new(0),
            is_recording: Mutex::new(false),
            amplitude: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Get current amplitude as f32 (0.0 to 1.0).
    pub fn get_amplitude(&self) -> f32 {
        f32::from_bits(self.amplitude.load(Ordering::Relaxed))
    }
}

/// Non-Send audio capture — lives on a dedicated thread.
/// We only expose `start` and `stop` via message passing through AudioBuffer.
pub struct AudioCapture;

impl AudioCapture {
    pub fn start(buffer: &AudioBuffer) -> anyhow::Result<cpal::Stream> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device found"))?;

        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        *buffer.sample_rate.lock().unwrap() = sample_rate;

        let data = buffer.data.clone();
        data.lock().unwrap().clear();

        let amplitude = buffer.amplitude.clone();
        amplitude.store(0f32.to_bits(), Ordering::Relaxed);

        let channels = config.channels() as usize;

        let stream = device.build_input_stream(
            &StreamConfig {
                channels: config.channels(),
                sample_rate: config.sample_rate(),
                buffer_size: cpal::BufferSize::Default,
            },
            move |samples: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buf = data.lock().unwrap();
                let mut sum_sq = 0.0f32;
                let mut count = 0usize;
                if channels > 1 {
                    for chunk in samples.chunks(channels) {
                        let s = chunk[0];
                        buf.push(s);
                        sum_sq += s * s;
                        count += 1;
                    }
                } else {
                    buf.extend_from_slice(samples);
                    for &s in samples {
                        sum_sq += s * s;
                    }
                    count = samples.len();
                }
                if count > 0 {
                    let rms = (sum_sq / count as f32).sqrt();
                    amplitude.store(rms.to_bits(), Ordering::Relaxed);
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        stream.play()?;
        *buffer.is_recording.lock().unwrap() = true;
        Ok(stream)
    }

    pub fn stop(buffer: &AudioBuffer) -> (Vec<f32>, u32) {
        *buffer.is_recording.lock().unwrap() = false;
        let samples = {
            let buf = buffer.data.lock().unwrap();
            buf.clone()
        };
        let sr = *buffer.sample_rate.lock().unwrap();
        (samples, sr)
    }
}
