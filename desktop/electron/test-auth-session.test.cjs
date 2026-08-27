'use strict';

const assert = require('node:assert/strict');
const Module = require('node:module');
const test = require('node:test');

const fakeApp = {
  isPackaged: true,
  getPath(name) {
    if (name !== 'userData') throw new Error(`unexpected path: ${name}`);
    return '/tmp/fabushi-test-auth-session';
  },
};

const originalLoad = Module._load;
Module._load = function load(request, parent, isMain) {
  if (request === 'electron') return { app: fakeApp };
  return originalLoad.call(this, request, parent, isMain);
};
const { MahayanaHostProcess } = require('./host-process.cjs');
Module._load = originalLoad;

function memoryFs() {
  const files = new Map();
  return {
    mkdirSync() {},
    readFileSync(file) {
      if (!files.has(file)) throw Object.assign(new Error('missing'), { code: 'ENOENT' });
      return files.get(file);
    },
    writeFileSync(file, content) {
      files.set(file, String(content));
    },
  };
}

test('packaged test browser auth is durable and never spawns the real Host', async () => {
  const fs = memoryFs();
  let spawnCount = 0;
  const makeHost = () => new MahayanaHostProcess({
    app: fakeApp,
    env: { FABUSHI_FEATURE_HOST_MODE: 'test' },
    fs,
    now: () => 1_780_000_000_000,
    spawn: () => {
      spawnCount += 1;
      throw new Error('test auth must not spawn the real Host');
    },
  });

  const first = makeHost();
  assert.deepEqual(await first.request('feature.auth.status'), { loggedIn: false, provider: 'test' });
  const attempt = await first.request('feature.auth.browserStart');
  assert.match(attempt.loginUrl, /^about:blank#fabushi-test-browser-login$/);
  const completed = await first.request('feature.auth.browserPoll', { attemptId: attempt.attemptId });
  assert.equal(completed.status, 'completed');
  assert.equal(completed.auth.loggedIn, true);
  assert.equal((await first.request('feature.auth.status')).loggedIn, true);
  assert.equal(spawnCount, 0);

  const restarted = makeHost();
  assert.equal((await restarted.request('feature.auth.status')).loggedIn, true);
  assert.equal(spawnCount, 0);

  assert.deepEqual(await restarted.request('feature.auth.logout'), { loggedIn: false, provider: 'test' });
  assert.deepEqual(await restarted.request('feature.auth.status'), { loggedIn: false, provider: 'test' });
  assert.equal(spawnCount, 0);
});

test('production auth requests still use the real Host boundary', async () => {
  let spawnCount = 0;
  const host = new MahayanaHostProcess({
    app: fakeApp,
    env: {},
    resourcesPath: '/tmp/fabushi-resources',
    fs: { readFileSync() { throw new Error('missing'); } },
    spawn: () => {
      spawnCount += 1;
      throw new Error('production boundary reached');
    },
  });
  await assert.rejects(host.request('feature.auth.status'), /production boundary reached/);
  assert.equal(spawnCount, 1);
});
