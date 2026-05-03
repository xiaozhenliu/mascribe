# Technology Stack

**Analysis Date:** 2026-05-03

## Languages

**Primary:**
- TypeScript 5.6.2 - Frontend UI and Tauri command invocation
- Rust (Edition 2021) - Backend application logic, audio processing, speech recognition

**Secondary:**
- Bash - Build scripts and model installation (`scripts/install-sensevoice-model.sh`)

## Runtime

**Environment:**
- Node.js (via npm) - Frontend development and build
- Rust toolchain - Backend compilation

**Package Manager:**
- npm - Frontend dependencies
- Cargo - Rust dependencies
- Lockfile: `package-lock.json`, `Cargo.lock` (present)

## Frameworks

**Core:**
- Tauri v2 - Desktop application framework with IPC bridge
- Vite 6.0.3 - Frontend build tool and dev server

**Frontend:**
- Vanilla TypeScript - No framework (custom HTML/CSS)
- Glassmorphism design - Custom CSS styling

**Backend:**
- Tauri v2 with features: `tray-icon`, `macos-private-api`, `image-png`

## Key Dependencies

**Frontend:**
- `@tauri-apps/api` ^2.10.1 - Tauri IPC client
- `@tauri-apps/plugin-global-shortcut` ^2.3.1 - Hotkey registration
- `@tauri-apps/plugin-dialog` ^2.6.0 - File dialogs
- `@tauri-apps/plugin-opener` ^2 - Open URLs/files
- `@tauri-apps/cli` ^2 - Build and dev tooling

**Backend (Core):**
- `sherpa-rs` 0.6 with `download-binaries` - SenseVoice speech recognition wrapper
- `cpal` 0.15 - Cross-platform audio capture
- `arboard` 3 - Clipboard access
- `llama-cpp-2` 0.1 with `sampler` - Local LLM inference (GGUF models)
- `ureq` 2 with `json` - HTTP client for online API calls

**Backend (Utilities):**
- `serde` 1 with `derive` - Serialization/deserialization
- `serde_json` 1 - JSON handling
- `anyhow` 1 - Error handling
- `dirs` 5 - Platform-specific directory paths
- `hound` 3.5 - WAV file I/O
- `log` 0.4 - Logging
- `chrono` 0.4 - Date/time utilities
- `encoding_rs` 0.8 - Character encoding conversion
- `image` 0.24 - Image processing (PNG support)
- `base64` 0.21 - Base64 encoding/decoding
- `once_cell` 1.19 - Lazy static initialization
- `libc` 0.2 - C library bindings

**macOS-specific:**
- `core-graphics` 0.24 - Graphics and window management
- `core-foundation` 0.10 - Foundation framework bindings
- `objc2` 0.6 - Objective-C runtime bindings
- `objc2-app-kit` 0.3 with `NSWindow` - AppKit bindings for window control
- `objc2-foundation` 0.3 - Foundation framework bindings
- `block2` 0.6 - Objective-C block support

**Windows-specific (in development):**
- `windows` 0.52 - Windows API bindings with features for keyboard, UI, graphics, OCR, WinRT

## Configuration

**Environment:**
- Configuration file: `~/Library/Application Support/com.mascribe/config.json` (macOS)
- Environment variables: Not used (config file based)

**Build:**
- `tsconfig.json` - TypeScript configuration (implicit, standard)
- `tauri.conf.json` - Tauri application configuration
- `vite.config.ts` - Vite build configuration (if present)

## Platform Requirements

**Development:**
- macOS 10.13+ (for Tauri v2 and NSPanel support)
- Xcode Command Line Tools (for Rust compilation)
- Node.js 16+ (for npm and Vite)

**Production:**
- macOS 10.13+ (primary target)
- Windows 10+ (in development)
- Apple Silicon and Intel support

**Model Requirements:**
- SenseVoice model: `model.int8.onnx` + `tokens.txt`
- Location: `~/Library/Application Support/com.mascribe/models/sensevoice/`
- Size: ~200MB (int8 quantized)
- Download: Manual via `scripts/install-sensevoice-model.sh`

## External Integrations

**Speech Recognition:**
- SenseVoice (via sherpa-onnx) - Local, offline transcription
- Supports: Chinese (Mandarin/Cantonese), English, Japanese, Korean

**AI Polishing:**
- Local: llama-cpp-2 with GGUF models (Qwen, Llama, etc.)
- Online: OpenAI-compatible API (Ollama, OpenAI, etc.)
- Endpoint: Configurable via `online_api_endpoint`

**System Integration:**
- Clipboard: arboard library
- Hotkey: Tauri global-shortcut plugin
- Audio: cpal (ALSA on Linux, CoreAudio on macOS, WASAPI on Windows)

## Build & Development Tools

**Development:**
```bash
npm run dev              # Vite dev server (frontend only)
npm run tauri dev       # Tauri dev mode (includes Vite + Rust backend)
npm run build           # TypeScript check + Vite build
npm run tauri build     # Production app bundle
```

**Output:**
- Frontend: `dist/` (Vite output)
- macOS app: `src-tauri/target/release/bundle/macos/MaScribe.app`
- Frameworks bundled: `libonnxruntime.1.17.1.dylib`, `libsherpa-onnx-c-api.dylib`, `libsherpa-onnx-cxx-api.dylib`

**Testing:**
- No automated test framework configured
- Manual testing required for all features

## Technical Constraints

**macOS-specific:**
- Fullscreen overlay uses NSPanel conversion via `object_setClass` at runtime
- Window level: NSScreenSaverWindowLevel (1000)
- Requires: Microphone, Accessibility, Input Monitoring, Screen Recording permissions

**Model Management:**
- SenseVoice model must be manually downloaded
- No automatic model updates
- Model caching: ~1-2s load time on first use

**Performance:**
- Transcription: ~50ms for 5s audio on Apple Silicon
- Audio buffer: 16kHz mono PCM, processed in chunks
- UI updates: Use requestAnimationFrame for smooth animations

---

*Stack analysis: 2026-05-03*
