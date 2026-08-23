import {
  _electron as electron,
  expect,
  test,
  type ElectronApplication,
  type Page,
} from '@playwright/test';
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

async function clickApplicationMenuItem(app: ElectronApplication, label: string): Promise<void> {
  const clicked = await app.evaluate(({ Menu }, targetLabel) => {
    const root = Menu.getApplicationMenu();
    const queue = root ? [...root.items] : [];
    while (queue.length) {
      const item = queue.shift() as any;
      if (item.label === targetLabel) {
        if (typeof item.click !== 'function') return false;
        item.click();
        return true;
      }
      if (item.submenu?.items?.length) queue.push(...item.submenu.items);
    }
    return false;
  }, label);
  expect(clicked).toBe(true);
}

async function waitForNativeEventAfterMenu(
  app: ElectronApplication,
  page: Page,
  label: string,
  eventName: string,
): Promise<Record<string, unknown>> {
  const probeId = `native-menu-${eventName}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  await page.evaluate(({ wantedEvent, id }) => {
    const target = window as any;
    const probes = target.__fabushiNativeMenuProbes ??= {};
    probes[id] = { payload: null };
    let unsubscribe = () => {};
    unsubscribe = target.fabushiNative.subscribe({
      [wantedEvent]: (payload: Record<string, unknown>) => {
        probes[id].payload = payload ?? {};
        unsubscribe();
      },
    });
  }, { wantedEvent: eventName, id: probeId });
  await clickApplicationMenuItem(app, label);
  await expect.poll(async () => page.evaluate((id) => {
    return (window as any).__fabushiNativeMenuProbes?.[id]?.payload ?? null;
  }, probeId), { timeout: 5_000 }).not.toBeNull();
  const payload = await page.evaluate((id) => {
    const target = window as any;
    const value = target.__fabushiNativeMenuProbes?.[id]?.payload ?? {};
    if (target.__fabushiNativeMenuProbes) delete target.__fabushiNativeMenuProbes[id];
    return value;
  }, probeId) as Record<string, unknown>;
  return payload;
}

async function executeHostCommand(page: Page, type: string, extra: Record<string, unknown> = {}) {
  const requestId = `surface-${type}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  const accepted = await page.evaluate(async (command) => {
    return (window as any).mahayana.invoke('feature.execute', { command });
  }, { type, requestId, ...extra }) as { requestId?: string };
  expect(accepted.requestId).toBe(requestId);
}

const safeNativeReads = [
  'getDesktopEnvironment',
  'getWindowState',
  'getThemeState',
  'getHardwareAcceleration',
  'getWebauthnProxyEnabled',
  'getUpdateStatus',
  'getComputeMigrationStatus',
  'getOnboardingSeen',
  'getTimeZone',
  'getAutoReviewInstructions',
  'getLocalToolPermission',
  'getLocalToolPermissionCeiling',
  'getSidebarCollapsed',
  'getExperimentsSnapshot',
  'getAgentDefaultModel',
  'getComputerUseModel',
  'getHostPinnedAgents',
  'getHostSidebarSections',
  'getAvailableModels',
  'getOfflineAsrStatus',
  'getReviewPreferences',
  'getPrivacyModeEnabled',
  'getRuntimeAccess',
  'listSecrets',
  'requestDiskSaverAudit',
  'getMcpState',
  'getEffectivePlugins',
  'getMcpCatalog',
  'getMcpTeamPopularity',
  'getProductionComputeAttachmentStatus',
] as const;

test('installed desktop exposes unified Messenger, native menu routing, browser lifecycle, and safe native reads', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-surface-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await expect(page.getByTestId('onboarding-gate')).toBeVisible();

    await test.step('browser auth start, secure reopen, and cancel stay secret-free', async () => {
      const lifecycle = await page.evaluate(async () => {
        const bridge = (window as any).mahayana;
        const start = await bridge.invoke('feature.auth.browserStart');
        const attemptId = String(start.attemptId ?? '');
        const reopen = await bridge.invoke('feature.auth.browserReopen', { attemptId });
        const cancel = await bridge.invoke('feature.auth.browserCancel', { attemptId });
        return { start, reopen, cancel };
      });
      expect(String(lifecycle.start.attemptId ?? '')).not.toBe('');
      expect(lifecycle.start.pollSecret).toBeUndefined();
      expect(lifecycle.start.accessToken).toBeUndefined();
      expect(lifecycle.reopen.status).toBe('pending');
      expect(lifecycle.reopen.pollSecret).toBeUndefined();
      expect(lifecycle.cancel.status).toBe('cancelled');
    });

    await completeBrowserLogin(page);

    await test.step('Telegram-class navigation and Grok/Fabushi agent identity share one shell', async () => {
      await expect(page.getByTestId('messenger-workspace')).toBeVisible();
      await expect(page.locator('.desktop-mode-switch')).toHaveCount(0);
      await page.getByTestId('profile-navigation-trigger').click();
      await expect(page.getByTestId('profile-navigation-menu')).toBeVisible();
      for (const label of ['聊天', '联系人', 'Bots', '群组', '频道', '通话', '收藏', '归档', '文件夹', 'Mini Apps', '支付', '设置']) {
        await expect(page.getByTitle(label, { exact: true })).toBeVisible();
      }
      const assistant = page.getByTestId('peer-legacy:conversation:codex:agent:assistant');
      await expect(assistant).toBeVisible();
      await expect(assistant.locator('[data-engine="fabushi-motion-v2"]').first()).toBeVisible();
      await assistant.click();
      await expect(page.getByTestId('messenger-input')).toBeVisible();
    });

    await test.step('Mahayana Host capabilities stay callable after the Messenger takeover', async () => {
      const info = await page.evaluate(() => (window as any).mahayana.invoke('feature.info')) as {
        platform?: string;
        protocolVersion?: string;
        runtimeVersion?: string;
      };
      expect(info.platform).toBe('electron');
      expect(String(info.protocolVersion ?? '')).not.toBe('');
      expect(String(info.runtimeVersion ?? '')).not.toBe('');
      await executeHostCommand(page, 'conversation.list');
      await executeHostCommand(page, 'capability.list');
      await executeHostCommand(page, 'automation.list');
      await executeHostCommand(page, 'connector.list');
    });

    await test.step('native application menus route into the unified renderer event bus', async () => {
      const settings = await waitForNativeEventAfterMenu(app, page, '设置', 'deep-link');
      expect(settings.route).toBe('settings');
      expect(settings.section).toBe('general');

      const offlineAsr = await waitForNativeEventAfterMenu(app, page, 'Offline ASR', 'open-offline-asr');
      expect(offlineAsr.source).toBe('menu');

      const widgets = await waitForNativeEventAfterMenu(app, page, 'Widget Gallery', 'widget-gallery');
      expect(widgets.source).toBe('menu');

      const about = await waitForNativeEventAfterMenu(app, page, '关于', 'open-about');
      expect(about.source).toBe('menu');

      const feedback = await waitForNativeEventAfterMenu(app, page, '发送反馈', 'open-feedback');
      expect(feedback.source).toBe('menu');
    });

    await test.step('safe native desktop reads execute in the installed package', async () => {
      const results = await page.evaluate(async (methods) => {
        const bridge = (window as any).fabushiNative;
        const output: Record<string, { ok: boolean; value?: unknown; error?: string }> = {};
        for (const method of methods) {
          try {
            output[method] = { ok: true, value: await bridge.invoke(method, {}) };
          } catch (error) {
            output[method] = { ok: false, error: error instanceof Error ? error.message : String(error) };
          }
        }
        return output;
      }, [...safeNativeReads]);
      const failures = Object.entries(results).filter(([, result]) => !result.ok);
      expect(failures, JSON.stringify(failures, null, 2)).toEqual([]);
      const asr = results.getOfflineAsrStatus.value as { binaryPath?: string; available?: boolean };
      if (packagedExecutable) {
        const asrExecutable = process.platform === 'win32' ? 'whisper-cli.exe' : 'whisper-cli';
        const asrPath = String(asr.binaryPath ?? '').replaceAll('\\', '/');
        expect(asrPath).toContain(`/asr/${process.platform}-${process.arch}/${asrExecutable}`);
      } else {
        expect(results.getOfflineAsrStatus.ok).toBe(true);
      }
    });
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
