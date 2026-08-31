'use strict';

// Electron sandboxed preloads may only require Electron and a small set of
// built-ins. Keep this file self-contained; the main process remains the
// authority that validates the registered edge/method allowlists.
const { contextBridge, ipcRenderer } = require('electron');

const MAHAYANA_EDGE = 'mahayana-host';
const NATIVE_EDGE = 'native-desktop';
const MAHAYANA_RUNTIME_EVENT = 'runtime-event';
const EDGE_CONTRACT_VERSION = 1;
const NATIVE_EVENTS = new Set([
  'app-agent-surface-request',
  'mcp-auth-completed',
  'focus-agent',
  'cloud-agent-open',
  'shared-room-changed',
  'deep-link',
  'compute-migration',
  'dev-compute-rebuild',
  'open-feedback',
  'open-about',
  'widget-gallery',
  'force-onboarding',
  'account-auth-changed',
  'experiments-changed',
  'window-state',
  'zoom-factor-changed',
  'update-computer-dispatched',
  'offline-asr-progress',
  'open-offline-asr',
  'remote-desktop-user-presence',
  'dev-compute-pull-progress',
  'egress-tunnel-changed',
  'egress-tunnel-status-changed',
  'webauthn-proxy-changed',
  'skip-onboarding',
  'theme-changed',
  'update-status',
  'messaging-call-signal',
  'messaging-call-status',
]);

function callChannel(edge, method) {
  return `fabushi-edge:${edge}:call:${method}`;
}

function eventChannel(edge, eventName) {
  return `fabushi-edge:${edge}:event:${eventName}`;
}

function failureMessage(failure) {
  if (!failure || typeof failure !== 'object') return 'Native edge call failed.';
  const code = typeof failure.code === 'string' ? failure.code : 'bridge/invoke-failed';
  const detail = typeof failure.detail === 'string' ? failure.detail : 'Native edge call failed.';
  return `${code}: ${detail}`;
}

async function invokeEdge(edge, method, params = {}) {
  const reply = await ipcRenderer.invoke(callChannel(edge, method), params ?? {});
  if (!reply || typeof reply !== 'object' || typeof reply.ok !== 'boolean') {
    throw new Error('bridge/invoke-failed: Native edge returned an invalid reply.');
  }
  if (reply.ok) return reply.value;
  throw new Error(failureMessage(reply.failure));
}

function subscribeEdge(edge, eventName, listener) {
  if (typeof listener !== 'function') return () => {};
  const channel = eventChannel(edge, eventName);
  const forward = (_event, payload) => listener(payload);
  ipcRenderer.on(channel, forward);
  return () => ipcRenderer.off(channel, forward);
}

// Runtime bootstrap events are one-shot projections emitted by the Rust Host.
// Keep one permanent preload listener so React surface transitions cannot drop
// them across any number of HostClient/Messenger transport subscriptions. Retain
// only the newest idempotent projection for the current authenticated account;
// transient deltas remain live-only and account boundaries clear the snapshots.
const MAHAYANA_REPLAYABLE_EVENTS = new Set([
  'host.ready',
  'conversation.listed',
  'bot.listed',
  'group.listed',
  'settings.changed',
]);
const mahayanaRuntimeListeners = new Set();
const mahayanaReplay = new Map();
const MAHAYANA_REPLAY_RESET_METHODS = new Set([
  'feature.auth.browserStart',
  'feature.auth.passwordLogin',
  'feature.auth.oauthStart',
  'feature.auth.logout',
]);

function clearMahayanaReplay() {
  mahayanaReplay.clear();
}
const mahayanaRuntimeChannel = eventChannel(MAHAYANA_EDGE, MAHAYANA_RUNTIME_EVENT);
ipcRenderer.on(mahayanaRuntimeChannel, (_event, payload) => {
  if (MAHAYANA_REPLAYABLE_EVENTS.has(payload?.type)) {
    mahayanaReplay.set(payload.type, payload);
  }
  for (const listener of mahayanaRuntimeListeners) listener(payload);
});

const mahayana = Object.freeze({
  contractVersion: EDGE_CONTRACT_VERSION,
  async invoke(method, params = {}) {
    if (MAHAYANA_REPLAY_RESET_METHODS.has(method)) clearMahayanaReplay();
    try {
      return await invokeEdge(MAHAYANA_EDGE, method, params);
    } finally {
      if (method === 'feature.auth.logout') clearMahayanaReplay();
    }
  },
  subscribe(listener) {
    if (typeof listener !== 'function') return () => {};
    mahayanaRuntimeListeners.add(listener);
    const replay = Array.from(mahayanaReplay.values());
    for (const payload of replay) listener(payload);
    return () => mahayanaRuntimeListeners.delete(listener);
  },
});

contextBridge.exposeInMainWorld('mahayana', mahayana);

contextBridge.exposeInMainWorld('fabushiNative', Object.freeze({
  contractVersion: EDGE_CONTRACT_VERSION,
  invoke(method, params = {}) {
    return invokeEdge(NATIVE_EDGE, method, params);
  },
  subscribe(listeners = {}) {
    if (!listeners || typeof listeners !== 'object') return () => {};
    const cleanup = [];
    for (const eventName of NATIVE_EVENTS) {
      const listener = listeners[eventName];
      if (typeof listener === 'function') cleanup.push(subscribeEdge(NATIVE_EDGE, eventName, listener));
    }
    return () => cleanup.splice(0).forEach((dispose) => dispose());
  },
}));

contextBridge.exposeInMainWorld('fabushi', Object.freeze({
  contractVersion: EDGE_CONTRACT_VERSION,
  pickFile() {
    return ipcRenderer.invoke('fabushi:pick-file');
  },
  notify(title, body) {
    return ipcRenderer.invoke('fabushi:notify', { title, body });
  },
  openExternal(url) {
    return ipcRenderer.invoke('fabushi:open-external', { url });
  },
  openSystemSettings(pane) {
    return ipcRenderer.invoke('fabushi:open-system-settings', { pane });
  },
  windowFocused() {
    return ipcRenderer.invoke('fabushi:window-focused');
  },
  registerMiniAppDocument(pluginId, html) {
    return ipcRenderer.invoke('fabushi:register-miniapp-document', { pluginId, html });
  },
}));
