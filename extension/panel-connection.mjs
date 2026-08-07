export const PANEL_HEARTBEAT_INTERVAL_MS = 20_000;

export function createPanelConnection({
  runtime,
  portName,
  sourceId,
  requestTimeoutMs = 5_000,
  reconnectDelayMs = 0,
  setIntervalFn = globalThis.setInterval,
  clearIntervalFn = globalThis.clearInterval,
  setTimeoutFn = globalThis.setTimeout,
  clearTimeoutFn = globalThis.clearTimeout
}) {
  let closed = false;
  let port = null;
  let reconnectTimer = null;
  let nextRequestId = 0;
  const pendingRequests = new Map();

  function rejectRequestsForPort(disconnectedPort) {
    for (const [requestId, pending] of pendingRequests) {
      if (pending.port !== disconnectedPort) {
        continue;
      }
      clearTimeoutFn(pending.timeout);
      pending.reject(new Error("The extension service worker disconnected"));
      pendingRequests.delete(requestId);
    }
  }

  function scheduleReconnect() {
    if (closed || reconnectTimer !== null) {
      return;
    }
    reconnectTimer = setTimeoutFn(() => {
      reconnectTimer = null;
      if (!closed) {
        connect();
        sendHeartbeat();
      }
    }, reconnectDelayMs);
  }

  function handleResponse(response) {
    const requestId = typeof response?.requestId === "string"
      ? response.requestId
      : "";
    const pending = pendingRequests.get(requestId);
    if (!pending) {
      return;
    }
    pendingRequests.delete(requestId);
    clearTimeoutFn(pending.timeout);
    if (response.ok) {
      pending.resolve(response);
    } else {
      pending.reject(new Error(response.error || "The extension service worker rejected the request"));
    }
  }

  function connect() {
    if (closed) {
      throw new Error("The side panel is closed");
    }
    if (port !== null) {
      return port;
    }
    const connectedPort = runtime.connect({ name: `${portName}:${sourceId}` });
    port = connectedPort;
    connectedPort.onMessage.addListener(handleResponse);
    connectedPort.onDisconnect.addListener(() => {
      if (port !== connectedPort) {
        return;
      }
      port = null;
      rejectRequestsForPort(connectedPort);
      scheduleReconnect();
    });
    return connectedPort;
  }

  function sendHeartbeat() {
    if (closed) {
      return;
    }
    const connectedPort = connect();
    try {
      connectedPort.postMessage({ type: "panel-heartbeat-v1" });
    } catch {
      if (port === connectedPort) {
        port = null;
      }
      rejectRequestsForPort(connectedPort);
      scheduleReconnect();
    }
  }

  function request(message) {
    const connectedPort = connect();
    const requestId = `${sourceId}:${nextRequestId += 1}`;
    return new Promise((resolve, reject) => {
      const timeout = setTimeoutFn(() => {
        pendingRequests.delete(requestId);
        reject(new Error("The extension service worker timed out"));
      }, requestTimeoutMs);
      pendingRequests.set(requestId, { port: connectedPort, resolve, reject, timeout });
      try {
        connectedPort.postMessage({ ...message, requestId });
      } catch (error) {
        pendingRequests.delete(requestId);
        clearTimeoutFn(timeout);
        if (port === connectedPort) {
          port = null;
        }
        scheduleReconnect();
        reject(error);
      }
    });
  }

  function close() {
    if (closed) {
      return;
    }
    closed = true;
    clearIntervalFn(heartbeatTimer);
    if (reconnectTimer !== null) {
      clearTimeoutFn(reconnectTimer);
      reconnectTimer = null;
    }
    const connectedPort = port;
    port = null;
    if (connectedPort !== null) {
      rejectRequestsForPort(connectedPort);
      connectedPort.disconnect();
    }
  }

  connect();
  sendHeartbeat();
  const heartbeatTimer = setIntervalFn(sendHeartbeat, PANEL_HEARTBEAT_INTERVAL_MS);

  return { close, request };
}
