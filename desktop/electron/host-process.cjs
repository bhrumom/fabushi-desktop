const { app } = require('electron');
const { spawn } = require('node:child_process');
const { EventEmitter } = require('node:events');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const readline = require('node:readline');
const { createTestAuthSession } = require('./test-auth-session.cjs');
const { createTestPlatformAccount } = require('./test-platform-account.cjs');

const PRODUCTION_PRODUCT_API_BASE_URL = 'https://api.ombhrum.com';
const DEVELOPMENT_PRODUCT_API_BASE_URL = 'https://mahayana-platform.bhrumom.workers.dev';
const INFERENCE_PROVIDERS = new Set(['fabushi', 'codex', 'claude-code', 'openrouter']);
const SANDBOX_RUNTIMES = new Set(['host', 'local-docker']);
const DEFAULT_DOCKER_IMAGE = 'mcr.microsoft.com/devcontainers/base:ubuntu24.04@sha256:c5cc2b45afe06a1df3aba17e58ba0dc4a02b999493198dab37dd0ccd4e2b0705';

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

function persistedInferenceProvider(appImpl = app, fsImpl = fs) {
  return persistedRouterSettings(appImpl, fsImpl).inferenceProvider;
}

function persistedRouterSettings(appImpl = app, fsImpl = fs) {
  try {
    const settingsPath = path.join(appImpl.getPath('userData'), 'feature-host', 'settings.json');
    const parsed = JSON.parse(fsImpl.readFileSync(settingsPath, 'utf8'));
    const provider = String(parsed?.inferenceProvider ?? 'fabushi');
    const sandboxRuntime = String(parsed?.sandboxRuntime ?? 'host');
    return {
      inferenceProvider: INFERENCE_PROVIDERS.has(provider) ? provider : 'fabushi',
      sandboxRuntime: SANDBOX_RUNTIMES.has(sandboxRuntime) ? sandboxRuntime : 'host',
    };
  } catch {
    return { inferenceProvider: 'fabushi', sandboxRuntime: 'host' };
  }
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
    this.fs = options.fs ?? fs;
    this.providerEnvironment = options.providerEnvironment ?? (() => ({}));
    this.testAuthSession = this.env.FABUSHI_FEATURE_HOST_MODE === 'test'
      ? createTestAuthSession({ app: this.app, fs: this.fs, now: this.now })
      : null;
    this.testPlatformAccount = this.env.FABUSHI_FEATURE_HOST_MODE === 'test'
      ? createTestPlatformAccount({ app: this.app, fs: this.fs, now: this.now })
      : null;

    this.child = null;
    this.currentGeneration = 0;
    this.nextId = 1;
    this.pending = new Map();
    this.closed = false;
    this.state = 'stopped';
    this.startedAt = null;
    this.lastExit = null;
    this.unexpectedExitCount = 0;
    this.lifecycleSequence = 0;
    this.lastLifecycleEvent = null;
    this.activeInferenceProvider = 'fabushi';
    this.activeSandboxRuntime = 'host';
    this.events = new EventEmitter();
    this.events.setMaxListeners(32);
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
      lifecycleSequence: this.lifecycleSequence,
      lastLifecycleEvent: this.lastLifecycleEvent ? { ...this.lastLifecycleEvent } : null,
      inferenceProvider: this.activeInferenceProvider,
      sandboxRuntime: this.activeSandboxRuntime,
    });
  }

  onLifecycle(listener) {
    if (typeof listener !== 'function') throw new TypeError('Mahayana host lifecycle listener must be a function.');
    this.events.on('lifecycle', listener);
    return () => this.events.off('lifecycle', listener);
  }

  emitLifecycle(type, detail = {}) {
    const event = Object.freeze({
      type,
      sequence: ++this.lifecycleSequence,
      at: this.now(),
      state: this.state,
      generation: this.currentGeneration,
      pid: this.child?.pid ?? null,
      pending: this.pending.size,
      unexpectedExitCount: this.unexpectedExitCount,
      ...detail,
    });
    this.lastLifecycleEvent = event;
    this.events.emit('lifecycle', event);
    return event;
  }

  start() {
    if (this.closed) throw new Error('Mahayana host is closed.');
    if (this.child) return this.child;

    const generation = this.currentGeneration + 1;
    const { inferenceProvider, sandboxRuntime } = persistedRouterSettings(this.app, this.fs);
    const providerEnvironment = this.providerEnvironment(inferenceProvider) ?? {};
    this.activeInferenceProvider = inferenceProvider;
    this.activeSandboxRuntime = sandboxRuntime;
    this.currentGeneration = generation;
    this.state = 'starting';
    this.emitLifecycle('starting');

    let child;
    try {
      child = this.spawn(this.executablePath(), [], {
        stdio: ['pipe', 'pipe', 'pipe'],
        env: {
          ...this.env,
          ANTHROPIC_API_KEY: '',
          OPENROUTER_API_KEY: '',
          MAHAYANA_MODEL_BEARER_TOKEN: '',
          MAHAYANA_API_BASE_URL: productApiBaseUrl(this.app, this.env),
          MAHAYANA_AUTH_STORAGE_NAMESPACE: this.env.MAHAYANA_AUTH_STORAGE_NAMESPACE || 'fabushi-desktop-v2',
          FABUSHI_APP_DATA: this.app.getPath('userData'),
          MAHAYANA_AGENT_ENGINE: inferenceProvider === 'codex' ? 'codex' : '',
          MAHAYANA_USE_CODEX_ACCOUNT: inferenceProvider === 'codex' ? '1' : '0',
          MAHAYANA_INFERENCE_PROVIDER: inferenceProvider,
          MAHAYANA_SANDBOX_RUNTIME: sandboxRuntime,
          MAHAYANA_DOCKER_BIN: this.env.MAHAYANA_DOCKER_BIN || this.env.DOCKER_PATH || 'docker',
          MAHAYANA_DOCKER_IMAGE: this.env.MAHAYANA_DOCKER_IMAGE || DEFAULT_DOCKER_IMAGE,
          MAHAYANA_CODEX_HOME: inferenceProvider === 'codex'
            ? (this.env.CODEX_HOME || path.join(os.homedir(), '.codex'))
            : '',
          ...providerEnvironment,
        },
        windowsHide: true,
      });
    } catch (error) {
      this.state = 'stopped';
      this.startedAt = null;
      this.unexpectedExitCount += 1;
      this.lastExit = Object.freeze({ error: error instanceof Error ? error.message : String(error), at: this.now() });
      this.emitLifecycle('spawn-failed', { error: this.lastExit.error });
      throw error;
    }

    this.child = child;
    this.state = 'running';
    this.startedAt = this.now();
    this.emitLifecycle('running');

    const lines = this.readline.createInterface({ input: child.stdout });
    lines.on('line', (line) => {
      let message;
      try {
        message = JSON.parse(line);
      } catch (error) {
        this.rejectGeneration(generation, new Error(`Invalid Mahayana host response: ${error}`));
        this.emitLifecycle('protocol-error', { error: error instanceof Error ? error.message : String(error) });
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
        this.emitLifecycle('stopped', { ...metadata, recoverable: true });
      }
    }
    this.rejectGeneration(generation, error);
  }

  request(method, params = {}, timeoutMs = 120000) {
    if (this.testAuthSession) {
      const result = this.testAuthSession.request(method, params);
      if (result !== null) return Promise.resolve(result);
    }
    if (method === 'platform.request' && this.testPlatformAccount) {
      const result = this.testPlatformAccount.request(params);
      if (result) return Promise.resolve(result);
    }
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
        this.emitLifecycle('request-timeout', { method: String(method), requestId: id });
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
    this.state = 'restarting';
    this.emitLifecycle('restarting', { reason: String(reason) });
    if (child) {
      this.child = null;
      this.startedAt = null;
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
    this.emitLifecycle('closed');
    child?.kill();
    this.events.removeAllListeners();
  }
}

module.exports = {
  DEVELOPMENT_PRODUCT_API_BASE_URL,
  DEFAULT_DOCKER_IMAGE,
  MahayanaHostProcess,
  PRODUCTION_PRODUCT_API_BASE_URL,
  productApiBaseUrl,
  persistedInferenceProvider,
  persistedRouterSettings,
};