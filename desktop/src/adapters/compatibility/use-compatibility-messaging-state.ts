import { useRef, useState } from 'react';
import type { InstalledPluginPointer, MarketplacePluginSummary } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import type { IncomingFabushiCall } from '../../webrtc-call-controller';
import type {
  MessagingBotExecution,
  MessagingBotProfile,
  MessagingCommunityState,
  MessagingInvoice,
  MessagingLedgerEntry,
  MessagingOrder,
  MessagingStory,
  MessagingWalletAccount,
} from '../../selfhosted-messaging-client-v2';
import type { CompatibilityPeerItem as PeerItem } from './messenger-compatibility-adapter';
import type {
  DisplayMessage,
  EditDialogState,
  ForwardDialogState,
  InvoiceDialogState,
  LocalCall,
  MessageMenu,
  MessengerProjection,
  MiniAppCallSession,
  NewDialog,
} from './compatibility-model';

export function useCompatibilityMessagingState(
  startupProjection: MessengerProjection | null | undefined,
  initialMessages: readonly DisplayMessage[],
) {
  const [conversations, setConversations] = useState(() => startupProjection?.legacyConversations ?? []);
  const [groups, setGroups] = useState(() => startupProjection?.legacyGroups ?? []);
  const [selfActors, setSelfActors] = useState(() => startupProjection?.selfActors ?? []);
  const [selfConversations, setSelfConversations] = useState(() => startupProjection?.selfConversations ?? []);
  const [selfMessages, setSelfMessages] = useState(() => startupProjection?.selfMessages ?? {});
  const [selfStories, setSelfStories] = useState<MessagingStory[]>([]);
  const [selfCommunities, setSelfCommunities] = useState<MessagingCommunityState[]>([]);
  const [selfBotProfiles, setSelfBotProfiles] = useState<MessagingBotProfile[]>([]);
  const [selfBotExecutions, setSelfBotExecutions] = useState<MessagingBotExecution[]>([]);
  const [activeStory, setActiveStory] = useState<MessagingStory | null>(null);
  const [communityDialogPeer, setCommunityDialogPeer] = useState<PeerItem | null>(null);
  const [selfInvoices, setSelfInvoices] = useState<MessagingInvoice[]>([]);
  const [selfOrders, setSelfOrders] = useState<MessagingOrder[]>([]);
  const [walletAccount, setWalletAccount] = useState<MessagingWalletAccount | null>(null);
  const [walletEntries, setWalletEntries] = useState<MessagingLedgerEntry[]>([]);
  const [messages, setMessages] = useState<DisplayMessage[]>(() => [...initialMessages]);
  const [composer, setComposer] = useState('');
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [legacyReplyTo, setLegacyReplyTo] = useState<DisplayMessage | null>(null);
  const [silentSend, setSilentSend] = useState(false);
  const [scheduledAtMs, setScheduledAtMs] = useState<number | undefined>();
  const [typingByConversation, setTypingByConversation] = useState<Record<string, Record<string, number>>>({});
  const typingExpiryTimersRef = useRef<Map<string, number>>(new Map());
  const [newDialog, setNewDialog] = useState<NewDialog>(null);
  const [messageMenu, setMessageMenu] = useState<MessageMenu>(null);
  const [forwardDialog, setForwardDialog] = useState<ForwardDialogState>(null);
  const [editDialog, setEditDialog] = useState<EditDialogState>(null);
  const [invoiceDialog, setInvoiceDialog] = useState<InvoiceDialogState>(null);
  const [attachmentMenuOpen, setAttachmentMenuOpen] = useState(false);
  const [attachmentProgress, setAttachmentProgress] = useState<string | null>(null);
  const [localCall, setLocalCall] = useState<LocalCall | null>(null);
  const [incomingCall, setIncomingCall] = useState<IncomingFabushiCall | null>(null);
  const [miniApp, setMiniApp] = useState<{ id: string; title: string; url: string } | null>(null);
  const [miniAppCall, setMiniAppCall] = useState<MiniAppCallSession | null>(null);
  const miniAppBotThreadsRef = useRef<Record<string, DisplayMessage[]>>({});
  const [accountBots, setAccountBots] = useState(() => startupProjection?.accountBots ?? []);
  const [marketplaceApps, setMarketplaceApps] = useState<MarketplacePluginSummary[]>([]);
  const [miniAppIdentityCatalog, setMiniAppIdentityCatalog] = useState<MarketplacePluginSummary[]>(
    () => startupProjection?.miniAppIdentityCatalog ?? [],
  );
  const [installedMiniApps, setInstalledMiniApps] = useState<Record<string, InstalledPluginPointer>>({});
  const [miniAppQuery, setMiniAppQuery] = useState('');
  const [miniAppLoading, setMiniAppLoading] = useState(false);
  const [miniAppBusy, setMiniAppBusy] = useState<Set<string>>(() => new Set());

  return {
    conversations, setConversations,
    groups, setGroups,
    selfActors, setSelfActors,
    selfConversations, setSelfConversations,
    selfMessages, setSelfMessages,
    selfStories, setSelfStories,
    selfCommunities, setSelfCommunities,
    selfBotProfiles, setSelfBotProfiles,
    selfBotExecutions, setSelfBotExecutions,
    activeStory, setActiveStory,
    communityDialogPeer, setCommunityDialogPeer,
    selfInvoices, setSelfInvoices,
    selfOrders, setSelfOrders,
    walletAccount, setWalletAccount,
    walletEntries, setWalletEntries,
    messages, setMessages,
    composer, setComposer,
    drafts, setDrafts,
    legacyReplyTo, setLegacyReplyTo,
    silentSend, setSilentSend,
    scheduledAtMs, setScheduledAtMs,
    typingByConversation, setTypingByConversation,
    typingExpiryTimersRef,
    newDialog, setNewDialog,
    messageMenu, setMessageMenu,
    forwardDialog, setForwardDialog,
    editDialog, setEditDialog,
    invoiceDialog, setInvoiceDialog,
    attachmentMenuOpen, setAttachmentMenuOpen,
    attachmentProgress, setAttachmentProgress,
    localCall, setLocalCall,
    incomingCall, setIncomingCall,
    miniApp, setMiniApp,
    miniAppCall, setMiniAppCall,
    miniAppBotThreadsRef,
    accountBots, setAccountBots,
    marketplaceApps, setMarketplaceApps,
    miniAppIdentityCatalog, setMiniAppIdentityCatalog,
    installedMiniApps, setInstalledMiniApps,
    miniAppQuery, setMiniAppQuery,
    miniAppLoading, setMiniAppLoading,
    miniAppBusy, setMiniAppBusy,
  };
}
