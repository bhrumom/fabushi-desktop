'use strict';

// Keep electron/main.cjs as the canonical desktop runtime. This bootstrap wraps
// the existing native capability factory before main loads so secrets stay in
// the trusted main process, and installs the updater quit compatibility guard
// before electron-updater can start its native Squirrel quit handshake.
const { BrowserWindow, autoUpdater: nativeAutoUpdater } = require('electron');
const nativeCapabilities = require('./native-capability-handlers.cjs');
const { wrapNativeCapabilityHandlers } = require('./credential-gateway.cjs');
const { installUpdateQuitGuard } = require('./update-quit-guard.cjs');

installUpdateQuitGuard({ BrowserWindow, nativeAutoUpdater });

nativeCapabilities.createNativeCapabilityHandlers = wrapNativeCapabilityHandlers(
  nativeCapabilities.createNativeCapabilityHandlers,
);

require('./main.cjs');
