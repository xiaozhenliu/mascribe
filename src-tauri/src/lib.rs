mod audio;
mod commands;
mod config;
mod correction;
mod insertion;
mod permissions;
mod recognition;
mod state;
mod tray;

use audio::capture::AudioBuffer;
use config::AppConfig;
use correction::dictionary::CorrectionDictionary;
use recognition::engine::RecognitionEngine;
use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

/// Redirect stdout/stderr to a log file so we can debug the .app bundle.
fn setup_file_logging() {
    use std::fs;
    use std::os::unix::io::AsRawFd;

    let log_dir = dirs::home_dir().unwrap().join("Library/Logs");
    let _ = fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("VoiceInput.log");

    // Truncate and open for writing
    if let Ok(file) = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
    {
        let fd = file.as_raw_fd();
        unsafe {
            libc::dup2(fd, 1); // stdout
            libc::dup2(fd, 2); // stderr
        }
        // Keep file open by leaking it (lives for process lifetime)
        std::mem::forget(file);
        println!("[VoiceInput] Logging to {}", log_path.display());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_file_logging();

    // ── Request permissions at startup ──
    // Both must be prompted BEFORE the app starts using audio or simulating keys.
    #[cfg(target_os = "macos")]
    {
        // 1. Microphone — cpal uses CoreAudio directly which does NOT trigger
        //    the TCC dialog. We call AVCaptureDevice.requestAccess instead.
        let mic_granted = permissions::request_microphone_permission();
        if !mic_granted {
            println!("[VoiceInput] WARNING: Microphone permission not granted! Audio will be silent.");
        }

        // 2. Accessibility — needed for CGEvent.post() to simulate Cmd+V.
        //    AXIsProcessTrustedWithOptions(prompt:true) shows a system dialog
        //    directing the user to System Settings → Accessibility.
        let ax_trusted = permissions::request_accessibility_permission();
        if !ax_trusted {
            println!("[VoiceInput] WARNING: Accessibility not granted! Cmd+V paste will not work.");
        }
    }

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
            commands::show_window,
            commands::hide_window,
        ])
        .setup(|app| {
            // Hide from Dock — app lives only in menu bar
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Set up tray icon in menu bar
            tray::setup_tray(app)?;

            // Start with window hidden — it shows when recording starts
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();

                // Pre-calculate position for when we show it later
                if let Some(monitor) = window.current_monitor().ok().flatten() {
                    let screen = monitor.size();
                    let scale = monitor.scale_factor();
                    let win_w = 220.0;
                    let win_h = 48.0;
                    let x = (screen.width as f64 / scale - win_w) / 2.0;
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
