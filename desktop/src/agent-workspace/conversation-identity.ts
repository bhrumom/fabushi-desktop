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

export type ConversationIdentityKind =
  | 'agent'
  | 'bot'
  | 'group'
  | 'contact'
  | 'telegram'
  | 'miniapp-bot';

export type AgentProjectionKind = Extract<ConversationIdentityKind, 'agent' | 'bot' | 'group'>;

export interface ConversationIdentityProjection {
  readonly id: string;
  readonly kind: ConversationIdentityKind;
  readonly agentId?: string;
  readonly actorId?: string;
  readonly conversationId?: string;
  readonly groupId?: string;
  readonly miniAppId?: string;
}

/**
 * Converts an explicitly typed projection into the domain identity consumed by
 * product surfaces. Adapters must decide the domain kind; this function never
 * guesses identity from IDs, prefixes, or optional fields.
 */
export function conversationIdentityOf(peer: ConversationIdentityProjection): ConversationIdentity {
  switch (peer.kind) {
    case 'agent':
    case 'bot':
    case 'group':
      return {
        type: 'agent',
        agentId: peer.agentId ?? peer.actorId ?? peer.id,
        ...(peer.conversationId ? { conversationId: peer.conversationId } : {}),
        ...(peer.groupId ? { groupId: peer.groupId } : {}),
      };
    case 'telegram':
      return {
        type: 'telegram',
        telegramId: peer.actorId ?? peer.id,
        ...(peer.conversationId ? { conversationId: peer.conversationId } : {}),
      };
    case 'contact':
      return {
        type: 'contact',
        contactId: peer.actorId ?? peer.id,
        ...(peer.conversationId ? { conversationId: peer.conversationId } : {}),
      };
    case 'miniapp-bot':
      if (!peer.miniAppId) {
        throw new Error('Mini App Bot identity requires an explicit miniAppId');
      }
      return {
        type: 'miniapp-bot',
        botId: peer.actorId ?? peer.id,
        miniAppId: peer.miniAppId,
        ...(peer.conversationId ? { conversationId: peer.conversationId } : {}),
      };
  }
}

export function isAgentIdentityexport function isAgentIdentity(identity: ConversationIdentity): identity is AgentIdentity {
  return identity.type === 'agent';
}
