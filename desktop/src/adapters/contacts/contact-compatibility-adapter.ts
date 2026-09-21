import type { ConversationSummary } from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MessagingActor, MessagingConversation } from '../../selfhosted-messaging-client-v2';

/** Compatibility classification for human/contact conversations only. */
export function isLegacyContactConversation(conversation: ConversationSummary): boolean {
  const id = conversation.id.toLowerCase();
  const kind = conversation.kind.toLowerCase();
  return id.startsWith('mahayana:contact:')
    || kind.includes('direct')
    || kind.includes('contact');
}

export function isSelfHostedContactConversation(
  conversation: MessagingConversation,
  counterparty?: MessagingActor,
): boolean {
  return (conversation.kind === 'direct' || conversation.kind === 'secret')
    && !counterparty
      ? true
      : (conversation.kind === 'direct' || conversation.kind === 'secret')
        && !['assistant', 'bot', 'service'].includes(counterparty?.kind ?? '');
}
