import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;

async function launchDesktopApp(appDataDir: string) {
  return electron.launch({
    ...(packagedExecutable ? { executablePath: packagedExecutable, args: [] } : { args: [appRoot] }),
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

async function attachScreenshot(page: Page, name: string): Promise<void> {
  // Visual evidence is intentionally deterministic. Motion behavior is verified
  // separately by grok-motion-parity.spec.ts; screenshots freeze CSS animations so
  // Xvfb/Chromium never captures an arbitrary transition frame or waits forever on
  // continuously animated BotMark aura layers.
  await test.info().attach(name, {
    body: await page.screenshot({ animations: 'disabled', fullPage: false }),
    contentType: 'image/png',
  });
}

test('capture Grok parity packaged visual evidence', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-grok-visual-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await page.setViewportSize({ width: 1440, height: 900 });
    await completeBrowserLogin(page);

    await expect(page.locator('body')).toHaveAttribute('data-fabushi-surface', 'grok-parity-v1');
    await attachScreenshot(page, 'grok-parity-shell-1440x900');

    await page.getByTestId('global-search-trigger').click();
    await expect(page.getByTestId('global-search-surface')).toBeVisible();
    await attachScreenshot(page, 'grok-parity-global-search-1440x900');
    await page.getByRole('button', { name: '关闭搜索' }).click();

    await page.getByTestId('profile-navigation-trigger').click();
    await page.getByTitle('设置', { exact: true }).click();
    await page.getByTestId('settings-category-router').click();
    await expect(page.getByTestId('router-provider-settings')).toBeVisible();
    await attachScreenshot(page, 'grok-parity-router-settings-1440x900');
    await page.getByTestId('settings-close').click();
    await expect(page.getByTestId('settings-modal-backdrop')).toHaveCount(0);
    await page.getByTestId('profile-navigation-trigger').click();
    await page.getByTitle('聊天', { exact: true }).click();

    const assistant = page.getByTestId('peer-legacy:conversation:codex:agent:assistant');
    await expect(assistant).toBeVisible();
    await assistant.click();
    await expect(page.getByTestId('messenger-input')).toBeVisible();
    await page.getByTestId('messenger-input').fill('Grok parity visual evidence');
    await page.getByTestId('messenger-send').click();
    await expect(page.getByText('收到：Grok parity visual evidence')).toBeVisible();
    await attachScreenshot(page, 'grok-parity-conversation-1440x900');
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
