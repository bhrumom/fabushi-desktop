import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

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

function rgbLuma(value: string): number {
  const components = value.match(/[\d.]+/g)?.slice(0, 3).map(Number) ?? [];
  if (components.length !== 3) return 255;
  return components[0] * 0.2126 + components[1] * 0.7152 + components[2] * 0.0722;
}

test('desktop uses the Fabushi-owned Grok parity surface without a parallel Messenger', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-grok-parity-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();

    await test.step('parity stylesheet and surface marker load before authentication', async () => {
      await expect(page.locator('body')).toHaveAttribute('data-fabushi-surface', 'grok-parity-v1');
      const parityLoaded = await page.evaluate(() => Array.from(document.styleSheets).some((sheet) =>
        String(sheet.href ?? '').includes('grok-parity.css')));
      expect(parityLoaded).toBe(true);
      const bodyBackground = await page.locator('body').evaluate((element) => getComputedStyle(element).backgroundColor);
      expect(rgbLuma(bodyBackground)).toBeLessThan(40);
    });

    await completeBrowserLogin(page);

    await test.step('canonical Messenger remains the only product shell', async () => {
      await expect(page.getByTestId('messenger-workspace')).toHaveCount(1);
      await expect(page.locator('.desktop-mode-switch')).toHaveCount(0);
      await expect(page.getByTestId('profile-navigation-trigger').locator('[data-engine="fabushi-motion-v3"]').first()).toBeVisible();
    });

    await test.step('conversation and composer expose dark low-contrast material', async () => {
      const peer = page.getByTestId('peer-legacy:conversation:mahayana-ai:agent:assistant');
      await expect(peer).toBeVisible();
      await peer.click();
      const input = page.getByTestId('messenger-input');
      await expect(input).toBeVisible();

      const material = await page.evaluate(() => {
        const inputElement = document.querySelector('[data-testid="messenger-input"]');
        const composer = inputElement?.parentElement;
        const peerElement = document.querySelector('[data-testid="peer-legacy:conversation:mahayana-ai:agent:assistant"]');
        if (!composer || !peerElement) return null;
        const composerStyle = getComputedStyle(composer);
        const peerStyle = getComputedStyle(peerElement);
        return {
          composerBackground: composerStyle.backgroundColor,
          composerRadius: composerStyle.borderRadius,
          peerBackground: peerStyle.backgroundColor,
          peerRadius: peerStyle.borderRadius,
        };
      });

      expect(material).not.toBeNull();
      expect(rgbLuma(material!.composerBackground)).toBeLessThan(70);
      expect(parseFloat(material!.composerRadius)).toBeGreaterThanOrEqual(14);
      expect(rgbLuma(material!.peerBackground)).toBeLessThan(80);
      expect(parseFloat(material!.peerRadius)).toBeGreaterThanOrEqual(10);
    });
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
