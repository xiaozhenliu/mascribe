use tauri::{AppHandle, Emitter, State};

use crate::audio::capture::AudioCapture;
use crate::state::AppState;

#[tauri::command]
pub fn get_amplitude(state: State<'_, AppState>) -> f32 {
    state.audio_buffer.get_amplitude()
}

/// Wrapper to allow cpal::Stream in a static Mutex.
/// Safety: We only access this from Tauri command handlers which are serialized.
struct StreamHolder(Option<cpal::Stream>);
unsafe impl Send for StreamHolder {}

static ACTIVE_STREAM: std::sync::Mutex<StreamHolder> =
    std::sync::Mutex::new(StreamHolder(None));

#[tauri::command]
pub fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    println!("[start_recording] called");
    let stream = AudioCapture::start(&state.audio_buffer).map_err(|e| {
        println!("[start_recording] ERROR: {}", e);
        e.to_string()
    })?;
    ACTIVE_STREAM.lock().unwrap().0 = Some(stream);
    println!("[start_recording] stream started OK");
    Ok(())
}

#[tauri::command]
pub fn stop_recording_and_transcribe(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    println!("[stop_and_transcribe] called");
    // 1. Stop recording — drop the stream
    {
        let mut holder = ACTIVE_STREAM.lock().unwrap();
        let had_stream = holder.0.is_some();
        drop(holder.0.take());
        println!("[stop_and_transcribe] stream dropped (had_stream={})", had_stream);
    }
    // Small delay to ensure last audio callback completes
    std::thread::sleep(std::time::Duration::from_millis(50));
    let (samples, sample_rate) = AudioCapture::stop(&state.audio_buffer);

    // 2. Check minimum duration (300ms)
    if sample_rate == 0 {
        return Err("No audio recorded".to_string());
    }
    let duration_ms = (samples.len() as f64 / sample_rate as f64 * 1000.0) as u64;
    println!("Audio: {} samples, {}Hz, {}ms", samples.len(), sample_rate, duration_ms);
    if duration_ms < 300 {
        return Err("Recording too short".to_string());
    }

    // 3. Save raw audio to WAV before any processing
    {
        let recordings_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("com.mac-voice-input")
            .join("recordings");
        match crate::audio::wav_save::save_wav(&samples, sample_rate, &recordings_dir) {
            Ok(path) => println!("[stop_and_transcribe] saved WAV: {}", path.display()),
            Err(e) => println!("[stop_and_transcribe] WARNING: failed to save WAV: {}", e),
        }
    }

    // 4. Resample to 16kHz (SenseVoice expects 16kHz)
    let target_rate = 16000u32;
    let samples_16k = if sample_rate != target_rate {
        println!("Resampling from {}Hz to {}Hz ({} -> {} samples)",
            sample_rate, target_rate, samples.len(),
            (samples.len() as f64 * target_rate as f64 / sample_rate as f64) as usize);
        crate::audio::resample::resample(&samples, sample_rate, target_rate)
    } else {
        samples
    };

    // 5. Transcribe
    let text = {
        let mut engine = state
            .recognition_engine
            .lock()
            .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        println!("[transcribe] sending {} samples at {}Hz to SenseVoice", samples_16k.len(), target_rate);
        let (text, _lang) = engine
            .transcribe(target_rate, &samples_16k)
            .map_err(|e: anyhow::Error| e.to_string())?;
        println!("[transcribe] result: '{}' (lang={})", text, _lang);
        text
    };

    // 6. Apply corrections
    let corrected = state.correction_dict.apply(&text);

    // 7. Insert text into active app
    crate::insertion::clipboard::insert_text(&corrected).map_err(|e: anyhow::Error| e.to_string())?;

    // 8. Update last result and emit event
    {
        let mut last = state
            .last_result
            .lock()
            .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        *last = corrected.clone();
    }
    let _ = app.emit("transcription-complete", &corrected);

    Ok(corrected)
}
