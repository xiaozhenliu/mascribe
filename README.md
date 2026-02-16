# MaScribe（马上听写）

[![Release](https://img.shields.io/github/v/release/xiaozhenliu/mascribe?label=release)](https://github.com/xiaozhenliu/mascribe/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Platform: macOS](https://img.shields.io/badge/platform-macOS-blue.svg)](docs/macos-guide.md)

[English](README.en.md) | 中文

MaScribe 是一个本地语音输入工具。按下快捷键说话，结束后自动把文本输入到当前光标位置。

当前发布版本：`v0.3.2`

## 功能概览

- 一键语音输入（默认快捷键 `Alt+Space`）
- 多语言转写（含中英混合）
- AI 润色（本地模型或在线 API）
- 截图 OCR 上下文辅助纠错

## 平台支持

- macOS：已发布，推荐使用
- Windows：开发中，暂未提供正式安装包

## 安装（macOS）

1. 打开 GitHub Releases，下载最新 `.dmg`
2. 将 `MaScribe.app` 拖到 `Applications`
3. 从 `Applications` 启动

## 首次使用（必须）

### 1) 准备本地转写模型

当前版本不会自动下载模型。缺少模型文件时，语音转写不可用。

源码用户（仓库根目录）：

```bash
./scripts/install-sensevoice-model.sh
```

仅下载 DMG 用户：

```bash
mkdir -p "$HOME/Library/Application Support/com.mascribe/models/sensevoice"
cd "$HOME/Library/Application Support/com.mascribe/models/sensevoice"
curl -L -o sensevoice.tar.bz2 \
  https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
tar -xjf sensevoice.tar.bz2
rm -f sensevoice.tar.bz2
```

验证模型文件：

```bash
ls "$HOME/Library/Application Support/com.mascribe/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/model.int8.onnx" \
   "$HOME/Library/Application Support/com.mascribe/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/tokens.txt"
```

### 2) 授权系统权限

首次启动请按提示授权：

- 麦克风
- 辅助功能（自动粘贴）
- 输入监控（全局快捷键）
- 屏幕录制（仅截图/OCR场景）

## 快速开始

1. 启动 MaScribe
2. 按 `Alt+Space` 开始录音
3. 再按一次停止
4. 文本自动插入当前输入位置

## AI 润色配置说明

### 本地模型模式（Local Model）

- 本地模式使用 `llama.cpp`，需要填写 **GGUF 文件路径**
- 推荐模型：`Qwen2.5-1.5B-Instruct`（GGUF）
- 在设置中：
  - `AI Polishing Engine -> Local Model`
  - 在 `Local Model Settings` 里填写或浏览选择 `*.gguf`

### Ollama 模式（适配不同机器，推荐）

如果你机器上模型由 Ollama 管理，建议使用 `Online API` 模式：

1. `AI Polishing Engine -> Online API`
2. `Endpoint` 填 `http://localhost:11434/v1`
3. 点击 `Detect Ollama Models` 自动识别本机已安装模型
4. 从 `Model` 下拉建议中选择（如 `qwen2.5:1.5b`）

## 常见问题

### 自动粘贴失败

通常是辅助功能权限未生效：

1. `系统设置 -> 隐私与安全性 -> 辅助功能`
2. 确认 `MaScribe` 已勾选（必要时删掉后重新添加）
3. 重启 MaScribe

可选重置：

```bash
tccutil reset Accessibility com.mascribe
```

## 文档

- macOS 安装与权限：`docs/macos-guide.md`
- 本地源码打包：`docs/local-build-guide.md`
- 在线 API 配置（中文）：`docs/online-api-guide-zh.md`
- Online API Guide (English)：`docs/online-api-guide-en.md`
- 产品与设计说明：`docs/PRD.md`

## 从源码构建

```bash
git clone git@github.com:xiaozhenliu/mascribe.git
cd mascribe
npm install
npx tauri build
```

构建产物：

- `src-tauri/target/release/bundle/macos/MaScribe.app`
- `src-tauri/target/release/bundle/dmg/MaScribe_*.dmg`

## License

MIT
