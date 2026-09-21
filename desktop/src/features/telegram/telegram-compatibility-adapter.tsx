import React, { useMemo, useState } from 'react';
import type { MahayanaHostTransport } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import { messagingText } from '../../selfhosted-messaging-client-v2';
import { FabButton, FabInput, FabSurface } from '../../ui/primitives/fab-primitives';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';
import { useCompatibilityMessagingRuntime } from '../compatibility/use-compatibility-messaging-runtime';
import styles from '../compatibility/compatibility-data.module.css';

export default function TelegramCompatibilityAdapter({
  transport,
  onClose,
}: {
  readonly transport: MahayanaHostTransport;
  readonly onClose: () => void;
}) {
  const runtime = useCompatibilityMessagingRuntime(transport);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [draft, setDraft] = useState('');
  const [sending, setSending] = useState(false);
  const conversations = useMemo(
    () => [...runtime.conversations]
      .filter((item) => item.kind !== 'savedMessages')
      .sort((left, right) => Number(right.pinnedMessageIds.length > 0) - Number(left.pinnedMessageIds.length > 0)
        || right.updatedAtMs - left.updatedAtMs),
    [runtime.conversations],
  );
  const selected = conversations.find((item) => item.id === selectedId) ?? conversations[0] ?? null;
  const messages = selected ? (runtime.messagesByConversation[selected.id] ?? []).filter((item) => !item.deleted) : [];

  async function send() {
    if (!selected || !draft.trim() || sending || !selected.permissions.canSendMessages) return;
    setSending(true);
    try {
      await runtime.client.sendText(selected.id, draft.trim());
      setDraft('');
    } finally {
      setSending(false);
    }
  }

  return <CompatibilityFeatureFrame
    title="Telegram / Messaging"
    description="Direct, group and channel messaging runs in the compatibility domain; it cannot own Agent runs or drafts."
    onClose={onClose}
  >
    <div className={styles.split}>
      <div className={styles.pane}>
        <div className={styles.toolbar}>
          <span className={styles.status}>{conversations.length} conversations</span>
          <FabButton type="button" variant="ghost" onClick={() => void runtime.refresh()}>Refresh</FabButton>
        </div>
        <div className={styles.list}>
          {conversations.map((conversation) => <FabButton
            type="button"
            variant={conversation.id === selected?.id ? 'primary' : 'ghost'}
            key={conversation.id}
            onClick={() => setSelectedId(conversation.id)}
          >
            {conversation.title} {conversation.unreadCount ? `· ${conversation.unreadCount}` : ''}
          </FabButton>)}
        </div>
      </div>
      <div className={styles.messages}>
        <div className={styles.messageList}>
          {selected ? messages.map((message) => <div
            key={message.id}
            className={styles.message}
            data-own={message.senderId === runtime.client.actorId || undefined}
          >
            {messagingText(message)}
          </div>) : <div className={styles.status}>Select a conversation.</div>}
        </div>
        {selected ? <div className={styles.composer}>
          <FabInput
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            placeholder={selected.permissions.canSendMessages ? `Message ${selected.title}` : 'Read only'}
            disabled={!selected.permissions.canSendMessages || sending}
            onKeyDown={(event) => {
              if (event.key === 'Enter' && !event.shiftKey) {
                event.preventDefault();
                void send();
              }
            }}
          />
          <FabButton type="button" variant="primary" disabled={!draft.trim() || sending || !selected.permissions.canSendMessages} onClick={() => void send()}>
            {sending ? 'Sending…' : 'Send'}
          </FabButton>
        </div> : null}
      </div>
    </div>
    {runtime.error ? <div className={styles.error} role="alert">{runtime.error}</div> : null}
  </CompatibilityFeatureFrame>;
}
