const { app } = require('electron');
const { spawn } = require('node:child_process');
const { EventEmitter } = require('node:events');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const readline = require('node:readline');
const { createTestPlatformAccount } = require('./test-platform-account.cjs');

const PRODUCTION_PRODUCT_API_BASE_URL = 'https://api.ombhrum.com';
const DEVELOPMENT_PRODUCT_API_BASE_URL = 'https://mahayana-platform.bhrumom.workers.dev';
const INFERENCE_PROVIDERS = new Set(['fabushi', 'codex', 'claude-code', 'openrouter']);
const SANDBOX_RUNTIMES = new Set(['host', 'local-docker']);
const DEFAULT_DOCKER_IMAGE = 'mcr.microsoft.com/devcontainers/base:ubuntu24.04@sha256:c5cc2b45afe06a1df3aba17e58ba0dc4a02b999493198dab37dd0ccd4e2b0705';
const PACKAGED_COMPUTER_RUNTIME_ID = /^v1-[a-f0-9]{20}$/;

function safeExistsSync(fsImpl, candidate) {
  const implementation = typeof fsImpl?.existsSync === 'function' ? fsImpl : fs;
  try { return implementation.existsSync(candidate); } catch { return false; }
}

function safeIsFileSync(fsImpl, candidate) {
  const implementation = typeof fsImpl?.statSync === 'function' ? fsImpl : fs;
  try { return implementation.statSync(candidate).isFile(); } catch { return false; }
}

function completeComputerRuntime(root, fsImpl = fs, expectedRuntimeId = path.basename(root)) {
  if (!PACKAGED_COMPUTER_RUNTIME_ID.test(expectedRuntimeId) || path.basename(root) !== expectedRuntimeId) return null;
  const implementation = typeof fsImpl?.readFileSync === 'function' ? fsImpl : fs;
  let manifest;
  try {
    manifest = JSON.parse(implementation.readFileSync(path.join(root, 'runtime-manifest.json'), 'utf8'));
  } catch {
    return null;
  }
  if (manifest?.layoutVersion !== 1
    || manifest?.runtimeId !== expectedRuntimeId
    || !/^[a-f0-9]{64}$/.test(String(manifest?.sourceHash || ''))
    || expectedRuntimeId !== `v1-${manifest.sourceHash.slice(0, 20)}`) return null;

  const mcpEntry = path.join(root, 'bin', 'fabushi-computer-mcp.js');
  const required = [
    mcpEntry,
    path.join(root, 'lib', 'fabushi-computer-policy.js'),
    path.join(root, 'node_modules', '@modelcontextprotocol', 'sdk', 'package.json'),
    path.join(root, 'node_modules', 'ws', 'package.json'),
    path.join(root, 'node_modules', 'zod', 'package.json'),
  ];
  return required.every((candidate) => safeIsFileSync(fsImpl, candidate)) ? { root, mcpEntry } : null;
}

function developmentComputerRuntime(root, fsImpl = fs) {
  const mcpEntry = path.join(root, 'bin', 'fabushi-computer-mcp.js');
  const required = [
    mcpEntry,
    path.join(root, 'lib', 'fabushi-computer-policy.js'),
    path.join(root, 'node_modules', '@modelcontextprotocol', 'sdk', 'package.json'),
    path.join(root, 'node_modules', 'ws', 'package.json'),
    path.join(root, 'node_modules', 'zod', 'package.json'),
  ];
  return required.every((candidate) => safeIsFileSync(fsImpl, candidate)) ? { root, mcpEntry } : null;
}

function firstCompleteComputerRuntime(runtimeBase, fsImpl = fs) {
  const implementation = typeof fsImpl?.readFileSync === 'function' ? fsImpl : fs;
  const pointerPath = path.join(path.dirname(runtimeBase), 'active-runtime.json');
  try {
    const pointer = JSON.parse(implementation.readFileSync(pointerPath, 'utf8'));
    const runtimeId = String(pointer?.runtimeId || '');
    if (!PACKAGED_COMPUTER_RUNTIME_ID.test(runtimeId)) return null;
    // A present pointer is authoritative. Do not silently select another
    // directory when it is malformed, stale, or incomplete.
    return completeComputerRuntime(path.join(runtimeBase, runtimeId), fsImpl, runtimeId);
  } catch (error) {
    if (error?.code !== 'ENOENT') return null;
    // Older packages did not carry an active pointer; validate fallback roots.
  }

  let entries;
  try {
    const directoryImplementation = typeof fsImpl?.readdirSync === 'function' ? fsImpl : fs;
    entries = directoryImplementation.readdirSync(runtimeBase, { withFileTypes: true });
  } catch {
    return null;
  }
  for (const entry of entries
    .filter((item) => item.isDirectory() && PACKAGED_COMPUTER_RUNTIME_ID.test(item.name))
    .sort((left, right) => right.name.localeCompare(left.name))) {
    const runtime = completeComputerRuntime(path.join(runtimeBase, entry.name), fsImpl, entry.name);
    if (runtime) return runtime;
  }
  return null;
}

function embeddedComputerControlEnvironment({ app: appImpl = app, env = process.env, platform = process.platform, resourcesPath = process.resourcesPath, fs: fsImpl = fs, execPath = process.execPath } = {}) {
  const explicitEntry = String(env.FABUSHI_COMPUTER_MCP_ENTRY || '').trim();
  const explicitCommand = String(env.FABUSHI_COMPUTER_MCP_COMMAND || '').trim();
  let runtime;
  let bundleHome;
  if (appImpl.isPackaged) {
    // A production package always uses its signed resources. Environment
    // overrides remain development-only and cannot replace the bundled MCP.
    bundleHome = path.join(resourcesPath, 'computer-control');
    runtime = firstCompleteComputerRuntime(path.join(bundleHome, 'runtime'), fsImpl);
  } else if (explicitEntry) {
    const entry = path.resolve(explicitEntry);
    runtime = safeIsFileSync(fsImpl, entry) ? { root: path.dirname(path.dirname(entry)), mcpEntry: entry } : null;
    bundleHome = String(env.FABUSHI_COMPUTER_BUNDLE_HOME || '').trim() || null;
  } else {
    const root = path.resolve(__dirname, '..', '..', 'chatgpt-vps-control');
    runtime = developmentComputerRuntime(root, fsImpl);
    bundleHome = root;
  }
  if (!runtime) return {};

  const computerHome = path.join(appImpl.getPath('userData'), 'computer-control');
  const policyFile = path.join(appImpl.getPath('userData'), 'feature-host', 'runtime', 'settings.json');
  const developmentCommand = explicitCommand || execPath;
  const result = {
    MAHAYANA_COMPUTER_MCP_COMMAND: appImpl.isPackaged ? execPath : developmentCommand,
    MAHAYANA_COMPUTER_MCP_ENTRY: runtime.mcpEntry,
    MAHAYANA_COMPUTER_MCP_CWD: runtime.root,
    MAHAYANA_COMPUTER_MCP_HOME: computerHome,
    MAHAYANA_COMPUTER_MCP_POLICY_FILE: policyFile,
    // The signed Electron executable is the private Node runtime in production.
    MAHAYANA_COMPUTER_MCP_ELECTRON_NODE: appImpl.isPackaged
      ? '1'
      : String(env.FABUSHI_COMPUTER_MCP_ELECTRON_NODE || (explicitCommand ? '0' : '1')),
  };
  const explicitHelper = String(env.FABUSHI_COMPUTER_NATIVE_HELPER || '').trim();
  if (!appImpl.isPackaged && explicitHelper && safeIsFileSync(fsImpl, explicitHelper)) {
    result.MAHAYANA_COMPUTER_MCP_NATIVE_HELPER = path.resolve(explicitHelper);
  } else if (bundleHome && platform === 'darwin') {
    const appDir = path.join(bundleHome, 'Applications', 'Fabushi Computer Control.app');
    const helper = path.join(appDir, 'Contents', 'MacOS', 'FabushiComputerControl');
    if (safeIsFileSync(fsImpl, helper)) {
      result.MAHAYANA_COMPUTER_MCP_MAC_APP_DIR = appDir;
      result.MAHAYANA_COMPUTER_MCP_NATIVE_HELPER = helper;
    }
  } else if (bundleHome && platform === 'win32') {
    const helper = path.join(bundleHome, 'native', 'computer-helper.ps1');
    if (safeIsFileSync(fsImpl, helper)) result.MAHAYANA_COMPUTER_MCP_NATIVE_HELPER = helper;
  }
  return result;
}

function productApiBaseUrl(appImpl = app, env = process.env) {
  const configured = env.MAHAYANA_API_BASE_URL?.trim();
  if (!appImpl.isPackaged && configured) {
    const parsed = new URL(configured);
    if (parsed.protocol !== 'https:' || parsed.username || parsed.password || parsed.search || parsed.hash) {
      throw new Error('MAHAYANA_API_BASE_URL must be a clean HTTPS origin/base URL');
    }
    return parsed.toString().replace(/\/$/, '');
  }
  // Signed production packages always use Fabushi's official account/control
  // plane. An inherited shell environment must not redirect credentials or
  // remote-computer traffic to another HTTPS service.
  return appImpl.isPackaged ? PRODUCTION_PRODUCT_API_BASE_URL : DEVELOPMENT_PRODUCT_API_BASE_URL;
}

function persistedInferenceProvider(appImpl = app, fsImpl = fs) {
  return persistedRouterSettings(appImpl, fsImpl).inferenceProvider;
}

function persistedRouterSettings(appImpl = app, fsImpl = fs) {
  try {
    const settingsPath = path.join(appImpl.getPath('userData'), 'feature-host', 'runtime', 'settings.json');
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
    const computerEnvironment = embeddedComputerControlEnvironment({
      app: this.app,
      env: this.env,
      platform: this.platform,
      resourcesPath: this.resourcesPath,
      fs: this.fs,
    });
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
          ...computerEnvironment,
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
  embeddedComputerControlEnvironment,
};
