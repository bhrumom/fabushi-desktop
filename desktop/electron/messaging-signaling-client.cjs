'use strict';

const net = require('node:net');
const tls = require('node:tls');

const MAX_SIGNAL_FRAME_BYTES = 1024 * 1024;
const LOOPBACK_HOSTS = new Set(['127.0.0.1', '::1', 'localhost']);

function parseEndpoint(value) {
  const raw = String(value || 'tcp://127.0.0.1:9410').trim();
  let url;
  try { url = new URL(raw); } catch { throw new Error('Invalid Fabushi call signaling endpoint.'); }
  if (!['tcp:', 'tls:'].includes(url.protocol) || url.username || url.password || (url.pathname && url.pathname !== '/') || url.search || url.hash) {
    throw new Error('Unsupported Fabushi call signaling endpoint.');
  }
  const host = url.hostname;
  const port = Number(url.port || (url.protocol === 'tls:' ? 443 : 9410));
  if (!host || !Number.isInteger(port) || port < 1 || port > 65535) throw new Error('Invalid Fabushi call signaling host or port.');
  if (url.protocol === 'tcp:' && !LOOPBACK_HOSTS.has(host)) {
    throw new Error('Remote Fabushi call signaling requires tls:// transport.');
  }
  return { secure: url.protocol === 'tls:', host, port };
}

class MessagingSignalingClient {
  constructor({ onSignal, onStatus }) {
    this.onSignal = typeof onSignal === 'function' ? onSignal : () => {};
    this.onStatus = typeof onStatus === 'function' ? onStatus : () => {};
    this.socket = null;
    this.buffer = '';
    this.ready = false;
    this.actorId = null;
    this.connectPromise = null;
  }

  async connect(endpoint, hello) {
    this.disconnect('reconnect');
    const parsed = parseEndpoint(endpoint);
    const actorId = String(hello?.actorId || '').trim();
    const deviceId = String(hello?.deviceId || '').trim();
    const sessionId = String(hello?.sessionId || '').trim();
    const accessToken = String(hello?.accessToken || '');
    if (!actorId || !deviceId || !sessionId || accessToken.length < 32) throw new Error('Invalid Fabushi call signaling identity.');

    this.buffer = '';
    this.ready = false;
    this.actorId = actorId;
    this.connectPromise = new Promise((resolve, reject) => {
      let settled = false;
      const socket = parsed.secure
        ? tls.connect({ host: parsed.host, port: parsed.port, servername: parsed.host, rejectUnauthorized: true })
        : net.connect({ host: parsed.host, port: parsed.port });
      this.socket = socket;
      socket.setNoDelay(true);
      socket.setTimeout(30_000);
      const fail = (error) => {
        if (!settled) { settled = true; reject(error); }
        this.onStatus({ state: 'failed', message: error instanceof Error ? error.message : String(error) });
      };
      const writeHello = () => {
        this.onStatus({ state: 'authenticating' });
        socket.write(`${JSON.stringify({ type: 'hello', accessToken, actorId, deviceId, sessionId })}\n`);
      };
      if (parsed.secure) socket.on('secureConnect', writeHello);
      else socket.on('connect', writeHello);
      socket.on('data', (chunk) => {
        this.buffer += chunk.toString('utf8');
        if (Buffer.byteLength(this.buffer, 'utf8') > MAX_SIGNAL_FRAME_BYTES * 2) {
          socket.destroy(new Error('Fabushi call signaling receive buffer exceeded its limit.'));
          return;
        }
        for (;;) {
          const newline = this.buffer.indexOf('\n');
          if (newline < 0) break;
          const line = this.buffer.slice(0, newline);
          this.buffer = this.buffer.slice(newline + 1);
          if (!line.trim()) continue;
          if (Buffer.byteLength(line, 'utf8') > MAX_SIGNAL_FRAME_BYTES) {
            socket.destroy(new Error('Fabushi call signaling frame exceeded its limit.'));
            return;
          }
          let frame;
          try { frame = JSON.parse(line); } catch { socket.destroy(new Error('Fabushi call signaling returned invalid JSON.')); return; }
          if (frame?.type === 'ready') {
            this.ready = true;
            if (!settled) { settled = true; resolve({ actorId: String(frame.actorId || actorId), secure: parsed.secure }); }
            this.onStatus({ state: 'ready', actorId: String(frame.actorId || actorId), secure: parsed.secure });
          } else if (frame?.type === 'signal' && frame.signal && typeof frame.signal === 'object') {
            this.onSignal(frame.signal);
          } else if (frame?.type === 'error') {
            const error = new Error(String(frame.message || frame.code || 'Fabushi call signaling rejected the request.'));
            fail(error);
          }
        }
      });
      socket.on('timeout', () => socket.destroy(new Error('Fabushi call signaling timed out.')));
      socket.on('error', fail);
      socket.on('close', () => {
        this.ready = false;
        if (!settled) { settled = true; reject(new Error('Fabushi call signaling closed before authentication.')); }
        this.onStatus({ state: 'closed' });
        if (this.socket === socket) this.socket = null;
      });
    });
    return this.connectPromise;
  }

  send(signal) {
    if (!this.socket || this.socket.destroyed || !this.ready) throw new Error('Fabushi call signaling is not connected.');
    const payload = JSON.stringify({ type: 'signal', signal });
    if (Buffer.byteLength(payload, 'utf8') > MAX_SIGNAL_FRAME_BYTES) throw new Error('Fabushi call signal exceeds its maximum size.');
    this.socket.write(`${payload}\n`);
    return true;
  }

  disconnect(reason = 'client_disconnect') {
    const socket = this.socket;
    this.socket = null;
    this.ready = false;
    this.buffer = '';
    this.actorId = null;
    if (socket && !socket.destroyed) socket.destroy();
    this.onStatus({ state: 'closed', reason });
  }
}

module.exports = { MessagingSignalingClient, parseEndpoint };
