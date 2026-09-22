const { app } = require('electron');
const { spawn } = require('node:child_process');
const { EventEmitter } = require('node:events');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const readline = require('node:readline');
const { createTestPlatformAccount } = require('./test-platform-account.cjs');
const { createChromePlatformServer } = require('./chrome-platform-server.cjs');

const PRODUCTION_PRODUCT_API_BASE_URL = 'https://api.ombhrum.com';
const DEVELOPMENT_PRODUCT_API_BASE_URL = 'https://mahayana-platform.bhrumom.workers.dev';
const INFERENCE_PROVIDERS = new Set(['fabushi', 'codex', 'claude-code', 'openrouter']);
const SANDBOX_RUNTIMES = new Set(['host', 'local-docker']);
const DEFAULT_DOCKER_IMAGE = 'mcr.microsoft.com/devcontainers/base:ubuntu24.04@sha256:c5cc2b45afe06a1df3aba17e58ba0dc4a02b999493198dab37dd0ccd4e2b0705';
const COORDINATOR_PROTOCOL_VERSION = 1;
const COORDINATOR_CONTROL_CHANNEL = 'coordinator-control';
const COORDINATOR_DATA_CHANNEL = 'coordinator-data';
const COORDINATOR_MAIN_DATA_CHANNEL = 'coordinator-main-data';
const PACKAGED_COMPUTER_RUNTIME_ID = /^v1-[a-f0-9]{20}$/;

function coordinatorEnvelope(channel, frame) {
  return { channel, frame };
}

function coordinatorChannelKey(channel) {
  if (channel === COORDINATOR_CONTROL_CHANNEL) return 'control';
  if (channel === COORDINATOR_DATA_CHANNEL) return 'data';
  if (channel === COORDINATOR_MAIN_DATA_CHANNEL) return 'mainData';
  return null;
}

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
    path.join(root, 'bin', 'fabushi-device-agent.js'),
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
    path.join(root, 'bin', 'fabushi-device-agent.js'),
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
  const appAgentDiscoveryFile = path.join(appImpl.getPath('userData'), 'agent-surface', 'bridge.json');
  const developmentCommand = explicitCommand || execPath;
  const result = {
    MAHAYANA_COMPUTER_MCP_COMMAND: appImpl.isPackaged ? execPath : developmentCommand,
    MAHAYANA_COMPUTER_MCP_ENTRY: runtime.mcpEntry,
    MAHAYANA_COMPUTER_MCP_CWD: runtime.root,
    MAHAYANA_COMPUTER_MCP_HOME: computerHome,
    MAHAYANA_COMPUTER_MCP_POLICY_FILE: policyFile,
    MAHAYANA_COMPUTER_MCP_APP_AGENT_DISCOVERY_FILE: appAgentDiscoveryFile,
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
    this.electronDir = options.electronDir ?? __dirname;
    this.now = options.now ?? Date.now;
    this.fs = options.fs ?? fs;
    this.providerEnvironment = options.providerEnvironment ?? (() => ({}));
    this.testPlatformAccount = this.env.FABUSHI_FEATURE_HOST_MODE === 'test'
      ? createTestPlatformAccount({ app: this.app, fs: this.fs, now: this.now })
      : null;
    this.chromePlatformServer = createChromePlatformServer({
      app: this.app,
      env: this.env,
      platform: this.platform,
      hostRequest: (method, params, timeoutMs) => this.request(method, params, timeoutMs),
    });

    this.child = null;
    this.currentGeneration = 0;
    this.nextId = 1;
    this.pending = new Map();
    this.protocolReady = false;
    this.channelReady = { control: false, data: false, mainData: false };
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
    const name = this.platform === 'win32' ? 'mahayana-app-host.exe' : 'mahayana-app-host';
    if (this.app.isPackaged) return path.join(this.resourcesPath, 'bin', name);

    const explicit = String(this.env.MAHAYANA_APP_HOST_BIN || '').trim();
    if (explicit) return explicit;

    // CI and development builds stage the exact Host generation that should
    // accompany the renderer into desktop/resources/bin. Prefer that staged
    // executable before falling back to a local release-profile Cargo build.
    const staged = path.resolve(this.electronDir, '..', 'resources', 'bin', name);
    if (safeIsFileSync(this.fs, staged)) return staged;

    return path.resolve(
      this.electronDir,
      '..',
      '..',
      'source',
      'host',
      'app',
      'target',
      'release',
      name,
    );
  }

  coordinatorExecutablePath() {
    const name = this.platform === 'win32'
      ? 'mahayana-node-agent-coordinator.exe'
      : 'mahayana-node-agent-coordinator';
    if (this.app.isPackaged) return path.join(this.resourcesPath, 'bin', name);

    const explicit = String(this.env.MAHAYANA_COORDINATOR_BIN || '').trim();
    if (explicit) return explicit;

    const staged = path.resolve(this.electronDir, '..', 'resources', 'bin', name);
    if (safeIsFileSync(this.fs, staged)) return staged;

    return path.resolve(
      this.electronDir,
      '..',
      '..',
      'source',
      'node-agent-coordinator',
      'target',
      'release',
      name,
    );
  }

  health() {
    return Object.freeze({
      state: this.state,
      closed: this.closed,
      generation: this.currentGeneration,
      pid: this.child?.pid ?? null,
      pending: this.pending.size,
      protocolReady: this.protocolReady,
      coordinatorChannels: { ...this.channelReady },
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

  onRuntimeEvent(listener) {
    if (typeof listener !== 'function') throw new TypeError('Mahayana runtime event listener must be a function.');
    this.events.on('runtime-event', listener);
    return () => this.events.off('runtime-event', listener);
  }

  onCoordinatorEvent(listener) {
    if (typeof listener !== 'function') throw new TypeError('Mahayana Coordinator event listener must be a function.');
    this.events.on('coordinator-event', listener);
    return () => this.events.off('coordinator-event', listener);
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

  writeCoordinatorFrame(child, channel, frame, callback) {
    const payload = JSON.stringify(coordinatorEnvelope(channel, frame));
    return child.stdin.write(`${payload}\n`, callback);
  }

  markCoordinatorChannelReady(channel, generation) {
    const key = coordinatorChannelKey(channel);
    if (!key || this.currentGeneration !== generation) return;
    this.channelReady[key] = true;
    if (!this.protocolReady
      && this.channelReady.control
      && this.channelReady.data
      && this.channelReady.mainData) {
      this.protocolReady = true;
      this.emitLifecycle('protocol-ready', { protocolVersion: COORDINATOR_PROTOCOL_VERSION });
    }
  }

  failCoordinatorProtocol(generation, message) {
    const error = message instanceof Error ? message : new Error(String(message));
    this.protocolReady = false;
    this.rejectGeneration(generation, error);
    this.emitLifecycle('protocol-error', { error: error.message });
  }

  handleCoordinatorControlFrame(child, generation, frame) {
    if (frame?.kind === 'lifecycle' && frame.phase === 'hello') {
      if (frame.protocolVersion !== COORDINATOR_PROTOCOL_VERSION) {
        this.failCoordinatorProtocol(
          generation,
          `Coordinator control protocol version ${String(frame.protocolVersion)} does not match ${COORDINATOR_PROTOCOL_VERSION}.`,
        );
        return;
      }
      this.writeCoordinatorFrame(child, COORDINATOR_CONTROL_CHANNEL, {
        kind: 'lifecycle',
        phase: 'ready',
        protocolVersion: COORDINATOR_PROTOCOL_VERSION,
      }, () => undefined);
      this.markCoordinatorChannelReady(COORDINATOR_CONTROL_CHANNEL, generation);
      return;
    }
    if (frame?.kind === 'lifecycle' && frame.phase === 'shutdown') {
      this.failCoordinatorProtocol(generation, frame.detail || frame.reason || 'Coordinator control channel shut down.');
      return;
    }
    if (frame?.kind === 'event') {
      this.events.emit('coordinator-control-event', {
        family: frame.family,
        payload: frame.payload,
      });
      return;
    }
    if (frame?.kind === 'request') {
      this.writeCoordinatorFrame(child, COORDINATOR_CONTROL_CHANNEL, {
        kind: 'reply',
        requestId: String(frame.requestId ?? ''),
        outcome: {
          status: 'failed',
          failure: {
            code: 'COORDINATOR_UNKNOWN_METHOD',
            message: `Electron main has no control handler named ${String(frame.method || '')}`,
          },
        },
      }, () => undefined);
      return;
    }
    if (frame?.kind === 'cancel') return;
    this.failCoordinatorProtocol(generation, 'Unexpected Coordinator control frame.');
  }

  handleCoordinatorDataFrame(channel, generation, frame) {
    const key = coordinatorChannelKey(channel);
    if (!key || key === 'control') {
      this.failCoordinatorProtocol(generation, `Unexpected Coordinator channel ${String(channel)}.`);
      return;
    }
    if (frame?.kind === 'lifecycle') {
      if (frame.phase === 'ready') {
        if (frame.protocolVersion !== COORDINATOR_PROTOCOL_VERSION) {
          this.failCoordinatorProtocol(
            generation,
            `Coordinator protocol version ${String(frame.protocolVersion)} does not match ${COORDINATOR_PROTOCOL_VERSION}.`,
          );
          return;
        }
        this.markCoordinatorChannelReady(channel, generation);
        return;
      }
      if (frame.phase === 'shutdown') {
        this.failCoordinatorProtocol(generation, frame.detail || frame.reason || 'Coordinator data channel shut down.');
        return;
      }
      this.failCoordinatorProtocol(generation, 'Unexpected Coordinator lifecycle frame.');
      return;
    }

    if (!this.channelReady[key]) {
      this.failCoordinatorProtocol(generation, `Coordinator emitted a ${key} frame before the ready handshake.`);
      return;
    }

    if (frame?.kind === 'event') {
      this.events.emit('coordinator-event', {
        channel,
        family: frame.family,
        payload: frame.payload,
      });
      if (channel === COORDINATOR_DATA_CHANNEL
        && frame.family === 'runtime'
        && frame.payload
        && typeof frame.payload === 'object') {
        this.chromePlatformServer.broadcastEvent(frame.payload);
        this.events.emit('runtime-event', frame.payload);
      }
      return;
    }

    if (frame?.kind !== 'reply') {
      this.failCoordinatorProtocol(generation, 'Unexpected Coordinator data frame.');
      return;
    }

    const requestId = String(frame.requestId ?? '');
    const pending = this.pending.get(requestId);
    if (!pending || pending.generation !== generation || pending.channel !== channel) return;
    this.pending.delete(requestId);
    if (frame.outcome?.status === 'ok') pending.resolve(frame.outcome.value);
    else pending.reject(new Error(
      frame.outcome?.failure?.message
        || frame.outcome?.failure?.code
        || 'Mahayana Coordinator request failed',
    ));
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
      const hostExecutable = this.executablePath();
      const coordinatorBootstrap = {
        processConfig: {
          appVersion: typeof this.app.getVersion === 'function' ? this.app.getVersion() : '0.0.0-dev',
          isPackaged: this.app.isPackaged === true,
          dataDir: this.app.getPath('userData'),
        },
      };
      child = this.spawn(
        this.coordinatorExecutablePath(),
        [`--bootstrap=${JSON.stringify(coordinatorBootstrap)}`],
        {
        stdio: ['pipe', 'pipe', 'pipe'],
        env: {
          MAHAYANA_APP_HOST_BIN: hostExecutable,
          ...this.env,
          ANTHROPIC_API_KEY: '',
          OPENROUTER_API_KEY: '',
          MAHAYANA_MODEL_BEARER_TOKEN: '',
          MAHAYANA_OPENROUTER_API_KEY: '',
          MAHAYANA_CLAUDE_API_KEY: '',
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
        },
      );
    } catch (error) {
      this.state = 'stopped';
      this.startedAt = null;
      this.unexpectedExitCount += 1;
      this.lastExit = Object.freeze({ error: error instanceof Error ? error.message : String(error), at: this.now() });
      this.emitLifecycle('spawn-failed', { error: this.lastExit.error });
      throw error;
    }

    this.child = child;
    this.protocolReady = false;
    this.channelReady = { control: false, data: false, mainData: false };
    this.state = 'running';
    this.startedAt = this.now();
    this.emitLifecycle('running');
    // Packaged Fabushi always exposes the Chrome platform. Development and
    // unit-test hosts opt in explicitly so a test process never creates a
    // real per-user socket or native-messaging state by accident.
    if (this.app.isPackaged || this.env.FABUSHI_ENABLE_CHROME_PLATFORM === '1') {
      void this.chromePlatformServer.start().catch((error) => {
        console.error('[chrome-platform] desktop bridge failed to start', error);
      });
    }

    const lines = this.readline.createInterface({ input: child.stdout });
    lines.on('line', (line) => {
      let envelope;
      try {
        envelope = JSON.parse(line);
      } catch (error) {
        this.failCoordinatorProtocol(generation, new Error(`Invalid Mahayana Coordinator response: ${error}`));
        return;
      }
      const channel = String(envelope?.channel || '');
      const frame = envelope?.frame;
      if (coordinatorChannelKey(channel) == null || !frame || typeof frame !== 'object') {
        this.failCoordinatorProtocol(generation, 'Invalid Mahayana Coordinator carrier envelope.');
        return;
      }
      if (channel === COORDINATOR_CONTROL_CHANNEL) {
        this.handleCoordinatorControlFrame(child, generation, frame);
        return;
      }
      this.handleCoordinatorDataFrame(channel, generation, frame);
    });

    this.writeCoordinatorFrame(child, COORDINATOR_DATA_CHANNEL, {
      kind: 'lifecycle',
      phase: 'hello',
      protocolVersion: COORDINATOR_PROTOCOL_VERSION,
    });
    this.writeCoordinatorFrame(child, COORDINATOR_MAIN_DATA_CHANNEL, {
      kind: 'lifecycle',
      phase: 'hello',
      protocolVersion: COORDINATOR_PROTOCOL_VERSION,
    });

    child.stderr.on('data', (chunk) => console.error(`[mahayana-coordinator] ${String(chunk).trimEnd()}`));
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
      this.protocolReady = false;
      this.channelReady = { control: false, data: false, mainData: false };
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
    return this.requestOnChannel(COORDINATOR_DATA_CHANNEL, method, params, timeoutMs);
  }

  mainRequest(method, params = {}, timeoutMs = 120000) {
    return this.requestOnChannel(COORDINATOR_MAIN_DATA_CHANNEL, method, params, timeoutMs);
  }

  requestOnChannel(channel, method, params = {}, timeoutMs = 120000) {
    if (method === 'platform.request' && this.testPlatformAccount) {
      const result = this.testPlatformAccount.request(params);
      if (result) return Promise.resolve(result);
    }
    if (channel !== COORDINATOR_DATA_CHANNEL && channel !== COORDINATOR_MAIN_DATA_CHANNEL) {
      return Promise.reject(new Error(`Invalid Coordinator request channel: ${String(channel)}`));
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
        if (!pending || pending.generation !== generation || pending.channel !== channel) return;
        this.pending.delete(key);
        this.emitLifecycle('request-timeout', { method: String(method), requestId: id, channel });
        if (this.child === child && !child.stdin?.destroyed) {
          this.writeCoordinatorFrame(child, channel, { kind: 'cancel', requestId: key }, () => undefined);
        }
        reject(new Error(`Mahayana Coordinator request timed out: ${method}`));
      }, timeoutMs);
      timer.unref?.();
      this.pending.set(key, {
        generation,
        channel,
        resolve: (value) => {
          clearTimeout(timer);
          resolve(value);
        },
        reject: (error) => { clearTimeout(timer); reject(error); },
      });
      this.writeCoordinatorFrame(
        child,
        channel,
        { kind: 'request', requestId: key, method, args: params },
        (error) => {
          if (!error) return;
          const pending = this.pending.get(key);
          if (!pending || pending.generation !== generation || pending.channel !== channel) return;
          this.pending.delete(key);
          pending.reject(error);
        },
      );
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
      this.protocolReady = false;
      this.channelReady = { control: false, data: false, mainData: false };
      this.startedAt = null;
      this.rejectGeneration(generation, new Error(`Mahayana host restarted: ${reason}`));
      child.stdin?.end?.();
      const fallbackKill = setTimeout(() => child.kill(), 1_000);
      fallbackKill.unref?.();
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
    this.protocolReady = false;
    this.channelReady = { control: false, data: false, mainData: false };
    this.startedAt = null;
    this.rejectGeneration(generation, new Error('Mahayana host closed.'));
    this.emitLifecycle('closed');
    if (child) {
      for (const channel of [
        COORDINATOR_CONTROL_CHANNEL,
        COORDINATOR_DATA_CHANNEL,
        COORDINATOR_MAIN_DATA_CHANNEL,
      ]) {
        this.writeCoordinatorFrame(child, channel, {
          kind: 'lifecycle',
          phase: 'shutdown',
          reason: 'requested',
        }, () => undefined);
      }
      child.stdin?.end?.();
      const fallbackKill = setTimeout(() => child.kill(), 1_000);
      fallbackKill.unref?.();
    }
    void this.chromePlatformServer.close().catch((error) => console.error('[chrome-platform] desktop bridge shutdown failed', error));
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
