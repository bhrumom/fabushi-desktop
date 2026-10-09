import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { ElectronMahayanaHostTransport } from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import type { RuntimeEvent } from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { isTerminalAuthSessionFailure } from '../src/auth-session';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;

async function launchDesktopApp(appDataDir: string) {
  return electron.launch({
    ...(packagedExecutable
      ? { executablePath: packagedExecutable, args: [] }
      : { args: [appRoot] }),
    env: {
      ...process.env,
      FABUSHI_APP_DATA: appDataDir,
      FABUSHI_FEATURE_HOST_MODE: process.env.FABUSHI_FEATURE_HOST_MODE || 'test',
      MAHAYANA_APP_HOST_BIN: process.env.MAHAYANA_APP_HOST_BIN || '',
    },
  });
}

async function completeBrowserLogin(page: Page): Promise<void> {
  const onboardingGate = page.getByTestId('onboarding-gate');
  const loginGate = page.getByTestId('login-gate');
  const workspace = page.getByTestId('messenger-workspace');
  type LoginPhase = 'onboarding' | 'login' | 'ready' | 'waiting';

  // Read the auth surface in one renderer evaluation. During the HostClient ->
  // Messenger transition individual locator probes can straddle a destroyed
  // execution context and wait on navigation even though auth already finished.
  const readPhase = async (): Promise<LoginPhase> => {
    try {
      return await page.evaluate(() => {
        if (document.querySelector('[data-testid="onboarding-gate"]')) return 'onboarding';
        if (document.querySelector('[data-testid="login-gate"]')) return 'login';
        const messenger = document.querySelector('[data-testid="messenger-workspace"]');
        if (messenger?.getAttribute('data-initial-host-hydrated') === 'true') return 'ready';
        return 'waiting';
      }) as LoginPhase;
    } catch {
      return 'waiting';
    }
  };

  for (let phase = 0; phase < 12; phase += 1) {
    await expect.poll(readPhase, { timeout: 15_000 }).not.toBe('waiting');
    const currentPhase = await readPhase();

    if (currentPhase === 'onboarding') {
      await page.getByTestId('onboarding-next').click();
      continue;
    }
    if (currentPhase === 'login') {
      await page.getByTestId('browser-login-start').click();
      await expect(loginGate).toBeHidden();
      continue;
    }
    if (currentPhase === 'ready') break;
  }

  await expect(workspace).toHaveAttribute('data-initial-host-hydrated', 'true', { timeout: 15_000 });
  await expect(workspace).toBeVisible();
}

async function expectComposerInsideViewport(page: Page): Promise<void> {
  const input = page.getByTestId('messenger-input');
  await expect(input).toBeVisible();
  const box = await input.boundingBox();
  const viewport = await page.evaluate(() => ({
    width: window.innerWidth,
    height: window.innerHeight,
  }));
  expect(box, 'composer input must have a rendered bounding box').not.toBeNull();
  if (!box) return;
  expect(viewport.width).toBeGreaterThan(0);
  expect(viewport.height).toBeGreaterThan(0);
  expect(box.x).toBeGreaterThanOrEqual(0);
  expect(box.y).toBeGreaterThanOrEqual(0);
  expect(box.x + box.width).toBeLessThanOrEqual(viewport.width);
  expect(box.y + box.height).toBeLessThanOrEqual(viewport.height);
}

test('terminal auth classifier distinguishes revoked sessions from transient failures', () => {
  expect(isTerminalAuthSessionFailure('Mahayana product API returned HTTP 401: refresh_token_reused: 登录会话已撤销，请重新登录')).toBe(true);
  expect(isTerminalAuthSessionFailure(new Error('session_revoked'))).toBe(true);
  expect(isTerminalAuthSessionFailure('Mahayana product API transport failed: timeout')).toBe(false);
  expect(isTerminalAuthSessionFailure('HTTP 503 upstream unavailable')).toBe(false);
});

test('Electron transport keeps Mini Apps out of chat conversations and routes direct opens to miniapp.open', async () => {
  let runtimeListener: ((event: RuntimeEvent) => void) | null = null;
  const calls: Array<{ method: string; params?: Record<string, unknown> }> = [];
  const fakeWindow = {
    mahayana: {
      contractVersion: 1,
      async invoke<T>(method: string, params?: Record<string, unknown>): Promise<T> {
        calls.push({ method, params });
        if (method === 'feature.info') {
          return {
            runtimeVersion: 'test',
            protocolVersion: '1',
            platform: 'electron',
          } as T;
        }
        return { requestId: String((params?.command as { requestId?: string } | undefined)?.requestId ?? 'test') } as T;
      },
      subscribe(listener: (event: RuntimeEvent) => void) {
        runtimeListener = listener;
        return () => { runtimeListener = null; };
      },
    },
    fabushi: {
      contractVersion: 1,
      async notify() {},
      async openExternal() {},
      async openSystemSettings() {},
      async windowFocused() { return true; },
    },
    addEventListener() {},
    removeEventListener() {},
  };
  const globalWithWindow = globalThis as unknown as { window?: typeof fakeWindow };
  const previousWindow = globalWithWindow.window;
  globalWithWindow.window = fakeWindow;

  try {
    const transport = new ElectronMahayanaHostTransport();
    const observed: RuntimeEvent[] = [];
    transport.subscribe((event) => observed.push(event));
    await transport.initialize({ profileId: 'transport-regression', mode: 'test' });

    const listedEvent: RuntimeEvent = {
      type: 'conversation.listed',
      timestamp: new Date().toISOString(),
      conversations: [
        {
          id: 'official:bot-father',
          title: 'Bot Father',
          kind: 'miniapp',
          pinned: false,
          unreadCount: 0,
          updatedAtMs: Date.now(),
        },
        {
          id: 'mahayana:contact:alice',
          title: 'Alice',
          kind: 'contact',
          pinned: false,
          unreadCount: 0,
          updatedAtMs: Date.now(),
        },
      ],
    };
    const emitRuntime = runtimeListener as ((event: RuntimeEvent) => void) | null;
    expect(emitRuntime).not.toBeNull();
    emitRuntime?.(listedEvent);

    const normalized = observed.find((event) => event.type === 'conversation.listed');
    expect(normalized?.type).toBe('conversation.listed');
    if (normalized?.type === 'conversation.listed') {
      expect(normalized.conversations.map((conversation) => conversation.id)).toEqual(['mahayana:contact:alice']);
    }

    await transport.execute({
      type: 'conversation.open',
      requestId: 'open-bot-father',
      conversationId: 'official:bot-father',
    });
    const routed = calls.at(-1)?.params?.command as { type?: string; miniAppId?: string } | undefined;
    expect(routed?.type).toBe('miniapp.open');
    expect(routed?.miniAppId).toBe('bot-father');
    await transport.close();
  } finally {
    if (previousWindow === undefined) delete globalWithWindow.window;
    else globalWithWindow.window = previousWindow;
  }
});

test('Messenger composer remains inside the desktop viewport for chat peers', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-composer-regression-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);

    const peers = page.locator('[data-testid^="peer-"]');
    await expect(peers.first()).toBeVisible();
    const count = Math.min(await peers.count(), 12);
    expect(count).toBeGreaterThan(0);

    // Opening a peer can reorder the live list. Snapshot the stable testing contract
    // before any click so subsequent actions keep targeting the same identities.
    const peerTestIds: string[] = [];
    for (let index = 0; index < count; index += 1) {
      const testId = await peers.nth(index).getAttribute('data-testid');
      expect(testId).toBeTruthy();
      if (testId) peerTestIds.push(testId);
    }

    for (const peerTestId of peerTestIds) {
      const peer = page.getByTestId(peerTestId);
      await peer.scrollIntoViewIfNeeded();
      await peer.click();
      await expectComposerInsideViewport(page);
    }
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('switching peers keeps dynamic avatars visible and narrow layouts can open the info panel', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-avatar-info-regression-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);

    let peers = page.locator('[data-testid^="peer-"]');
    await expect(peers.first()).toBeVisible();
    if (await peers.count() < 2) {
      await page.getByRole('button', { name: '新建', exact: true }).click();
      await expect(page.getByTestId('create-bot')).toBeVisible();
      await page.getByTestId('create-bot').click();
      await page.getByTestId('new-bot-name').fill('头像切换验收 Bot');
      await page.getByTestId('new-bot-description').fill('验证真实用户 Bot 的动态头像与会话切换。');
      await page.getByTestId('create-bot-submit').click();
      const createdBot = page.locator('[data-testid^="peer-legacy:bot:"]').filter({ hasText: '头像切换验收 Bot' }).first();
      await expect(createdBot).toBeVisible({ timeout: 10_000 });
      // The production Grok parity runtime auto-focuses a newly created Bot on a deferred
      // timer. Wait for that lifecycle to settle before validating a later manual switch.
      await expect(createdBot).toHaveClass(/peerActive/, { timeout: 10_000 });
      peers = page.locator('[data-testid^="peer-"]');
    }
    expect(await peers.count()).toBeGreaterThanOrEqual(2);

    // Playwright locators are live. Capture the first two peer identities before clicking,
    // because opening/reading a conversation may reorder the list and retarget nth().
    const firstTestId = await peers.nth(0).getAttribute('data-testid');
    const secondTestId = await peers.nth(1).getAttribute('data-testid');
    if (!firstTestId || !secondTestId) throw new Error('expected stable test ids for the first two peers');
    const first = page.getByTestId(firstTestId);
    const second = page.getByTestId(secondTestId);

    // BotMark intentionally exposes the engine marker on both its semantic wrapper and inner SVG.
    // Count only the semantic outer mark carrying data-bot-id, so one avatar equals one identity.
    const visibleMark = '[data-engine="fabushi-motion-v3"][data-bot-id]:visible';
    const firstMark = first.locator(visibleMark);
    const secondMark = second.locator(visibleMark);
    await expect(firstMark).toHaveCount(1);
    await expect(secondMark).toHaveCount(1);
    const firstBotId = await firstMark.getAttribute('data-bot-id');
    const secondBotId = await secondMark.getAttribute('data-bot-id');
    if (!firstBotId || !secondBotId) throw new Error('expected semantic bot ids for the first two peers');

    const headerIdentity = page.getByTestId('conversation-status').locator('xpath=../..');
    const headerMark = headerIdentity.locator(visibleMark);

    await first.click();
    await expect(first).toHaveClass(/peerActive/);
    await expect(firstMark).toHaveAttribute('data-bot-id', firstBotId);
    await expect(headerMark).toHaveCount(1);
    await expect(headerMark).toHaveAttribute('data-bot-id', firstBotId);

    await second.click();
    await expect(second).toHaveClass(/peerActive/);
    await expect(secondMark).toHaveAttribute('data-bot-id', secondBotId);
    await expect(firstMark).toHaveAttribute('data-bot-id', firstBotId);
    await expect(headerMark).toHaveCount(1);
    await expect(headerMark).toHaveAttribute('data-bot-id', secondBotId);

    await page.setViewportSize({ width: 1100, height: 800 });
    const toggle = page.getByTestId('conversation-info-toggle');
    const infoPanel = page.getByTestId('messenger-info-panel');
    await expect(toggle).toBeVisible();
    await expect(toggle).toHaveAttribute('data-active', 'false');
    await expect(infoPanel).toHaveCount(0);

    // The panel DOM can disappear for a render while the responsive grid switches from
    // docked to overlay. Drive the action from React's explicit toggle state instead of
    // sampling panel visibility at that transition boundary and accidentally closing it.
    if (await toggle.getAttribute('data-active') !== 'true') {
      await toggle.click();
    }
    await expect(toggle).toHaveAttribute('data-active', 'true');
    await expect(infoPanel).toBeVisible();
    await expect(infoPanel).toHaveAttribute('data-overlay', 'true');
    const infoMark = infoPanel.locator(visibleMark);
    await expect(infoMark).toHaveCount(1);
    await expect(infoMark).toHaveAttribute('data-bot-id', secondBotId);

    await first.click();
    await expect(first).toHaveClass(/peerActive/);
    await expect(headerMark).toHaveCount(1);
    await expect(headerMark).toHaveAttribute('data-bot-id', firstBotId);
    await expect(infoMark).toHaveCount(1);
    await expect(infoMark).toHaveAttribute('data-bot-id', firstBotId);
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});