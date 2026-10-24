import { _electron as electron, expect, test, type ElectronApplication, type Locator, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;
const coworkers = [
  ['Chief', 'Chief of staff'],
  ['Research', 'Research and evidence'],
  ['Builder', 'Product engineering'],
  ['Launch', 'Go-to-market'],
] as const;

async function launchDesktopApp(appDataDir: string): Promise<ElectronApplication> {
  return electron.launch({
    ...(packagedExecutable ? { executablePath: packagedExecutable, args: [] } : { args: [appRoot] }),
    env: {
      ...process.env,
      FABUSHI_APP_DATA: appDataDir,
      FABUSHI_FEATURE_HOST_MODE: process.env.FABUSHI_FEATURE_HOST_MODE || 'test',
      MAHAYANA_APP_HOST_BIN: process.env.MAHAYANA_APP_HOST_BIN || '',
    },
  });
}

async function completeBrowserLogin(page: Page): Promise<void> {
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
      const loginGate = page.getByTestId('login-gate');
      await page.getByTestId('browser-login-start').click();
      await expect(loginGate).toBeHidden();
      continue;
    }
    if (phase === 'ready') break;
  }
  const workspace = page.getByTestId('messenger-workspace');
  await expect(workspace).toHaveAttribute('data-initial-host-hydrated', 'true', { timeout: 15_000 });
  await expect(workspace).toBeVisible();
}

async function setReferenceViewport(app: ElectronApplication, page: Page): Promise<void> {
  await app.evaluate(({ BrowserWindow }) => {
    const win = BrowserWindow.getAllWindows()[0];
    win?.setContentSize(1671, 937, false);
    win?.center();
  });
  await page.emulateMedia({ colorScheme: 'dark', reducedMotion: 'reduce' });
  await expect(page.getByTestId('messenger-workspace')).toBeVisible();
}

async function createCoworker(page: Page, name: string, description: string): Promise<void> {
  await page.evaluate(async ({ name: botName, description: botDescription }) => {
    if (!window.mahayana?.invoke) throw new Error('Mahayana bridge unavailable');
    const now = Date.now();
    await window.mahayana.invoke('feature.execute', {
      command: {
        type: 'bot.create',
        requestId: `obf-bot-create-${botName}-${now}`,
        name: botName,
        description: botDescription,
      },
    });
    await window.mahayana.invoke('feature.execute', {
      command: { type: 'bot.list', requestId: `obf-bot-list-${botName}-${now}` },
    });
  }, { name, description });
  await expect(page.locator('[data-testid^="peer-legacy:bot:"]').filter({ hasText: name }).first()).toBeVisible({ timeout: 10_000 });
}

function peerByName(page: Page, name: string): Locator {
  return page.locator('[data-testid^="peer-legacy:bot:"]').filter({ hasText: name }).first();
}

async function botShape(locator: Locator): Promise<string> {
  const mark = locator.locator('[data-engine="fabushi-motion-v3"]').first();
  await expect(mark).toBeVisible();
  const shape = await mark.getAttribute('data-shape');
  expect(shape).toBeTruthy();
  return shape!;
}

async function sendRuntimeTurn(page: Page, prompt: string): Promise<void> {
  await page.getByTestId('messenger-input').fill(prompt);
  await page.getByTestId('messenger-send').click();
  await expect(page.getByRole('article').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 2_000 });
  const workbench = page.getByTestId('agent-workbench');
  await expect(workbench).toBeVisible({ timeout: 15_000 });
  const run = page.getByTestId('agent-run').last();
  await expect(run).toHaveAttribute('data-status', 'completed', { timeout: 20_000 });
  await expect.poll(async () => run.getByTestId('agent-step').count(), { timeout: 10_000 }).toBeGreaterThanOrEqual(3);
}

test('OBF real event projection keeps coworker identity stable across roster header action and transcript', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-obf-gap-closure-'));
  let app: ElectronApplication | null = null;
  try {
    app = await launchDesktopApp(appDataDir);
    let page = await app.firstWindow();
    await completeBrowserLogin(page);
    await setReferenceViewport(app, page);

    for (const [name, description] of coworkers) await createCoworker(page, name, description);
    for (const [name] of coworkers) await expect(peerByName(page, name)).toBeVisible();

    const chiefPeer = peerByName(page, 'Chief');
    const rosterShape = await botShape(chiefPeer);
    await chiefPeer.click();
    await expect(page.getByTestId('messenger-input')).toBeVisible();

    const headerMark = page.locator('[class*="chatIdentity"] [data-engine="fabushi-motion-v3"]').first();
    await expect(headerMark).toHaveAttribute('data-shape', rosterShape);

    await sendRuntimeTurn(page, 'Analyze the launch readiness, plan the work, use the available tools, and report the result.');
    const completedStep = page.locator('article[data-testid="agent-step"][data-status="completed"]').last();
    await expect(completedStep).toHaveCount(1);
    await expect(completedStep.locator('[data-engine="fabushi-motion-v3"]').first()).toHaveAttribute('data-shape', rosterShape);

    const finalPeerMessage = page.locator('article[class*="messagePeer"]').last();
    await expect(finalPeerMessage).toBeVisible();
    await expect(finalPeerMessage.locator('[data-engine="fabushi-motion-v3"]').first()).toHaveAttribute('data-shape', rosterShape);
    await finalPeerMessage.hover();
    await expect(finalPeerMessage.getByTestId('message-hover-actions')).toBeVisible();

  } finally {
    await app?.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('OBF reference visual geometry stays bounded at the 1671x937 dark content viewport', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-obf-visual-'));
  const app = await launchDesktopApp(appDataDir);
  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await setReferenceViewport(app, page);
    for (const [name, description] of coworkers) await createCoworker(page, name, description);

    const sidebar = page.getByTestId('messenger-sidebar');
    const rosterBox = await sidebar.boundingBox();
    expect(rosterBox?.width ?? 0).toBeGreaterThanOrEqual(250);
    expect(rosterBox?.width ?? 999).toBeLessThanOrEqual(390);

    for (const [name] of coworkers) {
      const peer = peerByName(page, name);
      const markBox = await peer.locator('[data-engine="fabushi-motion-v3"]').first().boundingBox();
      expect(markBox?.width ?? 0).toBeGreaterThanOrEqual(32);
      expect(markBox?.width ?? 99).toBeLessThanOrEqual(36);
      expect(markBox?.height ?? 0).toBeGreaterThanOrEqual(32);
      expect(markBox?.height ?? 99).toBeLessThanOrEqual(36);
    }

    const chief = peerByName(page, 'Chief');
    await chief.click();
    await expect(page.getByTestId('messenger-input')).toBeVisible();
    const composerBox = await page.getByTestId('messenger-input').locator('xpath=ancestor::form[1]').boundingBox();
    expect(composerBox?.height ?? 0).toBeGreaterThanOrEqual(50);
    expect(composerBox?.width ?? 0).toBeGreaterThan(600);

    await page.screenshot({ path: test.info().outputPath('obf-1671x937-dark.png'), fullPage: false });
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
