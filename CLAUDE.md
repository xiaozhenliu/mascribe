# mac-voice-input

macOS 语音输入工具 — 按住快捷键说话，本地转写后自动输入到任意 App。

## Tech Stack

- **Framework**: Tauri 2.0 (Rust backend + Web frontend)
- **Frontend**: TypeScript + Vite (vanilla, no framework)
- **Speech Model**: SenseVoice via sherpa-rs (ONNX, local inference)
- **Audio Capture**: cpal crate (cross-platform audio I/O)
- **Hotkey**: CGEventTap (Rust, macOS native, system-level key capture)
- **Text Insertion**: arboard (clipboard) + CGEvent (Cmd+V simulation)
- **Serialization**: serde + serde_json
- **AI Polishing (local)**: Qwen 2.5 1.5B Instruct (GGUF) via llama-cpp-2 crate
- **AI Polishing (online)**: OpenAI-compatible chat completions API via ureq (sync HTTP)
- **Screen OCR**: Native OS OCR (macOS Vision ~0.6s / Windows.Media.Ocr ~50-200ms) or GLM-OCR via Ollama (fallback)

## Key Paths

- **SenseVoice Model**: `~/.openclaw/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/`
- **Polish Model (Qwen)**: `~/.openclaw/models/qwen2.5-1.5b/qwen2.5-1.5b-instruct-q4_k_m.gguf`
- **App Config**: `~/Library/Application Support/com.mac-voice-input/config.json`
- **Corrections Dict**: `~/Library/Application Support/com.mac-voice-input/corrections.json`
- **PRD**: `docs/PRD.md`

## Development

```bash
cargo tauri dev        # Run in dev mode (hot-reload frontend, rebuilds Rust)
cargo tauri build      # Build .app + .dmg for distribution
npm run dev            # Frontend-only dev server (Vite)
npm run build          # Frontend production build
```

## Project Structure

```
src-tauri/src/         Rust backend
  lib.rs               App setup, plugin registration, orchestration
  commands.rs          Tauri IPC command handlers
  state.rs             Shared app state (AppState)
  config.rs            Settings persistence
  tray.rs              Menu bar tray icon
  audio/               Microphone capture + resampling
  hotkey/              CGEventTap key listener (configurable)
  recognition/         sherpa-rs SenseVoice wrapper
  insertion/           Clipboard paste + Cmd+V simulation
  correction/          JSON-based text correction dictionary
  ocr/                 Native OCR (macOS Vision framework VNRecognizeTextRequest)
  polishing/           AI text polishing (dual-engine)
    engine.rs          Local GGUF model (llama-cpp-2, ChatML/Gemma auto-detect)
    online.rs          OpenAI-compatible API client (ureq, sync HTTP)

src/                   Web frontend (TypeScript)
  index.html           Floating panel (frameless, always-on-top)
  settings.html        Settings window
  main.ts / settings.ts
  styles/              CSS
  lib/                 Waveform renderer, Tauri API wrappers, types
```

## Conventions

- Rust: standard `rustfmt` formatting, `clippy` clean
- Frontend: TypeScript strict mode, no UI frameworks (vanilla TS)
- Keep dependencies minimal — prefer standard library where possible
- All speech processing happens in Rust via sherpa-rs (no Python runtime)
- Chinese comments are acceptable for Chinese-specific logic
- Commit messages in English

## Required macOS Permissions

- **Microphone**: audio capture
- **Accessibility**: simulate Cmd+V keystrokes
- **Input Monitoring**: CGEventTap for hotkey capture
- **Screen Recording**: screenshot capture for OCR context (optional)

## Architecture Notes

- Single-process Rust app (no Python sidecar, no IPC)
- SenseVoice model loaded once at startup, stays in memory (~300MB)
- Hotkey is fully configurable via a key recorder widget in settings
- Text insertion uses clipboard save/restore pattern to not destroy user's clipboard
- App runs as Accessory (no Dock icon, menu bar only)

## AI Polishing Pipeline

```
screenshot → OCR (native Vision or Ollama) → screen context text (optional)
                                                    ↓
transcribe → corrections → polish(mode) + screen context → insert
                            ├── Local:  llama-cpp-2 (Qwen 2.5 1.5B, ChatML prompt)
                            └── Online: ureq HTTP POST (OpenAI /chat/completions)
```

- **Two-step OCR→Polish pipeline** (when Screen OCR is enabled):
  1. Screenshot → OCR (native `VNRecognizeTextRequest` or GLM-OCR via Ollama) → extracted screen text
  2. Screen text injected into polish prompt with `[SCREEN CONTEXT]` labels (clearly separated from `[TRANSCRIPT]`)
  3. Polishing model uses screen context to correct homophones (e.g., "把" vs "八")
  - **OCR modes**: `vision_mode` = "native" (macOS Vision, ~0.6s), "api" (Ollama, ~5-7s), or "disabled"
  - **Native OCR**: macOS uses objc2 `msg_send!` for Vision framework; Windows uses `Windows.Media.Ocr` WinRT via `windows` crate
  - Both support zh-Hans/zh-Hant/en-US/ja-JP/ko-KR (zh-Hans engine handles mixed CJK+ASCII natively)
  - **OCR context only injected for online API mode** — local Qwen 2.5 1.5B has ~512 token batch limit, too small for extra context
  - **Output validation**: rejects API output > 3× input length + 20 chars (prevents OCR content leakage)
  - OCR text truncated to 500 chars to keep prompt reasonable
  - Config fields: `vision_mode` ("disabled"/"native"/"api"), `ocr_endpoint`, `ocr_model`
- **Dual-engine**: user chooses "Local Model", "Online API", or "Off" in Settings
- **Local model**: Qwen 2.5 1.5B Instruct GGUF (~1.1GB), loaded once at startup via llama-cpp-2
  - Auto-detects chat template from filename: "qwen" → ChatML, otherwise Gemma
  - Temperature 0.15, greedy sampling for deterministic output
  - `validate_output()` rejects broken outputs (empty, too long, language switch)
  - `strip_meta_prefix()` removes LLM preamble like "Here is the cleaned text:"
- **Online API**: stateless `OnlinePolisher` created per-call (no startup cost)
  - Endpoint auto-appends `/chat/completions` if missing
  - Works with Step-Fun, DeepSeek, Qwen-Turbo, Groq, OpenAI, etc.
  - 10s read / 5s write timeout via ureq
  - HTTP error responses log response body (up to 300 chars) for diagnosing model name / endpoint issues
- **Prompt template**: supports `{text}` and `{lang}` placeholders
  - `{lang}` is auto-filled from SenseVoice detection (zh, en, ja, etc.)
- Config fields: `polish_enabled`, `polish_mode` ("local"/"api"), `api_endpoint`, `api_key`, `api_model`
