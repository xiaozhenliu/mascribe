export type UiLang = "en" | "zh";

// ── i18n dictionary ──

export const I18N: Record<UiLang, Record<string, string>> = {
  en: {
    page_title: "Settings",
    page_doc_title: "MaScribe Settings",
    shortcut_label: "Global Shortcut",
    shortcut_hint: "Click the box below and press your desired key combination",
    shortcut_placeholder: "Click and press keys...",
    shortcut_presets: "Presets ▾",
    shortcut_menu_key: "☰ Menu Key",
    shortcut_reset: "Reset",
    shortcut_reset_title: "Reset to default",
    shortcut_special_title: "Special keys",
    recordings_label: "Recordings Directory",
    recordings_hint: "Where WAV recordings of each speech input are saved",
    browse: "Browse...",
    polish_label: "AI Polishing Engine",
    polish_hint: "Choose how transcribed text is polished after speech recognition",
    polish_off: "Off",
    polish_local: "Local Model",
    polish_api: "Online API",
    api_settings_label: "Online API Settings",
    api_settings_hint: "OpenAI-compatible endpoint. Setup guide: docs/online-api-guide-en.md | docs/online-api-guide-zh.md",
    local_polish_settings_label: "Local Model Settings",
    local_polish_settings_hint: "Local mode expects a GGUF file path. Recommended: Qwen2.5-1.5B-Instruct GGUF.",
    local_polish_ollama_hint: 'If you use Ollama default install, choose "Online API" mode with endpoint http://localhost:11434/v1 and model qwen2.5:1.5b.',
    local_polish_model_path_placeholder: "/path/to/model.gguf",
    detect_ollama_models: "Detect Ollama Models",
    detecting_ollama_models: "Detecting...",
    detected_ollama_models: "Detected {count} Ollama models.",
    detect_ollama_models_failed: "Failed to detect Ollama models",
    test_connection: "Test Connection",
    testing_connection: "Testing...",
    connection_success: "✓ Connection successful (response time: {time}ms)",
    connection_failed: "✗ Connection failed: {error}",
    endpoint: "Endpoint",
    api_key: "API Key",
    model: "Model",
    polish_prompt_label: "AI Polish Prompt",
    polish_prompt_hint: "Template for text post-processing. Use {text} for transcribed text and {lang} for detected language.",
    ocr_label: "Screen OCR",
    ocr_hint: "OCR extracts text from screenshots, then AI polishing uses it as context to correct homophones",
    disabled: "Disabled",
    ollama_ocr: "Ollama OCR",
    ocr_settings_label: "OCR Settings",
    ocr_settings_hint: "Ollama + GLM-OCR recommended. Install: ollama pull glm-ocr",
    screenshot_label: "Screenshot Context",
    screenshot_hint: "Capture current window to provide visual context for AI",
    screenshot_save: "Save Only",
    screenshot_api: "Send to API",
    corrections_label: "Correction Dictionary",
    corrections_hint: "Auto-replace words after transcription (case-insensitive).",
    correction_from: "From",
    correction_to: "To",
    correction_add: "Add",
    save: "Save",
    press_keys: "Press keys...",
    no_rules: "No rules yet",
    delete: "Delete",
    load_failed: "Failed to load settings",
    save_ok: "Settings saved",
    save_failed: "Save failed",
    screenshot_saved_macos: "Screenshots will be saved to ~/Library/Application Support/com.mascribe/screenshots/",
    screenshot_saved_windows: "Screenshots will be saved to %APPDATA%/com.mascribe/screenshots/",
    screenshot_saved_other: "Screenshots will be saved to the app data screenshots folder.",
    screenshot_ocr_injected: "✓ Screenshot → OCR → screen context injected into AI polishing prompt for homophone correction.",
    screenshot_polish_off: "AI Polishing is disabled. Enable polishing + OCR to use screenshots for correction.",
    screenshot_need_ocr: "Enable Screen OCR above to use screenshots for context-aware correction.",
    native_macos: "macOS Built-in ⭐",
    native_windows: "Windows Built-in ⭐",
    native_other: "Native Built-in ⭐",
    vision_native_macos_hint: "Using macOS built-in text recognition (fast, no setup required)",
    vision_native_windows_hint: "Using Windows built-in text recognition (fast; install OCR language pack if needed)",
    vision_native_other_hint: "Using built-in system text recognition",
    vision_api_hint: "Using Ollama OCR model (requires Ollama running locally)",
    // Tab labels
    tab_basic: "Basic",
    tab_polish: "AI Polish",
    tab_vision: "Vision",
    tab_dictionary: "Dictionary",
  },
  zh: {
    page_title: "设置",
    page_doc_title: "MaScribe 设置",
    shortcut_label: "全局快捷键",
    shortcut_hint: "点击输入框后按下你想要的按键组合",
    shortcut_placeholder: "点击后按键...",
    shortcut_presets: "预设 ▾",
    shortcut_menu_key: "☰ 菜单键",
    shortcut_reset: "重置",
    shortcut_reset_title: "恢复默认值",
    shortcut_special_title: "特殊按键",
    recordings_label: "录音目录",
    recordings_hint: "每次语音输入生成的 WAV 文件保存位置",
    browse: "浏览...",
    polish_label: "AI 润色引擎",
    polish_hint: "选择语音识别后文本的润色方式",
    polish_off: "关闭",
    polish_local: "本地模型",
    polish_api: "在线 API",
    api_settings_label: "在线 API 设置",
    api_settings_hint: "OpenAI 兼容接口。配置指南：docs/online-api-guide-zh.md | docs/online-api-guide-en.md",
    local_polish_settings_label: "本地模型设置",
    local_polish_settings_hint: "本地模式需要填写 GGUF 模型文件路径。推荐：Qwen2.5-1.5B-Instruct GGUF。",
    local_polish_ollama_hint: '如果使用 Ollama 默认安装，请改用"在线 API"模式：Endpoint 填 http://localhost:11434/v1，Model 填 qwen2.5:1.5b。',
    local_polish_model_path_placeholder: "/path/to/model.gguf",
    detect_ollama_models: "识别 Ollama 模型",
    detecting_ollama_models: "识别中...",
    detected_ollama_models: "已识别 {count} 个 Ollama 模型。",
    detect_ollama_models_failed: "识别 Ollama 模型失败",
    test_connection: "测试连接",
    testing_connection: "测试中...",
    connection_success: "✓ 连接成功 (响应时间: {time}ms)",
    connection_failed: "✗ 连接失败: {error}",
    endpoint: "接口地址",
    api_key: "API 密钥",
    model: "模型",
    polish_prompt_label: "AI 润色提示词",
    polish_prompt_hint: "文本后处理模板。使用 {text} 表示转写文本，使用 {lang} 表示识别语言。",
    ocr_label: "屏幕 OCR",
    ocr_hint: "OCR 会从截图提取文字，AI 润色会利用这些上下文纠正同音字",
    disabled: "关闭",
    ollama_ocr: "Ollama OCR",
    ocr_settings_label: "OCR 设置",
    ocr_settings_hint: "推荐 Ollama + GLM-OCR。安装命令：ollama pull glm-ocr",
    screenshot_label: "截图上下文",
    screenshot_hint: "截图当前窗口，为 AI 提供视觉上下文",
    screenshot_save: "仅保存",
    screenshot_api: "发送到 API",
    corrections_label: "纠错词典",
    corrections_hint: "识别后自动替换词语（不区分大小写）。",
    correction_from: "原文",
    correction_to: "替换为",
    correction_add: "添加",
    save: "保存",
    press_keys: "请按键...",
    no_rules: "暂无规则",
    delete: "删除",
    load_failed: "加载设置失败",
    save_ok: "设置已保存",
    save_failed: "保存失败",
    screenshot_saved_macos: "截图将保存到 ~/Library/Application Support/com.mascribe/screenshots/",
    screenshot_saved_windows: "截图将保存到 %APPDATA%/com.mascribe/screenshots/",
    screenshot_saved_other: "截图将保存到应用数据目录中的 screenshots 文件夹。",
    screenshot_ocr_injected: "✓ 截图 → OCR → 屏幕文字已注入 AI 润色提示词用于同音字纠错。",
    screenshot_polish_off: "AI 润色已关闭。请启用“润色 + OCR”后再使用截图纠错。",
    screenshot_need_ocr: "请先在上方启用屏幕 OCR，再使用截图上下文纠错。",
    native_macos: "macOS 系统内置 ⭐",
    native_windows: "Windows 系统内置 ⭐",
    native_other: "系统内置 ⭐",
    vision_native_macos_hint: "使用 macOS 系统内置文字识别（快速，无需额外配置）",
    vision_native_windows_hint: "使用 Windows 系统内置文字识别（快速；若需中文请安装 OCR 语言包）",
    vision_native_other_hint: "使用系统内置文字识别",
    vision_api_hint: "使用 Ollama OCR 模型（需要本机启动 Ollama）",
    // Tab labels
    tab_basic: "基础",
    tab_polish: "AI 润色",
    tab_vision: "视觉",
    tab_dictionary: "词典",
  },
};

// ── Pure i18n helpers (take lang as param for testability) ──

export function t(key: string, lang: UiLang): string {
  return I18N[lang][key] || key;
}

export function tf(key: string, vars: Record<string, string | number>, lang: UiLang): string {
  let out = t(key, lang);
  for (const [k, v] of Object.entries(vars)) {
    out = out.split(`{${k}}`).join(String(v));
  }
  return out;
}

// ── Shortcut key mapping ──

/** Map browser key codes to Tauri key names. Returns null for lone modifiers or Escape. */
export function mapKey(event: KeyboardEvent): string | null {
  const { key, code } = event;

  // Skip lone modifier keys
  if (["Control", "Meta", "Alt", "Shift"].includes(key)) return null;

  // Special keys
  if (key === " ") return "Space";
  if (key === "Escape") return null; // cancel recording
  if (key === "ContextMenu") return "ContextMenu"; // Windows keyboard menu key
  if (key.startsWith("Arrow")) return key; // ArrowUp, ArrowLeft, etc.
  if (key.startsWith("F") && key.length >= 2 && key.length <= 3) return key; // F1-F12

  // Letter/number keys — use uppercase single char
  if (code.startsWith("Key")) return code.replace("Key", "");
  if (code.startsWith("Digit")) return code.replace("Digit", "");

  // Punctuation and other keys
  if (key === "Tab") return "Tab";
  if (key === "Enter") return "Enter";
  if (key === "Backspace") return "Backspace";
  if (key === "Delete") return "Delete";
  if (key === "[" || key === "]") return key;
  if (key === ";" || key === "'" || key === "," || key === "." || key === "/") return key;
  if (key === "-" || key === "=") return key;
  if (key === "`") return "`";

  return key.length === 1 ? key.toUpperCase() : key;
}

// ── Ollama URL helper ──

/** Convert an API endpoint URL to an Ollama /api/tags URL. */
export function toOllamaTagsUrl(endpoint: string): string {
  let e = endpoint.trim();
  if (!e) {
    e = "http://localhost:11434/v1";
  }
  e = e.replace(/\/chat\/completions\/?$/i, "");
  e = e.replace(/\/v1\/?$/i, "");
  return `${e}/api/tags`;
}
