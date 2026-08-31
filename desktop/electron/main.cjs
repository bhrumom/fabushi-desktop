const { app, BrowserWindow, dialog, ipcMain, Menu, nativeImage, net, nativeTheme, Notification, protocol, safeStorage, shell, session, Tray } = require('electron');
const { autoUpdater } = require('electron-updater');
const fs = require('node:fs/promises');
const fsSync = require('node:fs');
const path = require('node:path');
const { pathToFileURL, URL } = require('node:url');
const { Readable } = require('node:stream');
const { MahayanaHostProcess } = require('./host-process.cjs');
const { serveMainEdge } = require('./edge-ipc.cjs');
const { MAHAYANA_EDGE } = require('./mahayana-edge.cjs');
const { NATIVE_EDGE } = require('./native-edge.cjs');
const { createNativeCapabilityHandlers } = require('./native-capability-handlers.cjs');
const { MessagingSignalingClient } = require('./messaging-signaling-client.cjs');
const { createAppAgentSurfaceServer } = require('./app-agent-surface-server.cjs');
const { RemoteDeviceAgentSupervisor } = require('./remote-device-agent-supervisor.cjs');

const appDataOverride = process.env.FABUSHI_APP_DATA?.trim();
if (appDataOverride) app.setPath('userData', path.resolve(appDataOverride));

protocol.registerSchemesAsPrivileged([
  {
    scheme: 'app',
    privileges: { standard: true, secure: true, supportFetchAPI: true },
  },
  {
    scheme: 'fabushi-blob',
    privileges: { standard: true, secure: true, supportFetchAPI: true, stream: true },
  },
]);

function encryptedProviderSecret(name) {
  if (!safeStorage?.isEncryptionAvailable?.()) {
    return null;
  }
  const secretFile = path.join(app.getPath('userData'), 'secure', 'secrets.json');
  let vault;
  try { vault = JSON.parse(fsSync.readFileSync(secretFile, 'utf8')); }
  catch { return null; }
  const ciphertext = vault?.[name]?.ciphertext;
  if (typeof ciphertext !== 'string' || !ciphertext) {
    return null;
  }
  try {
    const value = safeStorage.decryptString(Buffer.from(ciphertext, 'base64'));
    return value && !/[\r\n]/.test(value) ? value : null;
  } catch {
    return null;
  }
}

function providerEnvironment(inferenceProvider) {
  if (inferenceProvider === 'openrouter') {
    const value = encryptedProviderSecret('inference/openrouter/api-key');
    if (!value) return {};
    return {
      MAHAYANA_MODEL_BEARER_TOKEN: value,
      MAHAYANA_OPENROUTER_MODEL: process.env.MAHAYANA_OPENROUTER_MODEL || 'openai/gpt-5.2',
    };
  }
  if (inferenceProvider === 'claude-code') {
    const value = encryptedProviderSecret('inference/claude/api-key') || process.env.ANTHROPIC_API_KEY?.trim();
    if (!value || /[\r\n]/.test(value)) return {};
    return {
      MAHAYANA_MODEL_BEARER_TOKEN: value,
      MAHAYANA_CLAUDE_MODEL: process.env.MAHAYANA_CLAUDE_MODEL || 'claude-sonnet-4-6',
    };
  }
  return {
    // Never forward provider credentials to the Fabushi/Codex Host generation.
  };
}

const host = new MahayanaHostProcess({ providerEnvironment });
let mahayanaEdgeServer = null;
let nativeEdgeServer = null;
let appAgentSurfaceServer = null;
let remoteDeviceAgentSupervisor = null;
let appAgentSurfaceShutdownPending = false;
let appAgentSurfaceShutdownComplete = false;
let hostEventPumpStopped = false;
let hostEventPump = null;
const messagingAccessCache = new Map();
let messagingSignalingClient = null;
let availableDesktopUpdateVersion = null;
let runtimeDesktopUpdateStatus = null;
const DESKTOP_UPDATE_CHECK_MIN_INTERVAL_MS = 60_000;
const DESKTOP_UPDATE_FOREGROUND_INTERVAL_MS = 5 * 60_000;
let lastAutomaticDesktopUpdateCheckAt = 0;
let automaticDesktopUpdateCheckTimer = null;
let automaticDesktopUpdateCheckPromise = null;
let mainWindow = null;
let backgroundTray = null;
let quitting = false;
const backgroundPersistenceEnabled = process.env.FABUSHI_E2E !== '1';

function appAgentControlPolicyDecision() {
  const configured = String(process.env.FABUSHI_COMPUTER_POLICY_FILE || '').trim();
  const policyFile = configured
    ? path.resolve(configured)
    : path.join(app.getPath('userData'), 'feature-host', 'runtime', 'settings.json');
  let settings;
  try { settings = JSON.parse(fsSync.readFileSync(policyFile, 'utf8')); }
  catch { return { allowed: false, reason: 'Fabushi computer-control policy is unavailable.' }; }
  if (!settings || typeof settings !== 'object' || Array.isArray(settings)) {
    return { allowed: false, reason: 'Fabushi computer-control policy is invalid.' };
  }
  if (settings.localExecution !== true || settings.aiComputerControlEnabled !== true) {
    return { allowed: false, reason: 'Fabushi AI computer control is disabled.' };
  }
  if (!['ask', 'always'].includes(settings.localToolPermission)) {
    return { allowed: false, reason: 'Fabushi local tool permission denies control.' };
  }
  return { allowed: true };
}

function appAgentSurfaceDiscoveryPath() {
  const configured = String(process.env.FABUSHI_APP_AGENT_DISCOVERY_FILE || '').trim();
  return configured
    ? path.resolve(configured)
    : path.join(app.getPath('userData'), 'agent-surface', 'bridge.json');
}

async function startAppAgentSurfaceServer() {
  if (appAgentSurfaceServer) return appAgentSurfaceServer;
  const bridge = createAppAgentSurfaceServer({
    discoveryPath: appAgentSurfaceDiscoveryPath(),
    authorize: () => appAgentControlPolicyDecision(),
    onRequest(request) {
      const win = mainWindow && !mainWindow.isDestroyed() ? mainWindow : null;
      if (!win || win.webContents.isDestroyed() || !nativeEdgeServer) {
        throw new Error('app_surface_renderer_unavailable');
      }
      nativeEdgeServer.emit(win.webContents, 'app-agent-surface-request', request);
    },
  });
  await bridge.start();
  appAgentSurfaceServer = bridge;
  console.info(JSON.stringify({
    type: 'fabushi.app-agent-surface.ready',
    origin: bridge.origin,
    discoveryPath: bridge.discoveryPath,
  }));
  return bridge;
}

function normalizeMessagingAccessParams(params) {
  const deviceId = String(params?.deviceId || 'desktop:electron').trim();
  const sessionId = String(params?.sessionId || '').trim();
  if (!deviceId || deviceId.length > 200 || !sessionId || sessionId.length > 200) {
    throw new Error('Messaging access requires a valid deviceId and sessionId.');
  }
  return { deviceId, sessionId };
}

async function getOrIssueMessagingAccess(params, requestedScopes) {
  const { deviceId, sessionId } = normalizeMessagingAccessParams(params);
  const scopes = [...new Set(requestedScopes.map((scope) => String(scope)))].sort();
  const key = `${deviceId}|${sessionId}|${scopes.join(',')}`;
  const cached = messagingAccessCache.get(key);
  if (cached && Number(cached.expiresAtMs || 0) > Date.now() + 60_000) return cached;
  const credential = await host.request('feature.messaging.access.issue', {
    deviceId,
    sessionId,
    scopes,
    ttlMs: 24 * 60 * 60 * 1000,
  });
  if (!credential || typeof credential !== 'object'
      || typeof credential.actorId !== 'string'
      || typeof credential.accessToken !== 'string'
      || credential.accessToken.length < 32) {
    throw new Error('Fabushi account session did not return a valid messaging credential.');
  }
  const normalized = { ...credential, deviceId, sessionId, scopes };
  messagingAccessCache.set(key, normalized);
  return normalized;
}

function callIceServers() {
  const raw = String(process.env.FABUSHI_CALL_ICE_SERVERS_JSON || '').trim();
  if (!raw) return [];
  let parsed;
  try { parsed = JSON.parse(raw); } catch { throw new Error('FABUSHI_CALL_ICE_SERVERS_JSON must be valid JSON.'); }
  if (!Array.isArray(parsed) || parsed.length > 16) throw new Error('Fabushi call ICE server configuration is invalid.');
  return parsed.map((entry) => {
    if (!entry || typeof entry !== 'object') throw new Error('Fabushi call ICE server entry is invalid.');
    const urls = (Array.isArray(entry.urls) ? entry.urls : [entry.urls])
      .map((value) => String(value || '').trim())
      .filter(Boolean);
    if (!urls.length || urls.some((url) => !/^(stun|turn|turns):/i.test(url))) {
      throw new Error('Fabushi call ICE server URLs must use stun:, turn:, or turns:.');
    }
    const normalized = { urls };
    if (entry.username != null) normalized.username = String(entry.username);
    if (entry.credential != null) normalized.credential = String(entry.credential);
    return normalized;
  });
}

function safeMessagingIdentity(credential) {
  return {
    actorId: String(credential.actorId),
    deviceId: String(credential.deviceId),
    sessionId: String(credential.sessionId),
    expiresAtMs: Number(credential.expiresAtMs || 0),
    scopes: Array.isArray(credential.scopes) ? [...credential.scopes] : [],
  };
}

function callSignalingClient() {
  if (messagingSignalingClient) return messagingSignalingClient;
  messagingSignalingClient = new MessagingSignalingClient({
    onSignal(signal) { broadcastNativeEvent('messaging-call-signal', signal); },
    onStatus(status) { broadcastNativeEvent('messaging-call-status', status); },
  });
  return messagingSignalingClient;
}

const DEEP_LINK_PROTOCOL = 'fabushi:';
const DEEP_LINK_PENDING_LIMIT = 32;
const DEEP_LINK_DEDUPE_MS = 5_000;

function focusMainWindow() {
  if (!app.isReady()) return;
  let win = mainWindow && !mainWindow.isDestroyed()
    ? mainWindow
    : BrowserWindow.getAllWindows().find((candidate) => !candidate.isDestroyed());
  if (!win) win = createWindow();
  if (win.isMinimized()) win.restore();
  win.show();
  win.focus();
}

function requestApplicationQuit() {
  quitting = true;
  app.quit();
}

function parseFabushiDeepLink(candidate) {
  if (typeof candidate !== 'string' || !candidate.toLowerCase().startsWith(DEEP_LINK_PROTOCOL)) return null;
  let url;
  try { url = new URL(candidate); } catch { return null; }
  if (url.protocol !== DEEP_LINK_PROTOCOL || url.username || url.password || url.port) return null;
  const hostName = url.hostname.toLowerCase();
  const pathParts = url.pathname.split('/').filter(Boolean).map((part) => decodeURIComponent(part));
  if (hostName === 'agent') {
    const agentId = String(pathParts[0] ?? url.searchParams.get('id') ?? '').trim().slice(0, 200);
    return agentId ? { version: 1, route: 'agent', agentId, source: 'protocol', canonicalUrl: `fabushi://agent/${encodeURIComponent(agentId)}` } : null;
  }
  if (hostName === 'auth' && pathParts[0] === 'complete') {
    const attemptId = String(url.searchParams.get('attemptId') ?? '').trim();
    const status = String(url.searchParams.get('status') ?? 'completed').trim().toLowerCase();
    if (!/^[A-Za-z0-9_-]{8,96}$/.test(attemptId) || !['completed', 'cancelled', 'failed'].includes(status)) return null;
    return {
      version: 1,
      route: 'auth',
      action: 'complete',
      attemptId,
      status,
      source: 'protocol',
      canonicalUrl: `fabushi://auth/complete?attemptId=${encodeURIComponent(attemptId)}&status=${status}`,
    };
  }
  if (hostName === 'settings') {
    const section = String(pathParts[0] ?? url.searchParams.get('section') ?? 'general').trim();
    if (!['general', 'mcp', 'usage', 'updates'].includes(section)) return null;
    return { version: 1, route: 'settings', section, source: 'protocol', canonicalUrl: `fabushi://settings/${section}` };
  }
  if (hostName === 'feedback') return { version: 1, route: 'feedback', source: 'protocol', canonicalUrl: 'fabushi://feedback' };
  if (hostName === 'about') return { version: 1, route: 'about', source: 'protocol', canonicalUrl: 'fabushi://about' };
  if (hostName === 'widgets') return { version: 1, route: 'widgets', source: 'protocol', canonicalUrl: 'fabushi://widgets' };
  if (hostName === 'onboarding') {
    const action = pathParts[0] === 'skip' ? 'skip' : 'start';
    return { version: 1, route: 'onboarding', action, source: 'protocol', canonicalUrl: `fabushi://onboarding/${action}` };
  }
  return null;
}

class FabushiDeepLinkRouter {
  constructor() {
    this.ready = false;
    this.pending = [];
    this.recent = new Map();
  }

  handle(candidate, source) {
    const parsed = parseFabushiDeepLink(candidate);
    if (!parsed) return false;
    parsed.source = source;
    const now = Date.now();
    for (const [key, acceptedAt] of this.recent) {
      if (now - acceptedAt > DEEP_LINK_DEDUPE_MS) this.recent.delete(key);
    }
    if (this.recent.has(parsed.canonicalUrl) || this.pending.some((entry) => entry.canonicalUrl === parsed.canonicalUrl)) return false;
    this.recent.set(parsed.canonicalUrl, now);
    focusMainWindow();
    if (!this.ready) {
      if (this.pending.length >= DEEP_LINK_PENDING_LIMIT) this.pending.shift();
      this.pending.push(parsed);
      return true;
    }
    this.dispatch(parsed);
    return true;
  }

  handleArgv(argv, source) {
    for (const value of Array.isArray(argv) ? argv : []) this.handle(value, source);
  }

  markReady() {
    this.ready = true;
    const pending = this.pending.splice(0);
    for (const entry of pending) this.dispatch(entry);
    return { ready: true, flushed: pending.length };
  }

  markNotReady() {
    this.ready = false;
  }

  dispatch(link) {
    broadcastNativeEvent('deep-link', link);
    if (link.route === 'agent') broadcastNativeEvent('focus-agent', { agentId: link.agentId, source: link.source });
    if (link.route === 'feedback') broadcastNativeEvent('open-feedback', { source: link.source });
    if (link.route === 'about') broadcastNativeEvent('open-about', { source: link.source });
    if (link.route === 'widgets') broadcastNativeEvent('widget-gallery', { source: link.source });
    if (link.route === 'onboarding' && link.action === 'skip') broadcastNativeEvent('skip-onboarding', { source: link.source });
    if (link.route === 'onboarding' && link.action !== 'skip') broadcastNativeEvent('force-onboarding', { source: link.source });
  }
}

const deepLinkRouter = new FabushiDeepLinkRouter();
const primaryInstance = !app.isPackaged || app.requestSingleInstanceLock();
if (!primaryInstance) app.quit();

if (primaryInstance) {
  app.on('second-instance', (_event, argv) => {
    focusMainWindow();
    deepLinkRouter.handleArgv(argv, 'second-instance');
  });
  app.on('open-url', (event, url) => {
    event.preventDefault();
    focusMainWindow();
    deepLinkRouter.handle(url, 'open-url');
  });
  deepLinkRouter.handleArgv(process.argv, 'initial-argv');
}

function isTrustedRendererUrl(value) {
  try {
    const parsed = new URL(value);
    if (parsed.protocol === 'app:' && parsed.hostname === 'bundle') return true;
    if (process.env.VITE_DEV_SERVER_URL) {
      const dev = new URL(process.env.VITE_DEV_SERVER_URL);
      return parsed.origin === dev.origin;
    }
  } catch {}
  return false;
}

function assertTrustedSender(event) {
  const url = event.senderFrame?.url || event.sender.getURL();
  if (isTrustedRendererUrl(url)) return;
  throw new Error(`Rejected IPC sender: ${url}`);
}

function isTrustedSender(event) {
  try {
    assertTrustedSender(event);
    return true;
  } catch {
    return false;
  }
}

function normalizeParams(value) {
  return value && typeof value === 'object' && !Array.isArray(value) ? value : {};
}

function safeHttpsUrl(value) {
  const parsed = new URL(String(value));
  if (parsed.protocol !== 'https:' || !parsed.hostname) throw new Error('Only HTTPS URLs may be opened externally.');
  return parsed.toString();
}

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

function nativeStatePath() {
  return path.join(app.getPath('userData'), 'fabushi-native-state.json');
}

let nativeStateWrite = Promise.resolve();

function applyStartupNativePreferences() {
  try {
    const raw = fsSync.readFileSync(nativeStatePath(), 'utf8');
    const state = JSON.parse(raw);
    if (state?.preferences?.hardwareAccelerationEnabled === false) app.disableHardwareAcceleration();
  } catch (error) {
    if (error?.code !== 'ENOENT') console.warn('[native-edge] unable to apply startup preferences', error);
  }
}

async function readNativeState() {
  try {
    const raw = await fs.readFile(nativeStatePath(), 'utf8');
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
  } catch (error) {
    if (error?.code === 'ENOENT') return {};
    throw error;
  }
}

async function mutateNativeState(mutator) {
  nativeStateWrite = nativeStateWrite.then(async () => {
    const state = await readNativeState();
    const next = mutator({ ...state }) ?? state;
    const file = nativeStatePath();
    await fs.mkdir(path.dirname(file), { recursive: true });
    const temp = `${file}.tmp`;
    await fs.writeFile(temp, `${JSON.stringify(next, null, 2)}\n`, { mode: 0o600 });
    await fs.rename(temp, file);
  });
  return nativeStateWrite;
}

function normalizePersistedDesktopUpdateStatus(status) {
  const currentVersion = app.getVersion();
  if (!status || typeof status !== 'object') return { type: 'upToDate', version: currentVersion };
  const version = typeof status.version === 'string' && status.version ? status.version : currentVersion;
  if (status.type === 'upToDate') return { ...status, version: currentVersion };
  if (version === currentVersion && ['available', 'downloading', 'ready', 'staging'].includes(status.type)) {
    return { type: 'upToDate', version: currentVersion };
  }
  return { ...status, version };
}

async function getDesktopUpdateStatus() {
  if (runtimeDesktopUpdateStatus) return runtimeDesktopUpdateStatus;
  const state = await readNativeState();
  runtimeDesktopUpdateStatus = normalizePersistedDesktopUpdateStatus(state.updateStatus);
  return runtimeDesktopUpdateStatus;
}

function setDesktopUpdateStatus(status, { broadcast = true } = {}) {
  // The updater is a live process state machine. Set memory before broadcasting so
  // a renderer click triggered by this exact event can never read stale disk state.
  runtimeDesktopUpdateStatus = status;
  if (broadcast) broadcastNativeEvent('update-status', status);
  return mutateNativeState((state) => ({ ...state, updateStatus: status }))
    .catch((error) => {
      console.warn('[updater] unable to persist live update status', error instanceof Error ? error.message : String(error));
    })
    .then(() => status);
}

function persistenceKey(value) {
  const key = String(value ?? '').trim();
  if (!key || key.length > 160 || !/^[a-zA-Z0-9._:/-]+$/.test(key)) {
    throw new Error('Invalid persistence key.');
  }
  return key;
}

const DISK_AUDIT_MAX_NODES = 50_000;
const DISK_AUDIT_MAX_DEPTH = 12;
const RECLAIMABLE_TOP_LEVEL_NAMES = new Set([
  'cache',
  'code cache',
  'gpucache',
  'logs',
  'temp',
  'tmp',
]);

async function measureDiskEntry(target, state, depth = 0) {
  if (state.scannedNodes >= DISK_AUDIT_MAX_NODES || depth > DISK_AUDIT_MAX_DEPTH) {
    state.truncated = true;
    return 0;
  }
  state.scannedNodes += 1;
  let stat;
  try {
    stat = await fs.lstat(target);
  } catch {
    return 0;
  }
  if (stat.isSymbolicLink()) return 0;
  if (stat.isFile()) return Math.max(0, Number(stat.size) || 0);
  if (!stat.isDirectory()) return 0;
  let children;
  try {
    children = await fs.readdir(target, { withFileTypes: true });
  } catch {
    return 0;
  }
  let bytes = 0;
  for (const child of children) {
    if (state.scannedNodes >= DISK_AUDIT_MAX_NODES) {
      state.truncated = true;
      break;
    }
    bytes += await measureDiskEntry(path.join(target, child.name), state, depth + 1);
  }
  return bytes;
}

async function auditUserDataStorage() {
  const root = app.getPath('userData');
  const state = { scannedNodes: 0, truncated: false };
  let topLevel = [];
  try {
    topLevel = await fs.readdir(root, { withFileTypes: true });
  } catch (error) {
    throw new Error(`Unable to inspect desktop storage: ${error instanceof Error ? error.message : String(error)}`);
  }
  const entries = [];
  for (const item of topLevel) {
    const bytes = await measureDiskEntry(path.join(root, item.name), state, 0);
    entries.push({
      name: item.name,
      bytes,
      reclaimable: RECLAIMABLE_TOP_LEVEL_NAMES.has(item.name.trim().toLowerCase()),
    });
  }
  entries.sort((left, right) => right.bytes - left.bytes || left.name.localeCompare(right.name));
  const totalBytes = entries.reduce((total, entry) => total + entry.bytes, 0);
  const reclaimableBytes = entries.reduce(
    (total, entry) => total + (entry.reclaimable ? entry.bytes : 0),
    0,
  );
  return {
    root,
    totalBytes,
    reclaimableBytes,
    scannedNodes: state.scannedNodes,
    truncated: state.truncated,
    scannedAtMs: Date.now(),
    entries: entries.slice(0, 50),
  };
}

function windowForEvent(event) {
  const win = BrowserWindow.fromWebContents(event.sender);
  if (!win || win.isDestroyed()) throw new Error('Renderer window is unavailable.');
  return win;
}

function describeWindow(win) {
  const bounds = win.getBounds();
  return {
    focused: win.isFocused(),
    minimized: win.isMinimized(),
    maximized: win.isMaximized(),
    fullScreen: win.isFullScreen(),
    bounds,
  };
}

function describeTheme() {
  return {
    preference: nativeTheme.themeSource,
    dark: nativeTheme.shouldUseDarkColors,
    highContrast: nativeTheme.shouldUseHighContrastColors,
  };
}

function broadcastNativeEvent(eventName, payload) {
  if (!nativeEdgeServer) return;
  for (const win of BrowserWindow.getAllWindows()) {
    if (win.isDestroyed() || win.webContents.isDestroyed()) continue;
    nativeEdgeServer.emit(win.webContents, eventName, payload);
  }
}

function logEdgeInvocation(record) {
  // Deliberately exclude args/results/URLs/tokens. This record is safe to retain as
  // operational telemetry and gives renderer -> edge correlation without secrets.
  console.info(JSON.stringify({ type: 'fabushi.edge.invoke', ...record }));
}

function installNativeEdge() {
  const handlers = {
    respondAppAgentSurfaceRequest(params, event) {
      const win = BrowserWindow.fromWebContents(event.sender);
      if (!mainWindow || mainWindow.isDestroyed() || win !== mainWindow) {
        throw new Error('Only the primary trusted Fabushi renderer may answer App Agent Surface requests.');
      }
      if (!appAgentSurfaceServer) throw new Error('App Agent Surface is unavailable.');
      if (!appAgentSurfaceServer.respond(params)) throw new Error('App Agent Surface request is missing or already settled.');
      return true;
    },
    async openExternal(params) {
      await shell.openExternal(safeHttpsUrl(params.url));
      return true;
    },
    async getMessagingIdentity(params) {
      const credential = await getOrIssueMessagingAccess(params, [
        'messaging', 'calls', 'blobsRead', 'blobsWrite', 'payments', 'miniApps',
      ]);
      return safeMessagingIdentity(credential);
    },
    async connectMessagingSignaling(params) {
      const credential = await getOrIssueMessagingAccess(params, ['calls']);
      const endpoint = String(process.env.FABUSHI_CALL_SIGNAL_ENDPOINT || 'tcp://127.0.0.1:9410').trim();
      const connection = await callSignalingClient().connect(endpoint, {
        actorId: credential.actorId,
        deviceId: credential.deviceId,
        sessionId: credential.sessionId,
        accessToken: credential.accessToken,
      });
      return {
        ...safeMessagingIdentity(credential),
        secure: connection.secure === true,
        iceServers: callIceServers(),
      };
    },
    sendMessagingSignal(params) {
      const signal = params?.signal;
      if (!signal || typeof signal !== 'object' || Array.isArray(signal)) {
        throw new Error('Fabushi call signaling requires a signal object.');
      }
      return callSignalingClient().send(signal);
    },
    disconnectMessagingSignaling() {
      callSignalingClient().disconnect();
      return true;
    },
    getDesktopEnvironment() {
      return {
        platform: process.platform,
        arch: process.arch,
        appVersion: app.getVersion(),
        electronVersion: process.versions.electron,
        packaged: app.isPackaged,
      };
    },
    getWindowState(_params, event) {
      return describeWindow(windowForEvent(event));
    },
    minimizeWindow(_params, event) {
      windowForEvent(event).minimize();
      return true;
    },
    toggleMaximizeWindow(_params, event) {
      const win = windowForEvent(event);
      if (win.isMaximized()) win.unmaximize();
      else win.maximize();
      return describeWindow(win);
    },
    closeWindow(_params, event) {
      windowForEvent(event).close();
      return true;
    },
    resizeWindowWidth(params, event) {
      const win = windowForEvent(event);
      const width = Math.max(880, Math.min(2400, Math.round(Number(params.width) || 1180)));
      const [currentWidth, height] = win.getSize();
      if (currentWidth !== width) win.setSize(width, height, true);
      return describeWindow(win);
    },
    getThemeState() {
      return describeTheme();
    },
    setThemePreference(params) {
      const preference = String(params.preference ?? 'system');
      if (!['system', 'light', 'dark'].includes(preference)) throw new Error('Unsupported theme preference.');
      nativeTheme.themeSource = preference;
      const state = describeTheme();
      broadcastNativeEvent('theme-changed', state);
      return state;
    },
    relaunchDesktop() {
      setImmediate(() => {
        app.relaunch();
        app.exit(0);
      });
      return true;
    },
    async getOnboardingSeen() {
      const state = await readNativeState();
      return state.onboardingSeen === true;
    },
    async setOnboardingSeen(params) {
      const value = params.seen !== false;
      await mutateNativeState((state) => ({ ...state, onboardingSeen: value }));
      return value;
    },
    async getTimeZone() {
      const state = await readNativeState();
      return typeof state.timeZoneOverride === 'string' && state.timeZoneOverride
        ? state.timeZoneOverride
        : Intl.DateTimeFormat().resolvedOptions().timeZone;
    },
    async setTimeZoneOverride(params) {
      const value = params.timeZone == null ? '' : String(params.timeZone).trim();
      if (value) new Intl.DateTimeFormat('en-US', { timeZone: value }).format(new Date());
      await mutateNativeState((state) => {
        if (value) state.timeZoneOverride = value;
        else delete state.timeZoneOverride;
        return state;
      });
      return value || null;
    },
    async getSidebarCollapsed() {
      const state = await readNativeState();
      return state.sidebarCollapsed === true;
    },
    async setSidebarCollapsed(params) {
      const value = params.collapsed === true;
      await mutateNativeState((state) => ({ ...state, sidebarCollapsed: value }));
      return value;
    },
    async readClientPersistence(params) {
      const key = persistenceKey(params.key);
      const state = await readNativeState();
      return state.clientPersistence?.[key] ?? null;
    },
    async writeClientPersistence(params) {
      const key = persistenceKey(params.key);
      const value = params.value;
      await mutateNativeState((state) => ({
        ...state,
        clientPersistence: { ...(state.clientPersistence ?? {}), [key]: value },
      }));
      return true;
    },
    async removeClientPersistence(params) {
      const key = persistenceKey(params.key);
      await mutateNativeState((state) => {
        const next = { ...(state.clientPersistence ?? {}) };
        delete next[key];
        return { ...state, clientPersistence: next };
      });
      return true;
    },
    async listClientPersistenceKeys(params) {
      const prefix = params.prefix == null ? '' : String(params.prefix);
      const state = await readNativeState();
      return Object.keys(state.clientPersistence ?? {}).filter((key) => key.startsWith(prefix)).sort();
    },
    requestDiskSaverAudit() {
      return auditUserDataStorage();
    },
  };

  Object.assign(handlers, createNativeCapabilityHandlers({
    app,
    autoUpdater,
    dialog,
    net,
    nativeTheme,
    safeStorage,
    shell,
    host,
    readNativeState,
    mutateNativeState,
    getDesktopUpdateStatus,
    setDesktopUpdateStatus,
    windowForEvent,
    broadcastNativeEvent,
    markDeepLinksReady: () => deepLinkRouter.markReady(),
  }));

  nativeEdgeServer = serveMainEdge(ipcMain, NATIVE_EDGE, handlers, {
    isTrustedSender,
    onInvocation: logEdgeInvocation,
    onHandlerError(method, error) {
      console.error(`[native-edge] ${method} failed`, error);
    },
  });
}

async function authorizeMahayanaParams(method, params) {
  const normalized = normalizeParams(params);
  if (method !== 'feature.execute') return normalized;
  const command = normalized.command;
  if (!command || command.type !== 'messaging.execute') return normalized;
  const context = command.envelope?.context;
  const deviceId = String(context?.deviceId || '').trim();
  const sessionId = String(context?.sessionId || '').trim();
  const actorId = String(context?.actorId || '').trim();
  if (!deviceId || !sessionId || !actorId) {
    throw new Error('Messaging envelope requires account-bound actor, device, and session identity.');
  }
  const credential = await getOrIssueMessagingAccess({ deviceId, sessionId }, ['messaging']);
  if (String(credential.actorId || '') !== actorId) {
    throw new Error('Messaging envelope actor does not match authenticated account.');
  }
  return normalized;
}

function installMahayanaEdge() {
  const handlers = Object.fromEntries(
    Object.keys(MAHAYANA_EDGE.methods).map((method) => [
      method,
      async (params) => host.request(method, await authorizeMahayanaParams(method, params)),
    ]),
  );

  mahayanaEdgeServer = serveMainEdge(ipcMain, MAHAYANA_EDGE, handlers, {
    isTrustedSender,
    onInvocation: logEdgeInvocation,
    onHandlerError(method, error) {
      console.error(`[mahayana-edge] ${method} failed`, error);
    },
  });
}

function noteRuntimeUsage(event) {
  if (!event || event.type !== 'usage.updated') return;
  const provider = String(host.activeInferenceProvider || 'fabushi');
  const inputTokens = Math.max(0, Number(event.inputTokens ?? 0));
  const cachedInputTokens = Math.max(0, Number(event.cachedInputTokens ?? 0));
  const outputTokens = Math.max(0, Number(event.outputTokens ?? 0));
  const reasoningTokens = Math.max(0, Number(event.reasoningTokens ?? 0));
  const totalTokens = Math.max(0, Number(event.totalTokens ?? 0));
  const item = { timestampMs: Date.now(), provider, inputTokens, cachedInputTokens, outputTokens, reasoningTokens, totalTokens };
  void mutateNativeState((state) => {
    const previous = Array.isArray(state.usageEvents) ? state.usageEvents : [];
    const cutoff = Date.now() - 35 * 24 * 60 * 60 * 1000;
    const usageEvents = [...previous.filter((entry) => Number(entry.timestampMs) >= cutoff), item].slice(-5000);
    return {
      ...state,
      usageEvents,
      usageLifetimeTokens: Number(state.usageLifetimeTokens ?? 0) + totalTokens,
      usageLifetimeByProvider: {
        ...(state.usageLifetimeByProvider ?? {}),
        [provider]: Number(state.usageLifetimeByProvider?.[provider] ?? 0) + totalTokens,
      },
      usageUpdatedAtMs: item.timestampMs,
    };
  }).catch((error) => console.error('[native-edge] failed to persist usage telemetry', error));
}

function broadcastMahayanaEvent(event) {
  noteRuntimeUsage(event);
  if (event?.type === 'remoteComputer.changed') {
    broadcastNativeEvent('remote-desktop-user-presence', { action: event.action, data: event.data ?? null });
  }
  if (event?.type === 'mcp.oauth' && !event.removed && !event.authorizationUrl) {
    broadcastNativeEvent('mcp-auth-completed', { server: event.server, completedAtMs: Date.now() });
  }
  if (!mahayanaEdgeServer) return;
  for (const win of BrowserWindow.getAllWindows()) {
    if (win.isDestroyed() || win.webContents.isDestroyed()) continue;
    mahayanaEdgeServer.emit(win.webContents, 'runtime-event', event);
  }
  if (event?.type === 'settings.changed'
      && ((typeof event.settings?.inferenceProvider === 'string'
        && event.settings.inferenceProvider !== host.activeInferenceProvider)
        || (typeof event.settings?.sandboxRuntime === 'string'
          && event.settings.sandboxRuntime !== host.activeSandboxRuntime))) {
    setImmediate(() => {
      try { host.restart('inference Router settings changed'); }
      catch (error) { console.error('[mahayana-edge] Router restart failed', error); }
    });
  }
}

function startHostEventPump() {
  if (hostEventPump) return;
  hostEventPumpStopped = false;
  hostEventPump = (async () => {
    while (!hostEventPumpStopped) {
      try {
        const event = await host.request('feature.receive', { timeoutMs: 500 });
        if (event) broadcastMahayanaEvent(event);
        // Yield after every receive, including a non-empty event. The Rust Host
        // processes requests serially; immediately enqueueing the next long-poll
        // from this Promise continuation can starve renderer IPC such as auth.
        await sleep(10);
      } catch (error) {
        if (hostEventPumpStopped) break;
        console.error('[mahayana-edge] runtime event pump failed', error);
        await sleep(100);
      }
    }
  })().finally(() => {
    hostEventPump = null;
  });
}

function installIpcHandlers() {
  installMahayanaEdge();
  installNativeEdge();

  ipcMain.handle('fabushi:pick-file', async (event) => {
    assertTrustedSender(event);
    const result = await dialog.showOpenDialog({ properties: ['openFile'] });
    return result.canceled ? null : result.filePaths[0] || null;
  });

  ipcMain.handle('fabushi:notify', async (event, payload) => {
    assertTrustedSender(event);
    const title = String(payload?.title || '').trim();
    const body = String(payload?.body || '');
    if (!title || title.length > 160 || body.length > 4000) throw new Error('Invalid notification content.');
    new Notification({ title, body }).show();
    return true;
  });

  ipcMain.handle('fabushi:open-external', async (event, payload) => {
    assertTrustedSender(event);
    await shell.openExternal(safeHttpsUrl(payload?.url));
    return true;
  });

  ipcMain.handle('fabushi:window-focused', async (event) => {
    assertTrustedSender(event);
    return BrowserWindow.fromWebContents(event.sender)?.isFocused() ?? false;
  });

  ipcMain.handle('fabushi:open-system-settings', async (event, payload) => {
    assertTrustedSender(event);
    const pane = String(payload?.pane || '');
    if (!['screen-recording', 'accessibility'].includes(pane)) {
      throw new Error(`Unsupported system settings pane: ${pane}`);
    }
    if (process.platform === 'darwin') {
      const url = pane === 'screen-recording'
        ? 'x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture'
        : 'x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility';
      await shell.openExternal(url);
      return true;
    }
    if (process.platform === 'win32') {
      await shell.openExternal('ms-settings:privacy');
      return true;
    }
    throw new Error('System privacy settings must be opened manually on this platform.');
  });
}

function createWindow() {
  if (mainWindow && !mainWindow.isDestroyed()) return mainWindow;
  const win = new BrowserWindow({
    title: '全球法布施',
    width: 1180,
    height: 840,
    minWidth: 880,
    minHeight: 640,
    show: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      webSecurity: true,
      allowRunningInsecureContent: false,
      // Remote presence, WebRTC signaling, and semantic computer-use polling
      // must continue when the user closes (hides) the desktop window.
      backgroundThrottling: false,
    },
  });
  mainWindow = win;

  win.webContents.setWindowOpenHandler(({ url }) => {
    try { void shell.openExternal(safeHttpsUrl(url)); } catch {}
    return { action: 'deny' };
  });
  win.webContents.on('will-navigate', (event, url) => {
    if (!isTrustedRendererUrl(url)) event.preventDefault();
  });
  const publishWindowState = () => broadcastNativeEvent('window-state', describeWindow(win));
  for (const eventName of ['focus', 'blur', 'minimize', 'restore', 'maximize', 'unmaximize', 'enter-full-screen', 'leave-full-screen']) {
    win.on(eventName, publishWindowState);
  }
  win.on('close', (event) => {
    if (quitting || !backgroundPersistenceEnabled) return;
    event.preventDefault();
    win.hide();
    publishWindowState();
  });
  win.on('closed', () => {
    if (mainWindow === win) mainWindow = null;
    deepLinkRouter.markNotReady();
  });
  win.webContents.on('zoom-changed', () => {
    broadcastNativeEvent('zoom-factor-changed', { factor: win.webContents.getZoomFactor() });
  });
  win.once('ready-to-show', () => {
    win.show();
    publishWindowState();
    broadcastNativeEvent('zoom-factor-changed', { factor: win.webContents.getZoomFactor() });
    void readNativeState().then((state) => {
      const migration = state.preferences?.computeMigrationStatus ?? { required: false, status: 'complete', provider: 'mahayana' };
      broadcastNativeEvent('compute-migration', migration);
    }).catch(() => undefined);
  });

  if (process.env.VITE_DEV_SERVER_URL) {
    void win.loadURL(process.env.VITE_DEV_SERVER_URL);
  } else {
    void win.loadURL('app://bundle/index.html');
  }
  return win;
}

function installBackgroundTray() {
  if (!backgroundPersistenceEnabled || backgroundTray) return;
  const iconPath = path.resolve(__dirname, '..', 'resources', 'icon.png');
  let image = nativeImage.createFromPath(iconPath);
  if (image.isEmpty()) {
    // Packaged builds may not carry the builder source icon inside app.asar.
    // Keep a dependency-free embedded fallback so tray persistence works on
    // every target even when only dist/** and electron/** are packaged.
    const fallbackSvg = '<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><rect x="2" y="2" width="28" height="28" rx="9" fill="#8b5cf6"/><path d="M9 11h14v9H9zm4 11h6v2h-6z" fill="white"/></svg>';
    image = nativeImage.createFromDataURL(`data:image/svg+xml;base64,${Buffer.from(fallbackSvg).toString('base64')}`);
  }
  if (process.platform === 'darwin' && !image.isEmpty()) {
    image = image.resize({ width: 18, height: 18 });
    image.setTemplateImage(true);
  }
  if (image.isEmpty()) {
    console.warn('[desktop] background tray icon could not be created; continuing without a tray');
    return;
  }
  try {
    const tray = new Tray(image);
    tray.setToolTip('Fabushi · 电脑连接在后台运行');
    tray.setContextMenu(Menu.buildFromTemplate([
      { label: '显示 Fabushi', click: focusMainWindow },
      { label: '电脑连接在后台运行', enabled: false },
      { type: 'separator' },
      { label: '退出 Fabushi', click: requestApplicationQuit },
    ]));
    tray.on('click', focusMainWindow);
    tray.on('double-click', focusMainWindow);
    backgroundTray = tray;
  } catch (cause) {
    // Some minimal Linux desktop sessions do not provide a status notifier or
    // system tray. Presence must remain available rather than crashing the Host;
    // relaunching the single-instance app still restores the hidden window.
    backgroundTray = null;
    console.warn('[desktop] background tray is unavailable; continuing in background mode', cause);
  }
}

function installAutoUpdaterEvents() {
  if (!autoUpdater?.on) return;
  autoUpdater.autoDownload = false;
  autoUpdater.autoInstallOnAppQuit = true;
  autoUpdater.allowPrerelease = false;
  autoUpdater.on('checking-for-update', () => {
    const status = { type: 'checking', version: app.getVersion() };
    void setDesktopUpdateStatus(status);
  });
  autoUpdater.on('update-not-available', (info) => {
    availableDesktopUpdateVersion = null;
    const status = { type: 'upToDate', version: info?.version ?? app.getVersion() };
    void setDesktopUpdateStatus(status);
  });
  autoUpdater.on('update-available', (info) => {
    availableDesktopUpdateVersion = info?.version ?? app.getVersion();
    const status = { type: 'available', version: availableDesktopUpdateVersion, notes: typeof info?.releaseNotes === 'string' ? info.releaseNotes : undefined };
    void setDesktopUpdateStatus(status);
  });
  autoUpdater.on('download-progress', (progress) => {
    const status = { type: 'downloading', version: availableDesktopUpdateVersion ?? app.getVersion(), progress: Number.isFinite(progress?.percent) ? Math.round(progress.percent) : undefined };
    void setDesktopUpdateStatus(status);
  });
  autoUpdater.on('update-downloaded', (info) => {
    availableDesktopUpdateVersion = info?.version ?? availableDesktopUpdateVersion ?? app.getVersion();
    const status = { type: 'ready', version: availableDesktopUpdateVersion };
    void setDesktopUpdateStatus(status);
  });
  autoUpdater.on('error', (error) => {
    const status = { type: 'error', message: error instanceof Error ? error.message : String(error) };
    void setDesktopUpdateStatus(status);
  });
}

async function checkForDesktopUpdateAutomatically({ force = false } = {}) {
  if (!app.isPackaged || !autoUpdater?.checkForUpdates) return null;
  const now = Date.now();
  if (!force && now - lastAutomaticDesktopUpdateCheckAt < DESKTOP_UPDATE_CHECK_MIN_INTERVAL_MS) return null;
  if (automaticDesktopUpdateCheckPromise) return automaticDesktopUpdateCheckPromise;
  lastAutomaticDesktopUpdateCheckAt = now;
  automaticDesktopUpdateCheckPromise = autoUpdater.checkForUpdates()
    .catch((error) => {
      console.warn('[updater] automatic update check failed', error instanceof Error ? error.message : String(error));
      return null;
    })
    .finally(() => { automaticDesktopUpdateCheckPromise = null; });
  return automaticDesktopUpdateCheckPromise;
}

function installAutomaticDesktopUpdateChecks() {
  if (!app.isPackaged) return;
  const startupTimer = setTimeout(() => { void checkForDesktopUpdateAutomatically({ force: true }); }, 4_000);
  startupTimer.unref?.();
  app.on('browser-window-focus', () => { void checkForDesktopUpdateAutomatically(); });
  automaticDesktopUpdateCheckTimer = setInterval(() => {
    const hasFocusedWindow = BrowserWindow.getAllWindows().some((win) => !win.isDestroyed() && win.isFocused());
    if (hasFocusedWindow) void checkForDesktopUpdateAutomatically();
  }, DESKTOP_UPDATE_FOREGROUND_INTERVAL_MS);
  automaticDesktopUpdateCheckTimer.unref?.();
}

function installApplicationMenu() {
  const send = (eventName, payload = {}) => broadcastNativeEvent(eventName, { ...payload, source: 'menu' });
  const appMenu = process.platform === 'darwin'
    ? [{
        label: app.name,
        submenu: [
          { label: '关于 Fabushi', click: () => send('open-about') },
          { type: 'separator' },
          { label: '发送反馈', click: () => send('open-feedback') },
          { label: 'Widget Gallery', click: () => send('widget-gallery') },
          { type: 'separator' },
          { role: 'services' },
          { type: 'separator' },
          { role: 'hide' },
          { role: 'hideOthers' },
          { role: 'unhide' },
          { type: 'separator' },
          { role: 'quit' },
        ],
      }]
    : [];
  const template = [
    ...appMenu,
    {
      label: 'Fabushi',
      submenu: [
        { label: '重新开始引导', click: () => send('force-onboarding') },
        { label: '跳过引导', click: () => send('skip-onboarding') },
        { type: 'separator' },
        { label: '发送反馈', click: () => send('open-feedback') },
        { label: '关于', click: () => send('open-about') },
        ...(process.platform === 'darwin' ? [] : [
          { type: 'separator' },
          { label: '退出 Fabushi', accelerator: 'CmdOrCtrl+Q', click: requestApplicationQuit },
        ]),
      ],
    },
    {
      label: '查看',
      submenu: [
        { role: 'reload' },
        { role: 'forceReload' },
        { type: 'separator' },
        { role: 'resetZoom' },
        { role: 'zoomIn' },
        { role: 'zoomOut' },
        { type: 'separator' },
        { role: 'togglefullscreen' },
      ],
    },
    {
      label: '工具',
      submenu: [
        { label: 'Widget Gallery', click: () => send('widget-gallery') },
        { label: 'Offline ASR', click: () => broadcastNativeEvent('open-offline-asr', { source: 'menu' }) },
        { label: '设置', accelerator: 'CmdOrCtrl+,', click: () => deepLinkRouter.dispatch({ version: 1, route: 'settings', section: 'general', source: 'menu', canonicalUrl: 'fabushi://settings/general' }) },
      ],
    },
  ];
  Menu.setApplicationMenu(Menu.buildFromTemplate(template));
}

function messagingBlobRoot() {
  return path.join(app.getPath('userData'), 'feature-host', 'runtime', 'agents', '_messaging', 'blobs');
}

function parseMessagingBlobId(requestUrl) {
  const url = new URL(requestUrl);
  const id = decodeURIComponent(url.hostname || url.pathname.replace(/^\/+/, ''));
  if (!id || id.length > 128 || !/^[A-Za-z0-9._-]+$/.test(id) || id === '.' || id === '..') {
    throw new Error('Invalid Fabushi blob id.');
  }
  return id;
}

function parseSingleRange(rangeHeader, size) {
  if (!rangeHeader) return null;
  const match = /^bytes=(\d*)-(\d*)$/.exec(String(rangeHeader).trim());
  if (!match) return { invalid: true };
  let start;
  let end;
  if (!match[1] && !match[2]) return { invalid: true };
  if (!match[1]) {
    const suffix = Number(match[2]);
    if (!Number.isSafeInteger(suffix) || suffix <= 0) return { invalid: true };
    start = Math.max(0, size - suffix);
    end = size - 1;
  } else {
    start = Number(match[1]);
    end = match[2] ? Number(match[2]) : size - 1;
  }
  if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start < 0 || start >= size || end < start) {
    return { invalid: true };
  }
  return { start, end: Math.min(end, size - 1) };
}

async function handleMessagingBlobRequest(request) {
  if (!['GET', 'HEAD'].includes(request.method)) return new Response('Method not allowed', { status: 405 });
  let id;
  try { id = parseMessagingBlobId(request.url); } catch { return new Response('Bad blob id', { status: 400 }); }
  const root = messagingBlobRoot();
  const blobPath = path.join(root, `${id}.blob`);
  const metadataPath = path.join(root, `${id}.json`);
  let metadata;
  let stat;
  try {
    const [rawMetadata, fileStat] = await Promise.all([fs.readFile(metadataPath, 'utf8'), fs.stat(blobPath)]);
    metadata = JSON.parse(rawMetadata);
    stat = fileStat;
  } catch (error) {
    if (error?.code === 'ENOENT') return new Response('Not found', { status: 404 });
    console.error('[messaging-blob] failed to read blob metadata', error);
    return new Response('Blob unavailable', { status: 500 });
  }
  const expectedId = typeof metadata?.id === 'string' ? metadata.id : '';
  const mimeType = String(metadata?.mimeType || 'application/octet-stream');
  if (expectedId !== id || !/^[-\w.+]+\/[-\w.+]+$/.test(mimeType)) {
    return new Response('Invalid blob metadata', { status: 500 });
  }
  const size = Number(stat.size);
  const range = parseSingleRange(request.headers.get('range'), size);
  if (range?.invalid) {
    return new Response(null, { status: 416, headers: { 'Content-Range': `bytes */${size}` } });
  }
  const start = range?.start ?? 0;
  const end = range?.end ?? Math.max(0, size - 1);
  const length = size === 0 ? 0 : end - start + 1;
  const headers = new Headers({
    'Content-Type': mimeType,
    'Accept-Ranges': 'bytes',
    'Content-Length': String(length),
    'Cache-Control': 'private, max-age=31536000, immutable',
    'X-Content-Type-Options': 'nosniff',
  });
  if (range) headers.set('Content-Range', `bytes ${start}-${end}/${size}`);
  if (request.method === 'HEAD' || size === 0) {
    return new Response(null, { status: range ? 206 : 200, headers });
  }
  const nodeStream = fsSync.createReadStream(blobPath, { start, end });
  return new Response(Readable.toWeb(nodeStream), { status: range ? 206 : 200, headers });
}

function installAppProtocol() {
  const distRoot = path.resolve(__dirname, '..', 'dist');
  protocol.handle('app', (request) => {
    const requested = new URL(request.url);
    if (requested.hostname !== 'bundle') return new Response('Not found', { status: 404 });
    const pathname = decodeURIComponent(requested.pathname === '/' ? '/index.html' : requested.pathname);
    const resolved = path.resolve(distRoot, `.${pathname}`);
    const withinRoot = resolved === distRoot || resolved.startsWith(`${distRoot}${path.sep}`);
    if (!withinRoot) return new Response('Forbidden', { status: 403 });
    return net.fetch(pathToFileURL(resolved).toString());
  });
  protocol.handle('fabushi-blob', handleMessagingBlobRequest);
}

applyStartupNativePreferences();

app.whenReady().then(async () => {
  installAppProtocol();
  installApplicationMenu();
  installAutoUpdaterEvents();
  if (primaryInstance && app.isPackaged) app.setAsDefaultProtocolClient('fabushi');
  session.defaultSession.setPermissionRequestHandler((_webContents, _permission, callback) => callback(false));
  installIpcHandlers();
  await startAppAgentSurfaceServer().catch((error) => {
    console.error('[app-agent-surface] failed to start', error);
  });
  host.start();
  remoteDeviceAgentSupervisor = new RemoteDeviceAgentSupervisor({ host, app });
  remoteDeviceAgentSupervisor.start();
  installBackgroundTray();
  createWindow();
  startHostEventPump();
  installAutomaticDesktopUpdateChecks();
  app.on('activate', () => {
    focusMainWindow();
    void checkForDesktopUpdateAutomatically();
  });
});

app.on('window-all-closed', () => {
  deepLinkRouter.markNotReady();
  // Production keeps the Host and account-bound computer presence alive after
  // the window is hidden. Automated user journeys opt into a deterministic
  // shutdown seam so Playwright can close each isolated application cleanly.
  if (!backgroundPersistenceEnabled) app.quit();
});
app.on('before-quit', (event) => {
  quitting = true;
  if (automaticDesktopUpdateCheckTimer) clearInterval(automaticDesktopUpdateCheckTimer);
  automaticDesktopUpdateCheckTimer = null;
  hostEventPumpStopped = true;
  mahayanaEdgeServer?.dispose();
  mahayanaEdgeServer = null;
  nativeEdgeServer?.dispose();
  nativeEdgeServer = null;
  messagingSignalingClient?.disconnect('app_quit');
  messagingSignalingClient = null;
  messagingAccessCache.clear();
  remoteDeviceAgentSupervisor?.close();
  remoteDeviceAgentSupervisor = null;
  const closingAppAgentSurface = appAgentSurfaceServer;
  appAgentSurfaceServer = null;
  if (closingAppAgentSurface && !appAgentSurfaceShutdownComplete) {
    event.preventDefault();
    if (!appAgentSurfaceShutdownPending) {
      appAgentSurfaceShutdownPending = true;
      void closingAppAgentSurface.close()
        .catch((error) => {
          console.error('[app-agent-surface] shutdown failed', error);
        })
        .finally(() => {
          appAgentSurfaceShutdownPending = false;
          appAgentSurfaceShutdownComplete = true;
          app.quit();
        });
    }
  }
  backgroundTray?.destroy();
  backgroundTray = null;
  host.close();
});
