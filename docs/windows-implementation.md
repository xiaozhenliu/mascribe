# Windows 平台迁移实现文档

## 概述

本文档记录了将 macOS 语音输入工具迁移到 Windows 平台的实际实现细节。

---

## 1. 项目结构变更

### 1.1 热键模块 (hotkey)

**重构前:**
```
src/hotkey/
  └── mod.rs          (包含所有热键逻辑，macOS专用)
```

**重构后:**
```
src/hotkey/
  ├── mod.rs          (跨平台接口)
  ├── keycode.rs      (平台无关的键码定义)
  ├── macos.rs        (macOS CGEventTap 实现)
  └── windows.rs      (Windows SetWindowsHookEx 实现)
```

#### 关键设计决策

1. **键码抽象层**: 创建了 `Key` 和 `Modifier` 枚举来统一表示按键
   - `Key::FKey(u8)` - 功能键 F1-F24（注意：原本想用 `F(u8)` 但与字母 F 冲突）
   - `Modifier::{Ctrl, Shift, Alt, Meta}` - 修饰键
   - 支持解析 "Ctrl+Shift+S" 这样的热键字符串

2. **平台适配**:
   - macOS: 使用 CGEventTap 监听全局键盘事件
   - Windows: 使用 SetWindowsHookEx + WH_KEYBOARD_LL 低级别键盘钩子

3. **热键匹配逻辑**:
   - macOS: 检查 keycode 和 flags (CGEventFlags)
   - Windows: 检查 VK code 和 GetAsyncKeyState 获取的修饰键状态

---

### 1.2 文本插入模块 (insertion)

**重构前:**
```
src/insertion/
  ├── mod.rs
  └── clipboard.rs    (macOS专用)
```

**重构后:**
```
src/insertion/
  ├── mod.rs          (跨平台接口)
  ├── macos.rs        (CGEvent Cmd+V 模拟)
  └── windows.rs      (SendInput Ctrl+V 模拟)
```

#### 实现细节

**macOS 实现:**
- 使用 `CGEvent` 创建键盘事件
- 设置 `CGEventFlags::CGEventFlagCommand` 标志位
- 需要 Accessibility 权限
- 备选方案：AppleScript 调用 System Events

**Windows 实现:**
- 使用 `SendInput` API 发送键盘输入
- 构造 `INPUT` 结构体数组：[Ctrl按下, V按下, V释放, Ctrl释放]
- 不需要特殊权限

---

### 1.3 截图模块 (screenshot)

**重构前:**
```
src/screenshot/
  └── mod.rs          (macOS CGWindow 实现)
```

**重构后:**
```
src/screenshot/
  ├── mod.rs          (跨平台接口 + 共享工具函数)
  ├── macos.rs        (CGWindow API)
  └── windows.rs      (GDI BitBlt)
```

#### 实现细节

**macOS 实现:**
- 使用 `CGDisplay::screenshot()` 捕获主显示器
- 将 `CGImage` 转换为 PNG 字节

**Windows 实现:**
- 使用 GDI API:
  1. `GetDesktopWindow()` 获取桌面窗口
  2. `GetDC()` 获取设备上下文
  3. `CreateCompatibleDC()` 和 `CreateCompatibleBitmap()` 创建兼容位图
  4. `BitBlt()` 复制屏幕内容
  5. `GetDIBits()` 获取像素数据
- 将 BGRA 格式转换为 RGBA
- 编码为 PNG

---

### 1.4 权限模块 (permissions)

**变更:** 添加 Windows 存根实现

**macOS:**
- `request_microphone_permission()` - 调用 AVCaptureDevice.requestAccess
- `request_accessibility_permission()` - 调用 AXIsProcessTrustedWithOptions
- `check_screen_recording_permission()` - 调用 CGPreflightScreenCaptureAccess

**Windows:**
- 所有权限函数返回 `true`
- 麦克风：Windows 在首次使用时自动提示
- 辅助功能/截图：Windows 没有类似的权限系统

---

### 1.5 主库 (lib.rs)

**变更:**

1. **日志设置平台化**:
   ```rust
   #[cfg(target_os = "macos")]
   fn setup_file_logging() { /* 使用 dup2 重定向 stdout/stderr */ }

   #[cfg(target_os = "windows")]
   fn setup_file_logging() { /* Windows 不支持 dup2，简化处理 */ }
   ```

2. **权限请求平台化**:
   ```rust
   #[cfg(target_os = "macos")]
   { /* 请求麦克风和辅助功能权限 */ }

   #[cfg(target_os = "windows")]
   { /* Windows 权限自动处理 */ }
   ```

---

## 2. Cargo.toml 变更

### 2.1 添加的依赖

```toml
# 全局依赖
once_cell = "1.19"  # 用于 Windows 热键的静态变量

# Windows 专用依赖
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.52", features = [
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Graphics_Gdi",
    "Win32_System_Threading",
    "Win32_Foundation",
    "Win32_System_LibraryLoader",
    "Win32_Graphics_Direct3D11",
    "Win32_Graphics_Dxgi_Common",
    "Win32_Graphics_Dxgi",
    "Win32_System_WinRT",
    "Win32_System_Com",
    "Graphics",
    "Graphics_Capture",
    "Graphics_DirectX",
    "Graphics_DirectX_Direct3D11",
] }
```

---

## 3. 编译验证

### 3.1 macOS 构建

```bash
cargo build --lib
```

**结果:** ✅ 成功

### 3.2 Windows 构建

需要在 Windows 环境或使用交叉编译工具链。

```bash
# Windows 目标
cargo build --target x86_64-pc-windows-msvc
```

---

## 4. 关键代码片段

### 4.1 热键解析

```rust
// 解析 "Ctrl+Shift+S" 为 HotkeyDefinition
pub fn parse_hotkey(hotkey: &str) -> Result<HotkeyDefinition, String> {
    let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();

    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers.push(Modifier::Ctrl),
            "shift" => modifiers.push(Modifier::Shift),
            "alt" | "option" | "opt" => modifiers.push(Modifier::Alt),
            "meta" | "command" | "cmd" | "win" | "windows" => {
                modifiers.push(Modifier::Meta)
            }
            _ => { /* 解析为 Key */ }
        }
    }

    Ok(HotkeyDefinition { modifiers, key })
}
```

### 4.2 Windows 热键钩子

```rust
extern "system" fn keyboard_hook_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM
) -> LRESULT {
    if n_code >= 0 && w_param.0 as u32 == WM_KEYDOWN {
        let kbd_struct = unsafe {
            &*(l_param.0 as *const KBDLLHOOKSTRUCT)
        };
        let vk = VIRTUAL_KEY(kbd_struct.vkCode as u16);

        // 检查是否匹配目标热键
        if let Ok(target) = TARGET_HOTKEY.lock() {
            if let Some((target_vk, target_mods)) = *target {
                if vk == target_vk {
                    let current_mods = get_modifier_state();
                    if (current_mods & target_mods) == target_mods {
                        // 热键匹配，执行回调
                        if let Ok(callback) = CALLBACK.lock() {
                            if let Some(ref cb) = *callback {
                                cb();
                            }
                        }
                    }
                }
            }
        }
    }

    unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
}
```

### 4.3 Windows 文本插入

```rust
pub fn insert_text(text: &str) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;

    thread::sleep(Duration::from_millis(50));

    // 构造 Ctrl+V 按键序列
    let inputs = [
        INPUT { /* Ctrl down */ },
        INPUT { /* V down */ },
        INPUT { /* V up */ },
        INPUT { /* Ctrl up */ },
    ];

    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }

    Ok(())
}
```

---

## 5. 已知问题和注意事项

### 5.1 键码冲突

**问题:** `Key` 枚举中字母 `F` 和功能键 `F(u8)` 命名冲突。

**解决:** 将功能键重命名为 `FKey(u8)`。

### 5.2 小键盘键

**问题:** macOS 实现中引用了 `Key::NumpadClear` 等不存在于 `Key` 枚举的键。

**解决:** 添加了完整的数字小键盘键支持：
- `Numpad0` - `Numpad9`
- `NumpadDecimal`, `NumpadMultiply`, `NumpadPlus`
- `NumpadMinus`, `NumpadDivide`, `NumpadEnter`

### 5.3 修饰键作为普通键

**问题:** macOS 实现中 `Key::Meta`, `Key::Shift` 等被当作普通键处理。

**解决:** 移除了这些键，因为修饰键应该通过 `Modifier` 枚举处理，而不是 `Key` 枚举。

### 5.4 Windows 日志

**问题:** Windows 不支持 Unix 的 `dup2` 系统调用。

**解决:** Windows 版本的 `setup_file_logging()` 仅打印日志路径，不重定向 stdout/stderr。

### 5.5 特殊热键处理

**问题:** `ContextMenu`、`F13-F24` 等特殊键无法通过 `tauri-plugin-global-shortcut` 正确注册。

**解决:** 在 `main.ts` 中添加 `NATIVE_ONLY_KEYS` 列表，对这些键直接跳过插件，使用原生热键 API：

```typescript
const NATIVE_ONLY_KEYS = ["ContextMenu", "F13", "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24"];

function shouldUseNativeHotkey(shortcut: string): boolean {
  return NATIVE_ONLY_KEYS.some(key => shortcut.includes(key));
}
```

对于特殊键，前端直接调用 `register_native_hotkey`，使用 Windows SetWindowsHookEx 捕获。

---

## 6. 屏幕 OCR 模块

### 6.1 当前状态

macOS 版本已实现原生 OCR（`src/ocr/macos.rs`），使用 Vision 框架 `VNRecognizeTextRequest`，约 0.6 秒完成识别。

Windows 版本 **暂未实现** 原生 OCR（`src/ocr/mod.rs` 返回 `"Native OCR not supported on Windows"`），可使用 Ollama API 作为替代方案。

### 6.2 OCR 工作流程

```
截图 → OCR (macOS 原生 / Ollama API) → 屏幕文字
                                             ↓
语音 → SenseVoice → 纠错词典 → AI 润色 + 屏幕文字上下文 → 粘贴输出
```

- OCR 上下文仅在「在线 API」润色模式下使用（本地 Qwen 2.5 模型上下文容量不足）
- OCR 结果截断至 500 字符
- 润色输出超过输入长度 3 倍 + 20 字符时被拒绝（防止 OCR 内容泄漏）

### 6.3 Windows 原生 OCR 实现方案

使用 `Windows.Media.Ocr` WinRT API（与 PowerToys Text Extractor 相同的引擎）。

#### 系统要求
- Windows 10 1507+ (API 自带，无需额外安装)
- 需要安装对应语言的 OCR 语言包
  - 英文通常预装
  - 中文简体：`Settings → Language & Region` 添加中文，或 PowerShell: `Add-WindowsCapability -Online -Name Language.OCR~~~zh-Hans~0.0.1.0`

#### Cargo.toml 新增 features

```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.52", features = [
    # ... existing features ...
    "Media_Ocr",
    "Graphics_Imaging",
    "Storage_Streams",
    "Globalization",
    "Foundation",
    "Foundation_Collections",
] }
```

#### 实现流程

```
PNG bytes → InMemoryRandomAccessStream → BitmapDecoder → SoftwareBitmap
    → OcrEngine("zh-Hans") → RecognizeAsync → 逐行提取 → join("\n")
```

1. 将 PNG 字节写入 `InMemoryRandomAccessStream`
2. `BitmapDecoder::CreateAsync()` 解码为 `SoftwareBitmap`
3. `OcrEngine::TryCreateFromLanguage("zh-Hans")`（中文引擎天然支持 ASCII/英文/数字）
4. `engine.RecognizeAsync(&bitmap).get()`（阻塞等待，在后台线程执行）
5. 遍历 `result.Lines()` 提取文字

#### 注意事项

| 问题 | 解决 |
|------|------|
| 语言包未安装 | fallback 到 `TryCreateFromUserProfileLanguages()` |
| 最大图片 2560px | 已有 `screenshot_max_size` 配置（默认 1024px） |
| WinRT 线程初始化 | 后台线程需调用 `RoInitialize(MTA)` |
| 文档称需要 MSIX 打包 | 实际不需要（PowerToys 证明了这一点） |

#### Fallback 方案
- Ollama API — 已实现，用户可在设置中切换

---

## 7. Phase 2: 本地多模态模型支持

### 7.1 实现状态

> **状态**: ✅ 基础框架已实现（2024-02-12）
>
> **注意**: 当前为 stub 实现，完整后端需要 ONNX Runtime 或 llama.cpp vision 支持

### 7.2 已完成工作

#### Vision 模型配置
- ✅ `config.rs` 添加 `vision_model_path`, `vision_mode`, `vision_max_image_size` 字段
- ✅ 默认路径: `~/.openclaw/models/minicpm-v-2_6/`

#### Vision 引擎模块
- ✅ 创建 `src/vision/mod.rs` 定义接口
  - `VisionEngine` trait: 处理图像 + 文本提示
  - `VisionConfig`: 模型配置
  - `preprocess_image()`: 图像预处理（调整大小）
  - `build_vision_prompt()`: 构建视觉提示
- ⚠️ 当前为 stub 实现，返回 `VisionError::NotSupported`

#### 流程集成
- ✅ `commands.rs` 集成 vision 处理流程
  - 当 `vision_mode = "local"` 且截图可用时，优先使用 vision 模型
  - Vision 失败时自动回退到文本润色
  - 支持图像预处理（调整大小）

#### 前端更新
- ✅ `settings.html` 添加 Vision Model 部分
  - "Disabled / 关闭" 选项
  - "Local Model / 本地模型 ⭐" 选项
  - 模型路径输入框
- ✅ `settings.ts` 添加 vision 设置逻辑
  - `getVisionMode()`, `setVisionMode()`
  - `updateVisionVisibility()` 控制 UI 显示
  - 保存/加载配置

### 7.3 待完成工作

#### 后端实现（需要进一步调研）
1. **ONNX Runtime 后端**
   - MiniCPM-V 有官方 ONNX 导出工具
   - 需要添加 `ort` crate 依赖
   - 实现 `OnnxVisionEngine`

2. **llama.cpp 后端**
   - 等待 `llama-cpp-2` crate 支持视觉模型
   - 或手动编译 llama.cpp vision 分支
   - 需要处理 `mmproj.bin` multimodal projector

### 7.4 模型推荐（RTX 4060 8GB）
- MiniCPM-V 2.6 INT4 (6-8GB VRAM)
- Qwen2-VL 7B INT4 (6-8GB VRAM)

---

## 8. 文件清单

### 新建文件
- `src/hotkey/keycode.rs` - 键码抽象
- `src/hotkey/macos.rs` - macOS 热键实现
- `src/hotkey/windows.rs` - Windows 热键实现
- `src/insertion/macos.rs` - macOS 文本插入
- `src/insertion/windows.rs` - Windows 文本插入
- `src/screenshot/macos.rs` - macOS 截图
- `src/screenshot/windows.rs` - Windows 截图
- `src/ocr/mod.rs` - OCR 跨平台接口
- `src/ocr/macos.rs` - macOS Vision 框架 OCR 实现
- `src/vision/mod.rs` - Vision 模型接口（Phase 2）
- `docs/windows-migration-plan.md` - 迁移计划
- `docs/windows-implementation.md` - 本文档

### 修改文件
- `Cargo.toml` - 添加 Windows 依赖
- `src/hotkey/mod.rs` - 重构为跨平台接口，添加 F13-F24 支持
- `src/hotkey/keycode.rs` - 添加 ContextMenu 和 F13-F24 键定义
- `src/insertion/mod.rs` - 重构为跨平台接口
- `src/insertion/clipboard.rs` - 删除（内容移至 macos.rs）
- `src/screenshot/mod.rs` - 重构为跨平台接口
- `src/permissions.rs` - 添加 Windows 存根
- `src/lib.rs` - 平台化日志和权限，添加 vision 模块
- `src/commands.rs` - 更新热键调用，集成 vision 处理流程
- `src/config.rs` - 添加 vision 配置字段
- `src/main.ts` - 添加特殊热键处理逻辑（NATIVE_ONLY_KEYS）
- `settings.html` - 添加 Vision Model UI
- `settings.ts` - 添加 vision 设置逻辑

---

## 9. 更新记录

### 2024-02-12
- 添加 `ContextMenu` 键支持
- 添加 F13-F24 功能键支持
- 修复特殊热键在前端的处理逻辑

### 2024-02-12 (Phase 2)
- 添加 Vision 模型配置字段 (`vision_model_path`, `vision_mode`, `vision_max_image_size`)
- 创建 `src/vision/mod.rs` 接口模块（stub 实现）
- 更新 `settings.html` 添加 Vision Model UI
- 更新 `settings.ts` 添加 vision 设置逻辑
- 在 `commands.rs` 集成 vision 处理流程（优先 vision，失败回退到文本润色）

## 10. 总结

本次迁移将原本 macOS 专用的语音输入工具改造为跨平台应用，主要工作包括：

1. **抽象层设计** - 创建了平台无关的接口，隐藏底层差异
2. **Windows 实现** - 使用 Win32 API 实现了热键、文本插入、截图功能
3. **特殊热键支持** - 对 `ContextMenu`、`F13-F24` 等键使用原生 API
4. **条件编译** - 使用 `#[cfg(target_os = "...")]` 管理平台特定代码
5. **向后兼容** - macOS 版本功能保持不变

Windows 版本与 macOS 版本的主要差异：
- 不需要 Accessibility 权限
- 不需要 Screen Recording 权限
- 热键使用 Ctrl 代替 Command
- 日志处理方式不同
- 特殊键（ContextMenu、F13-F24）使用原生热键 API
