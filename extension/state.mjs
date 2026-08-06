export const SNAPSHOT_VERSION = 1;
export const STORAGE_KEY = "hourglassStateV1";
export const ALARM_NAME = "hourglass-timer-complete";
export const NOTIFICATION_ID = "hourglass-timer-complete";
export const MAX_DURATION_MS = 24 * 60 * 60 * 1000;

const STATUSES = new Set(["idle", "running", "paused", "finished"]);
const COLOR_MODES = new Set(["static", "random", "rainbow"]);
const SHAPES = new Set(["classic", "modern", "slim", "wide"]);
const SHAPE_MODES = new Set(["static", "morphing"]);

const clamp = (value, minimum, maximum) =>
  Math.min(maximum, Math.max(minimum, Number.isFinite(value) ? value : minimum));

const oneOf = (value, allowed, fallback) =>
  allowed.has(value) ? value : fallback;

export function defaultState() {
  return {
    version: SNAPSHOT_VERSION,
    revision: 0,
    sourceId: "worker",
    durationMs: 180_000,
    remainingMs: 180_000,
    status: "idle",
    deadlineMs: null,
    runId: null,
    notifiedRunId: null,
    appearance: {
      colorMode: "static",
      colorRgba: [0.8, 0.6, 0.2, 1],
      shape: "classic",
      shapeMode: "static"
    }
  };
}

export function normalizeState(input, now = Date.now()) {
  const fallback = defaultState();
  if (!input || typeof input !== "object" || input.version !== SNAPSHOT_VERSION) {
    return fallback;
  }

  const durationMs = clamp(Number(input.durationMs), 0, MAX_DURATION_MS);
  let remainingMs = clamp(Number(input.remainingMs), 0, durationMs);
  let status = oneOf(input.status, STATUSES, remainingMs === durationMs ? "idle" : "paused");
  let deadlineMs = Number.isFinite(input.deadlineMs) ? Number(input.deadlineMs) : null;

  if (status === "running") {
    if (deadlineMs === null && remainingMs > 0) {
      deadlineMs = now + remainingMs;
    }
    remainingMs = deadlineMs === null ? remainingMs : Math.max(0, deadlineMs - now);
    if (remainingMs <= 0) {
      status = "finished";
      remainingMs = 0;
      deadlineMs = null;
    }
  } else if (status === "idle") {
    remainingMs = durationMs;
    deadlineMs = null;
  } else if (status === "finished") {
    remainingMs = 0;
    deadlineMs = null;
  } else {
    deadlineMs = null;
  }

  const appearance = input.appearance && typeof input.appearance === "object"
    ? input.appearance
    : fallback.appearance;
  const rgba = Array.isArray(appearance.colorRgba) && appearance.colorRgba.length === 4
    ? appearance.colorRgba.map((channel) => clamp(Number(channel), 0, 1))
    : fallback.appearance.colorRgba;

  return {
    version: SNAPSHOT_VERSION,
    revision: Number.isSafeInteger(input.revision) && input.revision >= 0 ? input.revision : 0,
    sourceId: typeof input.sourceId === "string" ? input.sourceId : "worker",
    durationMs,
    remainingMs,
    status,
    deadlineMs,
    runId: typeof input.runId === "string" ? input.runId : null,
    notifiedRunId: typeof input.notifiedRunId === "string" ? input.notifiedRunId : null,
    appearance: {
      colorMode: oneOf(appearance.colorMode, COLOR_MODES, "static"),
      colorRgba: rgba,
      shape: oneOf(appearance.shape, SHAPES, "classic"),
      shapeMode: oneOf(appearance.shapeMode, SHAPE_MODES, "static")
    }
  };
}

export function canonicalizePanelState(
  panelState,
  currentState,
  sourceId,
  now = Date.now(),
  createRunId = () => crypto.randomUUID()
) {
  const current = normalizeState(currentState, now);
  const next = normalizeState(panelState, now);
  next.revision = current.revision + 1;
  next.sourceId = sourceId;
  next.notifiedRunId = current.notifiedRunId;

  if (next.status === "running") {
    next.runId = createRunId();
  } else if (next.status === "finished") {
    next.runId = current.runId;
  } else {
    next.runId = null;
  }
  return next;
}

export function finishState(state, sourceId = "worker") {
  const next = normalizeState(state);
  next.revision += 1;
  next.sourceId = sourceId;
  next.status = "finished";
  next.remainingMs = 0;
  next.deadlineMs = null;
  return next;
}

export function alarmDecision(state, now = Date.now()) {
  const elapsedRunningDeadline = state?.status === "running"
    && Number.isFinite(state.deadlineMs)
    && state.deadlineMs <= now;
  const normalized = normalizeState(state, now);
  if (elapsedRunningDeadline) {
    return { type: "finish", state: finishState(normalized) };
  }
  if (normalized.status !== "running" || normalized.deadlineMs === null) {
    return { type: normalized.status === "finished" ? "finished" : "clear", state: normalized };
  }
  if (normalized.deadlineMs <= now) {
    return { type: "finish", state: finishState(normalized) };
  }
  return { type: "schedule", when: normalized.deadlineMs, state: normalized };
}

export function needsNotification(state) {
  return state.status === "finished"
    && typeof state.runId === "string"
    && state.notifiedRunId !== state.runId;
}

export function markNotified(state, sourceId = "worker") {
  const next = { ...state };
  next.revision += 1;
  next.sourceId = sourceId;
  next.notifiedRunId = next.runId;
  return next;
}
