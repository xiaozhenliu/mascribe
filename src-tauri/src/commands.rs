use tauri::{AppHandle, Emitter, Manager, State};

use crate::audio::capture::AudioCapture;
use crate::config::AppConfig;
use crate::hotkey;
use crate::polishing::online::OnlinePolisher;
use crate::screenshot;
use crate::state::AppState;

#[tauri::command]
pub fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        // On macOS we MUST NOT call Tauri's window.show() because tao
        // implements it via makeKeyAndOrderFront: which activates the app
        // and can pull the user out of a full-screen Space.
        // Instead we use orderFrontRegardless via native ObjC API.
        #[cfg(target_os = "macos")]
        {
            // Reposition to the monitor under cursor (PhysicalPosition handles
            // the coordinate conversion correctly — unlike raw setFrameTopLeftPoint
            // which uses the AppKit bottom-left-origin point coordinate system).
            if let Ok(cursor) = window.cursor_position() {
                if let Ok(Some(monitor)) = window.monitor_from_point(cursor.x, cursor.y) {
                    let screen = monitor.size();
                    let scale = monitor.scale_factor();
                    let mon_pos = monitor.position();
                    let win_w = 260.0;
                    let win_h = 100.0;
                    let x = mon_pos.x as f64 + (screen.width as f64 - win_w * scale) / 2.0;
                    let y = mon_pos.y as f64 + screen.height as f64 - (win_h + 80.0) * scale;
                    let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
                    println!(
                        "[show_window] repositioned to ({}, {})",
                        x as i32, y as i32
                    );
                }
            }

            // Show via native API — orderFrontRegardless doesn't activate
            // the app, so the user stays in the full-screen Space.
            let closure_win = window.clone();
            let _ = window.run_on_main_thread(move || {
                show_overlay_on_main_thread(&closure_win);
            });
        }

        #[cfg(not(target_os = "macos"))]
        window.show().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Show the floating overlay via native NSWindow API (main-thread only).
///
/// This replaces Tauri's `window.show()` which calls `makeKeyAndOrderFront:`
/// — that activates the app and can switch the user away from a full-screen
/// Space.  We use `orderFrontRegardless` instead, which places the window on
/// the *current* Space (including full-screen) without activation.
#[cfg(target_os = "macos")]
fn show_overlay_on_main_thread(window: &tauri::WebviewWindow) {
    use objc2_app_kit::{
        NSScreenSaverWindowLevel, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
    };

    let ns_window_ptr = match window.ns_window() {
        Ok(ptr) => ptr as *mut NSWindow,
        Err(e) => {
            println!("[show_window] failed to get ns_window: {}", e);
            return;
        }
    };

    unsafe {
        // Ensure the window is an NSPanel (may already be from setup,
        // but re-assert in case anything reconstructed the window).
        let panel_cls = objc2::runtime::AnyClass::get(c"NSPanel");
        if let Some(cls) = panel_cls {
            objc2::ffi::object_setClass(
                ns_window_ptr as *mut objc2::ffi::objc_object,
                cls as *const objc2::runtime::AnyClass as *const objc2::ffi::objc_class,
            );
        }

        let ns = &*ns_window_ptr;

        // ── Style ──
        let mut style = ns.styleMask();
        style.insert(NSWindowStyleMask::NonactivatingPanel);
        style.insert(NSWindowStyleMask::UtilityWindow);
        ns.setStyleMask(style);

        // ── Collection behavior ──
        let mut behavior = NSWindowCollectionBehavior::empty();
        behavior.insert(NSWindowCollectionBehavior::CanJoinAllSpaces);
        behavior.insert(NSWindowCollectionBehavior::FullScreenAuxiliary);
        behavior.insert(NSWindowCollectionBehavior::Transient);
        behavior.insert(NSWindowCollectionBehavior::Stationary);
        ns.setCollectionBehavior(behavior);

        // ── Window level (1000 — above full-screen apps) ──
        ns.setLevel(NSScreenSaverWindowLevel);
        ns.setHidesOnDeactivate(false);

        // ── Show without activating ──
        ns.orderFrontRegardless();
    }
    println!("[show_window] overlay shown via orderFrontRegardless");
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

#[tauri::command]
pub fn get_platform() -> String {
    std::env::consts::OS.to_string()
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
        .join("com.mascribe")
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
    crate::insertion::remember_frontmost_target_app();
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

    // Read config for recordings dir, polish settings, API config, screenshot settings, and vision/OCR settings
    let (recordings_dir, polish_prompt, polish_enabled, polish_mode,
         api_endpoint, api_key, api_model, screenshot_mode, screenshot_max_size,
         vision_mode, ocr_endpoint, ocr_model) = {
        let cfg = state.config.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        (cfg.recordings_dir.clone(), cfg.polish_prompt.clone(), cfg.polish_enabled,
         cfg.polish_mode.clone(), cfg.api_endpoint.clone(),
         cfg.api_key.clone(), cfg.api_model.clone(),
         cfg.screenshot_mode.clone(), cfg.screenshot_max_size,
         cfg.vision_mode.clone(), cfg.ocr_endpoint.clone(),
         cfg.ocr_model.clone())
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

    // 8a. OCR: extract screen context from screenshot (if vision/OCR enabled)
    let screen_context: Option<String> = if vision_mode != "disabled" && screenshot_result.is_some() {
        let start = std::time::Instant::now();

        let ocr_result = if vision_mode == "native" {
            // Native macOS Vision framework OCR (fast, <0.5s)
            println!("[ocr:native] running macOS Vision OCR...");
            crate::ocr::recognize_text(screenshot_result.as_ref().unwrap())
                .map_err(|e| anyhow::anyhow!("{}", e))
        } else {
            // Ollama API OCR (slower, 5-7s, but supports custom models)
            let screenshot_base64 = screenshot::encode_base64(screenshot_result.as_ref().unwrap());
            println!("[ocr:api] running OCR via {} model={}", ocr_endpoint, ocr_model);
            ocr_screenshot(&ocr_endpoint, &ocr_model, &screenshot_base64)
        };

        match ocr_result {
            Ok(text) => {
                println!("[ocr:{}] extracted {} chars ({:.1}s): '{}'",
                    vision_mode, text.len(), start.elapsed().as_secs_f64(),
                    text.chars().take(100).collect::<String>());
                Some(text)
            }
            Err(e) => {
                println!("[ocr:{}] ERROR: {}, continuing without screen context", vision_mode, e);
                None
            }
        }
    } else {
        None
    };

    // 8b. AI polishing — inject screen context only for online API mode
    //     Local Qwen 2.5 1.5B can't handle OCR context (outputs the context verbatim).
    let effective_prompt = if let Some(ref ctx) = screen_context {
        if polish_mode == "api" {
            let truncated: String = ctx.chars().take(500).collect();
            let base = if polish_prompt.is_empty() {
                crate::config::DEFAULT_POLISH_PROMPT.to_string()
            } else {
                polish_prompt.clone()
            };
            // Inject OCR as a separate reference block AFTER {text},
            // with clear instructions not to include it in output
            let labeled_text = format!(
                "[TRANSCRIPT START]\n{}\n[TRANSCRIPT END]\n\n\
                 [SCREEN CONTEXT - reference only, do NOT output this]\n{}\n[END SCREEN CONTEXT]",
                "{text}", truncated
            );
            let combined = base.replace("{text}", &labeled_text);
            println!("[ocr] injected {} chars of screen context into polish prompt", truncated.len());
            combined
        } else {
            println!("[ocr] skipping context injection for polish_mode='{}' (local model too weak)", polish_mode);
            polish_prompt.clone()
        }
    } else {
        polish_prompt.clone()
    };

    let polished = polish_text_only(
        &corrected, &detected_lang, polish_enabled, &polish_mode,
        &effective_prompt, &api_endpoint, &api_key, &api_model, &state,
    );

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

/// Call OCR model (e.g., GLM-OCR via Ollama) to extract text from screenshot.
/// This is step 1 of the two-step pipeline: OCR → Polish.
fn ocr_screenshot(
    endpoint: &str,
    model: &str,
    screenshot_base64: &str,
) -> anyhow::Result<String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_read(std::time::Duration::from_secs(30))
        .timeout_write(std::time::Duration::from_secs(5))
        .build();

    let url = if endpoint.ends_with("/chat/completions") {
        endpoint.to_string()
    } else {
        format!("{}/chat/completions", endpoint.trim_end_matches('/'))
    };

    let response = agent
        .post(&url)
        .set("Content-Type", "application/json")
        .send_json(ureq::json!({
            "model": model,
            "messages": [{
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "提取这张截图中所有可见的文字内容。只输出提取到的文字，不要解释。"
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", screenshot_base64)
                        }
                    }
                ]
            }],
            "temperature": 0.1,
            "max_tokens": 2048
        }))
        .map_err(|e| anyhow::anyhow!("OCR request failed: {}", e))?;

    let body: serde_json::Value = response
        .into_json()
        .map_err(|e| anyhow::anyhow!("Failed to parse OCR response: {}", e))?;

    let text = body["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();

    if text.is_empty() {
        anyhow::bail!("OCR returned empty text");
    }

    Ok(text)
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

// ── Test API connection command ──

#[derive(serde::Serialize)]
pub struct TestConnectionResult {
    success: bool,
    response_time_ms: u64,
    error_message: Option<String>,
}

#[tauri::command]
pub fn test_online_api_connection(
    endpoint: String,
    api_key: String,
    model: String,
) -> Result<TestConnectionResult, String> {
    let start = std::time::Instant::now();
    let polisher = OnlinePolisher::new(&endpoint, &api_key, &model);

    match polisher.polish("test", "Reply: {text}", "en", None) {
        Ok(_) => {
            let elapsed = start.elapsed().as_millis() as u64;
            Ok(TestConnectionResult {
                success: true,
                response_time_ms: elapsed,
                error_message: None,
            })
        }
        Err(e) => {
            Ok(TestConnectionResult {
                success: false,
                response_time_ms: 0,
                error_message: Some(e.to_string()),
            })
        }
    }
}
