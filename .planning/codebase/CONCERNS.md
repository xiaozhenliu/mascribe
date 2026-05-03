# Codebase Concerns

**Analysis Date:** 2026-05-03

## Security Concerns

**API Key Storage:**
- Issue: `online_api_key` stored in plaintext in `~/Library/Application Support/com.mascribe/config.json`
- Files: `src-tauri/src/config.rs`, `src-tauri/src/polishing/online.rs`
- Risk: If config file is compromised, API keys are exposed
- Recommendation: Use macOS Keychain for sensitive credentials, or encrypt config file

**Clipboard Security:**
- Issue: Transcribed text written to clipboard before insertion, remains accessible to other apps
- Files: `src-tauri/src/insertion/macos.rs`
- Risk: Sensitive voice input (passwords, personal info) could be read by clipboard monitoring apps
- Mitigation: Clear clipboard after insertion (currently not implemented)
- Fix approach: Add clipboard clearing after successful text insertion

**Unsafe Code in Fullscreen Overlay:**
- Issue: Runtime NSPanel conversion via `object_setClass` uses unsafe Objective-C runtime manipulation
- Files: `src-tauri/src/commands.rs` (line 73), `src-tauri/src/lib.rs` (line 206)
- Risk: Memory layout assumptions could break with macOS updates; potential crashes
- Current state: Works but fragile
- Fix approach: Add comprehensive safety comments, consider alternative NSPanel creation approach

**Permission Handling:**
- Issue: Accessibility permission check uses unsafe FFI call without error recovery
- Files: `src-tauri/src/permissions.rs` (lines 39, 89, 136, 145)
- Risk: If permission check fails, app may crash or behave unexpectedly
- Fix approach: Add proper error handling and user feedback for permission failures

## Performance Risks

**Synchronous Online API Calls:**
- Issue: Online polishing uses `ureq` (blocking HTTP client), blocks entire transcription pipeline
- Files: `src-tauri/src/polishing/online.rs` (line 155)
- Impact: 1-5s latency blocks UI, no user cancellation possible
- Bottleneck: Network round-trip + server inference
- Fix approach: Migrate to async HTTP client (tokio + reqwest), allow user to cancel mid-pipeline

**Model Loading at Startup:**
- Issue: SenseVoice and optional GGUF models loaded at app startup (1-2s delay)
- Files: `src-tauri/src/lib.rs` (initialization), `src-tauri/src/state.rs`
- Impact: Slow app launch, poor user experience
- Fix approach: Lazy-load models on first use with progress indicator

**Large Monolithic Command Handler:**
- Issue: `stop_recording_and_transcribe` in `commands.rs` handles entire pipeline (150+ lines)
- Files: `src-tauri/src/commands.rs` (main pipeline)
- Impact: Hard to optimize individual steps, difficult to add cancellation
- Fix approach: Extract pipeline steps into separate functions with clear boundaries

**OCR Context Injection Overhead:**
- Issue: Screenshot capture + OCR adds 5-7s latency when using API-based vision
- Files: `src-tauri/src/ocr/` (macos.rs, windows.rs)
- Impact: Total pipeline time can exceed 10s with online polishing + OCR
- Recommendation: Make OCR optional, add user-facing progress indicator

## Platform Compatibility Issues

**Windows Support Incomplete:**
- Issue: Windows API integration in progress, not production-ready
- Files: `src-tauri/src/permissions.rs`, `src-tauri/src/screenshot/windows.rs`, `src-tauri/src/ocr/windows.rs`, `src-tauri/src/insertion/windows.rs`
- Missing: Fullscreen overlay equivalent, complete hotkey registration, proper logging
- Impact: Windows users get incomplete feature set
- Fix approach: Complete Windows hotkey registration, test fullscreen overlay on Windows, implement proper logging

**Logging Infrastructure Incomplete on Windows:**
- Issue: Windows logging just prints location, doesn't actually redirect stderr/stdout
- Files: `src-tauri/src/lib.rs` (setup_file_logging)
- Impact: Windows users can't debug issues, no error logs available
- Fix approach: Implement proper Windows logging via file handle redirection or logging crate

**Hotkey Registration Dual-Path Fragility:**
- Issue: Falls back to CGEventTap if Tauri global-shortcut fails, but fallback may not work for all keys
- Files: `src/main.ts` (registerShortcut), `src-tauri/src/hotkey.rs`
- Risk: User may think hotkey is registered when it's not
- Fix approach: Add explicit feedback when fallback is used, test all key combinations

## Maintenance & Dependency Risks

**Manual Model Download Process:**
- Issue: Users must run `scripts/install-sensevoice-model.sh` manually
- Files: `scripts/install-sensevoice-model.sh`, `src-tauri/src/config.rs`
- Impact: Poor onboarding, high support burden, users may skip setup
- Fix approach: Implement automatic model download on first run with progress UI

**No Auto-update Mechanism:**
- Issue: Users must manually download new releases
- Files: No update logic present
- Impact: Users may miss critical bug fixes, security patches
- Fix approach: Implement Tauri updater plugin with staged rollout

**No Automated Tests:**
- Issue: Zero test coverage for critical paths
- Files: No `.test.ts`, `.spec.ts`, `.test.rs`, or `.spec.rs` files
- Impact: High regression risk, especially for transcription pipeline and fullscreen overlay
- Untested areas:
  - `stop_recording_and_transcribe` command (150+ lines)
  - NSPanel conversion logic
  - Hotkey registration fallback
  - Configuration persistence
- Fix approach: Set up Cargo test framework for Rust, add integration tests for critical paths

**Dependency Version Pinning:**
- Issue: Some dependencies use loose version constraints (e.g., `sherpa-rs` 0.6, `llama-cpp-2` 0.1)
- Files: `src-tauri/Cargo.toml`
- Risk: Minor version updates could introduce breaking changes
- Recommendation: Pin exact versions for critical dependencies, test updates before merging

## Technical Debt & Fragile Areas

**Unwrap Usage Throughout Codebase:**
- Issue: Multiple `.unwrap()` calls without error handling
- Files: `src-tauri/src/commands.rs` (lines 183, 223, 237, 354, 358, 534), `src-tauri/src/config.rs` (lines 28, 82, 116, 132), `src-tauri/src/lib.rs` (lines 30, 58, 115, 118)
- Risk: Panics if assumptions fail (e.g., home directory missing, config file corrupted)
- Fix approach: Replace `.unwrap()` with `.unwrap_or_else()` or proper error handling

**OCR Context Injection Logic:**
- Issue: Screen text injected into LLM prompt with "reference only" markers, but LLM may ignore markers
- Files: `src-tauri/src/polishing/online.rs`
- Risk: LLM outputs OCR text verbatim instead of correcting transcription
- Fix approach: Add post-processing to detect and remove OCR text from output

**Config Validation Missing:**
- Issue: Config values (API endpoints, model paths) not validated on load
- Files: `src-tauri/src/config.rs`
- Impact: Invalid config can cause silent failures or crashes
- Fix approach: Add validation in `AppConfig::load()`, provide user feedback for invalid values

**Audio Buffer Management:**
- Issue: Circular audio buffer in `AudioBuffer` may have edge cases with concurrent access
- Files: `src-tauri/src/audio/` (buffer implementation)
- Risk: Audio dropout or corruption if buffer fills faster than consumed
- Recommendation: Add buffer overflow detection and logging

## User Experience Concerns

**Permission Setup Complexity:**
- Issue: Requires 4 separate permissions (Microphone, Accessibility, Input Monitoring, Screen Recording)
- Files: `src-tauri/src/permissions.rs`
- Impact: Users may not grant all permissions, features silently fail
- Fix approach: Implement permission wizard on first run, explain why each is needed

**Error Messages Unclear:**
- Issue: Toast notifications show generic errors ("Mic error", "Recording too short")
- Files: `src/main.ts` (toast display)
- Impact: Users don't know how to fix issues
- Fix approach: Add detailed error messages with troubleshooting steps

**Model Download Failure Handling:**
- Issue: If model download fails, app may not start or may start with degraded functionality
- Files: `scripts/install-sensevoice-model.sh`, `src-tauri/src/lib.rs`
- Impact: Users stuck without clear recovery path
- Fix approach: Add in-app model download with retry logic and clear error messages

**No Progress Indication for Long Operations:**
- Issue: Polishing and OCR operations (1-10s) show no progress to user
- Files: `src-tauri/src/commands.rs` (pipeline), `src/main.ts` (UI)
- Impact: Users think app is frozen
- Fix approach: Emit progress events during pipeline, show spinner in UI

## Scaling & Capacity Limits

**Single-threaded Model Inference:**
- Issue: SenseVoice and GGUF models run single-threaded, no parallelization possible
- Files: `src-tauri/src/recognition/`, `src-tauri/src/polishing/engine.rs`
- Impact: Cannot process multiple recordings concurrently
- Limitation: Inherent to ONNX/GGUF runtime, not easily fixable

**Memory Usage with Large Models:**
- Issue: GGUF models (1GB+) loaded entirely into memory
- Files: `src-tauri/src/polishing/engine.rs`
- Impact: High memory footprint, may cause issues on low-end machines
- Recommendation: Document minimum system requirements, consider model quantization

---

*Concerns audit: 2026-05-03*
