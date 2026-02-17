# MaScribe v0.4.0

## 中文

**MaScribe v0.4.0** — 全屏覆盖支持

悬浮语音输入窗口现在可以在全屏应用上方正常显示和使用。无需退出全屏，录音、转写、OCR 上下文、AI 润色、粘贴的完整流程都可以在全屏模式下完成。

**主要更新：**

- 悬浮窗支持在全屏应用（AFFiNE、Chrome、VS Code 等）上方显示
- 录音开始时自动记住前台应用，粘贴时重新激活，确保文字输入到正确位置
- 辅助功能权限缺失时主动弹出授权提示，不再静默失败

**修复：**

- 修复 Tauri API 覆盖原生 ObjC 窗口标志导致悬浮窗无法出现在全屏 Space 的问题
- 修复坐标系不匹配导致窗口定位到屏幕外的问题

**升级说明：** 无需修改配置。已授权的权限保持不变（bundle ID 和签名身份未变）。

---

## English

**MaScribe v0.4.0** — Fullscreen Overlay Support

The floating voice input window now appears above full-screen applications. The entire pipeline — hotkey trigger, recording, transcription, OCR context, AI polishing, and paste — works seamlessly without leaving fullscreen mode.

**Highlights:**

- Floating overlay works above full-screen apps (AFFiNE, Chrome, VS Code, etc.)
- Smart paste target tracking: remembers the foreground app before recording, re-activates it before pasting
- On-demand Accessibility permission prompt when missing, instead of silent failure

**Fixes:**

- Fixed window invisible on fullscreen Spaces due to Tauri API overwriting native ObjC window flags
- Fixed window positioned off-screen due to coordinate system mismatch

**Upgrade notes:** No configuration changes required. Previously granted permissions are preserved.
