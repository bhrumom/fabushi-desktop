"use strict";

const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const { EventEmitter } = require('node:events');
const test = require('node:test');

const { createNativeCapabilityHandlers } = require('./native-capability-handlers.cjs');

async function harness(run, options = {}) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'fabushi-native-cap-test-'));
  let state = options.initialState ?? { preferences: {}, clientPersistence: {} };
  const safeStorage = options.safeStorage ?? {
    isEncryptionAvailable: () => true,
    encryptString: (value) => Buffer.from(`encrypted:${value}`, 'utf8'),
    decryptString: (bytes) => bytes.toString('utf8').replace(/^encrypted:/, ''),
  };
  const app = {
    isPackaged: options.isPackaged ?? false,
    getPath(name) {
      if (name === 'userData') return path.join(root, 'user-data');
      if (name === 'downloads') return path.join(root, 'downloads');
      if (name === 'temp') return path.join(root, 'temp');
      throw new Error(`unexpected app path ${name}`);
    },
    getVersion: () => 'test',
    quit: () => options.onAppQuit?.(),
  };
  for (const name of ['userData', 'downloads', 'temp']) await fs.mkdir(app.getPath(name), { recursive: true });
  const handlers = createNativeCapabilityHandlers({
    app,
    autoUpdater: options.autoUpdater ?? {},
    dialog: {},
    net: options.net ?? { fetch: async () => { throw new Error('unexpected fetch'); } },
    nativeTheme: {},
    safeStorage,
    shell: options.shell ?? {},
    host: options.host ?? { request: async () => ({ ok: true, data: null }) },
    readNativeState: async () => state,
    mutateNativeState: async (mutator) => { state = await mutator(state); return state; },
    getDesktopUpdateStatus: options.getDesktopUpdateStatus,
    setDesktopUpdateStatus: options.setDesktopUpdateStatus,
    windowForEvent: () => ({}),
    broadcastNativeEvent: () => {},
    setDesktopUpdateInstallInProgress: options.setDesktopUpdateInstallInProgress,
  });
  try {
    await run({ root, app, handlers, getState: () => state });
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
}


test('marketplace native capabilities use canonical Feature Host method names', async () => {
  const calls = [];
  const host = {
    async request(method, params = {}) {
      calls.push([method, params]);
      if (method === 'feature.marketplace.browse') {
        return { plugins: [{ pluginId: 'mcp-demo', displayName: 'MCP Demo', latestVersion: '1.0.0' }] };
      }
      if (method === 'feature.marketplace.release') {
        return { pluginId: params.pluginId, version: params.version, releaseManifest: { pluginId: params.pluginId, version: params.version } };
      }
      if (method === 'feature.plugin.active') return null;
      if (method === 'feature.plugin.install') return { pluginId: 'mcp-demo', version: '1.0.0' };
      if (method === 'feature.plugin.uninstall') return { pluginId: params.pluginId, removed: true };
      throw new Error(`unexpected Host method ${method}`);
    },
  };
  await harness(async ({ handlers }) => {
    const catalog = await handlers.getMcpCatalog({});
    assert.equal(catalog[0].pluginId, 'mcp-demo');
    await handlers.getEffectivePlugins({});
    await handlers.installEntry({ pluginId: 'mcp-demo', version: '1.0.0' });
    await handlers.uninstallPlugin({ pluginId: 'mcp-demo' });
  }, { host });
  assert.ok(calls.some(([method]) => method === 'feature.marketplace.browse'));
  assert.ok(calls.some(([method]) => method === 'feature.marketplace.release'));
  assert.ok(calls.some(([method]) => method === 'feature.plugin.active'));
  assert.ok(calls.some(([method]) => method === 'feature.plugin.install'));
  assert.ok(calls.some(([method]) => method === 'feature.plugin.uninstall'));
  assert.equal(calls.some(([method]) => /^(marketplace|plugin)\./.test(method)), false);
});

test('Mini App bot lifecycle uses authenticated marketplace platform routes', async () => {
  const calls = [];
  const host = {
    async request(method, params = {}) {
      calls.push([method, params]);
      if (method !== 'platform.request') throw new Error(`unexpected Host method ${method}`);
      return { ok: true, data: { method: params.method, path: params.path, body: params.body ?? null } };
    },
  };
  await harness(async ({ handlers }) => {
    const added = await handlers.addMiniAppToAccount({ pluginId: 'global-dharma' });
    assert.equal(added.path, '/v1/marketplace/plugins/global-dharma/add');
    const routed = await handlers.routeMiniAppInput({ pluginId: 'global-dharma', input: '/global-dharma:open' });
    assert.equal(routed.path, '/v1/marketplace/plugins/global-dharma/route');
    assert.equal(routed.body.input, '/global-dharma:open');
    const removed = await handlers.removeMiniAppFromAccount({ pluginId: 'global-dharma' });
    assert.equal(removed.path, '/v1/marketplace/plugins/global-dharma/add');
  }, { host });
  assert.deepEqual(calls.map(([, params]) => params.method), ['POST', 'POST', 'DELETE']);
  assert.ok(calls.every(([method]) => method === 'platform.request'));
});

test('Mini App runtime facade scopes renderer calls to installed Tool Contract and canonical entitlement', async () => {
  const calls = [];
  let entitled = false;
  const host = {
    async request(method, params = {}) {
      calls.push([method, params]);
      if (method === 'platform.request' && params.method === 'GET' && params.path === '/v1/marketplace/added') {
        return { ok: true, data: { apps: [{
          id: 'global-dharma',
          commands: [
            { name: 'status', tool: 'status' },
            { name: 'start', tool: 'start' },
          ],
        }] } };
      }
      if (method === 'platform.request' && params.method === 'GET' && params.path.includes('/entitlements/')) {
        return { ok: true, data: { access: { protected: true, allowed: entitled } } };
      }
      if (method === 'runtime.call') {
        return { content: [{ type: 'text', text: `${params.name} completed` }], structuredContent: { tool: params.name } };
      }
      throw new Error(`unexpected Host method ${method}`);
    },
  };
  await harness(async ({ handlers }) => {
    const status = await handlers.callMiniAppRuntimeTool({ pluginId: 'global-dharma', name: 'status', arguments: {} });
    assert.equal(status.structuredContent.tool, 'status');
    await assert.rejects(
      handlers.callMiniAppRuntimeTool({ pluginId: 'global-dharma', name: 'deploy_latest', arguments: {} }),
      /outside global-dharma's installed Tool Contract/,
    );
    await assert.rejects(
      handlers.callMiniAppRuntimeTool({ pluginId: 'global-dharma', name: 'start', arguments: {} }),
      /requires an active local\.prayer-wheel\.start entitlement/,
    );
    entitled = true;
    const started = await handlers.callMiniAppRuntimeTool({ pluginId: 'global-dharma', name: 'start', arguments: {} });
    assert.equal(started.structuredContent.tool, 'start');
    await assert.rejects(
      handlers.callMiniAppRuntimeTool({ pluginId: 'not-installed', name: 'status', arguments: {} }),
      /not installed for this account/,
    );
  }, { host });
  assert.deepEqual(
    calls.filter(([method]) => method === 'runtime.call').map(([, params]) => [params.pluginId, params.name]),
    [['global-dharma', 'status'], ['global-dharma', 'start']],
  );
  const mahayanaEdge = await fs.readFile(path.join(__dirname, 'mahayana-edge.cjs'), 'utf8');
  assert.doesNotMatch(mahayanaEdge, /['"]runtime\.call['"]/);
  const miniAppHost = await fs.readFile(path.join(__dirname, '../src/miniapp-webmcp-host.ts'), 'utf8');
  assert.match(miniAppHost, /const lifetimePurchaseKeys = new Map/);
  assert.doesNotMatch(miniAppHost, /sessionStorage\.(?:getItem|setItem)\('fabushi-global-dharma-lifetime-key'/);
});

test('Global Dharma desktop facade projects session and enforces canonical CNY 1080 lifetime purchase', async () => {
  const calls = [];
  let entitled = false;
  const host = {
    async request(method, params = {}) {
      calls.push([method, params]);
      if (method === 'feature.auth.status') return { loggedIn: true, provider: 'test', user: { id: 'ci-user', username: 'fabushi_mcp_ci_test', nickname: 'CI' } };
      if (method !== 'platform.request') throw new Error(`unexpected Host method ${method}`);
      if (params.method === 'GET' && params.path.includes('/entitlements/')) return { ok: true, data: {
        entitlement: entitled ? { status: 'active', capability: 'local.prayer-wheel.start', expiresAt: null } : null,
        access: { protected: true, allowed: entitled, reason: entitled ? 'active_durable_entitlement' : 'not_entitled' },
        purchaseOptions: [{ productId: 'prod.global-dharma.local-prayer-wheel.lifetime', sku: 'local-prayer-wheel.lifetime', displayName: '本地转经轮永久版', productKind: 'digital_durable', currency: 'CNY', amount: 108000, activeRails: ['web_provider'] }],
      } };
      if (params.method === 'POST' && params.path === '/v1/miniapps/global-dharma/pay/intents') {
        assert.deepEqual(params.body, { sku: 'local-prayer-wheel.lifetime', rail: 'web_provider', idempotencyKey: 'desktop-journey-1' });
        return { ok: true, data: { paymentId: 'pay-1', amount: 108000, currency: 'CNY' } };
      }
      if (params.method === 'POST' && params.path === '/v1/pay/intents/pay-1/checkout') {
        entitled = true;
        return { ok: true, data: { payment: { paymentId: 'pay-1', status: 'succeeded' }, checkoutAction: { kind: 'test' } } };
      }
      if (params.method === 'POST' && params.path === '/v1/purchases/restore') return { ok: true, data: { restored: true, purchases: [{ amount: 108000, currency: 'CNY' }] } };
      throw new Error(`unexpected platform request ${params.method} ${params.path}`);
    },
  };
  await harness(async ({ handlers }) => {
    const session = await handlers.getMiniAppSessionProjection({ pluginId: 'global-dharma' });
    assert.equal(session.loggedIn, true);
    assert.equal(session.account.username, 'fabushi_mcp_ci_test');
    assert.equal(session.tokenExposed, false);
    assert.equal(JSON.stringify(session).includes('accessToken'), false);
    const purchase = await handlers.purchaseMiniAppLifetime({ pluginId: 'global-dharma', capability: 'local.prayer-wheel.start', idempotencyKey: 'desktop-journey-1' });
    assert.equal(purchase.status, 'entitled');
    assert.equal(purchase.product.amount, 108000);
    assert.equal(purchase.entitlement.access.allowed, true);
    const restore = await handlers.restoreMiniAppPurchases({ pluginId: 'global-dharma', capability: 'local.prayer-wheel.start' });
    assert.equal(restore.entitlement.access.allowed, true);
  }, { host });
  assert.ok(calls.some(([, params]) => params.path === '/v1/miniapps/global-dharma/pay/intents'));
  assert.ok(calls.some(([, params]) => params.path === '/v1/pay/intents/pay-1/checkout'));
  assert.ok(calls.some(([, params]) => params.path === '/v1/purchases/restore'));
});

test('account sync native capabilities reconcile remote Mini Apps and expose Bot history/cloud routes', async () => {
  const calls = [];
  const installed = new Map();
  const host = {
    async request(method, params = {}) {
      calls.push([method, params]);
      if (method === 'platform.request') {
        if (params.path === '/v1/marketplace/added') {
          return { ok: true, data: { accountSynchronized: true, cursor: 'as1:2', apps: [{ id: 'global-dharma', version: '1.0.0' }] } };
        }
        return { ok: true, data: { method: params.method, path: params.path, query: params.query ?? null, body: params.body ?? null } };
      }
      if (method === 'feature.marketplace.browse') {
        return { plugins: [{ pluginId: 'global-dharma', displayName: 'Global Dharma', latestVersion: '1.0.0' }] };
      }
      if (method === 'feature.plugin.active') return installed.get(params.pluginId) ?? null;
      if (method === 'feature.marketplace.release') return { pluginId: params.pluginId, version: params.version, releaseManifest: { pluginId: params.pluginId, version: params.version } };
      if (method === 'feature.plugin.install') {
        const pointer = { pluginId: params.release.pluginId, version: params.release.version, installedPath: '/tmp/test' };
        installed.set(pointer.pluginId, pointer);
        return pointer;
      }
      if (method === 'feature.plugin.uninstall') {
        installed.delete(params.pluginId);
        return { pluginId: params.pluginId, removed: true };
      }
      throw new Error(`unexpected Host method ${method}`);
    },
  };
  await harness(async ({ handlers, getState }) => {
    const sync = await handlers.getAccountSync({ cursor: 'as1:1', limit: 20 });
    assert.equal(sync.path, '/v1/account/sync');
    assert.equal(sync.query.cursor, 'as1:1');
    const reconciliation = await handlers.reconcileAccountMiniApps({});
    assert.deepEqual(reconciliation.desired, ['global-dharma']);
    assert.equal(reconciliation.installed[0].pluginId, 'global-dharma');
    assert.deepEqual(getState().accountManagedMiniApps, ['global-dharma']);
    const history = await handlers.getMiniAppBotMessages({ pluginId: 'global-dharma', after: '2026-01-01', limit: 50 });
    assert.equal(history.path, '/api/miniapps/global-dharma/messages');
    const appended = await handlers.appendMiniAppBotMessages({ pluginId: 'global-dharma', messages: [{ messageId: 'm1', role: 'user', text: 'hello' }] });
    assert.equal(appended.path, '/api/miniapps/global-dharma/messages');
    const cloud = await handlers.setMiniAppCloudStorage({ pluginId: 'global-dharma', values: { mode: 'local' } });
    assert.equal(cloud.path, '/v1/miniapps/global-dharma/cloud-storage');
    const bots = await handlers.getAccountBots({});
    assert.equal(bots.path, '/v1/account/bots');
  }, { host });
  assert.ok(calls.some(([method]) => method === 'feature.plugin.install'));
});

test('local tool permission cannot exceed the administrator ceiling', async () => {
  const previous = process.env.FABUSHI_LOCAL_TOOL_PERMISSION_CEILING;
  process.env.FABUSHI_LOCAL_TOOL_PERMISSION_CEILING = 'ask';
  try {
    await harness(async ({ handlers, getState }) => {
      await assert.rejects(handlers.setLocalToolPermission({ permission: 'always' }), /administrator ceiling/);
      assert.equal(await handlers.setLocalToolPermission({ permission: 'ask' }), 'ask');
      assert.equal(getState().preferences.localToolPermission, 'ask');
      await assert.rejects(handlers.setLocalToolPermission({ permission: 'root' }), /Unsupported local tool permission/);
    });
  } finally {
    if (previous === undefined) delete process.env.FABUSHI_LOCAL_TOOL_PERMISSION_CEILING;
    else process.env.FABUSHI_LOCAL_TOOL_PERMISSION_CEILING = previous;
  }
});

test('secret operations fail closed when OS-backed encryption is unavailable', async () => {
  await harness(async ({ handlers }) => {
    await assert.rejects(handlers.revealSecret({ name: 'api/token' }), /OS-backed secret encryption is not available/);
    await assert.rejects(handlers.upsertSecrets({ name: 'api/token', value: 'plaintext' }), /OS-backed secret encryption is not available/);
  }, { safeStorage: { isEncryptionAvailable: () => false } });
});

test('secret vault never persists plaintext and listSecrets does not reveal values', async () => {
  await harness(async ({ app, handlers }) => {
    const listed = await handlers.upsertSecrets({ name: 'api/token', value: 'super-secret-value' });
    assert.deepEqual(listed.map((item) => item.name), ['api/token']);
    assert.equal(Object.prototype.hasOwnProperty.call(listed[0], 'value'), false);
    assert.equal(await handlers.revealSecret({ name: 'api/token' }), 'super-secret-value');
    const raw = await fs.readFile(path.join(app.getPath('userData'), 'secure', 'secrets.json'), 'utf8');
    assert.equal(raw.includes('super-secret-value'), false);
    assert.equal(raw.includes('encrypted:'), false, 'encrypted bytes are stored base64 encoded');
  });
});

test('inference Router readiness reports local sessions and encrypted OpenRouter configuration without secrets', async () => {
  const previous = {
    CODEX_HOME: process.env.CODEX_HOME,
    CODEX_PATH: process.env.CODEX_PATH,
    CLAUDE_CODE_PATH: process.env.CLAUDE_CODE_PATH,
    DOCKER_PATH: process.env.DOCKER_PATH,
    MAHAYANA_DOCKER_IMAGE: process.env.MAHAYANA_DOCKER_IMAGE,
  };
  try {
    await harness(async ({ root, handlers }) => {
      const codexHome = path.join(root, 'codex-home');
      const bin = path.join(root, process.platform === 'win32' ? 'provider.exe' : 'provider');
      await fs.mkdir(codexHome, { recursive: true });
      await fs.writeFile(path.join(codexHome, 'auth.json'), JSON.stringify({ tokens: { access_token: 'test-access-token' } }), { mode: 0o600 });
      await fs.writeFile(bin, 'test');
      process.env.CODEX_HOME = codexHome;
      process.env.CODEX_PATH = bin;
      process.env.CLAUDE_CODE_PATH = bin;
      process.env.DOCKER_PATH = bin;
      process.env.MAHAYANA_DOCKER_IMAGE = `ghcr.io/bhrumom/fabushi-sandbox@sha256:${'a'.repeat(64)}`;
      await handlers.upsertSecrets({ name: 'inference/openrouter/api-key', value: 'never-return-this-key' });
      await handlers.upsertSecrets({ name: 'inference/claude/api-key', value: 'never-return-this-claude-key' });

      const status = await handlers.getInferenceRouterStatus();
      assert.equal(status.schemaVersion, 1);
      assert.equal(status.providers.find((provider) => provider.id === 'fabushi').available, true);
      assert.equal(status.providers.find((provider) => provider.id === 'codex').authenticated, true);
      assert.equal(status.providers.find((provider) => provider.id === 'openrouter').available, true);
      assert.equal(status.providers.find((provider) => provider.id === 'claude-code').available, true);
      assert.equal(status.sandboxes.find((sandbox) => sandbox.id === 'local-docker').available, true);
      assert.equal(JSON.stringify(status).includes('never-return-this-key'), false);
      assert.equal(JSON.stringify(status).includes('never-return-this-claude-key'), false);
    });
  } finally {
    for (const [name, value] of Object.entries(previous)) {
      if (value === undefined) delete process.env[name];
      else process.env[name] = value;
    }
  }
});

test('inference readiness rejects an unreadable encrypted Provider credential', async () => {
  await harness(async ({ app, handlers }) => {
    const secureRoot = path.join(app.getPath('userData'), 'secure');
    await fs.mkdir(secureRoot, { recursive: true });
    await fs.writeFile(path.join(secureRoot, 'secrets.json'), JSON.stringify({
      'inference/openrouter/api-key': { ciphertext: Buffer.from('corrupt').toString('base64') },
    }));
    const status = await handlers.getInferenceRouterStatus();
    assert.equal(status.providers.find((provider) => provider.id === 'openrouter').available, false);
  }, {
    safeStorage: {
      isEncryptionAvailable: () => true,
      encryptString: (value) => Buffer.from(value),
      decryptString: () => { throw new Error('corrupt ciphertext'); },
    },
  });
});

test('usage summary preserves provider-reported token breakdown without prompt content', async () => {
  const timestampMs = Date.now();
  await harness(async ({ handlers }) => {
    const summary = await handlers.getUsageSummary();
    const codex = summary.byProvider.find((item) => item.provider === 'codex');
    assert.deepEqual(codex, {
      provider: 'codex',
      requests: 1,
      inputTokens: 120,
      cachedInputTokens: 40,
      outputTokens: 30,
      reasoningTokens: 10,
      totalTokens: 160,
      lifetimeTokens: 900,
      lastUsedAtMs: timestampMs,
    });
    assert.equal(JSON.stringify(summary).includes('prompt'), false);
  }, {
    initialState: {
      preferences: {},
      clientPersistence: {},
      usageEvents: [{ timestampMs, provider: 'codex', inputTokens: 120, cachedInputTokens: 40, outputTokens: 30, reasoningTokens: 10, totalTokens: 160 }],
      usageLifetimeTokens: 900,
      usageLifetimeByProvider: { codex: 900 },
      usageUpdatedAtMs: timestampMs,
    },
  });
});

test('credential changes can restart only the inference Host generation', async () => {
  const reasons = [];
  const host = {
    restart(reason) { reasons.push(reason); },
    health() { return { state: 'running', generation: 2 }; },
    async request() { return { ok: true, data: null }; },
  };
  await harness(async ({ handlers }) => {
    assert.deepEqual(handlers.restartInferenceRouter(), { state: 'running', generation: 2 });
  }, { host });
  assert.deepEqual(reasons, ['inference Provider credential changed']);
});

test('managed attachment operations reject path escape attempts', async () => {
  await harness(async ({ root, handlers }) => {
    const outside = path.join(root, 'outside.bin');
    await fs.writeFile(outside, Buffer.from('outside'));
    await assert.rejects(handlers.commitStagedAttachments({ paths: [outside] }), /escaped the managed desktop storage root/);
    await assert.rejects(handlers.discardStagedAttachment({ path: outside }), /escaped the managed desktop storage root/);
  });
});

test('attachment download rejects non-HTTPS URLs before network access', async () => {
  let fetches = 0;
  await harness(async ({ handlers }) => {
    await assert.rejects(handlers.downloadAttachment({ url: 'http://example.test/file.bin' }), /Only HTTPS attachments/);
    assert.equal(fetches, 0);
  }, { net: { fetch: async () => { fetches += 1; throw new Error('must not fetch'); } } });
});

test('diagnostic reports redact nested secrets before persistence', async () => {
  await harness(async ({ app, handlers }) => {
    await handlers.reportClientFailure({
      token: 'token-value',
      ordinary: 'safe-value',
      nested: { password: 'password-value', authorization: 'bearer value' },
    });
    const raw = await fs.readFile(path.join(app.getPath('userData'), 'diagnostics', 'native-events.ndjson'), 'utf8');
    const record = JSON.parse(raw.trim());
    assert.equal(record.payload.token, '[redacted]');
    assert.equal(record.payload.nested.password, '[redacted]');
    assert.equal(record.payload.nested.authorization, '[redacted]');
    assert.equal(record.payload.ordinary, 'safe-value');
    assert.equal(raw.includes('token-value'), false);
    assert.equal(raw.includes('password-value'), false);
  });
});


test('desktop update click uses live status even while persisted state is stale', async () => {
  const calls = [];
  let liveStatus = { type: 'available', version: '1.0.9' };
  const autoUpdater = new EventEmitter();
  autoUpdater.downloadUpdate = async () => {
    calls.push('download');
    setImmediate(() => autoUpdater.emit('update-downloaded', { version: '1.0.9' }));
    return ['/tmp/fabushi-update.zip'];
  };
  autoUpdater.quitAndInstall = (silent, forceRunAfter) => { calls.push(['install', silent, forceRunAfter]); };
  await harness(async ({ handlers, getState }) => {
    assert.equal(getState().updateStatus.version, '1.0.798', 'disk intentionally begins stale');
    const result = await handlers.quitAndInstallUpdate({ expectedVersion: '1.0.9' });
    assert.equal(result.installed, true);
    assert.equal(result.version, '1.0.9');
    await new Promise((resolve) => setTimeout(resolve, 180));
    assert.deepEqual(calls, ['download', ['install', false, true]]);
  }, {
    isPackaged: true,
    autoUpdater,
    initialState: { preferences: {}, clientPersistence: {}, updateStatus: { type: 'upToDate', version: '1.0.798' } },
    getDesktopUpdateStatus: async () => liveStatus,
    setDesktopUpdateStatus: async (status) => { liveStatus = status; return status; },
  });
});

test('desktop update click downloads a GitHub release and schedules replacement restart', async () => {
  const calls = [];
  let installationInProgress = false;
  let appQuitCount = 0;
  const autoUpdater = new EventEmitter();
  autoUpdater.downloadUpdate = async () => {
    calls.push('download');
    setImmediate(() => autoUpdater.emit('update-downloaded', { version: '1.0.3' }));
    return ['/tmp/fabushi-update.zip'];
  };
  autoUpdater.quitAndInstall = (silent, forceRunAfter) => { calls.push(['install', silent, forceRunAfter]); };
  await harness(async ({ handlers, getState }) => {
    const result = await handlers.quitAndInstallUpdate({ expectedVersion: '1.0.3' });
    assert.equal(result.installed, true);
    assert.equal(result.version, '1.0.3');
    assert.equal(getState().updateStatus.type, 'staging');
    await new Promise((resolve) => setTimeout(resolve, 180));
    assert.deepEqual(calls, ['download', ['install', false, true]]);
  }, {
    isPackaged: true,
    autoUpdater,
    initialState: {
      preferences: {},
      clientPersistence: {},
      updateStatus: { type: 'available', version: '1.0.3' },
    },
    setDesktopUpdateInstallInProgress: (value) => { installationInProgress = value; },
    onAppQuit: () => { appQuitCount += 1; },
  });
  assert.equal(installationInProgress, true);
  assert.equal(appQuitCount, 1);
});
