import { PANEL_PORT_NAME, STORAGE_KEY, defaultState } from "./state.mjs";

const sourceId = crypto.randomUUID();
const panelPort = chrome.runtime.connect({ name: PANEL_PORT_NAME });
const extensionVersion = chrome.runtime.getManifest().version;
const canvas = document.getElementById("hourglass-canvas");
const loading = document.getElementById("loading");
const errorPanel = document.getElementById("error");
const errorDetail = document.getElementById("error-detail");
let wasmReady = false;
let latestRevision = -1;
let pendingRestore = null;
let bootstrapRevision = -1;
let startupWatchdog = null;
let startupStage = "Loading Hourglass Timer…";
let preflightAdapter = null;
let panelClosed = false;

function reportPanelClosed() {
  if (panelClosed) {
    return;
  }
  panelClosed = true;
  void chrome.runtime.sendMessage({ type: "panel-closed-v1" }).catch(() => undefined);
  panelPort.disconnect();
}

window.addEventListener("pagehide", reportPanelClosed, { once: true });

function setStartupStage(stage) {
  startupStage = stage;
  loading.textContent = `v${extensionVersion} — ${stage}`;
}

function timeoutAfter(milliseconds, message) {
  return new Promise((_, reject) => {
    setTimeout(() => reject(new Error(message)), milliseconds);
  });
}

async function send(message) {
  const response = await Promise.race([
    chrome.runtime.sendMessage(message),
    timeoutAfter(5_000, "The extension service worker timed out")
  ]);
  if (!response?.ok) {
    throw new Error(response?.error || "The extension service worker did not respond");
  }
  return response.state;
}

function showStartupError(error) {
  if (startupWatchdog !== null) {
    clearTimeout(startupWatchdog);
    startupWatchdog = null;
  }
  console.error("WASM loading failed", error);
  loading.classList.add("hidden");
  errorDetail.textContent = error instanceof Error
    ? `${error.message} (last stage: ${startupStage})`
    : "Reload the extension and try again.";
  errorPanel.hidden = false;
}

function markWasmReady() {
  if (wasmReady) {
    return;
  }
  if (startupWatchdog !== null) {
    clearTimeout(startupWatchdog);
    startupWatchdog = null;
  }
  wasmReady = true;
  if (pendingRestore?.revision > bootstrapRevision) {
    const state = pendingRestore;
    pendingRestore = null;
    window.dispatchEvent(new CustomEvent("hourglass-restore-v1", {
      detail: JSON.stringify(state)
    }));
  } else {
    pendingRestore = null;
  }
  loading.classList.add("hidden");
}

window.addEventListener("hourglass-ready-v1", markWasmReady, { once: true });
window.addEventListener("hourglass-startup-stage-v1", (event) => {
  if (typeof event.detail === "string") {
    setStartupStage(event.detail);
  }
});
window.addEventListener("error", (event) => {
  if (!wasmReady) {
    showStartupError(event.error ?? new Error(event.message || "Unexpected startup error"));
  }
});
window.addEventListener("unhandledrejection", (event) => {
  if (!wasmReady) {
    showStartupError(event.reason ?? new Error("Unexpected startup rejection"));
  }
});

function restoreIntoBevy(state) {
  if (!Number.isSafeInteger(state?.revision) || state.revision <= latestRevision) {
    return;
  }
  latestRevision = state.revision;
  if (!wasmReady) {
    pendingRestore = state;
    return;
  }
  window.dispatchEvent(new CustomEvent("hourglass-restore-v1", {
    detail: JSON.stringify(state)
  }));
}

window.addEventListener("hourglass-state-v1", (event) => {
  try {
    const state = JSON.parse(event.detail);
    void send({ type: "set-state-v1", sourceId, state }).catch(console.error);
  } catch (error) {
    console.error("Ignoring invalid state from WASM", error);
  }
});

chrome.storage.onChanged.addListener((changes, areaName) => {
  if (areaName !== "local" || !changes[STORAGE_KEY]?.newValue) {
    return;
  }
  const state = changes[STORAGE_KEY].newValue;
  if (state.sourceId === sourceId) {
    latestRevision = Math.max(latestRevision, state.revision ?? -1);
    return;
  }
  restoreIntoBevy(state);
});

async function run() {
  setStartupStage("Restoring Hourglass Timer…");
  const initialState = await send({ type: "get-state-v1" }).catch((error) => {
    console.warn("Falling back to default state", error);
    return defaultState();
  });
  bootstrapRevision = initialState.revision;
  latestRevision = Math.max(latestRevision, bootstrapRevision);
  window.__HOURGLASS_BOOTSTRAP_V1__ = JSON.stringify(initialState);
  if (!navigator.gpu) {
    throw new Error("WebGPU is unavailable. Enable hardware acceleration in Chrome and reload the extension.");
  }

  setStartupStage("Checking WebGPU adapter…");
  preflightAdapter = await Promise.race([
    navigator.gpu.requestAdapter({ powerPreference: "high-performance" }),
    timeoutAfter(10_000, "Chrome timed out while creating a WebGPU adapter")
  ]);
  if (!preflightAdapter) {
    throw new Error("Chrome did not provide a WebGPU adapter. Check that hardware acceleration is enabled.");
  }

  setStartupStage("Checking WebGPU device…");
  const preflightDevice = await Promise.race([
    preflightAdapter.requestDevice(),
    timeoutAfter(10_000, "Chrome timed out while creating a WebGPU device")
  ]);

  setStartupStage("Checking WebGPU canvas…");
  const preflightContext = canvas?.getContext("webgpu");
  if (!preflightContext) {
    preflightDevice.destroy();
    throw new Error("Chrome could not create a WebGPU canvas in the side panel");
  }
  preflightContext.configure({
    device: preflightDevice,
    format: navigator.gpu.getPreferredCanvasFormat(),
    alphaMode: "opaque"
  });
  preflightContext.unconfigure();
  preflightDevice.destroy();

  setStartupStage("Loading Rust module…");
  const { default: init } = await import("./hourglass-timer.js");
  startupWatchdog = setTimeout(() => {
    if (!wasmReady) {
      showStartupError(new Error(
        "The renderer did not start within 20 seconds. Reload the extension and try again."
      ));
    }
  }, 20_000);
  setStartupStage("Starting Rust runtime…");
  void init().then(markWasmReady).catch(showStartupError);
}

run().catch(showStartupError);
