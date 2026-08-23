import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import journeyContract from '../../contracts/automation/cross-platform-journeys.json' with { type: 'json' };
import type { MahayanaHostFeature, MahayanaHostJourneyStep } from '../../frontend/packages/shared/src/mahayana-host-features';

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

function requestId(prefix: string): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
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
  await expect(page.getByTitle('聊天', { exact: true })).toBeVisible();
}

async function executeFeature(page: Page, command: Record<string, unknown>) {
  return page.evaluate(async (input) => {
    const bridge = (window as any).mahayana;
    return bridge.invoke('feature.execute', { command: input });
  }, command);
}

async function executeFeatureAndWaitForEvent(
  page: Page,
  command: Record<string, unknown>,
  eventType: string,
): Promise<Record<string, unknown>> {
  return page.evaluate(async ({ input, wantedType }) => {
    const bridge = (window as any).mahayana;
    return new Promise<Record<string, unknown>>((resolve, reject) => {
      let unsubscribe = () => {};
      const timer = window.setTimeout(() => {
        unsubscribe();
        reject(new Error(`Timed out waiting for ${wantedType}`));
      }, 10_000);
      unsubscribe = bridge.subscribe((event: Record<string, unknown>) => {
        if (event?.type !== wantedType) return;
        window.clearTimeout(timer);
        unsubscribe();
        resolve(event);
      });
      bridge.invoke('feature.execute', { command: input }).catch((error: unknown) => {
        window.clearTimeout(timer);
        unsubscribe();
        reject(error);
      });
    });
  }, { input: command, wantedType: eventType });
}

async function runJourneyStep(page: Page, step: MahayanaHostJourneyStep): Promise<void> {
  switch (step.action) {
    case 'oauthLogin':
    case 'login':
      await completeBrowserLogin(page);
      return;
    case 'expectReady': {
      const info = await page.evaluate(() => (window as any).mahayana.invoke('feature.info')) as {
        protocolVersion?: string;
        runtimeVersion?: string;
      };
      expect(String(info.protocolVersion ?? '')).not.toBe('');
      expect(String(info.runtimeVersion ?? '')).not.toBe('');
      await expect(page.getByTestId('messenger-workspace')).toBeVisible();
      return;
    }
    case 'sendChat': {
      const assistant = page.getByTestId('peer-legacy:conversation:codex:agent:assistant');
      await expect(assistant).toBeVisible();
      await assistant.click();
      const input = page.getByTestId('messenger-input');
      await input.fill(step.text);
      await page.getByTestId('messenger-send').click();
      await expect(page.getByText(step.expectedReply, { exact: true })).toBeVisible();
      return;
    }
    case 'installMiniApp':
      await executeFeature(page, {
        type: 'marketplace.install',
        requestId: requestId('marketplace-install'),
        miniAppId: step.miniAppId,
      });
      return;
    case 'openMiniApp':
      await page.getByTitle('Mini Apps', { exact: true }).click();
      if (step.miniAppId === 'global-dharma') {
        await page.getByRole('button', { name: /全球法布施/ }).last().click();
        await expect(page.getByText('Mini App · 受控宿主容器')).toBeVisible();
        await expect(page.locator('iframe[title="global-dharma"]')).toBeVisible();
      } else {
        await executeFeatureAndWaitForEvent(page, {
          type: 'miniapp.open',
          requestId: requestId('miniapp-open'),
          miniAppId: step.miniAppId,
        }, 'miniapp.opened');
      }
      return;
    case 'approveCapability': {
      const event = await executeFeatureAndWaitForEvent(page, {
        type: 'capability.request',
        requestId: requestId('capability'),
        miniAppId: step.miniAppId,
        capability: step.capability,
        reason: 'Electron unified Messenger quality gate',
      }, 'approval.requested');
      const approvalId = String(event.approvalId ?? '');
      expect(approvalId).not.toBe('');
      await page.evaluate(async ({ id, decision }) => {
        await (window as any).mahayana.invoke('feature.approval.resolve', {
          resolution: { approvalId: id, decision },
        });
      }, { id: approvalId, decision: step.decision });
      return;
    }
    case 'interruptOperation': {
      const accepted = await executeFeature(page, {
        type: 'runtime.longTask',
        requestId: requestId('long-task'),
        label: step.label,
      }) as { operationId?: string };
      expect(String(accepted.operationId ?? '')).not.toBe('');
      await page.evaluate(async (operationId) => {
        await (window as any).mahayana.invoke('feature.interrupt', { operationId });
      }, accepted.operationId);
      return;
    }
    case 'clearSession':
      await executeFeature(page, { type: 'session.clear', requestId: requestId('session-clear') });
      return;
    default:
      throw new Error(`Unhandled Host journey step: ${JSON.stringify(step)}`);
  }
}

async function runOfficialApps(page: Page): Promise<void> {
  for (const miniAppId of officialAppIds) {
    await test.step(`official app runtime: ${miniAppId}`, async () => {
      await executeFeature(page, {
        type: 'marketplace.install',
        requestId: requestId(`install-${miniAppId}`),
        miniAppId,
      });
      const event = await executeFeatureAndWaitForEvent(page, {
        type: 'miniapp.open',
        requestId: requestId(`open-${miniAppId}`),
        miniAppId,
      }, 'miniapp.opened');
      expect(event.miniAppId).toBe(miniAppId);
      expect(typeof event.html).toBe('string');
    });
  }
}

test('desktop package drives every declared Host journey through the unified Messenger, Electron IPC, and Rust', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-electron-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await expect(page.getByTestId('onboarding-gate')).toBeVisible();
    await expect(page.locator('.desktop-mode-switch')).toHaveCount(0);

    const security = await page.evaluate(() => ({
      nodeRequire: typeof (window as any).require,
      processGlobal: typeof (window as any).process,
      bridgeKeys: Object.keys((window as any).fabushi).sort(),
      mahayanaKeys: Object.keys((window as any).mahayana ?? {}).sort(),
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

    const hostInfo = await page.evaluate(() => (window as any).mahayana.invoke('feature.info')) as {
      platform: string;
      protocolVersion: string;
      runtimeVersion: string;
    };
    expect(hostInfo.platform).toBe('electron');
    expect(hostInfo.protocolVersion).not.toBe('');
    expect(hostInfo.runtimeVersion).toContain('test');

    const sessionClear = mahayanaHostFeatures.find((feature) => feature.id === 'session.clear');
    expect(sessionClear).toBeTruthy();

    for (const feature of mahayanaHostFeatures.filter((feature) => feature.id !== 'session.clear')) {
      await test.step(`${feature.id}: ${feature.label}`, async () => {
        for (const step of feature.steps) await runJourneyStep(page, step);
      });
    }

    await runOfficialApps(page);

    await test.step(`${sessionClear!.id}: ${sessionClear!.label}`, async () => {
      for (const step of sessionClear!.steps) await runJourneyStep(page, step);
    });

    await expect(page.getByTestId('messenger-workspace')).toBeVisible();
    await expect(page.getByTitle('聊天', { exact: true })).toBeVisible();
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
