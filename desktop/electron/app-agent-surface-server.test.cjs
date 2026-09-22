'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { createAppAgentSurfaceServer, isLoopbackAddress } = require('./app-agent-surface-server.cjs');

test('App Agent Surface bridge is private, authenticated, bounded, and removes discovery on close', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'fabushi-app-agent-'));
  const discoveryPath = path.join(root, 'private', 'bridge.json');
  let bridge;
  const received = [];
  try {
    bridge = createAppAgentSurfaceServer({
      discoveryPath,
      onRequest(request) {
        received.push(request);
        setImmediate(() => bridge.respond({
          requestId: request.requestId,
          ok: true,
          result: { available: true, operation: request.operation, generation: 7 },
        }));
      },
    });
    await bridge.start();
    const discovery = JSON.parse(await fs.readFile(discoveryPath, 'utf8'));
    assert.equal(discovery.version, 1);
    assert.equal(discovery.appId, 'fabushi.desktop');
    assert.match(discovery.origin, /^http:\/\/127\.0\.0\.1:\d+$/u);
    assert.match(discovery.token, /^[A-Za-z0-9_-]{64}$/u);
    if (process.platform !== 'win32') {
      const mode = (await fs.stat(discoveryPath)).mode & 0o777;
      assert.equal(mode, 0o600);
    }

    const unauthorized = await fetch(`${discovery.origin}/v1/status`, { method: 'POST', body: '{}' });
    assert.equal(unauthorized.status, 401);

    const response = await fetch(`${discovery.origin}/v1/status`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${discovery.token}`, 'Content-Type': 'application/json' },
      body: JSON.stringify({ input: { probe: true } }),
    });
    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), {
      ok: true,
      result: { available: true, operation: 'status', generation: 7 },
    });
    assert.equal(received.length, 1);
    assert.deepEqual(received[0].input, { probe: true });
    assert.equal(bridge.respond({ requestId: received[0].requestId, ok: true, result: {} }), false);

    const notFound = await fetch(`${discovery.origin}/v1/arbitrary-javascript`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${discovery.token}`, 'Content-Type': 'application/json' },
      body: '{}',
    });
    assert.equal(notFound.status, 404);
  } finally {
    await bridge?.close();
    await assert.rejects(fs.access(discoveryPath));
    await fs.rm(root, { recursive: true, force: true });
  }
});

test('App Agent Surface bridge rejects non-loopback binding and classifies loopback peers', () => {
  assert.equal(isLoopbackAddress('127.0.0.1'), true);
  assert.equal(isLoopbackAddress('::1'), true);
  assert.equal(isLoopbackAddress('::ffff:127.0.0.1'), true);
  assert.equal(isLoopbackAddress('10.0.0.4'), false);
  assert.throws(() => createAppAgentSurfaceServer({
    host: '0.0.0.0',
    discoveryPath: path.join(os.tmpdir(), 'bad.json'),
    onRequest() {},
  }), /loopback/u);
});


test('App Agent Surface bridge rechecks the local computer-control policy before every operation', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'fabushi-app-agent-policy-'));
  const discoveryPath = path.join(root, 'bridge.json');
  let allowed = false;
  let dispatched = 0;
  const bridge = createAppAgentSurfaceServer({
    discoveryPath,
    authorize() { return { allowed }; },
    onRequest(request) {
      dispatched += 1;
      setImmediate(() => bridge.respond({ requestId: request.requestId, ok: true, result: { available: true } }));
    },
  });
  try {
    await bridge.start();
    const discovery = JSON.parse(await fs.readFile(discoveryPath, 'utf8'));
    const request = () => fetch(`${discovery.origin}/v1/status`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${discovery.token}`, 'Content-Type': 'application/json' },
      body: '{"input":{}}',
    });
    const denied = await request();
    assert.equal(denied.status, 403);
    assert.equal(dispatched, 0);
    allowed = true;
    const accepted = await request();
    assert.equal(accepted.status, 200);
    assert.equal(dispatched, 1);
  } finally {
    await bridge.close();
    await fs.rm(root, { recursive: true, force: true });
  }
});
