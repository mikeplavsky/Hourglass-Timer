import {
  ALARM_NAME,
  NOTIFICATION_ID,
  PANEL_PORT_NAME,
  STORAGE_KEY,
  alarmDecision,
  canonicalizePanelState,
  defaultState,
  finishState,
  markNotified,
  needsNotification,
  normalizeState
} from "./state.mjs";

const WORKER_SOURCE = "worker";
let stateQueue = Promise.resolve();
const openPanelPorts = new Map();

function enqueueStateTask(task) {
  const result = stateQueue.then(task, task);
  stateQueue = result.catch(() => undefined);
  return result;
}

chrome.sidePanel
  .setPanelBehavior({ openPanelOnActionClick: true })
  .catch((error) => console.error("Could not configure side panel action", error));

async function loadState() {
  const stored = await chrome.storage.local.get(STORAGE_KEY);
  return normalizeState(stored[STORAGE_KEY] ?? defaultState());
}

async function saveState(state) {
  await chrome.storage.local.set({ [STORAGE_KEY]: state });
  return state;
}

async function clearStoredState() {
  await chrome.alarms.clear(ALARM_NAME);
  await chrome.storage.local.remove(STORAGE_KEY);
  return defaultState();
}

function requestStoredStateClear() {
  return enqueueStateTask(clearStoredState);
}

async function showCompletionNotification(state) {
  if (!needsNotification(state)) {
    return state;
  }

  try {
    const permission = await chrome.notifications.getPermissionLevel();
    if (permission === "granted") {
      await chrome.notifications.create(NOTIFICATION_ID, {
        type: "basic",
        iconUrl: "icons/icon-128.png",
        title: "Hourglass Timer",
        message: "Your timer is complete."
      });
    }
  } catch (error) {
    console.error("Could not show completion notification", error);
  }

  return saveState(markNotified(state, WORKER_SOURCE));
}

async function completeAndNotify(state) {
  await chrome.alarms.clear(ALARM_NAME);
  const finished = state.status === "finished"
    ? state
    : finishState(state, WORKER_SOURCE);
  await saveState(finished);
  return showCompletionNotification(finished);
}

async function scheduleState(state) {
  const decision = alarmDecision(state);
  await chrome.alarms.clear(ALARM_NAME);
  if (decision.type === "finish") {
    return completeAndNotify(decision.state);
  }
  if (decision.type === "finished") {
    if (JSON.stringify(decision.state) !== JSON.stringify(state)) {
      await saveState(decision.state);
    }
    return showCompletionNotification(decision.state);
  }
  if (decision.type === "schedule") {
    await chrome.alarms.create(ALARM_NAME, { when: decision.when });
  }
  if (JSON.stringify(decision.state) !== JSON.stringify(state)) {
    await saveState(decision.state);
  }
  return decision.state;
}

async function reconcileState() {
  return scheduleState(await loadState());
}

function handlePanelRequest(message, sourceId) {
  if (message?.type === "get-state-v1") {
    return enqueueStateTask(reconcileState);
  }

  if (message?.type === "set-state-v1") {
    return enqueueStateTask(async () => {
      const current = await loadState();
      const state = canonicalizePanelState(
        message.state,
        current,
        sourceId
      );
      await saveState(state);
      return state.status === "finished"
        ? completeAndNotify(state)
        : scheduleState(state);
    });
  }

  return Promise.reject(new Error("Unsupported panel request"));
}

function postPanelResponse(port, response) {
  try {
    port.postMessage(response);
  } catch {
    // The panel closed while its queued request was being processed.
  }
}

chrome.runtime.onConnect.addListener((port) => {
  const prefix = `${PANEL_PORT_NAME}:`;
  if (!port.name.startsWith(prefix)) {
    return;
  }
  const sourceId = port.name.slice(prefix.length);
  if (sourceId.length === 0) {
    return;
  }
  openPanelPorts.set(sourceId, port);
  port.onMessage.addListener((message) => {
    if (message?.type === "panel-heartbeat-v1") {
      return;
    }
    const requestId = typeof message?.requestId === "string" ? message.requestId : "";
    if (requestId.length === 0) {
      return;
    }
    if (openPanelPorts.get(sourceId) !== port) {
      postPanelResponse(port, {
        requestId,
        ok: false,
        error: "Panel is no longer connected"
      });
      return;
    }
    void handlePanelRequest(message, sourceId)
      .then((state) => postPanelResponse(port, { requestId, ok: true, state }))
      .catch((error) => postPanelResponse(port, {
        requestId,
        ok: false,
        error: String(error)
      }));
  });
  port.onDisconnect.addListener(() => {
    if (openPanelPorts.get(sourceId) === port) {
      openPanelPorts.delete(sourceId);
    }
    if (openPanelPorts.size === 0) {
      void requestStoredStateClear().catch(console.error);
    }
  });
});

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name !== ALARM_NAME) {
    return;
  }
  void enqueueStateTask(async () => {
    const state = await loadState();
    const decision = alarmDecision(state);
    return decision.type === "finish"
      ? completeAndNotify(decision.state)
      : scheduleState(decision.state);
  }).catch(console.error);
});

chrome.runtime.onStartup.addListener(() => {
  void requestStoredStateClear().catch(console.error);
});

chrome.runtime.onInstalled.addListener(() => {
  void requestStoredStateClear().catch(console.error);
});
