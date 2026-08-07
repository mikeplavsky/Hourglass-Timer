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

function createPanelPort() {
  const disconnectListeners = [];
  return {
    name: PANEL_PORT_NAME,
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
  const firstPanel = createPanelPort();
  const secondPanel = createPanelPort();
  listeners.connect(firstPanel);
  listeners.connect(secondPanel);

  firstPanel.disconnect();
  assert.equal(localStorage.has(STORAGE_KEY), true);

  secondPanel.disconnect();
  await waitFor(() => removedSnapshots === 1);
  assert.equal(localStorage.has(STORAGE_KEY), false);
  assert.deepEqual(clearedAlarms, [ALARM_NAME]);
});

test("the pagehide fallback and port disconnect share one reset", async () => {
  localStorage.set(STORAGE_KEY, { ...defaultState(), status: "paused" });
  const panel = createPanelPort();
  listeners.connect(panel);

  const response = new Promise((resolve) => {
    assert.equal(
      listeners.message({ type: "panel-closed-v1" }, {}, resolve),
      true
    );
  });
  panel.disconnect();

  assert.deepEqual(await response, { ok: true, state: defaultState() });
  assert.equal(removedSnapshots, 2);
  assert.equal(localStorage.has(STORAGE_KEY), false);
  assert.deepEqual(clearedAlarms, [ALARM_NAME, ALARM_NAME]);
});

test("browser startup discards stale timer state", async () => {
  localStorage.set(STORAGE_KEY, {
    ...defaultState(),
    status: "running",
    deadlineMs: Date.now() + 60_000
  });

  listeners.startup();
  await waitFor(() => removedSnapshots === 3);
  assert.equal(localStorage.has(STORAGE_KEY), false);
  assert.deepEqual(clearedAlarms, [ALARM_NAME, ALARM_NAME, ALARM_NAME]);
});
