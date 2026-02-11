mod audio;
mod commands;
mod config;
mod correction;
mod insertion;
mod recognition;
mod state;

use audio::capture::AudioBuffer;
use config::AppConfig;
use correction::dictionary::CorrectionDictionary;
use recognition::engine::RecognitionEngine;
use state::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::default();

    // Load SenseVoice model
    let engine = RecognitionEngine::new(
        &config.model_dir,
        &config.language,
        config.num_threads,
        config.use_itn,
    )
    .expect("Failed to load SenseVoice model");

    // Load correction dictionary
    let dict_path = dirs::home_dir()
        .unwrap()
        .join(".openclaw/workspace/clawd/memory/personal-corrections.json");
    let correction_dict = CorrectionDictionary::load(&dict_path).unwrap_or_else(|_| {
        println!("No correction dictionary found, using empty");
        CorrectionDictionary::empty()
    });

    let app_state = AppState {
        recognition_engine: Mutex::new(engine),
        audio_buffer: AudioBuffer::new(),
        correction_dict,
        last_result: Mutex::new(String::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording_and_transcribe,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
