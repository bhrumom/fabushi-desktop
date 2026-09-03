'use strict';

const { randomBytes } = require('node:crypto');
const { createServer } = require('node:http');
const fs = require('node:fs/promises');
const path = require('node:path');

const BRIDGE_VERSION = 1;
const DEFAULT_HOST = '127.0.0.1';
const DEFAULT_TIMEOUT_MS = 35_000;
const MAX_TIMEOUT_MS = 40_000;
const MAX_BODY_BYTES = 128 * 1024;
const MAX_RESULT_BYTES = 4 * 1024 * 1024;
const MAX_CONCURRENT_REQUESTS = 16;
const OPERATIONS = new Set(['status', 'snapshot', 'find', 'action', 'wait', 'assert']);
const REQUEST_ID = /^app-agent-[a-f0-9]{32}$/u;

function isLoopbackAddress(value) {
  const address = String(value || '').toLowerCase();
  return address === '127.0.0.1'
    || address === '::1'
    || address === '::ffff:127.0.0.1';
}

function bearerToken(request) {
  const value = String(request.headers.authorization || '');
  return value.toLowerCase().startsWith('bearer ') ? value.slice(7).trim() : '';
}

function constantTimeTokenEqual(left, right) {
  const a = Buffer.from(String(left));
  const b = Buffer.from(String(right));
  if (a.length !== b.length) return false;
  let difference = 0;
  for (let index = 0; index < a.length; index += 1) difference |= a[index] ^ b[index];
  return difference === 0;
}

function writeJson(response, status, payload) {
  const encoded = Buffer.from(JSON.stringify(payload));
  if (encoded.length > MAX_RESULT_BYTES) {
    response.writeHead(502, {
      'content-type': 'application/json; charset=utf-8',
      'cache-control': 'no-store',
      'x-content-type-options': 'nosniff',
    });
    response.end(JSON.stringify({ error: 'app_surface_result_too_large' }));
    return;
  }
  response.writeHead(status, {
    'content-type': 'application/json; charset=utf-8',
    'content-length': String(encoded.length),
    'cache-control': 'no-store',
    pragma: 'no-cache',
    'referrer-policy': 'no-referrer',
    'x-content-type-options': 'nosniff',
  });
  response.end(encoded);
}

async function readJsonBody(request) {
  const chunks = [];
  let size = 0;
  for await (const chunk of request) {
    size += chunk.length;
    if (size > MAX_BODY_BYTES) throw new Error('app_surface_request_too_large');
    chunks.push(chunk);
  }
  if (!chunks.length) return {};
  let value;
  try { value = JSON.parse(Buffer.concat(chunks).toString('utf8')); }
  catch { throw new Error('invalid_app_surface_json'); }
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('invalid_app_surface_body');
  const input = value.input ?? {};
  if (!input || typeof input !== 'object' || Array.isArray(input)) throw new Error('invalid_app_surface_input');
  return { input };
}

async function writePrivateJson(destination, value) {
  const directory = path.dirname(destination);
  await fs.mkdir(directory, { recursive: true, mode: 0o700 });
  await fs.chmod(directory, 0o700).catch(() => {});
  const temporary = `${destination}.${process.pid}.${Date.now()}.${randomBytes(4).toString('hex')}.tmp`;
  await fs.writeFile(temporary, `${JSON.stringify(value, null, 2)}\n`, { encoding: 'utf8', mode: 0o600, flag: 'wx' });
  await fs.chmod(temporary, 0o600).catch(() => {});
  await fs.rename(temporary, destination);
  await fs.chmod(destination, 0o600).catch(() => {});
}

async function removeOwnedDiscoveryFile(destination, token) {
  try {
    const metadata = await fs.lstat(destination);
    if (!metadata.isFile() || metadata.isSymbolicLink() || metadata.size > 64 * 1024) return;
    const value = JSON.parse(await fs.readFile(destination, 'utf8'));
    if (value?.pid !== process.pid || !constantTimeTokenEqual(value?.token, token)) return;
    await fs.rm(destination, { force: true });
  } catch (error) {
    if (error?.code !== 'ENOENT') throw error;
  }
}

function createAppAgentSurfaceServer(options = {}) {
  const host = String(options.host || DEFAULT_HOST);
  if (host !== '127.0.0.1' && host !== '::1') throw new Error('Fabushi App Agent Surface must bind to loopback.');
  const requestedPort = Number(options.port ?? 0);
  if (!Number.isInteger(requestedPort) || requestedPort < 0 || requestedPort > 65535) throw new Error('Invalid App Agent Surface port.');
  const discoveryPath = path.resolve(String(options.discoveryPath || ''));
  if (!options.discoveryPath || discoveryPath === path.parse(discoveryPath).root) throw new Error('A private App Agent Surface discovery path is required.');
  const onRequest = options.onRequest;
  if (typeof onRequest !== 'function') throw new TypeError('App Agent Surface onRequest callback is required.');
  const authorize = typeof options.authorize === 'function' ? options.authorize : () => true;
  const timeoutMs = Math.max(1_000, Math.min(Number(options.timeoutMs || DEFAULT_TIMEOUT_MS), MAX_TIMEOUT_MS));
  const token = randomBytes(48).toString('base64url');
  const pending = new Map();
  let listening = false;
  let activeRequests = 0;
  let origin = '';

  function settle(requestId, reply) {
    if (!REQUEST_ID.test(String(requestId || ''))) return false;
    const entry = pending.get(requestId);
    if (!entry) return false;
    pending.delete(requestId);
    clearTimeout(entry.timer);
    activeRequests = Math.max(0, activeRequests - 1);
    entry.resolve(reply);
    return true;
  }

  function rejectAll(reason) {
    for (const [requestId, entry] of pending) {
      pending.delete(requestId);
      clearTimeout(entry.timer);
      entry.resolve({ ok: false, error: reason });
    }
    activeRequests = 0;
  }

  const httpServer = createServer(async (request, response) => {
    try {
      if (!isLoopbackAddress(request.socket.remoteAddress)) return writeJson(response, 403, { error: 'loopback_only' });
      if (!constantTimeTokenEqual(bearerToken(request), token)) return writeJson(response, 401, { error: 'unauthorized' });
      const url = new URL(request.url || '/', origin || `http://${host}`);
      if (request.method === 'GET' && url.pathname === '/health') {
        return writeJson(response, 200, { ok: true, version: BRIDGE_VERSION, appId: 'fabushi.desktop' });
      }
      const match = /^\/v1\/(status|snapshot|find|action|wait|assert)$/u.exec(url.pathname);
      if (request.method !== 'POST' || !match) return writeJson(response, 404, { error: 'not_found' });
      if (activeRequests >= MAX_CONCURRENT_REQUESTS) return writeJson(response, 429, { error: 'app_surface_busy' });
      const { input } = await readJsonBody(request);
      const operation = match[1];
      if (!OPERATIONS.has(operation)) return writeJson(response, 404, { error: 'not_found' });
      const decision = await Promise.resolve(authorize(operation));
      const allowed = decision === true || decision?.allowed === true;
      if (!allowed) return writeJson(response, 403, { error: 'app_surface_policy_denied' });
      const requestId = `app-agent-${randomBytes(16).toString('hex')}`;
      activeRequests += 1;
      const result = await new Promise((resolve) => {
        const timer = setTimeout(() => {
          pending.delete(requestId);
          activeRequests = Math.max(0, activeRequests - 1);
          resolve({ ok: false, error: 'app_surface_renderer_timeout' });
        }, timeoutMs);
        timer.unref?.();
        pending.set(requestId, { resolve, timer });
        try {
          onRequest(Object.freeze({
            version: BRIDGE_VERSION,
            requestId,
            operation,
            input,
            deadlineAt: Date.now() + timeoutMs,
          }));
        } catch (error) {
          settle(requestId, { ok: false, error: error instanceof Error ? error.message : String(error) });
        }
      });
      if (result?.ok === true) return writeJson(response, 200, { ok: true, result: result.result });
      const error = String(result?.error || 'app_surface_failed').slice(0, 1000);
      const status = error.includes('timeout') ? 504 : error.includes('unavailable') ? 503 : 409;
      return writeJson(response, status, { ok: false, error });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      const status = message.includes('too_large') ? 413 : message.startsWith('invalid_') ? 400 : 500;
      return writeJson(response, status, { error: message.slice(0, 1000) });
    }
  });
  httpServer.maxHeadersCount = 32;
  httpServer.headersTimeout = 10_000;
  httpServer.requestTimeout = MAX_TIMEOUT_MS + 5_000;
  httpServer.keepAliveTimeout = 5_000;

  return Object.freeze({
    version: BRIDGE_VERSION,
    discoveryPath,
    get origin() { return origin; },
    get activeRequests() { return activeRequests; },
    async start() {
      if (listening) return { origin, discoveryPath };
      await new Promise((resolve, reject) => {
        httpServer.once('error', reject);
        httpServer.listen(requestedPort, host, () => {
          httpServer.off('error', reject);
          listening = true;
          resolve();
        });
      });
      const address = httpServer.address();
      const port = typeof address === 'object' && address ? address.port : requestedPort;
      origin = `http://${host === '::1' ? '[::1]' : host}:${port}`;
      await writePrivateJson(discoveryPath, {
        version: BRIDGE_VERSION,
        appId: 'fabushi.desktop',
        origin,
        token,
        pid: process.pid,
        createdAt: new Date().toISOString(),
      });
      return { origin, discoveryPath };
    },
    respond(payload) {
      const value = payload && typeof payload === 'object' && !Array.isArray(payload) ? payload : {};
      const requestId = String(value.requestId || '');
      if (value.ok === true) return settle(requestId, { ok: true, result: value.result });
      return settle(requestId, { ok: false, error: String(value.error || 'app_surface_failed').slice(0, 1000) });
    },
    async close() {
      rejectAll('app_surface_unavailable');
      if (listening) {
        // Shutdown must not wait forever for an external keep-alive socket. In
        // particular, an Electron updater quit is a time-sensitive replacement
        // handshake and cannot be blocked by this optional loopback bridge.
        httpServer.closeAllConnections?.();
        await new Promise((resolve) => httpServer.close(resolve));
      }
      listening = false;
      await removeOwnedDiscoveryFile(discoveryPath, token);
    },
  });
}

module.exports = {
  BRIDGE_VERSION,
  MAX_BODY_BYTES,
  MAX_CONCURRENT_REQUESTS,
  createAppAgentSurfaceServer,
  isLoopbackAddress,
};
