import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { AgentTranscriptStore } from '../src/agent-workspace/agent-transcript-store';
import { AgentWorkspaceController } from '../src/agent-workspace/agent-workspace-controller';
import { AgentRuntimeCoordinator } from '../src/agent-workspace/agent-runtime-coordinator';
import { composeAgentPromptText } from '../src/agent-workspace/prompt-context';
import { restoreAgentStoreWorkspace } from '../src/agent-workspace/agent-store-recovery';
import type { TranscriptEntry } from '../src/agent-workspace/transcript-model';
import {
  FABU_AGENT_ATTACHMENT_INDEX_PATH,
  FABU_AGENT_ROOT_PATH,
  FABU_AGENT_RUNTIME_CHECKPOINT_PATH,
  FabuAgentStore,
  fabuAgentConversationTranscriptPath,
} from '../src/fabu-runtime/agent-store';

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

function primaryMahayanaAgentPeer(page: Page) {
  return page.getByTestId('messenger-sidebar').locator('button[data-agent-id="mahayana-assistant"]');
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

      const claimedPeer = controller.claimRuntimeOperation('operation:b', controller.peerForRequest('request:b'));
      expect(claimedPeer).toBe('agent:b');
      expect(controller.peerForRuntimeId('operation:b')).toBe('agent:b');
      expect(controller.requestForPeer('agent:b')).toBeNull();
      expect(controller.finishRuntimeOperation('operation:b')).toBe('agent:b');
      expect(controller.isOperationFinished('operation:b')).toBe(true);
      expect(controller.claimRuntimeOperation('operation:b', 'agent:b')).toBeNull();

      controller.setDraft('agent:a', 'first prompt');
      controller.appendAttachments('agent:a', [{ id: 'attachment:a', name: 'a.txt' }]);
      controller.setReply('agent:a', { id: 'reply:a', role: 'peer', text: 'previous answer' });
      controller.upsertReference('agent:a', { kind: 'agent', id: 'agent:research', label: 'Research' });
      controller.upsertReference('agent:a', { kind: 'workflow', id: 'workflow:review', label: 'Review changes' });
      controller.setDraft('agent:b', 'independent draft');

      const submitted = controller.takeDraft('agent:a');
      expect(submitted.text).toBe('first prompt');
      expect(submitted.attachments.map((attachment) => attachment.id)).toEqual(['attachment:a']);
      expect(submitted.replyTo?.id).toBe('reply:a');
      expect(submitted.references).toEqual([
        { kind: 'agent', id: 'agent:research', label: 'Research' },
        { kind: 'workflow', id: 'workflow:review', label: 'Review changes' },
      ]);
      const composedPrompt = composeAgentPromptText(submitted.text, submitted.replyTo, submitted.references);
      expect(composedPrompt).toContain('@Research [agent:agent:research]');
      expect(composedPrompt).toContain('/Review changes [workflow:workflow:review]');
      expect(controller.draftForPeer('agent:a')).toBe('');
      expect(controller.draftForPeer('agent:b')).toBe('independent draft');

      controller.setDraft('agent:a', 'newer draft typed while the send was pending');
      controller.restoreDraft('agent:a', submitted);
      expect(controller.draftForPeer('agent:a')).toBe('newer draft typed while the send was pending');
      expect(controller.attachmentsForPeer('agent:a').map((attachment) => attachment.id)).toEqual(['attachment:a']);
      expect(controller.replyForPeer('agent:a')?.id).toBe('reply:a');
      expect(controller.referencesForPeer('agent:a')).toEqual([
        { kind: 'agent', id: 'agent:research', label: 'Research' },
        { kind: 'workflow', id: 'workflow:review', label: 'Review changes' },
      ]);
      controller.setDraft('agent:a', 'new draft without a mention');
      controller.pruneReferences('agent:a', 'new draft without a mention');
      expect(controller.referencesForPeer('agent:a')).toEqual([]);
    });

    await test.step('Agent Store root restores a cross-device transcript snapshot without last-write-wins guessing', async () => {
      const agentId = 'agent:cloud';
      const conversationId = 'conversation:cloud';
      const transcriptPath = fabuAgentConversationTranscriptPath(conversationId);
      const entries: TranscriptEntry[] = [{
        id: 'cloud:user:1',
        kind: 'message',
        role: 'me',
        text: 'restored prompt',
        createdAtMs: 1,
      }, {
        id: 'cloud:assistant:1',
        kind: 'message',
        role: 'peer',
        text: 'restored answer',
        createdAtMs: 2,
      }];
      const encode = (value: unknown) => Buffer.from(JSON.stringify(value), 'utf8').toString('base64');
      const objects = new Map([
        [FABU_AGENT_ROOT_PATH, {
          path: FABU_AGENT_ROOT_PATH,
          etag: 'root-etag',
          dataBase64: encode({
            schemaVersion: 1,
            agentId,
            updatedAtMs: 3,
            files: [{ path: transcriptPath, blobId: 'blob-transcript', etag: 'transcript-etag', revision: 4 }],
          }),
        }],
        [transcriptPath, {
          path: transcriptPath,
          blobId: 'blob-transcript',
          etag: 'transcript-etag',
          revision: 4,
          dataBase64: encode({ schemaVersion: 1, agentId, conversationId, entries, updatedAtMs: 3 }),
        }],
      ]);
      objects.set(FABU_AGENT_ATTACHMENT_INDEX_PATH, {
        path: FABU_AGENT_ATTACHMENT_INDEX_PATH,
        blobId: 'blob-attachments',
        etag: 'attachments-etag',
        revision: 2,
        dataBase64: encode({
          schemaVersion: 1,
          agentId,
          attachments: [{ id: 'attachment:cloud', name: 'cloud.txt', path: '/agent/cloud.txt', sizeBytes: 12 }],
        }),
      });
      objects.set(FABU_AGENT_RUNTIME_CHECKPOINT_PATH, {
        path: FABU_AGENT_RUNTIME_CHECKPOINT_PATH,
        blobId: 'blob-checkpoint',
        etag: 'checkpoint-etag',
        revision: 3,
        dataBase64: encode({
          schemaVersion: 1,
          agentId,
          conversationId,
          operationId: 'operation:cloud-stale',
          status: 'running',
          updatedAtMs: 3,
        }),
      });
      objects.set(FABU_AGENT_ROOT_PATH, {
        ...objects.get(FABU_AGENT_ROOT_PATH)!,
        dataBase64: encode({
          schemaVersion: 1,
          agentId,
          updatedAtMs: 4,
          files: [
            { path: transcriptPath, blobId: 'blob-transcript', etag: 'transcript-etag', revision: 4 },
            { path: FABU_AGENT_ATTACHMENT_INDEX_PATH, blobId: 'blob-attachments', etag: 'attachments-etag', revision: 2 },
            { path: FABU_AGENT_RUNTIME_CHECKPOINT_PATH, blobId: 'blob-checkpoint', etag: 'checkpoint-etag', revision: 3 },
          ],
        }),
      });
      const store = new FabuAgentStore(agentId, {
        async list() { return { files: [] }; },
        async read(_agentId, path) {
          const object = objects.get(path);
          if (!object) throw new Error(`missing ${path}`);
          return object;
        },
        async write() { return {}; },
        async delete() { return {}; },
      });
      const recovered = await restoreAgentStoreWorkspace(store, conversationId);
      expect(recovered.attachments.map((attachment) => attachment.id)).toEqual(['attachment:cloud']);
      expect(recovered.checkpoint?.operationId).toBe('operation:cloud-stale');
      expect(recovered.entries.filter((entry) => entry.kind === 'notice')).toHaveLength(1);
      expect(recovered.entries.find((entry) => entry.kind === 'notice')?.status).toBe('interrupted');
      const transcripts = new AgentTranscriptStore();
      transcripts.hydrateEntries('agent:cloud-peer', recovered.entries);
      expect(transcripts.entries('agent:cloud-peer').map((entry) => entry.text)).toEqual(['restored prompt', 'restored answer', '']);

      const corruptObjects = new Map(objects);
      corruptObjects.set(transcriptPath, {
        ...corruptObjects.get(transcriptPath)!,
        etag: 'unexpected-etag',
      });
      const corruptStore = new FabuAgentStore(agentId, {
        async list() { return { files: [] }; },
        async read(_agentId, path) {
          const object = corruptObjects.get(path);
          if (!object) throw new Error(`missing ${path}`);
          return object;
        },
        async write() { return {}; },
        async delete() { return {}; },
      });
      await expect(restoreAgentStoreWorkspace(corruptStore, conversationId)).rejects.toThrow('Agent Store etag mismatch');
    });

    await test.step('Agent runtime coordinator isolates concurrent Agent streams and preserves drafts on reconnect', async () => {
      const controller = new AgentWorkspaceController();
      const transcripts = new AgentTranscriptStore();
      const computerStatuses: import('../../frontend/apps/web/src/lib/mahayana-host/contracts').ComputerStatus[] = [];
      const coordinator = new AgentRuntimeCoordinator(controller, transcripts, {
        onComputerStatus: (status) => { computerStatuses.push(status); },
      });
      expect(coordinator.handle({
        type: 'computer.status',
        timestamp: new Date(1).toISOString(),
        requestId: 'computer-status:test',
        status: {
          platform: 'macos',
          available: true,
          captureSupported: true,
          inputSupported: true,
          accessibilityGranted: false,
          screenRecordingGranted: true,
          localExecutionEnabled: true,
          routeEgressLocally: true,
          remoteControlEnabled: false,
          aiControlEnabled: true,
        },
      })).toBe(true);
      expect(computerStatuses[0]?.platform).toBe('macos');
      expect(computerStatuses[0]?.accessibilityGranted).toBe(false);
      coordinator.bindAgentPeers([{ agentId: 'agent:runtime-b', peerKey: 'agent:b' }]);
      expect(coordinator.handle({
        type: 'computer.result',
        timestamp: new Date(1).toISOString(),
        requestId: 'computer:b',
        agentId: 'agent:runtime-b',
        result: {
          origin: 'ai',
          actionsExecuted: 1,
          snapshot: { capturedAtMs: 1, dataUrl: 'data:image/png;base64,AA==' },
        },
      })).toBe(true);
      expect(transcripts.entries('agent:b').filter((entry) => entry.kind === 'computer-handoff')).toHaveLength(1);
      expect(transcripts.entries('agent:a').filter((entry) => entry.kind === 'computer-handoff')).toHaveLength(0);
      expect(coordinator.handle({
        type: 'computer.result',
        timestamp: new Date(1).toISOString(),
        requestId: 'computer:unscoped',
        result: {
          origin: 'local-ui',
          actionsExecuted: 1,
          snapshot: { capturedAtMs: 1, dataUrl: 'data:image/png;base64,AA==' },
        },
      })).toBe(false);

      controller.setDraft('agent:a', 'draft survives reconnect');
      coordinator.beginLocalTurn({
        peerKey: 'agent:a',
        requestId: 'request:a',
        messageId: 'user:a',
        text: 'A',
        createdAtMs: 1,
      });
      coordinator.beginLocalTurn({
        peerKey: 'agent:b',
        requestId: 'request:b',
        messageId: 'user:b',
        text: 'B',
        createdAtMs: 2,
      });

      coordinator.adoptOperation('request:a', 'operation:a', 'agent:a');
      coordinator.adoptOperation('request:b', 'operation:b', 'agent:b');
      expect(controller.operationForPeer('agent:a')).toBe('operation:a');
      expect(controller.operationForPeer('agent:b')).toBe('operation:b');

      expect(coordinator.handle({
        type: 'chat.delta',
        timestamp: new Date(3).toISOString(),
        operationId: 'operation:a',
        delta: 'alpha',
      })).toBe(true);
      expect(coordinator.handle({
        type: 'chat.delta',
        timestamp: new Date(4).toISOString(),
        operationId: 'operation:b',
        delta: 'beta',
      })).toBe(true);
      coordinator.flushPendingDeltas();

      expect(coordinator.handle({
        type: 'approval.requested',
        timestamp: new Date(4).toISOString(),
        operationId: 'operation:b',
        agentId: 'agent:b',
        approvalId: 'approval:b',
        miniAppId: 'runtime',
        capability: 'filesystem.write',
        reason: 'Write the requested file.',
      })).toBe(true);
      expect(transcripts.entries('agent:b').filter((entry) => entry.kind === 'approval')).toHaveLength(1);
      expect(transcripts.entries('agent:a').filter((entry) => entry.kind === 'approval')).toHaveLength(0);
      expect(transcripts.entries('agent:b').find((entry) => entry.kind === 'approval')?.approval?.approvalId).toBe('approval:b');

      expect(coordinator.handle({
        type: 'approval.resolved',
        timestamp: new Date(4).toISOString(),
        operationId: 'operation:b',
        agentId: 'agent:b',
        approvalId: 'approval:b',
        decision: 'allow-once',
      })).toBe(true);
      expect(transcripts.entries('agent:b').find((entry) => entry.kind === 'approval')?.approval?.decision).toBe('allow-once');

      expect(coordinator.handle({
        type: 'operation.completed',
        timestamp: new Date(5).toISOString(),
        operationId: 'operation:a',
      })).toBe(true);
      expect(controller.isBusy('agent:a')).toBe(false);
      expect(controller.isBusy('agent:b')).toBe(true);
      expect(transcripts.entries('agent:a').map((entry) => entry.text).join(' ')).toContain('alpha');
      expect(transcripts.entries('agent:b').map((entry) => entry.text).join(' ')).toContain('beta');
      expect(transcripts.entries('agent:a').map((entry) => entry.text).join(' ')).not.toContain('beta');

      coordinator.resetOperations();
      expect(controller.draftForPeer('agent:a')).toBe('draft survives reconnect');
      expect(controller.isBusy('agent:b')).toBe(false);

      coordinator.beginLocalTurn({
        peerKey: 'agent:restart',
        requestId: 'request:restart',
        messageId: 'user:restart',
        text: 'survive host restart',
        createdAtMs: 6,
      });
      coordinator.adoptOperation('request:restart', 'operation:restart', 'agent:restart');
      expect(coordinator.handle({
        type: 'host.lifecycle',
        timestamp: new Date(7).toISOString(),
        lifecycle: 'stopped',
        state: 'stopped',
        generation: 9,
        sequence: 20,
        recoverable: true,
        error: 'fault injection',
      })).toBe(true);
      expect(controller.isBusy('agent:restart')).toBe(false);
      expect(controller.isOperationFinished('operation:restart')).toBe(true);
      expect(transcripts.entries('agent:restart').find((entry) => entry.kind === 'assistant-turn')?.assistantTurn?.status).toBe('interrupted');
      expect(controller.draftForPeer('agent:a')).toBe('draft survives reconnect');

      expect(coordinator.handleCommandBridge({
        phase: 'dispatch',
        command: { type: 'chat.send', requestId: 'request:c', text: 'C' },
        context: { conversationKey: 'agent:c' },
      })).toBe(true);
      expect(controller.requestForPeer('agent:c')).toBe('request:c');
      expect(coordinator.handleCommandBridge({
        phase: 'accepted',
        command: { type: 'chat.send', requestId: 'request:c', text: 'C' },
        accepted: { requestId: 'request:c', operationId: 'operation:c' },
        context: { conversationKey: 'agent:c' },
      })).toBe(true);
      expect(controller.operationForPeer('agent:c')).toBe('operation:c');
      expect(transcripts.entries('agent:c').filter((entry) => entry.kind === 'assistant-turn')).toHaveLength(1);

      transcripts.appendUserMessage('agent:q', {
        id: 'queued:q',
        text: 'queued once',
        createdAtMs: 10,
        optimistic: true,
        queued: true,
      });
      expect(transcripts.thread('agent:q').filter((message) => message.queued)).toHaveLength(1);
      coordinator.beginLocalTurn({
        peerKey: 'agent:q',
        requestId: 'request:q',
        messageId: 'queued:q',
        text: 'queued once',
        createdAtMs: 11,
      });
      transcripts.removeQueuedUserMessage('agent:q', 'queued:q');
      expect(transcripts.thread('agent:q').filter((message) => message.id === 'queued:q')).toHaveLength(1);
      expect(transcripts.thread('agent:q').find((message) => message.id === 'queued:q')?.queued).toBe(false);

      coordinator.dispose();
    });

    await test.step('Agent transcript store keeps one ordered assistant turn through adoption and finalization', async () => {
      const transcripts = new AgentTranscriptStore();
      transcripts.replace('agent:a', [{
        id: 'user:a',
        source: 'legacy',
        role: 'me',
        text: '你好',
        createdAtMs: 1,
        kind: 'message',
        operationId: 'request:a',
      }]);
      transcripts.appendAssistantTurnEvent('agent:a', {
        type: 'operation.started',
        timestamp: new Date(2).toISOString(),
        operationId: 'request:a',
        label: '正在思考',
        interruptible: true,
      });
      transcripts.appendAssistantTurnEvent('agent:a', {
        type: 'chat.delta',
        timestamp: new Date(3).toISOString(),
        operationId: 'request:a',
        delta: '你',
      });
      transcripts.appendAssistantTurnEvent('agent:a', {
        type: 'chat.delta',
        timestamp: new Date(4).toISOString(),
        operationId: 'request:a',
        delta: '好',
      });

      transcripts.adoptOperation('agent:a', 'request:a', 'operation:a');
      transcripts.appendAssistantTurnEvent('agent:a', {
        type: 'chat.message',
        timestamp: new Date(5).toISOString(),
        operationId: 'operation:a',
        role: 'assistant',
        text: '你好',
      });
      transcripts.appendAssistantTurnEvent('agent:a', {
        type: 'operation.completed',
        timestamp: new Date(6).toISOString(),
        operationId: 'operation:a',
      });

      const thread = transcripts.thread('agent:a');
      const assistantTurns = thread.filter((message) => message.kind === 'assistant-turn');
      expect(assistantTurns).toHaveLength(1);
      expect(assistantTurns[0]?.operationId).toBe('operation:a');
      expect(assistantTurns[0]?.text).toBe('你好');
      expect(transcripts.entries('agent:a').filter((entry) => entry.kind === 'assistant-turn')).toHaveLength(1);
    });

    await test.step('Agent transcript store owns regenerate prompt lookup', async () => {
      const transcripts = new AgentTranscriptStore();
      transcripts.replace('agent:regen', [{
        id: 'regen:user:1',
        source: 'legacy',
        role: 'me',
        text: 'Review this attachment',
        createdAtMs: 1,
        kind: 'message',
        attachments: [{ id: 'regen:attachment:1', name: 'input.txt' }],
      }, {
        id: 'regen:assistant:1',
        source: 'legacy',
        role: 'peer',
        text: 'First answer',
        createdAtMs: 2,
        kind: 'message',
      }, {
        id: 'regen:queued:2',
        source: 'legacy',
        role: 'me',
        text: 'Queued future prompt',
        createdAtMs: 3,
        kind: 'message',
        queued: true,
      }, {
        id: 'regen:assistant:2',
        source: 'legacy',
        role: 'peer',
        text: 'Target answer',
        createdAtMs: 4,
        kind: 'message',
      }]);

      const prompt = transcripts.userPromptBefore('agent:regen', 'regen:assistant:2');
      expect(prompt?.id).toBe('regen:user:1');
      expect(prompt?.text).toBe('Review this attachment');
      expect(prompt?.attachments?.map((attachment) => attachment.id)).toEqual(['regen:attachment:1']);
      expect(transcripts.userPromptBefore('agent:regen', 'missing')).toBeUndefined();
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
      const peer = primaryMahayanaAgentPeer(page);
      await expect(peer).toBeVisible();
      await peer.click();
      const input = page.getByTestId('messenger-input');
      await expect(input).toBeVisible();

      const material = await page.evaluate(() => {
        const inputElement = document.querySelector('[data-testid="messenger-input"]');
        const composer = inputElement?.closest('[data-testid="grok-agent-composer"]');
        const peerElement = document.querySelector('#root [data-testid="messenger-sidebar"] button[data-agent-id="mahayana-assistant"]');
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

      await page.keyboard.press(process.platform === 'darwin' ? 'Meta+K' : 'Control+K');
      const palette = page.getByRole('dialog', { name: 'Command palette' });
      await expect(palette).toBeVisible();
      await page.getByPlaceholder('Search agents or run a command').fill('agent-notes.txt');
      await expect(palette.getByText('agent-notes.txt')).toBeVisible();
      await page.keyboard.press('Escape');
    });

    await test.step('Agent settings are an Agent-owned secondary surface', async () => {
      await page.getByTestId('conversation-info-toggle').click();
      const overlays = page.getByTestId('agent-overlays');
      await expect(overlays).toBeVisible();
      await overlays.getByTestId('agent-settings-toggle').click();
      const settings = overlays.getByRole('region', { name: 'Agent settings' });
      await expect(settings).toBeVisible();
      await expect(settings.getByLabel('Agent name')).toHaveValue(/.+/);
      await expect(settings.getByLabel('Agent description')).toBeVisible();
      await expect(settings.getByRole('switch')).toBeVisible();
      await overlays.getByTestId('bot-computer-toggle').click();
      await expect(settings).toHaveCount(0);
      await expect(overlays.getByTestId('bot-computer-panel')).toBeVisible();
      await overlays.getByRole('button', { name: 'Close Agent info' }).click();
      await expect(overlays).toHaveCount(0);
    });

    await test.step('Agent sidebar supports modifier selection and account-scoped sections', async () => {
      const peer = primaryMahayanaAgentPeer(page);
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
