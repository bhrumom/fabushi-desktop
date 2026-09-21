import { type ReactNode } from 'react';
import type {
  BotSummary,
  ConversationSummary,
  GroupSummary,
} from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { InstalledPluginPointer, MarketplacePluginSummary } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import type { AccountBotMembership } from '../../account-sync-client';
import { installedMiniAppBotProjections, type MiniAppBotCallPrograms, type MiniAppBotCommand } from '../../miniapp-bot-projection';
import { asMessagingHostEvent, type MessagingActor, type MessagingConversation } from '../../selfhosted-messaging-client-v2';
import type { RuntimeEvent } from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';

export type CompatibilityPeerKind = 'conversation' | 'contact' | 'bot' | 'group' | 'channel' | 'saved';
export type CompatibilityPeerSource = 'legacy' | 'selfhosted';

export interface CompatibilityPeerItem {
  key: string;
  id: string;
  source: CompatibilityPeerSource;
  conversationId?: string;
  actorId?: string;
  /** Explicit runtime Agent identity. Never infer Agent-ness from a title/id string. */
  agentId?: string;
  groupId?: string;
  kind: CompatibilityPeerKind;
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
}

function legacyKind(conversation: ConversationSummary): CompatibilityPeerKind {
  const id = conversation.id.toLowerCase();
  const kind = conversation.kind.toLowerCase();
  if (id.startsWith('mahayana:contact:') || id.startsWith('telegram:user:') || kind.includes('direct') || kind.includes('contact')) return 'contact';
  if (id.startsWith('mahayana-ai:') || id.startsWith('codex:') || kind.includes('agent') || kind.includes('bot') || kind.includes('assistant')) return 'bot';
  if (kind.includes('saved')) return 'saved';
  if (kind.includes('channel')) return 'channel';
  return 'conversation';
}

function selfKind(conversation: MessagingConversation, counterparty?: MessagingActor): CompatibilityPeerKind {
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

export interface CompatibilityPeerProjectionInput {
  conversations: readonly ConversationSummary[];
  bots: readonly BotSummary[];
  accountBots: readonly AccountBotMembership[];
  groups: readonly GroupSummary[];
  selfActors: readonly MessagingActor[];
  selfConversations: readonly MessagingConversation[];
  pinnedPeerKeys: ReadonlySet<string>;
  archivedPeerKeys: ReadonlySet<string>;
  miniAppIdentityCatalog: readonly MarketplacePluginSummary[];
  installedMiniApps: Readonly<Record<string, InstalledPluginPointer>>;
  selfActorId?: string | null;
}

/**
 * Compatibility-only peer projection for Contacts/Telegram/Mini Apps/legacy groups.
 * AgentRootShell consumes the result, but none of these legacy domains are allowed
 * to own the primary product shell or Agent runtime state.
 */
export function buildCompatibilityPeers(input: CompatibilityPeerProjectionInput): CompatibilityPeerItem[] {
  const {
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
    selfActorId,
  } = input;

  const botByConversationId = new Map(
    bots
      .filter((bot) => Boolean(bot.conversationId))
      .map((bot) => [bot.conversationId!, bot] as const),
  );
  const legacyConversations = conversations.map((conversation): CompatibilityPeerItem => {
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
    .map((bot): CompatibilityPeerItem => ({
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
    .map((entry): CompatibilityPeerItem => {
      const miniAppSource = entry.sources.find((source) => source.source === 'miniapp');
      const projection = miniAppSource
        ? miniAppBotProjections.find((candidate) => candidate.miniAppId === miniAppSource.sourceId)
        : miniAppByBotId.get(entry.bot.id);
      const key = miniAppSource ? `miniapp:bot:${miniAppSource.sourceId}` : `account:bot:${entry.bot.id}`;
      return {
        key,
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
        pinned: pinnedPeerKeys.has(key),
        archived: archivedPeerKeys.has(key),
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
    .map((projection): CompatibilityPeerItem => ({
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
  const legacyGroups = groups.map((group): CompatibilityPeerItem => ({
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
  const nativePeers = selfConversations.map((conversation): CompatibilityPeerItem => {
    const actorId = conversation.participants.length === 2
      ? conversation.participants.find((participant) => participant.actorId !== selfActorId)?.actorId
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
  return [...legacyConversations, ...botPeers, ...accountBotPeers, ...miniAppBotPeers, ...legacyGroups, ...nativePeers]
    .sort((left, right) => left.pinned === right.pinned ? right.updatedAtMs - left.updatedAtMs : left.pinned ? -1 : 1);
}


export type CompatibilitySection = 'chats' | 'contacts' | 'bots' | 'groups' | 'channels' | 'calls' | 'saved' | 'archive' | 'folders' | 'miniapps' | 'payments' | 'settings';

export function compatibilityMessagingEnvelope(event: RuntimeEvent) {
  return asMessagingHostEvent(event)?.envelope ?? null;
}

export function CompatibilitySurface(props: {
  section: CompatibilitySection;
  miniApps: ReactNode;
  payments: ReactNode;
  calls: ReactNode;
  settings: ReactNode;
  fallback: ReactNode;
}) {
  if (props.section === 'miniapps') return <>{props.miniApps}</>;
  if (props.section === 'payments') return <>{props.payments}</>;
  if (props.section === 'calls') return <>{props.calls}</>;
  if (props.section === 'settings') return <>{props.settings}</>;
  return <>{props.fallback}</>;
}
