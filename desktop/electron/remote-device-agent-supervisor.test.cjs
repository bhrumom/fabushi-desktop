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
