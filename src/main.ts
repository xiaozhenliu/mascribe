import { invoke } from "@tauri-apps/api/core";
import { register } from "@tauri-apps/plugin-global-shortcut";

let isRecording = false;
let amplitudeTimer: number | null = null;
let hideTimer: number | null = null;

const WAVEFORM_BARS = 16;
const barHeights: number[] = new Array(WAVEFORM_BARS).fill(0);
const barTargets: number[] = new Array(WAVEFORM_BARS).fill(0);
let currentAmplitude = 0;

const dot = () => document.getElementById("status-dot")!;
const canvas = () => document.getElementById("waveform") as HTMLCanvasElement;
const label = () => document.getElementById("status-label")!;

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

// ── State transitions ──

function showIdle() {
  dot().className = "dot";
  canvas().classList.remove("hidden");
  label().textContent = "";
  barHeights.fill(0);
  barTargets.fill(0);
  currentAmplitude = 0;
  drawWaveform();
}

function showRecording() {
  dot().className = "dot recording";
  canvas().classList.remove("hidden");
  label().textContent = "";
}

function showProcessing() {
  dot().className = "dot processing";
  canvas().classList.add("hidden");
  label().textContent = "Processing...";
}

function showError(msg: string) {
  dot().className = "dot";
  canvas().classList.add("hidden");
  label().textContent = msg;
  label().style.color = "rgba(255, 59, 48, 0.8)";
  setTimeout(() => {
    if (label().textContent === msg) showIdle();
    label().style.color = "";
  }, 2000);
  // Hide window after showing error briefly
  scheduleHide(2500);
}

function showSuccess(text: string) {
  dot().className = "dot";
  canvas().classList.add("hidden");
  const display = text.length > 25 ? text.substring(0, 25) + "…" : text;
  label().textContent = display;
  label().style.color = "rgba(52, 199, 89, 0.8)";
  setTimeout(() => {
    if (label().textContent === display) showIdle();
    label().style.color = "";
  }, 2000);
  // Hide window after showing result briefly
  scheduleHide(2500);
}

// ── Waveform (in-place bouncing) ──

function updateBarTargets(amp: number) {
  const center = (WAVEFORM_BARS - 1) / 2;
  for (let i = 0; i < WAVEFORM_BARS; i++) {
    const dist = Math.abs(i - center) / center;
    const weight = 1.0 - dist * 0.5;
    const jitter = 0.6 + Math.random() * 0.8;
    barTargets[i] = Math.min(amp * weight * jitter, 1.0);
  }
}

function animateBars() {
  for (let i = 0; i < WAVEFORM_BARS; i++) {
    const diff = barTargets[i] - barHeights[i];
    if (diff > 0) {
      barHeights[i] += diff * 0.4;
    } else {
      barHeights[i] += diff * 0.15;
    }
    if (barHeights[i] < 0.01) barHeights[i] = 0;
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

  const gap = 2;
  const barW = Math.max(1, (w - gap * (WAVEFORM_BARS - 1)) / WAVEFORM_BARS);
  const maxH = h - 4;
  const cy = h / 2;

  for (let i = 0; i < WAVEFORM_BARS; i++) {
    const a = Math.min(barHeights[i], 1.0);
    const bh = Math.max(2, a * maxH);
    const x = i * (barW + gap);
    const y = cy - bh / 2;

    const r = Math.round(52 + a * 203);
    const g = Math.round(199 - a * 140);
    const b = Math.round(89 - a * 41);
    ctx.fillStyle = `rgba(${r},${g},${b},${0.4 + a * 0.6})`;
    ctx.beginPath();
    ctx.roundRect(x, y, barW, bh, barW / 2);
    ctx.fill();
  }
}

function startPolling() {
  if (amplitudeTimer !== null) return;
  amplitudeTimer = window.setInterval(async () => {
    try {
      const raw = (await invoke("get_amplitude")) as number;
      currentAmplitude = Math.min(raw * 8, 1.0);
      updateBarTargets(currentAmplitude);
      animateBars();
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
    stopPolling();
    showProcessing();
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
    }
  }
}

// ── Init ──

window.addEventListener("DOMContentLoaded", async () => {
  showIdle();

  await register("Alt+Space", (event) => {
    if (event.state === "Pressed") toggleRecording();
  });
});
