import React from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { ProductionRenderer } from '../../../frontend/src/production/ProductionRenderer';
import {
  parseCoordinatorHumanTranscriptPageRequest,
  type CoordinatorHumanTranscriptPageResponse,
} from '../../../source/shared/rpc/coordinator';
import type {
  CoordinatorPortBridge,
  CursorAuthStatus,
  DesktopBridge,
  TransferredCoordinatorPort,
} from '../../../frontend/src/recovered/contracts/desktop-bridge';

// Exercise the shipping renderer, coordinator client, feed, queue and cache.
// This is an in-memory external Host/desktop-port fixture. It neither executes
// Session/SQLite nor supplies evidence of Host persistence or restart recovery.
export type FixtureRow = Record<string, unknown> & { id: string; kind: string };
export interface FixtureRequest {
  sequence: number;
  portId: number;
  requestId: string;
  method: string;
  args: Record<string, unknown>;
  accountId: string | null;
  accountEpoch: number;
}
type Endpoint = {
  id: number;
  closed: boolean;
  listeners: Set<(event: { data: unknown }) => void>;
};
type Pending = {
  request: FixtureRequest;
  endpoint: Endpoint;
  value: unknown;
  settled: boolean;
  position?: number;
  revision?: number;
};
type HumanHistoryWalk = {
  snapshotId: string;
  conversationId: string;
  accountId: string | null;
  accountEpoch: number;
  members: ReadonlyArray<{ id: string; seq: number }>;
};
type PendingApprovalClear = {
  index: number;
  accountId: string | null;
  accountEpoch: number;
  settled: boolean;
  resolve(): void;
};
type DraftPersistenceWrite = {
  index: number; key: string; value: string; containsSubmissionBinding: boolean;
  state: 'held' | 'written' | 'failed'; failure: string | null;
  resolve(): void; reject(error: Error): void;
};
type PendingDraftRead = {
  index: number; key: string; value: string | null; settled: boolean;
  resolve(value: string | null): void;
};
const clone = <T,>(value: T): T => structuredClone(value);
const query = new URLSearchParams(location.search);
const theme = query.get('theme') === 'light' ? 'light' : 'dark';
const includeAgent = query.get('agent') === '1';
const accountListeners = new Set<(status: CursorAuthStatus) => void>();
const storage = new Map<string, string>();
const storageWrites: Array<{ key: string; value: string | null }> = [];
const requests: FixtureRequest[] = [];
const frames: Array<Record<string, unknown>> = [];
const unhandledRequests: FixtureRequest[] = [];
const desktopCalls: Array<Record<string, unknown>> = [];
const endpoints: Endpoint[] = [];
const initials: Pending[] = [];
const older: Pending[] = [];
const refreshes: Pending[] = [];
const sends: Pending[] = [];
const retries: Pending[] = [];
const agentSends: Pending[] = [];
const agentRosters: Pending[] = [];
const heldAgentTails: Pending[] = [];
// The external protocol retains the original admitted request body. This is
// an in-memory service control, not a second renderer queue or durable Host.
const humanIntents = new Map<string, Pending>();
const attachmentCommits: Array<{
  index: number; paths: string[]; names: string[]; committedPaths: string[]; failure: string | null;
}> = [];
const attachmentCommitFailures = new Map<number, string>();
const reactions: Pending[] = [];
const approvalClears: PendingApprovalClear[] = [];
const draftPersistenceWrites: DraftPersistenceWrite[] = [];
const heldDraftReads: PendingDraftRead[] = [];
const humanHistoryWalks = new Map<string, HumanHistoryWalk>();
const humanHistoryReads: Array<{
  request: FixtureRequest;
  snapshotId: string | null;
  selectedPhysicalSeqs: number[];
  response: CoordinatorHumanTranscriptPageResponse;
}> = [];
const humanHistoryRejections: Array<{ request: FixtureRequest; message: string }> = [];
let account: CursorAuthStatus = { kind: 'logged-in', authId: '1', email: 'one@example.test', displayName: 'Local' };
let accountEpoch = 0;
let holdOlder = true;
let holdAgentRoster = query.get('holdAgentRoster') === '1';
let holdAgentTails = false;
let holdInitial = query.get('holdInitial') === '1';
let holdApprovalClears = false;
let holdNextSubmissionBindingWrite = false;
let holdDraftReads = false;
let sequence = 0;
let revision = 0;
let physicalSequence = 0;
let humanWalkSequence = 0;
let root: Root | null = null;
let mountCount = 0;
let reservedNonce: string | null = null;
const nativeRandomUuid = crypto.randomUUID.bind(crypto);
Object.defineProperty(crypto, 'randomUUID', {
  configurable: true,
  value: () => reservedNonce ?? nativeRandomUuid(),
});
const currentAccountId = () => account.kind === 'logged-in' ? account.authId ?? null : null;
const row = (id: string, content: string, patch: Record<string, unknown> = {}): FixtureRow => {
  const remoteMessageId = /^human-server-message:([1-9]\d*)$/.exec(id)?.[1];
  return {
    id, kind: 'message', role: 'user', authorKind: 'human', authorId: '2',
    ...(remoteMessageId == null ? {} : { remoteMessageId }),
    content, timestampMs: 1_700_000_000_000 + (++sequence * 1000),
    delivery: 'sent', attachments: [], reactions: [], ...patch,
  };
};
const rows = new Map<string, FixtureRow[]>([
  ['human:a', [
    row('human-server-message:101', 'Old first message'),
    row('human-server-message:102', 'Old second message', { attachments: [{ path: 'memory:note.txt', name: 'note.txt' }] }),
    row('human-server-message:103', 'Tail third message'),
    row('human-server-message:104', 'Tail fourth message', { attachments: [{ path: 'memory:tail.txt', name: 'tail.txt' }], reactions: [{ emoji: '👍', by: 'me' }] }),
    row('human-server-message:105', 'Tail fifth message'),
  ]],
  ['human:b', [row('human-server-message:201', 'Bob conversation', { authorId: '3' })]],
  ['agent:helper', [
    { id: 'agent-request', kind: 'message', role: 'user', content: 'Inspect the attached file', timestampMs: 1_700_000_100_000,
      delivery: 'sent', attachments: [{ path: 'memory:agent.txt', name: 'agent.txt' }], clientNonce: 'agent-existing-nonce' },
    { id: 'agent-tool', kind: 'tool-call', name: 'read_file', status: 'done', summary: 'Read fixture.txt\nTool result retained', timestampMs: 1_700_000_101_000 },
    { id: 'agent-answer', kind: 'message', role: 'assistant', content: 'The Agent answer is intact.', timestampMs: 1_700_000_102_000, delivery: 'sent' },
  ]],
]);
const revisions = new Map<string, number>();
const physicalSequences = new Map<string, number>();
const revisionKey = (conversationId: string, entryId: string) => conversationId + '\n' + entryId;
const touch = (conversationId: string, entryId: string) => {
  const key = revisionKey(conversationId, entryId);
  if (!physicalSequences.has(key)) physicalSequences.set(key, ++physicalSequence);
  revisions.set(key, ++revision);
};
for (const [id, entries] of rows) for (const entry of entries) touch(id, entry.id);
const agents = () => includeAgent ? [{ id: 'agent:helper', name: 'Fixture Agent', isRunning: false, updatedAt: 50 }] : [];
const humans = () => [
  { id: 'human:a', kind: 'human', title: 'Alice', participantIds: [currentAccountId() ?? '', '2'], updatedAt: 200 },
  { id: 'human:b', kind: 'human', title: 'Bob', participantIds: [currentAccountId() ?? '', '3'], updatedAt: 100 },
];

function invalidateHumanHistory(conversationId?: string) {
  for (const [snapshotId, walk] of humanHistoryWalks) {
    if (conversationId == null || walk.conversationId === conversationId) humanHistoryWalks.delete(snapshotId);
  }
  for (const key of physicalSequences.keys()) {
    if (conversationId == null || key.startsWith(conversationId + '\n')) physicalSequences.delete(key);
  }
}

function humanHistoryPage(request: FixtureRequest): CoordinatorHumanTranscriptPageResponse {
  const args = parseCoordinatorHumanTranscriptPageRequest(request.args);
  if (args == null) throw new Error('Invalid typed Human transcript request');
  const transcript = rows.get(args.conversationId) ?? [];
  let walk: HumanHistoryWalk;
  let position = -1;
  if (args.beforeCursor != null) {
    const token = /^htp1:([0-9a-f]{32}):([0-9]+):([1-9][0-9]*)$/.exec(args.beforeCursor);
    const existing = token == null ? undefined : humanHistoryWalks.get(token[1]);
    if (existing == null || existing.conversationId !== args.conversationId
      || existing.accountId !== request.accountId || existing.accountEpoch !== request.accountEpoch) {
      throw new Error('Unknown, expired or foreign-scope Human transcript cursor');
    }
    position = Number(token![2]);
    const anchor = existing.members[position];
    if (!Number.isSafeInteger(position) || position < 0 || anchor == null
      || anchor.seq !== Number(token![3])
      || (args.beforeSeq != null && args.beforeSeq !== anchor.seq)) {
      throw new Error('Human transcript cursor position or beforeSeq does not match its fixed walk');
    }
    walk = existing;
  } else {
    // The Host still accepts legacy beforeSeq, but starts a new stable walk.
    // Tests below require the renderer to continue the issued opaque token;
    // accepting legacy input here must not hide a lost beforeCursor.
    const boundary = args.beforeSeq == null ? transcript.length : transcript.findIndex(
      (entry) => physicalSequences.get(revisionKey(args.conversationId, entry.id)) === args.beforeSeq,
    );
    if (boundary < 0) {
      const response = { entries: [] } satisfies CoordinatorHumanTranscriptPageResponse;
      humanHistoryReads.push({ request: clone(request), snapshotId: null, selectedPhysicalSeqs: [], response });
      return response;
    }
    walk = {
      snapshotId: (++humanWalkSequence).toString(16).padStart(32, '0'),
      conversationId: args.conversationId, accountId: request.accountId, accountEpoch: request.accountEpoch,
      members: transcript.slice(0, boundary).map((entry) => {
        const seq = physicalSequences.get(revisionKey(args.conversationId, entry.id));
        if (seq == null) throw new Error('Fixture service row has no physical identity');
        return { id: entry.id, seq };
      }).reverse(),
    };
    humanHistoryWalks.set(walk.snapshotId, walk);
  }
  // Small external page windows keep the UI trace legible. Membership is fixed
  // at capture; each request reads current payloads, then freezes its response.
  // This does not emulate or qualify the Host's durable SQLite implementation.
  const limit = Math.max(1, Math.min(3, args.limit ?? 200));
  const selected = walk.members.slice(position + 1, position + 1 + limit);
  const entries = selected.map((member) => {
    const entry = transcript.find((candidate) => candidate.id === member.id);
    if (entry == null || physicalSequences.get(revisionKey(args.conversationId, member.id)) !== member.seq) {
      throw new Error('Human transcript walk references a removed or replaced physical row');
    }
    return clone(entry);
  }).reverse();
  const nextPosition = position + selected.length;
  const last = selected.at(-1);
  const response: CoordinatorHumanTranscriptPageResponse = {
    entries,
    ...(last != null && nextPosition + 1 < walk.members.length ? {
      nextBeforeSeq: last.seq,
      nextBeforeCursor: 'htp1:' + walk.snapshotId + ':' + nextPosition + ':' + last.seq,
    } : {}),
  };
  humanHistoryReads.push({
    request: clone(request), snapshotId: walk.snapshotId,
    selectedPhysicalSeqs: selected.map((member) => member.seq).reverse(), response: clone(response),
  });
  return response;
}

function deliver(endpoint: Endpoint, frame: Record<string, unknown>) {
  frames.push({ portId: endpoint.id, dropped: endpoint.closed, frame: clone(frame) });
  if (!endpoint.closed) for (const listener of [...endpoint.listeners]) listener({ data: clone(frame) });
}
function reply(pending: Pending, failure?: { code: string; message: string }) {
  if (pending.settled) throw new Error('Fixture attempted to settle a request twice: ' + pending.request.requestId);
  pending.settled = true;
  deliver(pending.endpoint, {
    kind: 'reply', requestId: pending.request.requestId,
    outcome: failure == null ? { status: 'ok', value: clone(pending.value) } : { status: 'failed', failure },
  });
}
function emit(payload: Record<string, unknown>) {
  for (const endpoint of endpoints) {
    if (!endpoint.closed) deliver(endpoint, { kind: 'event', family: 'transcript', payload });
  }
}
function pendingAt(values: Pending[], index: number, kind: string): Pending {
  const pending = values[index];
  if (pending == null || pending.settled) throw new Error('No unsettled ' + kind + ' at index ' + index);
  return pending;
}
function insertSend(pending: Pending): FixtureRow {
  const entry = clone(pending.value as FixtureRow);
  const id = String(pending.request.args.conversationId ?? pending.request.args.agentId);
  if (pending.request.accountEpoch !== accountEpoch) throw new Error('Cannot publish an old account into the fixture service');
  const transcript = rows.get(id) ?? [];
  if (!transcript.some((candidate) => candidate.id === entry.id)) {
    transcript.splice(pending.position ?? transcript.length, 0, entry);
    rows.set(id, transcript);
    touch(id, entry.id);
  }
  return entry;
}
function receive(endpoint: Endpoint, frame: { requestId: string; method: string; args?: Record<string, unknown> }) {
  const request: FixtureRequest = {
    sequence: requests.length + 1, portId: endpoint.id, requestId: frame.requestId,
    method: frame.method, args: clone(frame.args ?? {}),
    accountId: currentAccountId(), accountEpoch,
  };
  requests.push(request);
  const id = String(request.args.conversationId ?? request.args.id ?? request.args.agentId ?? '');
  const transcript = rows.get(id) ?? [];
  const pending: Pending = { request, endpoint, value: null, settled: false };
  switch (request.method) {
    case 'listAgents':
      pending.value = clone(agents());
      if (holdAgentRoster) { agentRosters.push(pending); return; }
      break;
    case 'searchAgents': pending.value = clone(agents()); break;
    case 'getTrays': case 'listAllAutomations':
    case 'getAgentAutomations': case 'getAgentWorkflows':
    case 'getAsyncTasks': case 'getSubagents': case 'listHumanCalls': case 'syncHumanCalls':
    case 'getPendingHumanCalls': case 'listCallSessions':
      pending.value = []; break;
    case 'getListenerIntegrations': pending.value = { integrations: [] }; break;
    case 'getAgentChannels': pending.value = { manifests: [], connections: [] }; break;
    case 'getSkillPublishTargets': pending.value = { teams: [] }; break;
    case 'listHumanConversations': case 'syncHumanConversations':
      pending.value = clone(humans()); break;
    case 'syncHumanConversation': pending.value = clone(transcript); break;
    case 'getForeverBoxStatus': case 'ensureForeverBox':
      pending.value = null; break;
    case 'countAgents': pending.value = agents().length; break;
    case 'isAgentNetworkEnabled': case 'isGlobalSearchEnabled': case 'isEgressTunnelAvailable':
      pending.value = false; break;
    case 'getTeachRecordingStatus':
      pending.value = { state: 'idle', agentId: null, startedAtMs: null, maxDurationMs: 600_000 }; break;
    case 'getHumanConversationTranscript':
      // Freeze at request time. Returning the later mutable rows would hide
      // precisely the page/event and receipt/event races being qualified.
      pending.value = clone(transcript);
      refreshes.push(pending);
      return;
    case 'getHumanConversationTranscriptTail':
      try { pending.value = humanHistoryPage(request); }
      catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        humanHistoryRejections.push({ request: clone(request), message });
        reply(pending, { code: 'FIXTURE_INVALID_HUMAN_CURSOR', message });
        return;
      }
      if (request.args.beforeCursor != null || request.args.beforeSeq != null) {
        older.push(pending);
        if (!holdOlder) reply(pending);
        return;
      }
      if (holdInitial && id === 'human:a') { initials.push(pending); return; }
      break;
    case 'openAgentTail': case 'getAgentTranscriptTail':
      pending.value = { entries: clone(transcript) };
      if (holdAgentTails) { heldAgentTails.push(pending); return; }
      break;
    case 'sendHumanMessage': {
      reservedNonce = null;
      const intentKey = JSON.stringify([request.accountEpoch, id, request.args.clientNonce]);
      if (typeof request.args.clientNonce !== 'string' || request.args.clientNonce.length === 0
        || typeof request.args.text !== 'string' || humanIntents.has(intentKey)) {
        unhandledRequests.push(request);
        reply(pending, { code: 'FIXTURE_INVALID_HUMAN_SEND', message: 'Original Human input must be explicit and posted once' });
        return;
      }
      pending.value = row('human-server-message:' + (601 + sends.length), String(request.args.text), {
        authorId: request.accountId, clientNonce: request.args.clientNonce,
        ...(request.args.replyToId == null ? {} : { replyToId: request.args.replyToId }),
        attachments: clone(request.args.attachments ?? []),
      });
      pending.position = transcript.length;
      sends.push(pending);
      humanIntents.set(intentKey, pending);
      return;
    }
    case 'retryHumanMessage': {
      const original = humanIntents.get(JSON.stringify([request.accountEpoch, id, request.args.clientNonce]));
      if (Object.keys(request.args).sort().join(',') !== 'clientNonce,conversationId'
        || original == null || original.request.accountId !== request.accountId) {
        unhandledRequests.push(request);
        reply(pending, { code: 'FIXTURE_INVALID_HUMAN_RETRY', message: 'Retry must identify one original admitted Human intent without replaying its payload' });
        return;
      }
      // Freeze the response at retry admission, preserving the original nonce,
      // author, resources and canonical identity even if later events mutate it.
      pending.value = clone(original.value);
      pending.position = original.position;
      retries.push(pending);
      return;
    }
    case 'sendPrompt': {
      if (!includeAgent || id !== 'agent:helper' || typeof request.args.prompt !== 'string'
        || typeof request.args.clientNonce !== 'string' || request.args.clientNonce.length === 0) {
        unhandledRequests.push(request);
        reply(pending, { code: 'FIXTURE_INVALID_AGENT_SEND', message: 'Agent send requires the explicit existing owner identity and input' });
        return;
      }
      const paths = request.args.attachmentPaths as unknown[] | undefined;
      const names = request.args.attachmentNames as unknown[] | undefined;
      if (!Array.isArray(paths) || !Array.isArray(names) || paths.length !== names.length
        || paths.some((value) => typeof value !== 'string') || names.some((value) => typeof value !== 'string')) {
        unhandledRequests.push(request);
        reply(pending, { code: 'FIXTURE_INVALID_AGENT_ATTACHMENTS', message: 'Agent attachment arrays must agree' });
        return;
      }
      pending.value = {
        id: 'agent-accepted:' + (801 + agentSends.length), kind: 'message', role: 'user',
        content: request.args.prompt, timestampMs: 1_700_001_000_000 + (++sequence * 1000),
        delivery: 'sent', clientNonce: request.args.clientNonce,
        ...(request.args.richText == null ? {} : { richText: request.args.richText }),
        ...(request.args.replyToId == null ? {} : { replyToId: request.args.replyToId }),
        attachments: paths.map((path, index) => ({ path, name: names[index] })),
      };
      pending.position = transcript.length;
      agentSends.push(pending);
      return;
    }
    case 'reactHumanMessage': {
      const existing = transcript.find((entry) => entry.id === request.args.entryId);
      if (existing == null) throw new Error('Reaction target is absent from the fixture service: ' + String(request.args.entryId));
      pending.value = { ...clone(existing), reactions: [{ emoji: request.args.emoji, by: 'me' }] };
      pending.revision = revisions.get(revisionKey(id, existing.id));
      reactions.push(pending);
      return;
    }
    default:
      unhandledRequests.push(request);
      reply(pending, { code: 'FIXTURE_UNHANDLED_REQUEST', message: request.method });
      return;
  }
  reply(pending);
}

const coordinatorPort: CoordinatorPortBridge = {
  claim(consumer) {
    let owned: Endpoint | null = null;
    return {
      request() {
        const endpoint: Endpoint = { id: endpoints.length + 1, closed: false, listeners: new Set() };
        endpoints.push(endpoint);
        owned = endpoint;
        const port = {
          postMessage(frame: unknown) {
            const value = frame as Record<string, unknown>;
            if (value.kind === 'lifecycle' && value.phase === 'hello') {
              queueMicrotask(() => deliver(endpoint, { kind: 'lifecycle', phase: 'ready', protocolVersion: 1 }));
            } else if (value.kind === 'request') {
              queueMicrotask(() => receive(endpoint, value as unknown as { requestId: string; method: string; args?: Record<string, unknown> }));
            }
          },
          start() {},
          close() { endpoint.closed = true; endpoint.listeners.clear(); },
          addEventListener(type: string, listener: (event: { data: unknown }) => void) {
            if (type === 'message') endpoint.listeners.add(listener);
          },
        } as TransferredCoordinatorPort;
        consumer.onPort(port);
      },
      release() { if (owned != null) { owned.closed = true; owned.listeners.clear(); } },
    };
  },
};

// Non-message desktop capabilities remain external controls. Every coordinator
// method consumed by these scenarios has an explicit case above; unknown RPCs
// fail and are attached to the test report instead of receiving fake success.
function optionalCapability(name = '', values: Record<string, unknown> = {}): unknown {
  return new Proxy((...args: unknown[]) => {
    desktopCalls.push({ method: name, args: clone(args.filter((value) => typeof value !== 'function')) });
    return name.startsWith('on') ? () => {} : Promise.resolve(null);
  }, {
    get(_target, key) {
      if (key === 'then') return undefined;
      if (key in values) return values[String(key)];
      return optionalCapability(String(key));
    },
  });
}
const disabledUpdate = {
  state: { type: 'disabled', reason: 'not-packaged' }, currentVersion: '1.2.78', currentTrack: 'stable',
  trackOverride: null, buildDefaultTrack: 'stable', availableTracks: ['stable'], isTrackManagedByPolicy: false,
  isBelowMinimumVersion: false, autoUpdateWhenIdleOptIn: false, autoUpdateWhenIdleGateEnabled: false,
};
function writeClientPersistence(key: string, value: string): Promise<void> {
  let containsSubmissionBinding = false;
  if (key.endsWith('.composer-drafts')) {
    const envelope = JSON.parse(value) as { value?: { agents?: Record<string, { submissions?: unknown[] }> } };
    containsSubmissionBinding = Object.values(envelope.value?.agents ?? {}).some(
      (record) => Array.isArray(record.submissions) && record.submissions.length > 0,
    );
  }
  return new Promise<void>((resolve, reject) => {
    const held = holdNextSubmissionBindingWrite && containsSubmissionBinding;
    const pending: DraftPersistenceWrite = {
      index: draftPersistenceWrites.length, key, value, containsSubmissionBinding,
      state: held ? 'held' : 'written', failure: null, resolve, reject,
    };
    draftPersistenceWrites.push(pending);
    if (held) { holdNextSubmissionBindingWrite = false; return; }
    storage.set(key, value);
    storageWrites.push({ key, value });
    resolve();
  });
}
function readClientPersistence(key: string): Promise<string | null> {
  const value = storage.get(key) ?? null;
  if (!holdDraftReads || !key.endsWith('.composer-drafts')) return Promise.resolve(value);
  return new Promise<string | null>((resolve) => {
    heldDraftReads.push({ index: heldDraftReads.length, key, value, settled: false, resolve });
  });
}
const bridge = optionalCapability('', {
  platform: 'linux', isDev: false,
  getWindowState: async () => ({ isFullscreen: false, isMaximized: false }),
  getZoomFactor: () => 1,
  cursorAccount: optionalCapability('', {
    getStatus: async () => clone(account),
    getSandAccess: async () => ({ state: 'granted', reason: 'none' }),
    getSandAccessFresh: async () => ({ state: 'granted', reason: 'none' }),
    onStatusChanged: (listener: (status: CursorAuthStatus) => void) => { accountListeners.add(listener); return () => accountListeners.delete(listener); },
  }),
  onboarding: optionalCapability('', { getSeen: async () => true, setSeen: async () => {} }),
  theme: optionalCapability('', { initial: { preference: theme, resolved: theme }, get: async () => ({ preference: theme, resolved: theme }) }),
  experiments: optionalCapability('', { initialSnapshot: {}, getSnapshot: async () => ({}) }),
  uiPreferences: optionalCapability('', { get: async () => ({ locale: 'en', direction: 'ltr', reducedMotion: true, highContrast: false, textScale: 1 }) }),
  localToolPermission: optionalCapability('', {
    get: async () => 'ask', ceiling: async () => 'ask',
    clearApprovals: () => new Promise<void>((resolve) => {
      const pending: PendingApprovalClear = {
        index: approvalClears.length, accountId: currentAccountId(), accountEpoch, settled: false, resolve,
      };
      approvalClears.push(pending);
      desktopCalls.push({ method: 'localToolPermission.clearApprovals', index: pending.index });
      if (!holdApprovalClears) { pending.settled = true; resolve(); }
    }),
  }),
  autoReviewInstructions: optionalCapability('', { get: async () => ({ isEnabled: false, allowInstructions: [], blockInstructions: [] }) }),
  mcp: optionalCapability('', { list: async () => ({ servers: [] }), catalog: async () => [], effectivePlugins: async () => [], teamPopularity: async () => ({}) }),
  agent: optionalCapability('', {
    getPinnedAgents: async () => [], getSidebarSections: async () => [], getAvailableModels: async () => [],
    clientPersistence: {
      read: readClientPersistence,
      write: writeClientPersistence,
      remove: async (key: string) => { storage.delete(key); storageWrites.push({ key, value: null }); },
      listKeys: async (prefix: string) => [...storage.keys()].filter((key) => key.startsWith(prefix)),
      migrateFromLocalStorage: async () => true,
    },
  }),
  update: optionalCapability('', { getStatus: async () => disabledUpdate }),
  callMedia: optionalCapability('', { getPreferences: async () => ({ microphoneId: null, cameraId: null }) }),
  stageAttachmentBytes: async (name: string, bytes: Uint8Array) => {
    const path = 'memory:staged/' + name;
    desktopCalls.push({ method: 'stageAttachmentBytes', name, path, bytes: [...bytes] });
    return { ok: true, path };
  },
  commitStagedAttachments: async (paths: readonly string[], names: readonly string[]) => {
    desktopCalls.push({ method: 'commitStagedAttachments', paths: [...paths], names: [...names] });
    const index = attachmentCommits.length;
    const failure = attachmentCommitFailures.get(index) ?? null;
    const committedPaths = failure == null ? paths.map((_path, position) => 'memory:committed/' + names[position]) : [];
    attachmentCommits.push({ index, paths: [...paths], names: [...names], committedPaths, failure });
    if (failure != null) throw new Error(failure);
    return committedPaths;
  },
  discardStagedAttachment: async (path: string) => { desktopCalls.push({ method: 'discardStagedAttachment', path }); },
  resolveAttachmentMedia: async () => null,
}) as DesktopBridge;

function append(conversationId: string, entry: FixtureRow) {
  const transcript = rows.get(conversationId) ?? [];
  if (transcript.some((existing) => existing.id === entry.id)) throw new Error('Duplicate fixture entry: ' + entry.id);
  transcript.push(clone(entry));
  rows.set(conversationId, transcript);
  touch(conversationId, entry.id);
  emit({ type: 'appended', agentId: conversationId, entry: clone(entry) });
}
function mountRoot() {
  if (root != null) throw new Error('Fixture renderer is already mounted');
  const element = document.getElementById('root');
  if (element == null) throw new Error('Fixture root element is absent');
  root = createRoot(element);
  mountCount += 1;
  root.render(<ProductionRenderer bridge={bridge} coordinatorPort={coordinatorPort} />);
}
const fixture = {
  requests,
  pending: () => ({
    initials: initials.filter((entry) => !entry.settled).length,
    older: older.filter((entry) => !entry.settled).length,
    refreshes: refreshes.filter((entry) => !entry.settled).length,
    sends: sends.filter((entry) => !entry.settled).length,
    retries: retries.filter((entry) => !entry.settled).length,
    agentSends: agentSends.filter((entry) => !entry.settled).length,
    agentRosters: agentRosters.filter((entry) => !entry.settled).length,
    agentTails: heldAgentTails.filter((entry) => !entry.settled).length,
    reactions: reactions.filter((entry) => !entry.settled).length,
    sendRequests: sends.length, reactionRequests: reactions.length,
    retryRequests: retries.length, agentSendRequests: agentSends.length,
    approvalClears: approvalClears.filter((entry) => !entry.settled).length,
    approvalClearRequests: approvalClears.length,
    submissionBindingWrites: draftPersistenceWrites.filter((entry) => entry.state === 'held').length,
    draftReads: heldDraftReads.filter((entry) => !entry.settled).length,
  }),
  diagnostics: () => clone({
    account, accountEpoch, mountCount, requests, frames, unhandledRequests, desktopCalls, storageWrites,
    humanHistoryReads, humanHistoryRejections, attachmentCommits,
    heldAgentRosters: agentRosters.map(({ request, settled }) => ({ request, settled })),
    heldAgentTails: heldAgentTails.map(({ request, settled }) => ({ request, settled })),
    humanHistoryWalks: [...humanHistoryWalks.values()],
    approvalClears: approvalClears.map(({ index, accountId, accountEpoch, settled }) => ({ index, accountId, accountEpoch, settled })),
    draftPersistenceWrites: draftPersistenceWrites.map(({ index, key, value, containsSubmissionBinding, state, failure }) => ({ index, key, value, containsSubmissionBinding, state, failure })),
    heldDraftReads: heldDraftReads.map(({ index, key, value, settled }) => ({ index, key, value, settled })),
    clientPersistence: [...storage],
    endpoints: endpoints.map((endpoint) => ({ id: endpoint.id, closed: endpoint.closed, listeners: endpoint.listeners.size })),
    canonicalServiceRows: [...rows],
  }),
  releaseInitial() { holdInitial = false; for (const pending of initials) if (!pending.settled) reply(pending); },
  releaseAgentRosters() {
    holdAgentRoster = false;
    for (const pending of agentRosters) if (!pending.settled) reply(pending);
  },
  holdAgentTranscriptReads() {
    if (holdAgentTails) throw new Error('Agent transcript reads are already held');
    holdAgentTails = true;
  },
  releaseAgentTranscriptReads() {
    holdAgentTails = false;
    for (const pending of heldAgentTails) if (!pending.settled) reply(pending);
  },
  releaseOlder() { holdOlder = false; for (const pending of older) if (!pending.settled) reply(pending); },
  releaseRefreshes() { for (const pending of refreshes) if (!pending.settled) reply(pending); },
  holdApprovalClears() {
    if (holdApprovalClears) throw new Error('Approval clearing is already held');
    holdApprovalClears = true;
  },
  releaseApprovalClears() {
    holdApprovalClears = false;
    for (const pending of approvalClears) if (!pending.settled) { pending.settled = true; pending.resolve(); }
  },
  failAttachmentCommit(index: number, message: string) {
    if (!Number.isInteger(index) || index < attachmentCommits.length || attachmentCommitFailures.has(index)) {
      throw new Error('Attachment failure must select one future external commit call');
    }
    attachmentCommitFailures.set(index, message);
  },
  holdNextSubmissionBindingWrite() {
    if (holdNextSubmissionBindingWrite || draftPersistenceWrites.some((entry) => entry.state === 'held')) {
      throw new Error('A submission binding write is already held or armed');
    }
    holdNextSubmissionBindingWrite = true;
  },
  rejectSubmissionBindingWrite(message: string) {
    const pending = draftPersistenceWrites.find((entry) => entry.state === 'held');
    if (pending == null || !pending.containsSubmissionBinding) throw new Error('No held submission binding write');
    pending.state = 'failed';
    pending.failure = message;
    pending.reject(new Error(message));
  },
  holdComposerDraftReads() {
    if (holdDraftReads) throw new Error('Composer draft reads are already held');
    holdDraftReads = true;
  },
  releaseComposerDraftReads() {
    holdDraftReads = false;
    for (const pending of heldDraftReads) if (!pending.settled) { pending.settled = true; pending.resolve(pending.value); }
  },
  baseline(id: string, ids: string[]) {
    emit({ type: 'snapshot', activeAgentId: id, entries: clone((rows.get(id) ?? []).filter((entry) => ids.includes(entry.id))) });
  },
  mutate(id: string, entryId: string, patch: Record<string, unknown>) {
    const transcript = rows.get(id) ?? [];
    const index = transcript.findIndex((entry) => entry.id === entryId);
    if (index < 0) throw new Error('Missing fixture service row ' + entryId);
    const before = clone(transcript[index]);
    const after = { ...before, ...clone(patch) };
    transcript[index] = after;
    touch(id, entryId);
    emit({ type: 'updated', agentId: id, before, entry: clone(after) });
  },
  publishSend(index = 0, eventKind: 'appended' | 'updated' = 'appended') {
    const pending = pendingAt(sends, index, 'send');
    const entry = insertSend(pending);
    emit({
      type: eventKind, agentId: String(pending.request.args.conversationId), entry,
      ...(eventKind === 'updated' ? { before: { id: 'human-message:' + String(pending.request.args.clientNonce) } } : {}),
    });
    return entry.id;
  },
  peerCollision(index = 0) {
    const pending = pendingAt(sends, index, 'send');
    append(String(pending.request.args.conversationId), row('human-server-message:701', 'Independent peer message', { clientNonce: pending.request.args.clientNonce }));
  },
  legacyPeer(nonce: string) {
    append('human:a', row('human-server-message:702', 'Legacy peer content must survive', { clientNonce: nonce }));
  },
  nonceNegatives(index = 0) {
    const pending = pendingAt(sends, index, 'send');
    const nonce = pending.request.args.clientNonce;
    // Canonical ID and local Human author isolate the role-negative condition.
    append('human:a', row('human-server-message:703', 'Independent assistant content', { role: 'assistant', authorId: pending.request.accountId, clientNonce: nonce }));
    const unscoped = row('human-server-message:704', 'Unscoped Human content', { clientNonce: nonce });
    delete unscoped.authorId;
    append('human:a', unscoped);
  },
  reserveSendNonce(nonce: string) {
    if (!/^[0-9a-f-]{36}$/.test(nonce) || reservedNonce != null) throw new Error('Invalid or overlapping deterministic nonce reservation');
    reservedNonce = nonce;
  },
  settleSend(index = 0) {
    const pending = pendingAt(sends, index, 'send');
    if (pending.request.accountEpoch === accountEpoch) insertSend(pending);
    reply(pending);
  },
  rejectSend(index = 0, message = 'Native message acknowledgement rejected') {
    reply(pendingAt(sends, index, 'send'), { code: 'NATIVE_MESSAGE_REJECTED', message });
  },
  publishRetry(index = 0) {
    const pending = retries[index];
    if (pending == null) throw new Error('No Human retry at index ' + index);
    const entry = insertSend(pending);
    emit({ type: 'appended', agentId: String(pending.request.args.conversationId), entry });
    return entry.id;
  },
  settleRetry(index = 0) {
    const pending = pendingAt(retries, index, 'Human retry');
    if (pending.request.accountEpoch === accountEpoch) insertSend(pending);
    reply(pending);
  },
  rejectRetry(index = 0, message = 'Native retry acknowledgement rejected') {
    reply(pendingAt(retries, index, 'Human retry'), { code: 'NATIVE_MESSAGE_RETRY_REJECTED', message });
  },
  publishAgentSend(index = 0) {
    const pending = pendingAt(agentSends, index, 'Agent send');
    const entry = insertSend(pending);
    emit({ type: 'appended', agentId: String(pending.request.args.agentId), entry });
    return entry.id;
  },
  settleAgentSend(index = 0) {
    const pending = pendingAt(agentSends, index, 'Agent send');
    if (pending.request.accountEpoch === accountEpoch) insertSend(pending);
    pending.value = null; // sendPrompt acknowledges acceptance, not a Human row.
    reply(pending);
  },
  rejectAgentSend(index = 0, message = 'Agent dispatch rejected') {
    reply(pendingAt(agentSends, index, 'Agent send'), { code: 'AGENT_SEND_REJECTED', message });
  },
  settleReaction(index = 0) {
    const pending = pendingAt(reactions, index, 'reaction');
    const id = String(pending.request.args.conversationId);
    const entry = pending.value as FixtureRow;
    if (pending.request.accountEpoch === accountEpoch && revisions.get(revisionKey(id, entry.id)) === pending.revision) {
      const transcript = rows.get(id) ?? [];
      const position = transcript.findIndex((candidate) => candidate.id === entry.id);
      if (position >= 0) { transcript[position] = clone(entry); touch(id, entry.id); }
    }
    reply(pending);
  },
  disconnect() {
    for (const endpoint of endpoints) if (!endpoint.closed) {
      deliver(endpoint, { kind: 'event', family: 'coordinator-transport-state', payload: { state: 'down' } });
    }
  },
  reconnect() {
    for (const endpoint of endpoints) if (!endpoint.closed) {
      deliver(endpoint, { kind: 'event', family: 'coordinator-transport-state', payload: { state: 'connected' } });
    }
  },
  clear(id: string) {
    for (const entry of rows.get(id) ?? []) touch(id, entry.id);
    invalidateHumanHistory(id);
    rows.set(id, []);
    emit({ type: 'cleared', agentId: id });
  },
  appendCurrent(id: string, entryId: string, content: string) {
    append(id, row(entryId, content, { authorId: id === 'human:b' ? '3' : '2' }));
  },
  setAccount(authId: string | null, entryId = 'human-server-message:901', content = 'New account conversation') {
    accountEpoch += 1;
    invalidateHumanHistory();
    account = authId == null ? { kind: 'logged-out' } : { kind: 'logged-in', authId, email: authId + '@example.test', displayName: authId };
    if (authId != null) {
      rows.set('human:a', [row(entryId, content)]);
      rows.set('human:b', [row('human-server-message:902', 'New account Bob conversation', { authorId: '3' })]);
      touch('human:a', entryId);
      touch('human:b', 'human-server-message:902');
    }
    for (const listener of [...accountListeners]) listener(clone(account));
  },
  replaceConversation(id: string, entryId: string, content: string) {
    invalidateHumanHistory(id);
    rows.set(id, [row(entryId, content)]);
    touch(id, entryId);
  },
  unmount() {
    if (root == null) throw new Error('Fixture renderer is not mounted');
    root.unmount();
    root = null;
  },
  mount: mountRoot,
};

export type NativeMessageAuthorRecoveryFixture = typeof fixture;
declare global { interface Window { nativeMessageAuthorRecoveryFixture: NativeMessageAuthorRecoveryFixture; } }
window.nativeMessageAuthorRecoveryFixture = fixture;
mountRoot();
