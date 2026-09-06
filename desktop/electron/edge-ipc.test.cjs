"use strict";

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

require('./credential-gateway.test.cjs');

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
const { NATIVE_EDGE } = require('./native-edge.cjs');
const { createTestPlatformAccount } = require('./test-platform-account.cjs');

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

test('native desktop edge exposes the complete account synchronization surface', () => {
  const requiredMethods = [
    'getAccountSync',
    'getAccountMiniApps',
    'getAccountBots',
    'addBotToAccount',
    'removeBotFromAccount',
    'addMiniAppToAccount',
    'removeMiniAppFromAccount',
    'routeMiniAppInput',
    'getMiniAppSessionProjection',
    'getMiniAppEntitlement',
    'purchaseMiniAppLifetime',
    'restoreMiniAppPurchases',
    'getMiniAppBotMessages',
    'appendMiniAppBotMessages',
    'getMiniAppCloudStorage',
    'setMiniAppCloudStorage',
    'deleteMiniAppCloudStorage',
    'reconcileAccountMiniApps',
  ];
  for (const method of requiredMethods) {
    assert.ok(NATIVE_EDGE.methods[method], `native edge is missing account sync method ${method}`);
  }
});

test('native desktop edge never exposes a plaintext credential reveal method', () => {
  assert.equal(NATIVE_EDGE.methods.revealSecret, undefined);
});

test('deterministic test account platform survives host restart for Mini App, Bot, history and CloudStorage', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fabushi-test-platform-account-'));
  const app = { getPath(name) { assert.equal(name, 'userData'); return root; } };
  try {
    let now = 1_000;
    const first = createTestPlatformAccount({ app, fs, now: () => ++now });
    const added = first.request({ method: 'POST', path: '/v1/marketplace/plugins/global-dharma/add', body: { platform: 'desktop' } });
    assert.equal(added.ok, true);
    assert.equal(added.data.bot.id, 'global-dharma-bot');
    first.request({
      method: 'POST',
      path: '/api/miniapps/global-dharma/messages',
      body: { messages: [{ messageId: 'm1', role: 'user', text: 'persist me', createdAt: '2026-08-27T00:00:00.000Z' }] },
    });
    first.request({ method: 'PUT', path: '/v1/miniapps/global-dharma/cloud-storage', body: { values: { probe: 'survives' } } });

    const restarted = createTestPlatformAccount({ app, fs, now: () => ++now });
    const apps = restarted.request({ method: 'GET', path: '/v1/marketplace/added' });
    assert.deepEqual(apps.data.apps.map((item) => item.id), ['global-dharma']);
    assert.ok(apps.data.apps[0].commands.some((command) => command.name === 'status'));
    const bots = restarted.request({ method: 'GET', path: '/v1/account/bots' });
    assert.equal(bots.data.bots[0].bot.id, 'global-dharma-bot');
    const history = restarted.request({ method: 'GET', path: '/api/miniapps/global-dharma/messages', query: { limit: 100 } });
    assert.equal(history.data.messages[0].text, 'persist me');
    const cloud = restarted.request({ method: 'GET', path: '/v1/miniapps/global-dharma/cloud-storage', query: { key: 'probe' } });
    assert.equal(cloud.data.value, 'survives');
    const sync = restarted.request({ method: 'GET', path: '/v1/account/sync' });
    assert.equal(sync.data.mode, 'snapshot');
    assert.equal(sync.data.snapshot.miniApps[0].id, 'global-dharma');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test('deterministic Global Dharma Pay test provider is idempotent and durable', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fabushi-test-platform-pay-'));
  const app = { getPath(name) { assert.equal(name, 'userData'); return root; } };
  try {
    let now = 2_000;
    const platform = createTestPlatformAccount({ app, fs, now: () => ++now });
    const before = platform.request({ method: 'GET', path: '/v1/plugins/global-dharma/entitlements/local.prayer-wheel.start' });
    assert.equal(before.data.access.allowed, false);
    const lifetime = before.data.purchaseOptions.find((option) => option.sku === 'local-prayer-wheel.lifetime');
    assert.equal(lifetime.amount, 108000);
    assert.equal(lifetime.currency, 'CNY');
    assert.deepEqual(lifetime.activeRails, ['web_provider']);
    const request = { method: 'POST', path: '/v1/miniapps/global-dharma/pay/intents', body: { sku: 'local-prayer-wheel.lifetime', rail: 'web_provider', idempotencyKey: 'journey-pay-1' } };
    const firstIntent = platform.request(request);
    const replayIntent = platform.request(request);
    assert.equal(firstIntent.data.paymentId, replayIntent.data.paymentId);
    assert.equal(firstIntent.data.amount, 108000);
    const checkoutPath = `/v1/pay/intents/${firstIntent.data.paymentId}/checkout`;
    const firstCheckout = platform.request({ method: 'POST', path: checkoutPath });
    const replayCheckout = platform.request({ method: 'POST', path: checkoutPath });
    assert.equal(firstCheckout.data.callback.duplicate, false);
    assert.equal(replayCheckout.data.callback.duplicate, true);
    const after = platform.request({ method: 'GET', path: '/v1/plugins/global-dharma/entitlements/local.prayer-wheel.start' });
    assert.equal(after.data.access.allowed, true);
    assert.equal(after.data.entitlement.expiresAt, null);
    const restarted = createTestPlatformAccount({ app, fs, now: () => ++now });
    const restored = restarted.request({ method: 'POST', path: '/v1/purchases/restore', body: { pluginId: 'global-dharma' } });
    assert.equal(restored.data.restored, true);
    assert.equal(restored.data.purchases[0].amount, 108000);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
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
