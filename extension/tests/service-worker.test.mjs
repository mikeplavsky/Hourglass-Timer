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
    onMessage: {
      addListener: (listener) => {
        listeners.message = listener;
      }
    },
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
  const disconnectListeners = [];
  return {
    name: `${PANEL_PORT_NAME}:${sourceId}`,
    onDisconnect: {
      addListener: (listener) => disconnectListeners.push(listener)
    },
    disconnect: () => {
      for (const listener of disconnectListeners) {
        listener();
      }
    }
  };
}

function sendRuntimeMessage(message) {
  return new Promise((resolve) => {
    const keepChannelOpen = listeners.message(message, {}, resolve);
    if (keepChannelOpen === false) {
      queueMicrotask(() => resolve(undefined));
    }
  });
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

  const response = sendRuntimeMessage({
    type: "set-state-v1",
    sourceId: "panel-closing",
    state: { ...defaultState(), status: "paused", remainingMs: 60_000 }
  });
  panel.disconnect();

  assert.equal((await response).ok, true);
  await waitFor(() => removedSnapshots === 2);
  assert.equal(removedSnapshots, 2);
  assert.equal(localStorage.has(STORAGE_KEY), false);
  assert.deepEqual(clearedAlarms, [ALARM_NAME, ALARM_NAME, ALARM_NAME]);
});

test("updates from a disconnected panel cannot recreate cleared state", async () => {
  const panel = createPanelPort("panel-stale");
  listeners.connect(panel);
  panel.disconnect();
  await waitFor(() => removedSnapshots === 3);

  const response = await sendRuntimeMessage({
    type: "set-state-v1",
    sourceId: "panel-stale",
    state: { ...defaultState(), status: "paused", remainingMs: 60_000 }
  });

  assert.deepEqual(response, {
    ok: false,
    error: "Panel is no longer connected"
  });
  assert.equal(localStorage.has(STORAGE_KEY), false);
  assert.equal(removedSnapshots, 3);
  assert.deepEqual(clearedAlarms, [ALARM_NAME, ALARM_NAME, ALARM_NAME, ALARM_NAME]);
});

test("browser startup discards stale timer state", async () => {
  localStorage.set(STORAGE_KEY, {
    ...defaultState(),
    status: "running",
    deadlineMs: Date.now() + 60_000
  });

  listeners.startup();
  await waitFor(() => removedSnapshots === 4);
  assert.equal(localStorage.has(STORAGE_KEY), false);
  assert.deepEqual(clearedAlarms, [
    ALARM_NAME,
    ALARM_NAME,
    ALARM_NAME,
    ALARM_NAME,
    ALARM_NAME
  ]);
});
