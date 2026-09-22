'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const {
  credentialFetch,
  normalizeBinding,
  wrapNativeCapabilityHandlers,
} = require('./credential-gateway.cjs');

const CANARY = 'fabushi-secret-canary-9f2e1a';

async function fixture() {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'fabushi-credential-gateway-'));
  const calls = [];
  const approvalCalls = [];
  const safeStorage = {
    isEncryptionAvailable: () => true,
    encryptString: (value) => Buffer.from(`sealed:${Buffer.from(value, 'utf8').toString('base64')}`, 'utf8'),
    decryptString: (bytes) => {
      const stored = Buffer.from(bytes).toString('utf8');
      assert.match(stored, /^sealed:/);
      return Buffer.from(stored.slice('sealed:'.length), 'base64').toString('utf8');
    },
  };
  const net = {
    fetch: async (url, options) => {
      calls.push({ url, options });
      return new Response(JSON.stringify({ ok: true }), {
        status: 200,
        headers: {
          'content-type': 'application/json',
          'set-cookie': 'should-not-cross-boundary=1',
        },
      });
    },
  };
  const dialog = {
    showMessageBox: async (options) => {
      approvalCalls.push(options);
      return { response: 1 };
    },
  };
  const app = { getPath: (name) => {
    assert.equal(name, 'userData');
    return root;
  } };
  const deps = { app, safeStorage, net, dialog };
  const originalCalls = [];
  const originalFactory = () => ({
    egressFetch: async (params) => {
      originalCalls.push(params);
      return { legacy: true };
    },
  });
  const handlers = wrapNativeCapabilityHandlers(originalFactory)(deps);
  return {
    root,
    deps,
    handlers,
    calls,
    approvalCalls,
    originalCalls,
    cleanup: () => fs.rm(root, { recursive: true, force: true }),
  };
}

function githubBinding() {
  return {
    label: 'GitHub production',
    allowedOrigins: ['https://api.github.com'],
    injection: { type: 'bearer' },
  };
}

test('normalizes credential bindings to exact HTTPS origins', () => {
  assert.deepEqual(normalizeBinding({
    label: ' Example ',
    origins: ['https://api.example.com/v1/path', 'https://api.example.com'],
    injection: { type: 'header', headerName: 'X-API-Key', prefix: 'Key ' },
  }), {
    version: 1,
    label: 'Example',
    allowedOrigins: ['https://api.example.com'],
    injection: { type: 'header', headerName: 'X-API-Key', prefix: 'Key ' },
  });
  assert.throws(() => normalizeBinding({ origin: 'http://api.example.com' }), /HTTPS/u);
});

test('stores ciphertext only and never exposes plaintext through list or reveal', async () => {
  const ctx = await fixture();
  try {
    const listed = await ctx.handlers.upsertSecrets({
      name: 'connector/github/default',
      value: CANARY,
      binding: githubBinding(),
    });
    assert.equal(listed.length, 1);
    assert.equal(listed[0].name, 'connector/github/default');
    assert.equal(listed[0].configured, true);
    assert.equal(listed[0].revealable, false);
    assert.equal(JSON.stringify(listed).includes(CANARY), false);

    const vaultText = await fs.readFile(path.join(ctx.root, 'secure', 'secrets.json'), 'utf8');
    assert.equal(vaultText.includes(CANARY), false);
    assert.match(vaultText, /ciphertext/u);
    await assert.rejects(() => ctx.handlers.revealSecret({ name: 'connector/github/default' }), /non-revealable/u);
  } finally {
    await ctx.cleanup();
  }
});

test('injects a bound bearer credential only at the final HTTPS hop', async () => {
  const ctx = await fixture();
  try {
    await ctx.handlers.upsertSecrets({
      name: 'connector/github/default',
      value: CANARY,
      binding: githubBinding(),
    });
    const result = await ctx.handlers.egressFetch({
      secretRef: 'connector/github/default',
      url: 'https://api.github.com/repos/bhrumom/fabushi',
      method: 'GET',
      headers: { Accept: 'application/json' },
    });
    assert.equal(ctx.calls.length, 1);
    assert.equal(ctx.approvalCalls.length, 0);
    assert.equal(ctx.calls[0].url, 'https://api.github.com/repos/bhrumom/fabushi');
    assert.equal(ctx.calls[0].options.redirect, 'manual');
    assert.equal(ctx.calls[0].options.headers.Authorization, `Bearer ${CANARY}`);
    assert.equal(ctx.calls[0].options.headers.Accept, 'application/json');
    assert.equal(JSON.stringify(result).includes(CANARY), false);
    assert.equal(result.credential.secretRef, 'connector/github/default');
    assert.equal(result.headers['set-cookie'], undefined);
  } finally {
    await ctx.cleanup();
  }
});

test('scrubs a malicious endpoint that echoes credential material in response headers and body', async () => {
  const ctx = await fixture();
  try {
    await ctx.handlers.upsertSecrets({
      name: 'connector/github/default',
      value: CANARY,
      binding: githubBinding(),
    });
    const encoded = Buffer.from(CANARY, 'utf8').toString('base64');
    ctx.deps.net.fetch = async (_url, options) => new Response(JSON.stringify({
      authorization: options.headers.Authorization,
      raw: CANARY,
      encoded,
    }), {
      status: 200,
      headers: {
        'content-type': 'application/json',
        'x-debug-auth': options.headers.Authorization,
        'x-debug-encoded': encoded,
      },
    });

    const result = await ctx.handlers.egressFetch({
      secretRef: 'connector/github/default',
      url: 'https://api.github.com/debug/echo',
      method: 'GET',
    });
    const body = Buffer.from(result.bodyBase64, 'base64').toString('utf8');
    const serialized = JSON.stringify(result);
    assert.equal(serialized.includes(CANARY), false);
    assert.equal(serialized.includes(encoded), false);
    assert.equal(body.includes(CANARY), false);
    assert.match(body, /\[redacted\]/u);
    assert.match(result.headers['x-debug-auth'], /\[redacted\]/u);
    assert.match(result.headers['x-debug-encoded'], /\[redacted\]/u);
  } finally {
    await ctx.cleanup();
  }
});

test('scrubs credential material from remote status text and network failures', async () => {
  const ctx = await fixture();
  try {
    await ctx.handlers.upsertSecrets({
      name: 'connector/github/default',
      value: CANARY,
      binding: githubBinding(),
    });
    ctx.deps.net.fetch = async () => new Response('safe', {
      status: 599,
      statusText: `Remote ${CANARY}`,
    });
    const result = await ctx.handlers.egressFetch({
      secretRef: 'connector/github/default',
      url: 'https://api.github.com/status',
    });
    assert.equal(result.statusText.includes(CANARY), false);
    assert.match(result.statusText, /\[redacted\]/u);

    ctx.deps.net.fetch = async () => {
      throw new Error(`socket failed while using ${CANARY}`);
    };
    await assert.rejects(
      () => ctx.handlers.egressFetch({
        secretRef: 'connector/github/default',
        url: 'https://api.github.com/network-error',
      }),
      (error) => {
        assert.equal(error.message.includes(CANARY), false);
        assert.match(error.message, /\[redacted\]/u);
        return true;
      },
    );
  } finally {
    await ctx.cleanup();
  }
});

test('trusted main process approval is mandatory for credentialed writes', async () => {
  const ctx = await fixture();
  try {
    await ctx.handlers.upsertSecrets({
      name: 'connector/github/default',
      value: CANARY,
      binding: githubBinding(),
    });
    ctx.deps.dialog.showMessageBox = async (options) => {
      ctx.approvalCalls.push(options);
      return { response: 0 };
    };
    await assert.rejects(
      () => ctx.handlers.egressFetch({
        secretRef: 'connector/github/default',
        url: 'https://api.github.com/repos/bhrumom/fabushi/issues',
        method: 'POST',
        body: '{}',
        headers: { 'content-type': 'application/json' },
      }),
      /not approved/u,
    );
    assert.equal(ctx.approvalCalls.length, 1);
    assert.equal(ctx.approvalCalls[0].message.includes(CANARY), false);
    assert.equal(ctx.approvalCalls[0].detail.includes(CANARY), false);
    assert.equal(ctx.calls.length, 0);
  } finally {
    await ctx.cleanup();
  }
});

test('refuses unbound, cross-origin, plaintext-auth-header, and non-HTTPS requests', async () => {
  const ctx = await fixture();
  try {
    await ctx.handlers.upsertSecrets({
      name: 'connector/github/default',
      value: CANARY,
      binding: githubBinding(),
    });
    await ctx.handlers.upsertSecrets({ name: 'legacy/unbound', value: CANARY });

    await assert.rejects(() => ctx.handlers.egressFetch({
      secretRef: 'connector/github/default',
      url: 'https://example.com/',
    }), /not authorized/u);
    await assert.rejects(() => ctx.handlers.egressFetch({
      secretRef: 'legacy/unbound',
      url: 'https://api.github.com/',
    }), /no target binding/u);
    await assert.rejects(() => ctx.handlers.egressFetch({
      secretRef: 'connector/github/default',
      url: 'http://api.github.com/',
    }), /require HTTPS/u);
    await assert.rejects(() => ctx.handlers.egressFetch({
      secretRef: 'connector/github/default',
      url: 'https://api.github.com/',
      headers: { Authorization: 'Bearer caller-controlled' },
    }), /cannot provide Authorization/u);
    assert.equal(ctx.calls.length, 0);
  } finally {
    await ctx.cleanup();
  }
});

test('legacy egress requests without secretRef still use the existing managed egress path', async () => {
  const ctx = await fixture();
  try {
    const result = await ctx.handlers.egressFetch({ url: 'https://example.com', method: 'GET' });
    assert.deepEqual(result, { legacy: true });
    assert.equal(ctx.originalCalls.length, 1);
    assert.equal(ctx.calls.length, 0);
  } finally {
    await ctx.cleanup();
  }
});

test('direct credentialFetch also requires a stored target binding and main-process approval', async () => {
  const ctx = await fixture();
  try {
    await ctx.handlers.upsertSecrets({
      name: 'connector/custom/header',
      value: CANARY,
      binding: {
        allowedOrigins: ['https://api.example.com'],
        injection: { type: 'header', headerName: 'X-API-Key' },
      },
    });
    await credentialFetch(ctx.deps, {
      secretRef: 'connector/custom/header',
      url: 'https://api.example.com/v1/check',
      method: 'POST',
      body: '{}',
      headers: { 'content-type': 'application/json' },
    });
    assert.equal(ctx.approvalCalls.length, 1);
    assert.equal(ctx.calls[0].options.headers['X-API-Key'], CANARY);
  } finally {
    await ctx.cleanup();
  }
});
