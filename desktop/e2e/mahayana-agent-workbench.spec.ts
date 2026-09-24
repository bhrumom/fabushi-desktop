import { _electron as electron, expect, test, type ElectronApplication, type Locator, type Page } from '@playwright/test';
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
      // Focused Electron acceptance must opt out of production background
      // persistence so Playwright app.close() reaches the real before-quit
      // cleanup path instead of being converted into a hidden-window session.
      FABUSHI_E2E: '1',
      FABUSHI_APP_DATA: appDataDir,
      FABUSHI_FEATURE_HOST_MODE: process.env.FABUSHI_FEATURE_HOST_MODE || 'test',
      MAHAYANA_APP_HOST_BIN: process.env.MAHAYANA_APP_HOST_BIN || '',
    },
  });
}

async function completeBrowserLogin(page: Page): Promise<void> {
  const rendererErrors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') rendererErrors.push(message.text());
  });

  await page.waitForFunction(() => {
    const candidate = window as unknown as { desktop?: { cursorAccount?: { getStatus?: unknown }; onboarding?: { setSeen?: unknown } } };
    return typeof candidate.desktop?.cursorAccount?.getStatus === 'function'
      && typeof candidate.desktop?.onboarding?.setSeen === 'function';
  }, { timeout: 15_000 });

  const accountKind = async (): Promise<string> => page.evaluate(async () => {
    const candidate = window as unknown as {
      desktop: {
        cursorAccount: { getStatus(): Promise<{ kind?: string }> };
      };
    };
    return (await candidate.desktop.cursorAccount.getStatus()).kind ?? 'unknown';
  });

  const initialKind = await accountKind();
  if (initialKind === 'logged-out') {
    // Exercise the shipping recovered-Grok sign-in surface instead of the
    // retired DesktopAuthBoundary. Its button calls cursorAccount.login(),
    // which owns the Electron/main auth contract used by the production shell.
    const signInSurface = page.getByRole('main', { name: 'Grok Bot', exact: true });
    await expect(signInSurface).toBeVisible({ timeout: 15_000 });
    await signInSurface.getByRole('button', { name: 'Sign in', exact: true }).click();
  }

  await expect.poll(accountKind, { timeout: 20_000 }).toBe('logged-in');
  // Onboarding persistence is account-scoped. Complete the shipping login
  // transition first, then persist the public preference for the authenticated
  // account and verify the mirror before recreating the renderer. This avoids
  // writing into a departing anonymous scope while still exercising the real
  // cursorAccount -> Electron main -> Coordinator/Host authentication path.
  await page.evaluate(async () => {
    const candidate = window as unknown as {
      desktop: {
        onboarding: {
          setSeen(seen: boolean): Promise<unknown>;
        };
      };
    };
    await candidate.desktop.onboarding.setSeen(true);
  });
  const onboardingSeen = async (): Promise<boolean> => page.evaluate(async () => {
    const candidate = window as unknown as {
      desktop: {
        onboarding: {
          getSeen(): Promise<boolean>;
        };
      };
    };
    return (await candidate.desktop.onboarding.getSeen()) === true;
  });
  await expect.poll(onboardingSeen, { timeout: 10_000 }).toBe(true);

  // The production renderer resolves onboarding at account/bootstrap boundaries;
  // reload only the renderer after persisting the authenticated account setting.
  // Main/Coordinator/Host and the authenticated session remain live.
  await page.reload({ waitUntil: 'domcontentloaded' });
  await page.waitForFunction(() => {
    const candidate = window as unknown as { desktop?: { cursorAccount?: { getStatus?: unknown } } };
    return typeof candidate.desktop?.cursorAccount?.getStatus === 'function';
  }, { timeout: 15_000 });
  await expect.poll(accountKind, { timeout: 10_000 }).toBe('logged-in');
  await expect(page.getByRole('main', { name: 'Grok Bot', exact: true })).toBeHidden({ timeout: 10_000 });

  const fatal = page.locator('.sand-error-boundary--app');
  if (await fatal.count()) {
    const surfaceText = await fatal.first().innerText().catch(() => 'unknown renderer failure');
    throw new Error(`renderer root fatal: ${rendererErrors.at(-1) ?? surfaceText}`);
  }

  // listAgents is now owned by the shipping Rust Session store. A fresh
  // FABUSHI_APP_DATA directory is intentionally empty, so create the focused
  // fixture through the real New -> createAgent -> listAgents path instead of
  // relying on the retired compatibility Host's synthetic roster.
  const roster = page.getByRole('region', { name: 'Agent list' });
  const primary = roster.getByRole('button', { name: 'New chat', exact: true });
  if (await primary.count() === 0) {
    await page.getByRole('button', { name: 'New', exact: true }).click();
  }
  await expect(primary).toBeVisible({ timeout: 15_000 });
}

async function openMahayanaConversation(page: Page): Promise<void> {
  const peer = page.getByRole('region', { name: 'Agent list' }).getByRole('button', { name: 'New chat', exact: true });
  await expect(peer).toBeVisible({ timeout: 15_000 });
  await peer.click();
  const prompt = page.getByRole('textbox', { name: 'Prompt' });
  await expect(prompt).toBeVisible();
  // New -> createAgent -> refreshRoster -> openAgent is a real shipping async
  // transition. The production composer intentionally remains non-editable
  // while that transition owns the busy state, so acceptance must wait for
  // the same actionable contract a user sees instead of racing visibility.
  await expect(prompt).toHaveAttribute('contenteditable', 'true', { timeout: 15_000 });
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

async function expectHermesAssistantTurn(page: Page, expectedText: string): Promise<Locator> {
  const transcript = page.getByRole('log', { name: 'Conversation transcript' });
  const matchingMessages = transcript.getByRole('group', { name: 'Agent message' }).filter({ hasText: expectedText });
  await expect(matchingMessages).toHaveCount(1, { timeout: 15_000 });

  const message = matchingMessages.first();
  await expect(message).toBeVisible();
  const turn = message.locator('xpath=ancestor::*[@role="article" and @data-role="assistant"][1]');
  await expect(turn).toBeVisible();
  await expect(message).toBeVisible();
  const body = message.locator('.sand-message-prose');
  await expect(body).toContainText(expectedText);
  await expect(body).not.toContainText('chat-response');

  // Routine success belongs in the recovered Grok transcript. The retired
  // Mahayana turn/Workbench presentation must not reappear as a parallel
  // success surface, and no operation-scoped thinking/tool row may remain
  // pending after the final assistant message has settled.
  await expect(page.getByTestId('mahayana-assistant-turn')).toHaveCount(0);
  await expect(page.getByTestId('agent-workbench')).toBeHidden();
  await expect(transcript.locator('[data-kind="thinking"]')).toHaveCount(0);
  await expect(transcript.locator('[data-kind="tool-call"][data-status="pending"]')).toHaveCount(0);

  // Token-sized runtime deltas and the late final message must reconcile into
  // one canonical assistant body rather than producing duplicate replies.
  const bodyText = (await body.allTextContents()).join('');
  expect(bodyText.split(expectedText).length - 1).toBe(1);
  return turn;
}

test('Mahayana renders one Hermes-style assistant turn instead of a completion Workbench card', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-mahayana-assistant-turn-'));
  let app: ElectronApplication | null = null;

  try {
    app = await launchDesktopApp(appDataDir);
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMahayanaConversation(page);

    const prompt = '请分析这个任务，规划步骤，调用工具并给出最终结果。';
    const promptInput = page.getByRole('textbox', { name: 'Prompt' });
    await promptInput.fill(prompt);
    await page.getByRole('button', { name: 'Send message' }).click();

    // The user bubble is a local-first transition and must paint before the
    // Mahayana Host finishes accepting/routing the agent turn. The Hermes
    // assistant projection is validated below with the production 15s turn
    // contract; requiring it inside this 1s local-echo window races Host
    // acceptance and prevents the stronger lifecycle assertions from running.
    await expect(page.getByRole('article').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 1_000 });
    await expect(promptInput).toBeVisible();

    const turn = await expectHermesAssistantTurn(page, '收到：请分析这个任务');
    await expect(turn).toHaveCount(1);
    await expect(promptInput).toBeVisible();
  } finally {
    await app?.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('self-hosted Bot invocation projects into the same Hermes-style turn without actor impersonation', async () => {
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
    // Here we verify the Electron consumer half: invocation -> Mahayana -> one
    // ordinary assistant turn instead of a second task-card surface.
    const invocationId = await emitBotInvocationRequested(page, conversationId, prompt);
    expect(invocationId).toContain('invocation:e2e:');

    await expectHermesAssistantTurn(page, '收到：自建 Bot 请规划步骤');
    await expect(page.getByTestId('agent-run')).toBeHidden();

    // This journal is only the idempotency/claim record for consuming the Bot
    // invocation. It is not accepted as transcript or run-state authority.
    await expect.poll(async () => page.evaluate(() => {
      const journal = JSON.parse(localStorage.getItem('fabushi.desktop.selfhosted-mahayana-invocations.v1') || 'null');
      return Object.values(journal?.claims || {}).some((claim: unknown) =>
        Boolean(claim && typeof claim === 'object' && (claim as { state?: string }).state === 'accepted'));
    })).toBe(true);

    await app.close();
    app = null;

    // Restart coverage is intentionally limited to the canonical messaging
    // history here. Ordered AssistantTurn replay must come from the production
    // Rust gateway/session store and is a separate MSR-204 acceptance blocker.
    app = await launchDesktopApp(appDataDir);
    page = await app.firstWindow();
    await completeBrowserLogin(page);
    const restoredPeer = page.getByTestId(peerTestId);
    await expect(restoredPeer).toBeVisible({ timeout: 15_000 });
    await restoredPeer.click();
    await expect(page.getByRole('article').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 10_000 });
  } finally {
    await app?.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
  }
});
