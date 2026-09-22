'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const {
  createCoordinatorPortBroker,
  createProductionRendererDesktopBridge,
  normalizeCursorAuthStatus,
  normalizeThemeState,
} = require('./production-renderer-bridge.cjs');

test('legacy account and theme values are normalized to the recovered desktop contract', () => {
  assert.deepEqual(normalizeCursorAuthStatus({ loggedIn: false }), { kind: 'logged-out' });
  assert.deepEqual(
    normalizeCursorAuthStatus({ loggedIn: true, user: { id: 'u1', email: 'u@example.test' }, displayName: 'U' }),
    { kind: 'logged-in', authId: 'u1', email: 'u@example.test', displayName: 'U' },
  );
  assert.deepEqual(normalizeThemeState({ preference: 'system', dark: false }), { preference: 'system', resolved: 'light' });
});

test('desktop bridge routes recovered cursor/window/onboarding calls through the trusted native edge', async () => {
  const calls = [];
  const bridge = createProductionRendererDesktopBridge({
    invokeNative: async (method, params) => {
      calls.push({ method, params });
      if (method === 'getAccountAuthStatus') return { loggedIn: false };
      if (method === 'getThemeState') return { preference: 'dark', dark: true };
      if (method === 'getOnboardingSeen') return true;
      if (method === 'getWindowState') return { focused: true };
      return null;
    },
    subscribeNative: () => () => {},
    invokeMahayana: async () => null,
    getZoomFactor: () => 1.25,
    platform: 'darwin',
  });
  assert.deepEqual(await bridge.cursorAccount.getStatus(), { kind: 'logged-out' });
  assert.deepEqual(await bridge.theme.get(), { preference: 'dark', resolved: 'dark' });
  assert.equal(await bridge.onboarding.getSeen(), true);
  assert.deepEqual(await bridge.getWindowState(), { focused: true });
  assert.equal(bridge.getZoomFactor(), 1.25);
  assert.deepEqual(calls.map((call) => call.method), ['getAccountAuthStatus', 'getThemeState', 'getOnboardingSeen', 'getWindowState']);
});

test('coordinator broker has a single owner and redelivers replacement ports after request', () => {
  let requested = 0;
  const seen = [];
  const broker = createCoordinatorPortBroker(() => { requested += 1; });
  const claim = broker.bridge.claim({ onPort: (port) => seen.push(port) });
  assert.ok(claim);
  assert.equal(broker.bridge.claim({ onPort() {} }), null);
  claim.request();
  broker.deliver('p1');
  broker.deliver('p2');
  assert.equal(requested, 1);
  assert.deepEqual(seen, ['p1', 'p2']);
  claim.release();
});
