import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;
const syncProbeKey = 'm2_sync_packaged_probe';

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

async function waitForGlobalDharmaBot(page: Page) {
  await navigate(page, 'Bots');
  const botPeer = page.getByTestId('peer-miniapp:bot:global-dharma');
  await expect(botPeer).toBeVisible({ timeout: 15_000 });
  return botPeer;
}

async function writeCloudProbe(page: Page, value: string): Promise<void> {
  const frame = page.frameLocator('iframe[title="global-dharma"]');
  const stored = await frame.locator('body').evaluate(async (_body, probe) => {
    const api = (window as any).FabushiMiniApp?.CloudStorage;
    if (!api) throw new Error('FabushiMiniApp.CloudStorage is unavailable');
    await api.setItem('m2_sync_packaged_probe', probe);
    return api.getItem('m2_sync_packaged_probe');
  }, value);
  expect(stored).toBe(value);
}

async function readCloudProbe(page: Page): Promise<string | null> {
  const frame = page.frameLocator('iframe[title="global-dharma"]');
  return frame.locator('body').evaluate(async () => {
    const api = (window as any).FabushiMiniApp?.CloudStorage;
    if (!api) throw new Error('FabushiMiniApp.CloudStorage is unavailable');
    return api.getItem('m2_sync_packaged_probe');
  });
}

test('installed Mini App projects its Bot into Contacts/Bots and recovers Bot history plus CloudStorage', async ({}, testInfo) => {
  test.setTimeout(90_000);
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-miniapp-bot-e2e-'));
  const probeValue = `packaged-sync-${Date.now()}`;
  const firstApp = await launchDesktopApp(appDataDir);
  let restartedApp: Awaited<ReturnType<typeof launchDesktopApp>> | null = null;
  try {
    const page = await firstApp.firstWindow();
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
    const contactBot = page.getByTestId('peer-miniapp:bot:global-dharma');
    await expect(contactBot).toBeVisible();
    await expect(contactBot).toContainText('全球法布施');

    const botPeer = await waitForGlobalDharmaBot(page);
    await botPeer.click();
    await expect(page.getByTestId('miniapp-bot-open')).toBeVisible();

    const input = page.getByTestId('messenger-input');
    await input.fill('/');
    await expect(page.getByTestId('miniapp-bot-commands')).toBeVisible();
    await expect(page.getByTestId('miniapp-bot-commands')).toContainText('/status');

    const commandText = '/global-dharma:status {"detail":true}';
    const naturalText = 'please show status now';
    await input.fill(commandText);
    await page.getByTestId('messenger-send').click();
    await expect(page.getByText('command', { exact: true })).toBeVisible();

    await input.fill(naturalText);
    await page.getByTestId('messenger-send').click();
    await expect(page.getByText('natural-language', { exact: true })).toBeVisible();

    await page.getByTestId('miniapp-bot-open').click();
    await expect(page.getByText('Mini App · 已安装线上包 · 受控宿主容器')).toBeVisible();
    await expect(page.locator('iframe[title="global-dharma"]')).toBeVisible();
    await writeCloudProbe(page, probeValue);
    await page.screenshot({ path: testInfo.outputPath('miniapp-sync-before-restart.png'), fullPage: true });

    await firstApp.close();
    restartedApp = await launchDesktopApp(appDataDir);
    const restartedPage = await restartedApp.firstWindow();
    await completeBrowserLogin(restartedPage);

    const recoveredBot = await waitForGlobalDharmaBot(restartedPage);
    await recoveredBot.click();
    await expect(restartedPage.getByText(commandText, { exact: true })).toBeVisible({ timeout: 15_000 });
    await expect(restartedPage.getByText(naturalText, { exact: true })).toBeVisible();

    await restartedPage.getByTestId('miniapp-bot-open').click();
    await expect(restartedPage.locator('iframe[title="global-dharma"]')).toBeVisible();
    await expect.poll(() => readCloudProbe(restartedPage), { timeout: 15_000 }).toBe(probeValue);
    await restartedPage.screenshot({ path: testInfo.outputPath('miniapp-sync-after-restart.png'), fullPage: true });
  } finally {
    await restartedApp?.close().catch(() => {});
    await firstApp.close().catch(() => {});
    await rm(appDataDir, { recursive: true, force: true });
  }
});
