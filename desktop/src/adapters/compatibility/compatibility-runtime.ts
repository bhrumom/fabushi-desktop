import type { ProductHostSettings, UpdateState } from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { ElectronMahayanaHostTransport, isElectronMahayanaHostAvailable, MAHAYANA_ACCOUNT_SESSION_RESET_EVENT, readCachedConversationMessages } from '../../../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import { MockMahayanaHostTransport } from '../../../../frontend/apps/web/src/lib/mahayana-host/mock-transport';
import { invokeNativeDesktop } from '../../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import type { MahayanaHostTransport } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import type { MessagingCommunityState, MessagingMediaRef } from '../../selfhosted-messaging-client-v2';
import type { CompatibilityPeerItem as PeerItem, CompatibilitySection as MessengerSection } from './messaging-compatibility-adapter';
import type { DesktopMessengerPreferences, DisplayMessage, MessengerProjection } from './compatibility-model';

export const messengerSettingsKey = 'fabushi.desktop.messenger-settings.v2';
export const messengerDraftsKey = 'fabushi.desktop.messenger-drafts.v2';
export const messengerSidebarWidthKey = 'fabushi.desktop.sidebar-width.v3';
export const messengerProjectionKey = 'fabushi.desktop.messenger-projection.v1';
export const accountSyncCursorKey = 'fabushi.desktop.account-sync-cursor.v1';
export const messengerConversationJournalKey = 'fabushi.desktop.mahayana-conversation-journal.v1';
export const miniAppExecutionPersistencePrefix = 'fabushi.desktop.miniapp-execution.v1:';
export const messengerPreferencesKey = 'fabushi.desktop.telegram-settings.v1';
export const initialMessageRenderCount = 240;
export const initialSyncLimit = 20;
export const backgroundSyncLimit = 100;
export const projectionConversationLimit = 80;
export const projectionMessageLimit = 80;

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

export function readMessengerProjection(): MessengerProjection | null {
  if (typeof window === 'undefined') return null;
  try {
    return asMessengerProjection(JSON.parse(window.localStorage.getItem(messengerProjectionKey) || 'null'));
  } catch {
    return null;
  }
}

async export function readDurableMessengerProjection(): Promise<MessengerProjection | null> {
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

export function readDesktopMessengerPreferences(): DesktopMessengerPreferences {
  if (typeof window === 'undefined') return defaultDesktopMessengerPreferences;
  try {
    const stored = JSON.parse(window.localStorage.getItem(messengerPreferencesKey) || '{}') as Partial<DesktopMessengerPreferences>;
    return { ...defaultDesktopMessengerPreferences, ...stored };
  } catch {
    return defaultDesktopMessengerPreferences;
  }
}

export function persistMessengerProjection(projection: MessengerProjection): void {
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

export function readAccountSyncCursor(): string | null {
  if (typeof window === 'undefined') return null;
  try {
    const value = window.localStorage.getItem(accountSyncCursorKey)?.trim();
    return value || null;
  } catch {
    return null;
  }
}

export function persistAccountSyncCursor(cursor: string | null): void {
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

async export function clearAccountScopedDesktopCaches(): Promise<void> {
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

export function createTransport(): MahayanaHostTransport {
  if (isElectronMahayanaHostAvailable()) return new ElectronMahayanaHostTransport();
  return new MockMahayanaHostTransport({ authenticated: true });
}

export function isDesktopUpdateState(value: unknown): value is UpdateState {
  if (!value || typeof value !== 'object') return false;
  const type = (value as { type?: unknown }).type;
  return typeof type === 'string' && ['loading', 'disabled', 'checking', 'available', 'downloading', 'staging', 'ready', 'upToDate', 'error'].includes(type);
}

type ActionableDesktopUpdateState = Extract<UpdateState, { type: 'available' | 'downloading' | 'staging' | 'ready' }>;

export function isActionableDesktopUpdateState(value: UpdateState | null): value is ActionableDesktopUpdateState {
  return Boolean(value && ['available', 'downloading', 'staging', 'ready'].includes(value.type));
}


export function formatTime(timestamp: number): string {
  if (!timestamp) return '';
  const date = new Date(timestamp);
  if (date.toDateString() === new Date().toDateString()) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
  return date.toLocaleDateString([], { month: 'numeric', day: 'numeric' });
}

export function sectionTitle(section: MessengerSection): string {
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

export function matchesSection(peer: PeerItem, section: MessengerSection): boolean {
  if (section === 'chats') return !peer.archived;
  if (section === 'contacts') return peer.kind === 'contact';
  if (section === 'bots') return peer.kind === 'bot';
  if (section === 'groups') return peer.kind === 'group';
  if (section === 'channels') return peer.kind === 'channel';
  if (section === 'saved') return peer.kind === 'saved';
  if (section === 'archive') return peer.archived;
  return false;
}

export function blobMediaUrl(media?: MessagingMediaRef): string | undefined {
  if (!media?.id) return undefined;
  return `fabushi-blob://${encodeURIComponent(media.id)}`;
}

export function defaultCommunityState(conversationId: string, actorId: string): MessagingCommunityState {
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

export function cachedLegacyDisplayMessages(conversationId: string): DisplayMessage[] {
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

export function startupLegacyConversationId(projection: MessengerProjection | null | undefined): string | null {
  const key = projection?.activePeerKey;
  if (!key) return null;
  if (key.startsWith('legacy:conversation:')) return key.slice('legacy:conversation:'.length) || null;
  if (key.startsWith('legacy:bot:')) {
    const botId = key.slice('legacy:bot:'.length);
    return projection?.legacyBots?.find((bot) => bot.id === botId)?.conversationId ?? null;
  }
  return null;
}

