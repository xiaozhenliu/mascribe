# Rust Backend Guidelines

> Conventions for the Mascribe Tauri backend: Rust 2021 edition, Tauri 2.x.

---

## Module Structure

The backend is organized by **functional domain** in `src-tauri/src/`. Each module handles a distinct capability.

| Module | Purpose |
|--------|---------|
| `commands.rs` | All `#[tauri::command]` functions — the IPC boundary |
| `lib.rs` | Module declarations, app initialization, plugin registration |
| `main.rs` | Minimal entry point (just calls `run()`) |
| `config.rs` | `AppConfig` struct with serde load/save |
| `state.rs` | `AppState` — shared state managed by Tauri |
| `tray.rs` | System tray icon and menu |
| `permissions.rs` | macOS/Windows permission requests |
| `audio/` | Audio capture (cpal), resampling, WAV saving |
| `hotkey/` | Native hotkey listener (CGEventTap / SetWindowsHookEx) |
| `insertion/` | Text insertion into active app (CGEvent Cmd+V / SendInput) |
| `recognition/` | SenseVoice speech recognition engine |
| `polishing/` | AI text polishing (local GGUF + online API) |
| `correction/` | Post-transcription correction dictionary |
| `screenshot/` | Screen capture and image resizing |
| `ocr/` | Native OCR (macOS Vision framework) |
| `vision/` | Vision model integration |

---

## Error Handling

Two levels, strictly separated:

### Business logic → `anyhow::Result`

```rust
fn ocr_screenshot(endpoint: &str, model: &str, base64: &str) -> anyhow::Result<String> {
    // ...
    anyhow::bail!("OCR returned empty text");
}
```

### Tauri commands → `Result<T, String>`

Tauri commands must return `Result<T, String>` because errors cross the IPC boundary as strings.

```rust
#[tauri::command]
pub fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    let stream = AudioCapture::start(&state.audio_buffer).map_err(|e| e.to_string())?;
    Ok(())
}
```

**Pattern:** Use `.map_err(|e| e.to_string())` at the command boundary. Inside the module, use `anyhow::Result` freely.

### Mutex poisoning

Mutex locks use `.map_err(|e: std::sync::PoisonError<_>| e.to_string())` — never `.unwrap()` on locks in commands.

```rust
let config = state.config.lock()
    .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
```

---

## Tauri Command Conventions

### Registration

1. Define command in `commands.rs` with `#[tauri::command]`
2. Register in `lib.rs` inside `tauri::generate_handler![]`
3. Frontend calls with `invoke("command_name", { args })`

### Signature patterns

```rust
// Simple read
#[tauri::command]
pub fn get_amplitude(state: State<'_, AppState>) -> f32 { ... }

// Read with error
#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> { ... }

// Write with app handle (for emitting events)
#[tauri::command]
pub fn save_config(app: AppHandle, state: State<'_, AppState>, config: AppConfig) -> Result<(), String> { ... }

// Async (for blocking I/O that shouldn't freeze the UI)
#[tauri::command]
pub async fn test_online_api_connection(...) -> Result<TestConnectionResult, String> {
    tauri::async_runtime::spawn_blocking(move || { ... }).await.map_err(|e| e.to_string())?
}
```

---

## State Management

`AppState` in `state.rs` holds all shared state as `Mutex<T>` fields, managed by Tauri's `manage()`.

```rust
pub struct AppState {
    pub recognition_engine: Mutex<RecognitionEngine>,
    pub audio_buffer: AudioBuffer,            // lock-free internally
    pub correction_dict: Mutex<CorrectionDictionary>,
    pub last_result: Mutex<String>,
    pub polishing_engine: Option<Mutex<PolishingEngine>>,  // optional — may not be loaded
    pub config: Mutex<AppConfig>,
    pub native_hotkey: Mutex<Option<NativeHotkeyHandle>>,
}
```

**Rules:**
- Every mutable shared field uses `Mutex<T>`
- Optional engines use `Option<Mutex<T>>` — `None` if model not available
- `AudioBuffer` is lock-free (interior mutability pattern internally)
- All initialization happens in `lib.rs` `run()` before `tauri::Builder`

---

## Platform-Specific Code

Use `#[cfg(target_os = "macos")]` and `#[cfg(target_os = "windows")]` for platform divergence. Provide both branches when behavior differs.

```rust
#[cfg(target_os = "macos")]
{
    // ObjC native API calls
    let closure_win = window.clone();
    let _ = window.run_on_main_thread(move || {
        show_overlay_on_main_thread(&closure_win);
    });
}

#[cfg(not(target_os = "macos"))]
window.show().map_err(|e| e.to_string())?;
```

### macOS NSWindow → NSPanel conversion

The overlay window must be an NSPanel (not NSWindow) to appear above full-screen apps. This is done via `object_setClass` at runtime — see `commands.rs:show_overlay_on_main_thread`.

### Main thread requirement

macOS AppKit calls (`NSWindow` methods) must run on the main thread. Use `window.run_on_main_thread()` from command handlers, or call directly during `setup()` (which already runs on main thread).

---

## Logging

Use `println!` with a `[module_name]` prefix. This is a desktop app — stdout goes to a log file (set up in `setup_file_logging()`).

```rust
println!("[config] Loaded from {}", path.display());
println!("[polish:api] ERROR: {}, using original", e);
```

- Prefix format: `[module]` or `[module:sub]` for sub-operations
- Errors: print and recover (return original text, skip optional step) — don't panic
- Timing: use `std::time::Instant` for performance logging (`{:.1}s` format)

---

## Config Serialization

`AppConfig` uses serde with `Serialize` + `Deserialize` + `Clone`. Default values are defined in `impl Default`.

**Forward-compatible loading:** `AppConfig::load()` merges saved JSON on top of defaults, so new fields added in updates automatically get their default value without migration.

```rust
let defaults = Self::default();
let mut value = serde_json::to_value(&defaults).unwrap();
if let Ok(saved) = serde_json::from_str::<serde_json::Value>(&contents) {
    // overlay saved keys on top of defaults
}
```

---

## External Dependencies

| Crate | Purpose |
|-------|---------|
| `tauri` 2.x | App framework (with `tray-icon`, `macos-private-api` features) |
| `serde` / `serde_json` | Serialization |
| `anyhow` | Error handling in business logic |
| `cpal` | Cross-platform audio capture |
| `sherpa-rs` | SenseVoice speech recognition |
| `arboard` | Clipboard access |
| `dirs` | Platform data directories |
| `hound` | WAV file writing |
| `objc2` / `objc2-app-kit` | macOS native API access |
| `ureq` | HTTP client (blocking) for API calls |

---

## Adding a New Feature (Checklist)

1. Create module in `src-tauri/src/` (or subdirectory for complex features)
2. Add `mod` declaration in `lib.rs`
3. Add state fields to `AppState` in `state.rs` (if needed)
4. Initialize in `lib.rs` `run()` (if needed)
5. Write command functions in `commands.rs`
6. Register commands in `tauri::generate_handler![]` in `lib.rs`
7. Frontend calls with `invoke("command_name", { args })`

---

## Common Mistakes to Avoid

1. **Don't `.unwrap()` on Mutex locks in commands** — use `.map_err(|e| e.to_string())?`
2. **Don't call AppKit APIs off the main thread** — use `run_on_main_thread()`
3. **Don't use `window.show()` on macOS** — use `orderFrontRegardless` via native API
4. **Don't use Tauri's `set_visible_on_all_workspaces()`** — it overwrites our custom collection behavior
5. **Don't mix error types** — `anyhow` inside modules, `String` at the command boundary
