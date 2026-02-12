mod audio;
mod commands;
mod config;
mod correction;
pub mod hotkey;
mod insertion;
mod ocr;
mod permissions;
mod polishing;
mod recognition;
mod screenshot;
mod state;
mod tray;

use audio::capture::AudioBuffer;
use config::AppConfig;
use correction::dictionary::CorrectionDictionary;
use polishing::engine::PolishingEngine;
use recognition::engine::RecognitionEngine;
use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

/// Redirect stdout/stderr to a log file so we can debug the .app bundle.
#[cfg(target_os = "macos")]
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

/// Windows: Simple file logging setup (no dup2 available)
#[cfg(target_os = "windows")]
fn setup_file_logging() {
    use std::fs;

    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("VoiceInput")
        .join("logs");
    let _ = fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("VoiceInput.log");

    // On Windows, we can't easily redirect stdout/stderr like on Unix
    // For now, just print the log location
    println!("[VoiceInput] Logs would go to: {}", log_path.display());
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

    // Windows: Permissions are handled differently
    #[cfg(target_os = "windows")]
    {
        // Windows prompts for microphone on first use, no pre-authorization needed
        let _mic_granted = permissions::request_microphone_permission();
        // Windows doesn't require accessibility permission for SendInput
        let _ax_trusted = permissions::request_accessibility_permission();
    }

    let config = AppConfig::load();

    // Load SenseVoice model
    let engine = RecognitionEngine::new(
        &config.model_dir,
        &config.language,
        config.num_threads,
        config.use_itn,
    )
    .expect("Failed to load SenseVoice model");

    // Load correction dictionary from app data directory
    let dict_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("com.mac-voice-input")
        .join("corrections.json");
    let correction_dict = CorrectionDictionary::load(&dict_path).unwrap_or_else(|e| {
        println!("[correction] No dictionary found ({}), using empty", e);
        CorrectionDictionary::empty()
    });

    // Load polishing LLM (optional — if model not present, polishing is skipped)
    let polishing_engine = if config.polish_enabled {
        match PolishingEngine::new(&config.polish_model_path, 99) {
            Ok(engine) => {
                println!(
                    "[VoiceInput] Polishing LLM loaded from: {}",
                    config.polish_model_path
                );
                Some(Mutex::new(engine))
            }
            Err(e) => {
                println!(
                    "[VoiceInput] Polishing LLM not available ({}), polishing disabled",
                    e
                );
                None
            }
        }
    } else {
        println!("[VoiceInput] Polishing disabled in config");
        None
    };

    let app_state = AppState {
        recognition_engine: Mutex::new(engine),
        audio_buffer: AudioBuffer::new(),
        correction_dict: Mutex::new(correction_dict),
        last_result: Mutex::new(String::new()),
        polishing_engine,
        config: Mutex::new(config),
        native_hotkey: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording_and_transcribe,
            commands::get_amplitude,
            commands::show_window,
            commands::hide_window,
            commands::get_config,
            commands::save_config,
            commands::get_corrections,
            commands::save_corrections,
            commands::register_native_hotkey,
            commands::unregister_native_hotkey,
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

                // Disable native macOS window shadow — we use CSS box-shadow instead.
                // Without this, macOS draws its own shadow around the transparent window
                // creating an ugly, irregular "ghost frame" artifact.
                let _ = window.set_shadow(false);

                // Pre-calculate position: bottom-center, ~80px above screen bottom
                // Use primary_monitor() instead of current_monitor() because
                // the window is hidden and may not be associated with any monitor yet.
                let monitor = window.primary_monitor().ok().flatten()
                    .or_else(|| window.current_monitor().ok().flatten())
                    .or_else(|| window.available_monitors().ok()
                        .and_then(|m| m.into_iter().next()));
                if let Some(monitor) = monitor {
                    let screen = monitor.size();
                    let scale = monitor.scale_factor();
                    let mon_pos = monitor.position();
                    let win_w = 260.0;
                    let win_h = 100.0;
                    // Center horizontally on the monitor, 80px above bottom
                    let x = mon_pos.x as f64 + (screen.width as f64 - win_w * scale) / 2.0;
                    let y = mon_pos.y as f64 + screen.height as f64 - (win_h + 80.0) * scale;
                    let _ = window.set_position(tauri::PhysicalPosition::new(
                        x as i32,
                        y as i32,
                    ));
                    println!("[window] positioned at ({}, {}), monitor {}x{} @ ({},{}), scale={}",
                        x as i32, y as i32,
                        screen.width, screen.height,
                        mon_pos.x, mon_pos.y, scale);
                } else {
                    println!("[window] WARNING: no monitor detected, using default position");
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
