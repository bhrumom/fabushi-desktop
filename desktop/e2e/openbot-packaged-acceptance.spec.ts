import { _electron as electron, expect, test, type ElectronApplication, type Locator, type Page } from '@playwright/test';
import { mkdir, rm, writeFile } from 'node:fs/promises';
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

  await expect(page.getByTestId('desktop-shell')).toBeVisible({ timeout: 30_000 });

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

async function submitTurn(page: Page, prompt: string): Promise<void> {
  const input = page.getByTestId('messenger-input');
  await input.fill(prompt);
  await page.getByTestId('messenger-send').click();
  await expect(page.locator('[data-agent-message-role="me"]').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 5_000 });
}

async function waitForCompletedTurn(page: Page, prompt: string): Promise<Locator> {
  await expect(page.locator('[data-agent-message-role="me"]').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 10_000 });
  const turn = page.getByTestId('mahayana-assistant-turn').last();
  await expect(turn).toBeVisible({ timeout: 30_000 });
  await expect(turn).toHaveAttribute('data-status', 'completed', { timeout: 180_000 });
  return turn;
}

async function screenshot(page: Page, name: string): Promise<void> {
  await page.screenshot({ path: path.join(evidenceRoot, 'screenshots', `${name}.png`), fullPage: true });
}

async function installLifecycleCapture(page: Page): Promise<void> {
  await page.evaluate(() => {
    const scope = window as typeof window & {
      __candidateLifecycle?: LifecycleSample[];
      __candidateObserver?: MutationObserver;
    };
    scope.__candidateLifecycle = [];
    const sample = () => {
      const turn = document.querySelector<HTMLElement>('[data-testid="mahayana-assistant-turn"]');
      if (turn) {
        const status = turn.dataset.status || turn.querySelector<HTMLElement>('[data-turn-status]')?.dataset.turnStatus || 'unknown';
        const text = (turn.innerText || '').trim();
        const last = scope.__candidateLifecycle?.at(-1);
        if (!last || last.status !== status || last.text !== text) {
          scope.__candidateLifecycle?.push({ at: Date.now(), status, text });
        }
      }
      for (const step of document.querySelectorAll<HTMLElement>('[data-testid="agent-step"]')) {
        const status = step.dataset.status || 'unknown';
        const text = (step.innerText || '').trim();
        const last = scope.__candidateLifecycle?.at(-1);
        if (!last || last.status !== status || last.text !== text) {
          scope.__candidateLifecycle?.push({ at: Date.now(), status, text });
        }
      }
    };
    sample();
    const observer = new MutationObserver(sample);
    observer.observe(document.documentElement, {
      subtree: true,
      attributes: true,
      childList: true,
      characterData: true,
    });
    scope.__candidateObserver = observer;
  });
}

async function performDirectHandoff(page: Page): Promise<void> {
  await openAgent(page, 'Chief');
  await page.getByRole('button', { name: 'Agent network' }).click();
  const network = page.getByTestId('grok-agent-network');
  await expect(network).toBeVisible();

  const research = network.locator('article').filter({ hasText: 'Research' }).first();
  await research.getByRole('checkbox').check();
  await network.getByRole('textbox').fill('Research: verify the candidate handoff path and report one concise fact.');
  await network.getByRole('button', { name: /Handoff to Research/ }).click();
  await expect(network.getByRole('textbox')).toHaveValue('');
  await expect(network.getByText(/Chief.*Research|Research.*Chief/).first()).toBeVisible({ timeout: 20_000 });
  await network.getByRole('button', { name: 'Close Agent network' }).click();
}

async function performBroadcast(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Broadcast to agents' }).click();
  const network = page.getByTestId('grok-agent-network');
  await expect(network).toBeVisible();
  await network.getByRole('textbox').fill('Candidate broadcast: acknowledge the signed package acceptance run.');
  await network.getByRole('button', { name: /Broadcast to all/ }).click();
  await expect(network.getByRole('textbox')).toHaveValue('');
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
  test.skip(!realAcceptance, 'Set OBF_REAL_ACCEPTANCE=1 to run signed packaged acceptance.');

  test('exact candidate covers handoff, broadcast, two-Agent isolation and real lifecycle', async () => {
    test.setTimeout(12 * 60_000);
    expect(executable, 'FABUSHI_ELECTRON_EXECUTABLE is required').toBeTruthy();
    expect(sourceSha, 'OBF_SOURCE_SHA must be the exact candidate HEAD').toMatch(/^[0-9a-f]{40}$/);
    expect(expectedSourceSha, 'OBF_EXPECTED_SOURCE_SHA must be a full SHA').toMatch(/^[0-9a-f]{40}$/);
    expect(sourceSha, 'candidate executable must be built from the requested exact HEAD').toBe(expectedSourceSha);
    expect(process.env.FABUSHI_FEATURE_HOST_MODE || '').not.toBe('test');
    expect(process.env.FABUSHI_E2E || '').not.toBe('1');

    await rm(evidenceRoot, { recursive: true, force: true });
    await mkdir(path.join(evidenceRoot, 'screenshots'), { recursive: true });

    const appDataDir = path.join(evidenceRoot, 'app-data');
    await mkdir(appDataDir, { recursive: true });
    const runtimeLogs: RuntimeLog[] = [];
    let app: ElectronApplication | null = null;
    let pageForTrace: Page | null = null;
    let traceStarted = false;
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
      const page = await app.firstWindow();
      pageForTrace = page;
      await page.context().tracing.start({ screenshots: true, snapshots: true, sources: true });
      traceStarted = true;
      page.on('console', (message) => runtimeLogs.push({ at: Date.now(), source: 'page-console', text: `${message.type()}: ${message.text()}` }));
      page.on('pageerror', (error) => runtimeLogs.push({ at: Date.now(), source: 'page-error', text: error.stack || error.message }));
      app.process().stdout?.on('data', (chunk) => runtimeLogs.push({ at: Date.now(), source: 'app-stdout', text: String(chunk) }));
      app.process().stderr?.on('data', (chunk) => runtimeLogs.push({ at: Date.now(), source: 'app-stderr', text: String(chunk) }));

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
      const researchPrompt = 'Research isolation token: R-ONLY. Use a real Agent run and return the token.';
      await submitTurn(page, researchPrompt);

      await openAgent(page, 'Builder');
      const builderPrompt = 'Builder isolation token: B-ONLY. Use a real Agent run and return the token.';
      await submitTurn(page, builderPrompt);

      await openAgent(page, 'Research');
      const researchTurn = await waitForCompletedTurn(page, researchPrompt);
      await expect(researchTurn).toContainText('R-ONLY');
      await expect(page.getByTestId('message-list')).not.toContainText('B-ONLY');

      await openAgent(page, 'Builder');
      const builderTurn = await waitForCompletedTurn(page, builderPrompt);
      await expect(builderTurn).toContainText('B-ONLY');
      await expect(page.getByTestId('message-list')).not.toContainText('R-ONLY');
      await screenshot(page, '05-two-agent-isolation');

      await openAgent(page, 'Chief');
      await installLifecycleCapture(page);
      const lifecyclePrompt = 'Lifecycle acceptance: analyze the candidate, perform at least one safe read-only tool step, then finish with CANDIDATE-LIFECYCLE-OK.';
      await submitTurn(page, lifecyclePrompt);
      const lifecycleTurn = await waitForCompletedTurn(page, lifecyclePrompt);
      await expect(lifecycleTurn).toContainText('CANDIDATE-LIFECYCLE-OK');
      await expect.poll(async () => lifecycleTurn.getByTestId('agent-step').count(), { timeout: 30_000 }).toBeGreaterThan(0);
      await expect.poll(async () => lifecycleTurn.locator('[data-testid="agent-step"][data-status="completed"]').count(), { timeout: 30_000 }).toBeGreaterThan(0);
      const lifecycle = await page.evaluate(() => {
        const scope = window as typeof window & { __candidateLifecycle?: LifecycleSample[] };
        return scope.__candidateLifecycle ?? [];
      });
      expect(lifecycle.some((sample) => ['running', 'thinking', 'preparing', 'streaming'].includes(sample.status))).toBe(true);
      expect(lifecycle.some((sample) => sample.status === 'completed')).toBe(true);

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
      if (traceStarted && pageForTrace) {
        await pageForTrace.context().tracing.stop({
          path: path.join(evidenceRoot, 'trace.zip'),
        }).catch(() => undefined);
      }
      await app?.close().catch(() => undefined);
    }
  });
});
