'use strict';

const assert = require('node:assert/strict');
const { EventEmitter } = require('node:events');
const test = require('node:test');
const { RustDeskHostDaemonProcess, hostDaemonPath } = require('./rustdesk-host-daemon-process.cjs');

function fakeChild() {
  const child = new EventEmitter();
  child.stderr = new EventEmitter();
  child.killed = false;
  child.kill = () => { child.killed = true; };
  return child;
}

test('packaged host daemon ignores inherited executable override', () => {
  assert.equal(hostDaemonPath({
    app: { isPackaged: true },
    env: { FABUSHI_RUSTDESK_HOST_DAEMON_BIN: '/evil' },
    resourcesPath: '/signed',
    platform: 'win32',
    fsImpl: { existsSync: () => true },
  }), '/signed/rustdesk-sidecar/fabushi-host-daemon.exe');
});

test('host daemon starts with Fabushi credentials stripped', () => {
  let spawnOptions;
  const child = fakeChild();
  const manager = new RustDeskHostDaemonProcess({
    app: { isPackaged: false },
    env: {
      FABUSHI_RUSTDESK_HOST_DAEMON_BIN: '/tmp/host',
      PATH: '/bin',
      HOME: '/home/user',
      MAHAYANA_AUTH_TOKEN: 'secret',
      FABUSHI_ACCOUNT_TOKEN: 'secret2',
    },
    fs: { existsSync: () => true },
    spawn(_command, _args, options) { spawnOptions = options; return child; },
  });
  const status = manager.start();
  assert.equal(status.running, true);
  assert.equal(spawnOptions.env.PATH, '/bin');
  assert.equal(spawnOptions.env.HOME, '/home/user');
  assert.equal(spawnOptions.env.MAHAYANA_AUTH_TOKEN, undefined);
  assert.equal(spawnOptions.env.FABUSHI_ACCOUNT_TOKEN, undefined);
  manager.close();
  assert.equal(child.killed, true);
});

test('host daemon close revokes process lifetime', () => {
  const child = fakeChild();
  const manager = new RustDeskHostDaemonProcess({
    app: { isPackaged: false }, env: { FABUSHI_RUSTDESK_HOST_DAEMON_BIN: '/tmp/host' },
    fs: { existsSync: () => true }, spawn: () => child,
  });
  manager.start();
  assert.equal(manager.status().running, true);
  manager.close();
  assert.equal(manager.status().running, false);
});
