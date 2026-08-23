"use strict";

const assert = require('node:assert/strict');
const test = require('node:test');

const {
  BRIDGE_INVOKE_FAILED,
  BRIDGE_MISSING_HANDLER,
  BRIDGE_UNTRUSTED_SENDER,
  EdgeInvocationError,
  callChannel,
  createRendererEdge,
  defineEdge,
  pushChannel,
  serveMainEdge,
} = require('./edge-ipc.cjs');

function fakeIpcMain() {
  const handlers = new Map();
  return {
    handlers,
    handle(channel, handler) {
      assert.equal(handlers.has(channel), false, `duplicate handler ${channel}`);
      handlers.set(channel, handler);
    },
    removeHandler(channel) {
      handlers.delete(channel);
    },
  };
}

function fakeIpcRenderer(ipcMain, event = { senderFrame: { url: 'app://bundle/index.html' } }) {
  const listeners = new Map();
  return {
    listeners,
    async invoke(channel, args) {
      const handler = ipcMain.handlers.get(channel);
      if (!handler) throw new Error(`missing channel ${channel}`);
      return handler(event, args);
    },
    on(channel, listener) {
      const list = listeners.get(channel) ?? [];
      list.push(listener);
      listeners.set(channel, list);
    },
    off(channel, listener) {
      listeners.set(channel, (listeners.get(channel) ?? []).filter((entry) => entry !== listener));
    },
  };
}

test('edge exposes only declared methods and normalizes no-arg invocations', async () => {
  const edge = defineEdge('test', { ping: { args: 'none' }, echo: { args: 'object' } }, ['changed']);
  assert.equal(edge.version, 1);
  assert.throws(() => defineEdge('invalid', {}, [], 0), /positive integer/);
  const ipcMain = fakeIpcMain();
  serveMainEdge(ipcMain, edge, {
    ping: async (args) => ({ keys: Object.keys(args) }),
    echo: async (args) => args,
  });
  const renderer = fakeIpcRenderer(ipcMain);
  const client = createRendererEdge(renderer, edge);
  assert.equal(Object.isFrozen(client), true);
  assert.equal(client.unknown, undefined);
  assert.deepEqual(await client.ping(), { keys: [] });
  assert.deepEqual(await client.echo({ value: 7 }), { value: 7 });
});

test('main edge fails closed for untrusted senders and missing handlers', async () => {
  const edge = defineEdge('secure', { secret: { args: 'object' }, missing: { args: 'none' } });
  const ipcMain = fakeIpcMain();
  serveMainEdge(ipcMain, edge, { secret: async () => 'never' }, { isTrustedSender: () => false });
  const denied = await ipcMain.handlers.get(callChannel('secure', 'secret'))({}, {});
  assert.equal(denied.ok, false);
  assert.equal(denied.failure.code, BRIDGE_UNTRUSTED_SENDER);

  const second = fakeIpcMain();
  serveMainEdge(second, edge, { secret: async () => 'ok' });
  const missing = await second.handlers.get(callChannel('secure', 'missing'))({}, {});
  assert.equal(missing.ok, false);
  assert.equal(missing.failure.code, BRIDGE_MISSING_HANDLER);
});

test('handler exceptions become stable bridge failures and renderer raises EdgeInvocationError', async () => {
  const edge = defineEdge('errors', { explode: { args: 'none' } });
  const ipcMain = fakeIpcMain();
  const observed = [];
  serveMainEdge(ipcMain, edge, { explode: async () => { throw new Error('boom'); } }, {
    onHandlerError: (method, error) => observed.push([method, error.message]),
  });
  const client = createRendererEdge(fakeIpcRenderer(ipcMain), edge);
  await assert.rejects(client.explode(), (error) => {
    assert.equal(error instanceof EdgeInvocationError, true);
    assert.equal(error.code, BRIDGE_INVOKE_FAILED);
    assert.match(error.detail, /boom/);
    return true;
  });
  assert.deepEqual(observed, [['explode', 'boom']]);
});

test('event emission is allowlisted and dispose removes every registered handler', () => {
  const edge = defineEdge('events', { ping: { args: 'none' } }, ['changed']);
  const ipcMain = fakeIpcMain();
  const server = serveMainEdge(ipcMain, edge, { ping: async () => true });
  const sent = [];
  server.emit({ send: (...args) => sent.push(args) }, 'changed', { value: 1 });
  assert.deepEqual(sent, [[pushChannel('events', 'changed'), { value: 1 }]]);
  assert.throws(() => server.emit({ send() {} }, 'unknown', {}), /Unknown edge event/);
  assert.equal(ipcMain.handlers.size, 1);
  server.dispose();
  assert.equal(ipcMain.handlers.size, 0);
});

test('renderer subscriptions are scoped to declared events and can be disposed idempotently', () => {
  const edge = defineEdge('events', {}, ['changed', 'other']);
  const renderer = fakeIpcRenderer(fakeIpcMain());
  const client = createRendererEdge(renderer, edge);
  let seen = 0;
  const dispose = client.subscribe({ changed: () => { seen += 1; }, unknown: () => { seen += 100; } });
  const channel = pushChannel('events', 'changed');
  assert.equal(renderer.listeners.has(pushChannel('events', 'unknown')), false);
  renderer.listeners.get(channel)[0]({}, { value: 1 });
  assert.equal(seen, 1);
  dispose();
  dispose();
  assert.deepEqual(renderer.listeners.get(channel), []);
});


test('structured invocation traces contain correlation/status/duration but never args or results', async () => {
  const edge = defineEdge('trace', { echo: { args: 'object' } });
  const ipcMain = fakeIpcMain();
  const traces = [];
  let now = 100;
  serveMainEdge(ipcMain, edge, { echo: async ({ secret }) => ({ secret, token: 'result-token' }) }, {
    now: () => now += 5,
    nextCorrelationId: () => 'corr-1',
    onInvocation: (record) => traces.push(record),
  });
  const reply = await ipcMain.handlers.get(callChannel('trace', 'echo'))({}, { secret: 'input-secret' });
  assert.equal(reply.ok, true);
  assert.deepEqual(traces, [{ edge: 'trace', method: 'echo', correlationId: 'corr-1', status: 'ok', code: null, durationMs: 5 }]);
  const serialized = JSON.stringify(traces);
  assert.equal(serialized.includes('input-secret'), false);
  assert.equal(serialized.includes('result-token'), false);
});
