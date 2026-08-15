const { contextBridge, ipcRenderer } = require('electron');

const allowedMethods = new Set([
  'host.platform',
  'feature.info',
  'feature.execute',
  'feature.receive',
  'feature.approval.resolve',
  'feature.interrupt',
  'feature.auth.status',
  'feature.auth.providers',
  'feature.auth.passwordLogin',
  'feature.auth.oauthStart',
  'feature.auth.oauthPoll',
  'feature.auth.logout',
  'marketplace.browse',
  'marketplace.release',
  'plugin.install',
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
]);

contextBridge.exposeInMainWorld('fabushi', Object.freeze({
  invoke(method, params = {}) {
    if (!allowedMethods.has(method)) {
      return Promise.reject(new Error(`Host method is not allowed: ${method}`));
    }
    return ipcRenderer.invoke('fabushi:host', { method, params });
  },
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
}));
