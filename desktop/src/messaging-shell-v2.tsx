import {
  AppWindow,
  Archive,
  BellOff,
  Bot,
  Bookmark,
  Check,
  CloudDownload,
  Copy,
  Edit3,
  FileText,
  Folder,
  Forward,
  Image,
  Link2,
  MapPin,
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
  UpdateState,
} from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { ElectronMahayanaHostTransport, isElectronMahayanaHostAvailable } from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import { MockMahayanaHostTransport } from '../../frontend/apps/web/src/lib/mahayana-host/mock-transport';
import { invokeNativeDesktop, subscribeNativeDesktopEvents } from '../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
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
type SearchCategory = 'chats' | 'channels' | 'apps' | 'posts' | 'images' | 'videos' | 'downloads' | 'links' | 'files' | 'music' | 'audio';
type SettingsCategory = 'account' | 'notifications' | 'privacy' | 'data' | 'chat' | 'folders' | 'devices' | 'calls' | 'language' | 'advanced' | 'fabushi';

type DesktopMessengerPreferences = {
  showInfoPanel: boolean;
  messagePreview: boolean;
  autoPlayMedia: boolean;
  enterToSend: boolean;
  reducedMotion: boolean;
};

type MessengerProjection = {
  version: 1;
  savedAtMs: number;
  actorId?: string;
  cursor?: string | null;
  activePeerKey?: string | null;
  selfActors: MessagingActor[];
  selfConversations: MessagingConversation[];
  selfMessages: Record<string, MessagingMessage[]>;
};

const searchCategories: ReadonlyArray<{ id: SearchCategory; label: string }> = [
  { id: 'chats', label: '聊天' },
  { id: 'channels', label: '频道' },
  { id: 'apps', label: '应用' },
  { id: 'posts', label: '贴文' },
  { id: 'images', label: '图片' },
  { id: 'videos', label: '视频' },
  { id: 'downloads', label: '下载' },
  { id: 'links', label: '链接' },
  { id: 'files', label: '文件' },
  { id: 'music', label: '音乐' },
  { id: 'audio', label: '声音' },
];

const messengerSettingsKey = 'fabushi.desktop.messenger-settings.v2';
const messengerDraftsKey = 'fabushi.desktop.messenger-drafts.v2';
const messengerSidebarWidthKey = 'fabushi.desktop.sidebar-width.v3';
const messengerProjectionKey = 'fabushi.desktop.messenger-projection.v1';
const messengerPreferencesKey = 'fabushi.desktop.telegram-settings.v1';
const initialPeerRenderCount = 120;
const initialMessageRenderCount = 240;
const initialSyncLimit = 20;
const backgroundSyncLimit = 100;
const projectionConversationLimit = 80;
const projectionMessageLimit = 80;

const defaultDesktopMessengerPreferences: DesktopMessengerPreferences = {
  showInfoPanel: true,
  messagePreview: true,
  autoPlayMedia: false,
  enterToSend: true,
  reducedMotion: false,
};

function readMessengerProjection(): MessengerProjection | null {
  if (typeof window === 'undefined') return null;
  try {
    const parsed = JSON.parse(window.localStorage.getItem(messengerProjectionKey) || 'null') as MessengerProjection | null;
    if (!parsed || parsed.version !== 1 || !Array.isArray(parsed.selfConversations) || !parsed.selfMessages) return null;
    return parsed;
  } catch {
    return null;
  }
}

function readDesktopMessengerPreferences(): DesktopMessengerPreferences {
  if (typeof window === 'undefined') return defaultDesktopMessengerPreferences;
  try {
    const stored = JSON.parse(window.localStorage.getItem(messengerPreferencesKey) || '{}') as Partial<DesktopMessengerPreferences>;
    return { ...defaultDesktopMessengerPreferences, ...stored };
  } catch {
    return defaultDesktopMessengerPreferences;
  }
}

function persistMessengerProjection(projection: MessengerProjection): void {
  try {
    window.localStorage.setItem(messengerProjectionKey, JSON.stringify(projection));
  } catch {
    // Fast-start projection is best effort; Rust SQLite remains authoritative.
  }
}

function createTransport(): MahayanaHostTransport {
  if (isElectronMahayanaHostAvailable()) return new ElectronMahayanaHostTransport();
  return new MockMahayanaHostTransport({ authenticated: true });
}

function isDesktopUpdateState(value: unknown): value is UpdateState {
  if (!value || typeof value !== 'object') return false;
  const type = (value as { type?: unknown }).type;
  return typeof type === 'string' && ['loading', 'disabled', 'checking', 'available', 'downloading', 'staging', 'ready', 'upToDate', 'error'].includes(type);
}

type ActionableDesktopUpdateState = Extract<UpdateState, { type: 'available' | 'downloading' | 'staging' | 'ready' }>;

function isActionableDesktopUpdateState(value: UpdateState | null): value is ActionableDesktopUpdateState {
  return Boolean(value && ['available', 'downloading', 'staging', 'ready'].includes(value.type));
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
  const [hasReturningProjection] = useState(() => Boolean(readMessengerProjection()));
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

  const showMessenger = authenticated === true || (authenticated === null && hasReturningProjection);
  return (
    <div className={styles.desktopRoot} data-testid="desktop-shell" data-local-first={showMessenger && authenticated === null ? 'true' : undefined}>
      {showMessenger ? <MessengerWorkspace /> : <HostClient />}
    </div>
  );
}

function MessengerWorkspace() {
  const transport = useMemo(() => createTransport(), []);
  const startupProjection = useMemo(() => readMessengerProjection(), []);
  const selfHosted = useMemo(() => new SelfHostedMessagingClientV2(transport, { actorId: startupProjection?.actorId }), [transport, startupProjection]);
  const [hostReady, setHostReady] = useState(false);
  const [section, setSection] = useState<MessengerSection>('chats');
  const [conversations, setConversations] = useState<ConversationSummary[]>([]);
  const [bots, setBots] = useState<BotSummary[]>([]);
  const [groups, setGroups] = useState<GroupSummary[]>([]);
  const [selfActors, setSelfActors] = useState<MessagingActor[]>(startupProjection?.selfActors ?? []);
  const [selfConversations, setSelfConversations] = useState<MessagingConversation[]>(startupProjection?.selfConversations ?? []);
  const [selfMessages, setSelfMessages] = useState<Record<string, MessagingMessage[]>>(startupProjection?.selfMessages ?? {});
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
  const [activePeerKey, setActivePeerKey] = useState<string | null>(startupProjection?.activePeerKey ?? null);
  const [messages, setMessages] = useState<DisplayMessage[]>([]);
  const [composer, setComposer] = useState('');
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [replyTo, setReplyTo] = useState<DisplayMessage | null>(null);
  const [silentSend, setSilentSend] = useState(false);
  const [scheduledAtMs, setScheduledAtMs] = useState<number | undefined>();
  const [search, setSearch] = useState('');
  const [globalSearchOpen, setGlobalSearchOpen] = useState(false);
  const [globalSearchCategory, setGlobalSearchCategory] = useState<SearchCategory>('chats');
  const [profileMenuOpen, setProfileMenuOpen] = useState(false);
  const [createMenuOpen, setCreateMenuOpen] = useState(false);
  const [sidebarWidth, setSidebarWidth] = useState(330);
  const [conversationSearchOpen, setConversationSearchOpen] = useState(false);
  const [desktopUpdateState, setDesktopUpdateState] = useState<UpdateState | null>(null);
  const [desktopUpdateBusy, setDesktopUpdateBusy] = useState(false);
  const [peerRenderCount, setPeerRenderCount] = useState(initialPeerRenderCount);
  const [messageRenderCount, setMessageRenderCount] = useState(initialMessageRenderCount);
  const [desktopPreferences, setDesktopPreferences] = useState<DesktopMessengerPreferences>(() => readDesktopMessengerPreferences());
  const [settingsCategory, setSettingsCategory] = useState<SettingsCategory>('account');
  const [infoOpen, setInfoOpen] = useState(() => readDesktopMessengerPreferences().showInfoPanel);
  const [wideInfoLayout, setWideInfoLayout] = useState(() => typeof window === 'undefined' ? true : window.innerWidth > 1280);
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
  const messagingCursorRef = useRef<string | null>(startupProjection?.cursor ?? null);
  const syncInFlightRef = useRef(false);
  const typingStopTimerRef = useRef<number | null>(null);
  const peersRef = useRef<PeerItem[]>([]);
  const webRtcRef = useRef<FabushiWebRtcController | null>(null);
  const localVideoRef = useRef<HTMLVideoElement>(null);
  const remoteVideoRef = useRef<HTMLVideoElement>(null);
  const remoteAudioRef = useRef<HTMLAudioElement>(null);
  const mediaInputRef = useRef<HTMLInputElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    activePeerKeyRef.current = activePeerKey;
  }, [activePeerKey]);

  useEffect(() => {
    const onResize = () => setWideInfoLayout(window.innerWidth > 1280);
    onResize();
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);

  useEffect(() => {
    try {
      window.localStorage.setItem(messengerPreferencesKey, JSON.stringify(desktopPreferences));
    } catch {
      // Desktop-only preferences are best effort.
    }
  }, [desktopPreferences]);

  useEffect(() => {
    if (!startupProjection?.activePeerKey?.startsWith('selfhosted:')) return;
    const conversationId = startupProjection.activePeerKey.slice('selfhosted:'.length);
    const cached = (startupProjection.selfMessages[conversationId] ?? []).filter((message) => !message.deleted);
    setMessages(cached.map(displaySelfMessage));
  }, [startupProjection]);

  useEffect(() => {
    if (!selfConversations.length && !selfActors.length) return;
    const timer = window.setTimeout(() => {
      const conversationIds = new Set(
        [...selfConversations]
          .sort((left, right) => right.updatedAtMs - left.updatedAtMs)
          .slice(0, projectionConversationLimit)
          .map((conversation) => conversation.id),
      );
      const boundedMessages = Object.fromEntries(
        Object.entries(selfMessages)
          .filter(([conversationId]) => conversationIds.has(conversationId))
          .map(([conversationId, list]) => [conversationId, list.slice(-projectionMessageLimit)]),
      );
      persistMessengerProjection({
        version: 1,
        savedAtMs: Date.now(),
        actorId: selfHosted.actorId,
        cursor: messagingCursorRef.current,
        activePeerKey,
        selfActors,
        selfConversations: [...selfConversations]
          .sort((left, right) => right.updatedAtMs - left.updatedAtMs)
          .slice(0, projectionConversationLimit),
        selfMessages: boundedMessages,
      });
    }, 180);
    return () => window.clearTimeout(timer);
  }, [activePeerKey, selfActors, selfConversations, selfMessages, selfHosted.actorId]);

  function updateDesktopPreference<K extends keyof DesktopMessengerPreferences>(key: K, value: DesktopMessengerPreferences[K]) {
    setDesktopPreferences((current) => ({ ...current, [key]: value }));
    if (key === 'showInfoPanel') setInfoOpen(Boolean(value));
  }

  useEffect(() => {
    const stored = Number(window.localStorage.getItem(messengerSidebarWidthKey));
    if (Number.isFinite(stored) && stored >= 84 && stored <= 460) setSidebarWidth(stored);
  }, []);

  useEffect(() => {
    try { window.localStorage.setItem(messengerSidebarWidthKey, String(Math.round(sidebarWidth))); } catch { /* best effort */ }
  }, [sidebarWidth]);

  useEffect(() => {
    let disposed = false;
    const acceptUpdateState = (payload: unknown) => {
      if (disposed || !isDesktopUpdateState(payload)) return;
      setDesktopUpdateState(payload);
      if (payload.type !== 'ready') setDesktopUpdateBusy(false);
    };
    const unsubscribe = subscribeNativeDesktopEvents({ 'update-status': acceptUpdateState });
    void invokeNativeDesktop<UpdateState>('getUpdateStatus').then(acceptUpdateState).catch(() => {});
    const timer = window.setTimeout(() => {
      void invokeNativeDesktop<UpdateState>('checkForUpdates').then(acceptUpdateState).catch(() => {});
    }, 1_500);
    return () => {
      disposed = true;
      window.clearTimeout(timer);
      unsubscribe();
    };
  }, []);

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
    setSearch('');
    setGlobalSearchOpen(false);
    setConversationSearchOpen(false);
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
          await selfHosted.sync(initialSyncLimit, messagingCursorRef.current);
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
    if (!hostReady || !globalSearchOpen || globalSearchCategory !== 'apps') return;
    const timer = window.setTimeout(() => { void refreshMiniApps(search); }, 250);
    return () => window.clearTimeout(timer);
  }, [hostReady, globalSearchOpen, globalSearchCategory, search]);

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
      void selfHosted.sync(backgroundSyncLimit, messagingCursorRef.current)
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
    const previousCursor = messagingCursorRef.current;
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
        const grouped: Record<string, MessagingMessage[]> = {};
        for (const message of payload.messages ?? []) {
          (grouped[message.conversationId] ??= []).push(message);
        }
        for (const list of Object.values(grouped)) list.sort((a, b) => a.createdAtMs - b.createdAtMs);
        setSelfActors((current) => (payload.actors ?? []).reduce((items, actor) => upsertById(items, actor), previousCursor ? current : []));
        setSelfConversations((current) => (payload.conversations ?? []).reduce((items, conversation) => upsertById(items, conversation), previousCursor ? current : []));
        setSelfMessages((current) => {
          const next = previousCursor ? { ...current } : {};
          for (const [conversationId, incoming] of Object.entries(grouped)) {
            next[conversationId] = incoming.reduce((items, message) => upsertById(items, message), next[conversationId] ?? [])
              .sort((a, b) => a.createdAtMs - b.createdAtMs);
          }
          const active = activePeerKeyRef.current;
          if (active?.startsWith('selfhosted:')) {
            const conversationId = active.slice('selfhosted:'.length);
            setMessages((next[conversationId] ?? []).filter((message) => !message.deleted).map(displaySelfMessage));
          }
          return next;
        });
        setSelfInvoices((current) => (payload.invoices ?? []).reduce((items, item) => upsertById(items, item), previousCursor ? current : []));
        setSelfOrders((current) => (payload.orders ?? []).reduce((items, item) => upsertById(items, item), previousCursor ? current : []));
        setSelfStories((current) => (payload.stories ?? []).reduce((items, item) => upsertById(items, item), previousCursor ? current : []));
        setSelfCommunities((current) => (payload.communities ?? []).reduce((items, item) => [...items.filter((existing) => existing.conversationId !== item.conversationId), item], previousCursor ? current : []));
        setSelfBotProfiles((current) => (payload.bots ?? []).reduce((items, item) => [...items.filter((existing) => existing.actorId !== item.actorId), item], previousCursor ? current : []));
        setSelfBotExecutions((current) => (payload.botExecutions ?? []).reduce((items, item) => upsertById(items, item), previousCursor ? current : []));
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
        if (event.html) {
          const title = marketplaceApps.find((app) => app.pluginId === event.miniAppId)?.displayName ?? event.miniAppId;
          setMiniApp({ id: event.miniAppId, title, html: event.html });
        }
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
    .filter((story) => story.ownerId !== selfHosted.actorId && (story.pinnedToProfile || story.expiresAtMs > Date.now()))
    .sort((left, right) => right.createdAtMs - left.createdAtMs), [selfStories, selfHosted]);

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
  const infoPanelVisible = Boolean(infoOpen && wideInfoLayout && activePeer && sectionIsPeerList);
  const currentActor = selfActors.find((actor) => actor.id === selfHosted.actorId);
  const renderedPeers = visiblePeers.slice(0, peerRenderCount);
  const matchingMessages = messages;
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
    setSearch('');
    setGlobalSearchOpen(false);
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


  async function installDesktopUpdate(state: UpdateState) {
    if (!['available', 'downloading', 'staging', 'ready'].includes(state.type)) return;
    setDesktopUpdateBusy(true);
    try {
      await invokeNativeDesktop('quitAndInstallUpdate', {
        expectedVersion: 'version' in state ? state.version : undefined,
      });
    } catch (cause) {
      setDesktopUpdateBusy(false);
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  function startSidebarResize(event: React.PointerEvent<HTMLDivElement>) {
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = sidebarWidth;
    const onMove = (moveEvent: PointerEvent) => {
      const next = Math.max(84, Math.min(460, startWidth + moveEvent.clientX - startX));
      setSidebarWidth(next);
    };
    const onUp = (upEvent: PointerEvent) => {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
      const finalWidth = Math.max(84, Math.min(460, startWidth + upEvent.clientX - startX));
      setSidebarWidth(finalWidth < 145 ? 88 : finalWidth);
    };
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  }

  function navigateFromProfile(next: MessengerSection) {
    setProfileMenuOpen(false);
    setGlobalSearchOpen(false);
    if (next === 'saved') {
      void ensureSavedMessages();
      return;
    }
    setSection(next);
  }

  return (
    <main
      className={`${styles.messenger} ${styles.fabushiUnified}`}
      data-testid="messenger-workspace"
      data-sidebar-collapsed={sidebarWidth <= 112 || undefined}
      data-reduce-motion={desktopPreferences.reducedMotion || undefined}
      data-testid-ready-projection={startupProjection ? 'true' : undefined}
      style={{ gridTemplateColumns: infoPanelVisible ? `${sidebarWidth}px minmax(420px,1fr) 286px` : `${sidebarWidth}px minmax(420px,1fr)` }}
      onClick={() => { setMessageMenu(null); setProfileMenuOpen(false); setCreateMenuOpen(false); }}
    >
      <aside className={styles.chatList} data-testid="messenger-sidebar" data-collapsed={sidebarWidth <= 112 || undefined} onClick={(event) => event.stopPropagation()}>
        <header className={styles.listHeader}>
          <span className={styles.sidebarBrand} title="Fabushi"><BotMark botId="fabushi:brand" state="idle" size={30} label="Fabushi" /></span>
          <strong>{sectionTitle(section)}</strong>
          <div className={styles.createMenuWrap}>
            <button type="button" className={styles.iconButton} aria-label="新建" data-active={createMenuOpen} onClick={(event) => { event.stopPropagation(); setCreateMenuOpen((value) => !value); }}><SquarePen size={18} /></button>
            {createMenuOpen ? <div className={styles.createMenu} onClick={(event) => event.stopPropagation()}>
              <button type="button" onClick={() => { setCreateMenuOpen(false); setNewDialog({ type: 'group', name: '', selectedBotIds: new Set() }); }}><Users size={16} /><span>新建群组</span></button>
              <button type="button" onClick={() => { setCreateMenuOpen(false); setNewDialog({ type: 'channel', name: '', description: '' }); }}><Radio size={16} /><span>新建频道</span></button>
            </div> : null}
          </div>
        </header>
        <label className={styles.searchBox} data-testid="global-search-trigger" onClick={() => { if (sidebarWidth <= 112) setSidebarWidth(330); setGlobalSearchOpen(true); }}>
          <Search size={16} />
          <input
            ref={searchInputRef}
            data-testid="global-search-input"
            value={search}
            onFocus={() => { if (sidebarWidth <= 112) setSidebarWidth(330); setGlobalSearchOpen(true); }}
            onChange={(event) => setSearch(event.target.value)}
            placeholder={conversationSearchOpen && activePeer ? `搜索 ${activePeer.title}` : '搜索'}
          />
          {search ? <button type="button" aria-label="清除搜索" onClick={(event) => { event.stopPropagation(); setSearch(''); searchInputRef.current?.focus(); }}><X size={14} /></button> : globalSearchOpen && !conversationSearchOpen ? <button type="button" aria-label="关闭搜索" onClick={(event) => { event.preventDefault(); event.stopPropagation(); setGlobalSearchOpen(false); setGlobalSearchCategory('chats'); }}><X size={14} /></button> : null}
        </label>
        {conversationSearchOpen && activePeer ? <div className={styles.searchScope} data-testid="conversation-search-scope">
          <span>此聊天</span><strong>{activePeer.title}</strong>
          <button type="button" aria-label="退出当前会话搜索" onClick={() => { setConversationSearchOpen(false); setGlobalSearchCategory('chats'); setSearch(''); searchInputRef.current?.focus(); }}><X size={13} /></button>
        </div> : null}
        {['chats', 'contacts'].includes(section) && sidebarWidth > 112 && visibleStories.length ? <div className={extra.storyStrip}>
          {visibleStories.map((story) => {
            const actor = selfActors.find((item) => item.id === story.ownerId);
            const name = actor?.displayName ?? (story.ownerId === selfHosted.actorId ? '我' : story.ownerId);
            return <button type="button" className={extra.storyItem} key={story.id} onClick={() => void openStory(story)} title={name}>
              <span className={extra.storyRing}><BotMark botId={`story:${story.ownerId}`} state="idle" size={40} label={name} /></span><small>{name}</small>
            </button>;
          })}
        </div> : null}
        {globalSearchOpen ? (
          <GlobalSearchWorkspace
            query={search}
            category={globalSearchCategory}
            onCategory={setGlobalSearchCategory}
            scopePeer={conversationSearchOpen ? activePeer : null}
            peers={peers}
            messages={messages}
            miniApps={marketplaceApps}
            installedMiniApps={installedMiniApps}
            miniAppBusy={miniAppBusy}
            miniAppLoading={miniAppLoading}
            onOpenPeer={(peer) => { setGlobalSearchOpen(false); setConversationSearchOpen(false); setSearch(''); setSection(peer.kind === 'channel' ? 'channels' : 'chats'); void openPeer(peer); }}
            onOpenMiniApp={openMiniApp}
            onInstallMiniApp={installMiniApp}
            onUninstallMiniApp={uninstallMiniApp}
          />
        ) : sectionIsPeerList ? (
          <div className={styles.peerList}>
            {renderedPeers.map((peer) => (
              <button data-testid={`peer-${peer.key}`} key={peer.key} type="button" className={peer.key === activePeerKey ? styles.peerActive : styles.peer} onClick={() => void openPeer(peer)}>
                <BotMark
                  botId={`peer:${peer.kind}:${peer.actorId ?? peer.id}`}
                  state={isAgentPeer(peer) ? botMarkStateForPeer(peer, selfBotExecutions, peer.key === activePeerKey && pendingSend, hostReady) : peer.unread ? 'notifying' : 'idle'}
                  size={48}
                  className={styles.agentAvatarMark}
                  label={peer.title}
                />
                <span className={styles.peerCopy}>
                  <span><strong>{peer.title}</strong><time>{formatTime(peer.updatedAtMs)}</time></span>
                  <small>{desktopPreferences.messagePreview ? peer.subtitle : peer.unread ? '有新消息' : '消息预览已关闭'}</small>
                </span>
                <span className={styles.peerMeta}>{peer.pinned ? <Pin size={12} /> : null}{mutedPeerKeys.has(peer.key) ? <BellOff size={12} /> : null}{peer.unread ? <b>{peer.unread}</b> : null}</span>
              </button>
            ))}
            {visiblePeers.length > renderedPeers.length ? <button type="button" data-testid="peer-list-load-more" onClick={() => setPeerRenderCount((count) => count + initialPeerRenderCount)}>显示更多会话</button> : null}
            {!visiblePeers.length ? <EmptyList section={section} /> : null}
          </div>
        ) : (
          <SectionPanel section={section} onOpenMiniApp={openMiniApp} onInstallMiniApp={installMiniApp} onUninstallMiniApp={uninstallMiniApp} miniApps={marketplaceApps} installedMiniApps={installedMiniApps} miniAppQuery={miniAppQuery} onMiniAppQuery={setMiniAppQuery} miniAppLoading={miniAppLoading} miniAppBusy={miniAppBusy} onInvoice={() => void createInvoiceForActivePeer()} payment={{ account: walletAccount, entries: walletEntries, orders: selfOrders, invoices: selfInvoices, actorId: selfHosted.actorId }} onRefund={(orderId) => void refundOrder(orderId)} settings={{ category: settingsCategory, onCategory: setSettingsCategory }} />
        )}
        <div className={styles.sidebarFooter}>
          {profileMenuOpen ? <ProfileNavigationMenu section={section} onNavigate={navigateFromProfile} /> : null}
          {isActionableDesktopUpdateState(desktopUpdateState) ? <button
            type="button"
            className={styles.updateCloudButton}
            data-testid="desktop-update-cloud"
            data-state={desktopUpdateState.type}
            disabled={desktopUpdateBusy}
            title={desktopUpdateState.type === 'ready' ? '更新已下载，点击安装并重启' : '发现新版本，点击下载、替换并重启'}
            onClick={() => void installDesktopUpdate(desktopUpdateState)}
          >
            <CloudDownload size={18} />
            <span>{desktopUpdateState.type === 'ready' ? '安装更新' : desktopUpdateState.type === 'downloading' || desktopUpdateState.type === 'staging' ? '正在更新' : `更新 ${desktopUpdateState.version}`}</span>
          </button> : null}
          <button
            type="button"
            className={styles.profileNavigationTrigger}
            data-testid="profile-navigation-trigger"
            title="个人与导航"
            onClick={(event) => { event.stopPropagation(); setProfileMenuOpen((value) => !value); }}
          >
            <BotMark botId={`self:${selfHosted.actorId || 'local'}`} state="idle" size={38} label="我的头像" />
            <span><strong>我</strong><small>导航与设置</small></span>
          </button>
        </div>
        <div
          className={styles.sidebarResizer}
          data-testid="sidebar-resizer"
          role="separator"
          aria-orientation="vertical"
          aria-label="调整会话栏宽度"
          onPointerDown={startSidebarResize}
          onDoubleClick={() => setSidebarWidth((width) => width <= 112 ? 330 : 88)}
        />
      </aside>

      <section className={styles.chatWorkspace}>
        {activePeer && sectionIsPeerList ? (
          <>
            <header className={styles.chatHeader}>
              <div className={styles.chatIdentity}>
                <BotMark
                  botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`}
                  state={isAgentPeer(activePeer) ? botMarkStateForPeer(activePeer, selfBotExecutions, pendingSend, hostReady) : 'idle'}
                  size={40}
                  className={styles.agentAvatarMark}
                  label={activePeer.title}
                />
                <div><strong>{activePeer.title}</strong><small data-testid="conversation-status">{activeTypingActors.length ? '正在输入…' : `${activePeer.subtitle}${hostReady ? ' · 在线' : ' · 正在连接'}`}</small></div>
              </div>
              <div className={styles.headerActions}>
                <button type="button" title="语音通话" onClick={() => void startCall('voice')}><PhoneCall size={18} /></button>
                <button type="button" title="视频通话" onClick={() => void startCall('video')}><Video size={18} /></button>
                {activePeer.source === 'selfhosted' ? <button type="button" title="发送账单" onClick={() => void createInvoiceForActivePeer()}><WalletCards size={18} /></button> : null}
                <button type="button" title="搜索当前会话" data-active={conversationSearchOpen} onClick={() => {
                  const next = !conversationSearchOpen;
                  setConversationSearchOpen(next);
                  setGlobalSearchOpen(next);
                  setGlobalSearchCategory(next ? 'posts' : 'chats');
                  setSearch('');
                  window.setTimeout(() => searchInputRef.current?.focus(), 0);
                }}><Search size={18} /></button>
                <button type="button" title={activePeer.pinned ? '取消置顶' : '置顶'} onClick={() => void togglePinConversation(activePeer)}><Pin size={18} /></button>
                <button type="button" title={mutedPeerKeys.has(activePeer.key) ? '开启通知' : '静音'} onClick={() => void toggleMuteConversation(activePeer)}><BellOff size={18} /></button>
                <button type="button" title="资料" data-active={infoOpen} onClick={() => setInfoOpen((value) => !value)}><MoreVertical size={18} /></button>
              </div>
            </header>
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
                  {message.mediaType === 'video' && blobMediaUrl(message.media) ? <video className={extra.messageMedia} controls playsInline autoPlay={desktopPreferences.autoPlayMedia} src={blobMediaUrl(message.media)} /> : null}
                  {message.mediaType === 'document' && blobMediaUrl(message.media) ? <a className={extra.messageFile} href={blobMediaUrl(message.media)} download={message.media?.fileName}><FileText size={17} />{message.media?.fileName ?? '文件'}</a> : null}
                  <p>{message.text}</p>
                  {message.reactions?.length ? <div className={extra.reactions}>{message.reactions.map((reaction) => <span key={reaction}>{reaction}</span>)}</div> : null}
                  <small>{formatTime(message.createdAtMs)} {message.role === 'me' ? <Check size={12} /> : null}</small>
                </article>
              ))}
              {!matchingMessages.length ? <div className={styles.chatEmpty} data-testid="message-search-empty"><BotMark botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state={isAgentPeer(activePeer) ? botMarkStateForPeer(activePeer, selfBotExecutions, false, hostReady) : 'idle'} size={78} className={styles.agentAvatarMark} label={activePeer.title} /><strong>{activePeer.title}</strong><p>联系人、AI Bot、群组和频道使用同一个 Fabushi 消息产品层。</p></div> : null}
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
              <textarea data-testid="messenger-input" value={composer} onChange={(event) => updateComposer(event.target.value)} onKeyDown={(event) => {
                const submitWithEnter = desktopPreferences.enterToSend && event.key === 'Enter' && !event.shiftKey;
                const submitWithShortcut = !desktopPreferences.enterToSend && event.key === 'Enter' && (event.metaKey || event.ctrlKey);
                if (submitWithEnter || submitWithShortcut) { event.preventDefault(); event.currentTarget.form?.requestSubmit(); }
              }} placeholder="消息" rows={1} />
              <button type="button" title="表情"><Smile size={20} /></button>
              <button type="button" data-active={silentSend} title={silentSend ? '关闭静默发送' : '静默发送'} onClick={() => setSilentSend((value) => !value)}><BellOff size={19} /></button>
              {composer.trim() ? <button data-testid="messenger-send" className={styles.sendButton} type="submit" disabled={!hostReady || pendingSend}><Send size={19} /></button> : <button type="button" title="语音消息"><Mic size={20} /></button>}
            </form>
          </>
        ) : (
          <FeatureWorkspace section={section} onOpenMiniApp={openMiniApp} onInstallMiniApp={installMiniApp} onUninstallMiniApp={uninstallMiniApp} miniApps={marketplaceApps} installedMiniApps={installedMiniApps} miniAppQuery={miniAppQuery} onMiniAppQuery={setMiniAppQuery} miniAppLoading={miniAppLoading} miniAppBusy={miniAppBusy} onInvoice={() => void createInvoiceForActivePeer()} payment={{ account: walletAccount, entries: walletEntries, orders: selfOrders, invoices: selfInvoices, actorId: selfHosted.actorId }} onRefund={(orderId) => void refundOrder(orderId)} settings={{ category: settingsCategory, onCategory: setSettingsCategory, preferences: desktopPreferences, onPreference: updateDesktopPreference, actor: currentActor, actorId: selfHosted.actorId }} />
        )}
      </section>

      {infoPanelVisible && activePeer ? (
        <aside className={styles.infoPanel}>
          <header><strong>资料</strong><button type="button" onClick={() => setInfoOpen(false)}><X size={17} /></button></header>
          <div className={styles.profileCard}>
            <BotMark botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state={isAgentPeer(activePeer) ? botMarkStateForPeer(activePeer, selfBotExecutions, pendingSend, hostReady) : 'idle'} size={92} className={styles.agentProfileMark} label={activePeer.title} />
            <strong>{activePeer.title}</strong><small>{activePeer.subtitle}</small>
            <div><button type="button" onClick={() => void startCall('voice')}><PhoneCall size={18} /><span>通话</span></button><button type="button" onClick={() => void startCall('video')}><Video size={18} /><span>视频</span></button><button type="button" onClick={() => { setConversationSearchOpen(true); setGlobalSearchOpen(true); setGlobalSearchCategory('posts'); setSearch(''); window.setTimeout(() => searchInputRef.current?.focus(), 0); }}><Search size={18} /><span>搜索</span></button></div>
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

function ProfileNavigationMenu({ section, onNavigate }: { section: MessengerSection; onNavigate: (section: MessengerSection) => void }) {
  const items: Array<{ section: MessengerSection; label: string; icon: React.ReactNode }> = [
    { section: 'chats', label: '聊天', icon: <MessageCircle size={17} /> },
    { section: 'contacts', label: '联系人', icon: <Users size={17} /> },
    { section: 'bots', label: 'Bots', icon: <Bot size={17} /> },
    { section: 'groups', label: '群组', icon: <Users size={17} /> },
    { section: 'channels', label: '频道', icon: <Radio size={17} /> },
    { section: 'calls', label: '通话', icon: <Phone size={17} /> },
    { section: 'saved', label: '收藏', icon: <Bookmark size={17} /> },
    { section: 'archive', label: '归档', icon: <Archive size={17} /> },
    { section: 'folders', label: '文件夹', icon: <Folder size={17} /> },
    { section: 'miniapps', label: 'Mini Apps', icon: <AppWindow size={17} /> },
    { section: 'payments', label: '支付', icon: <WalletCards size={17} /> },
    { section: 'settings', label: '设置', icon: <Settings size={17} /> },
  ];
  return <div className={styles.profileNavigationMenu} data-testid="profile-navigation-menu" onClick={(event) => event.stopPropagation()}>
    <header><BotMark botId="fabushi:navigation" state="idle" size={34} label="Fabushi" /><div><strong>Fabushi</strong><small>统一导航</small></div></header>
    <div>{items.map((item) => <button key={item.section} type="button" title={item.label} data-active={section === item.section} onClick={() => onNavigate(item.section)}>{item.icon}<span>{item.label}</span></button>)}</div>
  </div>;
}

type GlobalSearchWorkspaceProps = {
  query: string;
  category: SearchCategory;
  onCategory: (value: SearchCategory) => void;
  scopePeer?: PeerItem | null;
  peers: PeerItem[];
  messages: DisplayMessage[];
  miniApps: MarketplacePluginSummary[];
  installedMiniApps: Record<string, InstalledPluginPointer>;
  miniAppBusy: Set<string>;
  miniAppLoading: boolean;
  onOpenPeer: (peer: PeerItem) => void;
  onOpenMiniApp: (id: string) => Promise<void>;
  onInstallMiniApp: (app: MarketplacePluginSummary) => Promise<void>;
  onUninstallMiniApp: (id: string) => Promise<void>;
};

function GlobalSearchWorkspace(props: GlobalSearchWorkspaceProps) {
  const normalized = props.query.trim().toLocaleLowerCase();
  const matches = (value: string) => !normalized || value.toLocaleLowerCase().includes(normalized);
  const peerResults = props.peers.filter((peer) => matches(`${peer.title} ${peer.subtitle}`));
  const messageResults = props.messages.filter((message) => matches(message.text));
  const mediaResults = messageResults.filter((message) => {
    if (props.category === 'images') return message.mediaType === 'photo';
    if (props.category === 'videos') return message.mediaType === 'video';
    if (props.category === 'files' || props.category === 'downloads') return message.mediaType === 'document';
    if (props.category === 'links') return /https?:\/\//iu.test(message.text);
    return props.category === 'posts';
  });
  const appResults = props.miniApps.filter((app) => matches(`${app.displayName} ${app.description} ${app.pluginId}`));
  const unsupportedMediaCategory = props.category === 'music' || props.category === 'audio';
  const categories = props.scopePeer
    ? searchCategories.filter((item) => ['posts', 'images', 'videos', 'downloads', 'links', 'files', 'music', 'audio'].includes(item.id))
    : searchCategories;

  return <div className={styles.globalSearch} data-testid="global-search-surface" data-scoped={props.scopePeer ? 'true' : undefined}>
    <nav className={styles.globalSearchTabs} aria-label={props.scopePeer ? '当前会话搜索分类' : '搜索分类'}>
      {categories.map((item) => <button key={item.id} type="button" data-testid={`global-search-tab-${item.id}`} data-active={props.category === item.id} onClick={() => props.onCategory(item.id)}>{props.scopePeer && item.id === 'posts' ? '消息' : item.label}</button>)}
    </nav>
    <div className={styles.globalSearchResults}>
      {!props.scopePeer && props.category === 'chats' ? peerResults.filter((peer) => peer.kind !== 'channel').map((peer) => <button key={peer.key} type="button" className={styles.searchResultRow} onClick={() => props.onOpenPeer(peer)}><BotMark botId={`peer:${peer.kind}:${peer.actorId ?? peer.id}`} state="idle" size={46} label={peer.title} /><span><strong>{peer.title}</strong><small>{peer.subtitle}</small></span><time>{formatTime(peer.updatedAtMs)}</time></button>) : null}
      {!props.scopePeer && props.category === 'channels' ? peerResults.filter((peer) => peer.kind === 'channel').map((peer) => <button key={peer.key} type="button" className={styles.searchResultRow} onClick={() => props.onOpenPeer(peer)}><BotMark botId={`peer:channel:${peer.id}`} state="idle" size={46} label={peer.title} /><span><strong>{peer.title}</strong><small>{peer.subtitle}</small></span><time>{formatTime(peer.updatedAtMs)}</time></button>) : null}
      {!props.scopePeer && props.category === 'apps' ? <div className={styles.searchAppResults}>{props.miniAppLoading ? <div className={styles.marketplaceStatus}>正在搜索在线应用市场…</div> : appResults.map((app) => {
        const installed = props.installedMiniApps[app.pluginId];
        const busy = props.miniAppBusy.has(app.pluginId);
        const update = Boolean(installed && installed.version !== app.latestVersion);
        return <article key={app.pluginId} className={styles.searchAppCard} data-testid={`global-search-app-${app.pluginId}`}><BotMark botId={`miniapp:${app.pluginId}`} state={installed ? 'idle' : 'sleeping'} size={52} label={app.displayName} /><div><strong>{app.displayName}</strong><small>{app.description}</small><em>{installed ? `已安装 ${installed.version}` : `在线 · ${app.latestVersion}`}</em></div><aside>{installed ? <button type="button" disabled={busy} onClick={() => void props.onOpenMiniApp(app.pluginId)}>打开</button> : null}{!installed || update ? <button type="button" disabled={busy} onClick={() => void props.onInstallMiniApp(app)}>{busy ? '处理中' : update ? '更新' : '安装'}</button> : null}{installed ? <button type="button" disabled={busy} onClick={() => void props.onUninstallMiniApp(app.pluginId)}>卸载</button> : null}</aside></article>;
      })}</div> : null}
      {['posts', 'images', 'videos', 'downloads', 'links', 'files'].includes(props.category) ? mediaResults.map((message) => <article key={message.id} className={styles.searchMessageResult}><BotMark botId={`search-message:${message.id}`} state="idle" size={38} label="消息" /><div><p>{message.text || (message.mediaType === 'photo' ? '图片' : message.mediaType === 'video' ? '视频' : '文件')}</p><small>{new Date(message.createdAtMs).toLocaleString()}</small></div></article>) : null}
      {unsupportedMediaCategory ? <SearchEmptyState label="当前会话尚无可搜索的音频索引" /> : null}
      {!props.scopePeer && props.category === 'chats' && !peerResults.filter((peer) => peer.kind !== 'channel').length ? <SearchEmptyState label={normalized ? '没有匹配的聊天' : '最近搜索结果将显示在此处'} /> : null}
      {!props.scopePeer && props.category === 'channels' && !peerResults.filter((peer) => peer.kind === 'channel').length ? <SearchEmptyState label={normalized ? '没有匹配的频道' : '最近搜索结果将显示在此处'} /> : null}
      {!props.scopePeer && props.category === 'apps' && !props.miniAppLoading && !appResults.length ? <SearchEmptyState label="没有匹配的在线应用" /> : null}
      {['posts', 'images', 'videos', 'downloads', 'links', 'files'].includes(props.category) && !mediaResults.length ? <SearchEmptyState label={normalized ? '当前已加载内容中没有匹配结果' : '最近搜索结果将显示在此处'} /> : null}
    </div>
  </div>;
}

function SearchEmptyState({ label }: { label: string }) {
  return <div className={styles.globalSearchEmpty}><Search size={44} /><strong>{label}</strong><small>在左侧搜索框输入关键词，或切换分类继续搜索。</small></div>;
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
      {peers.length ? peers.map((peer) => <button key={peer.key} type="button" onClick={() => onSelect(peer)}><BotMark botId={`peer:${peer.kind}:${peer.actorId ?? peer.id}`} state="idle" size={40} label={peer.title} /><span><strong>{peer.title}</strong><small>{peer.subtitle}</small></span><Forward size={16} /></button>) : <p>暂无可转发的自建会话，请先创建群组、频道或收藏消息。</p>}
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
      })}><BotMark botId={`bot:${bot.id}`} state="idle" size={34} label={bot.name} /><div><strong>{bot.name}</strong><small>{bot.description}</small></div>{selected ? <Check size={16} /> : <Plus size={16} />}</button>;
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
      <header><BotMark botId={`story:${story.ownerId}`} state="idle" size={40} label={owner?.displayName ?? story.ownerId} /><div><strong>{owner?.displayName ?? story.ownerId}</strong><small>{new Date(story.createdAtMs).toLocaleString()}</small></div><button type="button" onClick={onClose}><X size={18} /></button></header>
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
    <header><BotMark botId={`call:${call.title}`} state={call.status === "active" ? "speaking" : "listening"} size={78} label={call.title} /><strong>{call.title}</strong><small>{statusText}</small></header>
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
        <span className={styles.appIcon}><BotMark botId={`miniapp:${app.pluginId}`} state={installed ? "idle" : "sleeping"} size={34} label={app.displayName} /></span>
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
        <BotMark botId={`miniapp:${app.pluginId}`} state={installed ? "idle" : "sleeping"} size={48} label={app.displayName} />
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

type SettingsNavigationProps = {
  category: SettingsCategory;
  onCategory: (category: SettingsCategory) => void;
};

type SettingsWorkspaceProps = SettingsNavigationProps & {
  preferences: DesktopMessengerPreferences;
  onPreference: <K extends keyof DesktopMessengerPreferences>(key: K, value: DesktopMessengerPreferences[K]) => void;
  actor?: MessagingActor;
  actorId: string;
};

const settingsNavigationItems: ReadonlyArray<{ id: SettingsCategory; label: string; subtitle: string; glyph: string }> = [
  { id: 'account', label: '我的资料', subtitle: '头像、姓名、用户名', glyph: '👤' },
  { id: 'notifications', label: '通知与声音', subtitle: '通知、预览与静音', glyph: '🔔' },
  { id: 'privacy', label: '隐私与安全', subtitle: '屏蔽、密码、隐私控制', glyph: '🔒' },
  { id: 'data', label: '数据与存储', subtitle: '下载、缓存与媒体', glyph: '💾' },
  { id: 'chat', label: '聊天设置', subtitle: '外观、发送与动画', glyph: '💬' },
  { id: 'folders', label: '聊天文件夹', subtitle: '组织会话', glyph: '📁' },
  { id: 'devices', label: '设备', subtitle: '活动会话与登录设备', glyph: '💻' },
  { id: 'calls', label: '通话', subtitle: '麦克风、摄像头与网络', glyph: '📞' },
  { id: 'language', label: '语言', subtitle: '界面与翻译语言', glyph: '🌐' },
  { id: 'advanced', label: '高级', subtitle: '网络、更新与系统集成', glyph: '⚙️' },
  { id: 'fabushi', label: 'AI 与 Mini Apps', subtitle: 'Fabushi 原生能力', glyph: '✦' },
];

function SettingsNavigation({ category, onCategory }: SettingsNavigationProps) {
  return <div className={styles.settingsNavigation} data-testid="telegram-settings-navigation">
    {settingsNavigationItems.map((item) => <button key={item.id} type="button" data-testid={`settings-category-${item.id}`} data-active={category === item.id} onClick={() => onCategory(item.id)}>
      <span>{item.glyph}</span><div><strong>{item.label}</strong><small>{item.subtitle}</small></div>
    </button>)}
  </div>;
}

function SettingsToggleRow({ title, description, checked, onChange, testId }: { title: string; description: string; checked: boolean; onChange: (checked: boolean) => void; testId?: string }) {
  return <label className={styles.settingsRow}><div><strong>{title}</strong><small>{description}</small></div><input data-testid={testId} type="checkbox" role="switch" checked={checked} onChange={(event) => onChange(event.target.checked)} /></label>;
}

function SettingsPlannedRow({ title, description }: { title: string; description: string }) {
  return <div className={`${styles.settingsRow} ${styles.settingsRowDisabled}`}><div><strong>{title}</strong><small>{description}</small></div><em>后端接入中</em></div>;
}

function SettingsWorkspace({ category, preferences, onPreference, actor, actorId }: SettingsWorkspaceProps) {
  const meta = settingsNavigationItems.find((item) => item.id === category)!;
  const profileName = actor?.displayName || '当前用户';
  return <div className={styles.settingsWorkspace} data-testid="telegram-settings-workspace">
    <header><span className={styles.settingsGlyph}>{meta.glyph}</span><div><h2>{meta.label}</h2><p>{meta.subtitle} · 参考 Telegram Desktop 的分组方式，并绑定 Fabushi 自建能力。</p></div></header>
    {category === 'account' ? <section className={styles.settingsGroup}>
      <div className={styles.settingsProfile}><BotMark botId={`self:${actorId}`} state="idle" size={72} label={profileName} /><div><strong>{profileName}</strong><small>{actor?.username ? `@${actor.username}` : actorId}</small><p>{actor?.bio || 'Fabushi 统一 Actor 资料由 Rust 消息核心管理。'}</p></div></div>
      <SettingsPlannedRow title="编辑个人资料" description="姓名、用户名、简介与头像写回 Actor Profile。" />
      <SettingsPlannedRow title="账号与手机号" description="账号身份与恢复渠道将在统一账户服务接入后开放。" />
    </section> : null}
    {category === 'notifications' ? <section className={styles.settingsGroup}>
      <SettingsToggleRow testId="settings-toggle-message-preview" title="消息预览" description="在会话列表显示最近消息摘要。" checked={preferences.messagePreview} onChange={(value) => onPreference('messagePreview', value)} />
      <SettingsPlannedRow title="桌面通知" description="按私聊、群组和频道分别控制系统通知。" />
      <SettingsPlannedRow title="通知声音" description="选择提示音，并支持按会话覆盖。" />
    </section> : null}
    {category === 'privacy' ? <section className={styles.settingsGroup}>
      <SettingsPlannedRow title="屏蔽用户" description="查看和管理已屏蔽 Actor。" />
      <SettingsPlannedRow title="最后上线与在线状态" description="基于 Fabushi presence ACL 控制可见范围。" />
      <SettingsPlannedRow title="两步验证与本地锁" description="接入统一账户安全策略后启用。" />
    </section> : null}
    {category === 'data' ? <section className={styles.settingsGroup}>
      <SettingsToggleRow testId="settings-toggle-autoplay-media" title="自动播放视频" description="聊天内视频加载后自动播放；默认关闭以节省资源。" checked={preferences.autoPlayMedia} onChange={(value) => onPreference('autoPlayMedia', value)} />
      <div className={styles.settingsInfoRow}><strong>本地消息数据库</strong><small>Rust SQLite 是权威本地存储；Renderer 只保存有界快速启动投影。</small><em>已启用</em></div>
      <SettingsPlannedRow title="自动下载媒体" description="按网络类型、会话类型和文件大小设置策略。" />
      <SettingsPlannedRow title="存储占用" description="按媒体类型查看缓存并执行安全清理。" />
    </section> : null}
    {category === 'chat' ? <section className={styles.settingsGroup}>
      <SettingsToggleRow testId="settings-toggle-info-panel" title="显示资料侧栏" description="宽屏聊天时显示右侧资料栏；窄屏自动收起且不占布局宽度。" checked={preferences.showInfoPanel} onChange={(value) => onPreference('showInfoPanel', value)} />
      <SettingsToggleRow testId="settings-toggle-enter-send" title="Enter 发送消息" description="关闭后使用 Command/Ctrl + Enter 发送，Enter 换行。" checked={preferences.enterToSend} onChange={(value) => onPreference('enterToSend', value)} />
      <SettingsToggleRow testId="settings-toggle-reduced-motion" title="减少动态效果" description="关闭大部分界面过渡动画，适合低功耗或辅助功能场景。" checked={preferences.reducedMotion} onChange={(value) => onPreference('reducedMotion', value)} />
      <SettingsPlannedRow title="聊天背景与气泡" description="主题、背景、字号与消息密度将在统一主题引擎中开放。" />
    </section> : null}
    {category === 'folders' ? <section className={styles.settingsGroup}><SettingsPlannedRow title="聊天文件夹" description="创建、排序并共享自定义会话过滤器。" /><SettingsPlannedRow title="归档行为" description="设置新消息到来时是否自动移出归档。" /></section> : null}
    {category === 'devices' ? <section className={styles.settingsGroup}><SettingsPlannedRow title="活动会话" description="列出已授权设备、最近活动与远程退出操作。" /><SettingsPlannedRow title="新设备登录提醒" description="设备身份系统接入后开启安全提醒。" /></section> : null}
    {category === 'calls' ? <section className={styles.settingsGroup}><div className={styles.settingsInfoRow}><strong>通话引擎</strong><small>Fabushi 自建信令 + WebRTC，支持语音、视频与屏幕共享。</small><em>可用</em></div><SettingsPlannedRow title="输入/输出设备" description="选择麦克风、摄像头和扬声器，并进行测试。" /><SettingsPlannedRow title="点对点与 TURN 策略" description="按隐私策略选择直连或中继。" /></section> : null}
    {category === 'language' ? <section className={styles.settingsGroup}><div className={styles.settingsInfoRow}><strong>界面语言</strong><small>当前跟随系统语言。</small><em>跟随系统</em></div><SettingsPlannedRow title="翻译语言" description="为消息翻译和 AI 翻译指定首选语言。" /></section> : null}
    {category === 'advanced' ? <section className={styles.settingsGroup}><div className={styles.settingsInfoRow}><strong>增量同步</strong><small>首轮最多 20 条；后续基于 cursor 每批最多 100 条，避免启动大同步。</small><em>已启用</em></div><div className={styles.settingsInfoRow}><strong>应用更新</strong><small>检测到 GitHub Release 新版本后可从头像旁云朵入口下载并安装。</small><em>自动检测</em></div><SettingsPlannedRow title="代理与网络" description="支持直连、系统代理与自建代理节点。" /></section> : null}
    {category === 'fabushi' ? <section className={styles.settingsGroup}><div className={styles.settingsInfoRow}><strong>AI Bot / Agent</strong><small>联系人、Bot、群组与频道共用 Actor/Conversation 消息模型。</small><em>已融合</em></div><div className={styles.settingsInfoRow}><strong>Mini Apps</strong><small>从在线市场搜索、验证、安装，并在受控宿主容器运行。</small><em>已融合</em></div><SettingsPlannedRow title="AI 权限中心" description="统一管理电脑控制、敏感输入、Mini App 与 Bot 权限。" /></section> : null}
  </div>;
}

function SectionPanel({ section, onInvoice, payment, onRefund, settings, ...miniAppProps }: { section: MessengerSection; onInvoice: () => void; payment: PaymentUiState; onRefund: (orderId: string) => void; settings: SettingsNavigationProps } & MiniAppMarketplaceProps) {
  if (section === 'miniapps') return <MiniAppMarketplaceList {...miniAppProps} />;
  if (section === 'payments') return <div className={styles.sectionList}><PaymentOverview payment={payment} onInvoice={onInvoice} onRefund={onRefund} compact /></div>;
  if (section === 'folders') return <div className={styles.sectionList}><div className={styles.panelHint}><Folder size={24} /><strong>聊天文件夹</strong><p>按联系人、Bot、群组、频道、未读和静音状态组织。</p></div></div>;
  if (section === 'calls') return <div className={styles.sectionList}><div className={styles.panelHint}><Phone size={24} /><strong>最近通话</strong><p>从会话顶部发起语音/视频。</p></div></div>;
  if (section === 'settings') return <div className={styles.sectionList}><SettingsNavigation {...settings} /></div>;
  return <div className={styles.sectionList}><div className={styles.panelHint}><Settings size={24} /><strong>{sectionTitle(section)}</strong><p>该功能入口已合并进统一 Messenger。</p></div></div>;
}

function FeatureWorkspace({ section, onInvoice, payment, onRefund, settings, ...miniAppProps }: { section: MessengerSection; onInvoice: () => void; payment: PaymentUiState; onRefund: (orderId: string) => void; settings: SettingsWorkspaceProps } & MiniAppMarketplaceProps) {
  if (section === 'miniapps') return <MiniAppMarketplaceWorkspace {...miniAppProps} />;
  if (section === 'payments') return <div className={styles.featureWorkspace}><WalletCards size={54} /><h2>Fabushi Pay</h2><p>自建余额、Invoice、Order、退款与外部 settlement 都由 Rust 账本结算。</p><PaymentOverview payment={payment} onInvoice={onInvoice} onRefund={onRefund} /></div>;
  if (section === 'calls') return <div className={styles.featureWorkspace}><Phone size={54} /><h2>通话</h2><p>本机媒体已接通，Rust realtime 已具备一对一/群组通话信令状态。</p></div>;
  if (section === 'settings') return <SettingsWorkspace {...settings} />;
  return <div className={styles.featureWorkspace}><MessageCircle size={54} /><h2>{sectionTitle(section)}</h2><p>联系人、Bot、群组和频道正在统一到同一个 Fabushi Actor/Conversation 模型。</p></div>;
}
