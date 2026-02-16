# MaScribe

[![Release](https://img.shields.io/github/v/release/xiaozhenliu/mascribe?label=release)](https://github.com/xiaozhenliu/mascribe/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Platform: macOS](https://img.shields.io/badge/platform-macOS-blue.svg)](docs/macos-guide.md)

English | [中文](README.md)

MaScribe is a local voice-input app. Press a hotkey, speak, and your text is inserted at the current cursor position automatically.

## Features

- One-key voice input (default: `Alt+Space`)
- Multilingual transcription (including mixed Chinese/English)
- AI polishing (local model or online API)
- Screenshot OCR context for better correction

## Platform Support

- macOS: released and recommended
- Windows: in progress, no stable installer yet

## Install (macOS)

1. Download the latest `.dmg` from GitHub Releases
2. Drag `MaScribe.app` to `Applications`
3. Launch from `Applications`

## First Run (Required)

### 1) Prepare the local speech model

The current release does not auto-download the speech model. Transcription will fail if model files are missing.

If you are running from source (repo root):

```bash
./scripts/install-sensevoice-model.sh
```

If you only downloaded the DMG:

```bash
mkdir -p "$HOME/Library/Application Support/com.mascribe/models/sensevoice"
cd "$HOME/Library/Application Support/com.mascribe/models/sensevoice"
curl -L -o sensevoice.tar.bz2 \
  https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
tar -xjf sensevoice.tar.bz2
rm -f sensevoice.tar.bz2
```

Verify model files:

```bash
ls "$HOME/Library/Application Support/com.mascribe/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/model.int8.onnx" \
   "$HOME/Library/Application Support/com.mascribe/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/tokens.txt"
```

### 2) Grant required permissions

On first launch, allow:

- Microphone
- Accessibility (for auto-paste)
- Input Monitoring (for global hotkey)
- Screen Recording (only for screenshot/OCR features)

## Quick Start

1. Launch MaScribe
2. Press `Alt+Space` to start recording
3. Press again to stop
4. Text is inserted into the active input field

## Troubleshooting

### Auto-paste fails

Usually caused by missing Accessibility permission:

1. `System Settings -> Privacy & Security -> Accessibility`
2. Ensure `MaScribe` is enabled (remove and re-add if needed)
3. Restart MaScribe

Optional reset:

```bash
tccutil reset Accessibility com.mascribe
```

## Documentation

- macOS install and permissions: `docs/macos-guide.md`
- Local source build: `docs/local-build-guide.md`
- Online API config (Chinese): `docs/online-api-guide-zh.md`
- Online API config (English): `docs/online-api-guide-en.md`
- Product/design notes: `docs/PRD.md`

## Build From Source

```bash
git clone git@github.com:xiaozhenliu/mascribe.git
cd mascribe
npm install
npx tauri build
```

Build outputs:

- `src-tauri/target/release/bundle/macos/MaScribe.app`
- `src-tauri/target/release/bundle/dmg/MaScribe_*.dmg`

## License

MIT
