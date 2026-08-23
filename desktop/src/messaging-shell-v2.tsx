import {
  AppWindow,
  Archive,
  BellOff,
  Bot,
  Bookmark,
  Check,
  Copy,
  Edit3,
  FileText,
  Folder,
  Forward,
  Image,
  Link2,
  MapPin,
  Menu,
  MessageCircle,
  Mic,
  MoreVertical,
  Paperclip,
  Phone,
  PhoneCall,
  Pin,
  Plus,
  Radio,
  Reply,
  Search,
  Send,
  Settings,
  ShoppingBag,
  Smile,
  SquarePen,
  Trash2,
  UserPlus,
  Users,
  Video,
  WalletCards,
  X,
} from 'lucide-react';
import React, { useEffect, useMemo, useRef, useState, type FormEvent } from 'react';
import HostClient from '../../frontend/apps/web/src/app/host/host-client';
import { BotMark, type BotMarkState } from '../../frontend/apps/web/src/app/host/bot-mark';
import type {
  BotSummary,
  ConversationSummary,
  GroupSummary,
  RuntimeEvent,
} from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { ElectronMahayanaHostTransport, isElectronMahayanaHostAvailable } from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import { MockMahayanaHostTransport } from '../../frontend/apps/web/src/lib/mahayana-host/mock-transport';
import type { InstalledPluginPointer, MahayanaHostTransport, MarketplacePluginSummary } from '../../frontend/apps/web/src/lib/mahayana-host/transport';
import {
  SelfHostedMessagingClientV2,
  asMessagingHostEvent,
  messagingText,
  type MessagingActor,
  type MessagingBotExecution,
  type MessagingBotProfile,
  type MessagingCommunityMember,
  type MessagingCommunityState,
  type MessagingConversation,
  type MessagingForumTopic,
  type MessagingInvoice,
  type MessagingLedgerEntry,
  type MessagingMediaRef,
  type MessagingMessage,
  type MessagingOrder,
  type MessagingStory,
  type MessagingWalletAccount,
} from './selfhosted-messaging-client-v2';
import styles from './messaging-shell.module.css';
import extra from './messaging-shell-v2.module.css';
import {
  FabushiWebRtcController,
  type IncomingFabushiCall,
  type WebRtcCallStatus,
} from './webrtc-call-controller';

type MessengerSection =
  | 'chats'
  | 'contacts'
  | 'bots'
  | 'groups'
  | 'channels'
  | 'calls'
  | 'saved'
  | 'archive'
  | 'folders'
  | 'miniapps'
  | 'payments'
  | 'settings';
type PeerKind = 'conversation' | 'bot' | 'group' | 'channel' | 'saved';
type PeerSource = 'legacy' | 'selfhosted';

type PeerItem = {
  key: string;
  id: string;
  source: PeerSource;
  conversationId?: string;
  actorId?: string;
  groupId?: string;
  kind: PeerKind;
  title: string;
  subtitle: string;
  unread: number;
  pinned: boolean;
  archived: boolean;
  updatedAtMs: number;
  avatar?: string;
};

type DisplayMessage = {
  id: string;
  source: PeerSource;
  role: 'me' | 'peer';
  text: string;
  createdAtMs: number;
  pinned?: boolean;
  reactions?: string[];
  invoiceId?: string;
  media?: MessagingMediaRef;
  mediaType?: 'photo' | 'video' | 'document';
};

type NewDialog =
  | { type: 'group'; name: string; selectedBotIds: Set<string> }
  | { type: 'channel'; name: string; description: string }
  | null;

type LocalCall = {
  kind: 'voice' | 'video';
  title: string;
  status: WebRtcCallStatus;
  incoming: boolean;
  muted: boolean;
  videoEnabled: boolean;
  error?: string;
};

type MessageMenu = { message: DisplayMessage; x: number; y: number } | null;
type ForwardDialogState = { sourceConversationId: string; message: DisplayMessage } | null;
type EditDialogState = { conversationId: string; messageId: string; originalText: string; text: string } | null;
type InvoiceDialogState = { conversationId: string; title: string; amount: string } | null;
type InfoTab = 'media' | 'files' | 'links';

const messengerSettingsKey = 'fabushi.desktop.messenger-settings.v2';
const messengerDraftsKey = 'fabushi.desktop.messenger-drafts.v2';
const initialPeerRenderCount = 120;
const initialMessageRenderCount = 240;

function createTransport(): MahayanaHostTransport {
  if (isElectronMahayanaHostAvailable()) return new ElectronMahayanaHostTransport();
  return new MockMahayanaHostTransport({ authenticated: true });
}

function avatarText(title: string): string {
  const value = title.trim();
  return value ? Array.from(value)[0] ?? '聊' : '聊';
}


function isAgentPeer(peer: PeerItem): boolean {
  const identity = `${peer.kind} ${peer.id} ${peer.actorId ?? ''} ${peer.title}`.toLocaleLowerCase();
  return peer.kind === 'bot' || /(agent|assistant|mahayana|codex|grok|大乘|智能体)/u.test(identity);
}

function botMarkStateForPeer(
  peer: PeerItem,
  executions: MessagingBotExecution[],
  busy: boolean,
  hostReady: boolean,
): BotMarkState {
  if (busy) return 'sending';
  const identities = [peer.id, peer.actorId].filter((value): value is string => Boolean(value));
  const execution = [...executions]
    .sort((left, right) => (right.startedAtMs ?? 0) - (left.startedAtMs ?? 0))
    .find((candidate) => identities.includes(candidate.botId));
  if (!execution) return hostReady ? 'idle' : 'waking';
  switch (execution.state) {
    case 'queued': return 'waking';
    case 'running': return 'working';
    case 'waitingForApproval': return 'alerting';
    case 'failed': return 'error';
    case 'cancelled': return 'sleeping';
    case 'completed': return 'result';
  }
}

function formatTime(timestamp: number): string {
  if (!timestamp) return '';
  const date = new Date(timestamp);
  if (date.toDateString() === new Date().toDateString()) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
  return date.toLocaleDateString([], { month: 'numeric', day: 'numeric' });
}

function legacyKind(conversation: ConversationSummary): PeerKind {
  const key = `${conversation.id} ${conversation.kind}`.toLowerCase();
  if (key.includes('saved')) return 'saved';
  if (key.includes('channel')) return 'channel';
  return 'conversation';
}

function selfKind(conversation: MessagingConversation): PeerKind {
  if (conversation.kind === 'channel') return 'channel';
  if (conversation.kind === 'group') return 'group';
  if (conversation.kind === 'savedMessages') return 'saved';
  return 'conversation';
}

function sectionTitle(section: MessengerSection): string {
  return {
    chats: '聊天',
    contacts: '联系人',
    bots: 'AI Bots',
    groups: '群组',
    channels: '频道',
    calls: '通话',
    saved: '收藏',
    archive: '已归档',
    folders: '聊天文件夹',
    miniapps: 'Mini Apps',
    payments: '支付',
    settings: '消息设置',
  }[section];
}

function matchesSection(peer: PeerItem, section: MessengerSection): boolean {
  if (section === 'chats') return !peer.archived;
  if (section === 'contacts') return peer.source === 'legacy' && peer.kind === 'conversation' && peer.id.startsWith('mahayana:contact:');
  if (section === 'bots') return peer.kind === 'bot';
  if (section === 'groups') return peer.kind === 'group';
  if (section === 'channels') return peer.kind === 'channel';
  if (section === 'saved') return peer.kind === 'saved';
  if (section === 'archive') return peer.archived;
  return false;
}

function blobMediaUrl(media?: MessagingMediaRef): string | undefined {
  if (!media?.id) return undefined;
  return `fabushi-blob://${encodeURIComponent(media.id)}`;
}

function defaultCommunityState(conversationId: string, actorId: string): MessagingCommunityState {
  const adminRights = {
    changeInfo: true,
    postMessages: true,
    editMessages: true,
    deleteMessages: true,
    banMembers: true,
    inviteMembers: true,
    pinMessages: true,
    manageTopics: true,
    manageCalls: true,
    addAdmins: true,
    remainAnonymous: false,
  };
  return {
    conversationId,
    publicUsername: undefined,
    linkedDiscussionId: undefined,
    signaturesEnabled: false,
    joinToSend: false,
    joinRequestRequired: false,
    slowModeSeconds: undefined,
    members: {
      [actorId]: {
        actorId,
        status: 'owner',
        adminTitle: 'Owner',
        adminRights,
        restrictions: {
          sendMessages: true,
          sendMedia: true,
          sendPolls: true,
          embedLinks: true,
          addMembers: true,
          pinMessages: true,
          changeInfo: true,
        },
        joinedAtMs: Date.now(),
      },
    },
    inviteLinks: {},
    pendingJoinRequests: {},
    topics: {},
    bannedWords: [],
  };
}

function upsertById<T extends { id: string }>(items: T[], item: T): T[] {
  return [...items.filter((current) => current.id !== item.id), item];
}

export default function DesktopShellV2() {
  const authTransport = useMemo(() => createTransport(), []);
  const [authenticated, setAuthenticated] = useState<boolean | null>(null);

  useEffect(() => {
    let closed = false;
    let retryTimer: number | undefined;
    const checkAuth = async () => {
      try {
        const state = await authTransport.authStatus();
        if (closed) return;
        setAuthenticated(state.loggedIn);
        if (!state.loggedIn) retryTimer = window.setTimeout(() => void checkAuth(), 900);
      } catch {
        if (closed) return;
        setAuthenticated(false);
        retryTimer = window.setTimeout(() => void checkAuth(), 1_800);
      }
    };
    void checkAuth();
    return () => {
      closed = true;
      if (retryTimer) window.clearTimeout(retryTimer);
      void authTransport.close();
    };
  }, [authTransport]);

  return (
    <div className={styles.desktopRoot} data-testid="desktop-shell">
      {authenticated === true ? <MessengerWorkspace /> : <HostClient />}
    </div>
  );
}

function MessengerWorkspace() {
  const transport = useMemo(() => createTransport(), []);
  const selfHosted = useMemo(() => new SelfHostedMessagingClientV2(transport), [transport]);
  const [hostReady, setHostReady] = useState(false);
  const [section, setSection] = useState<MessengerSection>('chats');
  const [conversations, setConversations] = useState<ConversationSummary[]>([]);
  const [bots, setBots] = useState<BotSummary[]>([]);
  const [groups, setGroups] = useState<GroupSummary[]>([]);
  const [selfActors, setSelfActors] = useState<MessagingActor[]>([]);
  const [selfConversations, setSelfConversations] = useState<MessagingConversation[]>([]);
  const [selfMessages, setSelfMessages] = useState<Record<string, MessagingMessage[]>>({});
  const [selfStories, setSelfStories] = useState<MessagingStory[]>([]);
  const [selfCommunities, setSelfCommunities] = useState<MessagingCommunityState[]>([]);
  const [selfBotProfiles, setSelfBotProfiles] = useState<MessagingBotProfile[]>([]);
  const [selfBotExecutions, setSelfBotExecutions] = useState<MessagingBotExecution[]>([]);
  const [activeStory, setActiveStory] = useState<MessagingStory | null>(null);
  const [communityDialogPeer, setCommunityDialogPeer] = useState<PeerItem | null>(null);
  const [selfInvoices, setSelfInvoices] = useState<MessagingInvoice[]>([]);
  const [selfOrders, setSelfOrders] = useState<MessagingOrder[]>([]);
  const [walletAccount, setWalletAccount] = useState<MessagingWalletAccount | null>(null);
  const [walletEntries, setWalletEntries] = useState<MessagingLedgerEntry[]>([]);
  const [activePeerKey, setActivePeerKey] = useState<string | null>(null);
  const [messages, setMessages] = useState<DisplayMessage[]>([]);
  const [composer, setComposer] = useState('');
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [replyTo, setReplyTo] = useState<DisplayMessage | null>(null);
  const [silentSend, setSilentSend] = useState(false);
  const [scheduledAtMs, setScheduledAtMs] = useState<number | undefined>();
  const [search, setSearch] = useState('');
  const [conversationSearchOpen, setConversationSearchOpen] = useState(false);
  const [conversationSearch, setConversationSearch] = useState('');
  const [peerRenderCount, setPeerRenderCount] = useState(initialPeerRenderCount);
  const [messageRenderCount, setMessageRenderCount] = useState(initialMessageRenderCount);
  const [infoOpen, setInfoOpen] = useState(true);
  const [infoTab, setInfoTab] = useState<InfoTab>('media');
  const [pendingSend, setPendingSend] = useState(false);
  const [typingByConversation, setTypingByConversation] = useState<Record<string, Record<string, number>>>({});
  const [newDialog, setNewDialog] = useState<NewDialog>(null);
  const [messageMenu, setMessageMenu] = useState<MessageMenu>(null);
  const [forwardDialog, setForwardDialog] = useState<ForwardDialogState>(null);
  const [editDialog, setEditDialog] = useState<EditDialogState>(null);
  const [invoiceDialog, setInvoiceDialog] = useState<InvoiceDialogState>(null);
  const [attachmentMenuOpen, setAttachmentMenuOpen] = useState(false);
  const [attachmentProgress, setAttachmentProgress] = useState<string | null>(null);
  const [localCall, setLocalCall] = useState<LocalCall | null>(null);
  const [incomingCall, setIncomingCall] = useState<IncomingFabushiCall | null>(null);
  const [miniApp, setMiniApp] = useState<{ id: string; title: string; html: string } | null>(null);
  const [marketplaceApps, setMarketplaceApps] = useState<MarketplacePluginSummary[]>([]);
  const [installedMiniApps, setInstalledMiniApps] = useState<Record<string, InstalledPluginPointer>>({});
  const [miniAppQuery, setMiniAppQuery] = useState('');
  const [miniAppLoading, setMiniAppLoading] = useState(false);
  const [miniAppBusy, setMiniAppBusy] = useState<Set<string>>(() => new Set());
  const [error, setError] = useState<string | null>(null);
  const [mutedPeerKeys, setMutedPeerKeys] = useState<Set<string>>(() => new Set());
  const [pinnedPeerKeys, setPinnedPeerKeys] = useState<Set<string>>(() => new Set());
  const [archivedPeerKeys, setArchivedPeerKeys] = useState<Set<string>>(() => new Set());
  const activePeerKeyRef = useRef<string | null>(null);
  const messagingCursorRef = useRef<string | null>(null);
  const syncInFlightRef = useRef(false);
  const typingStopTimerRef = useRef<number | null>(null);
  const peersRef = useRef<PeerItem[]>([]);
  const webRtcRef = useRef<FabushiWebRtcController | null>(null);
  const localVideoRef = useRef<HTMLVideoElement>(null);
  const remoteVideoRef = useRef<HTMLVideoElement>(null);
  const remoteAudioRef = useRef<HTMLAudioElement>(null);
  const mediaInputRef = useRef<HTMLInputElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const storyInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    activePeerKeyRef.current = activePeerKey;
  }, [activePeerKey]);

  useEffect(() => {
    try {
      const stored = JSON.parse(window.localStorage.getItem(messengerDraftsKey) || '{}') as Record<string, unknown>;
      setDrafts(Object.fromEntries(Object.entries(stored).filter((entry): entry is [string, string] => typeof entry[1] === 'string')));
    } catch {
      setDrafts({});
    }
  }, []);

  useEffect(() => {
    try {
      window.localStorage.setItem(messengerDraftsKey, JSON.stringify(drafts));
    } catch {
      // Draft persistence is best-effort and must never block messaging.
    }
  }, [drafts]);

  useEffect(() => {
    if (!activePeerKey) return;
    setComposer(drafts[activePeerKey] ?? '');
    setConversationSearch('');
    setMessageRenderCount(initialMessageRenderCount);
  }, [activePeerKey]);

  useEffect(() => {
    try {
      const stored = JSON.parse(window.localStorage.getItem(messengerSettingsKey) || '{}') as {
        muted?: string[];
        pinned?: string[];
        archived?: string[];
      };
      setMutedPeerKeys(new Set(stored.muted ?? []));
      setPinnedPeerKeys(new Set(stored.pinned ?? []));
      setArchivedPeerKeys(new Set(stored.archived ?? []));
    } catch {
      // Ignore malformed old settings.
    }
  }, []);

  useEffect(() => {
    try {
      window.localStorage.setItem(messengerSettingsKey, JSON.stringify({
        muted: [...mutedPeerKeys],
        pinned: [...pinnedPeerKeys],
        archived: [...archivedPeerKeys],
      }));
    } catch {
      // Persistence is a convenience only.
    }
  }, [mutedPeerKeys, pinnedPeerKeys, archivedPeerKeys]);

  useEffect(() => {
    const controller = new FabushiWebRtcController(
      () => ({ deviceId: selfHosted.deviceId, sessionId: selfHosted.sessionId }),
      {
        onIdentity(identity) {
          selfHosted.actorId = identity.actorId;
          selfHosted.deviceId = identity.deviceId;
          selfHosted.sessionId = identity.sessionId;
        },
        onIncoming(call) {
          const peer = peersRef.current.find((candidate) =>
            candidate.conversationId === call.conversationId || candidate.actorId === call.fromActorId);
          setIncomingCall(call);
          setLocalCall({
            kind: call.kind,
            title: peer?.title ?? call.fromActorId,
            status: 'ringing',
            incoming: true,
            muted: false,
            videoEnabled: call.kind === 'video',
          });
        },
        onStatus(status, detail) {
          setLocalCall((current) => {
            if (!current) return current;
            if (status === 'ended') return null;
            return {
              ...current,
              status,
              error: status === 'failed' ? detail ?? 'WebRTC 连接失败' : current.error,
            };
          });
          if (status === 'connecting' || status === 'active') syncLocalCallMedia();
        },
        onRemoteStream(stream) {
          window.setTimeout(() => {
            const video = remoteVideoRef.current;
            const audio = remoteAudioRef.current;
            if (video) {
              video.srcObject = stream;
              if (stream) void video.play().catch(() => {});
            }
            if (audio) {
              audio.srcObject = stream;
              if (stream) void audio.play().catch(() => {});
            }
          }, 0);
        },
      },
    );
    webRtcRef.current = controller;
    return () => {
      if (webRtcRef.current === controller) webRtcRef.current = null;
      void controller.dispose();
    };
  }, [selfHosted]);

  useEffect(() => {
    let closed = false;
    const unsubscribe = transport.subscribe((event) => {
      if (!closed) handleRuntimeEvent(event);
    });
    void transport.initialize({ profileId: 'desktop-messenger-v2', mode: 'production' })
      .then(async () => {
        if (closed) return;
        setHostReady(true);
        refreshLegacy();
        try {
          await selfHosted.ensureCurrentActor();
          await selfHosted.sync();
          void webRtcRef.current?.connect().catch(() => {});
        } catch (cause) {
          setError(cause instanceof Error ? cause.message : String(cause));
        }
      })
      .catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
    return () => {
      closed = true;
      unsubscribe();
      void transport.close();
    };
  }, [transport, selfHosted]);

  useEffect(() => {
    if (!hostReady || section !== 'miniapps') return;
    const timer = window.setTimeout(() => {
      void refreshMiniApps(miniAppQuery);
    }, 250);
    return () => window.clearTimeout(timer);
  }, [hostReady, section, miniAppQuery]);

  useEffect(() => {
    if (!hostReady) return;
    const timer = window.setInterval(() => {
      const now = Date.now();
      setTypingByConversation((current) => {
        const next: Record<string, Record<string, number>> = {};
        for (const [conversationId, actors] of Object.entries(current)) {
          const active = Object.fromEntries(Object.entries(actors).filter(([, expiresAt]) => expiresAt > now));
          if (Object.keys(active).length) next[conversationId] = active;
        }
        return next;
      });
      if (syncInFlightRef.current) return;
      syncInFlightRef.current = true;
      void selfHosted.sync(1000, messagingCursorRef.current)
        .catch(() => {})
        .finally(() => { syncInFlightRef.current = false; });
    }, 2_000);
    return () => window.clearInterval(timer);
  }, [hostReady, selfHosted]);

  function nextRequestId(prefix: string): string {
    return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  }

  function execute(command: Parameters<MahayanaHostTransport['execute']>[0]) {
    return transport.execute(command).catch((cause: unknown) => {
      setError(cause instanceof Error ? cause.message : String(cause));
    });
  }

  function refreshLegacy() {
    void Promise.all([
      execute({ type: 'conversation.list', requestId: nextRequestId('conversation-list') }),
      execute({ type: 'bot.list', requestId: nextRequestId('bot-list') }),
      execute({ type: 'group.list', requestId: nextRequestId('group-list') }),
    ]).catch(() => {});
  }

  function displaySelfMessage(message: MessagingMessage): DisplayMessage {
    return {
      id: message.id,
      source: 'selfhosted',
      role: message.senderId === selfHosted.actorId ? 'me' : 'peer',
      text: messagingText(message),
      createdAtMs: message.createdAtMs,
      pinned: message.pinned,
      reactions: message.reactions.map((reaction) => `${reaction.reaction} ${reaction.count}`),
      invoiceId: message.content.type === 'invoice'
        ? (message.content.data as { invoiceId?: string } | undefined)?.invoiceId
        : undefined,
      media: ['photo', 'video', 'document'].includes(message.content.type)
        ? (message.content.data as { media?: MessagingMediaRef } | undefined)?.media
        : undefined,
      mediaType: ['photo', 'video', 'document'].includes(message.content.type)
        ? message.content.type as 'photo' | 'video' | 'document'
        : undefined,
    };
  }

  function showSelfConversation(conversationId: string) {
    setMessages((selfMessages[conversationId] ?? []).filter((message) => !message.deleted).map(displaySelfMessage));
  }

  function handleSelfHostedEvent(runtimeEvent: RuntimeEvent): boolean {
    const hostEvent = asMessagingHostEvent(runtimeEvent);
    if (!hostEvent) return false;
    if (hostEvent.envelope.cursor) messagingCursorRef.current = hostEvent.envelope.cursor;
    const event = hostEvent.envelope.event;
    switch (event.type) {
      case 'syncBatch': {
        const payload = event as unknown as {
          actors?: MessagingActor[];
          conversations?: MessagingConversation[];
          messages?: MessagingMessage[];
          invoices?: MessagingInvoice[];
          orders?: MessagingOrder[];
          stories?: MessagingStory[];
          communities?: MessagingCommunityState[];
          bots?: MessagingBotProfile[];
          botExecutions?: MessagingBotExecution[];
        };
        const nextConversations = payload.conversations ?? [];
        const grouped: Record<string, MessagingMessage[]> = {};
        for (const message of payload.messages ?? []) {
          (grouped[message.conversationId] ??= []).push(message);
        }
        for (const list of Object.values(grouped)) list.sort((a, b) => a.createdAtMs - b.createdAtMs);
        setSelfActors(payload.actors ?? []);
        setSelfConversations(nextConversations);
        setSelfMessages(grouped);
        setSelfInvoices(payload.invoices ?? []);
        setSelfOrders(payload.orders ?? []);
        setSelfStories(payload.stories ?? []);
        setSelfCommunities(payload.communities ?? []);
        setSelfBotProfiles(payload.bots ?? []);
        setSelfBotExecutions(payload.botExecutions ?? []);
        const active = activePeerKeyRef.current;
        if (active?.startsWith('selfhosted:')) {
          const conversationId = active.slice('selfhosted:'.length);
          setMessages((grouped[conversationId] ?? []).filter((message) => !message.deleted).map(displaySelfMessage));
        }
        break;
      }
      case 'conversationChanged': {
        const conversation = (event as unknown as { conversation: MessagingConversation }).conversation;
        setSelfConversations((current) => {
          const existing = current.find((item) => item.id === conversation.id);
          return upsertById(current, existing
            ? {
                ...conversation,
                lastReadMessageId: existing.lastReadMessageId,
                unreadCount: existing.unreadCount,
                markedUnread: existing.markedUnread,
              }
            : conversation);
        });
        break;
      }
      case 'messageAdded':
      case 'messageChanged': {
        const message = (event as unknown as { message: MessagingMessage }).message;
        const isActiveConversation = activePeerKeyRef.current === `selfhosted:${message.conversationId}`;
        const isIncoming = message.senderId !== selfHosted.actorId;
        setSelfMessages((current) => {
          const list = upsertById(current[message.conversationId] ?? [], message)
            .sort((a, b) => a.createdAtMs - b.createdAtMs);
          const next = { ...current, [message.conversationId]: list };
          if (isActiveConversation) {
            setMessages(list.filter((item) => !item.deleted).map(displaySelfMessage));
          }
          return next;
        });
        if (event.type === 'messageAdded') {
          setSelfConversations((current) => current.map((conversation) => conversation.id === message.conversationId
            ? {
                ...conversation,
                lastMessageId: message.id,
                updatedAtMs: Math.max(conversation.updatedAtMs, message.createdAtMs),
                unreadCount: isIncoming && !isActiveConversation
                  ? conversation.unreadCount + 1
                  : conversation.unreadCount,
              }
            : conversation));
          if (isIncoming && isActiveConversation) {
            void selfHosted.markRead(message.conversationId, message.id).catch(() => {});
          }
        }
        setPendingSend(false);
        break;
      }
      case 'readChanged': {
        const payload = event as unknown as { conversationId: string; actorId: string; messageId: string };
        if (payload.actorId === selfHosted.actorId) {
          setSelfConversations((current) => current.map((conversation) => conversation.id === payload.conversationId
            ? {
                ...conversation,
                lastReadMessageId: payload.messageId,
                unreadCount: 0,
                markedUnread: false,
              }
            : conversation));
        }
        break;
      }
      case 'messagesDeleted': {
        const payload = event as unknown as { conversationId: string; messageIds: string[] };
        setSelfMessages((current) => {
          const list = (current[payload.conversationId] ?? []).filter((message) => !payload.messageIds.includes(message.id));
          if (activePeerKeyRef.current === `selfhosted:${payload.conversationId}`) {
            setMessages(list.map(displaySelfMessage));
          }
          return { ...current, [payload.conversationId]: list };
        });
        break;
      }
      case 'actorChanged': {
        const actor = (event as unknown as { actor: MessagingActor }).actor;
        setSelfActors((current) => upsertById(current, actor));
        break;
      }
      case 'typingChanged': {
        const payload = event as unknown as { conversationId: string; actorId: string; action?: string | null; expiresAtMs?: number | null };
        if (payload.actorId === selfHosted.actorId) break;
        setTypingByConversation((current) => {
          const actors = { ...(current[payload.conversationId] ?? {}) };
          if (payload.action && (payload.expiresAtMs ?? 0) > Date.now()) actors[payload.actorId] = payload.expiresAtMs!;
          else delete actors[payload.actorId];
          const next = { ...current };
          if (Object.keys(actors).length) next[payload.conversationId] = actors;
          else delete next[payload.conversationId];
          return next;
        });
        break;
      }
      case 'storyChanged': {
        const story = (event as unknown as { story: MessagingStory }).story;
        setSelfStories((current) => upsertById(current, story));
        setActiveStory((current) => current?.id === story.id ? story : current);
        break;
      }
      case 'storyDeleted': {
        const storyId = String((event as unknown as { storyId: string }).storyId);
        setSelfStories((current) => current.filter((story) => story.id !== storyId));
        setActiveStory((current) => current?.id === storyId ? null : current);
        break;
      }
      case 'communityChanged': {
        const community = (event as unknown as { community: MessagingCommunityState }).community;
        setSelfCommunities((current) => [...current.filter((item) => item.conversationId !== community.conversationId), community]);
        break;
      }
      case 'botChanged': {
        const payload = event as unknown as { profile?: MessagingBotProfile | null; execution?: MessagingBotExecution | null };
        if (payload.profile) setSelfBotProfiles((current) => [...current.filter((item) => item.actorId !== payload.profile!.actorId), payload.profile!]);
        if (payload.execution) setSelfBotExecutions((current) => upsertById(current, payload.execution!));
        break;
      }
      case 'invoiceChanged': {
        const invoice = (event as unknown as { invoice: MessagingInvoice }).invoice;
        setSelfInvoices((current) => upsertById(current, invoice));
        break;
      }
      case 'orderChanged': {
        const order = (event as unknown as { order: MessagingOrder }).order;
        setSelfOrders((current) => upsertById(current, order));
        void selfHosted.requestWalletStatus().catch(() => {});
        break;
      }
      case 'walletStatus': {
        const payload = event as unknown as { account?: MessagingWalletAccount | null; recentEntries?: MessagingLedgerEntry[] };
        setWalletAccount(payload.account ?? null);
        setWalletEntries(payload.recentEntries ?? []);
        break;
      }
      default:
        break;
    }
    return true;
  }

  function handleRuntimeEvent(event: RuntimeEvent) {
    if (handleSelfHostedEvent(event)) return;
    switch (event.type) {
      case 'host.ready':
        setHostReady(true);
        break;
      case 'conversation.listed':
        setConversations(event.conversations);
        if (!activePeerKeyRef.current && event.conversations[0]) {
          const conversation = event.conversations[0];
          setActivePeerKey(`legacy:conversation:${conversation.id}`);
          void execute({ type: 'conversation.open', requestId: nextRequestId('conversation-open'), conversationId: conversation.id });
        }
        break;
      case 'conversation.opened':
        if (activePeerKeyRef.current === `legacy:conversation:${event.conversationId}`) {
          setMessages(event.messages.map((message) => ({
            id: message.id,
            source: 'legacy',
            role: message.role === 'user' ? 'me' : 'peer',
            text: message.text,
            createdAtMs: message.createdAtMs,
          })));
        }
        break;
      case 'bot.listed':
        setBots(event.bots);
        break;
      case 'bot.changed':
        setBots((current) => event.action === 'deleted'
          ? current.filter((bot) => bot.id !== event.bot.id)
          : upsertById(current, event.bot));
        break;
      case 'group.listed':
        setGroups(event.groups);
        break;
      case 'group.changed':
        setGroups((current) => event.action === 'deleted'
          ? current.filter((group) => group.id !== event.group.id)
          : upsertById(current, event.group));
        if (activePeerKeyRef.current === `legacy:group:${event.group.id}`) {
          setMessages(event.group.messages.map((message) => ({
            id: message.id,
            source: 'legacy',
            role: message.speaker.kind === 'user' ? 'me' : 'peer',
            text: message.content,
            createdAtMs: message.createdAtMs,
          })));
          setPendingSend(false);
        }
        break;
      case 'chat.message':
        setPendingSend(false);
        setMessages((current) => {
          const id = event.operationId || nextRequestId('message');
          if (current.some((message) => message.id === id && message.text === event.text)) return current;
          return [...current, {
            id,
            source: 'legacy',
            role: event.role === 'user' ? 'me' : 'peer',
            text: event.text,
            createdAtMs: Date.now(),
          }];
        });
        break;
      case 'chat.delta':
        setPendingSend(false);
        setMessages((current) => {
          const index = current.findIndex((message) => message.id === event.operationId);
          if (index < 0) return [...current, { id: event.operationId, source: 'legacy', role: 'peer', text: event.delta, createdAtMs: Date.now() }];
          return current.map((message, messageIndex) => messageIndex === index ? { ...message, text: `${message.text}${event.delta}` } : message);
        });
        break;
      case 'miniapp.opened':
        setMiniApp({ id: event.miniAppId, html: event.html });
        break;
      case 'operation.failed':
        setPendingSend(false);
        setError(event.message);
        break;
      case 'host.closed':
        setHostReady(false);
        break;
      default:
        break;
    }
  }

  useEffect(() => {
    if (section !== 'payments') return;
    void selfHosted.requestWalletStatus().catch((cause: unknown) => {
      setError(cause instanceof Error ? cause.message : String(cause));
    });
  }, [section, selfHosted]);

  const visibleStories = useMemo(() => selfStories
    .filter((story) => story.pinnedToProfile || story.expiresAtMs > Date.now())
    .sort((left, right) => right.createdAtMs - left.createdAtMs), [selfStories]);

  const peers = useMemo<PeerItem[]>(() => {
    const legacyConversations = conversations.map((conversation): PeerItem => ({
      key: `legacy:conversation:${conversation.id}`,
      id: conversation.id,
      source: 'legacy',
      conversationId: conversation.id,
      kind: legacyKind(conversation),
      title: conversation.title,
      subtitle: conversation.kind,
      unread: conversation.unreadCount,
      pinned: conversation.pinned || pinnedPeerKeys.has(`legacy:conversation:${conversation.id}`),
      archived: archivedPeerKeys.has(`legacy:conversation:${conversation.id}`),
      updatedAtMs: conversation.updatedAtMs,
    }));
    const conversationIds = new Set(conversations.map((conversation) => conversation.id));
    const botPeers = bots
      .filter((bot) => !bot.conversationId || !conversationIds.has(bot.conversationId))
      .map((bot): PeerItem => ({
        key: `legacy:bot:${bot.id}`,
        id: bot.id,
        source: 'legacy',
        actorId: bot.id,
        conversationId: bot.conversationId,
        kind: 'bot',
        title: bot.name,
        subtitle: bot.description || bot.title || 'AI Bot',
        unread: bot.unread ? 1 : 0,
        pinned: pinnedPeerKeys.has(`legacy:bot:${bot.id}`),
        archived: archivedPeerKeys.has(`legacy:bot:${bot.id}`),
        updatedAtMs: 0,
        avatar: bot.avatar,
      }));
    const legacyGroups = groups.map((group): PeerItem => ({
      key: `legacy:group:${group.id}`,
      id: group.id,
      source: 'legacy',
      groupId: group.id,
      kind: 'group',
      title: group.name,
      subtitle: `${group.memberIds.length} 个 AI / 成员`,
      unread: 0,
      pinned: pinnedPeerKeys.has(`legacy:group:${group.id}`),
      archived: archivedPeerKeys.has(`legacy:group:${group.id}`),
      updatedAtMs: group.updatedAtMs,
    }));
    const nativePeers = selfConversations.map((conversation): PeerItem => ({
      key: `selfhosted:${conversation.id}`,
      id: conversation.id,
      source: 'selfhosted',
      conversationId: conversation.id,
      actorId: conversation.participants.length === 2
        ? conversation.participants.find((participant) => participant.actorId !== selfHosted.actorId)?.actorId
        : undefined,
      kind: selfKind(conversation),
      title: conversation.title,
      subtitle: conversation.description || ({ channel: '频道', group: '群组', saved: '收藏消息', conversation: '私聊', bot: 'Bot' } as const)[selfKind(conversation)],
      unread: conversation.unreadCount,
      pinned: conversation.pinned,
      archived: conversation.archived,
      updatedAtMs: conversation.updatedAtMs,
      avatar: conversation.avatarUrl,
    }));
    return [...legacyConversations, ...botPeers, ...legacyGroups, ...nativePeers].sort((left, right) => {
      if (left.pinned !== right.pinned) return left.pinned ? -1 : 1;
      return right.updatedAtMs - left.updatedAtMs;
    });
  }, [conversations, bots, groups, selfConversations, pinnedPeerKeys, archivedPeerKeys]);

  peersRef.current = peers;
  const activePeer = peers.find((peer) => peer.key === activePeerKey) ?? null;
  const activeTypingActors = activePeer?.source === 'selfhosted' && activePeer.conversationId
    ? Object.keys(typingByConversation[activePeer.conversationId] ?? {})
    : [];
  const visiblePeers = peers.filter((peer) => {
    if (['chats', 'contacts', 'bots', 'groups', 'channels', 'saved', 'archive'].includes(section) && !matchesSection(peer, section)) return false;
    const query = search.trim().toLowerCase();
    return !query || `${peer.title} ${peer.subtitle}`.toLowerCase().includes(query);
  });
  const sectionIsPeerList = ['chats', 'contacts', 'bots', 'groups', 'channels', 'saved', 'archive'].includes(section);
  const renderedPeers = visiblePeers.slice(0, peerRenderCount);
  const conversationQuery = conversationSearch.trim().toLowerCase();
  const matchingMessages = conversationQuery
    ? messages.filter((message) => message.text.toLowerCase().includes(conversationQuery))
    : messages;
  const renderedMessages = matchingMessages.slice(Math.max(0, matchingMessages.length - messageRenderCount));

  useEffect(() => {
    setPeerRenderCount(initialPeerRenderCount);
  }, [section, search]);

  function updateComposer(value: string) {
    setComposer(value);
    const activeKey = activePeerKeyRef.current;
    if (!activeKey) return;
    if (activeKey.startsWith('selfhosted:')) {
      const conversationId = activeKey.slice('selfhosted:'.length);
      if (typingStopTimerRef.current) window.clearTimeout(typingStopTimerRef.current);
      if (value.trim()) {
        void selfHosted.startTyping(conversationId).catch(() => {});
        typingStopTimerRef.current = window.setTimeout(() => {
          void selfHosted.stopTyping(conversationId).catch(() => {});
        }, 3_000);
      } else {
        void selfHosted.stopTyping(conversationId).catch(() => {});
      }
    }
    setDrafts((current) => {
      const next = { ...current };
      if (value) next[activePeerKeyRef.current!] = value;
      else delete next[activePeerKeyRef.current!];
      return next;
    });
  }

  async function openPeer(peer: PeerItem) {
    setActivePeerKey(peer.key);
    setComposer(drafts[peer.key] ?? '');
    setConversationSearch('');
    setMessageRenderCount(initialMessageRenderCount);
    setReplyTo(null);
    setError(null);
    setConversationSearchOpen(false);
    if (peer.source === 'selfhosted' && peer.conversationId) {
      showSelfConversation(peer.conversationId);
      const list = selfMessages[peer.conversationId] ?? [];
      const last = list.filter((message) => !message.deleted).at(-1);
      if (last) void selfHosted.markRead(peer.conversationId, last.id).catch(() => {});
      return;
    }
    if (peer.kind === 'group' && peer.groupId) {
      const group = groups.find((item) => item.id === peer.groupId);
      setMessages(group?.messages.map((message) => ({
        id: message.id,
        source: 'legacy',
        role: message.speaker.kind === 'user' ? 'me' : 'peer',
        text: message.content,
        createdAtMs: message.createdAtMs,
      })) ?? []);
      return;
    }
    if (peer.conversationId) {
      setMessages([]);
      await execute({ type: 'conversation.open', requestId: nextRequestId('conversation-open'), conversationId: peer.conversationId });
      return;
    }
    setMessages([]);
  }

  async function sendMessage(event: FormEvent) {
    event.preventDefault();
    const text = composer.trim();
    if (!text || !activePeer || pendingSend) return;
    setPendingSend(true);
    updateComposer('');
    try {
      if (activePeer.source === 'selfhosted' && activePeer.conversationId) {
        await selfHosted.sendText(activePeer.conversationId, text, {
          replyToMessageId: replyTo?.id,
          scheduledAtMs,
          silent: silentSend,
        });
      } else if (activePeer.kind === 'group' && activePeer.groupId) {
        await execute({ type: 'group.send', requestId: nextRequestId('group-send'), id: activePeer.groupId, text });
      } else {
        await execute({
          type: 'chat.send',
          requestId: nextRequestId('chat-send'),
          text,
          conversationId: activePeer.conversationId,
          agentId: activePeer.actorId,
        });
      }
      setReplyTo(null);
      setScheduledAtMs(undefined);
    } catch (cause) {
      updateComposer(text);
      setPendingSend(false);
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function saveNewDialog() {
    if (!newDialog || !newDialog.name.trim()) return;
    try {
      if (newDialog.type === 'channel') {
        const conversation = await selfHosted.createChannel(newDialog.name, newDialog.description);
        setSelfConversations((current) => upsertById(current, conversation));
        setNewDialog(null);
        setSection('channels');
        await openPeer({
          key: `selfhosted:${conversation.id}`,
          id: conversation.id,
          source: 'selfhosted',
          conversationId: conversation.id,
          kind: 'channel',
          title: conversation.title,
          subtitle: conversation.description || '频道',
          unread: 0,
          pinned: false,
          archived: false,
          updatedAtMs: conversation.updatedAtMs,
        });
        return;
      }
      await execute({
        type: 'group.create',
        requestId: nextRequestId('group-create'),
        name: newDialog.name.trim(),
        description: '',
        memberIds: [...newDialog.selectedBotIds],
      });
      await execute({ type: 'group.list', requestId: nextRequestId('group-list-after-create') });
      setNewDialog(null);
      setSection('groups');
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function ensureSavedMessages() {
    try {
      const existing = selfConversations.find((conversation) => conversation.kind === 'savedMessages');
      const conversation = existing ?? await selfHosted.ensureSavedMessages();
      if (!existing) setSelfConversations((current) => upsertById(current, conversation));
      setSection('saved');
      await openPeer({
        key: `selfhosted:${conversation.id}`,
        id: conversation.id,
        source: 'selfhosted',
        conversationId: conversation.id,
        kind: 'saved',
        title: conversation.title,
        subtitle: '收藏消息',
        unread: conversation.unreadCount,
        pinned: conversation.pinned,
        archived: conversation.archived,
        updatedAtMs: conversation.updatedAtMs,
      });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function publishStoryFile(file: File) {
    setAttachmentProgress(`正在上传动态 ${file.name}…`);
    try {
      const media = await selfHosted.uploadBlob(file, (uploaded, total) => {
        setAttachmentProgress(`正在上传动态 ${file.name} · ${Math.round((uploaded / total) * 100)}%`);
      });
      const caption = window.prompt('动态说明（可选）', '') ?? '';
      const privacyInput = (window.prompt('可见范围：everyone / contacts / closeFriends / selected', 'everyone') ?? 'everyone').trim();
      const privacy = ['everyone', 'contacts', 'closeFriends', 'selected'].includes(privacyInput)
        ? privacyInput as 'everyone' | 'contacts' | 'closeFriends' | 'selected'
        : 'everyone';
      let includedActorIds: string[] = [];
      if (privacy === 'selected') {
        const selected = window.prompt('输入允许查看的 Actor ID，用逗号分隔', '') ?? '';
        includedActorIds = selected.split(',').map((value) => value.trim()).filter(Boolean);
        if (!includedActorIds.length) throw new Error('Selected 动态至少需要一名可见联系人。');
      }
      await selfHosted.publishStory({
        media,
        caption,
        privacy,
        includedActorIds,
        protectedContent: true,
        allowReplies: true,
      });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setAttachmentProgress(null);
    }
  }

  async function openStory(story: MessagingStory) {
    setActiveStory(story);
    if (story.ownerId !== selfHosted.actorId) {
      void selfHosted.viewStory(story.id).catch(() => {});
    }
  }

  async function reactToActiveStory(reaction: string) {
    if (!activeStory) return;
    try {
      await selfHosted.reactStory(activeStory.id, reaction);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function sendAttachmentFile(file: File) {
    if (!activePeer?.conversationId || activePeer.source !== 'selfhosted') {
      setError('附件需要发送到 Fabushi 自建会话。');
      return;
    }
    setAttachmentMenuOpen(false);
    setPendingSend(true);
    setAttachmentProgress(`正在上传 ${file.name}…`);
    try {
      await selfHosted.sendAttachment(
        activePeer.conversationId,
        file,
        { replyToMessageId: replyTo?.id, scheduledAtMs, silent: silentSend },
        (uploaded, total) => setAttachmentProgress(`正在上传 ${file.name} · ${Math.round((uploaded / total) * 100)}%`),
      );
      setReplyTo(null);
      setScheduledAtMs(undefined);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setPendingSend(false);
      setAttachmentProgress(null);
    }
  }

  async function sendPoll() {
    if (!activePeer?.conversationId || activePeer.source !== 'selfhosted') {
      setError('投票使用 Fabushi 自研消息协议；请先打开自建群组或频道。');
      return;
    }
    const question = window.prompt('投票问题');
    if (!question) return;
    const raw = window.prompt('选项（用 | 分隔，至少两个）', '选项一 | 选项二');
    if (!raw) return;
    try {
      await selfHosted.sendPoll(activePeer.conversationId, question, raw.split('|'));
      setAttachmentMenuOpen(false);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function sendLocation() {
    if (!activePeer?.conversationId || activePeer.source !== 'selfhosted') {
      setError('位置消息需要自建会话。');
      return;
    }
    if (!navigator.geolocation) {
      setError('当前平台没有提供定位能力。');
      return;
    }
    navigator.geolocation.getCurrentPosition(
      (position) => void selfHosted.sendContent(activePeer.conversationId!, {
        type: 'location',
        data: { latitude: position.coords.latitude, longitude: position.coords.longitude },
      }).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause))),
      (positionError) => setError(positionError.message),
    );
    setAttachmentMenuOpen(false);
  }

  function createInvoiceForActivePeer() {
  if (!activePeer?.conversationId || activePeer.source !== 'selfhosted') {
    setError('账单需要发送到 Fabushi 自建会话。');
    return;
  }
  setInvoiceDialog({ conversationId: activePeer.conversationId, title: '订单', amount: '9.99' });
}

async function saveInvoiceDialog() {
  if (!invoiceDialog) return;
  const title = invoiceDialog.title.trim();
  const amount = Number(invoiceDialog.amount);
  if (!title) {
    setError('账单名称不能为空。');
    return;
  }
  if (!Number.isFinite(amount) || amount <= 0) {
    setError('金额必须大于 0。');
    return;
  }
  try {
    await selfHosted.createInvoice({
      conversationId: invoiceDialog.conversationId,
      title,
      currency: 'USD',
      amountMinor: Math.round(amount * 100),
      providerId: 'fabushi-pay',
    });
    setInvoiceDialog(null);
  } catch (cause) {
    setError(cause instanceof Error ? cause.message : String(cause));
  }
}

  async function handleMessageAction(action: 'copy' | 'reply' | 'forward' | 'checkout' | 'edit' | 'delete' | 'react' | 'pin') {
    const target = messageMenu?.message;
    if (!target || !activePeer) return;
    setMessageMenu(null);
    if (action === 'copy') {
      await navigator.clipboard.writeText(target.text);
      return;
    }
    if (action === 'reply') {
      setReplyTo(target);
      return;
    }
    if (action === 'forward') {
      if (activePeer.source !== 'selfhosted' || !activePeer.conversationId || target.source !== 'selfhosted') {
        setError('转发需要源消息已迁移到 Fabushi 自建协议。');
        return;
      }
      setForwardDialog({ sourceConversationId: activePeer.conversationId, message: target });
      return;
    }
    if (action === 'checkout') {
      if (target.source !== 'selfhosted' || !target.invoiceId) {
        setError('该账单尚未迁移到 Fabushi Pay。');
        return;
      }
      const invoice = selfInvoices.find((item) => item.id === target.invoiceId);
      if (invoice?.sellerId === selfHosted.actorId) {
        setError('不能支付自己创建的账单。');
        return;
      }
      try {
        await selfHosted.checkoutInvoice(target.invoiceId);
        await selfHosted.requestWalletStatus();
        setSection('payments');
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
      return;
    }
    if (activePeer.source !== 'selfhosted' || !activePeer.conversationId || target.source !== 'selfhosted') {
      setError('该消息来自旧会话适配器；修改类操作正在统一迁移到 Fabushi 自建协议。');
      return;
    }
    try {
      if (action === 'delete') await selfHosted.deleteMessages(activePeer.conversationId, [target.id]);
      if (action === 'react') await selfHosted.setReaction(activePeer.conversationId, target.id, '👍');
      if (action === 'pin') await selfHosted.pinMessage(activePeer.conversationId, target.id, !target.pinned);
      if (action === 'edit') {
        setEditDialog({
          conversationId: activePeer.conversationId,
          messageId: target.id,
          originalText: target.text,
          text: target.text,
        });
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function saveEditedMessage() {
    if (!editDialog) return;
    const text = editDialog.text.trim();
    if (!text || text === editDialog.originalText) {
      setEditDialog(null);
      return;
    }
    try {
      await selfHosted.editText(editDialog.conversationId, editDialog.messageId, text);
      setEditDialog(null);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function forwardToPeer(peer: PeerItem) {
    if (!forwardDialog || peer.source !== 'selfhosted' || !peer.conversationId) return;
    try {
      await selfHosted.forwardMessage(
        forwardDialog.sourceConversationId,
        forwardDialog.message.id,
        peer.conversationId,
      );
      setForwardDialog(null);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function refundOrder(orderId: string) {
    try {
      await selfHosted.refundOrder(orderId);
      await selfHosted.requestWalletStatus();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  function syncLocalCallMedia() {
    const stream = webRtcRef.current?.localMedia() ?? null;
    window.setTimeout(() => {
      if (localVideoRef.current) {
        localVideoRef.current.srcObject = stream;
        if (stream) void localVideoRef.current.play().catch(() => {});
      }
    }, 0);
  }

  async function openCommunityAdmin(peer: PeerItem) {
    if (peer.source !== 'selfhosted' || !peer.conversationId || !['group', 'channel'].includes(peer.kind)) {
      setError('群组/频道管理只对 Fabushi 自建社区开放。');
      return;
    }
    const existing = selfCommunities.find((community) => community.conversationId === peer.conversationId);
    if (!existing) {
      const created = defaultCommunityState(peer.conversationId, selfHosted.actorId);
      try {
        await selfHosted.updateCommunity(created);
        setSelfCommunities((current) => [...current.filter((item) => item.conversationId !== created.conversationId), created]);
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : String(cause));
        return;
      }
    }
    setCommunityDialogPeer(peer);
  }

  async function saveCommunity(community: MessagingCommunityState) {
    try {
      await selfHosted.updateCommunity(community);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function startCall(kind: 'voice' | 'video') {
    if (!activePeer) return;
    if (activePeer.source !== 'selfhosted' || !activePeer.conversationId || !activePeer.actorId) {
      setError('远端通话只对已迁移到 Fabushi 自建协议的一对一联系人开放。');
      return;
    }
    const controller = webRtcRef.current;
    if (!controller) {
      setError('Fabushi WebRTC 控制器尚未准备好。');
      return;
    }
    setLocalCall({
      kind,
      title: activePeer.title,
      status: 'ringing',
      incoming: false,
      muted: false,
      videoEnabled: kind === 'video',
    });
    try {
      await controller.start({
        conversationId: activePeer.conversationId,
        peerActorId: activePeer.actorId,
        kind,
      });
    } catch (cause) {
      setLocalCall({
        kind,
        title: activePeer.title,
        status: 'failed',
        incoming: false,
        muted: false,
        videoEnabled: kind === 'video',
        error: cause instanceof Error ? cause.message : String(cause),
      });
    }
  }

  async function acceptIncomingCall() {
    const controller = webRtcRef.current;
    if (!controller || !incomingCall) return;
    try {
      const accepted = await controller.acceptIncoming();
      setIncomingCall(null);
      setLocalCall((current) => ({
        kind: accepted.kind,
        title: current?.title ?? accepted.peerActorId,
        status: 'connecting',
        incoming: true,
        muted: false,
        videoEnabled: accepted.kind === 'video',
      }));
      syncLocalCallMedia();
    } catch (cause) {
      setLocalCall((current) => current ? {
        ...current,
        status: 'failed',
        error: cause instanceof Error ? cause.message : String(cause),
      } : null);
    }
  }

  async function declineIncomingCall() {
    try {
      await webRtcRef.current?.declineIncoming();
    } finally {
      setIncomingCall(null);
      setLocalCall(null);
    }
  }

  async function endCall() {
    await webRtcRef.current?.hangup();
    setIncomingCall(null);
    setLocalCall(null);
  }

  async function toggleCallMute() {
    if (!localCall) return;
    const next = !localCall.muted;
    await webRtcRef.current?.setMuted(next);
    setLocalCall((current) => current ? { ...current, muted: next } : null);
  }

  async function toggleCallVideo() {
    if (!localCall || localCall.kind !== 'video') return;
    const next = !localCall.videoEnabled;
    await webRtcRef.current?.setVideoEnabled(next);
    setLocalCall((current) => current ? { ...current, videoEnabled: next } : null);
  }

  async function shareCallScreen() {
    try {
      await webRtcRef.current?.startScreenShare();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  function toggleLocalSet(setter: React.Dispatch<React.SetStateAction<Set<string>>>, key: string) {
    setter((current) => {
      const next = new Set(current);
      if (next.has(key)) next.delete(key); else next.add(key);
      return next;
    });
  }

  async function togglePinConversation(peer: PeerItem) {
    if (peer.source === 'selfhosted' && peer.conversationId) {
      await selfHosted.pinConversation(peer.conversationId, !peer.pinned);
      return;
    }
    toggleLocalSet(setPinnedPeerKeys, peer.key);
  }

  async function toggleArchiveConversation(peer: PeerItem) {
    if (peer.source === 'selfhosted' && peer.conversationId) {
      await selfHosted.archiveConversation(peer.conversationId, !peer.archived);
      return;
    }
    toggleLocalSet(setArchivedPeerKeys, peer.key);
  }

  async function toggleMuteConversation(peer: PeerItem) {
    if (peer.source === 'selfhosted' && peer.conversationId) {
      const muted = mutedPeerKeys.has(peer.key);
      await selfHosted.setConversationNotifications(peer.conversationId, muted ? undefined : Date.now() + 365 * 24 * 60 * 60 * 1000);
    }
    toggleLocalSet(setMutedPeerKeys, peer.key);
  }

  async function refreshMiniApps(query = miniAppQuery) {
    setMiniAppLoading(true);
    try {
      const [catalog, installed] = await Promise.all([
        transport.marketplaceBrowse(query),
        transport.pluginListInstalled(),
      ]);
      setMarketplaceApps(catalog.plugins);
      setInstalledMiniApps(Object.fromEntries(installed.plugins.map((plugin) => [plugin.pluginId, plugin])));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setMiniAppLoading(false);
    }
  }

  function setMiniAppBusyState(id: string, busy: boolean) {
    setMiniAppBusy((current) => {
      const next = new Set(current);
      if (busy) next.add(id); else next.delete(id);
      return next;
    });
  }

  async function installMiniApp(app: MarketplacePluginSummary) {
    setError(null);
    setMiniAppBusyState(app.pluginId, true);
    try {
      const release = await transport.marketplaceRelease(app.pluginId, app.latestVersion);
      if (release.releaseStatus && release.releaseStatus !== 'approved') {
        throw new Error(`Mini App release is not approved: ${release.releaseStatus}`);
      }
      if (release.releaseManifest?.protocol !== 'mahayana.external-release.v1') {
        throw new Error('Mini App release is missing a verified external release manifest');
      }
      await transport.pluginInstall(release.releaseManifest, 'desktop');
      await refreshMiniApps(miniAppQuery);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setMiniAppBusyState(app.pluginId, false);
    }
  }

  async function uninstallMiniApp(id: string) {
    setError(null);
    setMiniAppBusyState(id, true);
    try {
      await transport.pluginUninstall(id);
      if (miniApp?.id === id) setMiniApp(null);
      await refreshMiniApps(miniAppQuery);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setMiniAppBusyState(id, false);
    }
  }

  async function openMiniApp(id: string) {
    setError(null);
    setMiniAppBusyState(id, true);
    try {
      const installed = installedMiniApps[id] ?? await transport.pluginActive(id);
      if (!installed) throw new Error('请先从在线 Mini App 市场安装此应用');
      const document = await transport.pluginUiDocument(id);
      const title = marketplaceApps.find((app) => app.pluginId === id)?.displayName ?? id;
      setMiniApp({ id, title, html: document.html });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setMiniAppBusyState(id, false);
    }
  }

  return (
    <main className={`${styles.messenger} ${styles.fabushiUnified}`} data-testid="messenger-workspace" onClick={() => setMessageMenu(null)}>
      <aside className={styles.navRail}>
        <button className={styles.railBrand} type="button" onClick={() => setSection('chats')} title="Fabushi">法</button>
        <RailButton icon={<MessageCircle />} label="聊天" active={section === 'chats'} onClick={() => setSection('chats')} />
        <RailButton icon={<Users />} label="联系人" active={section === 'contacts'} onClick={() => setSection('contacts')} />
        <RailButton icon={<Bot />} label="Bots" active={section === 'bots'} onClick={() => setSection('bots')} />
        <RailButton icon={<Users />} label="群组" active={section === 'groups'} onClick={() => setSection('groups')} />
        <RailButton icon={<Radio />} label="频道" active={section === 'channels'} onClick={() => setSection('channels')} />
        <RailButton icon={<Phone />} label="通话" active={section === 'calls'} onClick={() => setSection('calls')} />
        <RailButton icon={<Bookmark />} label="收藏" active={section === 'saved'} onClick={() => void ensureSavedMessages()} />
        <RailButton icon={<Archive />} label="归档" active={section === 'archive'} onClick={() => setSection('archive')} />
        <RailButton icon={<Folder />} label="文件夹" active={section === 'folders'} onClick={() => setSection('folders')} />
        <div className={styles.railSpacer} />
        <RailButton icon={<AppWindow />} label="Mini Apps" active={section === 'miniapps'} onClick={() => setSection('miniapps')} />
        <RailButton icon={<WalletCards />} label="支付" active={section === 'payments'} onClick={() => setSection('payments')} />
        <RailButton icon={<Settings />} label="设置" active={section === 'settings'} onClick={() => setSection('settings')} />
      </aside>

      <aside className={styles.chatList}>
        <header className={styles.listHeader}>
          <button type="button" className={styles.iconButton} aria-label="菜单"><Menu size={19} /></button>
          <strong>{sectionTitle(section)}</strong>
          <button type="button" className={styles.iconButton} aria-label="新建" onClick={() => setNewDialog({ type: 'group', name: '', selectedBotIds: new Set() })}><SquarePen size={18} /></button>
        </header>
        <label className={styles.searchBox}>
          <Search size={16} />
          <input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="搜索消息、联系人和 Bot" />
          {search ? <button type="button" onClick={() => setSearch('')}><X size={14} /></button> : null}
        </label>
        {['chats', 'contacts'].includes(section) ? <div className={extra.storyStrip}>
          <button type="button" className={extra.storyItem} onClick={() => storyInputRef.current?.click()} title="发布动态">
            <span className={extra.storyRing} data-own="true">{avatarText('我')}<b>+</b></span><small>我的动态</small>
          </button>
          {visibleStories.map((story) => {
            const actor = selfActors.find((item) => item.id === story.ownerId);
            const name = actor?.displayName ?? (story.ownerId === selfHosted.actorId ? '我' : story.ownerId);
            return <button type="button" className={extra.storyItem} key={story.id} onClick={() => void openStory(story)} title={name}>
              <span className={extra.storyRing}>{actor?.avatarUrl ? <img src={actor.avatarUrl} alt="" /> : avatarText(name)}</span><small>{name}</small>
            </button>;
          })}
          <input ref={storyInputRef} type="file" accept="image/*,video/*" hidden onChange={(event) => { const file = event.currentTarget.files?.[0]; event.currentTarget.value = ''; if (file) void publishStoryFile(file); }} />
        </div> : null}
        {sectionIsPeerList ? (
          <div className={styles.peerList}>
            <div className={styles.quickActions}>
              <button type="button" onClick={() => setNewDialog({ type: 'group', name: '', selectedBotIds: new Set() })}><Users size={17} /><span>新建群组</span></button>
              <button type="button" onClick={() => setNewDialog({ type: 'channel', name: '', description: '' })}><Radio size={17} /><span>新建频道</span></button>
            </div>
            {renderedPeers.map((peer) => (
              <button data-testid={`peer-${peer.key}`} key={peer.key} type="button" className={peer.key === activePeerKey ? styles.peerActive : styles.peer} onClick={() => void openPeer(peer)}>
                {isAgentPeer(peer) ? (
                  <BotMark
                    botId={peer.actorId ?? peer.id}
                    state={botMarkStateForPeer(peer, selfBotExecutions, peer.key === activePeerKey && pendingSend, hostReady)}
                    size={48}
                    className={styles.agentAvatarMark}
                    label={peer.title}
                  />
                ) : <span className={styles.avatar}>{peer.avatar ? <img src={peer.avatar} alt="" /> : avatarText(peer.title)}<i data-kind={peer.kind} /></span>}
                <span className={styles.peerCopy}>
                  <span><strong>{peer.title}</strong><time>{formatTime(peer.updatedAtMs)}</time></span>
                  <small>{peer.subtitle}</small>
                </span>
                <span className={styles.peerMeta}>{peer.pinned ? <Pin size={12} /> : null}{mutedPeerKeys.has(peer.key) ? <BellOff size={12} /> : null}{peer.unread ? <b>{peer.unread}</b> : null}</span>
              </button>
            ))}
            {visiblePeers.length > renderedPeers.length ? <button type="button" data-testid="peer-list-load-more" onClick={() => setPeerRenderCount((count) => count + initialPeerRenderCount)}>显示更多会话</button> : null}
            {!visiblePeers.length ? <EmptyList section={section} /> : null}
          </div>
        ) : (
          <SectionPanel section={section} onOpenMiniApp={openMiniApp} onInstallMiniApp={installMiniApp} onUninstallMiniApp={uninstallMiniApp} miniApps={marketplaceApps} installedMiniApps={installedMiniApps} miniAppQuery={miniAppQuery} onMiniAppQuery={setMiniAppQuery} miniAppLoading={miniAppLoading} miniAppBusy={miniAppBusy} onInvoice={() => void createInvoiceForActivePeer()} payment={{ account: walletAccount, entries: walletEntries, orders: selfOrders, invoices: selfInvoices, actorId: selfHosted.actorId }} onRefund={(orderId) => void refundOrder(orderId)} />
        )}
      </aside>

      <section className={styles.chatWorkspace}>
        {activePeer && sectionIsPeerList ? (
          <>
            <header className={styles.chatHeader}>
              <div className={styles.chatIdentity}>
                {isAgentPeer(activePeer) ? (
                  <BotMark
                    botId={activePeer.actorId ?? activePeer.id}
                    state={botMarkStateForPeer(activePeer, selfBotExecutions, pendingSend, hostReady)}
                    size={40}
                    className={styles.agentAvatarMark}
                    label={activePeer.title}
                  />
                ) : <span className={styles.avatar}>{avatarText(activePeer.title)}<i data-kind={activePeer.kind} /></span>}
                <div><strong>{activePeer.title}</strong><small data-testid="conversation-status">{activeTypingActors.length ? '正在输入…' : `${activePeer.subtitle}${hostReady ? ' · 在线' : ' · 正在连接'}`}</small></div>
              </div>
              <div className={styles.headerActions}>
                <button type="button" title="语音通话" onClick={() => void startCall('voice')}><PhoneCall size={18} /></button>
                <button type="button" title="视频通话" onClick={() => void startCall('video')}><Video size={18} /></button>
                {activePeer.source === 'selfhosted' ? <button type="button" title="发送账单" onClick={() => void createInvoiceForActivePeer()}><WalletCards size={18} /></button> : null}
                <button type="button" title="搜索当前会话" data-active={conversationSearchOpen} onClick={() => setConversationSearchOpen((value) => !value)}><Search size={18} /></button>
                <button type="button" title={activePeer.pinned ? '取消置顶' : '置顶'} onClick={() => void togglePinConversation(activePeer)}><Pin size={18} /></button>
                <button type="button" title={mutedPeerKeys.has(activePeer.key) ? '开启通知' : '静音'} onClick={() => void toggleMuteConversation(activePeer)}><BellOff size={18} /></button>
                <button type="button" title="资料" data-active={infoOpen} onClick={() => setInfoOpen((value) => !value)}><MoreVertical size={18} /></button>
              </div>
            </header>
            {conversationSearchOpen ? <label className={styles.inChatSearch}><Search size={15} /><input data-testid="conversation-search-input" value={conversationSearch} onChange={(event) => { setConversationSearch(event.target.value); setMessageRenderCount(initialMessageRenderCount); }} placeholder="在当前会话中搜索" autoFocus /><button type="button" onClick={() => { setConversationSearch(''); setConversationSearchOpen(false); }}><X size={14} /></button></label> : null}
            {error ? <div className={styles.errorBanner} role="alert"><span>{error}</span><button type="button" onClick={() => setError(null)}><X size={14} /></button></div> : null}
            <div className={styles.messageArea}>
              <div className={styles.dayDivider}>今天</div>
              {matchingMessages.length > renderedMessages.length ? <button type="button" data-testid="message-list-load-earlier" onClick={() => setMessageRenderCount((count) => count + initialMessageRenderCount)}>加载更早消息</button> : null}
              {renderedMessages.map((message) => (
                <article
                  key={`${message.source}:${message.id}`}
                  className={message.role === 'me' ? styles.messageMine : styles.messagePeer}
                  onContextMenu={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    setMessageMenu({ message, x: event.clientX, y: event.clientY });
                  }}
                >
                  {message.pinned ? <Pin size={11} /> : null}
                  {message.mediaType === 'photo' && blobMediaUrl(message.media) ? <img className={extra.messageMedia} src={blobMediaUrl(message.media)} alt={message.media?.fileName ?? '图片'} /> : null}
                  {message.mediaType === 'video' && blobMediaUrl(message.media) ? <video className={extra.messageMedia} controls playsInline src={blobMediaUrl(message.media)} /> : null}
                  {message.mediaType === 'document' && blobMediaUrl(message.media) ? <a className={extra.messageFile} href={blobMediaUrl(message.media)} download={message.media?.fileName}><FileText size={17} />{message.media?.fileName ?? '文件'}</a> : null}
                  <p>{message.text}</p>
                  {message.reactions?.length ? <div className={extra.reactions}>{message.reactions.map((reaction) => <span key={reaction}>{reaction}</span>)}</div> : null}
                  <small>{formatTime(message.createdAtMs)} {message.role === 'me' ? <Check size={12} /> : null}</small>
                </article>
              ))}
              {!matchingMessages.length ? <div className={styles.chatEmpty} data-testid="message-search-empty">{isAgentPeer(activePeer) ? <BotMark botId={activePeer.actorId ?? activePeer.id} state={botMarkStateForPeer(activePeer, selfBotExecutions, false, hostReady)} size={78} className={styles.agentAvatarMark} label={activePeer.title} /> : <span className={styles.avatarLarge}>{avatarText(activePeer.title)}</span>}<strong>{conversationQuery ? '没有匹配消息' : activePeer.title}</strong><p>{conversationQuery ? '换一个关键词继续搜索。' : '联系人、AI Bot、群组和频道使用同一个 Fabushi 消息产品层。'}</p></div> : null}
            </div>
            {replyTo ? <div className={extra.composerBanner}><Reply size={15} /><div><strong>回复</strong><span>{replyTo.text}</span></div><button type="button" onClick={() => setReplyTo(null)}><X size={14} /></button></div> : null}
            {scheduledAtMs ? <div className={extra.composerBanner}><span>⏱</span><div><strong>定时发送</strong><span>{new Date(scheduledAtMs).toLocaleString()}</span></div><button type="button" onClick={() => setScheduledAtMs(undefined)}><X size={14} /></button></div> : null}
            <form className={styles.composer} onSubmit={(event) => void sendMessage(event)}>
              <div className={extra.attachmentAnchor}>
                <button type="button" title="附件" onClick={() => setAttachmentMenuOpen((value) => !value)}><Paperclip size={20} /></button>
                {attachmentMenuOpen ? <AttachmentMenu onMedia={() => mediaInputRef.current?.click()} onFile={() => fileInputRef.current?.click()} onPoll={() => void sendPoll()} onLocation={() => void sendLocation()} onSchedule={() => {
                  const minutes = Number(window.prompt('多少分钟后发送？', '10'));
                  if (Number.isFinite(minutes) && minutes > 0) setScheduledAtMs(Date.now() + minutes * 60_000);
                  setAttachmentMenuOpen(false);
                }} /> : null}
              </div>
              <input ref={mediaInputRef} type="file" accept="image/*,video/*" hidden onChange={(event) => { const file = event.currentTarget.files?.[0]; event.currentTarget.value = ''; if (file) void sendAttachmentFile(file); }} />
              <input ref={fileInputRef} type="file" hidden onChange={(event) => { const file = event.currentTarget.files?.[0]; event.currentTarget.value = ''; if (file) void sendAttachmentFile(file); }} />
              {attachmentProgress ? <span className={extra.uploadProgress}>{attachmentProgress}</span> : null}
              <textarea data-testid="messenger-input" value={composer} onChange={(event) => updateComposer(event.target.value)} onKeyDown={(event) => { if (event.key === 'Enter' && !event.shiftKey) { event.preventDefault(); event.currentTarget.form?.requestSubmit(); } }} placeholder="消息" rows={1} />
              <button type="button" title="表情"><Smile size={20} /></button>
              <button type="button" data-active={silentSend} title={silentSend ? '关闭静默发送' : '静默发送'} onClick={() => setSilentSend((value) => !value)}><BellOff size={19} /></button>
              {composer.trim() ? <button data-testid="messenger-send" className={styles.sendButton} type="submit" disabled={!hostReady || pendingSend}><Send size={19} /></button> : <button type="button" title="语音消息"><Mic size={20} /></button>}
            </form>
          </>
        ) : (
          <FeatureWorkspace section={section} onOpenMiniApp={openMiniApp} onInstallMiniApp={installMiniApp} onUninstallMiniApp={uninstallMiniApp} miniApps={marketplaceApps} installedMiniApps={installedMiniApps} miniAppQuery={miniAppQuery} onMiniAppQuery={setMiniAppQuery} miniAppLoading={miniAppLoading} miniAppBusy={miniAppBusy} onInvoice={() => void createInvoiceForActivePeer()} payment={{ account: walletAccount, entries: walletEntries, orders: selfOrders, invoices: selfInvoices, actorId: selfHosted.actorId }} onRefund={(orderId) => void refundOrder(orderId)} />
        )}
      </section>

      {infoOpen && activePeer && sectionIsPeerList ? (
        <aside className={styles.infoPanel}>
          <header><strong>资料</strong><button type="button" onClick={() => setInfoOpen(false)}><X size={17} /></button></header>
          <div className={styles.profileCard}>
            {isAgentPeer(activePeer) ? <BotMark botId={activePeer.actorId ?? activePeer.id} state={botMarkStateForPeer(activePeer, selfBotExecutions, pendingSend, hostReady)} size={92} className={styles.agentProfileMark} label={activePeer.title} /> : <span className={styles.avatarHuge}>{avatarText(activePeer.title)}</span>}
            <strong>{activePeer.title}</strong><small>{activePeer.subtitle}</small>
            <div><button type="button" onClick={() => void startCall('voice')}><PhoneCall size={18} /><span>通话</span></button><button type="button" onClick={() => void startCall('video')}><Video size={18} /><span>视频</span></button><button type="button" onClick={() => setConversationSearchOpen(true)}><Search size={18} /><span>搜索</span></button></div>
          </div>
          <div className={styles.profileActions}>
            <button type="button" onClick={() => void toggleMuteConversation(activePeer)}><BellOff size={17} /><span>{mutedPeerKeys.has(activePeer.key) ? '开启通知' : '静音通知'}</span></button>
            <button type="button" onClick={() => void togglePinConversation(activePeer)}><Pin size={17} /><span>{activePeer.pinned ? '取消置顶' : '置顶会话'}</span></button>
            <button type="button" onClick={() => { void toggleArchiveConversation(activePeer); setSection('archive'); }}><Archive size={17} /><span>{activePeer.archived ? '移出归档' : '归档会话'}</span></button>
            {activePeer.source === 'selfhosted' && ['group', 'channel'].includes(activePeer.kind) ? <button type="button" onClick={() => void openCommunityAdmin(activePeer)}><Settings size={17} /><span>管理群组/频道</span></button> : null}
          </div>
          <nav className={styles.infoTabs}><button type="button" data-active={infoTab === 'media'} onClick={() => setInfoTab('media')}>媒体</button><button type="button" data-active={infoTab === 'files'} onClick={() => setInfoTab('files')}>文件</button><button type="button" data-active={infoTab === 'links'} onClick={() => setInfoTab('links')}>链接</button></nav>
          <div className={styles.infoContent}>{infoTab === 'media' ? <><Image size={30} /><strong>共享媒体</strong><p>图片、视频和动画按消息索引展示。</p></> : null}{infoTab === 'files' ? <><FileText size={30} /><strong>共享文件</strong><p>文档、音频和附件由 Rust 媒体层管理。</p></> : null}{infoTab === 'links' ? <><Link2 size={30} /><strong>共享链接</strong><p>富文本 URL 建立可搜索索引。</p></> : null}</div>
        </aside>
      ) : null}

      {messageMenu ? <MessageContextMenu menu={messageMenu} onAction={(action) => void handleMessageAction(action)} /> : null}
      {forwardDialog ? <ForwardMessageDialog message={forwardDialog.message} peers={peers.filter((peer) => peer.source === 'selfhosted' && Boolean(peer.conversationId) && peer.conversationId !== forwardDialog.sourceConversationId)} onClose={() => setForwardDialog(null)} onSelect={(peer) => void forwardToPeer(peer)} /> : null}
      {editDialog ? <EditMessageDialog value={editDialog.text} onChange={(text) => setEditDialog((current) => current ? { ...current, text } : current)} onClose={() => setEditDialog(null)} onSave={() => void saveEditedMessage()} /> : null}
      {invoiceDialog ? <InvoiceDialog dialog={invoiceDialog} onChange={setInvoiceDialog} onClose={() => setInvoiceDialog(null)} onSave={() => void saveInvoiceDialog()} /> : null}
      {newDialog ? <NewConversationDialog dialog={newDialog} bots={bots} onChange={setNewDialog} onClose={() => setNewDialog(null)} onSave={() => void saveNewDialog()} /> : null}
      {communityDialogPeer ? <CommunityAdminDialog peer={communityDialogPeer} community={selfCommunities.find((item) => item.conversationId === communityDialogPeer.conversationId) ?? defaultCommunityState(communityDialogPeer.conversationId!, selfHosted.actorId)} actors={selfActors} actorId={selfHosted.actorId} onClose={() => setCommunityDialogPeer(null)} onSave={(community) => void saveCommunity(community)} onSetMember={(conversationId, member) => void selfHosted.setCommunityMember(conversationId, member).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onCreateInvite={(community) => { const invite = { id: `invite:${crypto.randomUUID()}`, conversationId: community.conversationId, creatorId: selfHosted.actorId, token: crypto.randomUUID().replaceAll('-', ''), name: '邀请链接', createdAtMs: Date.now(), expiresAtMs: undefined, memberLimit: undefined, joinRequest: community.joinRequestRequired, revoked: false, joinedCount: 0 }; void selfHosted.createInviteLink(invite).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause))); }} onRevokeInvite={(conversationId, inviteId) => void selfHosted.revokeInviteLink(conversationId, inviteId).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onJoinDecision={(conversationId, requesterId, approved) => void selfHosted.respondCommunityJoin(conversationId, requesterId, approved).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onUpsertTopic={(topic) => void selfHosted.upsertForumTopic(topic).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onDeleteTopic={(conversationId, topicId) => void selfHosted.deleteForumTopic(conversationId, topicId).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} /> : null}
      {activeStory ? <StoryViewer story={activeStory} owner={selfActors.find((actor) => actor.id === activeStory.ownerId)} own={activeStory.ownerId === selfHosted.actorId} onClose={() => setActiveStory(null)} onReact={(reaction) => void reactToActiveStory(reaction)} onDelete={() => void selfHosted.deleteStory(activeStory.id).then(() => setActiveStory(null)).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} /> : null}
      {localCall ? <CallDialog call={localCall} localVideoRef={localVideoRef} remoteVideoRef={remoteVideoRef} remoteAudioRef={remoteAudioRef} canAccept={Boolean(incomingCall)} onAccept={() => void acceptIncomingCall()} onDecline={() => void declineIncomingCall()} onMute={() => void toggleCallMute()} onVideo={() => void toggleCallVideo()} onShare={() => void shareCallScreen()} onEnd={() => void endCall()} /> : null}
      {miniApp ? <MiniAppDialog app={miniApp} onClose={() => setMiniApp(null)} /> : null}
    </main>
  );
}

function RailButton({ icon, label, active, onClick }: { icon: React.ReactNode; label: string; active: boolean; onClick: () => void }) {
  return <button type="button" data-active={active} onClick={onClick} title={label}><span>{icon}</span><small>{label}</small></button>;
}

function EmptyList({ section }: { section: MessengerSection }) {
  return <div className={styles.emptyList}><MessageCircle size={27} /><strong>暂无{sectionTitle(section)}</strong><p>新建会话后会显示在这里。</p></div>;
}

function AttachmentMenu({ onMedia, onFile, onPoll, onLocation, onSchedule }: { onMedia: () => void; onFile: () => void; onPoll: () => void; onLocation: () => void; onSchedule: () => void }) {
  return <div className={extra.popover} onClick={(event) => event.stopPropagation()}>
    <button type="button" onClick={onMedia}><Image size={17} />图片或视频</button>
    <button type="button" onClick={onFile}><FileText size={17} />文件</button>
    <button type="button" onClick={onPoll}><span>📊</span>投票</button>
    <button type="button" onClick={onLocation}><MapPin size={17} />位置</button>
    <button type="button" onClick={onSchedule}><span>⏱</span>定时发送</button>
  </div>;
}

function MessageContextMenu({ menu, onAction }: { menu: NonNullable<MessageMenu>; onAction: (action: 'copy' | 'reply' | 'forward' | 'checkout' | 'edit' | 'delete' | 'react' | 'pin') => void }) {
  return <div className={extra.contextMenu} style={{ left: menu.x, top: menu.y }} onClick={(event) => event.stopPropagation()}>
    <button type="button" onClick={() => onAction('reply')}><Reply size={16} />回复</button>
    <button type="button" onClick={() => onAction('copy')}><Copy size={16} />复制</button>
    <button type="button" onClick={() => onAction('react')}><Smile size={16} />👍 反应</button>
    {menu.message.role === 'me' ? <button type="button" onClick={() => onAction('edit')}><Edit3 size={16} />编辑</button> : null}
    <button type="button" onClick={() => onAction('pin')}><Pin size={16} />{menu.message.pinned ? '取消置顶' : '置顶'}</button>
    <button type="button" onClick={() => onAction('forward')}><Forward size={16} />转发</button>
    {menu.message.invoiceId ? <button type="button" onClick={() => onAction('checkout')}><WalletCards size={16} />支付账单</button> : null}
    <button type="button" onClick={() => onAction('delete')}><Trash2 size={16} />删除</button>
  </div>;
}

function ForwardMessageDialog({ message, peers, onClose, onSelect }: { message: DisplayMessage; peers: PeerItem[]; onClose: () => void; onSelect: (peer: PeerItem) => void }) {
  return <div className={styles.backdrop} onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>转发消息</strong><small>{message.text || '媒体消息'}</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <div className={extra.forwardList}>
      {peers.length ? peers.map((peer) => <button key={peer.key} type="button" onClick={() => onSelect(peer)}><span className={styles.avatar}>{avatarText(peer.title)}</span><span><strong>{peer.title}</strong><small>{peer.subtitle}</small></span><Forward size={16} /></button>) : <p>暂无可转发的自建会话，请先创建群组、频道或收藏消息。</p>}
    </div>
  </section></div>;
}

function EditMessageDialog({ value, onChange, onClose, onSave }: { value: string; onChange: (value: string) => void; onClose: () => void; onSave: () => void }) {
  return <div className={styles.backdrop} onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>编辑消息</strong><small>修改后会同步到 Fabushi 自建会话</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <label><span>消息内容</span><textarea autoFocus data-testid="edit-message-input" value={value} onChange={(event) => onChange(event.target.value)} rows={4} placeholder="编辑消息内容" /></label>
    <footer><button type="button" onClick={onClose}>取消</button><button type="button" className={styles.primaryButton} disabled={!value.trim()} onClick={onSave}>保存</button></footer>
  </section></div>;
}

function InvoiceDialog({ dialog, onChange, onClose, onSave }: { dialog: Exclude<InvoiceDialogState, null>; onChange: React.Dispatch<React.SetStateAction<InvoiceDialogState>>; onClose: () => void; onSave: () => void }) {
  const amount = Number(dialog.amount);
  const validAmount = Number.isFinite(amount) && amount > 0;
  return <div className={styles.backdrop} onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>发送账单</strong><small>Fabushi Pay · USD</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <label><span>账单名称</span><input autoFocus data-testid="invoice-title-input" value={dialog.title} onChange={(event) => onChange((current) => current ? { ...current, title: event.target.value } : current)} placeholder="订单" /></label>
    <label><span>金额（USD）</span><input data-testid="invoice-amount-input" inputMode="decimal" value={dialog.amount} onChange={(event) => onChange((current) => current ? { ...current, amount: event.target.value } : current)} placeholder="9.99" /></label>
    <footer><button type="button" onClick={onClose}>取消</button><button type="button" className={styles.primaryButton} disabled={!dialog.title.trim() || !validAmount} onClick={onSave}>创建账单</button></footer>
  </section></div>;
}

function NewConversationDialog({ dialog, bots, onChange, onClose, onSave }: { dialog: Exclude<NewDialog, null>; bots: BotSummary[]; onChange: React.Dispatch<React.SetStateAction<NewDialog>>; onClose: () => void; onSave: () => void }) {
  return <div className={styles.backdrop} onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>{dialog.type === 'group' ? '新建群组' : '新建频道'}</strong><small>{dialog.type === 'group' ? '现有 AI 群组 Host 会执行 Bot 多轮协作' : 'Fabushi 自建广播会话'}</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <label><span>名称</span><input autoFocus value={dialog.name} onChange={(event) => onChange((current) => current ? { ...current, name: event.target.value } : current)} placeholder={dialog.type === 'group' ? '群组名称' : '频道名称'} /></label>
    {dialog.type === 'channel' ? <label><span>描述</span><textarea value={dialog.description} onChange={(event) => onChange((current) => current?.type === 'channel' ? { ...current, description: event.target.value } : current)} rows={3} placeholder="频道简介" /></label> : null}
    {dialog.type === 'group' ? <div className={styles.memberPicker}><span>选择 AI Bot</span>{bots.map((bot) => {
      const selected = dialog.selectedBotIds.has(bot.id);
      return <button key={bot.id} type="button" data-testid={`group-bot-${bot.id}`} data-selected={selected} onClick={() => onChange((current) => {
        if (!current || current.type !== 'group') return current;
        const selectedBotIds = new Set(current.selectedBotIds);
        if (selectedBotIds.has(bot.id)) selectedBotIds.delete(bot.id); else selectedBotIds.add(bot.id);
        return { ...current, selectedBotIds };
      })}><span className={styles.avatarSmall}>{avatarText(bot.name)}</span><div><strong>{bot.name}</strong><small>{bot.description}</small></div>{selected ? <Check size={16} /> : <Plus size={16} />}</button>;
    })}</div> : null}
    <footer><button type="button" onClick={onClose}>取消</button><button type="button" className={styles.primaryButton} disabled={!dialog.name.trim() || (dialog.type === 'group' && dialog.selectedBotIds.size === 0)} onClick={onSave}>{dialog.type === 'group' ? '创建群组' : '创建频道'}</button></footer>
  </section></div>;
}

function CommunityAdminDialog({
  peer,
  community,
  actors,
  actorId,
  onClose,
  onSave,
  onSetMember,
  onCreateInvite,
  onRevokeInvite,
  onJoinDecision,
  onUpsertTopic,
  onDeleteTopic,
}: {
  peer: PeerItem;
  community: MessagingCommunityState;
  actors: MessagingActor[];
  actorId: string;
  onClose: () => void;
  onSave: (community: MessagingCommunityState) => void;
  onSetMember: (conversationId: string, member: MessagingCommunityMember) => void;
  onCreateInvite: (community: MessagingCommunityState) => void;
  onRevokeInvite: (conversationId: string, inviteId: string) => void;
  onJoinDecision: (conversationId: string, requesterId: string, approved: boolean) => void;
  onUpsertTopic: (topic: MessagingForumTopic) => void;
  onDeleteTopic: (conversationId: string, topicId: string) => void;
}) {
  const [draft, setDraft] = useState(community);
  useEffect(() => setDraft(community), [community]);
  const actorName = (id: string) => actors.find((actor) => actor.id === id)?.displayName ?? id;
  const members = Object.values(draft.members);
  const invites = Object.values(draft.inviteLinks).filter((invite) => !invite.revoked);
  const requests = Object.values(draft.pendingJoinRequests);
  const topics = Object.values(draft.topics);
  const canManage = draft.members[actorId]?.status === 'owner' || draft.members[actorId]?.status === 'administrator';

  function changeMemberRole(member: MessagingCommunityMember, status: MessagingCommunityMember['status']) {
    const next: MessagingCommunityMember = {
      ...member,
      status,
      adminRights: status === 'administrator' || status === 'owner'
        ? {
            ...member.adminRights,
            changeInfo: true,
            deleteMessages: true,
            banMembers: true,
            inviteMembers: true,
            pinMessages: true,
            manageTopics: true,
            manageCalls: true,
          }
        : member.adminRights,
    };
    onSetMember(draft.conversationId, next);
  }

  function createTopic() {
    const title = window.prompt('Topic 名称');
    if (!title?.trim()) return;
    onUpsertTopic({
      id: `topic:${crypto.randomUUID()}`,
      conversationId: draft.conversationId,
      title: title.trim(),
      creatorId: actorId,
      createdAtMs: Date.now(),
      pinned: false,
      closed: false,
      hidden: false,
      unreadCount: 0,
    });
  }

  return <div className={styles.backdrop} onMouseDown={onClose}><section className={extra.communityDialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>{peer.title} · 管理</strong><small>所有修改都由 Rust Community 权限层校验</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <div className={extra.communityBody}>
      <section>
        <h3>群组 / 频道设置</h3>
        <label><span>公开用户名</span><input value={draft.publicUsername ?? ''} onChange={(event) => setDraft((current) => ({ ...current, publicUsername: event.target.value || undefined }))} disabled={!canManage} placeholder="例如 fabushi" /></label>
        <label><span>慢速模式（秒）</span><input type="number" min={0} max={3600} value={draft.slowModeSeconds ?? 0} onChange={(event) => setDraft((current) => ({ ...current, slowModeSeconds: Number(event.target.value) || undefined }))} disabled={!canManage} /></label>
        <label><span>敏感词</span><textarea value={draft.bannedWords.join(', ')} onChange={(event) => setDraft((current) => ({ ...current, bannedWords: event.target.value.split(',').map((value) => value.trim()).filter(Boolean) }))} disabled={!canManage} rows={2} /></label>
        <div className={extra.communityToggles}>
          <label><input type="checkbox" checked={draft.joinRequestRequired} onChange={(event) => setDraft((current) => ({ ...current, joinRequestRequired: event.target.checked }))} disabled={!canManage} />入群需审批</label>
          <label><input type="checkbox" checked={draft.joinToSend} onChange={(event) => setDraft((current) => ({ ...current, joinToSend: event.target.checked }))} disabled={!canManage} />发言前必须入群</label>
          <label><input type="checkbox" checked={draft.signaturesEnabled} onChange={(event) => setDraft((current) => ({ ...current, signaturesEnabled: event.target.checked }))} disabled={!canManage} />频道签名</label>
        </div>
        <button type="button" className={styles.primaryButton} disabled={!canManage} onClick={() => onSave(draft)}>保存设置</button>
      </section>

      <section>
        <h3>成员</h3>
        <div className={extra.communityList}>{members.length ? members.map((member) => <div className={extra.communityRow} key={member.actorId}><div><strong>{actorName(member.actorId)}</strong><small>{member.actorId}</small></div><select value={member.status} disabled={!canManage || member.actorId === actorId && member.status === 'owner'} onChange={(event) => changeMemberRole(member, event.target.value as MessagingCommunityMember['status'])}><option value="owner">Owner</option><option value="administrator">Admin</option><option value="member">Member</option><option value="restricted">Restricted</option><option value="banned">Banned</option><option value="left">Left</option></select></div>) : <small>暂无成员记录</small>}</div>
      </section>

      <section>
        <div className={extra.communitySectionTitle}><h3>邀请链接</h3><button type="button" disabled={!canManage} onClick={() => onCreateInvite(draft)}><UserPlus size={15} />创建</button></div>
        <div className={extra.communityList}>{invites.length ? invites.map((invite) => <div className={extra.communityRow} key={invite.id}><div><strong>{invite.name ?? '邀请链接'}</strong><small>{invite.token}</small></div><div><button type="button" onClick={() => void navigator.clipboard.writeText(`fabushi://join/${invite.token}`)}><Copy size={14} /></button><button type="button" disabled={!canManage} onClick={() => onRevokeInvite(draft.conversationId, invite.id)}><Trash2 size={14} /></button></div></div>) : <small>暂无有效邀请链接</small>}</div>
      </section>

      <section>
        <h3>待审批</h3>
        <div className={extra.communityList}>{requests.length ? requests.map((request) => <div className={extra.communityRow} key={request.actorId}><div><strong>{actorName(request.actorId)}</strong><small>{request.bio ?? '请求加入'}</small></div><div><button type="button" disabled={!canManage} onClick={() => onJoinDecision(draft.conversationId, request.actorId, true)}>通过</button><button type="button" disabled={!canManage} onClick={() => onJoinDecision(draft.conversationId, request.actorId, false)}>拒绝</button></div></div>) : <small>暂无待审批成员</small>}</div>
      </section>

      <section>
        <div className={extra.communitySectionTitle}><h3>Forum Topics</h3><button type="button" disabled={!canManage} onClick={createTopic}><SquarePen size={15} />新建</button></div>
        <div className={extra.communityList}>{topics.length ? topics.map((topic) => <div className={extra.communityRow} key={topic.id}><div><strong>{topic.title}</strong><small>{topic.closed ? '已关闭' : topic.pinned ? '已置顶' : '开放'}</small></div><button type="button" disabled={!canManage} onClick={() => onDeleteTopic(draft.conversationId, topic.id)}><Trash2 size={14} /></button></div>) : <small>暂无 Topic</small>}</div>
      </section>
    </div>
  </section></div>;
}

function StoryViewer({ story, owner, own, onClose, onReact, onDelete }: { story: MessagingStory; owner?: MessagingActor; own: boolean; onClose: () => void; onReact: (reaction: string) => void; onDelete: () => void }) {
  const source = blobMediaUrl(story.media);
  const isVideo = story.media.mimeType?.startsWith('video/') === true;
  return <div className={extra.storyBackdrop} onMouseDown={onClose}>
    <section className={extra.storyViewer} onMouseDown={(event) => event.stopPropagation()}>
      <header><span className={styles.avatar}>{owner?.avatarUrl ? <img src={owner.avatarUrl} alt="" /> : avatarText(owner?.displayName ?? story.ownerId)}</span><div><strong>{owner?.displayName ?? story.ownerId}</strong><small>{new Date(story.createdAtMs).toLocaleString()}</small></div><button type="button" onClick={onClose}><X size={18} /></button></header>
      <div className={extra.storyMedia}>{source ? isVideo ? <video src={source} autoPlay controls playsInline /> : <img src={source} alt={story.caption.text || 'Story'} /> : <span>媒体不可用</span>}</div>
      {story.caption.text ? <p>{story.caption.text}</p> : null}
      <footer>{['👍', '❤️', '🔥', '🙏'].map((reaction) => <button key={reaction} type="button" onClick={() => onReact(reaction)}>{reaction}</button>)}{own ? <button type="button" onClick={onDelete}><Trash2 size={16} />删除</button> : null}</footer>
    </section>
  </div>;
}

function CallDialog({
  call,
  localVideoRef,
  remoteVideoRef,
  remoteAudioRef,
  canAccept,
  onAccept,
  onDecline,
  onMute,
  onVideo,
  onShare,
  onEnd,
}: {
  call: LocalCall;
  localVideoRef: React.RefObject<HTMLVideoElement | null>;
  remoteVideoRef: React.RefObject<HTMLVideoElement | null>;
  remoteAudioRef: React.RefObject<HTMLAudioElement | null>;
  canAccept: boolean;
  onAccept: () => void;
  onDecline: () => void;
  onMute: () => void;
  onVideo: () => void;
  onShare: () => void;
  onEnd: () => void;
}) {
  const statusText = call.status === 'ringing'
    ? call.incoming ? '来电…' : '正在呼叫…'
    : call.status === 'connecting'
      ? '正在建立端到端媒体连接…'
      : call.status === 'active'
        ? `${call.kind === 'video' ? '视频' : '语音'}通话中`
        : call.status === 'failed'
          ? '通话连接失败'
          : '通话已结束';
  return <div className={styles.backdrop}><section className={styles.callDialog}>
    <header><span className={styles.avatarHuge}>{avatarText(call.title)}</span><strong>{call.title}</strong><small>{statusText}</small></header>
    {call.kind === 'video' ? <div className={extra.callVideoStage}><video ref={remoteVideoRef} autoPlay playsInline className={extra.remoteVideo} /><video ref={localVideoRef} autoPlay muted playsInline className={extra.localVideoPip} /></div> : <audio ref={remoteAudioRef} autoPlay />}
    {call.error ? <p>{call.error}</p> : null}
    {canAccept && call.incoming && call.status === 'ringing' ? <div className={styles.callActions}><button type="button" onClick={onDecline} className={styles.hangup}><Phone size={20} /></button><button type="button" onClick={onAccept}><PhoneCall size={20} /></button></div> : <div className={styles.callActions}>
      <button type="button" data-active={call.muted} onClick={onMute} title={call.muted ? '取消静音' : '静音'}><Mic size={20} /></button>
      {call.kind === 'video' ? <button type="button" data-active={!call.videoEnabled} onClick={onVideo} title="摄像头"><Video size={20} /></button> : null}
      {call.kind === 'video' ? <button type="button" onClick={onShare} title="共享屏幕"><Radio size={20} /></button> : null}
      <button type="button" className={styles.hangup} onClick={onEnd}><Phone size={21} /></button>
    </div>}
    <p className={styles.callNote}>Fabushi 自建信令 + WebRTC 一对一媒体；远端信令强制 TLS，NAT 穿透使用部署方配置的自建 STUN/TURN。</p>
  </section></div>;
}

function MiniAppDialog({ app, onClose }: { app: { id: string; title: string; html: string }; onClose: () => void }) {
  return <div className={styles.backdrop} onMouseDown={onClose}><section className={styles.miniAppDialog} onMouseDown={(event) => event.stopPropagation()}><header><div><strong>{app.title}</strong><small>Mini App · 已安装线上包 · 受控宿主容器</small></div><button type="button" onClick={onClose}><X size={17} /></button></header><iframe title={app.id} sandbox="allow-scripts allow-forms" srcDoc={app.html} /></section></div>;
}

type PaymentUiState = {
  account: MessagingWalletAccount | null;
  entries: MessagingLedgerEntry[];
  orders: MessagingOrder[];
  invoices: MessagingInvoice[];
  actorId: string;
};

function moneyLabel(currency: string, amountMinor: number): string {
  return `${currency.toUpperCase()} ${(amountMinor / 100).toFixed(2)}`;
}

function PaymentOverview({ payment, onInvoice, onRefund, compact = false }: { payment: PaymentUiState; onInvoice: () => void; onRefund: (orderId: string) => void; compact?: boolean }) {
  const balances = Object.entries(payment.account?.balancesMinor ?? {});
  const invoiceById = new Map(payment.invoices.map((invoice) => [invoice.id, invoice]));
  const orders = [...payment.orders].sort((left, right) => right.updatedAtMs - left.updatedAtMs);
  return <div className={compact ? extra.paymentCompact : extra.paymentOverview}>
    <div className={extra.walletCard}><WalletCards size={24} /><div><strong>Fabushi Wallet</strong><small>{balances.length ? balances.map(([currency, amount]) => moneyLabel(currency, amount)).join(' · ') : '暂无余额'}</small></div></div>
    <button type="button" className={styles.primaryButton} onClick={onInvoice}><ShoppingBag size={17} />创建账单</button>
    {!compact ? <>
      <section className={extra.paymentSection}><strong>订单</strong>{orders.length ? orders.map((order) => {
        const invoice = invoiceById.get(order.invoiceId);
        const sellerOwned = invoice?.sellerId === payment.actorId;
        return <div className={extra.paymentRow} key={order.id}><div><strong>{invoice?.title ?? order.invoiceId}</strong><small>{moneyLabel(order.amount.currency, order.amount.amountMinor)} · {order.status}</small></div>{sellerOwned && order.status === 'paid' ? <button type="button" onClick={() => onRefund(order.id)}>退款</button> : null}</div>;
      }) : <small>暂无订单</small>}</section>
      <section className={extra.paymentSection}><strong>最近流水</strong>{payment.entries.length ? payment.entries.slice(0, 12).map((entry) => <div className={extra.paymentRow} key={entry.id}><div><strong>{entry.kind}</strong><small>{entry.reference ?? entry.id}</small></div><span>{moneyLabel(entry.amount.currency, entry.amount.amountMinor)}</span></div>) : <small>暂无流水</small>}</section>
    </> : null}
  </div>;
}

type MiniAppMarketplaceProps = {
  miniApps: MarketplacePluginSummary[];
  installedMiniApps: Record<string, InstalledPluginPointer>;
  miniAppQuery: string;
  onMiniAppQuery: (query: string) => void;
  miniAppLoading: boolean;
  miniAppBusy: Set<string>;
  onOpenMiniApp: (id: string) => Promise<void>;
  onInstallMiniApp: (app: MarketplacePluginSummary) => Promise<void>;
  onUninstallMiniApp: (id: string) => Promise<void>;
};

function MiniAppMarketplaceSearch({ query, onChange }: { query: string; onChange: (query: string) => void }) {
  return <label className={styles.marketplaceSearch}><Search size={15} /><input value={query} onChange={(event) => onChange(event.target.value)} placeholder="搜索线上 Mini App" />{query ? <button type="button" onClick={() => onChange('')}><X size={13} /></button> : null}</label>;
}

function MiniAppMarketplaceList(props: MiniAppMarketplaceProps) {
  return <div className={styles.sectionList}>
    <MiniAppMarketplaceSearch query={props.miniAppQuery} onChange={props.onMiniAppQuery} />
    {props.miniAppLoading ? <div className={styles.marketplaceStatus}>正在搜索在线市场…</div> : null}
    {!props.miniAppLoading && !props.miniApps.length ? <div className={styles.marketplaceStatus}>没有找到可安装的 Mini App</div> : null}
    {props.miniApps.map((app) => {
      const installed = props.installedMiniApps[app.pluginId];
      const busy = props.miniAppBusy.has(app.pluginId);
      const update = Boolean(installed && installed.version !== app.latestVersion);
      return <div className={styles.marketplaceRow} key={app.pluginId} data-testid={`miniapp-market-${app.pluginId}`}>
        <span className={styles.appIcon}><AppWindow size={18} /></span>
        <div className={styles.marketplaceCopy}><strong>{app.displayName}</strong><small>{app.description}</small><em>{installed ? `已安装 ${installed.version}` : `在线 · ${app.latestVersion}`}</em></div>
        <div className={styles.marketplaceActions}>
          {installed ? <button type="button" disabled={busy} onClick={() => void props.onOpenMiniApp(app.pluginId)}>打开</button> : null}
          {!installed || update ? <button type="button" disabled={busy} onClick={() => void props.onInstallMiniApp(app)}>{busy ? '处理中' : update ? '更新' : '安装'}</button> : null}
          {installed ? <button type="button" disabled={busy} title="卸载" onClick={() => void props.onUninstallMiniApp(app.pluginId)}><Trash2 size={13} /></button> : null}
        </div>
      </div>;
    })}
  </div>;
}

function MiniAppMarketplaceWorkspace(props: MiniAppMarketplaceProps) {
  return <div className={styles.featureWorkspace}>
    <AppWindow size={54} />
    <h2>Mini Apps</h2>
    <p>所有官方与第三方 Mini App 都从线上市场搜索、验证并安装；Fabushi 主程序不预装应用。</p>
    <MiniAppMarketplaceSearch query={props.miniAppQuery} onChange={props.onMiniAppQuery} />
    {props.miniAppLoading ? <div className={styles.marketplaceStatus}>正在搜索在线市场…</div> : null}
    <div className={styles.featureGrid}>{props.miniApps.map((app) => {
      const installed = props.installedMiniApps[app.pluginId];
      const busy = props.miniAppBusy.has(app.pluginId);
      const update = Boolean(installed && installed.version !== app.latestVersion);
      return <article className={styles.marketplaceCard} key={app.pluginId}>
        <AppWindow size={24} />
        <strong>{app.displayName}</strong>
        <small>{app.description}</small>
        <em>{installed ? `已安装 ${installed.version}` : `线上版本 ${app.latestVersion}`}</em>
        <div>
          {installed ? <button type="button" disabled={busy} onClick={() => void props.onOpenMiniApp(app.pluginId)}>打开</button> : null}
          {!installed || update ? <button type="button" disabled={busy} onClick={() => void props.onInstallMiniApp(app)}>{busy ? '处理中…' : update ? '更新' : '安装'}</button> : null}
          {installed ? <button type="button" disabled={busy} onClick={() => void props.onUninstallMiniApp(app.pluginId)}>卸载</button> : null}
        </div>
      </article>;
    })}</div>
    {!props.miniAppLoading && !props.miniApps.length ? <div className={styles.marketplaceStatus}>没有找到可安装的 Mini App</div> : null}
  </div>;
}

function SectionPanel({ section, onInvoice, payment, onRefund, ...miniAppProps }: { section: MessengerSection; onInvoice: () => void; payment: PaymentUiState; onRefund: (orderId: string) => void } & MiniAppMarketplaceProps) {
  if (section === 'miniapps') return <MiniAppMarketplaceList {...miniAppProps} />;
  if (section === 'payments') return <div className={styles.sectionList}><PaymentOverview payment={payment} onInvoice={onInvoice} onRefund={onRefund} compact /></div>;
  if (section === 'folders') return <div className={styles.sectionList}><div className={styles.panelHint}><Folder size={24} /><strong>聊天文件夹</strong><p>按联系人、Bot、群组、频道、未读和静音状态组织。</p></div></div>;
  if (section === 'calls') return <div className={styles.sectionList}><div className={styles.panelHint}><Phone size={24} /><strong>最近通话</strong><p>从会话顶部发起语音/视频。</p></div></div>;
  return <div className={styles.sectionList}><div className={styles.panelHint}><Settings size={24} /><strong>{sectionTitle(section)}</strong><p>该功能入口已合并进统一 Messenger。</p></div></div>;
}

function FeatureWorkspace({ section, onInvoice, payment, onRefund, ...miniAppProps }: { section: MessengerSection; onInvoice: () => void; payment: PaymentUiState; onRefund: (orderId: string) => void } & MiniAppMarketplaceProps) {
  if (section === 'miniapps') return <MiniAppMarketplaceWorkspace {...miniAppProps} />;
  if (section === 'payments') return <div className={styles.featureWorkspace}><WalletCards size={54} /><h2>Fabushi Pay</h2><p>自建余额、Invoice、Order、退款与外部 settlement 都由 Rust 账本结算。</p><PaymentOverview payment={payment} onInvoice={onInvoice} onRefund={onRefund} /></div>;
  if (section === 'calls') return <div className={styles.featureWorkspace}><Phone size={54} /><h2>通话</h2><p>本机媒体已接通，Rust realtime 已具备一对一/群组通话信令状态。</p></div>;
  return <div className={styles.featureWorkspace}><MessageCircle size={54} /><h2>{sectionTitle(section)}</h2><p>联系人、Bot、群组和频道正在统一到同一个 Fabushi Actor/Conversation 模型。</p></div>;
}
