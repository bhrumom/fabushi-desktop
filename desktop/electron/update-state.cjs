'use strict';

const ACTIONABLE_DESKTOP_UPDATE_TYPES = new Set(['available', 'downloading', 'ready', 'staging']);

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

module.exports = { normalizeDesktopUpdateStatus };
