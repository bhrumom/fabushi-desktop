import { _electron as electron, expect, test, type Page, type TestInfo } from '@playwright/test';
import { access, mkdtemp, rm } from 'node:fs/promises';
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
  const loginGate = page.getByTestId('login-gate');
  const workspace = page.getByTestId('messenger-workspace');
  type LoginPhase = 'onboarding' | 'login' | 'ready' | 'waiting';
  const readPhase = async (): Promise<LoginPhase> => {
    try {
      return await page.evaluate(() => {
        if (document.querySelector('[data-testid="onboarding-gate"]')) return 'onboarding';
        if (document.querySelector('[data-testid="login-gate"]')) return 'login';
        const messenger = document.querySelector('[data-testid="messenger-workspace"]');
        return messenger?.getAttribute('data-initial-host-hydrated') === 'true' ? 'ready' : 'waiting';
      }) as LoginPhase;
    } catch {
      return 'waiting';
    }
  };

  for (let attempt = 0; attempt < 12; attempt += 1) {
    await expect.poll(readPhase, { timeout: 15_000 }).not.toBe('waiting');
    const phase = await readPhase();
    if (phase === 'onboarding') {
      await page.getByTestId('onboarding-next').click();
      continue;
    }
    if (phase === 'login') {
      const start = page.getByTestId('browser-login-start');
      await expect(start).toBeEnabled({ timeout: 15_000 });
      await start.click();
      await expect(loginGate).toBeHidden({ timeout: 15_000 });
      continue;
    }
    if (phase === 'ready') break;
  }
  await expect(workspace).toHaveAttribute('data-initial-host-hydrated', 'true', { timeout: 15_000 });
}

async function shot(page: Page, testInfo: TestInfo, name: string) {
  await page.screenshot({ path: testInfo.outputPath(name), fullPage: true });
}

test('searching 小程序 exposes and installs the official 全球法布施 Mini App entry', async ({}, testInfo) => {
  test.setTimeout(60_000);
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-miniapp-search-entry-'));
  const app = await launchDesktopApp(appDataDir);
  let page: Page | null = null;
  let videoPath: string | null = null;
  let recording = false;
  try {
    page = await app.firstWindow();
    videoPath = testInfo.outputPath('miniapp-search-entry-user-journey.webm');
    await page.screencast.start({ path: videoPath, size: { width: 1280, height: 800 } });
    recording = true;
    await completeBrowserLogin(page);

    await page.getByTestId('global-search-trigger').click();
    await page.getByTestId('global-search-tab-apps').click();
    await page.getByTestId('global-search-input').fill('小程序');
    const appResult = page.getByTestId('global-search-app-global-dharma');
    await expect(appResult).toBeVisible({ timeout: 15_000 });
    await expect(appResult).toContainText('全球法布施');
    await shot(page, testInfo, '01-search-miniapp-finds-global-dharma.png');

    const install = appResult.getByRole('button', { name: '安装' });
    const open = appResult.getByRole('button', { name: '打开' });
    if (await install.isVisible().catch(() => false)) {
      await expect(install).toBeEnabled();
      await install.click();
    }
    await expect(open).toBeVisible({ timeout: 15_000 });
    await shot(page, testInfo, '02-global-dharma-installed-from-miniapp-search.png');

    await page.screencast.stop();
    recording = false;
    if (!videoPath) throw new Error('Mini App search-entry video path was not initialized.');
    await access(videoPath);
    await testInfo.attach('miniapp-search-entry-user-journey-video', { path: videoPath, contentType: 'video/webm' });
  } finally {
    if (page && recording) await page.screencast.stop().catch(() => {});
    if (videoPath) {
      try {
        await access(videoPath);
        await testInfo.attach('miniapp-search-entry-user-journey-video-partial', { path: videoPath, contentType: 'video/webm' });
      } catch {}
    }
    await app.close().catch(() => {});
    await rm(appDataDir, { recursive: true, force: true });
  }
});
