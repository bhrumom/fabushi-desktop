import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { AgentWorkspaceController } from '../src/agent-workspace/agent-workspace-controller';

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

function rgbLuma(value: string): number {
  const components = value.match(/[\d.]+/g)?.slice(0, 3).map(Number) ?? [];
  if (components.length !== 3) return 255;
  return components[0] * 0.2126 + components[1] * 0.7152 + components[2] * 0.0722;
}

test('desktop uses the Fabushi-owned Grok parity surface without a parallel Messenger', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-grok-parity-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();

    await test.step('Agent controller keeps simultaneous requests isolated by peer', async () => {
      const controller = new AgentWorkspaceController();
      controller.beginRequest('agent:a', 'request:a');
      controller.beginRequest('agent:b', 'request:b');
      expect(controller.onlyPendingPeer()).toBeNull();
      expect(controller.requestSnapshot()).toEqual({
        'agent:a': 'request:a',
        'agent:b': 'request:b',
      });

      controller.adoptOperation('request:a', 'operation:a', 'agent:a');
      expect(controller.operationForPeer('agent:a')).toBe('operation:a');
      expect(controller.requestForPeer('agent:b')).toBe('request:b');
      expect(controller.isBusy('agent:a')).toBe(true);
      expect(controller.isBusy('agent:b')).toBe(true);

      controller.finishOperation('operation:a');
      expect(controller.isBusy('agent:a')).toBe(false);
      expect(controller.isBusy('agent:b')).toBe(true);
      expect(controller.onlyPendingPeer()).toBe('agent:b');

      controller.setDraft('agent:a', 'first prompt');
      controller.appendAttachments('agent:a', [{ id: 'attachment:a', name: 'a.txt' }]);
      controller.setReply('agent:a', { id: 'reply:a', role: 'peer', text: 'previous answer' });
      controller.setDraft('agent:b', 'independent draft');

      const submitted = controller.takeDraft('agent:a');
      expect(submitted.text).toBe('first prompt');
      expect(submitted.attachments.map((attachment) => attachment.id)).toEqual(['attachment:a']);
      expect(submitted.replyTo?.id).toBe('reply:a');
      expect(controller.draftForPeer('agent:a')).toBe('');
      expect(controller.draftForPeer('agent:b')).toBe('independent draft');

      controller.setDraft('agent:a', 'newer draft typed while the send was pending');
      controller.restoreDraft('agent:a', submitted);
      expect(controller.draftForPeer('agent:a')).toBe('newer draft typed while the send was pending');
      expect(controller.attachmentsForPeer('agent:a').map((attachment) => attachment.id)).toEqual(['attachment:a']);
      expect(controller.replyForPeer('agent:a')?.id).toBe('reply:a');
    });

    await test.step('parity stylesheet and surface marker load before authentication', async () => {
      await expect(page.locator('body')).toHaveAttribute('data-fabushi-surface', 'grok-parity-v1');
      const parityLoaded = await page.evaluate(() => Array.from(document.styleSheets).some((sheet) =>
        String(sheet.href ?? '').includes('grok-parity.css')));
      expect(parityLoaded).toBe(true);
      const bodyBackground = await page.locator('body').evaluate((element) => getComputedStyle(element).backgroundColor);
      expect(rgbLuma(bodyBackground)).toBeLessThan(40);
    });

    await completeBrowserLogin(page);

    await test.step('canonical Agent workspace replaces the Messenger navigation shell', async () => {
      await expect(page.getByTestId('messenger-workspace')).toHaveCount(1);
      await expect(page.getByTestId('messenger-workspace')).toHaveAttribute('data-agent-root-shell', 'true');
      await expect(page.getByTestId('messenger-workspace')).toHaveAttribute('data-product-shell', 'agent');
      await expect(page.locator('.desktop-mode-switch')).toHaveCount(0);
      await expect(page.getByTestId('grok-new-agent')).toBeVisible();
      await expect(page.getByTestId('profile-navigation-trigger')).toHaveCount(0);
      await expect(page.locator('[data-testid^="legacy-peer-"]')).toHaveCount(0);

      await page.keyboard.press(process.platform === 'darwin' ? 'Meta+K' : 'Control+K');
      await expect(page.getByRole('dialog', { name: 'Command palette' })).toBeVisible();
      await expect(page.getByPlaceholder('Search agents or run a command')).toBeFocused();
      await page.keyboard.press('Escape');
      await expect(page.getByRole('dialog', { name: 'Command palette' })).toBeHidden();
    });

    await test.step('Agent conversation and composer expose dark low-contrast material', async () => {
      const peer = page.getByTestId('peer-legacy:conversation:mahayana-ai:agent:assistant');
      await expect(peer).toBeVisible();
      await peer.click();
      const input = page.getByTestId('messenger-input');
      await expect(input).toBeVisible();

      const material = await page.evaluate(() => {
        const inputElement = document.querySelector('[data-testid="messenger-input"]');
        const composer = inputElement?.closest('[data-testid="grok-agent-composer"]');
        const peerElement = document.querySelector('[data-testid="peer-legacy:conversation:mahayana-ai:agent:assistant"]');
        if (!composer || !peerElement) return null;
        const composerStyle = getComputedStyle(composer);
        const peerStyle = getComputedStyle(peerElement);
        return {
          composerBackground: composerStyle.backgroundColor,
          composerRadius: composerStyle.borderRadius,
          peerBackground: peerStyle.backgroundColor,
          peerRadius: peerStyle.borderRadius,
        };
      });

      expect(material).not.toBeNull();
      expect(rgbLuma(material!.composerBackground)).toBeLessThan(70);
      expect(parseFloat(material!.composerRadius)).toBeGreaterThanOrEqual(14);
      expect(rgbLuma(material!.peerBackground)).toBeLessThan(80);
      expect(parseFloat(material!.peerRadius)).toBeGreaterThanOrEqual(10);
    });

    await test.step('Agent uses one canonical workspace and Agent-scoped attachment draft', async () => {
      const composer = page.getByTestId('grok-agent-composer');
      await expect(composer).toHaveCount(1);
      await expect(page.getByTestId('message-list')).toHaveCount(1);

      const fileInput = composer.locator('input[type="file"]');
      await expect(fileInput).toHaveAttribute('multiple', '');
      await fileInput.setInputFiles({
        name: 'agent-notes.txt',
        mimeType: 'text/plain',
        buffer: Buffer.from('Agent-owned attachment context'),
      });
      await expect(composer.getByText('agent-notes.txt')).toBeVisible();

      const input = page.getByTestId('messenger-input');
      await expect(input).toHaveAttribute('contenteditable', 'true');
      await input.fill('Use the attached note.');
      await page.getByTestId('messenger-send').click();

      await expect(composer.getByText('agent-notes.txt')).toHaveCount(0);
      const firstUserTurn = page.locator('[data-agent-message-role="me"]').filter({ hasText: 'Use the attached note.' });
      await expect(firstUserTurn).toHaveCount(1);
      await expect(firstUserTurn.getByText('agent-notes.txt')).toBeVisible();

      await fileInput.setInputFiles({
        name: 'attachment-only.txt',
        mimeType: 'text/plain',
        buffer: Buffer.from('Attachment-only Agent submission'),
      });
      await expect(page.getByTestId('messenger-send')).toBeVisible();
      await page.getByTestId('messenger-send').click();
      await expect(page.getByTestId('message-list').getByText('attachment-only.txt')).toBeVisible();
    });

    await test.step('Agent sidebar supports modifier selection and account-scoped sections', async () => {
      const peer = page.getByTestId('peer-legacy:conversation:mahayana-ai:agent:assistant');
      await peer.click({ modifiers: [process.platform === 'darwin' ? 'Meta' : 'Control'] });
      const selectionBar = page.getByTestId('agent-selection-bar');
      await expect(selectionBar).toBeVisible();
      await expect(selectionBar).toContainText('1 selected');

      page.once('dialog', async (dialog) => {
        expect(dialog.type()).toBe('prompt');
        await dialog.accept('Focused work');
      });
      await selectionBar.getByRole('button', { name: 'Section' }).click();
      await expect(page.locator('[data-section-id]').filter({ hasText: 'Focused work' })).toBeVisible();
      await expect(selectionBar).toHaveCount(0);
    });
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
