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
}

// ── DOM refs ──
const shortcutInput = () => document.getElementById("shortcut") as HTMLInputElement;
const shortcutClear = () => document.getElementById("shortcut-clear") as HTMLButtonElement;
const shortcutPresets = () => document.getElementById("shortcut-presets") as HTMLSelectElement;
const recordingsDir = () => document.getElementById("recordings-dir") as HTMLInputElement;
const browseBtn = () => document.getElementById("browse-btn") as HTMLButtonElement;
const polishPrompt = () => document.getElementById("polish-prompt") as HTMLTextAreaElement;
const apiEndpoint = () => document.getElementById("api-endpoint") as HTMLInputElement;
const apiKey = () => document.getElementById("api-key") as HTMLInputElement;
const apiModel = () => document.getElementById("api-model") as HTMLInputElement;
const apiSettings = () => document.getElementById("api-settings") as HTMLElement;
const polishPromptSection = () => document.getElementById("polish-prompt-section") as HTMLElement;
const btnSave = () => document.getElementById("btn-save") as HTMLButtonElement;
let originalConfig: AppConfig | null = null;

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
    input.value = "Press keys...";
  });

  input.addEventListener("blur", () => {
    isListening = false;
    input.classList.remove("recording");
    // If value is still the placeholder, restore original
    if (input.value === "Press keys..." && originalConfig) {
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
  const prompt = polishPromptSection();

  if (!enabled) {
    api.classList.add("hidden");
    prompt.classList.add("hidden");
  } else if (mode === "api") {
    api.classList.remove("hidden");
    prompt.classList.remove("hidden");
  } else {
    // local mode
    api.classList.add("hidden");
    prompt.classList.remove("hidden");
  }
}

function setupPolishRadios() {
  const radios = document.querySelectorAll<HTMLInputElement>('input[name="polish-mode"]');
  for (const r of radios) {
    r.addEventListener("change", updatePolishVisibility);
  }
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
  } catch (e) {
    console.error("Failed to load config:", e);
    showToast("Failed to load settings", "error");
  }
}

async function saveConfig() {
  if (!originalConfig) return;

  const { enabled, mode } = getPolishMode();

  const updated: AppConfig = {
    ...originalConfig,
    shortcut: shortcutInput().value,
    recordings_dir: recordingsDir().value,
    polish_prompt: polishPrompt().value,
    polish_enabled: enabled,
    polish_mode: mode,
    api_endpoint: apiEndpoint().value.trim(),
    api_key: apiKey().value.trim(),
    api_model: apiModel().value.trim(),
  };

  try {
    await invoke("save_config", { config: updated });
    showToast("Settings saved", "success");
    // Close window after brief delay
    setTimeout(async () => {
      const win = getCurrentWindow();
      await win.close();
    }, 500);
  } catch (e) {
    console.error("Failed to save config:", e);
    showToast(`Save failed: ${e}`, "error");
  }
}

// ── Init ──

window.addEventListener("DOMContentLoaded", async () => {
  setupPolishRadios();
  await loadConfig();
  setupShortcutRecorder();
  await setupBrowse();

  btnSave().addEventListener("click", saveConfig);
});
