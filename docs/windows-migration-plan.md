# Windows 平台迁移 + 本地多模态模型支持计划

## 概述

将现有的 macOS 语音输入工具迁移到 Windows 平台，并添加本地多模态模型支持。用户的 RTX 4060 (8GB VRAM) 在 Windows 上，需要：
1. 跨平台支持（macOS + Windows）
2. Windows 版本的热键、截图、文本插入功能
3. 本地 Vision 模型支持（Qwen2-VL 或 MiniCPM-V）

---

## Phase 1: Windows 平台迁移

### 1.1 构建系统配置

**文件: `src-tauri/Cargo.toml`**

添加 Windows 依赖：
```toml
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

macOS 依赖保持不变（已有 `#[cfg(target_os = "macos")]` 条件）。

---

### 1.2 热键模块改造

**文件: `src-tauri/src/hotkey/keycode.rs`** (新建)

平台无关的键码解析：
```rust
/// Parse a hotkey string like "Ctrl+Shift+S" into platform-specific key codes
pub fn parse_hotkey(hotkey: &str) -> Result<HotkeyDefinition, String> {
    // Normalize: Command -> Ctrl on Windows, Option -> Alt
    // Parse modifiers and key
}

pub struct HotkeyDefinition {
    pub modifiers: Vec<Modifier>,
    pub key: Key,
}

pub enum Modifier { Ctrl, Shift, Alt, Meta }
pub enum Key { A-Z, Num0-9, FKey(u8), Escape, Space, Enter, Tab, ... }
```

**文件: `src-tauri/src/hotkey/mod.rs`** (重构)

重构为平台无关的接口：
- macOS: 使用 CGEventTap (已有代码)
- Windows: 使用 SetWindowsHookEx + WH_KEYBOARD_LL

**文件: `src-tauri/src/hotkey/macos.rs`** (新建)

将现有代码移至 macOS 专用模块。

**文件: `src-tauri/src/hotkey/windows.rs`** (新建)

Windows 11 实现使用 `SetWindowsHookEx` + `WH_KEYBOARD_LL`：
```rust
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct WindowsHotkeyHandle;

extern "system" fn keyboard_hook_proc(
    n_code: i32, w_param: WPARAM, l_param: LPARAM
) -> LRESULT {
    // Check if key + modifiers match target hotkey
    // Invoke callback if matched
}
```

---

### 1.3 文本插入模块改造

**文件: `src-tauri/src/insertion/mod.rs`** (重构)

创建平台无关接口：
```rust
pub fn insert_text(text: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    return macos::insert_text(text);

    #[cfg(target_os = "windows")]
    return windows::insert_text(text);
}
```

**文件: `src-tauri/src/insertion/macos.rs`** (新建，从 clipboard.rs 移动)

使用 CGEvent 模拟 Cmd+V。

**文件: `src-tauri/src/insertion/windows.rs`** (新建)

使用 `SendInput` 模拟 Ctrl+V：
```rust
use windows::Win32::UI::Input::KeyboardAndMouse::*;

pub fn insert_text(text: &str) -> anyhow::Result<()> {
    // 1. Save current clipboard
    // 2. Set text to clipboard using arboard
    // 3. Send Ctrl+V using SendInput
    // 4. Restore original clipboard
}
```

---

### 1.4 截图模块改造

**文件: `src-tauri/src/screenshot/mod.rs`** (重构)

重构为跨平台接口：
```rust
pub fn capture_active_window() -> Result<Vec<u8>, String> {
    #[cfg(target_os = "macos")]
    return macos::capture_screen();

    #[cfg(target_os = "windows")]
    return windows::capture_screen();
}
```

**文件: `src-tauri/src/screenshot/macos.rs`** (新建)

使用 CGWindow API 截图。

**文件: `src-tauri/src/screenshot/windows.rs`** (新建)

使用 GDI (BitBlt) 截图：
```rust
use windows::Win32::Graphics::Gdi::*;

pub fn capture_screen() -> Result<Vec<u8>, String> {
    // 1. Get desktop window handle
    // 2. Create compatible DC and bitmap
    // 3. BitBlt screen to bitmap
    // 4. Convert to PNG bytes
}
```

---

### 1.5 权限模块改造

**文件: `src-tauri/src/permissions.rs`** (修改)

Windows 不需要显式权限请求：
```rust
#[cfg(target_os = "windows")]
pub fn request_microphone_permission() -> bool {
    // Windows prompts automatically on first use
    true
}

#[cfg(target_os = "windows")]
pub fn request_accessibility_permission() -> bool {
    // SendInput doesn't require special permissions on Windows
    true
}

#[cfg(target_os = "windows")]
pub fn check_screen_recording_permission() -> bool {
    // No permission system for screenshots on Windows
    true
}
```

---

### 1.6 日志模块改造

**文件: `src-tauri/src/lib.rs`** (修改)

Windows 使用不同的日志方法：
```rust
#[cfg(target_os = "macos")]
fn setup_file_logging() {
    // Use dup2 to redirect stdout/stderr to file
}

#[cfg(target_os = "windows")]
fn setup_file_logging() {
    // Windows doesn't support dup2, use simple approach
}
```

---

## Phase 2: 本地多模态模型支持

### 2.1 模型选择

对于 RTX 4060 (8GB VRAM)，推荐：

| 模型 | 大小 | VRAM需求 | 特点 |
|------|------|---------|------|
| **MiniCPM-V 2.6** | ~8GB | 6-8GB (INT4) | OCR极强，端侧SOTA |
| **Qwen2-VL 7B** | ~16GB | 6-8GB (INT4) | 中文强，通用能力强 |

**推荐：MiniCPM-V 2.6 INT4**
- 4060 可以流畅运行
- OCR 能力超越 GPT-4V 在很多场景
- 适合截图文字识别

---

### 2.2 模型下载配置

**文件: `src-tauri/src/config.rs`** (修改)

添加 vision 模型配置：
```rust
pub struct AppConfig {
    // ... 现有字段 ...

    /// Vision model path for local multimodal processing
    pub vision_model_path: String,

    /// Vision mode: "disabled" | "local" | "api"
    pub vision_mode: String,

    /// Max image dimension for vision model
    pub vision_max_image_size: u32,
}
```

默认路径：
- Windows: `%USERPROFILE%\.openclaw\models\minicpm-v-2_6\`
- macOS: `~/.openclaw/models/minicpm-v-2_6/`

---

### 2.3 Vision 引擎实现

**文件: `src-tauri/src/vision/mod.rs`** (新建)

创建 vision 处理模块：
```rust
pub trait VisionEngine: Send + Sync {
    fn process(&self, image: &[u8], prompt: &str) -> anyhow::Result<String>;
}

/// Load vision model (llama.cpp with multimodal projector)
pub fn load_vision_model(model_path: &str) -> anyhow::Result<Box<dyn VisionEngine>> {
    // Use llama-cpp-2 with vision support
    // Requires mmproj.bin for multimodal projector
}
```

**文件: `src-tauri/src/vision/llama_cpp.rs`** (新建)

使用 llama.cpp 的 vision 分支：
```rust
use llama_cpp_2::model::Model;

pub struct LlamaVisionEngine {
    model: Model,
    mmproj: MultimodalProjector, // llama.cpp specific
}

impl VisionEngine for LlamaVisionEngine {
    fn process(&self, image: &[u8], prompt: &str) -> anyhow::Result<String> {
        // 1. Load image
        // 2. Encode with CLIP vision encoder (via mmproj)
        // 3. Generate text with LLM
    }
}
```

---

### 2.4 集成到转写流程

**文件: `src-tauri/src/commands.rs`** (修改)

修改流程支持 vision：
```rust
// AI polishing / Vision processing
let polished = if !polish_enabled {
    corrected.clone()
} else {
    match polish_mode.as_str() {
        "local" => { /* existing local text model */ }
        "api" => { /* existing API */ }
        "vision" => {
            // New: Local vision model
            if let Some(ref vision_engine) = state.vision_engine {
                let screenshot = screenshot_result.as_ref()
                    .ok_or("Vision mode requires screenshot")?;
                let prompt = format!(
                    "User said: {}\n\nDescribe what you see in the screenshot and help complete the task.",
                    corrected
                );
                vision_engine.process(screenshot, &prompt)?
            } else {
                corrected.clone()
            }
        }
        _ => corrected.clone()
    }
};
```

---

### 2.5 设置界面更新

**文件: `settings.html`** (修改)

添加 vision 模式选项：
```html
<section class="field">
  <label>AI Polishing Engine / AI 润色引擎</label>
  <div class="radio-group">
    <label class="radio-label">
      <input type="radio" name="polish-mode" value="disabled" />
      <span>Disabled / 关闭</span>
    </label>
    <label class="radio-label">
      <input type="radio" name="polish-mode" value="local" />
      <span>Local Text Model / 本地文本模型</span>
    </label>
    <label class="radio-label">
      <input type="radio" name="polish-mode" value="vision" />
      <span>Local Vision Model / 本地视觉模型 ⭐</span>
    </label>
    <label class="radio-label">
      <input type="radio" name="polish-mode" value="api" />
      <span>Online API / 在线 API</span>
    </label>
  </div>
</section>

<!-- Vision model path (shown when vision mode selected) -->
<section class="field" id="vision-model-section" style="display: none;">
  <label>Vision Model Path / 视觉模型路径</label>
  <p class="hint">MiniCPM-V 2.6 or Qwen2-VL GGUF + mmproj.bin</p>
  <div class="row">
    <input type="text" id="vision-model-path" placeholder="Select model folder..." />
    <button type="button" id="browse-vision-btn">Browse</button>
  </div>
</section>
```

---

## Phase 3: 依赖和构建

### 3.1 Cargo.toml 更新

```toml
[dependencies]
# ... existing ...
once_cell = "1.19"  # For lazy static initialization

[target.'cfg(target_os = "macos")'.dependencies]
# ... existing macOS deps ...

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.52", features = [...] }
```

---

### 3.2 构建配置

**本地构建：**
```bash
# Windows build (需要 Windows 环境或交叉编译)
cargo tauri build --target x86_64-pc-windows-msvc

# macOS build (existing)
cargo tauri build --target x86_64-apple-darwin
cargo tauri build --target aarch64-apple-darwin
```

---

## 关键文件清单

| 文件路径 | 修改类型 | 说明 |
|---------|---------|------|
| `src-tauri/Cargo.toml` | 修改 | 添加 Windows 依赖 |
| `src-tauri/tauri.conf.json` | 修改 | 添加 Windows 配置 |
| `src-tauri/src/hotkey/mod.rs` | 重构 | 平台抽象接口 |
| `src-tauri/src/hotkey/keycode.rs` | 新建 | 平台无关键码定义 |
| `src-tauri/src/hotkey/macos.rs` | 新建 | macOS 热键实现 |
| `src-tauri/src/hotkey/windows.rs` | 新建 | Windows 热键实现 |
| `src-tauri/src/insertion/mod.rs` | 重构 | 平台抽象接口 |
| `src-tauri/src/insertion/macos.rs` | 新建 | macOS 文本插入 |
| `src-tauri/src/insertion/windows.rs` | 新建 | Windows 文本插入 |
| `src-tauri/src/screenshot/mod.rs` | 重构 | 平台抽象接口 |
| `src-tauri/src/screenshot/macos.rs` | 新建 | macOS 截图 |
| `src-tauri/src/screenshot/windows.rs` | 新建 | Windows 截图 |
| `src-tauri/src/permissions.rs` | 修改 | Windows stub 实现 |
| `src-tauri/src/lib.rs` | 修改 | Windows 日志和权限 |
| `src-tauri/src/vision/mod.rs` | 新建 | Vision 引擎接口 |
| `src-tauri/src/vision/llama_cpp.rs` | 新建 | llama.cpp vision 实现 |
| `src-tauri/src/config.rs` | 修改 | 添加 vision 配置 |
| `src-tauri/src/commands.rs` | 修改 | 集成 vision 流程 |
| `settings.html` | 修改 | 添加 vision UI |
| `src/settings.ts` | 修改 | 添加 vision 设置逻辑 |

---

## 验证清单

### Windows 测试清单

- [ ] 热键注册（Ctrl+Shift+S 等）
- [ ] 音频捕获（cpal WASAPI）
- [ ] 截图功能（GDI BitBlt）
- [ ] 文本插入（Ctrl+V 模拟）
- [ ] 配置保存/加载
- [ ] 系统托盘

### Vision 模型测试清单

- [ ] MiniCPM-V 2.6 模型加载
- [ ] 截图 + vision 处理流程
- [ ] 显存占用 < 8GB
- [ ] 处理速度 > 5 tokens/s
- [ ] OCR 准确性

---

## 注意事项

- **Windows 版本**: Windows 11，使用 GDI 截图（兼容性更好）
- **热键**: 支持自定义，自动适配平台（Command -> Ctrl，Option -> Alt）
- **优先级**: Phase 1 (Windows 迁移) 优先，Phase 2 (Vision) 可以后续迭代
- **Vision 模型**: 需要 llama.cpp 的 vision 分支，标准 crate 可能不支持，需要自定义构建
- **模型下载**: 提供脚本自动下载 MiniCPM-V 2.6 GGUF + mmproj.bin

---

## 当前状态

### 已完成
- ✅ Cargo.toml Windows 依赖
- ✅ Hotkey 跨平台抽象
- ✅ Insertion 跨平台抽象
- ✅ Screenshot 跨平台抽象
- ✅ Permissions Windows stub
- ✅ lib.rs Windows 兼容

### 待完成
- ⏳ Vision 模型支持 (Phase 2)
- ⏳ Windows 构建测试
- ⏳ 前端设置界面更新
