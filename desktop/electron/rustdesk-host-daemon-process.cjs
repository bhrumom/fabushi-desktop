'use strict';

const { EventEmitter } = require('node:events');
const { spawn } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const { cleanEnvironment } = require('./rustdesk-sidecar-process.cjs');

function hostDaemonPath({ app, env = process.env, resourcesPath = process.resourcesPath, platform = process.platform, fsImpl = fs } = {}) {
  const name = platform === 'win32' ? 'fabushi-host-daemon.exe' : 'fabushi-host-daemon';
  if (app?.isPackaged) {
    const candidate = path.join(resourcesPath, 'rustdesk-sidecar', name);
    return fsImpl.existsSync(candidate) ? candidate : null;
  }
  const override = String(env.FABUSHI_RUSTDESK_HOST_DAEMON_BIN || '').trim();
  if (!override) return null;
  const candidate = path.resolve(override);
  return fsImpl.existsSync(candidate) ? candidate : null;
}

class RustDeskHostDaemonProcess extends EventEmitter {
  constructor(options = {}) {
    super();
    this.app = options.app;
    this.env = options.env || process.env;
    this.resourcesPath = options.resourcesPath || process.resourcesPath;
    this.platform = options.platform || process.platform;
    this.fs = options.fs || fs;
    this.spawn = options.spawn || spawn;
    this.child = null;
    this.startedAtMs = 0;
  }

  executablePath() {
    return hostDaemonPath({
      app: this.app,
      env: this.env,
      resourcesPath: this.resourcesPath,
      platform: this.platform,
      fsImpl: this.fs,
    });
  }

  start() {
    if (this.child) return { available: true, running: true, startedAtMs: this.startedAtMs };
    const executable = this.executablePath();
    if (!executable) return { available: false, running: false, startedAtMs: 0 };
    const child = this.spawn(executable, [], {
      stdio: ['ignore', 'ignore', 'pipe'],
      windowsHide: true,
      env: cleanEnvironment(this.env),
    });
    this.child = child;
    this.startedAtMs = Date.now();
    child.stderr?.on?.('data', (chunk) => {
      const message = String(chunk || '').slice(0, 4096);
      if (message) this.emit('diagnostic', { message });
    });
    child.once('error', (error) => {
      if (this.child === child) this.child = null;
      this.emit('error', error);
    });
    child.once('exit', (code, signal) => {
      if (this.child === child) this.child = null;
      this.emit('exit', { code, signal });
    });
    return { available: true, running: true, startedAtMs: this.startedAtMs };
  }

  status() {
    return {
      available: Boolean(this.executablePath()),
      running: Boolean(this.child),
      startedAtMs: this.child ? this.startedAtMs : 0,
    };
  }

  close() {
    const child = this.child;
    this.child = null;
    if (child && !child.killed) child.kill('SIGTERM');
  }
}

module.exports = { RustDeskHostDaemonProcess, hostDaemonPath };
