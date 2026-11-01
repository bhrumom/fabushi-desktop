import { _electron as electron, expect, test } from '@playwright/test';
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

test('desktop avatar cutover stays low-power and contains no legacy motion runtime', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-avatar-cutover-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await expect(page.getByTestId('desktop-shell')).toBeVisible({ timeout: 20_000 });
    const avatar = page.locator('[data-fab-avatar="true"]').first();
    await expect(avatar).toBeVisible();
    await expect(avatar).toHaveAttribute('data-state', /^(idle|thinking|working|waiting|speaking|success|error|offline)$/);

    const contract = await page.evaluate(() => ({
      legacyStylesheetLoaded: Array.from(document.styleSheets).some((sheet) =>
        String(sheet.href ?? '').includes('grok-motion-parity.css')),
      legacyAvatarNodes: document.querySelectorAll(
        '[data-fabushi-avatar-runtime], [data-engine="fabushi-motion-v3"], [data-renderer="fabushi-owned-svg-runtime"]',
      ).length,
      fabAvatars: document.querySelectorAll('[data-fab-avatar="true"]').length,
    }));

    expect(contract.legacyStylesheetLoaded).toBe(false);
    expect(contract.legacyAvatarNodes).toBe(0);
    expect(contract.fabAvatars).toBeGreaterThan(0);
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
