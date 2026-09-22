"use strict";

const assert = require('node:assert/strict');
const { EventEmitter } = require('node:events');
const fs = require('node:fs');
const Module = require('node:module');
const os = require('node:os');
const path = require('node:path');
const { PassThrough } = require('node:stream');
const test = require('node:test');

const defaultApp = {
  isPackaged: true,
  getPath(name) {
    if (name !== 'userData') throw new Error(`unexpected path: ${name}`);
    return '/tmp/fabushi-remote-device-test';
  },
};
const originalLoad = Module._load;
Module._load = function load(request, parent, isMain) {
  if (request === 'electron') return { app: defaultApp };
  return originalLoad.call(this, request, parent, isMain);
};
const {
  OFFICIAL_DEVICE_GATEWAY_URL,
  RemoteDeviceAgentSupervisor,
  inheritedNodeExecPath,
  remoteDeviceGatewayUrl,
  remoteDeviceRuntime,
  retryDelayMs,
  sessionExpirationMs,
  sessionRefreshDelay,
  validAgentSession,
} = require('./remote-device-agent-supervisor.cjs');
Module._load = originalLoad;

test('packaged apps always use the official account-scoped device gateway', () => {
  assert.equal(remoteDeviceGatewayUrl({ isPackaged: true }, {
    FABUSHI_REMOTE_DEVICE_GATEWAY_URL: 'wss://attacker.example/agent',
  }), OFFICIAL_DEVICE_GATEWAY_URL);
  assert.equal(remoteDeviceGatewayUrl({ isPackaged: false }, {}), null);
  assert.throws(
    () => remoteDeviceGatewayUrl({ isPackaged: false }, { FABUSHI_REMOTE_DEVICE_GATEWAY_URL: 'https://example.test/agent' }),
    /clean wss/u,
  );
});


test('packaged macOS remote agents use the sandbox-inheriting Electron Helper instead of respawning the app executable', () => {
  const resourcesPath = '/Applications/fabushi.app/Contents/Resources';
  const helper = '/Applications/fabushi.app/Contents/Frameworks/Fabushi Helper.app/Contents/MacOS/Fabushi Helper';
  const seen = [];
  const selected = inheritedNodeExecPath({
    app: { isPackaged: true },
    platform: 'darwin',
    resourcesPath,
    execPath: '/Applications/fabushi.app/Contents/MacOS/fabushi',
    fs: { statSync(candidate) { seen.push(candidate); return { isFile: () => candidate === helper }; } },
  });
  assert.equal(selected, helper);
  assert.ok(!seen.includes('/Applications/fabushi.app/Contents/MacOS/fabushi'));
});

test('packaged macOS remote agents fail closed when an inherited Electron Helper is unavailable', () => {
  assert.equal(inheritedNodeExecPath({
    app: { isPackaged: true },
    platform: 'darwin',
    resourcesPath: '/Applications/fabushi.app/Contents/Resources',
    execPath: '/Applications/fabushi.app/Contents/MacOS/fabushi',
    fs: { statSync() { throw new Error('missing'); } },
  }), null);
});

test('logout stops the app-owned device and removes its access credential', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fabushi-remote-lifecycle-'));
  try {
    const runtime = path.join(root, 'runtime');
    const bin = path.join(runtime, 'bin');
    for (const relative of [
      'bin/fabushi-computer-mcp.js',
      'bin/fabushi-device-agent.js',
      'lib/fabushi-computer-policy.js',
      'node_modules/@modelcontextprotocol/sdk/package.json',
      'node_modules/ws/package.json',
      'node_modules/zod/package.json',
    ]) {
      const target = path.join(runtime, relative);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, relative.endsWith('.json') ? '{}\n' : '#!/usr/bin/env node\n');
    }
    let loggedIn = true;
    let child;
    class FakeChild extends EventEmitter {
      constructor() {
        super();
        this.stdout = new PassThrough();
        this.stderr = new PassThrough();
        this.killed = false;
      }
      kill() { this.killed = true; return true; }
    }
    const supervisor = new RemoteDeviceAgentSupervisor({
      app: { isPackaged: false, getPath: () => path.join(root, 'data') },
      enabled: true,
      host: {
        async request() {
          if (!loggedIn) throw new Error('not logged in');
          return {
            accessToken: 'a'.repeat(64),
            deviceId: 'desktop:test',
            sessionId: 'account-session:test',
          };
        },
      },
      env: {
        FABUSHI_REMOTE_DEVICE_GATEWAY_URL: 'wss://gateway.example.test/agent',
        FABUSHI_COMPUTER_MCP_ENTRY: path.join(bin, 'fabushi-computer-mcp.js'),
      },
      fs,
      execPath: '/opt/fabushi/fabushi',
      spawn(_command, _args, options) {
        assert.equal(options.env.FABUSHI_ACCOUNT_SESSION_FILE, '');
        assert.equal(options.env.FABUSHI_ACCOUNT_ACCESS_TOKEN, '');
        child = new FakeChild();
        return child;
      },
    });
    await supervisor.sync();
    const tokenFile = path.join(root, 'data', 'remote-device', 'account-access-token');
    assert.equal(fs.readFileSync(tokenFile, 'utf8').trim(), 'a'.repeat(64));
    loggedIn = false;
    await supervisor.sync();
    assert.equal(child.killed, true);
    assert.equal(fs.existsSync(tokenFile), false);
    supervisor.close();
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test('remote helper is disabled by default and does not request account sessions', async () => {
  let requests = 0;
  let spawns = 0;
  const supervisor = new RemoteDeviceAgentSupervisor({
    app: { isPackaged: false, getPath: () => '/tmp/fabushi-disabled-remote-device' },
    host: { async request() { requests += 1; throw new Error('should not request'); } },
    env: { FABUSHI_REMOTE_DEVICE_GATEWAY_URL: 'wss://gateway.example.test/agent' },
    spawn() { spawns += 1; throw new Error('should not spawn'); },
  });
  supervisor.start();
  await supervisor.sync();
  assert.equal(supervisor.snapshot().enabled, false);
  assert.equal(supervisor.snapshot().running, false);
  assert.equal(requests, 0);
  assert.equal(spawns, 0);
  supervisor.close();
});

test('enabling and disabling remote control is an explicit lifecycle boundary', () => {
  const supervisor = new RemoteDeviceAgentSupervisor({
    app: { isPackaged: false, getPath: () => '/tmp/fabushi-toggle-remote-device' },
    host: { async request() { throw new Error('not logged in'); } },
    env: {},
  });
  supervisor.start();
  assert.equal(supervisor.snapshot().enabled, false);
  supervisor.setEnabled(true);
  assert.equal(supervisor.snapshot().enabled, true);
  supervisor.setEnabled(false);
  assert.equal(supervisor.snapshot().enabled, false);
  assert.equal(supervisor.snapshot().running, false);
  supervisor.close();
});

test('remote helper retry policy is exponential and bounded', () => {
  assert.equal(retryDelayMs(1), 1_000);
  assert.equal(retryDelayMs(2), 2_000);
  assert.equal(retryDelayMs(3), 4_000);
  assert.equal(retryDelayMs(6), 32_000);
  assert.equal(retryDelayMs(20), 60_000);
});

test('remote session refresh is expiry-driven instead of fixed interval polling', () => {
  const nowMs = 2_000_000_000_000;
  const expiryMs = nowMs + 60 * 60_000;
  assert.equal(sessionExpirationMs(expiryMs), expiryMs);
  assert.equal(sessionExpirationMs(Math.floor(expiryMs / 1_000)), Math.floor(expiryMs / 1_000) * 1_000);
  assert.equal(sessionRefreshDelay(expiryMs, nowMs), 55 * 60_000);
  assert.equal(sessionRefreshDelay(0, nowMs), 30 * 60_000);
});

test('device agent receives a bounded access session identity', () => {
  const session = validAgentSession({
    accessToken: 'a'.repeat(64),
    deviceId: 'desktop:stable-device',
    sessionId: 'account-session:stable-device',
    username: 'tester',
    accessTokenExpiresAt: 12345,
  });
  assert.equal(session.deviceId, 'desktop:stable-device');
  assert.equal(session.accessToken.length, 64);
  assert.equal(validAgentSession({ ...session, accessToken: 'short' }), null);
  assert.equal(validAgentSession({ ...session, deviceId: '../unsafe' }), null);
});

test('remote registration is available only from a complete embedded runtime', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fabushi-remote-runtime-'));
  try {
    const runtime = path.join(root, 'runtime');
    const bin = path.join(runtime, 'bin');
    fs.mkdirSync(bin, { recursive: true });
    fs.writeFileSync(path.join(bin, 'fabushi-computer-mcp.js'), '#!/usr/bin/env node\n');
    fs.writeFileSync(path.join(bin, 'fabushi-device-agent.js'), '#!/usr/bin/env node\n');
    for (const relative of [
      'lib/fabushi-computer-policy.js',
      'node_modules/@modelcontextprotocol/sdk/package.json',
      'node_modules/ws/package.json',
      'node_modules/zod/package.json',
    ]) {
      const target = path.join(runtime, relative);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, relative.endsWith('.json') ? '{}\n' : 'export {};\n');
    }
    const selected = remoteDeviceRuntime({
      app: { isPackaged: false, getPath: () => path.join(root, 'data') },
      env: { FABUSHI_COMPUTER_MCP_ENTRY: path.join(bin, 'fabushi-computer-mcp.js') },
      platform: 'linux',
      resourcesPath: root,
      fs,
      execPath: '/opt/fabushi/fabushi',
    });
    assert.equal(selected.agentEntry, path.join(bin, 'fabushi-device-agent.js'));
    fs.rmSync(selected.agentEntry);
    assert.equal(remoteDeviceRuntime({
      app: { isPackaged: false, getPath: () => path.join(root, 'data') },
      env: { FABUSHI_COMPUTER_MCP_ENTRY: path.join(bin, 'fabushi-computer-mcp.js') },
      platform: 'linux',
      resourcesPath: root,
      fs,
      execPath: '/opt/fabushi/fabushi',
    }), null);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
