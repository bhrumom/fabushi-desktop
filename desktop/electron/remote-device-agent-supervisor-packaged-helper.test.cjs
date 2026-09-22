'use strict';

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
    return '/tmp/fabushi-packaged-helper-test';
  },
};
const originalLoad = Module._load;
Module._load = function load(request, parent, isMain) {
  if (request === 'electron') return { app: defaultApp };
  return originalLoad.call(this, request, parent, isMain);
};
const {
  RemoteDeviceAgentSupervisor,
  remoteDeviceRuntime,
} = require('./remote-device-agent-supervisor.cjs');
Module._load = originalLoad;

function writePackagedRuntime(root) {
  const resourcesPath = path.join(root, 'resources');
  const bundleHome = path.join(resourcesPath, 'computer-control');
  const sourceHash = 'a'.repeat(64);
  const runtimeId = `v1-${sourceHash.slice(0, 20)}`;
  const runtimeRoot = path.join(bundleHome, 'runtime', runtimeId);
  for (const relative of [
    'bin/fabushi-computer-mcp.js',
    'bin/fabushi-device-agent.js',
    'lib/fabushi-computer-policy.js',
    'node_modules/@modelcontextprotocol/sdk/package.json',
    'node_modules/ws/package.json',
    'node_modules/zod/package.json',
  ]) {
    const target = path.join(runtimeRoot, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, relative.endsWith('.json') ? '{}\n' : '#!/usr/bin/env node\n');
  }
  fs.writeFileSync(path.join(runtimeRoot, 'runtime-manifest.json'), `${JSON.stringify({
    layoutVersion: 1,
    runtimeId,
    sourceHash,
  })}\n`);
  fs.mkdirSync(bundleHome, { recursive: true });
  fs.writeFileSync(path.join(bundleHome, 'active-runtime.json'), `${JSON.stringify({ runtimeId })}\n`);

  const helperApp = path.join(bundleHome, 'Applications', 'Fabushi Computer Control.app');
  const nativeHelper = path.join(helperApp, 'Contents', 'MacOS', 'FabushiComputerControl');
  fs.mkdirSync(path.dirname(nativeHelper), { recursive: true });
  fs.writeFileSync(nativeHelper, '#!/bin/sh\n');
  return { resourcesPath, helperApp, nativeHelper };
}

class FakeChild extends EventEmitter {
  constructor() {
    super();
    this.stdout = new PassThrough();
    this.stderr = new PassThrough();
    this.killed = false;
  }
  kill() { this.killed = true; return true; }
}

test('packaged App-owned device forwards only the signed bundle native helper to its direct MCP', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fabushi-packaged-helper-'));
  try {
    const data = path.join(root, 'data');
    const fixture = writePackagedRuntime(root);
    const app = { isPackaged: true, getPath: () => data };
    const execPath = '/Applications/Fabushi.app/Contents/MacOS/fabushi';
    const inherited = {
      GITHUB_ACTIONS: 'true',
      GITHUB_RUN_ID: '1',
      GITHUB_RUN_ATTEMPT: '1',
      GITHUB_JOB: 'macos-interactive',
      CHATGPT_COMPUTER_HOME: '/tmp/untrusted-home',
      CHATGPT_COMPUTER_NATIVE_HELPER: '/tmp/untrusted-helper',
      CHATGPT_COMPUTER_MAC_APP_DIR: '/tmp/untrusted-app',
    };

    const runtime = remoteDeviceRuntime({
      app,
      env: inherited,
      platform: 'darwin',
      resourcesPath: fixture.resourcesPath,
      fs,
      execPath,
    });
    assert.ok(runtime);
    assert.equal(runtime.childEnvironment.CHATGPT_COMPUTER_HOME, path.join(data, 'computer-control'));
    assert.equal(runtime.childEnvironment.CHATGPT_COMPUTER_NATIVE_HELPER, fixture.nativeHelper);
    assert.equal(runtime.childEnvironment.CHATGPT_COMPUTER_MAC_APP_DIR, fixture.helperApp);

    let child = null;
    const supervisor = new RemoteDeviceAgentSupervisor({
      app,
      host: {
        async request() {
          return {
            accessToken: 'a'.repeat(64),
            deviceId: 'gha-1-1-macos-app',
            sessionId: 'gha-1-1-macos-app-session',
          };
        },
      },
      env: inherited,
      platform: 'darwin',
      resourcesPath: fixture.resourcesPath,
      fs,
      execPath,
      spawn(command, args, options) {
        assert.equal(command, execPath);
        assert.equal(args[0], runtime.agentEntry);
        assert.equal(options.env.CHATGPT_COMPUTER_HOME, path.join(data, 'computer-control'));
        assert.equal(options.env.CHATGPT_COMPUTER_NATIVE_HELPER, fixture.nativeHelper);
        assert.equal(options.env.CHATGPT_COMPUTER_MAC_APP_DIR, fixture.helperApp);
        assert.equal(options.env.DEVICE_LOCAL_MCP_ENTRY, runtime.mcpEntry);
        assert.equal(options.env.DEVICE_LOCAL_MCP_ELECTRON_NODE, '1');
        child = new FakeChild();
        return child;
      },
    });

    await supervisor.sync();
    assert.ok(child);
    supervisor.close();
    assert.equal(child.killed, true);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
