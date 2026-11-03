import React from 'react';
import { createRoot } from 'react-dom/client';
import { ProductionRenderer } from '../../../frontend/src/production/ProductionRenderer';
import type { CoordinatorPortBridge, CursorAuthStatus, DesktopBridge, TransferredCoordinatorPort } from '../../../frontend/src/recovered/contracts/desktop-bridge';

// This fixture mounts the shipping root and its real coordinator client/feed.
// Only the external desktop port and service replies are controlled by the test.
type Row = Record<string, unknown> & { id: string };
type Request = { requestId: string; method: string; args: Record<string, unknown> };
type Pending = { request: Request; value: unknown };
const clone = <T,>(value: T): T => structuredClone(value);
const accountListeners = new Set<(status: CursorAuthStatus) => void>();
const focusListeners = new Set<(value: unknown) => void>();
const theme = new URLSearchParams(location.search).get('theme') === 'light' ? 'light' : 'dark';
let account: CursorAuthStatus = { kind: 'logged-in', authId: 'local', email: 'local@example.test', displayName: 'Local' };
const storage = new Map<string, string>();
let portListener: ((event: { data: unknown }) => void) | null = null;
const requests: Request[] = [];
const approvalClears: Array<string | null> = [];
const older: Pending[] = [];
const initials: Pending[] = [];
const refreshes: Pending[] = [];
const sends: Pending[] = [];
const humanRetries: Pending[] = [];
const reactions: Pending[] = [];
const sendRosterRefreshes: Pending[] = [];
const sendPositions = new Map<string, number>();
let holdOlder = true;
let holdInitial = new URLSearchParams(location.search).get('holdInitial') === '1';
let holdNextSendRoster = false;
let holdReady = false;
let failNextSendPost = false;
let attachmentCommitMode: 'normal' | 'fail' | 'hold' = 'normal';
const attachmentCommits: string[][] = [];
const heldAttachmentCommits: Array<() => void> = [];
let sequence = 0;
const row = (id: string, text: string, patch: Record<string, unknown> = {}): Row => ({
  id, kind: 'message', role: 'user', authorKind: 'human', authorId: 'peer-a', content: text,
  timestampMs: 1_700_000_000_000 + (++sequence * 1000), delivery: 'sent', attachments: [], reactions: [], ...patch,
});
const rows = new Map<string, Row[]>([
  ['human:a', [row('m1', 'Old first message'), row('m2', 'Old second message', { attachments: [{ path: 'memory:note.txt', name: 'note.txt' }] }), row('m3', 'Tail third message'), row('m4', 'Tail fourth message', { attachments: [{ path: 'memory:tail-note.txt', name: 'tail-note.txt' }] }), row('m5', 'Tail fifth message')]],
  ['human:b', [row('b1', 'Bob conversation', { authorId: 'peer-b' })]],
]);
const savedRows = sessionStorage.getItem('native-message-fixture-canonical-rows');
if (savedRows != null) {
  rows.clear();
  for (const [id, entries] of JSON.parse(savedRows) as Array<[string, Row[]]>) rows.set(id, entries);
}
const persistRows = () => sessionStorage.setItem('native-message-fixture-canonical-rows', JSON.stringify([...rows]));
const humans = () => [
  { id: 'human:a', kind: 'human', title: 'Alice', participantIds: [account.kind === 'logged-in' ? account.authId : '', 'peer-a'], updatedAt: 200 },
  { id: 'human:b', kind: 'human', title: 'Bob', participantIds: [account.kind === 'logged-in' ? account.authId : '', 'peer-b'], updatedAt: 100 },
];
const sendFrame = (frame: unknown) => portListener?.({ data: clone(frame) });
const reply = (pending: Pending) => sendFrame({ kind: 'reply', requestId: pending.request.requestId, outcome: { status: 'ok', value: pending.value } });
const emit = (payload: unknown) => sendFrame({ kind: 'event', family: 'transcript', payload });

function receive(request: Request) {
  requests.push(clone(request));
  const id = String(request.args.conversationId ?? request.args.id ?? '');
  const transcript = rows.get(id) ?? [];
  let value: unknown = null;
  switch (request.method) {
    case 'listAgents': case 'searchAgents': case 'getTrays': case 'listAllAutomations':
    case 'getListenerIntegrations': case 'getAgentAutomations': case 'getAgentChannels':
    case 'listHumanCalls': case 'getPendingHumanCalls': case 'listCallSessions': value = []; break;
    case 'listHumanConversations':
      if (holdNextSendRoster) {
        holdNextSendRoster = false;
        sendRosterRefreshes.push({ request, value: humans() });
        return;
      }
      value = humans(); break;
    case 'syncHumanConversations': value = humans(); break;
    case 'syncHumanConversation': value = { conversationId: id }; break;
    case 'countAgents': value = 0; break;
    case 'isAgentNetworkEnabled': value = false; break;
    case 'getHumanConversationTranscript':
      refreshes.push({ request, value: clone(transcript) });
      return;
    case 'getHumanConversationTranscriptTail':
      if (typeof request.args.beforeSeq === 'number') {
        const pending = { request, value: { entries: clone(transcript.slice(0, 2)) } };
        if (holdOlder) older.push(pending);
        else reply(pending);
        return;
      }
      value = id === 'human:a' && transcript.length > 3
        ? { entries: clone(transcript.slice(-3)), nextBeforeSeq: 3, nextBeforeCursor: 'fixture-human-walk:3' }
        : { entries: clone(transcript) };
      if (holdInitial && id === 'human:a') { initials.push({ request, value }); return; }
      break;
    case 'openAgentTail': case 'getAgentTranscriptTail': value = { entries: [] }; break;
    case 'retryHumanMessage': {
      // Exercise the real renderer/Coordinator boundary with a Host-authoritative
      // outbox identity. No renderer-provided replacement fields are accepted.
      if (Object.keys(request.args).sort().join(',') !== 'clientNonce,conversationId')
        throw new Error('Human retry added replacement payload outside its two-field contract');
      const existing = transcript.find((entry) =>
        entry.id === `human-message:${String(request.args.clientNonce)}`
        && entry.clientNonce === request.args.clientNonce
        && entry.authorKind === 'human'
        && entry.authorId === (account.kind === 'logged-in' ? account.authId : null)
      );
      if (existing == null) throw new Error('No durable locally authored Human message for retry');
      const remoteMessageId = '901';
      const recovered = {
        ...clone(existing), id: `human-server-message:${remoteMessageId}`,
        delivery: 'sent', remoteMessageId, timestampMs: Date.now(),
      };
      delete recovered.lastDispatchError;
      humanRetries.push({ request, value: recovered });
      return;
    }
    case 'sendHumanMessage': {
      const remoteMessageId = String(100 + sends.length);
      const entry = row(`human-server-message:${remoteMessageId}`, String(request.args.text), {
        authorId: account.kind === 'logged-in' ? account.authId : '', clientNonce: request.args.clientNonce,
        remoteMessageId,
      });
      sends.push({ request, value: entry });
      sendPositions.set(request.requestId, transcript.length);
      return;
    }
    case 'reactHumanMessage': {
      const existing = transcript.find((entry) => entry.id === request.args.entryId);
      if (existing == null) throw new Error('The fixture reaction target must be a canonical row');
      const entry = { ...existing, reactions: [{ emoji: request.args.emoji, by: 'me' }] };
      reactions.push({ request, value: clone(entry) });
      return;
    }
  }
  reply({ request, value });
}

const coordinatorPort: CoordinatorPortBridge = {
  claim(consumer) {
    const port: TransferredCoordinatorPort = {
      postMessage(frame) {
        const value = frame as Record<string, unknown>;
        if (value.kind === 'request' && value.method === 'sendHumanMessage' && failNextSendPost) {
          failNextSendPost = false;
          throw new Error('fixture request frame was not posted');
        }
        if (value.kind === 'lifecycle' && value.phase === 'hello') {
          if (!holdReady) queueMicrotask(() => sendFrame({ kind: 'lifecycle', phase: 'ready', protocolVersion: 1 }));
        } else if (value.kind === 'request') queueMicrotask(() => receive(value as unknown as Request));
      },
      start() {}, close() {},
      addEventListener(type: string, listener: (event: { data: unknown }) => void) {
        if (type === 'message') portListener = listener;
      },
    } as TransferredCoordinatorPort;
    return { request: () => consumer.onPort(port), release() {} };
  },
};

// Unused desktop capabilities are inert. Required data reads and subscriptions
// below have explicit production-shaped results; message calls are all above.
function optionalCapability(name = '', values: Record<string, unknown> = {}): unknown {
  return new Proxy(() => name.startsWith('on') ? () => {} : Promise.resolve(null), {
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
const bridge = optionalCapability('', {
  platform: 'linux', isDev: false,
  getWindowState: async () => ({ isFullscreen: false, isMaximized: false }),
  getZoomFactor: () => 1,
  onFocusAgent: (listener: (value: unknown) => void) => { focusListeners.add(listener); return () => focusListeners.delete(listener); },
  cursorAccount: optionalCapability('', {
    getStatus: async () => account,
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
    clearApprovals: async () => { approvalClears.push(account.kind === 'logged-in' ? account.authId ?? null : null); },
  }),
  autoReviewInstructions: optionalCapability('', { get: async () => ({ isEnabled: false, allowInstructions: [], blockInstructions: [] }) }),
  mcp: optionalCapability('', { list: async () => ({ servers: [] }), catalog: async () => [], effectivePlugins: async () => [], teamPopularity: async () => ({}) }),
  agent: optionalCapability('', {
    getPinnedAgents: async () => [], getSidebarSections: async () => [],
    clientPersistence: {
      read: async (key: string) => storage.get(key) ?? null,
      write: async (key: string, value: string) => { storage.set(key, value); },
      remove: async (key: string) => { storage.delete(key); },
      listKeys: async (prefix: string) => [...storage.keys()].filter((key) => key.startsWith(prefix)),
      migrateFromLocalStorage: async () => true,
    },
  }),
  update: optionalCapability('', { getStatus: async () => disabledUpdate }),
  callMedia: optionalCapability('', { getPreferences: async () => ({ microphoneId: null, cameraId: null }) }),
  stageAttachmentBytes: async (filename: string) => ({ ok: true, path: `/staged/${filename}` }),
  commitStagedAttachments: async (paths: readonly string[]) => {
    attachmentCommits.push([...paths]);
    const mode = attachmentCommitMode;
    attachmentCommitMode = 'normal';
    if (mode === 'fail') throw new Error('Attachment commit failed before Host admission');
    if (mode === 'hold') await new Promise<void>((resolve) => heldAttachmentCommits.push(resolve));
    return paths.map((path) => path.replace('/staged/', '/committed/'));
  },
}) as DesktopBridge;

const fixture = {
  requests,
  approvalClears,
  attachmentCommits,
  pending: () => ({ initials: initials.length, older: older.length, refreshes: refreshes.length, sends: sends.length, humanRetries: humanRetries.length, reactions: reactions.length, sendRosterRefreshes: sendRosterRefreshes.length }),
  stageFailedHuman(nonce: string) {
    const id = 'human:a';
    const source = row(`human-message:${nonce}`, 'Original pending Human text', {
      authorId: account.kind === 'logged-in' ? account.authId : '',
      clientNonce: nonce, delivery: 'pending', composedAtMs: 1234,
      lastDispatchError: 'Original ambiguous request ended without receipt',
      replyToId: 'human-server-message:7',
      attachments: [{ path: '/committed/original.txt', name: 'original.txt', resourceId: 'resource-1' }],
    });
    const list = rows.get(id) ?? [];
    list.push(source);
    rows.set(id, list);
    persistRows();
    emit({ type: 'appended', agentId: id, entry: clone(source) });
  },
  rejectHumanRetry(index: number) {
    const pending = humanRetries[index];
    if (pending == null) throw new Error('No Human retry to reject');
    sendFrame({ kind: 'reply', requestId: pending.request.requestId,
      outcome: { status: 'failed', failure: { code: 'fixture-retry', message: 'temporary retry refusal' } } });
  },
  replyHumanRetryOutcome(index: number, outcome: unknown) {
    const pending = humanRetries[index];
    if (pending == null) throw new Error('No Human retry for the supplied reply outcome');
    sendFrame({ kind: 'reply', requestId: pending.request.requestId, outcome });
  },
  replyHumanRetry(index: number, patch: Record<string, unknown> = {}) {
    const pending = humanRetries[index];
    if (pending == null) throw new Error('No Human retry reply');
    reply({ ...pending, value: { ...clone(pending.value as Row), ...patch } });
  },
  settleHumanRetry(index: number) {
    const pending = humanRetries[index];
    if (pending == null) throw new Error('No Human retry settlement');
    const conversationId = String(pending.request.args.conversationId);
    const value = clone(pending.value as Row);
    const entries = rows.get(conversationId) ?? [];
    const oldId = `human-message:${pending.request.args.clientNonce}`;
    const idx = entries.findIndex((entry) => entry.id === oldId);
    if (idx < 0) throw new Error('Retry settlement lost durable original message');
    entries[idx] = value;
    rows.set(conversationId, entries);
    persistRows();
    reply(pending);
  },
  releaseInitial(newestFirst = false) {
    holdInitial = false;
    const pending = initials.splice(0);
    for (const initial of newestFirst ? pending.reverse() : pending) reply(initial);
  },
  releaseOlder() { holdOlder = false; for (const pending of older.splice(0)) reply(pending); },
  releaseRefreshes() { for (const pending of refreshes.splice(0)) reply(pending); },
  baseline(id: string, ids: string[]) { emit({ type: 'snapshot', activeAgentId: id, entries: clone((rows.get(id) ?? []).filter((entry) => ids.includes(entry.id))) }); },
  clear(id: string) {
    rows.set(id, []);
    persistRows();
    emit({ type: 'cleared', agentId: id });
  },
  mutate(id: string, entryId: string, patch: Record<string, unknown>, omittedFields: string[] = []) {
    const transcript = rows.get(id) ?? [];
    const index = transcript.findIndex((entry) => entry.id === entryId);
    if (index < 0) throw new Error(`Missing canonical fixture row ${entryId}`);
    const before = clone(transcript[index]);
    const after = { ...before, ...patch };
    transcript[index] = after;
    persistRows();
    const eventEntry = clone(after);
    for (const field of omittedFields) delete eventEntry[field];
    emit({ type: 'updated', agentId: id, before, entry: eventEntry });
  },
  publishSend(index = 0) {
    const pending = sends[index];
    if (pending == null) throw new Error('No pending send');
    const id = String(pending.request.args.conversationId);
    const entry = clone(pending.value as Row);
    const transcript = rows.get(id) ?? [];
    if (!transcript.some((candidate) => candidate.id === entry.id)) transcript.splice(sendPositions.get(pending.request.requestId) ?? transcript.length, 0, entry);
    rows.set(id, transcript);
    persistRows();
    emit({ type: 'updated', agentId: id, before: { id: `pending-${pending.request.args.clientNonce}` }, entry });
    return entry.id;
  },
  peerCollision(index = 0) {
    const pending = sends[index];
    if (pending == null) throw new Error('No pending send');
    const id = String(pending.request.args.conversationId);
    const entry = row('human-server-message:200', 'Independent peer message', {
      clientNonce: pending.request.args.clientNonce, remoteMessageId: '200',
    });
    rows.get(id)!.push(entry);
    persistRows();
    emit({ type: 'appended', agentId: id, entry });
  },
  settleSend(index = 0) { const pending = sends[index]; if (pending == null) throw new Error('No pending send'); reply(pending); },
  replySend(index: number, patch: Record<string, unknown> = {}, omittedFields: string[] = []) {
    const pending = sends[index];
    if (pending == null) throw new Error('No pending send reply');
    const value = { ...clone(pending.value as Row), ...patch };
    for (const field of omittedFields) delete value[field];
    reply({ ...pending, value });
  },
  rejectSend(index = 0) {
    const pending = sends[index];
    if (pending == null) throw new Error('No pending send to reject');
    sendFrame({ kind: 'reply', requestId: pending.request.requestId,
      outcome: { status: 'failed', failure: { code: 'fixture-send', message: 'temporary send refusal' } } });
  },
  holdSendRosterRefresh() { holdNextSendRoster = true; },
  failSendRosterRefresh() {
    for (const pending of sendRosterRefreshes.splice(0)) sendFrame({
      kind: 'reply', requestId: pending.request.requestId,
      outcome: { status: 'failed', failure: { code: 'fixture-roster', message: 'post-send roster unavailable' } },
    });
  },
  settleReaction(index = 0) { const pending = reactions[index]; if (pending == null) throw new Error('No pending reaction'); reply(pending); },
  failNextSendPost() { failNextSendPost = true; },
  pauseReadiness() { holdReady = true; sendFrame({ kind: 'lifecycle', phase: 'shutdown' }); },
  resumeReadiness() { holdReady = false; sendFrame({ kind: 'lifecycle', phase: 'ready', protocolVersion: 1 }); },
  failNextAttachmentCommit() { attachmentCommitMode = 'fail'; },
  holdNextAttachmentCommit() { attachmentCommitMode = 'hold'; },
  heldAttachmentCommitCount: () => heldAttachmentCommits.length,
  releaseAttachmentCommits() { for (const release of heldAttachmentCommits.splice(0)) release(); },
  unmount() { root.unmount(); },
  disconnect() { sendFrame({ kind: 'event', family: 'coordinator-transport-state', payload: { state: 'down' } }); },
  reconnect() { sendFrame({ kind: 'event', family: 'coordinator-transport-state', payload: { state: 'connected' } }); },
  setAccount(authId: string | null) {
    account = authId == null ? { kind: 'logged-out' } : { kind: 'logged-in', authId, email: `${authId}@example.test`, displayName: authId };
    if (authId != null && authId !== 'local') {
      rows.set('human:a', [row('new-a', 'New account conversation', { authorId: authId })]);
      rows.set('human:b', [row('new-b', 'New account Bob conversation', { authorId: authId })]);
      persistRows();
    }
    for (const listener of accountListeners) listener(account);
  },
};

declare global { interface Window { nativeMessageFixture: typeof fixture; } }
window.nativeMessageFixture = fixture;
const root = createRoot(document.getElementById('root')!);
root.render(<ProductionRenderer bridge={bridge} coordinatorPort={coordinatorPort} />);
