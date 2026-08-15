const { app, BrowserWindow, dialog, ipcMain, net, Notification, protocol, shell, session } = require('electron');
const path = require('node:path');
const { pathToFileURL, URL } = require('node:url');
const { MahayanaHostProcess } = require('./host-process.cjs');

protocol.registerSchemesAsPrivileged([
  {
    scheme: 'app',
    privileges: { standard: true, secure: true, supportFetchAPI: true },
  },
]);

const host = new MahayanaHostProcess();
const allowedHostMethods = new Set([
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

function safeHttpsUrl(value) {
  const parsed = new URL(String(value));
  if (parsed.protocol !== 'https:' || !parsed.hostname) throw new Error('Only HTTPS URLs may be opened externally.');
  return parsed.toString();
}

function installIpcHandlers() {
  ipcMain.handle('fabushi:host', async (event, request) => {
    assertTrustedSender(event);
    const method = String(request?.method || '');
    if (!allowedHostMethods.has(method)) throw new Error(`Host method is not allowed: ${method}`);
    return host.request(method, request?.params && typeof request.params === 'object' ? request.params : {});
  });

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
    },
  });

  win.webContents.setWindowOpenHandler(({ url }) => {
    try { void shell.openExternal(safeHttpsUrl(url)); } catch {}
    return { action: 'deny' };
  });
  win.webContents.on('will-navigate', (event, url) => {
    if (!isTrustedRendererUrl(url)) event.preventDefault();
  });
  win.once('ready-to-show', () => win.show());

  if (process.env.VITE_DEV_SERVER_URL) {
    void win.loadURL(process.env.VITE_DEV_SERVER_URL);
  } else {
    void win.loadURL('app://bundle/index.html');
  }
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
}

app.whenReady().then(() => {
  installAppProtocol();
  session.defaultSession.setPermissionRequestHandler((_webContents, _permission, callback) => callback(false));
  installIpcHandlers();
  host.start();
  createWindow();
  app.on('activate', () => { if (BrowserWindow.getAllWindows().length === 0) createWindow(); });
});

app.on('window-all-closed', () => { if (process.platform !== 'darwin') app.quit(); });
app.on('before-quit', () => host.close());
