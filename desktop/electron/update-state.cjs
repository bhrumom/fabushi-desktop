'use strict';

const ACTIONABLE_DESKTOP_UPDATE_TYPES = new Set(['available', 'downloading', 'ready', 'staging']);
const RENDERER_UPDATE_TRACKS = new Set(['stable', 'nightly', 'dogfood']);
const LEGACY_TRACK_ALIASES = new Map([
  ['beta', 'nightly'],
  ['alpha', 'dogfood'],
]);
const DISABLED_REASONS = new Set(['not-packaged', 'lab-build', 'unsupported-platform', 'disabled-by-env']);

function normalizeDesktopUpdateTrack(value) {
  const raw = typeof value === 'string' ? value.trim() : '';
  const normalized = LEGACY_TRACK_ALIASES.get(raw) ?? raw;
  return RENDERER_UPDATE_TRACKS.has(normalized) ? normalized : 'stable';
}

function effectiveDesktopUpdateTrack(value) {
  const normalized = normalizeDesktopUpdateTrack(value);
  return normalized === 'nightly' ? 'stable' : normalized;
}

function normalizeDesktopUpdateStatus(status, currentVersion) {
  const installedVersion = typeof currentVersion === 'string' ? currentVersion : '';
  if (!status || typeof status !== 'object') return { type: 'upToDate', version: installedVersion };
  const version = typeof status.version === 'string' && status.version
    ? status.version
    : installedVersion;
  if (status.type === 'upToDate') return { ...status, version: installedVersion };
  if (version === installedVersion && ACTIONABLE_DESKTOP_UPDATE_TYPES.has(status.type)) {
    return { type: 'upToDate', version: installedVersion };
  }
  return { ...status, version };
}

function normalizeProgress(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return undefined;
  const fraction = numeric > 1 ? numeric / 100 : numeric;
  return Math.max(0, Math.min(1, fraction));
}

function projectDesktopUpdateState(status, currentVersion, nowMs) {
  const source = status && typeof status === 'object' && !Array.isArray(status) ? status : {};
  const canonical = source.state && typeof source.state === 'object' && !Array.isArray(source.state)
    ? source.state
    : null;
  const type = canonical?.type ?? source.type;
  const version = typeof (canonical?.version ?? source.version) === 'string'
    && (canonical?.version ?? source.version)
    ? (canonical?.version ?? source.version)
    : currentVersion;

  if (type === 'disabled') {
    const reason = DISABLED_REASONS.has(canonical?.reason ?? source.reason)
      ? (canonical?.reason ?? source.reason)
      : 'disabled-by-env';
    return { type: 'disabled', reason };
  }
  if (type === 'checking') return { type: 'checking' };
  if (type === 'available') return { type: 'available', version };
  if (type === 'downloading') {
    const progress = normalizeProgress(canonical?.progress ?? source.progress);
    return progress === undefined
      ? { type: 'downloading', version }
      : { type: 'downloading', version, progress };
  }
  if (type === 'staging') return { type: 'staging', version };
  if (type === 'ready') {
    const lastCheck = canonical?.lastCheck;
    return lastCheck && typeof lastCheck === 'object'
      ? { type: 'ready', version, lastCheck }
      : { type: 'ready', version };
  }
  if (type === 'error') {
    const message = typeof source.message === 'string' && source.message
      ? source.message
      : 'Desktop update check failed.';
    const at = Number.isFinite(Number(source.atMs ?? source.at))
      ? Number(source.atMs ?? source.at)
      : nowMs;
    return { type: 'idle', lastCheck: { at, result: 'error', errorMessage: message } };
  }
  if (canonical?.type === 'idle') {
    return canonical.lastCheck && typeof canonical.lastCheck === 'object'
      ? { type: 'idle', lastCheck: canonical.lastCheck }
      : { type: 'idle' };
  }
  return { type: 'idle' };
}

function projectDesktopUpdateStatus(status, currentVersion, options = {}) {
  const source = status && typeof status === 'object' && !Array.isArray(status) ? status : {};
  const trackOverrideSource = options.trackOverride !== undefined
    ? options.trackOverride
    : source.trackOverride;
  const trackOverride = trackOverrideSource == null
    ? null
    : normalizeDesktopUpdateTrack(trackOverrideSource);
  const currentTrack = effectiveDesktopUpdateTrack(
    options.currentTrack ?? source.currentTrack ?? source.track ?? trackOverride ?? 'stable',
  );
  const buildDefaultTrack = normalizeDesktopUpdateTrack(
    options.buildDefaultTrack ?? source.buildDefaultTrack ?? 'stable',
  );
  const rawAvailableTracks = options.availableTracks ?? source.availableTracks;
  const availableTracks = Array.isArray(rawAvailableTracks)
    ? [...new Set(rawAvailableTracks.map(normalizeDesktopUpdateTrack))]
    : currentTrack === 'stable'
      ? ['stable']
      : ['stable', currentTrack];
  if (!availableTracks.includes(currentTrack)) availableTracks.push(currentTrack);

  return {
    state: projectDesktopUpdateState(
      source,
      typeof currentVersion === 'string' ? currentVersion : '',
      Number.isFinite(Number(options.nowMs)) ? Number(options.nowMs) : Date.now(),
    ),
    currentVersion: typeof currentVersion === 'string' ? currentVersion : '',
    currentTrack,
    trackOverride,
    buildDefaultTrack,
    availableTracks,
    isTrackManagedByPolicy: options.isTrackManagedByPolicy ?? source.isTrackManagedByPolicy === true,
    isBelowMinimumVersion: options.isBelowMinimumVersion ?? source.isBelowMinimumVersion === true,
    autoUpdateWhenIdleOptIn: options.autoUpdateWhenIdleOptIn ?? source.autoUpdateWhenIdleOptIn === true,
    autoUpdateWhenIdleGateEnabled: options.autoUpdateWhenIdleGateEnabled ?? source.autoUpdateWhenIdleGateEnabled === true,
  };
}

module.exports = {
  effectiveDesktopUpdateTrack,
  normalizeDesktopUpdateStatus,
  normalizeDesktopUpdateTrack,
  projectDesktopUpdateStatus,
};
