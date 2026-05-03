# Changelog

All notable changes to MaScribe are documented here.

## Unreleased

## 0.4.2 - 2026-05-03

### Added
- About menu item in tray menu displaying app version, Git branch, and commit hash

## 0.4.1 - 2026-05-03

### Changed
- Added separator before "Quit" menu item in tray menu to prevent accidental app closure

## 0.4.0 - 2026-02-17

### Added
- **Fullscreen overlay support** — Floating pill window now appears above full-screen apps (AFFiNE, Chrome, VS Code, etc.)
  - Converts NSWindow → NSPanel at runtime via `object_setClass` for true panel behavior
  - Uses `orderFrontRegardless` instead of `makeKeyAndOrderFront:` to avoid Space switching
  - Window level set to `NSScreenSaverWindowLevel` (1000) to float above fullscreen windows
  - Collection behaviors: `CanJoinAllSpaces`, `FullScreenAuxiliary`, `Transient`, `Stationary`
- **Paste target tracking** — Records the frontmost app before recording starts, re-activates it before Cmd+V paste to ensure text goes to the correct app even with overlay visible
- **On-demand accessibility permission prompt** — If Accessibility permission is missing at paste time, actively prompts the user instead of silently failing

### Changed
- `show_window` bypasses Tauri's `window.show()` (which calls `makeKeyAndOrderFront:` causing Space switches) in favor of native `orderFrontRegardless` via ObjC
- Fullscreen overlay flags applied both at app startup and on each `show_window` call for resilience
- Added `objc2-app-kit` dependency for type-safe NSWindow/NSPanel/NSRunningApplication APIs

### Fixed
- Window not visible on fullscreen Spaces due to Tauri's `set_visible_on_all_workspaces()` and `set_always_on_top()` overwriting raw ObjC collection behavior and window level
- Window positioned off-screen (Y=-540) when using `setFrameTopLeftPoint` with physical pixel coordinates instead of AppKit's bottom-left-origin logical points

## 0.3.2 - 2026-02-16

### Added
- Settings now supports selecting a local GGUF path for local polishing mode.
- Added `Detect Ollama Models` in settings to auto-discover local Ollama models from `/api/tags`.
- Added model suggestions dropdown for API model input based on detected Ollama models.

### Changed
- Improved settings guidance for local GGUF vs Ollama API workflows (bilingual).
- Updated README and API docs with explicit local-model and Ollama setup instructions.
- Clarified Windows model setup doc to match runtime config paths and required files.

## 0.3.1 - 2026-02-16

### Changed
- Model storage switched to macOS-native app data path: `~/Library/Application Support/com.mascribe/models`.
- Removed release-side legacy OpenClaw model-path migration logic to keep first public package behavior explicit.
- Updated model install script and guides to use the native path and executable copy-paste commands for DMG users.

## 0.3.0 - 2026-02-16

### Added
- Native cross-platform OCR support (`macOS Vision` / `Windows.Media.Ocr`) and screenshot context injection into online AI polishing.
- Launch-at-login toggle in tray menu.
- Correction dictionary UI and save/load support in Settings.
- Special hotkey presets (`ContextMenu`, `F13`–`F15`) with native fallback listener.
- Security secret scanning workflow with `gitleaks` (`npm run security:secrets`).
- User-facing platform guides: `docs/macos-guide.md` and streamlined Windows guide.

### Changed
- Rebranded product from Voice Input to **MaScribe** (app name, bundle id, paths, docs).
- Unified app identifier to `com.mascribe` and updated config/data paths accordingly.
- Settings UI upgraded to clean bilingual mode (`中文 / English`) with language switching.
- Screen OCR options and hints are now platform-aware (`macOS Built-in` / `Windows Built-in`).
- Windows setup documentation simplified to user-first install + permission flow.
- README simplified to reduce mixed-language clutter and unnecessary technical details.

### Fixed
- macOS paste failure diagnostics improved with explicit accessibility/fallback logging.
- Online polisher serialization crash fixed (`ChatContent` request serialization).
- OCR context injection restricted to online API polishing to avoid local model context overflow.
- Floating window positioning made stable across hidden window / multi-monitor scenarios.

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
