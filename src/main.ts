import { invoke } from "@tauri-apps/api/core";
import { register, unregister } from "@tauri-apps/plugin-global-shortcut";

let isRecording = false;
let isProcessing = false; // true while transcribing+polishing — blocks new recordings
let amplitudeTimer: number | null = null;
let hideTimer: number | null = null;

const WAVEFORM_POINTS = 24;
const waveHeights: number[] = new Array(WAVEFORM_POINTS).fill(0);
const waveTargets: number[] = new Array(WAVEFORM_POINTS).fill(0);
let currentAmplitude = 0;

// ── Auto-gain control (AGC) ──
// Tracks recent peak so the waveform always fills the display,
// regardless of how loudly or quietly the user speaks.
let recentPeak = 0.01; // floor to avoid division by zero
const AGC_ATTACK = 0.3;  // how fast peak rises to a louder signal
const AGC_DECAY = 0.985; // how slowly peak fades (per 50ms tick → ~1.5s half-life)

const dot = () => document.getElementById("status-dot")!;
const canvas = () => document.getElementById("waveform") as HTMLCanvasElement;
const toast = () => document.getElementById("toast")!;

// ── Window visibility helpers ──

function cancelScheduledHide() {
  if (hideTimer !== null) {
    clearTimeout(hideTimer);
    hideTimer = null;
  }
}

function scheduleHide(delayMs: number) {
  cancelScheduledHide();
  hideTimer = window.setTimeout(() => {
    invoke("hide_window").catch(() => {});
    hideTimer = null;
  }, delayMs);
}

// ── Toast helpers ──

let toastTimer: number | null = null;

function showToast(text: string, cls?: "success" | "error") {
  const t = toast();
  t.textContent = text;
  t.className = "toast" + (cls ? ` ${cls}` : "");
  // Auto-clear previous timer
  if (toastTimer !== null) clearTimeout(toastTimer);
}

function hideToast() {
  toast().className = "toast hidden";
  toast().textContent = "";
  if (toastTimer !== null) {
    clearTimeout(toastTimer);
    toastTimer = null;
  }
}

// ── State transitions ──

function showIdle() {
  dot().className = "dot";
  canvas().classList.remove("hidden");
  waveHeights.fill(0);
  waveTargets.fill(0);
  currentAmplitude = 0;
  drawWaveform();
  hideToast();
}

function showRecording() {
  dot().className = "dot recording";
  canvas().classList.remove("hidden");
  hideToast();
}

let processingStart = 0;
let processingTimer: number | null = null;

function showProcessing() {
  dot().className = "dot processing";
  canvas().classList.add("hidden");
  // Show elapsed time in toast
  processingStart = Date.now();
  showToast("Processing...");
  if (processingTimer !== null) clearInterval(processingTimer);
  processingTimer = window.setInterval(() => {
    const elapsed = ((Date.now() - processingStart) / 1000).toFixed(1);
    showToast(`Processing... ${elapsed}s`);
  }, 200);
}

function stopProcessingTimer() {
  if (processingTimer !== null) {
    clearInterval(processingTimer);
    processingTimer = null;
  }
}

function showError(msg: string) {
  dot().className = "dot";
  canvas().classList.add("hidden");
  showToast(msg, "error");
  toastTimer = window.setTimeout(() => {
    showIdle();
  }, 2000);
  scheduleHide(2500);
}

function showSuccess(text: string) {
  dot().className = "dot";
  canvas().classList.add("hidden");
  const display = text.length > 30 ? text.substring(0, 30) + "…" : text;
  showToast(display, "success");
  toastTimer = window.setTimeout(() => {
    showIdle();
  }, 2000);
  scheduleHide(2500);
}

// ── Waveform (single-line ECG style) ──

function updateWaveTargets(amp: number) {
  const center = (WAVEFORM_POINTS - 1) / 2;
  for (let i = 0; i < WAVEFORM_POINTS; i++) {
    const dist = Math.abs(i - center) / center;
    // Bell-curve envelope: edges taper to ~20%, center gets full amplitude
    const envelope = 0.2 + 0.8 * (1.0 - dist * dist);
    // ECG-style: random positive or negative deflection (asymmetric)
    const sign = (Math.random() > 0.5) ? 1 : -1;
    const jitter = 0.3 + Math.random() * 0.7;
    waveTargets[i] = Math.max(Math.min(amp * envelope * jitter * sign, 1.0), -1.0);
  }
}

function animateWave() {
  for (let i = 0; i < WAVEFORM_POINTS; i++) {
    const diff = waveTargets[i] - waveHeights[i];
    if (Math.abs(diff) > Math.abs(waveHeights[i]) * 0.1) {
      // Fast attack toward target
      waveHeights[i] += diff * 0.35;
    } else {
      // Slow decay toward zero
      waveHeights[i] += diff * 0.12;
    }
    if (Math.abs(waveHeights[i]) < 0.005) waveHeights[i] = 0;
  }
}

function drawWaveform() {
  const c = canvas();
  if (c.classList.contains("hidden")) return;
  const ctx = c.getContext("2d");
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const w = c.clientWidth;
  const h = c.clientHeight;
  if (w === 0 || h === 0) return;

  if (c.width !== w * dpr || c.height !== h * dpr) {
    c.width = w * dpr;
    c.height = h * dpr;
    ctx.scale(dpr, dpr);
  }

  ctx.clearRect(0, 0, w, h);

  const cy = h / 2;
  const maxAmp = h / 2 - 2; // leave 2px margin top/bottom
  const step = w / (WAVEFORM_POINTS - 1);

  // Build y-values: single line that deflects above and below center
  const yVals: number[] = [];
  for (let i = 0; i < WAVEFORM_POINTS; i++) {
    const a = Math.max(Math.min(waveHeights[i], 1.0), -1.0);
    yVals.push(cy - a * maxAmp); // negative a → below center, positive → above
  }

  // Fill area between the line and center
  ctx.beginPath();
  ctx.moveTo(0, cy);
  for (let i = 0; i < WAVEFORM_POINTS; i++) {
    ctx.lineTo(i * step, yVals[i]);
  }
  ctx.lineTo((WAVEFORM_POINTS - 1) * step, cy);
  ctx.closePath();
  ctx.fillStyle = "rgba(52, 199, 89, 0.12)";
  ctx.fill();

  // Draw the main ECG line
  ctx.beginPath();
  ctx.moveTo(0, yVals[0]);
  for (let i = 1; i < WAVEFORM_POINTS; i++) {
    ctx.lineTo(i * step, yVals[i]);
  }
  ctx.strokeStyle = "rgba(52, 199, 89, 0.85)";
  ctx.lineWidth = 1.5;
  ctx.lineJoin = "round";
  ctx.lineCap = "round";
  ctx.stroke();

  // Draw faint center baseline
  ctx.beginPath();
  ctx.moveTo(0, cy);
  ctx.lineTo(w, cy);
  ctx.strokeStyle = "rgba(52, 199, 89, 0.15)";
  ctx.lineWidth = 0.5;
  ctx.stroke();
}

function startPolling() {
  if (amplitudeTimer !== null) return;
  amplitudeTimer = window.setInterval(async () => {
    try {
      const raw = (await invoke("get_amplitude")) as number;
      // AGC: track peak and normalize
      if (raw > recentPeak) {
        recentPeak += (raw - recentPeak) * AGC_ATTACK;
      } else {
        recentPeak *= AGC_DECAY;
      }
      recentPeak = Math.max(recentPeak, 0.01); // keep a floor
      currentAmplitude = Math.min(raw / recentPeak, 1.0);
      updateWaveTargets(currentAmplitude);
      animateWave();
      drawWaveform();
    } catch { /* ignore */ }
  }, 50);
}

function stopPolling() {
  if (amplitudeTimer !== null) {
    clearInterval(amplitudeTimer);
    amplitudeTimer = null;
  }
}

// ── Toggle ──

async function toggleRecording() {
  // Block hotkey while backend is processing (transcribe + polish + insert)
  if (isProcessing) return;

  if (!isRecording) {
    try {
      // Show window first, then start recording
      cancelScheduledHide();
      await invoke("show_window");
      await invoke("start_recording");
      isRecording = true;
      showRecording();
      startPolling();
    } catch (e) {
      console.error("start error:", e);
      showError("Mic error");
    }
  } else {
    isRecording = false;
    isProcessing = true;
    stopPolling();
    showProcessing();
    // Force repaint so the processing UI is visible before the long await
    await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    try {
      const result = await invoke("stop_recording_and_transcribe") as string;
      if (!result || result.trim() === "") {
        showError("No speech detected");
      } else {
        showSuccess(result);
      }
    } catch (e) {
      console.error("stop error:", e);
      showError(String(e).substring(0, 30));
    } finally {
      stopProcessingTimer();
      isProcessing = false;
    }
  }
}

// ── Hotkey registration ──

let currentShortcut: string | null = null;
let usingNativeHotkey = false; // true when CGEventTap fallback is active

async function registerShortcut(shortcut: string) {
  // Unregister previous shortcut (whichever system was active)
  if (currentShortcut) {
    try {
      if (usingNativeHotkey) {
        await invoke("unregister_native_hotkey");
      } else {
        await unregister(currentShortcut);
      }
    } catch { /* may not be registered */ }
    usingNativeHotkey = false;
  }

  // Try tauri-plugin-global-shortcut first (handles standard combos like Alt+Space)
  try {
    await register(shortcut, (event) => {
      if (event.state === "Pressed") toggleRecording();
    });
    currentShortcut = shortcut;
    console.log(`[main] registered shortcut via global-shortcut: ${shortcut}`);
  } catch (e) {
    // global-shortcut doesn't support this key — fall back to native CGEventTap
    console.warn(`[main] global-shortcut failed for "${shortcut}": ${e}, trying native listener`);
    try {
      await invoke("register_native_hotkey", { key: shortcut });
      usingNativeHotkey = true;
      currentShortcut = shortcut;
      console.log(`[main] registered shortcut via native CGEventTap: ${shortcut}`);
    } catch (e2) {
      console.error(`[main] native hotkey also failed for "${shortcut}": ${e2}`);
      throw e2;
    }
  }
}

// ── Init ──

window.addEventListener("DOMContentLoaded", async () => {
  showIdle();

  // Load configured shortcut from backend
  let shortcut = "Alt+Space"; // fallback
  try {
    const config = await invoke("get_config") as { shortcut: string };
    if (config.shortcut) shortcut = config.shortcut;
  } catch (e) {
    console.warn("[main] failed to load config, using default shortcut:", e);
  }

  await registerShortcut(shortcut);

  // Listen for native CGEventTap hotkey presses (fallback for unsupported keys)
  const { listen } = await import("@tauri-apps/api/event");
  await listen("native-hotkey-pressed", () => {
    toggleRecording();
  });

  // Listen for config changes (from Settings window) and re-register hotkey
  await listen("config-changed", async () => {
    try {
      const config = await invoke("get_config") as { shortcut: string };
      if (config.shortcut && config.shortcut !== currentShortcut) {
        await registerShortcut(config.shortcut);
      }
    } catch (e) {
      console.error("[main] failed to update shortcut:", e);
    }
  });
});
