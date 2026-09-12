const crypto = require('node:crypto');
const fs = require('node:fs');
const fsp = require('node:fs/promises');
const net = require('node:net');
const os = require('node:os');
const path = require('node:path');
const { execFile } = require('node:child_process');
const { promisify } = require('node:util');

const execFileAsync = promisify(execFile);
const MAX_LINE_BYTES = 2 * 1024 * 1024;
const ALLOWED_METHODS = new Set([
  'feature.info',
  'feature.auth.status',
  'feature.execute',
  'feature.marketplace.browse',
  'feature.marketplace.release',
  'feature.plugin.install',
  'feature.plugin.uninstall',
  'feature.plugin.active',
  'feature.plugin.listInstalled',
  'feature.plugin.uiDocument',
]);
const FORBIDDEN_EXTENSION_COMMANDS = new Set([
  'secret.provide',
  'session.clear',
  'update.install',
]);

function homePath(env = process.env) {
  return String(env.COMPUTER_BROWSER_EXTENSION_HOME || '').trim()
    || path.join(os.homedir(), '.chatgpt-computer-control', 'browser-extension');
}

function platformSocketPath(env = process.env, platform = process.platform) {
  const configured = String(env.FABUSHI_CHROME_PLATFORM_SOCKET || '').trim();
  if (configured) return configured;
  if (platform === 'win32') {
    const key = crypto.createHash('sha256').update(os.userInfo().username).digest('hex').slice(0, 12);
    return `\\\\.\\pipe\\fabushi-chrome-platform-${key}`;
  }
  return path.join(homePath(env), 'fabushi-platform.sock');
}

function secretPath(env = process.env) {
  return path.join(homePath(env), 'native-host.secret');
}

function safeEqual(left, right) {
  const a = Buffer.from(String(left ?? ''));
  const b = Buffer.from(String(right ?? ''));
  return a.length === b.length && crypto.timingSafeEqual(a, b);
}

async function ensureSecret(env) {
  const root = homePath(env);
  const file = secretPath(env);
  await fsp.mkdir(root, { recursive: true, mode: 0o700 });
  try {
    const current = (await fsp.readFile(file, 'utf8')).trim();
    if (current.length >= 32) return current;
  } catch {}
  const secret = crypto.randomBytes(32).toString('base64url');
  await fsp.writeFile(file, `${secret}\n`, { encoding: 'utf8', mode: 0o600 });
  return secret;
}

async function installedExtensionId(env) {
  const configured = String(env.FABUSHI_CHROME_EXTENSION_ID || '').trim();
  if (/^[a-p]{32}$/.test(configured)) return configured;
  try {
    const metadata = JSON.parse(await fsp.readFile(path.join(homePath(env), 'install.json'), 'utf8'));
    const extensionId = String(metadata?.extensionId || '');
    return metadata?.publishedExtensionId === extensionId && /^[a-p]{32}$/.test(extensionId) ? extensionId : '';
  } catch {
    return '';
  }
}

async function socketIsListening(socketPath) {
  return new Promise((resolve) => {
    const socket = net.connect(socketPath);
    socket.once('connect', () => { socket.destroy(); resolve(true); });
    socket.once('error', () => resolve(false));
  });
}

async function openDesktopSettings(section, platform = process.platform) {
  const selected = ['general', 'mcp', 'usage', 'updates'].includes(String(section || '')) ? String(section) : 'general';
  const target = `fabushi://settings/${selected}`;
  if (platform === 'darwin') {
    await execFileAsync('open', [target]);
    return { opened: true, section: selected };
  }
  if (platform === 'win32') {
    await execFileAsync('cmd.exe', ['/d', '/s', '/c', 'start', '', target], { windowsHide: true });
    return { opened: true, section: selected };
  }
  await execFileAsync('xdg-open', [target]);
  return { opened: true, section: selected };
}

function sanitizeRequest(method, params) {
  const normalizedMethod = String(method || '');
  if (!ALLOWED_METHODS.has(normalizedMethod)) throw new Error(`Chrome platform method is not allowed: ${normalizedMethod}`);
  const normalizedParams = params && typeof params === 'object' && !Array.isArray(params) ? { ...params } : {};
  if (normalizedMethod === 'feature.marketplace.browse') normalizedParams.platform = 'chrome-extension';
  if (normalizedMethod === 'feature.plugin.install') normalizedParams.platform = 'chrome-extension';
  if (normalizedMethod === 'feature.execute') {
    const command = normalizedParams.command;
    if (!command || typeof command !== 'object' || Array.isArray(command) || typeof command.type !== 'string') {
      throw new Error('feature.execute requires a valid Fabushi command.');
    }
    if (FORBIDDEN_EXTENSION_COMMANDS.has(command.type)) {
      throw new Error(`Chrome platform cannot execute ${command.type}; complete this action in the desktop app.`);
    }
  }
  return { method: normalizedMethod, params: normalizedParams };
}

class ChromePlatformServer {
  constructor({ app, hostRequest, env = process.env, platform = process.platform } = {}) {
    if (!app || typeof hostRequest !== 'function') throw new Error('ChromePlatformServer requires Electron app and Host request callback.');
    this.app = app;
    this.hostRequest = hostRequest;
    this.env = env;
    this.platform = platform;
    this.server = null;
    this.connections = new Set();
    this.startPromise = null;
  }

  async start() {
    if (this.server) return this.server;
    if (this.startPromise) return this.startPromise;
    this.startPromise = this.startInternal().finally(() => { this.startPromise = null; });
    return this.startPromise;
  }

  async startInternal() {
    const secret = await ensureSecret(this.env);
    const expectedExtensionId = await installedExtensionId(this.env);
    if (!expectedExtensionId) {
      throw new Error('Fabushi Chrome platform requires the published extension ID; run browser-extension install or set FABUSHI_CHROME_EXTENSION_ID.');
    }
    const socketPath = platformSocketPath(this.env, this.platform);
    if (this.platform !== 'win32') {
      const info = await fsp.stat(socketPath).catch(() => null);
      if (info?.isSocket()) {
        if (await socketIsListening(socketPath)) throw new Error('Another Fabushi Chrome platform server is already running for this user.');
        await fsp.rm(socketPath, { force: true });
      } else if (info) {
        throw new Error(`Fabushi Chrome platform IPC path is not a socket: ${socketPath}`);
      }
    }
    const server = net.createServer((socket) => this.accept(socket, secret, expectedExtensionId));
    await new Promise((resolve, reject) => {
      server.once('error', reject);
      server.listen(socketPath, () => { server.off('error', reject); resolve(); });
    });
    if (this.platform !== 'win32') await fsp.chmod(socketPath, 0o600);
    server.unref();
    server.on('close', () => { if (this.server === server) this.server = null; });
    this.server = server;
    return server;
  }

  accept(socket, secret, expectedExtensionId) {
    const connection = { socket, authenticated: false, extensionId: '', buffer: '' };
    socket.setEncoding('utf8');
    socket.on('data', (chunk) => {
      connection.buffer += chunk;
      if (Buffer.byteLength(connection.buffer, 'utf8') > MAX_LINE_BYTES) { socket.destroy(); return; }
      while (connection.buffer.includes('\n')) {
        const index = connection.buffer.indexOf('\n');
        const line = connection.buffer.slice(0, index);
        connection.buffer = connection.buffer.slice(index + 1);
        if (!line.trim()) continue;
        let message;
        try { message = JSON.parse(line); } catch { socket.destroy(); return; }
        if (!connection.authenticated) {
          if (message.type !== 'platform_hello'
              || !safeEqual(message.secret, secret)
              || !/^[a-p]{32}$/.test(String(message.extensionId || ''))
              || (expectedExtensionId && String(message.extensionId) !== expectedExtensionId)) {
            socket.destroy();
            return;
          }
          connection.authenticated = true;
          connection.extensionId = String(message.extensionId);
          this.connections.add(connection);
          this.send(connection, {
            type: 'platform_hello_ack',
            desktopVersion: String(this.app.getVersion?.() || ''),
            platform: this.platform,
          });
          continue;
        }
        if (message.type === 'platform_heartbeat') {
          this.send(connection, { type: 'platform_heartbeat_ack', timestamp: Date.now() });
          continue;
        }
        if (message.type === 'platform_request') void this.handleRequest(connection, message);
      }
    });
    socket.on('error', () => {});
    socket.on('close', () => this.connections.delete(connection));
  }

  send(connection, message) {
    if (!connection?.socket?.destroyed) connection.socket.write(`${JSON.stringify(message)}\n`);
  }

  async handleRequest(connection, message) {
    const requestId = String(message.requestId || '');
    if (!requestId || requestId.length > 200) { connection.socket.destroy(); return; }
    try {
      let result;
      if (message.method === 'desktop.status') {
        const auth = await this.hostRequest('feature.auth.status', {}, 15_000);
        result = {
          connected: true,
          auth,
          desktopVersion: String(this.app.getVersion?.() || ''),
          platform: this.platform,
          credentialBoundary: 'desktop-host',
        };
      } else if (message.method === 'desktop.settings.open') {
        result = await openDesktopSettings(message.params?.section, this.platform);
      } else {
        const request = sanitizeRequest(message.method, message.params);
        result = await this.hostRequest(request.method, request.params, 60_000);
      }
      this.send(connection, { type: 'platform_response', requestId, ok: true, result });
    } catch (error) {
      this.send(connection, { type: 'platform_response', requestId, ok: false, error: error instanceof Error ? error.message : String(error) });
    }
  }

  broadcastEvent(event) {
    if (!event || typeof event !== 'object') return;
    for (const connection of this.connections) this.send(connection, { type: 'platform_event', event });
  }

  async close() {
    for (const connection of this.connections) connection.socket.destroy();
    this.connections.clear();
    if (!this.server) return;
    const server = this.server;
    this.server = null;
    await new Promise((resolve) => server.close(resolve));
    if (this.platform !== 'win32') await fsp.rm(platformSocketPath(this.env, this.platform), { force: true });
  }
}

function createChromePlatformServer(options) {
  return new ChromePlatformServer(options);
}

module.exports = {
  ChromePlatformServer,
  createChromePlatformServer,
  homePath,
  openDesktopSettings,
  platformSocketPath,
  sanitizeRequest,
  secretPath,
};
