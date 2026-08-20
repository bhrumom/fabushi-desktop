const { app } = require('electron');
const { spawn } = require('node:child_process');
const path = require('node:path');
const readline = require('node:readline');

const DEFAULT_DESKTOP_PRODUCT_API_BASE_URL = 'https://mahayana-platform.bhrumom.workers.dev';

class MahayanaHostProcess {
  constructor() {
    this.child = null;
    this.nextId = 1;
    this.pending = new Map();
  }

  executablePath() {
    if (process.env.MAHAYANA_APP_HOST_BIN) {
      return process.env.MAHAYANA_APP_HOST_BIN;
    }
    if (app.isPackaged) {
      const name = process.platform === 'win32' ? 'mahayana-app-host.exe' : 'mahayana-app-host';
      return path.join(process.resourcesPath, 'bin', name);
    }
    const name = process.platform === 'win32' ? 'mahayana-app-host.exe' : 'mahayana-app-host';
    return path.resolve(__dirname, '..', '..', 'third_party', 'mahayana', 'mahayana-rs', 'target', 'release', name);
  }

  start() {
    if (this.child) return;
    const child = spawn(this.executablePath(), [], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: {
        ...process.env,
        MAHAYANA_API_BASE_URL: process.env.MAHAYANA_API_BASE_URL?.trim() || DEFAULT_DESKTOP_PRODUCT_API_BASE_URL,
        FABUSHI_APP_DATA: app.getPath('userData'),
      },
      windowsHide: true,
    });
    this.child = child;

    const lines = readline.createInterface({ input: child.stdout });
    lines.on('line', (line) => {
      let message;
      try {
        message = JSON.parse(line);
      } catch (error) {
        this.rejectAll(new Error(`Invalid Mahayana host response: ${error}`));
        return;
      }
      const key = String(message.id ?? '');
      const pending = this.pending.get(key);
      if (!pending) return;
      this.pending.delete(key);
      if (message.ok) pending.resolve(message.result);
      else pending.reject(new Error(message.error || 'Mahayana host request failed'));
    });
    child.stderr.on('data', (chunk) => console.error(`[mahayana-app-host] ${String(chunk).trimEnd()}`));
    child.on('error', (error) => this.rejectAll(error));
    child.on('exit', (code, signal) => {
      this.child = null;
      this.rejectAll(new Error(`Mahayana host exited (${code ?? 'null'}, ${signal ?? 'none'})`));
    });
  }

  request(method, params = {}, timeoutMs = 120000) {
    this.start();
    const id = this.nextId++;
    const key = String(id);
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(key);
        reject(new Error(`Mahayana host request timed out: ${method}`));
      }, timeoutMs);
      timer.unref?.();
      this.pending.set(key, {
        resolve: (value) => { clearTimeout(timer); resolve(value); },
        reject: (error) => { clearTimeout(timer); reject(error); },
      });
      const payload = JSON.stringify({ id, method, params });
      this.child.stdin.write(`${payload}\n`, (error) => {
        if (!error) return;
        const pending = this.pending.get(key);
        this.pending.delete(key);
        pending?.reject(error);
      });
    });
  }

  rejectAll(error) {
    for (const pending of this.pending.values()) pending.reject(error);
    this.pending.clear();
  }

  close() {
    if (!this.child) return;
    this.child.kill();
    this.child = null;
  }
}

module.exports = { MahayanaHostProcess };
