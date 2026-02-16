# Changelog

All notable changes to MaScribe are documented here.

## Unreleased

### Added
- **Native cross-platform OCR** — macOS: `VNRecognizeTextRequest` via Neural Engine (~0.6s); Windows: `Windows.Media.Ocr` WinRT API (~50-200ms, same engine as PowerToys Text Extractor). Both handle mixed Chinese/English/code text natively. Zero external dependencies
- **Screen OCR context** — Two-step pipeline: screenshot → OCR extracts screen text → injected into AI polishing prompt for homophone correction (e.g., "把" vs "八")
- **OCR settings** — Three modes: "macOS Built-in" (native, recommended), "Ollama OCR" (GLM-OCR), or "Disabled"
- **Launch at Login** — "Launch at Login" checkbox in tray menu via `tauri-plugin-autostart` (macOS Login Items)
- **Correction dictionary** — Settings UI for managing auto-replace rules (from → to), applied after transcription
- **Shortcut presets** — Dropdown for special keys (ContextMenu, F13–F15) that can't be captured via keydown
- **Sticky settings header** — Header stays visible while scrolling long settings page

### Changed
- **Polish prompt restructured** — Transcript and OCR context are now clearly labeled with `[TRANSCRIPT START/END]` and `[SCREEN CONTEXT]` markers, preventing models from regurgitating OCR content
- **Output length validation** — Online API responses exceeding 3× input length + 20 chars are rejected (falls back to raw transcript), preventing OCR content leakage
- **Settings window** — Now resizable with minimum size constraint; removed Cancel button (close window instead)
- **Vision → Screen OCR** — Renamed "Vision Model" settings to "Screen OCR"; removed local vision model option (stub), replaced with practical OCR-only pipeline
- **Hotkey event swallowing** — CGEventTap changed from `ListenOnly` to `Default` mode, returns `None` on match to prevent hotkey from leaking to other apps
- **Local polish context size** — Bumped llama-cpp context/batch from 1024/512 to 2048/2048
- **API error logging** — Online polisher now extracts and logs HTTP response body on errors (up to 300 chars) for easier debugging of model name / endpoint issues

### Fixed
- **Online polisher crash** — `#[serde(flatten)]` on `ChatMessage.content` caused panic when serializing `ChatContent::Text(String)` via ureq; replaced with regular field + explicit `serde_json::to_value()` serialization
- **Crash on local polish with OCR** — OCR context is now only injected for online API mode; local Qwen 2.5 model has ~512 token batch limit, extra context caused `GGML_ASSERT` crash
- **Floating window position** — Use `primary_monitor()` instead of `current_monitor()` (which returns None for hidden windows); add monitor position offset for multi-monitor support

### Known Issues
- **ContextMenu key triggers right-click in some apps** — On macOS, the ContextMenu key (mapped to F16) can trigger a right-click context menu in certain apps (e.g. Terminal). The hotkey itself works correctly, but the right-click side effect cannot be suppressed via CGEventTap as the event goes through a different system path. Other apps (browsers, editors) are unaffected.

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
