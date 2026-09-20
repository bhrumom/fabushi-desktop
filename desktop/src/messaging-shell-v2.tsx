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
  AttachmentContext,
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
  SelfHostedMessagingClientV2,
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
  miniAppBotResponseText,
  type MiniAppBotCallProgram,
} from './miniapp-bot-projection';
import { MiniAppCallDialog } from './miniapp-call-dialog';
import { executeDesktopMiniAppBotInput, prepareDesktopMiniAppWebMcpDocument } from './miniapp-webmcp-host';
import BotConversationView from './bot-conversation-view';
import type { BotTranscriptMessage } from './bot-conversation-view';
import AgentWorkspace from './agent-workspace/agent-workspace';
import { CompatibilitySurface, buildCompatibilityPeers, compatibilityMessagingEnvelope, type CompatibilityPeerItem as PeerItem, type CompatibilityPeerKind as PeerKind, type CompatibilityPeerSource as PeerSource, type CompatibilitySection as MessengerSection } from './agent-workspace/messenger-compatibility-adapter';
import AgentOverlays from './agent-workspace/agent-overlays';
import { AgentCoordinatorClient } from './agent-workspace/coordinator-client';
import type { AgentTranscriptSourceMessage } from './agent-workspace/agent-transcript-store';
import { useAgentWorkspaceRuntime } from './agent-workspace/use-agent-workspace-runtime';
import { useAgentComputerController } from './agent-workspace/use-agent-computer-controller';
import { projectTranscriptEntries, type TranscriptEntry } from './agent-workspace/transcript-model';
import {
  AGENT_ATTACHMENT_LIMIT,
  agentFileToBase64,
  enrichAgentAttachmentPreview,
  validateAgentAttachment,
} from './agent-workspace/agent-attachments';
import type { AgentPromptReference, AgentReplyContext } from './agent-workspace/prompt-context';
import type { AgentSidebarSection } from './agent-workspace/agent-sidebar-state';
import { useAgentSidebarController } from './agent-workspace/use-agent-sidebar-controller';
import { useAgentNetworkController } from './agent-workspace/use-agent-network-controller';
import { useAgentCommandPaletteController } from './agent-workspace/use-agent-command-palette-controller';
import { useAgentWorkflowController } from './agent-workspace/use-agent-workflow-controller';
import { useAgentMcpController } from './agent-workspace/use-agent-mcp-controller';
import { useAgentPrSuggestionController } from './agent-workspace/use-agent-pr-suggestion-controller';
import { useAgentStoreSyncController } from './agent-workspace/use-agent-store-sync-controller';
import { useAgentDirectoryController } from './agent-workspace/use-agent-directory-controller';
import { useAgentSettingsController } from './agent-workspace/use-agent-settings-controller';
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
import {
  FABU_AGENT_ATTACHMENT_INDEX_PATH,
  FABU_AGENT_RUNTIME_CHECKPOINT_PATH,
  FABU_AGENT_WORKFLOW_INDEX_PATH,
  FabuAgentStore,
  fabuAgentConversationTranscriptPath,
} from './fabu-runtime/agent-store';
import { restoreAgentStoreWorkspace } from './agent-workspace/agent-store-recovery';
import AgentRootShell from './agent-workspace/agent-root-shell';
import AgentSidebar from './agent-workspace/agent-sidebar';
import { agentWorkspaceKey, projectActiveAgentKey, projectAgentSidebarItems, type AgentSidebarItem } from './agent-workspace/agent-model';
import AgentSearch from './agent-workspace/agent-search';
import AgentHeader from './agent-workspace/agent-header';
import AgentNetwork from './agent-workspace/agent-network';
import AgentCommandPalette from './agent-workspace/agent-command-palette';
import { MahayanaAssistantTurnView } from './mahayana-assistant-turn-view';
import type { AssistantTurn } from './mahayana-assistant-turn';

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
  attachments?: readonly AttachmentContext[];
  miniAppId?: string;
  pinned?: boolean;
  reactions?: string[];
  invoiceId?: string;
  media?: MessagingMediaRef;
  mediaType?: 'photo' | 'video' | 'document';
};

function toAgentTranscriptSources(
  messages: readonly DisplayMessage[],
): AgentTranscriptSourceMessage[] {
  return messages.map((message) => ({ ...message, source: 'legacy' as const }));
}

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

const messengerSettingsKey = 'fabushi.desktop.messenger-settings.v2';
const messengerDraftsKey = 'fabushi.desktop.messenger-drafts.v2';
const messengerSidebarWidthKey = 'fabushi.desktop.sidebar-width.v3';
const messengerProjectionKey = 'fabushi.desktop.messenger-projection.v1';
const accountSyncCursorKey = 'fabushi.desktop.account-sync-cursor.v1';
const messengerConversationJournalKey = 'fabushi.desktop.mahayana-conversation-journal.v1';
const miniAppExecutionPersistencePrefix = 'fabushi.desktop.miniapp-execution.v1:';
const messengerPreferencesKey = 'fabushi.desktop.telegram-settings.v1';
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
  const bootstrapAgent: AgentSidebarItem = {
    key: 'agent:mahayana-assistant',
    peerKey: 'bootstrap:mahayana-assistant',
    id: 'mahayana-assistant',
    agentId: 'mahayana-assistant',
    name: 'Mahayana',
    description: 'Connecting to local Agent…',
    pinned: true,
    hidden: false,
    unread: 0,
    busy: false,
    isGroup: false,
    updatedAtMs: Date.now(),
  };
  const noop = () => {};
  return (
    <main
      className={`${styles.messenger} ${styles.fabushiUnified} ${styles.grokParity}`}
      data-testid="desktop-fast-start-bootstrap"
      aria-busy="true"
      aria-label="Fabushi is connecting"
      style={{ gridTemplateColumns: '300px minmax(420px,1fr)' }}
    >
      <aside className={styles.chatList}>
        <AgentSidebar
          agents={[bootstrapAgent]}
          activeKey={bootstrapAgent.key}
          query=""
          collapsed={false}
          hostReady={false}
          accountLabel="Connecting…"
          onQuery={noop}
          onOpen={noop}
          onNewAgent={noop}
          onToggleCollapsed={noop}
          onTogglePin={noop}
          onRename={noop}
          onHide={noop}
          onDuplicate={noop}
          onDelete={noop}
          onReorderPinned={noop}
          onBroadcast={noop}
          onOpenNetwork={noop}
          onOpenPlugins={noop}
          onOpenSettings={noop}
        />
      </aside>
      <section className={styles.chatWorkspace}>
        <div className={styles.chatEmpty}>
          <BotMark botId="fabushi:bootstrap:workspace" state="waking" size={72} paused label="Fabushi" />
          <strong>Mahayana</strong>
          <p>The Agent workspace is ready. Connecting to the local runtime…</p>
        </div>
      </section>
    </main>
  );
}

function MessengerWorkspace({ initialProjection, onLogout }: { initialProjection?: MessengerProjection | null; onLogout: () => Promise<void> }) {
  const transport = useMemo(() => createTransport(), []);
  const agentCoordinatorClient = useMemo(() => new AgentCoordinatorClient(transport), [transport]);
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
  // Compatibility Messenger drafts remain React-owned. Agent drafts live in
  // AgentWorkspaceController and never share this renderer-global map.
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [legacyReplyTo, setLegacyReplyTo] = useState<DisplayMessage | null>(null);
  const [silentSend, setSilentSend] = useState(false);
  const [scheduledAtMs, setScheduledAtMs] = useState<number | undefined>();
  const [search, setSearch] = useState('');
  const agentPaletteController = useAgentCommandPaletteController();
  const agentSidebarController = useAgentSidebarController(remoteAccountScope);
  const agentPinnedOrder = agentSidebarController.pinnedOrder;
  const agentSidebarSections = agentSidebarController.sections;
  const agentSelectedKeys = agentSidebarController.selectedKeys;
  const [sidebarWidth, setSidebarWidth] = useState(330);
  const [conversationSearchOpen, setConversationSearchOpen] = useState(false);
  const [agentConversationSearch, setAgentConversationSearch] = useState('');
  const [desktopUpdateState, setDesktopUpdateState] = useState<UpdateState | null>(null);
  const [desktopUpdateBusy, setDesktopUpdateBusy] = useState(false);
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
  const [agentSettingsOpen, setAgentSettingsOpen] = useState(false);
  // Compatibility Messenger/Mini App sends retain transport pending state.
  // Agent request/operation ownership is exclusively per-peer in the workspace controller.
  const [legacySendPending, setLegacySendPending] = useState(false);
  const [showScrollToLatest, setShowScrollToLatest] = useState(false);
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
  const [accountBots, setAccountBots] = useState<AccountBotMembership[]>(startupProjection?.accountBots ?? []);
  const [marketplaceApps, setMarketplaceApps] = useState<MarketplacePluginSummary[]>([]);
  const [miniAppIdentityCatalog, setMiniAppIdentityCatalog] = useState<MarketplacePluginSummary[]>(startupProjection?.miniAppIdentityCatalog ?? []);
  const [installedMiniApps, setInstalledMiniApps] = useState<Record<string, InstalledPluginPointer>>({});
  const [miniAppQuery, setMiniAppQuery] = useState('');
  const [miniAppLoading, setMiniAppLoading] = useState(false);
  const [miniAppBusy, setMiniAppBusy] = useState<Set<string>>(() => new Set());
  const [error, setError] = useState<string | null>(null);
  const agentNetworkController = useAgentNetworkController(agentCoordinatorClient, setError);
  const agentWorkflowController = useAgentWorkflowController(agentCoordinatorClient, {
    onListed: (agentId, workflows) => {
      void mirrorAgentCloudSnapshot(agentId, FABU_AGENT_WORKFLOW_INDEX_PATH, {
        version: 1,
        workflows,
      });
    },
    onError: (_agentId, message) => setError(message),
  });
  const agentMcpController = useAgentMcpController(agentCoordinatorClient, {
    onError: setError,
  });
  const agentPrSuggestionController = useAgentPrSuggestionController(
    agentCoordinatorClient,
    agentMcpController.pullRequestTool,
  );
  const agentStoreSyncController = useAgentStoreSyncController(agentCoordinatorClient, {
    mirror: (agentId, objectPath, value) => mirrorAgentCloudSnapshot(agentId, objectPath, value),
    remove: (agentId, objectPath) => removeAgentCloudObject(agentId, objectPath),
    onError: (_agentId, message) => setError(message),
  });
  const agentDirectoryController = useAgentDirectoryController(
    agentCoordinatorClient,
    startupProjection?.legacyBots ?? [],
    {
      onListed: (agents) => {
        initialLegacyHydrationMaskRef.current |= 0b010;
        setInitialLegacyHydrationMask(initialLegacyHydrationMaskRef.current);
        for (const agent of agents) {
          void mirrorBotAgentCloud(agent).catch(() => {});
        }
      },
      onChanged: (action, agent) => {
        if (action === 'created' && newAgentRequestPendingRef.current) {
          newAgentRequestPendingRef.current = false;
          setPendingOpenAgentId(agent.agentId ?? agent.id);
        }
        if (action === 'deleted') {
          void invokeNativeDesktop('removeBotFromAccount', { botId: agent.id }).catch(() => {});
        } else {
          void mirrorBotAgentCloud(agent).catch(() => {});
        }
      },
      onError: setError,
    },
  );
  const bots = agentDirectoryController.agents;
  // AgentRootShell first-frame readiness is owned by the Agent domain, not by
  // legacy conversations/groups. A durable Agent projection is enough to render
  // and recover immediately while the authoritative directory refreshes in the
  // background; fresh accounts wait for the first Agent directory response.
  const initialAgentWorkspaceHydrated = hostReady
    && ((initialLegacyHydrationMask & 0b010) !== 0
      || bots.length > 0
      || Boolean(startupProjection?.legacyBots?.length));
  const [mutedPeerKeys, setMutedPeerKeys] = useState<Set<string>>(() => new Set());
  const [pinnedPeerKeys, setPinnedPeerKeys] = useState<Set<string>>(() => new Set());
  const [archivedPeerKeys, setArchivedPeerKeys] = useState<Set<string>>(() => new Set());
  const activePeerKeyRef = useRef<string | null>(null);
  const messagingCursorRef = useRef<string | null>(startupProjection?.cursor ?? null);
  const accountSyncCursorRef = useRef<string | null>(readAccountSyncCursor());
  const syncInFlightRef = useRef(false);
  const accountSyncInFlightRef = useRef(false);
  const typingStopTimerRef = useRef<number | null>(null);
  const peersRef = useRef<PeerItem[]>([]);
  const webRtcRef = useRef<FabushiWebRtcController | null>(null);
  const localVideoRef = useRef<HTMLVideoElement>(null);
  const remoteVideoRef = useRef<HTMLVideoElement>(null);
  const remoteAudioRef = useRef<HTMLAudioElement>(null);
  const mediaInputRef = useRef<HTMLInputElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const sessionResetInFlightRef = useRef(false);
  const messageAreaRef = useRef<HTMLDivElement | null>(null);
  const stickToLatestRef = useRef(true);
  const agentStoresRef = useRef(new Map<string, FabuAgentStore>());
  const agentComputer = useAgentComputerController({
    hostReady,
    hydrated: initialAgentWorkspaceHydrated,
    accountScope: remoteAccountScope,
    activePeerKey,
    transport,
    coordinatorClient: agentCoordinatorClient,
    label: localComputerLabel(),
    remoteControlEnabled: hostSettings.remoteControlEnabled,
    resolveAgentId: (requestedAgentId) => requestedAgentId === 'mahayana-assistant'
      || peersRef.current.some((peer) => isAgentPeer(peer)
        && (peer.agentId ?? peer.actorId ?? peer.id) === requestedAgentId)
      ? requestedAgentId
      : null,
    onError: setError,
  });

  const {
    controller: agentWorkspaceController,
    transcriptStore: agentTranscriptStore,
    coordinator: agentRuntimeCoordinator,
    submit: submitAgentWorkspace,
    openConversation: openAgentConversation,
    uploadAttachment: uploadAgentAttachment,
    resolveApproval: resolveAgentApproval,
    interrupt: interruptAgentWorkspace,
    revision: agentWorkspaceRevision,
    notify: notifyAgentWorkspaceState,
  } = useAgentWorkspaceRuntime({
    hostReady,
    coordinatorClient: agentCoordinatorClient,
    onError: (peerKey, message) => {
      if (peerKey === activePeerKeyRef.current) setError(message);
    },
    onComputerStatus: agentComputer.handleCapabilityStatus,
    onOperationStarted: (peerKey, operationId) => {
      mirrorAgentRuntimeCheckpoint(peerKey, 'running', operationId);
    },
    onOperationTerminal: (peerKey, operationId, status, message) => {
      mirrorAgentRuntimeCheckpoint(peerKey, status, operationId, message);
      mirrorAgentConversationSnapshot(peerKey);
      if (status === 'failed' && message && peerKey === activePeerKeyRef.current) {
        setError(message);
      }
    },
  });

  useEffect(() => {
    activePeerKeyRef.current = activePeerKey;
  }, [activePeerKey]);

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
    const peer = peersRef.current.find((candidate) => candidate.key === activePeerKey);
    const isWorkspaceAgent = Boolean(
      peer
      && peer.source === 'legacy'
      && peer.kind !== 'group'
      && !peer.miniAppId
      && isAgentPeer(peer)
    );
    // Compatibility Messenger owns the renderer-global composer string.
    // Normal Agents render directly from AgentWorkspaceController and must
    // never copy their draft into this global state.
    if (!isWorkspaceAgent) setComposer(drafts[activePeerKey] ?? '');
    setSearch('');
    setConversationSearchOpen(false);
    setAgentConversationSearch('');
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
    const connection = agentCoordinatorClient.connect({
      config: { profileId: 'desktop-messenger-v2', mode: 'production' },
      onEvent: (event) => {
        if (!closed) handleRuntimeEvent(event);
      },
      onState: (state) => {
        if (closed) return;
        setHostReady(state.phase === 'ready');
        if (state.phase !== 'ready' || !state.recovered) return;
        void execute({ type: 'settings.get', requestId: nextRequestId('settings-recover') }).catch(() => {});
        refreshLegacy();
        const activeKey = activePeerKeyRef.current;
        const active = peersRef.current.find((peer) => peer.key === activeKey);
        if (active?.conversationId) {
          if (isAgentPeer(active) && !active.miniAppId) {
            void openAgentConversation(active.key, active.conversationId).catch(() => {});
          } else {
            void agentCoordinatorClient.openConversation(
              nextRequestId('conversation-recover'),
              active.conversationId,
            ).catch(() => {});
          }
        }
      },
    });
    const onCommandBridge = (event: Event) => {
      const detail = (event as CustomEvent<MahayanaCommandBridgeDetail>).detail;
      if (detail) agentRuntimeCoordinator.handleCommandBridge(detail);
    };
    window.addEventListener(MAHAYANA_COMMAND_EVENT_NAME, onCommandBridge);
    void connection.ready
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
          const account = await agentCoordinatorClient.authStatus().catch(() => null);
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
          // Contacts/Telegram/account-cursor reconciliation are compatibility
          // capabilities. They must never hold AgentRootShell, Agent Computer or
          // the Composer behind a legacy bootstrap round trip.
          void (async () => {
            await selfHosted.ensureCurrentActor(displayName, username);
            await selfHosted.sync(initialSyncLimit, messagingCursorRef.current);
            await synchronizeAccountState();
            void webRtcRef.current?.connect().catch(() => {});
          })().catch((cause: unknown) => {
            if (!closed) setError(cause instanceof Error ? cause.message : String(cause));
          });
        } catch (cause) {
          setError(cause instanceof Error ? cause.message : String(cause));
        }
      })
      .catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
    return () => {
      closed = true;
      window.removeEventListener(MAHAYANA_COMMAND_EVENT_NAME, onCommandBridge);
      void connection.dispose();
    };
  }, [agentCoordinatorClient, selfHosted, startupProjection]);

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
    if (!initialAgentWorkspaceHydrated) return;
    // MCP discovery belongs to the Agent Composer. Start it after the Agent
    // surface is usable, independently of compatibility conversation/group
    // hydration, and yield one frame so it cannot delay the first paint.
    const timer = window.setTimeout(() => {
      void agentMcpController.list().catch(() => {});
    }, 0);
    return () => window.clearTimeout(timer);
  }, [initialAgentWorkspaceHydrated, agentMcpController.list]);

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
    if ((mask & 0b010) === 0) requests.push(agentDirectoryController.list());
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

  async function mirrorAgentCloudSnapshot(
    agentId: string,
    path: string,
    value: unknown,
  ): Promise<void> {
    if (!agentId.trim()) return;
    const store = agentStoreFor(agentId);
    try {
      await store.writeJson(path, value);
      await store.checkpointRoot();
    } catch {
      // The local Host remains authoritative while offline or during an
      // etag conflict. The content-addressed backend preserves conflicting
      // writes instead of silently overwriting another device.
    }
  }

  async function removeAgentCloudObject(agentId: string, path: string): Promise<void> {
    if (!agentId.trim()) return;
    try {
      const store = agentStoreFor(agentId);
      await store.delete(path);
      await store.checkpointRoot();
    } catch {
      // Deletion is best-effort; account sync/reconciliation can retry.
    }
  }

  function agentPeerForRuntimeKey(peerKey: string | null | undefined): PeerItem | undefined {
    if (!peerKey) return undefined;
    const peer = peersRef.current.find((candidate) => candidate.key === peerKey);
    return peer && isAgentPeer(peer) && !peer.miniAppId ? peer : undefined;
  }

  function mirrorAgentRuntimeCheckpoint(
    peerKey: string | null | undefined,
    status: 'running' | 'completed' | 'failed' | 'interrupted',
    operationId: string,
    message?: string,
  ): void {
    const peer = agentPeerForRuntimeKey(peerKey);
    if (!peer) return;
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    void mirrorAgentCloudSnapshot(agentId, FABU_AGENT_RUNTIME_CHECKPOINT_PATH, {
      schemaVersion: 1,
      agentId,
      conversationId: peer.conversationId,
      operationId,
      status,
      updatedAtMs: Date.now(),
      ...(message ? { message } : {}),
    });
  }

  function mirrorAgentConversationSnapshot(peerKey: string | null | undefined): void {
    const peer = agentPeerForRuntimeKey(peerKey);
    if (!peer?.conversationId) return;
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    window.setTimeout(() => {
      void mirrorAgentCloudSnapshot(agentId, fabuAgentConversationTranscriptPath(peer.conversationId!), {
        schemaVersion: 1,
        agentId,
        conversationId: peer.conversationId,
        entries: agentTranscriptStore.entries(peer.key),
        updatedAtMs: Date.now(),
      });
    }, 0);
  }

  async function restoreAgentConversationFromCloud(peer: PeerItem): Promise<boolean> {
    if (!isAgentPeer(peer) || peer.miniAppId || !peer.conversationId) return false;
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    try {
      const recovered = await restoreAgentStoreWorkspace(agentStoreFor(agentId), peer.conversationId);
      const controller = agentWorkspaceController;
      let changed = false;

      if (
        recovered.entries.length
        && !controller.operationForPeer(peer.key)
        && agentTranscriptStore.entries(peer.key).length === 0
      ) {
        agentTranscriptStore.hydrateEntries(peer.key, recovered.entries);
        changed = true;
      }

      if (recovered.attachments.length && controller.attachmentsForPeer(peer.key).length === 0) {
        controller.setAttachments(peer.key, recovered.attachments);
        changed = true;
      }

      if (changed) notifyAgentWorkspaceState();
      return recovered.entries.length > 0;
    } catch {
      return false;
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
    await store.checkpointRoot();
  }

  function showSelfConversation(conversationId: string) {
    setMessages((selfMessages[conversationId] ?? []).filter((message) => !message.deleted).map(displaySelfMessage));
  }

  function handleSelfHostedEvent(runtimeEvent: RuntimeEvent): boolean {
    const envelope = compatibilityMessagingEnvelope(runtimeEvent);
    if (!envelope) return false;
    const previousCursor = messagingCursorRef.current;
    if (envelope.cursor) messagingCursorRef.current = envelope.cursor;
    const event = envelope.event;
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
        setLegacySendPending(false);
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
    if (agentDirectoryController.handle(event)) return;
    if (agentNetworkController.handle(event)) return;
    if (agentStoreSyncController.handle(event)) return;
    if (agentWorkflowController.handle(event)) return;
    if (agentMcpController.handle(event)) return;
    // Normal Agent chat/operation events are owned by AgentRuntimeCoordinator.
    // The switch below remains only as a compatibility fallback for legacy
    // Messenger/Host event shapes that cannot be attributed to an Agent.
    if (agentRuntimeCoordinator.handle(event)) return;

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
      case 'conversation.opened': {
        const ownerPeer = peersRef.current.find((peer) =>
          peer.conversationId === event.conversationId && isAgentPeer(peer) && !peer.miniAppId,
        );
        const openedMessages: DisplayMessage[] = event.messages.map((message) => ({
          id: message.id,
          source: 'legacy',
          role: message.role === 'user' ? 'me' : 'peer',
          text: message.text,
          createdAtMs: message.createdAtMs,
          kind: 'message',
        }));
        if (ownerPeer) {
          const agentId = ownerPeer.agentId ?? ownerPeer.actorId ?? ownerPeer.id;
          void mirrorAgentCloudSnapshot(agentId, fabuAgentConversationTranscriptPath(event.conversationId), {
            schemaVersion: 1,
            agentId,
            conversationId: event.conversationId,
            entries: projectTranscriptEntries(openedMessages),
            updatedAtMs: Date.now(),
          });
          // Hydration is owned by the Agent whose conversation was opened.
          // A different visible Agent being busy must never suppress this
          // history, and an owner that is actively streaming must never be
          // overwritten by a late conversation.opened response.
          if (!agentWorkspaceController.operationForPeer(ownerPeer.key)) {
            const currentEntries = agentTranscriptStore.entries(ownerPeer.key);
            if (openedMessages.length || currentEntries.length === 0) {
              agentTranscriptStore.replace(ownerPeer.key, toAgentTranscriptSources(openedMessages));
              notifyAgentWorkspaceState();
            }
          }
          break;
        }
        if (agentWorkspaceController.operationForPeer(activePeerKeyRef.current)) break;
        if (activePeerKeyRef.current === `legacy:conversation:${event.conversationId}`
          || peersRef.current.some((peer) => peer.key === activePeerKeyRef.current
            && peer.source === 'legacy'
            && peer.conversationId === event.conversationId)) {
          setMessages(openedMessages);
        }
        break;
      }
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
          setLegacySendPending(false);
        }
        break;
      case 'settings.changed':
        setHostSettings(event.settings);
        break;
      case 'chat.message':
        // Agent-owned chat is consumed above by AgentRuntimeCoordinator.
        // What reaches this branch belongs to compatibility Messenger surfaces.
        setMessages((current) => {
          if (event.role === 'user') {
            const optimisticIndex = current.findIndex((message) =>
              message.role === 'me' && message.optimistic === true && message.text === event.text,
            );
            if (optimisticIndex >= 0) {
              return current.map((message, messageIndex) => messageIndex === optimisticIndex
                ? { ...message, optimistic: false, operationId: event.operationId ?? message.operationId }
                : message);
            }
          } else if (event.operationId) {
            const streamingIndex = current.findIndex((message) =>
              message.role === 'peer'
              && message.kind === 'message'
              && message.operationId === event.operationId
              && message.streaming === true,
            );
            if (streamingIndex >= 0) {
              return current.map((message, messageIndex) => messageIndex === streamingIndex
                ? { ...message, text: event.text, streaming: false }
                : message);
            }
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
      case 'chat.delta':
        setMessages((current) => {
          const operationId = event.operationId;
          const index = current.findIndex((message) =>
            message.kind === 'message'
            && message.role === 'peer'
            && message.operationId === operationId
            && message.streaming === true,
          );
          if (index < 0) return [...current, {
            id: `${operationId ?? 'legacy'}:stream`,
            source: 'legacy',
            role: 'peer',
            text: event.delta,
            createdAtMs: Date.now(),
            kind: 'message',
            operationId,
            streaming: true,
          }];
          return current.map((message, messageIndex) => messageIndex === index
            ? { ...message, text: `${message.text}${event.delta}`, streaming: true }
            : message);
        });
        break;
      case 'operation.started':
      case 'model.routed':
      case 'agent.step':
      case 'operation.interrupted':
      case 'operation.completed':
        break;
      case 'miniapp.opened':
        if (event.html) {
          const title = miniAppIdentityCatalog.find((app) => app.pluginId === event.miniAppId)?.displayName ?? marketplaceApps.find((app) => app.pluginId === event.miniAppId)?.displayName ?? event.miniAppId;
          void showMiniAppDocument(event.miniAppId, title, event.html).catch((cause) => {
            setError(cause instanceof Error ? cause.message : String(cause));
          });
        }
        break;
      case 'operation.failed':
        // Agent failures are consumed by AgentRuntimeCoordinator. Preserve the
        // legacy error banner only for compatibility operations.
        setError(event.message);
        break;
      case 'host.closed':
        agentRuntimeCoordinator.resetOperations();
        notifyAgentWorkspaceState();
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

  const peers = useMemo<PeerItem[]>(() => buildCompatibilityPeers({
    conversations,
    bots,
    accountBots,
    groups,
    selfActors,
    selfConversations,
    pinnedPeerKeys,
    archivedPeerKeys,
    miniAppIdentityCatalog,
    installedMiniApps,
    selfActorId: selfHosted.actorId,
  }), [conversations, bots, accountBots, groups, selfActors, selfConversations, pinnedPeerKeys, archivedPeerKeys, miniAppIdentityCatalog, installedMiniApps, selfHosted.actorId]);

  peersRef.current = peers;
  useEffect(() => {
    agentRuntimeCoordinator.bindAgentPeers(peers.flatMap((peer) => {
      if (!isAgentPeer(peer) || peer.miniAppId) return [];
      const agentId = peer.agentId ?? peer.actorId;
      return agentId ? [{ agentId, peerKey: peer.key }] : [];
    }));
  }, [agentRuntimeCoordinator, peers]);
  const agentActivityByPeer = Object.fromEntries(peers.map((peer) => {
    const thread = isAgentPeer(peer) && !peer.miniAppId
      ? agentTranscriptStore.thread(peer.key)
      : peer.key === activePeerKey
        ? messages
        : peer.miniAppId
          ? miniAppBotThreadsRef.current[peer.miniAppId] ?? []
          : [];
    const lastMessage = [...thread]
      .reverse()
      .find((message) => message.kind === 'message' && Boolean(message.text.trim()));
    const runningTurn = [...thread]
      .reverse()
      .find((message) => message.kind === 'assistant-turn' && message.assistantTurn?.status === 'running')
      ?.assistantTurn;
    const runningActivity = runningTurn
      ? [...runningTurn.parts].reverse().find((part) =>
        (part.kind === 'tool' || part.kind === 'activity') && part.status === 'running',
      )
      : undefined;
    const waitingActivity = runningTurn
      ? [...runningTurn.parts].reverse().find((part) =>
        part.kind === 'activity'
        && part.status === 'running'
        && (part.title === '等待授权' || part.title === '需要补充信息'),
      )
      : undefined;
    const isComposingMessage = peer.source === 'selfhosted'
      && Boolean(peer.conversationId)
      && Object.keys(typingByConversation[peer.conversationId!] ?? {}).length > 0;
    return [peer.key, {
      draftPrompt: peer.source === 'legacy' && peer.kind !== 'group' && !peer.miniAppId
        ? agentWorkspaceController.draftForPeer(peer.key)
        : drafts[peer.key],
      lastMessage: lastMessage?.text.trim().slice(0, 180),
      waitingReason: waitingActivity?.kind === 'activity'
        ? (waitingActivity.detail?.trim() || waitingActivity.title)
        : undefined,
      currentActivity: runningActivity && (runningActivity.kind === 'tool' || runningActivity.kind === 'activity')
        ? (runningActivity.detail?.trim() || runningActivity.title)
        : undefined,
      isComposingMessage,
    }] as const;
  }));
  const agentOperationSnapshot = useMemo(
    () => agentWorkspaceController.snapshot(),
    [agentWorkspaceRevision],
  );
  const agentRequestSnapshot = useMemo(
    () => agentWorkspaceController.requestSnapshot(),
    [agentWorkspaceRevision],
  );
  const agentBusyByPeer = Object.fromEntries(peers.flatMap((peer) => {
    const activityId = agentOperationSnapshot[peer.key] ?? agentRequestSnapshot[peer.key];
    return activityId ? [[peer.key, activityId] as const] : [];
  }));
  const legacyPinnedAgentKeys = [...new Set(peers
    .filter((peer) => peer.pinned && (peer.kind === 'bot' || peer.kind === 'group'))
    .map((peer) => agentWorkspaceKey(peer)))];
  const legacyPinnedAgentSignature = legacyPinnedAgentKeys.join('\u001f');
  const agentItems: AgentSidebarItem[] = projectAgentSidebarItems(
    peers,
    agentBusyByPeer,
    agentActivityByPeer,
    agentPinnedOrder,
  );
  const activePeer = peers.find((peer) => peer.key === activePeerKey) ?? null;
  const activeAgentBot: BotSummary | null = activePeer && isAgentPeer(activePeer) && !activePeer.miniAppId
    ? bots.find((bot) => bot.id === (activePeer.actorId ?? activePeer.id))
      ?? (() => {
        const membership = accountBots.find((entry) => entry.bot.id === (activePeer.actorId ?? activePeer.id));
        if (!membership) return null;
        const bot = membership.bot;
        return {
          id: bot.id,
          agentId: bot.agentId ?? activePeer.agentId ?? bot.id,
          name: bot.displayName ?? bot.username ?? activePeer.title,
          description: bot.description ?? '',
          title: bot.title ?? '',
          hidden: bot.hidden === true,
          ...(bot.avatar ? { avatar: bot.avatar } : {}),
          ...(bot.avatarShape ? { avatarShape: bot.avatarShape } : {}),
          ...(bot.avatarColor ? { avatarColor: bot.avatarColor } : {}),
          notificationsEnabled: bot.notificationsEnabled ?? true,
          notifyOnUpdates: bot.notifyOnUpdates ?? bot.notificationsEnabled ?? true,
          unread: bot.unread === true,
          ...(bot.conversationId ? { conversationId: bot.conversationId } : {}),
        } satisfies BotSummary;
      })()
      ?? {
        id: activePeer.actorId ?? activePeer.id,
        agentId: activePeer.agentId ?? activePeer.actorId ?? activePeer.id,
        name: activePeer.title,
        description: activePeer.subtitle,
        title: '',
        hidden: activePeer.hidden === true,
        notificationsEnabled: true,
        notifyOnUpdates: true,
        unread: activePeer.unread > 0,
        ...(activePeer.conversationId ? { conversationId: activePeer.conversationId } : {}),
      }
    : null;
  const activeAgentReply = activePeer && isAgentPeer(activePeer) && !activePeer.miniAppId
    ? agentWorkspaceController.replyForPeer(activePeer.key)
    : undefined;
  const {
    controller: agentSettingsController,
    snapshot: agentSettingsSnapshot,
  } = useAgentSettingsController(agentDirectoryController, activeAgentBot);
  const activeAgentKey = projectActiveAgentKey(activePeer);
  const activeAgentItem = activeAgentKey
    ? agentItems.find((item) => item.key === activeAgentKey) ?? null
    : null;
  const activeAgentPinned = activeAgentItem?.pinned === true;

  useEffect(() => {
    if (!initialAgentWorkspaceHydrated || !agentSidebarController.ready) return;
    agentSidebarController.adoptLegacyPinnedState(legacyPinnedAgentKeys);
  }, [
    initialAgentWorkspaceHydrated,
    agentSidebarController.ready,
    agentSidebarController.adoptLegacyPinnedState,
    legacyPinnedAgentSignature,
  ]);

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
  const sectionIsPeerList = ['chats', 'contacts', 'bots', 'groups', 'channels', 'saved', 'archive'].includes(section);
  const layoutInfoOpen = wideInfoLayout ? infoOpen : narrowInfoOpen;
  const infoPanelVisible = Boolean(layoutInfoOpen && activePeer && sectionIsPeerList);
  const infoPanelDocked = Boolean(infoPanelVisible && wideInfoLayout);
  const currentActor = selfActors.find((actor) => actor.id === selfHosted.actorId);
  const localComputerOnline = agentComputer.online;
  const localComputerStatus = agentComputer.status;
  const pendingRemoteAuthorization = agentComputer.state?.pendingAuthorization ?? null;
  // Normal Agent timelines render directly from AgentTranscriptStore. The
  // renderer-global messages array is now compatibility-only for Messenger,
  // groups and Mini Apps.
  const matchingMessages = messages;
  const renderedMessages = matchingMessages.slice(Math.max(0, matchingMessages.length - messageRenderCount));
  const botTranscriptMessages = renderedMessages;
  const allAgentTranscriptEntries: TranscriptEntry[] = activePeer && isAgentPeer(activePeer) && !activePeer.miniAppId
    ? agentTranscriptStore.entries(activePeer.key)
    : [];
  const agentTranscriptEntries: TranscriptEntry[] = allAgentTranscriptEntries.length || (activePeer && isAgentPeer(activePeer) && !activePeer.miniAppId)
    ? allAgentTranscriptEntries.slice(Math.max(0, allAgentTranscriptEntries.length - messageRenderCount))
    : activePeer && isAgentPeer(activePeer)
      ? projectTranscriptEntries(botTranscriptMessages)
      : [];
  const agentHasEarlierEntries = allAgentTranscriptEntries.length > agentTranscriptEntries.length;
  const activeAgentOperationId = activePeer
    ? agentOperationSnapshot[activePeer.key] ?? null
    : null;
  const activePeerBusy = Boolean(activePeer && (
    activeAgentOperationId
    || agentRequestSnapshot[activePeer.key]
  ));

  async function createAgent(): Promise<void> {
    setSection('bots');
    setSearch('');
    newAgentRequestPendingRef.current = true;
    let accepted: unknown;
    try {
      accepted = await agentDirectoryController.create({ name: 'New chat', description: '' });
    } catch {
      newAgentRequestPendingRef.current = false;
      return;
    }
    if (!accepted) newAgentRequestPendingRef.current = false;
  }

  function peerForAgent(item: AgentSidebarItem): PeerItem | undefined {
    return peersRef.current.find((peer) => peer.key === item.peerKey)
      ?? peersRef.current.find((peer) =>
        (peer.kind === 'bot' || peer.kind === 'group') && agentWorkspaceKey(peer) === item.key,
      );
  }

  async function renameAgent(item: AgentSidebarItem): Promise<void> {
    const peer = peerForAgent(item);
    if (!peer || peer.kind !== 'bot' || !peer.key.startsWith('legacy:bot:') && !peer.key.startsWith('legacy:conversation:')) {
      setError('Only locally managed Agents can be renamed from this shell.');
      return;
    }
    const name = window.prompt('Rename Agent', peer.title)?.trim();
    if (!name || name === peer.title) return;
    try {
      await agentDirectoryController.update(peer.actorId ?? peer.id, { name });
    } catch {
      return;
    }
  }

  async function duplicateAgent(item: AgentSidebarItem): Promise<void> {
    const peer = peerForAgent(item);
    const botId = peer?.source === 'legacy' && peer.kind === 'bot'
      ? peer.actorId ?? (peer.key.startsWith('legacy:bot:') ? peer.id : undefined)
      : undefined;
    if (!peer || !botId) {
      setError('Only locally managed Agents can be duplicated from this shell.');
      return;
    }
    try {
      await agentDirectoryController.duplicate(botId);
    } catch {
      return;
    }
  }

  async function deleteAgent(item: AgentSidebarItem, confirmDelete = true): Promise<void> {
    const peer = peerForAgent(item);
    const botId = peer?.source === 'legacy' && peer.kind === 'bot'
      ? peer.actorId ?? (peer.key.startsWith('legacy:bot:') ? peer.id : undefined)
      : undefined;
    if (!peer || !botId) {
      setError('Only locally managed Agents can be deleted from this shell.');
      return;
    }
    if (confirmDelete && !window.confirm(`Delete “${peer.title}”? The Agent store is retained for recovery.`)) return;
    agentWorkspaceController.clearPeer(peer.key);
    notifyAgentWorkspaceState();
    try {
      await agentDirectoryController.delete(botId);
    } catch {
      return;
    }
    if (peer.key === activePeerKeyRef.current) {
      activePeerKeyRef.current = null;
      setActivePeerKey(null);
      setMessages([]);
    }
  }

  async function hideAgent(item: AgentSidebarItem): Promise<void> {
    const peer = peerForAgent(item);
    const botId = peer?.source === 'legacy' && peer.kind === 'bot'
      ? peer.actorId ?? (peer.key.startsWith('legacy:bot:') ? peer.id : undefined)
      : undefined;
    if (!peer || !botId) {
      setError('Only locally managed Agents can be hidden from this shell.');
      return;
    }
    try {
      await agentDirectoryController.setHidden(botId, true);
    } catch {
      return;
    }
    if (peer.key === activePeerKeyRef.current) {
      setActivePeerKey(null);
      setMessages([]);
    }
  }

  function reorderPinnedAgents(
    moved: AgentSidebarItem,
    target: AgentSidebarItem,
    position: 'before' | 'after',
  ): void {
    agentSidebarController.reorderPinned(
      moved.key,
      target.key,
      position,
      agentItems.filter((item) => item.pinned).map((item) => item.key),
    );
  }

  function toggleAgentSelection(item: AgentSidebarItem): void {
    agentSidebarController.toggleSelection(item.key);
  }

  function rangeSelectAgent(item: AgentSidebarItem): void {
    const query = search.trim().toLocaleLowerCase();
    const orderedKeys = agentItems
      .filter((candidate) =>
        !candidate.hidden
        && (!query || `${candidate.name} ${candidate.description}`.toLocaleLowerCase().includes(query)),
      )
      .map((candidate) => candidate.key);
    agentSidebarController.rangeSelect(item.key, orderedKeys);
  }

  function clearAgentSelection(): void {
    agentSidebarController.clearSelection();
  }

  function createAgentSidebarSection(name: string, items: readonly AgentSidebarItem[]): void {
    agentSidebarController.createSection(name, items);
  }

  function moveAgentsToSection(items: readonly AgentSidebarItem[], sectionId: string): void {
    agentSidebarController.moveToSection(items, sectionId);
  }

  function moveAgentToSection(item: AgentSidebarItem, sectionId: string): void {
    agentSidebarController.moveToSection([item], sectionId);
  }

  function renameAgentSidebarSection(section: AgentSidebarSection): void {
    const name = window.prompt('Rename section', section.name)?.trim();
    if (!name || name === section.name) return;
    agentSidebarController.renameSection(section.id, name);
  }

  function deleteAgentSidebarSection(section: AgentSidebarSection): void {
    if (!window.confirm(`Delete “${section.name}”? Its Agents move to Unassigned.`)) return;
    agentSidebarController.removeSection(section.id);
  }

  async function deleteSelectedAgents(items: readonly AgentSidebarItem[]): Promise<void> {
    const deletable = items.filter((item) => !item.isGroup);
    if (!deletable.length) return;
    if (!window.confirm(`Delete ${deletable.length} selected Agent${deletable.length === 1 ? '' : 's'}? Agent stores are retained for recovery.`)) return;
    for (const item of deletable) {
      await deleteAgent(item, false);
    }
    clearAgentSelection();
  }

  function openAgent(item: AgentSidebarItem): void {
    const peer = peerForAgent(item);
    if (!peer) return;
    agentNetworkController.close();
    setSection('bots');
    void openPeer(peer);
  }

  function updateAgentComposer(peerKey: string, value: string, richText?: string) {
    agentWorkspaceController.setDraftDocument(peerKey, value, richText);
    notifyAgentWorkspaceState();
  }

  function updateComposer(value: string) {
    // Compatibility-only composer state for Messenger, Mini Apps and groups.
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
      if (value) next[activeKey] = value;
      else delete next[activeKey];
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

  function scrollToTranscriptEntry(entryId: string) {
    const root = messageAreaRef.current;
    if (!root) return;
    const target = Array.from(root.querySelectorAll<HTMLElement>('[data-transcript-entry-id]'))
      .find((element) => element.dataset.transcriptEntryId === entryId);
    target?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }

  async function copyBotMessage(message: BotTranscriptMessage): Promise<void> {
    try {
      await navigator.clipboard?.writeText(message.text);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : '复制消息失败');
    }
  }

  function clearAgentReply(peerKey: string): void {
    agentWorkspaceController.clearReply(peerKey);
    notifyAgentWorkspaceState();
  }

  function setReplyTarget(message: DisplayMessage): void {
    const peer = activePeer;
    if (peer && isAgentPeer(peer) && !peer.miniAppId) {
      agentWorkspaceController.setReply(peer.key, {
        id: message.id,
        role: message.role,
        text: message.text,
      });
      notifyAgentWorkspaceState();
      return;
    }
    setLegacyReplyTo(message);
  }

  function editBotMessage(message: BotTranscriptMessage | TranscriptEntry) {
    if (activePeer && isAgentPeer(activePeer) && !activePeer.miniAppId) {
      agentWorkspaceController.setDraftDocument(activePeer.key, message.text, 'richText' in message ? message.richText : undefined);
      notifyAgentWorkspaceState();
      clearAgentReply(activePeer.key);
      return;
    }
    updateComposer(message.text);
  }

  function enqueueAgentPrompt(
    peer: PeerItem,
    text: string,
    existingMessageId?: string,
    attachments: readonly AttachmentContext[] = [],
    replyContext?: AgentReplyContext,
    references: readonly AgentPromptReference[] = [],
    richText?: string,
  ) {    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    submitAgentWorkspace({
      ...(existingMessageId ? { messageId: existingMessageId } : {}),
      peerKey: peer.key,
      agentId,
      ...(peer.conversationId ? { conversationId: peer.conversationId } : {}),
      prompt: text,
      ...(richText ? { richText } : {}),
      ...(attachments.length ? { attachments: [...attachments] } : {}),
      ...(replyContext ? { replyTo: replyContext } : {}),
      ...(references.length ? { references: [...references] } : {}),
    });
  }

  function sendAgentMessage(event: FormEvent, peer: PeerItem): void {
    event.preventDefault();
    if (!isAgentPeer(peer) || peer.miniAppId || peer.kind === 'group') return;
    const text = agentWorkspaceController.draftForPeer(peer.key).trim();
    const attachments = agentWorkspaceController.attachmentsForPeer(peer.key);
    if (!text && !attachments.length) return;

    // Take the complete Agent-owned draft atomically. A failed/queued send is
    // restored by useAgentWorkspaceRuntime without overwriting a newer draft.
    const submittedDraft = agentWorkspaceController.takeDraft(peer.key);
    notifyAgentWorkspaceState();
    enqueueAgentPrompt(
      peer,
      submittedDraft.text.trim(),
      undefined,
      submittedDraft.attachments,
      submittedDraft.replyTo,
      submittedDraft.references ?? [],
      submittedDraft.richText,
    );
    setScheduledAtMs(undefined);
  }

  function regenerateBotMessage(message: BotTranscriptMessage | TranscriptEntry) {
    if (!activePeer || !isAgentPeer(activePeer) || activePeer.miniAppId) return;
    const prompt = agentTranscriptStore.userPromptBefore(activePeer.key, message.id);
    if (!prompt) {
      setError('找不到这条回复对应的用户消息。');
      return;
    }
    agentTranscriptStore.removeByIds(activePeer.key, [message.id]);
    notifyAgentWorkspaceState();
    enqueueAgentPrompt(activePeer, prompt.text, undefined, prompt.attachments ?? [], undefined, [], prompt.richText);
  }

  async function stopAgentOperation(): Promise<void> {
    try {
      await interruptAgentWorkspace(activePeerKeyRef.current);
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
  }, [activePeerKey, messages, agentWorkspaceRevision]);

  async function openPeer(peer: PeerItem) {
    activePeerKeyRef.current = peer.key;
    setActivePeerKey(peer.key);
    if (!isAgentPeer(peer)) setLegacySendPending(false);
    stickToLatestRef.current = true;
    setShowScrollToLatest(false);
    const isWorkspaceAgent = peer.source === 'legacy'
      && peer.kind !== 'group'
      && !peer.miniAppId
      && isAgentPeer(peer);
    if (!isWorkspaceAgent) setComposer(drafts[peer.key] ?? '');
    setSearch('');
    setMessageRenderCount(initialMessageRenderCount);
    setLegacyReplyTo(null);
    setError(null);
    setConversationSearchOpen(false);
    setAgentConversationSearch('');
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
      if (!agentTranscriptStore.has(peer.key) && peer.conversationId) {
        const cached = cachedLegacyDisplayMessages(peer.conversationId);
        if (cached.length) {
          agentTranscriptStore.replace(peer.key, toAgentTranscriptSources(cached));
        } else if (!(await restoreAgentConversationFromCloud(peer))) {
          agentTranscriptStore.replace(peer.key, []);
        }
      } else if (!agentTranscriptStore.has(peer.key)) {
        agentTranscriptStore.replace(peer.key, []);
      }
      notifyAgentWorkspaceState();
      if (peer.conversationId) {
        await openAgentConversation(peer.key, peer.conversationId).catch(() => {});
      }
      void agentWorkflowController.list(peer.agentId ?? peer.actorId ?? peer.id).catch(() => {});
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
      await agentCoordinatorClient.openConversation(nextRequestId('conversation-open'), peer.conversationId).catch((cause: unknown) => {
          setError(cause instanceof Error ? cause.message : String(cause));
        });
      return;
    }
    setMessages([]);
  }

  async function sendMessage(event: FormEvent) {
    event.preventDefault();
    if (!activePeer) return;
    const agentRequest = activePeer.source === 'legacy'
      && activePeer.kind !== 'group'
      && !activePeer.miniAppId
      && isAgentPeer(activePeer);
    if (agentRequest) {
      sendAgentMessage(event, activePeer);
      return;
    }
    const text = composer.trim();
    if (!text || legacySendPending) return;
    setLegacySendPending(true);
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
          replyToMessageId: legacyReplyTo?.id,
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
      }
      setLegacyReplyTo(null);
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
      setLegacySendPending(false);
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

  async function transcribeAgentVoice(peer: PeerItem, file: File): Promise<string> {
    if (!isAgentPeer(peer) || peer.miniAppId) throw new Error('Voice dictation is only available for Agents.');
    const validationError = validateAgentAttachment(file);
    if (validationError) throw new Error(validationError);
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    const stored = await uploadAgentAttachment(peer.key, {
      agentId,
      filename: file.name,
      mimeType: file.type || 'audio/webm',
      bytesBase64: await agentFileToBase64(file),
    });
    if (!stored.path) throw new Error('Voice recording was stored without a local path.');
    const result = await invokeNativeDesktop<Record<string, unknown>>('transcribeAudio', {
      path: stored.path,
      language: 'auto',
    });
    const nested = result?.result && typeof result.result === 'object'
      ? result.result as Record<string, unknown>
      : null;
    const text = typeof result?.text === 'string'
      ? result.text
      : typeof result?.transcript === 'string'
        ? result.transcript
        : typeof nested?.text === 'string'
          ? nested.text
          : '';
    if (text.trim()) return text.trim();
    if (result?.available === false) {
      throw new Error('Voice transcription is unavailable. Install the offline ASR model or enable a runtime transcription tool.');
    }
    throw new Error('Voice transcription returned no text.');
  }

  async function stageAgentFiles(peer: PeerItem, files: readonly File[]): Promise<void> {
    if (!isAgentPeer(peer) || peer.miniAppId || files.length === 0) return;
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    const controller = agentWorkspaceController;
    const alreadyStaged = controller.attachmentsForPeer(peer.key);
    const availableSlots = Math.max(0, AGENT_ATTACHMENT_LIMIT - alreadyStaged.length);
    if (availableSlots === 0) {
      setError(`You can attach up to ${AGENT_ATTACHMENT_LIMIT} files to one Agent draft.`);
      return;
    }

    controller.setUploading(peer.key, true);
    notifyAgentWorkspaceState();
    const staged: AttachmentContext[] = [];
    try {
      for (const file of files.slice(0, availableSlots)) {
        const validationError = validateAgentAttachment(file);
        if (validationError) {
          setError(validationError);
          continue;
        }
        try {
          const stored = await uploadAgentAttachment(peer.key, {
            agentId,
            filename: file.name,
            mimeType: file.type || undefined,
            bytesBase64: await agentFileToBase64(file),
          });
          staged.push(await enrichAgentAttachmentPreview(stored, file));
        } catch (cause) {
          setError(cause instanceof Error ? cause.message : String(cause));
        }
      }
      if (staged.length) {
        controller.appendAttachments(peer.key, staged);
        notifyAgentWorkspaceState();
        const nextAttachments = controller.attachmentsForPeer(peer.key);
        void mirrorAgentCloudSnapshot(agentId, FABU_AGENT_ATTACHMENT_INDEX_PATH, {
          schemaVersion: 1,
          agentId,
          attachments: nextAttachments.map((attachment) => ({
            id: attachment.id,
            name: attachment.name,
            path: attachment.path,
            mimeType: attachment.mimeType,
            sizeBytes: attachment.sizeBytes,
          })),
          updatedAtMs: Date.now(),
        });
      }
    } finally {
      controller.setUploading(peer.key, false);
      notifyAgentWorkspaceState();
    }
  }

  function removeAgentAttachment(peerKey: string, attachmentId: string): void {
    const controller = agentWorkspaceController;
    controller.removeAttachment(peerKey, attachmentId);
    notifyAgentWorkspaceState();
    const remaining = controller.attachmentsForPeer(peerKey);
    const peer = agentPeerForRuntimeKey(peerKey);
    if (!peer) return;
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    void mirrorAgentCloudSnapshot(agentId, FABU_AGENT_ATTACHMENT_INDEX_PATH, {
      schemaVersion: 1,
      agentId,
      attachments: remaining.map((attachment) => ({
        id: attachment.id,
        name: attachment.name,
        path: attachment.path,
        mimeType: attachment.mimeType,
        sizeBytes: attachment.sizeBytes,
      })),
      updatedAtMs: Date.now(),
    });
  }

  async function sendAttachmentFile(file: File) {
    if (!activePeer?.conversationId || activePeer.source !== 'selfhosted') {
      setError('附件需要发送到 Fabushi 自建会话。');
      return;
    }
    setAttachmentMenuOpen(false);
    setLegacySendPending(true);
    setAttachmentProgress(`正在上传 ${file.name}…`);
    try {
      await selfHosted.sendAttachment(
        activePeer.conversationId,
        file,
        { replyToMessageId: legacyReplyTo?.id, scheduledAtMs, silent: silentSend },
        (uploaded, total) => setAttachmentProgress(`正在上传 ${file.name} · ${Math.round((uploaded / total) * 100)}%`),
      );
      setLegacyReplyTo(null);
      setScheduledAtMs(undefined);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLegacySendPending(false);
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
      setReplyTarget(target);
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

  function closeSettings() {
    setSection(settingsReturnSectionRef.current === 'settings' ? 'bots' : settingsReturnSectionRef.current);
  }

  return (
    <AgentRootShell
      className={`${styles.messenger} ${styles.fabushiUnified}`}
      data-testid="messenger-workspace"
      data-product-shell="agent"
      data-initial-host-hydrated={initialAgentWorkspaceHydrated ? 'true' : undefined}
      data-legacy-compatibility-hydrated={initialLegacyHydrated ? 'true' : undefined}
      data-sidebar-collapsed={sidebarWidth <= 112 || undefined}
      data-reduce-motion={desktopPreferences.reducedMotion || undefined}
      data-testid-ready-projection={startupProjection ? 'true' : undefined}
      style={{ gridTemplateColumns: infoPanelDocked ? `${sidebarWidth}px minmax(420px,1fr) 286px` : `${sidebarWidth}px minmax(420px,1fr)` }}
      onClick={() => { setMessageMenu(null); }}
    >
      <aside className={styles.chatList} data-testid="messenger-sidebar" data-collapsed={sidebarWidth <= 112 || undefined} onClick={(event) => event.stopPropagation()}>
        <AgentSidebar
          agents={agentItems}
          activeKey={activeAgentKey}
          query={search}
          collapsed={sidebarWidth <= 112}
          hostReady={hostReady}
          accountLabel={currentActor?.displayName ?? 'Account'}
          sections={agentSidebarSections}
          selectedKeys={agentSelectedKeys}
          onQuery={setSearch}
          onOpen={openAgent}
          onNewAgent={() => void createAgent()}
          onToggleCollapsed={() => setSidebarWidth((width) => width <= 112 ? 300 : 88)}
          onTogglePin={(item) => {
            agentSidebarController.togglePin(item.key);
          }}
          onRename={(item) => void renameAgent(item)}
          onHide={(item) => void hideAgent(item)}
          onDuplicate={(item) => void duplicateAgent(item)}
          onDelete={(item) => void deleteAgent(item)}
          onReorderPinned={reorderPinnedAgents}
          onToggleSelection={toggleAgentSelection}
          onRangeSelection={rangeSelectAgent}
          onClearSelection={clearAgentSelection}
          onDeleteSelected={(items) => void deleteSelectedAgents(items)}
          onMoveSelectedToSection={moveAgentsToSection}
          onCreateSection={createAgentSidebarSection}
          onToggleSection={(section) => agentSidebarController.toggleSection(section.id)}
          onRenameSection={renameAgentSidebarSection}
          onDeleteSection={deleteAgentSidebarSection}
          onMoveToSection={moveAgentToSection}
          onBroadcast={agentNetworkController.openBroadcast}
          onOpenNetwork={agentNetworkController.openNetwork}
          onOpenPlugins={() => {
            setSearch('');
            setSection('miniapps');
          }}
          onOpenSettings={() => {
            settingsReturnSectionRef.current = 'bots';
            setSection('settings');
          }}
        />
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

      <AgentCommandPalette
        open={agentPaletteController.open}
        agents={agentItems}
        query={agentPaletteController.query}
        onQuery={agentPaletteController.setQuery}
        onClose={agentPaletteController.close}
        onOpenAgent={openAgent}
        onNewAgent={() => void createAgent()}
        onNetwork={agentNetworkController.openNetwork}
        onBroadcast={agentNetworkController.openBroadcast}
        onPlugins={() => {
          setSearch('');
          setSection('miniapps');
        }}
        onSettings={() => {
          settingsReturnSectionRef.current = 'bots';
          setSection('settings');
        }}
        entries={allAgentTranscriptEntries}
        onOpenTranscriptEntry={scrollToTranscriptEntry}
        onConversationSearch={() => {
          setConversationSearchOpen(true);
          setAgentConversationSearch('');
        }}
        onComputer={() => {
          if (!activePeer || !isAgentPeer(activePeer) || activePeer.miniAppId) return;
          setAgentSettingsOpen(false);
          agentComputer.openForAgent(activePeer.agentId ?? activePeer.actorId ?? activePeer.id, 'command-palette');
          if (wideInfoLayout) setInfoOpen(true); else setNarrowInfoOpen(true);
        }}
      />

      <section className={styles.chatWorkspace}>
        <AgentNetwork
          open={agentNetworkController.open}
          agents={agentItems}
          groups={agentNetworkController.groups}
          peerMessagesByAgentId={agentNetworkController.peerMessagesByAgentId}
          activeKey={activeAgentKey}
          broadcastMode={agentNetworkController.broadcastMode}
          onClose={agentNetworkController.close}
          onOpenAgent={openAgent}
          onRefreshGroups={agentNetworkController.refreshGroups}
          onRefreshPeerHistory={agentNetworkController.refreshPeerHistory}
          onCreateGroup={agentNetworkController.createGroup}
          onUpdateGroup={agentNetworkController.updateGroup}
          onDeleteGroup={agentNetworkController.deleteGroup}
          onSendGroup={agentNetworkController.sendGroup}
          onSendPeer={agentNetworkController.sendPeer}
          onBroadcast={agentNetworkController.broadcast}
        />
        {agentNetworkController.open ? null : activePeer && sectionIsPeerList ? (
          <>
            {isAgentPeer(activePeer) && !activePeer.miniAppId ? (
              <AgentWorkspace
                title={activePeer.title}
                description={activePeer.subtitle}
                botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`}
                botState={botMarkStateForPeer(activePeer, selfBotExecutions, activePeerBusy, hostReady)}
                status={activeTypingActors.length
                  ? 'Typing…'
                  : activePeerBusy
                    ? 'Working…'
                    : `${activePeer.subtitle}${hostReady ? ' · Online' : ' · Connecting'}`}
                pinned={activeAgentPinned}
                searchActive={conversationSearchOpen}
                searchQuery={agentConversationSearch}
                computerActive={agentComputer.open}
                infoActive={layoutInfoOpen}
                onToggleSearch={() => {
                  const next = !conversationSearchOpen;
                  setConversationSearchOpen(next);
                  if (!next) setAgentConversationSearch('');
                }}
                onSearchQuery={setAgentConversationSearch}
                onSelectSearchResult={scrollToTranscriptEntry}
                onToggleComputer={() => {
                  setAgentSettingsOpen(false);
                  agentComputer.toggleForAgent(activePeer.agentId ?? activePeer.actorId ?? activePeer.id, 'agent-header');
                  if (wideInfoLayout) setInfoOpen(true); else setNarrowInfoOpen(true);
                }}
                onTogglePin={() => {
                  if (activeAgentKey) agentSidebarController.togglePin(activeAgentKey);
                }}
                onToggleInfo={() => wideInfoLayout ? setInfoOpen((value) => !value) : setNarrowInfoOpen((value) => !value)}
                entries={agentTranscriptEntries}
                activeOperationId={activeAgentOperationId}
                hasEarlierMessages={agentHasEarlierEntries}
                messageAreaRef={messageAreaRef}
                showScrollToLatest={showScrollToLatest}
                onLoadEarlier={() => setMessageRenderCount((count) => count + initialMessageRenderCount)}
                onOpenMiniApp={(id) => void openMiniApp(id)}
                onScroll={handleMessageAreaScroll}
                onScrollToLatest={scrollToLatest}
                onCopyMessage={(entry) => void copyBotMessage(entry as BotTranscriptMessage)}
                onRegenerate={(entry) => regenerateBotMessage(entry as BotTranscriptMessage)}
                onEdit={(entry) => editBotMessage(entry)}
                onResolveApproval={(approvalId, decision) => {
                  void resolveAgentApproval({ approvalId, decision }).catch((cause: unknown) => {
                    setError(cause instanceof Error ? cause.message : String(cause));
                  });
                }}
                onContextMenu={(event, entry) => {
                  if (entry.kind !== 'message' && entry.kind !== 'assistant-turn') return;
                  event.preventDefault();
                  event.stopPropagation();
                  setMessageMenu({
                    message: {
                      id: entry.id,
                      source: 'legacy',
                      role: entry.role,
                      text: entry.text,
                      createdAtMs: entry.createdAtMs,
                      kind: entry.kind,
                      ...(entry.operationId ? { operationId: entry.operationId } : {}),
                      ...(entry.streaming ? { streaming: true } : {}),
                      ...(entry.optimistic ? { optimistic: true } : {}),
                      ...(entry.queued ? { queued: true } : {}),
                      ...(entry.assistantTurn ? { assistantTurn: entry.assistantTurn } : {}),
                      ...(entry.attachments?.length ? { attachments: entry.attachments } : {}),
                    },
                    x: event.clientX,
                    y: event.clientY,
                  });
                }}
                notice={error ? <div className={styles.errorBanner} role="alert"><span>{error}</span><button type="button" onClick={() => setError(null)}><X size={14} /></button></div> : null}
                composerReplyTarget={activeAgentReply ? { id: activeAgentReply.id, label: '回复', text: activeAgentReply.text } : undefined}
                onClearComposerReply={() => clearAgentReply(activePeer.key)}
                composerAccessory={agentWorkspaceController.isUploading(activePeer.key)
                  ? <span className={extra.uploadProgress}>Uploading attachments…</span>
                  : null}
                composerValue={agentWorkspaceController.draftForPeer(activePeer.key)}
                composerRichText={agentWorkspaceController.richTextForPeer(activePeer.key)}
                composerReady={hostReady}
                composerBusy={Boolean(activeAgentOperationId)}
                composerUploading={agentWorkspaceController.isUploading(activePeer.key)}
                composerAttachments={agentWorkspaceController.attachmentsForPeer(activePeer.key)}
                composerMentionCandidates={[
                  ...agentItems
                    .filter((item) => item.key !== activePeer.key)
                    .map((item) => ({
                      id: item.agentId || item.key,
                      name: item.name,
                      description: item.description,
                      kind: 'agent' as const,
                    })),
                  ...agentMcpController.references.map((reference) => ({
                    id: reference.id,
                    name: reference.name,
                    description: reference.description,
                    kind: 'mcp' as const,
                  })),
                ]}
                composerWorkflowCandidates={(agentWorkflowController.workflowsByAgentId[activePeer.agentId ?? activePeer.actorId ?? activePeer.id] ?? [])
                  .filter((workflow) => workflow.isEnabledForAgent)
                  .map((workflow) => ({ id: workflow.id, name: workflow.name, description: workflow.description }))}
                composerPullRequestCandidates={agentPrSuggestionController.candidates}
                enterToSend={desktopPreferences.enterToSend}
                onComposerChange={(value, richText) => updateAgentComposer(activePeer.key, value, richText)}
                onComposerMention={(candidate) => {
                  agentWorkspaceController.upsertReference(activePeer.key, {
                    kind: candidate.kind === 'mcp' ? 'mcp' : 'agent',
                    id: candidate.id,
                    label: candidate.name,
                  });
                  notifyAgentWorkspaceState();
                }}
                onComposerWorkflowReference={(candidate) => {
                  agentWorkspaceController.upsertReference(activePeer.key, {
                    kind: 'workflow',
                    id: candidate.id,
                    label: candidate.name,
                  });
                  notifyAgentWorkspaceState();
                }}
                onComposerSubmit={(event) => sendAgentMessage(event, activePeer)}
                onComposerFiles={(files) => void stageAgentFiles(activePeer, files)}
                onRemoveComposerAttachment={(attachmentId) => removeAgentAttachment(activePeer.key, attachmentId)}
                onTranscribeVoice={(file) => transcribeAgentVoice(activePeer, file)}
                onStop={() => void stopAgentOperation()}
              />
            ) : (
              <>
            {isAgentPeer(activePeer) ? (
              <AgentHeader
                title={activePeer.title}
                description={activePeer.subtitle}
                botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`}
                botState={botMarkStateForPeer(activePeer, selfBotExecutions, activePeerBusy, hostReady)}
                status={activeTypingActors.length
                  ? 'Typing…'
                  : activePeerBusy
                    ? 'Working…'
                    : `${activePeer.subtitle}${hostReady ? ' · Online' : ' · Connecting'}`}
                pinned={activePeer.pinned}
                searchActive={conversationSearchOpen}
                computerActive={agentComputer.open}
                infoActive={layoutInfoOpen}
                miniAppTitle={activePeer.miniAppMenuButtonText}
                onOpenMiniApp={activePeer.miniAppId ? () => void openMiniApp(activePeer.miniAppId!) : undefined}
                onToggleSearch={() => {
                  const next = !conversationSearchOpen;
                  setConversationSearchOpen(next);
                  if (!next) setAgentConversationSearch('');
                  }}
                onToggleComputer={() => {
                  agentComputer.toggleForAgent(activePeer.agentId ?? activePeer.actorId ?? activePeer.id, 'compatibility-agent-header');
                  if (wideInfoLayout) setInfoOpen(true); else setNarrowInfoOpen(true);
                }}
                onTogglePin={() => void togglePinConversation(activePeer)}
                onToggleInfo={() => wideInfoLayout ? setInfoOpen((value) => !value) : setNarrowInfoOpen((value) => !value)}
              />
            ) : (
              <header className={styles.chatHeader}>
                <div className={styles.chatIdentity}>
                  <BotMark botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state="idle" size={40} className={styles.agentAvatarMark} label={activePeer.title} />
                  <div><strong>{activePeer.title}</strong><small data-testid="conversation-status">{activeTypingActors.length ? '正在输入…' : `${activePeer.subtitle}${hostReady ? ' · 在线' : ' · 正在连接'}`}</small></div>
                </div>
                <div className={styles.headerActions}>
                  <button type="button" title="搜索当前会话" data-active={conversationSearchOpen} onClick={() => {
                    const next = !conversationSearchOpen;
                    setConversationSearchOpen(next);
                    setSearch('');
                    }}><Search size={18} /></button>
                  <button type="button" title={activePeer.pinned ? '取消置顶' : '置顶'} onClick={() => void togglePinConversation(activePeer)}><Pin size={18} /></button>
                  <button type="button" title="资料" data-testid="conversation-info-toggle" data-active={layoutInfoOpen} onClick={() => wideInfoLayout ? setInfoOpen((value) => !value) : setNarrowInfoOpen((value) => !value)}><MoreVertical size={18} /></button>
                </div>
              </header>
            )}
            {conversationSearchOpen ? <AgentSearch
              entries={isAgentPeer(activePeer) ? agentTranscriptEntries : projectTranscriptEntries(renderedMessages)}
              query={agentConversationSearch}
              onQuery={setAgentConversationSearch}
              onClose={() => {
                setConversationSearchOpen(false);
                setAgentConversationSearch('');
              }}
              onSelect={scrollToTranscriptEntry}
            /> : null}
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
            <div className={styles.messageArea} data-testid="message-list" data-agent-operation-id={activeAgentOperationId ?? undefined}>
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
                    <button type="button" title="回复" aria-label="回复" onClick={() => setReplyTarget(message)}><span aria-hidden="true">↩</span></button>
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
            {legacyReplyTo ? <div className={extra.composerBanner} data-testid="reply-message-banner"><Reply size={15} /><div><strong>回复</strong><span>{legacyReplyTo.text}</span></div><button type="button" data-testid="reply-message-cancel" onClick={() => setLegacyReplyTo(null)}><X size={14} /></button></div> : null}
            {scheduledAtMs ? <div className={extra.composerBanner}><span>⏱</span><div><strong>定时发送</strong><span>{new Date(scheduledAtMs).toLocaleString()}</span></div><button type="button" onClick={() => setScheduledAtMs(undefined)}><X size={14} /></button></div> : null}
            {activePeer.miniAppId && composer.trimStart().startsWith('/') && activePeer.miniAppCommands?.length ? <div className={extra.composerBanner} data-testid="miniapp-bot-commands"><AppWindow size={15} /><div><strong>小程序命令</strong><span>{activePeer.miniAppCommands.map((command) => `/${command.name}`).join(' · ')}</span></div>{activePeer.miniAppCommands.slice(0, 4).map((command) => <button key={command.name} type="button" title={command.description} onClick={() => updateComposer(command.usage)}>{`/${command.name}`}</button>)}</div> : null}
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
                {composer.trim()
                  ? <button data-testid="messenger-send" className={styles.sendButton} type="submit" disabled={!hostReady || legacySendPending}><Send size={19} /></button>
                  : null}
              </form>
              </>
            )}
          </>
        ) : (
          <>
            {error ? <div className={styles.errorBanner} role="alert"><span>{error}</span><button type="button" onClick={() => setError(null)}><X size={14} /></button></div> : null}
            <FeatureWorkspace section={section} onOpenMiniApp={openMiniApp} onInstallMiniApp={installMiniApp} onUninstallMiniApp={uninstallMiniApp} miniApps={marketplaceApps} installedMiniApps={installedMiniApps} miniAppQuery={miniAppQuery} onMiniAppQuery={setMiniAppQuery} miniAppLoading={miniAppLoading} miniAppBusy={miniAppBusy} onInvoice={() => void createInvoiceForActivePeer()} payment={{ account: walletAccount, entries: walletEntries, orders: selfOrders, invoices: selfInvoices, actorId: selfHosted.actorId }} onRefund={(orderId) => void refundOrder(orderId)} settings={{ category: settingsCategory, onCategory: setSettingsCategory, preferences: desktopPreferences, onPreference: updateDesktopPreference, actor: currentActor, actorId: selfHosted.actorId, hostSettings, onHostSetting: updateHostSetting, onConfigureProviderSecret: configureProviderSecret, onRemoveProviderSecret: removeProviderSecret, routerStatus, usageSummary, onInstallUpdate: installDesktopUpdate, onLogout }} />
          </>
        )}
      </section>

      {infoPanelVisible && activePeer ? (
        isAgentPeer(activePeer) && !activePeer.miniAppId ? (
          <AgentOverlays
            title={activePeer.title}
            description={activePeer.subtitle}
            botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`}
            botState={botMarkStateForPeer(activePeer, selfBotExecutions, activePeerBusy, hostReady)}
            pinned={activeAgentPinned}
            overlay={!wideInfoLayout}
            onClose={() => wideInfoLayout ? setInfoOpen(false) : setNarrowInfoOpen(false)}
            onSearch={() => {
              setConversationSearchOpen(true);
              setAgentConversationSearch('');
            }}
            onTogglePin={() => {
              if (activeAgentKey) agentSidebarController.togglePin(activeAgentKey);
            }}
            computer={{
              agentId: activePeer.agentId ?? activePeer.actorId ?? activePeer.id,
              open: agentComputer.open,
              label: localComputerLabel(),
              status: localComputerStatus,
              online: localComputerOnline,
              aiControlEnabled: hostSettings.aiComputerControlEnabled,
              remoteControlEnabled: hostSettings.remoteControlEnabled,
              state: agentComputer.state,
              capabilityStatus: agentComputer.capabilityStatus,
              onToggle: () => {
                setAgentSettingsOpen(false);
                agentComputer.toggleForAgent(activePeer.agentId ?? activePeer.actorId ?? activePeer.id, 'agent-overlay');
              },
              onRefreshPairingCode: agentComputer.refreshPairingCode,
              onApproveSession: agentComputer.approveSession,
              onDenySession: agentComputer.denySession,
              onDisconnect: agentComputer.disconnect,
              onToggleRemoteControl: () => updateHostSetting('remoteControlEnabled', !hostSettings.remoteControlEnabled),
              onOpenControlPage: () => agentComputer.openControlPage(activePeer.agentId ?? activePeer.actorId ?? activePeer.id),
            }}
            settings={{
              agentId: activePeer.agentId ?? activePeer.actorId ?? activePeer.id,
              open: agentSettingsOpen,
              value: agentSettingsSnapshot.value ?? {
                name: activePeer.title,
                title: '',
                description: activePeer.subtitle,
                avatarShape: activeAgentBot?.avatarShape ?? '',
                avatarColor: activeAgentBot?.avatarColor ?? '',
                notifyOnUpdatesEnabled: true,
                inferenceProvider: activeAgentBot?.inferenceProvider ?? 'account-default',
              },
              pending: agentSettingsSnapshot.pending,
              error: agentSettingsSnapshot.error,
              onToggle: () => {
                agentComputer.close();
                setAgentSettingsOpen((value) => !value);
              },
              onUpdateProfile: (profile) => agentSettingsController.updateProfile(profile),
              onSetNotifications: (enabled) => agentSettingsController.setNotifications(enabled),
              onSetInferenceProvider: (provider) => agentSettingsController.setInferenceProvider(provider),
            }}
          />
        ) : (
        <aside className={styles.infoPanel} data-testid="messenger-info-panel" data-overlay={!wideInfoLayout || undefined}>
          <header><strong>资料</strong><button type="button" onClick={() => wideInfoLayout ? setInfoOpen(false) : setNarrowInfoOpen(false)}><X size={17} /></button></header>
          <div className={styles.profileCard}>
            <BotMark botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state={isAgentPeer(activePeer) ? botMarkStateForPeer(activePeer, selfBotExecutions, activePeerBusy, hostReady) : 'idle'} size={92} className={styles.agentProfileMark} label={activePeer.title} />
            <strong>{activePeer.title}</strong><small>{activePeer.subtitle}</small>
            <div className={styles.profileQuickActions} data-columns={isAgentPeer(activePeer) ? '4' : '3'}><button type="button" onClick={() => void startCall('voice')}><PhoneCall size={18} /><span>通话</span></button><button type="button" onClick={() => void startCall('video')}><Video size={18} /><span>视频</span></button><button type="button" onClick={() => { setConversationSearchOpen(true); setAgentConversationSearch(''); }}><Search size={18} /><span>搜索</span></button>{isAgentPeer(activePeer) ? <button type="button" data-testid="bot-computer-toggle" data-active={agentComputer.open} onClick={() => {
              agentComputer.toggleForAgent(activePeer.agentId ?? activePeer.actorId ?? activePeer.id, 'bot-profile');
            }}><Monitor size={18} /><span>电脑</span></button> : null}</div>
          </div>
          <div className={styles.profileActions}>
            <button type="button" onClick={() => void toggleMuteConversation(activePeer)}><BellOff size={17} /><span>{mutedPeerKeys.has(activePeer.key) ? '开启通知' : '静音通知'}</span></button>
            <button type="button" onClick={() => void togglePinConversation(activePeer)}><Pin size={17} /><span>{activePeer.pinned ? '取消置顶' : '置顶会话'}</span></button>
            <button type="button" onClick={() => { void toggleArchiveConversation(activePeer); setSection('archive'); }}><Archive size={17} /><span>{activePeer.archived ? '移出归档' : '归档会话'}</span></button>
            {activePeer.source === 'selfhosted' && ['group', 'channel'].includes(activePeer.kind) ? <button type="button" onClick={() => void openCommunityAdmin(activePeer)}><Settings size={17} /><span>管理群组/频道</span></button> : null}
          </div>
          {isAgentPeer(activePeer) && agentComputer.open ? <section className={styles.computerProfile} data-testid="bot-computer-panel">
            <header>
              <span className={styles.computerProfileIcon}><Monitor size={18} /></span>
              <span><strong>这台电脑</strong><small>{localComputerLabel()}</small></span>
              <i data-live={agentComputer.state?.channelOpen ? 'active' : localComputerOnline ? 'online' : 'offline'} />
            </header>
            <div className={styles.computerProfileStatus}>
              <span><small>设备状态</small><strong>{localComputerStatus}</strong></span>
              <span><small>已授权客户端</small><strong>{agentComputer.state?.clients.length ?? 0}</strong></span>
              <span><small>AI 操控</small><strong>{hostSettings.aiComputerControlEnabled ? '已允许' : '已关闭'}</strong></span>
            </div>
            <p>Fabushi 登录后会在后台保持设备在线；远控关闭时只能被同账号发现，不能读取画面或发送输入。</p>
            {hostSettings.remoteControlEnabled && agentComputer.state?.registration?.pairingCode ? <div className={styles.computerPairingCode}>
              <span><small>配对码</small><strong>{agentComputer.state.registration.pairingCode}</strong></span>
              <button type="button" onClick={() => agentComputer.refreshPairingCode()}>刷新</button>
            </div> : null}
            {pendingRemoteAuthorization ? <div className={styles.computerPairingCode} data-testid="remote-session-consent">
              <span><small>远控请求</small><strong>{pendingRemoteAuthorization.clientLabel || '已配对设备'}</strong></span>
              <button type="button" onClick={() => agentComputer.approveSession(pendingRemoteAuthorization.sessionId)}>允许本次连接</button>
              <button type="button" className={styles.computerDangerButton} onClick={() => agentComputer.denySession(pendingRemoteAuthorization.sessionId)}>拒绝</button>
            </div> : null}
            {agentComputer.state?.activeSessionId ? <button type="button" className={styles.computerDangerButton} onClick={() => agentComputer.disconnect()}>断开当前远控</button> : null}
            <div className={styles.computerProfileButtons}>
              <button type="button" data-enabled={hostSettings.remoteControlEnabled} onClick={() => updateHostSetting('remoteControlEnabled', !hostSettings.remoteControlEnabled)}>{hostSettings.remoteControlEnabled ? '关闭远程控制' : '开启远程控制'}</button>
              <button type="button" onClick={() => agentComputer.openControlPage(activePeer.agentId ?? activePeer.actorId ?? activePeer.id)}>打开控制页面</button>
            </div>
            {agentComputer.state?.error ? <small className={styles.computerProfileError}>{agentComputer.state.error}</small> : null}
          </section> : null}
          <nav className={styles.infoTabs}><button type="button" data-active={infoTab === 'media'} onClick={() => setInfoTab('media')}>媒体</button><button type="button" data-active={infoTab === 'files'} onClick={() => setInfoTab('files')}>文件</button><button type="button" data-active={infoTab === 'links'} onClick={() => setInfoTab('links')}>链接</button></nav>
          <div className={styles.infoContent}>{infoTab === 'media' ? <><Image size={30} /><strong>共享媒体</strong><p>图片、视频和动画按消息索引展示。</p></> : null}{infoTab === 'files' ? <><FileText size={30} /><strong>共享文件</strong><p>文档、音频和附件由 Rust 媒体层管理。</p></> : null}{infoTab === 'links' ? <><Link2 size={30} /><strong>共享链接</strong><p>富文本 URL 建立可搜索索引。</p></> : null}</div>
        </aside>
        )
      ) : null}

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
    </AgentRootShell>
  );
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

function FeatureWorkspace({ section, onInvoice, payment, onRefund, settings: _settings, ...miniAppProps }: { section: MessengerSection; onInvoice: () => void; payment: PaymentUiState; onRefund: (orderId: string) => void; settings: SettingsWorkspaceProps } & MiniAppMarketplaceProps) {
  return <CompatibilitySurface
    section={section}
    miniApps={<MiniAppMarketplaceWorkspace {...miniAppProps} />}
    payments={<div className={styles.featureWorkspace}><WalletCards size={54} /><h2>Fabushi Pay</h2><p>自建余额、Invoice、Order、退款与外部 settlement 都由 Rust 账本结算。</p><PaymentOverview payment={payment} onInvoice={onInvoice} onRefund={onRefund} /></div>}
    calls={<div className={styles.featureWorkspace}><Phone size={54} /><h2>通话</h2><p>本机媒体已接通，Rust realtime 已具备一对一/群组通话信令状态。</p></div>}
    settings={<div className={styles.featureWorkspace} aria-hidden="true" />}
    fallback={<div className={styles.featureWorkspace}><MessageCircle size={54} /><h2>{sectionTitle(section)}</h2><p>联系人、Bot、群组和频道正在统一到同一个 Fabushi Actor/Conversation 模型。</p></div>}
  />;
}
