import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;

let e2eAuthServer: ReturnType<typeof createServer> | null = null;
let e2eAuthBackendPromise: Promise<string> | null = null;

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
      if (requestUrl.pathname === '/api/social/friends' && request.method === 'GET') {
        if (request.headers.authorization !== `Bearer ${token}`
          || request.headers['x-fabushi-device-id'] !== 'fabushi-agent-network-e2e-device') {
          response.statusCode = 401;
          response.end(JSON.stringify({ success: false, error: 'invalid-session' }));
          return;
        }
        response.statusCode = 200;
        response.end(JSON.stringify({ success: true, data: { friends: [] } }));
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

async function launchDesktopApp(appDataDir: string) {
  const e2eAuthBackendUrl = await ensureE2eAuthBackend();
  return electron.launch({
    ...(packagedExecutable
      ? { executablePath: packagedExecutable, args: [] }
      : { args: [appRoot] }),
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
    },
  });
}

async function completeBrowserLogin(page: Page): Promise<void> {
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
  await expect(page.getByRole('region', { name: 'Agent list' })).toBeVisible({ timeout: 15_000 });
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
    // Exercise the native wheel path rather than dispatching a synthetic WheelEvent; the screenshot below is the visual evidence companion.\n    const transformBeforeWheel = await scene.evaluate((element) => getComputedStyle(element).transform);
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
