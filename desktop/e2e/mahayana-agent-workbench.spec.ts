import { _electron as electron, expect, test, type ElectronApplication, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;

async function launchDesktopApp(appDataDir: string): Promise<ElectronApplication> {
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

async function openMahayanaConversation(page: Page): Promise<void> {
  const peer = page.getByTestId('peer-legacy:conversation:codex:agent:assistant');
  await expect(peer).toBeVisible({ timeout: 15_000 });
  await peer.click();
  await expect(page.getByTestId('messenger-input')).toBeVisible();
}

test('bot runs through Mahayana as a visible multi-step task and restores its run journal', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-mahayana-workbench-'));
  let app: ElectronApplication | null = null;

  try {
    app = await launchDesktopApp(appDataDir);
    let page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMahayanaConversation(page);

    await page.getByTestId('messenger-input').fill('请分析这个任务，规划步骤，调用工具并给出最终结果。');
    await page.getByTestId('messenger-send').click();

    const workbench = page.getByTestId('agent-workbench');
    await expect(workbench).toBeVisible({ timeout: 15_000 });
    const run = page.getByTestId('agent-run').last();
    await expect(run).toHaveAttribute('data-status', 'completed', { timeout: 15_000 });
    await expect.poll(async () => run.getByTestId('agent-step').count()).toBeGreaterThanOrEqual(3);
    await expect(run.getByTestId('agent-output')).toContainText('收到：');
    await expect(page.locator('#mahayana-agent-header-avatar [data-agent-state="result"]')).toBeVisible();

    const persistedRunId = await run.getAttribute('data-run-id');
    expect(persistedRunId).toBeTruthy();

    await app.close();
    app = null;

    app = await launchDesktopApp(appDataDir);
    page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMahayanaConversation(page);

    const restoredRun = page.locator(`[data-testid="agent-run"][data-run-id="${persistedRunId}"]`);
    await expect(restoredRun).toBeVisible({ timeout: 15_000 });
    await expect(restoredRun).toHaveAttribute('data-status', 'completed');
    await expect.poll(async () => restoredRun.getByTestId('agent-step').count()).toBeGreaterThanOrEqual(3);
    await expect(restoredRun.getByTestId('agent-output')).toContainText('收到：');
  } finally {
    await app?.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
  }
});
