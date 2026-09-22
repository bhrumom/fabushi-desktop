'use strict';

const assert = require('node:assert/strict');
const { EventEmitter } = require('node:events');
const { PassThrough } = require('node:stream');
const test = require('node:test');
const { PROTOCOL, bootstrapBinaryPath, readHostInfo, rotateTemporaryPassword } = require('./rustdesk-host-bootstrap.cjs');

function fakeProcess(response) {
  const child = new EventEmitter();
  child.stdin = new PassThrough();
  child.stdout = new PassThrough();
  child.stderr = new PassThrough();
  child.killed = false;
  child.kill = () => { child.killed = true; };
  child.stdin.on('data', () => queueMicrotask(() => child.stdout.write(`${JSON.stringify(response)}\n`)));
  return child;
}

function options(response) {
  return {
    app: { isPackaged: false },
    fs: { existsSync: () => true },
    env: { PATH: '/bin', FABUSHI_RUSTDESK_HOST_BOOTSTRAP_BIN: '/tmp/fabushi-host-bootstrap', FABUSHI_ACCOUNT_TOKEN: 'never-forward' },
    spawn(_command, _args, spawnOptions) {
      assert.equal(spawnOptions.env.FABUSHI_ACCOUNT_TOKEN, undefined);
      return fakeProcess(response);
    },
    timeoutMs: 1000,
  };
}

test('packaged host bootstrap ignores environment overrides', () => {
  assert.equal(bootstrapBinaryPath({ app: { isPackaged: true }, env: { FABUSHI_RUSTDESK_HOST_BOOTSTRAP_BIN: '/evil' }, platform: 'linux', resourcesPath: '/signed', fsImpl: { existsSync: () => true } }), '/signed/rustdesk-sidecar/fabushi-host-bootstrap');
});

test('host info returns only a validated public peer id', async () => {
  const info = await readHostInfo(options({ protocol: PROTOCOL, type: 'hostInfo', peerId: '123456789' }));
  assert.deepEqual(info, { peerId: '123456789' });
});

test('temporary password is exposed only by explicit rotation command', async () => {
  const info = await rotateTemporaryPassword(options({ protocol: PROTOCOL, type: 'hostInfo', peerId: '123456789', temporaryPassword: 'aB3dE7' }));
  assert.deepEqual(info, { peerId: '123456789', temporaryPassword: 'aB3dE7' });
  await assert.rejects(() => rotateTemporaryPassword(options({ protocol: PROTOCOL, type: 'hostInfo', peerId: '123456789' })), /temporary password is invalid/);
});

test('malformed peer identifiers fail closed', async () => {
  await assert.rejects(() => readHostInfo(options({ protocol: PROTOCOL, type: 'hostInfo', peerId: '../escape' })), /peer id is invalid/);
});
