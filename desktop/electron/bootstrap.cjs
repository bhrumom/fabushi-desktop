'use strict';

// Keep electron/main.cjs as the canonical desktop runtime. This bootstrap only
// wraps the existing native capability factory before main loads so secrets can
// be resolved and injected in the trusted main process without exposing
// plaintext through the renderer IPC surface.
const nativeCapabilities = require('./native-capability-handlers.cjs');
const { wrapNativeCapabilityHandlers } = require('./credential-gateway.cjs');

nativeCapabilities.createNativeCapabilityHandlers = wrapNativeCapabilityHandlers(
  nativeCapabilities.createNativeCapabilityHandlers,
);

require('./main.cjs');
