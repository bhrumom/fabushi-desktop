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

async function navigate(page: Page, title: string): Promise<void> {
  await page.getByTestId('profile-navigation-trigger').click();
  await expect(page.getByTestId('profile-navigation-menu')).toBeVisible();
  await page.getByTitle(title, { exact: true }).click();
}

test('installed Mini App projects its Bot into Contacts and Bots and keeps Bot chat as the launch center', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-miniapp-bot-e2e-'));
  const app = await launchDesktopApp(appDataDir);
  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);

    await page.getByTestId('global-search-trigger').click();
    await page.getByTestId('global-search-tab-apps').click();
    await page.getByTestId('global-search-input').fill('全球法布施');
    const appResult = page.getByTestId('global-search-app-global-dharma');
    await expect(appResult).toBeVisible();
    const install = appResult.getByRole('button', { name: '安装' });
    if (await install.isVisible().catch(() => false)) await install.click();
    await expect(appResult.getByRole('button', { name: '打开' })).toBeVisible();

    await navigate(page, '联系人');
    const botPeer = page.getByTestId('peer-miniapp:bot:global-dharma');
    await expect(botPeer).toBeVisible();
    await expect(botPeer).toContainText('全球法布施');

    await navigate(page, 'Bots');
    await expect(botPeer).toBeVisible();
    await botPeer.click();
    await expect(page.getByTestId('miniapp-bot-open')).toBeVisible();

    const input = page.getByTestId('messenger-input');
    await input.fill('/');
    await expect(page.getByTestId('miniapp-bot-commands')).toBeVisible();
    await expect(page.getByTestId('miniapp-bot-commands')).toContainText('/status');

    await input.fill('/global-dharma:status {"detail":true}');
    await page.getByTestId('messenger-send').click();
    await expect(page.getByText('command', { exact: true })).toBeVisible();

    await input.fill('please show status now');
    await page.getByTestId('messenger-send').click();
    await expect(page.getByText('natural-language', { exact: true })).toBeVisible();

    await page.getByTestId('miniapp-bot-open').click();
    await expect(page.getByText('Mini App · 已安装线上包 · 受控宿主容器')).toBeVisible();
    await expect(page.locator('iframe[title="global-dharma"]')).toBeVisible();
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
