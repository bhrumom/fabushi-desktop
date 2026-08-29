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

async function openMahayanaConversation(page: Page): Promise<void> {
  const peer = page.getByTestId('peer-legacy:conversation:codex:agent:assistant');
  await expect(peer).toBeVisible({ timeout: 15_000 });
  await peer.click();
  await expect(page.getByTestId('messenger-input')).toBeVisible();
}

async function createSelfHostedBotAcceptanceChannel(page: Page): Promise<{ conversationId: string; peerTestId: string }> {
  await page.getByTestId('profile-navigation-trigger').click();
  const chats = page.getByTitle('聊天', { exact: true });
  if (await chats.isVisible().catch(() => false)) await chats.click();
  await page.getByRole('button', { name: '新建', exact: true }).click();
  await page.getByRole('button', { name: '新建频道' }).click();
  await page.getByPlaceholder('频道名称').fill('自建 Bot Mahayana 验收');
  await page.getByPlaceholder('频道简介').fill('Rust messaging → Mahayana multi-step runtime');
  await page.getByRole('button', { name: '创建频道' }).click();

  const peer = page.locator('[data-testid^="peer-selfhosted:channel:"]').filter({ hasText: '自建 Bot Mahayana 验收' }).first();
  await expect(peer).toBeVisible({ timeout: 10_000 });
  const peerTestId = await peer.getAttribute('data-testid');
  expect(peerTestId).toBeTruthy();
  await peer.click();
  await expect(page.getByTestId('messenger-input')).toBeVisible();
  return {
    peerTestId: peerTestId!,
    conversationId: peerTestId!.replace(/^peer-selfhosted:/, ''),
  };
}

async function emitBotInvocationRequested(
  page: Page,
  conversationId: string,
  text: string,
): Promise<string> {
  return page.evaluate(({ conversationId: id, text: prompt }) => {
    const invocationId = `invocation:e2e:${Date.now()}:${Math.random().toString(36).slice(2, 8)}`;
    window.dispatchEvent(new CustomEvent('fabushi:mahayana-runtime-event', {
      detail: {
        type: 'messaging.event',
        timestamp: new Date().toISOString(),
        requestId: `messaging:e2e:${Date.now()}`,
        envelope: {
          protocolVersion: 2,
          cursor: String(Date.now()),
          serverTimeMs: Date.now(),
          event: {
            type: 'botInvocationRequested',
            invocation: {
              id: invocationId,
              botId: 'bot:e2e:helper',
              senderId: 'human:e2e',
              conversationId: id,
              text: { text: prompt, entities: [] },
              metadata: { source: 'messaging-service' },
              createdAtMs: Date.now(),
            },
          },
        },
      },
    }));
    return invocationId;
  }, { conversationId, text });
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
    await expect(page.getByRole('article').filter({ hasText: '收到：请分析这个任务' }).last()).toBeVisible();
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
    await expect(page.getByRole('article').filter({ hasText: '收到：请分析这个任务' }).last()).toBeVisible();
  } finally {
    await app?.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('self-hosted Bot invocation is consumed by Mahayana multi-step runtime without actor impersonation and restores after restart', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-selfhosted-bot-mahayana-'));
  let app: ElectronApplication | null = null;
  const prompt = '自建 Bot 请规划步骤，调用 Mahayana 工具并完成这个任务。';

  try {
    app = await launchDesktopApp(appDataDir);
    let page = await app.firstWindow();
    await completeBrowserLogin(page);
    const { conversationId, peerTestId } = await createSelfHostedBotAcceptanceChannel(page);

    // The authenticated human turn first goes through the canonical Rust messaging store.
    await page.getByTestId('messenger-input').fill(prompt);
    await page.getByTestId('messenger-send').click();
    await expect(page.getByRole('article').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 10_000 });

    // The Rust messaging service has a separate contract test proving that a
    // human message to a Bot produces this exact BotInvocationRequested event.
    // Here we verify the Electron consumer half: event -> Mahayana -> visible run.
    const invocationId = await emitBotInvocationRequested(page, conversationId, prompt);
    expect(invocationId).toContain('invocation:e2e:');

    const run = page.getByTestId('agent-run').last();
    await expect(run).toBeVisible({ timeout: 15_000 });
    await expect(run).toHaveAttribute('data-status', 'completed', { timeout: 15_000 });
    await expect.poll(async () => run.getByTestId('agent-step').count()).toBeGreaterThanOrEqual(3);
    await expect(run).toContainText('Mahayana');
    await expect(page.locator('#mahayana-agent-header-avatar [data-agent-state="result"]')).toBeVisible();

    const persistedRunId = await run.getAttribute('data-run-id');
    expect(persistedRunId).toBeTruthy();
    await expect.poll(async () => page.evaluate(() => {
      const journal = JSON.parse(localStorage.getItem('fabushi.desktop.selfhosted-mahayana-invocations.v1') || 'null');
      return Object.values(journal?.claims || {}).some((claim: unknown) =>
        Boolean(claim && typeof claim === 'object' && (claim as { state?: string }).state === 'accepted'));
    })).toBe(true);

    await app.close();
    app = null;

    app = await launchDesktopApp(appDataDir);
    page = await app.firstWindow();
    await completeBrowserLogin(page);
    const restoredPeer = page.getByTestId(peerTestId);
    await expect(restoredPeer).toBeVisible({ timeout: 15_000 });
    await restoredPeer.click();
    await expect(page.getByRole('article').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 10_000 });

    const restoredRun = page.locator(`[data-testid="agent-run"][data-run-id="${persistedRunId}"]`);
    await expect(restoredRun).toBeVisible({ timeout: 15_000 });
    await expect(restoredRun).toHaveAttribute('data-status', 'completed');
    await expect.poll(async () => restoredRun.getByTestId('agent-step').count()).toBeGreaterThanOrEqual(3);
  } finally {
    await app?.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
  }
});
