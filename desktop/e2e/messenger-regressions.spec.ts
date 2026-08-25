import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { ElectronMahayanaHostTransport } from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import type { RuntimeEvent } from '../../frontend/apps/web/src/lib/mahayana-host/contracts';

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
  await page.waitForLoadState('domcontentloaded');
  await expect.poll(async () => {
    for (const testId of ['onboarding-gate', 'login-gate', 'messenger-workspace']) {
      if (await page.getByTestId(testId).isVisible().catch(() => false)) return true;
    }
    return false;
  }, { timeout: 15_000 }).toBe(true);

  while (await page.getByTestId('onboarding-gate').isVisible().catch(() => false)) {
    await page.getByTestId('onboarding-next').click();
  }
  const loginGate = page.getByTestId('login-gate');
  if (await loginGate.isVisible().catch(() => false)) {
    await page.getByTestId('browser-login-start').click();
    await expect(loginGate).toBeHidden();
  }
  await expect(page.getByTestId('messenger-workspace')).toBeVisible({ timeout: 15_000 });
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

    for (let index = 0; index < count; index += 1) {
      const peer = peers.nth(index);
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

    const peers = page.locator('[data-testid^="peer-"]');
    await expect(peers.first()).toBeVisible();
    expect(await peers.count()).toBeGreaterThanOrEqual(2);

    const first = peers.nth(0);
    const second = peers.nth(1);
    // BotMark intentionally exposes the engine marker on both its semantic wrapper and inner SVG.
    // Count only the semantic outer mark carrying data-bot-id, so one avatar equals one identity.
    const visibleMark = '[data-engine="fabushi-motion-v2"][data-bot-id]:visible';

    await first.click();
    await expect(first.locator(visibleMark)).toHaveCount(1);

    await second.click();
    await expect(second.locator(visibleMark)).toHaveCount(1);
    await expect(first.locator(visibleMark)).toHaveCount(1);

    const headerIdentity = page.getByTestId('conversation-status').locator('xpath=../..');
    const headerMark = headerIdentity.locator(visibleMark);
    await expect(headerMark).toHaveCount(1);
    expect(await headerMark.getAttribute('data-bot-id')).toBe(await second.locator(visibleMark).getAttribute('data-bot-id'));

    await page.setViewportSize({ width: 1100, height: 800 });
    const toggle = page.getByTestId('conversation-info-toggle');
    const infoPanel = page.getByTestId('messenger-info-panel');
    await expect(toggle).toBeVisible();

    if (await infoPanel.isVisible().catch(() => false)) {
      await toggle.click();
      await expect(infoPanel).toBeHidden();
    }
    await toggle.click();
    await expect(infoPanel).toBeVisible();
    await expect(infoPanel).toHaveAttribute('data-overlay', 'true');
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
