import { _electron as electron, expect, test, type ElectronApplication, type Page, type TestInfo } from '@playwright/test';
import { createReadStream } from 'node:fs';
import { lstat, mkdtemp, readFile, readdir, readlink, rm } from 'node:fs/promises';
import { execFile } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createServer, type IncomingMessage, type ServerResponse } from 'node:http';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;

let e2eAuthServer: ReturnType<typeof createServer> | null = null;
let e2eAuthBackendPromise: Promise<string> | null = null;
const execFileAsync = promisify(execFile);

const humanAccountId = 'fabushi-e2e-account';
const humanPeerId = 'fabushi-e2e-peer';
const humanPeerName = 'Human restart peer';
// These exact background dependencies remain unavailable in the Human fixture.
// Keep their real 404 behavior without classifying declared failures as unknown
// HTTP contracts. This does not qualify Plugins, cloud Box or telemetry success.
const humanUnavailablePostPaths = new Set([
  '/aiserver.v1.AnalyticsService/BootstrapStatsig',
  '/aiserver.v1.AnalyticsService/SubmitLogs',
  '/aiserver.v1.DashboardService/ListMarketplacePlugins',
  '/aiserver.v1.DashboardService/GetUserPrivacyMode',
  '/aiserver.v1.DashboardService/GetAvailableMcpServers',
  '/aiserver.v1.DashboardService/GetEffectiveUserPlugins',
  '/aiserver.v1.GrokBotService/EnsureSandBox',
  '/aiserver.v1.GrokBotService/WatchSandBoxMigration',
  '/sand-box/local-exec-daemon-credential',
]);
interface HumanHttpRequest {
  sequence: number;
  phase: string;
  method: string;
  path: string;
  authorizationMatches: boolean;
  deviceMatches: boolean;
  body?: unknown;
}
interface HumanRemoteMessage {
  id: number;
  senderUserId: string;
  recipientUserId: string;
  text: string;
  clientRequestId: string;
  createdAt: string;
  readAt: null;
  isOutgoing: true;
  replyToMessageId: null;
  attachments: never[];
  reactions: never[];
}
interface HeldHumanReply {
  request: HumanHttpRequest;
  response: ServerResponse;
  message: HumanRemoteMessage;
  closed: boolean;
  released: boolean;
}
interface HumanElectronScenario {
  phase: string;
  historyAvailable: boolean;
  holdPosts: boolean;
  messages: Map<string, HumanRemoteMessage>;
  requests: HumanHttpRequest[];
  events: Array<Record<string, unknown>>;
  unexpected: Array<Record<string, unknown>>;
  held: HeldHumanReply[];
}
// Opt in only for the two NMR-ELECTRON-01 cases. The existing Plugins and Agent
// Network fixtures keep an empty social account and all their original checks.
let activeHumanScenario: HumanElectronScenario | null = null;

function newHumanScenario(): HumanElectronScenario {
  return {
    phase: 'first-lifetime', historyAvailable: true, holdPosts: false,
    messages: new Map(), requests: [], events: [], unexpected: [], held: [],
  };
}

function humanHttpReply(
  scenario: HumanElectronScenario, request: HumanHttpRequest,
  response: ServerResponse, status: number, value: unknown,
) {
  scenario.events.push({ at: Date.now(), kind: 'http-reply', request: request.sequence, phase: scenario.phase, status, value });
  response.statusCode = status;
  response.end(JSON.stringify(value));
}

async function handleHumanMessages(
  scenario: HumanElectronScenario, trace: HumanHttpRequest,
  request: IncomingMessage, response: ServerResponse, url: URL,
) {
  const reject = (detail: string) => {
    scenario.unexpected.push({ request: trace.sequence, detail });
    humanHttpReply(scenario, trace, response, 400, { success: false, error: detail });
  };
  if (!trace.authorizationMatches || !trace.deviceMatches) return reject('invalid Human fixture session binding');
  if (request.method === 'GET') {
    const limitText = url.searchParams.get('limit') ?? '';
    const limit = Number(limitText);
    if (url.searchParams.get('contactId') !== humanPeerId
      || !/^[1-9][0-9]*$/.test(limitText) || !Number.isSafeInteger(limit) || limit > 200
      || [...url.searchParams.keys()].some((key) => !['contactId', 'limit', 'afterId'].includes(key))) {
      return reject('unexpected bounded Human history request');
    }
    // An explicit zero is a real origin request, never a falsy tail alias.
    const hasAfter = url.searchParams.has('afterId');
    const afterText = url.searchParams.get('afterId') ?? '';
    const afterId = Number(afterText);
    if (hasAfter && (!/^(0|[1-9][0-9]*)$/.test(afterText) || !Number.isSafeInteger(afterId))) {
      return reject('invalid explicit Human afterId');
    }
    if (!scenario.historyAvailable) {
      humanHttpReply(scenario, trace, response, 503, { success: false, error: 'fixture-history-unavailable' });
      return;
    }
    const canonical = [...scenario.messages.values()].sort((left, right) => left.id - right.id);
    const messages = hasAfter
      ? canonical.filter((message) => message.id > afterId).slice(0, limit)
      : canonical.slice(-limit).reverse();
    humanHttpReply(scenario, trace, response, 200, {
      success: true,
      data: {
        messages,
        ...(hasAfter ? {
          historyCursor: { mode: 'after-id', afterId },
          nextAfterId: messages.at(-1)?.id ?? afterId,
        } : {}),
      },
    });
    return;
  }
  if (request.method !== 'POST') return reject('unexpected Human messages method');
  let body: Record<string, unknown>;
  try { body = JSON.parse((await readRequestBody(request)).toString('utf8')) as Record<string, unknown>; }
  catch { return reject('invalid Human message JSON'); }
  trace.body = body;
  if (body == null || Array.isArray(body) || typeof body !== 'object'
    || body.targetUserId !== humanPeerId || typeof body.text !== 'string' || body.text.trim().length === 0
    || typeof body.clientRequestId !== 'string' || body.clientRequestId.length === 0
    || body.replyToMessageId != null
    || (body.attachments != null && (!Array.isArray(body.attachments) || body.attachments.length !== 0))
    || Object.keys(body).some((key) => !['targetUserId', 'text', 'clientRequestId', 'replyToMessageId', 'attachments'].includes(key))) {
    return reject('unexpected Human message payload');
  }
  const existing = scenario.messages.get(body.clientRequestId);
  if (existing != null && existing.text !== body.text) return reject('nonce replay changed its committed content');
  const message: HumanRemoteMessage = existing ?? {
    id: scenario.messages.size + 1,
    senderUserId: humanAccountId, recipientUserId: humanPeerId,
    text: body.text, clientRequestId: body.clientRequestId,
    createdAt: new Date(1_735_787_045_000 + scenario.messages.size * 1000).toISOString(),
    readAt: null, isOutgoing: true, replyToMessageId: null, attachments: [], reactions: [],
  };
  scenario.messages.set(message.clientRequestId, message);
  scenario.events.push({
    at: Date.now(), kind: existing == null ? 'remote-commit' : 'idempotent-replay',
    request: trace.sequence, phase: scenario.phase, message,
  });
  if (scenario.holdPosts) {
    const held: HeldHumanReply = { request: trace, response, message, closed: false, released: false };
    scenario.held.push(held);
    response.once('close', () => {
      held.closed = true;
      scenario.events.push({ at: Date.now(), kind: 'held-response-closed', request: trace.sequence, phase: scenario.phase });
    });
    response.on('error', (error) => scenario.events.push({
      at: Date.now(), kind: 'held-response-error', request: trace.sequence, message: error.message,
    }));
    scenario.events.push({ at: Date.now(), kind: 'response-held-after-commit', request: trace.sequence, phase: scenario.phase });
    return;
  }
  humanHttpReply(scenario, trace, response, 200, { success: true, message });
}

function releaseOldHumanResponses(scenario: HumanElectronScenario) {
  for (const held of scenario.held) {
    if (held.released) throw new Error('An old Human response was released twice');
    held.released = true;
    // This is a recorded attempt on the original socket after its actual
    // Electron/Host lifetime ended, not a fabricated callback into the new UI.
    scenario.events.push({
      at: Date.now(), kind: 'late-old-response-release', request: held.request.sequence,
      phase: scenario.phase, closed: held.closed, destroyed: held.response.destroyed,
      canonicalId: held.message.id, clientRequestId: held.message.clientRequestId,
    });
    held.response.statusCode = 200;
    held.response.end(JSON.stringify({ success: true, message: held.message }));
  }
}

function e2eAuthToken(): string {
  const encode = (value: Record<string, unknown>) => Buffer.from(JSON.stringify(value), 'utf8').toString('base64url');
  return [
    encode({ alg: 'none', typ: 'JWT' }),
    encode({ sub: 'fabushi-e2e-account', email: 'e2e@fabushi.local', exp: 4_102_444_800 }),
    'fabushi-e2e',
  ].join('.');
}

async function readRequestBody(request: import('node:http').IncomingMessage): Promise<Buffer> {
  const chunks: Buffer[] = [];
  for await (const chunk of request) chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  return Buffer.concat(chunks);
}

async function ensureE2eAuthBackend(): Promise<string> {
  if (e2eAuthBackendPromise != null) return await e2eAuthBackendPromise;
  e2eAuthBackendPromise = new Promise<string>((resolve, reject) => {
    const token = e2eAuthToken();
    const server = createServer(async (request, response) => {
      const requestUrl = new URL(request.url ?? '/', 'http://127.0.0.1');
      const human = activeHumanScenario;
      const trace: HumanHttpRequest | null = human == null ? null : {
        sequence: human.requests.length + 1, phase: human.phase,
        method: request.method ?? 'GET', path: requestUrl.pathname + requestUrl.search,
        authorizationMatches: request.headers.authorization === 'Bearer ' + token,
        deviceMatches: request.headers['x-fabushi-device-id'] === 'fabushi-agent-network-e2e-device',
      };
      if (human != null && trace != null) {
        human.requests.push(trace);
        human.events.push({ at: Date.now(), kind: 'http-request', request: trace.sequence, phase: human.phase, method: trace.method, path: trace.path });
        response.once('finish', () => human.events.push({
          at: Date.now(), kind: 'http-finished', request: trace.sequence,
          status: response.statusCode, phase: human.phase,
        }));
      }
      response.setHeader('content-type', 'application/json');
      if (requestUrl.pathname === '/auth/poll') {
        response.statusCode = 200;
        response.end(JSON.stringify({ accessToken: token, refreshToken: token }));
        return;
      }
      if (requestUrl.pathname === '/oauth/token') {
        response.statusCode = 200;
        response.end(JSON.stringify({ access_token: token, refresh_token: token }));
        return;
      }
      if (requestUrl.pathname === '/api/auth/browser/start' && request.method === 'POST') {
        const origin = `http://${request.headers.host ?? '127.0.0.1'}`;
        response.statusCode = 200;
        response.end(JSON.stringify({
          attemptId: 'fabushi-agent-network-e2e-attempt',
          loginUrl: `${origin}/fabushi-agent-network-e2e-login`,
          pollSecret: 'fabushi-agent-network-e2e-poll-secret',
          expiresAt: Date.now() + 60_000,
          pollAfterMs: 250,
        }));
        return;
      }
      if (requestUrl.pathname === '/api/auth/browser/attempts/fabushi-agent-network-e2e-attempt' && request.method === 'POST') {
        const body = JSON.parse((await readRequestBody(request)).toString('utf8')) as { pollSecret?: unknown };
        if (body.pollSecret !== 'fabushi-agent-network-e2e-poll-secret') {
          response.statusCode = 403;
          response.end(JSON.stringify({ error: { code: 'invalid-poll-secret' } }));
          return;
        }
        const expiresAt = 4_102_444_800_000;
        response.statusCode = 200;
        response.end(JSON.stringify({
          status: 'completed',
          session: {
            accessToken: token,
            refreshToken: token,
            accessTokenExpiresAt: expiresAt,
            refreshTokenExpiresAt: expiresAt,
            sessionId: 'fabushi-agent-network-e2e-session',
            deviceId: 'fabushi-agent-network-e2e-device',
            username: 'e2e@fabushi.local',
            userId: 'fabushi-e2e-account',
            provider: 'focused-e2e',
          },
        }));
        return;
      }
      if (requestUrl.pathname === '/api/auth/user-info') {
        if (request.headers.authorization !== `Bearer ${token}`) {
          response.statusCode = 401;
          response.end(JSON.stringify({ error: { code: 'invalid-access-token' } }));
          return;
        }
        response.statusCode = 200;
        response.end(JSON.stringify({
          id: 'fabushi-e2e-account',
          username: 'e2e@fabushi.local',
          email: 'e2e@fabushi.local',
          displayName: 'Fabushi E2E',
        }));
        return;
      }
      if (requestUrl.pathname === '/api/auth/refresh' && request.method === 'POST') {
        const expiresAt = 4_102_444_800_000;
        response.statusCode = 200;
        response.end(JSON.stringify({
          accessToken: token,
          refreshToken: token,
          accessTokenExpiresAt: expiresAt,
          refreshTokenExpiresAt: expiresAt,
          sessionId: 'fabushi-agent-network-e2e-session',
          deviceId: 'fabushi-agent-network-e2e-device',
          username: 'e2e@fabushi.local',
          userId: 'fabushi-e2e-account',
          provider: 'focused-e2e',
        }));
        return;
      }
      // The shipping logged-in workspace immediately asks the Host to reconcile
      // canonical Human contacts and calls. Keep this focused backend honest by
      // implementing the production HTTP envelopes instead of letting those
      // requests fall through to the auth-only 404 handler. An empty social
      // account is valid and prevents unrelated Human sync failures from
      // aborting the Agent Network route transition.
      if (human != null && trace != null && requestUrl.pathname === '/api/social/messages') {
        await handleHumanMessages(human, trace, request, response, requestUrl);
        return;
      }
      if (requestUrl.pathname === '/api/social/friends' && request.method === 'GET') {
        if (request.headers.authorization !== `Bearer ${token}`
          || request.headers['x-fabushi-device-id'] !== 'fabushi-agent-network-e2e-device') {
          response.statusCode = 401;
          response.end(JSON.stringify({ success: false, error: 'invalid-session' }));
          return;
        }
        response.statusCode = 200;
        response.end(JSON.stringify({ success: true, data: { friends: human == null ? [] : [{
          id: 'human-electron-contact', userId: humanPeerId, username: 'human-peer@example.test',
          displayName: humanPeerName, avatarUrl: null, status: 'accepted',
        }] } }));
        return;
      }
      if (requestUrl.pathname === '/api/social/calls' && request.method === 'GET') {
        if (request.headers.authorization !== `Bearer ${token}`
          || request.headers['x-fabushi-device-id'] !== 'fabushi-agent-network-e2e-device') {
          response.statusCode = 401;
          response.end(JSON.stringify({ success: false, error: 'invalid-session' }));
          return;
        }
        response.statusCode = 200;
        response.end(JSON.stringify({ success: true, calls: [] }));
        return;
      }
      if (human != null && trace != null) {
        if (trace.method === 'POST' && humanUnavailablePostPaths.has(trace.path)) {
          human.events.push({
            at: Date.now(), kind: 'controlled-auxiliary-unavailable', request: trace.sequence,
            phase: human.phase, method: trace.method, path: trace.path, status: 404,
          });
          humanHttpReply(human, trace, response, 404, { error: 'not-found' });
          return;
        }
        if (trace.method === 'GET' && trace.path === '/fabushi-agent-network-e2e-login') {
          response.setHeader('content-type', 'text/html; charset=utf-8');
          response.statusCode = 200;
          response.end('<!doctype html><title>Fabushi test sign in</title><p>Return to the desktop app.</p>');
          return;
        }
        if (trace.method === 'GET' && trace.path === '/favicon.ico') {
          response.statusCode = 204;
          response.end();
          return;
        }
      }
      if (human != null && trace != null) human.unexpected.push({
        request: trace.sequence, detail: 'unimplemented HTTP endpoint', method: trace.method, path: trace.path,
      });
      response.statusCode = 404;
      response.end(JSON.stringify({ error: 'not-found' }));
    });
    e2eAuthServer = server;
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      if (address == null || typeof address === 'string') {
        reject(new Error('Focused Electron auth backend did not bind a TCP address.'));
        return;
      }
      resolve(`http://127.0.0.1:${address.port}`);
    });
  });
  return await e2eAuthBackendPromise;
}

test.afterAll(async () => {
  const server = e2eAuthServer;
  e2eAuthServer = null;
  e2eAuthBackendPromise = null;
  if (server == null || !server.listening) return;
  await new Promise<void>((resolve, reject) => {
    server.close((error) => error == null ? resolve() : reject(error));
  });
});

async function launchDesktopApp(appDataDir: string, capture?: { info: TestInfo; label: string }) {
  const e2eAuthBackendUrl = await ensureE2eAuthBackend();
  return electron.launch({
    ...(packagedExecutable
      ? { executablePath: packagedExecutable, args: [] }
      : { args: [appRoot] }),
    ...(capture == null ? {} : {
      recordVideo: { dir: capture.info.outputPath(capture.label + '-video'), size: { width: 1280, height: 900 } },
      tracesDir: capture.info.outputPath(capture.label + '-trace-parts'),
    }),
    env: {
      ...process.env,
      FABUSHI_E2E: '1',
      SAND_USER_DATA_DIR: appDataDir,
      SAND_BACKEND_URL: e2eAuthBackendUrl,
      CURSOR_API_BASE_URL: e2eAuthBackendUrl,
      FABUSHI_API_BASE_URL: e2eAuthBackendUrl,
      SAND_CURSOR_WEBSITE_URL: e2eAuthBackendUrl,
      SAND_DISABLE_SENTRY: '1',
      FABUSHI_FEATURE_HOST_MODE: process.env.FABUSHI_FEATURE_HOST_MODE || 'test',
      SAND_FEATURE_GATE_OVERRIDES: 'sand_agent_network=1',
      MAHAYANA_APP_HOST_BIN: process.env.MAHAYANA_APP_HOST_BIN || '',
      ...(capture == null ? {} : {
        // These two cases must restore the account actually saved by Sign in,
        // and may read/write only this test-owned production data directory.
        FABUSHI_CI_ACCOUNT_SESSION_FILE: '',
        SAND_DATA_ROOT: path.join(appDataDir, 'sand-data'),
        FABUSHI_APP_DATA: path.join(appDataDir, 'feature-app-data'),
      }),
    },
  });
}

async function completeBrowserLogin(
  page: Page,
  destination: 'agent-network' | 'plugins' = 'agent-network',
  options: { ensureAgent?: boolean } = {},
): Promise<void> {
  const rendererErrors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') rendererErrors.push(message.text());
  });

  await page.waitForFunction(() => {
    const candidate = window as unknown as {
      desktop?: {
        cursorAccount?: { getStatus?: unknown };
        onboarding?: { setSeen?: unknown };
      };
    };
    return typeof candidate.desktop?.cursorAccount?.getStatus === 'function'
      && typeof candidate.desktop?.onboarding?.setSeen === 'function';
  }, { timeout: 15_000 });

  const accountKind = async (): Promise<string> => page.evaluate(async () => {
    const candidate = window as unknown as {
      desktop: {
        cursorAccount: { getStatus(): Promise<{ kind?: string }> };
      };
    };
    return (await candidate.desktop.cursorAccount.getStatus()).kind ?? 'unknown';
  });

  if (await accountKind() === 'logged-out') {
    const signInSurface = page.getByRole('main', { name: 'Fabushi', exact: true });
    await expect(signInSurface).toBeVisible({ timeout: 15_000 });
    await signInSurface.getByRole('button', { name: 'Sign in', exact: true }).click();
  }

  await expect.poll(accountKind, { timeout: 20_000 }).toBe('logged-in');
  await page.evaluate(async () => {
    const candidate = window as unknown as {
      desktop: {
        onboarding: { setSeen(seen: boolean): Promise<unknown> };
      };
    };
    await candidate.desktop.onboarding.setSeen(true);
  });

  const onboardingSeen = async (): Promise<boolean> => page.evaluate(async () => {
    const candidate = window as unknown as {
      desktop: {
        onboarding: { getSeen(): Promise<boolean> };
      };
    };
    return (await candidate.desktop.onboarding.getSeen()) === true;
  });
  await expect.poll(onboardingSeen, { timeout: 10_000 }).toBe(true);

  await page.reload({ waitUntil: 'domcontentloaded' });
  await page.waitForFunction(() => {
    const candidate = window as unknown as { desktop?: { cursorAccount?: { getStatus?: unknown } } };
    return typeof candidate.desktop?.cursorAccount?.getStatus === 'function';
  }, { timeout: 15_000 });
  await expect.poll(accountKind, { timeout: 10_000 }).toBe('logged-in');
  await expect(page.getByRole('main', { name: 'Fabushi', exact: true })).toBeHidden({ timeout: 10_000 });

  const fatal = page.locator('.sand-error-boundary--app');
  if (await fatal.count()) {
    const surfaceText = await fatal.first().innerText().catch(() => 'unknown renderer failure');
    throw new Error(`renderer root fatal: ${rendererErrors.at(-1) ?? surfaceText}`);
  }

  // The current shipping shell no longer materializes a synthetic "New chat"
  // roster row. Readiness is the canonical Agent list plus its stable shell
  // actions; opening "New" would enter the Create agent dialog and is not an
  // account/session readiness signal.
  const roster = page.getByRole('region', { name: 'Agent list' });
  await expect(roster).toBeVisible({ timeout: 15_000 });

  // FPM-UI-CLOSE-01 exercises the authenticated shell's Close control even when
  // the independent Host roster cannot load. Keep that observed state in the
  // trace; only Agent Network requires a real Agent before its own assertions.
  if (destination === 'plugins') {
    await test.info().attach('plugins-roster-observation', {
      body: Buffer.from(JSON.stringify({
        agentRowCount: await roster.locator('button.sand-agent-item').count(),
        unavailable: await page.getByText("Can't reach your computer", { exact: true }).isVisible(),
        scope: 'Plugins pointer only; Host roster recovery remains unqualified',
      })),
      contentType: 'application/json',
    });
    return;
  }
  if (options.ensureAgent === false) return;

  // Agent Network is intentionally fail-closed until a real Agent exists.
  // A fresh authenticated account can have an empty canonical Host roster, so
  // establish the fixture through the shipping New -> Create agent flow instead
  // of bypassing the product gate or injecting synthetic Host state.
  const loadedAgentRows = roster.locator('button.sand-agent-item');
  const emptyRoster = roster.getByText('No saved agents yet.', { exact: true });
  await expect.poll(async () => {
    if (await loadedAgentRows.count() > 0) return 'loaded';
    return await emptyRoster.isVisible().catch(() => false) ? 'empty' : 'pending';
  }, { timeout: 15_000 }).not.toBe('pending');
  if (await loadedAgentRows.count() === 0) {
    await page.getByRole('button', { name: 'New', exact: true }).click();
    const createAgent = page.getByRole('dialog', { name: 'Create agent' });
    await expect(createAgent).toBeVisible();
    await createAgent.getByRole('button', { name: 'Create', exact: true }).click();
    await expect(createAgent).toBeHidden();
    await expect.poll(async () => loadedAgentRows.count(), { timeout: 15_000 }).toBeGreaterThan(0);
  }

  await expect(page.getByRole('button', { name: 'Agent network', exact: true })).toBeVisible({ timeout: 15_000 });
}

async function measuredGeometry(network: ReturnType<Page['getByRole']>) {
  return network.evaluate((element) => {
    const rect = element.getBoundingClientRect();
    const parentRect = element.parentElement?.getBoundingClientRect();
    return {
      width: rect.width,
      height: rect.height,
      parentWidth: parentRect?.width ?? 0,
      parentHeight: parentRect?.height ?? 0,
    };
  });
}

async function withPluginsDeadline<T>(label: string, task: Promise<T>, timeoutMs = 5_000): Promise<T> {
  let timer: NodeJS.Timeout | undefined;
  try {
    return await Promise.race([
      task,
      new Promise<T>((_, reject) => { timer = setTimeout(() => reject(new Error(`${label} exceeded ${timeoutMs}ms`)), timeoutMs); }),
    ]);
  } finally {
    if (timer) clearTimeout(timer);
  }
}

async function pluginsPointerGeometry(page: Page) {
  return withPluginsDeadline('Plugins pointer geometry', page.evaluate(() => {
    const describe = (element: Element | null) => {
      if (!element) return null;
      const rect = element.getBoundingClientRect();
      const style = getComputedStyle(element);
      return {
        tag: element.tagName,
        classes: [...element.classList],
        bounds: { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
        style: {
          zIndex: style.zIndex, pointerEvents: style.pointerEvents,
          appRegion: style.getPropertyValue('-webkit-app-region') || style.getPropertyValue('app-region'),
        },
      };
    };
    const dialog = document.querySelector('[role="dialog"][aria-label="Plugins"]');
    const buttons = dialog?.querySelectorAll('button[aria-label="Close"]') ?? [];
    const close = buttons[0] ?? null;
    const rect = close?.getBoundingClientRect();
    const point = rect ? { x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 } : null;
    const hit = point ? document.elementFromPoint(point.x, point.y) : null;
    const windowControls = [...document.querySelectorAll('.sand-window-controls button')].map((control) => {
      const box = control.getBoundingClientRect();
      const target = document.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2);
      return { ...describe(control), centerHitsControl: target != null && (target === control || control.contains(target)) };
    });
    return {
      viewport: { width: innerWidth, height: innerHeight },
      closeCount: buttons.length, close: describe(close), dialog: describe(dialog), frame: describe(dialog?.parentElement ?? null),
      drag: describe(document.querySelector('.sand-cover-drag')),
      header: describe(document.querySelector('.sand-chat-header')),
      hit: describe(hit), point, windowControls,
      windowControlEvidence: {
        domControlCount: windowControls.length,
        nativeButtonActionsExecuted: 0,
        scope: windowControls.length > 0 ? 'DOM-hit-test-and-reserved-band-geometry' : 'reserved-band-geometry-only',
      },
      centerHitsClose: close != null && hit != null && (hit === close || close.contains(hit)),
      hitStack: point ? document.elementsFromPoint(point.x, point.y).slice(0, 8).map(describe) : [],
    };
  }));
}

interface HumanProcessIdentity {
  pid: number;
  parentPid: number;
  started: string;
  command: string;
  executable?: string;
}
interface HumanLifetime {
  label: string;
  app: ElectronApplication;
  page: Page;
  pid: number;
  identities: HumanProcessIdentity[];
  stdout: string[];
  stderr: string[];
  traceStopped: boolean;
  closed: boolean;
}
interface HumanCapture {
  info: TestInfo;
  lifetimes: HumanLifetime[];
  evidence: Array<Record<string, unknown>>;
  pageEvents: Array<Record<string, unknown>>;
  pageErrors: Array<Record<string, unknown>>;
}
interface HumanDbSnapshot {
  relativePath: string;
  conversationId: string;
  metadata: Record<string, unknown>;
  rows: Array<{ seq: number; id: string; entry: Record<string, unknown> }>;
}

async function sha256File(file: string) {
  const digest = createHash('sha256');
  let bytes = 0;
  for await (const chunk of createReadStream(file)) { digest.update(chunk); bytes += chunk.length; }
  return { sha256: digest.digest('hex'), bytes };
}

async function startHumanCapture(info: TestInfo): Promise<HumanCapture> {
  expect(process.env.GITHUB_ACTIONS, 'NMR-ELECTRON-01 executes only in exact-H GitHub Actions').toBe('true');
  const [major, minor] = process.versions.node.split('.').map(Number);
  expect(major > 22 || (major === 22 && minor >= 16), 'read-only node:sqlite requires the declared Node >=22.16 runner').toBe(true);
  const { stdout } = await execFileAsync('git', ['rev-parse', 'HEAD'], { cwd: appRoot });
  const head = stdout.trim();
  const target = process.env.NMR_ELECTRON_TARGET_SHA || process.env.GITHUB_SHA || '';
  expect(target).toMatch(/^[0-9a-f]{40}$/);
  expect(head).toBe(target);
  const capture: HumanCapture = { info, lifetimes: [], evidence: [], pageEvents: [], pageErrors: [] };
  const files = [
    'dist/electron-main/main.cjs', 'dist/electron-preload/preload.cjs',
    'dist/node-agent-coordinator/main.cjs', 'dist/renderer/index.html',
    'resources/bin/mahayana-node-agent-coordinator',
    process.env.MAHAYANA_APP_HOST_BIN || 'resources/bin/mahayana-app-host',
  ];
  const inputs = [];
  for (const file of files) {
    const resolved = path.resolve(appRoot, file);
    inputs.push({ path: path.relative(appRoot, resolved), ...await sha256File(resolved) });
  }
  capture.evidence.push({
    kind: 'exact-build-inputs', head, workflowHead: process.env.GITHUB_SHA,
    node: process.versions.node, inputs,
    scope: 'real Electron/IPC/Host with a controlled loopback HTTP service and test-owned directory',
  });
  return capture;
}

async function readProcessTable(): Promise<HumanProcessIdentity[]> {
  const { stdout } = await execFileAsync('ps', ['-ww', '-axo', 'pid=,ppid=,lstart=,comm='], {
    env: { ...process.env, LC_ALL: 'C' }, timeout: 5_000, maxBuffer: 4 * 1024 * 1024,
  });
  return stdout.split('\n').flatMap((line) => {
    const match = /^\s*(\d+)\s+(\d+)\s+(\w{3}\s+\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}\s+\d{4})\s+(.+)$/.exec(line);
    return match == null ? [] : [{
      pid: Number(match[1]), parentPid: Number(match[2]),
      started: match[3].replace(/\s+/g, ' '), command: match[4].trim(),
    }];
  });
}

function processTree(table: HumanProcessIdentity[], rootPid: number) {
  const selected = new Set([rootPid]);
  let size = -1;
  while (size !== selected.size) {
    size = selected.size;
    for (const entry of table) if (selected.has(entry.parentPid)) selected.add(entry.pid);
  }
  return table.filter((entry) => selected.has(entry.pid));
}

function observedLiveProcessTree(table: HumanProcessIdentity[], lifetime: Pick<HumanLifetime, 'identities'>) {
  const owned = table.filter((current) => lifetime.identities.some((prior) =>
    prior.pid === current.pid && prior.started === current.started && prior.command === current.command));
  const selected = new Set<number>();
  // Expand only from still-identical observed parents. A reused root PID must
  // never grant ownership of an unrelated replacement process or its children.
  for (const parent of owned) for (const entry of processTree(table, parent.pid)) selected.add(entry.pid);
  return table.filter((entry) => selected.has(entry.pid));
}

async function remainingHumanProcesses(lifetime: Pick<HumanLifetime, 'identities'>) {
  const table = await readProcessTable();
  return table.filter((current) => lifetime.identities.some((prior) =>
    prior.pid === current.pid && prior.started === current.started && prior.command === current.command));
}

async function killFailedHumanProcesses(
  lifetime: Pick<HumanLifetime, 'label' | 'pid' | 'identities'>, capture: HumanCapture,
) {
  const current = observedLiveProcessTree(await readProcessTable(), lifetime);
  for (const entry of current) if (!lifetime.identities.some((prior) =>
    prior.pid === entry.pid && prior.started === entry.started && prior.command === entry.command)) lifetime.identities.push(entry);
  const survivors = await remainingHumanProcesses(lifetime);
  // Failure cleanup is limited to observed, still-identical owned processes.
  // It never converts a failed graceful-reopen assertion into a pass.
  for (const entry of [...survivors].reverse()) {
    const stillOwned = (await remainingHumanProcesses(lifetime)).some((current) =>
      current.pid === entry.pid && current.started === entry.started && current.command === entry.command);
    if (!stillOwned) continue;
    capture.evidence.push({
      kind: 'failure-cleanup-signal', lifetime: lifetime.label, pid: entry.pid, started: entry.started, signal: 'SIGKILL',
    });
    try { process.kill(entry.pid, 'SIGKILL'); } catch (error) {
      capture.evidence.push({ kind: 'failure-cleanup-signal-error', lifetime: lifetime.label, error: String(error) });
    }
  }
  await expect.poll(() => remainingHumanProcesses(lifetime), { timeout: 10_000 }).toEqual([]);
}

async function captureHumanProcessChain(lifetime: HumanLifetime, capture: HumanCapture) {
  let currentTree: HumanProcessIdentity[] = [];
  await expect.poll(async () => {
    const tree = observedLiveProcessTree(await readProcessTable(), lifetime);
    if (process.platform === 'linux') {
      // Linux comm can be truncated. Resolve only already-owned PIDs; never
      // print a command's argv/bootstrap payload to identify its executable.
      for (const entry of tree) entry.executable = await readlink('/proc/' + entry.pid + '/exe').catch(() => undefined);
    }
    currentTree = tree;
    for (const entry of tree) if (!lifetime.identities.some((prior) =>
      prior.pid === entry.pid && prior.started === entry.started && prior.command === entry.command)) lifetime.identities.push(entry);
    return {
      coordinator: tree.filter((entry) => path.basename(entry.executable ?? entry.command) === 'mahayana-node-agent-coordinator').length,
      host: tree.filter((entry) => path.basename(entry.executable ?? entry.command) === 'mahayana-app-host').length,
    };
  }, { timeout: 20_000, message: 'Real Rust Coordinator and AppHost must be descendants of this Electron lifetime' })
    .toEqual({ coordinator: 1, host: 1 });
  const coordinator = currentTree.find((entry) => path.basename(entry.executable ?? entry.command) === 'mahayana-node-agent-coordinator')!;
  const host = currentTree.find((entry) => path.basename(entry.executable ?? entry.command) === 'mahayana-app-host')!;
  expect(host.parentPid).toBe(coordinator.pid);
  const metrics = await lifetime.app.evaluate(({ app }) => app.getAppMetrics().map((metric) => ({
    pid: metric.pid, type: metric.type, name: metric.name, serviceName: metric.serviceName, creationTime: metric.creationTime,
  })));
  const carrier = metrics.find((metric) => metric.pid === coordinator.parentPid);
  expect(carrier, 'The Rust Coordinator must be launched by the actual Electron utility-process carrier').toMatchObject({
    type: 'Utility', name: 'sand-node-agent-coordinator',
  });
  capture.evidence.push({ kind: 'production-process-chain', lifetime: lifetime.label, processes: lifetime.identities, currentTree, carrier, metrics });
}

async function launchHumanLifetime(appDataDir: string, label: string, capture: HumanCapture): Promise<HumanLifetime> {
  const app = await launchDesktopApp(appDataDir, { info: capture.info, label });
  const child = app.process();
  const pid = child.pid;
  if (pid == null) throw new Error('Actual Electron child omitted its PID');
  const stdout: string[] = [];
  const stderr: string[] = [];
  child.stdout?.on('data', (chunk) => stdout.push(String(chunk)));
  child.stderr?.on('data', (chunk) => stderr.push(String(chunk)));
  let initialIdentities: HumanProcessIdentity[] = [];
  let page: Page;
  try {
    initialIdentities = processTree(await readProcessTable(), pid);
    expect(initialIdentities.some((entry) => entry.pid === pid), 'Capture the actual Electron process identity before observing its children').toBe(true);
    page = await app.firstWindow();
  }
  catch (error) {
    const failed = { label, pid, identities: initialIdentities };
    capture.evidence.push({ kind: 'first-window-failure', lifetime: label, error: String(error), observedProcesses: failed.identities });
    try {
      await withPluginsDeadline('Windowless Human Electron cleanup', app.close(), 10_000).catch(() => undefined);
      await killFailedHumanProcesses(failed, capture);
    } finally {
      capture.evidence.push({ kind: 'windowless-electron-output', lifetime: label, stdout: stdout.join(''), stderr: stderr.join('') });
    }
    throw error;
  }
  const lifetime: HumanLifetime = {
    label, app, page, pid, identities: initialIdentities, stdout, stderr, traceStopped: false, closed: false,
  };
  capture.lifetimes.push(lifetime);
  page.on('console', (message) => capture.pageEvents.push({
    at: Date.now(), lifetime: label, kind: 'console', level: message.type(), text: message.text(), location: message.location(),
  }));
  page.on('pageerror', (error) => {
    const event = { at: Date.now(), lifetime: label, kind: 'pageerror', message: error.message, stack: error.stack };
    capture.pageErrors.push(event); capture.pageEvents.push(event);
  });
  page.on('requestfailed', (request) => capture.pageEvents.push({
    at: Date.now(), lifetime: label, kind: 'requestfailed', url: request.url(), failure: request.failure(),
  }));
  page.on('response', (response) => capture.pageEvents.push({
    at: Date.now(), lifetime: label, kind: 'renderer-response', url: response.url(), status: response.status(),
  }));
  let readinessPhase = 'video-recorder';
  try {
    const video = page.video();
    expect(video, 'Each Human Electron lifetime must retain its requested video capture').not.toBeNull();
    // In pinned Playwright, a failed page initialization can still emit a
    // firstWindow with a dummy frame. The video handle alone is lazy; path()
    // proves the recorder actually created its artifact before target resume.
    const videoPath = await withPluginsDeadline('Human Electron video recorder initialization', video!.path(), 5_000);
    readinessPhase = 'production-document';
    await page.waitForURL(/^(app:|file:)/, { waitUntil: 'domcontentloaded', timeout: 15_000 });
    readinessPhase = 'preload-ipc';
    await page.waitForFunction(() => {
      const desktop = (window as unknown as {
        desktop?: {
          cursorAccount?: { getStatus?: unknown };
          onboarding?: { setSeen?: unknown };
        };
      }).desktop;
      return typeof desktop?.cursorAccount?.getStatus === 'function'
        && typeof desktop?.onboarding?.setSeen === 'function';
    }, undefined, { timeout: 15_000 });
    capture.evidence.push({
      kind: 'electron-capture-ready', lifetime: label, at: Date.now(),
      url: page.url(), videoArtifact: path.basename(videoPath), documentReady: 'domcontentloaded', preloadReady: true,
    });
    readinessPhase = 'trace-start';
    // Electron.launch in pinned Playwright does not enter the test runner's
    // automatic context trace hook. This explicit per-process trace is its
    // only capture owner; failure here must fail the case, never skip tracing.
    await withPluginsDeadline('Human Electron trace start',
      app.context().tracing.start({ screenshots: true, snapshots: true, sources: true }), 15_000);
    capture.evidence.push({ kind: 'electron-trace-started', lifetime: label, at: Date.now() });
  } catch (error) {
    let windows: unknown;
    try {
      windows = await withPluginsDeadline('Human Electron startup diagnostics', app.evaluate(({ BrowserWindow }) =>
        BrowserWindow.getAllWindows().map((window) => ({
          destroyed: window.isDestroyed(),
          rendererDestroyed: window.webContents.isDestroyed(),
          url: window.webContents.isDestroyed() ? null : window.webContents.getURL(),
          loading: window.webContents.isDestroyed() ? null : window.webContents.isLoading(),
        }))), 5_000);
    } catch (diagnosticError) {
      windows = { diagnosticError: String(diagnosticError) };
    }
    capture.evidence.push({
      kind: 'electron-capture-startup-failure', lifetime: label, at: Date.now(),
      phase: readinessPhase, url: page.url(), closed: page.isClosed(), windows,
      error: error instanceof Error ? error.stack ?? error.message : String(error),
    });
    throw error;
  }
  const runtime = await app.evaluate(({ app, safeStorage }) => ({
    pid: process.pid, electron: process.versions.electron, node: process.versions.node,
    appPath: app.getAppPath(), userData: app.getPath('userData'), packaged: app.isPackaged,
    encryptionAvailable: safeStorage.isEncryptionAvailable(),
    storageBackend: process.platform === 'linux' ? safeStorage.getSelectedStorageBackend() : process.platform,
    simulatedUnavailable: process.env.SAND_SIMULATE_SECURE_STORAGE_UNAVAILABLE === '1',
    ciSessionSupplied: Boolean(process.env.FABUSHI_CI_ACCOUNT_SESSION_FILE),
  }));
  capture.evidence.push({ kind: 'electron-runtime', lifetime: label, ...runtime });
  expect(path.resolve(runtime.userData)).toBe(path.resolve(appDataDir));
  expect(runtime.ciSessionSupplied).toBe(false);
  expect(runtime.simulatedUnavailable, 'Simulated unavailable storage cannot qualify account persistence').toBe(false);
  expect(runtime.encryptionAvailable, 'Persistent account recovery requires actual Electron safeStorage').toBe(true);
  await app.evaluate(({ BrowserWindow }) => {
    const window = BrowserWindow.getAllWindows()[0];
    if (window == null) throw new Error('Production BrowserWindow is absent');
    window.setContentSize(1280, 900, false);
  });
  return lifetime;
}

async function readHumanAccount(lifetime: HumanLifetime, capture: HumanCapture) {
  const status = await lifetime.page.evaluate(async () => {
    const desktop = (window as unknown as {
      desktop: { cursorAccount: { getStatus(): Promise<{ kind?: string; authId?: string }> } };
    }).desktop;
    const value = await desktop.cursorAccount.getStatus();
    return { kind: value.kind, authId: value.authId };
  });
  capture.evidence.push({ at: Date.now(), kind: 'real-preload-ipc-read', lifetime: lifetime.label, method: 'cursorAccount.getStatus', status });
  return status;
}

async function assertHumanSessionSaved(appDataDir: string, capture: HumanCapture) {
  const file = path.join(appDataDir, 'sand-secrets.json');
  await expect.poll(async () => {
    const values = JSON.parse(await readFile(file, 'utf8')) as Record<string, unknown>;
    const saved = values['fabushi-account-session-v1'];
    return typeof saved === 'string' && saved.length > 0 && !saved.startsWith('plaintext:v1:');
  }, { message: 'The first real Sign in must save an encrypted session, not an in-memory fallback' }).toBe(true);
  capture.evidence.push({
    kind: 'encrypted-session-file', file: 'sand-secrets.json', accountSessionKeyPresent: true, ...await sha256File(file),
  });
}

async function openHumanWorkspace(lifetime: HumanLifetime, capture: HumanCapture, restore: boolean) {
  const page = lifetime.page;
  if (restore) {
    await page.waitForFunction(() => {
      const candidate = window as unknown as { desktop?: { cursorAccount?: { getStatus?: unknown } } };
      return typeof candidate.desktop?.cursorAccount?.getStatus === 'function';
    }, undefined, { timeout: 20_000 });
    // No second Sign in, CI session file, secret injection or silent fallback.
    await expect.poll(() => readHumanAccount(lifetime, capture), { timeout: 20_000 })
      .toEqual({ kind: 'logged-in', authId: humanAccountId });
    await expect(page.getByRole('main', { name: 'Fabushi', exact: true })).toBeHidden();
  } else {
    await completeBrowserLogin(page, 'agent-network', { ensureAgent: false });
    expect(await readHumanAccount(lifetime, capture)).toEqual({ kind: 'logged-in', authId: humanAccountId });
  }
  expect(page.url(), 'Actual packaged renderer entry, not the loopback service or a browser fake-port fixture').toMatch(/^(app:|file:)/);
  await expect(page.getByTestId('messenger-workspace')).toHaveAttribute('data-agent-root-shell', 'true');
  await expect(page.getByRole('button', { name: humanPeerName, exact: true })).toBeVisible({ timeout: 30_000 });
  await page.getByRole('button', { name: humanPeerName, exact: true }).click();
  await expect(humanEditor(page)).toBeEditable();
  await captureHumanProcessChain(lifetime, capture);
}

const humanTranscript = (page: Page) => page.getByRole('log', { name: 'Conversation transcript' });
const humanEditor = (page: Page) => page.locator('.sand-prompt-form [contenteditable]');
const humanRow = (page: Page, id: string) => humanTranscript(page).locator('[data-entry-id="' + id + '"]');

async function assertHumanCanonicalUi(page: Page, message: HumanRemoteMessage) {
  const row = humanRow(page, 'human-server-message:' + message.id);
  await expect(row).toHaveCount(1);
  const body = row.locator('.sand-message-prose');
  await expect(body).toHaveCount(1);
  await expect(body).toHaveText(message.text);
  await expect(row).not.toHaveAttribute('data-pending', 'true');
  await expect(row).not.toHaveAttribute('data-failed', 'true');
  await expect(row.locator('time')).toHaveAttribute('datetime', message.createdAt);
  await expect(humanRow(page, 'human-message:' + message.clientRequestId)).toHaveCount(0);
  await expect(humanRow(page, 'pending-' + message.clientRequestId)).toHaveCount(0);
  // These cases compose every Send while connected; a held receipt is not
  // evidence that the user composed the message while offline.
  await expect(row.locator('.sand-sent-while-offline-notice')).toHaveCount(0);
}

async function sendHumanThroughUi(lifetime: HumanLifetime, scenario: HumanElectronScenario, text: string) {
  const before = scenario.requests.filter((request) => request.method === 'POST' && request.path === '/api/social/messages').length;
  await humanEditor(lifetime.page).fill(text);
  await lifetime.page.getByRole('button', { name: 'Send message', exact: true }).click();
  await expect.poll(() => scenario.requests.filter((request) => request.method === 'POST' && request.path === '/api/social/messages').length)
    .toBeGreaterThan(before);
  const request = scenario.requests.filter((entry) => entry.method === 'POST' && entry.path === '/api/social/messages')[before];
  await expect.poll(() => request.body).toMatchObject({ targetUserId: humanPeerId, text, clientRequestId: expect.any(String) });
  const nonce = (request.body as { clientRequestId: string }).clientRequestId;
  await expect.poll(() => scenario.messages.has(nonce)).toBe(true);
  return scenario.messages.get(nonce)!;
}

async function readHumanDatabase(appDataDir: string): Promise<HumanDbSnapshot> {
  const files: string[] = [];
  const walk = async (directory: string): Promise<void> => {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      if (entry.isSymbolicLink()) continue;
      const filename = path.join(directory, entry.name);
      if (entry.isDirectory()) await walk(filename);
      else if (entry.isFile() && entry.name === 'store.db'
        && path.basename(path.dirname(path.dirname(filename))) === '.conversations') files.push(filename);
    }
  };
  await walk(path.join(appDataDir, 'sand-data'));
  expect(files, 'One real account-bound Human conversation store must exist').toHaveLength(1);
  const file = files[0];
  expect((await lstat(file)).isSymbolicLink()).toBe(false);
  // Read-only diagnostics of the real Session owner. Never seed, repair or
  // mutate its tables, metadata, outbox or coverage from this harness.
  const { DatabaseSync } = await import('node:sqlite');
  const db = new DatabaseSync(file, { readOnly: true, timeout: 5_000 });
  try {
    const raw = db.prepare('SELECT seq, id, entry FROM transcript_entries ORDER BY seq').all() as unknown as Array<{ seq: number; id: string; entry: string }>;
    const stored = db.prepare("SELECT value FROM kv WHERE key = 'metadata'").get() as { value: string } | undefined;
    expect(stored?.value).toMatch(/^(?:[0-9a-f]{2})+$/i);
    const metadata = JSON.parse(Buffer.from(stored!.value, 'hex').toString('utf8')) as Record<string, unknown>;
    expect(metadata).toMatchObject({ conversationKind: 'human', title: humanPeerName });
    expect(metadata.participantIds).toEqual([humanAccountId, humanPeerId]);
    return {
      relativePath: path.relative(appDataDir, file), conversationId: path.basename(path.dirname(file)), metadata,
      rows: raw.map((entry) => ({ seq: Number(entry.seq), id: entry.id, entry: JSON.parse(entry.entry) as Record<string, unknown> })),
    };
  } finally { db.close(); }
}

function assertHumanCanonicalDb(snapshot: HumanDbSnapshot, message: HumanRemoteMessage) {
  const matches = snapshot.rows.filter((row) => row.entry.clientNonce === message.clientRequestId);
  expect(matches).toHaveLength(1);
  expect(matches[0].entry).toMatchObject({
    id: 'human-server-message:' + message.id, kind: 'message', role: 'user',
    authorKind: 'human', authorId: humanAccountId, clientNonce: message.clientRequestId,
    content: message.text, delivery: 'sent', remoteMessageId: String(message.id),
    timestampMs: Date.parse(message.createdAt), remoteCreatedAt: message.createdAt, reactions: [],
  });
  expect(snapshot.rows.some((row) => row.id === 'human-message:' + message.clientRequestId)).toBe(false);
}

async function readHumanDraft(lifetime: HumanLifetime, capture: HumanCapture, conversationId: string) {
  const key = 'sand.client.slice.account.' + humanAccountId + '.composer-drafts';
  const raw = await lifetime.page.evaluate(async (storageKey) => {
    const desktop = (window as unknown as {
      desktop: { agent: { clientPersistence: { read(key: string): Promise<string | null> } } };
    }).desktop;
    return await desktop.agent.clientPersistence.read(storageKey);
  }, key);
  const envelope = raw == null ? null : JSON.parse(raw);
  const draft = envelope?.value?.agents?.[conversationId] ?? null;
  capture.evidence.push({
    at: Date.now(), kind: 'real-preload-ipc-read', lifetime: lifetime.label,
    method: 'agent.clientPersistence.read', key, draft,
  });
  return draft as {
    draftId: string | null;
    draft: { prompt: string } | null;
    recovery: unknown;
    recoveryId?: string | null;
    submissions?: Array<{ authorId: string; nonce: string; draftId: string | null; recoveryId: string | null }>;
  } | null;
}

async function humanMilestone(lifetime: HumanLifetime, capture: HumanCapture, name: string) {
  const dom = await lifetime.page.evaluate(() => ({
    url: location.href,
    rows: [...document.querySelectorAll('[role="log"][aria-label="Conversation transcript"] [data-entry-id]')].map((entry) => ({
      id: entry.getAttribute('data-entry-id'), role: entry.getAttribute('data-role'),
      text: entry.querySelector('.sand-message')?.textContent,
      pending: entry.getAttribute('data-pending'), failed: entry.getAttribute('data-failed'),
      timestamp: entry.querySelector('time')?.getAttribute('datetime'),
    })),
    composer: document.querySelector('.sand-prompt-form [contenteditable]')?.textContent,
    notice: document.querySelector('.sand-prompt-attachment-notice')?.textContent,
  }));
  await capture.info.attach(name + '-dom', { body: JSON.stringify(dom, null, 2), contentType: 'application/json' });
  const screenshot = capture.info.outputPath(name + '.png');
  await lifetime.page.screenshot({ path: screenshot, fullPage: true, timeout: 5_000 });
  await capture.info.attach(name, { path: screenshot, contentType: 'image/png' });
}

async function closeHumanLifetime(lifetime: HumanLifetime, capture: HumanCapture) {
  if (lifetime.closed) throw new Error('Human Electron lifetime was closed twice');
  const ownedProcess = lifetime.app.process();
  const current = observedLiveProcessTree(await readProcessTable(), lifetime);
  for (const entry of current) if (!lifetime.identities.some((prior) =>
    prior.pid === entry.pid && prior.started === entry.started && prior.command === entry.command)) lifetime.identities.push(entry);
  capture.evidence.push({ kind: 'owned-processes-before-shutdown', lifetime: lifetime.label, processes: lifetime.identities });
  const tracePath = capture.info.outputPath(lifetime.label + '-trace.zip');
  await withPluginsDeadline('Human Electron trace stop', lifetime.app.context().tracing.stop({ path: tracePath }), 15_000);
  lifetime.traceStopped = true;
  await capture.info.attach(lifetime.label + '-trace', { path: tracePath, contentType: 'application/zip' });
  const started = Date.now();
  capture.evidence.push({ kind: 'shutdown-requested', lifetime: lifetime.label, at: started, requestedMode: 'ElectronApplication.close' });
  await withPluginsDeadline('Human Electron graceful shutdown', lifetime.app.close(), 15_000);
  // app.close alone does not establish that detached Rust/Host descendants
  // exited. A PID is compared with its start time and executable to avoid reuse.
  await expect.poll(() => remainingHumanProcesses(lifetime), {
    timeout: 15_000, message: 'Old Electron, utility carrier, Rust Coordinator and AppHost must actually exit before reopen',
  }).toEqual([]);
  lifetime.closed = true;
  capture.evidence.push({
    kind: 'shutdown-observed', lifetime: lifetime.label, at: Date.now(), durationMs: Date.now() - started,
    actualMode: 'ElectronApplication.close', exitCode: ownedProcess.exitCode,
    signal: ownedProcess.signalCode, remainingObservedProcesses: [],
  });
}

async function finishHumanCapture(appDataDir: string, scenario: HumanElectronScenario, capture: HumanCapture) {
  const cleanupErrors: unknown[] = [];
  for (const held of scenario.held) if (!held.response.destroyed && !held.response.writableEnded) held.response.destroy();
  for (const lifetime of capture.lifetimes) {
    if (lifetime.closed) continue;
    try {
      if (!lifetime.traceStopped) {
        const failureTrace = capture.info.outputPath(lifetime.label + '-failure-trace.zip');
        await withPluginsDeadline('Failed Human Electron trace preservation',
          lifetime.app.context().tracing.stop({ path: failureTrace }), 5_000)
          .then(() => capture.info.attach(lifetime.label + '-failure-trace', { path: failureTrace, contentType: 'application/zip' }))
          .catch((error: unknown) => capture.evidence.push({ kind: 'failure-trace-error', lifetime: lifetime.label, error: String(error) }));
      }
      if (!lifetime.page.isClosed()) {
        await withPluginsDeadline('Failed Human renderer diagnostics',
          humanMilestone(lifetime, capture, lifetime.label + '-failure'), 6_000).catch((error: unknown) =>
          capture.evidence.push({ kind: 'failure-capture-error', lifetime: lifetime.label, error: String(error) }));
      }
      const current = observedLiveProcessTree(await readProcessTable(), lifetime);
      for (const entry of current) if (!lifetime.identities.some((prior) =>
        prior.pid === entry.pid && prior.started === entry.started && prior.command === entry.command)) lifetime.identities.push(entry);
      await withPluginsDeadline('Failed Human Electron cleanup', lifetime.app.close(), 10_000).catch(() => undefined);
      await killFailedHumanProcesses(lifetime, capture);
    } catch (error) {
      cleanupErrors.push(error);
      capture.evidence.push({ kind: 'failure-cleanup-error', lifetime: lifetime.label, error: String(error) });
    }
  }
  activeHumanScenario = null;
  for (const lifetime of capture.lifetimes) {
    await capture.info.attach(lifetime.label + '-electron-stdout', { body: lifetime.stdout.join(''), contentType: 'text/plain' });
    await capture.info.attach(lifetime.label + '-electron-stderr', { body: lifetime.stderr.join(''), contentType: 'text/plain' });
    try {
      const video = lifetime.page.video();
      if (video == null) throw new Error('Explicit Electron recordVideo produced no video handle');
      const file = await withPluginsDeadline('Electron video finalization', video.path(), 5_000);
      await capture.info.attach(lifetime.label + '-video', { path: file, contentType: 'video/webm' });
    } catch (error) {
      cleanupErrors.push(error);
      capture.evidence.push({ kind: 'video-artifact-error', lifetime: lifetime.label, error: String(error) });
    }
  }
  if (cleanupErrors.length === 0) {
    try { await rm(appDataDir, { recursive: true, force: true }); } catch (error) {
      cleanupErrors.push(error);
      capture.evidence.push({ kind: 'test-directory-removal-error', path: appDataDir, error: String(error) });
    }
  }
  if (cleanupErrors.length !== 0) capture.evidence.push({ kind: 'test-directory-retained-after-cleanup-failure', path: appDataDir });
  await capture.info.attach('human-http-request-response-release-order', {
    body: JSON.stringify({
      requests: scenario.requests, events: scenario.events, unexpected: scenario.unexpected,
      canonicalServiceRows: [...scenario.messages.values()],
      held: scenario.held.map((entry) => ({
        request: entry.request.sequence, clientRequestId: entry.message.clientRequestId,
        closed: entry.closed, released: entry.released, destroyed: entry.response.destroyed,
      })),
    }, null, 2), contentType: 'application/json',
  });
  await capture.info.attach('human-process-ipc-and-sqlite-evidence', { body: JSON.stringify(capture.evidence, null, 2), contentType: 'application/json' });
  await capture.info.attach('human-page-console-and-errors', { body: JSON.stringify(capture.pageEvents, null, 2), contentType: 'application/json' });
  if (cleanupErrors.length !== 0) throw new AggregateError(cleanupErrors, 'Human Electron cleanup/artifact capture failed; inspect retained evidence');
}

// FPM-UI-CLOSE-01: use the shipping renderer, its real WindowChrome and the
// controlled Host/auth backend above. No isolated PluginsBrowser imitation and
// no credential or marketplace readiness gate may stand in for the Close path.
for (const viewport of [{ width: 1024, height: 684 }, { width: 1280, height: 900 }]) {
  test(`shipping Plugins close ${viewport.width}x${viewport.height} uses a real pointer and preserves the window-control band`, async () => {
    const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-plugins-close-'));
    let app: ElectronApplication | null = null;
    let page: Page | null = null;
    try {
      app = await launchDesktopApp(appDataDir);
      page = await app.firstWindow();
      await completeBrowserLogin(page, 'plugins');
      await expect(page.getByTestId('messenger-workspace')).toHaveAttribute('data-agent-root-shell', 'true');
      await app.evaluate(({ BrowserWindow }, size) => {
        const win = BrowserWindow.getAllWindows()[0];
        if (!win) throw new Error('Shipping BrowserWindow is absent');
        win.setContentSize(size.width, size.height, false);
      }, viewport);
      await expect.poll(() => page!.evaluate(() => ({ width: innerWidth, height: innerHeight }))).toEqual(viewport);

      const plugins = page.getByRole('button', { name: 'Plugins', exact: true });
      const dialog = page.getByRole('dialog', { name: 'Plugins', exact: true });
      for (const phase of ['initial', 'reopened']) {
        await plugins.click({ timeout: 5_000 });
        await expect(dialog).toBeVisible({ timeout: 5_000 });
        const geometry = await pluginsPointerGeometry(page);
        await test.info().attach(`plugins-pointer-${phase}`, { body: Buffer.from(JSON.stringify(geometry, null, 2)), contentType: 'application/json' });
        expect(geometry.closeCount).toBe(1);
        expect(geometry.close).not.toBeNull();
        expect(geometry.frame).not.toBeNull();
        expect(geometry.drag).not.toBeNull();
        expect(geometry.centerHitsClose, 'Close must receive the normal pointer before any click is sent').toBe(true);
        expect(geometry.close!.style.appRegion).toBe('no-drag');
        expect(geometry.frame!.bounds.y).toBeGreaterThanOrEqual(geometry.drag!.bounds.y + geometry.drag!.bounds.height);
        expect(geometry.close!.bounds.y).toBeGreaterThanOrEqual(geometry.drag!.bounds.y + geometry.drag!.bounds.height);
        expect(geometry.close!.bounds.y + geometry.close!.bounds.height).toBeLessThanOrEqual(viewport.height);
        expect(Number(geometry.frame!.style.zIndex)).toBeGreaterThan(Number(geometry.drag!.style.zIndex));
        if (geometry.header) expect(Number(geometry.frame!.style.zIndex)).toBeGreaterThan(Number(geometry.header.style.zIndex));
        for (const control of geometry.windowControls) expect(control.centerHitsControl, 'Native window control remains reachable').toBe(true);
        await page.screenshot({ path: test.info().outputPath(`plugins-${phase}-${viewport.width}x${viewport.height}.png`), timeout: 4_000 });
        await dialog.getByRole('button', { name: 'Close', exact: true }).click({ timeout: 5_000 });
        await expect(dialog).toBeHidden({ timeout: 5_000 });
      }
      await expect(plugins).toBeVisible();
    } catch (error) {
      if (page) {
        const geometry = await pluginsPointerGeometry(page).catch((failure) => ({ captureError: String(failure) }));
        await test.info().attach('plugins-pointer-failure', { body: Buffer.from(JSON.stringify({ error: error instanceof Error ? error.stack : String(error), geometry }, null, 2)), contentType: 'application/json' });
        await withPluginsDeadline('Plugins failure screenshot', page.screenshot({ path: test.info().outputPath('plugins-failure.png'), timeout: 4_000 })).catch(() => undefined);
      }
      throw error;
    } finally {
      if (app) {
        const ownedProcess = app.process();
        await withPluginsDeadline('Plugins Electron shutdown', app.close(), 10_000).catch(() => {
          if (ownedProcess.exitCode == null && ownedProcess.signalCode == null) ownedProcess.kill('SIGKILL');
        });
      }
      await rm(appDataDir, { recursive: true, force: true });
    }
  });
}

// This acceptance intentionally imports no retired desktop/src/agent-workspace modules:
// it validates only the shipping recovered ProductShell composition after the architecture cutover,
// so exact-HEAD evidence cannot accidentally pass through the removed parallel owner.
test('shipping Agent Network binds to the live workspace and supports real wheel zoom', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-agent-network-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await expect(page.locator('body')).toHaveAttribute('data-fabushi-surface', 'grok-parity-v1');
    await completeBrowserLogin(page);

    await expect(page.getByTestId('messenger-workspace')).toHaveAttribute('data-agent-root-shell', 'true');
    await expect(page.getByTestId('messenger-workspace')).toHaveAttribute('data-product-shell', 'agent');
    await expect(page.locator('.desktop-mode-switch')).toHaveCount(0);

    await page.getByRole('button', { name: 'Agent network' }).click();
    const orgChart = page.getByRole('main').filter({ has: page.getByRole('heading', { name: 'Org chart' }) });
    await expect(orgChart.getByRole('heading', { name: 'Org chart' })).toBeVisible();

    const network = orgChart.getByRole('region', { name: 'Agent network' });
    await expect(network).toBeVisible();

    const firstGeometry = await measuredGeometry(network);
    expect(firstGeometry.width).toBeGreaterThan(0);
    expect(firstGeometry.height).toBeGreaterThan(0);
    expect(Math.abs(firstGeometry.width - firstGeometry.parentWidth)).toBeLessThanOrEqual(1);
    expect(Math.abs(firstGeometry.height - firstGeometry.parentHeight)).toBeLessThanOrEqual(1);

    await page.setViewportSize({ width: 980, height: 680 });
    await expect.poll(async () => {
      const geometry = await measuredGeometry(network);
      return Math.abs(geometry.width - geometry.parentWidth) <= 1
        && Math.abs(geometry.height - geometry.parentHeight) <= 1;
    }).toBe(true);

    const resizedGeometry = await measuredGeometry(network);
    expect(
      Math.abs(resizedGeometry.width - firstGeometry.width) > 1
        || Math.abs(resizedGeometry.height - firstGeometry.height) > 1,
    ).toBe(true);

    const firstAgentNode = network.locator('.sand-org-chart-network__node').first();
    await expect(firstAgentNode).toBeVisible();
    await expect(firstAgentNode).toHaveAttribute('aria-pressed', /true|false/);

    const scene = network.locator('.sand-org-chart-network__scene');
    // Exercise the native wheel path rather than dispatching a synthetic WheelEvent; the screenshot below is the visual evidence companion.
    const transformBeforeWheel = await scene.evaluate((element) => getComputedStyle(element).transform);
    const sceneBox = await scene.boundingBox();
    expect(sceneBox).not.toBeNull();
    await page.mouse.move(
      sceneBox!.x + Math.min(120, sceneBox!.width / 2),
      sceneBox!.y + Math.min(120, sceneBox!.height / 2),
    );
    await page.mouse.wheel(0, -240);
    await expect.poll(async () => scene.evaluate((element) => getComputedStyle(element).transform))
      .not.toBe(transformBeforeWheel);

    await network.screenshot({ path: test.info().outputPath('agent-network-responsive.png') });
    await expect(orgChart.getByText(/\d+ agents? · \d+ groups? · \d+ message links?/)).toBeVisible();
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});

// Exactly two independently executable cases. Each launches the shipping
// Electron entry and its real preload/carrier/Coordinator/AppHost chain.
// Authored cases are not execution evidence; only the exact-H Actions report
// and these process/HTTP/SQLite artifacts qualify this controlled boundary.
test.describe('NMR-ELECTRON-01 actual Human IPC', () => {
  test.describe.configure({ mode: 'default', retries: 0, timeout: 240_000 });

  test('NMR-ELECTRON-01 actual Human IPC retains a canonical row across offline same-directory reopen', async ({}, info) => {
    const capture = await startHumanCapture(info);
    const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-human-offline-reopen-'));
    const scenario = newHumanScenario();
    activeHumanScenario = scenario;
    try {
      const first = await launchHumanLifetime(appDataDir, 'offline-first', capture);
      await openHumanWorkspace(first, capture, false);
      await assertHumanSessionSaved(appDataDir, capture);
      expect(scenario.requests.some((request) => request.method === 'GET'
        && new URL(request.path, 'http://fixture.invalid').searchParams.get('afterId') === '0')).toBe(true);
      const message = await sendHumanThroughUi(first, scenario,
        'Canonical Human message: every character, punctuation, and 中文 survives the offline desktop restart.');
      await assertHumanCanonicalUi(first.page, message);
      const before = await readHumanDatabase(appDataDir);
      assertHumanCanonicalDb(before, message);
      expect(before.rows).toHaveLength(1);
      capture.evidence.push({ kind: 'sqlite-before-offline-close', snapshot: before });
      await humanMilestone(first, capture, 'canonical-before-offline-close');
      const browserStarts = scenario.requests.filter((request) => request.path === '/api/auth/browser/start').length;
      expect(browserStarts).toBe(1);
      scenario.historyAvailable = false;
      scenario.phase = 'offline-shutdown';
      await closeHumanLifetime(first, capture);

      scenario.phase = 'offline-reopened';
      const second = await launchHumanLifetime(appDataDir, 'offline-reopened', capture);
      await openHumanWorkspace(second, capture, true);
      expect(second.identities.find((entry) => entry.pid === second.pid))
        .not.toEqual(first.identities.find((entry) => entry.pid === first.pid));
      expect(scenario.requests.filter((request) => request.path === '/api/auth/browser/start')).toHaveLength(browserStarts);
      await expect.poll(() => scenario.events.filter((event) =>
        event.kind === 'http-reply' && event.phase === 'offline-reopened' && event.status === 503).length).toBeGreaterThan(0);
      // A failed remote sync must leave the durable local transcript usable.
      // Do not return a fake successful history page to make this UI pass.
      await assertHumanCanonicalUi(second.page, message);
      await expect(second.page.locator('.sand-prompt-attachment-notice')).toContainText(/Human sync unavailable|503|fixture-history-unavailable/);
      const offline = await readHumanDatabase(appDataDir);
      assertHumanCanonicalDb(offline, message);
      expect(offline.relativePath).toBe(before.relativePath);
      expect(offline.rows, 'The complete persisted row, identity and physical sequence survive without remote history').toEqual(before.rows);
      expect(scenario.messages.size).toBe(1);
      capture.evidence.push({ kind: 'sqlite-during-remote-503', snapshot: offline });
      await humanMilestone(second, capture, 'offline-reopened-local-history');

      scenario.historyAvailable = true;
      scenario.phase = 'service-recovered';
      // An actual renderer reload requests a new renderer/port scope and a
      // real sync; a cached-row click would not do so.
      await second.page.reload({ waitUntil: 'domcontentloaded' });
      await openHumanWorkspace(second, capture, true);
      await expect.poll(() => scenario.events.filter((event) =>
        event.kind === 'http-reply' && event.phase === 'service-recovered' && event.status === 200).length).toBeGreaterThan(0);
      await assertHumanCanonicalUi(second.page, message);
      const fresh = await sendHumanThroughUi(second, scenario,
        'The recovered service acknowledges a new Human send through the reopened desktop.');
      expect(fresh.clientRequestId).not.toBe(message.clientRequestId);
      await assertHumanCanonicalUi(second.page, fresh);
      await assertHumanCanonicalUi(second.page, message);
      const final = await readHumanDatabase(appDataDir);
      assertHumanCanonicalDb(final, message);
      assertHumanCanonicalDb(final, fresh);
      expect(final.rows).toHaveLength(2);
      expect(scenario.messages.size).toBe(2);
      capture.evidence.push({ kind: 'sqlite-after-service-positive-control', snapshot: final });
      await humanMilestone(second, capture, 'service-restored-new-send');
      await closeHumanLifetime(second, capture);
      expect(scenario.unexpected).toEqual([]);
      expect(capture.pageErrors).toEqual([]);
    } catch (error) {
      capture.evidence.push({ kind: 'case-failure', error: error instanceof Error ? error.stack : String(error) });
      throw error;
    } finally {
      await finishHumanCapture(appDataDir, scenario, capture);
    }
  });

  test('NMR-ELECTRON-01 actual Human IPC replays a committed nonce after held-receipt process reopen', async ({}, info) => {
    const capture = await startHumanCapture(info);
    const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-human-held-reopen-'));
    const scenario = newHumanScenario();
    activeHumanScenario = scenario;
    try {
      const first = await launchHumanLifetime(appDataDir, 'held-first', capture);
      await openHumanWorkspace(first, capture, false);
      await assertHumanSessionSaved(appDataDir, capture);
      // Keep the first lifetime genuinely unacknowledged: its POSTs commit
      // idempotently but wait, while remote history explicitly reports 503.
      // Neither a fabricated port event nor a planted SQLite row settles A.
      scenario.holdPosts = true;
      scenario.historyAvailable = false;
      const originalText = 'A was committed remotely while its first desktop receipt remained withheld.';
      const message = await sendHumanThroughUi(first, scenario, originalText);
      await expect.poll(() => scenario.held.length).toBeGreaterThan(0);
      expect(scenario.held.every((entry) => !entry.closed && !entry.released)).toBe(true);
      const pending = await readHumanDatabase(appDataDir);
      const pendingRows = pending.rows.filter((row) => row.entry.clientNonce === message.clientRequestId);
      expect(pendingRows).toHaveLength(1);
      expect(pendingRows[0].entry).toMatchObject({
        id: 'human-message:' + message.clientRequestId, authorKind: 'human', authorId: humanAccountId,
        clientNonce: message.clientRequestId, content: originalText,
      });
      expect(['pending', 'dispatching']).toContain(pendingRows[0].entry.delivery);
      expect(pendingRows[0].entry.remoteMessageId).toBeUndefined();
      await expect(humanRow(first.page, 'human-server-message:' + message.id)).toHaveCount(0);
      await expect.poll(async () => (await readHumanDraft(first, capture, pending.conversationId))?.draft?.prompt).toBe(originalText);
      const originalDraft = await readHumanDraft(first, capture, pending.conversationId);
      expect(originalDraft?.draft?.prompt).toBe(originalText);
      expect(originalDraft?.draftId).toEqual(expect.stringMatching(/\S/));
      // Observe the real persisted binding while the first POST response is
      // still held. This is a read through preload IPC, never a seeded owner.
      expect(originalDraft?.submissions).toEqual([{
        authorId: humanAccountId, nonce: message.clientRequestId,
        draftId: originalDraft?.draftId, recoveryId: null,
      }]);
      expect(originalDraft?.recovery ?? null).toBeNull();
      expect(originalDraft?.recoveryId ?? null).toBeNull();
      expect(scenario.held.every((entry) => !entry.closed && !entry.released)).toBe(true);
      capture.evidence.push({ kind: 'actual-pending-outbox-and-original-draft', snapshot: pending, originalDraft });
      await humanMilestone(first, capture, 'remote-commit-original-receipt-held');
      const browserStarts = scenario.requests.filter((request) => request.path === '/api/auth/browser/start').length;
      expect(browserStarts).toBe(1);
      scenario.phase = 'held-first-shutdown';
      await closeHumanLifetime(first, capture);
      await expect.poll(() => scenario.held.every((entry) => entry.closed)).toBe(true);
      const closed = await readHumanDatabase(appDataDir);
      const closedRows = closed.rows.filter((row) => row.entry.clientNonce === message.clientRequestId);
      expect(closedRows).toHaveLength(1);
      expect(['pending', 'dispatching']).toContain(closedRows[0].entry.delivery);
      expect(closedRows[0].entry.remoteMessageId).toBeUndefined();
      capture.evidence.push({ kind: 'durable-pending-after-actual-old-process-exit', snapshot: closed });

      scenario.phase = 'held-replay-reopened';
      scenario.holdPosts = false;
      scenario.historyAvailable = true;
      const second = await launchHumanLifetime(appDataDir, 'held-replay-reopened', capture);
      await openHumanWorkspace(second, capture, true);
      await assertHumanCanonicalUi(second.page, message);
      const replayRequests = scenario.requests.filter((request) =>
        request.phase === 'held-replay-reopened' && request.method === 'POST' && request.path === '/api/social/messages');
      expect(replayRequests.length, 'The actual restarted Host must replay its own durable outbox').toBeGreaterThan(0);
      for (const request of replayRequests) expect(request.body).toMatchObject({
        targetUserId: humanPeerId, text: originalText, clientRequestId: message.clientRequestId,
      });
      expect(scenario.messages.size).toBe(1);
      const recovered = await readHumanDatabase(appDataDir);
      assertHumanCanonicalDb(recovered, message);
      expect(recovered.rows).toHaveLength(1);
      expect(recovered.relativePath).toBe(pending.relativePath);
      // Before any edit in this lifetime, canonical replay must retire A's
      // recorded IDs and binding. Replacing A with B cannot satisfy this check.
      await expect.poll(async () => {
        const stored = await readHumanDraft(second, capture, recovered.conversationId);
        return {
          draft: stored?.draft ?? null, draftId: stored?.draftId ?? null,
          recovery: stored?.recovery ?? null, recoveryId: stored?.recoveryId ?? null,
          submissions: stored?.submissions ?? [],
        };
      }, { message: 'Canonical restart recovery must durably retire A before a new draft is typed' }).toEqual({
        draft: null, draftId: null, recovery: null, recoveryId: null, submissions: [],
      });
      await expect(humanEditor(second.page)).toHaveText('');
      const restoredOriginalDraft = await readHumanDraft(second, capture, recovered.conversationId);
      const draftObservation = {
        kind: 'original-A-draft-after-recovery', originalDraft, observed: restoredOriginalDraft,
        retirementStatus: 'asserted-empty-draft-recovery-and-binding-before-any-new-edit',
      };
      capture.evidence.push(draftObservation);
      await info.attach('original-A-draft-retirement', { body: JSON.stringify(draftObservation, null, 2), contentType: 'application/json' });
      await humanMilestone(second, capture, 'same-nonce-recovery-before-new-draft');

      // Equal text is deliberately not equal ownership. Every transition is
      // a real editor action followed by a real persisted-state observation.
      const newerText = originalText;
      const differentText = 'B is a new draft edited away from A before returning to the same text.';
      await expect(humanEditor(second.page)).toBeEditable();
      await humanEditor(second.page).fill(originalText);
      await expect.poll(async () => (await readHumanDraft(second, capture, recovered.conversationId))?.draft?.prompt).toBe(originalText);
      const firstNewDraft = await readHumanDraft(second, capture, recovered.conversationId);
      expect(firstNewDraft?.draftId).toEqual(expect.stringMatching(/\S/));
      expect(firstNewDraft?.draftId).not.toBe(originalDraft?.draftId);
      await humanEditor(second.page).fill(differentText);
      await expect.poll(async () => (await readHumanDraft(second, capture, recovered.conversationId))?.draft?.prompt).toBe(differentText);
      const differentDraft = await readHumanDraft(second, capture, recovered.conversationId);
      expect(differentDraft?.draftId).toEqual(expect.stringMatching(/\S/));
      expect(differentDraft?.draftId).not.toBe(firstNewDraft?.draftId);
      await humanEditor(second.page).fill(newerText);
      await expect.poll(async () => (await readHumanDraft(second, capture, recovered.conversationId))?.draft?.prompt).toBe(newerText);
      const newerDraft = await readHumanDraft(second, capture, recovered.conversationId);
      expect(newerDraft?.draftId).toEqual(expect.stringMatching(/\S/));
      expect(newerDraft?.draftId).not.toBe(originalDraft?.draftId);
      expect(newerDraft?.draftId).not.toBe(firstNewDraft?.draftId);
      expect(newerDraft?.draftId).not.toBe(differentDraft?.draftId);
      expect(newerDraft?.submissions ?? []).toEqual([]);
      capture.evidence.push({ kind: 'new-B-same-text-distinct-identity', originalDraft, firstNewDraft, differentDraft, newerDraft });
      await humanMilestone(second, capture, 'new-B-draft-before-another-real-reopen');
      scenario.phase = 'new-draft-shutdown';
      await closeHumanLifetime(second, capture);

      scenario.phase = 'new-draft-reopened';
      const third = await launchHumanLifetime(appDataDir, 'new-draft-reopened', capture);
      await openHumanWorkspace(third, capture, true);
      expect(scenario.requests.filter((request) => request.path === '/api/auth/browser/start')).toHaveLength(browserStarts);
      await assertHumanCanonicalUi(third.page, message);
      await expect(humanEditor(third.page)).toHaveText(newerText);
      expect(await readHumanDraft(third, capture, recovered.conversationId)).toEqual(newerDraft);
      expect(scenario.requests.filter((request) => request.phase === 'new-draft-reopened'
        && request.method === 'POST' && request.path === '/api/social/messages')).toEqual([]);
      expect(scenario.held.every((entry) => entry.closed && !entry.released)).toBe(true);
      releaseOldHumanResponses(scenario);
      await third.page.evaluate(async () => {
        await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
        await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
      });
      await assertHumanCanonicalUi(third.page, message);
      await expect(humanEditor(third.page)).toHaveText(newerText);
      expect(await readHumanDraft(third, capture, recovered.conversationId)).toEqual(newerDraft);
      expect(scenario.held.every((entry) => entry.closed && entry.released)).toBe(true);
      const beforeNewSend = await readHumanDatabase(appDataDir);
      assertHumanCanonicalDb(beforeNewSend, message);
      expect(beforeNewSend.rows).toHaveLength(1);
      capture.evidence.push({ kind: 'new-B-draft-survives-old-response-attempt', newerDraft, snapshot: beforeNewSend });
      await humanMilestone(third, capture, 'new-draft-after-old-socket-release');

      const fresh = await sendHumanThroughUi(third, scenario, newerText);
      expect(fresh.clientRequestId).not.toBe(message.clientRequestId);
      await assertHumanCanonicalUi(third.page, fresh);
      await assertHumanCanonicalUi(third.page, message);
      await expect(humanEditor(third.page)).toHaveText('');
      const final = await readHumanDatabase(appDataDir);
      assertHumanCanonicalDb(final, message);
      assertHumanCanonicalDb(final, fresh);
      expect(final.rows).toHaveLength(2);
      expect(scenario.messages.size).toBe(2);
      capture.evidence.push({ kind: 'new-lifetime-send-positive-control', snapshot: final });
      await humanMilestone(third, capture, 'three-generations-final-unique-canonical-messages');
      await closeHumanLifetime(third, capture);
      expect(scenario.unexpected).toEqual([]);
      expect(capture.pageErrors).toEqual([]);
    } catch (error) {
      capture.evidence.push({ kind: 'case-failure', error: error instanceof Error ? error.stack : String(error) });
      throw error;
    } finally {
      await finishHumanCapture(appDataDir, scenario, capture);
    }
  });
});

