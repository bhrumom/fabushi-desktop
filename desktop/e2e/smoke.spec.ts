import { _electron as electron, expect, test } from '@playwright/test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

test('desktop boots with a sandboxed renderer and exposes only the approved bridge', async () => {
  const app = await electron.launch({
    args: [appRoot],
    env: {
      ...process.env,
      MAHAYANA_APP_HOST_BIN: process.env.MAHAYANA_APP_HOST_BIN || '',
    },
  });

  try {
    const page = await app.firstWindow();
    await expect(page.getByTestId('app-shell')).toBeVisible();
    await expect(page.getByTestId('runtime-badge')).toContainText('Electron');

    const security = await page.evaluate(() => ({
      nodeRequire: typeof (window as unknown as { require?: unknown }).require,
      processGlobal: typeof (window as unknown as { process?: unknown }).process,
      bridgeKeys: Object.keys(window.fabushi).sort(),
    }));
    expect(security.nodeRequire).toBe('undefined');
    expect(security.processGlobal).toBe('undefined');
    expect(security.bridgeKeys).toEqual(['invoke', 'notify', 'openExternal', 'pickFile']);
  } finally {
    await app.close();
  }
});
