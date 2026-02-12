# Voice Input for macOS

Local speech-to-text tool for macOS. Press a hotkey, speak, and the transcribed text is automatically pasted into any app.

Powered by [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) (Alibaba) running locally via [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx). Optional AI text polishing via local [Qwen 2.5](https://huggingface.co/Qwen) model or any OpenAI-compatible API. No cloud required — all core features run entirely on your machine.

---

**macOS 本地语音输入工具。** 按下快捷键说话，识别结果自动输入到任意 App。基于阿里 SenseVoice 模型本地运行，可选 Qwen 2.5 本地模型或在线 API 进行 AI 润色。核心功能完全离线，数据不出机器。

## Features / 功能

- **Multi-language** — Chinese (Mandarin/Cantonese), English, Japanese, Korean with auto-detection
- **Fast** — ~50ms to transcribe 5 seconds of audio on Apple Silicon
- **Private** — Everything runs locally, no network required
- **AI polishing (dual-engine)** — Local Qwen 2.5 model or any OpenAI-compatible API for punctuation, grammar, and homophone correction
- **Screen OCR context** — Captures screenshot and extracts text via native OS OCR (macOS Vision framework / Windows.Media.Ocr) or Ollama GLM-OCR; injects screen text into the polishing prompt for accurate homophone correction
- **Mixed-language friendly** — Preserves Chinese-English code-switching as spoken
- **Universal paste** — Works in any macOS app (Chrome, VS Code, WeChat, etc.)
- **Configurable hotkey** — Set any key or combo; supports special keys like ContextMenu via presets
- **Correction dictionary** — Auto-fix common transcription errors with customizable JSON rules
- **Launch at Login** — Optional autostart via macOS Login Items, toggled from tray menu
- **Minimal UI** — Floating panel with ECG waveform indicator, or hide to menu bar only

---

- **多语言** — 中文（普通话/粤语）、英语、日语、韩语，自动检测
- **快速** — Apple Silicon 上 5 秒音频仅需约 50ms 转写
- **隐私** — 完全本地运行，无需网络
- **AI 润色（双引擎）** — 本地 Qwen 2.5 模型或在线 OpenAI 兼容 API，自动修正标点、语法和同音字
- **屏幕 OCR 上下文** — 截取当前窗口，通过系统原生 OCR（macOS Vision 框架 / Windows.Media.Ocr）或 Ollama GLM-OCR 提取屏幕文字，注入润色提示词中，精准纠正同音字
- **中英混合友好** — 保留说话时的中英混杂，不自动翻译
- **通用粘贴** — 适用于任何 macOS 应用
- **自定义快捷键** — 支持任意单键或组合键，特殊键（如 ContextMenu）可通过预设选择
- **纠错词典** — 自定义 JSON 规则自动修正常见识别错误
- **开机启动** — 可选登录时自动启动，在托盘菜单中切换
- **极简 UI** — 悬浮面板 + ECG 波形动画，或仅菜单栏图标

## Requirements / 系统要求

- macOS 12+ (**Apple Silicon required** — sherpa-onnx runtime libraries are arm64 only)
- ~1.2 GB RAM for SenseVoice model
- ~1.1 GB additional RAM if using local AI polishing (Qwen 2.5)
- Microphone permission
- Accessibility permission (for Cmd+V simulation)
- Input Monitoring permission (for global hotkey capture)

## Prerequisites / 前置要求

### For building from source / 从源码构建的前置条件

| Dependency | Install command | Purpose |
|------------|----------------|---------|
| **Rust toolchain** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | Compile Rust backend |
| **Node.js 18+** | `brew install node` or [nodejs.org](https://nodejs.org/) | Build frontend & run Tauri CLI |
| **CMake** | `brew install cmake` | Compile llama.cpp (for AI polishing) & sherpa-onnx |
| **Xcode CLI Tools** | `xcode-select --install` | C/C++ compiler, Metal framework |

### Models / 模型下载

#### 1. SenseVoice (Required / 必须)

Speech recognition model (~228 MB). App will not start without it.

```bash
mkdir -p ~/.openclaw/models/sensevoice
cd ~/.openclaw/models/sensevoice

# Download from HuggingFace / 从 HuggingFace 下载
git lfs install
git clone https://huggingface.co/csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

Expected path: `~/.openclaw/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/`

#### 2. Qwen 2.5 1.5B Instruct (Optional / 可选 — for local AI polishing)

Text polishing model (~1.1 GB GGUF, Q4_K_M quantization). If not present, local polishing is unavailable (you can still use Online API mode).

```bash
mkdir -p ~/.openclaw/models/qwen2.5-1.5b

# Using huggingface-cli
pip install huggingface-hub
huggingface-cli download Qwen/Qwen2.5-1.5B-Instruct-GGUF \
    qwen2.5-1.5b-instruct-q4_k_m.gguf \
    --local-dir ~/.openclaw/models/qwen2.5-1.5b
```

Expected path: `~/.openclaw/models/qwen2.5-1.5b/qwen2.5-1.5b-instruct-q4_k_m.gguf`

> **Tip / 提示:** If you prefer not to download the local model, you can use the **Online API** mode instead — configure any OpenAI-compatible endpoint (Step-Fun, DeepSeek, Groq, etc.) in Settings.

#### 3. Screen OCR (Optional / 可选 — for screen context)

Screen OCR extracts text from your current window and injects it into the AI polishing prompt, helping correct homophones based on context (e.g., "把" vs "八").

屏幕 OCR 从当前窗口提取可见文字，注入 AI 润色提示词中作为上下文，帮助润色模型准确区分同音字。

**Option A: macOS Built-in (Recommended / 推荐)**

Uses the macOS Vision framework (`VNRecognizeTextRequest`) via the Neural Engine. ~0.6s per screenshot, zero setup, no extra downloads.

使用 macOS Vision 框架，利用 Neural Engine 加速。每次截图仅需约 0.6 秒，无需任何额外安装。

In Settings, select **Screen OCR → macOS Built-in**. That's it.

**Option B: GLM-OCR via Ollama**

Alternative OCR using GLM-OCR model (~2.2 GB). Slower (~5–7s) but works on non-Apple-Silicon Macs or for custom models. Requires [Ollama](https://ollama.com/).

```bash
brew install ollama       # Install Ollama
ollama serve              # Start service
ollama pull glm-ocr       # Download model (~2.2 GB)
```

In Settings, select **Screen OCR → Ollama OCR**, then configure endpoint (`http://localhost:11434/v1`) and model (`glm-ocr`).

**How it works / 工作原理:**

```
Screenshot → OCR (native or Ollama) → screen context
                                           ↓
Voice → SenseVoice → corrections → AI polish + screen context → corrected text
```

The OCR extracts visible text from your current window. That text is injected into the AI polishing prompt as context, allowing the polishing model to correctly disambiguate homophones (e.g., knowing "handle" is on screen helps pick "把" vs "八").

OCR 从当前窗口提取可见文字，注入 AI 润色提示词中作为上下文，帮助润色模型准确区分同音字（例如，看到屏幕上有 "handle" 就能正确选择 "把" 而不是 "八"）。

> **Note / 注意:** OCR context is only used with the **Online API** polishing mode. The local Qwen 2.5 model has limited context capacity (~512 tokens) and cannot process extra screen text.
> OCR 上下文仅在「在线 API」润色模式下使用。本地 Qwen 2.5 模型上下文容量有限，无法处理额外的屏幕文字。

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
2. Grant Microphone, Accessibility, and Input Monitoring permissions when prompted
3. Press **Alt+Space** (default) to start recording
4. Speak in any supported language
5. Press again to stop — text is automatically pasted at your cursor

---

1. 启动 Voice Input — 仅在菜单栏显示图标
2. 授予麦克风、辅助功能和输入监控权限
3. 按 **Alt+Space**（默认）开始录音
4. 用任何支持的语言说话
5. 再按一次停止 — 文字自动粘贴到光标位置

## Tech Stack / 技术栈

| Component | Technology |
|-----------|-----------|
| Framework | Tauri 2.0 (Rust + Web) |
| Speech Model | SenseVoice via sherpa-rs |
| AI Polishing (local) | Qwen 2.5 1.5B Instruct via llama-cpp-2 |
| AI Polishing (online) | OpenAI-compatible API via ureq |
| Screen OCR | Native OS OCR (macOS Vision / Windows.Media.Ocr) or GLM-OCR via Ollama |
| Audio | cpal (CoreAudio) |
| Hotkey | CGEventTap (native) + tauri-plugin-global-shortcut |
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
  polishing/engine.rs    AI text polishing (Qwen 2.5 local, llama-cpp-2)
  polishing/online.rs    AI text polishing (OpenAI-compatible API)
  insertion/clipboard.rs Clipboard paste simulation
  correction/dictionary.rs  Text correction rules
  hotkey/mod.rs          CGEventTap native key listener
  ocr/                   Native OCR (macOS Vision framework)
  permissions.rs         macOS TCC permission helpers
  tray.rs                Menu bar tray icon

src/                     Web frontend
  index.html             Floating panel UI
  settings.html          Settings window
  main.ts                Frontend logic
  settings.ts            Settings logic
  settings.css           Settings styles
  styles/                Panel CSS
```

## Permissions / 权限说明

| Permission | Why / 用途 |
|-----------|-----------|
| Microphone | Record audio / 录制音频 |
| Accessibility | Simulate Cmd+V paste / 模拟粘贴按键 |
| Input Monitoring | System-level hotkey capture via CGEventTap / 全局快捷键捕获 |
| Screen Recording | Capture screenshots for OCR context (optional) / 截图用于 OCR 上下文（可选） |

## License

MIT
