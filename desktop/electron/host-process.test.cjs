"use strict";

const assert = require('node:assert/strict');
const { EventEmitter } = require('node:events');
const Module = require('node:module');
const { PassThrough } = require('node:stream');
const test = require('node:test');

const defaultApp = {
  isPackaged: false,
  getPath(name) {
    if (name !== 'userData') throw new Error(`unexpected path: ${name}`);
    return '/tmp/fabushi-host-test';
  },
};
const originalLoad = Module._load;
Module._load = function load(request, parent, isMain) {
  if (request === 'electron') return { app: defaultApp };
  return originalLoad.call(this, request, parent, isMain);
};
const {
  DEVELOPMENT_PRODUCT_API_BASE_URL,
  MahayanaHostProcess,
  PRODUCTION_PRODUCT_API_BASE_URL,
  productApiBaseUrl,
} = require('./host-process.cjs');
Module._load = originalLoad;

class FakeChild extends EventEmitter {
  constructor(pid) {
    super();
    this.pid = pid;
    this.killed = false;
    this.stdout = new PassThrough();
    this.stderr = new PassThrough();
    this.writes = [];
    this.stdin = {
      write: (data, callback) => {
        this.writes.push(String(data));
        queueMicrotask(() => callback?.(null));
        return true;
      },
    };
  }

  requestAt(index = this.writes.length - 1) {
    return JSON.parse(this.writes[index]);
  }

  respond(id, result) {
    this.stdout.write(`${JSON.stringify({ id, ok: true, result })}\n`);
  }

  fail(id, error) {
    this.stdout.write(`${JSON.stringify({ id, ok: false, error })}\n`);
  }

  kill() {
    if (this.killed) return false;
    this.killed = true;
    this.emit('exit', 0, null);
    return true;
  }
}

function harness() {
  const children = [];
  let now = 1000;
  const host = new MahayanaHostProcess({
    app: defaultApp,
    env: { MAHAYANA_API_BASE_URL: 'https://api.example.test/' },
    platform: 'linux',
    now: () => ++now,
    spawn: (_bin, _args, options) => {
      assert.equal(options.env.MAHAYANA_API_BASE_URL, 'https://api.example.test');
      assert.equal(options.env.FABUSHI_APP_DATA, '/tmp/fabushi-host-test');
      const child = new FakeChild(7000 + children.length);
      children.push(child);
      return child;
    },
  });
  return { host, children };
}

test('product API base URL is environment-aware and rejects unsafe overrides', () => {
  assert.equal(productApiBaseUrl({ isPackaged: false }, {}), DEVELOPMENT_PRODUCT_API_BASE_URL);
  assert.equal(productApiBaseUrl({ isPackaged: true }, {}), PRODUCTION_PRODUCT_API_BASE_URL);
  assert.equal(productApiBaseUrl({ isPackaged: true }, { MAHAYANA_API_BASE_URL: 'https://api.example.test/' }), 'https://api.example.test');
  assert.throws(() => productApiBaseUrl({ isPackaged: true }, { MAHAYANA_API_BASE_URL: 'http://api.example.test' }), /clean HTTPS/);
  assert.throws(() => productApiBaseUrl({ isPackaged: true }, { MAHAYANA_API_BASE_URL: 'https://u:p@api.example.test' }), /clean HTTPS/);
  assert.throws(() => productApiBaseUrl({ isPackaged: true }, { MAHAYANA_API_BASE_URL: 'https://api.example.test?x=1' }), /clean HTTPS/);
});

test('host resolves structured requests and reports health for the active generation', async () => {
  const { host, children } = harness();
  const pending = host.request('feature.info', { hello: 'world' });
  assert.equal(children.length, 1);
  const request = children[0].requestAt();
  assert.equal(request.method, 'feature.info');
  children[0].respond(request.id, { ready: true });
  assert.deepEqual(await pending, { ready: true });
  const health = host.health();
  assert.equal(health.state, 'running');
  assert.equal(health.generation, 1);
  assert.equal(health.pid, 7000);
  assert.equal(health.pending, 0);
  assert.equal(health.unexpectedExitCount, 0);
  host.close();
});

test('stale process termination cannot reject requests from a newer generation', async () => {
  const { host, children } = harness();
  const first = host.request('feature.receive', {});
  const old = children[0];
  old.emit('error', new Error('old generation failed'));
  await assert.rejects(first, /old generation failed/);
  assert.equal(host.health().state, 'stopped');
  assert.equal(host.health().unexpectedExitCount, 1);

  const second = host.request('feature.info', {});
  assert.equal(children.length, 2);
  const current = children[1];
  assert.equal(host.health().generation, 2);

  old.emit('exit', 1, 'SIGTERM');
  assert.equal(host.health().generation, 2);
  assert.equal(host.health().state, 'running');
  assert.equal(host.health().unexpectedExitCount, 1);

  const request = current.requestAt();
  current.respond(request.id, { generation: 2 });
  assert.deepEqual(await second, { generation: 2 });
  host.close();
});

test('restart rejects only the active generation and immediately creates a fresh process', async () => {
  const { host, children } = harness();
  const pending = host.request('feature.receive', {});
  const previous = children[0];
  const fresh = host.restart('fault injection');
  await assert.rejects(pending, /restarted: fault injection/);
  assert.equal(previous.killed, true);
  assert.equal(fresh, children[1]);
  assert.equal(host.health().generation, 2);
  assert.equal(host.health().state, 'running');
  host.close();
});

test('close rejects pending work and makes shutdown terminal', async () => {
  const { host } = harness();
  const pending = host.request('feature.receive', {});
  host.close();
  await assert.rejects(pending, /host closed/);
  assert.equal(host.health().state, 'closed');
  assert.equal(host.health().closed, true);
  await assert.rejects(host.request('feature.info', {}), /host is closed/);
});
