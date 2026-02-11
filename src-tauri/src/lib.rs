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
use tauri::Manager;

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
            commands::get_amplitude,
        ])
        .setup(|app| {
            // Position window at bottom-center, just above the Dock
            if let Some(window) = app.get_webview_window("main") {
                if let Some(monitor) = window.current_monitor().ok().flatten() {
                    let screen = monitor.size();
                    let scale = monitor.scale_factor();
                    let win_w = 220.0;
                    let win_h = 48.0;
                    let x = (screen.width as f64 / scale - win_w) / 2.0;
                    // ~90px above bottom edge (Dock is typically ~70px)
                    let y = screen.height as f64 / scale - win_h - 90.0;
                    let _ = window.set_position(tauri::PhysicalPosition::new(
                        (x * scale) as i32,
                        (y * scale) as i32,
                    ));
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
