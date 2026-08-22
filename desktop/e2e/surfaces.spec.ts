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
  await expect(page.getByTestId('login-gate')).toBeVisible();
  await page.getByTestId('browser-login-start').click();
  await expect(page.getByTestId('login-gate')).toBeHidden();
  await expect(page.getByTestId('host-status')).toHaveAttribute('data-state', 'ready');
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

test('installed desktop exposes product surfaces, browser lifecycle, and safe native reads', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-surface-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await expect(page.getByTestId('onboarding-gate')).toBeVisible();

    await test.step('browser auth start, secure reopen, and cancel stay secret-free', async () => {
      const lifecycle = await page.evaluate(async () => {
        const start = await window.mahayana!.invoke<Record<string, unknown>>('feature.auth.browserStart');
        const attemptId = String(start.attemptId ?? '');
        const reopen = await window.mahayana!.invoke<Record<string, unknown>>('feature.auth.browserReopen', { attemptId });
        const cancel = await window.mahayana!.invoke<Record<string, unknown>>('feature.auth.browserCancel', { attemptId });
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

    await test.step('composer exposes agent mode and model routing', async () => {
      const mode = page.getByTestId('agent-mode');
      await mode.selectOption('plan');
      await expect(mode).toHaveValue('plan');
      await mode.selectOption('ask');
      await expect(mode).toHaveValue('ask');
      await mode.selectOption('agent');
      await expect(mode).toHaveValue('agent');
      expect(await page.getByTestId('agent-model').locator('option').count()).toBeGreaterThan(0);
    });

    await test.step('notification and error tray is reachable', async () => {
      const trigger = page.getByRole('button', { name: '通知与错误' });
      await trigger.click();
      await expect(page.locator('section[aria-label="通知与错误"]')).toBeVisible();
      await trigger.click();
      await expect(page.locator('section[aria-label="通知与错误"]')).toBeHidden();
    });

    await test.step('Agent Network, Shared Rooms, and Workspace are reachable', async () => {
      await page.getByRole('button', { name: '智能体网络' }).click();
      const dialog = page.getByRole('dialog', { name: '智能体网络' });
      await expect(dialog).toBeVisible();
      await dialog.getByRole('button', { name: 'Shared Rooms' }).click();
      await expect(dialog.locator('section[aria-label="共享房间"]')).toBeVisible();
      await dialog.getByRole('button', { name: 'Workspace' }).click();
      await expect(dialog.locator('section[aria-label="Agent workspace"]')).toBeVisible();
      await expect(dialog).toContainText('桌面存储审计');
      await dialog.getByRole('button', { name: 'Agent Network' }).click();
      await page.getByRole('button', { name: '关闭智能体网络' }).click();
      await expect(dialog).toBeHidden();
    });

    await test.step('automation can be created and deleted through Fabushi confirmation', async () => {
      await page.getByRole('button', { name: /自动化例程/ }).click();
      const dialog = page.getByRole('dialog', { name: '自动化例程' });
      await expect(dialog).toBeVisible();
      await dialog.getByLabel('名称').fill('自动化验证');
      await dialog.getByLabel('执行指令').fill('验证已安装桌面的自动化执行路径');
      await dialog.getByRole('button', { name: '创建例程' }).click();
      await expect(dialog).toContainText('自动化验证');
      const row = dialog.locator('article').filter({ hasText: '自动化验证' }).first();
      await row.getByRole('button', { name: '删除' }).click();
      const confirm = page.getByTestId('confirm-dialog');
      await expect(confirm).toContainText('删除例程「自动化验证」？');
      await confirm.getByRole('button', { name: '删除例程' }).click();
      await expect(confirm).toBeHidden();
      await expect(dialog).not.toContainText('自动化验证');
      await page.getByRole('button', { name: '关闭自动化' }).click();
    });

    await test.step('Computer panel and agent settings are reachable', async () => {
      await page.getByRole('button', { name: '大乘助手的电脑' }).click();
      await expect(page.getByTestId('feature-coverage')).toBeVisible();
      await page.getByRole('button', { name: '智能体设置' }).click();
      await expect(page.getByRole('button', { name: '← 返回电脑与例程' })).toBeVisible();
      await page.getByRole('button', { name: '← 返回电脑与例程' }).click();
      await page.getByRole('button', { name: '关闭电脑面板' }).click();
    });

    await test.step('application menu opens Settings and every settings section', async () => {
      await clickApplicationMenuItem(app, '设置');
      await expect(page.getByRole('dialog', { name: '通用设置' })).toBeVisible();
      await page.getByRole('button', { name: /^MCP/ }).click();
      await expect(page.getByRole('dialog', { name: 'MCP 与 Apps' })).toBeVisible();
      await page.getByRole('button', { name: /用量与计费/ }).click();
      await expect(page.getByRole('dialog', { name: '用量与计费' })).toBeVisible();
      await page.getByRole('button', { name: /更新/ }).click();
      await expect(page.getByRole('dialog', { name: '全球法布施更新' })).toBeVisible();
      await page.getByRole('button', { name: '关闭设置' }).click();
    });

    await test.step('packaged Offline ASR is discoverable through the real native menu event', async () => {
      await clickApplicationMenuItem(app, 'Offline ASR');
      const dialog = page.getByRole('dialog', { name: '离线语音转写' });
      await expect(dialog).toBeVisible();
      await expect(dialog).toContainText('本地引擎');
      if (packagedExecutable) {
        await expect(dialog).toContainText('已就绪');
      } else {
        await expect(dialog).toContainText('可用状态');
      }
      await page.getByRole('button', { name: '关闭离线语音转写' }).click();
    });

    await test.step('Widget Gallery, About, and Feedback use native menu events', async () => {
      await clickApplicationMenuItem(app, 'Widget Gallery');
      await expect(page.getByRole('dialog', { name: 'Widget Gallery' })).toBeVisible();
      await expect(page.getByRole('dialog', { name: 'Widget Gallery' })).toContainText('thinking');
      await page.getByRole('button', { name: '关闭组件画廊' }).click();

      await clickApplicationMenuItem(app, '关于');
      const about = page.getByRole('dialog', { name: '关于 Fabushi' });
      await expect(about).toBeVisible();
      await expect(about).toContainText('Mahayana Feature Host');
      await page.getByRole('button', { name: '关闭关于' }).click();

      await clickApplicationMenuItem(app, '发送反馈');
      const feedback = page.getByRole('dialog', { name: '发送反馈' });
      await expect(feedback).toBeVisible();
      await feedback.getByPlaceholder('告诉我们哪里可以更快、更稳或更好用…').fill('Fabushi installed desktop automated validation');
      await feedback.getByRole('button', { name: '提交反馈' }).click();
      await expect(feedback).toContainText('已保存反馈');
      await page.getByRole('button', { name: '关闭反馈' }).click();
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
