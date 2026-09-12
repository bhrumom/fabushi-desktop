import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createAppAgentSurfaceClient } from '../../chatgpt-vps-control/lib/app-agent-surface-client.js';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;
const assistantAgentId = 'test:peer-legacy:conversation:mahayana-ai:agent:assistant';

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
  const loginGate = page.getByTestId('login-gate');
  const workspace = page.getByTestId('messenger-workspace');
  type LoginPhase = 'onboarding' | 'login' | 'ready' | 'waiting';
  const readPhase = async (): Promise<LoginPhase> => {
    try {
      return await page.evaluate(() => {
        if (document.querySelector('[data-testid="onboarding-gate"]')) return 'onboarding';
        if (document.querySelector('[data-testid="login-gate"]')) return 'login';
        const messenger = document.querySelector('[data-testid="messenger-workspace"]');
        return messenger?.getAttribute('data-initial-host-hydrated') === 'true' ? 'ready' : 'waiting';
      }) as LoginPhase;
    } catch {
      return 'waiting';
    }
  };
  for (let attempt = 0; attempt < 12; attempt += 1) {
    await expect.poll(readPhase, { timeout: 15_000 }).not.toBe('waiting');
    const phase = await readPhase();
    if (phase === 'onboarding') {
      await page.getByTestId('onboarding-next').click();
      continue;
    }
    if (phase === 'login') {
      await page.getByTestId('browser-login-start').click();
      await expect(loginGate).toBeHidden();
      continue;
    }
    if (phase === 'ready') break;
  }
  await expect(workspace).toHaveAttribute('data-initial-host-hydrated', 'true', { timeout: 15_000 });
  await expect(workspace).toBeVisible();
}

async function createChannel(page: Page, name: string): Promise<void> {
  await page.getByRole('button', { name: '新建', exact: true }).click();
  await page.getByRole('button', { name: '新建频道' }).click();
  await page.getByPlaceholder('频道名称').fill(name);
  await page.getByPlaceholder('频道简介').fill('FCM-010.13.11 semantic projection regression');
  await page.getByRole('button', { name: '创建频道' }).click();
  await expect(page.locator('[data-testid^="peer-selfhosted:channel:"]').filter({ hasText: name }).first()).toBeVisible();
}

async function findAssistant(client: ReturnType<typeof createAppAgentSurfaceClient>) {
  return client.call('find', { agentId: assistantAgentId, limit: 1 }) as Promise<{
    generation: number;
    count: number;
    matches: Array<{ agentId?: string }>;
  }>;
}

async function navigateToChats(page: Page): Promise<void> {
  const menu = page.getByTestId('profile-navigation-menu');
  if (!(await menu.isVisible())) {
    await page.getByTestId('profile-navigation-trigger').click();
  }
  await expect(menu).toBeVisible();
  await page.getByRole('button', { name: '聊天', exact: true }).click();
  await expect(page.getByRole('button', { name: '新建', exact: true })).toBeVisible();
}

test('FCM-010.13.11 real App Surface journey reprojects the canonical assistant after returning from two channel creations', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-fcm-010-13-assistant-projection-'));
  const policyDir = path.join(appDataDir, 'feature-host', 'runtime');
  await mkdir(policyDir, { recursive: true });
  await writeFile(path.join(policyDir, 'settings.json'), JSON.stringify({
    notifications: true,
    autoUpdateWhenIdle: true,
    localExecution: true,
    routeEgressLocally: false,
    securityKeys: false,
    webauthnProxyEnabled: false,
    localToolPermission: 'ask',
    remoteControlEnabled: false,
    aiComputerControlEnabled: true,
    autoReviewRules: [],
    inferenceProvider: 'fabushi',
    sandboxRuntime: 'host',
  }));

  const app = await launchDesktopApp(appDataDir);
  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    const client = createAppAgentSurfaceClient({
      discoveryPath: path.join(appDataDir, 'agent-surface', 'bridge.json'),
    });
    await expect.poll(async () => (await client.status()).available, { timeout: 15_000 }).toBe(true);

    const initial = await findAssistant(client);
    expect(initial.count).toBe(1);
    expect(initial.matches[0]?.agentId).toBe(assistantAgentId);

    await createChannel(page, 'FCM semantic projection A');
    await createChannel(page, 'FCM semantic projection B');

    const hiddenOnChannelSurface = await findAssistant(client);
    expect(hiddenOnChannelSurface.count).toBe(0);

    await navigateToChats(page);
    const afterChannels = await findAssistant(client);
    expect(afterChannels.count).toBe(1);
    expect(afterChannels.matches[0]?.agentId).toBe(assistantAgentId);

    const opened = await client.call('action', {
      generation: afterChannels.generation,
      agentId: assistantAgentId,
      action: 'invoke',
    }) as { status?: string; target?: { agentId?: string } };
    expect(opened).toMatchObject({ status: 'completed', target: { agentId: assistantAgentId } });
    await expect(page.getByTestId('peer-legacy:conversation:mahayana-ai:agent:assistant')).toBeVisible();
    await expect(page.getByTestId('messenger-input')).toBeVisible();
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
