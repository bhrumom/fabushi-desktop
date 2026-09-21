import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { MahayanaHostTransport } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import {
  SelfHostedMessagingClientV2,
  asMessagingHostEvent,
  type MessagingActor,
  type MessagingConversation,
  type MessagingInvoice,
  type MessagingLedgerEntry,
  type MessagingMessage,
  type MessagingOrder,
  type MessagingWalletAccount,
} from '../../selfhosted-messaging-client-v2';

function upsertById<T extends { id: string }>(items: readonly T[], item: T): T[] {
  const index = items.findIndex((candidate) => candidate.id === item.id);
  if (index < 0) return [...items, item];
  const next = [...items];
  next[index] = item;
  return next;
}

export interface CompatibilityMessagingSnapshot {
  readonly client: SelfHostedMessagingClientV2;
  readonly actors: readonly MessagingActor[];
  readonly conversations: readonly MessagingConversation[];
  readonly messagesByConversation: Readonly<Record<string, readonly MessagingMessage[]>>;
  readonly invoices: readonly MessagingInvoice[];
  readonly orders: readonly MessagingOrder[];
  readonly wallet: MessagingWalletAccount | null;
  readonly ledger: readonly MessagingLedgerEntry[];
  readonly loading: boolean;
  readonly error: string | null;
  refresh(): Promise<void>;
}

/**
 * Compatibility-only messaging projection.
 *
 * This hook intentionally owns no Agent roster, run, turn, draft, permission,
 * computer or ConversationActor state. It performs one bounded sync on mount
 * and then converges from pushed Mahayana messaging events.
 */
export function useCompatibilityMessagingRuntime(
  transport: MahayanaHostTransport,
  options: { readonly includeWallet?: boolean } = {},
): CompatibilityMessagingSnapshot {
  const client = useMemo(() => new SelfHostedMessagingClientV2(transport), [transport]);
  const [actors, setActors] = useState<MessagingActor[]>([]);
  const [conversations, setConversations] = useState<MessagingConversation[]>([]);
  const [messagesByConversation, setMessagesByConversation] = useState<Record<string, MessagingMessage[]>>({});
  const [invoices, setInvoices] = useState<MessagingInvoice[]>([]);
  const [orders, setOrders] = useState<MessagingOrder[]>([]);
  const [wallet, setWallet] = useState<MessagingWalletAccount | null>(null);
  const [ledger, setLedger] = useState<MessagingLedgerEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const cursorRef = useRef<string | null>(null);
  const includeWalletRef = useRef(Boolean(options.includeWallet));
  includeWalletRef.current = Boolean(options.includeWallet);

  const refresh = useCallback(async () => {
    setError(null);
    setLoading(true);
    try {
      await client.ensureCurrentActor();
      await client.sync(100, cursorRef.current);
      if (includeWalletRef.current) await client.requestWalletStatus();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLoading(false);
    }
  }, [client]);

  useEffect(() => {
    const unsubscribe = transport.subscribe((runtimeEvent) => {
      const envelope = asMessagingHostEvent(runtimeEvent)?.envelope;
      if (!envelope) return;
      const previousCursor = cursorRef.current;
      if (envelope.cursor) cursorRef.current = envelope.cursor;
      const event = envelope.event;
      switch (event.type) {
        case 'syncBatch': {
          const payload = event as unknown as {
            actors?: MessagingActor[];
            conversations?: MessagingConversation[];
            messages?: MessagingMessage[];
            invoices?: MessagingInvoice[];
            orders?: MessagingOrder[];
          };
          setActors((current) => (payload.actors ?? []).reduce(upsertById, previousCursor ? current : []));
          setConversations((current) => (payload.conversations ?? []).reduce(upsertById, previousCursor ? current : []));
          setInvoices((current) => (payload.invoices ?? []).reduce(upsertById, previousCursor ? current : []));
          setOrders((current) => (payload.orders ?? []).reduce(upsertById, previousCursor ? current : []));
          setMessagesByConversation((current) => {
            const next: Record<string, MessagingMessage[]> = previousCursor ? { ...current } : {};
            for (const message of payload.messages ?? []) {
              next[message.conversationId] = upsertById(next[message.conversationId] ?? [], message)
                .sort((left, right) => left.createdAtMs - right.createdAtMs);
            }
            return next;
          });
          break;
        }
        case 'actorChanged': {
          const actor = (event as unknown as { actor: MessagingActor }).actor;
          setActors((current) => upsertById(current, actor));
          break;
        }
        case 'conversationChanged': {
          const conversation = (event as unknown as { conversation: MessagingConversation }).conversation;
          setConversations((current) => upsertById(current, conversation));
          break;
        }
        case 'messageAdded':
        case 'messageChanged': {
          const message = (event as unknown as { message: MessagingMessage }).message;
          setMessagesByConversation((current) => ({
            ...current,
            [message.conversationId]: upsertById(current[message.conversationId] ?? [], message)
              .sort((left, right) => left.createdAtMs - right.createdAtMs),
          }));
          if (event.type === 'messageAdded') {
            setConversations((current) => current.map((conversation) => conversation.id === message.conversationId
              ? {
                  ...conversation,
                  lastMessageId: message.id,
                  updatedAtMs: Math.max(conversation.updatedAtMs, message.createdAtMs),
                }
              : conversation));
          }
          break;
        }
        case 'messagesDeleted': {
          const payload = event as unknown as { conversationId: string; messageIds: string[] };
          setMessagesByConversation((current) => ({
            ...current,
            [payload.conversationId]: (current[payload.conversationId] ?? [])
              .filter((message) => !payload.messageIds.includes(message.id)),
          }));
          break;
        }
        case 'invoiceChanged': {
          const invoice = (event as unknown as { invoice: MessagingInvoice }).invoice;
          setInvoices((current) => upsertById(current, invoice));
          break;
        }
        case 'orderChanged': {
          const order = (event as unknown as { order: MessagingOrder }).order;
          setOrders((current) => upsertById(current, order));
          if (includeWalletRef.current) void client.requestWalletStatus().catch(() => {});
          break;
        }
        case 'walletStatus': {
          const payload = event as unknown as {
            account?: MessagingWalletAccount | null;
            recentEntries?: MessagingLedgerEntry[];
          };
          setWallet(payload.account ?? null);
          setLedger(payload.recentEntries ?? []);
          break;
        }
        default:
          break;
      }
    });
    void refresh();
    return unsubscribe;
  }, [client, refresh, transport]);

  return {
    client,
    actors,
    conversations,
    messagesByConversation,
    invoices,
    orders,
    wallet,
    ledger,
    loading,
    error,
    refresh,
  };
}
