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
      await expect(page.getByTestId('profile-navigation-trigger').locator('[data-engine="fabushi-motion-v2"]').first()).toBeVisible();
    });

    await test.step('conversation and composer expose dark low-contrast material', async () => {
      const peer = page.getByTestId('peer-legacy:conversation:codex:agent:assistant');
      await expect(peer).toBeVisible();
      await peer.click();
      const input = page.getByTestId('messenger-input');
      await expect(input).toBeVisible();

      const material = await page.evaluate(() => {
        const inputElement = document.querySelector('[data-testid="messenger-input"]');
        const composer = inputElement?.parentElement;
        const peerElement = document.querySelector('[data-testid="peer-legacy:conversation:codex:agent:assistant"]');
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
