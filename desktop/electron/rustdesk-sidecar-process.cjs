'use strict';

const { spawn } = require('node:child_process');
const { EventEmitter } = require('node:events');
const fs = require('node:fs');
const path = require('node:path');
const readline = require('node:readline');

const PROTOCOL = 'fabushi.rustdesk-sidecar.v1';
const MAX_EVENT_BYTES = 2 * 1024 * 1024;
const MAX_COMMAND_BYTES = 1024 * 1024;
const SESSION_ID = /^[A-Za-z0-9._:-]{1,160}$/;

function binaryPath({ app, env = process.env, platform = process.platform, resourcesPath = process.resourcesPath, fsImpl = fs } = {}) {
  const name = platform === 'win32' ? 'fabushi-sidecar.exe' : 'fabushi-sidecar';
  if (app?.isPackaged) {
    const candidate = path.join(resourcesPath, 'rustdesk-sidecar', name);
    return fsImpl.existsSync(candidate) ? candidate : null;
  }
  const override = String(env.FABUSHI_RUSTDESK_SIDECAR_BIN || '').trim();
  if (!override) return null;
  const resolved = path.resolve(override);
  return fsImpl.existsSync(resolved) ? resolved : null;
}

function cleanEnvironment(env = process.env) {
  const keep = ['PATH', 'SystemRoot', 'WINDIR', 'HOME', 'USERPROFILE', 'TMPDIR', 'TMP', 'TEMP', 'LANG', 'LC_ALL', 'DISPLAY', 'WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR', 'DBUS_SESSION_BUS_ADDRESS'];
  return Object.fromEntries(keep.flatMap((key) => env[key] == null ? [] : [[key, String(env[key])]]));
}

class RustDeskSidecarProcess extends EventEmitter {
  constructor(options = {}) {
    super();
    this.app = options.app;
    this.spawn = options.spawn || spawn;
    this.readline = options.readline || readline;
    this.fs = options.fs || fs;
    this.env = options.env || process.env;
    this.platform = options.platform || process.platform;
    this.resourcesPath = options.resourcesPath || process.resourcesPath;
    this.child = null;
    this.lines = null;
    this.sessions = new Map();
    this.ready = false;
    this.closed = false;
  }

  executablePath() {
    return binaryPath({ app: this.app, env: this.env, platform: this.platform, resourcesPath: this.resourcesPath, fsImpl: this.fs });
  }

  start() {
    if (this.closed) throw new Error('RustDesk sidecar manager is closed.');
    if (this.child) return this.child;
    const executable = this.executablePath();
    if (!executable) throw new Error('RustDesk sidecar binary is unavailable.');
    const child = this.spawn(executable, [], {
      stdio: ['pipe', 'pipe', 'pipe'],
      windowsHide: true,
      env: cleanEnvironment(this.env),
    });
    this.child = child;
    this.ready = false;
    child.once('exit', (code, signal) => this.handleExit(code, signal));
    child.once('error', (error) => this.handleExit(null, null, error));
    child.stderr?.on('data', (chunk) => {
      const message = String(chunk).slice(0, 4096).replace(/[\r\n]+/g, ' ');
      if (message) this.emit('diagnostic', { message });
    });
    this.lines = this.readline.createInterface({ input: child.stdout, crlfDelay: Infinity });
    this.lines.on('line', (line) => this.handleLine(line));
    this.send({ type: 'hello', protocol: PROTOCOL });
    return child;
  }

  handleLine(line) {
    if (Buffer.byteLength(line, 'utf8') > MAX_EVENT_BYTES) {
      this.terminate('oversized-event');
      return;
    }
    let event;
    try { event = JSON.parse(line); } catch { this.terminate('invalid-json'); return; }
    if (!event || typeof event !== 'object' || Array.isArray(event) || event.protocol !== PROTOCOL || typeof event.type !== 'string') {
      this.terminate('invalid-protocol-event');
      return;
    }
    if (event.type === 'hello') this.ready = true;
    const sessionId = typeof event.sessionId === 'string' ? event.sessionId : null;
    if (sessionId && !this.sessions.has(sessionId) && !['closed', 'error'].includes(event.type)) {
      this.terminate('unknown-session-event');
      return;
    }
    if (event.type === 'closed' && sessionId) this.sessions.delete(sessionId);
    this.emit('event', Object.freeze(event));
  }

  handleExit(code, signal, error) {
    const active = [...this.sessions.keys()];
    this.lines?.close?.();
    this.lines = null;
    this.child = null;
    this.ready = false;
    this.sessions.clear();
    this.emit('exit', { code, signal, error: error instanceof Error ? error.message : null, sessions: active });
  }

  send(command) {
    const child = this.child || this.start();
    const payload = JSON.stringify(command);
    if (Buffer.byteLength(payload, 'utf8') > MAX_COMMAND_BYTES) throw new Error('RustDesk sidecar command exceeds the safety limit.');
    if (!child.stdin?.writable) throw new Error('RustDesk sidecar stdin is unavailable.');
    child.stdin.write(`${payload}\n`);
  }

  open(params) {
    const sessionId = String(params?.sessionId || '');
    const peerId = String(params?.peerId || '');
    const password = String(params?.password || '');
    if (!SESSION_ID.test(sessionId) || !SESSION_ID.test(peerId)) throw new Error('RustDesk session or peer id is invalid.');
    if (!password || password.length > 256 || /[\r\n]/.test(password)) throw new Error('RustDesk ephemeral credential is invalid.');
    if (this.sessions.has(sessionId)) throw new Error('RustDesk session is already active.');
    const grant = params?.grant;
    if (!grant || grant.display !== true || ['input', 'clipboard', 'fileTransfer', 'audio'].some((key) => typeof grant[key] !== 'boolean')) {
      throw new Error('RustDesk session grant is invalid.');
    }
    const frozenGrant = Object.freeze({ display: true, input: grant.input, clipboard: grant.clipboard, fileTransfer: grant.fileTransfer, audio: grant.audio });
    this.sessions.set(sessionId, frozenGrant);
    try {
      this.send({ type: 'open', sessionId, peerId, password, forceRelay: params?.forceRelay === true, grant: frozenGrant });
    } catch (error) {
      this.sessions.delete(sessionId);
      throw error;
    }
    return { sessionId, grant: frozenGrant };
  }

  command(sessionId, command) {
    const id = String(sessionId || '');
    if (!SESSION_ID.test(id) || !this.sessions.has(id)) throw new Error('RustDesk session is not active.');
    const type = String(command?.type || '');
    const grant = this.sessions.get(id);
    if (['mouse', 'key', 'text'].includes(type) && grant.input !== true) throw new Error('RustDesk input is not granted.');
    if (type === 'clipboard' && grant.clipboard !== true) throw new Error('RustDesk clipboard is not granted.');
    if (type === 'file' && grant.fileTransfer !== true) throw new Error('RustDesk file transfer is not granted.');
    if (type === 'audio' && grant.audio !== true) throw new Error('RustDesk audio is not granted.');
    if (!['mouse', 'key', 'text', 'clipboard', 'file', 'audio', 'reconnect'].includes(type)) throw new Error('Unsupported RustDesk command.');
    this.send({ ...command, type, sessionId: id });
    return true;
  }

  closeSession(sessionId) {
    const id = String(sessionId || '');
    if (!this.sessions.has(id)) return false;
    this.send({ type: 'close', sessionId: id });
    this.sessions.delete(id);
    return true;
  }

  terminate(reason = 'shutdown') {
    const child = this.child;
    this.sessions.clear();
    this.ready = false;
    this.lines?.close?.();
    this.lines = null;
    this.child = null;
    if (child && !child.killed) child.kill();
    this.emit('terminated', { reason });
  }

  close() {
    this.closed = true;
    this.terminate('manager-close');
  }
}

module.exports = { PROTOCOL, RustDeskSidecarProcess, binaryPath, cleanEnvironment };
