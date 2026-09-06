'use strict';

const { inspectOfflineAsr, downloadOfflineAsrModel, transcribeOfflineAudio } = require('./offline-asr.cjs');

const fs = require('node:fs/promises');
const path = require('node:path');
const crypto = require('node:crypto');
const os = require('node:os');

const MAX_TEXT_BYTES = 5 * 1024 * 1024;
const MAX_BINARY_BYTES = 32 * 1024 * 1024;
const MAX_DOWNLOAD_BYTES = 64 * 1024 * 1024;
const MAX_DIAGNOSTIC_BYTES = 256 * 1024;
const SENSITIVE_KEY = /(secret|token|password|authorization|cookie|credential|private.?key)/i;
const LOCAL_TOOL_PERMISSIONS = new Set(['never', 'ask', 'always']);
const UPDATE_TRACKS = new Set(['stable', 'beta', 'alpha']);
const GLOBAL_DHARMA_ID = 'global-dharma';
const LOCAL_PRAYER_WHEEL_CAPABILITY = 'local.prayer-wheel.start';
const LOCAL_PRAYER_WHEEL_LIFETIME_SKU = 'local-prayer-wheel.lifetime';
const LOCAL_PRAYER_WHEEL_LIFETIME_AMOUNT = 108000;
const LOCAL_PRAYER_WHEEL_CURRENCY = 'CNY';
const DEFAULT_DOCKER_IMAGE = 'mcr.microsoft.com/devcontainers/base:ubuntu24.04@sha256:c5cc2b45afe06a1df3aba17e58ba0dc4a02b999493198dab37dd0ccd4e2b0705';

function cleanString(value, limit = 4096) {
  return String(value ?? '').replace(/\0/g, '').trim().slice(0, limit);
}

function isPinnedContainerImage(value) {
  return /^[^\s@]+@sha256:[a-fA-F0-9]{64}$/.test(cleanString(value, 1000));
}

function safeFileName(value, fallback = 'attachment.bin') {
  const base = path.basename(cleanString(value, 220) || fallback);
  const sanitized = base.replace(/[<>:"/\\|?*\x00-\x1f]/g, '_').replace(/^\.+/, '').trim();
  return sanitized || fallback;
}

function mimeForPath(filePath) {
  switch (path.extname(filePath).toLowerCase()) {
    case '.png': return 'image/png';
    case '.jpg': case '.jpeg': return 'image/jpeg';
    case '.webp': return 'image/webp';
    case '.gif': return 'image/gif';
    case '.svg': return 'image/svg+xml';
    case '.pdf': return 'application/pdf';
    case '.json': return 'application/json';
    case '.md': case '.markdown': return 'text/markdown';
    case '.txt': case '.log': case '.csv': case '.tsv': return 'text/plain';
    case '.mp3': return 'audio/mpeg';
    case '.wav': return 'audio/wav';
    case '.m4a': return 'audio/mp4';
    case '.mp4': return 'video/mp4';
    default: return 'application/octet-stream';
  }
}

function redact(value, depth = 0) {
  if (depth > 6) return '[truncated]';
  if (value == null || typeof value === 'number' || typeof value === 'boolean') return value;
  if (typeof value === 'string') return value.length > 4000 ? `${value.slice(0, 4000)}…` : value;
  if (Array.isArray(value)) return value.slice(0, 100).map((item) => redact(item, depth + 1));
  if (typeof value !== 'object') return String(value);
  const out = {};
  for (const [key, item] of Object.entries(value).slice(0, 100)) {
    out[key] = SENSITIVE_KEY.test(key) ? '[redacted]' : redact(item, depth + 1);
  }
  return out;
}

function xmlEscape(value) {
  return String(value).replace(/[&<>"']/g, (character) => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&apos;',
  })[character]);
}

function dataUrl(mimeType, bytes) {
  return `data:${mimeType};base64,${bytes.toString('base64')}`;
}

function requestId(prefix) {
  return `${prefix}-${Date.now()}-${crypto.randomBytes(4).toString('hex')}`;
}

function ensureWithin(root, candidate) {
  const normalizedRoot = path.resolve(root);
  const normalized = path.resolve(candidate);
  if (normalized !== normalizedRoot && !normalized.startsWith(`${normalizedRoot}${path.sep}`)) {
    throw new Error('Path escaped the managed desktop storage root.');
  }
  return normalized;
}

async function readLimitedFile(filePath, maxBytes) {
  const stat = await fs.stat(filePath);
  if (!stat.isFile()) throw new Error('Requested path is not a file.');
  if (stat.size > maxBytes) throw new Error(`File exceeds ${maxBytes} byte limit.`);
  return { stat, bytes: await fs.readFile(filePath) };
}

function recordValue(value) {
  return value && typeof value === 'object' && !Array.isArray(value) ? value : null;
}

function miniAppEntitlementPath(pluginId, capability) {
  return `/v1/plugins/${encodeURIComponent(pluginId)}/entitlements/${encodeURIComponent(capability)}`;
}

function normalizedMiniAppId(value) {
  const id = cleanString(value, 200).toLowerCase();
  if (!/^[a-z0-9][a-z0-9-]{1,63}$/.test(id)) throw new Error('Invalid Mini App id.');
  return id;
}

function normalizedCapability(value) {
  const capability = cleanString(value, 200);
  if (!/^[A-Za-z0-9][A-Za-z0-9._:-]{1,199}$/.test(capability)) throw new Error('Invalid capability.');
  return capability;
}

function lifetimePrayerWheelOption(entitlement) {
  const options = Array.isArray(entitlement?.purchaseOptions) ? entitlement.purchaseOptions : [];
  const option = options.find((candidate) => candidate?.sku === LOCAL_PRAYER_WHEEL_LIFETIME_SKU);
  if (!option) throw new Error('Fabushi Pay did not return the lifetime local prayer-wheel product.');
  if (option.productId !== 'prod.global-dharma.local-prayer-wheel.lifetime'
    || option.productKind !== 'digital_durable'
    || option.currency !== LOCAL_PRAYER_WHEEL_CURRENCY
    || Number(option.amount) !== LOCAL_PRAYER_WHEEL_LIFETIME_AMOUNT) {
    throw new Error('Fabushi Pay lifetime local prayer-wheel product does not match the canonical CNY 1080 contract.');
  }
  const activeRails = Array.isArray(option.activeRails) ? option.activeRails.map((value) => String(value)) : [];
  if (!activeRails.includes('web_provider')) {
    throw new Error('The CNY 1080 lifetime local prayer-wheel web payment rail is not active.');
  }
  return option;
}

function paymentIdFromIntent(intent) {
  return cleanString(intent?.payment?.paymentId ?? intent?.paymentId, 160);
}

function safeCheckoutRedirect(checkout) {
  const url = cleanString(checkout?.checkoutAction?.url, 4096);
  if (!url) return null;
  let parsed;
  try { parsed = new URL(url); } catch { throw new Error('Fabushi Pay returned an invalid checkout URL.'); }
  if (parsed.protocol !== 'https:' || parsed.username || parsed.password) {
    throw new Error('Fabushi Pay checkout URL must be HTTPS and credential-free.');
  }
  return parsed.toString();
}

function createNativeCapabilityHandlers(deps) {
  const {
    app,
    autoUpdater,
    dialog,
    net,
    safeStorage,
    shell,
    host,
    readNativeState,
    mutateNativeState,
    getDesktopUpdateStatus,
    setDesktopUpdateStatus,
    windowForEvent,
    broadcastNativeEvent,
    clearAccountBoundMessagingState,
    setDesktopUpdateInstallInProgress,
  } = deps;

  const telemetryPath = () => path.join(app.getPath('userData'), 'diagnostics', 'native-events.ndjson');
  const feedbackPath = () => path.join(app.getPath('userData'), 'feedback', 'feedback.ndjson');
  const secretPath = () => path.join(app.getPath('userData'), 'secure', 'secrets.json');
  const stagedRoot = () => path.join(app.getPath('userData'), 'attachments', 'staged');
  const committedRoot = () => path.join(app.getPath('userData'), 'attachments', 'committed');

  async function appendJsonLine(target, record) {
    await fs.mkdir(path.dirname(target), { recursive: true });
    let line = JSON.stringify(record);
    if (Buffer.byteLength(line) > MAX_DIAGNOSTIC_BYTES) {
      line = JSON.stringify({
        type: record.type ?? 'oversized',
        timestamp: record.timestamp ?? new Date().toISOString(),
        truncated: true,
      });
    }
    await fs.appendFile(target, `${line}\n`, { mode: 0o600 });
  }

  async function report(type, params) {
    const record = {
      type,
      timestamp: new Date().toISOString(),
      payload: redact(params),
    };
    await appendJsonLine(telemetryPath(), record);
    return { recorded: true, type, timestamp: record.timestamp };
  }

  async function getPreference(key, fallback = null) {
    const state = await readNativeState();
    return state.preferences?.[key] ?? fallback;
  }

  async function setPreference(key, value) {
    await mutateNativeState((state) => ({
      ...state,
      preferences: { ...(state.preferences ?? {}), [key]: value },
    }));
    return value;
  }

  async function featureExecute(command) {
    return host.request('feature.execute', { command });
  }

  async function platformRequest(method, requestPath, options = {}) {
    const response = await host.request('platform.request', {
      method,
      path: requestPath,
      authenticated: options.authenticated !== false,
      ...(options.query ? { query: options.query } : {}),
      ...(options.body !== undefined ? { body: options.body } : {}),
    });
    if (!response || response.ok !== true) {
      const detail = response?.data?.message ?? response?.bodyText ?? `HTTP ${response?.statusCode ?? 'unknown'}`;
      throw new Error(`Fabushi platform request failed: ${String(detail).slice(0, 1000)}`);
    }
    return response.data ?? null;
  }

  async function loadSecretVault() {
    try {
      const parsed = JSON.parse(await fs.readFile(secretPath(), 'utf8'));
      return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
    } catch (error) {
      if (error?.code === 'ENOENT') return {};
      throw error;
    }
  }

  async function saveSecretVault(vault) {
    await fs.mkdir(path.dirname(secretPath()), { recursive: true });
    const temp = `${secretPath()}.tmp`;
    await fs.writeFile(temp, `${JSON.stringify(vault, null, 2)}\n`, { mode: 0o600 });
    await fs.rename(temp, secretPath());
  }

  function vaultSecretConfigured(vault, name) {
    const ciphertext = vault?.[name]?.ciphertext;
    if (typeof ciphertext !== 'string' || !ciphertext || !safeStorage?.isEncryptionAvailable?.()) return false;
    try {
      const value = safeStorage.decryptString(Buffer.from(ciphertext, 'base64'));
      return Boolean(value) && !/[\r\n]/.test(value);
    } catch {
      return false;
    }
  }

  async function isRegularFile(filePath) {
    if (!filePath) return false;
    try { return (await fs.stat(filePath)).isFile(); } catch { return false; }
  }

  async function inspectCodexAuth(filePath) {
    if (!filePath) return { authenticated: false, reason: 'missing' };
    try {
      const metadata = await fs.lstat(filePath);
      if (!metadata.isFile() || metadata.isSymbolicLink()) {
        return { authenticated: false, reason: 'unsafe-file-type' };
      }
      if (process.platform !== 'win32' && (metadata.mode & 0o077) !== 0) {
        return { authenticated: false, reason: 'unsafe-permissions' };
      }
      if (metadata.size > 1024 * 1024) return { authenticated: false, reason: 'oversized' };
      const value = JSON.parse(await fs.readFile(filePath, 'utf8'));
      const apiKey = cleanString(value?.OPENAI_API_KEY, 16);
      const accessToken = cleanString(value?.tokens?.access_token, 16);
      const idToken = cleanString(value?.tokens?.id_token, 16);
      const authenticated = Boolean(apiKey || accessToken || idToken);
      return { authenticated, reason: authenticated ? 'credential-present' : 'credential-missing' };
    } catch {
      return { authenticated: false, reason: 'unreadable' };
    }
  }

  async function firstAvailableExecutable(explicitPath, command) {
    const suffix = process.platform === 'win32' ? '.exe' : '';
    const candidates = [
      cleanString(explicitPath, 4096),
      ...(process.env.PATH ?? '').split(path.delimiter).filter(Boolean).map((directory) => path.join(directory, `${command}${suffix}`)),
    ].filter(Boolean);
    for (const candidate of candidates) {
      if (await isRegularFile(candidate)) return candidate;
    }
    return null;
  }

  function requireSecretEncryption() {
    if (!safeStorage?.isEncryptionAvailable?.()) {
      throw new Error('OS-backed secret encryption is not available on this device.');
    }
  }

  async function installedPluginPointers() {
    const catalog = await host.request('feature.marketplace.browse', { query: '', platform: 'desktop' }).catch(() => []);
    const entries = Array.isArray(catalog) ? catalog : Array.isArray(catalog?.plugins) ? catalog.plugins : [];
    const pointers = [];
    for (const entry of entries.slice(0, 200)) {
      const pluginId = cleanString(entry?.id ?? entry?.pluginId, 200);
      if (!pluginId) continue;
      const pointer = await host.request('feature.plugin.active', { pluginId }).catch(() => null);
      if (pointer) pointers.push({ pluginId, ...pointer });
    }
    return pointers;
  }

  async function currentUpdateStatus() {
    const state = await readNativeState();
    const status = typeof getDesktopUpdateStatus === 'function'
      ? await getDesktopUpdateStatus()
      : state.updateStatus ?? {
      type: 'upToDate',
      version: app.getVersion(),
    };
    return { ...status, track: status?.track ?? state.preferences?.updateTrack ?? 'stable' };
  }

  async function writeUpdateStatus(status) {
    if (typeof setDesktopUpdateStatus === 'function') return setDesktopUpdateStatus(status);
    await mutateNativeState((state) => ({ ...state, updateStatus: status }));
    broadcastNativeEvent('update-status', status);
    return status;
  }

  function waitForUpdateDownloaded(timeoutMs = 180_000) {
    if (!autoUpdater?.once) return Promise.resolve(null);
    return new Promise((resolve, reject) => {
      let timer = null;
      const cleanup = () => {
        if (timer) clearTimeout(timer);
        autoUpdater.removeListener?.('update-downloaded', onDownloaded);
        autoUpdater.removeListener?.('error', onError);
      };
      const onDownloaded = (info) => { cleanup(); resolve(info ?? null); };
      const onError = (error) => { cleanup(); reject(error instanceof Error ? error : new Error(String(error))); };
      autoUpdater.once('update-downloaded', onDownloaded);
      autoUpdater.once('error', onError);
      timer = setTimeout(() => {
        cleanup();
        reject(new Error('Timed out waiting for the desktop update to finish downloading.'));
      }, timeoutMs);
      timer.unref?.();
    });
  }

  const handlers = {
    async submitFeedback(params) {
      const id = crypto.randomUUID();
      const record = {
        id,
        createdAtMs: Date.now(),
        category: cleanString(params.category ?? 'general', 80),
        message: cleanString(params.message ?? params.text, 12000),
        context: redact(params.context ?? {}),
      };
      if (!record.message) throw new Error('Feedback message is required.');
      await appendJsonLine(feedbackPath(), record);
      return { id, stored: true, createdAtMs: record.createdAtMs };
    },

    setTitleBarOverlayTone(params, event) {
      const win = windowForEvent(event);
      if (typeof win.setTitleBarOverlay !== 'function') return { supported: false };
      const tone = cleanString(params.tone ?? params.preference ?? 'system', 20);
      const dark = tone === 'dark' || (tone === 'system' && deps.nativeTheme?.shouldUseDarkColors);
      const overlay = {
        color: cleanString(params.color, 32) || (dark ? '#111111' : '#ffffff'),
        symbolColor: cleanString(params.symbolColor, 32) || (dark ? '#ffffff' : '#111111'),
        height: Number.isFinite(Number(params.height)) ? Math.max(20, Math.min(80, Number(params.height))) : 32,
      };
      win.setTitleBarOverlay(overlay);
      return { supported: true, ...overlay };
    },

    async getHardwareAcceleration() {
      const enabled = await getPreference('hardwareAccelerationEnabled', true);
      return { enabled: enabled !== false, requiresRelaunch: false };
    },

    async setHardwareAccelerationEnabled(params) {
      const enabled = params.enabled !== false;
      await setPreference('hardwareAccelerationEnabled', enabled);
      return { enabled, requiresRelaunch: true };
    },

    async getEgressTunnelEnabled() {
      const providerAvailable = Boolean(process.env.FABUSHI_EGRESS_TUNNEL_URL?.trim());
      const requested = await getPreference('egressTunnelEnabled', false);
      return providerAvailable && requested === true;
    },

    async setEgressTunnelEnabled(params) {
      const providerAvailable = Boolean(process.env.FABUSHI_EGRESS_TUNNEL_URL?.trim());
      const enabled = params.enabled === true && providerAvailable;
      await setPreference('egressTunnelEnabled', enabled);
      broadcastNativeEvent('egress-tunnel-changed', enabled);
      broadcastNativeEvent('egress-tunnel-status-changed', await this.getEgressTunnelStatus());
      return enabled;
    },

    async getEgressTunnelStatus() {
      const configuredUrl = cleanString(process.env.FABUSHI_EGRESS_TUNNEL_URL, 4096);
      if (configuredUrl) {
        let parsed;
        try { parsed = new URL(configuredUrl); } catch { parsed = null; }
        if (parsed?.protocol === 'https:') {
          return {
            available: true,
            enabled: true,
            provider: 'configured-https-tunnel',
            transport: 'https',
            url: parsed.origin,
            agentEnabled: String(process.env.FABUSHI_EGRESS_AGENT_ENABLED ?? '').toLowerCase() === 'true',
          };
        }
      }
      try {
        const result = await platformRequest('GET', '/api/egress/status');
        return result?.egress ?? {
          available: false,
          enabled: false,
          provider: null,
          transport: null,
          agentEnabled: false,
          reason: 'Fabushi Platform returned no egress status.',
        };
      } catch (error) {
        return {
          available: false,
          enabled: false,
          provider: null,
          transport: null,
          agentEnabled: false,
          reason: error instanceof Error ? cleanString(error.message, 1000) : 'Fabushi Platform egress is unavailable.',
        };
      }
    },

    async egressFetch(params) {
      const url = cleanString(params.url, 4096);
      if (!url) throw new Error('Egress URL is required.');
      const method = cleanString(params.method, 16).toUpperCase() || 'GET';
      const headers = params.headers && typeof params.headers === 'object' && !Array.isArray(params.headers)
        ? params.headers
        : {};
      let bodyBase64 = cleanString(params.bodyBase64, 8 * 1024 * 1024);
      if (!bodyBase64 && typeof params.body === 'string') {
        bodyBase64 = Buffer.from(params.body, 'utf8').toString('base64');
      }
      const result = await platformRequest('POST', '/api/egress/fetch', {
        body: { url, method, headers, bodyBase64: bodyBase64 || null },
      });
      return result?.response ?? null;
    },

    async getWebauthnProxyEnabled() {
      const available = process.env.FABUSHI_WEBAUTHN_PROXY_AVAILABLE === '1';
      return available && await getPreference('webauthnProxyEnabled', false) === true;
    },

    async setWebauthnProxyEnabled(params) {
      const available = process.env.FABUSHI_WEBAUTHN_PROXY_AVAILABLE === '1';
      const enabled = params.enabled === true && available;
      await setPreference('webauthnProxyEnabled', enabled);
      broadcastNativeEvent('webauthn-proxy-changed', enabled);
      return enabled;
    },

    getUpdateStatus() {
      return currentUpdateStatus();
    },

    async checkForUpdates() {
      if (!app.isPackaged || !autoUpdater?.checkForUpdates) {
        return writeUpdateStatus({ type: 'upToDate', version: app.getVersion(), source: 'local-build' });
      }
      await writeUpdateStatus({ type: 'checking', version: app.getVersion() });
      try {
        await autoUpdater.checkForUpdates();
        return currentUpdateStatus();
      } catch (error) {
        return writeUpdateStatus({ type: 'error', message: error instanceof Error ? error.message : String(error) });
      }
    },

    async setUpdateTrack(params) {
      const track = cleanString(params.track ?? 'stable', 20);
      if (!UPDATE_TRACKS.has(track)) throw new Error('Unsupported update track.');
      await setPreference('updateTrack', track);
      return { ...(await currentUpdateStatus()), track };
    },

    async quitAndInstallUpdate(params) {
      let status = await currentUpdateStatus();
      const expected = cleanString(params.expectedVersion, 80);
      if (expected && status.version && expected !== status.version) throw new Error('Update version changed before install.');
      if (!autoUpdater?.quitAndInstall) {
        return { installed: false, reason: 'Desktop updater is unavailable.' };
      }
      if (status.type === 'available' || status.type === 'downloading' || status.type === 'staging') {
        if (!autoUpdater?.downloadUpdate) {
          return { installed: false, reason: 'Desktop updater cannot download this release.' };
        }
        const version = status.version ?? expected ?? app.getVersion();
        const downloaded = waitForUpdateDownloaded();
        await writeUpdateStatus({ type: 'downloading', version, progress: 0 });
        await Promise.all([autoUpdater.downloadUpdate(), downloaded]);
        status = await currentUpdateStatus();
        if (status.type !== 'ready') status = await writeUpdateStatus({ type: 'ready', version });
      }
      if (status.type !== 'ready') {
        return { installed: false, reason: 'No downloaded desktop update is ready.' };
      }
      const version = status.version ?? expected ?? app.getVersion();
      await writeUpdateStatus({ type: 'staging', version });
      if (typeof setDesktopUpdateInstallInProgress === 'function') {
        setDesktopUpdateInstallInProgress(true);
      }
      const installTimer = setTimeout(() => {
        try {
          // electron-updater's macOS adapter asks Squirrel.Mac to fetch the
          // staged ZIP here and Squirrel.Mac quits only after that fetch is
          // complete. Do not call app.quit() on macOS: an eager quit can close
          // the app before Squirrel has replaced the bundle. The main process
          // already marks this as an update shutdown so optional cleanup cannot
          // cancel the handshake. Other platforms keep the explicit fallback.
          autoUpdater.quitAndInstall(false, true);
          if (process.platform !== 'darwin') app.quit();
        } catch (error) {
          if (typeof setDesktopUpdateInstallInProgress === 'function') {
            setDesktopUpdateInstallInProgress(false);
          }
          void writeUpdateStatus({
            type: 'error',
            message: error instanceof Error ? error.message : String(error),
          });
        }
      }, 120);
      installTimer.unref?.();
      return { installed: true, version };
    },

    async setAutoUpdateWhenIdleOptIn(params) {
      return setPreference('autoUpdateWhenIdle', params.enabled === true);
    },

    async getComputeMigrationStatus() {
      const value = await getPreference('computeMigrationStatus', null);
      const status = value ?? { required: false, status: 'complete', provider: 'mahayana' };
      broadcastNativeEvent('compute-migration', status);
      return status;
    },

    async markDeepLinksReady() {
      await setPreference('deepLinksReady', true);
      return typeof deps.markDeepLinksReady === 'function'
        ? deps.markDeepLinksReady()
        : { ready: true, flushed: 0 };
    },

    getAutoReviewInstructions() {
      return getPreference('autoReviewInstructions', '');
    },

    async setAutoReviewInstructions(params) {
      return setPreference('autoReviewInstructions', cleanString(params.instructions, 20000));
    },

    getLocalToolPermission() {
      return getPreference('localToolPermission', 'ask');
    },

    async getLocalToolPermissionCeiling() {
      const ceiling = cleanString(process.env.FABUSHI_LOCAL_TOOL_PERMISSION_CEILING ?? 'always', 20);
      return LOCAL_TOOL_PERMISSIONS.has(ceiling) ? ceiling : 'always';
    },

    async setLocalToolPermission(params) {
      const permission = cleanString(params.permission ?? 'ask', 20);
      if (!LOCAL_TOOL_PERMISSIONS.has(permission)) throw new Error('Unsupported local tool permission.');
      const ceiling = await this.getLocalToolPermissionCeiling();
      const order = ['never', 'ask', 'always'];
      if (order.indexOf(permission) > order.indexOf(ceiling)) throw new Error('Permission exceeds administrator ceiling.');
      await setPreference('localToolPermission', permission);
      return permission;
    },

    async recordLocalToolApproval(params) {
      const approvalId = cleanString(params.approvalId, 160);
      if (!approvalId) throw new Error('Approval ID is required.');
      const entry = {
        approvalId,
        action: cleanString(params.action, 120),
        target: cleanString(params.target, 1000),
        recordedAtMs: Date.now(),
      };
      await mutateNativeState((state) => ({
        ...state,
        localToolApprovals: { ...(state.localToolApprovals ?? {}), [approvalId]: entry },
      }));
      return entry;
    },

    async clearLocalToolApprovals() {
      await mutateNativeState((state) => ({ ...state, localToolApprovals: {} }));
      return true;
    },

    async pickAvatarSource() {
      const result = await dialog.showOpenDialog({
        title: 'Choose avatar source',
        properties: ['openFile'],
        filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg'] }],
      });
      return result.canceled || !result.filePaths[0] ? null : { path: result.filePaths[0] };
    },

    async pickAvatarFile() {
      const result = await this.pickAvatarSource();
      if (!result) return null;
      const { bytes, stat } = await readLimitedFile(result.path, 8 * 1024 * 1024);
      const mimeType = mimeForPath(result.path);
      return { path: result.path, name: path.basename(result.path), sizeBytes: stat.size, mimeType, bytesBase64: bytes.toString('base64'), dataUrl: dataUrl(mimeType, bytes) };
    },

    async generateAgentAvatarImage(params) {
      const label = cleanString(params.prompt ?? params.name ?? params.agentId ?? 'Fabushi', 80) || 'Fabushi';
      const digest = crypto.createHash('sha256').update(label).digest();
      const hueA = digest.readUInt16BE(0) % 360;
      const hueB = (hueA + 40 + (digest[2] % 120)) % 360;
      const initials = label.split(/\s+/u).filter(Boolean).slice(0, 2).map((part) => [...part][0]).join('').toUpperCase().slice(0, 2) || 'F';
      const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop stop-color="hsl(${hueA} 72% 54%)"/><stop offset="1" stop-color="hsl(${hueB} 68% 38%)"/></linearGradient></defs><rect width="512" height="512" rx="144" fill="url(#g)"/><circle cx="256" cy="226" r="112" fill="rgba(255,255,255,.14)"/><text x="256" y="286" text-anchor="middle" font-family="system-ui,sans-serif" font-size="148" font-weight="700" fill="white">${xmlEscape(initials)}</text></svg>`;
      const bytes = Buffer.from(svg);
      return { mimeType: 'image/svg+xml', width: 512, height: 512, bytesBase64: bytes.toString('base64'), dataUrl: dataUrl('image/svg+xml', bytes), source: 'fabushi-local-generator' };
    },

    async resolveAttachmentMedia(params) {
      const filePath = path.resolve(cleanString(params.path ?? params.filePath, 4096));
      const { stat } = await readLimitedFile(filePath, MAX_BINARY_BYTES);
      return { path: filePath, name: path.basename(filePath), mimeType: mimeForPath(filePath), sizeBytes: stat.size, modifiedAtMs: stat.mtimeMs };
    },

    async readAttachmentText(params) {
      const filePath = path.resolve(cleanString(params.path ?? params.filePath, 4096));
      const { bytes, stat } = await readLimitedFile(filePath, MAX_TEXT_BYTES);
      return { path: filePath, text: bytes.toString(params.encoding === 'latin1' ? 'latin1' : 'utf8'), sizeBytes: stat.size, mimeType: mimeForPath(filePath) };
    },

    async readAttachmentBytes(params) {
      const filePath = path.resolve(cleanString(params.path ?? params.filePath, 4096));
      const { bytes, stat } = await readLimitedFile(filePath, MAX_BINARY_BYTES);
      return { path: filePath, bytesBase64: bytes.toString('base64'), sizeBytes: stat.size, mimeType: mimeForPath(filePath) };
    },

    async stageAttachmentBytes(params) {
      const raw = cleanString(params.bytesBase64, MAX_BINARY_BYTES * 2);
      const bytes = Buffer.from(raw, 'base64');
      if (!bytes.length || bytes.length > MAX_BINARY_BYTES) throw new Error('Attachment bytes are empty or exceed the limit.');
      const id = crypto.randomUUID();
      const name = safeFileName(params.filename ?? params.name);
      const dir = path.join(stagedRoot(), id);
      await fs.mkdir(dir, { recursive: true });
      const filePath = ensureWithin(dir, path.join(dir, name));
      await fs.writeFile(filePath, bytes, { mode: 0o600 });
      return { id, path: filePath, name, mimeType: cleanString(params.mimeType, 120) || mimeForPath(name), sizeBytes: bytes.length };
    },

    async downloadAttachment(params) {
      const url = new URL(cleanString(params.url, 4096));
      if (url.protocol !== 'https:') throw new Error('Only HTTPS attachments may be downloaded.');
      const response = await net.fetch(url.toString());
      if (!response.ok) throw new Error(`Attachment download failed with HTTP ${response.status}.`);
      const declared = Number(response.headers.get('content-length') ?? 0);
      if (declared > MAX_DOWNLOAD_BYTES) throw new Error('Attachment download exceeds the size limit.');
      const bytes = Buffer.from(await response.arrayBuffer());
      if (bytes.length > MAX_DOWNLOAD_BYTES) throw new Error('Attachment download exceeds the size limit.');
      const name = safeFileName(params.filename ?? path.basename(url.pathname), 'download.bin');
      const target = path.join(app.getPath('downloads'), name);
      await fs.writeFile(target, bytes);
      return { path: target, name, sizeBytes: bytes.length, mimeType: response.headers.get('content-type') || mimeForPath(name) };
    },

    async commitStagedAttachments(params) {
      const items = Array.isArray(params.items) ? params.items : Array.isArray(params.paths) ? params.paths.map((item) => ({ path: item })) : [];
      await fs.mkdir(committedRoot(), { recursive: true });
      const committed = [];
      for (const item of items.slice(0, 100)) {
        const source = ensureWithin(stagedRoot(), cleanString(item.path ?? item.filePath, 4096));
        const name = safeFileName(item.name ?? path.basename(source));
        const destination = ensureWithin(committedRoot(), path.join(committedRoot(), `${crypto.randomUUID()}-${name}`));
        await fs.rename(source, destination);
        const stat = await fs.stat(destination);
        committed.push({ path: destination, name, sizeBytes: stat.size, mimeType: mimeForPath(destination) });
      }
      return committed;
    },

    async discardStagedAttachment(params) {
      const target = ensureWithin(stagedRoot(), cleanString(params.path ?? params.filePath, 4096));
      await fs.rm(target, { recursive: true, force: true });
      return true;
    },

    async forceRecreateComputer() {
      const generation = Number(await getPreference('computerGeneration', 0)) + 1;
      await setPreference('computerGeneration', generation);
      const accepted = await featureExecute({ type: 'computer.status', requestId: requestId('computer-recreate') });
      const result = { dispatched: true, generation, accepted };
      broadcastNativeEvent('dev-compute-rebuild', { state: 'requested', generation, requestedAtMs: Date.now() });
      broadcastNativeEvent('update-computer-dispatched', result);
      return result;
    },

    async updateComputer(params) {
      const command = params.command && typeof params.command === 'object'
        ? params.command
        : { type: 'computer.status', requestId: requestId('computer-update') };
      if (!command.requestId) command.requestId = requestId('computer-update');
      const startedAtMs = Date.now();
      broadcastNativeEvent('dev-compute-pull-progress', { state: 'dispatching', progress: 0, startedAtMs });
      const accepted = await featureExecute(command);
      const result = { dispatched: true, accepted };
      broadcastNativeEvent('dev-compute-pull-progress', { state: 'dispatched', progress: 1, startedAtMs, completedAtMs: Date.now() });
      broadcastNativeEvent('update-computer-dispatched', result);
      return result;
    },

    async forceReconnectGateway() {
      const info = await host.request('feature.info', {});
      return { connected: true, checkedAtMs: Date.now(), info };
    },

    async getExperimentsSnapshot() {
      const state = await readNativeState();
      return { overrides: state.featureFlags ?? {}, refreshedAtMs: state.featureFlagsRefreshedAtMs ?? null };
    },

    async applyFeatureFlagOverride(params) {
      const key = cleanString(params.key ?? params.flag, 160);
      if (!key) throw new Error('Feature flag key is required.');
      await mutateNativeState((state) => ({ ...state, featureFlags: { ...(state.featureFlags ?? {}), [key]: params.value } }));
      const snapshot = await this.getExperimentsSnapshot();
      broadcastNativeEvent('experiments-changed', snapshot);
      return snapshot;
    },

    async refreshFeatureFlags() {
      await mutateNativeState((state) => ({ ...state, featureFlagsRefreshedAtMs: Date.now() }));
      const snapshot = await this.getExperimentsSnapshot();
      broadcastNativeEvent('experiments-changed', snapshot);
      return snapshot;
    },

    async startRpcTraceWindow() {
      const trace = { traceId: crypto.randomUUID(), startedAtMs: Date.now(), expiresAtMs: Date.now() + 60_000 };
      await setPreference('rpcTraceWindow', trace);
      return trace;
    },

    getAgentDefaultModel() {
      return getPreference('agentDefaultModel', 'auto');
    },

    async setAgentDefaultModel(params) {
      return setPreference('agentDefaultModel', cleanString(params.model ?? 'auto', 160) || 'auto');
    },

    getComputerUseModel() {
      return getPreference('computerUseModel', 'auto');
    },

    async setComputerUseModel(params) {
      return setPreference('computerUseModel', cleanString(params.model ?? 'auto', 160) || 'auto');
    },

    getHostPinnedAgents() {
      return getPreference('hostPinnedAgents', []);
    },

    async setHostPinnedAgents(params) {
      const ids = Array.isArray(params.agentIds ?? params.ids) ? (params.agentIds ?? params.ids).map((value) => cleanString(value, 160)).filter(Boolean) : [];
      return setPreference('hostPinnedAgents', [...new Set(ids)].slice(0, 100));
    },

    getHostSidebarSections() {
      return getPreference('hostSidebarSections', ['agents', 'groups', 'automations', 'skills']);
    },

    async setHostSidebarSections(params) {
      const sections = Array.isArray(params.sections) ? params.sections.map((value) => cleanString(value, 80)).filter(Boolean) : [];
      return setPreference('hostSidebarSections', [...new Set(sections)].slice(0, 30));
    },

    async getAvailableModels() {
      const configured = cleanString(process.env.FABUSHI_AVAILABLE_MODELS, 4096)
        .split(',').map((value) => value.trim()).filter(Boolean);
      const defaults = [await this.getAgentDefaultModel(), await this.getComputerUseModel(), 'auto'];
      let remote = [];
      try {
        const catalog = await platformRequest('GET', '/api/ai/models', { authenticated: false });
        remote = Array.isArray(catalog?.models)
          ? catalog.models.map((item) => cleanString(item?.id ?? item, 160)).filter(Boolean)
          : [];
      } catch {
        // Local/runtime defaults remain available when the product catalog cannot be reached.
      }
      return [...new Set([...remote, ...configured, ...defaults])];
    },

    async getOfflineAsrStatus() {
      const state = await readNativeState();
      const config = state.offlineAsrModel ?? {};
      return inspectOfflineAsr({ app, resourcesPath: process.resourcesPath, config });
    },

    async configureOfflineAsrModel(params) {
      const modelUrl = cleanString(params.modelUrl, 4096);
      const sha256 = cleanString(params.sha256, 128).toLowerCase();
      if (modelUrl && new URL(modelUrl).protocol !== 'https:') {
        throw new Error('Offline ASR model URL must use HTTPS.');
      }
      if (modelUrl && !/^[0-9a-f]{64}$/.test(sha256)) {
        throw new Error('Offline ASR model downloads require a verified SHA-256 digest.');
      }
      if (sha256 && !/^[0-9a-f]{64}$/.test(sha256)) {
        throw new Error('Offline ASR model SHA-256 must be a 64-character hexadecimal digest.');
      }
      const config = {
        ...(modelUrl ? { modelUrl } : {}),
        ...(sha256 ? { sha256 } : {}),
        updatedAtMs: Date.now(),
      };
      await mutateNativeState((state) => ({ ...state, offlineAsrModel: config }));
      return inspectOfflineAsr({ app, resourcesPath: process.resourcesPath, config });
    },

    async downloadOfflineAsrModel(params) {
      const state = await readNativeState();
      const config = {
        ...(state.offlineAsrModel ?? {}),
        ...(params.modelUrl ? { modelUrl: cleanString(params.modelUrl, 4096) } : {}),
        ...(params.sha256 ? { sha256: cleanString(params.sha256, 128).toLowerCase() } : {}),
      };
      const status = await downloadOfflineAsrModel({
        app,
        net,
        resourcesPath: process.resourcesPath,
        config,
        onProgress: (progress) => broadcastNativeEvent('offline-asr-progress', {
          phase: 'model-download',
          ...progress,
        }),
      });
      await mutateNativeState((current) => ({ ...current, offlineAsrModel: config }));
      broadcastNativeEvent('offline-asr-progress', { phase: 'ready', progress: 1, status });
      return status;
    },

    async transcribeAudio(params) {
      const inventory = await host.request('runtime.tools', {}).catch(() => null);
      const tools = Array.isArray(inventory?.tools) ? inventory.tools : Array.isArray(inventory) ? inventory : [];
      const transcriber = tools.find((tool) => /transcrib|speech.?to.?text|audio.?to.?text/i.test(String(tool?.name ?? tool?.id ?? '')));
      if (transcriber) {
        const toolName = cleanString(transcriber.name ?? transcriber.id, 240);
        if (toolName) {
          try {
            return await host.request('runtime.call', { tool: toolName, input: params });
          } catch (error) {
            console.warn('[native-edge] runtime transcriber failed; trying offline ASR', error);
          }
        }
      }
      const state = await readNativeState();
      broadcastNativeEvent('offline-asr-progress', { phase: 'transcribing', progress: 0 });
      const result = await transcribeOfflineAudio({
        app,
        resourcesPath: process.resourcesPath,
        config: state.offlineAsrModel ?? {},
        params,
      });
      broadcastNativeEvent('offline-asr-progress', {
        phase: result.available ? 'complete' : 'unavailable',
        progress: result.available ? 1 : 0,
        result,
      });
      return result;
    },

    async getAccountAuthStatus() {
      const auth = await host.request('feature.auth.status', {});
      const state = await readNativeState();
      return { ...auth, displayName: state.accountDisplayName ?? null, avatar: state.accountAvatar ?? null };
    },

    async loginAccount(params) {
      if (params.username != null || params.password != null) {
        const auth = await host.request('feature.auth.passwordLogin', { username: String(params.username ?? ''), password: String(params.password ?? '') });
        if (auth?.loggedIn === true || auth?.auth?.loggedIn === true) clearAccountBoundMessagingState?.();
        return auth;
      }
      const providers = await host.request('feature.auth.providers', {});
      const provider = cleanString(params.provider, 80) || (Array.isArray(providers) ? cleanString(providers[0]?.id ?? providers[0], 80) : '');
      if (!provider) throw new Error('No account login provider is available.');
      const attempt = await host.request('feature.auth.oauthStart', { provider });
      await setPreference('activeLoginAttempt', attempt?.attemptId ?? null);
      return attempt;
    },

    async cancelAccountLogin() {
      await setPreference('activeLoginAttempt', null);
      return { cancelled: true };
    },

    async logoutAccount() {
      const auth = await host.request('feature.auth.logout', {});
      clearAccountBoundMessagingState?.();
      broadcastNativeEvent('account-auth-changed', auth);
      return auth;
    },

    async updateAccountName(params) {
      const name = cleanString(params.name ?? params.displayName, 160);
      await mutateNativeState((state) => ({ ...state, accountDisplayName: name || null }));
      return name || null;
    },

    async getAccountAvatar() {
      const state = await readNativeState();
      return state.accountAvatar ?? null;
    },

    async getWeeklyUsage() {
      const state = await readNativeState();
      const entries = Array.isArray(state.usageEvents) ? state.usageEvents : [];
      const cutoff = Date.now() - 7 * 24 * 60 * 60 * 1000;
      const recent = entries.filter((item) => Number(item.timestampMs) >= cutoff);
      const localTokens = recent.reduce((sum, item) => sum + Number(item.totalTokens ?? 0), 0);
      let quota = null;
      try { quota = await platformRequest('GET', '/api/ai/quota'); } catch { /* anonymous/offline fallback */ }
      return {
        windowDays: 7,
        totalTokens: localTokens,
        events: recent.length,
        source: quota ? 'fabushi-platform+local-runtime' : 'local-runtime-telemetry',
        quota,
      };
    },

    async getUsageSummary() {
      const weekly = await this.getWeeklyUsage();
      const state = await readNativeState();
      const entries = Array.isArray(state.usageEvents) ? state.usageEvents : [];
      const cutoff = Date.now() - 7 * 24 * 60 * 60 * 1000;
      const byProvider = Object.values(entries.filter((item) => Number(item.timestampMs) >= cutoff).reduce((result, item) => {
        const provider = cleanString(item.provider, 80) || 'fabushi';
        const current = result[provider] ?? {
          provider,
          requests: 0,
          inputTokens: 0,
          cachedInputTokens: 0,
          outputTokens: 0,
          reasoningTokens: 0,
          totalTokens: 0,
          lifetimeTokens: Number(state.usageLifetimeByProvider?.[provider] ?? 0),
          lastUsedAtMs: null,
        };
        current.requests += 1;
        current.inputTokens += Number(item.inputTokens ?? 0);
        current.cachedInputTokens += Number(item.cachedInputTokens ?? 0);
        current.outputTokens += Number(item.outputTokens ?? 0);
        current.reasoningTokens += Number(item.reasoningTokens ?? 0);
        current.totalTokens += Number(item.totalTokens ?? 0);
        current.lastUsedAtMs = Math.max(Number(current.lastUsedAtMs ?? 0), Number(item.timestampMs ?? 0)) || null;
        result[provider] = current;
        return result;
      }, {}));
      let membership = null;
      try { membership = await platformRequest('GET', '/api/stripe/membership-status'); } catch { /* offline/not logged in */ }
      return {
        ...weekly,
        membership,
        lifetimeTokens: Number(state.usageLifetimeTokens ?? weekly.totalTokens),
        updatedAtMs: state.usageUpdatedAtMs ?? null,
        byProvider,
      };
    },

    async getInferenceRouterStatus() {
      const home = os.homedir();
      const codexHome = cleanString(process.env.CODEX_HOME, 4096) || path.join(home, '.codex');
      const codexAuth = path.join(codexHome, 'auth.json');
      const claudeCredentials = path.join(home, '.claude', '.credentials.json');
      const [codexCli, claudeCli, codexAuthStatus, claudeCredentialFile, vault] = await Promise.all([
        firstAvailableExecutable(process.env.CODEX_PATH, 'codex'),
        firstAvailableExecutable(process.env.CLAUDE_CODE_PATH, 'claude'),
        inspectCodexAuth(codexAuth),
        isRegularFile(claudeCredentials),
        loadSecretVault(),
      ]);
      const claudeAuthenticated = claudeCredentialFile || Boolean(cleanString(process.env.ANTHROPIC_API_KEY, 16));
      const codexAuthenticated = codexAuthStatus.authenticated;
      const openRouterConfigured = vaultSecretConfigured(vault, 'inference/openrouter/api-key');
      const claudeApiConfigured = vaultSecretConfigured(vault, 'inference/claude/api-key')
        || Boolean(cleanString(process.env.ANTHROPIC_API_KEY, 16));
      const dockerCli = await firstAvailableExecutable(process.env.DOCKER_PATH, 'docker');
      const dockerImagePinned = isPinnedContainerImage(process.env.MAHAYANA_DOCKER_IMAGE || DEFAULT_DOCKER_IMAGE);
      return {
        schemaVersion: 1,
        providers: [
          { id: 'fabushi', label: 'Fabushi', available: true, authenticated: true, source: 'mahayana' },
          { id: 'codex', label: 'Codex', available: codexCli != null && codexAuthenticated, authenticated: codexAuthenticated, installed: codexCli != null, source: 'local-session', reason: codexAuthStatus.reason },
          { id: 'claude-code', label: 'Claude', available: claudeApiConfigured, authenticated: claudeApiConfigured, installed: claudeCli != null, localSessionAuthenticated: claudeAuthenticated, source: claudeApiConfigured ? 'os-secret-vault' : 'local-session-diagnostic' },
          { id: 'openrouter', label: 'OpenRouter', available: openRouterConfigured, authenticated: openRouterConfigured, installed: true, source: 'os-secret-vault' },
        ],
        sandboxes: [
          { id: 'host', label: 'Fabushi Host', available: true, source: 'mahayana' },
          { id: 'local-docker', label: 'Local Docker', available: dockerCli != null && dockerImagePinned, installed: dockerCli != null, imagePinned: dockerImagePinned, source: 'local-cli+pinned-image' },
        ],
      };
    },

    restartInferenceRouter() {
      host.restart('inference Provider credential changed');
      return host.health();
    },

    async getReviewPreferences() {
      return {
        autoReviewInstructions: await this.getAutoReviewInstructions(),
        localToolPermission: await this.getLocalToolPermission(),
      };
    },

    getPrivacyModeEnabled() {
      return getPreference('privacyModeEnabled', false);
    },

    async getRuntimeAccess() {
      try {
        const info = await host.request('feature.info', {});
        return { available: true, status: 'ready', info };
      } catch (error) {
        return { available: false, status: 'unavailable', reason: error instanceof Error ? error.message : String(error) };
      }
    },

    refreshRuntimeAccess() {
      return this.getRuntimeAccess();
    },

    async invokeAccountDashboardAction(params) {
      const action = cleanString(params.action, 120);
      if (action === 'logout') return this.logoutAccount();
      if (action === 'relaunch') return { relaunching: true };
      if (action === 'open-url' && params.url) {
        const url = new URL(String(params.url));
        if (url.protocol !== 'https:' || !url.hostname) throw new Error('Only HTTPS dashboard URLs may be opened.');
        await shell.openExternal(url.toString());
        return { opened: true };
      }
      return { handled: false, action, reason: 'No local dashboard adapter is registered for this action.' };
    },

    async cancelRuntimeTrial() {
      const result = await platformRequest('POST', '/api/stripe/cancel-subscription', { body: {} });
      return { cancelled: result?.success !== false, available: true, provider: 'fabushi-platform', result };
    },

    reportAgentLoad(params) { return report('agent-load', params); },
    reportAccessBlocked(params) { return report('access-blocked', params); },
    reportAgentsUnreachable(params) { return report('agents-unreachable', params); },
    reportRecoveryAction(params) { return report('recovery-action', params); },
    reportRebuildLifecycle(params) { return report('rebuild-lifecycle', params); },
    reportReconciliation(params) { return report('reconciliation', params); },
    reportComputeVisibility(params) { return report('compute-visibility', params); },
    reportSendLatency(params) { return report('send-latency', params); },
    reportSendAck(params) { return report('send-ack', params); },
    reportReactionAck(params) { return report('reaction-ack', params); },
    reportRenderTtfr(params) { return report('render-ttfr', params); },
    reportRenderStream(params) { return report('render-stream', params); },
    reportRemoteDesktopSession(params) { return report('remote-desktop-session', params); },
    reportRemoteDesktopLiveness(params) { return report('remote-desktop-liveness', params); },
    reportOpenComputer(params) { return report('open-computer', params); },
    reportUpdatePrompt(params) { return report('update-prompt', params); },
    reportSigninGate(params) { return report('signin-gate', params); },
    reportOnboardingStep(params) { return report('onboarding-step', params); },
    reportClientFailure(params) { return report('client-failure', params); },
    reportHeapMetrics(params) { return report('heap-metrics', params); },
    noteConversationForDiagnostics(params) { return report('conversation-diagnostics', params); },

    async getDeveloperCommerceProfile() {
      return platformRequest('GET', '/v1/developer/commerce/profile');
    },

    async upsertDeveloperCommerceProfile(params) {
      const displayName = cleanString(params.displayName, 80);
      if (!displayName) throw new Error('Developer display name is required.');
      return platformRequest('POST', '/v1/developer/commerce/profile', { body: { displayName } });
    },

    async listDeveloperCommerceMiniApps() {
      return platformRequest('GET', '/v1/developer/commerce/miniapps');
    },

    async registerDeveloperCommerceMiniApp(params) {
      const miniAppId = cleanString(params.miniAppId, 128);
      const displayName = cleanString(params.displayName, 30);
      if (!miniAppId || !/^[A-Za-z0-9._:-]+$/.test(miniAppId) || !displayName) {
        throw new Error('Valid Mini App ID and display name are required.');
      }
      return platformRequest('POST', `/v1/developer/commerce/miniapps/${encodeURIComponent(miniAppId)}`, { body: { displayName } });
    },

    async listDeveloperCommerceProducts(params) {
      const miniAppId = cleanString(params.miniAppId, 128);
      if (!miniAppId || !/^[A-Za-z0-9._:-]+$/.test(miniAppId)) throw new Error('Valid Mini App ID is required.');
      return platformRequest('GET', `/v1/developer/commerce/miniapps/${encodeURIComponent(miniAppId)}/products`);
    },

    async createDeveloperCommerceProduct(params) {
      const miniAppId = cleanString(params.miniAppId, 128);
      const sku = cleanString(params.sku, 128);
      const displayName = cleanString(params.displayName, 30);
      const description = cleanString(params.description, 45);
      const productKind = cleanString(params.productKind, 40);
      const entitlementCapability = cleanString(params.entitlementCapability, 128);
      const currency = cleanString(params.currency, 3).toUpperCase();
      const taxCode = cleanString(params.taxCode, 64) || undefined;
      const amount = Number(params.amount);
      const allowedKinds = new Set(['digital_consumable','digital_durable','subscription','physical','service']);
      const allowedRails = new Set(['apple_advanced_commerce','google_play','web_provider','merchant_provider','credits']);
      const rails = Array.isArray(params.rails) ? [...new Set(params.rails.map((value) => cleanString(value, 40)).filter((value) => allowedRails.has(value)))].slice(0, 5) : [];
      if (!miniAppId || !sku || !displayName || !entitlementCapability || !allowedKinds.has(productKind) || !/^[A-Z]{3}$/.test(currency) || !Number.isSafeInteger(amount) || amount <= 0) {
        throw new Error('Invalid Developer Commerce product payload.');
      }
      const body = { sku, displayName, description, productKind, entitlementCapability, currency, amount, rails };
      if (taxCode) body.taxCode = taxCode;
      if (productKind === 'subscription') body.subscriptionPeriodSeconds = 2592000;
      return platformRequest('POST', `/v1/developer/commerce/miniapps/${encodeURIComponent(miniAppId)}/products`, { body });
    },

    async updateDeveloperCommerceProduct(params) {
      const miniAppId = cleanString(params.miniAppId, 128);
      const sku = cleanString(params.sku, 128);
      const displayName = cleanString(params.displayName, 30);
      const description = cleanString(params.description, 45);
      const productKind = cleanString(params.productKind, 40);
      const entitlementCapability = cleanString(params.entitlementCapability, 128);
      const currency = cleanString(params.currency, 3).toUpperCase();
      const taxCode = cleanString(params.taxCode, 64) || undefined;
      const amount = Number(params.amount);
      const allowedKinds = new Set(['digital_consumable','digital_durable','subscription','physical','service']);
      const allowedRails = new Set(['apple_advanced_commerce','google_play','web_provider','merchant_provider','credits']);
      const rails = Array.isArray(params.rails) ? [...new Set(params.rails.map((value) => cleanString(value, 40)).filter((value) => allowedRails.has(value)))].slice(0, 5) : [];
      if (!miniAppId || !sku || !displayName || !entitlementCapability || !allowedKinds.has(productKind) || !/^[A-Z]{3}$/.test(currency) || !Number.isSafeInteger(amount) || amount <= 0) {
        throw new Error('Invalid Developer Commerce product payload.');
      }
      const body = { sku, displayName, description, productKind, entitlementCapability, currency, amount, rails };
      if (taxCode) body.taxCode = taxCode;
      if (productKind === 'subscription') body.subscriptionPeriodSeconds = 2592000;
      const productId = cleanString(params.productId, 128);
      if (!productId) throw new Error('Product ID is required.');
      return platformRequest('POST', `/v1/developer/commerce/miniapps/${encodeURIComponent(miniAppId)}/products/${encodeURIComponent(productId)}`, { body });
    },

    async syncDeveloperCommerceGoogleProduct(params) {
      const miniAppId = cleanString(params.miniAppId, 128);
      const productId = cleanString(params.productId, 128);
      if (!miniAppId || !productId) throw new Error('Mini App ID and product ID are required.');
      return platformRequest('POST', `/v1/developer/commerce/miniapps/${encodeURIComponent(miniAppId)}/products/${encodeURIComponent(productId)}/google/sync`, { body: {} });
    },

    async getDeveloperPayoutOverview() {
      return platformRequest('GET', '/v1/developer/commerce/payout');
    },

    async upsertDeveloperPayoutProfile(params) {
      const countryCode = cleanString(params.countryCode, 2).toUpperCase();
      const legalEntityType = cleanString(params.legalEntityType, 32);
      const preferredCurrency = cleanString(params.preferredCurrency, 3).toUpperCase();
      const payoutSchedule = cleanString(params.payoutSchedule, 16);
      const entityKinds = new Set(['individual','individual_business','company','nonprofit']);
      const schedules = new Set(['manual','daily','weekly','monthly']);
      if (!/^[A-Z]{2}$/.test(countryCode) || !entityKinds.has(legalEntityType) || !/^[A-Z]{3}$/.test(preferredCurrency) || !schedules.has(payoutSchedule)) {
        throw new Error('Invalid developer payout profile.');
      }
      return platformRequest('POST', '/v1/developer/commerce/payout/profile', { body: { countryCode, legalEntityType, preferredCurrency, payoutSchedule } });
    },

    async requestDeveloperPayout(params) {
      const payoutAccountId = cleanString(params.payoutAccountId, 160);
      const currency = cleanString(params.currency, 3).toUpperCase();
      const idempotencyKey = cleanString(params.idempotencyKey, 160);
      const amount = Number(params.amount);
      if (!payoutAccountId || !/^[A-Za-z0-9._:-]+$/.test(payoutAccountId) || !/^[A-Z]{3}$/.test(currency) || !idempotencyKey || !/^[A-Za-z0-9._:-]+$/.test(idempotencyKey) || !Number.isSafeInteger(amount) || amount <= 0) {
        throw new Error('Invalid developer payout request.');
      }
      return platformRequest('POST', '/v1/developer/commerce/payout/request', { body: { payoutAccountId, currency, amount, idempotencyKey } });
    },

    async createDeveloperPayoutOnboarding(params) {
      const provider = cleanString(params.provider, 48);
      const purpose = cleanString(params.purpose, 48);
      const providers = new Set(['stripe_connect','adyen_platform','paypal_multiparty','paypal_payouts','wechat_platform','alipay_platform','lianlian_account_plus','huifu_dougong']);
      const purposes = new Set(['original_order_split','external_proceeds_payout','marketplace_payout']);
      if (!providers.has(provider) || !purposes.has(purpose)) throw new Error('Invalid payout onboarding route.');
      return platformRequest('POST', '/v1/developer/commerce/payout/onboarding', { body: { provider, purpose } });
    },

    async getSharingState(params) {
      const result = await platformRequest('GET', '/api/collaboration/state');
      const state = result?.state ?? { scope: 'fabushi-platform', rooms: [], joinRequests: [], typing: [], fetchedAtMs: Date.now() };
      const agentId = cleanString(params.agentId, 200);
      if (!agentId) return state;
      const rooms = Array.isArray(state.rooms) ? state.rooms.filter((room) => Array.isArray(room.memberAgentIds) && room.memberAgentIds.includes(agentId)) : [];
      const roomIds = new Set(rooms.map((room) => room.id));
      return {
        ...state,
        rooms,
        joinRequests: Array.isArray(state.joinRequests) ? state.joinRequests.filter((request) => roomIds.has(request.roomId) || request.agentId === agentId) : [],
        typing: Array.isArray(state.typing) ? state.typing.filter((entry) => roomIds.has(entry.roomId)) : [],
      };
    },

    async createRoomFromAgent(params) {
      const agentId = cleanString(params.agentId, 200);
      if (!agentId) throw new Error('Agent ID is required.');
      const result = await platformRequest('POST', '/api/collaboration/rooms', {
        body: {
          name: cleanString(params.name, 96) || 'Fabushi shared room',
          ownerAgentId: agentId,
          memberAgentIds: [agentId],
        },
      });
      broadcastNativeEvent('shared-room-changed', { action: 'room-created', room: result?.room ?? null });
      return result?.room ?? null;
    },

    async createSharedRoom(params) {
      const memberAgentIds = Array.isArray(params.memberAgentIds)
        ? params.memberAgentIds.map((value) => cleanString(value, 200)).filter(Boolean).slice(0, 100)
        : [];
      const result = await platformRequest('POST', '/api/collaboration/rooms', {
        body: {
          name: cleanString(params.name, 96),
          ownerAgentId: cleanString(params.ownerAgentId, 200) || null,
          memberAgentIds,
        },
      });
      broadcastNativeEvent('shared-room-changed', { action: 'room-created', room: result?.room ?? null });
      return result?.room ?? null;
    },

    async createRoomInvite(params) {
      const roomId = cleanString(params.roomId, 240);
      if (!roomId) throw new Error('Room ID is required.');
      const result = await platformRequest('POST', `/api/collaboration/rooms/${encodeURIComponent(roomId)}/invites`, { body: {} });
      return result?.invite ?? null;
    },

    async joinSharedRoom(params) {
      const token = cleanString(params.token, 1000);
      const agentId = cleanString(params.agentId, 200);
      if (!token || !agentId) throw new Error('Invite token and agent ID are required.');
      const result = await platformRequest('POST', `/api/collaboration/invites/${encodeURIComponent(token)}/join`, {
        body: { agentId, displayName: cleanString(params.displayName, 96) || agentId },
      });
      broadcastNativeEvent('shared-room-changed', { action: 'join-requested', request: result?.request ?? null });
      return result?.request ?? null;
    },

    async respondToRoomJoinRequest(params) {
      const requestId = cleanString(params.requestId, 240);
      if (!requestId) throw new Error('Join request ID is required.');
      const result = await platformRequest('POST', `/api/collaboration/join-requests/${encodeURIComponent(requestId)}/respond`, {
        body: { accept: params.accept === true },
      });
      broadcastNativeEvent('shared-room-changed', { action: 'join-resolved', request: result?.request ?? null, room: result?.room ?? null });
      return result?.request ?? null;
    },

    async addOwnAgentToSharedRoom(params) {
      const roomId = cleanString(params.roomId, 240);
      const agentId = cleanString(params.agentId, 200);
      if (!roomId || !agentId) throw new Error('Room ID and agent ID are required.');
      const result = await platformRequest('POST', `/api/collaboration/rooms/${encodeURIComponent(roomId)}/members`, {
        body: { agentId, displayName: cleanString(params.displayName, 96) || agentId },
      });
      broadcastNativeEvent('shared-room-changed', { action: 'member-added', room: result?.room ?? null });
      return result?.room ?? null;
    },

    async removeOwnAgentFromSharedRoom(params) {
      const roomId = cleanString(params.roomId, 240);
      const agentId = cleanString(params.agentId, 200);
      if (!roomId || !agentId) throw new Error('Room ID and agent ID are required.');
      const result = await platformRequest('DELETE', `/api/collaboration/rooms/${encodeURIComponent(roomId)}/members/${encodeURIComponent(agentId)}`);
      broadcastNativeEvent('shared-room-changed', { action: 'member-removed', room: result?.room ?? null });
      return result?.room ?? null;
    },

    async setSharedRoomTyping(params) {
      const roomId = cleanString(params.roomId, 240);
      const participantId = cleanString(params.participantId, 200);
      if (!roomId || !participantId) throw new Error('Room ID and participant ID are required.');
      const result = await platformRequest('POST', `/api/collaboration/rooms/${encodeURIComponent(roomId)}/typing`, {
        body: { participantId, isTyping: params.isTyping === true },
      });
      broadcastNativeEvent('shared-room-changed', { action: 'typing', typing: result?.typing ?? null });
      return result?.typing ?? null;
    },

    async leaveSharedRoom(params) {
      const roomId = cleanString(params.roomId, 240);
      const agentId = cleanString(params.agentId, 200);
      if (!roomId || !agentId) throw new Error('Room ID and agent ID are required.');
      const result = await platformRequest('POST', `/api/collaboration/rooms/${encodeURIComponent(roomId)}/leave`, { body: { agentId } });
      broadcastNativeEvent('shared-room-changed', { action: 'left', roomId, agentId, room: result?.room ?? null });
      return result?.room ?? null;
    },

    async getForeverBoxStatus(params) {
      const agentId = cleanString(params.agentId, 200);
      if (!agentId) throw new Error('Agent ID is required.');
      const state = await readNativeState();
      const workspace = state.workspaces?.[agentId] ?? null;
      if (!workspace?.boxId) {
        return {
          agentId,
          boxId: null,
          status: 'released',
          provider: null,
          createdAtMs: null,
          updatedAtMs: Date.now(),
          reason: 'No persistent desktop workspace is provisioned for this agent.',
        };
      }
      const workspaceRoot = path.join(app.getPath('userData'), 'workspaces', 'active');
      const workspacePath = ensureWithin(workspaceRoot, path.join(workspaceRoot, workspace.boxId));
      try {
        const stat = await fs.stat(workspacePath);
        if (!stat.isDirectory()) throw new Error('workspace path is not a directory');
      } catch (error) {
        if (error?.code === 'ENOENT') {
          return {
            agentId,
            boxId: workspace.boxId,
            status: 'unavailable',
            provider: 'fabushi-desktop',
            createdAtMs: workspace.createdAtMs ?? null,
            updatedAtMs: Date.now(),
            reason: 'Workspace metadata exists but its managed directory is missing.',
          };
        }
        throw error;
      }
      return {
        agentId,
        boxId: workspace.boxId,
        status: 'ready',
        provider: 'fabushi-desktop',
        createdAtMs: workspace.createdAtMs ?? null,
        updatedAtMs: workspace.updatedAtMs ?? Date.now(),
        reason: null,
      };
    },

    async ensureForeverBox(params) {
      const agentId = cleanString(params.agentId, 200);
      if (!agentId) throw new Error('Agent ID is required.');
      const now = Date.now();
      const state = await readNativeState();
      const current = state.workspaces?.[agentId] ?? null;
      const boxId = current?.boxId || `box-${crypto.createHash('sha256').update(agentId).digest('hex').slice(0, 20)}`;
      const workspaceRoot = path.join(app.getPath('userData'), 'workspaces', 'active');
      const workspacePath = ensureWithin(workspaceRoot, path.join(workspaceRoot, boxId));
      await fs.mkdir(path.join(workspacePath, 'files'), { recursive: true, mode: 0o700 });
      await fs.mkdir(path.join(workspacePath, '.fabushi'), { recursive: true, mode: 0o700 });
      const metadata = {
        schemaVersion: 1,
        agentId,
        boxId,
        provider: 'fabushi-desktop',
        createdAtMs: current?.createdAtMs ?? now,
        updatedAtMs: now,
      };
      await fs.writeFile(path.join(workspacePath, '.fabushi', 'workspace.json'), `${JSON.stringify(metadata, null, 2)}\n`, { mode: 0o600 });
      await mutateNativeState((nativeState) => ({
        ...nativeState,
        workspaces: {
          ...(nativeState.workspaces ?? {}),
          [agentId]: metadata,
        },
      }));
      return this.getForeverBoxStatus({ agentId });
    },

    async handBackForeverBox(params) {
      const agentId = cleanString(params.agentId, 200);
      if (!agentId) throw new Error('Agent ID is required.');
      const state = await readNativeState();
      const workspace = state.workspaces?.[agentId] ?? null;
      if (workspace?.boxId) {
        const activeRoot = path.join(app.getPath('userData'), 'workspaces', 'active');
        const releasedRoot = path.join(app.getPath('userData'), 'workspaces', 'released');
        const source = ensureWithin(activeRoot, path.join(activeRoot, workspace.boxId));
        const target = ensureWithin(releasedRoot, path.join(releasedRoot, `${workspace.boxId}-${Date.now()}`));
        await fs.mkdir(releasedRoot, { recursive: true, mode: 0o700 });
        try {
          await fs.rename(source, target);
        } catch (error) {
          if (error?.code !== 'ENOENT') throw error;
        }
        await mutateNativeState((nativeState) => {
          const workspaces = { ...(nativeState.workspaces ?? {}) };
          delete workspaces[agentId];
          return { ...nativeState, workspaces };
        });
      }
      return this.getForeverBoxStatus({ agentId });
    },

    async getBoxSecretsStatus(params) {
      const agentId = cleanString(params.agentId, 200);
      const box = await this.getForeverBoxStatus({ agentId });
      if (!box.boxId) {
        return { agentId, boxId: null, configured: false, secretCount: 0, provider: box.provider };
      }
      const vault = await loadSecretVault();
      const prefix = `box:${box.boxId}:`;
      const secretCount = Object.keys(vault).filter((name) => name.startsWith(prefix)).length;
      return {
        agentId,
        boxId: box.boxId,
        configured: secretCount > 0,
        secretCount,
        provider: box.provider,
      };
    },

    async isAgentNetworkEnabled(params) {
      const agentId = cleanString(params.agentId, 200);
      if (!agentId) throw new Error('Agent ID is required.');
      const status = await this.getEgressTunnelStatus();
      return status.agentEnabled === true;
    },

    async getCloudAgentInfo(params) {
      const runId = cleanString(params.bcId ?? params.runId ?? params.id, 240);
      if (!runId) throw new Error('Cloud run ID is required.');
      const result = await platformRequest('GET', `/api/agent/runs/${encodeURIComponent(runId)}`);
      const run = result?.run;
      if (!run || typeof run !== 'object') throw new Error('Cloud run response did not include run metadata.');
      const rawStatus = cleanString(run.status, 40) || 'unknown';
      const status = rawStatus === 'completed'
        ? 'finished'
        : rawStatus === 'failed'
          ? 'error'
          : rawStatus === 'cancelled'
            ? 'expired'
            : rawStatus === 'queued'
              ? 'queued'
              : rawStatus === 'running'
                ? 'running'
                : 'unknown';
      return {
        id: runId,
        runId,
        conversationId: cleanString(run.conversationId, 240) || undefined,
        name: cleanString(params.name ?? `Cloud run ${runId.slice(-8)}`, 160),
        available: true,
        provider: 'fabushi-platform',
        status,
        rawStatus,
        model: run.model ?? null,
        inputTokens: Number(run.inputTokens ?? 0),
        outputTokens: Number(run.outputTokens ?? 0),
        toolCallCount: Number(run.toolCallCount ?? 0),
        startedAt: run.startedAt ?? null,
        completedAt: run.completedAt ?? null,
        failedAt: run.failedAt ?? null,
        errorCode: run.errorCode ?? null,
        errorMessage: run.errorMessage ?? null,
        reason: status === 'error' ? cleanString(run.errorMessage, 1000) || 'Cloud run failed.' : null,
      };
    },

    async cancelCloudAgent(params) {
      const runId = cleanString(params.bcId ?? params.runId ?? params.id, 240);
      if (!runId) throw new Error('Cloud run ID is required.');
      const result = await platformRequest('POST', `/api/agent/runs/${encodeURIComponent(runId)}/cancel`, { body: {} });
      const info = await this.getCloudAgentInfo({ bcId: runId }).catch(() => null);
      if (info) broadcastNativeEvent('cloud-agent-open', info);
      return { cancelled: result?.success !== false, runId, result, info };
    },

    async openCloudAgent(params) {
      const runId = cleanString(params.bcId ?? params.runId ?? params.id, 240);
      if (runId) {
        const info = await this.getCloudAgentInfo({ ...params, bcId: runId });
        broadcastNativeEvent('cloud-agent-open', info);
        return { opened: true, provider: 'fabushi-platform', info };
      }
      const url = cleanString(params.url, 4096);
      if (url) {
        const parsed = new URL(url);
        if (parsed.protocol !== 'https:') throw new Error('Only HTTPS cloud-agent URLs may be opened.');
        await shell.openExternal(parsed.toString());
        return { opened: true, provider: 'external-url' };
      }
      return { opened: false, available: false, reason: 'No cloud-agent provider URL was supplied.' };
    },

    async getLinkMetadata(params) {
      const parsed = new URL(cleanString(params.url, 4096));
      if (parsed.protocol !== 'https:') throw new Error('Only HTTPS link metadata is supported.');
      const response = await net.fetch(parsed.toString(), { headers: { accept: 'text/html,application/xhtml+xml' } });
      if (!response.ok) throw new Error(`Link metadata request failed with HTTP ${response.status}.`);
      const bytes = Buffer.from(await response.arrayBuffer());
      const html = bytes.subarray(0, 512 * 1024).toString('utf8');
      const title = html.match(/<title[^>]*>([\s\S]*?)<\/title>/i)?.[1]?.replace(/<[^>]+>/g, '').replace(/\s+/g, ' ').trim() ?? null;
      const description = html.match(/<meta[^>]+(?:name|property)=["'](?:description|og:description)["'][^>]+content=["']([^"']*)["']/i)?.[1] ?? null;
      const image = html.match(/<meta[^>]+property=["']og:image["'][^>]+content=["']([^"']*)["']/i)?.[1] ?? null;
      return { url: response.url || parsed.toString(), title, description, image, contentType: response.headers.get('content-type') };
    },

    async listSecrets() {
      const vault = await loadSecretVault();
      return Object.entries(vault).map(([name, item]) => ({ name, updatedAtMs: item.updatedAtMs ?? null })).sort((a, b) => a.name.localeCompare(b.name));
    },

    async revealSecret(params) {
      requireSecretEncryption();
      const name = cleanString(params.name ?? params.key, 200);
      const item = (await loadSecretVault())[name];
      if (!item?.ciphertext) return null;
      return safeStorage.decryptString(Buffer.from(item.ciphertext, 'base64'));
    },

    async upsertSecrets(params) {
      requireSecretEncryption();
      const candidates = [];
      if (Array.isArray(params.secrets)) candidates.push(...params.secrets);
      else if (params.secrets && typeof params.secrets === 'object') candidates.push(...Object.entries(params.secrets).map(([name, value]) => ({ name, value })));
      else if (params.name != null) candidates.push({ name: params.name, value: params.value });
      const vault = await loadSecretVault();
      const now = Date.now();
      for (const candidate of candidates.slice(0, 100)) {
        const name = cleanString(candidate.name ?? candidate.key, 200);
        if (!name || !/^[a-zA-Z0-9._:/-]+$/.test(name)) throw new Error('Invalid secret name.');
        const value = String(candidate.value ?? '');
        const ciphertext = safeStorage.encryptString(value).toString('base64');
        vault[name] = { ciphertext, updatedAtMs: now };
      }
      await saveSecretVault(vault);
      return this.listSecrets();
    },

    async removeSecrets(params) {
      const names = Array.isArray(params.names) ? params.names : [params.name ?? params.key].filter(Boolean);
      const vault = await loadSecretVault();
      for (const raw of names) delete vault[cleanString(raw, 200)];
      await saveSecretVault(vault);
      return this.listSecrets();
    },

    async migrateClientPersistence(params) {
      const fromPrefix = cleanString(params.fromPrefix ?? params.from, 160);
      const toPrefix = cleanString(params.toPrefix ?? params.to, 160);
      if (!fromPrefix || !toPrefix) throw new Error('Persistence migration requires fromPrefix and toPrefix.');
      const state = await readNativeState();
      const store = { ...(state.clientPersistence ?? {}) };
      let migrated = 0;
      for (const [key, value] of Object.entries({ ...store })) {
        if (!key.startsWith(fromPrefix)) continue;
        const nextKey = `${toPrefix}${key.slice(fromPrefix.length)}`;
        if (store[nextKey] === undefined || params.overwrite === true) store[nextKey] = value;
        if (params.copy !== true) delete store[key];
        migrated += 1;
      }
      await mutateNativeState((current) => ({ ...current, clientPersistence: store }));
      return { migrated, fromPrefix, toPrefix, copied: params.copy === true };
    },

    async getMcpState() {
      const [tools, plugins] = await Promise.all([
        host.request('runtime.tools', {}).catch(() => []),
        installedPluginPointers(),
      ]);
      const state = await readNativeState();
      return {
        available: true,
        tools: Array.isArray(tools) ? tools : [],
        plugins,
        customInstructions: state.mcpCustomInstructions ?? {},
        disabledTools: state.mcpDisabledTools ?? {},
      };
    },

    async addMiniAppToAccount(params) {
      const pluginId = cleanString(params.pluginId ?? params.id, 200);
      if (!pluginId) throw new Error('Mini App id is required.');
      return platformRequest('POST', `/v1/marketplace/plugins/${encodeURIComponent(pluginId)}/add`, {
        body: { platform: 'desktop' },
      });
    },

    async removeMiniAppFromAccount(params) {
      const pluginId = cleanString(params.pluginId ?? params.id, 200);
      if (!pluginId) throw new Error('Mini App id is required.');
      return platformRequest('DELETE', `/v1/marketplace/plugins/${encodeURIComponent(pluginId)}/add`);
    },

    async routeMiniAppInput(params) {
      const pluginId = cleanString(params.pluginId ?? params.id, 200);
      const input = cleanString(params.input ?? params.message, 10_000);
      if (!pluginId) throw new Error('Mini App id is required.');
      if (!input) throw new Error('Mini App Bot input is required.');
      return platformRequest('POST', `/v1/marketplace/plugins/${encodeURIComponent(pluginId)}/route`, {
        body: { input },
      });
    },

    async callMiniAppRuntimeTool(params) {
      const pluginId = normalizedMiniAppId(params.pluginId ?? params.id);
      const name = cleanString(params.name ?? params.tool, 128);
      if (!/^[A-Za-z0-9_.-]{1,128}$/.test(name)) throw new Error('Invalid Mini App runtime tool name.');
      const argumentsValue = params.arguments ?? params.input ?? {};
      const argumentsObject = recordValue(argumentsValue);
      if (!argumentsObject) throw new Error('Mini App runtime tool arguments must be an object.');

      const account = await platformRequest('GET', '/v1/marketplace/added');
      const apps = Array.isArray(account?.apps) ? account.apps : [];
      const app = apps.find((candidate) => cleanString(candidate?.id ?? candidate?.pluginId, 200).toLowerCase() === pluginId);
      if (!app) throw new Error(`Mini App ${pluginId} is not installed for this account.`);
      const commands = Array.isArray(app.commands) ? app.commands : [];
      const allowed = commands.some((candidate) => {
        const command = recordValue(candidate);
        return command && cleanString(command.tool ?? command.name, 128) === name;
      });
      if (!allowed) throw new Error(`Mini App runtime tool ${name} is outside ${pluginId}'s installed Tool Contract.`);

      if (pluginId === GLOBAL_DHARMA_ID && name === 'start') {
        const entitlement = await platformRequest('GET', miniAppEntitlementPath(pluginId, LOCAL_PRAYER_WHEEL_CAPABILITY));
        if (entitlement?.access?.allowed !== true) {
          throw new Error(`Mini App runtime tool ${name} requires an active ${LOCAL_PRAYER_WHEEL_CAPABILITY} entitlement.`);
        }
      }

      return host.request('runtime.call', { pluginId, name, arguments: argumentsObject });
    },

    async getMiniAppSessionProjection(params) {
      const pluginId = normalizedMiniAppId(params.pluginId ?? params.id);
      const auth = await host.request('feature.auth.status', {});
      const user = recordValue(auth?.user) ?? {};
      return {
        protocol: 'fabushi.miniapp.session.v1',
        pluginId,
        loggedIn: auth?.loggedIn === true,
        provider: cleanString(auth?.provider, 80) || null,
        account: auth?.loggedIn === true ? {
          id: user.id == null ? null : String(user.id),
          username: cleanString(user.username, 160) || null,
          nickname: cleanString(user.nickname, 160) || null,
          avatar: cleanString(user.avatar, 4096) || null,
        } : null,
        tokenExposed: false,
      };
    },

    async getMiniAppEntitlement(params) {
      const pluginId = normalizedMiniAppId(params.pluginId ?? params.id);
      const capability = normalizedCapability(params.capability);
      return platformRequest('GET', miniAppEntitlementPath(pluginId, capability));
    },

    async purchaseMiniAppLifetime(params) {
      const pluginId = normalizedMiniAppId(params.pluginId ?? params.id);
      const capability = normalizedCapability(params.capability ?? LOCAL_PRAYER_WHEEL_CAPABILITY);
      if (pluginId !== GLOBAL_DHARMA_ID || capability !== LOCAL_PRAYER_WHEEL_CAPABILITY) {
        throw new Error('This desktop lifetime purchase facade is scoped to the official Global Dharma local prayer-wheel capability.');
      }
      const auth = await host.request('feature.auth.status', {});
      const sessionUser = recordValue(auth?.user) ?? {};
      const session = {
        protocol: 'fabushi.miniapp.session.v1', pluginId, loggedIn: auth?.loggedIn === true,
        provider: cleanString(auth?.provider, 80) || null,
        account: auth?.loggedIn === true ? {
          id: sessionUser.id == null ? null : String(sessionUser.id),
          username: cleanString(sessionUser.username, 160) || null,
          nickname: cleanString(sessionUser.nickname, 160) || null,
          avatar: cleanString(sessionUser.avatar, 4096) || null,
        } : null,
        tokenExposed: false,
      };
      if (!session.loggedIn) throw new Error('Fabushi login is required before purchasing a Mini App entitlement.');
      const before = await platformRequest('GET', miniAppEntitlementPath(pluginId, capability));
      if (before?.access?.allowed === true) {
        return { status: 'already-entitled', session, entitlement: before, checkout: null };
      }
      const option = lifetimePrayerWheelOption(before);
      const idempotencyKey = cleanString(params.idempotencyKey, 160);
      if (!idempotencyKey || !/^[A-Za-z0-9][A-Za-z0-9._:-]{7,159}$/.test(idempotencyKey)) {
        throw new Error('A stable Fabushi Pay idempotency key is required.');
      }
      const intent = await platformRequest('POST', `/v1/miniapps/${encodeURIComponent(pluginId)}/pay/intents`, {
        body: { sku: option.sku, rail: 'web_provider', idempotencyKey },
      });
      const paymentId = paymentIdFromIntent(intent);
      if (!paymentId) throw new Error('Fabushi Pay did not return a payment id.');
      const checkout = await platformRequest('POST', `/v1/pay/intents/${encodeURIComponent(paymentId)}/checkout`, { body: {} });
      const redirect = safeCheckoutRedirect(checkout);
      if (redirect) await shell.openExternal(redirect);
      const entitlement = await platformRequest('GET', miniAppEntitlementPath(pluginId, capability));
      return {
        status: entitlement?.access?.allowed === true ? 'entitled' : 'checkout-required',
        session,
        product: option,
        paymentId,
        intent,
        checkout,
        checkoutOpened: Boolean(redirect),
        entitlement,
      };
    },

    async restoreMiniAppPurchases(params) {
      const pluginId = normalizedMiniAppId(params.pluginId ?? params.id);
      const capability = normalizedCapability(params.capability ?? LOCAL_PRAYER_WHEEL_CAPABILITY);
      const auth = await host.request('feature.auth.status', {});
      const session = { protocol: 'fabushi.miniapp.session.v1', pluginId, loggedIn: auth?.loggedIn === true, tokenExposed: false };
      if (!session.loggedIn) throw new Error('Fabushi login is required before restoring Mini App purchases.');
      const restored = await platformRequest('POST', '/v1/purchases/restore', { body: { pluginId } });
      const entitlement = await platformRequest('GET', miniAppEntitlementPath(pluginId, capability));
      return { session, restored, entitlement };
    },

    async getAccountSync(params) {
      const cursor = cleanString(params.cursor, 160);
      const limit = Math.max(1, Math.min(1000, Number(params.limit) || 200));
      return platformRequest('GET', '/v1/account/sync', {
        query: { ...(cursor ? { cursor } : {}), limit },
      });
    },

    getAccountMiniApps() {
      return platformRequest('GET', '/v1/marketplace/added');
    },

    getAccountBots() {
      return platformRequest('GET', '/v1/account/bots');
    },

    async addBotToAccount(params) {
      const botId = cleanString(params.botId ?? params.id, 160);
      if (!botId) throw new Error('Bot id is required.');
      const bot = params.bot && typeof params.bot === 'object' ? params.bot : params;
      return platformRequest('POST', `/v1/account/bots/${encodeURIComponent(botId)}/add`, { body: { bot } });
    },

    async removeBotFromAccount(params) {
      const botId = cleanString(params.botId ?? params.id, 160);
      if (!botId) throw new Error('Bot id is required.');
      return platformRequest('DELETE', `/v1/account/bots/${encodeURIComponent(botId)}/add`);
    },

    async getMiniAppBotMessages(params) {
      const pluginId = cleanString(params.pluginId ?? params.id, 200);
      if (!pluginId) throw new Error('Mini App id is required.');
      const after = cleanString(params.after, 160);
      const limit = Math.max(1, Math.min(1000, Number(params.limit) || 500));
      return platformRequest('GET', `/api/miniapps/${encodeURIComponent(pluginId)}/messages`, {
        query: { ...(after ? { after } : {}), limit },
      });
    },

    async appendMiniAppBotMessages(params) {
      const pluginId = cleanString(params.pluginId ?? params.id, 200);
      if (!pluginId) throw new Error('Mini App id is required.');
      const messages = Array.isArray(params.messages) ? params.messages : [];
      if (!messages.length || messages.length > 100) throw new Error('Mini App Bot messages must contain 1-100 entries.');
      return platformRequest('POST', `/api/miniapps/${encodeURIComponent(pluginId)}/messages`, { body: { messages } });
    },

    async getMiniAppCloudStorage(params) {
      const pluginId = cleanString(params.pluginId ?? params.id, 200);
      if (!pluginId) throw new Error('Mini App id is required.');
      const key = cleanString(params.key, 128);
      return platformRequest('GET', `/v1/miniapps/${encodeURIComponent(pluginId)}/cloud-storage`, {
        query: key ? { key } : undefined,
      });
    },

    async setMiniAppCloudStorage(params) {
      const pluginId = cleanString(params.pluginId ?? params.id, 200);
      if (!pluginId) throw new Error('Mini App id is required.');
      const values = params.values && typeof params.values === 'object' && !Array.isArray(params.values) ? params.values : {};
      return platformRequest('PUT', `/v1/miniapps/${encodeURIComponent(pluginId)}/cloud-storage`, { body: { values } });
    },

    async deleteMiniAppCloudStorage(params) {
      const pluginId = cleanString(params.pluginId ?? params.id, 200);
      const key = cleanString(params.key, 128);
      if (!pluginId || !key) throw new Error('Mini App id and CloudStorage key are required.');
      return platformRequest('DELETE', `/v1/miniapps/${encodeURIComponent(pluginId)}/cloud-storage`, { query: { key } });
    },

    async reconcileAccountMiniApps() {
      const account = await this.getAccountMiniApps();
      const apps = Array.isArray(account?.apps) ? account.apps : [];
      const desired = new Map(apps.map((entry) => [cleanString(entry?.id ?? entry?.pluginId, 200), entry]).filter(([id]) => id));
      const installed = new Map((await installedPluginPointers()).map((entry) => [entry.pluginId, entry]));
      const state = await readNativeState();
      const previousManaged = new Set(Array.isArray(state.accountManagedMiniApps) ? state.accountManagedMiniApps.map((id) => cleanString(id, 200)).filter(Boolean) : []);
      const installedNow = [];
      const removedNow = [];
      const failures = [];

      for (const [pluginId, entry] of desired) {
        if (installed.has(pluginId)) continue;
        const version = cleanString(entry?.version ?? entry?.latestVersion, 100);
        if (!version) {
          failures.push({ pluginId, reason: 'account Mini App has no version' });
          continue;
        }
        try {
          const release = await host.request('feature.marketplace.release', { pluginId, version });
          const pointer = await host.request('feature.plugin.install', { release, platform: 'desktop' });
          installedNow.push({ pluginId, version, pointer });
        } catch (error) {
          failures.push({ pluginId, reason: error instanceof Error ? error.message : String(error) });
        }
      }

      for (const pluginId of previousManaged) {
        if (desired.has(pluginId) || !installed.has(pluginId)) continue;
        try {
          await host.request('feature.plugin.uninstall', { pluginId });
          removedNow.push(pluginId);
        } catch (error) {
          failures.push({ pluginId, reason: error instanceof Error ? error.message : String(error) });
        }
      }

      await mutateNativeState((current) => ({
        ...current,
        accountManagedMiniApps: [...desired.keys()].sort(),
      }));
      return {
        accountSynchronized: account?.accountSynchronized === true,
        cursor: account?.cursor ?? null,
        desired: [...desired.keys()],
        installed: installedNow,
        removed: removedNow,
        failures,
      };
    },

    getEffectivePlugins() {
      return installedPluginPointers();
    },

    async getMcpCatalog() {
      const catalog = await host.request('feature.marketplace.browse', { query: 'mcp', platform: 'desktop' });
      return Array.isArray(catalog) ? catalog : catalog?.plugins ?? catalog;
    },

    async getMcpTeamPopularity() {
      const catalog = await host.request('feature.marketplace.browse', { query: 'mcp', platform: 'desktop' }).catch(() => []);
      const entries = Array.isArray(catalog) ? catalog : Array.isArray(catalog?.plugins) ? catalog.plugins : [];
      const items = entries
        .map((item) => ({
          id: cleanString(item?.id ?? item?.pluginId, 200),
          name: cleanString(item?.name ?? item?.title ?? item?.id, 200),
          score: Number(item?.popularity ?? item?.installCount ?? item?.downloads ?? 0) || 0,
          source: 'marketplace-catalog',
        }))
        .filter((item) => item.id)
        .sort((left, right) => right.score - left.score || left.name.localeCompare(right.name))
        .slice(0, 50);
      return { available: true, items, source: 'marketplace-catalog' };
    },

    async getMcpPluginLogo(params) {
      const pluginId = cleanString(params.pluginId ?? params.id, 200);
      const catalog = await host.request('feature.marketplace.browse', { query: pluginId, platform: 'desktop' }).catch(() => []);
      const entries = Array.isArray(catalog) ? catalog : catalog?.plugins ?? [];
      const match = entries.find((item) => cleanString(item?.id ?? item?.pluginId, 200) === pluginId) ?? entries[0];
      return match?.logo ?? match?.icon ?? null;
    },

    async installEntry(params) {
      const pluginId = cleanString(params.pluginId ?? params.id ?? params.entry?.id, 200);
      const version = cleanString(params.version ?? params.entry?.version, 100);
      if (!pluginId || !version) throw new Error('Plugin ID and version are required.');
      const release = params.release ?? await host.request('feature.marketplace.release', { pluginId, version });
      return host.request('feature.plugin.install', { release, platform: 'desktop' });
    },

    updatePluginInstall(params) {
      return this.installEntry(params);
    },

    async removeMcpServer(params) {
      const server = cleanString(params.server ?? params.name, 200);
      if (!server) throw new Error('MCP server name is required.');
      const accepted = await featureExecute({ type: 'mcp.remove', requestId: requestId('mcp-remove'), server });
      return { removed: true, server, accepted };
    },

    async uninstallPlugin(params) {
      const pluginId = cleanString(params.pluginId ?? params.id, 200);
      if (!pluginId) throw new Error('Plugin ID is required.');
      return host.request('feature.plugin.uninstall', { pluginId });
    },

    async authenticateMcpServer(params) {
      const server = cleanString(params.server ?? params.name, 200);
      const accepted = await featureExecute({ type: 'mcp.oauthLogin', requestId: requestId('mcp-login'), server });
      return { server, accepted };
    },

    async renameMcpAccount(params) {
      const connectorId = cleanString(params.connectorId ?? params.server, 200);
      const accountId = cleanString(params.accountId, 200);
      const label = cleanString(params.label ?? params.name, 200);
      const accepted = await featureExecute({ type: 'connector.renameAccount', requestId: requestId('mcp-rename'), connectorId, accountId, label });
      return { connectorId, accountId, label, accepted };
    },

    async removeMcpAccount(params) {
      const connectorId = cleanString(params.connectorId ?? params.server, 200);
      const accountId = cleanString(params.accountId, 200);
      const accepted = await featureExecute({ type: 'connector.removeAccount', requestId: requestId('mcp-account-remove'), connectorId, accountId });
      return { connectorId, accountId, accepted };
    },

    async setMcpCustomInstructions(params) {
      const server = cleanString(params.server ?? params.name, 200);
      const instructions = cleanString(params.instructions, 20000);
      if (!server) throw new Error('MCP server name is required.');
      const accepted = await featureExecute({
        type: 'mcp.setCustomInstructions',
        requestId: requestId('mcp-instructions'),
        server,
        instructions,
      });
      await mutateNativeState((current) => {
        const all = { ...(current.mcpCustomInstructions ?? {}) };
        if (instructions) all[server] = instructions;
        else delete all[server];
        return { ...current, mcpCustomInstructions: all };
      });
      return { server, instructions, accepted };
    },

    async listMcpServerTools(params) {
      const server = cleanString(params.server ?? params.name, 200);
      const tools = await host.request('runtime.tools', {}).catch(() => []);
      return (Array.isArray(tools) ? tools : []).filter((tool) => !server || cleanString(tool?.server ?? tool?.pluginId, 200) === server);
    },

    async toggleMcpToolDisabled(params) {
      const server = cleanString(params.server ?? params.pluginId, 200);
      const tool = cleanString(params.tool ?? params.toolId, 240);
      const disabled = params.disabled === true;
      if (!server || !tool) throw new Error('MCP server and tool are required.');
      const accepted = await featureExecute({
        type: 'mcp.setToolDisabled',
        requestId: requestId('mcp-tool-disabled'),
        server,
        tool,
        disabled,
      });
      const key = `${server}:${tool}`;
      await mutateNativeState((state) => {
        const next = { ...(state.mcpDisabledTools ?? {}) };
        if (disabled) next[key] = true;
        else delete next[key];
        return { ...state, mcpDisabledTools: next };
      });
      return { server, tool, disabled, accepted };
    },

    devRestart() {
      setImmediate(() => { app.relaunch(); app.exit(0); });
      return true;
    },

    async getProductionComputeAttachmentStatus() {
      const enabled = await getPreference('productionComputeAttachmentEnabled', false);
      return { enabled: enabled === true, available: true, provider: 'mahayana' };
    },

    async setProductionComputeAttachmentEnabled(params) {
      const enabled = params.enabled === true;
      await setPreference('productionComputeAttachmentEnabled', enabled);
      return { enabled, available: true, provider: 'mahayana' };
    },
  };

  for (const [name, handler] of Object.entries(handlers)) {
    if (typeof handler === 'function') handlers[name] = handler.bind(handlers);
  }
  return handlers;
}

module.exports = { createNativeCapabilityHandlers };
