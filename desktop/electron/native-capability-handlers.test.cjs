"use strict";

const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const { createNativeCapabilityHandlers } = require('./native-capability-handlers.cjs');

async function harness(run, options = {}) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'fabushi-native-cap-test-'));
  let state = { preferences: {}, clientPersistence: {} };
  const safeStorage = options.safeStorage ?? {
    isEncryptionAvailable: () => true,
    encryptString: (value) => Buffer.from(`encrypted:${value}`, 'utf8'),
    decryptString: (bytes) => bytes.toString('utf8').replace(/^encrypted:/, ''),
  };
  const app = {
    isPackaged: false,
    getPath(name) {
      if (name === 'userData') return path.join(root, 'user-data');
      if (name === 'downloads') return path.join(root, 'downloads');
      if (name === 'temp') return path.join(root, 'temp');
      throw new Error(`unexpected app path ${name}`);
    },
    getVersion: () => 'test',
  };
  for (const name of ['userData', 'downloads', 'temp']) await fs.mkdir(app.getPath(name), { recursive: true });
  const handlers = createNativeCapabilityHandlers({
    app,
    autoUpdater: {},
    dialog: {},
    net: options.net ?? { fetch: async () => { throw new Error('unexpected fetch'); } },
    nativeTheme: {},
    safeStorage,
    shell: {},
    host: options.host ?? { request: async () => ({ ok: true, data: null }) },
    readNativeState: async () => state,
    mutateNativeState: async (mutator) => { state = await mutator(state); return state; },
    windowForEvent: () => ({}),
    broadcastNativeEvent: () => {},
  });
  try {
    await run({ root, app, handlers, getState: () => state });
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
}

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
