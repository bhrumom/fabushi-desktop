'use strict';

const fs = require('node:fs/promises');
const path = require('node:path');

const VAULT_SCHEMA_VERSION = 2;
const MAX_SECRET_NAME = 200;
const MAX_LABEL = 160;
const MAX_ORIGINS = 24;
const MAX_BODY_BYTES = 8 * 1024 * 1024;
const MAX_RESPONSE_BYTES = 8 * 1024 * 1024;
const ALLOWED_METHODS = new Set(['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']);
const READ_LIKE_METHODS = new Set(['GET', 'HEAD', 'OPTIONS']);
const FORBIDDEN_CALLER_HEADERS = new Set([
  'authorization',
  'proxy-authorization',
  'cookie',
  'set-cookie',
  'x-api-key',
  'api-key',
]);
const REDACTED_RESPONSE_HEADERS = new Set(['set-cookie', 'authorization', 'proxy-authorization']);
const REDACTION = '[redacted]';

function cleanString(value, limit = 4096) {
  return String(value ?? '').replace(/\0/g, '').trim().slice(0, limit);
}

function normalizeSecretName(value) {
  const name = cleanString(value, MAX_SECRET_NAME);
  if (!name || !/^[a-zA-Z0-9._:/-]+$/.test(name)) throw new Error('Invalid secret reference.');
  return name;
}

function normalizeOrigin(value) {
  let parsed;
  try { parsed = new URL(cleanString(value, 4096)); } catch { throw new Error('Credential origin must be a valid HTTPS URL.'); }
  if (parsed.protocol !== 'https:') throw new Error('Credential origins must use HTTPS.');
  if (parsed.username || parsed.password) throw new Error('Credential origins cannot contain user info.');
  return parsed.origin.toLowerCase();
}

function normalizeAllowedOrigins(value) {
  const rows = Array.isArray(value) ? value : typeof value === 'string' ? value.split(/[\s,]+/u) : [];
  return [...new Set(rows.filter(Boolean).slice(0, MAX_ORIGINS).map(normalizeOrigin))].sort();
}

function normalizeHeaderName(value) {
  const name = cleanString(value, 100);
  if (!name || !/^[A-Za-z0-9-]+$/.test(name)) throw new Error('Credential header name is invalid.');
  if (FORBIDDEN_CALLER_HEADERS.has(name.toLowerCase()) && name.toLowerCase() !== 'authorization' && name.toLowerCase() !== 'x-api-key') {
    throw new Error('Credential header name is not allowed.');
  }
  return name;
}

function normalizeInjection(raw = {}) {
  const value = raw && typeof raw === 'object' && !Array.isArray(raw) ? raw : {};
  const type = cleanString(value.type ?? value.kind ?? 'bearer', 20).toLowerCase();
  if (type === 'bearer') return { type: 'bearer' };
  if (type === 'header') {
    const headerName = normalizeHeaderName(value.headerName ?? value.header ?? 'X-API-Key');
    // Prefix whitespace is semantically meaningful for schemes such as `Token `
    // or `Key `. Do not trim it like user-facing labels; only strip NUL and
    // reject CR/LF before it can become a request header.
    const prefix = String(value.prefix ?? '').replace(/\0/g, '').slice(0, 120);
    if (/[\r\n]/.test(prefix)) throw new Error('Credential header prefix cannot contain newlines.');
    return { type: 'header', headerName, ...(prefix ? { prefix } : {}) };
  }
  if (type === 'basic') {
    const username = cleanString(value.username, 500);
    if (!username || /[\r\n]/.test(username)) throw new Error('Basic-auth username is required.');
    return { type: 'basic', username };
  }
  throw new Error('Unsupported credential injection type.');
}

function normalizeBinding(raw) {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return null;
  const allowedOrigins = normalizeAllowedOrigins(raw.allowedOrigins ?? raw.origins ?? raw.origin);
  const label = cleanString(raw.label, MAX_LABEL);
  const injection = normalizeInjection(raw.injection ?? raw);
  return {
    version: 1,
    allowedOrigins,
    injection,
    ...(label ? { label } : {}),
  };
}

function publicVaultEntry(name, item) {
  const binding = normalizeBinding(item?.binding);
  return {
    name,
    configured: typeof item?.ciphertext === 'string' && item.ciphertext.length > 0,
    revealable: false,
    createdAtMs: Number.isFinite(Number(item?.createdAtMs)) ? Number(item.createdAtMs) : null,
    updatedAtMs: Number.isFinite(Number(item?.updatedAtMs)) ? Number(item.updatedAtMs) : null,
    lastUsedAtMs: Number.isFinite(Number(item?.lastUsedAtMs)) ? Number(item.lastUsedAtMs) : null,
    binding,
  };
}

function secretPath(app) {
  return path.join(app.getPath('userData'), 'secure', 'secrets.json');
}

async function loadVault(app) {
  try {
    const parsed = JSON.parse(await fs.readFile(secretPath(app), 'utf8'));
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return {};
    // The historical file is a plain name -> entry object. Keep that shape so
    // providerEnvironment() in main.cjs remains backwards compatible.
    return parsed.entries && typeof parsed.entries === 'object' && !Array.isArray(parsed.entries)
      ? parsed.entries
      : parsed;
  } catch (error) {
    if (error?.code === 'ENOENT') return {};
    throw error;
  }
}

async function saveVault(app, vault) {
  const target = secretPath(app);
  await fs.mkdir(path.dirname(target), { recursive: true, mode: 0o700 });
  const temporary = `${target}.${process.pid}.${Date.now()}.tmp`;
  await fs.writeFile(temporary, `${JSON.stringify(vault, null, 2)}\n`, { mode: 0o600 });
  await fs.rename(temporary, target);
  try { await fs.chmod(target, 0o600); } catch { /* Windows ACL owns privacy. */ }
}

function requireEncryption(safeStorage) {
  if (!safeStorage?.isEncryptionAvailable?.()) {
    throw new Error('OS-backed secret encryption is not available on this device.');
  }
  const backend = typeof safeStorage.getSelectedStorageBackend === 'function'
    ? safeStorage.getSelectedStorageBackend()
    : null;
  if (backend === 'basic_text') {
    throw new Error('Secure OS credential storage is unavailable; refusing Electron basic_text fallback.');
  }
}

function decryptSecret(safeStorage, item) {
  requireEncryption(safeStorage);
  if (!item?.ciphertext || typeof item.ciphertext !== 'string') throw new Error('Credential is not configured.');
  const value = safeStorage.decryptString(Buffer.from(item.ciphertext, 'base64'));
  if (!value || /[\r\n]/.test(value)) throw new Error('Credential value is empty or unsafe for request injection.');
  return value;
}

function encodedBody(params) {
  const bodyBase64 = cleanString(params.bodyBase64, Math.ceil(MAX_BODY_BYTES * 4 / 3) + 16);
  const bytes = bodyBase64
    ? Buffer.from(bodyBase64, 'base64')
    : typeof params.body === 'string'
      ? Buffer.from(params.body, 'utf8')
      : null;
  if (bytes && bytes.byteLength > MAX_BODY_BYTES) throw new Error('Credential request body exceeds the safety limit.');
  return bytes;
}

function sanitizeCallerHeaders(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return {};
  const headers = {};
  for (const [rawName, rawValue] of Object.entries(value).slice(0, 80)) {
    const name = normalizeHeaderName(rawName);
    if (FORBIDDEN_CALLER_HEADERS.has(name.toLowerCase())) {
      throw new Error(`Credentialed requests cannot provide ${name}; authentication is injected by the gateway.`);
    }
    const headerValue = cleanString(rawValue, 8192);
    if (/[\r\n]/.test(headerValue)) throw new Error(`Header ${name} contains an unsafe newline.`);
    headers[name] = headerValue;
  }
  return headers;
}

function injectCredential(headers, injection, secret) {
  const next = { ...headers };
  if (injection.type === 'bearer') {
    next.Authorization = `Bearer ${secret}`;
  } else if (injection.type === 'header') {
    next[injection.headerName] = `${injection.prefix ?? ''}${secret}`;
  } else if (injection.type === 'basic') {
    next.Authorization = `Basic ${Buffer.from(`${injection.username}:${secret}`, 'utf8').toString('base64')}`;
  } else {
    throw new Error('Credential binding has an unsupported injection type.');
  }
  return next;
}

function credentialRedactionTokens(injection, secret) {
  const tokens = new Set([
    secret,
    Buffer.from(secret, 'utf8').toString('base64'),
  ]);
  try { tokens.add(encodeURIComponent(secret)); } catch { /* malformed Unicode cannot become a URL token */ }
  try {
    const escaped = JSON.stringify(secret);
    if (escaped.length >= 2) tokens.add(escaped.slice(1, -1));
  } catch { /* a normal JS string is JSON-serializable */ }
  if (injection.type === 'basic') {
    tokens.add(Buffer.from(`${injection.username}:${secret}`, 'utf8').toString('base64'));
  }
  return [...tokens].filter((token) => typeof token === 'string' && token.length > 0);
}

function redactText(value, tokens) {
  let output = String(value ?? '');
  for (const token of tokens) output = output.split(token).join(REDACTION);
  return output;
}

function redactBuffer(value, tokens) {
  let output = Buffer.from(value);
  const replacement = Buffer.from(REDACTION, 'utf8');
  for (const token of tokens) {
    const needle = Buffer.from(token, 'utf8');
    if (!needle.length) continue;
    const parts = [];
    let start = 0;
    let index = output.indexOf(needle, start);
    if (index < 0) continue;
    while (index >= 0) {
      parts.push(output.subarray(start, index), replacement);
      start = index + needle.length;
      index = output.indexOf(needle, start);
    }
    parts.push(output.subarray(start));
    output = Buffer.concat(parts);
  }
  return output;
}

async function requireCredentialWriteApproval(deps, { secretRef, method, origin }) {
  if (READ_LIKE_METHODS.has(method)) return;
  if (!deps.dialog?.showMessageBox) {
    throw new Error('Credential write approval UI is unavailable.');
  }
  const result = await deps.dialog.showMessageBox({
    type: 'warning',
    title: '允许凭据请求？',
    message: `允许使用 ${secretRef} 执行 ${method} 请求？`,
    detail: `目标：${origin}\n\n密钥不会显示给模型或页面，但这个请求可能修改远端数据。`,
    buttons: ['取消', '允许本次'],
    defaultId: 0,
    cancelId: 0,
    noLink: true,
  });
  if (result?.response !== 1) {
    throw new Error('Credential write request was not approved by the user.');
  }
}

async function credentialFetch(deps, params) {
  const { app, safeStorage, net } = deps;
  if (!net?.fetch) throw new Error('Electron network service is unavailable.');
  const secretRef = normalizeSecretName(params.secretRef ?? params.credential?.secretRef ?? params.credential?.ref);
  const rawUrl = cleanString(params.url, 4096);
  let url;
  try { url = new URL(rawUrl); } catch { throw new Error('Credential request URL is invalid.'); }
  if (url.protocol !== 'https:') throw new Error('Credentialed requests require HTTPS.');
  if (url.username || url.password) throw new Error('Credential request URLs cannot contain user info.');
  url.hash = '';
  const requestOrigin = url.origin.toLowerCase();

  const vault = await loadVault(app);
  const item = vault[secretRef];
  if (!item) throw new Error(`Credential ${secretRef} is not configured.`);
  const binding = normalizeBinding(item.binding);
  if (!binding || binding.allowedOrigins.length === 0) {
    throw new Error(`Credential ${secretRef} has no target binding and cannot be used by tools.`);
  }
  if (!binding.allowedOrigins.includes(requestOrigin)) {
    throw new Error(`Credential ${secretRef} is not authorized for ${requestOrigin}.`);
  }

  const method = cleanString(params.method, 16).toUpperCase() || 'GET';
  if (!ALLOWED_METHODS.has(method)) throw new Error('Credential request method is not allowed.');
  const body = encodedBody(params);
  if ((method === 'GET' || method === 'HEAD') && body?.length) throw new Error(`${method} credential requests cannot contain a body.`);
  await requireCredentialWriteApproval(deps, { secretRef, method, origin: requestOrigin });
  const secret = decryptSecret(safeStorage, item);
  const redactionTokens = credentialRedactionTokens(binding.injection, secret);
  const headers = injectCredential(sanitizeCallerHeaders(params.headers), binding.injection, secret);

  let response;
  try {
    // Redirects are intentionally never followed with credentials. A 3xx is
    // returned to the caller, which can request a separately bound origin.
    response = await net.fetch(url.toString(), {
      method,
      headers,
      redirect: 'manual',
      ...(body?.length ? { body } : {}),
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(redactText(message, redactionTokens));
  }
  const rawResponseBytes = Buffer.from(await response.arrayBuffer());
  if (rawResponseBytes.byteLength > MAX_RESPONSE_BYTES) throw new Error('Credential response exceeds the safety limit.');
  // Do not trust the remote endpoint to keep credentials opaque: diagnostic
  // and echo APIs can reflect request headers back to the caller. Scrub the
  // raw secret plus common encoded forms before any result crosses back into
  // the Renderer/model boundary.
  const responseBytes = redactBuffer(rawResponseBytes, redactionTokens);
  const responseHeaders = {};
  for (const [name, value] of response.headers.entries()) {
    if (!REDACTED_RESPONSE_HEADERS.has(name.toLowerCase())) {
      responseHeaders[name] = redactText(value, redactionTokens);
    }
  }

  item.lastUsedAtMs = Date.now();
  vault[secretRef] = item;
  await saveVault(app, vault);

  return {
    status: response.status,
    statusText: redactText(response.statusText, redactionTokens),
    headers: responseHeaders,
    bodyBase64: responseBytes.toString('base64'),
    url: url.toString(),
    credential: {
      secretRef,
      origin: requestOrigin,
      injectionType: binding.injection.type,
    },
  };
}

function normalizeUpsertCandidates(params) {
  if (Array.isArray(params?.secrets)) return params.secrets.slice(0, 100);
  if (params?.secrets && typeof params.secrets === 'object') {
    return Object.entries(params.secrets).slice(0, 100).map(([name, value]) => ({ name, value }));
  }
  return params?.name != null ? [{ ...params, name: params.name, value: params.value }] : [];
}

function wrapNativeCapabilityHandlers(originalFactory) {
  if (typeof originalFactory !== 'function') throw new TypeError('Original native capability factory is required.');
  return function createCredentialAwareNativeCapabilityHandlers(deps) {
    const handlers = originalFactory(deps);
    const { app, safeStorage } = deps;

    handlers.listSecrets = async () => {
      const vault = await loadVault(app);
      return Object.entries(vault)
        .map(([name, item]) => publicVaultEntry(name, item))
        .sort((left, right) => left.name.localeCompare(right.name));
    };

    handlers.upsertSecrets = async (params = {}) => {
      requireEncryption(safeStorage);
      const vault = await loadVault(app);
      const now = Date.now();
      for (const candidate of normalizeUpsertCandidates(params)) {
        const name = normalizeSecretName(candidate?.name ?? candidate?.key);
        const value = String(candidate?.value ?? '');
        if (!value) throw new Error('Credential value must not be empty.');
        if (/[\0\r\n]/.test(value)) throw new Error('Credential value contains unsupported control characters.');
        const previous = vault[name] && typeof vault[name] === 'object' ? vault[name] : {};
        const explicitBinding = candidate?.binding ?? params.binding;
        const binding = explicitBinding === undefined ? previous.binding ?? null : normalizeBinding(explicitBinding);
        vault[name] = {
          ...previous,
          schemaVersion: VAULT_SCHEMA_VERSION,
          ciphertext: safeStorage.encryptString(value).toString('base64'),
          createdAtMs: Number.isFinite(Number(previous.createdAtMs)) ? Number(previous.createdAtMs) : now,
          updatedAtMs: now,
          ...(binding ? { binding } : {}),
        };
      }
      await saveVault(app, vault);
      return handlers.listSecrets();
    };

    handlers.removeSecrets = async (params = {}) => {
      const names = Array.isArray(params.names) ? params.names : [params.name ?? params.key].filter(Boolean);
      const vault = await loadVault(app);
      for (const raw of names.slice(0, 100)) delete vault[normalizeSecretName(raw)];
      await saveVault(app, vault);
      return handlers.listSecrets();
    };

    handlers.revealSecret = async () => {
      throw new Error('Secret plaintext is non-revealable. Use a target-bound credential request instead.');
    };

    const originalEgressFetch = typeof handlers.egressFetch === 'function' ? handlers.egressFetch.bind(handlers) : null;
    handlers.egressFetch = async (params = {}) => {
      const hasCredential = Boolean(params.secretRef || params.credential?.secretRef || params.credential?.ref);
      if (!hasCredential) {
        if (!originalEgressFetch) throw new Error('Egress fetch is unavailable.');
        return originalEgressFetch(params);
      }
      return credentialFetch(deps, params);
    };

    return handlers;
  };
}

module.exports = {
  wrapNativeCapabilityHandlers,
  credentialFetch,
  normalizeBinding,
  publicVaultEntry,
};
