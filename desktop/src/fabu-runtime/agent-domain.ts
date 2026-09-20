export interface FabuBotLike {
  id: string;
  agentId?: string | null;
  conversationId?: string;
  name?: string;
  displayName?: string;
  description?: string;
  title?: string;
  avatarShape?: string;
  avatarColor?: string;
  hidden?: boolean;
  notificationsEnabled?: boolean;
  notifyOnUpdates?: boolean;
}

export interface FabuBotIdentity {
  botId: string;
  agentId: string;
  conversationId?: string;
}

export interface FabuAgentProfile {
  name: string;
  description: string;
  title: string;
  avatarShape: string;
  avatarColor: string;
}

export interface FabuAgentSettings {
  notifyOnAgentUpdates: boolean;
  hiddenFromSidebar: boolean;
}

function clean(value: unknown): string {
  return typeof value === 'string' ? value.trim() : '';
}

/**
 * Fabu keeps Bot identity and Agent identity as separate domains.
 * The Bot owns presentation/conversation identity; agentId owns runtime state.
 */
export function resolveRuntimeAgentId(bot: Pick<FabuBotLike, 'id' | 'agentId'>): string {
  return clean(bot.agentId) || clean(bot.id);
}

export function projectFabuBotIdentity(bot: Pick<FabuBotLike, 'id' | 'agentId' | 'conversationId'>): FabuBotIdentity {
  const botId = clean(bot.id);
  if (!botId) throw new Error('Bot id is required.');
  const conversationId = clean(bot.conversationId);
  return {
    botId,
    agentId: resolveRuntimeAgentId(bot),
    ...(conversationId ? { conversationId } : {}),
  };
}

export function isBotActorKind(kind: unknown): boolean {
  const normalized = clean(kind).toLowerCase();
  return normalized === 'assistant' || normalized === 'bot' || normalized === 'service';
}

/**
 * Mirrors Fabu's profile.json shape. Runtime identity is intentionally not
 * embedded in this document; it is carried by the Agent directory/store path.
 */
export function projectFabuAgentProfile(bot: FabuBotLike): FabuAgentProfile {
  return {
    name: clean(bot.displayName) || clean(bot.name) || clean(bot.id),
    description: clean(bot.description),
    title: clean(bot.title),
    avatarShape: clean(bot.avatarShape),
    avatarColor: clean(bot.avatarColor),
  };
}

/** Mirrors Fabu's settings.json defaults and field names. */
export function projectFabuAgentSettings(bot: FabuBotLike): FabuAgentSettings {
  return {
    notifyOnAgentUpdates: typeof bot.notifyOnUpdates === 'boolean'
      ? bot.notifyOnUpdates
      : typeof bot.notificationsEnabled === 'boolean'
        ? bot.notificationsEnabled
        : true,
    hiddenFromSidebar: bot.hidden === true,
  };
}
