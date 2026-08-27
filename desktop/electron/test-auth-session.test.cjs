'use strict';

const assert = require('node:assert/strict');
const { EventEmitter } = require('node:events');
const Module = require('node:module');
const { PassThrough } = require('node:stream');
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

class FakeChild extends EventEmitter {
  constructor() {
    super();
    this.pid = 8801;
    this.stdout = new PassThrough();
    this.stderr = new PassThrough();
    this.writes = [];
    this.stdin = {
      write: (data, callback) => {
        this.writes.push(JSON.parse(String(data)));
        queueMicrotask(() => callback?.(null));
        return true;
      },
    };
  }

  latest() {
    return this.writes.at(-1);
  }

  respond(result) {
    const request = this.latest();
    this.stdout.write(`${JSON.stringify({ id: request.id, ok: true, result })}\n`);
  }

  kill() {
    this.emit('exit', 0, null);
    return true;
  }
}

function makeHost(env = { FABUSHI_FEATURE_HOST_MODE: 'test' }) {
  const child = new FakeChild();
  let spawnCount = 0;
  const host = new MahayanaHostProcess({
    app: fakeApp,
    env,
    resourcesPath: '/tmp/fabushi-resources',
    fs: { readFileSync() { throw new Error('missing'); } },
    spawn: () => {
      spawnCount += 1;
      return child;
    },
  });
  return { host, child, spawnCount: () => spawnCount };
}

async function requestAndRespond(host, child, method, params, result) {
  const pending = host.request(method, params);
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(child.latest().method, method);
  child.respond(result);
  return pending;
}

test('test auth and runtime commands share one Rust Host generation', async () => {
  const { host, child, spawnCount } = makeHost();

  assert.deepEqual(
    await requestAndRespond(host, child, 'feature.auth.status', {}, { loggedIn: false, provider: 'test' }),
    { loggedIn: false, provider: 'test' },
  );
  const attempt = await requestAndRespond(host, child, 'feature.auth.browserStart', {}, {
    attemptId: 'test-browser-login',
    loginUrl: 'about:blank#fabushi-test-browser-login',
    pollAfterMs: 120,
  });
  assert.equal(attempt.attemptId, 'test-browser-login');
  const completed = await requestAndRespond(host, child, 'feature.auth.browserPoll', { attemptId: attempt.attemptId }, {
    status: 'completed',
    provider: 'browser',
    auth: { loggedIn: true, provider: 'browser', user: { id: 'fast-e2e-browser-user' } },
  });
  assert.equal(completed.auth.loggedIn, true);
  assert.equal((await requestAndRespond(host, child, 'feature.auth.status', {}, { loggedIn: true, provider: 'browser' })).loggedIn, true);

  const accepted = await requestAndRespond(host, child, 'feature.execute', {
    command: { type: 'conversation.list', requestId: 'conversation-list-test' },
  }, { requestId: 'conversation-list-test' });
  assert.equal(accepted.requestId, 'conversation-list-test');
  const event = await requestAndRespond(host, child, 'feature.receive', { timeoutMs: 500 }, {
    type: 'conversation.listed',
    timestamp: '2026-08-27T00:00:00.000Z',
    conversations: [{ id: 'codex:agent:assistant', title: '大乘助手', kind: 'codex', pinned: true, unreadCount: 0, updatedAtMs: 0 }],
  });
  assert.equal(event.type, 'conversation.listed');
  assert.equal(event.conversations[0].id, 'codex:agent:assistant');
  assert.equal(spawnCount(), 1);
  assert.equal(host.health().generation, 1);
  host.close();
});

test('production auth requests use the same real Host boundary', async () => {
  const { host, child, spawnCount } = makeHost({});
  const pending = host.request('feature.auth.status');
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(child.latest().method, 'feature.auth.status');
  child.respond({ loggedIn: false });
  assert.deepEqual(await pending, { loggedIn: false });
  assert.equal(spawnCount(), 1);
  host.close();
});
