'use strict';

const fs = require('node:fs');
const path = require('node:path');

const TEST_AUTH_FILE = 'test-auth-session.json';

function cleanAuthState(value) {
  if (!value || typeof value !== 'object' || value.loggedIn !== true) {
    return { loggedIn: false, provider: 'test' };
  }
  const user = value.user && typeof value.user === 'object'
    ? {
        id: value.user.id,
        username: value.user.username,
        nickname: value.user.nickname,
        email: value.user.email,
        avatar: value.user.avatar,
      }
    : undefined;
  return {
    loggedIn: true,
    provider: typeof value.provider === 'string' && value.provider ? value.provider : 'test',
    ...(user ? { user } : {}),
  };
}

function createTestAuthSession(options = {}) {
  const app = options.app;
  const fsImpl = options.fs ?? fs;
  const now = options.now ?? Date.now;
  if (!app?.getPath) throw new TypeError('test auth session requires an Electron app implementation');

  const statePath = path.join(app.getPath('userData'), 'feature-host', TEST_AUTH_FILE);
  let browserAttempt = null;
  let oauthAttempt = null;
  let sequence = 0;
  let auth = readPersistedAuth();

  function readPersistedAuth() {
    try {
      if (typeof fsImpl.readFileSync !== 'function') return { loggedIn: false, provider: 'test' };
      return cleanAuthState(JSON.parse(fsImpl.readFileSync(statePath, 'utf8')));
    } catch {
      return { loggedIn: false, provider: 'test' };
    }
  }

  function persistAuth(next) {
    auth = cleanAuthState(next);
    try {
      if (typeof fsImpl.mkdirSync !== 'function' || typeof fsImpl.writeFileSync !== 'function') return;
      fsImpl.mkdirSync(path.dirname(statePath), { recursive: true });
      fsImpl.writeFileSync(statePath, `${JSON.stringify(auth)}\n`, { mode: 0o600 });
    } catch {
      // The E2E seam remains deterministic in memory with read-only test stubs.
    }
  }

  function nextId(prefix) {
    sequence += 1;
    return `${prefix}-${sequence}`;
  }

  function request(method, params = {}) {
    switch (method) {
      case 'feature.auth.status':
        return { ...auth, ...(auth.user ? { user: { ...auth.user } } : {}) };
      case 'feature.auth.providers':
        return [
          { id: 'google', displayName: 'Google', enabled: true },
          { id: 'apple', displayName: 'Apple', enabled: true },
          { id: 'microsoft', displayName: 'Microsoft', enabled: true },
          { id: 'github', displayName: 'GitHub', enabled: true },
        ];
      case 'feature.auth.browserStart': {
        browserAttempt = {
          attemptId: `browser-${nextId('attempt')}`,
          loginUrl: 'about:blank#fabushi-test-browser-login',
          expiresAt: Math.floor(now() / 1000) + 600,
          pollAfterMs: 120,
        };
        return { ...browserAttempt };
      }
      case 'feature.auth.browserPoll': {
        if (!browserAttempt || browserAttempt.attemptId !== String(params.attemptId ?? '')) return { status: 'expired' };
        persistAuth({
          loggedIn: true,
          provider: 'browser',
          user: { id: 'fast-e2e-browser-user', email: 'browser@example.test', nickname: 'Browser 测试用户' },
        });
        browserAttempt = null;
        return { status: 'completed', provider: 'browser', auth: { ...auth, user: { ...auth.user } } };
      }
      case 'feature.auth.browserCancel':
        if (!browserAttempt || browserAttempt.attemptId !== String(params.attemptId ?? '')) return { status: 'expired' };
        browserAttempt = null;
        return { status: 'cancelled' };
      case 'feature.auth.browserReopen':
        if (!browserAttempt || browserAttempt.attemptId !== String(params.attemptId ?? '')) return { status: 'expired' };
        browserAttempt = { ...browserAttempt, loginUrl: 'about:blank#fabushi-test-browser-login', pollAfterMs: 120 };
        return { ...browserAttempt, status: 'pending' };
      case 'feature.auth.oauthStart': {
        const provider = String(params.provider ?? '').trim();
        if (!['google', 'apple', 'microsoft', 'github'].includes(provider)) throw new Error(`Unsupported test OAuth provider: ${provider || 'empty'}`);
        oauthAttempt = {
          attemptId: `oauth-${provider}-${nextId('attempt')}`,
          provider,
          authorizationUrl: `about:blank#fabushi-test-oauth-${provider}`,
          expiresAt: Math.floor(now() / 1000) + 600,
        };
        return { ...oauthAttempt };
      }
      case 'feature.auth.oauthPoll': {
        if (!oauthAttempt || oauthAttempt.attemptId !== String(params.attemptId ?? '')) return { status: 'expired' };
        const provider = oauthAttempt.provider;
        persistAuth({
          loggedIn: true,
          provider,
          user: { id: `fast-e2e-${provider}`, email: `test@${provider}.example`, nickname: `${provider} 测试用户` },
        });
        oauthAttempt = null;
        return { status: 'completed', auth: { ...auth, user: { ...auth.user } } };
      }
      case 'feature.auth.logout':
        browserAttempt = null;
        oauthAttempt = null;
        persistAuth({ loggedIn: false, provider: 'test' });
        return { ...auth };
      default:
        return null;
    }
  }

  return Object.freeze({ request, statePath });
}

module.exports = { TEST_AUTH_FILE, createTestAuthSession };
