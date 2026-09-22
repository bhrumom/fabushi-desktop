import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { AgentTranscriptStore } from '../src/agent-workspace/agent-transcript-store';
import { AgentWorkspaceController } from '../src/agent-workspace/agent-workspace-controller';
import { AgentRuntimeCoordinator } from '../src/agent-workspace/agent-runtime-coordinator';
import { mergeAccountSidebarLayoutState } from '../src/agent-workspace/account-sidebar-layout';
import { agentMatchesGroupMember, indexAgentsByRuntimeOrSurfaceId, type AgentSidebarItem } from '../src/agent-workspace/agent-model';
import { composeAgentPromptText } from '../src/agent-workspace/prompt-context';
import { restoreAgentStoreWorkspace } from '../src/agent-workspace/agent-store-recovery';
import { projectAgentMcpReferences } from '../src/agent-workspace/use-agent-mcp-controller';
import {
  emojiSuggestions,
  findPullRequestReadTool,
  parsePullRequestToolResult,
} from '../src/agent-workspace/agent-composer-suggestion-provider';
import type { TranscriptEntry } from '../src/agent-workspace/transcript-model';
import type { RuntimeEvent } from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
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
  const rendererErrors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') rendererErrors.push(message.text());
  });
  type LoginPhase = 'onboarding' | 'login' | 'ready' | 'fatal' | 'waiting';

  // Read the auth surface in one renderer evaluation. During the HostClient ->
  // Messenger transition individual locator probes can straddle a destroyed
  // execution context and wait on navigation even though auth already finished.
  const readPhase = async (): Promise<LoginPhase> => {
    try {
      return await page.evaluate(() => {
        if (document.querySelector('[data-testid="onboarding-gate"]')) return 'onboarding';
        if (document.querySelector('[data-testid="login-gate"]')) return 'login';
        if (document.querySelector('.sand-error-boundary--app')) return 'fatal';
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

    if (currentPhase === 'fatal') {
      const fatal = await page.evaluate(async () => {
        const node = document.querySelector<HTMLElement>('.sand-error-boundary--app');
        const scriptUrl = Array.from(document.scripts)
          .map((script) => script.src)
          .find((source) => /\/assets\/index-[^/]+\.js$/.test(source)) ?? null;
        let sourceMapText: string | null = null;
        if (scriptUrl != null) {
          try {
            const response = await fetch(`${scriptUrl}.map`);
            if (response.ok) sourceMapText = await response.text();
          } catch {
            // The source map is diagnostic-only and must not change product behavior.
          }
        }
        return {
          surfaceText: node?.innerText ?? 'unknown renderer failure',
          scriptUrl,
          sourceMapText,
        };
      });
      if (fatal.sourceMapText != null) {
        await test.info().attach('renderer-root-fatal-source-map', {
          body: fatal.sourceMapText,
          contentType: 'application/json',
        });
      }
      const consoleDetail = rendererErrors.at(-1) ?? fatal.surfaceText;
      throw new Error(`renderer root fatal: ${consoleDetail}\nsource=${fatal.scriptUrl ?? 'unknown'}`);
    }
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
      expect(controller.requestSnapshot()).toEqual({
        'agent:a': 'request:a',
        'agent:b': 'request:b',
      });
      expect(controller.snapshot()).toEqual({});
      expect(controller.operationForPeer('agent:a')).toBeNull();
      expect(controller.operationForPeer('agent:b')).toBeNull();

      controller.adoptOperation('request:a', 'operation:a', 'agent:a');
      expect(controller.operationForPeer('agent:a')).toBe('operation:a');
      expect(controller.requestForPeer('agent:b')).toBe('request:b');
      expect(controller.isBusy('agent:a')).toBe(true);
      expect(controller.isBusy('agent:b')).toBe(true);

      controller.finishOperation('operation:a');
      expect(controller.isBusy('agent:a')).toBe(false);
      expect(controller.isBusy('agent:b')).toBe(true);
      expect(controller.claimRuntimeOperation('operation:unknown')).toBeNull();

      const claimedPeer = controller.claimRuntimeOperation('operation:b', controller.peerForRequest('request:b'));
      expect(claimedPeer).toBe('agent:b');
      expect(controller.peerForRuntimeId('operation:b')).toBe('agent:b');
      expect(controller.requestForPeer('agent:b')).toBeNull();
      expect(controller.finishRuntimeOperation('operation:b')).toBe('agent:b');
      expect(controller.isOperationFinished('operation:b')).toBe(true);
      expect(controller.claimRuntimeOperation('operation:b', 'agent:b')).toBeNull();

      controller.setDraft('agent:a', 'first prompt @Research /Review changes @GitHub');
      controller.appendAttachments('agent:a', [{ id: 'attachment:a', name: 'a.txt' }]);
      controller.setReply('agent:a', { id: 'reply:a', role: 'peer', text: 'previous answer' });
      controller.upsertReference('agent:a', { kind: 'agent', id: 'agent:research', label: 'Research' });
      controller.upsertReference('agent:a', { kind: 'workflow', id: 'workflow:review', label: 'Review changes' });
      controller.upsertReference('agent:a', { kind: 'mcp', id: 'mcp:github', label: 'GitHub' });
      controller.setDraft('agent:b', 'independent draft');

      const submitted = controller.takeDraft('agent:a');
      expect(submitted.text).toBe('first prompt @Research /Review changes @GitHub');
      expect(submitted.attachments.map((attachment) => attachment.id)).toEqual(['attachment:a']);
      expect(submitted.replyTo?.id).toBe('reply:a');
      expect(submitted.references).toEqual([
        { kind: 'agent', id: 'agent:research', label: 'Research' },
        { kind: 'workflow', id: 'workflow:review', label: 'Review changes' },
        { kind: 'mcp', id: 'mcp:github', label: 'GitHub' },
      ]);
      expect(submitted.richText).toContain('"type":"doc"');
      expect(submitted.richText).toContain('"type":"mention"');
      expect(submitted.richText).toContain('"type":"workflowReference"');
      const composedPrompt = composeAgentPromptText(submitted.text, submitted.replyTo, submitted.references);
      expect(composedPrompt).toContain('@Research [agent:agent:research]');
      expect(composedPrompt).toContain('/Review changes [workflow:workflow:review]');
      expect(composedPrompt).toContain('@GitHub [mcp:mcp:github]');
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
        { kind: 'mcp', id: 'mcp:github', label: 'GitHub' },
      ]);
      controller.setDraft('agent:a', 'new draft without a mention');
      controller.pruneReferences('agent:a', 'new draft without a mention');
      expect(controller.referencesForPeer('agent:a')).toEqual([]);
    });

    await test.step('Account sidebar CAS preserves concurrent cross-device edits', async () => {
      const base = {
        pinnedOrder: ['agent:a', 'agent:b'],
        sections: [{
          id: 'focus',
          name: 'Focus',
          agentKeys: ['agent:a'],
          isCollapsed: false,
        }],
      };
      const merged = mergeAccountSidebarLayoutState(
        base,
        {
          pinnedOrder: ['agent:b'],
          sections: [
            {
              id: 'focus',
              name: 'Focused work',
              agentKeys: ['agent:a'],
              isCollapsed: false,
            },
            {
              id: 'local',
              name: 'Local only',
              agentKeys: ['agent:b'],
              isCollapsed: false,
            },
          ],
        },
        {
          pinnedOrder: ['agent:a', 'agent:b', 'agent:c'],
          sections: [
            {
              id: 'focus',
              name: 'Focus',
              agentKeys: ['agent:a'],
              isCollapsed: true,
            },
            {
              id: 'remote',
              name: 'Remote only',
              agentKeys: ['agent:c'],
              isCollapsed: false,
            },
          ],
        },
      );

      // Local unpin wins over a concurrent remote reorder, while the remote-only
      // pinned Agent is retained. Independent section edits from both devices
      // are also merged rather than overwritten by a CAS retry.
      expect(merged.pinnedOrder).toEqual(['agent:b', 'agent:c']);
      expect(merged.sections.map((section) => section.id)).toEqual(['focus', 'local', 'remote']);
      expect(merged.sections[0]).toEqual({
        id: 'focus',
        name: 'Focused work',
        agentKeys: ['agent:a'],
        isCollapsed: true,
      });
      expect(merged.sections[1]?.agentKeys).toEqual(['agent:b']);
      expect(merged.sections[2]?.agentKeys).toEqual(['agent:c']);
    });

    await test.step('Agent MCP catalog normalizes untyped Host rows into stable Composer references', async () => {
      expect(projectAgentMcpReferences([
        { name: 'github', status: 'connected', transport: 'streamable_http', tools: [{ name: 'search' }, { name: 'pull' }] },
        { id: 'custom-id', displayName: 'Custom MCP', status: 'auth_required', tools: [] },
        { name: 'github', status: 'connected', tools: [] },
        null,
        { status: 'connected' },
      ])).toEqual([
        {
          id: 'mcp:github',
          name: 'github',
          description: 'connected · 2 tools · streamable_http',
          status: 'connected',
          toolCount: 2,
        },
        {
          id: 'mcp:custom-id',
          name: 'Custom MCP',
          description: 'auth_required',
          status: 'auth_required',
          toolCount: 0,
        },
      ]);
    });

    await test.step('Composer suggestions project emoji and read-only GitHub PR references', async () => {
      expect(emojiSuggestions('thi').some((candidate) => candidate.shortcodes.includes('thinking'))).toBe(true);
      expect(findPullRequestReadTool([
        {
          name: 'github',
          tools: [
            { name: 'pull' },
            { name: 'create_pull_request' },
            { name: 'search_pull_requests' },
          ],
        },
      ])).toEqual({ server: 'github', tool: 'search_pull_requests' });
      expect(parsePullRequestToolResult({
        items: [{
          number: 7,
          title: 'Agent workspace parity',
          html_url: 'https://github.com/bhrumom/fabushi-desktop/pull/7',
        }],
      })).toEqual([{
        prNumber: 7,
        title: 'Agent workspace parity',
        url: 'https://github.com/bhrumom/fabushi-desktop/pull/7',
      }]);

      const controller = new AgentWorkspaceController();
      const richText = JSON.stringify({
        type: 'doc',
        content: [{
          type: 'paragraph',
          content: [
            { type: 'text', text: 'Review ' },
            {
              type: 'prReference',
              attrs: {
                prNumber: 7,
                title: 'Agent workspace parity',
                url: 'https://github.com/bhrumom/fabushi-desktop/pull/7',
              },
            },
          ],
        }],
      });
      controller.setDraftDocument('agent:pr', 'Review #7', richText);
      expect(controller.referencesForPeer('agent:pr')).toContainEqual({
        kind: 'pull-request',
        id: 'https://github.com/bhrumom/fabushi-desktop/pull/7',
        label: '7',
      });
      expect(composeAgentPromptText(
        controller.draftForPeer('agent:pr'),
        undefined,
        controller.referencesForPeer('agent:pr'),
      )).toContain('#7 [pull-request:https://github.com/bhrumom/fabushi-desktop/pull/7]');
    });

    await test.step('Agent transcript canonicalizes duplicate legacy assistant rows into one operation timeline', async () => {
      const transcripts = new AgentTranscriptStore();
      transcripts.replace('agent:canonical', [{
        id: 'operation:canonical:assistant-turn',
        source: 'legacy',
        role: 'peer',
        text: '',
        createdAtMs: 1,
        kind: 'assistant-turn',
        operationId: 'operation:canonical',
        assistantTurn: {
          id: 'assistant-turn:operation:canonical',
          operationId: 'operation:canonical',
          createdAtMs: 1,
          updatedAtMs: 1,
          status: 'completed',
          parts: [],
        },
      }, {
        id: 'legacy-final',
        source: 'legacy',
        role: 'peer',
        text: 'single canonical answer',
        createdAtMs: 2,
        kind: 'message',
        operationId: 'operation:canonical',
      }, {
        id: 'operation:canonical:duplicate',
        source: 'legacy',
        role: 'peer',
        text: 'stale duplicate',
        createdAtMs: 0,
        kind: 'assistant-turn',
        operationId: 'operation:canonical',
        assistantTurn: {
          id: 'assistant-turn:operation:canonical:duplicate',
          operationId: 'operation:canonical',
          createdAtMs: 0,
          updatedAtMs: 0,
          status: 'running',
          parts: [],
        },
      }]);
      const entries = transcripts.entries('agent:canonical');
      expect(entries.filter((entry) => entry.operationId === 'operation:canonical')).toHaveLength(1);
      expect(entries[0]?.kind).toBe('assistant-turn');
      expect(entries[0]?.text).toBe('single canonical answer');
      expect(entries[0]?.assistantTurn?.parts.filter((part) => part.kind === 'text')).toHaveLength(1);
    });

    await test.step('Agent group identity survives Host runtime-to-surface normalization', async () => {
      const agent: AgentSidebarItem = {
        key: 'agent:runtime-agent',
        peerKey: 'legacy:bot:surface-bot',
        id: 'surface-bot',
        agentId: 'runtime-agent',
        name: 'Research',
        description: 'Research Agent',
        pinned: false,
        hidden: false,
        unread: 0,
        busy: false,
        isGroup: false,
        updatedAtMs: 1,
      };
      expect(agentMatchesGroupMember(agent, 'runtime-agent')).toBe(true);
      expect(agentMatchesGroupMember(agent, 'surface-bot')).toBe(true);
      expect(agentMatchesGroupMember(agent, 'other-agent')).toBe(false);
      const index = indexAgentsByRuntimeOrSurfaceId([agent]);
      expect(index.get('runtime-agent')?.key).toBe(agent.key);
      expect(index.get('surface-bot')?.key).toBe(agent.key);
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

      coordinator.bindAgentPeers([{
        agentId: 'agent:restart-runtime',
        peerKey: 'agent:restart',
        conversationId: 'codex:agent:restart',
      }]);
      coordinator.beginLocalTurn({
        peerKey: 'agent:restart',
        requestId: 'request:restart',
        messageId: 'user:restart',
        text: 'survive host restart',
        createdAtMs: 6,
      });
      coordinator.adoptOperation('request:restart', 'operation:restart', 'agent:restart');
      expect(coordinator.handle({
        type: 'turn.state',
        timestamp: new Date(6).toISOString(),
        operationId: 'operation:restart',
        turnId: 'turn:restart',
        runId: 'run:restart:1',
        conversationId: 'codex:agent:restart',
        state: 'thinking',
        sequence: 1,
      })).toBe(true);
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
      expect(transcripts.entries('agent:restart').filter((entry) => entry.kind === 'assistant-turn')).toHaveLength(0);

      expect(coordinator.handle({
        type: 'turn.state',
        timestamp: new Date(8).toISOString(),
        operationId: 'operation:restart:recovered',
        turnId: 'turn:restart',
        runId: 'run:restart:2',
        conversationId: 'codex:agent:restart',
        state: 'recovering',
        sequence: 2,
      })).toBe(true);
      expect(controller.operationForPeer('agent:restart')).toBe('operation:restart:recovered');
      expect(transcripts.thread('agent:restart').find((message) => message.id === 'user:restart')?.operationId)
        .toBe('operation:restart:recovered');
      expect(transcripts.entries('agent:restart').filter((entry) => entry.kind === 'assistant-turn')).toHaveLength(1);
      expect(controller.draftForPeer('agent:a')).toBe('draft survives reconnect');
      expect(coordinator.handle({
        type: 'turn.state',
        timestamp: new Date(9).toISOString(),
        operationId: 'operation:restart:recovered',
        turnId: 'turn:restart',
        runId: 'run:restart:2',
        conversationId: 'codex:agent:restart',
        state: 'completed',
        sequence: 3,
      })).toBe(true);
      expect(controller.isBusy('agent:restart')).toBe(false);

      coordinator.beginLocalTurn({
        peerKey: 'agent:waiting-restart',
        requestId: 'request:waiting-restart',
        messageId: 'user:waiting-restart',
        text: 'needs explicit approval',
        createdAtMs: 10,
      });
      coordinator.adoptOperation(
        'request:waiting-restart',
        'operation:waiting-restart',
        'agent:waiting-restart',
      );
      expect(coordinator.handle({
        type: 'turn.state',
        timestamp: new Date(10).toISOString(),
        operationId: 'operation:waiting-restart',
        turnId: 'turn:waiting-restart',
        runId: 'run:waiting-restart',
        conversationId: 'codex:agent:waiting-restart',
        state: 'waiting-user',
        sequence: 1,
      })).toBe(true);
      expect(coordinator.handle({
        type: 'host.lifecycle',
        timestamp: new Date(11).toISOString(),
        lifecycle: 'stopped',
        state: 'stopped',
        generation: 10,
        sequence: 21,
        recoverable: true,
        error: 'approval channel reset',
      })).toBe(true);
      expect(controller.isBusy('agent:waiting-restart')).toBe(false);
      expect(
        transcripts.entries('agent:waiting-restart')
          .find((entry) => entry.kind === 'assistant-turn')
          ?.assistantTurn?.status,
      ).toBe('interrupted');

      coordinator.bindAgentPeers([{ agentId: 'c', peerKey: 'agent:c' }]);
      expect(coordinator.handleCommandBridge({
        phase: 'dispatch',
        command: { type: 'chat.send', requestId: 'request:c', text: 'C', agentId: 'c' },
        context: { conversationKey: 'agent:c', agentId: 'c' },
      })).toBe(true);
      expect(controller.requestForPeer('agent:c')).toBe('request:c');
      expect(coordinator.handleCommandBridge({
        phase: 'accepted',
        command: { type: 'chat.send', requestId: 'request:c', text: 'C', agentId: 'c' },
        accepted: { requestId: 'request:c', operationId: 'operation:c' },
        context: { conversationKey: 'agent:c', agentId: 'c' },
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

    await test.step('Agent runtime correlation fails closed without a canonical operation id', async () => {
      const controller = new AgentWorkspaceController();
      const transcripts = new AgentTranscriptStore();
      const coordinator = new AgentRuntimeCoordinator(controller, transcripts);
      coordinator.bindAgentPeers([{
        agentId: 'strict',
        peerKey: 'agent:strict',
        conversationId: 'codex:agent:strict',
      }]);

      coordinator.beginLocalTurn({
        peerKey: 'agent:strict',
        requestId: 'request:strict',
        messageId: 'user:strict',
        text: 'strict ownership',
        createdAtMs: 30,
      });

      expect(coordinator.handle({
        type: 'chat.delta',
        timestamp: new Date(31).toISOString(),
        delta: 'must-not-be-inferred',
      } as unknown as RuntimeEvent)).toBe(false);
      expect(coordinator.handle({
        type: 'chat.message',
        timestamp: new Date(32).toISOString(),
        role: 'assistant',
        text: 'must-not-be-inferred',
      })).toBe(false);
      expect(controller.requestForPeer('agent:strict')).toBe('request:strict');
      expect(controller.operationForPeer('agent:strict')).toBeNull();
      expect(transcripts.entries('agent:strict').map((entry) => entry.text).join(' ')).not.toContain('must-not-be-inferred');

      expect(coordinator.handle({
        type: 'turn.state',
        timestamp: new Date(33).toISOString(),
        operationId: 'operation:strict',
        turnId: 'turn:strict',
        runId: 'run:strict',
        conversationId: 'codex:agent:strict',
        state: 'thinking',
        sequence: 1,
      })).toBe(true);
      expect(controller.operationForPeer('agent:strict')).toBe('operation:strict');
      expect(controller.requestForPeer('agent:strict')).toBeNull();

      expect(coordinator.handle({
        type: 'chat.delta',
        timestamp: new Date(34).toISOString(),
        operationId: 'operation:strict',
        delta: 'canonical',
      })).toBe(true);
      coordinator.flushPendingDeltas();
      expect(transcripts.entries('agent:strict').map((entry) => entry.text).join(' ')).toContain('canonical');
      coordinator.dispose();
    });

    await test.step('Agent command bridge resolves Rust conversation ids back to canonical peers under concurrent sends', async () => {
      const controller = new AgentWorkspaceController();
      const transcripts = new AgentTranscriptStore();
      const coordinator = new AgentRuntimeCoordinator(controller, transcripts);
      coordinator.bindAgentPeers([
        { agentId: 'research', peerKey: 'agent:research', conversationId: 'codex:agent:research' },
        { agentId: 'builder', peerKey: 'agent:builder', conversationId: 'codex:agent:builder' },
      ]);

      coordinator.beginLocalTurn({
        peerKey: 'agent:research',
        requestId: 'request:research',
        messageId: 'user:research',
        text: 'Research',
        createdAtMs: 20,
      });
      coordinator.beginLocalTurn({
        peerKey: 'agent:builder',
        requestId: 'request:builder',
        messageId: 'user:builder',
        text: 'Builder',
        createdAtMs: 21,
      });

      const researchCommand = {
        type: 'chat.send' as const,
        requestId: 'request:research',
        conversationId: 'codex:agent:research',
        agentId: 'research',
        text: 'Research',
      };
      const builderCommand = {
        type: 'chat.send' as const,
        requestId: 'request:builder',
        conversationId: 'codex:agent:builder',
        agentId: 'builder',
        text: 'Builder',
      };

      expect(coordinator.handleCommandBridge({
        phase: 'dispatch',
        command: researchCommand,
        context: {
          conversationKey: 'codex:agent:research',
          conversationId: 'codex:agent:research',
          agentId: 'research',
        },
      })).toBe(true);
      expect(coordinator.handleCommandBridge({
        phase: 'dispatch',
        command: builderCommand,
        context: {
          conversationKey: 'codex:agent:builder',
          conversationId: 'codex:agent:builder',
          agentId: 'builder',
        },
      })).toBe(true);

      expect(controller.requestForPeer('agent:research')).toBe('request:research');
      expect(controller.requestForPeer('agent:builder')).toBe('request:builder');
      expect(controller.requestForPeer('codex:agent:research')).toBeNull();
      expect(controller.requestForPeer('codex:agent:builder')).toBeNull();

      expect(coordinator.handleCommandBridge({
        phase: 'accepted',
        command: researchCommand,
        accepted: { requestId: 'request:research', operationId: 'operation:research' },
        context: {
          conversationKey: 'codex:agent:research',
          conversationId: 'codex:agent:research',
          agentId: 'research',
        },
      })).toBe(true);
      expect(coordinator.handleCommandBridge({
        phase: 'accepted',
        command: builderCommand,
        accepted: { requestId: 'request:builder', operationId: 'operation:builder' },
        context: {
          conversationKey: 'codex:agent:builder',
          conversationId: 'codex:agent:builder',
          agentId: 'builder',
        },
      })).toBe(true);

      expect(controller.operationForPeer('agent:research')).toBe('operation:research');
      expect(controller.operationForPeer('agent:builder')).toBe('operation:builder');
      expect(controller.operationForPeer('codex:agent:research')).toBeNull();
      expect(controller.operationForPeer('codex:agent:builder')).toBeNull();

      for (const [operationId, conversationId, text] of [
        ['operation:research', 'codex:agent:research', 'research complete'],
        ['operation:builder', 'codex:agent:builder', 'builder complete'],
      ] as const) {
        expect(coordinator.handle({
          type: 'chat.message',
          timestamp: new Date(22).toISOString(),
          operationId,
          role: 'assistant',
          text,
        })).toBe(true);
        expect(coordinator.handle({
          type: 'turn.state',
          timestamp: new Date(23).toISOString(),
          operationId,
          turnId: `turn:${operationId}`,
          runId: `run:${operationId}`,
          conversationId,
          state: 'completed',
          sequence: 4,
        })).toBe(true);
        expect(coordinator.handle({
          type: 'operation.completed',
          timestamp: new Date(24).toISOString(),
          operationId,
        })).toBe(true);
      }

      expect(controller.isBusy('agent:research')).toBe(false);
      expect(controller.isBusy('agent:builder')).toBe(false);
      expect(transcripts.entries('agent:research').map((entry) => entry.text).join(' ')).toContain('research complete');
      expect(transcripts.entries('agent:builder').map((entry) => entry.text).join(' ')).toContain('builder complete');
      expect(transcripts.entries('agent:research').map((entry) => entry.text).join(' ')).not.toContain('builder complete');
      expect(transcripts.entries('agent:builder').map((entry) => entry.text).join(' ')).not.toContain('research complete');
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

    await test.step('Agent Network is the Agent-domain group surface', async () => {
      await page.getByRole('button', { name: 'Agent network' }).click();
      const network = page.getByTestId('grok-agent-network');
      await expect(network).toBeVisible();
      await expect(network.getByRole('button', { name: 'Create from selected' })).toBeVisible();
      await expect(network.getByText(/agents · \d+ groups/)).toBeVisible();
      await network.getByRole('button', { name: 'Close Agent network' }).click();
      await expect(network).toBeHidden();
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

      const takeover = overlays.getByTestId('agent-computer-takeover');
      await expect(takeover.getByRole('button', { name: 'Take Control' })).toBeVisible();
      await takeover.getByRole('button', { name: 'Take Control' }).click();
      await expect(takeover).toContainText('You have control');
      await expect(takeover.getByRole('button', { name: 'Release Control' })).toBeVisible();
      await takeover.getByRole('button', { name: 'Release Control' }).click();
      await expect(takeover.getByRole('button', { name: 'Take Control' })).toBeVisible();

      await overlays.getByRole('button', { name: 'Close Agent info' }).click();
      await expect(overlays).toHaveCount(0);
    });

    await test.step('Agent sidebar supports modifier selection and account-scoped sections', async () => {
      const peer = primaryMahayanaAgentPeer(page);
      await peer.click({ modifiers: [process.platform === 'darwin' ? 'Meta' : 'Control'] });
      const selectionBar = page.getByTestId('agent-selection-bar');
      await expect(selectionBar).toBeVisible();
      await expect(selectionBar).toContainText('1 selected');

      await selectionBar.getByRole('button', { name: 'Section' }).click();
      const sectionDialog = page.getByRole('dialog', { name: 'Create section' });
      await expect(sectionDialog).toBeVisible();
      await sectionDialog.getByLabel('Section name').fill('Focused work');
      await sectionDialog.getByRole('button', { name: 'Create' }).click();
      const focusedWork = page.locator('[data-section-id]').filter({ hasText: 'Focused work' });
      await expect(focusedWork).toBeVisible();
      await expect(selectionBar).toHaveCount(0);

      // Pinning is a presentation dimension, not section ownership. The Agent
      // stays under Pinned while pinned, then must project back into the section
      // that was just persisted when it is unpinned. This guards the original
      // 35522950977 failure instead of merely asserting that an empty header exists.
      const pinnedRow = peer.locator('..');
      const focusedAgentRow = focusedWork.locator('button[data-agent-key="agent:mahayana-assistant"]');
      await expect(pinnedRow).toHaveAttribute('data-pinned', 'true');
      await expect(focusedAgentRow).toHaveCount(0);
      await pinnedRow.hover();
      await pinnedRow.getByRole('button', { name: /大乘助手 actions/ }).click();
      await page.getByRole('menuitem', { name: 'Unpin' }).click();
      await expect(focusedAgentRow).toHaveCount(1);
      await expect(focusedAgentRow).toBeVisible();
    });
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
