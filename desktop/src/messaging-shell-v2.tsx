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
  Monitor,
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
import React, { useCallback, useEffect, useMemo, useRef, useState, type FormEvent } from 'react';
import HostClient from '../../frontend/apps/web/src/app/host/host-client';
import { BotMark, type BotMarkState } from '../../frontend/apps/web/src/app/host/bot-mark';
import type {
  AuthState,
  BotSummary,
  ConversationSummary,
  GroupSummary,
  InferenceProvider,
  ProductHostSettings,
  RuntimeEvent,
  SandboxRuntime,
  UpdateState,
} from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  ElectronMahayanaHostTransport,
  isElectronMahayanaHostAvailable,
  MAHAYANA_ACCOUNT_SESSION_RESET_EVENT,
  MAHAYANA_COMMAND_EVENT_NAME,
  readCachedConversationMessages,
  type MahayanaCommandBridgeDetail,
} from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import { MockMahayanaHostTransport } from '../../frontend/apps/web/src/lib/mahayana-host/mock-transport';
import { invokeNativeDesktop, subscribeNativeDesktopEvents } from '../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import type { InstalledPluginPointer, MahayanaHostTransport, MarketplacePluginSummary } from '../../frontend/apps/web/src/lib/mahayana-host/transport';
import { marketplaceInstallAction, marketplaceInstallActionLabel } from '../../frontend/apps/web/src/lib/marketplace-install-contract';
import {
  RemoteComputerDesktopController,
  type RemoteComputerDesktopState,
} from '../../frontend/apps/web/src/lib/remote-computer/desktop-peer';
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
import { isTerminalAuthSessionFailure } from './auth-session';
import {
  installedMiniAppBotProjections,
  miniAppBotResponseText,
  type MiniAppBotCallProgram,
  type MiniAppBotCallPrograms,
  type MiniAppBotCommand,
} from './miniapp-bot-projection';
import { MiniAppCallDialog } from './miniapp-call-dialog';
import { executeDesktopMiniAppBotInput, prepareDesktopMiniAppWebMcpDocument } from './miniapp-webmcp-host';
import BotConversationView from './bot-conversation-view';
import type { BotTranscriptMessage } from './bot-conversation-view';
import {
  accountMiniAppsAsMarketplaceSummaries,
  appendMiniAppBotMessages,
  deleteAccountAgentStoreObject,
  deleteMiniAppCloudStorage,
  listAccountAgentStore,
  readAccountAgentStoreObject,
  readAccountBots,
  readAccountMiniApps,
  readAccountSync,
  readMiniAppBotMessages,
  readMiniAppCloudStorage,
  reconcileAccountMiniApps,
  upsertAccountAgent,
  writeAccountAgentStoreObject,
  writeMiniAppCloudStorage,
  type AccountBotMembership,
} from './account-sync-client';
import { projectFabuAgentProfile, projectFabuAgentSettings, projectFabuBotIdentity } from './fabu-runtime/agent-domain';
import { FabuAgentStore } from './fabu-runtime/agent-store';
import { createAgentSubmissionQueue } from './fabu-runtime/submission-queue';
import GrokAgentSidebar, { type GrokAgentSidebarItem } from './grok-shell/grok-agent-sidebar';
import GrokCommandPalette from './grok-shell/grok-command-palette';
import { AgentOperationRegistry } from './grok-runtime/agent-operation-registry';
import {
  SidebarContactGroupManager,
  projectSidebarContactGroups,
  useSidebarContactGroups,
} from './sidebar-contact-groups';
import { MahayanaAssistantTurnView } from './mahayana-assistant-turn-view';
import {
  assistantTurnPlainText,
  createAssistantTurn,
  reduceAssistantTurn,
  type AssistantTurn,
} from './mahayana-assistant-turn';

function miniAppMarketplaceAction(
  app: MarketplacePluginSummary,
  installed: InstalledPluginPointer | undefined,
) {
  return marketplaceInstallAction(
    {
      id: app.pluginId,
      latestVersion: app.latestVersion,
      install: app.install,
      releaseManifest: app.releaseManifest,
    },
    installed
      ? { version: installed.version, artifactSha256: installed.artifactSha256 }
      : undefined,
  );
}

function miniAppReleaseLabel(app: MarketplacePluginSummary): string {
  const source = app.install?.source as { sourceRef?: unknown } | undefined;
  const sourceRef = typeof source?.sourceRef === 'string' ? source.sourceRef.slice(0, 9) : '';
  return sourceRef ? `GitHub · ${sourceRef}` : 'GitHub 来源待确认';
}

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
type PeerKind = 'conversation' | 'contact' | 'bot' | 'group' | 'channel' | 'saved';
type PeerSource = 'legacy' | 'selfhosted';

type PeerItem = {
  key: string;
  id: string;
  source: PeerSource;
  conversationId?: string;
  actorId?: string;
  /** Explicit runtime Agent identity. Never infer Agent-ness from a title/id string. */
  agentId?: string;
  groupId?: string;
  kind: PeerKind;
  hidden?: boolean;
  title: string;
  subtitle: string;
  unread: number;
  pinned: boolean;
  archived: boolean;
  updatedAtMs: number;
  avatar?: string;
  miniAppId?: string;
  miniAppCommands?: MiniAppBotCommand[];
  miniAppMenuButtonText?: string;
  miniAppCalls?: MiniAppBotCallPrograms;
};

type GeneratedMiniAppPreview = {
  id: string;
  title: string;
  html: string;
  complete: boolean;
};

function generatedMiniAppPreview(text: string, stableId: string): GeneratedMiniAppPreview | null {
  const fence = text.match(/```(?:html)?\s*\n([\s\S]*)/i);
  const raw = fence?.[1] ?? text;
  const start = raw.search(/<!doctype\s+html|<html\b/i);
  if (start < 0) return null;
  const candidate = raw.slice(start);
  if (!/<body\b/i.test(candidate) || candidate.length < 160) return null;
  const close = candidate.search(/<\/html>/i);
  const html = close >= 0 ? candidate.slice(0, close + candidate.slice(close).match(/^<\/html>/i)![0].length) : candidate;
  const titleMatch = html.match(/<title[^>]*>([\s\S]*?)<\/title>/i);
  const title = (titleMatch?.[1] ?? 'AI 生成小程序').replace(/<[^>]+>/g, '').trim().slice(0, 80) || 'AI 生成小程序';
  const suffix = String(stableId || Date.now()).toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 44) || 'preview';
  return { id: `generated-${suffix}`.slice(0, 64), title, html, complete: close >= 0 };
}

type DisplayMessage = {
  id: string;
  source: PeerSource;
  role: 'me' | 'peer';
  text: string;
  createdAtMs: number;
  kind?: 'message' | 'assistant-turn' | 'action' | 'thinking';
  operationId?: string;
  streaming?: boolean;
  optimistic?: boolean;
  queued?: boolean;
  actionTitle?: string;
  actionDetail?: string;
  actionStatus?: 'running' | 'completed' | 'failed' | 'interrupted';
  assistantTurn?: AssistantTurn;
  miniAppId?: string;
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

type MiniAppCallSession = {
  callId: string;
  miniAppId: string;
  title: string;
  kind: 'voice' | 'video';
  program: MiniAppBotCallProgram;
  html?: string;
};

type MessageMenu = { message: DisplayMessage; x: number; y: number } | null;
type ForwardDialogState = { sourceConversationId: string; message: DisplayMessage } | null;
type EditDialogState = { conversationId: string; messageId: string; originalText: string; text: string } | null;
type InvoiceDialogState = { conversationId: string; title: string; amount: string } | null;
type InfoTab = 'media' | 'files' | 'links';
type SearchCategory = 'chats' | 'channels' | 'apps' | 'posts' | 'images' | 'videos' | 'downloads' | 'links' | 'files' | 'music' | 'audio';
type SettingsCategory = 'account' | 'router' | 'usage' | 'updates' | 'notifications' | 'privacy' | 'data' | 'chat' | 'folders' | 'devices' | 'calls' | 'language' | 'advanced' | 'fabushi';

type InferenceRouterStatus = {
  schemaVersion: 1;
  providers: Array<{ id: InferenceProvider; label: string; available: boolean; authenticated: boolean; installed?: boolean; source: string }>;
  sandboxes: Array<{ id: SandboxRuntime; label: string; available: boolean; source: string }>;
};

type ProviderUsageSummary = { provider: string; requests: number; inputTokens: number; cachedInputTokens: number; outputTokens: number; reasoningTokens: number; totalTokens: number; lifetimeTokens: number; lastUsedAtMs: number | null };
type UsageSummary = { totalTokens: number; lifetimeTokens?: number; events: number; source: string; updatedAtMs?: number | null; byProvider?: ProviderUsageSummary[] };

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
  // The complete lightweight left-rail model is persisted for first-frame paint.
  // Rust/Host remains authoritative and reconciles these display-only summaries in background.
  legacyConversations?: ConversationSummary[];
  legacyBots?: BotSummary[];
  legacyGroups?: GroupSummary[];
  // Account-installed Bot/Mini App identities are also lightweight first-frame
  // summaries. Persist them so bots such as 全球法布施 do not disappear until
  // a slower account sync finishes after every launch.
  accountBots?: AccountBotMembership[];
  miniAppIdentityCatalog?: MarketplacePluginSummary[];
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
const accountSyncCursorKey = 'fabushi.desktop.account-sync-cursor.v1';
const messengerConversationJournalKey = 'fabushi.desktop.mahayana-conversation-journal.v1';
const miniAppExecutionPersistencePrefix = 'fabushi.desktop.miniapp-execution.v1:';
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

const defaultProductHostSettings: ProductHostSettings = {
  notifications: true,
  autoUpdateWhenIdle: true,
  localExecution: true,
  routeEgressLocally: false,
  securityKeys: false,
  webauthnProxyEnabled: false,
  localToolPermission: 'ask',
  remoteControlEnabled: false,
  aiComputerControlEnabled: true,
  autoReviewRules: [],
  inferenceProvider: 'fabushi',
  sandboxRuntime: 'host',
};

function asMessengerProjection(value: unknown): MessengerProjection | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
  const parsed = value as Partial<MessengerProjection>;
  if (parsed.version !== 1 || !Array.isArray(parsed.selfConversations) || !Array.isArray(parsed.selfActors)) return null;
  if (!parsed.selfMessages || typeof parsed.selfMessages !== 'object' || Array.isArray(parsed.selfMessages)) return null;
  if (parsed.legacyConversations !== undefined && !Array.isArray(parsed.legacyConversations)) return null;
  if (parsed.legacyBots !== undefined && !Array.isArray(parsed.legacyBots)) return null;
  if (parsed.legacyGroups !== undefined && !Array.isArray(parsed.legacyGroups)) return null;
  if (parsed.accountBots !== undefined && !Array.isArray(parsed.accountBots)) return null;
  if (parsed.miniAppIdentityCatalog !== undefined && !Array.isArray(parsed.miniAppIdentityCatalog)) return null;
  return parsed as MessengerProjection;
}

function readMessengerProjection(): MessengerProjection | null {
  if (typeof window === 'undefined') return null;
  try {
    return asMessengerProjection(JSON.parse(window.localStorage.getItem(messengerProjectionKey) || 'null'));
  } catch {
    return null;
  }
}

async function readDurableMessengerProjection(): Promise<MessengerProjection | null> {
  const local = readMessengerProjection();
  if (local) return local;
  try {
    const durable = asMessengerProjection(await invokeNativeDesktop<unknown>('readClientPersistence', { key: messengerProjectionKey }));
    if (!durable) return null;
    try {
      window.localStorage.setItem(messengerProjectionKey, JSON.stringify(durable));
    } catch {
      // The native client-persistence mirror remains available for the next launch.
    }
    return durable;
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
  void invokeNativeDesktop<boolean>('writeClientPersistence', {
    key: messengerProjectionKey,
    value: projection,
  }).catch(() => {
    // Native persistence is a durability mirror only; canonical Rust SQLite remains authoritative.
  });
}

function readAccountSyncCursor(): string | null {
  if (typeof window === 'undefined') return null;
  try {
    const value = window.localStorage.getItem(accountSyncCursorKey)?.trim();
    return value || null;
  } catch {
    return null;
  }
}

function persistAccountSyncCursor(cursor: string | null): void {
  if (typeof window === 'undefined') return;
  try {
    if (cursor) window.localStorage.setItem(accountSyncCursorKey, cursor);
    else window.localStorage.removeItem(accountSyncCursorKey);
  } catch {
    // Native persistence remains a best-effort durability mirror.
  }
  if (cursor) {
    void invokeNativeDesktop<boolean>('writeClientPersistence', {
      key: accountSyncCursorKey,
      value: { cursor, updatedAtMs: Date.now() },
    }).catch(() => {});
  } else {
    void invokeNativeDesktop<boolean>('removeClientPersistence', { key: accountSyncCursorKey }).catch(() => {});
  }
}

async function clearAccountScopedDesktopCaches(): Promise<void> {
  if (typeof window !== 'undefined') {
    // Tell every live Mahayana renderer transport to discard its in-memory
    // account journal before React unmount cleanup can flush it back to disk.
    window.dispatchEvent(new Event(MAHAYANA_ACCOUNT_SESSION_RESET_EVENT));
    try {
      window.localStorage.removeItem(messengerProjectionKey);
      window.localStorage.removeItem(messengerDraftsKey);
      window.localStorage.removeItem(messengerConversationJournalKey);
      for (const key of Object.keys(window.localStorage)) {
        if (key.startsWith(miniAppExecutionPersistencePrefix)) window.localStorage.removeItem(key);
      }
    } catch {
      // Native persistence cleanup below remains authoritative for fast-start.
    }
  }
  try {
    const executionKeys = await invokeNativeDesktop<string[]>('listClientPersistenceKeys', { prefix: miniAppExecutionPersistencePrefix }).catch(() => []);
    await Promise.all([
      invokeNativeDesktop<boolean>('removeClientPersistence', { key: messengerProjectionKey }),
      invokeNativeDesktop<boolean>('removeClientPersistence', { key: accountSyncCursorKey }),
      ...executionKeys.map((key) => invokeNativeDesktop<boolean>('removeClientPersistence', { key })),
    ]);
  } catch {
    // Older/unavailable native edges must not block signing out locally.
  }
  try { window.localStorage.removeItem(accountSyncCursorKey); } catch {}
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
  // Grok-style identity boundary: an Agent/Bot is a declared domain object,
  // never a contact whose title/id happens to contain "agent" or "assistant".
  return peer.kind === 'bot';
}

function localComputerLabel(): string {
  if (typeof navigator === 'undefined') return 'Fabushi 桌面端';
  const identity = `${navigator.platform || ''} ${navigator.userAgent || ''}`.toLowerCase();
  const platform = identity.includes('win')
    ? 'Windows'
    : identity.includes('mac')
      ? 'Mac'
      : identity.includes('linux')
        ? 'Linux'
        : '桌面端';
  return `Fabushi · ${platform}`;
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
  const id = conversation.id.toLowerCase();
  const kind = conversation.kind.toLowerCase();
  if (id.startsWith('mahayana:contact:') || id.startsWith('telegram:user:') || kind.includes('direct') || kind.includes('contact')) return 'contact';
  if (id.startsWith('mahayana-ai:') || id.startsWith('codex:') || kind.includes('agent') || kind.includes('bot') || kind.includes('assistant')) return 'bot';
  if (kind.includes('saved')) return 'saved';
  if (kind.includes('channel')) return 'channel';
  return 'conversation';
}

function selfKind(conversation: MessagingConversation, counterparty?: MessagingActor): PeerKind {
  if (conversation.kind === 'channel') return 'channel';
  if (conversation.kind === 'group') return 'group';
  if (conversation.kind === 'savedMessages') return 'saved';
  if (conversation.kind === 'direct' || conversation.kind === 'secret') {
    return counterparty && ['assistant', 'bot', 'service'].includes(counterparty.kind)
      ? 'bot'
      : 'contact';
  }
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
  if (section === 'contacts') return peer.kind === 'contact';
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

function cachedLegacyDisplayMessages(conversationId: string): DisplayMessage[] {
  return readCachedConversationMessages(conversationId).map((message) => ({
    id: message.id,
    source: 'legacy',
    role: message.role === 'user' ? 'me' : 'peer',
    text: message.text,
    createdAtMs: message.createdAtMs,
    kind: 'message',
    streaming: message.streaming,
  }));
}

type StructuredMessageBlock =
  | { type: 'paragraph'; text: string }
  | { type: 'heading'; level: number; text: string }
  | { type: 'table'; headers: string[]; rows: string[][] }
  | { type: 'sources'; files: string[] };

function markdownTableCells(line: string): string[] {
  return line.trim().replace(/^\|/, '').replace(/\|$/, '').split('|').map((cell) => cell.trim());
}

function isMarkdownTableDivider(line: string): boolean {
  const cells = markdownTableCells(line);
  return cells.length > 1 && cells.every((cell) => /^:?-{3,}:?$/.test(cell));
}

function sourceFileNames(line: string): string[] {
  if (!/^source files?\s*:/i.test(line.trim())) return [];
  const body = line.replace(/^source files?\s*:/i, '');
  const names = new Set<string>();
  for (const match of body.matchAll(/`([^`]+)`/g)) names.add(match[1].trim());
  for (const match of body.matchAll(/\b[\w@./-]+\.(?:md|csv|txt|json|pdf|docx?|xlsx?|png|jpe?g|ts|tsx|js|jsx|rs|swift|kt)\b/gi)) names.add(match[0]);
  return [...names].filter(Boolean);
}

export function parseStructuredMessage(text: string): StructuredMessageBlock[] {
  const lines = text.replace(/\r\n?/g, '\n').split('\n');
  const blocks: StructuredMessageBlock[] = [];
  for (let index = 0; index < lines.length;) {
    const line = lines[index].trim();
    if (!line) { index += 1; continue; }
    const heading = /^(#{1,3})\s+(.+)$/.exec(line);
    if (heading) {
      blocks.push({ type: 'heading', level: heading[1].length, text: heading[2].trim() });
      index += 1;
      continue;
    }
    if (index + 1 < lines.length && line.includes('|') && isMarkdownTableDivider(lines[index + 1])) {
      const headers = markdownTableCells(line);
      const rows: string[][] = [];
      index += 2;
      while (index < lines.length && lines[index].trim().includes('|') && lines[index].trim()) {
        const cells = markdownTableCells(lines[index]);
        if (cells.length !== headers.length) break;
        rows.push(cells);
        index += 1;
      }
      blocks.push({ type: 'table', headers, rows });
      continue;
    }
    const files = sourceFileNames(line);
    if (files.length) {
      blocks.push({ type: 'sources', files });
      index += 1;
      continue;
    }
    const paragraph: string[] = [line];
    index += 1;
    while (index < lines.length) {
      const candidate = lines[index].trim();
      if (!candidate) break;
      if (/^(#{1,3})\s+/.test(candidate)) break;
      if (index + 1 < lines.length && candidate.includes('|') && isMarkdownTableDivider(lines[index + 1])) break;
      if (sourceFileNames(candidate).length) break;
      paragraph.push(candidate);
      index += 1;
    }
    blocks.push({ type: 'paragraph', text: paragraph.join(' ') });
  }
  return blocks;
}

function StructuredMessageBody({ text, peers }: { text: string; peers: PeerItem[] }) {
  const blocks = parseStructuredMessage(text);
  const rich = blocks.some((block) => block.type !== 'paragraph');
  if (!rich) return <p>{text}</p>;
  return <div className={extra.structuredMessage} data-testid="structured-message-body">
    {blocks.map((block, blockIndex) => {
      if (block.type === 'heading') {
        return <strong key={`heading-${blockIndex}`} className={extra.structuredHeading} data-level={block.level}>{block.text}</strong>;
      }
      if (block.type === 'table') {
        return <div key={`table-${blockIndex}`} className={extra.structuredTableWrap}>
          <table className={extra.structuredTable} data-testid="assistant-result-table">
            <thead><tr>{block.headers.map((cell, index) => <th key={`${index}:${cell}`}>{cell}</th>)}</tr></thead>
            <tbody>{block.rows.map((row, rowIndex) => <tr key={rowIndex}>{row.map((cell, cellIndex) => {
              const peer = peers.find((candidate) => candidate.title.trim().toLocaleLowerCase() === cell.trim().replace(/^@/, '').toLocaleLowerCase());
              return <td key={`${rowIndex}:${cellIndex}`}>{peer ? <span className={extra.structuredPeerChip} data-testid="assistant-owner-chip"><BotMark botId={`peer:${peer.kind}:${peer.actorId ?? peer.id}`} state="result" size={16} label={peer.title} />{cell}</span> : cell}</td>;
            })}</tr>)}</tbody>
          </table>
        </div>;
      }
      if (block.type === 'sources') {
        return <div key={`sources-${blockIndex}`} className={extra.sourceFiles} data-testid="assistant-source-files">
          <span>Source files:</span>
          {block.files.map((file) => <span key={file} className={extra.sourceFileChip}><FileText size={13} />{file}</span>)}
        </div>;
      }
      return <p key={`paragraph-${blockIndex}`}>{block.text}</p>;
    })}
  </div>;
}

function startupLegacyConversationId(projection: MessengerProjection | null | undefined): string | null {
  const key = projection?.activePeerKey;
  if (!key) return null;
  if (key.startsWith('legacy:conversation:')) return key.slice('legacy:conversation:'.length) || null;
  if (key.startsWith('legacy:bot:')) {
    const botId = key.slice('legacy:bot:'.length);
    return projection?.legacyBots?.find((bot) => bot.id === botId)?.conversationId ?? null;
  }
  return null;
}

export default function DesktopShellV2() {
  const authTransport = useMemo(() => createTransport(), []);
  const localProjection = useMemo(() => readMessengerProjection(), []);
  const [startupProjection, setStartupProjection] = useState<MessengerProjection | null>(localProjection);
  const [projectionLookupComplete, setProjectionLookupComplete] = useState(Boolean(localProjection));
  const [authenticated, setAuthenticated] = useState<boolean | null>(null);
  const authTransitionEpoch = useRef(0);

  const resetToLogin = useCallback(async (revokeSession = true) => {
    // Invalidate any authStatus request that started before this explicit transition.
    const transitionEpoch = ++authTransitionEpoch.current;
    try {
      if (revokeSession) await authTransport.logout();
    } catch {
      // Product logout is best effort remotely; local account state is cleared below.
    } finally {
      await clearAccountScopedDesktopCaches();
      // A newer login may complete while logout/cache cleanup is in flight.
      if (transitionEpoch !== authTransitionEpoch.current) return;
      setStartupProjection(null);
      setProjectionLookupComplete(true);
      setAuthenticated(false);
    }
  }, [authTransport]);

  const handleHostAuthStateChange = useCallback((state: AuthState) => {
    if (state.loggedIn) {
      // Invalidate a signed-out authStatus snapshot that may still be clearing
      // account caches before it gets a chance to overwrite this fresh login.
      authTransitionEpoch.current += 1;
      // A fresh login must not remain behind the durable projection lookup.
      // The projection is optional startup data and can hydrate in the background.
      setProjectionLookupComplete(true);
      setAuthenticated(true);
    }
  }, []);

  useEffect(() => {
    if (localProjection) return;
    let closed = false;
    void readDurableMessengerProjection().then((projection) => {
      if (closed) return;
      setStartupProjection(projection);
      setProjectionLookupComplete(true);
    });
    return () => { closed = true; };
  }, [localProjection]);

  useEffect(() => {
    let closed = false;
    let retryTimer: number | undefined;
    const checkAuth = async () => {
      const requestEpoch = authTransitionEpoch.current;
      try {
        const state = await authTransport.authStatus();
        if (closed || requestEpoch !== authTransitionEpoch.current) return;
        if (!state.loggedIn) {
          await clearAccountScopedDesktopCaches();
          // Browser login can finish while the signed-out cache cleanup awaits.
          // Never let that older false snapshot replace the authenticated shell.
          if (closed || requestEpoch !== authTransitionEpoch.current) return;
          setStartupProjection(null);
        }
        setAuthenticated(state.loggedIn);
        if (!state.loggedIn) retryTimer = window.setTimeout(() => void checkAuth(), 900);
      } catch (cause) {
        if (closed) return;
        if (isTerminalAuthSessionFailure(cause)) {
          await resetToLogin(true);
          return;
        }
        // A transient Host/network failure must not replace a valid local-first shell
        // with a login/restore screen. Only an explicit loggedIn=false response signs out.
        retryTimer = window.setTimeout(() => void checkAuth(), 1_800);
      }
    };
    void checkAuth();
    return () => {
      closed = true;
      if (retryTimer) window.clearTimeout(retryTimer);
      void authTransport.close();
    };
  }, [authTransport, resetToLogin]);

  const showMessenger = projectionLookupComplete
    && authenticated !== false
    && (authenticated === true || Boolean(startupProjection));
  const showLogin = projectionLookupComplete && authenticated === false;
  return (
    <div className={styles.desktopRoot} data-testid="desktop-shell" data-local-first={showMessenger && authenticated !== true ? 'true' : undefined}>
      {showMessenger
        ? <MessengerWorkspace initialProjection={startupProjection} onLogout={() => resetToLogin(true)} />
        : showLogin
          ? <HostClient onAuthStateChange={handleHostAuthStateChange} />
          : <DesktopFastStartBootstrap />}
    </div>
  );
}

function DesktopFastStartBootstrap() {
  return (
    <main
      className={`${styles.messenger} ${styles.fabushiUnified} ${styles.grokParity}`}
      data-testid="desktop-fast-start-bootstrap"
      aria-busy="true"
      aria-label="Fabushi 正在连接"
      style={{ gridTemplateColumns: '330px minmax(420px,1fr)' }}
    >
      <aside className={styles.chatList}>
        <header className={styles.listHeader}>
          <span className={styles.sidebarBrand} title="Fabushi"><BotMark botId="fabushi:bootstrap" state="idle" size={30} paused label="Fabushi" /></span>
          <strong>聊天</strong>
        </header>
        <label className={styles.searchBox}>
          <Search size={16} />
          <input disabled placeholder="搜索" aria-label="搜索" />
        </label>
        <div className={styles.peerList} aria-hidden="true" />
        <div className={styles.sidebarFooter}>
          <button type="button" className={styles.profileNavigationTrigger} disabled>
            <BotMark botId="fabushi:bootstrap:self" state="idle" size={38} paused label="我的头像" />
            <span><strong>我</strong><small>正在连接</small></span>
          </button>
        </div>
      </aside>
      <section className={styles.chatWorkspace}>
        <div className={styles.chatEmpty}>
          <BotMark botId="fabushi:bootstrap:workspace" state="idle" size={72} paused label="Fabushi" />
          <strong>Fabushi</strong>
          <p>界面已经就绪，正在后台连接本机服务。</p>
        </div>
      </section>
    </main>
  );
}

function MessengerWorkspace({ initialProjection, onLogout }: { initialProjection?: MessengerProjection | null; onLogout: () => Promise<void> }) {
  const transport = useMemo(() => createTransport(), []);
  const startupProjection = useMemo(() => initialProjection ?? readMessengerProjection(), [initialProjection]);
  const startupLegacyConversation = useMemo(() => startupLegacyConversationId(startupProjection), [startupProjection]);
  const selfHosted = useMemo(() => new SelfHostedMessagingClientV2(transport, { actorId: startupProjection?.actorId }), [transport, startupProjection]);
  const [hostReady, setHostReady] = useState(false);
  const [remoteAccountScope, setRemoteAccountScope] = useState<string | null>(null);
  const [initialLegacyHydrationMask, setInitialLegacyHydrationMask] = useState(0);
  const initialLegacyHydrationMaskRef = useRef(0);
  const initialLegacyHydrated = initialLegacyHydrationMask === 0b111;
  // Grok/Fabu parity: the primary desktop shell is Agent-first. Messenger,
  // Contacts, Channels, Calls and Payments remain backend capabilities but no
  // longer own top-level navigation.
  const [section, setSection] = useState<MessengerSection>('bots');
  const newAgentRequestPendingRef = useRef(false);
  const [pendingOpenAgentId, setPendingOpenAgentId] = useState<string | null>(null);
  const [conversations, setConversations] = useState<ConversationSummary[]>(startupProjection?.legacyConversations ?? []);
  const [bots, setBots] = useState<BotSummary[]>(startupProjection?.legacyBots ?? []);
  const [groups, setGroups] = useState<GroupSummary[]>(startupProjection?.legacyGroups ?? []);
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
  const [messages, setMessages] = useState<DisplayMessage[]>(() =>
    startupLegacyConversation ? cachedLegacyDisplayMessages(startupLegacyConversation) : [],
  );
  const [composer, setComposer] = useState('');
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [replyTo, setReplyTo] = useState<DisplayMessage | null>(null);
  const [silentSend, setSilentSend] = useState(false);
  const [scheduledAtMs, setScheduledAtMs] = useState<number | undefined>();
  const [search, setSearch] = useState('');
  const [grokPaletteOpen, setGrokPaletteOpen] = useState(false);
  const [grokPaletteQuery, setGrokPaletteQuery] = useState('');
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
  const settingsReturnSectionRef = useRef<MessengerSection>('chats');
  const [hostSettings, setHostSettings] = useState<ProductHostSettings>(defaultProductHostSettings);
  const [routerStatus, setRouterStatus] = useState<InferenceRouterStatus | null>(null);
  const [usageSummary, setUsageSummary] = useState<UsageSummary | null>(null);
  const [infoOpen, setInfoOpen] = useState(() => readDesktopMessengerPreferences().showInfoPanel);
  const [narrowInfoOpen, setNarrowInfoOpen] = useState(false);
  const [wideInfoLayout, setWideInfoLayout] = useState(() => typeof window === 'undefined' ? true : window.innerWidth > 1280);
  const [infoTab, setInfoTab] = useState<InfoTab>('media');
  const [computerProfileOpen, setComputerProfileOpen] = useState(false);
  const [remoteComputerState, setRemoteComputerState] = useState<RemoteComputerDesktopState | null>(null);
  const [pendingSend, setPendingSend] = useState(false);
  // Kept as the active-peer projection for compatibility with existing
  // transcript DOM contracts. The authoritative ownership is per Agent below.
  const [agentOperationId, setAgentOperationId] = useState<string | null>(null);
  const [agentOperationByPeer, setAgentOperationByPeer] = useState<Record<string, string>>({});
  const [showScrollToLatest, setShowScrollToLatest] = useState(false);
  const [queuedAgentPrompts, setQueuedAgentPrompts] = useState<Record<string, DisplayMessage[]>>({});
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
  const [miniApp, setMiniApp] = useState<{ id: string; title: string; url: string } | null>(null);
  const [miniAppCall, setMiniAppCall] = useState<MiniAppCallSession | null>(null);
  const miniAppBotThreadsRef = useRef<Record<string, DisplayMessage[]>>({});
  const botThreadsRef = useRef<Record<string, DisplayMessage[]>>({});
  const [accountBots, setAccountBots] = useState<AccountBotMembership[]>(startupProjection?.accountBots ?? []);
  const [marketplaceApps, setMarketplaceApps] = useState<MarketplacePluginSummary[]>([]);
  const [miniAppIdentityCatalog, setMiniAppIdentityCatalog] = useState<MarketplacePluginSummary[]>(startupProjection?.miniAppIdentityCatalog ?? []);
  const [installedMiniApps, setInstalledMiniApps] = useState<Record<string, InstalledPluginPointer>>({});
  const [miniAppQuery, setMiniAppQuery] = useState('');
  const [miniAppLoading, setMiniAppLoading] = useState(false);
  const [miniAppBusy, setMiniAppBusy] = useState<Set<string>>(() => new Set());
  const [error, setError] = useState<string | null>(null);
  const [mutedPeerKeys, setMutedPeerKeys] = useState<Set<string>>(() => new Set());
  const [pinnedPeerKeys, setPinnedPeerKeys] = useState<Set<string>>(() => new Set());
  const [archivedPeerKeys, setArchivedPeerKeys] = useState<Set<string>>(() => new Set());
  const contactGroups = useSidebarContactGroups();
  const activePeerKeyRef = useRef<string | null>(null);
  const messagingCursorRef = useRef<string | null>(startupProjection?.cursor ?? null);
  const accountSyncCursorRef = useRef<string | null>(readAccountSyncCursor());
  const syncInFlightRef = useRef(false);
  const accountSyncInFlightRef = useRef(false);
  const typingStopTimerRef = useRef<number | null>(null);
  const peersRef = useRef<PeerItem[]>([]);
  const webRtcRef = useRef<FabushiWebRtcController | null>(null);
  const remoteComputerControllerRef = useRef<RemoteComputerDesktopController | null>(null);
  const localVideoRef = useRef<HTMLVideoElement>(null);
  const remoteVideoRef = useRef<HTMLVideoElement>(null);
  const remoteAudioRef = useRef<HTMLAudioElement>(null);
  const mediaInputRef = useRef<HTMLInputElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const sessionResetInFlightRef = useRef(false);
  const agentOperationIdRef = useRef<string | null>(null);
  const agentOperationRegistryRef = useRef(new AgentOperationRegistry());
  const agentRequestPendingRef = useRef(false);
  const agentRequestPeerRef = useRef<string | null>(null);
  const agentRequestIdRef = useRef<string | null>(null);
  const pendingAgentDeltaRef = useRef(new Map<string, Extract<RuntimeEvent, { type: 'chat.delta' }>>());
  const agentDeltaFrameRef = useRef<number | null>(null);
  const finishedAgentOperationsRef = useRef(new Set<string>());
  const agentPeerKeyRef = useRef<Record<string, string>>({});
  const messageAreaRef = useRef<HTMLDivElement | null>(null);
  const stickToLatestRef = useRef(true);
  const remoteControlEnabledRef = useRef(hostSettings.remoteControlEnabled);
  remoteControlEnabledRef.current = hostSettings.remoteControlEnabled;
  const hostReadyRef = useRef(hostReady);
  hostReadyRef.current = hostReady;
  const agentStoresRef = useRef(new Map<string, FabuAgentStore>());

  const removeQueuedAgentPrompt = useCallback((peerKey: string, messageId: string) => {
    setQueuedAgentPrompts((current) => {
      const pending = (current[peerKey] ?? []).filter((message) => message.id !== messageId);
      const next = { ...current };
      if (pending.length) next[peerKey] = pending;
      else delete next[peerKey];
      return next;
    });
  }, []);

  const agentSubmissionQueue = useMemo(() => createAgentSubmissionQueue({
    isBlocked: (input) => !hostReadyRef.current
      || agentOperationRegistryRef.current.isBusy(input.peerKey),
    send: async (input) => {
      const peer = peersRef.current.find((candidate) => candidate.key === input.peerKey);
      if (!peer || !isAgentPeer(peer)) throw new Error('Agent peer is no longer available.');
      await dispatchAgentPromptNow(peer, input.prompt, input.messageId);
    },
    onPhase: (submission) => {
      if (submission.phase === 'queued') {
        const queuedMessage: DisplayMessage = {
          id: submission.messageId,
          source: 'legacy',
          role: 'me',
          text: submission.prompt,
          createdAtMs: submission.createdAtMs,
          kind: 'message',
          optimistic: true,
          queued: true,
        };
        setQueuedAgentPrompts((current) => {
          const list = current[submission.peerKey] ?? [];
          const index = list.findIndex((message) => message.id === submission.messageId);
          const nextList = index < 0
            ? [...list, queuedMessage]
            : list.map((message, messageIndex) => messageIndex === index ? queuedMessage : message);
          return { ...current, [submission.peerKey]: nextList };
        });
        return;
      }
      removeQueuedAgentPrompt(submission.peerKey, submission.messageId);
    },
    onFailure: (submission, cause) => {
      removeQueuedAgentPrompt(submission.peerKey, submission.messageId);
      setError(cause instanceof Error ? cause.message : String(cause));
    },
  }), [removeQueuedAgentPrompt]);

  useEffect(() => () => agentSubmissionQueue.dispose(), [agentSubmissionQueue]);

  useEffect(() => {
    if (hostReady) agentSubmissionQueue.flush();
  }, [agentSubmissionQueue, hostReady]);

  useEffect(() => {
    activePeerKeyRef.current = activePeerKey;
  }, [activePeerKey]);

  useEffect(() => () => {
    if (agentDeltaFrameRef.current !== null) {
      window.cancelAnimationFrame(agentDeltaFrameRef.current);
      agentDeltaFrameRef.current = null;
    }
    pendingAgentDeltaRef.current.clear();
  }, []);

  useEffect(() => {
    const onResize = () => {
      const nextWide = window.innerWidth > 1280;
      setWideInfoLayout(nextWide);
      if (nextWide) setNarrowInfoOpen(false);
    };
    onResize();
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);

  useEffect(() => {
    if (!error || !isTerminalAuthSessionFailure(error) || sessionResetInFlightRef.current) return;
    sessionResetInFlightRef.current = true;
    void onLogout()
      .catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))
      .finally(() => { sessionResetInFlightRef.current = false; });
  }, [error, onLogout]);

  useEffect(() => {
    try {
      window.localStorage.setItem(messengerPreferencesKey, JSON.stringify(desktopPreferences));
    } catch {
      // Desktop-only preferences are best effort.
    }
  }, [desktopPreferences]);

  useEffect(() => {
    if (startupProjection?.activePeerKey?.startsWith('selfhosted:')) {
      const conversationId = startupProjection.activePeerKey.slice('selfhosted:'.length);
      const cached = (startupProjection.selfMessages[conversationId] ?? []).filter((message) => !message.deleted);
      setMessages(cached.map(displaySelfMessage));
      return;
    }
    const legacyConversationId = startupLegacyConversationId(startupProjection);
    if (legacyConversationId) setMessages(cachedLegacyDisplayMessages(legacyConversationId));
  }, [startupProjection]);

  useEffect(() => {
    if (
      !selfConversations.length
      && !selfActors.length
      && !conversations.length
      && !bots.length
      && !groups.length
      && !accountBots.length
      && !miniAppIdentityCatalog.length
    ) return;
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
        legacyConversations: conversations,
        legacyBots: bots,
        legacyGroups: groups,
        accountBots,
        miniAppIdentityCatalog,
        selfActors,
        selfConversations: [...selfConversations]
          .sort((left, right) => right.updatedAtMs - left.updatedAtMs)
          .slice(0, projectionConversationLimit),
        selfMessages: boundedMessages,
      });
    }, 60);
    return () => window.clearTimeout(timer);
  }, [activePeerKey, accountBots, bots, conversations, groups, miniAppIdentityCatalog, selfActors, selfConversations, selfMessages, selfHosted.actorId]);

  function updateDesktopPreference<K extends keyof DesktopMessengerPreferences>(key: K, value: DesktopMessengerPreferences[K]) {
    setDesktopPreferences((current) => ({ ...current, [key]: value }));
    if (key === 'showInfoPanel') setInfoOpen(Boolean(value));
  }

  function updateHostSetting<K extends keyof ProductHostSettings>(key: K, value: ProductHostSettings[K]) {
    const settings = { ...hostSettings, [key]: value };
    setHostSettings(settings);
    void execute({ type: 'settings.update', requestId: nextRequestId('settings-update'), settings });
  }

  async function configureProviderSecret(provider: 'claude-code' | 'openrouter', value: string) {
    try {
      const name = provider === 'claude-code' ? 'inference/claude/api-key' : 'inference/openrouter/api-key';
      await invokeNativeDesktop('upsertSecrets', { name, value });
      if (hostSettings.inferenceProvider === provider) await invokeNativeDesktop('restartInferenceRouter');
      const status = await invokeNativeDesktop<InferenceRouterStatus>('getInferenceRouterStatus');
      setRouterStatus(status);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
      throw cause;
    }
  }

  async function removeProviderSecret(provider: 'claude-code' | 'openrouter') {
    try {
      const name = provider === 'claude-code' ? 'inference/claude/api-key' : 'inference/openrouter/api-key';
      await invokeNativeDesktop('removeSecrets', { name });
      const status = await invokeNativeDesktop<InferenceRouterStatus>('getInferenceRouterStatus');
      setRouterStatus(status);
      if (hostSettings.inferenceProvider === provider) updateHostSetting('inferenceProvider', 'fabushi');
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
      throw cause;
    }
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
      if (payload.type === 'available' || payload.type === 'upToDate' || payload.type === 'error') {
        setDesktopUpdateBusy(false);
      }
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
    const onCommandBridge = (event: Event) => {
      const detail = (event as CustomEvent<MahayanaCommandBridgeDetail>).detail;
      if (!detail || detail.command.type !== 'chat.send') return;
      const conversationKey = detail.context?.conversationKey;
      if (!conversationKey || activePeerKeyRef.current !== conversationKey) return;

      if (detail.phase === 'dispatch') {
        agentRequestPendingRef.current = true;
        setPendingSend(true);
        return;
      }
      if (detail.phase === 'accepted') {
        const operationId = detail.accepted.operationId;
        if (!operationId || !claimAgentOperation(operationId)) return;
        appendAssistantTurnEvent({
          type: 'operation.started',
          timestamp: new Date().toISOString(),
          operationId,
          label: '正在思考',
          interruptible: true,
        });
        return;
      }
      agentRequestPendingRef.current = false;
      setPendingSend(false);
      setError(detail.error);
    };
    window.addEventListener(MAHAYANA_COMMAND_EVENT_NAME, onCommandBridge);
    void transport.initialize({ profileId: 'desktop-messenger-v2', mode: 'production' })
      .then(async () => {
        if (closed) return;
        setHostReady(true);
        // First-frame identity hydration must not wait behind self-hosted sync or
        // account cursor reconciliation. These reads are lightweight and let
        // installed Mini App Bots (for example 全球法布施) appear immediately.
        void readAccountBots()
          .then((entries) => { if (!closed) setAccountBots(entries); })
          .catch(() => {});
        void readAccountMiniApps()
          .then((account) => {
            if (closed) return;
            const accountApps = accountMiniAppsAsMarketplaceSummaries(account);
            if (!accountApps.length) return;
            setMiniAppIdentityCatalog((current) => {
              const merged = new Map(current.map((app) => [app.pluginId, app]));
              for (const accountApp of accountApps) {
                const existing = merged.get(accountApp.pluginId);
                merged.set(accountApp.pluginId, {
                  ...existing,
                  ...accountApp,
                  bot: accountApp.bot ?? existing?.bot,
                  commands: accountApp.commands?.length ? accountApp.commands : existing?.commands,
                  surfaces: accountApp.surfaces?.length ? accountApp.surfaces : existing?.surfaces,
                });
              }
              return [...merged.values()];
            });
          })
          .catch(() => {});
        void execute({ type: 'settings.get', requestId: nextRequestId('settings-get') });
        refreshLegacy();
        if (startupLegacyConversation) {
          void execute({
            type: 'conversation.open',
            requestId: nextRequestId('conversation-open-startup'),
            conversationId: startupLegacyConversation,
          });
        }
        try {
          const account = await transport.authStatus().catch(() => null);
          const cachedActor = startupProjection?.selfActors.find((actor) => actor.id === selfHosted.actorId);
          const username = account?.user?.username?.trim() || cachedActor?.username;
          const displayName = account?.user?.nickname?.trim()
            || username
            || account?.user?.email?.trim()
            || cachedActor?.displayName
            || '当前用户';
          const identityScope = account?.user?.id ?? username ?? account?.user?.email?.trim() ?? startupProjection?.actorId;
          if (identityScope === undefined || identityScope === null || String(identityScope).trim() === '') {
            throw new Error('当前账号缺少电脑注册身份，无法启动后台在线状态');
          }
          setRemoteAccountScope(String(identityScope));
          await selfHosted.ensureCurrentActor(displayName, username);
          await selfHosted.sync(initialSyncLimit, messagingCursorRef.current);
          await synchronizeAccountState();
          void webRtcRef.current?.connect().catch(() => {});
        } catch (cause) {
          setError(cause instanceof Error ? cause.message : String(cause));
        }
      })
      .catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
    return () => {
      closed = true;
      unsubscribe();
      window.removeEventListener(MAHAYANA_COMMAND_EVENT_NAME, onCommandBridge);
      const remoteComputer = remoteComputerControllerRef.current;
      if (remoteComputer) void remoteComputer.stop().finally(() => transport.close());
      else void transport.close();
    };
  }, [transport, selfHosted, startupProjection]);

  useEffect(() => {
    if (!hostReady || initialLegacyHydrated) return;
    let attempts = 0;
    const retryMissing = () => {
      if (initialLegacyHydrationMaskRef.current === 0b111 || attempts >= 8) return;
      attempts += 1;
      refreshLegacy(initialLegacyHydrationMaskRef.current);
    };
    const timer = window.setInterval(retryMissing, 650);
    retryMissing();
    return () => window.clearInterval(timer);
  }, [hostReady, initialLegacyHydrated]);

  useEffect(() => {
    // Remote presence shares the Host request path with Messenger bootstrap.
    // Keep background registration off that path until all core legacy lists
    // have produced their first authoritative projection.
    if (!hostReady || !remoteAccountScope) return;
    if (!initialLegacyHydrated) return;
    let disposed = false;
    const controller = new RemoteComputerDesktopController({
      transport,
      label: localComputerLabel(),
      identityScope: remoteAccountScope,
      // Presence begins automatically after login. The controller receives the
      // persisted opt-in before any remote session polling is allowed.
      controlEnabled: remoteControlEnabledRef.current,
      resolveAgentId: (requestedAgentId) => requestedAgentId === 'mahayana-assistant'
        || peersRef.current.some((peer) => isAgentPeer(peer)
          && (peer.agentId ?? peer.actorId ?? peer.id) === requestedAgentId)
        ? requestedAgentId
        : null,
      onState: (state) => {
        if (!disposed) setRemoteComputerState(state);
      },
    });
    remoteComputerControllerRef.current = controller;
    setRemoteComputerState(controller.snapshot());
    void controller.start();
    return () => {
      disposed = true;
      if (remoteComputerControllerRef.current === controller) remoteComputerControllerRef.current = null;
      void controller.stop();
    };
  }, [hostReady, initialLegacyHydrated, remoteAccountScope, transport]);

  useEffect(() => {
    if (!hostReady) return;
    const controller = remoteComputerControllerRef.current;
    if (!controller) return;
    void controller.setControlEnabled(hostSettings.remoteControlEnabled).catch((cause: unknown) => {
      setError(cause instanceof Error ? cause.message : String(cause));
    });
  }, [hostReady, hostSettings.remoteControlEnabled]);

  useEffect(() => {
    setComputerProfileOpen(false);
  }, [activePeerKey]);

  useEffect(() => {
    if (!hostReady || section !== 'settings' || !['router', 'usage'].includes(settingsCategory)) return;
    let disposed = false;
    void Promise.all([
      invokeNativeDesktop<InferenceRouterStatus>('getInferenceRouterStatus'),
      invokeNativeDesktop<UsageSummary>('getUsageSummary'),
    ]).then(([status, usage]) => {
      if (disposed) return;
      setRouterStatus(status);
      setUsageSummary(usage);
    }).catch((cause: unknown) => {
      if (!disposed) setError(cause instanceof Error ? cause.message : String(cause));
    });
    return () => { disposed = true; };
  }, [hostReady, section, settingsCategory]);

  useEffect(() => {
    const handleGrokPaletteKeys = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setGrokPaletteQuery('');
        setGrokPaletteOpen((value) => !value);
        return;
      }
      if (event.key === 'Escape') setGrokPaletteOpen(false);
    };
    window.addEventListener('keydown', handleGrokPaletteKeys);
    return () => window.removeEventListener('keydown', handleGrokPaletteKeys);
  }, []);

  useEffect(() => {
    if (section !== 'settings') return;
    const modal = document.querySelector<HTMLElement>('[data-testid="settings-modal-backdrop"] [role="dialog"]');
    const focusable = () => Array.from(modal?.querySelectorAll<HTMLElement>('button:not(:disabled), select:not(:disabled), input:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])') ?? []);
    window.requestAnimationFrame(() => modal?.querySelector<HTMLElement>('[data-testid="settings-close"]')?.focus());
    const handleSettingsKeys = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        closeSettings();
        return;
      }
      if (event.key !== 'Tab') return;
      const items = focusable();
      if (!items.length) return;
      const first = items[0];
      const last = items[items.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    window.addEventListener('keydown', handleSettingsKeys);
    return () => window.removeEventListener('keydown', handleSettingsKeys);
  }, [section]);

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
      void synchronizeAccountState();
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

  function refreshLegacy(mask = 0) {
    const requests: Array<Promise<unknown>> = [];
    if ((mask & 0b001) === 0) requests.push(execute({ type: 'conversation.list', requestId: nextRequestId('conversation-list') }));
    if ((mask & 0b010) === 0) requests.push(execute({ type: 'bot.list', requestId: nextRequestId('bot-list') }));
    if ((mask & 0b100) === 0) requests.push(execute({ type: 'group.list', requestId: nextRequestId('group-list') }));
    void Promise.all(requests).catch(() => {});
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

  function agentStoreFor(agentId: string): FabuAgentStore {
    const existing = agentStoresRef.current.get(agentId);
    if (existing) return existing;
    const store = new FabuAgentStore(agentId, {
      list: (id, prefix) => listAccountAgentStore(id, prefix),
      read: (id, path) => readAccountAgentStoreObject(id, path),
      write: (id, path, dataBase64, options) => writeAccountAgentStoreObject(id, path, dataBase64, options),
      delete: (id, path, baseEtag) => deleteAccountAgentStoreObject(id, path, baseEtag),
    });
    agentStoresRef.current.set(agentId, store);
    return store;
  }

  function agentStorePathSegment(value: string): string {
    return encodeURIComponent(value.trim()).replace(/%2F/gi, '_');
  }

  async function mirrorAgentCloudSnapshot(
    agentId: string,
    path: string,
    value: unknown,
  ): Promise<void> {
    if (!agentId.trim()) return;
    const store = agentStoreFor(agentId);
    try {
      await store.writeJson(path, value);
    } catch {
      // The local Host remains authoritative while offline or during an
      // etag conflict. The content-addressed backend preserves conflicting
      // writes instead of silently overwriting another device.
    }
  }

  async function removeAgentCloudObject(agentId: string, path: string): Promise<void> {
    if (!agentId.trim()) return;
    try {
      await agentStoreFor(agentId).delete(path);
    } catch {
      // Deletion is best-effort; account sync/reconciliation can retry.
    }
  }

  async function mirrorBotAgentCloud(bot: BotSummary): Promise<void> {
    const identity = projectFabuBotIdentity(bot);
    const profile = projectFabuAgentProfile({
      id: bot.id,
      agentId: bot.agentId,
      conversationId: bot.conversationId,
      name: bot.name,
      description: bot.description,
      title: bot.title,
      avatarShape: bot.avatarShape,
      avatarColor: bot.avatarColor,
      hidden: bot.hidden,
      notificationsEnabled: bot.notificationsEnabled,
      notifyOnUpdates: bot.notifyOnUpdates,
    });
    const settings = projectFabuAgentSettings({
      id: bot.id,
      hidden: bot.hidden,
      notificationsEnabled: bot.notificationsEnabled,
      notifyOnUpdates: bot.notifyOnUpdates,
    });
    await invokeNativeDesktop('addBotToAccount', {
      botId: identity.botId,
      bot: {
        ...bot,
        agentId: identity.agentId,
        displayName: bot.name,
      },
    });
    await upsertAccountAgent(identity.agentId, profile, {
      agentId: identity.agentId,
      name: profile.name,
      mode: 'default',
    });
    const store = agentStoreFor(identity.agentId);
    await Promise.all([
      store.writeProfile(profile),
      store.writeSettings(settings),
    ]);
  }

  function rememberAgentPeer(operationId: string, peerKey: string | null | undefined) {
    if (peerKey) agentPeerKeyRef.current[operationId] = peerKey;
  }

  function flushPendingAgentDelta(operationId?: string) {
    const pending = pendingAgentDeltaRef.current;
    const operationIds = operationId ? [operationId] : [...pending.keys()];
    for (const id of operationIds) {
      const event = pending.get(id);
      if (!event) continue;
      pending.delete(id);
      appendAssistantTurnEvent(event);
    }
    if (pending.size === 0 && agentDeltaFrameRef.current !== null) {
      window.cancelAnimationFrame(agentDeltaFrameRef.current);
      agentDeltaFrameRef.current = null;
    }
  }

  function queueAgentDelta(event: Extract<RuntimeEvent, { type: 'chat.delta' }> & { operationId: string }) {
    const current = pendingAgentDeltaRef.current.get(event.operationId);
    pendingAgentDeltaRef.current.set(event.operationId, current
      ? { ...event, delta: `${current.delta}${event.delta}` }
      : event);
    if (agentDeltaFrameRef.current !== null) return;
    agentDeltaFrameRef.current = window.requestAnimationFrame(() => {
      agentDeltaFrameRef.current = null;
      flushPendingAgentDelta();
    });
  }

  function updateAgentThread(operationId: string | undefined, update: (current: DisplayMessage[]) => DisplayMessage[]) {
    const peerKey = operationId ? agentPeerKeyRef.current[operationId] : undefined;
    if (peerKey && peerKey !== activePeerKeyRef.current) {
      botThreadsRef.current[peerKey] = update(botThreadsRef.current[peerKey] ?? []);
      return;
    }
    setMessages(update);
  }

  function rekeyAssistantTurn(turn: AssistantTurn, operationId: string): AssistantTurn {
    if (turn.operationId === operationId) return turn;
    const previousPrefix = `${turn.operationId}:`;
    const nextPrefix = `${operationId}:`;
    return {
      ...turn,
      id: `assistant-turn:${operationId}`,
      operationId,
      parts: turn.parts.map((part) => ({
        ...part,
        id: part.id.startsWith(previousPrefix)
          ? `${nextPrefix}${part.id.slice(previousPrefix.length)}`
          : part.id,
      })),
    };
  }

  function adoptAgentRequestOperation(requestId: string | null, operationId: string, peerKey?: string | null) {
    if (!requestId || requestId === operationId) return;
    rememberAgentPeer(operationId, peerKey ?? agentPeerKeyRef.current[requestId] ?? agentRequestPeerRef.current);
    updateAgentThread(requestId, (current) => {
      const alreadyAuthoritative = current.some((message) =>
        message.kind === 'assistant-turn' && message.operationId === operationId,
      );
      return current.flatMap((message) => {
        if (message.operationId !== requestId) return [message];
        if (message.kind === 'assistant-turn' && message.assistantTurn) {
          if (alreadyAuthoritative) return [];
          const assistantTurn = rekeyAssistantTurn(message.assistantTurn, operationId);
          return [{
            ...message,
            id: `${operationId}:assistant-turn`,
            operationId,
            text: assistantTurnPlainText(assistantTurn),
            assistantTurn,
          }];
        }
        return [{ ...message, operationId }];
      });
    });
    delete agentPeerKeyRef.current[requestId];
  }

  function rememberActiveBotThread() {
    const peerKey = activePeerKeyRef.current;
    const peer = peersRef.current.find((candidate) => candidate.key === peerKey);
    if (!peerKey || !peer || !isAgentPeer(peer) || peer.miniAppId) return;
    botThreadsRef.current[peerKey] = messages;
  }

  function syncAgentOperationSnapshot() {
    setAgentOperationByPeer({ ...agentOperationRegistryRef.current.snapshot() });
  }

  function unambiguousAgentOperationId(): string | undefined {
    const running = [...new Set(Object.values(agentOperationRegistryRef.current.snapshot()))];
    if (running.length === 1) return running[0];
    if (running.length > 1) return undefined;
    return agentOperationIdRef.current ?? agentRequestIdRef.current ?? undefined;
  }

  function projectActiveAgentOperation(peerKey: string | null | undefined) {
    const operationId = agentOperationRegistryRef.current.operationForPeer(peerKey);
    const requestId = agentOperationRegistryRef.current.requestForPeer(peerKey);
    agentOperationIdRef.current = operationId;
    setAgentOperationId(operationId);
    agentRequestPendingRef.current = Boolean(requestId);
    agentRequestPeerRef.current = requestId ? peerKey ?? null : null;
    agentRequestIdRef.current = requestId;
    setPendingSend(Boolean(operationId || requestId));
  }

  function claimAgentOperation(operationId?: string): boolean {
    if (!operationId || finishedAgentOperationsRef.current.has(operationId)) return false;
    const registry = agentOperationRegistryRef.current;
    let peerKey = registry.peerForOperation(operationId)
      ?? agentPeerKeyRef.current[operationId]
      ?? null;

    if (!peerKey && agentRequestIdRef.current) {
      peerKey = registry.peerForRequest(agentRequestIdRef.current)
        ?? agentPeerKeyRef.current[agentRequestIdRef.current]
        ?? agentRequestPeerRef.current;
    }
    if (!peerKey) return false;

    const requestId = registry.requestForPeer(peerKey);
    rememberAgentPeer(operationId, peerKey);
    if (requestId) adoptAgentRequestOperation(requestId, operationId, peerKey);
    registry.adoptOperation(requestId, operationId, peerKey);
    syncAgentOperationSnapshot();

    if (activePeerKeyRef.current === peerKey) {
      agentOperationIdRef.current = operationId;
      setAgentOperationId(operationId);
      agentRequestPendingRef.current = false;
      agentRequestPeerRef.current = null;
      agentRequestIdRef.current = null;
      setPendingSend(true);
    }
    return true;
  }

  function clearAgentOperation(operationId: string, terminalStatus: 'completed' | 'failed' | 'interrupted' = 'completed') {
    const registry = agentOperationRegistryRef.current;
    const peerKey = registry.finishOperation(operationId)
      ?? registry.peerForOperation(operationId)
      ?? agentPeerKeyRef.current[operationId]
      ?? null;
    if (!peerKey && agentOperationIdRef.current !== operationId) return false;

    finishedAgentOperationsRef.current.add(operationId);
    if (finishedAgentOperationsRef.current.size > 500) {
      finishedAgentOperationsRef.current.delete(finishedAgentOperationsRef.current.values().next().value!);
    }
    syncAgentOperationSnapshot();

    if (agentOperationIdRef.current === operationId || activePeerKeyRef.current === peerKey) {
      projectActiveAgentOperation(activePeerKeyRef.current);
    }

    updateAgentThread(operationId, (current) => current
      .filter((message) => !(message.kind === 'thinking' && message.operationId === operationId))
      .map((message) => {
        if (message.kind === 'action' &&
          message.operationId === operationId &&
          message.actionStatus === 'running') {
          return { ...message, actionStatus: terminalStatus };
        }
        if (message.kind === 'message' && message.operationId === operationId && message.streaming) {
          return { ...message, streaming: false, optimistic: false };
        }
        return message;
      }));
    delete agentPeerKeyRef.current[operationId];
    return true;
  }

  function appendAgentThinking(operationId: string, label: string) {
    updateAgentThread(operationId, (current) => [
      ...current.filter((message) => !(message.kind === 'thinking' && message.operationId === operationId)),
      {
        id: `${operationId}:thinking`,
        source: 'legacy',
        role: 'peer',
        text: '',
        createdAtMs: Date.now(),
        kind: 'thinking',
        operationId,
        actionTitle: label || '正在思考',
        actionStatus: 'running',
      },
    ]);
  }

  function upsertAgentAction(input: {
    id: string;
    operationId: string;
    title: string;
    detail?: string;
    status: 'running' | 'completed' | 'failed';
  }) {
    updateAgentThread(input.operationId, (current) => {
      const next: DisplayMessage = {
        id: input.id,
        source: 'legacy',
        role: 'peer',
        text: '',
        createdAtMs: Date.now(),
        kind: 'action',
        operationId: input.operationId,
        actionTitle: input.title,
        actionDetail: input.detail,
        actionStatus: input.status,
      };
      const index = current.findIndex((message) => message.kind === 'action' && message.id === input.id);
      if (index < 0) return [...current, next];
      return current.map((message, messageIndex) => messageIndex === index ? { ...message, ...next, createdAtMs: message.createdAtMs } : message);
    });
  }

  function appendAssistantTurnEvent(event: RuntimeEvent) {
    const operationId = 'operationId' in event && typeof event.operationId === 'string'
      ? event.operationId
      : undefined;
    if (!operationId) return;
    updateAgentThread(operationId, (current) => {
      const index = current.findIndex((message) =>
        message.kind === 'assistant-turn' && message.operationId === operationId,
      );
      const existingTurn = index >= 0 ? current[index]?.assistantTurn : undefined;
      const assistantTurn = reduceAssistantTurn(existingTurn ?? createAssistantTurn(operationId), event);
      const next: DisplayMessage = {
        id: `${operationId}:assistant-turn`,
        source: 'legacy',
        role: 'peer',
        text: assistantTurnPlainText(assistantTurn),
        createdAtMs: assistantTurn.createdAtMs,
        kind: 'assistant-turn',
        operationId,
        streaming: assistantTurn.status === 'running',
        assistantTurn,
      };
      if (index < 0) return [...current, next];
      return current.map((message, messageIndex) => messageIndex === index ? { ...message, ...next, createdAtMs: message.createdAtMs } : message);
    });
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

    // Keep stream ordering deterministic while avoiding one React update per
    // token. Grok App uses the same backpressure idea: high-frequency deltas
    // are coalesced, but we flush them before any ordered step/final/terminal
    // event so tools and the final answer never overtake visible text.
    if (
      event.type === 'chat.message'
      || event.type === 'agent.step'
      || event.type === 'operation.completed'
      || event.type === 'operation.failed'
      || event.type === 'operation.interrupted'
    ) {
      const operationId = ('operationId' in event && typeof event.operationId === 'string'
        ? event.operationId
        : unambiguousAgentOperationId()) ?? undefined;
      if (operationId) flushPendingAgentDelta(operationId);
    }

    switch (event.type) {
      case 'host.ready':
        setHostReady(true);
        break;
      case 'conversation.listed':
        initialLegacyHydrationMaskRef.current |= 0b001;
        setInitialLegacyHydrationMask(initialLegacyHydrationMaskRef.current);
        setConversations(event.conversations);
        if (!activePeerKeyRef.current && event.conversations[0]) {
          const conversation = event.conversations[0];
          setActivePeerKey(`legacy:conversation:${conversation.id}`);
          setMessages(cachedLegacyDisplayMessages(conversation.id));
          void execute({ type: 'conversation.open', requestId: nextRequestId('conversation-open'), conversationId: conversation.id });
        }
        break;
      case 'conversation.opened':
        if (agentOperationRegistryRef.current.operationForPeer(activePeerKeyRef.current)) break;
        if (activePeerKeyRef.current === `legacy:conversation:${event.conversationId}`
          || peersRef.current.some((peer) => peer.key === activePeerKeyRef.current
            && peer.source === 'legacy'
            && peer.conversationId === event.conversationId)) {
          setMessages(event.messages.map((message) => ({
            id: message.id,
            source: 'legacy',
            role: message.role === 'user' ? 'me' : 'peer',
            text: message.text,
            createdAtMs: message.createdAtMs,
          })));
        }
        break;
      case 'memory.changed':
        // Fabu persists Agent-owned memory independently of chat delivery.
        // Re-list after every mutation so removals/clears also produce a
        // complete cloud snapshot instead of an append-only partial mirror.
        void execute({
          type: 'memory.list',
          requestId: nextRequestId('memory-cloud-sync'),
          agentId: event.agentId,
          limit: 1000,
        }).catch(() => {});
        break;
      case 'memory.listed':
        void mirrorAgentCloudSnapshot(event.agentId, 'memory/index.json', {
          version: 1,
          count: event.count,
          memories: event.memories,
        });
        break;
      case 'workflow.changed':
        void execute({
          type: 'workflow.list',
          requestId: nextRequestId('workflow-cloud-sync'),
          agentId: event.agentId,
        }).catch(() => {});
        break;
      case 'workflow.listed':
        void mirrorAgentCloudSnapshot(event.agentId, 'workflows/index.json', {
          version: 1,
          workflows: event.workflows,
        });
        break;
      case 'automation.changed': {
        const owner = event.automation.agentId;
        if (owner) {
          const path = `automations/${agentStorePathSegment(event.automation.id)}/automation.json`;
          if (event.action === 'deleted') void removeAgentCloudObject(owner, path);
          else void mirrorAgentCloudSnapshot(owner, path, event.automation);
        }
        break;
      }
      case 'automation.listed':
        for (const automation of event.automations) {
          if (!automation.agentId) continue;
          void mirrorAgentCloudSnapshot(
            automation.agentId,
            `automations/${agentStorePathSegment(automation.id)}/automation.json`,
            automation,
          );
        }
        break;
      case 'bot.listed':
        initialLegacyHydrationMaskRef.current |= 0b010;
        setInitialLegacyHydrationMask(initialLegacyHydrationMaskRef.current);
        setBots(event.bots);
        for (const bot of event.bots) {
          void mirrorBotAgentCloud(bot).catch(() => {});
        }
        break;
      case 'bot.changed':
        setBots((current) => event.action === 'deleted'
          ? current.filter((bot) => bot.id !== event.bot.id)
          : upsertById(current, event.bot));
        if (event.action === 'created' && newAgentRequestPendingRef.current) {
          newAgentRequestPendingRef.current = false;
          setPendingOpenAgentId(event.bot.agentId ?? event.bot.id);
        }
        // Local Host profile remains usable offline, while the account service
        // mirrors the complete Bot profile and its explicit Agent binding.
        // Deleting a Bot membership intentionally leaves the Agent store intact,
        // matching the Grok-style separation between surface and Agent state.
        if (event.action === 'deleted') {
          void invokeNativeDesktop('removeBotFromAccount', { botId: event.bot.id }).catch(() => {});
        } else {
          void mirrorBotAgentCloud(event.bot).catch(() => {
            // Local Host state stays authoritative while account sync is unavailable.
            // A later account reconciliation retries the durable cloud projection.
          });
        }
        break;
      case 'group.listed':
        initialLegacyHydrationMaskRef.current |= 0b100;
        setInitialLegacyHydrationMask(initialLegacyHydrationMaskRef.current);
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
      case 'settings.changed':
        setHostSettings(event.settings);
        break;
      case 'chat.message':
        if (event.role === 'assistant') {
          // Some legacy adapters omit operationId on the assistant body even
          // though the active send already owns a provisional/authoritative turn.
          // Bind that body to the current turn instead of painting a second
          // peer message beside the AssistantTurn.
          const assistantOperationId = event.operationId ?? unambiguousAgentOperationId();
          if (assistantOperationId) {
            const ownedOperation = claimAgentOperation(assistantOperationId)
              || Boolean(agentPeerKeyRef.current[assistantOperationId]);
            if (ownedOperation) {
              appendAssistantTurnEvent({ ...event, operationId: assistantOperationId });
              break;
            }
            if (finishedAgentOperationsRef.current.has(assistantOperationId)) break;
          }
        }
        updateAgentThread(event.operationId, (current) => {
          if (event.role === 'user') {
            const optimisticIndex = current.findIndex((message) =>
              message.role === 'me' && message.optimistic === true && message.text === event.text,
            );
            if (optimisticIndex >= 0) {
              return current.map((message, messageIndex) => messageIndex === optimisticIndex
                ? { ...message, optimistic: false, operationId: event.operationId ?? message.operationId }
                : message);
            }
            if (current.some((message) =>
              message.role === 'me' && message.text === event.text &&
              (!event.operationId || message.operationId === event.operationId))) return current;
          }
          return [...current, {
            id: nextRequestId('message'),
            source: 'legacy',
            role: event.role === 'user' ? 'me' : 'peer',
            text: event.text,
            createdAtMs: Date.now(),
            kind: 'message',
            operationId: event.operationId,
            streaming: false,
          }];
        });
        break;
      case 'chat.delta': {
        const deltaOperationId = event.operationId ?? unambiguousAgentOperationId();
        if (deltaOperationId && finishedAgentOperationsRef.current.has(deltaOperationId)) break;
        if (deltaOperationId && (
          claimAgentOperation(deltaOperationId)
          || Boolean(agentPeerKeyRef.current[deltaOperationId])
        )) {
          queueAgentDelta({ ...event, operationId: deltaOperationId });
          break;
        }
        updateAgentThread(deltaOperationId, (current) => {
          const index = current.findIndex((message) => message.kind === 'message' && message.operationId === deltaOperationId && message.streaming);
          if (index < 0) return [...current, { id: `${deltaOperationId ?? 'legacy'}:stream`, source: 'legacy', role: 'peer', text: event.delta, createdAtMs: Date.now(), kind: 'message', operationId: deltaOperationId, streaming: true }];
          return current.map((message, messageIndex) => messageIndex === index ? { ...message, text: `${message.text}${event.delta}`, kind: 'message', streaming: true } : message);
        });
        break;
      }
      case 'operation.started':
        if (claimAgentOperation(event.operationId)) appendAssistantTurnEvent(event);
        break;
      case 'model.routed':
        if (claimAgentOperation(event.operationId)) appendAssistantTurnEvent(event);
        break;
      case 'agent.step':
        if (claimAgentOperation(event.operationId)) appendAssistantTurnEvent(event);
        break;
      case 'operation.interrupted':
        if (agentOperationRegistryRef.current.peerForOperation(event.operationId)
          || agentPeerKeyRef.current[event.operationId]) {
          appendAssistantTurnEvent(event);
        }
        if (clearAgentOperation(event.operationId, 'interrupted')) agentSubmissionQueue.flush();
        break;
      case 'operation.completed':
        if (agentOperationRegistryRef.current.peerForOperation(event.operationId)
          || agentPeerKeyRef.current[event.operationId]) {
          appendAssistantTurnEvent(event);
        }
        if (clearAgentOperation(event.operationId)) agentSubmissionQueue.flush();
        break;
      case 'miniapp.opened':
        if (event.html) {
          const title = miniAppIdentityCatalog.find((app) => app.pluginId === event.miniAppId)?.displayName ?? marketplaceApps.find((app) => app.pluginId === event.miniAppId)?.displayName ?? event.miniAppId;
          void showMiniAppDocument(event.miniAppId, title, event.html).catch((cause) => {
            setError(cause instanceof Error ? cause.message : String(cause));
          });
        }
        break;
      case 'operation.failed': {
        const owner = agentOperationRegistryRef.current.peerForOperation(event.operationId)
          ?? agentPeerKeyRef.current[event.operationId]
          ?? null;
        if (owner) appendAssistantTurnEvent(event);
        if (clearAgentOperation(event.operationId, 'failed')) {
          if (owner === activePeerKeyRef.current) setError(event.message);
          agentSubmissionQueue.flush();
        }
        break;
      }
      case 'host.closed':
        agentOperationRegistryRef.current.clear();
        syncAgentOperationSnapshot();
        projectActiveAgentOperation(activePeerKeyRef.current);
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
    const botByConversationId = new Map(
      bots
        .filter((bot) => Boolean(bot.conversationId))
        .map((bot) => [bot.conversationId!, bot] as const),
    );
    const legacyConversations = conversations.map((conversation): PeerItem => {
      const bot = botByConversationId.get(conversation.id);
      const kind = bot ? 'bot' : legacyKind(conversation);
      return {
      key: `legacy:conversation:${conversation.id}`,
      id: conversation.id,
      source: 'legacy',
      conversationId: conversation.id,
      actorId: bot?.id,
      agentId: bot?.agentId ?? bot?.id,
      kind,
      hidden: bot?.hidden,
      title: conversation.title,
      subtitle: kind === 'bot' ? bot?.description || bot?.title || 'AI Bot' : conversation.kind,
      unread: conversation.unreadCount,
      pinned: conversation.pinned || pinnedPeerKeys.has(`legacy:conversation:${conversation.id}`),
      archived: archivedPeerKeys.has(`legacy:conversation:${conversation.id}`),
      updatedAtMs: conversation.updatedAtMs,
      };
    });
    const miniAppBotProjections = installedMiniAppBotProjections(miniAppIdentityCatalog, installedMiniApps);
    const miniAppByBotId = new Map(miniAppBotProjections.map((projection) => [projection.id, projection]));
    const conversationIds = new Set(conversations.map((conversation) => conversation.id));
    const botPeers = bots
      .filter((bot) => !bot.conversationId || !conversationIds.has(bot.conversationId))
      .map((bot): PeerItem => ({
        key: `legacy:bot:${bot.id}`,
        id: bot.id,
        source: 'legacy',
        actorId: bot.id,
        agentId: bot.agentId ?? bot.id,
        conversationId: bot.conversationId,
        kind: 'bot',
        hidden: bot.hidden,
        title: bot.name,
        subtitle: bot.description || bot.title || 'AI Bot',
        unread: bot.unread ? 1 : 0,
        pinned: pinnedPeerKeys.has(`legacy:bot:${bot.id}`),
        archived: archivedPeerKeys.has(`legacy:bot:${bot.id}`),
        updatedAtMs: 0,
        avatar: bot.avatar,
        miniAppId: miniAppByBotId.get(bot.id)?.miniAppId,
        miniAppCommands: miniAppByBotId.get(bot.id)?.commands,
        miniAppMenuButtonText: miniAppByBotId.get(bot.id)?.menuButtonText,
        miniAppCalls: miniAppByBotId.get(bot.id)?.calls,
      }));
    const existingBotIds = new Set(botPeers.map((peer) => peer.actorId ?? peer.id));
    const accountBotPeers = accountBots
      .filter((entry) => entry?.bot?.id && !existingBotIds.has(entry.bot.id))
      .map((entry): PeerItem => {
        const miniAppSource = entry.sources.find((source) => source.source === 'miniapp');
        const projection = miniAppSource
          ? miniAppBotProjections.find((candidate) => candidate.miniAppId === miniAppSource.sourceId)
          : miniAppByBotId.get(entry.bot.id);
        return {
          // A Mini App-backed account Bot must keep the same peer identity before and
          // after the asynchronous account-membership projection arrives. Otherwise
          // activePeerKey points at the retired synthetic peer and the composer vanishes.
          key: miniAppSource ? `miniapp:bot:${miniAppSource.sourceId}` : `account:bot:${entry.bot.id}`,
          id: entry.bot.id,
          source: 'legacy',
          kind: 'bot',
          hidden: entry.bot.hidden,
          title: entry.bot.displayName ?? entry.bot.username ?? entry.bot.id,
          subtitle: entry.bot.username ? `@${entry.bot.username}` : entry.bot.description ?? 'Bot',
          actorId: entry.bot.id,
          agentId: entry.bot.agentId ?? entry.bot.id,
          conversationId: entry.bot.conversationId,
          unread: 0,
          pinned: pinnedPeerKeys.has(miniAppSource ? `miniapp:bot:${miniAppSource.sourceId}` : `account:bot:${entry.bot.id}`),
          archived: archivedPeerKeys.has(miniAppSource ? `miniapp:bot:${miniAppSource.sourceId}` : `account:bot:${entry.bot.id}`),
          updatedAtMs: entry.updatedAtMs ?? 0,
          miniAppId: miniAppSource?.sourceId,
          miniAppCommands: projection?.commands,
          miniAppMenuButtonText: projection?.menuButtonText ?? (miniAppSource ? '打开小程序' : undefined),
          miniAppCalls: projection?.calls,
        };
      });
    for (const peer of accountBotPeers) existingBotIds.add(peer.actorId ?? peer.id);
    const miniAppBotPeers = miniAppBotProjections
      .filter((projection) => !existingBotIds.has(projection.id))
      .map((projection): PeerItem => ({
        key: `miniapp:bot:${projection.miniAppId}`,
        id: projection.id,
        source: 'legacy',
        actorId: projection.id,
        agentId: projection.id,
        conversationId: projection.conversationId,
        kind: 'bot',
        title: projection.displayName,
        subtitle: projection.username ? `@${projection.username} · ${projection.description}` : projection.description,
        unread: 0,
        pinned: pinnedPeerKeys.has(`miniapp:bot:${projection.miniAppId}`),
        archived: archivedPeerKeys.has(`miniapp:bot:${projection.miniAppId}`),
        updatedAtMs: 0,
        miniAppId: projection.miniAppId,
        miniAppCommands: projection.commands,
        miniAppMenuButtonText: projection.menuButtonText,
        miniAppCalls: projection.calls,
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
    const actorById = new Map(selfActors.map((actor) => [actor.id, actor] as const));
    const nativePeers = selfConversations.map((conversation): PeerItem => {
      const actorId = conversation.participants.length === 2
        ? conversation.participants.find((participant) => participant.actorId !== selfHosted.actorId)?.actorId
        : undefined;
      const actor = actorId ? actorById.get(actorId) : undefined;
      const kind = selfKind(conversation, actor);
      return {
        key: `selfhosted:${conversation.id}`,
        id: conversation.id,
        source: 'selfhosted',
        conversationId: conversation.id,
        actorId,
        agentId: kind === 'bot' ? actorId : undefined,
        kind,
        title: conversation.title,
        subtitle: conversation.description || ({
          channel: '频道',
          group: '群组',
          saved: '收藏消息',
          conversation: '会话',
          contact: '联系人',
          bot: 'AI Bot',
        } as const)[kind],
        unread: conversation.unreadCount,
        pinned: conversation.pinned,
        archived: conversation.archived,
        updatedAtMs: conversation.updatedAtMs,
        avatar: conversation.avatarUrl,
      };
    });
    return [...legacyConversations, ...botPeers, ...accountBotPeers, ...miniAppBotPeers, ...legacyGroups, ...nativePeers].sort((left, right) => {
      if (left.pinned !== right.pinned) return left.pinned ? -1 : 1;
      return right.updatedAtMs - left.updatedAtMs;
    });
  }, [conversations, bots, accountBots, groups, selfActors, selfConversations, pinnedPeerKeys, archivedPeerKeys, miniAppIdentityCatalog, installedMiniApps, selfHosted.actorId]);

  peersRef.current = peers;
  const grokAgentKey = (peer: PeerItem): string => peer.kind === 'group'
    ? `group:${peer.id}`
    : `agent:${peer.agentId ?? peer.actorId ?? peer.id}`;
  const grokAgentItems: GrokAgentSidebarItem[] = Array.from(
    peers
      .filter((peer) => peer.kind === 'bot' || peer.kind === 'group')
      .reduce((items, peer) => {
        const key = grokAgentKey(peer);
        const existing = items.get(key);
        const candidate: GrokAgentSidebarItem = {
          key,
          peerKey: peer.key,
          id: peer.id,
          agentId: peer.agentId ?? peer.actorId ?? peer.id,
          name: peer.title,
          description: peer.subtitle,
          pinned: peer.pinned,
          hidden: peer.hidden === true,
          unread: peer.unread,
          busy: Boolean(agentOperationByPeer[peer.key]),
          isGroup: peer.kind === 'group',
          updatedAtMs: peer.updatedAtMs,
        };
        // Prefer the concrete conversation projection over profile-only peers
        // so opening an Agent always restores its transcript. Otherwise keep
        // the freshest/pinned projection for the same runtime Agent identity.
        const candidateHasConversation = Boolean(peer.conversationId);
        const existingPeer = existing
          ? peers.find((entry) => entry.key === existing.peerKey)
          : undefined;
        const existingHasConversation = Boolean(existingPeer?.conversationId);
        if (!existing
          || (candidateHasConversation && !existingHasConversation)
          || (candidateHasConversation === existingHasConversation
            && (candidate.pinned && !existing.pinned
              || candidate.updatedAtMs > existing.updatedAtMs))) {
          items.set(key, candidate);
        } else if (existing) {
          items.set(key, {
            ...existing,
            pinned: existing.pinned || candidate.pinned,
            hidden: existing.hidden && candidate.hidden,
            unread: Math.max(existing.unread, candidate.unread),
            busy: existing.busy || candidate.busy,
            updatedAtMs: Math.max(existing.updatedAtMs, candidate.updatedAtMs),
          });
        }
        return items;
      }, new Map<string, GrokAgentSidebarItem>())
      .values(),
  );
  const activePeer = peers.find((peer) => peer.key === activePeerKey) ?? null;
  const activeGrokAgentKey = activePeer && (activePeer.kind === 'bot' || activePeer.kind === 'group')
    ? grokAgentKey(activePeer)
    : null;

  useEffect(() => {
    if (!pendingOpenAgentId) return;
    const peer = peers.find((candidate) =>
      candidate.kind === 'bot'
      && (candidate.agentId ?? candidate.actorId ?? candidate.id) === pendingOpenAgentId,
    );
    if (!peer) return;
    setPendingOpenAgentId(null);
    setSection('bots');
    void openPeer(peer);
  }, [pendingOpenAgentId, peers]);

  const activeTypingActors = activePeer?.source === 'selfhosted' && activePeer.conversationId
    ? Object.keys(typingByConversation[activePeer.conversationId] ?? {})
    : [];
  const visiblePeers = peers.filter((peer) => {
    if (['chats', 'contacts', 'bots', 'groups', 'channels', 'saved', 'archive'].includes(section) && !matchesSection(peer, section)) return false;
    const query = search.trim().toLowerCase();
    return !query || `${peer.title} ${peer.subtitle}`.toLowerCase().includes(query);
  });
  const sectionIsPeerList = ['chats', 'contacts', 'bots', 'groups', 'channels', 'saved', 'archive'].includes(section);
  const layoutInfoOpen = wideInfoLayout ? infoOpen : narrowInfoOpen;
  const infoPanelVisible = Boolean(layoutInfoOpen && activePeer && sectionIsPeerList);
  const infoPanelDocked = Boolean(infoPanelVisible && wideInfoLayout);
  const currentActor = selfActors.find((actor) => actor.id === selfHosted.actorId);
  const localComputerOnline = Boolean(remoteComputerState?.running && remoteComputerState.registration);
  const localComputerStatus = remoteComputerState?.channelOpen
    ? '正在远程控制'
    : localComputerOnline
      ? hostSettings.remoteControlEnabled ? '在线，等待连接' : '在线，仅可发现'
      : remoteComputerState?.running ? '正在注册' : '离线';
  const renderedPeers = visiblePeers.slice(0, peerRenderCount);
  const renderedContactGroups = ['chats', 'contacts'].includes(section) && contactGroups.groups.length
    ? projectSidebarContactGroups(renderedPeers, contactGroups.groups, Boolean(search.trim()))
    : [];
  const matchingMessages = messages;
  const renderedMessages = matchingMessages.slice(Math.max(0, matchingMessages.length - messageRenderCount));
  const botTranscriptMessages: BotTranscriptMessage[] = activePeer && isAgentPeer(activePeer)
    ? [...renderedMessages, ...(queuedAgentPrompts[activePeer.key] ?? [])]
    : renderedMessages;
  const activeAgentOperationId = activePeer
    ? agentOperationByPeer[activePeer.key] ?? null
    : null;
  const activePeerBusy = Boolean(activePeer && (
    activeAgentOperationId
    || agentOperationRegistryRef.current.requestForPeer(activePeer.key)
  ));

  function renderPeerRow(peer: PeerItem) {
    const peerBusy = Boolean(agentOperationByPeer[peer.key]
      || agentOperationRegistryRef.current.requestForPeer(peer.key));
    return <button data-testid={`legacy-peer-${peer.key}`} key={peer.key} type="button" className={peer.key === activePeerKey ? styles.peerActive : styles.peer} onClick={() => void openPeer(peer)}>
      <BotMark
        botId={`peer:${peer.kind}:${peer.actorId ?? peer.id}`}
        state={isAgentPeer(peer) ? botMarkStateForPeer(peer, selfBotExecutions, peerBusy, hostReady) : peer.unread ? 'notifying' : 'idle'}
        size={48}
        className={styles.agentAvatarMark}
        label={peer.title}
      />
      <span className={styles.peerCopy}>
        <span><strong>{peer.title}</strong><time>{formatTime(peer.updatedAtMs)}</time></span>
        <small>{desktopPreferences.messagePreview ? peer.subtitle : peer.unread ? '有新消息' : '消息预览已关闭'}</small>
      </span>
      <span className={styles.peerMeta}>{peer.pinned ? <Pin size={12} /> : null}{mutedPeerKeys.has(peer.key) ? <BellOff size={12} /> : null}{peer.unread ? <b>{peer.unread}</b> : null}</span>
    </button>;
  }

  useEffect(() => {
    setPeerRenderCount(initialPeerRenderCount);
  }, [section, search]);

  async function createGrokAgent(): Promise<void> {
    setSection('bots');
    setGlobalSearchOpen(false);
    setSearch('');
    newAgentRequestPendingRef.current = true;
    const accepted = await execute({
      type: 'bot.create',
      requestId: nextRequestId('grok-new-agent'),
      name: 'New chat',
      description: '',
    });
    if (!accepted) newAgentRequestPendingRef.current = false;
  }

  function peerForGrokAgent(item: GrokAgentSidebarItem): PeerItem | undefined {
    return peersRef.current.find((peer) => peer.key === item.peerKey)
      ?? peersRef.current.find((peer) =>
        (peer.kind === 'bot' || peer.kind === 'group') && grokAgentKey(peer) === item.key,
      );
  }

  async function renameGrokAgent(item: GrokAgentSidebarItem): Promise<void> {
    const peer = peerForGrokAgent(item);
    if (!peer || peer.kind !== 'bot' || !peer.key.startsWith('legacy:bot:') && !peer.key.startsWith('legacy:conversation:')) {
      setError('Only locally managed Agents can be renamed from this shell.');
      return;
    }
    const name = window.prompt('Rename Agent', peer.title)?.trim();
    if (!name || name === peer.title) return;
    await execute({
      type: 'bot.update',
      requestId: nextRequestId('grok-rename-agent'),
      id: peer.actorId ?? peer.id,
      name,
    });
  }

  async function duplicateGrokAgent(item: GrokAgentSidebarItem): Promise<void> {
    const peer = peerForGrokAgent(item);
    const botId = peer?.source === 'legacy' && peer.kind === 'bot'
      ? peer.actorId ?? (peer.key.startsWith('legacy:bot:') ? peer.id : undefined)
      : undefined;
    if (!peer || !botId) {
      setError('Only locally managed Agents can be duplicated from this shell.');
      return;
    }
    await execute({
      type: 'bot.clone',
      requestId: nextRequestId('grok-duplicate-agent'),
      id: botId,
    });
  }

  async function deleteGrokAgent(item: GrokAgentSidebarItem): Promise<void> {
    const peer = peerForGrokAgent(item);
    const botId = peer?.source === 'legacy' && peer.kind === 'bot'
      ? peer.actorId ?? (peer.key.startsWith('legacy:bot:') ? peer.id : undefined)
      : undefined;
    if (!peer || !botId) {
      setError('Only locally managed Agents can be deleted from this shell.');
      return;
    }
    if (!window.confirm(`Delete “${peer.title}”? The Agent store is retained for recovery.`)) return;
    agentOperationRegistryRef.current.clearPeer(peer.key);
    syncAgentOperationSnapshot();
    await execute({
      type: 'bot.delete',
      requestId: nextRequestId('grok-delete-agent'),
      id: botId,
    });
    if (peer.key === activePeerKeyRef.current) {
      activePeerKeyRef.current = null;
      setActivePeerKey(null);
      setMessages([]);
      projectActiveAgentOperation(null);
    }
  }

  async function hideGrokAgent(item: GrokAgentSidebarItem): Promise<void> {
    const peer = peerForGrokAgent(item);
    const botId = peer?.source === 'legacy' && peer.kind === 'bot'
      ? peer.actorId ?? (peer.key.startsWith('legacy:bot:') ? peer.id : undefined)
      : undefined;
    if (!peer || !botId) {
      setError('Only locally managed Agents can be hidden from this shell.');
      return;
    }
    await execute({
      type: 'bot.setHidden',
      requestId: nextRequestId('grok-hide-agent'),
      id: botId,
      hidden: true,
    });
    if (peer.key === activePeerKeyRef.current) {
      setActivePeerKey(null);
      setMessages([]);
    }
  }

  function openGrokAgent(item: GrokAgentSidebarItem): void {
    const peer = peerForGrokAgent(item);
    if (!peer) return;
    setSection('bots');
    void openPeer(peer);
  }

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

  function handleMessageAreaScroll(event: React.UIEvent<HTMLDivElement>) {
    const node = event.currentTarget;
    const distanceFromLatest = node.scrollHeight - node.scrollTop - node.clientHeight;
    const atLatest = distanceFromLatest < 96;
    stickToLatestRef.current = atLatest;
    setShowScrollToLatest(!atLatest);
  }

  function scrollToLatest() {
    const node = messageAreaRef.current;
    if (!node) return;
    stickToLatestRef.current = true;
    setShowScrollToLatest(false);
    node.scrollTo({ top: node.scrollHeight, behavior: 'smooth' });
  }

  async function copyBotMessage(message: BotTranscriptMessage): Promise<void> {
    try {
      await navigator.clipboard?.writeText(message.text);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : '复制消息失败');
    }
  }

  function editBotMessage(message: BotTranscriptMessage) {
    updateComposer(message.text);
    setReplyTo(null);
  }

  async function dispatchAgentPromptNow(peer: PeerItem, text: string, existingMessageId?: string): Promise<void> {
    const registry = agentOperationRegistryRef.current;
    if (registry.isBusy(peer.key)) {
      throw new Error('This Agent is already busy.');
    }
    const requestId = nextRequestId('chat-send');
    registry.beginRequest(peer.key, requestId);
    syncAgentOperationSnapshot();
    rememberAgentPeer(requestId, peer.key);
    const optimisticId = existingMessageId ?? 'optimistic:' + requestId;
    updateAgentThread(requestId, (current) => {
      const nextMessage: DisplayMessage = {
        id: optimisticId,
        source: 'legacy',
        role: 'me',
        text,
        createdAtMs: Date.now(),
        kind: 'message',
        operationId: requestId,
        optimistic: true,
        queued: false,
      };
      const existingIndex = current.findIndex((message) => message.id === optimisticId);
      if (existingIndex < 0) return [...current, nextMessage];
      return current.map((message, index) => index === existingIndex
        ? { ...message, ...nextMessage }
        : message);
    });
    // Paint the assistant turn locally in the same renderer tick as the user
    // bubble. The Host operation id is adopted later without creating a second
    // "thinking" row, so slow process wake-up never leaves the chat visually idle.
    appendAssistantTurnEvent({
      type: 'operation.started',
      timestamp: new Date().toISOString(),
      operationId: requestId,
      label: '正在思考',
      interruptible: true,
    });
    if (activePeerKeyRef.current === peer.key) {
      setPendingSend(true);
      agentRequestPendingRef.current = true;
      agentRequestPeerRef.current = peer.key;
      agentRequestIdRef.current = requestId;
    }
    try {
      const accepted = await execute({
        type: 'chat.send',
        requestId,
        text,
        conversationId: peer.conversationId,
        agentId: peer.agentId ?? peer.actorId,
      });
      if (!accepted) {
        updateAgentThread(requestId, (current) => current.filter((message) => message.id !== optimisticId));
        return;
      }
      const operationId = accepted.operationId ?? requestId;
      rememberAgentPeer(operationId, peer.key);
      if (finishedAgentOperationsRef.current.has(operationId)) {
        registry.cancelRequest(requestId);
        syncAgentOperationSnapshot();
        return;
      }
      adoptAgentRequestOperation(requestId, operationId, peer.key);
      registry.adoptOperation(requestId, operationId, peer.key);
      syncAgentOperationSnapshot();
      if (activePeerKeyRef.current === peer.key) {
        agentRequestIdRef.current = null;
      }
      updateAgentThread(operationId, (current) => current.map((message) => message.id === optimisticId
        ? { ...message, operationId, optimistic: false, queued: false }
        : message));
      claimAgentOperation(operationId);
    } catch (cause) {
      updateAgentThread(requestId, (current) => current.filter((message) =>
        message.id !== optimisticId && message.operationId !== requestId,
      ));
      registry.cancelRequest(requestId);
      syncAgentOperationSnapshot();
      if (activePeerKeyRef.current === peer.key) agentRequestIdRef.current = null;
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      if (activePeerKeyRef.current === peer.key) {
        agentRequestPendingRef.current = false;
        projectActiveAgentOperation(peer.key);
      }
    }
  }

  function enqueueAgentPrompt(peer: PeerItem, text: string, existingMessageId?: string) {
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    const messageId = existingMessageId ?? nextRequestId('queued-chat-send');
    agentSubmissionQueue.submit({
      nonce: nextRequestId('agent-submission'),
      messageId,
      peerKey: peer.key,
      agentId,
      prompt: text,
      createdAtMs: Date.now(),
    });
  }

  function regenerateBotMessage(message: BotTranscriptMessage) {
    if (!activePeer || !isAgentPeer(activePeer) || activePeer.miniAppId) return;
    const index = messages.findIndex((candidate) => candidate.id === message.id);
    if (index < 0) return;
    const prompt = [...messages.slice(0, index)].reverse().find((candidate) => candidate.role === 'me' && !candidate.queued);
    if (!prompt) {
      setError('找不到这条回复对应的用户消息。');
      return;
    }
    setMessages((current) => current.filter((candidate) => candidate.id !== message.id));
    enqueueAgentPrompt(activePeer, prompt.text);
  }

  async function stopAgentOperation(): Promise<void> {
    const operationId = agentOperationRegistryRef.current.operationForPeer(activePeerKeyRef.current)
      ?? agentOperationIdRef.current;
    if (!operationId) return;
    try {
      await transport.interrupt(operationId);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  useEffect(() => {
    if (!activePeer || !isAgentPeer(activePeer) || !stickToLatestRef.current) return;
    const frame = window.requestAnimationFrame(() => {
      const node = messageAreaRef.current;
      if (node) node.scrollTop = node.scrollHeight;
    });
    return () => window.cancelAnimationFrame(frame);
  }, [activePeerKey, messages, pendingSend, queuedAgentPrompts]);

  async function openPeer(peer: PeerItem) {
    rememberActiveBotThread();
    activePeerKeyRef.current = peer.key;
    setActivePeerKey(peer.key);
    if (isAgentPeer(peer)) projectActiveAgentOperation(peer.key);
    else {
      agentOperationIdRef.current = null;
      setAgentOperationId(null);
      setPendingSend(false);
      agentRequestPendingRef.current = false;
      agentRequestPeerRef.current = null;
      agentRequestIdRef.current = null;
    }
    stickToLatestRef.current = true;
    setShowScrollToLatest(false);
    setComposer(drafts[peer.key] ?? '');
    setSearch('');
    setGlobalSearchOpen(false);
    setMessageRenderCount(initialMessageRenderCount);
    setReplyTo(null);
    setError(null);
    setConversationSearchOpen(false);
    if (peer.miniAppId) {
      setMessages(miniAppBotThreadsRef.current[peer.miniAppId] ?? []);
      void loadMiniAppBotThread(peer.miniAppId).catch(() => {});
      return;
    }
    if (peer.source === 'selfhosted' && peer.conversationId) {
      showSelfConversation(peer.conversationId);
      const list = selfMessages[peer.conversationId] ?? [];
      const last = list.filter((message) => !message.deleted).at(-1);
      if (last) void selfHosted.markRead(peer.conversationId, last.id).catch(() => {});
      return;
    }
    if (isAgentPeer(peer)) {
      const remembered = botThreadsRef.current[peer.key];
      if (remembered) setMessages(remembered);
      else if (peer.conversationId) setMessages(cachedLegacyDisplayMessages(peer.conversationId));
      else setMessages([]);
      if (peer.conversationId) {
        await execute({ type: 'conversation.open', requestId: nextRequestId('conversation-open'), conversationId: peer.conversationId });
      }
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
      setMessages(cachedLegacyDisplayMessages(peer.conversationId));
      await execute({ type: 'conversation.open', requestId: nextRequestId('conversation-open'), conversationId: peer.conversationId });
      return;
    }
    setMessages([]);
  }

  async function sendMessage(event: FormEvent) {
    event.preventDefault();
    const text = composer.trim();
    if (!text || !activePeer) return;
    const agentRequest = activePeer.source === 'legacy'
      && activePeer.kind !== 'group'
      && !activePeer.miniAppId
      && isAgentPeer(activePeer);
    if (pendingSend && !agentRequest) return;
    if (agentRequest) {
      updateComposer('');
      enqueueAgentPrompt(activePeer, text);
      setReplyTo(null);
      setScheduledAtMs(undefined);
      return;
    }
    setPendingSend(true);
    updateComposer('');
    try {
      if (activePeer.miniAppId) {
        const createdAtMs = Date.now();
        const userMessage: DisplayMessage = {
          id: nextRequestId('miniapp-bot-user'),
          source: 'legacy',
          role: 'me',
          text,
          createdAtMs,
        };
        const pendingThread = [...(miniAppBotThreadsRef.current[activePeer.miniAppId] ?? []), userMessage];
        const thinkingMessage: DisplayMessage = {
          id: userMessage.id + ':thinking',
          source: 'legacy',
          role: 'peer',
          text: '',
          createdAtMs: Date.now(),
          kind: 'thinking',
          operationId: userMessage.id,
          actionTitle: '正在处理',
          actionDetail: '正在调用 Mini App…',
          actionStatus: 'running',
        };
        const pendingWithThinking = [...pendingThread, thinkingMessage];
        miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [activePeer.miniAppId]: pendingWithThinking };
        setMessages(pendingWithThinking);
        await appendMiniAppBotMessages(activePeer.miniAppId, [{
          messageId: userMessage.id,
          role: 'user',
          text: userMessage.text,
          createdAt: new Date(userMessage.createdAtMs).toISOString(),
        }]);
        const routed = await invokeNativeDesktop<Record<string, unknown>>('routeMiniAppInput', {
          pluginId: activePeer.miniAppId,
          input: text,
        });
        const executed = await executeDesktopMiniAppBotInput(activePeer.miniAppId, routed, text);
        const responseMessage: DisplayMessage = {
          id: nextRequestId('miniapp-bot-response'),
          source: 'legacy',
          role: 'peer',
          text: miniAppBotResponseText(executed),
          miniAppId: activePeer.miniAppId,
          createdAtMs: Date.now(),
        };
        const completedThread = [...pendingThread, responseMessage];
        miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [activePeer.miniAppId]: completedThread };
        if (activePeerKeyRef.current === activePeer.key) setMessages(completedThread);
        await appendMiniAppBotMessages(activePeer.miniAppId, [{
          messageId: responseMessage.id,
          role: 'assistant',
          text: responseMessage.text,
          payload: { miniAppId: activePeer.miniAppId },
          createdAt: new Date(responseMessage.createdAtMs).toISOString(),
        }]);
      } else if (activePeer.source === 'selfhosted' && activePeer.conversationId) {
        await selfHosted.sendText(activePeer.conversationId, text, {
          replyToMessageId: replyTo?.id,
          scheduledAtMs,
          silent: silentSend,
        });
      } else if (activePeer.kind === 'group' && activePeer.groupId) {
        await execute({ type: 'group.send', requestId: nextRequestId('group-send'), id: activePeer.groupId, text });
      } else {
        const requestId = nextRequestId('chat-send');
        const optimisticId = `optimistic:${requestId}`;
        setMessages((current) => [...current, {
          id: optimisticId,
          source: 'legacy',
          role: 'me',
          text,
          createdAtMs: Date.now(),
          kind: 'message',
          operationId: requestId,
          optimistic: true,
        }]);
        const accepted = await execute({
          type: 'chat.send',
          requestId,
          text,
          conversationId: activePeer.conversationId,
          agentId: activePeer.actorId,
        });
        if (!accepted) {
          setMessages((current) => current.filter((message) => message.id !== optimisticId));
          updateComposer(text);
          return;
        }
        if (accepted.operationId) {
          setMessages((current) => current.map((message) => message.id === optimisticId
            ? { ...message, operationId: accepted.operationId, optimistic: false, queued: false }
            : message));
        }
        if (agentRequest && accepted.operationId && claimAgentOperation(accepted.operationId)) {
          appendAssistantTurnEvent({ type: 'operation.started', timestamp: new Date().toISOString(), operationId: accepted.operationId, label: '正在思考', interruptible: true });
        }
      }
      setReplyTo(null);
      setScheduledAtMs(undefined);
    } catch (cause) {
      if (activePeer.miniAppId) {
        const thread = (miniAppBotThreadsRef.current[activePeer.miniAppId] ?? [])
          .filter((message) => !(message.kind === 'thinking' && message.operationId));
        miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [activePeer.miniAppId]: thread };
        if (activePeerKeyRef.current === activePeer.key) setMessages(thread);
      }
      updateComposer(text);
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setPendingSend(false);
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

  async function appendMiniAppCallExchange(miniAppId: string, input: string, visibleText = input): Promise<void> {
    const createdAtMs = Date.now();
    const userMessage: DisplayMessage = {
      id: nextRequestId('miniapp-call-user'),
      source: 'legacy',
      role: 'me',
      text: visibleText,
      createdAtMs,
    };
    const pendingThread = [...(miniAppBotThreadsRef.current[miniAppId] ?? []), userMessage];
    miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [miniAppId]: pendingThread };
    const active = peersRef.current.find((peer) => peer.key === activePeerKeyRef.current);
    if (active?.miniAppId === miniAppId) setMessages(pendingThread);
    await appendMiniAppBotMessages(miniAppId, [{
      messageId: userMessage.id,
      role: 'user',
      text: userMessage.text,
      createdAt: new Date(userMessage.createdAtMs).toISOString(),
      payload: { source: 'miniapp-call' },
    }]);
    const routed = await invokeNativeDesktop<Record<string, unknown>>('routeMiniAppInput', {
      pluginId: miniAppId,
      input,
    });
    const executed = await executeDesktopMiniAppBotInput(miniAppId, routed, input);
    const responseMessage: DisplayMessage = {
      id: nextRequestId('miniapp-call-response'),
      source: 'legacy',
      role: 'peer',
      text: miniAppBotResponseText(executed),
      createdAtMs: Date.now(),
    };
    const completedThread = [...pendingThread, responseMessage];
    miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [miniAppId]: completedThread };
    if (active?.miniAppId === miniAppId) setMessages(completedThread);
    await appendMiniAppBotMessages(miniAppId, [{
      messageId: responseMessage.id,
      role: 'assistant',
      text: responseMessage.text,
      createdAt: new Date(responseMessage.createdAtMs).toISOString(),
      payload: { source: 'miniapp-call' },
    }]);
  }

  async function runMiniAppCallCommand(session: MiniAppCallSession, command: string, args?: Record<string, unknown>): Promise<void> {
    const suffix = args && Object.keys(args).length ? ' ' + JSON.stringify(args) : '';
    const input = '/' + session.miniAppId + ':' + command + suffix;
    await appendMiniAppCallExchange(session.miniAppId, input, '通话服务 · /' + command);
  }

  async function saveMiniAppCallRecording(session: MiniAppCallSession, blob: Blob, mimeType: string): Promise<void> {
    const extension = mimeType.includes('mp4') ? 'mp4' : 'webm';
    const fileName = 'teleprompter-' + new Date().toISOString().replaceAll(':', '-') + '.' + extension;
    const file = new File([blob], fileName, { type: mimeType || 'video/webm' });
    setAttachmentProgress('正在保存 ' + fileName + '…');
    try {
      const media = await selfHosted.uploadBlob(file, (uploaded, total) => {
        setAttachmentProgress('正在保存 ' + fileName + ' · ' + Math.round((uploaded / Math.max(total, 1)) * 100) + '%');
      });
      const message: DisplayMessage = {
        id: nextRequestId('miniapp-call-video'),
        source: 'legacy',
        role: 'me',
        text: '口播录制视频',
        createdAtMs: Date.now(),
        media,
        mediaType: 'video',
      };
      const thread = [...(miniAppBotThreadsRef.current[session.miniAppId] ?? []), message];
      miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [session.miniAppId]: thread };
      const active = peersRef.current.find((peer) => peer.key === activePeerKeyRef.current);
      if (active?.miniAppId === session.miniAppId) setMessages(thread);
      await appendMiniAppBotMessages(session.miniAppId, [{
        messageId: message.id,
        role: 'user',
        text: message.text,
        createdAt: new Date(message.createdAtMs).toISOString(),
        payload: {
          source: 'miniapp-call-recording',
          miniAppId: session.miniAppId,
          callId: session.callId,
          mediaType: 'video',
          media,
        },
      }]);
    } finally {
      setAttachmentProgress(null);
    }
  }

  async function startCall(kind: 'voice' | 'video') {
    if (!activePeer) return;
    if (activePeer.miniAppId) {
      const program = activePeer.miniAppCalls?.[kind];
      if (!program) {
        setError('这个 Mini App 没有声明' + (kind === 'video' ? '视频' : '语音') + '通话程序。');
        return;
      }
      setError(null);
      try {
        let html: string | undefined;
        if (program.type === 'miniapp-surface') {
          const installed = installedMiniApps[activePeer.miniAppId] ?? await transport.pluginActive(activePeer.miniAppId);
          if (!installed) throw new Error('请先安装这个 Mini App，再使用它自定义的通话界面。');
          const document = await transport.pluginUiDocument(activePeer.miniAppId);
          html = prepareDesktopMiniAppWebMcpDocument(activePeer.miniAppId, document.html);
        }
        setMiniAppCall({
          callId: 'miniapp-call:' + crypto.randomUUID(),
          miniAppId: activePeer.miniAppId,
          title: activePeer.title,
          kind,
          program,
          html,
        });
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
      return;
    }
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

  async function loadMiniAppBotThread(miniAppId: string): Promise<DisplayMessage[]> {
    const beforeRead = miniAppBotThreadsRef.current[miniAppId];
    const page = await readMiniAppBotMessages(miniAppId, '', 500);
    if (miniAppBotThreadsRef.current[miniAppId] !== beforeRead) return miniAppBotThreadsRef.current[miniAppId] ?? [];
    if (beforeRead?.some((message) => message.kind === 'thinking' || message.streaming)) return beforeRead;
    const thread = (page.messages ?? []).map((message): DisplayMessage => {
      const mediaTypeValue = message.payload?.mediaType;
      const mediaType = mediaTypeValue === 'photo' || mediaTypeValue === 'video' || mediaTypeValue === 'document'
        ? mediaTypeValue
        : undefined;
      const mediaValue = message.payload?.media;
      const media = mediaValue && typeof mediaValue === 'object' && !Array.isArray(mediaValue)
        ? mediaValue as unknown as MessagingMediaRef
        : undefined;
      return {
        id: message.messageId,
        role: message.role === 'user' ? 'me' : 'peer',
        text: message.text,
        createdAtMs: Number.isFinite(Date.parse(message.createdAt)) ? Date.parse(message.createdAt) : Date.now(),
        source: 'legacy',
        media,
        mediaType,
        miniAppId: message.role !== 'user' && message.payload?.miniAppId === miniAppId ? miniAppId : undefined,
      };
    });
    miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [miniAppId]: thread };
    const active = peersRef.current.find((peer) => peer.key === activePeerKeyRef.current);
    if (active?.miniAppId === miniAppId) setMessages(thread);
    return thread;
  }

  async function synchronizeAccountState(): Promise<void> {
    if (accountSyncInFlightRef.current) return;
    accountSyncInFlightRef.current = true;
    try {
      let cursor = accountSyncCursorRef.current;
      let reconcileApps = false;
      let refreshBots = false;
      const changedMiniAppThreads = new Set<string>();
      for (let pageIndex = 0; pageIndex < 16; pageIndex += 1) {
        const envelope = await readAccountSync(cursor, 200);
        cursor = envelope.cursor;
        if (envelope.mode === 'snapshot') {
          reconcileApps = true;
          refreshBots = true;
        }
        for (const event of envelope.events ?? []) {
          if (event.type.startsWith('miniapp.') && !event.type.startsWith('miniapp.bot.message') && !event.type.startsWith('miniapp.content.')) {
            reconcileApps = true;
          }
          if (event.type === 'bot.added' || event.type === 'bot.updated' || event.type === 'bot.removed') refreshBots = true;
          if (event.type === 'miniapp.bot.message') {
            const miniAppId = typeof event.payload?.miniAppId === 'string' ? event.payload.miniAppId : '';
            if (miniAppId) changedMiniAppThreads.add(miniAppId);
          }
        }
        if (!envelope.hasMore) break;
      }
      accountSyncCursorRef.current = cursor;
      persistAccountSyncCursor(cursor);
      if (reconcileApps) {
        await reconcileAccountMiniApps();
        await refreshMiniApps(miniAppQuery, false);
      }
      if (refreshBots || accountBots.length === 0) {
        setAccountBots(await readAccountBots());
      }
      const active = peersRef.current.find((peer) => peer.key === activePeerKeyRef.current);
      if (active?.miniAppId && changedMiniAppThreads.has(active.miniAppId)) {
        await loadMiniAppBotThread(active.miniAppId);
      }
    } catch {
      // Offline/account bootstrap failures must not block the local Messenger.
      // The next periodic tick will retry from the last durable cursor.
    } finally {
      accountSyncInFlightRef.current = false;
    }
  }

  async function refreshMiniApps(query = miniAppQuery, reconcileAccount = true) {
    setMiniAppLoading(true);
    try {
      if (reconcileAccount) await reconcileAccountMiniApps().catch(() => undefined);
      const catalogPromise = transport.marketplaceBrowse(query);
      const identityCatalogPromise = query.trim() ? transport.marketplaceBrowse('') : catalogPromise;
      const accountCatalogPromise = readAccountMiniApps();
      const [catalogResult, identityCatalogResult, accountCatalogResult, installedResult] = await Promise.allSettled([
        catalogPromise,
        identityCatalogPromise,
        accountCatalogPromise,
        transport.pluginListInstalled(),
      ]);
      if (catalogResult.status === 'fulfilled') {
        setMarketplaceApps(catalogResult.value.plugins);
      } else {
        setError(catalogResult.reason instanceof Error ? catalogResult.reason.message : String(catalogResult.reason));
      }
      const discoveredIdentityCatalog = identityCatalogResult.status === 'fulfilled'
        ? identityCatalogResult.value.plugins
        : [];
      const accountIdentityCatalog = accountCatalogResult.status === 'fulfilled'
        ? accountMiniAppsAsMarketplaceSummaries(accountCatalogResult.value)
        : [];
      if (discoveredIdentityCatalog.length || accountIdentityCatalog.length) {
        const merged = new Map(discoveredIdentityCatalog.map((app) => [app.pluginId, app]));
        for (const accountApp of accountIdentityCatalog) {
          const discovered = merged.get(accountApp.pluginId);
          merged.set(accountApp.pluginId, {
            ...discovered,
            ...accountApp,
            bot: accountApp.bot ?? discovered?.bot,
            commands: accountApp.commands?.length ? accountApp.commands : discovered?.commands,
            surfaces: accountApp.surfaces?.length ? accountApp.surfaces : discovered?.surfaces,
          });
        }
        setMiniAppIdentityCatalog([...merged.values()]);
      } else if (identityCatalogResult.status === 'rejected') {
        setError(identityCatalogResult.reason instanceof Error ? identityCatalogResult.reason.message : String(identityCatalogResult.reason));
      } else if (accountCatalogResult.status === 'rejected') {
        setError(accountCatalogResult.reason instanceof Error ? accountCatalogResult.reason.message : String(accountCatalogResult.reason));
      }
      if (installedResult.status === 'fulfilled') {
        setInstalledMiniApps(Object.fromEntries(installedResult.value.plugins.map((plugin) => [plugin.pluginId, plugin])));
      } else {
        setError(installedResult.reason instanceof Error ? installedResult.reason.message : String(installedResult.reason));
      }
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
      const installContract = release.install
        ?? (release.releaseManifest.install as Record<string, unknown> | undefined);
      const installSource = installContract?.source as Record<string, unknown> | undefined;
      if (installContract?.protocol !== 'fabushi.marketplace.install.v1'
        || installContract.strategy !== 'github-immutable'
        || installSource?.marketplaceHostsPackage === true
        || typeof installSource?.repository !== 'string'
        || typeof installSource?.sourceRef !== 'string'
        || !installSource.sourceRef.trim()) {
        throw new Error('Mini App release is not a GitHub immutable package');
      }
      const pointer = await transport.pluginInstall(release.releaseManifest, 'desktop');
      setInstalledMiniApps((current) => ({ ...current, [app.pluginId]: pointer }));
      setMiniAppIdentityCatalog((current) => {
        const next = new Map(current.map((entry) => [entry.pluginId, entry]));
        const existing = next.get(app.pluginId);
        next.set(app.pluginId, existing ? { ...existing, ...app } : app);
        return [...next.values()];
      });
      try {
        await invokeNativeDesktop('addMiniAppToAccount', { pluginId: app.pluginId });
      } catch (cause) {
        await transport.pluginUninstall(app.pluginId).catch(() => undefined);
        setInstalledMiniApps((current) => {
          const next = { ...current };
          delete next[app.pluginId];
          return next;
        });
        throw cause;
      }
      const active = await transport.pluginActive(app.pluginId);
      if (!active) throw new Error('Mini App 安装完成后没有生成本地激活指针。');
      setInstalledMiniApps((current) => ({ ...current, [app.pluginId]: active }));
      setAccountBots(await readAccountBots().catch(() => []));
      await refreshMiniApps(miniAppQuery, false);
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
      await invokeNativeDesktop('removeMiniAppFromAccount', { pluginId: id });
      delete miniAppBotThreadsRef.current[id];
      if (miniApp?.id === id) setMiniApp(null);
      await refreshMiniApps(miniAppQuery);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setMiniAppBusyState(id, false);
    }
  }

  async function showMiniAppDocument(id: string, title: string, html: string) {
    const preparedHtml = miniAppCloudBridgeDocument(prepareDesktopMiniAppWebMcpDocument(id, html));
    const url = await window.fabushi.registerMiniAppDocument(id, preparedHtml);
    setMiniApp({ id, title, url });
  }

  async function openMiniApp(id: string) {
    setError(null);
    setMiniAppBusyState(id, true);
    try {
      const installed = installedMiniApps[id] ?? await transport.pluginActive(id);
      if (!installed) await reconcileAccountMiniApps().catch(() => undefined);
      const reconciledInstalled = installed ?? await transport.pluginActive(id);
      if (!reconciledInstalled) throw new Error('请先从在线 Mini App 市场安装此应用');
      const document = await transport.pluginUiDocument(id);
      const title = miniAppIdentityCatalog.find((app) => app.pluginId === id)?.displayName ?? marketplaceApps.find((app) => app.pluginId === id)?.displayName ?? id;
      await showMiniAppDocument(id, title, document.html);
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
      const result = await invokeNativeDesktop<{ installed?: boolean; reason?: string }>('quitAndInstallUpdate', {
        expectedVersion: 'version' in state ? state.version : undefined,
      });
      if (result?.installed === false) throw new Error(result.reason || '无法开始桌面更新');
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
    if (next === 'settings' && section !== 'settings') settingsReturnSectionRef.current = section;
    setSection(next);
  }

  function closeSettings() {
    setSection(settingsReturnSectionRef.current === 'settings' ? 'bots' : settingsReturnSectionRef.current);
  }

  return (
    <main
      className={`${styles.messenger} ${styles.fabushiUnified}`}
      data-testid="messenger-workspace"
      data-initial-host-hydrated={initialLegacyHydrated ? 'true' : undefined}
      data-sidebar-collapsed={sidebarWidth <= 112 || undefined}
      data-reduce-motion={desktopPreferences.reducedMotion || undefined}
      data-testid-ready-projection={startupProjection ? 'true' : undefined}
      style={{ gridTemplateColumns: infoPanelDocked ? `${sidebarWidth}px minmax(420px,1fr) 286px` : `${sidebarWidth}px minmax(420px,1fr)` }}
      onClick={() => { setMessageMenu(null); setProfileMenuOpen(false); setCreateMenuOpen(false); }}
    >
      <aside className={styles.chatList} data-testid="messenger-sidebar" data-collapsed={sidebarWidth <= 112 || undefined} onClick={(event) => event.stopPropagation()}>
        <GrokAgentSidebar
          agents={grokAgentItems}
          activeKey={activeGrokAgentKey}
          query={search}
          collapsed={sidebarWidth <= 112}
          hostReady={hostReady}
          accountLabel={currentActor?.displayName ?? 'Account'}
          onQuery={setSearch}
          onOpen={openGrokAgent}
          onNewAgent={() => void createGrokAgent()}
          onToggleCollapsed={() => setSidebarWidth((width) => width <= 112 ? 300 : 88)}
          onTogglePin={(item) => {
            const peer = peerForGrokAgent(item);
            if (peer) void togglePinConversation(peer);
          }}
          onRename={(item) => void renameGrokAgent(item)}
          onHide={(item) => void hideGrokAgent(item)}
          onDuplicate={(item) => void duplicateGrokAgent(item)}
          onDelete={(item) => void deleteGrokAgent(item)}
          onOpenPlugins={() => {
            setSearch('');
            setGlobalSearchOpen(false);
            setSection('miniapps');
          }}
          onOpenSettings={() => {
            settingsReturnSectionRef.current = 'bots';
            setSection('settings');
          }}
        />
        <div hidden aria-hidden="true">
        <header className={styles.listHeader}>
          <span className={styles.sidebarBrand} title="Fabushi"><BotMark botId="fabushi:brand" state="idle" size={30} label="Fabushi" /></span>
          <strong>{sectionTitle(section)}</strong>
          <div className={styles.createMenuWrap}>
            <button type="button" className={styles.iconButton} aria-label="新建" data-active={createMenuOpen} onClick={(event) => { event.stopPropagation(); setCreateMenuOpen((value) => !value); }}><SquarePen size={18} /></button>
            {createMenuOpen ? <div className={styles.createMenu} onClick={(event) => event.stopPropagation()}>
              <button type="button" onClick={() => { setCreateMenuOpen(false); setNewDialog({ type: 'group', name: '', selectedBotIds: new Set() }); }}><Users size={16} /><span>新建群组</span></button>
              <button type="button" onClick={() => { setCreateMenuOpen(false); setNewDialog({ type: 'channel', name: '', description: '' }); }}><Radio size={16} /><span>新建频道</span></button>
              {['chats', 'contacts'].includes(section) ? <button type="button" data-testid="open-contact-groups" onClick={() => { setCreateMenuOpen(false); contactGroups.openManager(); }}><Folder size={16} /><span>联系人分组</span></button> : null}
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
              <span className={extra.storyRing}><BotMark botId={`story:${story.ownerId}`} state="idle" size={40} animated={false} label={name} /></span><small>{name}</small>
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
            {renderedContactGroups.length ? renderedContactGroups.map((group) => (
              <section className="fabushi-contact-group" data-testid={`contact-group-${group.id}`} key={group.id}>
                <button
                  type="button"
                  className="fabushi-contact-group__header"
                  data-collapsed={group.isCollapsed}
                  data-synthetic={group.isSynthetic}
                  aria-expanded={!group.isCollapsed}
                  onClick={() => { if (!group.isSynthetic) contactGroups.toggleCollapsed(group.id); }}
                >
                  <span>›</span><strong>{group.name}</strong><em>{group.peers.length}</em>
                </button>
                {!group.isCollapsed ? group.peers.map(renderPeerRow) : null}
              </section>
            )) : renderedPeers.map(renderPeerRow)}
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
            title={desktopUpdateState.type === 'ready' ? '更新已下载，点击后立即安装并重启' : '点击一次即可下载、安装并自动重启'}
            onClick={() => void installDesktopUpdate(desktopUpdateState)}
          >
            <CloudDownload size={18} />
            <span>{desktopUpdateState.type === 'ready'
              ? '重启并更新'
              : desktopUpdateState.type === 'downloading'
                ? `下载更新 ${Math.max(0, Math.min(100, Math.round(desktopUpdateState.progress ?? 0)))}%`
                : desktopUpdateState.type === 'staging'
                  ? '正在安装并重启…'
                  : `更新 ${desktopUpdateState.version}`}</span>
            {desktopUpdateState.type === 'downloading' || desktopUpdateState.type === 'staging' ? (
              <span
                className={styles.updateProgressTrack}
                data-testid="desktop-update-progress"
                data-indeterminate={desktopUpdateState.type === 'staging' || undefined}
                role="progressbar"
                aria-valuemin={0}
                aria-valuemax={100}
                aria-valuenow={desktopUpdateState.type === 'downloading' ? Math.max(0, Math.min(100, Math.round(desktopUpdateState.progress ?? 0))) : undefined}
              >
                <i
                  className={styles.updateProgressFill}
                  style={desktopUpdateState.type === 'downloading' ? { width: `${Math.max(0, Math.min(100, desktopUpdateState.progress ?? 0))}%` } : undefined}
                />
              </span>
            ) : null}
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

      <GrokCommandPalette
        open={grokPaletteOpen}
        agents={grokAgentItems}
        query={grokPaletteQuery}
        onQuery={setGrokPaletteQuery}
        onClose={() => setGrokPaletteOpen(false)}
        onOpenAgent={openGrokAgent}
        onNewAgent={() => void createGrokAgent()}
        onPlugins={() => {
          setSearch('');
          setGlobalSearchOpen(false);
          setSection('miniapps');
        }}
        onSettings={() => {
          settingsReturnSectionRef.current = 'bots';
          setSection('settings');
        }}
      />

      <section className={styles.chatWorkspace}>
        {activePeer && sectionIsPeerList ? (
          <>
            <header className={styles.chatHeader}>
              <div className={styles.chatIdentity}>
                <BotMark
                  botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`}
                  state={isAgentPeer(activePeer) ? botMarkStateForPeer(activePeer, selfBotExecutions, activePeerBusy, hostReady) : 'idle'}
                  size={40}
                  className={styles.agentAvatarMark}
                  label={activePeer.title}
                />
                <div><strong>{activePeer.title}</strong><small data-testid="conversation-status">{activeTypingActors.length ? '正在输入…' : `${activePeer.subtitle}${hostReady ? ' · 在线' : ' · 正在连接'}`}</small></div>
              </div>
              <div className={styles.headerActions}>
                {activePeer.miniAppId ? <button type="button" data-testid="miniapp-bot-open" title={activePeer.miniAppMenuButtonText ?? 'Open app'} onClick={() => void openMiniApp(activePeer.miniAppId!)}><AppWindow size={18} /></button> : null}
                <button type="button" title="Search conversation" data-active={conversationSearchOpen} onClick={() => {
                  const next = !conversationSearchOpen;
                  setConversationSearchOpen(next);
                  setGlobalSearchOpen(next);
                  setGlobalSearchCategory(next ? 'posts' : 'chats');
                  setSearch('');
                  window.setTimeout(() => searchInputRef.current?.focus(), 0);
                }}><Search size={18} /></button>
                {isAgentPeer(activePeer) ? <button type="button" title="Computer" data-active={computerProfileOpen} onClick={() => {
                  setComputerProfileOpen((value) => !value);
                  if (wideInfoLayout) setInfoOpen(true); else setNarrowInfoOpen(true);
                }}><Monitor size={18} /></button> : null}
                <button type="button" title={activePeer.pinned ? 'Unpin' : 'Pin'} onClick={() => void togglePinConversation(activePeer)}><Pin size={18} /></button>
                <button type="button" title="Agent info" data-testid="conversation-info-toggle" data-active={layoutInfoOpen} onClick={() => wideInfoLayout ? setInfoOpen((value) => !value) : setNarrowInfoOpen((value) => !value)}><MoreVertical size={18} /></button>
              </div>
            </header>
            {error ? <div className={styles.errorBanner} role="alert"><span>{error}</span><button type="button" onClick={() => setError(null)}><X size={14} /></button></div> : null}
            {isAgentPeer(activePeer) ? (
              <BotConversationView
                title={activePeer.title}
                description={activePeer.subtitle}
                botId={'peer:' + activePeer.kind + ':' + (activePeer.actorId ?? activePeer.id)}
                messages={botTranscriptMessages}
                activeOperationId={activeAgentOperationId}
                hasEarlierMessages={matchingMessages.length > renderedMessages.length}
                messageAreaRef={messageAreaRef}
                showScrollToLatest={showScrollToLatest}
                onOpenMiniApp={(id) => void openMiniApp(id)}
                onLoadEarlier={() => setMessageRenderCount((count) => count + initialMessageRenderCount)}
                onScroll={handleMessageAreaScroll}
                onScrollToLatest={scrollToLatest}
                onCopyMessage={(message) => void copyBotMessage(message)}
                onRegenerate={regenerateBotMessage}
                onEdit={editBotMessage}
                onContextMenu={(event, message) => {
                  event.preventDefault();
                  event.stopPropagation();
                  setMessageMenu({ message: message as DisplayMessage, x: event.clientX, y: event.clientY });
                }}
              />
            ) : (
            <div className={styles.messageArea} data-testid="message-list" data-agent-operation-id={agentOperationId ?? undefined}>
              <div className={styles.dayDivider}>今天</div>
              {matchingMessages.length > renderedMessages.length ? <button type="button" data-testid="message-list-load-earlier" onClick={() => setMessageRenderCount((count) => count + initialMessageRenderCount)}>加载更早消息</button> : null}
              {renderedMessages.map((message) => message.kind === 'assistant-turn' && message.assistantTurn ? (
                <MahayanaAssistantTurnView
                  key={`${message.source}:${message.id}`}
                  turn={message.assistantTurn}
                  label={activePeer.title}
                  avatar={<BotMark
                    botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`}
                    state={message.assistantTurn.status === 'running' ? 'thinking' : message.assistantTurn.status === 'failed' ? 'error' : 'result'}
                    size={30}
                    className={styles.agentStreamAvatar}
                    label={activePeer.title}
                  />}
                />
              ) : message.kind === 'thinking' ? (
                <article key={`${message.source}:${message.id}`} className={styles.agentThinkingRow} data-testid="agent-thinking" data-operation-id={message.operationId}>
                  <BotMark botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state="thinking" size={30} className={styles.agentStreamAvatar} label={activePeer.title} />
                  <div><strong>{message.actionTitle ?? '正在思考'}</strong><span>大乘助手正在处理这条消息…</span></div>
                </article>
              ) : message.kind === 'action' ? (
                <article key={`${message.source}:${message.id}`} className={styles.agentActionRow} data-testid="agent-step" data-operation-id={message.operationId} data-status={message.actionStatus}>
                  <BotMark botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state={message.actionStatus === 'running' ? 'working' : message.actionStatus === 'failed' ? 'error' : 'result'} size={25} className={styles.agentStreamAvatar} label={activePeer.title} />
                  <div><strong>{message.actionTitle}</strong>{message.actionDetail ? <span>{message.actionDetail}</span> : null}</div>
                  <small>{message.actionStatus === 'running' ? '进行中' : message.actionStatus === 'failed' ? '失败' : '完成'}</small>
                </article>
              ) : (() => {
                const generatedPreview = message.role === 'peer'
                  ? generatedMiniAppPreview(message.text, message.operationId ?? message.id)
                  : null;
                return <article
                  key={`${message.source}:${message.id}`}
                  className={message.role === 'me' ? styles.messageMine : styles.messagePeer}
                  data-operation-id={message.operationId ?? undefined}
                  data-agent-id={`message-actions:${message.source}:${message.id}`}
                  data-agent-invoke="contextmenu"
                  data-agent-message-role={message.role}
                  onContextMenu={(event) => {
                    if (message.kind === 'assistant-turn' || message.kind === 'action' || message.kind === 'thinking') return;
                    event.preventDefault();
                    event.stopPropagation();
                    setMessageMenu({ message, x: event.clientX, y: event.clientY });
                  }}
                >
                  {message.pinned ? <Pin size={11} /> : null}
                  {message.mediaType === 'photo' && blobMediaUrl(message.media) ? <img className={extra.messageMedia} src={blobMediaUrl(message.media)} alt={message.media?.fileName ?? '图片'} /> : null}
                  {message.mediaType === 'video' && blobMediaUrl(message.media) ? <video className={extra.messageMedia} controls playsInline autoPlay={desktopPreferences.autoPlayMedia} src={blobMediaUrl(message.media)} /> : null}
                  {message.role === 'peer' ? <BotMark botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state="result" size={28} className={extra.messageAuthorAvatar} label={activePeer.title} /> : null}
                  {message.mediaType === 'document' && blobMediaUrl(message.media) ? <a className={extra.messageFile} href={blobMediaUrl(message.media)} download={message.media?.fileName}><FileText size={17} />{message.media?.fileName ?? '文件'}</a> : null}
                  {generatedPreview ? <div className={extra.generatedMiniAppCard} data-testid="generated-miniapp-card">
                    <AppWindow size={22} />
                    <div><strong>{generatedPreview.title}</strong><span>{generatedPreview.complete ? '小程序已生成，可以直接打开试玩。' : '正在生成可运行的小程序…'}</span></div>
                    {generatedPreview.complete ? <button type="button" data-testid="generated-miniapp-open" onClick={() => void showMiniAppDocument(generatedPreview.id, generatedPreview.title, generatedPreview.html)}>打开小程序</button> : null}
                  </div> : <StructuredMessageBody text={message.text} peers={peers} />}
                  <div className={extra.messageHoverActions} data-testid="message-hover-actions">
                    <button type="button" title="回复" aria-label="回复" onClick={() => setReplyTo(message)}><span aria-hidden="true">↩</span></button>
                    <button type="button" title="复制" aria-label="复制" onClick={() => void navigator.clipboard.writeText(message.text)}><span aria-hidden="true">⧉</span></button>
                    <button type="button" title="更多" aria-label="更多" onClick={(event) => {
                      const rect = event.currentTarget.getBoundingClientRect();
                      setMessageMenu({ message, x: rect.right, y: rect.bottom + 4 });
                    }}><span aria-hidden="true">•••</span></button>
                  </div>
                  {message.reactions?.length ? <div className={extra.reactions}>{message.reactions.map((reaction) => <span key={reaction}>{reaction}</span>)}</div> : null}
                  <small>{formatTime(message.createdAtMs)} {message.role === 'me' ? <Check size={12} /> : null}</small>
                </article>;
              })())}
              {!matchingMessages.length ? <div className={styles.chatEmpty} data-testid="message-search-empty"><BotMark botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state={isAgentPeer(activePeer) ? botMarkStateForPeer(activePeer, selfBotExecutions, false, hostReady) : 'idle'} size={78} className={styles.agentAvatarMark} label={activePeer.title} /><strong>{activePeer.title}</strong><p>联系人、AI Bot、群组和频道使用同一个 Fabushi 消息产品层。</p></div> : null}
            </div>
            )}
            {replyTo ? <div className={extra.composerBanner} data-testid="reply-message-banner"><Reply size={15} /><div><strong>回复</strong><span>{replyTo.text}</span></div><button type="button" data-testid="reply-message-cancel" onClick={() => setReplyTo(null)}><X size={14} /></button></div> : null}
            {scheduledAtMs ? <div className={extra.composerBanner}><span>⏱</span><div><strong>定时发送</strong><span>{new Date(scheduledAtMs).toLocaleString()}</span></div><button type="button" onClick={() => setScheduledAtMs(undefined)}><X size={14} /></button></div> : null}
            {activePeer.miniAppId && composer.trimStart().startsWith('/') && activePeer.miniAppCommands?.length ? <div className={extra.composerBanner} data-testid="miniapp-bot-commands"><AppWindow size={15} /><div><strong>小程序命令</strong><span>{activePeer.miniAppCommands.map((command) => `/${command.name}`).join(' · ')}</span></div>{activePeer.miniAppCommands.slice(0, 4).map((command) => <button key={command.name} type="button" title={command.description} onClick={() => updateComposer(command.usage)}>{`/${command.name}`}</button>)}</div> : null}
            <form className={styles.composer} onSubmit={(event) => void sendMessage(event)}>
              {isAgentPeer(activePeer) ? (
                <button type="button" title="Attach files" onClick={() => fileInputRef.current?.click()}><Paperclip size={20} /></button>
              ) : (
                <div className={extra.attachmentAnchor}>
                  <button type="button" title="附件" onClick={() => setAttachmentMenuOpen((value) => !value)}><Paperclip size={20} /></button>
                  {attachmentMenuOpen ? <AttachmentMenu onMedia={() => mediaInputRef.current?.click()} onFile={() => fileInputRef.current?.click()} onPoll={() => void sendPoll()} onLocation={() => void sendLocation()} onSchedule={() => {
                    const minutes = Number(window.prompt('多少分钟后发送？', '10'));
                    if (Number.isFinite(minutes) && minutes > 0) setScheduledAtMs(Date.now() + minutes * 60_000);
                    setAttachmentMenuOpen(false);
                  }} /> : null}
                </div>
              )}
              <input ref={mediaInputRef} type="file" accept="image/*,video/*" hidden onChange={(event) => { const file = event.currentTarget.files?.[0]; event.currentTarget.value = ''; if (file) void sendAttachmentFile(file); }} />
              <input ref={fileInputRef} type="file" hidden onChange={(event) => { const file = event.currentTarget.files?.[0]; event.currentTarget.value = ''; if (file) void sendAttachmentFile(file); }} />
              {attachmentProgress ? <span className={extra.uploadProgress}>{attachmentProgress}</span> : null}
              <textarea data-testid="messenger-input" value={composer} onChange={(event) => updateComposer(event.target.value)} onKeyDown={(event) => {
                const submitWithEnter = desktopPreferences.enterToSend && event.key === 'Enter' && !event.shiftKey;
                const submitWithShortcut = !desktopPreferences.enterToSend && event.key === 'Enter' && (event.metaKey || event.ctrlKey);
                if (submitWithEnter || submitWithShortcut) { event.preventDefault(); event.currentTarget.form?.requestSubmit(); }
              }} placeholder={isAgentPeer(activePeer) ? `Message ${activePeer.title}` : '消息'} rows={1} />
              {composer.trim()
                ? <button data-testid="messenger-send" className={styles.sendButton} type="submit" disabled={!hostReady || (pendingSend && (!isAgentPeer(activePeer) || Boolean(activePeer.miniAppId)))}><Send size={19} /></button>
                : activeAgentOperationId && isAgentPeer(activePeer) && !activePeer.miniAppId
                  ? <button data-testid="messenger-stop" className={styles.sendButton} type="button" title="Stop" aria-label="Stop" onClick={() => void stopAgentOperation()}><X size={19} /></button>
                  : null}
            </form>
          </>
        ) : (
          <>
            {error ? <div className={styles.errorBanner} role="alert"><span>{error}</span><button type="button" onClick={() => setError(null)}><X size={14} /></button></div> : null}
            <FeatureWorkspace section={section} onOpenMiniApp={openMiniApp} onInstallMiniApp={installMiniApp} onUninstallMiniApp={uninstallMiniApp} miniApps={marketplaceApps} installedMiniApps={installedMiniApps} miniAppQuery={miniAppQuery} onMiniAppQuery={setMiniAppQuery} miniAppLoading={miniAppLoading} miniAppBusy={miniAppBusy} onInvoice={() => void createInvoiceForActivePeer()} payment={{ account: walletAccount, entries: walletEntries, orders: selfOrders, invoices: selfInvoices, actorId: selfHosted.actorId }} onRefund={(orderId) => void refundOrder(orderId)} settings={{ category: settingsCategory, onCategory: setSettingsCategory, preferences: desktopPreferences, onPreference: updateDesktopPreference, actor: currentActor, actorId: selfHosted.actorId, hostSettings, onHostSetting: updateHostSetting, onConfigureProviderSecret: configureProviderSecret, onRemoveProviderSecret: removeProviderSecret, routerStatus, usageSummary, onInstallUpdate: installDesktopUpdate, onLogout }} />
          </>
        )}
      </section>

      {infoPanelVisible && activePeer ? (
        <aside className={styles.infoPanel} data-testid="messenger-info-panel" data-overlay={!wideInfoLayout || undefined}>
          <header><strong>资料</strong><button type="button" onClick={() => wideInfoLayout ? setInfoOpen(false) : setNarrowInfoOpen(false)}><X size={17} /></button></header>
          <div className={styles.profileCard}>
            <BotMark botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state={isAgentPeer(activePeer) ? botMarkStateForPeer(activePeer, selfBotExecutions, activePeerBusy, hostReady) : 'idle'} size={92} className={styles.agentProfileMark} label={activePeer.title} />
            <strong>{activePeer.title}</strong><small>{activePeer.subtitle}</small>
            <div className={styles.profileQuickActions} data-columns={isAgentPeer(activePeer) ? '4' : '3'}><button type="button" onClick={() => void startCall('voice')}><PhoneCall size={18} /><span>通话</span></button><button type="button" onClick={() => void startCall('video')}><Video size={18} /><span>视频</span></button><button type="button" onClick={() => { setConversationSearchOpen(true); setGlobalSearchOpen(true); setGlobalSearchCategory('posts'); setSearch(''); window.setTimeout(() => searchInputRef.current?.focus(), 0); }}><Search size={18} /><span>搜索</span></button>{isAgentPeer(activePeer) ? <button type="button" data-testid="bot-computer-toggle" data-active={computerProfileOpen} onClick={() => {
              setComputerProfileOpen((value) => !value);
              void invokeNativeDesktop('reportOpenComputer', {
                source: 'bot-profile',
                agentId: activePeer.actorId ?? activePeer.id,
                connected: remoteComputerState?.channelOpen === true,
              }).catch(() => {});
            }}><Monitor size={18} /><span>电脑</span></button> : null}</div>
          </div>
          <div className={styles.profileActions}>
            <button type="button" onClick={() => void toggleMuteConversation(activePeer)}><BellOff size={17} /><span>{mutedPeerKeys.has(activePeer.key) ? '开启通知' : '静音通知'}</span></button>
            <button type="button" onClick={() => void togglePinConversation(activePeer)}><Pin size={17} /><span>{activePeer.pinned ? '取消置顶' : '置顶会话'}</span></button>
            <button type="button" onClick={() => { void toggleArchiveConversation(activePeer); setSection('archive'); }}><Archive size={17} /><span>{activePeer.archived ? '移出归档' : '归档会话'}</span></button>
            {activePeer.source === 'selfhosted' && ['group', 'channel'].includes(activePeer.kind) ? <button type="button" onClick={() => void openCommunityAdmin(activePeer)}><Settings size={17} /><span>管理群组/频道</span></button> : null}
          </div>
          {isAgentPeer(activePeer) && computerProfileOpen ? <section className={styles.computerProfile} data-testid="bot-computer-panel">
            <header>
              <span className={styles.computerProfileIcon}><Monitor size={18} /></span>
              <span><strong>这台电脑</strong><small>{localComputerLabel()}</small></span>
              <i data-live={remoteComputerState?.channelOpen ? 'active' : localComputerOnline ? 'online' : 'offline'} />
            </header>
            <div className={styles.computerProfileStatus}>
              <span><small>设备状态</small><strong>{localComputerStatus}</strong></span>
              <span><small>已授权客户端</small><strong>{remoteComputerState?.clients.length ?? 0}</strong></span>
              <span><small>AI 操控</small><strong>{hostSettings.aiComputerControlEnabled ? '已允许' : '已关闭'}</strong></span>
            </div>
            <p>Fabushi 登录后会在后台保持设备在线；远控关闭时只能被同账号发现，不能读取画面或发送输入。</p>
            {hostSettings.remoteControlEnabled && remoteComputerState?.registration?.pairingCode ? <div className={styles.computerPairingCode}>
              <span><small>配对码</small><strong>{remoteComputerState.registration.pairingCode}</strong></span>
              <button type="button" onClick={() => void remoteComputerControllerRef.current?.refreshPairingCode().catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))}>刷新</button>
            </div> : null}
            {remoteComputerState?.pendingAuthorization ? <div className={styles.computerPairingCode} data-testid="remote-session-consent">
              <span><small>远控请求</small><strong>{remoteComputerState.pendingAuthorization.clientLabel || '已配对设备'}</strong></span>
              <button type="button" onClick={() => void remoteComputerControllerRef.current?.approvePendingSession(remoteComputerState.pendingAuthorization!.sessionId).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))}>允许本次连接</button>
              <button type="button" className={styles.computerDangerButton} onClick={() => void remoteComputerControllerRef.current?.denyPendingSession(remoteComputerState.pendingAuthorization!.sessionId).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))}>拒绝</button>
            </div> : null}
            {remoteComputerState?.activeSessionId ? <button type="button" className={styles.computerDangerButton} onClick={() => void remoteComputerControllerRef.current?.disconnectActive()}>断开当前远控</button> : null}
            <div className={styles.computerProfileButtons}>
              <button type="button" data-enabled={hostSettings.remoteControlEnabled} onClick={() => updateHostSetting('remoteControlEnabled', !hostSettings.remoteControlEnabled)}>{hostSettings.remoteControlEnabled ? '关闭远程控制' : '开启远程控制'}</button>
              <button type="button" onClick={() => void invokeNativeDesktop('openExternal', { url: `https://fabushi.ombhrum.com/remote-computer?agentId=${encodeURIComponent(activePeer.actorId ?? activePeer.id)}` }).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))}>打开控制页面</button>
            </div>
            {remoteComputerState?.error ? <small className={styles.computerProfileError}>{remoteComputerState.error}</small> : null}
          </section> : null}
          <nav className={styles.infoTabs}><button type="button" data-active={infoTab === 'media'} onClick={() => setInfoTab('media')}>媒体</button><button type="button" data-active={infoTab === 'files'} onClick={() => setInfoTab('files')}>文件</button><button type="button" data-active={infoTab === 'links'} onClick={() => setInfoTab('links')}>链接</button></nav>
          <div className={styles.infoContent}>{infoTab === 'media' ? <><Image size={30} /><strong>共享媒体</strong><p>图片、视频和动画按消息索引展示。</p></> : null}{infoTab === 'files' ? <><FileText size={30} /><strong>共享文件</strong><p>文档、音频和附件由 Rust 媒体层管理。</p></> : null}{infoTab === 'links' ? <><Link2 size={30} /><strong>共享链接</strong><p>富文本 URL 建立可搜索索引。</p></> : null}</div>
        </aside>
      ) : null}

      {contactGroups.managerOpen ? <SidebarContactGroupManager
        peers={peers.filter((peer) => !peer.archived && peer.kind === 'contact').map((peer) => ({ key: peer.key, title: peer.title, subtitle: peer.subtitle, pinned: peer.pinned }))}
        groups={contactGroups.groups}
        onCreate={contactGroups.create}
        onUpdate={contactGroups.update}
        onRemove={contactGroups.remove}
        onMove={contactGroups.move}
        onClose={contactGroups.closeManager}
      /> : null}
      {messageMenu ? <MessageContextMenu menu={messageMenu} onAction={(action) => void handleMessageAction(action)} /> : null}
      {forwardDialog ? <ForwardMessageDialog message={forwardDialog.message} peers={peers.filter((peer) => peer.source === 'selfhosted' && Boolean(peer.conversationId) && peer.conversationId !== forwardDialog.sourceConversationId)} onClose={() => setForwardDialog(null)} onSelect={(peer) => void forwardToPeer(peer)} /> : null}
      {editDialog ? <EditMessageDialog value={editDialog.text} onChange={(text) => setEditDialog((current) => current ? { ...current, text } : current)} onClose={() => setEditDialog(null)} onSave={() => void saveEditedMessage()} /> : null}
      {invoiceDialog ? <InvoiceDialog dialog={invoiceDialog} onChange={setInvoiceDialog} onClose={() => setInvoiceDialog(null)} onSave={() => void saveInvoiceDialog()} /> : null}
      {newDialog ? <NewConversationDialog dialog={newDialog} bots={bots} onChange={setNewDialog} onClose={() => setNewDialog(null)} onSave={() => void saveNewDialog()} /> : null}
      {communityDialogPeer ? <CommunityAdminDialog peer={communityDialogPeer} community={selfCommunities.find((item) => item.conversationId === communityDialogPeer.conversationId) ?? defaultCommunityState(communityDialogPeer.conversationId!, selfHosted.actorId)} actors={selfActors} actorId={selfHosted.actorId} onClose={() => setCommunityDialogPeer(null)} onSave={(community) => void saveCommunity(community)} onSetMember={(conversationId, member) => void selfHosted.setCommunityMember(conversationId, member).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onCreateInvite={(community) => { const invite = { id: `invite:${crypto.randomUUID()}`, conversationId: community.conversationId, creatorId: selfHosted.actorId, token: crypto.randomUUID().replaceAll('-', ''), name: '邀请链接', createdAtMs: Date.now(), expiresAtMs: undefined, memberLimit: undefined, joinRequest: community.joinRequestRequired, revoked: false, joinedCount: 0 }; void selfHosted.createInviteLink(invite).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause))); }} onRevokeInvite={(conversationId, inviteId) => void selfHosted.revokeInviteLink(conversationId, inviteId).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onJoinDecision={(conversationId, requesterId, approved) => void selfHosted.respondCommunityJoin(conversationId, requesterId, approved).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onUpsertTopic={(topic) => void selfHosted.upsertForumTopic(topic).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onDeleteTopic={(conversationId, topicId) => void selfHosted.deleteForumTopic(conversationId, topicId).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} /> : null}
      {activeStory ? <StoryViewer story={activeStory} owner={selfActors.find((actor) => actor.id === activeStory.ownerId)} own={activeStory.ownerId === selfHosted.actorId} onClose={() => setActiveStory(null)} onReact={(reaction) => void reactToActiveStory(reaction)} onDelete={() => void selfHosted.deleteStory(activeStory.id).then(() => setActiveStory(null)).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} /> : null}
      {localCall ? <CallDialog call={localCall} localVideoRef={localVideoRef} remoteVideoRef={remoteVideoRef} remoteAudioRef={remoteAudioRef} canAccept={Boolean(incomingCall)} onAccept={() => void acceptIncomingCall()} onDecline={() => void declineIncomingCall()} onMute={() => void toggleCallMute()} onVideo={() => void toggleCallVideo()} onShare={() => void shareCallScreen()} onEnd={() => void endCall()} /> : null}
      {miniAppCall ? <MiniAppCallDialog
        callId={miniAppCall.callId}
        miniAppId={miniAppCall.miniAppId}
        title={miniAppCall.title}
        kind={miniAppCall.kind}
        program={miniAppCall.program}
        html={miniAppCall.html}
        onCommand={(command, args) => runMiniAppCallCommand(miniAppCall, command, args)}
        onNaturalLanguage={miniAppCall.program.aiMode === 'optional'
          ? (input) => appendMiniAppCallExchange(miniAppCall.miniAppId, input, '通话语音/自然语言 · ' + input)
          : undefined}
        onSaveRecording={(blob, mimeType) => saveMiniAppCallRecording(miniAppCall, blob, mimeType)}
        onClose={() => setMiniAppCall(null)}
      /> : null}
      {miniApp ? <MiniAppDialog app={miniApp} onClose={() => setMiniApp(null)} /> : null}
      {section === 'settings' ? <div className={styles.settingsModalBackdrop} data-testid="settings-modal-backdrop" onMouseDown={closeSettings}>
        <section className={styles.settingsModal} role="dialog" aria-modal="true" aria-label="设置" onMouseDown={(event) => event.stopPropagation()}>
          <aside className={styles.settingsModalSidebar}>
            <header><strong>Settings</strong></header>
            <SettingsNavigation category={settingsCategory} onCategory={setSettingsCategory} />
          </aside>
          <div className={styles.settingsModalContent}>
            <button type="button" className={styles.settingsModalClose} data-testid="settings-close" aria-label="关闭设置" onClick={closeSettings}><X size={19} /></button>
            {error ? <div className={styles.settingsModalError} role="alert"><span>{error}</span><button type="button" aria-label="关闭错误" onClick={() => setError(null)}><X size={14} /></button></div> : null}
            <SettingsWorkspace category={settingsCategory} onCategory={setSettingsCategory} preferences={desktopPreferences} onPreference={updateDesktopPreference} actor={currentActor} actorId={selfHosted.actorId} hostSettings={hostSettings} onHostSetting={updateHostSetting} onConfigureProviderSecret={configureProviderSecret} onRemoveProviderSecret={removeProviderSecret} routerStatus={routerStatus} usageSummary={usageSummary} onInstallUpdate={installDesktopUpdate} onLogout={onLogout} />
          </div>
        </section>
      </div> : null}
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
    <div>{items.map((item) => <button key={item.section} type="button" title={item.label} data-testid={"profile-navigation-" + item.section} data-active={section === item.section} onClick={() => onNavigate(item.section)}>{item.icon}<span>{item.label}</span></button>)}</div>
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
      {!props.scopePeer && props.category === 'chats' ? peerResults.filter((peer) => peer.kind !== 'channel').map((peer) => <button key={peer.key} type="button" className={styles.searchResultRow} onClick={() => props.onOpenPeer(peer)}><BotMark botId={`peer:${peer.kind}:${peer.actorId ?? peer.id}`} state="idle" size={46} animated={false} label={peer.title} /><span><strong>{peer.title}</strong><small>{peer.subtitle}</small></span><time>{formatTime(peer.updatedAtMs)}</time></button>) : null}
      {!props.scopePeer && props.category === 'channels' ? peerResults.filter((peer) => peer.kind === 'channel').map((peer) => <button key={peer.key} type="button" className={styles.searchResultRow} onClick={() => props.onOpenPeer(peer)}><BotMark botId={`peer:channel:${peer.id}`} state="idle" size={46} animated={false} label={peer.title} /><span><strong>{peer.title}</strong><small>{peer.subtitle}</small></span><time>{formatTime(peer.updatedAtMs)}</time></button>) : null}
      {!props.scopePeer && props.category === 'apps' ? <div className={styles.searchAppResults}>{props.miniAppLoading ? <div className={styles.marketplaceStatus}>正在搜索在线应用市场…</div> : appResults.map((app) => {
        const installed = props.installedMiniApps[app.pluginId];
        const busy = props.miniAppBusy.has(app.pluginId);
        const action = miniAppMarketplaceAction(app, installed);
        const needsInstall = action === 'install' || action === 'update' || action === 'reinstall';
        return <article key={app.pluginId} className={styles.searchAppCard} data-testid={`global-search-app-${app.pluginId}`}><BotMark botId={`miniapp:${app.pluginId}`} state={installed ? 'idle' : 'sleeping'} size={52} animated={false} label={app.displayName} /><div><strong>{app.displayName}</strong><small>{app.description}</small><em>{installed ? `已安装 ${installed.version}` : `在线 · ${app.latestVersion}`} · {miniAppReleaseLabel(app)}</em></div><aside>{installed ? <button type="button" disabled={busy} onClick={() => void props.onOpenMiniApp(app.pluginId)}>打开</button> : null}{needsInstall ? <button type="button" disabled={busy} onClick={() => void props.onInstallMiniApp(app)}>{busy ? '处理中' : marketplaceInstallActionLabel(action)}</button> : action === 'blocked' ? <button type="button" disabled>阻止降级</button> : null}{installed ? <button type="button" disabled={busy} onClick={() => void props.onUninstallMiniApp(app.pluginId)}>卸载</button> : null}</aside></article>;
      })}</div> : null}
      {['posts', 'images', 'videos', 'downloads', 'links', 'files'].includes(props.category) ? mediaResults.map((message) => <article key={message.id} className={styles.searchMessageResult}><BotMark botId={`search-message:${message.id}`} state="idle" size={38} animated={false} label="消息" /><div><p>{message.text || (message.mediaType === 'photo' ? '图片' : message.mediaType === 'video' ? '视频' : '文件')}</p><small>{new Date(message.createdAtMs).toLocaleString()}</small></div></article>) : null}
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
  return <div className={extra.contextMenu} data-testid="message-context-menu" style={{ left: menu.x, top: menu.y }} onClick={(event) => event.stopPropagation()}>
    <button type="button" data-testid="message-action-reply" onClick={() => onAction('reply')}><Reply size={16} />回复</button>
    <button type="button" data-testid="message-action-copy" onClick={() => onAction('copy')}><Copy size={16} />复制</button>
    <button type="button" data-testid="message-action-react" onClick={() => onAction('react')}><Smile size={16} />👍 反应</button>
    {menu.message.role === 'me' ? <button type="button" data-testid="message-action-edit" onClick={() => onAction('edit')}><Edit3 size={16} />编辑</button> : null}
    <button type="button" data-testid="message-action-pin" onClick={() => onAction('pin')}><Pin size={16} />{menu.message.pinned ? '取消置顶' : '置顶'}</button>
    <button type="button" data-testid="message-action-forward" onClick={() => onAction('forward')}><Forward size={16} />转发</button>
    {menu.message.invoiceId ? <button type="button" data-testid="message-action-checkout" onClick={() => onAction('checkout')}><WalletCards size={16} />支付账单</button> : null}
    <button type="button" data-testid="message-action-delete" onClick={() => onAction('delete')}><Trash2 size={16} />删除</button>
  </div>;
}

function ForwardMessageDialog({ message, peers, onClose, onSelect }: { message: DisplayMessage; peers: PeerItem[]; onClose: () => void; onSelect: (peer: PeerItem) => void }) {
  return <div className={styles.backdrop} data-testid="forward-message-dialog" onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>转发消息</strong><small>{message.text || '媒体消息'}</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <div className={extra.forwardList}>
      {peers.length ? peers.map((peer) => <button key={peer.key} type="button" data-agent-id={`forward-message-peer:${peer.key}`} onClick={() => onSelect(peer)}><BotMark botId={`peer:${peer.kind}:${peer.actorId ?? peer.id}`} state="idle" size={40} label={peer.title} /><span><strong>{peer.title}</strong><small>{peer.subtitle}</small></span><Forward size={16} /></button>) : <p>暂无可转发的自建会话，请先创建群组、频道或收藏消息。</p>}
    </div>
  </section></div>;
}

function EditMessageDialog({ value, onChange, onClose, onSave }: { value: string; onChange: (value: string) => void; onClose: () => void; onSave: () => void }) {
  return <div className={styles.backdrop} data-testid="edit-message-dialog" onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>编辑消息</strong><small>修改后会同步到 Fabushi 自建会话</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <label><span>消息内容</span><textarea autoFocus data-testid="edit-message-input" value={value} onChange={(event) => onChange(event.target.value)} rows={4} placeholder="编辑消息内容" /></label>
    <footer><button type="button" data-testid="edit-message-cancel" onClick={onClose}>取消</button><button type="button" data-testid="edit-message-save" className={styles.primaryButton} disabled={!value.trim()} onClick={onSave}>保存</button></footer>
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

function miniAppCloudBridgeDocument(html: string): string {
  const bootstrap = `<script>(function(){
    const protocol='fabushi.miniapp.storage.v1';
    let sequence=0; const pending=new Map();
    function request(action,payload){return new Promise((resolve,reject)=>{const requestId='storage-'+Date.now()+'-'+(++sequence);pending.set(requestId,{resolve,reject});window.parent.postMessage({protocol,requestId,action,...(payload||{})},'*');});}
    window.addEventListener('message',(event)=>{const data=event.data||{};if(data.protocol!==protocol||!data.requestId||!pending.has(data.requestId))return;const task=pending.get(data.requestId);pending.delete(data.requestId);if(data.ok)task.resolve(data.data);else task.reject(new Error(data.error||'CloudStorage request failed'));});
    const api={
      getItem:async(key,callback)=>{const data=await request('get',{key});const value=data&&data.item?String(data.item.value??''):'';if(typeof callback==='function')callback(null,value);return value;},
      setItem:async(key,value,callback)=>{await request('set',{values:{[key]:String(value)}});if(typeof callback==='function')callback(null,true);return true;},
      getItems:async(keys,callback)=>{const data=await request('list');const wanted=new Set(Array.isArray(keys)?keys:[]);const values=Object.fromEntries((data.items||[]).filter(item=>wanted.size===0||wanted.has(item.key)).map(item=>[item.key,item.value]));if(typeof callback==='function')callback(null,values);return values;},
      setItems:async(values,callback)=>{await request('set',{values:values||{}});if(typeof callback==='function')callback(null,true);return true;},
      removeItem:async(key,callback)=>{await request('delete',{key});if(typeof callback==='function')callback(null,true);return true;},
      getKeys:async(callback)=>{const data=await request('list');const keys=(data.items||[]).map(item=>item.key);if(typeof callback==='function')callback(null,keys);return keys;}
    };
    window.FabushiMiniApp=Object.assign({},window.FabushiMiniApp||{},{CloudStorage:api});
  })();</script>`;
  return html.includes('</head>') ? html.replace('</head>', `${bootstrap}</head>`) : `${bootstrap}${html}`;
}

function MiniAppDialog({ app, onClose }: { app: { id: string; title: string; url: string }; onClose: () => void }) {
  const frameRef = useRef<HTMLIFrameElement>(null);
  useEffect(() => {
    const onMessage = (event: MessageEvent) => {
      if (event.source !== frameRef.current?.contentWindow) return;
      const data = event.data as { protocol?: string; requestId?: string; action?: string; key?: string; values?: Record<string, string> } | null;
      if (!data || data.protocol !== 'fabushi.miniapp.storage.v1' || !data.requestId) return;
      const respond = (ok: boolean, payload: unknown) => frameRef.current?.contentWindow?.postMessage({
        protocol: 'fabushi.miniapp.storage.v1', requestId: data.requestId, ok,
        ...(ok ? { data: payload } : { error: payload instanceof Error ? payload.message : String(payload) }),
      }, '*');
      void (async () => {
        try {
          if (data.action === 'get') respond(true, await readMiniAppCloudStorage(app.id, data.key));
          else if (data.action === 'list') respond(true, await readMiniAppCloudStorage(app.id));
          else if (data.action === 'set') respond(true, await writeMiniAppCloudStorage(app.id, data.values ?? {}));
          else if (data.action === 'delete' && data.key) respond(true, await deleteMiniAppCloudStorage(app.id, data.key));
          else throw new Error('Unsupported Mini App CloudStorage operation');
        } catch (cause) { respond(false, cause); }
      })();
    };
    window.addEventListener('message', onMessage);
    return () => window.removeEventListener('message', onMessage);
  }, [app.id]);
  return <div className={styles.backdrop} onMouseDown={onClose}><section className={styles.miniAppDialog} onMouseDown={(event) => event.stopPropagation()}><header><div><strong>{app.title}</strong><small>Mini App · 已安装线上包 · 账号云同步</small></div><button type="button" data-testid="miniapp-close" aria-label="关闭小程序" onClick={onClose}><X size={17} /></button></header><iframe ref={frameRef} title={app.id} sandbox="allow-scripts allow-forms" src={app.url} /></section></div>;
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
      const action = miniAppMarketplaceAction(app, installed);
      const needsInstall = action === 'install' || action === 'update' || action === 'reinstall';
      return <div className={styles.marketplaceRow} key={app.pluginId} data-testid={`miniapp-market-${app.pluginId}`}>
        <span className={styles.appIcon}><BotMark botId={`miniapp:${app.pluginId}`} state={installed ? "idle" : "sleeping"} size={34} label={app.displayName} /></span>
        <div className={styles.marketplaceCopy}><strong>{app.displayName}</strong><small>{app.description}</small><em>{installed ? `已安装 ${installed.version}` : `在线 · ${app.latestVersion}`} · {miniAppReleaseLabel(app)}</em></div>
        <div className={styles.marketplaceActions}>
          {installed ? <button type="button" disabled={busy} onClick={() => void props.onOpenMiniApp(app.pluginId)}>打开</button> : null}
          {needsInstall ? <button type="button" disabled={busy} onClick={() => void props.onInstallMiniApp(app)}>{busy ? '处理中' : marketplaceInstallActionLabel(action)}</button> : action === 'blocked' ? <button type="button" disabled>阻止降级</button> : null}
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
      const action = miniAppMarketplaceAction(app, installed);
      const needsInstall = action === 'install' || action === 'update' || action === 'reinstall';
      return <article className={styles.marketplaceCard} key={app.pluginId}>
        <BotMark botId={`miniapp:${app.pluginId}`} state={installed ? "idle" : "sleeping"} size={48} label={app.displayName} />
        <strong>{app.displayName}</strong>
        <small>{app.description}</small>
        <em>{installed ? `已安装 ${installed.version}` : `线上版本 ${app.latestVersion}`} · {miniAppReleaseLabel(app)}</em>
        <div>
          {installed ? <button type="button" disabled={busy} onClick={() => void props.onOpenMiniApp(app.pluginId)}>打开</button> : null}
          {needsInstall ? <button type="button" disabled={busy} onClick={() => void props.onInstallMiniApp(app)}>{busy ? '处理中…' : marketplaceInstallActionLabel(action)}</button> : action === 'blocked' ? <button type="button" disabled>阻止降级</button> : null}
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
  hostSettings: ProductHostSettings;
  onHostSetting: <K extends keyof ProductHostSettings>(key: K, value: ProductHostSettings[K]) => void;
  onConfigureProviderSecret: (provider: 'claude-code' | 'openrouter', value: string) => Promise<void>;
  onRemoveProviderSecret: (provider: 'claude-code' | 'openrouter') => Promise<void>;
  routerStatus: InferenceRouterStatus | null;
  usageSummary: UsageSummary | null;
  onInstallUpdate: (state: UpdateState) => Promise<void>;
  onLogout: () => Promise<void>;
};

const settingsNavigationItems: ReadonlyArray<{ id: SettingsCategory; label: string; subtitle: string; glyph: string }> = [
  { id: 'account', label: 'General', subtitle: '资料与通用设置', glyph: '⚙' },
  { id: 'router', label: 'Router', subtitle: '模型与执行环境', glyph: '⌁' },
  { id: 'usage', label: 'Usage & Billing', subtitle: '用量与账户额度', glyph: '▥' },
  { id: 'updates', label: 'Updates', subtitle: '版本与自动更新', glyph: '↻' },
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

const inferenceProviderCopy: ReadonlyArray<{ id: InferenceProvider; label: string; description: string }> = [
  { id: 'fabushi', label: 'Fabushi', description: '使用 Fabushi/Mahayana 自有推理服务、统一工具权限与账号额度。' },
  { id: 'codex', label: 'Codex', description: '复用本机 Codex 登录；新会话在同一 Mahayana runtime 中运行。' },
  { id: 'claude-code', label: 'Claude', description: '通过 Claude Messages API 接入同一 Mahayana MCP、工具与审批链。' },
  { id: 'openrouter', label: 'OpenRouter', description: '使用保存在系统加密保险库中的 OpenRouter 凭据。' },
];

function SettingsChoiceRow({ title, description, status, selected, disabled, testId, onSelect }: { title: string; description: string; status: string; selected: boolean; disabled: boolean; testId: string; onSelect: () => void }) {
  return <button className={styles.settingsChoiceRow} type="button" data-testid={testId} data-selected={selected} disabled={disabled} onClick={onSelect}>
    <span className={styles.settingsChoiceDot} aria-hidden="true" />
    <div><strong>{title}</strong><small>{description}</small></div>
    <em>{status}</em>
  </button>;
}

function SettingsWorkspace({ category, preferences, onPreference, actor, actorId, hostSettings, onHostSetting, onConfigureProviderSecret, onRemoveProviderSecret, routerStatus, usageSummary, onInstallUpdate, onLogout }: SettingsWorkspaceProps) {
  const [openRouterKey, setOpenRouterKey] = useState('');
  const [openRouterSaving, setOpenRouterSaving] = useState(false);
  const [claudeKey, setClaudeKey] = useState('');
  const [claudeSaving, setClaudeSaving] = useState(false);
  const [themePreference, setThemePreference] = useState<'system' | 'light' | 'dark'>('system');
  const [timeZone, setTimeZone] = useState('');
  const [localToolPermission, setLocalToolPermission] = useState<'always' | 'ask' | 'never'>('ask');
  const [localToolCeiling, setLocalToolCeiling] = useState<'always' | 'ask' | 'never'>('always');
  const [autoReviewInstructions, setAutoReviewInstructions] = useState('');
  const [securityKeyEnabled, setSecurityKeyEnabled] = useState(false);
  const [settingsUpdateStatus, setSettingsUpdateStatus] = useState<(UpdateState & { track?: 'stable' | 'beta' | 'alpha' }) | null>(null);
  const [updateTrack, setUpdateTrack] = useState<'stable' | 'beta' | 'alpha'>('stable');
  const [settingsActionError, setSettingsActionError] = useState<string | null>(null);
  const [logoutBusy, setLogoutBusy] = useState(false);
  const meta = settingsNavigationItems.find((item) => item.id === category)!;
  const profileName = actor?.displayName || '当前用户';
  const selectedProviderUsage = usageSummary?.byProvider?.find((item) => item.provider === hostSettings.inferenceProvider);
  const selectedProviderReadiness = routerStatus?.providers.find((item) => item.id === hostSettings.inferenceProvider);
  function runNativeSetting<T>(promise: Promise<T>, apply?: (value: T) => void) {
    setSettingsActionError(null);
    void promise.then((value) => apply?.(value)).catch((cause: unknown) => setSettingsActionError(cause instanceof Error ? cause.message : String(cause)));
  }
  useEffect(() => {
    if (category !== 'account') return;
    let disposed = false;
    void Promise.all([
      invokeNativeDesktop<{ preference?: 'system' | 'light' | 'dark' }>('getThemeState'),
      invokeNativeDesktop<string>('getTimeZone'),
      invokeNativeDesktop<'always' | 'ask' | 'never'>('getLocalToolPermission'),
      invokeNativeDesktop<'always' | 'ask' | 'never'>('getLocalToolPermissionCeiling'),
      invokeNativeDesktop<string>('getAutoReviewInstructions'),
      invokeNativeDesktop<boolean>('getWebauthnProxyEnabled'),
    ]).then(([theme, zone, permission, ceiling, instructions, securityKey]) => {
      if (disposed) return;
      setThemePreference(theme.preference ?? 'system');
      setTimeZone(zone);
      setLocalToolPermission(permission);
      setLocalToolCeiling(ceiling);
      setAutoReviewInstructions(instructions);
      setSecurityKeyEnabled(securityKey);
    }).catch(() => {});
    return () => { disposed = true; };
  }, [category]);
  useEffect(() => {
    if (category !== 'updates') return;
    let disposed = false;
    void invokeNativeDesktop<UpdateState & { track?: 'stable' | 'beta' | 'alpha' }>('getUpdateStatus').then((status) => {
      if (disposed) return;
      setSettingsUpdateStatus(status);
      setUpdateTrack(status.track ?? 'stable');
    }).catch(() => {});
    return () => { disposed = true; };
  }, [category]);
  return <div className={styles.settingsWorkspace} data-testid="telegram-settings-workspace">
    <header><div><h2>{meta.label}</h2><p>{meta.subtitle}</p></div></header>
    {settingsActionError ? <div className={styles.settingsInlineError} role="alert"><span>{settingsActionError}</span><button type="button" aria-label="关闭设置错误" onClick={() => setSettingsActionError(null)}><X size={13} /></button></div> : null}
    {category === 'account' ? <section className={styles.settingsGroup}>
      <div className={styles.settingsProfile}><BotMark botId={`self:${actorId}`} state="idle" size={72} label={profileName} /><div><strong>{profileName}</strong><small>{actor?.username ? `@${actor.username}` : actorId}</small><p>{actor?.bio || 'Fabushi 统一 Actor 资料由 Rust 消息核心管理。'}</p></div></div>
      <div className={styles.settingsActionRow}><div><strong>退出登录</strong><small>撤销当前 Fabushi 会话并清除本机账户快速启动缓存；设备通用偏好设置保留。</small></div><button data-agent-id="settings-logout" data-testid="settings-logout" data-danger="true" type="button" disabled={logoutBusy} onClick={() => { if (logoutBusy) return; setLogoutBusy(true); void onLogout().catch((cause: unknown) => setSettingsActionError(cause instanceof Error ? cause.message : String(cause))).finally(() => setLogoutBusy(false)); }}>{logoutBusy ? '退出中…' : '退出登录'}</button></div>
      <label className={styles.settingsSelectRow}><div><strong>Theme</strong><small>跟随系统，或固定浅色/深色界面。</small></div><select data-testid="settings-theme" value={themePreference} onChange={(event) => { const preference = event.target.value as 'system' | 'light' | 'dark'; setThemePreference(preference); runNativeSetting(invokeNativeDesktop('setThemePreference', { preference })); }}><option value="system">Follow System</option><option value="light">Light</option><option value="dark">Dark</option></select></label>
      <label className={styles.settingsSelectRow}><div><strong>Execution on Local Computer</strong><small>权限不能超过管理员上限：{localToolCeiling}。</small></div><select data-testid="settings-local-tool-permission" value={localToolPermission} onChange={(event) => { const permission = event.target.value as 'always' | 'ask' | 'never'; runNativeSetting(invokeNativeDesktop<'always' | 'ask' | 'never'>('setLocalToolPermission', { permission }), setLocalToolPermission); }}><option value="always" disabled={localToolCeiling !== 'always'}>Always allow</option><option value="ask" disabled={localToolCeiling === 'never'}>Ask every time</option><option value="never">Never allow</option></select></label>
      <form className={styles.settingsSecretRow} onSubmit={(event) => { event.preventDefault(); runNativeSetting(invokeNativeDesktop('setTimeZoneOverride', { timeZone: timeZone.trim() || null })); }}><div><strong>Timezone</strong><small>用于计划任务、消息时间和本地自动化。</small></div><input data-testid="settings-time-zone" value={timeZone} onChange={(event) => setTimeZone(event.target.value)} placeholder="Asia/Shanghai" /><button type="submit">保存</button></form>
      <form className={styles.settingsSecretRow} onSubmit={(event) => { event.preventDefault(); runNativeSetting(invokeNativeDesktop('setAutoReviewInstructions', { instructions: autoReviewInstructions })); }}><div><strong>Auto-review instructions</strong><small>本地工具执行前的附加审查规则。</small></div><input data-testid="settings-auto-review" value={autoReviewInstructions} onChange={(event) => setAutoReviewInstructions(event.target.value)} placeholder="每次修改配置前先询问" /><button type="submit">保存</button></form>
      <SettingsToggleRow title="Use hardware security keys" description="在当前平台可用时允许安全密钥代理；每次使用仍需批准。" checked={securityKeyEnabled} onChange={(enabled) => { setSecurityKeyEnabled(enabled); runNativeSetting(invokeNativeDesktop<boolean>('setWebauthnProxyEnabled', { enabled }), setSecurityKeyEnabled); }} />
      <SettingsToggleRow testId="settings-toggle-message-preview" title="消息预览" description="在会话列表显示最近消息摘要。" checked={preferences.messagePreview} onChange={(value) => onPreference('messagePreview', value)} />
      <SettingsToggleRow testId="settings-toggle-autoplay-media" title="自动播放视频" description="聊天内视频加载后自动播放。" checked={preferences.autoPlayMedia} onChange={(value) => onPreference('autoPlayMedia', value)} />
      <SettingsToggleRow testId="settings-toggle-info-panel" title="显示资料侧栏" description="宽屏聊天时显示右侧资料栏。" checked={preferences.showInfoPanel} onChange={(value) => onPreference('showInfoPanel', value)} />
      <SettingsToggleRow testId="settings-toggle-enter-send" title="Enter 发送消息" description="关闭后使用 Command/Ctrl + Enter 发送。" checked={preferences.enterToSend} onChange={(value) => onPreference('enterToSend', value)} />
      <SettingsToggleRow testId="settings-toggle-reduced-motion" title="减少动态效果" description="关闭大部分界面过渡动画。" checked={preferences.reducedMotion} onChange={(value) => onPreference('reducedMotion', value)} />
    </section> : null}
    {category === 'router' ? <>
      <section className={styles.settingsGroup} data-testid="router-provider-settings">
        <label className={styles.settingsSelectRow}>
          <div><strong>Provider</strong><small>{inferenceProviderCopy.find((item) => item.id === hostSettings.inferenceProvider)?.description}</small></div>
          <select data-testid="router-provider-select" value={hostSettings.inferenceProvider} onChange={(event) => onHostSetting('inferenceProvider', event.target.value as InferenceProvider)}>
            {inferenceProviderCopy.map((provider) => {
              const readiness = routerStatus?.providers.find((item) => item.id === provider.id);
              const adapterReady = true;
              const available = readiness?.available ?? provider.id === 'fabushi';
              return <option key={provider.id} data-testid={`router-provider-${provider.id}`} value={provider.id} disabled={!adapterReady || !available}>{provider.label}</option>;
            })}
          </select>
        </label>
        <div className={styles.settingsInfoRow}><strong>账户状态</strong><small>{hostSettings.inferenceProvider === 'fabushi' ? '使用当前 Fabushi/Mahayana 账户。' : '凭据由本机系统加密存储或受控 Provider 会话提供。'}</small><em>{(selectedProviderReadiness?.available ?? hostSettings.inferenceProvider === 'fabushi') ? '就绪' : '未配置'}</em></div>
        <form className={styles.settingsSecretRow} onSubmit={(event) => {
          event.preventDefault();
          if (!openRouterKey.trim() || openRouterSaving) return;
          setOpenRouterSaving(true);
          void onConfigureProviderSecret('openrouter', openRouterKey.trim()).then(() => setOpenRouterKey('')).catch(() => {}).finally(() => setOpenRouterSaving(false));
        }}>
          <div><strong>OpenRouter API key</strong><small>仅保存到操作系统加密保险库；不会回显或写入项目设置。</small></div>
          <input data-testid="router-openrouter-key" type="password" autoComplete="off" value={openRouterKey} onChange={(event) => setOpenRouterKey(event.target.value)} placeholder="sk-or-…" />
          <span className={styles.settingsSecretActions}><button data-testid="router-openrouter-save" type="submit" disabled={!openRouterKey.trim() || openRouterSaving}>{openRouterSaving ? '保存中…' : '保存'}</button>{routerStatus?.providers.find((item) => item.id === 'openrouter')?.authenticated ? <button data-testid="router-openrouter-remove" type="button" data-secondary="true" onClick={() => void onRemoveProviderSecret('openrouter')}>移除</button> : null}</span>
        </form>
        <form className={styles.settingsSecretRow} onSubmit={(event) => {
          event.preventDefault();
          if (!claudeKey.trim() || claudeSaving) return;
          setClaudeSaving(true);
          void onConfigureProviderSecret('claude-code', claudeKey.trim()).then(() => setClaudeKey('')).catch(() => {}).finally(() => setClaudeSaving(false));
        }}>
          <div><strong>Claude API key</strong><small>Claude Code 本机会话只用于诊断；API 推理凭据单独保存在系统加密保险库。</small></div>
          <input data-testid="router-claude-key" type="password" autoComplete="off" value={claudeKey} onChange={(event) => setClaudeKey(event.target.value)} placeholder="sk-ant-…" />
          <span className={styles.settingsSecretActions}><button data-testid="router-claude-save" type="submit" disabled={!claudeKey.trim() || claudeSaving}>{claudeSaving ? '保存中…' : '保存'}</button>{routerStatus?.providers.find((item) => item.id === 'claude-code')?.authenticated ? <button data-testid="router-claude-remove" type="button" data-secondary="true" onClick={() => void onRemoveProviderSecret('claude-code')}>移除</button> : null}</span>
        </form>
      </section>
      <section className={styles.settingsGroup} data-testid="router-usage-settings">
        <div className={styles.settingsInfoRow}><strong>请求</strong><small>最近 7 天由 {hostSettings.inferenceProvider} 返回用量的请求。</small><em>{selectedProviderUsage?.requests.toLocaleString() ?? '0'}</em></div>
        <div className={styles.settingsInfoRow}><strong>输入 tokens</strong><small>提供方报告的输入总量。</small><em>{selectedProviderUsage?.inputTokens.toLocaleString() ?? '0'}</em></div>
        <div className={styles.settingsInfoRow}><strong>输出 tokens</strong><small>提供方报告的输出总量。</small><em>{selectedProviderUsage?.outputTokens.toLocaleString() ?? '0'}</em></div>
        <div className={styles.settingsInfoRow}><strong>缓存 tokens</strong><small>命中提供方 prompt cache 的输入。</small><em>{selectedProviderUsage?.cachedInputTokens.toLocaleString() ?? '0'}</em></div>
        <div className={styles.settingsInfoRow}><strong>累计用量</strong><small>本机有界运行时计数；账单仍以提供方为准。</small><em>{selectedProviderUsage ? `${selectedProviderUsage.lifetimeTokens.toLocaleString()} tokens` : '0 tokens'}</em></div>
        <div className={styles.settingsInfoRow}><strong>最后使用</strong><small>不会记录或上传 prompt 内容。</small><em>{selectedProviderUsage?.lastUsedAtMs ? new Date(selectedProviderUsage.lastUsedAtMs).toLocaleString() : '尚未使用'}</em></div>
      </section>
      <section className={styles.settingsGroup} data-testid="router-sandbox-settings">
        <SettingsChoiceRow testId="router-sandbox-host" title="Fabushi Host" description="使用当前设备的 Mahayana capability-gated Host。" status="可用" selected={hostSettings.sandboxRuntime === 'host'} disabled={false} onSelect={() => onHostSetting('sandboxRuntime', 'host')} />
        <SettingsChoiceRow testId="router-sandbox-local-docker" title="Local Docker" description="无网络、只读根目录、资源受限且 owner-bound 的本地容器执行环境。" status={routerStatus?.sandboxes.find((item) => item.id === 'local-docker')?.available ? '可用' : '需要 Docker 与固定摘要镜像'} selected={hostSettings.sandboxRuntime === 'local-docker'} disabled={!routerStatus?.sandboxes.find((item) => item.id === 'local-docker')?.available} onSelect={() => onHostSetting('sandboxRuntime', 'local-docker')} />
      </section>
    </> : null}
    {category === 'usage' ? <section className={styles.settingsGroup} data-testid="usage-billing-settings">
      <div className={styles.settingsInfoRow}><strong>最近 7 天</strong><small>所有 Provider 返回并由本机汇总的 token 用量。</small><em>{usageSummary?.totalTokens.toLocaleString() ?? '0'} tokens</em></div>
      <div className={styles.settingsInfoRow}><strong>累计用量</strong><small>本机有界计数，仅用于使用趋势；账单以提供方为准。</small><em>{usageSummary?.lifetimeTokens?.toLocaleString() ?? '0'} tokens</em></div>
      <div className={styles.settingsInfoRow}><strong>请求事件</strong><small>不会记录或上传 prompt 与回复正文。</small><em>{usageSummary?.events.toLocaleString() ?? '0'}</em></div>
      {(usageSummary?.byProvider ?? []).map((item) => <div className={styles.settingsInfoRow} key={item.provider}><strong>{inferenceProviderCopy.find((provider) => provider.id === item.provider)?.label ?? item.provider}</strong><small>{item.requests.toLocaleString()} 次请求 · 输入 {item.inputTokens.toLocaleString()} · 输出 {item.outputTokens.toLocaleString()}</small><em>{item.totalTokens.toLocaleString()} tokens</em></div>)}
    </section> : null}
    {category === 'updates' ? <section className={styles.settingsGroup} data-testid="updates-settings">
      <div className={styles.settingsInfoRow}><strong>当前状态</strong><small>从已签名的 GitHub Release 更新通道检查新版本。</small><em>{settingsUpdateStatus?.type ?? '读取中'}</em></div>
      <label className={styles.settingsSelectRow}><div><strong>Release track</strong><small>稳定版、Beta 或 Alpha；切换不会跳过签名与完整性验证。</small></div><select data-testid="settings-update-track" value={updateTrack} onChange={(event) => { const track = event.target.value as 'stable' | 'beta' | 'alpha'; setUpdateTrack(track); runNativeSetting(invokeNativeDesktop<UpdateState & { track?: 'stable' | 'beta' | 'alpha' }>('setUpdateTrack', { track }), setSettingsUpdateStatus); }}><option value="stable">Stable</option><option value="beta">Beta</option><option value="alpha">Alpha</option></select></label>
      <div className={styles.settingsActionRow}><div><strong>Check for updates</strong><small>立即刷新所选发布通道。</small></div><button type="button" data-testid="settings-check-updates" onClick={() => runNativeSetting(invokeNativeDesktop<UpdateState>('checkForUpdates'), setSettingsUpdateStatus)}>检查更新</button></div>
      {settingsUpdateStatus && isActionableDesktopUpdateState(settingsUpdateStatus) ? <div className={styles.settingsActionRow}><div><strong>安装 {settingsUpdateStatus.version}</strong><small>下载完成后安全退出、安装并重新启动。</small></div><button type="button" data-testid="settings-install-update" onClick={() => void onInstallUpdate(settingsUpdateStatus)}>下载并安装</button></div> : null}
      <div className={styles.settingsInfoRow}><strong>安装方式</strong><small>下载完成后从头像旁的更新入口安装并重启。</small><em>electron-updater</em></div>
      <div className={styles.settingsInfoRow}><strong>发布完整性</strong><small>安装包与更新元数据必须来自同一 canonical main 构建。</small><em>强制验证</em></div>
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

function SectionPanel({ section, onInvoice, payment, onRefund, settings: _settings, ...miniAppProps }: { section: MessengerSection; onInvoice: () => void; payment: PaymentUiState; onRefund: (orderId: string) => void; settings: SettingsNavigationProps } & MiniAppMarketplaceProps) {
  if (section === 'miniapps') return <MiniAppMarketplaceList {...miniAppProps} />;
  if (section === 'payments') return <div className={styles.sectionList}><PaymentOverview payment={payment} onInvoice={onInvoice} onRefund={onRefund} compact /></div>;
  if (section === 'folders') return <div className={styles.sectionList}><div className={styles.panelHint}><Folder size={24} /><strong>聊天文件夹</strong><p>按联系人、Bot、群组、频道、未读和静音状态组织。</p></div></div>;
  if (section === 'calls') return <div className={styles.sectionList}><div className={styles.panelHint}><Phone size={24} /><strong>最近通话</strong><p>从会话顶部发起语音/视频。</p></div></div>;
  if (section === 'settings') return <div className={styles.sectionList} aria-hidden="true" />;
  return <div className={styles.sectionList}><div className={styles.panelHint}><Settings size={24} /><strong>{sectionTitle(section)}</strong><p>该功能入口已合并进统一 Messenger。</p></div></div>;
}

function FeatureWorkspace({ section, onInvoice, payment, onRefund, settings: _settings, ...miniAppProps }: { section: MessengerSection; onInvoice: () => void; payment: PaymentUiState; onRefund: (orderId: string) => void; settings: SettingsWorkspaceProps } & MiniAppMarketplaceProps) {
  if (section === 'miniapps') return <MiniAppMarketplaceWorkspace {...miniAppProps} />;
  if (section === 'payments') return <div className={styles.featureWorkspace}><WalletCards size={54} /><h2>Fabushi Pay</h2><p>自建余额、Invoice、Order、退款与外部 settlement 都由 Rust 账本结算。</p><PaymentOverview payment={payment} onInvoice={onInvoice} onRefund={onRefund} /></div>;
  if (section === 'calls') return <div className={styles.featureWorkspace}><Phone size={54} /><h2>通话</h2><p>本机媒体已接通，Rust realtime 已具备一对一/群组通话信令状态。</p></div>;
  if (section === 'settings') return <div className={styles.featureWorkspace} aria-hidden="true" />;
  return <div className={styles.featureWorkspace}><MessageCircle size={54} /><h2>{sectionTitle(section)}</h2><p>联系人、Bot、群组和频道正在统一到同一个 Fabushi Actor/Conversation 模型。</p></div>;
}
