# MaScribe Windows 安装与构建指南

这份文档只保留 Windows 用户真正需要的内容：安装、启动、权限确认、常见问题。

## 一、普通用户安装（推荐）

1. 在 GitHub Releases 下载 Windows 安装包（`.msi` 或 `.exe`）
2. 双击安装
3. 启动 MaScribe，按提示完成首次授权

安装后默认快捷键是 `Alt+Space`。

---

## 二、首次启动权限弹窗（请引导客户这样点）

Windows 上可能出现以下弹窗，这些都属于正常现象：

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
- MaScribe 的自动粘贴、快捷键、截图在 Windows 通常不需要额外系统授权弹窗。
- 如果启用“屏幕 OCR（Windows Built-in）”并要识别中文，需额外安装 OCR 中文语言包（见下文）。

---

## 三、可选功能：Windows 原生 OCR（中文）

如果你要使用“截图 OCR 上下文”且需要中文识别：

```powershell
# 管理员 PowerShell
Add-WindowsCapability -Online -Name Language.OCR~~~zh-Hans~0.0.1.0
```

安装后在应用设置中选择：
- `Screen OCR -> Windows Built-in`

---

## 四、从源码构建（开发者）

### 1) 安装依赖

```powershell
winget install Rustlang.Rustup
winget install OpenJS.NodeJS
winget install Microsoft.VisualStudio.2022.BuildTools
winget install Git.Git
```

安装 Build Tools 时请勾选：`使用 C++ 的桌面开发`。

### 2) 拉代码并构建

```powershell
git clone git@github.com:xiaozhenliu/mascribe.git
cd mascribe
npm install
npm run tauri -- build
```

构建产物位置：
- `src-tauri/target/release/bundle/msi/`
- `src-tauri/target/release/bundle/nsis/`

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
- 按第三节安装 `Language.OCR~~~zh-Hans~0.0.1.0`

