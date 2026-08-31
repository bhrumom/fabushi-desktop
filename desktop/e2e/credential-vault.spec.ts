import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;
const SECRET_REF = 'connector/credential-e2e/default';
const FIRST_CANARY = 'fabushi-credential-e2e-canary-first';
const ROTATED_CANARY = 'fabushi-credential-e2e-canary-rotated';
const CANCELLED_CANARY = 'fabushi-credential-e2e-cancelled';

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

async function listSecrets(page: Page): Promise<Array<Record<string, unknown>>> {
  return page.evaluate(async () => {
    const value = await (window as any).fabushiNative.invoke('listSecrets', {});
    return Array.isArray(value) ? value : [];
  });
}

test('installed Credential Vault keeps plaintext opaque, stays usable with OS encryption, and fails closed without it', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-credential-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);

    await page.getByTestId('credential-vault-button').click();
    await expect(page.getByRole('dialog', { name: '凭据保险库' })).toBeVisible();
    await expect(page.getByText('保存后不可读回')).toBeVisible();

    await page.getByTestId('credential-secret-value').fill(CANCELLED_CANARY);
    await page.getByRole('button', { name: '关闭' }).click();
    await expect(page.getByRole('dialog', { name: '凭据保险库' })).toBeHidden();
    await page.getByTestId('credential-vault-button').click();
    await expect(page.getByTestId('credential-secret-value')).toHaveValue('');

    await page.getByTestId('credential-secret-ref').fill(SECRET_REF);
    await page.getByPlaceholder('GitHub Production').fill('Credential E2E');
    await page.getByTestId('credential-secret-value').fill(FIRST_CANARY);
    await page.getByPlaceholder('https://api.github.com\nhttps://uploads.github.com').fill('https://api.example.com');
    await page.getByRole('button', { name: '保存凭据' }).click();

    const encryptionUnavailable = page.getByRole('alert').filter({ hasText: /OS-backed secret encryption is not available|Secure OS credential storage is unavailable/ });
    const savedRow = page.locator('article').filter({ hasText: SECRET_REF });
    const outcome = await expect.poll(async () => {
      if (await savedRow.isVisible().catch(() => false)) return 'saved';
      if (await encryptionUnavailable.isVisible().catch(() => false)) return 'fail-closed';
      return 'pending';
    }, { timeout: 15_000 }).not.toBe('pending').then(async () =>
      await savedRow.isVisible().catch(() => false) ? 'saved' : 'fail-closed');
    if (outcome === 'fail-closed') {
      await expect(encryptionUnavailable).toBeVisible();
      await expect(page.locator('body')).not.toContainText(FIRST_CANARY);
      await expect(page.locator('body')).not.toContainText(CANCELLED_CANARY);
      expect(await listSecrets(page)).toEqual([]);
      return;
    }

    await expect(savedRow).toBeVisible();
    await expect(savedRow).toContainText('https://api.example.com');
    await expect(page.locator('body')).not.toContainText(FIRST_CANARY);
    await expect(page.locator('body')).not.toContainText(CANCELLED_CANARY);
    const revealSecretControls = page.locator('button, [role="button"], a').filter({ hasText: /显示.*密钥|查看.*密钥|Reveal Secret/i });
    await expect(revealSecretControls).toHaveCount(0);

    const afterCreate = await listSecrets(page);
    const created = afterCreate.find((entry) => entry.name === SECRET_REF) as Record<string, any> | undefined;
    expect(created?.configured).toBe(true);
    expect(created?.revealable).toBe(false);
    expect(created?.binding?.allowedOrigins).toEqual(['https://api.example.com']);
    expect(JSON.stringify(afterCreate)).not.toContain(FIRST_CANARY);

    const revealResult = await page.evaluate(async (secretRef) => {
      try {
        await (window as any).fabushiNative.invoke('revealSecret', { name: secretRef });
        return { ok: true, error: '' };
      } catch (error) {
        return { ok: false, error: error instanceof Error ? error.message : String(error) };
      }
    }, SECRET_REF);
    expect(revealResult.ok).toBe(false);

    await savedRow.getByRole('button', { name: '轮换' }).click();
    await page.getByTestId('credential-secret-value').fill(ROTATED_CANARY);
    await page.getByRole('button', { name: '轮换并保存' }).click();
    await expect(page.locator('body')).not.toContainText(ROTATED_CANARY);
    const afterRotate = await listSecrets(page);
    expect(JSON.stringify(afterRotate)).not.toContain(ROTATED_CANARY);
    const rotated = afterRotate.find((entry) => entry.name === SECRET_REF) as Record<string, any> | undefined;
    expect(Number(rotated?.updatedAtMs ?? 0)).toBeGreaterThanOrEqual(Number(rotated?.createdAtMs ?? 0));

    page.once('dialog', async (dialog) => dialog.accept());
    await savedRow.getByRole('button', { name: `撤销 ${SECRET_REF}` }).click();
    await expect(page.locator('article').filter({ hasText: SECRET_REF })).toHaveCount(0);
    expect((await listSecrets(page)).some((entry) => entry.name === SECRET_REF)).toBe(false);
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
