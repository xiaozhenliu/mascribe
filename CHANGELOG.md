# Changelog

All notable changes to Voice Input are documented here.

## Unreleased

### Added
- **Screen OCR context** — Two-step pipeline: screenshot → GLM-OCR (via Ollama) extracts screen text → injected into AI polishing prompt for homophone correction (e.g., "把" vs "八")
- **OCR settings** — Configurable OCR endpoint (default: `http://localhost:11434/v1`) and model (default: `glm-ocr`) in Settings
- **Launch at Login** — "Launch at Login" checkbox in tray menu via `tauri-plugin-autostart` (macOS Login Items)
- **Correction dictionary** — Settings UI for managing auto-replace rules (from → to), applied after transcription
- **Shortcut presets** — Dropdown for special keys (ContextMenu, F13–F15) that can't be captured via keydown
- **Sticky settings header** — Header stays visible while scrolling long settings page

### Changed
- **Polish prompt** — Improved mixed Chinese/English handling: no longer translates between languages
- **Settings window** — Now resizable with minimum size constraint; removed Cancel button (close window instead)
- **Vision → Screen OCR** — Renamed "Vision Model" settings to "Screen OCR"; removed local vision model option (stub), replaced with practical OCR-only pipeline
- **Hotkey event swallowing** — CGEventTap changed from `ListenOnly` to `Default` mode, returns `None` on match to prevent ContextMenu key from leaking to other apps

### Fixed
- **Crash on local polish with OCR** — OCR context is now only injected for online API mode; local Qwen 2.5 model has ~512 token batch limit, extra context caused `GGML_ASSERT` crash
- **Floating window position** — Use `primary_monitor()` instead of `current_monitor()` (which returns None for hidden windows); add monitor position offset for multi-monitor support

## 0.2.0

### Added
- **Dual-engine AI polishing** — Local Qwen 2.5 1.5B Instruct (GGUF) or any OpenAI-compatible API
- **Configurable hotkey** — Key recorder widget in Settings; supports single keys and combos
- **CGEventTap native hotkey** — System-level key capture for keys unsupported by global-shortcut plugin
- **Menu bar tray** — App lives in menu bar only (no Dock icon), with Settings and Quit
- **ECG waveform** — Animated recording indicator with 16 bouncing bars
- **WAV recording save** — Each recording saved to configurable directory
- **Correction dictionary backend** — JSON-based text replacement engine (longest-match-first, case-insensitive)

### Changed
- Switched from Gemma 3 to Qwen 2.5 for local AI polishing (much better Chinese support)
- Redesigned floating panel: compact pill shape with mic+T icon

## 0.1.0

### Added
- Initial release
- SenseVoice speech-to-text via sherpa-onnx (local, offline)
- Multi-language support: Chinese, English, Japanese, Korean, Cantonese
- 16kHz resampling for SenseVoice compatibility
- Clipboard-based text insertion with Cmd+V simulation
- Transparent floating panel UI
