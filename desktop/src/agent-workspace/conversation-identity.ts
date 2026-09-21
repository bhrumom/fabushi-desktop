export interface AgentIdentity {
  readonly type: 'agent';
  readonly agentId: string;
  readonly conversationId?: string;
  readonly groupId?: string;
}

export interface ContactIdentity {
  readonly type: 'contact';
  readonly contactId: string;
  readonly conversationId?: string;
}

export interface TelegramIdentity {
  readonly type: 'telegram';
  readonly telegramId: string;
  readonly conversationId?: string;
}

export interface MiniAppBotIdentity {
  readonly type: 'miniapp-bot';
  readonly botId: string;
  readonly miniAppId: string;
  readonly conversationId?: string;
}

export type ConversationIdentity =
  | AgentIdentity
  | ContactIdentity
  | TelegramIdentity
  | MiniAppBotIdentity;

export interface ConversationIdentityProjection {
  readonly id: string;
  readonly kind: string;
  readonly agentId?: string;
  readonly actorId?: string;
  readonly conversationId?: string;
  readonly groupId?: string;
  readonly miniAppId?: string;
}

/**
 * Legacy peer parsing is isolated here. Product Agent components consume the
 * discriminated union and never inspect source prefixes to decide identity.
 */
export function conversationIdentityOf(peer: ConversationIdentityProjection): ConversationIdentity {
  if (peer.miniAppId) {
    return {
      type: 'miniapp-bot',
      botId: peer.actorId ?? peer.id,
      miniAppId: peer.miniAppId,
      ...(peer.conversationId ? { conversationId: peer.conversationId } : {}),
    };
  }
  if (peer.kind === 'bot' || peer.kind === 'group') {
    return {
      type: 'agent',
      agentId: peer.agentId ?? peer.actorId ?? peer.id,
      ...(peer.conversationId ? { conversationId: peer.conversationId } : {}),
      ...(peer.groupId ? { groupId: peer.groupId } : {}),
    };
  }
  if (peer.id.startsWith('telegram:') || peer.conversationId?.startsWith('telegram:')) {
    return {
      type: 'telegram',
      telegramId: peer.actorId ?? peer.id,
      ...(peer.conversationId ? { conversationId: peer.conversationId } : {}),
    };
  }
  return {
    type: 'contact',
    contactId: peer.actorId ?? peer.id,
    ...(peer.conversationId ? { conversationId: peer.conversationId } : {}),
  };
}

export function isAgentIdentity(identity: ConversationIdentity): identity is AgentIdentity {
  return identity.type === 'agent';
}
