# MaScribe Project

**Version:** v0.4.0  
**Type:** Desktop Application (macOS)  
**Status:** Active Development

## Overview

MaScribe（马上听写）is a local voice-to-text input tool for macOS. Users press a hotkey to speak, and the transcribed text is automatically inserted at the current cursor position. All processing happens locally by default, ensuring privacy and offline capability.

## Core Value Proposition

- **One-key voice input**: Press hotkey (default: Alt+Space) to record, release to transcribe
- **Local-first**: Speech recognition runs entirely on-device using SenseVoice model
- **Multi-language support**: Chinese (Mandarin/Cantonese), English, Japanese, Korean
- **AI polishing**: Optional text refinement using local or online LLM
- **Fullscreen compatibility**: Overlay window works above fullscreen apps (v0.4.0+)

## Tech Stack

### Frontend
- **Framework**: Vanilla TypeScript + Vite 6.0.3
- **UI**: Custom HTML/CSS with glassmorphism design
- **Build**: Vite for fast development and optimized production builds

### Backend (Tauri)
- **Framework**: Tauri v2 (Rust)
- **Speech Recognition**: sherpa-onnx with SenseVoice model (local, offline)
- **Audio Capture**: cpal for cross-platform audio input
- **AI Polishing**: 
  - Local: llama-cpp-2 (GGUF models)
  - Online: OpenAI-compatible API (supports Ollama, DeepSeek, etc.)
- **Clipboard**: arboard for cross-platform clipboard access
- **Platform-specific**:
  - macOS: core-graphics, objc2, NSPanel for fullscreen overlay
  - Windows: Windows API (in development)

## Architecture

### Key Components

```
src/
├── main.ts          # Frontend entry, UI logic
└── settings.ts      # Settings page logic

src-tauri/src/
├── lib.rs           # Tauri app setup, command registration
├── commands.rs      # Tauri commands (transcribe, polish, etc.)
├── config.rs        # Configuration management
├── state.rs         # Application state
└── tray.rs          # System tray icon
```

### Critical Implementation Details

#### Fullscreen Overlay (v0.4.0)
- Uses NSPanel conversion via `object_setClass` at runtime
- Window level: NSScreenSaverWindowLevel (1000)
- Collection behavior: `FullScreenAuxiliary` + `CanJoinAllSpaces`
- Uses `orderFrontRegardless` to avoid Space switching

#### Auto-paste Mechanism
- Writes text to clipboard
- Simulates Cmd+V using CGEvent (requires Accessibility permission)
- Fallback to AppleScript if CGEvent fails
- Window is `focusable: false` to preserve target app focus

#### Model Management
- Models stored in: `~/Library/Application Support/com.mascribe/models/sensevoice/`
- Required files: `model.int8.onnx`, `tokens.txt`
- Download script: `scripts/install-sensevoice-model.sh`

## Current State (v0.4.0)

### Completed Features
- ✅ Voice recording with hotkey trigger
- ✅ Local speech-to-text using SenseVoice
- ✅ Multi-language support (Chinese, English, Japanese, Korean)
- ✅ AI polishing (local and online modes)
- ✅ Screenshot OCR for context-aware correction
- ✅ Fullscreen app overlay support
- ✅ System tray integration
- ✅ Settings UI with model detection

### Platform Support
- **macOS**: Production-ready, released
- **Windows**: In development, not yet released

### Known Limitations
1. Model download not automatic (users must run script manually)
2. No auto-update mechanism
3. Windows support incomplete
4. No automated tests

## Development Workflow

### Setup
```bash
npm install
./scripts/install-sensevoice-model.sh
```

### Development
```bash
npm run tauri dev    # Run in dev mode
```

### Build
```bash
npm run tauri build  # Build production app
# Output: src-tauri/target/release/bundle/macos/MaScribe.app
```

## Configuration

### User Settings Location
- macOS: `~/Library/Application Support/com.mascribe/config.json`
- Windows: `%APPDATA%\com.mascribe\config.json`

### Settings Structure
```json
{
  "hotkey": "RAlt",
  "model_path": "~/Library/Application Support/com.mascribe/models/sensevoice/...",
  "language": "auto",
  "ai_engine": "none|local|online",
  "local_model_path": "/path/to/model.gguf",
  "online_api_endpoint": "http://localhost:11434/v1",
  "online_api_key": "",
  "online_model_name": "qwen2.5:1.5b"
}
```

## Project Management

- **Issue Tracking**: Linear
- **Project**: MaScribe
- **Team**: Growthrocketstudio
- **Repository**: https://github.com/xiaozhenliu/mascribe

## Security & Privacy

- All processing is local by default
- Online API mode only used if explicitly configured
- No telemetry or data collection
- Signing identity: "xzliu Dev" (for macOS notarization)

## Documentation

- `README.md`: User guide (Chinese)
- `README.en.md`: User guide (English)
- `docs/PRD.md`: Product requirements
- `docs/macos-guide.md`: macOS installation & permissions
- `docs/local-build-guide.md`: Build instructions
- `docs/online-api-guide-zh.md`: Online API setup (Chinese)
- `docs/online-api-guide-en.md`: Online API setup (English)
- `CLAUDE.md`: Codebase context for AI assistants
