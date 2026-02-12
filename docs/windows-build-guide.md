# Windows 构建指南

本文档介绍如何从零开始在 Windows 上构建语音输入工具。

---

## 前置要求

你需要安装以下软件：

### 1. Rust 工具链

```powershell
# 下载 Rust 安装程序
# 访问 https://rustup.rs/ 下载 rustup-init.exe
# 或者使用 winget：
winget install Rustlang.Rustup

# 安装完成后，重启终端，然后运行：
rustup default stable
rustc --version  # 验证安装
```

### 2. Visual Studio 2022 (Build Tools)

```powershell
# 方法1：使用 winget 安装 Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools

# 方法2：手动下载
# 访问 https://visualstudio.microsoft.com/downloads/
# 下载 "Visual Studio Build Tools 2022"
# 安装时选择 "使用 C++ 的桌面开发" 工作负载
```

### 3. Node.js 和 npm

```powershell
# 使用 winget 安装
winget install OpenJS.NodeJS

# 验证安装
node --version
npm --version
```

### 4. Git

```powershell
# 使用 winget 安装
winget install Git.Git

# 验证安装
git --version
```

---

## 下载项目代码

```powershell
# 克隆仓库
git clone <你的仓库地址> mac-voice-input
cd mac-voice-input
```

---

## 安装前端依赖

```powershell
# 在项目根目录下
npm install
```

---

## 下载语音模型

SenseVoice 模型是必需的，用于语音识别。

### 方法1：自动下载（推荐）

```powershell
# 创建模型目录
mkdir "%USERPROFILE%\.openclaw\models\sensevoice"

# 下载模型文件（使用 PowerShell）
# 模型文件较大（约 300MB），需要一些时间

# 下载模型配置和字典文件
Invoke-WebRequest -Uri "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2" -OutFile "sensevoice.tar.bz2"

# 解压（需要安装 7-Zip 或 tar）
# 如果使用 7-Zip：
# 7z x sensevoice.tar.bz2
# 7z x sensevoice.tar

# 或者使用 Windows 11 自带的 tar：
tar -xjf sensevoice.tar.bz2

# 将解压后的文件移动到模型目录
move sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17 "%USERPROFILE%\.openclaw\models\sensevoice\"
```

### 方法2：手动下载

1. 访问 https://github.com/k2-fsa/sherpa-onnx/releases/tag/asr-models
2. 搜索 "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17"
3. 下载 `.tar.bz2` 文件
4. 解压到 `%USERPROFILE%\.openclaw\models\sensevoice\`

### 验证模型文件

模型目录应该包含以下文件：

```
%USERPROFILE%\.openclaw\models\sensevoice\
├── model.int8.onnx
├── tokens.txt
├── sense-voice-zh-en-ja-ko-yue.int8.onnx
└── ... (其他配置文件)
```

---

## 构建项目

### 开发模式（推荐用于测试）

```powershell
# 在项目根目录下运行
cargo tauri dev
```

这会：
1. 编译 Rust 后端
2. 启动 Vite 开发服务器
3. 打开应用窗口

第一次编译需要较长时间（5-15分钟），因为需要下载和编译所有依赖。

### 生产构建

```powershell
# 构建可执行文件
cargo tauri build
```

构建完成后，安装包位于：
- `src-tauri/target/release/bundle/msi/*.msi`
- `src-tauri/target/release/bundle/nsis/*.exe`

---

## 常见问题

### 1. 编译错误：找不到 Windows SDK

```powershell
# 安装 Windows SDK
winget install Microsoft.WindowsSDK.10.0.22621
```

### 2. 编译错误：linker 找不到

确保安装了 Visual Studio Build Tools 的 "使用 C++ 的桌面开发" 工作负载。

### 3. 模型加载失败

检查模型路径是否正确：

```powershell
# 检查模型目录是否存在
ls "%USERPROFILE%\.openclaw\models\sensevoice"

# 如果不存在，手动创建并下载模型
```

### 4. 缺少 DLL 错误

如果运行时提示缺少 `VCRUNTIME140.dll` 等：

```powershell
# 安装 Visual C++ Redistributable
winget install Microsoft.VCRedist.2015+.x64
```

---

## 完整构建脚本

创建一个 `build-windows.ps1` 文件：

```powershell
# build-windows.ps1
# 一键构建脚本

Write-Host "=== Windows 构建脚本 ===" -ForegroundColor Green

# 检查必要工具
$tools = @("rustc", "node", "npm", "git")
foreach ($tool in $tools) {
    if (!(Get-Command $tool -ErrorAction SilentlyContinue)) {
        Write-Error "缺少工具: $tool，请先安装"
        exit 1
    }
}

# 安装前端依赖
Write-Host "安装前端依赖..." -ForegroundColor Yellow
npm install

# 检查模型
$modelPath = "$env:USERPROFILE\.openclaw\models\sensevoice"
if (!(Test-Path $modelPath)) {
    Write-Host "模型未找到，请手动下载到: $modelPath" -ForegroundColor Red
    Write-Host "下载地址: https://github.com/k2-fsa/sherpa-onnx/releases/tag/asr-models"
    exit 1
}

# 开发构建
Write-Host "启动开发构建..." -ForegroundColor Yellow
cargo tauri dev
```

运行：

```powershell
# 设置执行策略（首次运行 PowerShell 脚本需要）
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# 运行构建脚本
.\build-windows.ps1
```

---

## 下一步

构建成功后，你可以：

1. **测试基本功能**：
   - 按热键（默认 Ctrl+Shift+S）开始录音
   - 说话后松开，看是否能识别并输入文字

2. **配置设置**：
   - 右键系统托盘图标打开设置
   - 修改热键、选择语言等

3. **（可选）下载 AI 润色模型**：
   - 下载 Qwen 2.5 1.5B GGUF 模型
   - 放到 `%USERPROFILE%\.openclaw\models\qwen2.5-1.5b\`

---

## 热键配置说明

### 支持的热键类型

**标准组合键**（使用 tauri-plugin-global-shortcut）：
- `Alt+Space` - Alt+空格
- `Ctrl+Shift+S` - Ctrl+Shift+S
- `CmdOrCtrl+Shift+A` - Command/Ctrl+Shift+A

**特殊键**（使用原生 Windows API）：
- `ContextMenu` - 菜单键（键盘上的右键菜单键）
- `F13` - `F24` - 扩展功能键

### 特殊键说明

某些键（如 `ContextMenu`、`F13-F24`）无法通过标准全局快捷键 API 注册，应用会自动使用 Windows 原生低级别键盘钩子（SetWindowsHookEx）来捕获这些键。

在设置界面中：
- 点击热键输入框并按想要的组合键
- 或者使用 "Presets" 下拉菜单选择特殊键
- `ContextMenu` 键可以通过下拉菜单选择，或在输入框中右键点击捕获

---

## 需要帮助？

如果遇到问题：

1. 查看 `src-tauri/target/release/VoiceInput.log` 日志文件
2. 在终端运行 `cargo tauri dev` 查看实时输出
3. 检查模型路径是否正确
