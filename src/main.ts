import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { register } from "@tauri-apps/plugin-global-shortcut";

let isRecording = false;
let amplitudeTimer: number | null = null;

const WAVEFORM_BARS = 24;
const ampHistory: number[] = new Array(WAVEFORM_BARS).fill(0);

const dot = () => document.getElementById("status-dot")!;
const canvas = () => document.getElementById("waveform") as HTMLCanvasElement;
const label = () => document.getElementById("status-label")!;

// ── State transitions ──

function showIdle() {
  dot().className = "dot";
  canvas().classList.remove("hidden");
  label().textContent = "";
  ampHistory.fill(0);
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

// ── Waveform ──

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
    const a = Math.min(ampHistory[i], 1.0);
    const bh = Math.max(2, a * maxH);
    const x = i * (barW + gap);
    const y = cy - bh / 2;

    // Green → red gradient
    const r = Math.round(52 + a * 203);  // #34c759 → #ff3b30
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
      ampHistory.shift();
      ampHistory.push(Math.min(raw * 8, 1.0));
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
      await invoke("start_recording");
      isRecording = true;
      showRecording();
      startPolling();
    } catch (e) {
      console.error("start error:", e);
      showIdle();
    }
  } else {
    isRecording = false;
    stopPolling();
    showProcessing();
    try {
      await invoke("stop_recording_and_transcribe");
    } catch (e) {
      console.error("stop error:", e);
    }
    showIdle();
  }
}

// ── Init ──

window.addEventListener("DOMContentLoaded", async () => {
  showIdle();

  await register("CommandOrControl+Shift+Space", (event) => {
    if (event.state === "Pressed") toggleRecording();
  });

  await listen<string>("transcription-complete", () => {
    // Text already pasted — nothing to show
  });
});
