'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { parseEndpoint } = require('./messaging-signaling-client.cjs');

test('loopback tcp signaling is allowed', () => {
  assert.deepEqual(parseEndpoint('tcp://127.0.0.1:9410'), {
    secure: false,
    host: '127.0.0.1',
    port: 9410,
  });
});

test('remote signaling requires TLS', () => {
  assert.throws(
    () => parseEndpoint('tcp://call.example.com:9410'),
    /requires tls:\/\//,
  );
  assert.deepEqual(parseEndpoint('tls://call.example.com:443'), {
    secure: true,
    host: 'call.example.com',
    port: 443,
  });
});

test('credentials and unrelated schemes are rejected', () => {
  assert.throws(() => parseEndpoint('tls://user:pass@call.example.com:443'));
  assert.throws(() => parseEndpoint('https://call.example.com:443'));
});
