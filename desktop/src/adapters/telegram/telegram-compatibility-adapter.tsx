import React from 'react';
import { MessageCircle } from 'lucide-react';
import type { ConversationSummary } from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { FabSurface } from '../../ui/primitives/fab-primitives';
import styles from '../../messaging-shell.module.css';

export type TelegramCompatibilityKind = 'contact' | 'channel' | 'conversation';

export function telegramKindForLegacyConversation(conversation: ConversationSummary): TelegramCompatibilityKind | null {
  const id = conversation.id.toLowerCase();
  const kind = conversation.kind.toLowerCase();
  if (!id.startsWith('telegram:') && !kind.includes('telegram')) return null;
  if (id.startsWith('telegram:user:') || kind.includes('direct') || kind.includes('contact')) return 'contact';
  if (id.startsWith('telegram:channel:') || kind.includes('channel')) return 'channel';
  return 'conversation';
}

export function TelegramCompatibilityWorkspace() {
  return <FabSurface className={styles.featureWorkspace} elevated data-compatibility-feature="telegram">
    <MessageCircle size={54} /><h2>Telegram</h2>
    <p>Telegram identity and transport compatibility stay behind this adapter and cannot own Agent runtime state.</p>
  </FabSurface>;
}
