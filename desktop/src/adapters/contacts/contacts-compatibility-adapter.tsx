import React from 'react';
import { MessageCircle, Users } from 'lucide-react';
import type { ConversationSummary } from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MessagingActor, MessagingConversation } from '../../selfhosted-messaging-client-v2';
import { FabSurface } from '../../ui/primitives/fab-primitives';
import styles from '../../messaging-shell.module.css';

export type ContactCompatibilityKind = 'contact' | 'bot';

export function contactKindForLegacyConversation(conversation: ConversationSummary): ContactCompatibilityKind | null {
  const id = conversation.id.toLowerCase();
  const kind = conversation.kind.toLowerCase();
  if (id.startsWith('telegram:')) return null;
  if (id.startsWith('mahayana:contact:') || kind.includes('direct') || kind.includes('contact')) return 'contact';
  if (id.startsWith('mahayana-ai:') || id.startsWith('codex:') || kind.includes('agent') || kind.includes('bot') || kind.includes('assistant')) return 'bot';
  return null;
}

export function contactKindForSelfHostedConversation(
  conversation: MessagingConversation,
  counterparty?: MessagingActor,
): ContactCompatibilityKind | null {
  if (conversation.kind !== 'direct' && conversation.kind !== 'secret') return null;
  return counterparty && ['assistant', 'bot', 'service'].includes(counterparty.kind) ? 'bot' : 'contact';
}

export function ContactsCompatibilityWorkspace() {
  return <FabSurface className={styles.featureWorkspace} elevated data-compatibility-feature="contacts">
    <Users size={54} /><h2>Contacts</h2>
    <p>联系人身份和 direct conversation 投影由 Contacts adapter 管理；Agent 主壳只消费投影。</p>
    <MessageCircle size={24} />
  </FabSurface>;
}
