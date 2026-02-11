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

## Key Paths

- **Model**: `~/.openclaw/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/`
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

## Architecture Notes

- Single-process Rust app (no Python sidecar, no IPC)
- SenseVoice model loaded once at startup, stays in memory (~300MB)
- Hotkey is fully configurable via a key recorder widget in settings
- Text insertion uses clipboard save/restore pattern to not destroy user's clipboard
- App runs as Accessory (no Dock icon, menu bar only)
