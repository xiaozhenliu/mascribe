import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";

// ── Config type (matches Rust AppConfig) ──
interface AppConfig {
  model_dir: string;
  language: string;
  num_threads: number;
  use_itn: boolean;
  polish_model_path: string;
  polish_enabled: boolean;
  polish_mode: string;
  recordings_dir: string;
  polish_prompt: string;
  shortcut: string;
  api_endpoint: string;
  api_key: string;
  api_model: string;
  screenshot_mode: string;
  screenshot_max_size: number;
  vision_model_path: string;
  vision_mode: string;
  vision_max_image_size: number;
  ocr_endpoint: string;
  ocr_model: string;
}

type AppPlatform = "macos" | "windows" | "linux" | "unknown";
type UiLang = "en" | "zh";

// ── DOM refs ──
const shortcutInput = () => document.getElementById("shortcut") as HTMLInputElement;
const shortcutClear = () => document.getElementById("shortcut-clear") as HTMLButtonElement;
const shortcutPresets = () => document.getElementById("shortcut-presets") as HTMLSelectElement;
const recordingsDir = () => document.getElementById("recordings-dir") as HTMLInputElement;
const browseBtn = () => document.getElementById("browse-btn") as HTMLButtonElement;
const polishModelPath = () => document.getElementById("polish-model-path") as HTMLInputElement;
const polishModelBrowseBtn = () => document.getElementById("polish-model-browse-btn") as HTMLButtonElement;
const polishPrompt = () => document.getElementById("polish-prompt") as HTMLTextAreaElement;
const apiEndpoint = () => document.getElementById("api-endpoint") as HTMLInputElement;
const apiKey = () => document.getElementById("api-key") as HTMLInputElement;
const apiModel = () => document.getElementById("api-model") as HTMLInputElement;
const apiModelSuggestions = () => document.getElementById("api-model-suggestions") as HTMLDataListElement;
const detectOllamaModelsBtn = () => document.getElementById("detect-ollama-models-btn") as HTMLButtonElement;
const detectOllamaModelsHint = () => document.getElementById("detect-ollama-models-hint") as HTMLElement;
const testApiConnectionBtn = () => document.getElementById("test-api-connection-btn") as HTMLButtonElement;
const testApiConnectionHint = () => document.getElementById("test-api-connection-hint") as HTMLElement;
const apiSettings = () => document.getElementById("api-settings") as HTMLElement;
const localPolishSettings = () => document.getElementById("local-polish-settings") as HTMLElement;
const polishPromptSection = () => document.getElementById("polish-prompt-section") as HTMLElement;
const screenshotHint = () => document.getElementById("screenshot-hint") as HTMLElement;
const visionHint = () => document.getElementById("vision-hint") as HTMLElement;
const visionNativeLabel = () => document.getElementById("vision-native-label") as HTMLElement;
const ocrSettings = () => document.getElementById("ocr-settings") as HTMLElement;
const ocrEndpoint = () => document.getElementById("ocr-endpoint") as HTMLInputElement;
const ocrModel = () => document.getElementById("ocr-model") as HTMLInputElement;
const languageSelect = () => document.getElementById("language-select") as HTMLSelectElement;
const btnSave = () => document.getElementById("btn-save") as HTMLButtonElement;
let originalConfig: AppConfig | null = null;
let currentPlatform: AppPlatform = "unknown";
let currentUiLang: UiLang = "en";

const I18N: Record<UiLang, Record<string, string>> = {
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
    local_polish_ollama_hint: '如果使用 Ollama 默认安装，请改用“在线 API”模式：Endpoint 填 http://localhost:11434/v1，Model 填 qwen2.5:1.5b。',
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
  },
};

function t(key: string): string {
  return I18N[currentUiLang][key] || key;
}

function tf(key: string, vars: Record<string, string | number>): string {
  let out = t(key);
  for (const [k, v] of Object.entries(vars)) {
    out = out.split(`{${k}}`).join(String(v));
  }
  return out;
}

function setText(id: string, key: string) {
  const el = document.getElementById(id);
  if (el) el.textContent = t(key);
}

function setPlaceholder(id: string, key: string) {
  const el = document.getElementById(id) as HTMLInputElement | null;
  if (el) el.placeholder = t(key);
}

function applyLanguage() {
  document.documentElement.lang = currentUiLang === "zh" ? "zh-CN" : "en";
  document.title = t("page_doc_title");
  setText("page-title", "page_title");
  setText("label-shortcut", "shortcut_label");
  setText("hint-shortcut", "shortcut_hint");
  setPlaceholder("shortcut", "shortcut_placeholder");
  setText("preset-title", "shortcut_presets");
  setText("preset-contextmenu", "shortcut_menu_key");
  setText("shortcut-clear", "shortcut_reset");
  shortcutClear().title = t("shortcut_reset_title");
  shortcutPresets().title = t("shortcut_special_title");
  setText("label-recordings", "recordings_label");
  setText("hint-recordings", "recordings_hint");
  setText("browse-btn", "browse");
  setText("label-polish", "polish_label");
  setText("hint-polish", "polish_hint");
  setText("polish-off", "polish_off");
  setText("polish-local", "polish_local");
  setText("polish-api", "polish_api");
  setText("label-api-settings", "api_settings_label");
  setText("hint-api-settings", "api_settings_hint");
  setText("label-local-polish-settings", "local_polish_settings_label");
  setText("hint-local-polish-settings", "local_polish_settings_hint");
  setText("hint-local-polish-ollama", "local_polish_ollama_hint");
  setPlaceholder("polish-model-path", "local_polish_model_path_placeholder");
  setText("polish-model-browse-btn", "browse");
  setText("detect-ollama-models-btn", "detect_ollama_models");
  setText("test-api-connection-btn", "test_connection");
  setText("api-endpoint-label", "endpoint");
  setText("api-key-label", "api_key");
  setText("api-model-label", "model");
  setText("label-polish-prompt", "polish_prompt_label");
  setText("hint-polish-prompt", "polish_prompt_hint");
  setText("label-ocr", "ocr_label");
  setText("hint-ocr", "ocr_hint");
  setText("vision-disabled", "disabled");
  setText("vision-api", "ollama_ocr");
  setText("label-ocr-settings", "ocr_settings_label");
  setText("hint-ocr-settings", "ocr_settings_hint");
  setText("ocr-endpoint-label", "endpoint");
  setText("ocr-model-label", "model");
  setText("label-screenshot", "screenshot_label");
  setText("hint-screenshot", "screenshot_hint");
  setText("screenshot-disabled", "disabled");
  setText("screenshot-save", "screenshot_save");
  setText("screenshot-api", "screenshot_api");
  setText("label-corrections", "corrections_label");
  setText("hint-corrections", "corrections_hint");
  setPlaceholder("correction-from", "correction_from");
  setPlaceholder("correction-to", "correction_to");
  setText("correction-add-btn", "correction_add");
  setText("btn-save", "save");
  updateVisionNativeLabel();
  updateVisionVisibility();
  updateScreenshotHint();
  renderCorrections();
}

async function detectPlatform() {
  try {
    const p = (await invoke("get_platform")) as string;
    if (p === "macos" || p === "windows" || p === "linux") {
      currentPlatform = p;
      return;
    }
  } catch (e) {
    console.warn("Failed to detect platform:", e);
  }
  currentPlatform = "unknown";
}

function detectUiLanguage() {
  const saved = localStorage.getItem("settings_lang");
  if (saved === "en" || saved === "zh") {
    currentUiLang = saved;
    return;
  }
  const navLang = (navigator.language || "en").toLowerCase();
  currentUiLang = navLang.startsWith("zh") ? "zh" : "en";
}

function setupLanguageSelector() {
  const select = languageSelect();
  select.value = currentUiLang;
  select.addEventListener("change", () => {
    currentUiLang = select.value === "zh" ? "zh" : "en";
    localStorage.setItem("settings_lang", currentUiLang);
    applyLanguage();
  });
}

function updateVisionNativeLabel() {
  const label = visionNativeLabel();
  if (currentPlatform === "windows") {
    label.textContent = t("native_windows");
  } else if (currentPlatform === "macos") {
    label.textContent = t("native_macos");
  } else {
    label.textContent = t("native_other");
  }
}

// ── Shortcut key recorder ──

// Map browser key codes to Tauri key names
function mapKey(event: KeyboardEvent): string | null {
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

function setupShortcutRecorder() {
  const input = shortcutInput();
  let isListening = false;

  input.addEventListener("focus", () => {
    isListening = true;
    input.classList.add("recording");
    input.value = t("press_keys");
  });

  input.addEventListener("blur", () => {
    isListening = false;
    input.classList.remove("recording");
    // If value is still the placeholder, restore original
    if (input.value === t("press_keys") && originalConfig) {
      input.value = originalConfig.shortcut;
    }
  });

  input.addEventListener("keydown", (e) => {
    if (!isListening) return;
    e.preventDefault();
    e.stopPropagation();

    // Escape cancels recording
    if (e.key === "Escape") {
      input.blur();
      return;
    }

    const mainKey = mapKey(e);
    if (!mainKey) return; // Still pressing only modifiers

    // Build modifier string
    const parts: string[] = [];
    if (e.ctrlKey || e.metaKey) parts.push("CmdOrCtrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    parts.push(mainKey);

    input.value = parts.join("+");
    input.blur();
  });

  // Handle ContextMenu key: on macOS, pressing the Application/ContextMenu key
  // on a Windows keyboard fires a "contextmenu" event but NOT a "keydown" event.
  // So we must capture it here as the primary recording path for this key.
  input.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    if (!isListening) return;
    input.value = "ContextMenu";
    input.blur();
  });

  shortcutClear().addEventListener("click", () => {
    input.value = "Alt+Space";
  });

  // Preset dropdown for special keys (ContextMenu, etc.) that can't be
  // captured via keydown in WebView
  shortcutPresets().addEventListener("change", (e) => {
    const select = e.target as HTMLSelectElement;
    if (select.value) {
      input.value = select.value;
      // Reset select to placeholder so it can be re-selected
      select.selectedIndex = 0;
    }
  });
}

// ── Folder picker ──

async function setupBrowse() {
  browseBtn().addEventListener("click", async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: recordingsDir().value || undefined,
      });
      if (selected && typeof selected === "string") {
        recordingsDir().value = selected;
      }
    } catch (e) {
      console.error("browse error:", e);
    }
  });

  polishModelBrowseBtn().addEventListener("click", async () => {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        defaultPath: polishModelPath().value || undefined,
        filters: [{ name: "GGUF", extensions: ["gguf"] }],
      });
      if (selected && typeof selected === "string") {
        polishModelPath().value = selected;
      }
    } catch (e) {
      console.error("polish model browse error:", e);
    }
  });
}

// ── Polish mode radio ──

/** Get current polish mode from radio buttons */
function getPolishMode(): { enabled: boolean; mode: string } {
  const radios = document.querySelectorAll<HTMLInputElement>('input[name="polish-mode"]');
  for (const r of radios) {
    if (r.checked) {
      return r.value === "disabled"
        ? { enabled: false, mode: "local" }
        : { enabled: true, mode: r.value };
    }
  }
  return { enabled: true, mode: "local" };
}

/** Set polish mode radio and update section visibility */
function setPolishMode(enabled: boolean, mode: string) {
  const value = enabled ? mode : "disabled";
  const radios = document.querySelectorAll<HTMLInputElement>('input[name="polish-mode"]');
  for (const r of radios) {
    r.checked = r.value === value;
  }
  updatePolishVisibility();
}

/** Show/hide API settings and polish prompt based on selected mode */
function updatePolishVisibility() {
  const { enabled, mode } = getPolishMode();
  const api = apiSettings();
  const local = localPolishSettings();
  const prompt = polishPromptSection();

  if (!enabled) {
    api.classList.add("hidden");
    local.classList.add("hidden");
    prompt.classList.add("hidden");
  } else if (mode === "api") {
    api.classList.remove("hidden");
    local.classList.add("hidden");
    prompt.classList.remove("hidden");
  } else {
    // local mode
    api.classList.add("hidden");
    local.classList.remove("hidden");
    prompt.classList.remove("hidden");
  }
}

function setupPolishRadios() {
  const radios = document.querySelectorAll<HTMLInputElement>('input[name="polish-mode"]');
  for (const r of radios) {
    r.addEventListener("change", () => {
      updatePolishVisibility();
      updateScreenshotHint();
    });
  }
}

function toOllamaTagsUrl(endpoint: string): string {
  let e = endpoint.trim();
  if (!e) {
    e = "http://localhost:11434/v1";
  }
  e = e.replace(/\/chat\/completions\/?$/i, "");
  e = e.replace(/\/v1\/?$/i, "");
  return `${e}/api/tags`;
}

function setOllamaSuggestions(models: string[]) {
  const list = apiModelSuggestions();
  list.innerHTML = "";
  for (const m of models) {
    const opt = document.createElement("option");
    opt.value = m;
    list.appendChild(opt);
  }
}

async function detectOllamaModels() {
  const btn = detectOllamaModelsBtn();
  const hint = detectOllamaModelsHint();
  const oldText = btn.textContent || t("detect_ollama_models");
  btn.disabled = true;
  btn.textContent = t("detecting_ollama_models");
  hint.textContent = "";

  try {
    const url = toOllamaTagsUrl(apiEndpoint().value);
    const res = await fetch(url);
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}`);
    }
    const data = (await res.json()) as { models?: Array<{ name?: string }> };
    const names = (data.models || [])
      .map((x) => (x.name || "").trim())
      .filter(Boolean);

    if (names.length === 0) {
      throw new Error("No models found");
    }

    setOllamaSuggestions(names);
    if (!apiModel().value.trim()) {
      apiModel().value = names[0];
    }
    hint.textContent = tf("detected_ollama_models", { count: names.length });
  } catch (e) {
    console.error("Detect ollama models failed:", e);
    hint.textContent = `${t("detect_ollama_models_failed")}: ${e}`;
  } finally {
    btn.disabled = false;
    btn.textContent = oldText;
  }
}

function setupOllamaModelDetection() {
  detectOllamaModelsBtn().addEventListener("click", () => {
    void detectOllamaModels();
  });
}

async function testApiConnection() {
  const btn = testApiConnectionBtn();
  const hint = testApiConnectionHint();
  const oldText = btn.textContent || t("test_connection");
  btn.disabled = true;
  btn.textContent = t("testing_connection");
  hint.textContent = "";

  try {
    const endpoint = apiEndpoint().value.trim();
    const key = apiKey().value.trim();
    const model = apiModel().value.trim();

    if (!endpoint || !key || !model) {
      throw new Error("Please fill in all API fields");
    }

    const result = await invoke("test_online_api_connection", {
      endpoint,
      apiKey: key,
      model,
    }) as { success: boolean; response_time_ms: number; error_message?: string };

    if (result.success) {
      hint.textContent = tf("connection_success", { time: result.response_time_ms });
      hint.style.color = "#4caf50";
    } else {
      hint.textContent = tf("connection_failed", { error: result.error_message || "Unknown error" });
      hint.style.color = "#f44336";
    }
  } catch (e) {
    hint.textContent = tf("connection_failed", { error: String(e) });
    hint.style.color = "#f44336";
  } finally {
    btn.disabled = false;
    btn.textContent = oldText;
  }
}

function setupApiConnectionTest() {
  testApiConnectionBtn().addEventListener("click", () => {
    void testApiConnection();
  });
}

// ── Screenshot mode ──

/** Get current screenshot mode from radio buttons */
function getScreenshotMode(): string {
  const radios = document.querySelectorAll<HTMLInputElement>('input[name="screenshot-mode"]');
  for (const r of radios) {
    if (r.checked) {
      return r.value;
    }
  }
  return "disabled";
}

/** Set screenshot mode radio */
function setScreenshotMode(mode: string) {
  const radios = document.querySelectorAll<HTMLInputElement>('input[name="screenshot-mode"]');
  for (const r of radios) {
    r.checked = r.value === mode;
  }
  updateScreenshotHint();
}

/** Update screenshot hint based on selected mode and polish mode */
function updateScreenshotHint() {
  const screenshotMode = getScreenshotMode();
  const { enabled: polishEnabled } = getPolishMode();
  const visionMode = getVisionMode();
  const hint = screenshotHint();

  if (screenshotMode === "disabled") {
    hint.textContent = "";
  } else if (screenshotMode === "save") {
    if (currentPlatform === "windows") {
      hint.textContent = t("screenshot_saved_windows");
    } else if (currentPlatform === "macos") {
      hint.textContent = t("screenshot_saved_macos");
    } else {
      hint.textContent = t("screenshot_saved_other");
    }
  } else if (screenshotMode === "api") {
    if (visionMode !== "disabled") {
      hint.textContent = t("screenshot_ocr_injected");
    } else if (!polishEnabled) {
      hint.textContent = t("screenshot_polish_off");
    } else {
      hint.textContent = t("screenshot_need_ocr");
    }
  }
}

function setupScreenshotRadios() {
  const radios = document.querySelectorAll<HTMLInputElement>('input[name="screenshot-mode"]');
  for (const r of radios) {
    r.addEventListener("change", updateScreenshotHint);
  }
}

// ── Vision mode ──

/** Get current vision mode from radio buttons */
function getVisionMode(): string {
  const radios = document.querySelectorAll<HTMLInputElement>('input[name="vision-mode"]');
  for (const r of radios) {
    if (r.checked) {
      return r.value;
    }
  }
  return "disabled";
}

/** Set vision mode radio */
function setVisionMode(mode: string) {
  const radios = document.querySelectorAll<HTMLInputElement>('input[name="vision-mode"]');
  for (const r of radios) {
    r.checked = r.value === mode;
  }
  updateVisionVisibility();
}

/** Update vision/OCR UI visibility and hint */
function updateVisionVisibility() {
  const mode = getVisionMode();
  const ocr = ocrSettings();
  const hint = visionHint();

  if (mode === "disabled") {
    ocr.classList.add("hidden");
    hint.textContent = "";
  } else if (mode === "native") {
    ocr.classList.add("hidden");
    if (currentPlatform === "windows") {
      hint.textContent = t("vision_native_windows_hint");
    } else if (currentPlatform === "macos") {
      hint.textContent = t("vision_native_macos_hint");
    } else {
      hint.textContent = t("vision_native_other_hint");
    }
  } else {
    // Ollama API OCR — show endpoint/model config
    ocr.classList.remove("hidden");
    hint.textContent = t("vision_api_hint");
  }
}

function setupVisionRadios() {
  const radios = document.querySelectorAll<HTMLInputElement>('input[name="vision-mode"]');
  for (const r of radios) {
    r.addEventListener("change", () => {
      updateVisionVisibility();
      updateScreenshotHint();
    });
  }
}


// ── Correction Dictionary ──

let corrections: [string, string][] = [];

async function loadCorrections() {
  try {
    corrections = (await invoke("get_corrections")) as [string, string][];
    renderCorrections();
  } catch (e) {
    console.error("Failed to load corrections:", e);
  }
}

function renderCorrections() {
  const list = document.getElementById("corrections-list")!;
  // Clear existing content
  list.textContent = "";

  if (corrections.length === 0) {
    const empty = document.createElement("p");
    empty.className = "corrections-empty";
    empty.textContent = t("no_rules");
    list.appendChild(empty);
    return;
  }

  for (let i = 0; i < corrections.length; i++) {
    const [from, to] = corrections[i];
    const row = document.createElement("div");
    row.className = "correction-row";

    const fromSpan = document.createElement("span");
    fromSpan.className = "correction-from";
    fromSpan.textContent = from;

    const arrow = document.createElement("span");
    arrow.className = "corrections-arrow";
    arrow.textContent = "→";

    const toSpan = document.createElement("span");
    toSpan.className = "correction-to";
    toSpan.textContent = to;

    const deleteBtn = document.createElement("button");
    deleteBtn.className = "correction-delete";
    deleteBtn.textContent = "✕";
    deleteBtn.title = t("delete");
    deleteBtn.addEventListener("click", () => {
      corrections.splice(i, 1);
      renderCorrections();
    });

    row.appendChild(fromSpan);
    row.appendChild(arrow);
    row.appendChild(toSpan);
    row.appendChild(deleteBtn);
    list.appendChild(row);
  }
}

function setupCorrections() {
  const addBtn = document.getElementById("correction-add-btn")!;
  const fromInput = document.getElementById("correction-from") as HTMLInputElement;
  const toInput = document.getElementById("correction-to") as HTMLInputElement;

  const addRule = () => {
    const from = fromInput.value.trim();
    const to = toInput.value.trim();
    if (!from) return;
    corrections.push([from, to]);
    fromInput.value = "";
    toInput.value = "";
    fromInput.focus();
    renderCorrections();
  };

  addBtn.addEventListener("click", addRule);

  // Allow pressing Enter in the "to" field to add the rule
  toInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      addRule();
    }
  });
}

// ── Toast notification ──

function showToast(message: string, type: "success" | "error") {
  let toast = document.querySelector(".toast") as HTMLElement;
  if (!toast) {
    toast = document.createElement("div");
    toast.className = "toast";
    document.body.appendChild(toast);
  }
  toast.textContent = message;
  toast.className = `toast ${type} visible`;
  setTimeout(() => toast.classList.remove("visible"), 2000);
}

// ── Load / Save ──

async function loadConfig() {
  try {
    const config = (await invoke("get_config")) as AppConfig;
    originalConfig = config;

    shortcutInput().value = config.shortcut;
    recordingsDir().value = config.recordings_dir;
    polishPrompt().value = config.polish_prompt;

    // Polish engine mode
    setPolishMode(config.polish_enabled, config.polish_mode || "local");

    // API config
    apiEndpoint().value = config.api_endpoint || "";
    apiKey().value = config.api_key || "";
    apiModel().value = config.api_model || "";
    polishModelPath().value = config.polish_model_path || "";

    // Screenshot mode
    setScreenshotMode(config.screenshot_mode || "disabled");

    // Vision/OCR mode
    setVisionMode(config.vision_mode || "disabled");
    ocrEndpoint().value = config.ocr_endpoint || "http://localhost:11434/v1";
    ocrModel().value = config.ocr_model || "glm-ocr";
  } catch (e) {
    console.error("Failed to load config:", e);
    showToast(t("load_failed"), "error");
  }
}

async function saveConfig() {
  if (!originalConfig) return;

  const { enabled, mode } = getPolishMode();
  const screenshotMode = getScreenshotMode();
  const visionMode = getVisionMode();

  const updated: AppConfig = {
    ...originalConfig,
    shortcut: shortcutInput().value,
    recordings_dir: recordingsDir().value,
    polish_prompt: polishPrompt().value,
    polish_enabled: enabled,
    polish_mode: mode,
    polish_model_path: polishModelPath().value.trim(),
    api_endpoint: apiEndpoint().value.trim(),
    api_key: apiKey().value.trim(),
    api_model: apiModel().value.trim(),
    screenshot_mode: screenshotMode,
    screenshot_max_size: originalConfig.screenshot_max_size || 1024,
    vision_mode: visionMode,
    vision_model_path: originalConfig.vision_model_path || "",
    vision_max_image_size: originalConfig.vision_max_image_size || 448,
    ocr_endpoint: ocrEndpoint().value.trim() || "http://localhost:11434/v1",
    ocr_model: ocrModel().value.trim() || "glm-ocr",
  };

  try {
    await invoke("save_config", { config: updated });
    await invoke("save_corrections", { entries: corrections });
    showToast(t("save_ok"), "success");
    // Close window after brief delay
    setTimeout(async () => {
      const win = getCurrentWindow();
      await win.close();
    }, 500);
  } catch (e) {
    console.error("Failed to save config:", e);
    showToast(`${t("save_failed")}: ${e}`, "error");
  }
}

// ── Init ──

window.addEventListener("DOMContentLoaded", async () => {
  detectUiLanguage();
  setupLanguageSelector();
  await detectPlatform();
  applyLanguage();
  setupPolishRadios();
  setupOllamaModelDetection();
  setupApiConnectionTest();
  setupScreenshotRadios();
  setupVisionRadios();
  await loadConfig();
  await loadCorrections();
  setupShortcutRecorder();
  setupCorrections();
  await setupBrowse();

  btnSave().addEventListener("click", saveConfig);
});
