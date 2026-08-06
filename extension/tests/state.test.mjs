import test from "node:test";
import assert from "node:assert/strict";
import {
  alarmDecision,
  canonicalizePanelState,
  defaultState,
  finishState,
  markNotified,
  needsNotification,
  normalizeState
} from "../state.mjs";

test("running state reconciles against its absolute deadline", () => {
  const input = {
    ...defaultState(),
    status: "running",
    remainingMs: 90_000,
    deadlineMs: 160_000,
    runId: "run-1"
  };
  const result = normalizeState(input, 100_000);
  assert.equal(result.remainingMs, 60_000);
  assert.equal(result.status, "running");
});

test("expired running state becomes finished", () => {
  const input = {
    ...defaultState(),
    status: "running",
    deadlineMs: 99_000,
    runId: "run-1"
  };
  const result = normalizeState(input, 100_000);
  assert.equal(result.status, "finished");
  assert.equal(result.remainingMs, 0);
  assert.equal(result.deadlineMs, null);
  assert.equal(needsNotification(result), true);
});

test("panel updates increment the shared revision and create a run id", () => {
  const current = { ...defaultState(), revision: 4 };
  const incoming = {
    ...defaultState(),
    status: "running",
    remainingMs: 120_000,
    deadlineMs: 220_000
  };
  const result = canonicalizePanelState(
    incoming,
    current,
    "panel-a",
    100_000,
    () => "run-new"
  );
  assert.equal(result.revision, 5);
  assert.equal(result.sourceId, "panel-a");
  assert.equal(result.runId, "run-new");
});

test("cross-window updates advance from the latest shared revision", () => {
  const panelA = canonicalizePanelState(
    { ...defaultState(), status: "paused", remainingMs: 120_000 },
    { ...defaultState(), revision: 8 },
    "panel-a",
    100_000
  );
  const panelB = canonicalizePanelState(
    { ...panelA, remainingMs: 90_000 },
    panelA,
    "panel-b",
    100_000
  );
  assert.equal(panelA.revision, 9);
  assert.equal(panelB.revision, 10);
  assert.equal(panelB.sourceId, "panel-b");
});

test("alarm decision rejects a stale elapsed deadline", () => {
  const running = {
    ...defaultState(),
    status: "running",
    deadlineMs: 99_000,
    runId: "run-1"
  };
  const decision = alarmDecision(running, 100_000);
  assert.equal(decision.type, "finish");
  assert.equal(decision.state.status, "finished");
});

test("completion notification is deduplicated by run id", () => {
  const finished = finishState({
    ...defaultState(),
    status: "running",
    deadlineMs: 100_000,
    runId: "run-1"
  });
  assert.equal(needsNotification(finished), true);
  const notified = markNotified(finished);
  assert.equal(needsNotification(notified), false);
  assert.equal(notified.notifiedRunId, "run-1");
});

test("invalid snapshots fall back to safe defaults", () => {
  const result = normalizeState({ version: 99, durationMs: Infinity });
  assert.deepEqual(result, defaultState());
});
