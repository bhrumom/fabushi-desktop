'use strict';

const BRIDGE_MISSING_HANDLER = 'bridge/missing-handler';
const BRIDGE_INVOKE_FAILED = 'bridge/invoke-failed';
const BRIDGE_UNTRUSTED_SENDER = 'bridge/untrusted-sender';

function callChannel(edge, method) {
  return `fabushi-edge:${edge}:call:${method}`;
}

function pushChannel(edge, event) {
  return `fabushi-edge:${edge}:event:${event}`;
}

function defineEdge(edge, methods, events = [], version = 1) {
  if (!Number.isInteger(version) || version < 1) throw new Error('Edge contract version must be a positive integer.');
  const eventNames = Object.freeze([...events]);
  return Object.freeze({
    edge,
    version,
    methods: Object.freeze({ ...methods }),
    events: eventNames,
    eventSet: new Set(eventNames),
  });
}

function failure(code, error) {
  return {
    code,
    detail: error instanceof Error ? error.message : String(error),
  };
}

function isReply(value) {
  return Boolean(value && typeof value === 'object' && typeof value.ok === 'boolean');
}

class EdgeInvocationError extends Error {
  constructor(problem) {
    super(`${problem.code}: ${problem.detail}`);
    this.name = 'EdgeInvocationError';
    this.code = problem.code;
    this.detail = problem.detail;
  }
}

function createRendererEdge(ipcRenderer, edge) {
  async function invoke(method, args) {
    let reply;
    try {
      reply = await ipcRenderer.invoke(callChannel(edge.edge, method), args ?? {});
    } catch (error) {
      throw new EdgeInvocationError(failure(BRIDGE_INVOKE_FAILED, error));
    }
    if (!isReply(reply)) {
      throw new EdgeInvocationError({
        code: BRIDGE_INVOKE_FAILED,
        detail: 'Native edge returned an invalid reply.',
      });
    }
    if (reply.ok) return reply.value;
    throw new EdgeInvocationError(reply.failure);
  }

  const client = {};
  for (const [method, spec] of Object.entries(edge.methods)) {
    client[method] = spec.args === 'none'
      ? () => invoke(method, {})
      : (args = {}) => invoke(method, args);
  }

  client.subscribe = (listeners = {}) => {
    const cleanup = [];
    for (const eventName of edge.events) {
      const listener = listeners[eventName];
      if (typeof listener !== 'function') continue;
      const channel = pushChannel(edge.edge, eventName);
      const forward = (_event, payload) => listener(payload);
      ipcRenderer.on(channel, forward);
      cleanup.push(() => ipcRenderer.off(channel, forward));
    }
    return () => cleanup.splice(0).forEach((dispose) => dispose());
  };

  return Object.freeze(client);
}

function serveMainEdge(ipcMain, edge, handlers, options = {}) {
  const registered = [];
  for (const method of Object.keys(edge.methods)) {
    const channel = callChannel(edge.edge, method);
    registered.push(channel);
    ipcMain.handle(channel, async (event, args) => {
      if (options.isTrustedSender && !options.isTrustedSender(event)) {
        return { ok: false, failure: { code: BRIDGE_UNTRUSTED_SENDER, detail: 'IPC sender is not trusted.' } };
      }
      const handler = handlers[method];
      if (typeof handler !== 'function') {
        return { ok: false, failure: { code: BRIDGE_MISSING_HANDLER, detail: `No handler for ${method}.` } };
      }
      try {
        return { ok: true, value: await handler(args ?? {}, event) };
      } catch (error) {
        options.onHandlerError?.(method, error);
        return { ok: false, failure: failure(BRIDGE_INVOKE_FAILED, error) };
      }
    });
  }

  return Object.freeze({
    emit(webContents, eventName, payload) {
      if (!edge.eventSet.has(eventName)) throw new Error(`Unknown edge event: ${eventName}`);
      webContents.send(pushChannel(edge.edge, eventName), payload);
    },
    dispose() {
      for (const channel of registered) ipcMain.removeHandler(channel);
    },
  });
}

module.exports = {
  BRIDGE_INVOKE_FAILED,
  BRIDGE_MISSING_HANDLER,
  BRIDGE_UNTRUSTED_SENDER,
  EdgeInvocationError,
  callChannel,
  createRendererEdge,
  defineEdge,
  pushChannel,
  serveMainEdge,
};
