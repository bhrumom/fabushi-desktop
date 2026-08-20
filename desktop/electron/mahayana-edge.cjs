'use strict';

const { defineEdge } = require('./edge-ipc.cjs');

const methodNames = [
  'host.platform',
  'feature.info',
  'feature.execute',
  'feature.receive',
  'feature.approval.resolve',
  'feature.interrupt',
  'feature.auth.status',
  'feature.auth.providers',
  'feature.auth.passwordLogin',
  'feature.auth.browserStart',
  'feature.auth.browserPoll',
  'feature.auth.browserCancel',
  'feature.auth.browserReopen',
  'feature.auth.oauthStart',
  'feature.auth.oauthPoll',
  'feature.auth.logout',
  'marketplace.browse',
  'marketplace.release',
  'plugin.install',
  'plugin.uninstall',
  'plugin.active',
  'plugin.permissions',
  'plugin.permission.grant',
  'plugin.permission.revoke',
  'plugin.compatibility',
  'plugin.uiDocument',
  'runtime.start',
  'runtime.stop',
  'runtime.tools',
  'runtime.callTool',
];

const methods = Object.fromEntries(methodNames.map((name) => [name, { args: 'object' }]));
const MAHAYANA_EDGE = defineEdge('mahayana-host', methods, ['runtime-event']);

module.exports = { MAHAYANA_EDGE };
