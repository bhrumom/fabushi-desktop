'use strict';

const { spawn } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { embeddedComputerControlEnvironment } = require('./host-process.cjs');

const OFFICIAL_DEVICE_GATEWAY_URL = 'wss://fabushi-mcp.ombhrum.com/agent';
const SESSION_POLL_MS = 30_000;
const RETRY_MS = 5_000;

function remoteDeviceGatewayUrl(app, env = process.env) {
  if (app.isPackaged) return OFFICIAL_DEVICE_GATEWAY_URL;
  const value = String(env.FABUSHI_REMOTE_DEVICE_GATEWAY_URL || '').trim();
  if (!value) return null;
  const url = new URL(value);
  if (url.protocol !== 'wss:' || url.pathname !== '/agent' || url.username || url.password || url.search || url.hash) {
    throw new Error('FABUSHI_REMOTE_DEVICE_GATEWAY_URL must be a clean wss:// origin ending in /agent.');
  }
  return url.toString();
}

function directComputerRuntimeEnvironment(environment = {}) {
  const mappings = [
    ['MAHAYANA_COMPUTER_MCP_HOME', 'CHATGPT_COMPUTER_HOME'],
    ['MAHAYANA_COMPUTER_MCP_NATIVE_HELPER', 'CHATGPT_COMPUTER_NATIVE_HELPER'],
    ['MAHAYANA_COMPUTER_MCP_MAC_APP_DIR', 'CHATGPT_COMPUTER_MAC_APP_DIR'],
  ];
  const result = {};
  for (const [source, destination] of mappings) {
    const value = String(environment[source] || '').trim();
    if (value) result[destination] = value;
  }
  return result;
}

function remoteDeviceRuntime(options = {}) {
  const environment = embeddedComputerControlEnvironment(options);
  const mcpEntry = String(environment.MAHAYANA_COMPUTER_MCP_ENTRY || '');
  if (!mcpEntry) return null;
  const root = path.dirname(path.dirname(mcpEntry));
  const agentEntry = path.join(root, 'bin', 'fabushi-device-agent.js');
  const fsImpl = options.fs ?? fs;
  try {
    if (!fsImpl.statSync(agentEntry).isFile()) return null;
  } catch {
    return null;
  }
  return {
    root,
    mcpEntry,
    agentEntry,
    childEnvironment: directComputerRuntimeEnvironment(environment),
  };
}

function validAgentSession(value) {
  const accessToken = String(value?.accessToken || '').trim();
  const deviceId = String(value?.deviceId || '').trim();
  const sessionId = String(value?.sessionId || '').trim();
  if (accessToken.length < 24 || accessToken.length > 16 * 1024 || /\s/u.test(accessToken)) return null;
  if (!/^[a-zA-Z0-9._:-]{1,128}$/u.test(deviceId)) return null;
  if (!sessionId || sessionId.length > 200) return null;
  return {
    accessToken,
    deviceId,
    sessionId,
    username: String(value?.username || value?.user?.username || '').trim().slice(0, 200),
    expiresAt: Number(value?.accessTokenExpiresAt || 0),
  };
}

function writePrivateToken(fsImpl, destination, token) {
  const directory = path.dirname(destination);
  fsImpl.mkdirSync(directory, { recursive: true, mode: 0o700 });
  try { fsImpl.chmodSync(directory, 0o700); } catch {}
  const temporary = `${destination}.${process.pid}.${Date.now()}.tmp`;
  fsImpl.writeFileSync(temporary, `${token}\n`, { encoding: 'utf8', mode: 0o600, flag: 'wx' });
  try { fsImpl.chmodSync(temporary, 0o600); } catch {}
  fsImpl.renameSync(temporary, destination);
  try { fsImpl.chmodSync(destination, 0o600); } catch {}
}

class RemoteDeviceAgentSupervisor {
  constructor(options) {
    this.host = options.host;
    this.app = options.app;
    this.env = options.env ?? process.env;
    this.fs = options.fs ?? fs;
    this.spawn = options.spawn ?? spawn;
    this.platform = options.platform ?? process.platform;
    this.resourcesPath = options.resourcesPath ?? process.resourcesPath;
    this.execPath = options.execPath ?? process.execPath;
    this.timer = null;
    this.child = null;
    this.activeKey = '';
    this.closed = false;
    this.syncing = false;
    this.tokenFile = path.join(this.app.getPath('userData'), 'remote-device', 'account-access-token');
  }

  start() {
    if (this.closed) throw new Error('Fabushi remote device supervisor is closed.');
    this.schedule(0);
  }

  schedule(delay = SESSION_POLL_MS) {
    if (this.closed || this.timer) return;
    this.timer = setTimeout(() => {
      this.timer = null;
      void this.sync();
    }, delay);
    this.timer.unref?.();
  }

  stopAgent() {
    const child = this.child;
    this.child = null;
    this.activeKey = '';
    child?.kill();
    try { this.fs.rmSync(this.tokenFile, { force: true }); } catch {}
  }

  async sync() {
    if (this.closed || this.syncing) return;
    this.syncing = true;
    try {
      const gatewayUrl = remoteDeviceGatewayUrl(this.app, this.env);
      const runtime = gatewayUrl ? remoteDeviceRuntime({
        app: this.app,
        env: this.env,
        platform: this.platform,
        resourcesPath: this.resourcesPath,
        fs: this.fs,
        execPath: this.execPath,
      }) : null;
      if (!gatewayUrl || !runtime) {
        this.stopAgent();
        return;
      }

      const session = validAgentSession(await this.host.request('feature.auth.deviceAgentSession', {}, 30_000));
      if (!session) throw new Error('Fabushi account did not return a valid remote-device session.');
      const key = `${session.deviceId}\0${session.sessionId}\0${session.accessToken}`;
      if (this.child && this.activeKey === key) return;

      this.stopAgent();
      writePrivateToken(this.fs, this.tokenFile, session.accessToken);
      const policyFile = String(this.env.FABUSHI_COMPUTER_POLICY_FILE || '').trim()
        || path.join(this.app.getPath('userData'), 'feature-host', 'runtime', 'settings.json');
      const discoveryFile = String(this.env.FABUSHI_APP_AGENT_DISCOVERY_FILE || '').trim()
        || path.join(this.app.getPath('userData'), 'agent-surface', 'bridge.json');
      const child = this.spawn(this.execPath, [runtime.agentEntry], {
        stdio: ['ignore', 'pipe', 'pipe'],
        windowsHide: true,
        env: {
          ...this.env,
          ...runtime.childEnvironment,
          ELECTRON_RUN_AS_NODE: '1',
          RUNNER_TRACKING_ID: '',
          DEVICE_GATEWAY_URL: gatewayUrl,
          DEVICE_GATEWAY_TOKEN: '',
          FABUSHI_ACCOUNT_ACCESS_TOKEN: '',
          FABUSHI_ACCOUNT_SESSION_FILE: '',
          FABUSHI_ACCOUNT_TOKEN_FILE: this.tokenFile,
          DEVICE_ID: session.deviceId,
          DEVICE_NAME: String(this.env.DEVICE_NAME || `Fabushi on ${os.hostname()}`).slice(0, 200),
          DEVICE_KIND: this.env.GITHUB_ACTIONS === 'true' ? 'github-actions' : 'fabushi-desktop',
          DEVICE_LEASE_SECONDS: String(this.env.DEVICE_LEASE_SECONDS || '14400'),
          DEVICE_LOCAL_MCP_COMMAND: this.execPath,
          DEVICE_LOCAL_MCP_ENTRY: runtime.mcpEntry,
          DEVICE_LOCAL_MCP_CWD: runtime.root,
          DEVICE_LOCAL_MCP_ELECTRON_NODE: '1',
          FABUSHI_COMPUTER_POLICY_FILE: policyFile,
          FABUSHI_APP_AGENT_DISCOVERY_FILE: discoveryFile,
        },
      });
      this.child = child;
      this.activeKey = key;
      child.stdout?.on('data', (chunk) => console.info(`[fabushi-remote-device] ${String(chunk).trimEnd()}`));
      child.stderr?.on('data', (chunk) => console.error(`[fabushi-remote-device] ${String(chunk).trimEnd()}`));
      child.on('error', (error) => console.error('[fabushi-remote-device] agent error', error));
      child.on('exit', (code, signal) => {
        if (this.child !== child) return;
        this.child = null;
        this.activeKey = '';
        try { this.fs.rmSync(this.tokenFile, { force: true }); } catch {}
        if (!this.closed) {
          console.error(`[fabushi-remote-device] agent exited (${code ?? 'null'}, ${signal ?? 'none'})`);
          this.schedule(RETRY_MS);
        }
      });
    } catch (error) {
      this.stopAgent();
      if (!/not logged in|notloggedin|missing account|session expired/iu.test(String(error?.message || error))) {
        console.error('[fabushi-remote-device] session sync failed', error);
      }
    } finally {
      this.syncing = false;
      this.schedule();
    }
  }

  close() {
    this.closed = true;
    if (this.timer) clearTimeout(this.timer);
    this.timer = null;
    this.stopAgent();
  }
}

module.exports = {
  OFFICIAL_DEVICE_GATEWAY_URL,
  RemoteDeviceAgentSupervisor,
  remoteDeviceGatewayUrl,
  remoteDeviceRuntime,
  validAgentSession,
};
