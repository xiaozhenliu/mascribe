# MaScribe — 产品需求文档 (PRD)

## 概述

**产品名称**: MaScribe（马上听写） (mascribe)

**一句话描述**: macOS 桌面应用，按住快捷键说话，本地语音转文字后自动输入到任意 App。

**目标用户**: 需要在 macOS 上快速进行语音输入的用户，尤其是中英文混合输入场景。

---

## 背景与动机

macOS 自带的语音听写功能依赖网络且准确率有限，特别是对中英混合输入支持不佳。阿里的 SenseVoice 模型在多语言语音识别上表现优异，且可完全本地运行，保障隐私。

本项目旨在将 SenseVoice 模型的能力封装为一个易用的桌面工具，实现"按键说话 → 文字自动输入"的流畅体验。

---

## 核心功能

### 1. 语音录制 (Hold-to-Talk)

- 按住用户自定义的快捷键开始录音
- 松开快捷键结束录音
- 录音期间显示实时波形动画
- 支持的音频格式：16kHz 单声道 PCM（内部自动转换）

### 2. 语音转文字

- 使用本地 SenseVoice 模型（sherpa-onnx ONNX 格式，int8 量化）
- 支持语言：中文（普通话/粤语）、英语、日语、韩语
- 自动语言检测，无需手动切换
- 推理速度：约 50ms 处理 5 秒音频（Apple Silicon CPU）
- 支持逆文本规范化（ITN），如"一百二十三"→"123"

### 3. 文字插入

- 将识别结果自动输入到当前获得焦点的 App
- 实现方式：写入识别文字到剪贴板 → 模拟 Cmd+V 粘贴
- 文字保留在剪贴板中，用户可随时 Cmd+V 重新粘贴
- Cmd+V 模拟策略：
  - 优先使用 CGEvent（需 Accessibility 权限）
  - 无权限时自动回退到 AppleScript（`osascript` 调用 System Events）
- 当系统拒绝按键注入（例如 TCC 未授权）时，必须在 UI 中明确提示“自动粘贴失败，请检查辅助功能权限”，并保留结果到剪贴板
- 悬浮窗口设为 `focusable: false`，确保粘贴目标为之前活跃的 App
- 兼容几乎所有 macOS 应用（包括 Chrome、VS Code、微信等）

### 4. 悬浮面板（按需显示）

- 小型药丸形悬浮窗口（220×48px），始终置顶
- 无边框、半透明毛玻璃背景、不可聚焦
- **默认隐藏**，仅在以下时机显示：
  - 按下快捷键开始录音时自动出现
  - 录音 → 处理 → 显示结果/错误后自动隐藏（约 2.5 秒后）
- 显示内容：
  - 状态指示灯（空闲绿/录音红/处理橙 + 脉冲动画）
  - 实时音频波形（原地跳动竖条，16 条，中间高两侧低的钟形分布）
  - 识别结果或错误信息（简短显示后隐藏）
- 位于屏幕底部居中，Dock 上方约 90px
- 可拖拽移动
- 全屏应用场景：
  - 当前版本以“稳定不崩溃”为优先，允许在部分全屏 App 中存在显示限制
  - vNext 目标：在独立 Space 的全屏应用中保持可见

### 5. 菜单栏图标

- 在 macOS 菜单栏显示应用图标（template icon，适配明暗模式）
- 右键菜单：
  - 退出应用
  - （v2：显示/隐藏悬浮面板、打开设置）
- 应用不在 Dock 中显示（Accessory activation policy）
- 菜单栏图标为应用的唯一常驻 UI 入口

### 6. 设置界面

独立的设置窗口，包含以下配置项：

#### 快捷键设置
- **按键录制器组件**：用户点击输入框后按下任意键即可设置
- 支持单键（Fn、Right Option、Right Command 等）和组合键（Cmd+Shift+Space 等）
- 显示实际按键名称，适配不同键盘布局（兼容 Windows 键盘）
- 默认快捷键：Right Option（在 Windows 键盘上对应右 Alt）

#### 模型设置
- 模型文件目录路径（带浏览按钮）
- 显示当前模型信息（名称、大小、支持语言）
- 推理线程数配置（默认 4）

#### 语言设置
- 语言偏好选择：自动检测 / 指定语言
- 语言列表：中文、英语、日语、韩语、粤语

#### 纠错词典
- 以表格形式展示纠错规则
- 支持添加/删除规则
- 内置常用纠错（如 "cloud"→"Claude", "open ai"→"OpenAI"）

#### 高级设置
- AI 后处理开关（v2，暂时灰置显示"即将推出"）
- 调试模式开关

### 7. 录音保存与重试

- 每次录音结束后，先将原始音频保存为 WAV 文件，再进行转写
- WAV 文件保存位置：`~/Library/Application Support/com.mascribe/recordings/`
- 文件命名格式：`recording-YYYYMMDD-HHmmss.wav`（时间戳精确到秒）
- 保存原始采样率（48kHz），未经重采样的完整音频
- 目的：转写出错时可重试，也可用于后续分析和调试
- 自动清理策略（v2）：保留最近 N 天或最大 M 份录音

### 8. 纠错词典

- 基于简单字符串替换的纠错系统
- JSON 格式存储，可手动编辑
- 支持通过设置界面管理
- 在语音识别后、文字插入前应用

### 9. AI 润色（双引擎）

- 对转写结果进行智能纠错和润色
- **本地模式**: Qwen 2.5 1.5B Instruct (GGUF, Q4_K_M) via llama-cpp-2
  - 自动检测 ChatML/Gemma 模板格式
  - ~512 token 上下文限制
- **在线 API 模式**: 任何 OpenAI 兼容接口 (DeepSeek, Step-Fun, Groq 等)
  - 10s 读/5s 写超时
  - HTTP 错误时记录响应体用于诊断
- **Prompt 模板**: 支持 `{text}` 和 `{lang}` 占位符
- **输出验证**: 拒绝空输出、超长输出（>3x 输入）、语言切换

### 10. 屏幕 OCR 上下文

截取当前屏幕内容，通过 OCR 提取可见文字，注入 AI 润色提示词中，帮助模型准确纠正同音字。

#### 使用场景

用户的屏幕上可能包含：
- 代码编辑器中的中英文混合代码和注释
- 浏览器中的网页内容（中文文章、英文文档）
- 聊天窗口中的对话内容
- 终端中的命令输出

这些屏幕内容提供了语境，帮助 AI 润色模型区分同音字（如"把"vs"八"、"是"vs"事"）。

#### 工作流程

```
截图 → OCR (原生/Ollama) → 屏幕文字（截断至 500 字符）
                                    ↓
语音 → SenseVoice → 纠错词典 → AI 润色 + [SCREEN CONTEXT] → 粘贴输出
```

#### OCR 引擎（跨平台原生优先）

| 平台 | 原生 OCR | 技术 | 速度 |
|------|---------|------|------|
| macOS | VNRecognizeTextRequest | Vision 框架 (Neural Engine) | ~0.6s |
| Windows | Windows.Media.Ocr | OcrEngine (WinRT) | ~50-200ms |
| Fallback | Ollama GLM-OCR | HTTP API (需要 Ollama 运行) | ~5-7s |

- **原生 OCR 优先**：零依赖、零配置、速度快
- **macOS**: `VNRecognizeTextRequest`，支持 zh-Hans/zh-Hant/en-US/ja-JP/ko-KR
- **Windows**: `Windows.Media.Ocr.OcrEngine`，需要对应语言包已安装（中文简体需 `Language.OCR~~~zh-Hans~0.0.1.0`）
- **Ollama fallback**: 适用于自定义模型或原生 OCR 不可用的场景

#### 中英文混合与代码支持

OCR 引擎以 `zh-Hans`（中文简体）初始化时，**天然支持** ASCII 字母、数字和符号的识别。这意味着：
- 代码编辑器截图中的函数名、变量名、运算符 → 正常识别
- 中文注释 + 英文代码混合 → 正常识别
- 网页中的中英文混排 → 正常识别

不需要多次调用不同语言引擎 — 单次 `zh-Hans` OCR 即可覆盖中英混合场景。

#### 约束

- OCR 上下文仅在「在线 API」润色模式下注入（本地 Qwen 2.5 上下文容量不足）
- 输出长度验证：润色结果超过输入 3 倍 + 20 字符时拒绝（防止 OCR 内容泄漏）
- 截图需缩放至 `screenshot_max_size`（默认 1024px）以控制处理时间
- Windows OCR 最大图片尺寸 2560px（超过需缩小）

#### 配置项

| 字段 | 值 | 说明 |
|------|---|------|
| `vision_mode` | "native" / "api" / "disabled" | OCR 模式 |
| `screenshot_mode` | "disabled" / 其他 | 截图开关 |
| `screenshot_max_size` | 1024 | 截图最大边长(px) |
| `ocr_endpoint` | URL | Ollama API 地址 |
| `ocr_model` | 模型名 | Ollama 模型 |

---

## 快捷键系统详细设计

### 捕获机制

采用 macOS CGEventTap 实现系统级按键捕获：

- **CGEventTap** (kCGHIDEventTap): 在硬件层面监听按键事件
- **监听模式** (kCGEventTapOptionListenOnly): 只监听不消费事件
- 监听 `kCGEventFlagsChanged`（修饰键）和 `kCGEventKeyDown`/`kCGEventKeyUp`（普通键）

### 支持的按键类型

| 按键类型 | 示例 | macOS keycode |
|----------|------|---------------|
| 功能键 | Fn | 63 |
| 修饰键 | Left/Right Option, Left/Right Command, Left/Right Control, Left/Right Shift | 各不相同 |
| 普通键 | 空格、字母、数字 | 标准 keycode |
| 组合键 | Cmd+Shift+Space | 修饰符 + keycode |

### Windows 键盘映射

| Windows 键 | macOS 映射 | 说明 |
|-----------|----------|------|
| Win (左/右) | Command (左/右) | 系统默认映射 |
| Alt (左/右) | Option (左/右) | 系统默认映射 |
| Ctrl (左/右) | Control (左/右) | 相同 |

### Hold-to-Talk 行为

```
按键按下 (key down)
  → 延迟 100ms（防抖，避免误触）
  → 开始录音
  → UI 切换到"录音中"状态

按键松开 (key up)
  → 停止录音
  → 如果录音时长 < 300ms，忽略（太短，可能是误触）
  → 如果录音时长 >= 300ms，执行转写流程
  → UI 切换到"处理中"状态
```

---

## 技术架构

### 整体架构

```
┌─────────────────────────────────────┐
│  Tauri 2.0 Desktop App              │
│                                     │
│  ┌───────────────────────────────┐  │
│  │ Web Frontend (TS + Vite)      │  │
│  │ - 悬浮面板 (index.html)       │  │
│  │ - 设置窗口 (settings.html)    │  │
│  │ - 波形可视化 (Canvas)         │  │
│  │ - 按键录制器组件              │  │
│  └────────────┬──────────────────┘  │
│               │ Tauri IPC (invoke)  │
│  ┌────────────▼──────────────────┐  │
│  │ Rust Backend                  │  │
│  │                               │  │
│  │ ┌─ audio/                     │  │
│  │ │  capture.rs    麦克风采集    │  │
│  │ │  resampler.rs  重采样       │  │
│  │ │                             │  │
│  │ ┌─ hotkey/                    │  │
│  │ │  mod.rs        快捷键监听   │  │
│  │ │                             │  │
│  │ ┌─ recognition/               │  │
│  │ │  engine.rs     SenseVoice   │  │
│  │ │                             │  │
│  │ ┌─ insertion/                 │  │
│  │ │  clipboard.rs  文字插入     │  │
│  │ │                             │  │
│  │ ┌─ correction/                │  │
│  │ │  dictionary.rs 纠错词典     │  │
│  │ │                             │  │
│  │ config.rs       配置持久化    │  │
│  │ commands.rs     IPC 命令      │  │
│  │ state.rs        应用状态      │  │
│  │ tray.rs         菜单栏图标    │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

### 核心依赖

**Rust (src-tauri/Cargo.toml)**:
| Crate | 用途 |
|-------|------|
| tauri 2.x | 应用框架 |
| sherpa-rs | SenseVoice 语音识别 |
| cpal | 跨平台音频采集 |
| arboard | 剪贴板操作 |
| core-graphics | CGEventTap, CGEvent |
| core-foundation | CFRunLoop |
| serde / serde_json | 序列化 |
| tokio | 异步运行时 |
| dirs | 系统目录 |
| anyhow | 错误处理 |
| hound | WAV 文件读写 |
| chrono | 时间戳生成 |

**Frontend (package.json)**:
| Package | 用途 |
|---------|------|
| vite | 构建工具 |
| typescript | 类型安全 |
| @tauri-apps/api | Tauri 前端 API |

### 数据流

```
用户按住快捷键
  → CGEventTap 检测按键按下
  → 100ms 防抖
  → 启动 cpal 麦克风输入流
  → 音频 PCM 数据写入 Arc<Mutex<Vec<f32>>> 缓冲区
  → 每 10ms 计算 RMS 振幅 → Tauri event → 前端波形渲染

用户松开快捷键
  → CGEventTap 检测按键松开
  → 停止 cpal 输入流，获取完整音频缓冲区
  → 校验时长（< 300ms 则丢弃）
  → 将原始音频保存为 WAV 文件（recording-YYYYMMDD-HHmmss.wav）
  → tokio::spawn_blocking:
    1. 如采样率非 16kHz，进行重采样
    2. sherpa-rs SenseVoiceRecognizer.transcribe(16000, &samples)
       → 返回 (text: String, lang: String)
    3. CorrectionDictionary.apply(text) → corrected_text
    4. (v2) AI 后处理 → final_text
    5. insertion::insert_text(final_text):
       a. 设置识别文字到剪贴板
       b. 等待 50ms
       c. 检查 Accessibility 权限
       d. 有权限 → CGEvent 模拟 Cmd+V；无权限 → AppleScript 回退
       e. 文字保留在剪贴板供用户重新粘贴
  → Tauri event: "transcription-complete" → 前端显示结果
```

---

## 项目结构

```
mascribe/
├── docs/
│   └── PRD.md                    # 本文档
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   │   ├── icon.png
│   │   ├── icon.icns
│   │   └── tray-icon.png        # 22x22 菜单栏模板图标
│   ├── src/
│   │   ├── main.rs              # 入口
│   │   ├── lib.rs               # Tauri 应用构建和初始化
│   │   ├── commands.rs          # Tauri IPC 命令处理
│   │   ├── state.rs             # 应用共享状态
│   │   ├── config.rs            # 配置持久化
│   │   ├── tray.rs              # 菜单栏图标
│   │   ├── audio/
│   │   │   ├── mod.rs
│   │   │   ├── capture.rs       # cpal 麦克风采集
│   │   │   ├── resampler.rs     # 重采样到 16kHz
│   │   │   └── wav_save.rs      # 录音保存为 WAV 文件
│   │   ├── hotkey/
│   │   │   ├── mod.rs
│   │   │   └── listener.rs      # CGEventTap 快捷键监听
│   │   ├── recognition/
│   │   │   ├── mod.rs
│   │   │   └── engine.rs        # sherpa-rs SenseVoice 封装
│   │   ├── insertion/
│   │   │   ├── mod.rs
│   │   │   └── clipboard.rs     # 剪贴板 + Cmd+V 模拟
│   │   └── correction/
│   │       ├── mod.rs
│   │       └── dictionary.rs    # JSON 纠错词典
│   └── build.rs
├── src/                          # Web 前端
│   ├── index.html                # 悬浮面板
│   ├── settings.html             # 设置窗口
│   ├── main.ts                   # 面板逻辑
│   ├── settings.ts               # 设置逻辑
│   ├── styles/
│   │   ├── panel.css             # 面板样式
│   │   └── settings.css          # 设置样式
│   └── lib/
│       ├── waveform.ts           # Canvas 波形渲染
│       ├── tauri-api.ts          # Tauri invoke 封装
│       └── types.ts              # TypeScript 类型定义
├── package.json
├── tsconfig.json
├── vite.config.ts
├── CLAUDE.md                     # 项目开发指引
├── .gitignore
└── README.md
```

---

## 所需系统权限

| 权限 | 用途 | 触发时机 |
|------|------|----------|
| 麦克风 (Microphone) | 录制语音 | 首次录音时弹窗 |
| 辅助功能 (Accessibility) | 模拟 Cmd+V 按键 | 首次文字插入时 |
| 输入监控 (Input Monitoring) | CGEventTap 捕获快捷键 | 应用启动时 |

应用应在首次启动时引导用户开启所需权限，并在权限缺失时显示友好提示。
若 `Accessibility` 缺失，系统可能返回“not allowed to send keystrokes”，此时自动粘贴会失败，但识别文本仍需保留在剪贴板。

---

## 当前迭代约束（2026-02）

- 分发策略：默认使用 `no sign` 本地构建，避免签名链路导致安装/启动不稳定
- 稳定性优先级：先保证“可启动、可录音、可识别、结果可复制”，再推进全屏浮窗兼容

---

## 非功能性需求

| 指标 | 目标 |
|------|------|
| 识别延迟 | < 500ms（5 秒音频，含转写+纠错+插入） |
| 内存占用 | < 500MB（含模型常驻内存） |
| 模型加载 | < 2s（首次启动） |
| 应用体积 | < 50MB（不含模型文件） |
| 隐私 | 完全本地运行，无数据上传 |
| 兼容性 | macOS 13.0+ (Ventura 及以上) |

---

## 外部依赖

| 依赖 | 路径 | 说明 |
|------|------|------|
| SenseVoice 模型 | `~/.openclaw/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/` | int8 量化 ONNX 模型，约 228MB |
| 模型文件 | `model.int8.onnx` | 主模型 |
| 词表文件 | `tokens.txt` | Token 词表 |

用户需自行确保模型文件存在。应用设置中提供模型路径配置。

---

## 未来规划

### 实时流式识别
- 边说边显示文字（SenseVoice 目前为离线模式，需评估流式支持）

### 多模型支持
- 允许用户选择不同的语音模型
- 支持 Whisper 等替代模型

---

## 参考项目

- [Superwhisper](https://superwhisper.com/) — 商业 macOS 语音输入工具
- [MacWhisper](https://goodsnooze.gumroad.com/l/macwhisper) — 本地 Whisper 转写工具
- [VoiceInk](https://github.com/VoiceInk/VoiceInk) — 开源 macOS 语音听写
- [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) — 语音识别推理框架
- [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) — 阿里多语言语音模型
