import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import journeyContract from '../../contracts/automation/cross-platform-journeys.json' with { type: 'json' };
import type {
  MahayanaHostFeature,
  MahayanaHostJourneyStep,
} from '../../frontend/packages/shared/src/mahayana-host-features';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;
const mahayanaHostFeatures = journeyContract.features as ReadonlyArray<MahayanaHostFeature>;
const officialAppIds = [
  'global-dharma',
  'faliu-flashcards',
  'platform-publish',
  'hermes-installer',
  'bot-father',
  'chatgpt-auto-confirm',
] as const;

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
  await expect(page.getByTestId('login-gate')).toBeVisible();
  await page.getByTestId('browser-login-start').click();
  await expect(page.getByTestId('login-gate')).toBeHidden();
}

async function ensureComputerPanel(page: Page): Promise<void> {
  if (await page.getByTestId('feature-coverage').isVisible().catch(() => false)) return;
  await page.getByRole('button', { name: '大乘助手的电脑' }).click();
  await expect(page.getByTestId('feature-coverage')).toBeVisible();
}

async function runJourneyStep(page: Page, step: MahayanaHostJourneyStep): Promise<void> {
  switch (step.action) {
    case 'oauthLogin':
    case 'login':
      await completeBrowserLogin(page);
      return;
    case 'expectReady':
      await expect(page.getByTestId('host-status')).toHaveAttribute('data-state', 'ready');
      return;
    case 'sendChat':
      await page.getByTestId('chat-input').fill(step.text);
      await page.getByTestId('send-message').click();
      await expect(page.getByTestId('messages')).toContainText(step.expectedReply);
      return;
    case 'installMiniApp':
      await page.getByTestId('open-marketplace').click();
      await expect(page.getByRole('dialog', { name: '插件市场' })).toBeVisible();
      await page.getByTestId('install-miniapp').click();
      await expect(page.getByTestId('marketplace-state')).toHaveText('installed');
      return;
    case 'openMiniApp':
      await page.getByRole('button', { name: '关闭插件市场' }).click();
      await page.getByTestId(`agent-${step.miniAppId}`).click();
      await expect(page.getByTestId('miniapp-panel')).toContainText(step.miniAppId);
      await expect(page.getByTestId('miniapp-frame')).toBeVisible();
      return;
    case 'approveCapability':
      await page.getByTestId('request-capability').click();
      await expect(page.getByRole('dialog', { name: '能力审批' })).toContainText(step.capability);
      await page
        .getByTestId(step.decision === 'allow-once' ? 'approve-capability' : 'deny-capability')
        .click();
      await expect(page.getByTestId('approval-state')).toHaveText(
        step.decision === 'allow-once' ? 'allowed-once' : 'denied',
      );
      return;
    case 'interruptOperation':
      await page.getByTestId('start-long-operation').click();
      await expect(page.getByTestId('operation-state')).toHaveText('running');
      await page.getByTestId('interrupt-operation').click();
      await expect(page.getByTestId('operation-state')).toHaveText('interrupted');
      return;
    case 'clearSession':
      await page.getByTestId('clear-session').click();
      await expect(page.getByTestId('session-state')).toHaveText('cleared');
      return;
    default: {
      const unhandled: never = step;
      throw new Error(`Unhandled Host journey step: ${JSON.stringify(unhandled)}`);
    }
  }
}

async function runOfficialApps(page: Page): Promise<void> {
  for (const appId of officialAppIds) {
    await test.step(`official app: ${appId}`, async () => {
      await page.getByTestId('open-marketplace').click();
      const installId = appId === 'global-dharma' ? 'install-miniapp' : `install-${appId}`;
      const install = page.getByTestId(installId);
      if (await install.isEnabled()) await install.click();
      await expect(install).toBeDisabled();
      await page.getByRole('button', { name: '关闭插件市场' }).click();
      await expect(page.getByTestId(`agent-${appId}`)).toBeVisible();
      await page.getByTestId(`agent-${appId}`).click();
      await expect(page.getByTestId('miniapp-panel')).toContainText(appId);
      await expect(page.getByTestId('miniapp-frame')).toBeVisible();
    });
  }
}

test('desktop package drives every declared Host journey through Electron IPC and Rust', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-electron-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await expect(page.getByTestId('onboarding-gate')).toBeVisible();
    await expect(page.locator('.desktop-mode-switch')).toHaveCount(0);

    const security = await page.evaluate(() => ({
      nodeRequire: typeof (window as unknown as { require?: unknown }).require,
      processGlobal: typeof (window as unknown as { process?: unknown }).process,
      bridgeKeys: Object.keys(window.fabushi).sort(),
      mahayanaKeys: Object.keys(window.mahayana ?? {}).sort(),
    }));
    expect(security.nodeRequire).toBe('undefined');
    expect(security.processGlobal).toBe('undefined');
    expect(security.bridgeKeys).toEqual([
      'contractVersion',
      'notify',
      'openExternal',
      'openSystemSettings',
      'pickFile',
      'windowFocused',
    ]);
    expect(security.mahayanaKeys).toEqual(['contractVersion', 'invoke', 'subscribe']);

    const hostInfo = await page.evaluate(() => window.mahayana!.invoke<{
      platform: string;
      protocolVersion: string;
      runtimeVersion: string;
    }>('feature.info'));
    expect(hostInfo.platform).toBe('electron');
    expect(hostInfo.protocolVersion).not.toBe('');
    expect(hostInfo.runtimeVersion).toContain('test');

    const sessionClearFeature = mahayanaHostFeatures.find((feature) => feature.id === 'session.clear');
    expect(sessionClearFeature).toBeTruthy();

    for (const feature of mahayanaHostFeatures.filter((feature) => feature.id !== 'session.clear')) {
      await test.step(`${feature.id}: ${feature.label}`, async () => {
        for (const step of feature.steps) await runJourneyStep(page, step);
        await ensureComputerPanel(page);
        await expect(page.getByTestId(`feature-result-${feature.id}`)).toHaveAttribute(
          'data-state',
          'passed',
        );
      });
    }

    await runOfficialApps(page);

    await test.step('Fabushi modal keyboard focus returns to the account trigger', async () => {
      const accountTrigger = page.getByTestId('account-menu');
      await accountTrigger.focus();
      await accountTrigger.click();
      await expect(page.getByTestId('logout')).toBeVisible();
      await page.keyboard.press('Escape');
      await expect(page.getByTestId('logout')).toBeHidden();
      await expect(accountTrigger).toBeFocused();
    });

    await test.step(`${sessionClearFeature!.id}: ${sessionClearFeature!.label}`, async () => {
      for (const step of sessionClearFeature!.steps) await runJourneyStep(page, step);
      await ensureComputerPanel(page);
      await expect(page.getByTestId(`feature-result-${sessionClearFeature!.id}`)).toHaveAttribute(
        'data-state',
        'passed',
      );
    });

    await expect(page.getByTestId('host-status')).toHaveAttribute('data-state', 'ready');
    await expect(page.getByTestId('messages')).toBeVisible();
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
