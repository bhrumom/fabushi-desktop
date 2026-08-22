const { app } = require('electron');
const { spawn } = require('node:child_process');
const path = require('node:path');
const readline = require('node:readline');

const PRODUCTION_PRODUCT_API_BASE_URL = 'https://api.ombhrum.com';
const DEVELOPMENT_PRODUCT_API_BASE_URL = 'https://mahayana-platform.bhrumom.workers.dev';

function productApiBaseUrl(appImpl = app, env = process.env) {
  const configured = env.MAHAYANA_API_BASE_URL?.trim();
  if (configured) {
    const parsed = new URL(configured);
    if (parsed.protocol !== 'https:' || parsed.username || parsed.password || parsed.search || parsed.hash) {
      throw new Error('MAHAYANA_API_BASE_URL must be a clean HTTPS origin/base URL');
    }
    return parsed.toString().replace(/\/$/, '');
  }
  return appImpl.isPackaged ? PRODUCTION_PRODUCT_API_BASE_URL : DEVELOPMENT_PRODUCT_API_BASE_URL;
}

class MahayanaHostProcess {
  constructor(options = {}) {
    this.app = options.app ?? app;
    this.spawn = options.spawn ?? spawn;
    this.readline = options.readline ?? readline;
    this.env = options.env ?? process.env;
    this.platform = options.platform ?? process.platform;
    this.resourcesPath = options.resourcesPath ?? process.resourcesPath;
    this.now = options.now ?? Date.now;

    this.child = null;
    this.currentGeneration = 0;
    this.nextId = 1;
    this.pending = new Map();
    this.closed = false;
    this.state = 'stopped';
    this.startedAt = null;
    this.lastExit = null;
    this.unexpectedExitCount = 0;
  }

  executablePath() {
    if (!this.app.isPackaged && this.env.MAHAYANA_APP_HOST_BIN) {
      return this.env.MAHAYANA_APP_HOST_BIN;
    }
    const name = this.platform === 'win32' ? 'mahayana-app-host.exe' : 'mahayana-app-host';
    if (this.app.isPackaged) return path.join(this.resourcesPath, 'bin', name);
    return path.resolve(__dirname, '..', '..', 'third_party', 'mahayana', 'mahayana-rs', 'target', 'release', name);
  }

  health() {
    return Object.freeze({
      state: this.state,
      closed: this.closed,
      generation: this.currentGeneration,
      pid: this.child?.pid ?? null,
      pending: this.pending.size,
      startedAt: this.startedAt,
      lastExit: this.lastExit ? { ...this.lastExit } : null,
      unexpectedExitCount: this.unexpectedExitCount,
    });
  }

  start() {
    if (this.closed) throw new Error('Mahayana host is closed.');
    if (this.child) return this.child;

    const generation = this.currentGeneration + 1;
    const child = this.spawn(this.executablePath(), [], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: {
        ...this.env,
        MAHAYANA_API_BASE_URL: productApiBaseUrl(this.app, this.env),
        FABUSHI_APP_DATA: this.app.getPath('userData'),
      },
      windowsHide: true,
    });

    this.currentGeneration = generation;
    this.child = child;
    this.state = 'running';
    this.startedAt = this.now();

    const lines = this.readline.createInterface({ input: child.stdout });
    lines.on('line', (line) => {
      let message;
      try {
        message = JSON.parse(line);
      } catch (error) {
        this.rejectGeneration(generation, new Error(`Invalid Mahayana host response: ${error}`));
        return;
      }
      const key = String(message.id ?? '');
      const pending = this.pending.get(key);
      if (!pending || pending.generation !== generation) return;
      this.pending.delete(key);
      if (message.ok) pending.resolve(message.result);
      else pending.reject(new Error(message.error || 'Mahayana host request failed'));
    });

    child.stderr.on('data', (chunk) => console.error(`[mahayana-app-host] ${String(chunk).trimEnd()}`));
    child.on('error', (error) => {
      this.handleTermination(child, generation, error, { error: error.message });
    });
    child.on('exit', (code, signal) => {
      lines.close();
      this.handleTermination(
        child,
        generation,
        new Error(`Mahayana host exited (${code ?? 'null'}, ${signal ?? 'none'})`),
        { code: code ?? null, signal: signal ?? null },
      );
    });

    return child;
  }

  handleTermination(child, generation, error, metadata) {
    const isCurrent = this.child === child && this.currentGeneration === generation;
    if (isCurrent) {
      this.child = null;
      this.startedAt = null;
      if (!this.closed) {
        this.state = 'stopped';
        this.unexpectedExitCount += 1;
        this.lastExit = Object.freeze({ ...metadata, at: this.now() });
      }
    }
    this.rejectGeneration(generation, error);
  }

  request(method, params = {}, timeoutMs = 120000) {
    let child;
    try {
      child = this.start();
    } catch (error) {
      return Promise.reject(error);
    }
    const generation = this.currentGeneration;
    const id = this.nextId++;
    const key = String(id);
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        const pending = this.pending.get(key);
        if (!pending || pending.generation !== generation) return;
        this.pending.delete(key);
        reject(new Error(`Mahayana host request timed out: ${method}`));
      }, timeoutMs);
      timer.unref?.();
      this.pending.set(key, {
        generation,
        resolve: (value) => { clearTimeout(timer); resolve(value); },
        reject: (error) => { clearTimeout(timer); reject(error); },
      });
      const payload = JSON.stringify({ id, method, params });
      child.stdin.write(`${payload}\n`, (error) => {
        if (!error) return;
        const pending = this.pending.get(key);
        if (!pending || pending.generation !== generation) return;
        this.pending.delete(key);
        pending.reject(error);
      });
    });
  }

  rejectGeneration(generation, error) {
    for (const [key, pending] of this.pending.entries()) {
      if (pending.generation !== generation) continue;
      this.pending.delete(key);
      pending.reject(error);
    }
  }

  restart(reason = 'manual restart') {
    if (this.closed) throw new Error('Mahayana host is closed.');
    const child = this.child;
    const generation = this.currentGeneration;
    if (child) {
      this.child = null;
      this.startedAt = null;
      this.state = 'stopped';
      this.rejectGeneration(generation, new Error(`Mahayana host restarted: ${reason}`));
      child.kill();
    }
    return this.start();
  }

  close() {
    if (this.closed) return;
    this.closed = true;
    this.state = 'closed';
    const child = this.child;
    const generation = this.currentGeneration;
    this.child = null;
    this.startedAt = null;
    this.rejectGeneration(generation, new Error('Mahayana host closed.'));
    child?.kill();
  }
}

module.exports = {
  DEVELOPMENT_PRODUCT_API_BASE_URL,
  MahayanaHostProcess,
  PRODUCTION_PRODUCT_API_BASE_URL,
  productApiBaseUrl,
};
