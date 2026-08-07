import test from "node:test";
import assert from "node:assert/strict";
import {
  PANEL_HEARTBEAT_INTERVAL_MS,
  createPanelConnection
} from "../panel-connection.mjs";

function createFakePort(name) {
  const messageListeners = [];
  const disconnectListeners = [];
  return {
    name,
    sent: [],
    disconnected: false,
    onMessage: {
      addListener: (listener) => messageListeners.push(listener)
    },
    onDisconnect: {
      addListener: (listener) => disconnectListeners.push(listener)
    },
    postMessage(message) {
      this.sent.push(message);
    },
    receive(message) {
      for (const listener of messageListeners) {
        listener(message);
      }
    },
    disconnect() {
      this.disconnected = true;
      for (const listener of disconnectListeners) {
        listener();
      }
    }
  };
}

function createHarness() {
  const ports = [];
  const intervals = [];
  const timeouts = [];
  const clearedIntervals = [];
  const clearedTimeouts = [];
  const runtime = {
    connect({ name }) {
      const port = createFakePort(name);
      ports.push(port);
      return port;
    }
  };
  return {
    ports,
    intervals,
    timeouts,
    clearedIntervals,
    clearedTimeouts,
    connection: createPanelConnection({
      runtime,
      portName: "panel",
      sourceId: "source",
      setIntervalFn: (callback, delay) => {
        const timer = { callback, delay };
        intervals.push(timer);
        return timer;
      },
      clearIntervalFn: (timer) => clearedIntervals.push(timer),
      setTimeoutFn: (callback, delay) => {
        const timer = { callback, delay };
        timeouts.push(timer);
        return timer;
      },
      clearTimeoutFn: (timer) => clearedTimeouts.push(timer)
    })
  };
}

test("panel connection sends heartbeats and reconnects after worker disconnect", () => {
  const harness = createHarness();
  assert.equal(harness.ports.length, 1);
  assert.equal(harness.ports[0].name, "panel:source");
  assert.deepEqual(harness.ports[0].sent, [{ type: "panel-heartbeat-v1" }]);
  assert.equal(harness.intervals[0].delay, PANEL_HEARTBEAT_INTERVAL_MS);

  harness.intervals[0].callback();
  assert.deepEqual(harness.ports[0].sent, [
    { type: "panel-heartbeat-v1" },
    { type: "panel-heartbeat-v1" }
  ]);

  harness.ports[0].disconnect();
  const reconnect = harness.timeouts.find((timer) => timer.delay === 0);
  reconnect.callback();
  assert.equal(harness.ports.length, 2);
  assert.deepEqual(harness.ports[1].sent, [{ type: "panel-heartbeat-v1" }]);

  harness.connection.close();
  assert.equal(harness.ports[1].disconnected, true);
  assert.deepEqual(harness.clearedIntervals, [harness.intervals[0]]);
});

test("panel requests and responses travel over the connected port", async () => {
  const harness = createHarness();
  const response = harness.connection.request({ type: "get-state-v1" });
  const request = harness.ports[0].sent.at(-1);
  assert.equal(request.type, "get-state-v1");
  assert.match(request.requestId, /^source:/);

  harness.ports[0].receive({
    requestId: request.requestId,
    ok: true,
    state: { status: "idle" }
  });
  assert.deepEqual(await response, {
    requestId: request.requestId,
    ok: true,
    state: { status: "idle" }
  });
  harness.connection.close();
});
