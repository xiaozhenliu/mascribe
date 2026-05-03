# Code Quality Analysis

**Analysis Date:** 2026-05-03

## Code Organization

### Frontend (TypeScript)
- **Files:** `src/main.ts` (main UI logic), `src/settings.ts` (settings UI)
- **Structure:** Vanilla TypeScript with inline HTML/CSS, no framework
- **Organization:** Functional approach with state variables at module level
- **Size:** Minimal, focused on UI interaction and Tauri IPC

### Backend (Rust)
- **Core files:** `src-tauri/src/lib.rs` (287 lines), `src-tauri/src/commands.rs` (624 lines)
- **Modular structure:** Organized into logical modules:
  - `audio/` - Audio capture and resampling
  - `recognition/` - SenseVoice speech recognition engine
  - `polishing/` - Local and online LLM text refinement
  - `ocr/` - Screenshot OCR (macOS/Windows variants)
  - `screenshot/` - Screenshot capture (platform-specific)
  - `correction/` - User correction dictionary
  - `insertion/` - Text insertion via CGEvent/Windows API
  - `hotkey/` - Native hotkey registration (dual-path strategy)
  - `permissions/` - System permission requests
  - `config/` - Configuration management
  - `state/` - Application state (AppState struct)
  - `tray/` - System tray integration

**Largest files:**
- `commands.rs` (624 lines) - Main command handlers
- `lib.rs` (287 lines) - App setup and initialization
- `permissions.rs` (222 lines) - Permission request logic

## Naming Conventions

### TypeScript
- **Functions:** camelCase (`toggleRecording`, `showToast`, `drawWaveform`)
- **Variables:** camelCase (`isRecording`, `isProcessing`, `currentAmplitude`)
- **Constants:** UPPER_SNAKE_CASE (`WAVEFORM_POINTS`, `AGC_ATTACK`, `AGC_DECAY`)
- **DOM helpers:** Lowercase with underscores (`status-dot`, `waveform`)

### Rust
- **Functions:** snake_case (`show_window`, `start_recording`, `stop_recording_and_transcribe`)
- **Structs:** PascalCase (`AppState`, `AppConfig`, `RecognitionEngine`, `PolishingEngine`)
- **Constants:** UPPER_SNAKE_CASE (`SENSEVOICE_DIR`, `POLISH_MODEL_FILE`, `DEFAULT_POLISH_PROMPT`)
- **Modules:** snake_case (`audio`, `recognition`, `polishing`, `ocr`)

**Consistency:** Naming is consistent within each language. No mixed conventions detected.

## Code Style

### TypeScript
- **Linting:** No ESLint configuration detected (`.eslintrc*` not present)
- **Formatting:** No Prettier configuration detected
- **Style:** Loose, no strict type checking enforced
- **Patterns:** 
  - Async/await used for Tauri `invoke()` calls
  - Event listeners for hotkey registration
  - Canvas-based waveform rendering with requestAnimationFrame
  - State management via module-level variables

### Rust
- **Conventions:** Follows Rust standard conventions
- **Error handling:** Uses `anyhow::Result` for error propagation
- **Serialization:** Uses `serde_json` for JSON handling
- **Platform-specific code:** Conditional compilation via `#[cfg(target_os = "...")]`
- **Unsafe code:** Used sparingly for:
  - NSPanel conversion via `object_setClass` (macOS fullscreen overlay)
  - File descriptor redirection for logging (`libc::dup2`)
  - Objective-C runtime calls via `objc2`

**Example error handling** (`src-tauri/src/commands.rs`):
```rust
pub fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        // Platform-specific logic
    }
    Ok(())
}
```

## Testing Status

**Current State:** No automated tests present
- No `.test.ts`, `.spec.ts`, `.test.rs`, or `.spec.rs` files found
- No test framework configured (Jest, Vitest, Cargo test not set up)
- **Testing approach:** Manual testing only (documented in CLAUDE.md)

**Test Scenarios (Manual):**
- Voice recording in various apps
- Fullscreen app overlay functionality
- Auto-paste with/without Accessibility permission
- AI polishing (local and online modes)
- Multi-language transcription

**Impact:** High-risk areas lack automated coverage:
- Transcription pipeline (`stop_recording_and_transcribe` command)
- Fullscreen overlay NSPanel conversion
- Hotkey registration dual-path fallback
- Configuration persistence and loading

## Error Handling

### Rust Backend
**Strategy:** Result-based error propagation with `anyhow::Result`

**Patterns observed:**
- Commands return `Result<T, String>` for Tauri IPC
- Internal functions use `anyhow::Result` for error context
- Errors logged to `~/Library/Logs/MaScribe.log` (macOS) or `%APPDATA%\MaScribe\logs` (Windows)

**Error scenarios:**
- **Audio capture:** Logged, user sees toast "Mic error", recording cancelled
- **Transcription:** If audio <300ms: "Recording too short"; if model fails: error logged, toast shown
- **Polishing:** Falls back to uncorrected text, logs error, pipeline continues (optional feature)
- **Insertion:** CGEvent fails → tries AppleScript fallback → if both fail, text stays in clipboard

**Logging setup** (`src-tauri/src/lib.rs`):
- macOS: Redirects stdout/stderr to `~/Library/Logs/MaScribe.log` via `libc::dup2`
- Windows: Prints log location (no actual redirection)
- Uses `println!` for debug output

### TypeScript Frontend
**Patterns:**
- Tauri `invoke()` calls use `.catch(() => {})` for silent error handling
- Toast notifications for user-facing errors
- No structured error logging

**Example** (`src/main.ts`):
```typescript
invoke("hide_window").catch(() => {});
```

## Documentation

### Code Comments
**Rust:**
- Well-commented in critical sections:
  - Fullscreen overlay logic: Explains NSPanel conversion, window levels, collection behavior
  - Auto-paste mechanism: Documents clipboard + CGEvent approach
  - Hotkey registration: Explains dual-path strategy (Tauri global-shortcut + CGEventTap fallback)
  - Permission requests: Clear comments on why each permission is needed
- Comments use English (per CLAUDE.md rules)

**TypeScript:**
- Minimal comments, mostly self-documenting code
- State variables have inline comments explaining purpose (AGC, waveform tracking)

### API Documentation
- No JSDoc/TSDoc annotations in TypeScript
- No Rust doc comments (`///`) for public functions
- Tauri commands documented in ARCH.md instead of inline

### User Documentation
- `README.md` (Chinese) - User guide
- `README.en.md` (English) - User guide
- `docs/PRD.md` - Product requirements
- `docs/macos-guide.md` - macOS installation & permissions
- `docs/local-build-guide.md` - Build instructions
- `docs/online-api-guide-zh.md` - Online API setup (Chinese)
- `docs/online-api-guide-en.md` - Online API setup (English)
- `CLAUDE.md` - Codebase context for AI assistants

## Technical Debt & Concerns

### Critical Issues

**1. No Automated Tests**
- **Impact:** High — transcription pipeline, fullscreen overlay, hotkey fallback untested
- **Files:** `src-tauri/src/commands.rs` (624 lines), `src/main.ts` (hotkey logic)
- **Fix approach:** Set up Cargo test framework for Rust, add integration tests for critical paths

**2. Manual Model Download**
- **Issue:** Users must run `scripts/install-sensevoice-model.sh` manually
- **Files:** `scripts/install-sensevoice-model.sh`, `src-tauri/src/config.rs`
- **Impact:** Poor onboarding experience, support burden
- **Fix approach:** Implement automatic model download on first run with progress UI

**3. Windows Support Incomplete**
- **Issue:** Windows API integration in progress, not production-ready
- **Files:** `src-tauri/src/permissions.rs`, `src-tauri/src/screenshot/windows.rs`, `src-tauri/src/ocr/windows.rs`
- **Impact:** Windows users get incomplete feature set
- **Fix approach:** Complete Windows hotkey registration, test fullscreen overlay on Windows

**4. No Auto-update Mechanism**
- **Issue:** Users must manually download new releases
- **Files:** No update logic present
- **Impact:** Users may miss critical bug fixes
- **Fix approach:** Implement Tauri updater plugin

### Code Quality Issues

**5. Large Command Handler**
- **Issue:** `stop_recording_and_transcribe` in `commands.rs` handles entire pipeline (audio → transcription → correction → OCR → polishing → insertion)
- **Lines:** ~150+ lines in single function
- **Impact:** Hard to test, modify, or debug individual steps
- **Fix approach:** Extract pipeline steps into separate functions with clear error handling

**6. Unsafe Code in Fullscreen Overlay**
- **Issue:** `object_setClass` runtime NSPanel conversion is memory-unsafe
- **Files:** `src-tauri/src/commands.rs` (show_overlay_on_main_thread)
- **Impact:** Potential crashes if NSWindow/NSPanel memory layout changes
- **Mitigation:** Currently works, but fragile
- **Fix approach:** Add safety comments, consider alternative approaches (e.g., custom window creation)

**7. Logging Infrastructure Incomplete**
- **Issue:** Windows logging just prints location, doesn't actually redirect
- **Files:** `src-tauri/src/lib.rs` (setup_file_logging)
- **Impact:** Windows users can't debug issues
- **Fix approach:** Implement proper Windows logging via file handle redirection or logging crate

**8. No Input Validation**
- **Issue:** Config values (API endpoints, model paths) not validated on load
- **Files:** `src-tauri/src/config.rs`
- **Impact:** Invalid config can cause silent failures
- **Fix approach:** Add validation in `AppConfig::load()`, provide user feedback

### Fragile Areas

**9. Hotkey Registration Dual-Path**
- **Issue:** Falls back to CGEventTap if Tauri global-shortcut fails, but fallback may not work for all keys
- **Files:** `src/main.ts` (registerShortcut), `src-tauri/src/hotkey.rs`
- **Risk:** User may think hotkey is registered when it's not
- **Fix approach:** Add explicit feedback when fallback is used, test all key combinations

**10. OCR Context Injection**
- **Issue:** Screen text injected into LLM prompt with "reference only" markers, but LLM may ignore markers
- **Files:** `src-tauri/src/polishing/online.rs`
- **Risk:** LLM outputs OCR text verbatim instead of correcting transcription
- **Fix approach:** Add post-processing to detect and remove OCR text from output

### Performance Concerns

**11. Synchronous Online API Calls**
- **Issue:** Online polishing uses `ureq` (blocking HTTP), blocks entire pipeline
- **Files:** `src-tauri/src/polishing/online.rs`
- **Impact:** 1-5s latency blocks UI, no cancellation possible
- **Fix approach:** Use async HTTP client (tokio + reqwest), allow user to cancel

**12. Model Loading at Startup**
- **Issue:** SenseVoice and optional GGUF models loaded at app startup (1-2s)
- **Files:** `src-tauri/src/lib.rs`
- **Impact:** Slow app launch
- **Fix approach:** Lazy-load models on first use, show progress indicator

## Summary

**Strengths:**
- Clear modular architecture with platform-specific code separation
- Consistent naming conventions within each language
- Well-documented critical implementation details (fullscreen overlay, auto-paste, hotkey fallback)
- Comprehensive user documentation

**Weaknesses:**
- No automated tests (high risk for regression)
- Large monolithic command handlers
- Incomplete Windows support
- Manual model download process
- No auto-update mechanism
- Unsafe code in fullscreen overlay (fragile)
- Incomplete logging on Windows

**Priority Fixes:**
1. Add automated tests for transcription pipeline
2. Extract pipeline steps into testable functions
3. Implement automatic model download
4. Complete Windows support
5. Add proper logging on Windows

---

*Quality analysis: 2026-05-03*
