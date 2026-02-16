# MaScribe Windows 状态说明

当前版本尚未发布可交付的 Windows 安装包，因此这里不再提供 `MSI/EXE` 安装步骤。

如果你是终端用户，请先使用 macOS 版本；Windows 版本待后续发布。

---

## 一、Windows 权限与弹窗（预备说明）

后续 Windows 版本发布后，通常会遇到以下弹窗，这些都属于正常现象：

1. SmartScreen（“Windows 已保护你的电脑”）
- 点击 `更多信息`
- 再点击 `仍要运行`

2. UAC（用户账户控制）
- 点击 `是`

3. 麦克风权限提示（首次录音时）
- 请选择 `允许`

如果客户没看到麦克风弹窗但录音无效：
- 打开 `设置 -> 隐私和安全性 -> 麦克风`
- 打开 `允许应用访问麦克风`
- 打开 `允许桌面应用访问麦克风`

说明：
- 自动粘贴、快捷键、截图在 Windows 通常不需要额外系统授权弹窗。
- 如果启用“屏幕 OCR（Windows Built-in）”并要识别中文，需额外安装 OCR 中文语言包（见下文）。

---

## 二、可选功能：Windows 原生 OCR（中文）

如果你要使用“截图 OCR 上下文”且需要中文识别：

```powershell
# 管理员 PowerShell
Add-WindowsCapability -Online -Name Language.OCR~~~zh-Hans~0.0.1.0
```

后续版本可在应用设置中选择：
- `Screen OCR -> Windows Built-in`

---

## 三、从源码构建（开发者，预研）

### 1) 安装依赖

```powershell
winget install Rustlang.Rustup
winget install OpenJS.NodeJS
winget install Microsoft.VisualStudio.2022.BuildTools
winget install Git.Git
```

安装 Build Tools 时请勾选：`使用 C++ 的桌面开发`。

### 2) 拉代码并构建（仅供预研验证）

```powershell
git clone git@github.com:xiaozhenliu/mascribe.git
cd mascribe
npm install
npm run tauri -- build
```

说明：Windows 安装包形态与产物路径尚未作为正式发布能力承诺，请以实际 CI/发布流程为准。

---

## 四、Windows 本地语音模型准备（必须）

即使后续拿到 Windows 可执行包，首次使用前也必须准备本地 SenseVoice 模型。  
缺少模型文件时，语音转写会失败。

默认目录（与代码一致）：

`%APPDATA%\com.mascribe\models\sensevoice\sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17\`

### 1) 下载并解压模型（PowerShell）

```powershell
$modelsRoot = Join-Path $env:APPDATA "com.mascribe\models\sensevoice"
New-Item -ItemType Directory -Force -Path $modelsRoot | Out-Null
Set-Location $modelsRoot

$url = "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2"
$archive = "sensevoice.tar.bz2"

Invoke-WebRequest -Uri $url -OutFile $archive
tar -xjf $archive
Remove-Item $archive
```

### 2) 验证模型文件

```powershell
$base = Join-Path $env:APPDATA "com.mascribe\models\sensevoice\sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17"
Test-Path (Join-Path $base "model.int8.onnx")
Test-Path (Join-Path $base "tokens.txt")
```

两个命令都返回 `True` 才表示模型准备完成。

---

## 五、最常见问题

1. 启动后无文字自动输入
- 先确认麦克风权限已开启
- 再确认你光标在可编辑输入框中
- 查看日志：`%APPDATA%\MaScribe\logs\MaScribe.log`

2. 构建报 linker / SDK 错误
- 补装：`Desktop development with C++`
- 必要时安装 Windows SDK：
```powershell
winget install Microsoft.WindowsSDK.10.0.22621
```

3. 识别中文 OCR 失败
- 按第二节安装 `Language.OCR~~~zh-Hans~0.0.1.0`
