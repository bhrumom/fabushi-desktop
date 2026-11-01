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

    await page.getByTestId('messenger-input').fill('semantic message action probe');
    await page.getByTestId('messenger-send').click();
    const semanticMessage = page.locator('[data-agent-id^="message-actions:"][data-agent-invoke="contextmenu"][data-agent-message-role="me"]').filter({ hasText: 'semantic message action probe' }).last();
    await expect(semanticMessage).toBeVisible();
    await expect(semanticMessage).toHaveAttribute('data-agent-message-role', 'me');
    const semanticMessageAgentId = await semanticMessage.getAttribute('data-agent-id');
    expect(semanticMessageAgentId).toBeTruthy();

    await page.evaluate(() => {
      const overflow = document.createElement('div');
      overflow.id = 'semantic-overflow-decoys';
      overflow.style.display = 'none';
      for (let index = 0; index < 520; index += 1) {
        const button = document.createElement('button');
        button.dataset.agentId = `semantic-overflow-decoy:${index}`;
        button.textContent = `semantic overflow decoy ${index}`;
        overflow.appendChild(button);
      }
      document.body.prepend(overflow);
    });
    await page.waitForTimeout(50);

    const messageSnapshot = await client.call('snapshot', { maxElements: 500, includeText: true }) as {
      generation: number;
      truncated: boolean;
      elements: Array<{ agentId?: string }>;
    };
    expect(messageSnapshot.truncated).toBe(true);
    expect(messageSnapshot.elements.some((element) => element.agentId === semanticMessageAgentId)).toBe(false);
    const freshSemanticMessage = await client.call('find', { agentId: semanticMessageAgentId, limit: 1 }) as {
      generation: number;
      count: number;
      matches: Array<{ agentId?: string }>;
    };
    expect(freshSemanticMessage.count).toBe(1);
    expect(freshSemanticMessage.matches[0]?.agentId).toBe(semanticMessageAgentId);
    await client.call('action', {
      generation: freshSemanticMessage.generation,
      agentId: semanticMessageAgentId,
      action: 'invoke',
    });
    await expect(page.getByTestId('message-context-menu')).toBeVisible();

    for (const actionName of ['reply', 'copy', 'react', 'edit', 'pin', 'forward', 'delete']) {
      const found = await client.call('find', { agentId: `test:message-action-${actionName}`, limit: 2 }) as {
        generation: number;
        count: number;
        matches: Array<{ agentId?: string }>;
      };
      await expect(page.getByTestId(`message-action-${actionName}`), `DOM menu action ${actionName} must be rendered for the authored message`).toBeVisible();
      expect(found.count, `semantic find must resolve message action ${actionName}`).toBe(1);
      expect(found.matches[0]?.agentId).toBe(`test:message-action-${actionName}`);
    }
    const freshReply = await client.call('find', { agentId: 'test:message-action-reply', limit: 1 }) as {
      generation: number;
      count: number;
    };
    expect(freshReply.count).toBe(1);
    await client.call('action', {
      generation: freshReply.generation,
      agentId: 'test:message-action-reply',
      action: 'invoke',
    });
    await expect(page.getByTestId('reply-message-banner')).toBeVisible();
    await page.evaluate(() => document.getElementById('semantic-overflow-decoys')?.remove());

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
