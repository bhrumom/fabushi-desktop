import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createAppAgentSurfaceClient } from '../../chatgpt-vps-control/lib/app-agent-surface-client.js';

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


test('packaged Fabushi publishes a generation-safe semantic App MCP over the private loopback bridge', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-app-agent-e2e-'));
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
    await expect(page.getByTestId('profile-navigation-trigger')).toBeVisible();

    const client = createAppAgentSurfaceClient({
      discoveryPath: path.join(appDataDir, 'agent-surface', 'bridge.json'),
    });
    await expect.poll(async () => (await client.status()).available, { timeout: 15_000 }).toBe(true);

    const status = await client.status();
    expect(status.appId).toBe('fabushi.desktop');
    expect(status.platform).toBe('electron');
    expect(status.generation).toBeGreaterThan(0);

    const snapshot = await client.call('snapshot', { maxElements: 500, includeText: true }) as {
      generation: number;
      elements: Array<{ agentId?: string; sensitive: boolean; value?: unknown }>;
    };
    const trigger = snapshot.elements.find((element) => element.agentId === 'test:profile-navigation-trigger');
    expect(trigger).toBeTruthy();
    expect(snapshot.elements.some((element) => element.sensitive && element.value !== undefined)).toBe(false);

    await client.call('action', {
      generation: snapshot.generation,
      agentId: 'test:profile-navigation-trigger',
      action: 'invoke',
    });
    await expect(page.getByTestId('profile-navigation-menu')).toBeVisible();

    await expect.poll(async () => {
      const assertion = await client.call('assert', {
        agentId: 'test:profile-navigation-menu',
        state: 'visible',
      }) as { passed: boolean };
      return assertion.passed;
    }).toBe(true);

    const rebasedAction = await client.call('action', {
      generation: snapshot.generation,
      agentId: 'test:profile-navigation-trigger',
      action: 'invoke',
    }) as { status?: string; target?: { agentId?: string } };
    expect(rebasedAction).toMatchObject({
      status: 'completed',
      target: { agentId: 'test:profile-navigation-trigger' },
    });
    await expect(page.getByTestId('profile-navigation-menu')).toBeHidden();

    await expect(client.call('action', {
      generation: snapshot.generation,
      agentId: 'test:profile-navigation-trigger',
      ref: 'g0:volatile',
      action: 'invoke',
    })).rejects.toThrow(/stale_app_surface_generation/u);

    const settingsTriggerSnapshot = await client.call('snapshot', { maxElements: 500, includeText: true }) as {
      generation: number;
    };
    await client.call('action', {
      generation: settingsTriggerSnapshot.generation,
      agentId: 'test:profile-navigation-trigger',
      action: 'invoke',
    });
    const settingsNavigationSnapshot = await client.call('snapshot', { maxElements: 500, includeText: true }) as {
      generation: number;
    };
    await client.call('action', {
      generation: settingsNavigationSnapshot.generation,
      agentId: 'test:profile-navigation-settings',
      action: 'invoke',
    });
    await expect(page.getByTestId('settings-modal-backdrop')).toBeVisible();
    const accountCategorySnapshot = await client.call('snapshot', { maxElements: 500, includeText: true }) as {
      generation: number;
    };
    await client.call('action', {
      generation: accountCategorySnapshot.generation,
      agentId: 'test:settings-category-account',
      action: 'invoke',
    });
    const logoutTarget = await client.call('find', { agentId: 'settings-logout', limit: 5 }) as {
      count: number;
      matches: Array<{ agentId?: string; stable?: boolean; visible?: boolean; enabled?: boolean }>;
    };
    expect(logoutTarget.count).toBe(1);
    expect(logoutTarget.matches[0]).toMatchObject({
      agentId: 'settings-logout',
      stable: true,
      visible: true,
      enabled: true,
    });

    const webFallback = await page.evaluate(async () => {
      const registry = (window as unknown as {
        __fabushiAppMcp?: {
          list(): Array<{ name: string }>;
          call(name: string, input?: Record<string, unknown>): Promise<unknown>;
        };
      }).__fabushiAppMcp;
      if (!registry) return null;
      const localStatus = await registry.call('fabushi.app.status', {});
      return { tools: registry.list().map((tool) => tool.name), status: localStatus };
    });
    expect(webFallback?.tools).toContain('fabushi.app.snapshot');
    expect((webFallback?.status as { available?: boolean })?.available).toBe(true);
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
