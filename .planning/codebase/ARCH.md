# Architecture

**Analysis Date:** 2026-05-03

## System Overview

MaScribe is a Tauri-based desktop application with a clear separation between frontend (TypeScript/Vite) and backend (Rust). The architecture follows a command-based IPC pattern where the frontend triggers backend operations via Tauri's `invoke()` mechanism.

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (TypeScript)                    │
│  src/main.ts (UI, hotkey registration, state management)    │
│  src/settings.ts (Settings UI)                              │
└────────────────────┬────────────────────────────────────────┘
                     │ Tauri IPC (invoke/emit)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  Backend (Rust/Tauri)                        │
│  lib.rs (app setup, state initialization)                   │
│  commands.rs (command handlers)                             │
│  state.rs (AppState with shared resources)                  │
│  config.rs (configuration management)                       │
└─────────────────────────────────────────────────────────────┘
                     │
        ┌────────────┼────────────┬──────────────┐
        ▼            ▼            ▼              ▼
    Audio       Recognition   Polishing    Insertion
    (cpal)      (sherpa-onnx)  (llama-cpp)  (CGEvent)
```

## Core Layers

### Frontend Layer (`src/main.ts`)

**Purpose:** UI rendering, user interaction, hotkey management, state visualization

**Key Responsibilities:**
- Display recording UI with waveform visualization (ECG-style canvas animation)
- Manage hotkey registration (dual-path: Tauri global-shortcut + native CGEventTap fallback)
- Handle recording lifecycle: show window → start recording → stop & transcribe → hide window
- Display toast notifications for status/errors
- Auto-gain control (AGC) for waveform normalization

**State Variables:**
- `isRecording`: Whether audio capture is active
- `isProcessing`: Blocks new recordings while transcribing/polishing/inserting
- `currentShortcut`: Currently registered hotkey
- `usingNativeHotkey`: Whether CGEventTap fallback is active

**Key Functions:**
- `toggleRecording()`: Main entry point for hotkey press/release
- `registerShortcut()`: Handles hotkey registration with fallback logic
- `startPolling()`: Polls backend for audio amplitude every 50ms during recording
- `drawWaveform()`: Renders ECG-style waveform on canvas

### Backend Command Layer (`src-tauri/src/commands.rs`)

**Purpose:** Tauri command handlers that bridge frontend requests to backend services

**Commands:**
- `start_recording()`: Initializes audio capture stream
- `stop_recording_and_transcribe()`: Main pipeline—audio → transcription → correction → OCR → polishing → insertion
- `get_amplitude()`: Returns current audio level for waveform display
- `show_window()` / `hide_window()`: Window visibility (macOS uses native NSPanel API)
- `get_config()` / `save_config()`: Configuration persistence
- `get_corrections()` / `save_corrections()`: Correction dictionary management
- `register_native_hotkey()` / `unregister_native_hotkey()`: CGEventTap-based hotkey fallback

**Critical Pipeline (`stop_recording_and_transcribe`):**
1. Stop audio stream, collect samples
2. Validate minimum duration (300ms)
3. Capture screenshot (if enabled)
4. Save raw audio to WAV
5. Resample to 16kHz (SenseVoice requirement)
6. Transcribe via SenseVoice model
7. Apply correction dictionary
8. Extract screen context via OCR (native Vision or Ollama API)
9. Polish text via LLM (local GGUF or online API)
10. Insert text into active app via CGEvent
11. Emit `transcription-complete` event

### Application State (`src-tauri/src/state.rs`)

**Purpose:** Shared, thread-safe state accessible to all command handlers

**Key Fields:**
- `recognition_engine: Mutex<RecognitionEngine>`: SenseVoice model instance
- `audio_buffer: AudioBuffer`: Circular buffer for audio samples during recording
- `correction_dict: Mutex<CorrectionDictionary>`: User-defined text replacements
- `polishing_engine: Option<Mutex<PolishingEngine>>`: Local GGUF model (optional)
- `config: Mutex<AppConfig>`: User settings (hotkey, model paths, API endpoints)
- `native_hotkey: Mutex<Option<NativeHotkeyHandle>>`: CGEventTap listener handle

### Configuration Layer (`src-tauri/src/config.rs`)

**Purpose:** Load/save user settings from disk

**Location:** `~/Library/Application Support/com.mascribe/config.json` (macOS)

**Key Settings:**
- `shortcut`: Hotkey string (e.g., "Alt+Space", "F13")
- `language`: Auto-detect or fixed language
- `model_dir`: Path to SenseVoice model files
- `polish_enabled`: Whether to apply LLM polishing
- `polish_mode`: "none" | "local" | "api"
- `polish_prompt`: Custom prompt template for LLM
- `api_endpoint`: OpenAI-compatible API URL (for online polishing)
- `api_key`: API authentication token
- `api_model`: Model name on API server
- `screenshot_mode`: "disabled" | "enabled"
- `vision_mode`: "disabled" | "native" | "api" (OCR mode)
- `ocr_endpoint`: Ollama/API endpoint for OCR
- `ocr_model`: OCR model name

## Data Flow

### Voice Recording Flow

```
User presses hotkey
    ↓
toggleRecording() [frontend]
    ↓
show_window() [command] → NSPanel overlay appears
    ↓
start_recording() [command] → cpal stream starts
    ↓
Audio samples → AudioBuffer (circular, 16-bit PCM)
    ↓
get_amplitude() polled every 50ms [command]
    ↓
Frontend draws waveform with AGC normalization
    ↓
User releases hotkey
    ↓
toggleRecording() [frontend]
    ↓
stop_recording_and_transcribe() [command] → MAIN PIPELINE
```

### Transcription Pipeline

```
Audio samples (variable rate)
    ↓
Resample to 16kHz (SenseVoice requirement)
    ↓
RecognitionEngine.transcribe() → SenseVoice model
    ↓
Raw text + detected language
    ↓
CorrectionDictionary.apply() → user replacements
    ↓
Corrected text
    ↓
[Optional] Screenshot capture + OCR
    ↓
[Optional] LLM polishing (local or online)
    ↓
Final text
    ↓
insertion::insert_text() → CGEvent Cmd+V
    ↓
Text appears in active app
```

### Polishing Integration

**Local Mode:**
- Uses `llama-cpp-2` with GGUF model (e.g., Qwen 2.5 1.5B)
- Loaded at startup if `polish_enabled=true` and model exists
- Synchronous call during pipeline (blocks until complete)
- No OCR context injection (model too weak)

**Online Mode:**
- Uses OpenAI-compatible API (Ollama, OpenAI, DeepSeek, etc.)
- Endpoint and model configured in settings
- Supports OCR context injection (screen text as reference)
- Async HTTP call via `ureq` (blocking, but acceptable for ~1-2s latency)

**OCR Integration:**
- Native macOS Vision framework (fast, <0.5s)
- OR Ollama API with vision model (slower, 5-7s)
- Screen context injected into polish prompt with clear "reference only" markers
- Prevents LLM from outputting OCR text verbatim

## Critical Implementation Details

### Fullscreen Overlay (macOS)

**Problem:** Standard Tauri windows can't appear above fullscreen apps (e.g., fullscreen video, presentation mode)

**Solution:** Runtime NSPanel conversion via `object_setClass`

**Implementation** (`src-tauri/src/lib.rs` setup + `src-tauri/src/commands.rs` show_overlay_on_main_thread):
- Convert NSWindow → NSPanel at runtime (memory-safe, no extra ivars)
- Set window level to `NSScreenSaverWindowLevel` (1000, above fullscreen ~0-24)
- Collection behavior: `CanJoinAllSpaces` + `FullScreenAuxiliary` + `Transient` + `Stationary`
- Style: `NonactivatingPanel` + `UtilityWindow` (non-activating, doesn't steal focus)
- Show via `orderFrontRegardless()` instead of `makeKeyAndOrderFront()` (avoids Space switching)

**Why this matters:** User can record voice while in fullscreen app without being pulled out of fullscreen

### Auto-paste Mechanism

**Problem:** Text must be inserted at cursor position in any app, even if MaScribe window isn't focused

**Solution:** Clipboard + CGEvent simulation

**Implementation** (`src-tauri/src/insertion.rs`):
1. Write transcribed text to clipboard
2. Simulate Cmd+V using `CGEvent::createKeyboardEvent()` (requires Accessibility permission)
3. Fallback to AppleScript if CGEvent fails
4. Window is `focusable: false` to preserve target app focus

**Why this matters:** User stays in their app; text appears automatically without manual paste

### Hotkey Registration (Dual-Path)

**Problem:** Tauri's global-shortcut plugin doesn't support all keys (e.g., F13-F24, ContextMenu)

**Solution:** Dual registration strategy

**Implementation** (`src/main.ts` registerShortcut + `src-tauri/src/hotkey.rs`):
1. Try Tauri's `@tauri-apps/plugin-global-shortcut` first (handles Alt+Space, Ctrl+Shift+X, etc.)
2. If unsupported key detected or registration fails, fall back to native CGEventTap (macOS) or SetWindowsHookEx (Windows)
3. CGEventTap listens for key events at system level, emits `native-hotkey-pressed` event
4. Frontend listens for event and calls `toggleRecording()`

**Why this matters:** Users can bind to any key, including media keys and function keys

### Model Management

**SenseVoice Model:**
- Location: `~/Library/Application Support/com.mascribe/models/sensevoice/`
- Files: `model.int8.onnx` (quantized), `tokens.txt` (vocabulary)
- Size: ~200MB
- Download: Manual via `scripts/install-sensevoice-model.sh`
- Loaded at app startup (1-2s first load, cached after)

**Polishing Model (Optional):**
- Local: GGUF format (e.g., Qwen 2.5 1.5B, ~1GB)
- Path: Configured in settings
- Loaded at startup if enabled and file exists
- If missing, polishing is silently disabled

## Integration Points

### System Permissions (macOS)

**Microphone:** Requested at startup via `AVCaptureDevice.requestAccess()`
- Required for audio capture
- Shows system dialog on first run

**Accessibility:** Requested at startup via `AXIsProcessTrustedWithOptions(prompt:true)`
- Required for CGEvent.post() (Cmd+V simulation)
- Shows system dialog directing to System Settings → Accessibility

**Input Monitoring:** Required for CGEventTap hotkey listener
- Requested implicitly when CGEventTap is created
- Shows system dialog if not already granted

**Screen Recording:** Required for screenshot capture
- Requested implicitly when screenshot is taken
- Shows system dialog if not already granted

### Clipboard Integration

**Library:** `arboard` (cross-platform)

**Usage:**
- Write transcribed text before insertion
- Read/write correction dictionary (future)

### External APIs

**Online Polishing:**
- Endpoint: OpenAI-compatible (e.g., `http://localhost:11434/v1/chat/completions`)
- Auth: Bearer token in `Authorization` header
- Request: Standard OpenAI chat completion format
- Timeout: 30s read, 5s write

**OCR via API:**
- Endpoint: OpenAI-compatible vision API
- Request: Base64-encoded PNG + text prompt
- Response: Extracted text from screenshot

## Error Handling Strategy

**Audio Capture Errors:**
- Logged to `~/Library/Logs/MaScribe.log`
- User sees toast: "Mic error"
- Recording cancelled, window hidden

**Transcription Errors:**
- If audio too short (<300ms): "Recording too short"
- If model fails: Error logged, user sees toast
- Pipeline aborts, window hidden

**Polishing Errors:**
- Local model: Falls back to uncorrected text, logs error
- Online API: Falls back to uncorrected text, logs error
- Pipeline continues (polishing is optional)

**Insertion Errors:**
- CGEvent fails: Tries AppleScript fallback
- Both fail: Error logged, user sees toast
- Text remains in clipboard for manual paste

## Performance Characteristics

**Transcription:** ~50ms for 5s audio (Apple Silicon)
- Bottleneck: SenseVoice model inference
- Parallelizable: No (single-threaded model)

**Polishing (Local):** ~1-3s for typical sentence
- Bottleneck: GGUF model inference
- Parallelizable: No (single-threaded)

**Polishing (Online):** ~1-5s depending on API latency
- Bottleneck: Network round-trip + server inference
- Parallelizable: Yes (async HTTP)

**OCR (Native):** <0.5s
- Bottleneck: Vision framework processing
- Parallelizable: No (synchronous)

**OCR (API):** 5-7s
- Bottleneck: Network + server inference
- Parallelizable: Yes (async HTTP)

**Total Pipeline:** 100ms - 10s depending on configuration
- Typical (transcribe only): 100-200ms
- With local polishing: 1-3s
- With online polishing + OCR: 5-10s

---

*Architecture analysis: 2026-05-03*
