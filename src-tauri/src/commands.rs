use tauri::{AppHandle, Emitter, Manager, State};

use crate::audio::capture::AudioCapture;
use crate::config::AppConfig;
use crate::hotkey;
use crate::polishing::online::OnlinePolisher;
use crate::screenshot;
use crate::state::AppState;
use crate::vision::{self, VisionConfig};

#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_amplitude(state: State<'_, AppState>) -> f32 {
    state.audio_buffer.get_amplitude()
}

// ── Config commands ──

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state
        .config
        .lock()
        .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn save_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    // Persist to disk
    config.save()?;
    // Update in-memory config
    {
        let mut current = state
            .config
            .lock()
            .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        *current = config;
    }
    // Notify frontends that config changed (for hotkey re-registration)
    let _ = app.emit("config-changed", ());
    println!("[save_config] config saved and updated");
    Ok(())
}

// ── Correction dictionary commands ──

#[tauri::command]
pub fn get_corrections(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    let dict = state
        .correction_dict
        .lock()
        .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    Ok(dict.entries().to_vec())
}

#[tauri::command]
pub fn save_corrections(
    state: State<'_, AppState>,
    entries: Vec<(String, String)>,
) -> Result<(), String> {
    use crate::correction::dictionary::CorrectionDictionary;

    let new_dict = CorrectionDictionary::from_entries(entries);

    // Save to disk
    let dict_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("com.mac-voice-input")
        .join("corrections.json");
    new_dict
        .save(&dict_path)
        .map_err(|e| format!("Failed to save corrections: {}", e))?;

    // Update in-memory dictionary
    let mut dict = state
        .correction_dict
        .lock()
        .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    *dict = new_dict;

    println!(
        "[save_corrections] saved {} rules to {}",
        dict.entries().len(),
        dict_path.display()
    );
    Ok(())
}

// ── Recording commands ──

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
    let pipeline_start = std::time::Instant::now();
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

    // Read config for recordings dir, polish settings, API config, screenshot settings, and vision settings
    let (recordings_dir, polish_prompt, polish_enabled, polish_mode,
         api_endpoint, api_key, api_model, screenshot_mode, screenshot_max_size,
         vision_mode, vision_model_path, vision_max_image_size) = {
        let cfg = state.config.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        (cfg.recordings_dir.clone(), cfg.polish_prompt.clone(), cfg.polish_enabled,
         cfg.polish_mode.clone(), cfg.api_endpoint.clone(),
         cfg.api_key.clone(), cfg.api_model.clone(),
         cfg.screenshot_mode.clone(), cfg.screenshot_max_size,
         cfg.vision_mode.clone(), cfg.vision_model_path.clone(),
         cfg.vision_max_image_size)
    };

    // 3. Capture screenshot if enabled (before transcription for minimal delay)
    let screenshot_result: Option<Vec<u8>> = if screenshot_mode != "disabled" {
        match screenshot::capture_active_window() {
            Ok(png_bytes) => {
                // Resize if needed
                let resized = if screenshot_max_size > 0 {
                    match screenshot::resize_if_needed(png_bytes.clone(), screenshot_max_size) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            println!("[screenshot] resize failed: {}, using original", e);
                            png_bytes
                        }
                    }
                } else {
                    png_bytes
                };

                // Always save to disk
                if let Err(e) = screenshot::save_screenshot(&resized) {
                    println!("[screenshot] failed to save: {}", e);
                }

                println!("[screenshot] captured {} bytes", resized.len());
                Some(resized)
            }
            Err(e) => {
                println!("[screenshot] failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    // 4. Save raw audio to WAV before any processing (using configured path)
    {
        let rec_path = std::path::PathBuf::from(&recordings_dir);
        match crate::audio::wav_save::save_wav(&samples, sample_rate, &rec_path) {
            Ok(path) => println!("[stop_and_transcribe] saved WAV: {}", path.display()),
            Err(e) => println!("[stop_and_transcribe] WARNING: failed to save WAV: {}", e),
        }
    }

    // 5. Resample to 16kHz (SenseVoice expects 16kHz)
    let target_rate = 16000u32;
    let samples_16k = if sample_rate != target_rate {
        println!("Resampling from {}Hz to {}Hz ({} -> {} samples)",
            sample_rate, target_rate, samples.len(),
            (samples.len() as f64 * target_rate as f64 / sample_rate as f64) as usize);
        crate::audio::resample::resample(&samples, sample_rate, target_rate)
    } else {
        samples
    };

    // 6. Transcribe
    let (text, detected_lang) = {
        let mut engine = state
            .recognition_engine
            .lock()
            .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        println!("[transcribe] sending {} samples at {}Hz to SenseVoice", samples_16k.len(), target_rate);
        let t_start = std::time::Instant::now();
        let (text, lang) = engine
            .transcribe(target_rate, &samples_16k)
            .map_err(|e: anyhow::Error| e.to_string())?;
        println!("[transcribe] result: '{}' (lang={}) ({:.1}s)", text, lang, t_start.elapsed().as_secs_f64());
        (text, lang)
    };

    // 7. Apply corrections
    let corrected = {
        let dict = state
            .correction_dict
            .lock()
            .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        dict.apply(&text)
    };

    // 8. AI polishing / Vision processing — dispatch based on config
    let polished = if !polish_enabled && vision_mode != "local" {
        println!("[polish] disabled in config and vision not enabled, skipping");
        corrected.clone()
    } else {
        // Check if we should use vision model
        let use_vision = vision_mode == "local" && screenshot_result.is_some();

        if use_vision {
            // Try to use vision model
            println!("[vision] attempting to process with vision model");
            let vision_config = VisionConfig {
                model_path: vision_model_path.clone(),
                max_image_size: vision_max_image_size,
                num_threads: 4,
            };

            match vision::load_vision_engine(vision_config) {
                Ok(engine) => {
                    if engine.is_ready() {
                        let prompt = vision::build_vision_prompt(&corrected, &detected_lang);
                        let start = std::time::Instant::now();

                        // Preprocess image if needed
                        let processed_image = if vision_max_image_size > 0 {
                            match vision::preprocess_image(screenshot_result.as_ref().unwrap(), vision_max_image_size) {
                                Ok(img) => img,
                                Err(e) => {
                                    println!("[vision] preprocess failed: {}, using original", e);
                                    screenshot_result.as_ref().unwrap().clone()
                                }
                            }
                        } else {
                            screenshot_result.as_ref().unwrap().clone()
                        };

                        match engine.process(&processed_image, &prompt) {
                            Ok(result) => {
                                println!("[vision] result: '{}' ({:.1}s)", result, start.elapsed().as_secs_f64());
                                result
                            }
                            Err(e) => {
                                println!("[vision] ERROR: {}, falling back to text polish", e);
                                // Fall back to text-only polishing
                                polish_text_only(&corrected, &detected_lang, polish_enabled, &polish_mode,
                                    &polish_prompt, &api_endpoint, &api_key, &api_model, &state)
                            }
                        }
                    } else {
                        println!("[vision] engine not ready, falling back to text polish");
                        polish_text_only(&corrected, &detected_lang, polish_enabled, &polish_mode,
                            &polish_prompt, &api_endpoint, &api_key, &api_model, &state)
                    }
                }
                Err(e) => {
                    println!("[vision] failed to load engine: {}, falling back to text polish", e);
                    polish_text_only(&corrected, &detected_lang, polish_enabled, &polish_mode,
                        &polish_prompt, &api_endpoint, &api_key, &api_model, &state)
                }
            }
        } else {
            // Use text-only polishing
            polish_text_only(&corrected, &detected_lang, polish_enabled, &polish_mode,
                &polish_prompt, &api_endpoint, &api_key, &api_model, &state)
        }
    };

    // 9. Insert text into active app
    crate::insertion::insert_text(&polished).map_err(|e: anyhow::Error| e.to_string())?;

    println!("[pipeline] total: {:.1}s", pipeline_start.elapsed().as_secs_f64());

    // 8. Update last result and emit event
    {
        let mut last = state
            .last_result
            .lock()
            .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        *last = polished.clone();
    }
    let _ = app.emit("transcription-complete", &polished);

    Ok(polished)
}

// ── Native hotkey commands (CGEventTap fallback for unsupported keys) ──

#[tauri::command]
pub fn register_native_hotkey(
    app: AppHandle,
    state: State<'_, AppState>,
    key: String,
) -> Result<(), String> {
    println!("[native_hotkey] registering key: {}", key);

    // Stop any existing native hotkey listener first
    {
        let mut handle = state
            .native_hotkey
            .lock()
            .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        if handle.is_some() {
            println!("[native_hotkey] stopping previous listener");
            *handle = None; // Drop stops the CGEventTap
        }
    }

    // Parse the hotkey string into a HotkeyDefinition
    let hotkey_def = hotkey::parse_hotkey(&key)?;

    let app_clone = app.clone();
    let new_handle = hotkey::start_native_listener(&hotkey_def, move || {
        let _ = app_clone.emit("native-hotkey-pressed", ());
    })?;

    {
        let mut handle = state
            .native_hotkey
            .lock()
            .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        *handle = Some(new_handle);
    }

    println!("[native_hotkey] listener started for: {}", key);
    Ok(())
}

#[tauri::command]
pub fn unregister_native_hotkey(state: State<'_, AppState>) -> Result<(), String> {
    let mut handle = state
        .native_hotkey
        .lock()
        .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    if handle.is_some() {
        println!("[native_hotkey] stopping listener");
        *handle = None; // Drop stops the CGEventTap
    }
    Ok(())
}

/// Helper function for text-only polishing (used as fallback when vision fails)
fn polish_text_only(
    text: &str,
    detected_lang: &str,
    polish_enabled: bool,
    polish_mode: &str,
    polish_prompt: &str,
    api_endpoint: &str,
    api_key: &str,
    api_model: &str,
    state: &State<'_, AppState>,
) -> String {
    if !polish_enabled {
        return text.to_string();
    }

    match polish_mode {
        "local" => {
            // Local GGUF model polishing
            if let Some(ref engine_mutex) = state.polishing_engine {
                match engine_mutex.lock() {
                    Ok(engine) => {
                        println!("[polish:local] '{}' (lang={})", text, detected_lang);
                        let start = std::time::Instant::now();
                        let custom = if polish_prompt.is_empty() { None } else { Some(polish_prompt) };
                        let lang_ref = if detected_lang.is_empty() { None } else { Some(detected_lang) };
                        match engine.polish(text, custom, lang_ref) {
                            Ok(result) => {
                                println!("[polish:local] result: '{}' ({:.1}s)", result, start.elapsed().as_secs_f64());
                                result
                            }
                            Err(e) => {
                                println!("[polish:local] ERROR: {}, using original", e);
                                text.to_string()
                            }
                        }
                    }
                    Err(e) => {
                        println!("[polish:local] lock error: {}, using original", e);
                        text.to_string()
                    }
                }
            } else {
                println!("[polish:local] engine not loaded, skipping");
                text.to_string()
            }
        }
        "api" => {
            // Online API polishing (OpenAI-compatible)
            if api_endpoint.is_empty() || api_key.is_empty() || api_model.is_empty() {
                println!("[polish:api] API not configured (endpoint/key/model empty), skipping");
                text.to_string()
            } else {
                let lang = if detected_lang.is_empty() { "auto" } else { detected_lang };
                let prompt = if polish_prompt.is_empty() {
                    crate::config::DEFAULT_POLISH_PROMPT.to_string()
                } else {
                    polish_prompt.to_string()
                };
                println!("[polish:api] '{}' (lang={}) via {}", text, lang, api_model);
                let start = std::time::Instant::now();
                let polisher = OnlinePolisher::new(api_endpoint, api_key, api_model);
                match polisher.polish(text, &prompt, lang, None) {
                    Ok(result) => {
                        println!("[polish:api] result: '{}' ({:.1}s)", result, start.elapsed().as_secs_f64());
                        result
                    }
                    Err(e) => {
                        println!("[polish:api] ERROR: {}, using original", e);
                        text.to_string()
                    }
                }
            }
        }
        _ => {
            println!("[polish] unknown mode '{}', skipping", polish_mode);
            text.to_string()
        }
    }
}
