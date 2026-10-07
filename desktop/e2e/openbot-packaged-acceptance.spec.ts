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
  readonly type: 'turn.accepted' | 'turn.output' | 'turn.text' | 'agent.step' | 'turn.completed' | 'turn.failed';
  readonly entryId?: string;
  readonly status: string;
  readonly text: string;
};

type RuntimeLog = {
  readonly at: number;
  readonly source: string;
  readonly text: string;
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
    .locator('aside[aria-label="Agents"]')
    .getByRole('button', { name, exact: true })
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

  const hostHydrated = async (): Promise<boolean> => {
    const connected = page.getByRole('status', { name: 'Connected' });
    if (!(await connected.first().isVisible().catch(() => false))) return false;
    const sidebar = page.locator('aside[aria-label="Agents"]');
    if (!(await sidebar.isVisible().catch(() => false))) return false;
    return await sidebar.locator('button.sand-agent-item').count() > 0;
  };

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
          'Canonical ProductionRenderer mounted and authenticated, but the signed candidate never hydrated a usable Coordinator-backed Agent roster.',
        );
      }
      return;
    }
  }

  throw new Error('Packaged Fabushi did not reach the canonical Agent workspace');
}

async function createCoworker(page: Page, name: string, description: string): Promise<void> {
  const sidebar = page.locator('aside[aria-label="Agents"]');
  const heading = page.locator('#sand-conversation-heading');

  // The signed product can already own one empty "New chat" after first-run
  // bootstrap. Reuse that canonical empty Agent when it is selected. Otherwise
  // exercise the shipping New -> Create agent identity picker and create the
  // named Agent with its description before the first conversation starts.
  if ((await heading.textContent().catch(() => null))?.trim() !== 'New chat') {
    const create = page.getByRole('button', { name: 'New', exact: true });
    await expect(create).toBeVisible({ timeout: 20_000 });
    await create.click();

    const dialog = page.getByRole('dialog', { name: 'Create agent' });
    await expect(dialog).toBeVisible({ timeout: 20_000 });
    await dialog.getByLabel('Name').fill(name);
    if (description.trim().length > 0) {
      await dialog.getByLabel('Description').fill(description);
    }
    await dialog.getByRole('button', { name: 'Create', exact: true }).click();
    await expect(dialog).toBeHidden({ timeout: 20_000 });
    await expect(peerByName(page, name)).toBeVisible({ timeout: 20_000 });
    await expect(heading).toHaveText(name, { timeout: 20_000 });
    return;
  }

  const activeRow = sidebar.locator('button.sand-agent-item[aria-current="page"]').first();
  await expect(activeRow).toBeVisible({ timeout: 20_000 });
  await expect(heading).toHaveText('New chat', { timeout: 20_000 });
  await activeRow.dblclick();

  const rename = sidebar.getByRole('textbox', { name: 'Rename agent' });
  await expect(rename).toBeVisible({ timeout: 10_000 });
  await rename.fill(name);
  await rename.press('Enter');
  await expect(peerByName(page, name)).toBeVisible({ timeout: 20_000 });
  await expect(heading).toHaveText(name, { timeout: 20_000 });
}

async function openAgent(page: Page, name: string): Promise<void> {
  const peer = peerByName(page, name);
  await expect(peer).toBeVisible({ timeout: 20_000 });
  await peer.click();
  await expect(page.getByRole('textbox', { name: 'Prompt' })).toBeVisible();
  await expect(page.locator('#sand-conversation-heading')).toHaveText(name);
}

function completedAssistantTurns(page: Page): Locator {
  // The shipping transcript has two canonical assistant completion shapes:
  // ordinary assistant message articles, and SendMessage text-card articles
  // whose inner message group owns data-role=assistant. Keep both fail-closed:
  // a streaming/pending or failed article cannot satisfy packaged acceptance.
  return page.locator([
    '[aria-label="Conversation transcript"] [role="article"][data-role="assistant"]:not([aria-busy="true"]):not([data-failed="true"])',
    '[aria-label="Conversation transcript"] [role="article"]:not([aria-busy="true"]):not([data-failed="true"]):has(.sand-message[data-role="assistant"])',
  ].join(', '));
}

async function submitTurn(page: Page, prompt: string): Promise<number> {
  const previousAssistantCount = await completedAssistantTurns(page).count();
  const input = page.getByRole('textbox', { name: 'Prompt' });
  await input.fill(prompt);
  await page.getByRole('button', { name: 'Send message' }).click();
  await expect(
    page
      .locator('[aria-label="Conversation transcript"] [role="article"][data-role="user"]')
      .filter({ hasText: prompt })
      .last(),
  ).toBeVisible({ timeout: 5_000 });
  return previousAssistantCount;
}

type SubmitTiming = {
  readonly previousAssistantCount: number;
  readonly sendGestureAt: number;
  readonly localSubmitPaintMs: number;
};

async function submitTimedTurn(page: Page, prompt: string): Promise<SubmitTiming> {
  const previousAssistantCount = await completedAssistantTurns(page).count();
  const input = page.getByRole('textbox', { name: 'Prompt' });
  await input.fill(prompt);
  const send = page.getByRole('button', { name: 'Send message' });
  await expect(send).toBeEnabled();

  // PERF-001 is explicitly measured from the real renderer send gesture, not
  // from Playwright setup work such as counting turns, filling the composer,
  // or auto-waiting before the click. Observe the canonical user row in the
  // renderer and cross a paint boundary before recording the local paint time.
  await page.evaluate((expectedPrompt) => {
    const scope = window as typeof window & {
      __candidateSubmitPaintProbe?: { gestureAt: number | null; paintedAt: number | null };
      __candidateSubmitPaintObserver?: MutationObserver;
    };
    scope.__candidateSubmitPaintObserver?.disconnect();

    const transcript = document.querySelector<HTMLElement>('[aria-label="Conversation transcript"]');
    const button = [...document.querySelectorAll<HTMLButtonElement>('button')]
      .find((candidate) => candidate.getAttribute('aria-label') === 'Send message');
    if (transcript == null || button == null) {
      throw new Error('Canonical transcript/send button is unavailable for PERF-001.');
    }

    const probe = { gestureAt: null as number | null, paintedAt: null as number | null };
    scope.__candidateSubmitPaintProbe = probe;
    let paintScheduled = false;
    const submittedTurnExists = () => [...transcript.querySelectorAll<HTMLElement>('[role="article"][data-role="user"]')]
      .some((row) => (row.innerText || '').includes(expectedPrompt));
    const schedulePaintSample = () => {
      if (paintScheduled || probe.gestureAt == null || !submittedTurnExists()) return;
      paintScheduled = true;
      // The first callback is the frame in which the committed DOM can render;
      // the second callback runs after that frame's paint opportunity.
      requestAnimationFrame(() => requestAnimationFrame(() => {
        probe.paintedAt = Date.now();
        scope.__candidateSubmitPaintObserver?.disconnect();
      }));
    };

    const observer = new MutationObserver(schedulePaintSample);
    observer.observe(transcript, { subtree: true, childList: true, characterData: true });
    scope.__candidateSubmitPaintObserver = observer;
    button.addEventListener('click', () => {
      probe.gestureAt = Date.now();
      schedulePaintSample();
    }, { capture: true, once: true });
  }, prompt);

  await send.click();
  await expect(
    page
      .locator('[aria-label="Conversation transcript"] [role="article"][data-role="user"]')
      .filter({ hasText: prompt })
      .last(),
  ).toBeVisible({ timeout: 5_000 });

  await expect.poll(
    async () => page.evaluate(() => {
      const scope = window as typeof window & {
        __candidateSubmitPaintProbe?: { gestureAt: number | null; paintedAt: number | null };
      };
      const probe = scope.__candidateSubmitPaintProbe;
      return probe?.gestureAt != null && probe?.paintedAt != null;
    }),
    { timeout: 5_000, message: 'PERF-001 must observe the canonical user bubble across a paint boundary.' },
  ).toBe(true);

  const probe = await page.evaluate(() => {
    const scope = window as typeof window & {
      __candidateSubmitPaintProbe?: { gestureAt: number | null; paintedAt: number | null };
      __candidateSubmitPaintObserver?: MutationObserver;
    };
    scope.__candidateSubmitPaintObserver?.disconnect();
    return scope.__candidateSubmitPaintProbe ?? null;
  });
  if (probe?.gestureAt == null || probe.paintedAt == null) {
    throw new Error('PERF-001 submit paint probe did not settle.');
  }
  return {
    previousAssistantCount,
    sendGestureAt: probe.gestureAt,
    localSubmitPaintMs: Math.max(0, probe.paintedAt - probe.gestureAt),
  };
}

async function waitForCompletedTurn(
  page: Page,
  prompt: string,
  previousAssistantCount: number,
  expectedText?: string,
): Promise<Locator> {
  await expect(
    page
      .locator('[aria-label="Conversation transcript"] [role="article"][data-role="user"]')
      .filter({ hasText: prompt })
      .last(),
  ).toBeVisible({ timeout: 10_000 });
  const assistantTurns = completedAssistantTurns(page);
  await expect.poll(
    async () => assistantTurns.count(),
    { timeout: 180_000, message: 'A new canonical completed assistant turn must be committed after the submitted turn.' },
  ).toBeGreaterThan(previousAssistantCount);

  if (expectedText != null) {
    // Routed Agent turns may legitimately commit an assistant preamble before
    // one or more tool-call/result entries and the terminal assistant segment.
    // Do not mistake that intermediate completed segment for the turn terminal:
    // keep the semantic gate fail-closed on the expected terminal marker.
    const terminal = assistantTurns.filter({ hasText: expectedText }).last();
    await expect(terminal).toContainText(expectedText, { timeout: 180_000 });
    return terminal;
  }

  const turn = assistantTurns.last();
  await expect(turn).toBeVisible({ timeout: 10_000 });
  return turn;
}

async function screenshot(page: Page, name: string): Promise<void> {
  await page.screenshot({ path: path.join(evidenceRoot, 'screenshots', `${name}.png`), fullPage: true });
}

async function installLifecycleCapture(page: Page): Promise<void> {
  await page.evaluate(() => {
    const scope = window as typeof window & {
      __candidateLifecycle?: LifecycleSample[];
      __candidateLifecycleObserver?: MutationObserver;
    };
    scope.__candidateLifecycleObserver?.disconnect();
    scope.__candidateLifecycle = [];

    const transcript = document.querySelector<HTMLElement>('[aria-label="Conversation transcript"]');
    if (transcript == null) throw new Error('Canonical conversation transcript is unavailable.');

    const assistantState = new Map<string, { busy: boolean; failed: boolean; text: string }>();
    const toolState = new Map<string, string>();
    let acceptedVisible = false;
    const capture = () => {
      const at = Date.now();
      const typingVisible = document.querySelector('.sand-typing-indicator') != null;
      if (typingVisible && !acceptedVisible) {
        scope.__candidateLifecycle?.push({
          at,
          type: 'turn.accepted',
          status: 'running',
          text: '',
        });
      }
      acceptedVisible = typingVisible;

      for (const row of transcript.querySelectorAll<HTMLElement>('[role="article"][data-entry-id]')) {
        const assistantSurface = row.matches('[data-role="assistant"]')
          ? row
          : row.querySelector<HTMLElement>('[data-role="assistant"]');
        if (assistantSurface == null) continue;
        const entryId = row.dataset.entryId;
        if (!entryId) continue;
        const busy = row.getAttribute('aria-busy') === 'true'
          || assistantSurface.getAttribute('aria-busy') === 'true';
        const failed = row.getAttribute('data-failed') === 'true'
          || assistantSurface.getAttribute('data-failed') === 'true';
        const text = (row.innerText || '').trim();
        const previous = assistantState.get(entryId);

        if (previous == null) {
          scope.__candidateLifecycle?.push({
            at,
            type: 'turn.output',
            entryId,
            status: failed ? 'failed' : busy ? 'streaming' : 'completed',
            text: '',
          });
        }
        if (text.length > 0 && previous?.text !== text) {
          scope.__candidateLifecycle?.push({
            at,
            type: 'turn.text',
            entryId,
            status: busy ? 'streaming' : failed ? 'failed' : 'completed',
            text,
          });
        }
        if (failed && previous?.failed !== true) {
          scope.__candidateLifecycle?.push({
            at,
            type: 'turn.failed',
            entryId,
            status: 'failed',
            text,
          });
        } else if (!busy && !failed && (previous == null || previous.busy)) {
          scope.__candidateLifecycle?.push({
            at,
            type: 'turn.completed',
            entryId,
            status: 'completed',
            text,
          });
        }
        assistantState.set(entryId, { busy, failed, text });
      }

      for (const tool of transcript.querySelectorAll<HTMLElement>('.sand-outline-item[data-kind="tool-call"]')) {
        const entryId = tool.closest<HTMLElement>('[data-entry-id]')?.dataset.entryId;
        const status = tool.dataset.status || 'unknown';
        const text = (tool.innerText || '').trim();
        const key = `${entryId || 'unscoped'}:${text}`;
        if (toolState.get(key) === status) continue;
        toolState.set(key, status);
        scope.__candidateLifecycle?.push({
          at,
          type: 'agent.step',
          ...(entryId ? { entryId } : {}),
          status,
          text,
        });
      }
    };

    capture();
    const observer = new MutationObserver(capture);
    observer.observe(transcript, {
      subtree: true,
      childList: true,
      characterData: true,
      attributes: true,
      attributeFilter: ['aria-busy', 'data-failed', 'data-status', 'data-entry-id'],
    });
    scope.__candidateLifecycleObserver = observer;
  });
}

async function openAgentNetworkReference(page: Page): Promise<void> {
  const trigger = page.getByRole('button', { name: 'Agent network' });
  await expect(trigger).toBeVisible({ timeout: 20_000 });
  await trigger.click();
  await expect(page.getByRole('heading', { name: 'Org chart', exact: true })).toBeVisible({ timeout: 20_000 });
  await expect(page.getByText(/Solid links are real agent-to-agent message history/)).toBeVisible();
}

function avatarFor(locator: Locator): Locator {
  return locator.locator('.sand-agent-avatar[data-avatar-shape]').first();
}

async function stableAvatarShape(locator: Locator): Promise<string> {
  const avatar = avatarFor(locator);
  await expect(avatar).toBeVisible();
  const shape = await avatar.getAttribute('data-avatar-shape');
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
  readonly entryId: string;
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
  const { previousAssistantCount, sendGestureAt, localSubmitPaintMs } = await submitTimedTurn(page, prompt);
  const turn = await waitForCompletedTurn(page, prompt, previousAssistantCount);
  expect(((await turn.textContent()) ?? '').trim().length, `latency turn ${index} must render ordinary assistant text`).toBeGreaterThan(0);

  const entryId = await turn.getAttribute('data-entry-id');
  expect(entryId, `latency turn ${index} must expose a canonical transcript entry id`).toBeTruthy();
  const lifecycle = await page.evaluate(() => {
    const scope = window as typeof window & { __candidateLifecycle?: LifecycleSample[] };
    return scope.__candidateLifecycle ?? [];
  });
  const entryLifecycle = lifecycle.filter((sample) => sample.entryId === entryId);
  const accepted = lifecycle.find((sample) => sample.type === 'turn.accepted' && sample.at >= sendGestureAt)
    ?? entryLifecycle.find((sample) => sample.type === 'turn.output');
  const firstOutput = entryLifecycle.find((sample) => sample.type === 'turn.output');
  const firstText = entryLifecycle.find((sample) => sample.type === 'turn.text' && sample.text.length > 0);
  const completed = entryLifecycle.find((sample) => sample.type === 'turn.completed');
  expect(accepted, `latency turn ${index} must expose canonical accepted/running visibility`).toBeTruthy();
  expect(firstOutput, `latency turn ${index} must expose first output`).toBeTruthy();
  expect(firstText, `latency turn ${index} must expose first text`).toBeTruthy();
  expect(completed, `latency turn ${index} must complete`).toBeTruthy();
  return {
    prompt,
    localSubmitPaintMs,
    // Host acceptance occurs after the send gesture. Measuring from the earlier
    // gesture is a conservative upper bound for PERF-002 and does not weaken it.
    acceptanceVisibilityMs: accepted!.at - sendGestureAt,
    firstOutputMs: firstOutput!.at - sendGestureAt,
    firstTextMs: firstText!.at - sendGestureAt,
    completionMs: completed!.at - sendGestureAt,
    entryId: entryId!,
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

  test('exact candidate covers Agent network, two-Agent isolation, lifecycle, latency and plugins', async () => {
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

      await openAgentNetworkReference(page);
      await screenshot(page, '03-agent-network-reference');
      await page.getByRole('button', { name: 'Close org chart' }).click();
      await expect(page.getByRole('heading', { name: 'Org chart', exact: true })).toHaveCount(0);

      await openAgent(page, 'Research');
      const researchPrompt = 'Two-Agent isolation acceptance for Research. Your entire final answer must be exactly FABUSHI-RESEARCH-ONLY-7421. Copy those characters verbatim; do not explain, describe, quote, or paraphrase the marker.';
      const researchAssistantCount = await submitTurn(page, researchPrompt);

      await openAgent(page, 'Builder');
      const builderPrompt = 'Two-Agent isolation acceptance for Builder. Your entire final answer must be exactly FABUSHI-BUILDER-ONLY-5937. Copy those characters verbatim; do not explain, describe, quote, or paraphrase the marker.';
      const builderAssistantCount = await submitTurn(page, builderPrompt);

      await openAgent(page, 'Research');
      const researchTurn = await waitForCompletedTurn(page, researchPrompt, researchAssistantCount);
      await expect(researchTurn).toContainText('FABUSHI-RESEARCH-ONLY-7421');
      await expect(page.getByRole('log', { name: 'Conversation transcript' })).not.toContainText('FABUSHI-BUILDER-ONLY-5937');

      await openAgent(page, 'Builder');
      const builderTurn = await waitForCompletedTurn(page, builderPrompt, builderAssistantCount);
      await expect(builderTurn).toContainText('FABUSHI-BUILDER-ONLY-5937');
      await expect(page.getByRole('log', { name: 'Conversation transcript' })).not.toContainText('FABUSHI-RESEARCH-ONLY-7421');
      await screenshot(page, '04-two-agent-isolation');

      await openAgent(page, 'Chief');
      await installLifecycleCapture(page);
      const lifecyclePrompt = 'Lifecycle acceptance. Do not use tools. Your entire final answer must be exactly CANDIDATE-LIFECYCLE-OK. Copy those characters verbatim; do not explain, describe, quote, or paraphrase the marker.';
      const lifecycleSubmittedAt = Date.now();
      const lifecycleAssistantCount = await submitTurn(page, lifecyclePrompt);
      // Canonical Agent sends intentionally remain owned by the addressed
      // transcript even if roster hydration changes the visible selection
      // while Host acceptance is starting. Research/Builder acceptance above
      // already verifies this off-screen persistence contract by reopening the
      // addressed Agent. Do the same for Chief before observing lifecycle
      // completion so the gate follows the canonical addressed transcript
      // rather than whichever row happens to be selected after hydration.
      await openAgent(page, 'Chief');
      await expect(
        page
          .locator('[aria-label="Conversation transcript"] [role="article"][data-role="user"]')
          .filter({ hasText: lifecyclePrompt })
          .last(),
      ).toBeVisible({ timeout: 10_000 });
      const lifecycleTurn = await waitForCompletedTurn(
        page,
        lifecyclePrompt,
        lifecycleAssistantCount,
        'CANDIDATE-LIFECYCLE-OK',
      );
      const lifecycle = await page.evaluate(() => {
        const scope = window as typeof window & { __candidateLifecycle?: LifecycleSample[] };
        return scope.__candidateLifecycle ?? [];
      });
      const lifecycleEntryId = await lifecycleTurn.getAttribute('data-entry-id');
      expect(lifecycleEntryId, 'real lifecycle must expose a canonical transcript entry id').toBeTruthy();
      const operationLifecycle = lifecycle.filter((sample) => sample.entryId === lifecycleEntryId);
      const accepted = lifecycle.find((sample) => sample.type === 'turn.accepted' && sample.at >= lifecycleSubmittedAt)
        ?? operationLifecycle.find((sample) => sample.type === 'turn.output');
      expect(accepted, 'real lifecycle must expose accepted/running state before terminal completion').toBeTruthy();
      expect(
        operationLifecycle.some((sample) => sample.type === 'turn.output'),
        'real lifecycle must expose canonical assistant output',
      ).toBe(true);
      expect(
        operationLifecycle.some((sample) => sample.type === 'turn.text' && sample.text.includes('CANDIDATE-LIFECYCLE-OK')),
        'real lifecycle must stream/render the expected assistant result on the same canonical transcript entry',
      ).toBe(true);
      expect(
        operationLifecycle.some((sample) => sample.type === 'turn.completed' && sample.status === 'completed'),
        'real lifecycle must settle the same transcript entry as completed',
      ).toBe(true);
      expect(
        operationLifecycle.some((sample) => sample.type === 'turn.failed'),
        'real lifecycle must not fail',
      ).toBe(false);
      const toolSteps = operationLifecycle.filter((sample) => sample.type === 'agent.step');
      if (toolSteps.length > 0) {
        expect(toolSteps.some((sample) => ['done', 'completed'].includes(sample.status))).toBe(true);
      }

      const rosterShape = await stableAvatarShape(peerByName(page, 'Chief'));
      const headerShape = await stableAvatarShape(page.locator('.sand-chat-header'));
      expect(headerShape).toBe(rosterShape);
      await screenshot(page, '05-real-lifecycle-complete');

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
      await screenshot(page, '06-stop-and-new-turn');

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
      // Persist the raw timing evidence before enforcing thresholds so a failed
      // performance gate remains diagnosable without weakening the contract.
      await writeFile(path.join(evidenceRoot, 'timings.json'), JSON.stringify(latencySummary, null, 2));
      expect(latencySummary.p95.localSubmitPaintMs, 'PERF-001 packaged p95 local submit paint').toBeLessThanOrEqual(250);
      expect(latencySummary.p95.acceptanceVisibilityMs, 'PERF-002 packaged p95 acceptance visibility').toBeLessThanOrEqual(500);
      // PERF-003 p50 is captured in timings.json as a trend metric; only p95 is release-gated.
      expect(latencySummary.p95.firstOutputMs, 'PERF-003 packaged p95 first output').toBeLessThanOrEqual(8_000);
      expect(latencySummary.p50.firstTextMs, 'packaged p50 first text must be recorded').toBeGreaterThanOrEqual(0);
      expect(latencySummary.p95.firstTextMs, 'packaged p95 first text must be recorded').toBeGreaterThanOrEqual(latencySummary.p50.firstTextMs);
      expect(latencySummary.p50.completionMs, 'packaged p50 completion must be recorded').toBeGreaterThanOrEqual(latencySummary.p50.firstTextMs);
      expect(latencySummary.p95.completionMs, 'packaged p95 completion must be recorded').toBeGreaterThanOrEqual(latencySummary.p95.firstTextMs);

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
          agentNetworkReference: true,
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
