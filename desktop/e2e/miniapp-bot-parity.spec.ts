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

async function navigate(page: Page, title: string): Promise<void> {
  await page.getByTestId('profile-navigation-trigger').click();
  await expect(page.getByTestId('profile-navigation-menu')).toBeVisible();
  await page.getByTitle(title, { exact: true }).click();
}

function globalDharmaBotPeer(page: Page) {
  return page.getByRole('button', { name: /@global_dharma_bot\b/ }).first();
}

async function waitForGlobalDharmaBot(page: Page) {
  await navigate(page, 'Bots');
  const botPeer = globalDharmaBotPeer(page);
  await expect(botPeer).toBeVisible({ timeout: 15_000 });
  await expect(botPeer).toContainText('全球法布施');
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

async function readWebMcpTools(page: Page): Promise<string[]> {
  const frame = page.frameLocator('iframe[title="global-dharma"]');
  return frame.locator('body').evaluate(() => {
    const registry = (window as any).__fabushiWebMcp;
    if (!registry?.list) throw new Error('Fabushi WebMCP registry is unavailable');
    return registry.list().map((tool: { name?: unknown }) => String(tool.name ?? '')).filter(Boolean);
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
    const contactBot = globalDharmaBotPeer(page);
    await expect(contactBot).toBeVisible({ timeout: 15_000 });
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
    await expect(page.getByText('Mini App · 已安装线上包 · 账号云同步')).toBeVisible();
    await expect(page.locator('iframe[title="global-dharma"]')).toBeVisible();
    await expect.poll(() => readWebMcpTools(page), { timeout: 15_000 }).toEqual(
      expect.arrayContaining(['status', 'start', 'stop', 'send']),
    );
    await writeCloudProbe(page, probeValue);
    await page.screenshot({ path: testInfo.outputPath('miniapp-webmcp-sync-before-restart.png'), fullPage: true });

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
    await expect.poll(() => readWebMcpTools(restartedPage), { timeout: 15_000 }).toContain('status');
    await expect.poll(() => readCloudProbe(restartedPage), { timeout: 15_000 }).toBe(probeValue);
    await restartedPage.screenshot({ path: testInfo.outputPath('miniapp-webmcp-sync-after-restart.png'), fullPage: true });
  } finally {
    await restartedApp?.close().catch(() => {});
    await firstApp.close().catch(() => {});
    await rm(appDataDir, { recursive: true, force: true });
  }
});
