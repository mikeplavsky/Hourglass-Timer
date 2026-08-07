import test from "node:test";
import assert from "node:assert/strict";
import {
  ALARM_NAME,
  PANEL_PORT_NAME,
  STORAGE_KEY,
  defaultState
} from "../state.mjs";

const listeners = {};
const localStorage = new Map();
const clearedAlarms = [];
let removedSnapshots = 0;

globalThis.chrome = {
  sidePanel: {
    setPanelBehavior: async () => undefined
  },
  storage: {
    local: {
      get: async (key) => localStorage.has(key) ? { [key]: localStorage.get(key) } : {},
      set: async (values) => {
        for (const [key, value] of Object.entries(values)) {
          localStorage.set(key, value);
        }
      },
      remove: async (key) => {
        localStorage.delete(key);
        removedSnapshots += 1;
      }
    }
  },
  alarms: {
    clear: async (name) => {
      clearedAlarms.push(name);
      return true;
    },
    create: async () => undefined,
    onAlarm: {
      addListener: (listener) => {
        listeners.alarm = listener;
      }
    }
  },
  notifications: {
    getPermissionLevel: async () => "denied",
    create: async () => undefined
  },
  runtime: {
    onConnect: {
      addListener: (listener) => {
        listeners.connect = listener;
      }
    },
    onStartup: {
      addListener: (listener) => {
        listeners.startup = listener;
      }
    },
    onInstalled: {
      addListener: (listener) => {
        listeners.installed = listener;
      }
    }
  }
};

await import(`../service-worker.mjs?test=${Date.now()}`);

function createPanelPort(sourceId) {
  const messageListeners = [];
  const disconnectListeners = [];
  const pendingResponses = new Map();
  let nextRequestId = 0;
  let disconnected = false;
  return {
    name: `${PANEL_PORT_NAME}:${sourceId}`,
    onMessage: {
      addListener: (listener) => messageListeners.push(listener)
    },
    onDisconnect: {
      addListener: (listener) => disconnectListeners.push(listener)
    },
    postMessage: (response) => {
      if (disconnected) {
        throw new Error("Port is disconnected");
      }
      const resolve = pendingResponses.get(response.requestId);
      if (resolve) {
        pendingResponses.delete(response.requestId);
        resolve(response);
      }
    },
    request(message) {
      const requestId = `${sourceId}:${nextRequestId += 1}`;
      const response = new Promise((resolve) => {
        pendingResponses.set(requestId, resolve);
      });
      for (const listener of messageListeners) {
        listener({ ...message, requestId });
      }
      return response;
    },
    heartbeat() {
      for (const listener of messageListeners) {
        listener({ type: "panel-heartbeat-v1" });
      }
    },
    disconnect() {
      disconnected = true;
      for (const listener of disconnectListeners) {
        listener();
      }
    }
  };
}

async function waitFor(predicate) {
  for (let attempt = 0; attempt < 50; attempt += 1) {
    if (predicate()) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  assert.fail("Timed out waiting for service-worker state task");
}

test("the last closed panel clears its snapshot and completion alarm", async () => {
  localStorage.set(STORAGE_KEY, { ...defaultState(), status: "paused" });
  const firstPanel = createPanelPort("panel-a");
  const secondPanel = createPanelPort("panel-b");
  listeners.connect(firstPanel);
  listeners.connect(secondPanel);

  firstPanel.disconnect();
  assert.equal(localStorage.has(STORAGE_KEY), true);

  secondPanel.disconnect();
  await waitFor(() => removedSnapshots === 1);
  assert.equal(localStorage.has(STORAGE_KEY), false);
  assert.deepEqual(clearedAlarms, [ALARM_NAME]);
});

test("a queued update from the last panel is cleared after disconnect", async () => {
  localStorage.set(STORAGE_KEY, { ...defaultState(), status: "paused" });
  const panel = createPanelPort("panel-closing");
  listeners.connect(panel);

  void panel.request({
    type: "set-state-v1",
    state: { ...defaultState(), status: "paused", remainingMs: 60_000 }
  });
  panel.disconnect();

  await waitFor(() => removedSnapshots === 2);
  assert.equal(removedSnapshots, 2);
  assert.equal(localStorage.has(STORAGE_KEY), false);
  assert.deepEqual(clearedAlarms, [ALARM_NAME, ALARM_NAME, ALARM_NAME]);
});

test("a replacement connection supersedes stale requests without clearing live state", async () => {
  const stalePanel = createPanelPort("panel-reconnected");
  const livePanel = createPanelPort("panel-reconnected");
  listeners.connect(stalePanel);
  listeners.connect(livePanel);

  const staleResponse = await stalePanel.request({
    type: "set-state-v1",
    state: { ...defaultState(), status: "paused", remainingMs: 60_000 }
  });
  assert.deepEqual(staleResponse, {
    requestId: "panel-reconnected:1",
    ok: false,
    error: "Panel is no longer connected"
  });

  stalePanel.disconnect();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(removedSnapshots, 2);

  const liveResponse = await livePanel.request({
    type: "set-state-v1",
    state: {
      ...defaultState(),
      sourceId: "spoofed-source",
      status: "paused",
      remainingMs: 60_000
    }
  });
  assert.equal(liveResponse.ok, true);
  assert.equal(liveResponse.state.sourceId, "panel-reconnected");
  assert.equal(localStorage.has(STORAGE_KEY), true);

  livePanel.disconnect();
  await waitFor(() => removedSnapshots === 3);
  assert.equal(localStorage.has(STORAGE_KEY), false);
});

test("panel heartbeats keep the connection active without changing state", async () => {
  const panel = createPanelPort("panel-heartbeat");
  listeners.connect(panel);
  panel.heartbeat();
  await new Promise((resolve) => setTimeout(resolve, 0));

  assert.equal(removedSnapshots, 3);
  assert.equal(localStorage.has(STORAGE_KEY), false);

  panel.disconnect();
  await waitFor(() => removedSnapshots === 4);
});

test("browser startup discards stale timer state", async () => {
  localStorage.set(STORAGE_KEY, {
    ...defaultState(),
    status: "running",
    deadlineMs: Date.now() + 60_000
  });

  listeners.startup();
  await waitFor(() => removedSnapshots === 5);
  assert.equal(localStorage.has(STORAGE_KEY), false);
  assert.equal(clearedAlarms.at(-1), ALARM_NAME);
});
