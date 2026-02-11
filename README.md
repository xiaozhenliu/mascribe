# Voice Input for macOS

Local speech-to-text tool for macOS. Press a hotkey, speak, and the transcribed text is automatically pasted into any app.

Powered by [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) (Alibaba) running locally via [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx). Optional AI text polishing via [Gemma 3](https://ai.google.dev/gemma) (Google) with local llama.cpp inference. No cloud, no API keys, no data leaves your machine.

---

**macOS 本地语音输入工具。** 按下快捷键说话，识别结果自动输入到任意 App。基于阿里 SenseVoice 模型本地运行，可选 Gemma 3 AI 润色。完全离线，数据不出机器。

## Features / 功能

- **Multi-language** — Chinese (Mandarin/Cantonese), English, Japanese, Korean with auto-detection
- **Fast** — ~50ms to transcribe 5 seconds of audio on Apple Silicon
- **Private** — Everything runs locally, no network required
- **AI polishing** — Optional LLM-based text post-processing for better punctuation & grammar
- **Universal paste** — Works in any macOS app (Chrome, VS Code, WeChat, etc.)
- **Configurable hotkey** — Set any key or combo via a key recorder widget
- **Correction dictionary** — Auto-fix common transcription errors with customizable rules
- **Minimal UI** — Floating panel with status indicator, or hide to menu bar only

---

- **多语言** — 中文（普通话/粤语）、英语、日语、韩语，自动检测
- **快速** — Apple Silicon 上 5 秒音频仅需约 50ms 转写
- **隐私** — 完全本地运行，无需网络
- **AI 润色** — 可选的 LLM 文本后处理，自动修正标点和语法
- **通用粘贴** — 适用于任何 macOS 应用
- **自定义快捷键** — 支持任意单键或组合键
- **纠错词典** — 自定义规则自动修正常见识别错误
- **极简 UI** — 悬浮面板或仅菜单栏图标

## Requirements / 系统要求

- macOS 12+ (Apple Silicon recommended, Intel also supported)
- ~1.2 GB RAM for SenseVoice model
- ~0.75 GB additional RAM if using AI polishing (Gemma 3)
- Microphone permission
- Accessibility permission (for Cmd+V simulation)

## Prerequisites / 前置要求

### For building from source / 从源码构建的前置条件

| Dependency | Install command | Purpose |
|------------|----------------|---------|
| **Rust toolchain** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | Compile Rust backend |
| **Node.js 18+** | `brew install node` or [nodejs.org](https://nodejs.org/) | Build frontend & run Tauri CLI |
| **CMake** | `brew install cmake` | Compile llama.cpp (for AI polishing) & sherpa-onnx |
| **Xcode CLI Tools** | `xcode-select --install` | C/C++ compiler, Metal framework |

### Models / 模型下载

#### SenseVoice (Required / 必须)

The speech recognition model (~228 MB). App will not start without it.

```bash
mkdir -p ~/.openclaw/models/sensevoice
cd ~/.openclaw/models/sensevoice

# Download from HuggingFace / 从 HuggingFace 下载
git lfs install
git clone https://huggingface.co/csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

Expected path: `~/.openclaw/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/`

#### Gemma 3 1B (Optional / 可选 — for AI polishing)

The text polishing model (~1 GB GGUF). If not present, polishing is silently skipped.

```bash
mkdir -p ~/.openclaw/models/gemma3

# Option 1: Using huggingface-cli
pip install huggingface-hub
huggingface-cli download google/gemma-3-1b-it-qat-q4_0-gguf \
    --local-dir ~/.openclaw/models/gemma3

# Option 2: Manual download from https://huggingface.co/google/gemma-3-1b-it-qat-q4_0-gguf
```

> **Note / 注意:** Gemma 3 requires accepting Google's license on HuggingFace before downloading.

Expected path: `~/.openclaw/models/gemma3/gemma-3-1b-it-q4_0.gguf`

## Quick Start / 快速开始

### Download / 下载

Download the latest `.dmg` from [Releases](../../releases) and drag to Applications.

### Build from Source / 从源码构建

```bash
# 1. Install prerequisites (see table above)
# 1. 安装前置条件（见上方表格）

# 2. Clone and build / 克隆并构建
git clone https://github.com/user/mac-voice-input.git
cd mac-voice-input
npm install
npx tauri build
```

The built app will be in `src-tauri/target/release/bundle/macos/`.

## Usage / 使用方法

1. Launch Voice Input — it appears as a menu bar icon (no Dock icon)
2. Grant Microphone and Accessibility permissions when prompted
3. Press **Option+Space** (default) to start recording
4. Speak in any supported language
5. Press again to stop — text is automatically pasted at your cursor

---

1. 启动 Voice Input — 仅在菜单栏显示图标
2. 授予麦克风和辅助功能权限
3. 按 **Option+Space**（默认）开始录音
4. 用任何支持的语言说话
5. 再按一次停止 — 文字自动粘贴到光标位置

## Tech Stack / 技术栈

| Component | Technology |
|-----------|-----------|
| Framework | Tauri 2.0 (Rust + Web) |
| Speech Model | SenseVoice via sherpa-rs |
| AI Polishing | Gemma 3 1B via llama-cpp-2 (optional) |
| Audio | cpal (CoreAudio) |
| Hotkey | tauri-plugin-global-shortcut |
| Text Insertion | Clipboard + CGEvent Cmd+V |
| Frontend | TypeScript + Vite (vanilla) |

## Development / 开发

```bash
npm install              # Install frontend dependencies
npx tauri dev            # Run in dev mode (hot-reload)
npx tauri build          # Build for distribution
```

### Project Structure / 项目结构

```
src-tauri/src/           Rust backend
  lib.rs                 App entry point and setup
  commands.rs            Tauri IPC commands (pipeline orchestration)
  state.rs               Shared state (AppState)
  config.rs              App configuration
  audio/capture.rs       Microphone recording
  recognition/engine.rs  SenseVoice transcription
  polishing/engine.rs    AI text polishing (Gemma 3 via llama.cpp)
  insertion/clipboard.rs Clipboard paste simulation
  correction/dictionary.rs  Text correction rules
  permissions.rs         macOS TCC permission helpers
  tray.rs                Menu bar tray icon

src/                     Web frontend
  index.html             Floating panel UI
  main.ts                Frontend logic
  styles/                CSS
```

## Permissions / 权限说明

| Permission | Why / 用途 |
|-----------|-----------|
| Microphone | Record audio / 录制音频 |
| Accessibility | Simulate Cmd+V paste / 模拟粘贴按键 |
| Input Monitoring | System-level hotkey capture / 全局快捷键捕获 |

## License

MIT
