import type { ConversationSummary } from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';

/** Telegram stays a compatibility transport and never becomes an Agent identity. */
export function isTelegramCompatibilityConversation(conversation: ConversationSummary): boolean {
  const id = conversation.id.toLowerCase();
  const kind = conversation.kind.toLowerCase();
  return id.startsWith('telegram:') || kind.includes('telegram');
}
