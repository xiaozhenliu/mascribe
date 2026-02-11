import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { register } from "@tauri-apps/plugin-global-shortcut";

let isRecording = false;
const statusDot = () => document.getElementById("status-dot")!;
const statusText = () => document.getElementById("status-text")!;
const resultEl = () => document.getElementById("result")!;

function setStatus(state: "idle" | "recording" | "processing", text: string) {
  const dot = statusDot();
  dot.className = "status-dot " + state;
  statusText().textContent = text;
}

async function toggleRecording() {
  if (!isRecording) {
    try {
      await invoke("start_recording");
      isRecording = true;
      setStatus("recording", "Recording... Press again to stop");
    } catch (e) {
      setStatus("idle", "Error: " + e);
    }
  } else {
    isRecording = false;
    setStatus("processing", "Processing...");
    try {
      const result = await invoke("stop_recording_and_transcribe");
      setStatus("idle", "Done — Press Cmd+Shift+Space");
      resultEl().textContent = result as string;
    } catch (e) {
      setStatus("idle", "Error: " + e);
    }
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  // Register global shortcut
  await register("CommandOrControl+Shift+Space", (event) => {
    if (event.state === "Pressed") {
      toggleRecording();
    }
  });

  // Listen for transcription complete events
  await listen<string>("transcription-complete", (event) => {
    resultEl().textContent = event.payload;
  });
});
