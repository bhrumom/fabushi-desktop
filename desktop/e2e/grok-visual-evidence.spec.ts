import { _electron as electron, expect, test, type Page } from '@playwright/test';
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

async function attachScreenshot(page: Page, name: string): Promise<void> {
  // Visual evidence is intentionally deterministic. Motion behavior is verified
  // separately by grok-motion-parity.spec.ts; screenshots freeze CSS animations so
  // Xvfb/Chromium never captures an arbitrary transition frame.
  const screenshotPath = test.info().outputPath(`${name}.png`);
  await page.screenshot({
    path: screenshotPath,
    animations: 'disabled',
    fullPage: false,
  });
  await test.info().attach(name, {
    path: screenshotPath,
    contentType: 'image/png',
  });
}

test('capture Grok parity packaged visual evidence', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-grok-visual-'));
  const app = await launchDesktopApp(appDataDir);
  let page: Page | null = null;
  let videoPath: string | null = null;
  let screencastStarted = false;

  try {
    page = await app.firstWindow();
    await page.setViewportSize({ width: 1440, height: 900 });
    videoPath = test.info().outputPath('grok-parity-user-journey.webm');
    await page.screencast.start({ path: videoPath, size: { width: 1280, height: 800 } });
    screencastStarted = true;
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
    await expect(page.getByTestId('message-list').locator(':scope > article').getByText('收到：Grok parity visual evidence')).toBeVisible();
    await attachScreenshot(page, 'grok-parity-conversation-1440x900');
  } finally {
    if (page && screencastStarted) {
      await page.screencast.stop().catch(() => {});
      if (videoPath) {
        try {
          await access(videoPath);
          await test.info().attach('grok-parity-user-journey-video', {
            path: videoPath,
            contentType: 'video/webm',
          });
        } catch {
          // The missing artifact will be visible in the uploaded diagnostics and is a
          // test-evidence failure to investigate, not a reason to hide product failures.
        }
      }
    }
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
