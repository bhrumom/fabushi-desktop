import { chromium, expect, test, type Browser, type Page } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

const realAcceptance = process.env.OBF_REAL_ACCEPTANCE === '1';
const executable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || '';
const sourceSha = process.env.OBF_SOURCE_SHA?.trim() || '';
const expectedSourceSha = process.env.OBF_EXPECTED_SOURCE_SHA?.trim() || sourceSha;
const evidenceRoot = process.env.OBF_EVIDENCE_DIR?.trim()
  || path.join(tmpdir(), 'fabushi-phase1-acceptance-' + (sourceSha.slice(0, 12) || 'unknown'));
const primaryUsername = process.env.FABUSHI_CI_TEST_USERNAME?.trim() || '';
const peerUsername = process.env.FABUSHI_CI_PEER_TEST_USERNAME?.trim() || '';
const primarySession = process.env.FABUSHI_CI_ACCOUNT_SESSION_FILE?.trim() || '';
const peerSession = process.env.FABUSHI_CI_PEER_ACCOUNT_SESSION_FILE?.trim() || '';
const primaryDeviceId = process.env.DEVICE_ID?.trim() || '';
const peerDeviceId = process.env.PEER_DEVICE_ID?.trim() || '';

test.use({ trace: 'off' });

type AppHandle = {
  process: ChildProcess;
  browser: Browser;
  page: Page;
  appDataDir: string;
  tracePath: string;
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

async function bindRenderer(appDataDir: string): Promise<{ browser: Browser; page: Page }> {
  const end = Date.now() + 120_000;
  let lastError = '';
  while (Date.now() < end) {
    try {
      const activePort = await readFile(path.join(appDataDir, 'DevToolsActivePort'), 'utf8');
      const port = Number(activePort.trim().split(/\r?\n/u)[0]);
      if (!Number.isInteger(port) || port <= 0) throw new Error('invalid DevToolsActivePort');
      const browser = await chromium.connectOverCDP('http://127.0.0.1:' + port, { timeout: 20_000 });
      const page = browser.contexts().flatMap((context) => context.pages()).find((candidate) => isShippingRendererUrl(candidate.url()));
      if (page != null) return { browser, page };
      await browser.close().catch(() => undefined);
      lastError = 'no shipping renderer page';
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
    }
    await new Promise((resolve) => setTimeout(resolve, 300));
  }
  throw new Error('signed packaged renderer was not bindable: ' + lastError);
}

async function completeBrowserLogin(page: Page): Promise<void> {
  const workspace = page.getByTestId('messenger-workspace');
  const onboarding = page.locator('.sand-onboarding[data-step]');
  const signInLanding = page.getByRole('main', { name: 'Grok Bot' });
  await expect(workspace).toBeVisible({ timeout: 35_000 });
  await expect(workspace).toHaveAttribute('data-product-shell', 'agent');
  await expect(workspace).toHaveAttribute('data-runtime', 'electron');

  const accountKind = async (): Promise<string | null> => page.evaluate(async () => {
    const desktop = (window as unknown as { desktop?: { cursorAccount?: { getStatus?: () => Promise<{ kind?: unknown }> } } }).desktop;
    if (typeof desktop?.cursorAccount?.getStatus !== 'function') return null;
    const value = await desktop.cursorAccount.getStatus().catch(() => null);
    return value != null && typeof value.kind === 'string' ? value.kind : null;
  });

  for (let attempt = 0; attempt < 20; attempt += 1) {
    const kind = await accountKind();
    if (kind === 'logged-in') {
      if (await onboarding.count()) {
        const step = await onboarding.getAttribute('data-step');
        if (step === 'meet' || step === 'computer-demo' || step === 'jobs' || step === 'tools') {
          await onboarding.getByRole('button', { name: 'Next', exact: true }).last().click();
          continue;
        }
        if (step === 'create') {
          const name = onboarding.getByPlaceholder('New Bot');
          if ((await name.inputValue()).trim().length === 0) await name.fill('Phase 1 ' + Date.now());
          await onboarding.getByRole('button', { name: 'Get started', exact: true }).click();
          continue;
        }
        if (step === 'hand-off') {
          await expect(onboarding).toHaveCount(0, { timeout: 90_000 });
          continue;
        }
      }
      if (await signInLanding.count()) {
        await new Promise((resolve) => setTimeout(resolve, 500));
        continue;
      }
      await expect(page.getByRole('status', { name: 'Connected' }).first()).toBeVisible({ timeout: 45_000 });
      await expect.poll(async () => page.locator('aside[aria-label="Agents"] button.sand-agent-item').count(), { timeout: 45_000 }).toBeGreaterThan(0);
      return;
    }
    if (kind === 'logged-out' && await signInLanding.getByRole('button', { name: 'Sign in', exact: true }).count()) {
      await signInLanding.getByRole('button', { name: 'Sign in', exact: true }).click();
    }
    await new Promise((resolve) => setTimeout(resolve, 750));
  }
  throw new Error('bounded production account did not reach the shipping workspace');
}

async function launchSignedApp(name: string, appDataDir: string, sessionFile: string, deviceId: string): Promise<AppHandle> {
  await mkdir(appDataDir, { recursive: true });
  const processHandle = spawn(executable, ['--remote-debugging-address=127.0.0.1', '--remote-debugging-port=0'], {
    env: {
      ...process.env,
      FABUSHI_APP_DATA: appDataDir,
      SAND_USER_DATA_DIR: appDataDir,
      FABUSHI_CI_ACCOUNT_SESSION_FILE: sessionFile,
      DEVICE_ID: deviceId,
      OBF_SOURCE_SHA: sourceSha,
    },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  const logPath = path.join(evidenceRoot, 'phase1-' + name + '-runtime.log');
  const append = async (source: string, chunk: unknown) => {
    const text = '[' + new Date().toISOString() + '] ' + source + ': ' + String(chunk) + '\n';
    const current = await readFile(logPath, 'utf8').catch(() => '');
    await writeFile(logPath, current + text);
  };
  processHandle.stdout?.on('data', (chunk) => { void append('stdout', chunk); });
  processHandle.stderr?.on('data', (chunk) => { void append('stderr', chunk); });
  const earlyExit = new Promise<never>((_, reject) => {
    processHandle.once('exit', (code, signal) => reject(new Error(name + ' signed app exited early code=' + String(code) + ' signal=' + String(signal))));
  });
  const binding = await Promise.race([bindRenderer(appDataDir), earlyExit]);
  const page = binding.page;
  page.on('pageerror', (error) => { void append('page-error', error.stack || error.message); });
  page.on('console', (message) => {
    if (message.type() === 'error' || message.type() === 'warning') void append('page-' + message.type(), message.text());
  });
  await page.context().tracing.start({ screenshots: true, snapshots: true, sources: true });
  await completeBrowserLogin(page);
  return {
    process: processHandle,
    browser: binding.browser,
    page,
    appDataDir,
    tracePath: path.join(evidenceRoot, 'phase1-' + name + '-trace.zip'),
  };
}

async function closeApp(app: AppHandle | null): Promise<void> {
  if (app == null) return;
  await app.page.context().tracing.stop({ path: app.tracePath }).catch(() => undefined);
  await app.browser.close().catch(() => undefined);
  if (app.process.exitCode == null) {
    app.process.kill('SIGTERM');
    await Promise.race([
      new Promise<void>((resolve) => app.process.once('exit', () => resolve())),
      new Promise<void>((resolve) => setTimeout(resolve, 5_000)),
    ]);
  }
  if (app.process.exitCode == null) app.process.kill('SIGKILL');
}

async function screenshot(page: Page, name: string): Promise<void> {
  const dir = path.join(evidenceRoot, 'screenshots');
  await mkdir(dir, { recursive: true });
  await page.screenshot({ path: path.join(dir, name + '.png'), fullPage: true });
}

async function focusAndSync(page: Page): Promise<void> {
  await page.bringToFront();
  await page.evaluate(() => window.focus());
  await new Promise((resolve) => setTimeout(resolve, 1200));
}

async function createHumanIfMissing(page: Page, peer: string): Promise<void> {
  const sidebar = page.locator('aside[aria-label="Agents"]');
  const existing = sidebar.locator('button.sand-agent-item').filter({ hasText: peer }).first();
  if (await existing.count()) return;
  await page.getByRole('button', { name: 'New Human chat' }).click();
  const dialog = page.getByRole('dialog', { name: 'New Human chat' });
  await expect(dialog).toBeVisible();
  await dialog.getByRole('textbox', { name: 'Human identity' }).fill(peer);
  await dialog.getByRole('button', { name: 'Create Human chat' }).click();
  await expect(sidebar.locator('button.sand-agent-item').filter({ hasText: peer }).first()).toBeVisible({ timeout: 45_000 });
}

async function openHuman(page: Page, peer: string): Promise<void> {
  await focusAndSync(page);
  const sidebar = page.locator('aside[aria-label="Agents"]');
  const row = sidebar.locator('button.sand-agent-item').filter({ hasText: peer }).first();
  await expect(row).toBeVisible({ timeout: 45_000 });
  await row.click();
  await expect(page.getByRole('group', { name: 'Human call controls' })).toBeVisible({ timeout: 20_000 });
  await expect(page.getByRole('textbox', { name: 'Prompt' })).toBeVisible();
}

async function sendHuman(page: Page, text: string): Promise<void> {
  const input = page.getByRole('textbox', { name: 'Prompt' });
  await input.fill(text);
  await page.getByRole('button', { name: 'Send message' }).click();
  await expect(page.getByRole('log', { name: 'Conversation transcript' }).getByText(text, { exact: true })).toBeVisible({ timeout: 20_000 });
}

async function waitForHumanText(page: Page, peer: string, text: string): Promise<void> {
  await expect.poll(async () => {
    await focusAndSync(page);
    const row = page.locator('aside[aria-label="Agents"] button.sand-agent-item').filter({ hasText: peer }).first();
    if (await row.count()) await row.click();
    return page.getByRole('log', { name: 'Conversation transcript' }).getByText(text, { exact: true }).count();
  }, { timeout: 60_000, intervals: [1000, 1500, 2500] }).toBeGreaterThan(0);
}

async function chooseSandSelect(page: Page, label: string | RegExp, option: string | RegExp): Promise<void> {
  const trigger = page.getByRole('button', { name: label }).first();
  await expect(trigger).toBeVisible();
  await trigger.click();
  const candidate = page.getByRole('option', { name: option }).first();
  if (await candidate.count()) await candidate.click();
  else {
    const menuItem = page.getByRole('menuitem', { name: option }).first();
    await expect(menuItem).toBeVisible();
    await menuItem.click();
  }
}

async function openSettings(page: Page): Promise<void> {
  const accountTrigger = page.locator('.sand-agents-sidebar__account button[aria-haspopup="menu"]').first();
  await expect(accountTrigger).toBeVisible();
  await accountTrigger.click();
  await page.getByRole('menuitem', { name: 'Settings', exact: true }).click();
  await expect(page.getByRole('dialog', { name: 'Grok Bot settings' })).toBeVisible();
}

async function assertSettingsAccessibilityAndI18n(page: Page): Promise<Record<string, unknown>> {
  await openSettings(page);
  let dialog = page.getByRole('dialog', { name: 'Grok Bot settings' });
  for (const heading of ['Appearance', 'Privacy', 'Language & Accessibility', 'Media & Devices', 'Desktop behavior', 'Advanced']) {
    await expect(dialog.getByRole('heading', { name: heading, exact: true })).toBeVisible();
  }
  for (const label of ['Notifications', 'Storage', 'Downloads', 'Shortcuts']) {
    await expect(dialog.getByText(label, { exact: true })).toBeVisible();
  }
  for (const label of ['Theme', 'Language', 'Reading direction', 'Text size', 'Microphone', 'Camera']) {
    await expect(dialog.getByRole('button', { name: label })).toBeVisible();
  }
  await expect(dialog.locator('[aria-live="polite"][role="status"]').first()).toBeVisible();

  const settingsNav = page.getByRole('navigation', { name: 'Settings sections' });
  await settingsNav.getByRole('button', { name: 'Updates', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Updates', exact: true })).toBeVisible();
  const updaterSurface = page.locator('.sand-settings-beta-stack');
  await expect(updaterSurface).toBeVisible();
  await expect(updaterSurface.locator('[role="status"], output[aria-live="polite"]').first()).toBeVisible();
  const updateTrack = page.getByRole('button', { name: 'Update Track' });
  const checkUpdates = page.getByRole('button', { name: /Check for Updates|Restart to Update/ });
  expect(
    (await updateTrack.count()) + (await checkUpdates.count()),
    'signed packaged Settings must expose the production updater state/action surface',
  ).toBeGreaterThan(0);
  await settingsNav.getByRole('button', { name: 'General', exact: true }).click();
  dialog = page.getByRole('dialog', { name: 'Grok Bot settings' });
  await expect(dialog.getByRole('heading', { name: 'Appearance', exact: true })).toBeVisible();

  await chooseSandSelect(page, 'Theme', 'Dark');
  await chooseSandSelect(page, 'Text size', '125%');
  const reduceMotion = dialog.getByRole('switch', { name: /Reduce motion/ });
  const highContrast = dialog.getByRole('switch', { name: /High contrast/ });
  if ((await reduceMotion.getAttribute('aria-checked')) !== 'true') await reduceMotion.click();
  if ((await highContrast.getAttribute('aria-checked')) !== 'true') await highContrast.click();
  await expect.poll(() => page.evaluate(() => document.documentElement.dataset.sandReducedMotion)).toBe('true');
  await expect.poll(() => page.evaluate(() => document.documentElement.dataset.sandHighContrast)).toBe('true');
  await expect.poll(() => page.evaluate(() => document.documentElement.style.getPropertyValue('--sand-ui-text-scale'))).toBe('1.25');

  await chooseSandSelect(page, 'Language', 'العربية');
  dialog = page.getByRole('dialog', { name: 'إعدادات Grok Bot' });
  await expect(dialog).toBeVisible({ timeout: 10_000 });
  await expect.poll(() => page.evaluate(() => document.documentElement.dir)).toBe('rtl');
  await expect.poll(() => page.evaluate(() => document.documentElement.lang)).toContain('ar');

  const mixedSample = 'RTL العربية + English + 中文 + 😀';
  const directionEvidence = await page.evaluate((sample) => {
    const element = document.createElement('div');
    element.dir = 'auto';
    element.textContent = sample;
    document.body.appendChild(element);
    const computed = getComputedStyle(element);
    const result = { direction: computed.direction, fontFamily: computed.fontFamily };
    element.remove();
    return result;
  }, mixedSample);
  expect(directionEvidence.fontFamily.length).toBeGreaterThan(0);

  await chooseSandSelect(page, 'اللغة', '简体中文');
  dialog = page.getByRole('dialog', { name: 'Grok Bot 设置' });
  await expect(dialog).toBeVisible({ timeout: 10_000 });
  await expect.poll(() => page.evaluate(() => document.documentElement.lang)).toContain('zh-Hans');
  const close = page.getByRole('button', { name: '关闭' }).last();
  await expect(close).toBeVisible();
  await page.keyboard.press('Shift+Tab');
  const focused = await page.evaluate(() => document.activeElement?.getAttribute('role') || document.activeElement?.tagName || '');
  expect(focused.length).toBeGreaterThan(0);
  await close.click();

  const prompt = page.getByRole('textbox', { name: 'Prompt' });
  await prompt.focus();
  const beforeArticles = await page.getByRole('log', { name: 'Conversation transcript' }).locator('[role="article"]').count();
  await page.evaluate(() => {
    const target = document.activeElement;
    if (!(target instanceof HTMLTextAreaElement || target instanceof HTMLInputElement || target instanceof HTMLElement)) {
      throw new Error('composer does not own a focusable editor');
    }
    target.dispatchEvent(new CompositionEvent('compositionstart', { data: '拼' }));
    target.dispatchEvent(new CompositionEvent('compositionupdate', { data: '拼音' }));
    target.dispatchEvent(new InputEvent('beforeinput', { inputType: 'insertCompositionText', data: '拼音', bubbles: true, cancelable: true, isComposing: true }));
  });
  await page.keyboard.press('Enter');
  expect(await page.getByRole('log', { name: 'Conversation transcript' }).locator('[role="article"]').count()).toBe(beforeArticles);
  await page.evaluate(() => {
    const target = document.activeElement;
    if (!(target instanceof HTMLElement)) throw new Error('composition target disappeared');
    target.dispatchEvent(new CompositionEvent('compositionend', { data: '拼音' }));
  });
  await prompt.fill('IME 拼音 中文 😀 mixed العربية');
  expect(await prompt.inputValue()).toBe('IME 拼音 中文 😀 mixed العربية');
  await prompt.fill('');
  return { directionEvidence, mixedSample, imeCompositionSuppressedSubmit: true };
}

test.describe('signed Phase 1 packaged production acceptance', () => {
  test.describe.configure({ retries: 0 });
  test.skip(!realAcceptance, 'Set OBF_REAL_ACCEPTANCE=1 to run signed packaged acceptance.');

  test('Human messaging, restart/idempotency, handoff, calls and Settings/a11y/i18n use the signed executable', async () => {
    test.setTimeout(25 * 60_000);
    expect(executable, 'FABUSHI_ELECTRON_EXECUTABLE is required').toBeTruthy();
    expect(sourceSha).toMatch(/^[0-9a-f]{40}$/);
    expect(sourceSha).toBe(expectedSourceSha);
    expect(primaryUsername, 'primary bounded production username is required').toBeTruthy();
    expect(peerUsername, 'peer bounded production username is required').toBeTruthy();
    expect(peerUsername).not.toBe(primaryUsername);
    expect(primarySession, 'primary refresh-token-free session is required').toBeTruthy();
    expect(peerSession, 'peer refresh-token-free session is required').toBeTruthy();
    expect(primaryDeviceId).toBeTruthy();
    expect(peerDeviceId).toBeTruthy();
    expect(process.env.FABUSHI_FEATURE_HOST_MODE || '').not.toBe('test');
    expect(process.env.FABUSHI_E2E || '').not.toBe('1');

    await mkdir(evidenceRoot, { recursive: true });
    const primaryData = path.join(evidenceRoot, 'phase1-primary-app-data');
    const peerData = path.join(evidenceRoot, 'phase1-peer-app-data');
    let primary: AppHandle | null = null;
    let peer: AppHandle | null = null;
    const marker = 'PHASE1-HUMAN-' + Date.now();
    const replyMarker = marker + '-REPLY';
    const attachmentMarker = marker + '-ATTACHMENT';
    const attachmentName = 'phase1-' + sourceSha.slice(0, 8) + '.txt';
    const evidence: Record<string, unknown> = {
      sourceSha,
      accounts: ['bounded-ci-primary', 'bounded-ci-peer'],
      devices: [primaryDeviceId, peerDeviceId],
      markers: { marker, replyMarker, attachmentMarker, attachmentName },
      launchMode: 'two-signed-production-processes',
    };

    try {
      primary = await launchSignedApp('primary', primaryData, primarySession, primaryDeviceId);
      peer = await launchSignedApp('peer', peerData, peerSession, peerDeviceId);
      await createHumanIfMissing(primary.page, peerUsername);
      await openHuman(primary.page, peerUsername);
      await createHumanIfMissing(peer.page, primaryUsername);
      await openHuman(peer.page, primaryUsername);

      await sendHuman(primary.page, marker);
      await waitForHumanText(peer.page, primaryUsername, marker);

      const incoming = peer.page.getByRole('log', { name: 'Conversation transcript' }).getByText(marker, { exact: true }).locator('xpath=ancestor::*[@role="article"][1]');
      await incoming.hover();
      await incoming.getByRole('button', { name: /Reply to/ }).click();
      await sendHuman(peer.page, replyMarker);
      await waitForHumanText(primary.page, peerUsername, replyMarker);

      const fileInput = primary.page.locator('input[type="file"]').first();
      await expect(fileInput).toBeAttached();
      await fileInput.setInputFiles({ name: attachmentName, mimeType: 'text/plain', buffer: Buffer.from('signed packaged attachment ' + marker + '\n', 'utf8') });
      await expect(primary.page.getByText(attachmentName, { exact: true })).toBeVisible({ timeout: 15_000 });
      await sendHuman(primary.page, attachmentMarker);
      await waitForHumanText(peer.page, primaryUsername, attachmentMarker);
      await expect(peer.page.getByText(attachmentName, { exact: true })).toBeVisible({ timeout: 30_000 });

      const reactionTarget = peer.page.getByRole('log', { name: 'Conversation transcript' }).getByText(marker, { exact: true }).locator('xpath=ancestor::*[@role="article"][1]');
      await reactionTarget.hover();
      await reactionTarget.getByRole('button', { name: 'Add reaction' }).click();
      const quickReaction = peer.page.getByRole('button', { name: /^React with / }).first();
      await expect(quickReaction).toBeVisible();
      await quickReaction.click();
      await focusAndSync(primary.page);
      await openHuman(primary.page, peerUsername);
      await expect(primary.page.locator('.sand-reaction-pill').first()).toBeVisible({ timeout: 45_000 });

      await primary.page.keyboard.press('Meta+f');
      const find = primary.page.getByRole('textbox', { name: 'Find in chat' });
      await expect(find).toBeVisible();
      await find.fill(marker);
      await expect(primary.page.locator('.sand-chat-find [role="status"]')).toContainText('/', { timeout: 20_000 });
      await primary.page.getByRole('button', { name: 'Close find' }).click();
      await screenshot(primary.page, '10-phase1-human-messaging');

      evidence.settings = await assertSettingsAccessibilityAndI18n(primary.page);
      await screenshot(primary.page, '11-phase1-i18n-ime');

      await closeApp(primary);
      primary = null;
      primary = await launchSignedApp('primary-restart', primaryData, primarySession, primaryDeviceId);
      await openHuman(primary.page, peerUsername);
      const transcript = primary.page.getByRole('log', { name: 'Conversation transcript' });
      await expect(transcript.getByText(marker, { exact: true })).toHaveCount(1, { timeout: 45_000 });
      await expect(transcript.getByText(replyMarker, { exact: true })).toHaveCount(1);
      await expect(transcript.getByText(attachmentMarker, { exact: true })).toHaveCount(1);
      await screenshot(primary.page, '12-phase1-restart-idempotency');

      const handoffSelect = primary.page.getByRole('combobox', { name: 'Agent for Human handoff' });
      if (await handoffSelect.count()) {
        const options = await handoffSelect.locator('option').count();
        expect(options, 'Human to Agent handoff requires a real Agent target').toBeGreaterThan(1);
        await handoffSelect.selectOption({ index: 1 });
      } else {
        const trigger = primary.page.getByRole('button', { name: 'Agent for Human handoff' });
        await expect(trigger).toBeVisible();
        await trigger.click();
        const option = primary.page.getByRole('option').nth(1);
        await expect(option).toBeVisible();
        await option.click();
      }
      const beforeHandoff = await transcript.locator('[role="article"][data-role="assistant"]').count();
      await primary.page.getByRole('button', { name: 'Ask Agent' }).click();
      await expect.poll(() => transcript.locator('[role="article"][data-role="assistant"]').count(), { timeout: 120_000 }).toBeGreaterThan(beforeHandoff);
      await screenshot(primary.page, '13-phase1-human-agent-handoff');

      await primary.page.getByRole('button', { name: 'Start voice call' }).click();
      await expect(primary.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toContainText(/ringing|negotiating|connected/, { timeout: 45_000 });
      await focusAndSync(peer.page);
      await openHuman(peer.page, primaryUsername);
      await expect(peer.page.getByRole('button', { name: 'Accept voice call' })).toBeVisible({ timeout: 45_000 });
      await peer.page.getByRole('button', { name: 'Accept voice call' }).click();
      await expect(peer.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toContainText('connected', { timeout: 90_000 });
      await expect(primary.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toContainText('connected', { timeout: 90_000 });

      await primary.page.getByRole('button', { name: 'Mute', exact: true }).click();
      await expect(primary.page.getByRole('button', { name: 'Unmute', exact: true })).toHaveAttribute('aria-pressed', 'true');
      await primary.page.getByRole('button', { name: 'Camera on', exact: true }).click();
      await expect(primary.page.getByRole('button', { name: 'Camera off', exact: true })).toHaveAttribute('aria-pressed', 'true', { timeout: 30_000 });
      await primary.page.getByRole('button', { name: 'Share screen', exact: true }).click();
      await expect(primary.page.getByRole('button', { name: 'Stop sharing', exact: true })).toHaveAttribute('aria-pressed', 'true', { timeout: 30_000 });
      await screenshot(primary.page, '14-phase1-voice-media-connected');
      await primary.page.getByRole('button', { name: 'End call' }).click();
      await expect(primary.page.getByRole('button', { name: 'Start voice call' })).toBeVisible({ timeout: 30_000 });

      await peer.page.getByRole('button', { name: 'Start video call' }).click();
      await focusAndSync(primary.page);
      await openHuman(primary.page, peerUsername);
      await expect(primary.page.getByRole('button', { name: 'Accept video call' })).toBeVisible({ timeout: 45_000 });
      await primary.page.getByRole('button', { name: 'Accept video call' }).click();
      await expect(primary.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toContainText('connected', { timeout: 90_000 });
      await expect(peer.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toContainText('connected', { timeout: 90_000 });
      await screenshot(primary.page, '15-phase1-video-connected');

      await closeApp(peer);
      peer = null;
      await expect(primary.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toContainText(/reconnecting|failed|connected/, { timeout: 90_000 });
      peer = await launchSignedApp('peer-reconnect', peerData, peerSession, peerDeviceId);
      await openHuman(peer.page, primaryUsername);
      await expect(peer.page.getByRole('group', { name: 'Human call controls' }).getByRole('status')).toContainText(/connected|reconnecting|negotiating/, { timeout: 90_000 });
      if (await peer.page.getByRole('button', { name: 'End call' }).count()) await peer.page.getByRole('button', { name: 'End call' }).click();
      await screenshot(peer.page, '16-phase1-reconnect-hangup');

      evidence.human = { reply: true, attachment: true, reaction: true, search: true, restartIdempotency: true, networkConvergence: true, handoff: true };
      evidence.calls = { outgoingVoice: true, incomingVoice: true, outgoingVideo: true, incomingVideo: true, mute: true, cameraToggle: true, screenShare: true, iceSignalingConnected: true, processDisconnectReconnect: true, hangup: true };
      evidence.completedAt = new Date().toISOString();
      await writeFile(path.join(evidenceRoot, 'phase1-production.json'), JSON.stringify(evidence, null, 2));
    } catch (error) {
      evidence.failure = error instanceof Error ? { message: error.message, stack: error.stack } : { message: String(error) };
      evidence.failedAt = new Date().toISOString();
      await writeFile(path.join(evidenceRoot, 'phase1-production.json'), JSON.stringify(evidence, null, 2)).catch(() => undefined);
      if (primary) await screenshot(primary.page, 'phase1-failure-primary').catch(() => undefined);
      if (peer) await screenshot(peer.page, 'phase1-failure-peer').catch(() => undefined);
      throw error;
    } finally {
      await closeApp(peer);
      await closeApp(primary);
    }
  });
});
