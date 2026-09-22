'use strict';

const COORDINATOR_PROTOCOL_VERSION = 1;
const COORDINATOR_DATA_CHANNEL = 'coordinator-data';
const TRANSPORT_FAMILY = 'coordinator-transport-state';

function failureOf(error) {
  return {
    code: typeof error?.code === 'string' && error.code ? error.code : 'COORDINATOR_REQUEST_FAILED',
    message: error instanceof Error ? error.message : String(error),
  };
}

function bindCoordinatorRendererPort({ port, host }) {
  if (!port || typeof port.on !== 'function' || typeof port.postMessage !== 'function' || typeof port.start !== 'function') {
    throw new TypeError('Coordinator renderer port is invalid.');
  }
  if (!host || typeof host.request !== 'function' || typeof host.onCoordinatorEvent !== 'function') {
    throw new TypeError('Coordinator renderer bridge requires the Mahayana Coordinator host facade.');
  }

  let disposed = false;
  let greeted = false;
  const post = (frame) => {
    if (disposed) return;
    try { port.postMessage(frame); } catch { dispose(); }
  };
  const disposeCoordinator = host.onCoordinatorEvent((event) => {
    if (disposed || event?.channel !== COORDINATOR_DATA_CHANNEL || typeof event?.family !== 'string') return;
    post({ kind: 'event', family: event.family, payload: event.payload });
  });
  const disposeLifecycle = typeof host.onLifecycle === 'function'
    ? host.onLifecycle((event) => {
        if (disposed || !greeted) return;
        const type = String(event?.type || '');
        if (type === 'protocol-ready') {
          post({ kind: 'event', family: TRANSPORT_FAMILY, payload: { state: 'connected' } });
        } else if (['protocol-error', 'exited', 'spawn-failed', 'restart'].includes(type)) {
          post({ kind: 'event', family: TRANSPORT_FAMILY, payload: { state: 'down' } });
        }
      })
    : () => {};

  function dispose() {
    if (disposed) return;
    disposed = true;
    try { disposeCoordinator?.(); } catch {}
    try { disposeLifecycle?.(); } catch {}
    try { port.close?.(); } catch {}
  }

  port.on('message', (event) => {
    if (disposed) return;
    const frame = event?.data;
    if (!frame || typeof frame !== 'object' || typeof frame.kind !== 'string') {
      post({ kind: 'lifecycle', phase: 'shutdown', reason: 'protocol-error', detail: 'renderer posted a malformed coordinator frame' });
      dispose();
      return;
    }
    if (frame.kind === 'lifecycle' && frame.phase === 'hello') {
      if (frame.protocolVersion !== COORDINATOR_PROTOCOL_VERSION) {
        post({ kind: 'lifecycle', phase: 'shutdown', reason: 'protocol-error', detail: 'coordinator protocol version mismatch' });
        dispose();
        return;
      }
      greeted = true;
      post({ kind: 'lifecycle', phase: 'ready', protocolVersion: COORDINATOR_PROTOCOL_VERSION });
      post({ kind: 'event', family: TRANSPORT_FAMILY, payload: { state: 'connected' } });
      return;
    }
    if (frame.kind === 'lifecycle' && frame.phase === 'shutdown') {
      dispose();
      return;
    }
    if (frame.kind !== 'request' || typeof frame.requestId !== 'string' || typeof frame.method !== 'string') {
      post({ kind: 'lifecycle', phase: 'shutdown', reason: 'protocol-error', detail: 'renderer posted an unsupported coordinator frame' });
      dispose();
      return;
    }
    if (!greeted) {
      post({
        kind: 'reply',
        requestId: frame.requestId,
        outcome: { status: 'failed', failure: { code: 'COORDINATOR_NOT_READY', message: 'Coordinator renderer port has not completed hello.' } },
      });
      return;
    }
    Promise.resolve(host.request(frame.method, frame.args ?? {})).then(
      (value) => post({ kind: 'reply', requestId: frame.requestId, outcome: { status: 'ok', value } }),
      (error) => post({ kind: 'reply', requestId: frame.requestId, outcome: { status: 'failed', failure: failureOf(error) } }),
    );
  });
  port.on('close', dispose);
  port.start();
  return dispose;
}

module.exports = {
  COORDINATOR_PROTOCOL_VERSION,
  COORDINATOR_DATA_CHANNEL,
  bindCoordinatorRendererPort,
};
