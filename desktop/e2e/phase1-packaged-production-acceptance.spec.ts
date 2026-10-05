import { chromium, expect, test, type Browser, type Page } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import { appendFile, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

const realAcceptance = process.env.OBF_REAL_ACCEPTANCE === '1';
const executable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || '';
const sourceSha = process.env.OBF_SOURCE_SHA?.trim() || '';
const expectedSourceSha = process.env.OBF_EXPECTED_SOURCE_SHA?.trim() || sourceSha;
const primarySessionPath = process.env.FABUSHI_CI_ACCOUNT_SESSION_FILE?.trim() || '';
const peerSessionPath = process.env.FABUSHI_CI_PEER_ACCOUNT_SESSION_FILE?.trim() || '';
const evidenceRoot = process.env.OBF_EVIDENCE_DIR?.trim()
  || path.join(tmpdir(), `fabushi-openbot-acceptance-${sourceSha.slice(0, 12) || 'unknown'}`);
const phaseRoot = path.join(evidenceRoot, 'phase1');

test.use({ trace: 'off' });

type SessionIdentity = {
  readonly userId: string;
  readonly username: string;
  readonly deviceId: string;
};

type Candidate = {
  readonly label: 'a' | 'b';
  readonly appDataDir: string;
  readonly sessionPath: string;
  readonly process: ChildProcess;
  readonly browser: Browser;
  readonly page: Page;
};

async function readIdentity(sessionPath: string): Promise<SessionIdentity> {
  const payload = JSON.parse(await readFile(sessionPath, 'utf8')) as Record<string, unknown>;
  expect(payload.provider).toBe('github-actions');
  expect(payload.ciRunner).toBe(true);
  expect('refreshToken' in payload).toBe(false);
  const user = payload.user != null && typeof payload.user === 'object' ? payload.user as Record<string, unknown> : {};
  const userId = String(payload.userId ?? user.id ?? '').trim();
  const username = String(payload.username ?? user.username ?? '').trim();
  const deviceId = String(payload.deviceId ?? '').trim();
  expect(userId).not.toBe('');
  expect(username).not.toBe('');
  expect(deviceId).toMatch(/^gha-[0-9]+-[0-9]+-macos-app-[ab]$/u);
  return { userId, username, deviceId };
}

function isShippingRendererUrl(url: string): boolean {
  if (url.startsWith('app://bundle/')) return true;
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'file:' && parsed.pathname.endsWith('/dist/renderer/index.html');
  } catch {
    return false;
  }
}

async function waitForPackagedRenderer(appDataDir: string): Promise<{ browser: Browser; page: Page }> {
  const deadline = Date.now() + 120_000;
  let lastError = '';
  while (Date.now() < deadline) {
    try {
      const [portText] = (await readFile(path.join(appDataDir, 'DevToolsActivePort'), 'utf8')).trim().split(/\r?\n/u);
      const port = Number(portText);
      if (!Number.isInteger(port) || port <= 0) throw new Error('invalid DevToolsActivePort');
      const response = await fetch(`http://127.0.0.1:${port}/json/list`);
      const targets = await response.json() as Array<{ type?: string; url?: string }>;
      if (!targets.some((target) => target.type === 'page' && typeof target.url === 'string' && isShippingRendererUrl(target.url))) {
        throw new Error('shipping renderer target not mounted');
      }
      const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`, { timeout: 15_000 });
      const page = browser.contexts().flatMap((context) => context.pages()).find((candidate) => isShippingRendererUrl(candidate.url()));
      if (page != null) return { browser, page };
      await browser.close().catch(() => undefined);
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`signed candidate did not expose a shipping renderer: ${lastError}`);
}

async function completeBrowserLogin(page: Page): Promise<void> {
  const workspace = page.getByTestId('messenger-workspace');
  const signInLanding = page.getByRole('main', { name: 'Grok Bot' });
  const onboarding = page.locator('.sand-onboarding[data-step]');
  await expect(workspace).toBeVisible({ timeout: 35_000 });

  const readAccountKind = async () => page.evaluate(async () => {
    const desktop = (window as unknown as { desktop?: { cursorAccount?: { getStatus?: () => Promise<{ kind?: unknown }> } } }).desktop;
    if (typeof desktop?.cursorAccount?.getStatus !== 'function') return null;
    const status = await desktop.cursorAccount.getStatus();
    return typeof status?.kind === 'string' ? status.kind : null;
  });

  for (let attempt = 0; attempt < 18; attempt += 1) {
    const kind = await readAccountKind().catch(() => null);
    if (kind === 'logged-in') {
      if (await onboarding.count()) {
        const step = await onboarding.getAttribute('data-step');
        if (step === 'meet' || step === 'computer-demo' || step === 'jobs' || step === 'tools') {
          await onboarding.getByRole('button', { name: 'Next', exact: true }).last().click();
          continue;
        }
        if (step === 'create') {
          const name = onboarding.getByPlaceholder('New Bot');
          if ((await name.inputValue()).trim().length === 0) await name.fill(`Phase 1 ${Date.now()}`);
          await onboarding.getByRole('button', { name: 'Get started', exact: true }).click();
          continue;
        }
        if (step === 'hand-off') {
          await expect(onboarding).toHaveCount(0, { timeout: 90_000 });
          continue;
        }
        throw new Error(`unexpected onboarding step: ${step ?? 'unknown'}`);
      }
      await expect(signInLanding).toHaveCount(0);
      await expect(page.getByRole('status', { name: 'Connected' }).first()).toBeVisible({ timeout: 45_000 });
      await expect(page.locator('aside[aria-label="Agents"] button.sand-agent-item').first()).toBeVisible({ timeout: 45_000 });
      return;
    }
    if (kind === 'logged-out') {
      const signIn = signInLanding.getByRole('button', { name: 'Sign in', exact: true });
      if (await signIn.count()) await signIn.click();
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  throw new Error('bounded production account did not reach the canonical signed workspace');
}

async function launchCandidate(label: 'a' | 'b', sessionPath: string, appDataDir: string): Promise<Candidate> {
  await mkdir(appDataDir, { recursive: true });
  await rm(path.join(appDataDir, 'DevToolsActivePort'), { force: true });
  const runtimeLog = path.join(phaseRoot, `runtime-${label}.log`);
  const appProcess = spawn(executable, ['--remote-debugging-address=127.0.0.1', '--remote-debugging-port=0'], {
    env: {
      ...process.env,
      FABUSHI_APP_DATA: appDataDir,
      SAND_USER_DATA_DIR: appDataDir,
      FABUSHI_CI_ACCOUNT_SESSION_FILE: sessionPath,
      OBF_SOURCE_SHA: sourceSha,
    },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  const write = (source: string, chunk: unknown) =>
    appendFile(runtimeLog, `[${new Date().toISOString()}] ${source}: ${String(chunk)}\n`).catch(() => undefined);
  appProcess.stdout?.on('data', (chunk) => void write('stdout', chunk));
  appProcess.stderr?.on('data', (chunk) => void write('stderr', chunk));
  const { browser, page } = await Promise.race([
    waitForPackagedRenderer(appDataDir),
    new Promise<never>((_, reject) => appProcess.once('exit', (code, signal) =>
      reject(new Error(`candidate ${label} exited before renderer binding code=${code} signal=${signal}`)))),
  ]);
  page.on('console', (message) => void write(`console-${message.type()}`, message.text()));
  page.on('pageerror', (error) => void write('page-error', error.stack || error.message));
  page.on('requestfailed', (request) => void write('request-failed', `${request.method()} ${request.url()} :: ${request.failure()?.errorText || 'unknown'}`));
  await page.context().tracing.start({ screenshots: true, snapshots: true, sources: true });
  await completeBrowserLogin(page);
  await expect(page.getByTestId('messenger-workspace')).toHaveAttribute('data-runtime', 'electron');
  await expect(page.getByTestId('messenger-workspace')).toHaveAttribute('data-product-shell', 'agent');
  return { label, appDataDir, sessionPath, process: appProcess, browser, page };
}

async function stopCandidate(candidate: Candidate, keepTrace = true): Promise<void> {
  if (keepTrace) {
    await candidate.page.context().tracing.stop({ path: path.join(phaseRoot, `trace-${candidate.label}.zip`) }).catch(() => undefined);
  }
  candidate.process.kill('SIGTERM');
  await Promise.race([
    new Promise<void>((resolve) => candidate.process.once('exit', () => resolve())),
    new Promise<void>((resolve) => setTimeout(resolve, 5000)),
  ]);
  await candidate.browser.close().catch(() => undefined);
}

async function screenshot(candidate: Candidate, name: string): Promise<void> {
  await candidate.page.screenshot({ path: path.join(phaseRoot, 'screenshots', `${name}.png`), fullPage: true });
}

async function createHumanConversation(page: Page, peer: SessionIdentity, title: string): Promise<void> {
  const existing = page.locator('aside[aria-label="Agents"]').getByRole('button', { name: title, exact: true });
  if (await existing.count()) {
    await existing.click();
    return;
  }
  await page.getByRole('button', { name: 'New Human chat', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'New Human chat' });
  await dialog.getByRole('textbox', { name: 'Human identity' }).fill(peer.userId);
  await dialog.getByRole('textbox', { name: 'Conversation title' }).fill(title);
  await dialog.getByRole('button', { name: 'Create', exact: true }).click();
  await expect(page.getByRole('group', { name: 'Human call controls' })).toBeVisible({ timeout: 20_000 });
}

async function openHuman(page: Page, title: string): Promise<void> {
  const row = page.locator('aside[aria-label="Agents"]').getByRole('button', { name: title, exact: true });
  await expect(row).toBeVisible({ timeout: 20_000 });
  await row.click();
  await expect(page.getByRole('group', { name: 'Human call controls' })).toBeVisible();
}

async function sendHumanMessage(page: Page, text: string): Promise<void> {
  const prompt = page.getByRole('textbox', { name: 'Prompt' });
  await prompt.pressSequentially(text);
  await page.getByRole('button', { name: 'Send message' }).click();
  const turn = page.getByRole('article').filter({ hasText: text }).last();
  await expect(turn).toBeVisible({ timeout: 15_000 });
  await expect(turn).not.toHaveAttribute('data-pending', { timeout: 30_000 });
}

async function forceHumanSync(page: Page, title: string): Promise<void> {
  const rows = page.locator('aside[aria-label="Agents"] button.sand-agent-item');
  if (await rows.count() > 1) {
    const active = rows.filter({ has: page.locator('[aria-current="page"]') });
    const first = rows.first();
    await first.click().catch(() => undefined);
  }
  await openHuman(page, title);
}

async function assertSettingsA11yI18n(candidate: Candidate): Promise<void> {
  const page = candidate.page;
  await page.getByRole('button', { name: 'Account', exact: true }).click();
  await page.getByText('Settings', { exact: true }).click();
  const settings = page.getByRole('dialog', { name: 'Grok Bot settings' });
  await expect(settings).toBeVisible();
  await expect(settings.getByRole('navigation', { name: 'Settings sections' })).toBeVisible();

  for (const text of ['Language & Accessibility', 'Privacy', 'Desktop behavior', 'Notifications', 'Storage', 'Downloads', 'Shortcuts', 'Advanced']) {
    await expect(settings.getByText(text, { exact: true })).toBeVisible();
  }
  await expect(settings.getByRole('status', { name: 'Privacy mode' })).toHaveText('Enabled');

  const theme = settings.getByRole('button', { name: 'Theme' });
  await theme.focus();
  await page.keyboard.press('Enter');
  await page.getByRole('option', { name: 'Dark', exact: true }).press('Enter');
  await expect(theme).toContainText('Dark');

  await settings.getByRole('button', { name: 'Updates', exact: true }).click();
  await expect(settings.getByRole('heading', { name: 'Updates', exact: true })).toBeVisible();
  await expect(settings.locator('output[aria-live="polite"]')).toBeVisible();

  await settings.getByRole('button', { name: 'General', exact: true }).click();
  const language = settings.getByRole('button', { name: 'Language' });
  await language.focus();
  await page.keyboard.press('Enter');
  await page.getByRole('option', { name: 'العربية', exact: true }).press('Enter');
  const localized = page.locator('.sand-settings-dialog');
  await expect(localized).toHaveAttribute('aria-label', 'إعدادات Grok Bot');
  await expect.poll(() => page.evaluate(() => ({ lang: document.documentElement.lang, dir: document.documentElement.dir })))
    .toEqual({ lang: 'ar', dir: 'rtl' });

  const reduceMotion = localized.getByRole('switch', { name: /تقليل الحركة/u });
  await reduceMotion.focus();
  await page.keyboard.press('Space');
  const highContrast = localized.getByRole('switch', { name: /تباين عالٍ/u });
  await highContrast.focus();
  await page.keyboard.press('Space');

  const direction = localized.getByRole('button', { name: 'اتجاه القراءة' });
  await direction.focus();
  await page.keyboard.press('Enter');
  await page.getByRole('option', { name: 'من اليمين إلى اليسار', exact: true }).press('Enter');
  const textSize = localized.getByRole('button', { name: 'حجم النص' });
  await textSize.focus();
  await page.keyboard.press('Enter');
  await page.getByRole('option', { name: '125%', exact: true }).press('Enter');

  const liveStatus = localized.locator('.sand-settings-a11y-status');
  await expect(liveStatus).toHaveAttribute('role', 'status');
  await expect.poll(() => page.evaluate(() => ({
    reducedMotion: document.documentElement.dataset.sandReducedMotion,
    highContrast: document.documentElement.dataset.sandHighContrast,
    textScale: document.documentElement.style.getPropertyValue('--sand-ui-text-scale'),
  }))).toEqual({ reducedMotion: 'true', highContrast: 'true', textScale: '1.25' });

  const devices = await page.evaluate(async () => (await navigator.mediaDevices.enumerateDevices()).map((device) => ({
    deviceId: device.deviceId,
    kind: device.kind,
    label: device.label,
  })));
  expect(devices.some((device) => device.kind === 'audioinput'), 'signed runner must expose a real microphone input').toBe(true);
  expect(devices.some((device) => device.kind === 'videoinput'), 'signed runner must expose a real camera input').toBe(true);

  await localized.getByRole('button', { name: 'إغلاق', exact: true }).click();
  await openHuman(page, (await page.locator('aside[aria-label="Agents"] button.sand-agent-item[aria-current="page"]').getAttribute('aria-label')) || '').catch(() => undefined);
  const prompt = page.getByRole('textbox', { name: 'Prompt' });
  await expect(prompt).toHaveAttribute('dir', 'auto');
  const typography = await prompt.evaluate((element) => {
    const root = getComputedStyle(document.documentElement);
    const style = getComputedStyle(element);
    return {
      fallback: root.getPropertyValue('--fabushi-ui-font-fallback'),
      unicodeBidi: style.unicodeBidi,
    };
  });
  expect(typography.fallback).toContain('Noto Sans CJK SC');
  expect(typography.fallback).toContain('Noto Sans Arabic');
  expect(typography.fallback).toContain('Apple Color Emoji');
  expect(typography.unicodeBidi).toBe('plaintext');

  await prompt.focus();
  await prompt.dispatchEvent('compositionstart', { data: '' });
  await page.keyboard.insertText('かな漢字');
  await page.keyboard.press('Enter');
  await expect(prompt).toContainText('かな漢字');
  await expect(page.getByRole('button', { name: 'Stop response' })).toHaveCount(0);
  await prompt.dispatchEvent('compositionend', { data: 'かな漢字' });
  await page.keyboard.insertText('🙂 العربية Fabushi');
  await expect(prompt).toContainText('かな漢字🙂 العربية Fabushi');
  await screenshot(candidate, '07-settings-a11y-i18n');
}

async function runVoiceCall(a: Candidate, b: Candidate): Promise<void> {
  await a.page.getByRole('button', { name: 'Start voice call' }).click();
  await expect(a.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toHaveText(/ringing|negotiating/u, { timeout: 30_000 });
  await expect(b.page.getByText('Incoming call', { exact: true })).toBeVisible({ timeout: 30_000 });
  await b.page.getByRole('button', { name: 'Accept voice call' }).click();
  await expect(a.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toHaveText('connected', { timeout: 60_000 });
  await expect(b.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toHaveText('connected', { timeout: 60_000 });

  const mute = a.page.getByRole('button', { name: 'Mute', exact: true });
  await mute.click();
  await expect(a.page.getByRole('button', { name: 'Unmute', exact: true })).toHaveAttribute('aria-pressed', 'true');
  await a.page.getByRole('button', { name: 'Unmute', exact: true }).click();

  await a.page.getByRole('button', { name: 'Camera on', exact: true }).click();
  await expect(a.page.getByRole('button', { name: 'Camera off', exact: true })).toHaveAttribute('aria-pressed', 'true');

  await a.page.getByRole('button', { name: 'Share screen', exact: true }).click();
  await expect(a.page.getByRole('button', { name: 'Stop sharing', exact: true })).toHaveAttribute('aria-pressed', 'true', { timeout: 30_000 });
  await a.page.getByRole('button', { name: 'Stop sharing', exact: true }).click();

  await b.page.context().setOffline(true);
  await expect(a.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toHaveText(/reconnecting|failed/u, { timeout: 45_000 });
  await b.page.context().setOffline(false);
  await expect(a.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toHaveText('connected', { timeout: 90_000 });

  await a.page.getByRole('button', { name: 'End call' }).click();
  await expect(a.page.getByRole('button', { name: 'Start voice call' })).toBeVisible({ timeout: 30_000 });
  await expect(b.page.getByRole('button', { name: 'Start voice call' })).toBeVisible({ timeout: 30_000 });
}

async function runVideoCall(a: Candidate, b: Candidate): Promise<void> {
  await b.page.getByRole('button', { name: 'Start video call' }).click();
  await expect(a.page.getByText('Incoming call', { exact: true })).toBeVisible({ timeout: 30_000 });
  await a.page.getByRole('button', { name: 'Accept video call' }).click();
  await expect(a.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toHaveText('connected', { timeout: 60_000 });
  await expect(b.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toHaveText('connected', { timeout: 60_000 });
  await expect(a.page.getByRole('button', { name: 'Camera off', exact: true })).toBeVisible();
  await expect(b.page.getByRole('button', { name: 'Camera off', exact: true })).toBeVisible();
  await b.page.getByRole('button', { name: 'End call' }).click();
}

test.describe('signed candidate Phase 1 production acceptance', () => {
  test.describe.configure({ retries: 0 });
  test.skip(!realAcceptance, 'Set OBF_REAL_ACCEPTANCE=1 to run signed packaged production acceptance.');

  test('Human, calls, Settings, accessibility and i18n execute on two bounded production accounts', async () => {
    test.setTimeout(30 * 60_000);
    expect(executable).toBeTruthy();
    expect(sourceSha).toMatch(/^[0-9a-f]{40}$/u);
    expect(expectedSourceSha).toBe(sourceSha);
    expect(primarySessionPath).toBeTruthy();
    expect(peerSessionPath).toBeTruthy();
    expect(process.env.FABUSHI_FEATURE_HOST_MODE || '').not.toBe('test');
    expect(process.env.FABUSHI_E2E || '').not.toBe('1');

    await mkdir(path.join(phaseRoot, 'screenshots'), { recursive: true });
    const firstIdentity = await readIdentity(primarySessionPath);
    const secondIdentity = await readIdentity(peerSessionPath);
    expect(firstIdentity.userId).not.toBe(secondIdentity.userId);
    expect(firstIdentity.deviceId).not.toBe(secondIdentity.deviceId);

    const titleA = `Phase1 ${sourceSha.slice(0, 8)} ${secondIdentity.username}`;
    const titleB = `Phase1 ${sourceSha.slice(0, 8)} ${firstIdentity.username}`;
    const appDataA = path.join(phaseRoot, 'app-data-a');
    const appDataB = path.join(phaseRoot, 'app-data-b');
    let a: Candidate | null = null;
    let b: Candidate | null = null;
    try {
      a = await launchCandidate('a', primarySessionPath, appDataA);
      b = await launchCandidate('b', peerSessionPath, appDataB);
      await createHumanConversation(a.page, secondIdentity, titleA);
      await createHumanConversation(b.page, firstIdentity, titleB);
      await screenshot(a, '01-human-a');
      await screenshot(b, '02-human-b');

      const root = `PHASE1-HUMAN-ROOT-${sourceSha.slice(0, 12)}`;
      const reply = `PHASE1-HUMAN-REPLY-${sourceSha.slice(0, 12)}`;
      await sendHumanMessage(a.page, root);
      const rootTurn = a.page.getByRole('article').filter({ hasText: root }).last();
      await rootTurn.hover();
      await rootTurn.getByRole('button', { name: 'Reply to your message' }).click();
      const attachmentName = `phase1-${sourceSha.slice(0, 8)}.txt`;
      await a.page.locator('input.sand-prompt-file-input').setInputFiles({
        name: attachmentName,
        mimeType: 'text/plain',
        buffer: Buffer.from(`signed packaged Phase 1 ${sourceSha}\n`, 'utf8'),
      });
      await sendHumanMessage(a.page, reply);

      await forceHumanSync(b.page, titleB);
      await expect(b.page.getByRole('article').filter({ hasText: root }).last()).toBeVisible({ timeout: 30_000 });
      const remoteReply = b.page.getByRole('article').filter({ hasText: reply }).last();
      await expect(remoteReply).toBeVisible({ timeout: 30_000 });
      await expect(remoteReply.getByText(attachmentName, { exact: true })).toBeVisible();

      const receivedRoot = b.page.getByRole('article').filter({ hasText: root }).last();
      await receivedRoot.hover();
      await receivedRoot.getByRole('button', { name: 'Add reaction' }).click();
      await b.page.getByRole('button', { name: 'React with 👍' }).click();
      await sendHumanMessage(b.page, `B-ACK-${sourceSha.slice(0, 12)}`);

      await forceHumanSync(a.page, titleA);
      await expect(a.page.getByRole('button', { name: /reacted with 👍/iu })).toBeVisible({ timeout: 30_000 });
      await expect(a.page.getByRole('article').filter({ hasText: `B-ACK-${sourceSha.slice(0, 12)}` }).last()).toBeVisible({ timeout: 30_000 });

      await a.page.keyboard.press('Control+f');
      const find = a.page.getByRole('textbox', { name: 'Find in chat' });
      await find.fill(root);
      await expect(a.page.locator('.sand-chat-find').getByRole('status')).toHaveText('1/1');
      await a.page.getByRole('button', { name: 'Close find' }).click();
      await screenshot(a, '03-human-reply-attachment-reaction-search');

      await stopCandidate(a);
      a = await launchCandidate('a', primarySessionPath, appDataA);
      await openHuman(a.page, titleA);
      await expect(a.page.getByRole('article').filter({ hasText: root })).toHaveCount(1);
      await expect(a.page.getByRole('article').filter({ hasText: reply })).toHaveCount(1);
      await expect(a.page.getByRole('article').filter({ hasText: `B-ACK-${sourceSha.slice(0, 12)}` })).toHaveCount(1);

      await a.page.getByRole('button', { name: 'Ask Agent', exact: true }).click();
      await expect(a.page.locator('[aria-label="Conversation transcript"] [role="article"][data-role="assistant"]').last())
        .toBeVisible({ timeout: 120_000 });
      await screenshot(a, '04-human-agent-handoff');

      await runVoiceCall(a, b);
      await screenshot(a, '05-voice-reconnect');
      await runVideoCall(a, b);
      await screenshot(b, '06-video-call');

      await assertSettingsA11yI18n(a);

      await writeFile(path.join(phaseRoot, 'summary.json'), JSON.stringify({
        sourceSha,
        expectedSourceSha,
        executable,
        accounts: [
          { ref: firstIdentity.userId, deviceId: firstIdentity.deviceId },
          { ref: secondIdentity.userId, deviceId: secondIdentity.deviceId },
        ],
        coverage: {
          humanReplyAttachmentReactionSearch: true,
          restartIdempotency: true,
          remoteConvergence: true,
          humanToAgentHandoff: true,
          outgoingIncomingVoiceVideo: true,
          permissionsDevicesMuteCameraScreenShare: true,
          iceSignalingReconnectHangup: true,
          settingsAccessibilityI18n: true,
        },
      }, null, 2));
    } finally {
      if (b != null) await stopCandidate(b).catch(() => undefined);
      if (a != null) await stopCandidate(a).catch(() => undefined);
    }
  });
});
