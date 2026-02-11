use tauri::{AppHandle, Emitter, State};

use crate::audio::capture::AudioCapture;
use crate::state::AppState;

/// Wrapper to allow cpal::Stream in a static Mutex.
/// Safety: We only access this from Tauri command handlers which are serialized.
struct StreamHolder(Option<cpal::Stream>);
unsafe impl Send for StreamHolder {}

static ACTIVE_STREAM: std::sync::Mutex<StreamHolder> =
    std::sync::Mutex::new(StreamHolder(None));

#[tauri::command]
pub fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    let stream = AudioCapture::start(&state.audio_buffer).map_err(|e| e.to_string())?;
    ACTIVE_STREAM.lock().unwrap().0 = Some(stream);
    Ok(())
}

#[tauri::command]
pub fn stop_recording_and_transcribe(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 1. Stop recording — drop the stream
    {
        let mut holder = ACTIVE_STREAM.lock().unwrap();
        drop(holder.0.take());
    }
    let (samples, sample_rate) = AudioCapture::stop(&state.audio_buffer);

    // 2. Check minimum duration (300ms)
    if sample_rate == 0 {
        return Err("No audio recorded".to_string());
    }
    let duration_ms = (samples.len() as f64 / sample_rate as f64 * 1000.0) as u64;
    if duration_ms < 300 {
        return Err("Recording too short".to_string());
    }

    // 3. Transcribe
    let text = {
        let mut engine = state
            .recognition_engine
            .lock()
            .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        let (text, _lang) = engine
            .transcribe(sample_rate, &samples)
            .map_err(|e: anyhow::Error| e.to_string())?;
        text
    };

    // 4. Apply corrections
    let corrected = state.correction_dict.apply(&text);

    // 5. Insert text into active app
    crate::insertion::clipboard::insert_text(&corrected).map_err(|e: anyhow::Error| e.to_string())?;

    // 6. Update last result and emit event
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
