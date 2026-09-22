import type { RuntimeCommand, RuntimeEvent } from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MahayanaHostTransport } from '../../frontend/apps/web/src/lib/mahayana-host/transport';

export type StartupCriticalPathPhase = 'P0' | 'P1' | 'P2' | 'P3' | 'P4' | 'P5' | 'P6' | 'P7' | 'P8' | 'P9';

export type StartupCriticalPathEntry = {
  phase: StartupCriticalPathPhase;
  label: string;
  atMs: number;
  sinceNavigationMs: number;
  durationMs?: number;
  sourceFile: string;
  sourceFunction: string;
  details: Record<string, unknown>;
};

export type StartupCriticalPathTrace = {
  schemaVersion: 1;
  taskId: 'M3-DESKTOP-003';
  timeOriginEpochMs: number;
  navigationStartMs: number;
  entries: StartupCriticalPathEntry[];
  longTasks: Array<{ startTimeMs: number; durationMs: number; name: string }>;
  longTaskObservation: { supported: boolean; reason?: string };
  phaseSources: Record<StartupCriticalPathPhase, string[]>;
};

type StartupCriticalPathWindow = Window & {
  __fabushiStartupCriticalPath?: StartupCriticalPathTrace;
  __fabushiStartupCriticalPathLongTaskObserver?: PerformanceObserver;
};

const startupCriticalPathFile = 'desktop/src/selfhosted-messaging-client-v2.ts';
const startupCriticalPathPhaseSources: Record<StartupCriticalPathPhase, string[]> = {
  P0: ['initializeStartupCriticalPathDiagnostics'],
  P1: ['initializeStartupCriticalPathDiagnostics'],
  P2: ['initializeStartupCriticalPathDiagnostics'],
  P3: ['instrumentStartupTransport.authStatus'],
  P4: ['instrumentStartupTransport.initialize'],
  P5: ['SelfHostedMessagingClientV2.sync', 'asMessagingHostEvent'],
  P6: ['asMessagingHostEvent'],
  P7: ['messagingText'],
  P8: ['instrumentStartupTransport.subscribe', 'asMessagingHostEvent'],
  P9: ['SelfHostedMessagingClientV2.sync'],
};
const startupCriticalPathWindow = typeof window === 'undefined' ? null : window as StartupCriticalPathWindow;
let initialSyncRequestStartedAtMs: number | null = null;
let firstVisibleMessageScheduled = false;

function startupNow(): number {
  return typeof performance === 'undefined' ? 0 : performance.now();
}

function startupSerializedByteLength(value: unknown): number | null {
  try {
    const serialized = typeof value === 'string' ? value : JSON.stringify(value);
    if (typeof TextEncoder !== 'undefined') return new TextEncoder().encode(serialized).byteLength;
    return serialized.length;
  } catch {
    return null;
  }
}

function ensureStartupCriticalPathTrace(): StartupCriticalPathTrace | null {
  if (!startupCriticalPathWindow || typeof performance === 'undefined') return null;
  if (!startupCriticalPathWindow.__fabushiStartupCriticalPath) {
    const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming | undefined;
    startupCriticalPathWindow.__fabushiStartupCriticalPath = {
      schemaVersion: 1,
      taskId: 'M3-DESKTOP-003',
      timeOriginEpochMs: performance.timeOrigin,
      navigationStartMs: navigation?.startTime ?? 0,
      entries: [],
      longTasks: [],
      longTaskObservation: { supported: typeof PerformanceObserver !== 'undefined' },
      phaseSources: startupCriticalPathPhaseSources,
    };
  }
  return startupCriticalPathWindow.__fabushiStartupCriticalPath;
}

function recordStartupCriticalPath(
  phase: StartupCriticalPathPhase,
  label: string,
  details: Record<string, unknown> = {},
  durationMs?: number,
  sourceFunction = startupCriticalPathPhaseSources[phase][0],
): void {
  const trace = ensureStartupCriticalPathTrace();
  if (!trace) return;
  const atMs = startupNow();
  trace.entries.push({
    phase,
    label,
    atMs,
    sinceNavigationMs: Math.max(0, atMs - trace.navigationStartMs),
    ...(durationMs === undefined ? {} : { durationMs }),
    sourceFile: startupCriticalPathFile,
    sourceFunction,
    details,
  });
}

function recordStartupCriticalPathOnce(
  phase: StartupCriticalPathPhase,
  label: string,
  details: Record<string, unknown> = {},
  durationMs?: number,
  sourceFunction?: string,
): void {
  const trace = ensureStartupCriticalPathTrace();
  if (!trace || trace.entries.some((entry) => entry.phase === phase && entry.label === label)) return;
  recordStartupCriticalPath(phase, label, details, durationMs, sourceFunction);
}

function initializeStartupCriticalPathDiagnostics(): void {
  const trace = ensureStartupCriticalPathTrace();
  if (!trace || !startupCriticalPathWindow) return;
  recordStartupCriticalPathOnce('P0', 'renderer-navigation-observed', {
    documentReadyState: document.readyState,
    navigationStartMs: trace.navigationStartMs,
    timeOriginEpochMs: trace.timeOriginEpochMs,
  });

  const projectionStartedAtMs = startupNow();
  try {
    const rawProjection = window.localStorage.getItem('fabushi.desktop.messenger-projection.v1');
    recordStartupCriticalPathOnce('P1', 'projection-cache-observed', {
      source: 'localStorage',
      cacheHit: rawProjection !== null,
      bytes: rawProjection === null ? 0 : startupSerializedByteLength(rawProjection),
    }, startupNow() - projectionStartedAtMs);
  } catch (cause) {
    recordStartupCriticalPathOnce('P1', 'projection-cache-observed', {
      source: 'localStorage',
      cacheHit: false,
      error: cause instanceof Error ? cause.name : 'unknown',
    }, startupNow() - projectionStartedAtMs);
  }

  if (typeof PerformanceObserver === 'undefined') {
    trace.longTaskObservation = { supported: false, reason: 'PerformanceObserver unavailable' };
  } else if (!startupCriticalPathWindow.__fabushiStartupCriticalPathLongTaskObserver) {
    try {
      const observer = new PerformanceObserver((list) => {
        const current = ensureStartupCriticalPathTrace();
        if (!current) return;
        for (const entry of list.getEntries()) {
          current.longTasks.push({ startTimeMs: entry.startTime, durationMs: entry.duration, name: entry.name });
        }
      });
      observer.observe({ type: 'longtask', buffered: true } as PerformanceObserverInit);
      startupCriticalPathWindow.__fabushiStartupCriticalPathLongTaskObserver = observer;
      trace.longTaskObservation = { supported: true };
    } catch (cause) {
      trace.longTaskObservation = {
        supported: false,
        reason: cause instanceof Error ? cause.message : String(cause),
      };
    }
  }

  window.requestAnimationFrame(() => {
    window.requestAnimationFrame(() => {
      const firstPaint = performance.getEntriesByName('first-paint')[0];
      const firstContentfulPaint = performance.getEntriesByName('first-contentful-paint')[0];
      recordStartupCriticalPathOnce('P2', 'renderer-first-frame-interactive', {
        documentReadyState: document.readyState,
        firstPaintMs: firstPaint?.startTime ?? null,
        firstContentfulPaintMs: firstContentfulPaint?.startTime ?? null,
      });
    });
  });
}

function instrumentStartupTransport(transport: MahayanaHostTransport): void {
  if (!startupCriticalPathWindow) return;
  const instrumented = transport as MahayanaHostTransport & { __fabushiStartupCriticalPathInstrumentedV1?: boolean };
  if (instrumented.__fabushiStartupCriticalPathInstrumentedV1) return;
  instrumented.__fabushiStartupCriticalPathInstrumentedV1 = true;

  const initialize = transport.initialize.bind(transport);
  instrumented.initialize = async (config) => {
    const startedAtMs = startupNow();
    try {
      const result = await initialize(config);
      recordStartupCriticalPathOnce('P4', 'host-initialize-resolved', { outcome: 'resolved' }, startupNow() - startedAtMs, 'instrumentStartupTransport.initialize');
      return result;
    } catch (cause) {
      recordStartupCriticalPathOnce('P4', 'host-initialize-rejected', {
        outcome: 'rejected',
        error: cause instanceof Error ? cause.name : 'unknown',
      }, startupNow() - startedAtMs, 'instrumentStartupTransport.initialize');
      throw cause;
    }
  };

  const authStatus = transport.authStatus.bind(transport);
  instrumented.authStatus = async () => {
    const startedAtMs = startupNow();
    try {
      const state = await authStatus();
      recordStartupCriticalPathOnce('P3', 'auth-restore-observed', {
        loggedIn: state.loggedIn,
        outcome: 'resolved',
      }, startupNow() - startedAtMs, 'instrumentStartupTransport.authStatus');
      return state;
    } catch (cause) {
      recordStartupCriticalPathOnce('P3', 'auth-restore-observed', {
        outcome: 'rejected',
        error: cause instanceof Error ? cause.name : 'unknown',
      }, startupNow() - startedAtMs, 'instrumentStartupTransport.authStatus');
      throw cause;
    }
  };

  const subscribe = transport.subscribe.bind(transport);
  instrumented.subscribe = (listener) => {
    const installedAtMs = startupNow();
    let firstEvent = true;
    return subscribe((event) => {
      if (firstEvent) {
        firstEvent = false;
        recordStartupCriticalPathOnce('P8', 'event-stream-first-event', {
          eventType: event.type,
          subscriptionToFirstEventMs: startupNow() - installedAtMs,
        }, undefined, 'instrumentStartupTransport.subscribe');
      }
      listener(event);
    });
  };
}

initializeStartupCriticalPathDiagnostics();

export const FABUSHI_MESSAGING_PROTOCOL_VERSION = 2 as const;

export type ActorKind = 'human' | 'assistant' | 'bot' | 'service';
export type ConversationKind = 'direct' | 'group' | 'channel' | 'savedMessages' | 'secret';
export type ParticipantRole = 'owner' | 'admin' | 'member' | 'restricted';

export interface MessagingActor {
  id: string;
  kind: ActorKind;
  displayName: string;
  username?: string;
  avatarUrl?: string;
  bio?: string;
  capabilities: string[];
  presence: {
    status: 'offline' | 'online' | 'away' | 'doNotDisturb';
    lastSeenAtMs?: number;
    statusText?: string;
  };
  verified: boolean;
}

export interface MessagingParticipant {
  actorId: string;
  role: ParticipantRole;
  joinedAtMs: number;
  mutedUntilMs?: number;
}

export interface MessagingConversation {
  id: string;
  kind: ConversationKind;
  title: string;
  description?: string;
  avatarUrl?: string;
  participants: MessagingParticipant[];
  ownerId?: string;
  lastMessageId?: string;
  lastReadMessageId?: string;
  unreadCount: number;
  mentionCount: number;
  pinnedMessageIds: string[];
  notificationSettings: {
    mutedUntilMs?: number;
    sound?: string;
    showPreview: boolean;
    notifyMentions: boolean;
  };
  permissions: {
    canSendMessages: boolean;
    canSendMedia: boolean;
    canSendPolls: boolean;
    canAddMembers: boolean;
    canPinMessages: boolean;
    canManageTopics: boolean;
    canManageCalls: boolean;
  };
  historyVisibility: 'newMembersOnly' | 'allMembers';
  topics: Array<{
    id: string;
    title: string;
    icon?: string;
    createdBy: string;
    closed: boolean;
    hidden: boolean;
  }>;
  folderIds: string[];
  archived: boolean;
  pinned: boolean;
  markedUnread: boolean;
  createdAtMs: number;
  updatedAtMs: number;
}

export interface MessagingMessage {
  id: string;
  conversationId: string;
  senderId: string;
  content: Record<string, unknown> & { type: string };
  replyToMessageId?: string;
  threadRootMessageId?: string;
  forwardOrigin?: string;
  reactions: Array<{
    reaction: string;
    count: number;
    chosenByMe: boolean;
    recentActorIds: string[];
  }>;
  deliveryState: Record<string, unknown> & { state: string };
  createdAtMs: number;
  editedAtMs?: number;
  scheduledAtMs?: number;
  silent: boolean;
  protectedContent: boolean;
  pinned: boolean;
  deleted: boolean;
}

export interface MessagingInvoice {
  id: string;
  conversationId: string;
  sellerId: string;
  title: string;
  description: string;
  kind: 'oneTime' | 'subscription' | 'donation' | 'digitalGoods';
  currency: string;
  prices: Array<{ label: string; amount: { currency: string; amountMinor: number } }>;
  payload: string;
  providerId: string;
  createdAtMs: number;
  expiresAtMs?: number;
}

export interface MessagingOrder {
  id: string;
  invoiceId: string;
  buyerId: string;
  status: 'draft' | 'pending' | 'requiresAction' | 'authorized' | 'paid' | 'refunded' | 'partiallyRefunded' | 'cancelled' | 'failed';
  amount: { currency: string; amountMinor: number };
  providerPaymentId?: string;
  providerReceiptUrl?: string;
  createdAtMs: number;
  updatedAtMs: number;
}

export interface MessagingWalletAccount {
  id: string;
  ownerId: string;
  balancesMinor: Record<string, number>;
  frozen: boolean;
  createdAtMs: number;
  updatedAtMs: number;
}

export type MessagingStoryPrivacyKind = 'everyone' | 'contacts' | 'closeFriends' | 'selected';

export interface MessagingStory {
  id: string;
  ownerId: string;
  media: MessagingMediaRef;
  caption: { text: string; entities: unknown[] };
  privacy: {
    kind: MessagingStoryPrivacyKind;
    includedActorIds: string[];
    excludedActorIds: string[];
  };
  createdAtMs: number;
  expiresAtMs: number;
  editedAtMs?: number;
  pinnedToProfile: boolean;
  protectedContent: boolean;
  allowReplies: boolean;
  views: Record<string, {
    actorId: string;
    viewedAtMs: number;
    reaction?: string;
    forwarded: boolean;
  }>;
}

export type MessagingMemberStatus = 'owner' | 'administrator' | 'member' | 'restricted' | 'left' | 'banned';

export interface MessagingCommunityMember {
  actorId: string;
  status: MessagingMemberStatus;
  adminTitle?: string;
  adminRights: {
    changeInfo: boolean;
    postMessages: boolean;
    editMessages: boolean;
    deleteMessages: boolean;
    banMembers: boolean;
    inviteMembers: boolean;
    pinMessages: boolean;
    manageTopics: boolean;
    manageCalls: boolean;
    addAdmins: boolean;
    remainAnonymous: boolean;
  };
  restrictions: {
    sendMessages: boolean;
    sendMedia: boolean;
    sendPolls: boolean;
    embedLinks: boolean;
    addMembers: boolean;
    pinMessages: boolean;
    changeInfo: boolean;
    untilMs?: number;
  };
  joinedAtMs: number;
  invitedBy?: string;
}

export interface MessagingInviteLink {
  id: string;
  conversationId: string;
  creatorId: string;
  token: string;
  name?: string;
  createdAtMs: number;
  expiresAtMs?: number;
  memberLimit?: number;
  joinRequest: boolean;
  revoked: boolean;
  joinedCount: number;
}

export interface MessagingJoinRequest {
  conversationId: string;
  actorId: string;
  inviteLinkId?: string;
  bio?: string;
  requestedAtMs: number;
}

export interface MessagingForumTopic {
  id: string;
  conversationId: string;
  title: string;
  icon?: string;
  creatorId: string;
  createdAtMs: number;
  pinned: boolean;
  closed: boolean;
  hidden: boolean;
  unreadCount: number;
  lastMessageId?: string;
}

export interface MessagingCommunityState {
  conversationId: string;
  publicUsername?: string;
  linkedDiscussionId?: string;
  signaturesEnabled: boolean;
  joinToSend: boolean;
  joinRequestRequired: boolean;
  slowModeSeconds?: number;
  members: Record<string, MessagingCommunityMember>;
  inviteLinks: Record<string, MessagingInviteLink>;
  pendingJoinRequests: Record<string, MessagingJoinRequest>;
  topics: Record<string, MessagingForumTopic>;
  bannedWords: string[];
}

export interface MessagingBotProfile {
  actorId: string;
  description: string;
  about: string;
  commands: Array<{ command: string; description: string; scopes: string[] }>;
  inlineModeEnabled: boolean;
  inlinePlaceholder?: string;
  groupsAllowed: boolean;
  privacyMode: boolean;
  miniAppId?: string;
  paymentProviderIds: string[];
  businessMode: boolean;
}

export interface MessagingBotExecution {
  id: string;
  botId: string;
  invocationId: string;
  state: 'queued' | 'running' | 'waitingForApproval' | 'completed' | 'failed' | 'cancelled';
  startedAtMs?: number;
  finishedAtMs?: number;
  error?: string;
}

export interface MessagingLedgerEntry {
  id: string;
  requestId: string;
  kind: 'credit' | 'transfer' | 'refund' | 'adjustment';
  fromAccountId?: string;
  toAccountId?: string;
  amount: { currency: string; amountMinor: number };
  reference?: string;
  createdAtMs: number;
}

export interface MessagingClientEnvelope {
  protocolVersion: typeof FABUSHI_MESSAGING_PROTOCOL_VERSION;
  context: {
    requestId: string;
    deviceId: string;
    actorId: string;
    sessionId: string;
    sentAtMs: number;
  };
  command: Record<string, unknown> & { type: string };
}

export interface MessagingServerEnvelope {
  protocolVersion: typeof FABUSHI_MESSAGING_PROTOCOL_VERSION;
  cursor?: string;
  serverTimeMs: number;
  event: Record<string, unknown> & { type: string };
}

export interface MessagingHostEvent {
  type: 'messaging.event';
  timestamp: string;
  requestId: string;
  envelope: MessagingServerEnvelope;
}

export interface MessagingMediaRef {
  id: string;
  fileName?: string;
  mimeType?: string;
  sizeBytes?: number;
  width?: number;
  height?: number;
  durationMs?: number;
  thumbnailId?: string;
  localPath?: string;
  remoteUrl?: string;
  contentHash?: string;
}

export type MessagingContent =
  | { type: 'text'; data: { text: { text: string; entities: unknown[] } } }
  | {
      type: 'poll';
      data: {
        question: { text: string; entities: unknown[] };
        options: Array<{ id: string; text: string; voterCount: number; chosen: boolean; correct?: boolean }>;
        anonymous: boolean;
        multipleAnswers: boolean;
        quiz: boolean;
      };
    }
  | { type: 'photo'; data: { media: MessagingMediaRef; caption: { text: string; entities: unknown[] }; spoiler: boolean } }
  | { type: 'video'; data: { media: MessagingMediaRef; caption: { text: string; entities: unknown[] }; spoiler: boolean; streaming: boolean } }
  | { type: 'document'; data: { media: MessagingMediaRef; caption: { text: string; entities: unknown[] } } }
  | { type: 'contact'; data: { actorId?: string; displayName: string; phoneNumber?: string } }
  | { type: 'location'; data: { latitude: number; longitude: number; liveUntilMs?: number } }
  | { type: 'invoice'; data: { invoiceId: string } }
  | { type: 'miniApp'; data: { miniAppId: string; title: string; startParameter?: string } };

export function asMessagingHostEvent(event: RuntimeEvent): MessagingHostEvent | null {
  const candidate = event as unknown as MessagingHostEvent;
  if (candidate.type !== 'messaging.event') return null;
  if (candidate.envelope.event.type === 'syncBatch') {
    const payload = candidate.envelope.event as unknown as {
      actors?: unknown[];
      conversations?: unknown[];
      messages?: unknown[];
      invoices?: unknown[];
      orders?: unknown[];
      stories?: unknown[];
      communities?: unknown[];
      bots?: unknown[];
      botExecutions?: unknown[];
    };
    const counts = {
      actors: payload.actors?.length ?? 0,
      conversations: payload.conversations?.length ?? 0,
      messages: payload.messages?.length ?? 0,
      invoices: payload.invoices?.length ?? 0,
      orders: payload.orders?.length ?? 0,
      stories: payload.stories?.length ?? 0,
      communities: payload.communities?.length ?? 0,
      bots: payload.bots?.length ?? 0,
      botExecutions: payload.botExecutions?.length ?? 0,
    };
    const batchDurationMs = initialSyncRequestStartedAtMs === null ? undefined : startupNow() - initialSyncRequestStartedAtMs;
    recordStartupCriticalPathOnce('P5', 'initial-snapshot-batch', {
      counts,
      bytes: startupSerializedByteLength(candidate.envelope),
      cursorPresent: Boolean(candidate.envelope.cursor),
    }, batchDurationMs, 'asMessagingHostEvent');
    recordStartupCriticalPathOnce('P6', 'conversation-metadata-complete', {
      actorCount: counts.actors,
      conversationCount: counts.conversations,
      cursorPresent: Boolean(candidate.envelope.cursor),
    }, undefined, 'asMessagingHostEvent');
    recordStartupCriticalPathOnce('P8', 'event-stream-backlog-observed', {
      eventType: candidate.envelope.event.type,
      counts,
      bytes: startupSerializedByteLength(candidate.envelope),
      cursorPresent: Boolean(candidate.envelope.cursor),
    }, undefined, 'asMessagingHostEvent');
  }
  return candidate;
}

export function messagingText(message: MessagingMessage): string {
  if (!firstVisibleMessageScheduled && typeof window !== 'undefined') {
    firstVisibleMessageScheduled = true;
    const details = {
      messageId: message.id,
      conversationId: message.conversationId,
      contentType: message.content.type,
      bytes: startupSerializedByteLength({ id: message.id, conversationId: message.conversationId, content: message.content }),
    };
    window.requestAnimationFrame(() => {
      recordStartupCriticalPathOnce('P7', 'first-visible-message', details, undefined, 'messagingText');
    });
  }
  if (message.deleted) return '消息已删除';
  if (message.content.type === 'text') {
    const data = message.content.data as { text?: { text?: string } } | undefined;
    return data?.text?.text ?? '';
  }
  if (message.content.type === 'poll') {
    const data = message.content.data as { question?: { text?: string } } | undefined;
    return `📊 ${data?.question?.text ?? '投票'}`;
  }
  if (message.content.type === 'photo') return '🖼 图片';
  if (message.content.type === 'video') return '🎬 视频';
  if (message.content.type === 'document') {
    const data = message.content.data as { media?: { fileName?: string } } | undefined;
    return `📎 ${data?.media?.fileName ?? '文件'}`;
  }
  if (message.content.type === 'location') return '📍 位置';
  if (message.content.type === 'contact') return '👤 联系人';
  if (message.content.type === 'invoice') return '🧾 账单';
  if (message.content.type === 'miniApp') return '▣ Mini App';
  return `[${message.content.type}]`;
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = '';
  const stride = 0x8000;
  for (let offset = 0; offset < bytes.length; offset += stride) {
    binary += String.fromCharCode(...bytes.subarray(offset, Math.min(bytes.length, offset + stride)));
  }
  return btoa(binary);
}

function bridgeCommand(requestId: string, envelope: MessagingClientEnvelope): RuntimeCommand {
  return { type: 'messaging.execute', requestId, envelope } as unknown as RuntimeCommand;
}

function defaultPermissions() {
  return {
    canSendMessages: true,
    canSendMedia: true,
    canSendPolls: true,
    canAddMembers: true,
    canPinMessages: true,
    canManageTopics: true,
    canManageCalls: true,
  };
}

export class SelfHostedMessagingClientV2 {
  actorId: string;
  deviceId: string;
  sessionId: string;
  private readonly transport: MahayanaHostTransport;
  private identityResolved = false;
  private identityPromise: Promise<void> | null = null;
  private lastIdentityFailureAtMs = 0;
  private currentActorDisplayName = '当前用户';
  private currentActorUsername: string | undefined;
  private syncCallCount = 0;

  constructor(transport: MahayanaHostTransport, options: { actorId?: string; deviceId?: string; sessionId?: string } = {}) {
    instrumentStartupTransport(transport);
    this.transport = transport;
    this.actorId = options.actorId ?? 'human:desktop:current';
    this.deviceId = options.deviceId ?? 'desktop:electron';
    this.sessionId = options.sessionId ?? `messenger:${Date.now().toString(36)}`;
  }

  resetIdentity(): void {
    this.identityResolved = false;
    this.identityPromise = null;
    this.lastIdentityFailureAtMs = 0;
    this.actorId = 'human:desktop:current';
    this.sessionId = `messenger:${Date.now().toString(36)}`;
  }

  private async ensureNativeIdentity(): Promise<void> {
    if (this.identityResolved || typeof window === 'undefined' || typeof window.fabushiNative?.invoke !== 'function') return;
    if (Date.now() - this.lastIdentityFailureAtMs < 5_000) return;
    if (!this.identityPromise) {
      this.identityPromise = window.fabushiNative.invoke<{ actorId?: string; deviceId?: string; sessionId?: string }>(
        'getMessagingIdentity',
        { deviceId: this.deviceId, sessionId: this.sessionId },
      ).then((identity) => {
        const actorId = String(identity?.actorId || '').trim();
        const deviceId = String(identity?.deviceId || '').trim();
        const sessionId = String(identity?.sessionId || '').trim();
        if (!actorId || !deviceId || !sessionId) throw new Error('Fabushi messaging identity is incomplete.');
        this.actorId = actorId;
        this.deviceId = deviceId;
        this.sessionId = sessionId;
        this.identityResolved = true;
      }).catch((cause) => {
        this.lastIdentityFailureAtMs = Date.now();
        throw cause instanceof Error ? cause : new Error(String(cause));
      }).finally(() => {
        this.identityPromise = null;
      });
    }
    await this.identityPromise;
  }

  async execute(command: MessagingClientEnvelope['command']): Promise<void> {
    await this.ensureNativeIdentity();
    const requestId = `messaging-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    const envelope: MessagingClientEnvelope = {
      protocolVersion: FABUSHI_MESSAGING_PROTOCOL_VERSION,
      context: {
        requestId,
        deviceId: this.deviceId,
        actorId: this.actorId,
        sessionId: this.sessionId,
        sentAtMs: Date.now(),
      },
      command,
    };
    await this.transport.execute(bridgeCommand(requestId, envelope));
  }

  async ensureActor(actor: MessagingActor): Promise<void> {
    await this.execute({ type: 'upsertProfile', actor });
  }

  async ensureCurrentActor(displayName?: string, username?: string): Promise<void> {
    const normalizedDisplayName = displayName?.trim();
    if (normalizedDisplayName) this.currentActorDisplayName = normalizedDisplayName;
    if (username !== undefined) this.currentActorUsername = username.trim() || undefined;
    await this.ensureNativeIdentity();
    await this.ensureActor({
      id: this.actorId,
      kind: 'human',
      displayName: this.currentActorDisplayName,
      ...(this.currentActorUsername ? { username: this.currentActorUsername } : {}),
      capabilities: ['messages', 'groups', 'channels', 'calls', 'payments', 'miniApps'],
      presence: { status: 'online', lastSeenAtMs: Date.now() },
      verified: false,
    });
  }

  async sync(limit = 1000, cursor: string | null = null): Promise<void> {
    const callNumber = ++this.syncCallCount;
    const startedAtMs = startupNow();
    const phase: StartupCriticalPathPhase | null = callNumber === 1 ? 'P5' : callNumber === 2 ? 'P9' : null;
    const label = callNumber === 1 ? 'initial-snapshot-request' : 'background-reconcile-request';
    if (callNumber === 1) initialSyncRequestStartedAtMs = startedAtMs;
    if (phase) {
      recordStartupCriticalPathOnce(phase, label, {
        callNumber,
        limit,
        cursorPresent: Boolean(cursor),
        bytes: startupSerializedByteLength({ type: 'sync', cursor, limit }),
      }, undefined, 'SelfHostedMessagingClientV2.sync');
    }
    try {
      await this.execute({ type: 'sync', cursor, limit });
      if (phase) {
        recordStartupCriticalPathOnce(phase, `${label}-complete`, {
          callNumber,
          limit,
          cursorPresent: Boolean(cursor),
          outcome: 'resolved',
        }, startupNow() - startedAtMs, 'SelfHostedMessagingClientV2.sync');
      }
    } catch (cause) {
      if (phase) {
        recordStartupCriticalPathOnce(phase, `${label}-complete`, {
          callNumber,
          limit,
          cursorPresent: Boolean(cursor),
          outcome: 'rejected',
          error: cause instanceof Error ? cause.name : 'unknown',
        }, startupNow() - startedAtMs, 'SelfHostedMessagingClientV2.sync');
      }
      throw cause;
    }
  }

  startTyping(conversationId: string, action = 'typing'): Promise<void> {
    return this.execute({ type: 'startTyping', conversationId, action });
  }

  stopTyping(conversationId: string): Promise<void> {
    return this.execute({ type: 'stopTyping', conversationId });
  }

  async createConversation(kind: ConversationKind, title: string, description = '', participantActorIds: string[] = []): Promise<MessagingConversation> {
    await this.ensureCurrentActor();
    const now = Date.now();
    const id = `${kind}:${crypto.randomUUID()}`;
    const participantIds = [this.actorId, ...participantActorIds.filter((id) => id !== this.actorId)];
    const conversation: MessagingConversation = {
      id,
      kind,
      title: title.trim(),
      description: description.trim() || undefined,
      participants: participantIds.map((actorId, index) => ({
        actorId,
        role: index === 0 ? 'owner' : 'member',
        joinedAtMs: now,
      })),
      ownerId: this.actorId,
      unreadCount: 0,
      mentionCount: 0,
      pinnedMessageIds: [],
      notificationSettings: { showPreview: true, notifyMentions: true },
      permissions: defaultPermissions(),
      historyVisibility: 'allMembers',
      topics: [],
      folderIds: [],
      archived: false,
      pinned: false,
      markedUnread: false,
      createdAtMs: now,
      updatedAtMs: now,
    };
    if (!conversation.title) throw new Error('会话名称不能为空');
    await this.execute({ type: 'createConversation', conversation });
    return conversation;
  }

  createChannel(title: string, description = ''): Promise<MessagingConversation> {
    return this.createConversation('channel', title, description);
  }

  createGroup(title: string, participantActorIds: string[] = []): Promise<MessagingConversation> {
    return this.createConversation('group', title, '', participantActorIds);
  }

  async ensureSavedMessages(): Promise<MessagingConversation> {
    await this.ensureCurrentActor();
    const now = Date.now();
    const conversation: MessagingConversation = {
      id: `saved:${this.actorId}`,
      kind: 'savedMessages',
      title: '收藏消息',
      participants: [{ actorId: this.actorId, role: 'owner', joinedAtMs: now }],
      ownerId: this.actorId,
      unreadCount: 0,
      mentionCount: 0,
      pinnedMessageIds: [],
      notificationSettings: { showPreview: true, notifyMentions: false },
      permissions: defaultPermissions(),
      historyVisibility: 'allMembers',
      topics: [],
      folderIds: [],
      archived: false,
      pinned: true,
      markedUnread: false,
      createdAtMs: now,
      updatedAtMs: now,
    };
    await this.execute({ type: 'createConversation', conversation });
    return conversation;
  }

  async sendContent(
    conversationId: string,
    content: MessagingContent,
    options: {
      replyToMessageId?: string;
      threadRootMessageId?: string;
      scheduledAtMs?: number;
      silent?: boolean;
      protectedContent?: boolean;
    } = {},
  ): Promise<void> {
    await this.ensureCurrentActor();
    await this.execute({
      type: 'sendMessage',
      conversationId,
      clientMessageId: `desktop:${crypto.randomUUID()}`,
      content,
      replyToMessageId: options.replyToMessageId ?? null,
      threadRootMessageId: options.threadRootMessageId ?? null,
      scheduledAtMs: options.scheduledAtMs ?? null,
      silent: options.silent ?? false,
      protectedContent: options.protectedContent ?? false,
    });
  }

  sendText(conversationId: string, text: string, options: Parameters<SelfHostedMessagingClientV2['sendContent']>[2] = {}): Promise<void> {
    const trimmed = text.trim();
    if (!trimmed) return Promise.resolve();
    return this.sendContent(
      conversationId,
      { type: 'text', data: { text: { text: trimmed, entities: [] } } },
      options,
    );
  }

  sendPoll(conversationId: string, question: string, optionTexts: string[], anonymous = true, multipleAnswers = false): Promise<void> {
    const options = optionTexts.map((text, index) => ({
      id: `option:${index + 1}`,
      text: text.trim(),
      voterCount: 0,
      chosen: false,
    })).filter((option) => option.text);
    if (!question.trim() || options.length < 2) throw new Error('投票至少需要一个问题和两个选项');
    return this.sendContent(conversationId, {
      type: 'poll',
      data: {
        question: { text: question.trim(), entities: [] },
        options,
        anonymous,
        multipleAnswers,
        quiz: false,
      },
    });
  }

  async uploadBlob(file: File, onProgress?: (uploadedBytes: number, totalBytes: number) => void): Promise<MessagingMediaRef> {
    if (!file.size) throw new Error('不能发送空文件');
    const id = `blob-${crypto.randomUUID()}`;
    await this.execute({
      type: 'beginBlobUpload',
      metadata: {
        id,
        fileName: file.name || 'attachment',
        mimeType: file.type || 'application/octet-stream',
        sizeBytes: file.size,
        contentHash: null,
        createdAtMs: Date.now(),
      },
    });
    const chunkBytes = 512 * 1024;
    let offset = 0;
    while (offset < file.size) {
      const end = Math.min(file.size, offset + chunkBytes);
      const bytes = new Uint8Array(await file.slice(offset, end).arrayBuffer());
      await this.execute({
        type: 'appendBlobChunk',
        blobId: id,
        offset,
        dataBase64: bytesToBase64(bytes),
      });
      offset = end;
      onProgress?.(offset, file.size);
    }
    await this.execute({ type: 'finishBlobUpload', blobId: id });
    return {
      id,
      fileName: file.name || 'attachment',
      mimeType: file.type || 'application/octet-stream',
      sizeBytes: file.size,
      remoteUrl: `fabushi-blob://${id}`,
    };
  }

  async sendAttachment(
    conversationId: string,
    file: File,
    options: Parameters<SelfHostedMessagingClientV2['sendContent']>[2] = {},
    onProgress?: (uploadedBytes: number, totalBytes: number) => void,
  ): Promise<void> {
    const media = await this.uploadBlob(file, onProgress);
    const caption = { text: '', entities: [] as unknown[] };
    if (file.type.startsWith('image/')) {
      await this.sendContent(conversationId, { type: 'photo', data: { media, caption, spoiler: false } }, options);
      return;
    }
    if (file.type.startsWith('video/')) {
      await this.sendContent(conversationId, { type: 'video', data: { media, caption, spoiler: false, streaming: true } }, options);
      return;
    }
    await this.sendContent(conversationId, { type: 'document', data: { media, caption } }, options);
  }

  forwardMessage(sourceConversationId: string, messageId: string, destinationConversationId: string): Promise<void> {
    return this.execute({
      type: 'forwardMessage',
      sourceConversationId,
      messageId,
      destinationConversationId,
      clientMessageId: `desktop:${crypto.randomUUID()}`,
    });
  }

  async editText(conversationId: string, messageId: string, text: string): Promise<void> {
    await this.execute({
      type: 'editMessage',
      conversationId,
      messageId,
      content: { type: 'text', data: { text: { text, entities: [] } } },
    });
  }

  deleteMessages(conversationId: string, messageIds: string[], forEveryone = true): Promise<void> {
    return this.execute({ type: 'deleteMessages', conversationId, messageIds, forEveryone });
  }

  markRead(conversationId: string, messageId: string): Promise<void> {
    return this.execute({ type: 'markRead', conversationId, messageId });
  }

  setReaction(conversationId: string, messageId: string, reaction: string, count = 1): Promise<void> {
    return this.execute({
      type: 'setReaction',
      conversationId,
      messageId,
      reaction: {
        reaction,
        count,
        chosenByMe: count > 0,
        recentActorIds: count > 0 ? [this.actorId] : [],
      },
    });
  }

  pinMessage(conversationId: string, messageId: string, pinned: boolean): Promise<void> {
    return this.execute({ type: 'pinMessage', conversationId, messageId, pinned });
  }

  archiveConversation(conversationId: string, archived: boolean): Promise<void> {
    return this.execute({ type: 'archiveConversation', conversationId, archived });
  }

  pinConversation(conversationId: string, pinned: boolean): Promise<void> {
    return this.execute({ type: 'pinConversation', conversationId, pinned });
  }

  setConversationNotifications(conversationId: string, mutedUntilMs?: number): Promise<void> {
    return this.execute({
      type: 'setConversationNotifications',
      conversationId,
      settings: {
        mutedUntilMs: mutedUntilMs ?? null,
        sound: null,
        showPreview: true,
        notifyMentions: true,
      },
    });
  }

  publishStory(input: {
    media: MessagingMediaRef;
    caption?: string;
    privacy?: MessagingStoryPrivacyKind;
    includedActorIds?: string[];
    excludedActorIds?: string[];
    expiresAtMs?: number;
    pinnedToProfile?: boolean;
    protectedContent?: boolean;
    allowReplies?: boolean;
  }): Promise<string> {
    const now = Date.now();
    const id = `story:${crypto.randomUUID()}`;
    return this.execute({
      type: 'publishStory',
      story: {
        id,
        ownerId: this.actorId,
        media: input.media,
        caption: { text: input.caption?.trim() ?? '', entities: [] },
        privacy: {
          kind: input.privacy ?? 'everyone',
          includedActorIds: input.includedActorIds ?? [],
          excludedActorIds: input.excludedActorIds ?? [],
        },
        createdAtMs: now,
        expiresAtMs: input.expiresAtMs ?? now + 24 * 60 * 60 * 1000,
        editedAtMs: null,
        pinnedToProfile: input.pinnedToProfile ?? false,
        protectedContent: input.protectedContent ?? true,
        allowReplies: input.allowReplies ?? true,
        views: {},
      },
    }).then(() => id);
  }

  deleteStory(storyId: string): Promise<void> {
    return this.execute({ type: 'deleteStory', storyId });
  }

  viewStory(storyId: string): Promise<void> {
    return this.execute({ type: 'viewStory', storyId });
  }

  reactStory(storyId: string, reaction?: string): Promise<void> {
    return this.execute({ type: 'reactStory', storyId, reaction: reaction ?? null });
  }

  updateCommunity(community: MessagingCommunityState): Promise<void> {
    return this.execute({ type: 'updateCommunity', community });
  }

  setCommunityMember(conversationId: string, member: MessagingCommunityMember): Promise<void> {
    return this.execute({ type: 'setCommunityMember', conversationId, member });
  }

  createInviteLink(invite: MessagingInviteLink): Promise<void> {
    return this.execute({ type: 'createInviteLink', invite });
  }

  revokeInviteLink(conversationId: string, inviteId: string): Promise<void> {
    return this.execute({ type: 'revokeInviteLink', conversationId, inviteId });
  }

  respondCommunityJoin(conversationId: string, requesterId: string, approved: boolean): Promise<void> {
    return this.execute({ type: 'respondCommunityJoin', conversationId, requesterId, approved });
  }

  upsertForumTopic(topic: MessagingForumTopic): Promise<void> {
    return this.execute({ type: 'upsertForumTopic', topic });
  }

  deleteForumTopic(conversationId: string, topicId: string): Promise<void> {
    return this.execute({ type: 'deleteForumTopic', conversationId, topicId });
  }

  registerBot(profile: MessagingBotProfile): Promise<void> {
    return this.execute({ type: 'registerBot', profile });
  }

  invokeBot(input: {
    botId: string;
    conversationId: string;
    text: string;
    command?: string;
    replyToMessageId?: string;
  }): Promise<string> {
    const id = `invoke:${crypto.randomUUID()}`;
    return this.execute({
      type: 'invokeBot',
      invocation: {
        id,
        botId: input.botId,
        senderId: this.actorId,
        conversationId: input.conversationId,
        command: input.command ?? null,
        text: { text: input.text, entities: [] },
        replyToMessageId: input.replyToMessageId ?? null,
        metadata: {},
        createdAtMs: Date.now(),
      },
    }).then(() => id);
  }

  requestWalletStatus(): Promise<void> {
    return this.execute({ type: 'walletStatus' });
  }

  checkoutInvoice(invoiceId: string, orderId = `order:${crypto.randomUUID()}`): Promise<string> {
    return this.execute({
      type: 'checkoutInvoice',
      invoiceId,
      orderId,
      customer: null,
    }).then(() => orderId);
  }

  refundOrder(orderId: string): Promise<void> {
    return this.execute({
      type: 'refundOrder',
      orderId,
      requestId: `refund:${crypto.randomUUID()}`,
    });
  }

  async createInvoice(input: {
    conversationId: string;
    title: string;
    description?: string;
    currency: string;
    amountMinor: number;
    providerId: string;
  }): Promise<string> {
    await this.ensureCurrentActor();
    const id = `invoice:${crypto.randomUUID()}`;
    const currency = input.currency.toUpperCase();
    await this.execute({
      type: 'createInvoice',
      invoice: {
        id,
        conversationId: input.conversationId,
        sellerId: this.actorId,
        title: input.title,
        description: input.description ?? '',
        kind: 'oneTime',
        currency,
        prices: [{ label: input.title, amount: { currency, amountMinor: input.amountMinor } }],
        payload: id,
        providerId: input.providerId,
        requestName: false,
        requestEmail: false,
        requestPhone: false,
        requestShippingAddress: false,
        flexibleShipping: false,
        createdAtMs: Date.now(),
      },
    });
    await this.sendContent(input.conversationId, { type: 'invoice', data: { invoiceId: id } });
    return id;
  }
}
