import { _electron as electron, chromium, expect, test, type Browser, type ElectronApplication, type Locator, type Page } from '@playwright/test';
import { appendFile, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

const realAcceptance = process.env.OBF_REAL_ACCEPTANCE === '1';
const executable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || '';
const sourceSha = process.env.OBF_SOURCE_SHA?.trim() || '';
const expectedSourceSha = process.env.OBF_EXPECTED_SOURCE_SHA?.trim() || sourceSha;
const evidenceRoot = process.env.OBF_EVIDENCE_DIR?.trim()
  || path.join(tmpdir(), `fabushi-openbot-acceptance-${sourceSha.slice(0, 12) || 'unknown'}`);

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

async function waitForPackagedRendererBinding(
  app: ElectronApplication,
  initialPage: Page,
  appDataDir: string,
): Promise<{ page: Page; browser: Browser | null; binding: 'electron' | 'cdp' }> {
  const deadline = Date.now() + 35_000;
  let cdpBrowser: Browser | null = null;
  let lastMainState: { url: string; loading: boolean; title: string } | null = null;
  let lastPageUrls: string[] = [];
  let lastCdpError = '';

  while (Date.now() < deadline) {
    lastMainState = await app.evaluate(({ BrowserWindow }) => {
      const win = BrowserWindow.getAllWindows()[0];
      return win
        ? {
            url: win.webContents.getURL(),
            loading: win.webContents.isLoadingMainFrame(),
            title: win.getTitle(),
          }
        : { url: '', loading: true, title: '' };
    });

    const electronPages = app.windows();
    lastPageUrls = electronPages.map((candidate) => candidate.url());
    const electronBound = electronPages.find((candidate) => candidate.url().startsWith('app://bundle/'));
    if (electronBound) return { page: electronBound, browser: null, binding: 'electron' };
    if (initialPage.url().startsWith('app://bundle/')) {
      return { page: initialPage, browser: null, binding: 'electron' };
    }

    // Playwright's Electron Page wrapper can miss a custom-protocol navigation
    // that completed before attachment even though BrowserWindow.webContents is
    // already on app://bundle. _electron.launch enables a Chromium remote
    // debugging endpoint and writes DevToolsActivePort under userData. Bind the
    // *same packaged renderer target* over that endpoint instead of reloading,
    // replacing the URL, or falling back to a test host.
    if (!cdpBrowser) {
      try {
        const activePort = await readFile(path.join(appDataDir, 'DevToolsActivePort'), 'utf8');
        const [portText] = activePort.trim().split(/\r?\n/u);
        const port = Number(portText);
        if (Number.isInteger(port) && port > 0 && port <= 65_535) {
          cdpBrowser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
        }
      } catch (cause) {
        lastCdpError = cause instanceof Error ? cause.message : String(cause);
      }
    }
    if (cdpBrowser) {
      const cdpPages = cdpBrowser.contexts().flatMap((context) => context.pages());
      lastPageUrls = [...lastPageUrls, ...cdpPages.map((candidate) => candidate.url())];
      const cdpBound = cdpPages.find((candidate) => candidate.url().startsWith('app://bundle/'));
      if (cdpBound) return { page: cdpBound, browser: cdpBrowser, binding: 'cdp' };
    }

    await new Promise((resolve) => setTimeout(resolve, 250));
  }

  throw new Error(
    `Playwright did not bind the packaged BrowserWindow. main=${JSON.stringify(lastMainState)} pages=${JSON.stringify(lastPageUrls)} cdp=${lastCdpError || 'no target'}`,
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
  const readPhase = async (): Promise<LoginPhase> => {
    if (await page.getByTestId('messenger-workspace').count()) {
      const hydrated = await page.getByTestId('messenger-workspace').getAttribute('data-initial-host-hydrated').catch(() => null);
      if (hydrated === 'true') return 'ready';
    }
    if (await page.getByTestId('onboarding-gate').count()) return 'onboarding';
    if (await page.getByTestId('browser-login-waiting').count()) return 'browser-waiting';
    if (await page.getByTestId('login-gate').count()) return 'login';
    return 'waiting';
  };

  await withNodeDeadline(
    'Packaged renderer did not mount desktop-shell',
    35_000,
    expect(page.getByTestId('desktop-shell')).toBeVisible({ timeout: 30_000 }),
  );

  for (let attempt = 0; attempt < 12; attempt += 1) {
    await expect.poll(readPhase, { timeout: 15_000 }).not.toBe('waiting');
    const phase = await readPhase();
    if (phase === 'onboarding') {
      await page.getByTestId('onboarding-next').click();
      continue;
    }
    if (phase === 'login') {
      const start = page.getByTestId('browser-login-start');
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
    if (phase === 'ready') return;
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

async function submitTurn(page: Page, prompt: string): Promise<number> {
  const peerMessages = page.locator('[data-agent-message-role="peer"]');
  const previousAssistantCount = await peerMessages.count();
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
  const peerMessages = page.locator('[data-agent-message-role="peer"]');
  await expect.poll(
    async () => peerMessages.count(),
    { timeout: 180_000, message: 'A new final assistant message must be committed after the submitted turn.' },
  ).toBeGreaterThan(previousAssistantCount);
  const turn = peerMessages.last();
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
  await research.getByRole('checkbox').check();
  await network.getByRole('textbox').fill('Research: verify the candidate handoff path and report one concise fact.');
  await network.getByRole('button', { name: /Handoff to Research/ }).click();
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
    await checkbox.check();
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

test.describe('signed candidate packaged acceptance', () => {
  test.describe.configure({ retries: 0 });
  test.skip(!realAcceptance, 'Set OBF_REAL_ACCEPTANCE=1 to run signed packaged acceptance.');

  test('exact candidate covers handoff, broadcast, two-Agent isolation and real lifecycle', async () => {
    test.setTimeout(12 * 60_000);
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
    let app: ElectronApplication | null = null;
    let cdpBrowser: Browser | null = null;
    let pageForTrace: Page | null = null;
    let acceptanceCompleted = false;

    try {
      app = await electron.launch({
        executablePath: executable,
        args: [],
        env: {
          ...process.env,
          FABUSHI_APP_DATA: appDataDir,
          OBF_SOURCE_SHA: sourceSha,
        },
        recordVideo: { dir: path.join(evidenceRoot, 'video'), size: { width: 1671, height: 937 } },
      });
      let page = await app.firstWindow();
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
      app.process().stdout?.on('data', (chunk) => captureRuntimeLog('app-stdout', String(chunk)));
      app.process().stderr?.on('data', (chunk) => captureRuntimeLog('app-stderr', String(chunk)));

      const startupWindows = await withNodeDeadline(
        'Inspect packaged BrowserWindow state',
        5_000,
        app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows().map((win) => ({
          title: win.getTitle(),
          url: win.webContents.getURL(),
          visible: win.isVisible(),
          loading: win.webContents.isLoading(),
        }))),
      );
      const initialPageUrl = page.url();
      const binding = await waitForPackagedRendererBinding(app, page, appDataDir);
      cdpBrowser = binding.browser;
      if (binding.page !== page) {
        page = binding.page;
        attachPageDiagnostics(page);
      }
      pageForTrace = page;
      await writeFile(path.join(evidenceRoot, 'startup.json'), JSON.stringify({
        sourceSha,
        initialPageUrl,
        pageUrl: page.url(),
        binding: binding.binding,
        windows: startupWindows,
      }, null, 2));

      await app.evaluate(({ BrowserWindow }) => {
        const win = BrowserWindow.getAllWindows()[0];
        if (!win) throw new Error('Fabushi BrowserWindow missing');
        win.setContentSize(1671, 937, false);
        win.center();
      });
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

      await writeFile(path.join(evidenceRoot, 'lifecycle.json'), JSON.stringify(lifecycle, null, 2));
      await writeFile(path.join(evidenceRoot, 'runtime.log'), runtimeLogs.map((row) => `[${new Date(row.at).toISOString()}] ${row.source}: ${row.text}`).join('\n'));
      await writeFile(path.join(evidenceRoot, 'candidate.json'), JSON.stringify({
        sourceSha,
        expectedSourceSha,
        executable,
        acceptance: {
          directHandoff: true,
          broadcast: true,
          twoAgentIsolation: true,
          realLifecycle: true,
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
      if (app) {
        try {
          await withNodeDeadline('Packaged Electron shutdown', 10_000, app.close());
        } catch {
          app.process().kill('SIGKILL');
        }
      }
      if (cdpBrowser) {
        await cdpBrowser.close().catch(() => undefined);
      }
    }
  });
});
