import { chromium, expect, test, type Browser, type BrowserContext, type Locator, type Page } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import { appendFile, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

const realAcceptance = process.env.OBF_REAL_ACCEPTANCE === '1';
const executable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || '';
const sourceSha = process.env.OBF_SOURCE_SHA?.trim() || '';
const expectedSourceSha = process.env.OBF_EXPECTED_SOURCE_SHA?.trim() || sourceSha;
const evidenceRoot = process.env.OBF_EVIDENCE_DIR?.trim()
  || path.join(tmpdir(), `fabushi-openbot-acceptance-${sourceSha.slice(0, 12) || 'unknown'}`);

// This file owns tracing so the canonical evidence path is stable.
// Playwright requires test.use() at file scope because a describe-scoped trace
// override would force a new worker and fail before packaged acceptance starts.
test.use({ trace: 'off' });

type LifecycleSample = {
  readonly at: number;
  readonly type: 'operation.started' | 'turn.state' | 'chat.delta' | 'chat.message' | 'agent.step' | 'operation.completed' | 'operation.failed';
  readonly operationId?: string;
  readonly status: string;
  readonly text: string;
};

type RuntimeLog = {
  readonly at: number;
  readonly source: string;
  readonly text: string;
};

type BackgroundEventSample = {
  readonly at: number;
  readonly type: 'agent.backgroundStarted' | 'agent.backgroundFinished';
  readonly agentId: string;
  readonly agentName: string;
  readonly operationId: string;
  readonly source: string;
  readonly error?: string;
};

const coworkers = [
  ['Chief', 'Coordinates decisions and synthesizes final output.'],
  ['Research', 'Collects source material and facts.'],
  ['Builder', 'Executes implementation work.'],
  ['Launch', 'Validates release readiness.'],
] as const;

async function withNodeDeadline<T>(label: string, timeoutMs: number, task: Promise<T>): Promise<T> {
  let timer: NodeJS.Timeout | undefined;
  try {
    return await Promise.race([
      task,
      new Promise<T>((_, reject) => {
        timer = setTimeout(() => reject(new Error(`${label} exceeded ${timeoutMs}ms`)), timeoutMs);
      }),
    ]);
  } finally {
    if (timer) clearTimeout(timer);
  }
}

type CdpTargetInfo = {
  readonly id?: string;
  readonly type?: string;
  readonly title?: string;
  readonly url?: string;
};

function isShippingRendererUrl(url: string): boolean {
  if (url.startsWith('app://bundle/')) return true;
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'file:' && parsed.pathname.endsWith('/dist/renderer/index.html');
  } catch {
    return false;
  }
}

async function waitForPackagedRendererBinding(
  appDataDir: string,
): Promise<{
  page: Page;
  browser: Browser;
  binding: 'cdp';
  cdpPort: number;
  targets: readonly CdpTargetInfo[];
}> {
  const deadline = Date.now() + 120_000;
  let lastTargets: readonly CdpTargetInfo[] = [];
  let lastCdpError = '';
  let lastPort = 0;

  while (Date.now() < deadline) {
    try {
      const activePort = await readFile(path.join(appDataDir, 'DevToolsActivePort'), 'utf8');
      const [portText] = activePort.trim().split(/\r?\n/u);
      const port = Number(portText);
      if (!Number.isInteger(port) || port <= 0 || port > 65_535) {
        lastCdpError = `invalid DevToolsActivePort: ${JSON.stringify(portText)}`;
        await new Promise((resolve) => setTimeout(resolve, 250));
        continue;
      }
      lastPort = port;

      // Probe Chromium's target registry without auto-attaching. The signed app
      // must first expose its real shipping renderer target. Playwright only
      // attaches after that production-owned target exists.
      const response = await withNodeDeadline(
        'Read packaged CDP target registry',
        2_000,
        fetch(`http://127.0.0.1:${port}/json/list`),
      );
      if (!response.ok) {
        lastCdpError = `CDP target registry returned HTTP ${response.status}`;
        await new Promise((resolve) => setTimeout(resolve, 250));
        continue;
      }
      const payload = await response.json();
      lastTargets = Array.isArray(payload)
        ? payload.filter((entry): entry is CdpTargetInfo => entry != null && typeof entry === 'object')
        : [];
      if (!lastTargets.some((target) => target.type === 'page' && typeof target.url === 'string' && isShippingRendererUrl(target.url))) {
        await new Promise((resolve) => setTimeout(resolve, 250));
        continue;
      }

      const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`, { timeout: 30_000 });
      const page = browser.contexts()
        .flatMap((context) => context.pages())
        .find((candidate) => isShippingRendererUrl(candidate.url()));
      if (page) {
        return { page, browser, binding: 'cdp', cdpPort: port, targets: lastTargets };
      }
      await browser.close().catch(() => undefined);
      lastCdpError = 'shipping target existed in /json/list but Playwright did not expose its Page';
    } catch (cause) {
      lastCdpError = cause instanceof Error ? cause.message : String(cause);
    }

    await new Promise((resolve) => setTimeout(resolve, 250));
  }

  throw new Error(
    `Signed production process did not expose a bindable packaged BrowserWindow target. port=${lastPort || 'none'} targets=${JSON.stringify(lastTargets)} cdp=${lastCdpError || 'no target'}`,
  );
}

function peerByName(page: Page, name: string): Locator {
  return page
    .getByTestId('messenger-sidebar')
    .locator('button[data-agent-id]')
    .filter({ hasText: name })
    .first();
}

async function completeBrowserLogin(page: Page): Promise<void> {
  type LoginPhase = 'onboarding' | 'login' | 'browser-waiting' | 'ready' | 'waiting';
  const workspace = page.getByTestId('messenger-workspace');
  const signInLanding = page.getByRole('main', { name: 'Grok Bot' });
  const onboarding = page.locator('.sand-onboarding[data-step]');

  const readAccountKind = async (): Promise<string | null> => page.evaluate(async () => {
    const desktop = (window as unknown as {
      desktop?: {
        cursorAccount?: {
          getStatus?: () => Promise<{ kind?: unknown }>;
        };
      };
    }).desktop;
    if (typeof desktop?.cursorAccount?.getStatus !== 'function') return null;
    try {
      const status = await desktop.cursorAccount.getStatus();
      return typeof status?.kind === 'string' ? status.kind : null;
    } catch {
      return null;
    }
  });

  const readPhase = async (): Promise<LoginPhase> => {
    if (!(await workspace.count())) return 'waiting';

    const accountKind = await readAccountKind();
    if (accountKind === 'logged-in') {
      if (await onboarding.count()) return 'onboarding';
      if (await signInLanding.count()) return 'waiting';
      return 'ready';
    }
    if (accountKind === 'logging-in') return 'browser-waiting';
    if (accountKind === 'logged-out') return 'login';

    if (await signInLanding.count()) {
      if (await signInLanding.getByRole('button', { name: 'Sign in', exact: true }).count()) return 'login';
      return 'browser-waiting';
    }
    return 'waiting';
  };

  const hostHydrated = async (): Promise<boolean> => page.evaluate(async () => {
    const bridge = window.mahayana;
    if (!bridge?.invoke) return false;
    try {
      await bridge.invoke('feature.execute', {
        command: {
          type: 'bot.list',
          requestId: `candidate-ready-${Date.now()}-${Math.random().toString(16).slice(2)}`,
        },
      });
      return true;
    } catch {
      return false;
    }
  });

  await withNodeDeadline(
    'Packaged renderer did not mount the canonical ProductionRenderer workspace',
    35_000,
    expect(workspace).toBeVisible({ timeout: 30_000 }),
  );
  await expect(workspace).toHaveAttribute('data-product-shell', 'agent');
  await expect(workspace).toHaveAttribute('data-runtime', 'electron');

  for (let attempt = 0; attempt < 18; attempt += 1) {
    await expect.poll(readPhase, { timeout: 15_000 }).not.toBe('waiting');
    const phase = await readPhase();

    if (phase === 'onboarding') {
      const step = await onboarding.getAttribute('data-step');
      if (step === 'meet' || step === 'computer-demo' || step === 'jobs' || step === 'tools') {
        const next = onboarding.getByRole('button', { name: 'Next', exact: true }).last();
        await expect(next).toBeVisible();
        await next.click();
        continue;
      }
      if (step === 'create') {
        const name = onboarding.getByPlaceholder('New Bot');
        await expect(name).toBeVisible();
        if ((await name.inputValue()).trim().length === 0) {
          await name.fill(`Candidate Acceptance ${Date.now()}`);
        }
        const getStarted = onboarding.getByRole('button', { name: 'Get started', exact: true });
        await expect(getStarted).toBeEnabled();
        await getStarted.click();
        continue;
      }
      if (step === 'hand-off') {
        const retry = onboarding.getByRole('button', { name: 'Try again', exact: true });
        if (await retry.count()) {
          throw new Error('Signed-in onboarding failed while preparing the production computer/Agent hand-off');
        }
        try {
          await expect.poll(readPhase, { timeout: 90_000 }).not.toBe('onboarding');
        } catch {
          throw new Error('Signed-in onboarding did not settle into the canonical Agent workspace');
        }
        continue;
      }
      throw new Error(`Unexpected signed-in onboarding step: ${step ?? 'unknown'}`);
    }

    if (phase === 'login') {
      const start = signInLanding.getByRole('button', { name: 'Sign in', exact: true });
      await expect(start).toBeVisible();
      await start.click();
      continue;
    }

    if (phase === 'browser-waiting') {
      try {
        await expect.poll(readPhase, { timeout: 45_000 }).not.toBe('browser-waiting');
      } catch {
        throw new Error(
          'Production browser authorization did not complete. Signed candidate acceptance requires a pre-authorized CI account/session; test-mode auth fallback is forbidden.',
        );
      }
      continue;
    }

    if (phase === 'ready') {
      await expect(signInLanding).toHaveCount(0);
      await expect(onboarding).toHaveCount(0);
      try {
        await expect.poll(hostHydrated, { timeout: 45_000 }).toBe(true);
      } catch {
        throw new Error(
          'Canonical ProductionRenderer mounted and authenticated, but the signed candidate never hydrated a usable Mahayana Host/Coordinator bot.list path.',
        );
      }
      return;
    }
  }

  throw new Error('Packaged Fabushi did not reach the canonical Agent workspace');
}

async function createCoworker(page: Page, name: string, description: string): Promise<void> {
  await page.evaluate(async ({ botName, botDescription }) => {
    const bridge = window.mahayana;
    if (!bridge?.invoke) throw new Error('Mahayana bridge unavailable');
    const now = Date.now();
    await bridge.invoke('feature.execute', {
      command: {
        type: 'bot.create',
        requestId: `candidate-bot-create-${botName}-${now}`,
        name: botName,
        description: botDescription,
      },
    });
    await bridge.invoke('feature.execute', {
      command: {
        type: 'bot.list',
        requestId: `candidate-bot-list-${botName}-${now}`,
      },
    });
  }, { botName: name, botDescription: description });
  await expect(peerByName(page, name)).toBeVisible({ timeout: 20_000 });
}

async function openAgent(page: Page, name: string): Promise<void> {
  const peer = peerByName(page, name);
  await expect(peer).toBeVisible({ timeout: 20_000 });
  await peer.click();
  await expect(page.getByTestId('messenger-input')).toBeVisible();
  await expect(page.getByTestId('grok-agent-header')).toContainText(name);
}

function completedAssistantTurns(page: Page): Locator {
  // The Agent-first transcript renders the canonical Rust-owned assistant turn
  // directly. Count only terminal completed turns: an optimistic/streaming turn
  // must never satisfy packaged acceptance merely because it is visible.
  return page.locator('[data-testid="mahayana-assistant-turn"][data-status="completed"]');
}

async function submitTurn(page: Page, prompt: string): Promise<number> {
  const previousAssistantCount = await completedAssistantTurns(page).count();
  const input = page.getByTestId('messenger-input');
  await input.fill(prompt);
  await page.getByTestId('messenger-send').click();
  await expect(page.locator('[data-agent-message-role="me"]').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 5_000 });
  return previousAssistantCount;
}

async function waitForCompletedTurn(
  page: Page,
  prompt: string,
  previousAssistantCount: number,
): Promise<Locator> {
  await expect(page.locator('[data-agent-message-role="me"]').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 10_000 });
  const assistantTurns = completedAssistantTurns(page);
  await expect.poll(
    async () => assistantTurns.count(),
    { timeout: 180_000, message: 'A new canonical completed assistant turn must be committed after the submitted turn.' },
  ).toBeGreaterThan(previousAssistantCount);
  const turn = assistantTurns.last();
  await expect(turn).toBeVisible({ timeout: 10_000 });
  return turn;
}

async function screenshot(page: Page, name: string): Promise<void> {
  await page.screenshot({ path: path.join(evidenceRoot, 'screenshots', `${name}.png`), fullPage: true });
}

async function resetBackgroundCapture(page: Page): Promise<void> {
  await page.evaluate(() => {
    const scope = window as typeof window & {
      __candidateBackgroundEvents?: BackgroundEventSample[];
      __candidateBackgroundUnsubscribe?: () => void;
    };
    scope.__candidateBackgroundEvents = [];
    if (scope.__candidateBackgroundUnsubscribe) return;
    const bridge = window.mahayana;
    if (!bridge?.subscribe) throw new Error('Mahayana runtime event subscription is unavailable.');
    scope.__candidateBackgroundUnsubscribe = bridge.subscribe((event) => {
      if (event.type !== 'agent.backgroundStarted' && event.type !== 'agent.backgroundFinished') return;
      scope.__candidateBackgroundEvents?.push({
        at: Date.now(),
        type: event.type,
        agentId: event.agentId,
        agentName: event.agentName,
        operationId: event.operationId,
        source: event.source,
        ...(event.type === 'agent.backgroundFinished' && event.error ? { error: event.error } : {}),
      });
    });
  });
}

async function readBackgroundEvents(page: Page): Promise<BackgroundEventSample[]> {
  return page.evaluate(() => {
    const scope = window as typeof window & { __candidateBackgroundEvents?: BackgroundEventSample[] };
    return scope.__candidateBackgroundEvents ?? [];
  });
}

async function waitForBackgroundFinished(
  page: Page,
  agentNames: readonly string[],
  source: string,
): Promise<void> {
  for (const agentName of agentNames) {
    await expect.poll(async () => {
      const events = await readBackgroundEvents(page);
      const started = events.find((event) =>
        event.type === 'agent.backgroundStarted'
        && event.agentName.includes(agentName)
        && event.source.startsWith(source));
      if (!started) return 'not-started';
      const finished = events.find((event) =>
        event.type === 'agent.backgroundFinished'
        && event.operationId === started.operationId);
      if (!finished) return 'running';
      return finished.error ? `error:${finished.error}` : 'completed';
    }, { timeout: 180_000 }).toBe('completed');
  }
}

async function installLifecycleCapture(page: Page): Promise<void> {
  await page.evaluate(() => {
    const scope = window as typeof window & {
      __candidateLifecycle?: LifecycleSample[];
      __candidateLifecycleUnsubscribe?: () => void;
    };
    scope.__candidateLifecycleUnsubscribe?.();
    scope.__candidateLifecycle = [];

    const bridge = window.mahayana;
    if (!bridge?.subscribe) throw new Error('Mahayana runtime event subscription is unavailable.');
    scope.__candidateLifecycleUnsubscribe = bridge.subscribe((event) => {
      const at = Date.now();
      if (event.type === 'operation.started') {
        scope.__candidateLifecycle?.push({
          at,
          type: event.type,
          operationId: event.operationId,
          status: 'running',
          text: event.label,
        });
        return;
      }
      if (event.type === 'turn.state') {
        scope.__candidateLifecycle?.push({
          at,
          type: event.type,
          operationId: event.operationId,
          status: event.state,
          text: `${event.turnId}:${event.runId}:${event.sequence}`,
        });
        return;
      }
      if (event.type === 'chat.delta') {
        scope.__candidateLifecycle?.push({
          at,
          type: event.type,
          operationId: event.operationId,
          status: 'streaming',
          text: event.delta,
        });
        return;
      }
      if (event.type === 'chat.message' && event.operationId) {
        scope.__candidateLifecycle?.push({
          at,
          type: event.type,
          operationId: event.operationId,
          status: event.role === 'assistant' ? 'streaming' : 'message',
          text: event.text,
        });
        return;
      }
      if (event.type === 'agent.step') {
        scope.__candidateLifecycle?.push({
          at,
          type: event.type,
          operationId: event.operationId,
          status: event.status,
          text: `${event.kind}:${event.title}`,
        });
        return;
      }
      if (event.type === 'operation.completed') {
        scope.__candidateLifecycle?.push({
          at,
          type: event.type,
          operationId: event.operationId,
          status: 'completed',
          text: '',
        });
        return;
      }
      if (event.type === 'operation.failed') {
        scope.__candidateLifecycle?.push({
          at,
          type: event.type,
          operationId: event.operationId,
          status: 'failed',
          text: `${event.code}:${event.message}`,
        });
      }
    });
  });
}

async function performDirectHandoff(page: Page): Promise<void> {
  await openAgent(page, 'Chief');
  await page.getByRole('button', { name: 'Agent network' }).click();
  const network = page.getByTestId('grok-agent-network');
  await expect(network).toBeVisible();

  await resetBackgroundCapture(page);
  const research = network.locator('article').filter({ hasText: 'Research' }).first();
  const researchCheckbox = research.getByRole('checkbox');
  await expect(researchCheckbox).toBeVisible({ timeout: 10_000 });
  await researchCheckbox.check({ timeout: 10_000 });
  await expect(researchCheckbox).toBeChecked();
  await network.getByRole('textbox').fill('Research: verify the candidate handoff path and report one concise fact.');
  const handoffButton = network.getByRole('button', { name: /Handoff to Research/ });
  await expect(handoffButton).toBeVisible({ timeout: 10_000 });
  await handoffButton.click({ timeout: 10_000 });
  await expect(network.getByRole('textbox')).toHaveValue('');
  await expect(network.getByText(/Chief.*Research|Research.*Chief/).first()).toBeVisible({ timeout: 20_000 });
  await waitForBackgroundFinished(page, ['Research'], 'agent-');
  await network.getByRole('button', { name: 'Close Agent network' }).click();
}

async function performBroadcast(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Broadcast to agents' }).click();
  const network = page.getByTestId('grok-agent-network');
  await expect(network).toBeVisible();
  await resetBackgroundCapture(page);

  const selectedTargets = network.getByRole('checkbox', { name: 'Broadcast' });
  for (let index = 0; index < await selectedTargets.count(); index += 1) {
    const checkbox = selectedTargets.nth(index);
    if (await checkbox.isChecked()) await checkbox.uncheck();
    await expect(checkbox).not.toBeChecked();
  }
  for (const targetName of ['Chief', 'Launch']) {
    const target = network.locator('article').filter({ hasText: targetName }).first();
    const checkbox = target.getByRole('checkbox', { name: 'Broadcast' });
    await expect(checkbox).toBeVisible({ timeout: 10_000 });
    await checkbox.check({ timeout: 10_000 });
    await expect(checkbox).toBeChecked();
  }

  await network.getByRole('textbox').fill('Candidate broadcast: acknowledge the signed package acceptance run.');
  await network.getByRole('button', { name: 'Send to selected' }).click();
  await expect(network.getByRole('textbox')).toHaveValue('');
  await waitForBackgroundFinished(page, ['Chief', 'Launch'], 'broadcast');
  await network.getByRole('button', { name: 'Close Agent network' }).click();
}

function avatarFor(locator: Locator): Locator {
  return locator.locator('[data-fab-avatar="true"]').first();
}

async function stableAvatarShape(locator: Locator): Promise<string> {
  const avatar = avatarFor(locator);
  await expect(avatar).toBeVisible();
  const shape = await avatar.getAttribute('data-shape');
  expect(shape).toBeTruthy();
  return shape!;
}

type LatencySample = {
  readonly prompt: string;
  readonly localSubmitPaintMs: number;
  readonly acceptanceVisibilityMs: number;
  readonly firstOutputMs: number;
  readonly firstTextMs: number;
  readonly completionMs: number;
  readonly operationId: string;
};

function percentile(samples: readonly number[], quantile: number): number {
  if (samples.length === 0) throw new Error('percentile requires at least one sample');
  const sorted = [...samples].sort((left, right) => left - right);
  const rank = Math.max(1, Math.ceil(quantile * sorted.length));
  return sorted[Math.min(sorted.length - 1, rank - 1)]!;
}

const ordinaryLatencyPrompts = [
  '用一句话解释为什么海水有咸味。',
  'In one sentence, explain why the daytime sky appears blue.',
  '用一句话说明植物为什么需要阳光。',
  'In one sentence, explain what a compiler does.',
  '用一句话解释月相为什么会变化。',
  'In one sentence, describe the purpose of DNS.',
  '用一句话说明为什么冰会浮在水面上。',
  'In one sentence, explain what an operating system scheduler does.',
  '用一句话解释彩虹是怎样形成的。',
  'In one sentence, explain why version control is useful.',
  '用一句话说明电池为什么能够供电。',
  'In one sentence, explain what HTTPS protects.',
  '用一句话解释潮汐主要由什么引起。',
  'In one sentence, describe what a database index is for.',
  '用一句话说明声音为什么不能在真空中传播。',
  'In one sentence, explain the difference between RAM and storage.',
  '用一句话解释为什么金属通常能导电。',
  'In one sentence, explain what a process ID identifies.',
  '用一句话说明为什么白天通常比夜晚暖。',
  'In one sentence, explain what a network retry is meant to accomplish.',
] as const;

async function runLatencyProbe(page: Page, index: number, prompt: string): Promise<LatencySample> {
  await installLifecycleCapture(page);
  const submittedAt = Date.now();
  const previousAssistantCount = await submitTurn(page, prompt);
  const localSubmitPaintMs = Date.now() - submittedAt;
  const turn = await waitForCompletedTurn(page, prompt, previousAssistantCount);
  expect(((await turn.textContent()) ?? '').trim().length, `latency turn ${index} must render ordinary assistant text`).toBeGreaterThan(0);

  const lifecycle = await page.evaluate(() => {
    const scope = window as typeof window & { __candidateLifecycle?: LifecycleSample[] };
    return scope.__candidateLifecycle ?? [];
  });
  const started = lifecycle.find((sample) => sample.type === 'operation.started' && sample.at >= submittedAt);
  expect(started?.operationId, `latency turn ${index} must emit operation.started`).toBeTruthy();
  const operationId = started!.operationId!;
  const operationLifecycle = lifecycle.filter((sample) => sample.operationId === operationId);
  const active = operationLifecycle.find((sample) =>
    sample.type === 'turn.state'
    && ['preparing', 'thinking', 'streaming', 'tool-running'].includes(sample.status));
  const firstOutput = operationLifecycle.find((sample) =>
    (sample.type === 'agent.step' && sample.text.length > 0)
    || ((sample.type === 'chat.delta' || sample.type === 'chat.message') && sample.text.length > 0));
  const firstText = operationLifecycle.find((sample) =>
    (sample.type === 'chat.delta' || sample.type === 'chat.message')
    && sample.text.length > 0);
  const completed = operationLifecycle.find((sample) =>
    sample.type === 'operation.completed' && sample.status === 'completed');
  expect(active, `latency turn ${index} must expose active state`).toBeTruthy();
  expect(firstOutput, `latency turn ${index} must expose first output`).toBeTruthy();
  expect(firstText, `latency turn ${index} must expose first text`).toBeTruthy();
  expect(completed, `latency turn ${index} must complete`).toBeTruthy();
  return {
    prompt,
    localSubmitPaintMs,
    acceptanceVisibilityMs: active!.at - started!.at,
    firstOutputMs: firstOutput!.at - started!.at,
    firstTextMs: firstText!.at - started!.at,
    completionMs: completed!.at - started!.at,
    operationId,
  };
}

async function capturePluginsEvidence(page: Page): Promise<{ text: string; itemCount: number }> {
  const button = page.getByRole('button', { name: 'Plugins' });
  await expect(button).toBeVisible({ timeout: 20_000 });
  await button.click();
  const dialog = page.getByRole('dialog', { name: 'Plugins' });
  await expect(dialog).toBeVisible({ timeout: 20_000 });
  await expect.poll(async () => (await dialog.textContent()) ?? '', { timeout: 30_000 })
    .not.toContain('Loading the marketplace');
  const text = ((await dialog.textContent()) ?? '').trim();
  const itemCount = await dialog.locator('article, [role="listitem"], button').count();
  expect(text.length, 'production Plugins surface must expose connector/catalog state').toBeGreaterThan(0);
  await screenshot(page, '08-plugins-production-surface');
  await dialog.getByRole('button', { name: 'Close' }).click();
  await expect(dialog).toBeHidden();
  return { text, itemCount };
}

test.describe('signed candidate packaged acceptance', () => {
  test.describe.configure({ retries: 0 });
  test.skip(!realAcceptance, 'Set OBF_REAL_ACCEPTANCE=1 to run signed packaged acceptance.');

  test('exact candidate covers handoff, broadcast, two-Agent isolation, lifecycle, latency and plugins', async () => {
    test.setTimeout(20 * 60_000);
    expect(executable, 'FABUSHI_ELECTRON_EXECUTABLE is required').toBeTruthy();
    expect(sourceSha, 'OBF_SOURCE_SHA must be the exact candidate HEAD').toMatch(/^[0-9a-f]{40}$/);
    expect(expectedSourceSha, 'OBF_EXPECTED_SOURCE_SHA must be a full SHA').toMatch(/^[0-9a-f]{40}$/);
    expect(sourceSha, 'candidate executable must be built from the requested exact HEAD').toBe(expectedSourceSha);
    expect(process.env.FABUSHI_FEATURE_HOST_MODE || '').not.toBe('test');
    expect(process.env.FABUSHI_E2E || '').not.toBe('1');

    // The workflow starts the fail-closed whole-session recorder before Playwright.
    // Preserve recorder-owned PID/preflight/session-frames while clearing only
    // test-owned evidence from an earlier attempt.
    await mkdir(evidenceRoot, { recursive: true });
    for (const relativePath of [
      'screenshots',
      'video',
      'app-data',
      'runtime.log',
      'startup.json',
      'failure.json',
      'candidate.json',
      'lifecycle.json',
      'timings.json',
      'connectors.json',
      'trace.zip',
    ]) {
      await rm(path.join(evidenceRoot, relativePath), { recursive: true, force: true });
    }
    await mkdir(path.join(evidenceRoot, 'screenshots'), { recursive: true });

    const appDataDir = path.join(evidenceRoot, 'app-data');
    await mkdir(appDataDir, { recursive: true });
    const runtimeLogs: RuntimeLog[] = [];
    const runtimeLogPath = path.join(evidenceRoot, 'runtime.log');
    const captureRuntimeLog = (source: string, text: string) => {
      const row: RuntimeLog = { at: Date.now(), source, text };
      runtimeLogs.push(row);
      void appendFile(runtimeLogPath, `[${new Date(row.at).toISOString()}] ${source}: ${text}\n`).catch(() => undefined);
      if (source === 'page-error' || source === 'page-crash' || source === 'request-failed' || source === 'app-stderr') {
        process.stderr.write(`[candidate ${source}] ${text}\n`);
      }
    };
    let appProcess: ChildProcess | null = null;
    let cdpBrowser: Browser | null = null;
    let pageForTrace: Page | null = null;
    let traceContext: BrowserContext | null = null;
    let traceStarted = false;
    let acceptanceCompleted = false;

    try {
      // Launch the exact installed, signed candidate as a normal production
      // process. Playwright's Electron launcher injects a Node inspector/loader
      // and changes startup semantics, so it is not the packaged launch contract.
      // The only added Chromium switch exposes loopback CDP for readiness
      // discovery and attachment; no test host, reload, or synthetic page exists.
      appProcess = spawn(executable, ['--remote-debugging-address=127.0.0.1', '--remote-debugging-port=0'], {
        env: {
          ...process.env,
          FABUSHI_APP_DATA: appDataDir,
          SAND_USER_DATA_DIR: appDataDir,
          OBF_SOURCE_SHA: sourceSha,
        },
        stdio: ['ignore', 'pipe', 'pipe'],
      });
      appProcess.stdout?.on('data', (chunk) => captureRuntimeLog('app-stdout', String(chunk)));
      appProcess.stderr?.on('data', (chunk) => captureRuntimeLog('app-stderr', String(chunk)));
      const earlyExit = new Promise<never>((_, reject) => {
        appProcess?.once('exit', (code, signal) => {
          reject(new Error(`Signed candidate exited before renderer binding (code=${code ?? 'null'}, signal=${signal ?? 'null'}).`));
        });
      });

      // Shipping startup owns the 120s SAND-E0602 threshold. Poll the browser
      // target registry until the real packaged renderer URL exists, then attach
      // Playwright to that already-created renderer.
      const binding = await Promise.race([
        waitForPackagedRendererBinding(appDataDir),
        earlyExit,
      ]);
      cdpBrowser = binding.browser;
      const page = binding.page;
      pageForTrace = page;
      const attachPageDiagnostics = (target: Page) => {
        target.on('console', (message) => captureRuntimeLog('page-console', `${message.type()}: ${message.text()}`));
        target.on('pageerror', (error) => captureRuntimeLog('page-error', error.stack || error.message));
        target.on('crash', () => captureRuntimeLog('page-crash', 'renderer page crashed'));
        target.on('requestfailed', (request) => captureRuntimeLog(
          'request-failed',
          `${request.method()} ${request.url()} :: ${request.failure()?.errorText || 'unknown'}`,
        ));
      };
      attachPageDiagnostics(page);

      traceContext = page.context();
      await traceContext.tracing.start({
        screenshots: true,
        snapshots: true,
        sources: true,
      });
      traceStarted = true;
      await writeFile(path.join(evidenceRoot, 'startup.json'), JSON.stringify({
        sourceSha,
        pageUrl: page.url(),
        binding: binding.binding,
        launchMode: 'signed-production-process',
        cdpPort: binding.cdpPort,
        targets: binding.targets,
      }, null, 2));

      await page.emulateMedia({ colorScheme: 'dark', reducedMotion: 'reduce' });
      await completeBrowserLogin(page);
      await expect(page.getByTestId('messenger-workspace')).toHaveAttribute('data-agent-root-shell', 'true');
      await screenshot(page, '01-canonical-agent-root');

      for (const [name, description] of coworkers) {
        if (await peerByName(page, name).count()) continue;
        await createCoworker(page, name, description);
      }
      await screenshot(page, '02-multi-agent-roster');

      await performDirectHandoff(page);
      await screenshot(page, '03-direct-handoff');
      await performBroadcast(page);
      await screenshot(page, '04-broadcast');

      await openAgent(page, 'Research');
      const researchPrompt = 'Two-Agent isolation acceptance for Research. Reply briefly and include marker FABUSHI-RESEARCH-ONLY-7421.';
      const researchAssistantCount = await submitTurn(page, researchPrompt);

      await openAgent(page, 'Builder');
      const builderPrompt = 'Two-Agent isolation acceptance for Builder. Reply briefly and include marker FABUSHI-BUILDER-ONLY-5937.';
      const builderAssistantCount = await submitTurn(page, builderPrompt);

      await openAgent(page, 'Research');
      const researchTurn = await waitForCompletedTurn(page, researchPrompt, researchAssistantCount);
      await expect(researchTurn).toContainText('FABUSHI-RESEARCH-ONLY-7421');
      await expect(page.getByTestId('message-list')).not.toContainText('FABUSHI-BUILDER-ONLY-5937');

      await openAgent(page, 'Builder');
      const builderTurn = await waitForCompletedTurn(page, builderPrompt, builderAssistantCount);
      await expect(builderTurn).toContainText('FABUSHI-BUILDER-ONLY-5937');
      await expect(page.getByTestId('message-list')).not.toContainText('FABUSHI-RESEARCH-ONLY-7421');
      await screenshot(page, '05-two-agent-isolation');

      await openAgent(page, 'Chief');
      await installLifecycleCapture(page);
      const lifecyclePrompt = 'Lifecycle acceptance: analyze the signed candidate and finish with CANDIDATE-LIFECYCLE-OK.';
      const lifecycleAssistantCount = await submitTurn(page, lifecyclePrompt);
      const lifecycleTurn = await waitForCompletedTurn(page, lifecyclePrompt, lifecycleAssistantCount);
      await expect(lifecycleTurn).toContainText('CANDIDATE-LIFECYCLE-OK');
      const lifecycle = await page.evaluate(() => {
        const scope = window as typeof window & { __candidateLifecycle?: LifecycleSample[] };
        return scope.__candidateLifecycle ?? [];
      });
      const started = lifecycle.find((sample) => sample.type === 'operation.started');
      expect(started?.operationId, 'real lifecycle must emit operation.started').toBeTruthy();
      const operationId = started!.operationId!;
      const operationLifecycle = lifecycle.filter((sample) => sample.operationId === operationId);
      expect(
        operationLifecycle.some((sample) => sample.type === 'turn.state' && ['preparing', 'thinking', 'streaming', 'tool-running'].includes(sample.status)),
        'real lifecycle must expose an active Rust-owned turn state',
      ).toBe(true);
      const streamedResult = operationLifecycle
        .filter((sample) => sample.type === 'chat.delta' || sample.type === 'chat.message')
        .map((sample) => sample.text)
        .join('');
      expect(
        streamedResult.includes('CANDIDATE-LIFECYCLE-OK'),
        'real lifecycle must stream or emit the expected assistant result on the same operation',
      ).toBe(true);
      expect(
        operationLifecycle.some((sample) => sample.type === 'turn.state' && sample.status === 'completed'),
        'real lifecycle must emit the actor-owned completed turn state',
      ).toBe(true);
      expect(
        operationLifecycle.some((sample) => sample.type === 'operation.completed' && sample.status === 'completed'),
        'real lifecycle must emit operation.completed for the same operation',
      ).toBe(true);
      expect(
        operationLifecycle.some((sample) => sample.type === 'operation.failed'),
        'real lifecycle must not fail',
      ).toBe(false);
      const toolSteps = operationLifecycle.filter((sample) => sample.type === 'agent.step');
      if (toolSteps.length > 0) {
        expect(toolSteps.some((sample) => sample.status === 'completed')).toBe(true);
      }

      const rosterShape = await stableAvatarShape(peerByName(page, 'Chief'));
      const headerShape = await stableAvatarShape(page.getByTestId('grok-agent-header'));
      const transcriptShape = await stableAvatarShape(lifecycleTurn);
      expect(headerShape).toBe(rosterShape);
      expect(transcriptShape).toBe(rosterShape);
      await screenshot(page, '06-real-lifecycle-complete');

      await openAgent(page, 'Chief');
      await installLifecycleCapture(page);
      const cancellablePrompt = 'Write a detailed multi-section analysis of desktop Agent recovery behavior, including several concrete examples and edge cases.';
      await submitTurn(page, cancellablePrompt);
      const stopResponse = page.getByRole('button', { name: 'Stop response' });
      await expect(stopResponse).toBeVisible({ timeout: 20_000 });
      await stopResponse.click();
      await expect(stopResponse).toBeHidden({ timeout: 30_000 });
      const postCancelPrompt = 'After the cancelled turn, reply briefly with the words cancellation recovery confirmed.';
      const postCancelCount = await submitTurn(page, postCancelPrompt);
      const postCancelTurn = await waitForCompletedTurn(page, postCancelPrompt, postCancelCount);
      await expect(postCancelTurn).toContainText(/cancellation recovery confirmed/i);
      await screenshot(page, '07-stop-and-new-turn');

      const latencySamples: LatencySample[] = [];
      for (const [offset, prompt] of ordinaryLatencyPrompts.entries()) {
        latencySamples.push(await runLatencyProbe(page, offset + 1, prompt));
      }
      expect(latencySamples).toHaveLength(20);
      const latencySummary = {
        samples: latencySamples,
        p50: {
          localSubmitPaintMs: percentile(latencySamples.map((sample) => sample.localSubmitPaintMs), 0.50),
          acceptanceVisibilityMs: percentile(latencySamples.map((sample) => sample.acceptanceVisibilityMs), 0.50),
          firstOutputMs: percentile(latencySamples.map((sample) => sample.firstOutputMs), 0.50),
          firstTextMs: percentile(latencySamples.map((sample) => sample.firstTextMs), 0.50),
          completionMs: percentile(latencySamples.map((sample) => sample.completionMs), 0.50),
        },
        p95: {
          localSubmitPaintMs: percentile(latencySamples.map((sample) => sample.localSubmitPaintMs), 0.95),
          acceptanceVisibilityMs: percentile(latencySamples.map((sample) => sample.acceptanceVisibilityMs), 0.95),
          firstOutputMs: percentile(latencySamples.map((sample) => sample.firstOutputMs), 0.95),
          firstTextMs: percentile(latencySamples.map((sample) => sample.firstTextMs), 0.95),
          completionMs: percentile(latencySamples.map((sample) => sample.completionMs), 0.95),
        },
      };
      expect(latencySummary.p95.localSubmitPaintMs, 'PERF-001 packaged p95 local submit paint').toBeLessThanOrEqual(250);
      expect(latencySummary.p95.acceptanceVisibilityMs, 'PERF-002 packaged p95 acceptance visibility').toBeLessThanOrEqual(500);
      expect(latencySummary.p50.firstOutputMs, 'PERF-003 packaged p50 first output').toBeLessThanOrEqual(3_000);
      expect(latencySummary.p95.firstOutputMs, 'PERF-003 packaged p95 first output').toBeLessThanOrEqual(8_000);
      expect(latencySummary.p50.firstTextMs, 'packaged p50 first text must be recorded').toBeGreaterThanOrEqual(0);
      expect(latencySummary.p95.firstTextMs, 'packaged p95 first text must be recorded').toBeGreaterThanOrEqual(latencySummary.p50.firstTextMs);
      expect(latencySummary.p50.completionMs, 'packaged p50 completion must be recorded').toBeGreaterThanOrEqual(latencySummary.p50.firstTextMs);
      expect(latencySummary.p95.completionMs, 'packaged p95 completion must be recorded').toBeGreaterThanOrEqual(latencySummary.p95.firstTextMs);
      await writeFile(path.join(evidenceRoot, 'timings.json'), JSON.stringify(latencySummary, null, 2));

      const connectorEvidence = await capturePluginsEvidence(page);
      await writeFile(path.join(evidenceRoot, 'connectors.json'), JSON.stringify({
        sourceSha,
        capturedAt: Date.now(),
        ...connectorEvidence,
      }, null, 2));

      await writeFile(path.join(evidenceRoot, 'lifecycle.json'), JSON.stringify(lifecycle, null, 2));
      await writeFile(path.join(evidenceRoot, 'runtime.log'), runtimeLogs.map((row) => `[${new Date(row.at).toISOString()}] ${row.source}: ${row.text}`).join('\n'));
      await writeFile(path.join(evidenceRoot, 'candidate.json'), JSON.stringify({
        sourceSha,
        expectedSourceSha,
        executable,
        launchMode: 'signed-production-process-cdp',
        acceptance: {
          directHandoff: true,
          broadcast: true,
          twoAgentIsolation: true,
          realLifecycle: true,
          packagedLatency: true,
          pluginsSurface: true,
          lowPowerAvatarCutover: true,
        },
      }, null, 2));
      acceptanceCompleted = true;
    } finally {
      await writeFile(
        path.join(evidenceRoot, 'runtime.log'),
        runtimeLogs.map((row) => `[${new Date(row.at).toISOString()}] ${row.source}: ${row.text}`).join('\n'),
      ).catch(() => undefined);
      if (!acceptanceCompleted && pageForTrace) {
        await pageForTrace.screenshot({
          path: path.join(evidenceRoot, 'screenshots', '00-failure-state.png'),
          fullPage: true,
        }).catch(() => undefined);
        await writeFile(path.join(evidenceRoot, 'failure.json'), JSON.stringify({
          sourceSha,
          expectedSourceSha,
          executable,
          url: pageForTrace.url(),
        }, null, 2)).catch(() => undefined);
      }
      if (traceStarted && traceContext) {
        await traceContext.tracing.stop({
          path: path.join(evidenceRoot, 'trace.zip'),
        }).catch((error) => {
          captureRuntimeLog('trace-error', error instanceof Error ? error.stack || error.message : String(error));
        });
      }
      if (cdpBrowser) {
        await cdpBrowser.close().catch(() => undefined);
      }
      if (appProcess && appProcess.exitCode == null && appProcess.signalCode == null) {
        appProcess.kill('SIGTERM');
        await withNodeDeadline(
          'Packaged Electron shutdown',
          10_000,
          new Promise<void>((resolve) => appProcess?.once('exit', () => resolve())),
        ).catch(() => {
          appProcess?.kill('SIGKILL');
        });
      }
    }
  });
});
