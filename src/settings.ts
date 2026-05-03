import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { t as tUtil, tf as tfUtil, mapKey, toOllamaTagsUrl } from "./utils";
import type { UiLang } from "./utils";

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

// ── DOM refs ──
const shortcutInput = () => document.getElementById("shortcut") as HTMLInputElement;
const shortcutClear = () => document.getElementById("shortcut-clear") as HTMLButtonElement;
const shortcutPresets = () => document.getElementById("shortcut-presets") as HTMLSelectElement;
const recordingsDir = () => document.getElementById("recordings-dir") as HTMLInputElement;
const browseBtn = () => document.getElementById("browse-btn") as HTMLButtonElement;
const polishModelPath = () => document.getElementById("polish-model-path") as HTMLInputElement;
const polishModelBrowseBtn = () => document.getElementById("polish-model-browse-btn") as HTMLButtonElement;
const polishPrompt = () => document.getElementById("polish-prompt") as HTMLTextAreaElement;
const promptDisabledHint = () => document.getElementById("prompt-disabled-hint") as HTMLElement;
const apiEndpoint = () => document.getElementById("api-endpoint") as HTMLInputElement;
const apiKey = () => document.getElementById("api-key") as HTMLInputElement;
const apiModel = () => document.getElementById("api-model") as HTMLInputElement;
const apiModelSuggestions = () => document.getElementById("api-model-suggestions") as HTMLDataListElement;
const detectOllamaModelsBtn = () => document.getElementById("detect-ollama-models-btn") as HTMLButtonElement;
const detectOllamaModelsHint = () => document.getElementById("detect-ollama-models-hint") as HTMLElement;
const testApiConnectionBtn = () => document.getElementById("test-api-connection-btn") as HTMLButtonElement;
const testApiConnectionHint = () => document.getElementById("test-api-connection-hint") as HTMLElement;
const apiGuideBtn = () => document.getElementById("api-guide-btn") as HTMLButtonElement;
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

function t(key: string): string {
  return tUtil(key, currentUiLang);
}

function tf(key: string, vars: Record<string, string | number>): string {
  return tfUtil(key, vars, currentUiLang);
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
  setText("hint-api-settings", "api_settings_hint");
  setText("api-guide-btn", "api_guide_btn");
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
  setText("tab-label-basic", "tab_basic");
  setText("tab-label-polish", "tab_polish");
  setText("tab-label-prompt", "tab_prompt");
  setText("tab-label-vision", "tab_vision");
  setText("tab-label-dictionary", "tab_dictionary");
  setText("prompt-disabled-hint", "prompt_disabled_hint");
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
  const hint = promptDisabledHint();

  if (!enabled) {
    api.classList.add("hidden");
    local.classList.add("hidden");
    prompt.classList.add("hidden");
    hint.classList.remove("hidden");
  } else if (mode === "api") {
    api.classList.remove("hidden");
    local.classList.add("hidden");
    prompt.classList.remove("hidden");
    hint.classList.add("hidden");
  } else {
    // local mode
    api.classList.add("hidden");
    local.classList.remove("hidden");
    prompt.classList.remove("hidden");
    hint.classList.add("hidden");
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
    hint.style.color = "#34c759";
    scheduleHintClear(hint);
  } catch (e) {
    console.error("Detect ollama models failed:", e);
    hint.textContent = `${t("detect_ollama_models_failed")}: ${e}`;
    hint.style.color = "#ff3b30";
    scheduleHintClear(hint);
  } finally {
    btn.disabled = false;
    btn.textContent = oldText;
  }
}

function scheduleHintClear(el: HTMLElement) {
  setTimeout(() => { el.textContent = ""; el.style.color = ""; }, 15000);
}

function setupOllamaModelDetection() {
  detectOllamaModelsBtn().addEventListener("click", () => {
    void detectOllamaModels();
  });
}

function setupApiGuide() {
  apiGuideBtn().addEventListener("click", async () => {
    const lang = currentUiLang === "zh" ? "zh" : "en";
    const url = `https://github.com/user/mascribe/blob/main/docs/online-api-guide-${lang}.md`;
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(url);
    } catch {
      window.open(url, "_blank");
    }
  });
}

async function testApiConnection() {
  const btn = testApiConnectionBtn();
  const hint = testApiConnectionHint();
  const oldText = btn.textContent || t("test_connection");
  btn.disabled = true;
  btn.textContent = t("testing_connection");
  hint.textContent = t("testing_connection");
  hint.style.color = "#aaa";

  // Push invoke to next macrotask so WKWebView renders the DOM update first
  await new Promise(resolve => setTimeout(resolve, 0));

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
    await new Promise(resolve => setTimeout(resolve, 300));
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

// ── Tab navigation ──

const VALID_TABS = ["basic", "polish", "prompt", "vision", "dictionary"] as const;
type TabId = (typeof VALID_TABS)[number];

function setupTabs() {
  const tabButtons = document.querySelectorAll<HTMLButtonElement>(".tab-item[data-tab]");
  const tabPanels = document.querySelectorAll<HTMLElement>(".tab-panel[data-tab]");

  // Restore last active tab from localStorage
  const savedTab = localStorage.getItem("settings_active_tab");
  const initialTab: TabId = VALID_TABS.includes(savedTab as TabId)
    ? (savedTab as TabId)
    : "basic";

  function switchTab(tabId: TabId) {
    // Update buttons
    tabButtons.forEach((btn) => {
      btn.classList.toggle("active", btn.dataset["tab"] === tabId);
    });
    // Update panels
    tabPanels.forEach((panel) => {
      panel.classList.toggle("active", panel.dataset["tab"] === tabId);
    });
    // Persist
    localStorage.setItem("settings_active_tab", tabId);
  }

  // Click handler for each tab button
  tabButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
      const tabId = btn.dataset["tab"] as TabId;
      if (VALID_TABS.includes(tabId)) {
        switchTab(tabId);
      }
    });
  });

  // Apply initial tab
  switchTab(initialTab);
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
  setupTabs();
  setupPolishRadios();
  setupOllamaModelDetection();
  setupApiGuide();
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
