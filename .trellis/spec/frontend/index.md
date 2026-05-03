# Frontend Development Guidelines

> Conventions for the Mascribe frontend: vanilla TypeScript + direct DOM manipulation, no framework.

---

## Tech Stack

- **TypeScript** (strict mode via `tsconfig.json`)
- **Vite** as bundler
- **No UI framework** — no React, Vue, Svelte, or similar
- **No CSS framework** — styles are inline in HTML files (see CLAUDE.md)
- Tauri `@tauri-apps/api` for Rust communication

---

## DOM Access Pattern

DOM elements are accessed through **arrow-function refs** at the top of the file. This centralizes element references and avoids repeated `getElementById` calls scattered through logic.

```ts
const dot = () => document.getElementById("status-dot")!;
const canvas = () => document.getElementById("waveform") as HTMLCanvasElement;
const btnSave = () => document.getElementById("btn-save") as HTMLButtonElement;
```

**Rules:**
- Use `document.getElementById()` — never `querySelector` for IDs
- Use non-null assertion (`!`) for elements guaranteed to exist in HTML
- Use `as HTMLInputElement` etc. when the specific element type matters (form controls)
- Group all DOM refs at the top of the file, before any logic

---

## State Management

Module-level variables. No state library, no framework context.

```ts
let isRecording = false;
let isProcessing = false;
let currentPlatform: AppPlatform = "unknown";
let corrections: [string, string][] = [];
```

- Boolean flags for UI state (`isRecording`, `isProcessing`)
- `let` for mutable state, `const` for immutable refs and configuration
- Timers stored as `number | null` with manual cleanup pattern

---

## Tauri Communication

Use `invoke()` from `@tauri-apps/api/core` to call Rust commands. Cast the return type explicitly.

```ts
import { invoke } from "@tauri-apps/api/core";

const config = await invoke("get_config") as AppConfig;
const result = await invoke("stop_recording_and_transcribe") as string;
```

**Rules:**
- Always cast `invoke()` return with `as Type`
- Use `.catch(() => {})` for fire-and-forget calls (e.g., `invoke("hide_window").catch(() => {})`)
- Use `listen()` from `@tauri-apps/api/event` for Rust → frontend events (dynamic import is OK)

---

## Naming Conventions

| Element | Convention | Example |
|---------|-----------|---------|
| Functions | camelCase | `toggleRecording`, `showToast` |
| Constants | UPPER_SNAKE_CASE | `WAVEFORM_POINTS`, `AGC_ATTACK` |
| DOM IDs | kebab-case | `status-dot`, `btn-save` |
| Type aliases | PascalCase | `AppPlatform`, `UiLang` |
| Interfaces | PascalCase | `AppConfig` |
| Event listeners | `setup` prefix | `setupShortcutRecorder`, `setupBrowse` |

---

## Timer Cleanup Pattern

Timers are stored in module-level variables with explicit cleanup. Always check for existing timer before creating a new one.

```ts
let toastTimer: number | null = null;

function hideToast() {
  toast().className = "toast hidden";
  if (toastTimer !== null) {
    clearTimeout(toastTimer);
    toastTimer = null;
  }
}
```

---

## Internationalization (i18n)

Settings page uses a manual `I18N` dictionary with `en` and `zh` keys. Language detected from `navigator.language`, persisted in `localStorage`.

```ts
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
```

- `t(key)` for simple strings
- `tf(key, {var: value})` for templated strings with `{placeholder}` substitution
- Use `setText(id, key)` and `setPlaceholder(id, key)` helpers for bulk DOM updates

---

## Initialization

All pages use a single `DOMContentLoaded` listener that calls setup functions in order. Async setup functions are awaited sequentially.

```ts
window.addEventListener("DOMContentLoaded", async () => {
  detectUiLanguage();
  setupLanguageSelector();
  await detectPlatform();
  applyLanguage();
  setupPolishRadios();
  await loadConfig();
  btnSave().addEventListener("click", saveConfig);
});
```

---

## File Organization

Each HTML page has its own TS entry point. No shared modules between pages — keep logic self-contained per page.

| File | Purpose |
|------|---------|
| `src/main.ts` | Main overlay (recording UI, waveform, hotkey) |
| `src/settings.ts` | Settings window (config editing, corrections) |

---

## Error Handling

- `try/catch` around all `invoke()` calls
- Errors shown to user via `showToast(msg, "error")`
- Console logging for debugging: `console.error("context:", e)`
- Backend errors come as `String` — display first 30 chars to user if long

---

## Common Mistakes to Avoid

1. **Don't use `querySelector`** when `getElementById` suffices
2. **Don't forget timer cleanup** — every `setTimeout`/`setInterval` must have a corresponding cleanup path
3. **Don't add CSS files** — styles go inline in HTML
4. **Don't introduce a framework** — this is vanilla TS by design
5. **Don't use `async` in event callbacks without handling errors** — wrap in try/catch
