# MaScribe macOS 安装与使用指南

这份文档面向普通用户，重点是安装、首次授权和常见问题。

## 一、安装（推荐）

1. 在 GitHub Releases 下载最新 `.dmg`
2. 打开 `.dmg`，把 `MaScribe.app` 拖到 `Applications`
3. 从 `Applications` 启动 MaScribe

---

## 二、安装后先准备本地转写模型（必须）

当前版本不会自动下载语音模型。  
如果缺少模型文件，语音转写会失败。

### 1) 一键安装模型（推荐）

打开 Terminal，进入仓库目录后执行：

```bash
./scripts/install-sensevoice-model.sh
```

脚本会自动下载并安装到默认目录。

### 1.1 只有 DMG、没有源码时

直接执行下面命令：

```bash
mkdir -p ~/.openclaw/models/sensevoice
cd ~/.openclaw/models/sensevoice
curl -L -o sensevoice.tar.bz2 \
  https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
tar -xjf sensevoice.tar.bz2
rm -f sensevoice.tar.bz2
```

默认模型目录：

`~/.openclaw/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/`

必须包含：

- `model.int8.onnx`
- `tokens.txt`

### 2) 检查是否安装成功

执行：

```bash
ls ~/.openclaw/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/{model.int8.onnx,tokens.txt}
```

如果能看到这两个文件路径，说明模型已可用。

---

## 三、首次启动弹窗（请引导客户这样操作）

macOS 首次使用时，常见弹窗如下。

1. “无法验证开发者 / 已损坏 / 不允许打开”
- 打开：`系统设置 -> 隐私与安全性`
- 在底部找到被阻止的 MaScribe
- 点击 `仍要打开`

2. 麦克风权限
- 选择 `允许`

3. 辅助功能权限（用于自动粘贴 Cmd+V）
- 打开：`系统设置 -> 隐私与安全性 -> 辅助功能`
- 勾选 `MaScribe`

4. 输入监控权限（全局按键监听）
- 打开：`系统设置 -> 隐私与安全性 -> 输入监控`
- 勾选 `MaScribe`

5. 屏幕录制权限（仅在启用截图/OCR时需要）
- 打开：`系统设置 -> 隐私与安全性 -> 屏幕录制`
- 勾选 `MaScribe`

建议：权限改动后，退出并重启 MaScribe 一次。

---

## 四、基础使用

1. 启动 MaScribe（菜单栏图标）
2. 按默认快捷键 `Alt+Space` 开始录音
3. 再按一次停止
4. 文本会自动粘贴到当前光标位置

---

## 五、常见问题

### 1) 录音正常但不会自动粘贴

优先检查“辅助功能”权限是否开启：

- `系统设置 -> 隐私与安全性 -> 辅助功能`
- 若已开启仍失败，先关闭再开启，重启应用

可选重置命令：

```bash
tccutil reset Accessibility com.mascribe
```

### 2) 快捷键不触发

检查“输入监控”权限：

- `系统设置 -> 隐私与安全性 -> 输入监控`
- 确认 `MaScribe` 已勾选

### 3) 截图 OCR 不工作

检查“屏幕录制”权限：

- `系统设置 -> 隐私与安全性 -> 屏幕录制`
- 确认 `MaScribe` 已勾选

### 4) 日志在哪

```bash
~/Library/Logs/MaScribe.log
```
