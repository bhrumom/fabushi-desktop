'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { EventEmitter } = require('node:events');
const {
  COORDINATOR_PROTOCOL_VERSION,
  COORDINATOR_DATA_CHANNEL,
  bindCoordinatorRendererPort,
} = require('./coordinator-renderer-port.cjs');

class FakePort extends EventEmitter {
  constructor() {
    super();
    this.sent = [];
    this.started = false;
    this.closed = false;
  }
  postMessage(frame) { this.sent.push(frame); }
  start() { this.started = true; }
  close() {
    if (this.closed) return;
    this.closed = true;
  }
  receive(data) { this.emit('message', { data }); }
}

class FakeHost {
  constructor() {
    this.events = new EventEmitter();
    this.requests = [];
  }
  onCoordinatorEvent(listener) {
    this.events.on('coordinator-event', listener);
    return () => this.events.off('coordinator-event', listener);
  }
  onLifecycle(listener) {
    this.events.on('lifecycle', listener);
    return () => this.events.off('lifecycle', listener);
  }
  async request(method, args) {
    this.requests.push({ method, args });
    if (method === 'explode') {
      const error = new Error('boom');
      error.code = 'BOOM';
      throw error;
    }
    return { method, args, routed: true };
  }
}

test('renderer port handoff speaks the production Coordinator lifecycle and request protocol', async () => {
  const port = new FakePort();
  const host = new FakeHost();
  bindCoordinatorRendererPort({ port, host });
  assert.equal(port.started, true);

  port.receive({ kind: 'lifecycle', phase: 'hello', protocolVersion: COORDINATOR_PROTOCOL_VERSION });
  assert.deepEqual(port.sent[0], { kind: 'lifecycle', phase: 'ready', protocolVersion: COORDINATOR_PROTOCOL_VERSION });

  port.receive({ kind: 'request', requestId: 'r-1', method: 'listAgents', args: { limit: 3 } });
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(host.requests, [{ method: 'listAgents', args: { limit: 3 } }]);
  assert.deepEqual(port.sent.at(-1), {
    kind: 'reply',
    requestId: 'r-1',
    outcome: { status: 'ok', value: { method: 'listAgents', args: { limit: 3 }, routed: true } },
  });
});

test('real Coordinator events are projected to the Renderer port and failures keep request identity', async () => {
  const port = new FakePort();
  const host = new FakeHost();
  bindCoordinatorRendererPort({ port, host });
  port.receive({ kind: 'lifecycle', phase: 'hello', protocolVersion: COORDINATOR_PROTOCOL_VERSION });

  host.events.emit('coordinator-event', {
    channel: COORDINATOR_DATA_CHANNEL,
    family: 'agents',
    payload: [{ id: 'agent:1' }],
  });
  assert.deepEqual(port.sent.at(-1), { kind: 'event', family: 'agents', payload: [{ id: 'agent:1' }] });

  port.receive({ kind: 'request', requestId: 'r-fail', method: 'explode', args: {} });
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(port.sent.at(-1), {
    kind: 'reply',
    requestId: 'r-fail',
    outcome: { status: 'failed', failure: { code: 'BOOM', message: 'boom' } },
  });
});
