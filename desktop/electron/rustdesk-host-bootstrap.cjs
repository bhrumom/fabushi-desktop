'use strict';

const { spawn } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const readline = require('node:readline');
const { cleanEnvironment } = require('./rustdesk-sidecar-process.cjs');

const PROTOCOL = 'fabushi.rustdesk-host-bootstrap.v1';
const TIMEOUT_MS = 5000;
const PEER_ID = /^[A-Za-z0-9._:-]{1,160}$/;

function bootstrapBinaryPath({ app, env = process.env, platform = process.platform, resourcesPath = process.resourcesPath, fsImpl = fs } = {}) {
  const name = platform === 'win32' ? 'fabushi-host-bootstrap.exe' : 'fabushi-host-bootstrap';
  if (app?.isPackaged) {
    const candidate = path.join(resourcesPath, 'rustdesk-sidecar', name);
    return fsImpl.existsSync(candidate) ? candidate : null;
  }
  const override = String(env.FABUSHI_RUSTDESK_HOST_BOOTSTRAP_BIN || '').trim();
  if (!override) return null;
  const resolved = path.resolve(override);
  return fsImpl.existsSync(resolved) ? resolved : null;
}

function requestHostBootstrap(command, options = {}) {
  const executable = bootstrapBinaryPath({
    app: options.app,
    env: options.env || process.env,
    platform: options.platform || process.platform,
    resourcesPath: options.resourcesPath || process.resourcesPath,
    fsImpl: options.fs || fs,
  });
  if (!executable) return Promise.reject(new Error('RustDesk host bootstrap binary is unavailable.'));
  const spawnImpl = options.spawn || spawn;
  const child = spawnImpl(executable, [], {
    stdio: ['pipe', 'pipe', 'pipe'],
    windowsHide: true,
    env: cleanEnvironment(options.env || process.env),
  });
  return new Promise((resolve, reject) => {
    let settled = false;
    let lines;
    const finish = (error, value) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      lines?.close?.();
      if (!child.killed) child.kill();
      if (error) reject(error);
      else resolve(value);
    };
    const timer = setTimeout(() => finish(new Error('RustDesk host bootstrap timed out.')), options.timeoutMs || TIMEOUT_MS);
    child.once('error', (error) => finish(error));
    child.once('exit', (code) => {
      if (!settled && code !== 0) finish(new Error(`RustDesk host bootstrap exited with code ${code}.`));
    });
    lines = (options.readline || readline).createInterface({ input: child.stdout, crlfDelay: Infinity });
    lines.on('line', (line) => {
      if (Buffer.byteLength(line, 'utf8') > 64 * 1024) return finish(new Error('RustDesk host bootstrap response is too large.'));
      let event;
      try { event = JSON.parse(line); } catch { return finish(new Error('RustDesk host bootstrap returned invalid JSON.')); }
      if (!event || event.protocol !== PROTOCOL || typeof event.type !== 'string') return finish(new Error('RustDesk host bootstrap protocol mismatch.'));
      if (event.type === 'error') return finish(new Error(`RustDesk host bootstrap failed: ${String(event.code || 'unknown')}`));
      if (event.type !== 'hostInfo') return;
      if (!PEER_ID.test(String(event.peerId || ''))) return finish(new Error('RustDesk host peer id is invalid.'));
      if (command === 'rotateTemporaryPassword') {
        const password = String(event.temporaryPassword || '');
        if (password.length < 6 || password.length > 32 || /\s/.test(password)) return finish(new Error('RustDesk temporary password is invalid.'));
        return finish(null, { peerId: event.peerId, temporaryPassword: password });
      }
      return finish(null, { peerId: event.peerId });
    });
    child.stdin.write(`${JSON.stringify({ type: command })}\n`);
    child.stdin.end();
  });
}

function readHostInfo(options) {
  return requestHostBootstrap('hostInfo', options);
}

function rotateTemporaryPassword(options) {
  return requestHostBootstrap('rotateTemporaryPassword', options);
}

module.exports = { PROTOCOL, bootstrapBinaryPath, readHostInfo, rotateTemporaryPassword };
