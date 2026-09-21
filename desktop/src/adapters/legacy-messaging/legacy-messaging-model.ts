import type {
  AttachmentContext,
  BotSummary,
  ConversationSummary,
  GroupSummary,
  InferenceProvider,
  SandboxRuntime,
} from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MarketplacePluginSummary } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import type {
  MessagingActor,
  MessagingBotExecution,
  MessagingBotProfile,
  MessagingCommunityState,
  MessagingConversation,
  MessagingInvoice,
  MessagingLedgerEntry,
  MessagingMediaRef,
  MessagingMessage,
  MessagingOrder,
  MessagingStory,
  MessagingWalletAccount,
} from '../../selfhosted-messaging-client-v2';
import type { WebRtcCallStatus } from '../../webrtc-call-controller';
import type { MiniAppBotCallProgram } from '../../miniapp-bot-projection';
import type { AssistantTurn } from '../../mahayana-assistant-turn';
import type { AccountBotMembership } from '../../account-sync-client';
import type { CompatibilityPeerItem as PeerItem } from '../compatibility/messenger-compatibility-adapter';

export type DisplayMessage = {
  id: string;
  source: 'legacy' | 'selfhosted';
  role: 'me' | 'peer';
  text: string;
  createdAtMs: number;
  kind?: 'message' | 'assistant-turn' | 'action' | 'thinking';
  operationId?: string;
  streaming?: boolean;
  optimistic?: boolean;
  queued?: boolean;
  actionTitle?: string;
  actionDetail?: string;
  actionStatus?: 'running' | 'completed' | 'failed' | 'interrupted';
  assistantTurn?: AssistantTurn;
  attachments?: readonly AttachmentContext[];
  miniAppId?: string;
  pinned?: boolean;
  reactions?: string[];
  invoiceId?: string;
  media?: MessagingMediaRef;
  mediaType?: 'photo' | 'video' | 'document';
};

export type NewDialog =
  | { type: 'group'; name: string; selectedBotIds: Set<string> }
  | { type: 'channel'; name: string; description: string }
  | null;

export type LocalCall = {
  kind: 'voice' | 'video';
  title: string;
  status: WebRtcCallStatus;
  incoming: boolean;
  muted: boolean;
  videoEnabled: boolean;
  error?: string;
};

export type MiniAppCallSession = {
  callId: string;
  miniAppId: string;
  title: string;
  kind: 'voice' | 'video';
  program: MiniAppBotCallProgram;
  html?: string;
};

export type MessageMenu = { message: DisplayMessage; x: number; y: number } | null;
export type ForwardDialogState = { sourceConversationId: string; message: DisplayMessage } | null;
export type EditDialogState = { conversationId: string; messageId: string; originalText: string; text: string } | null;
export type InvoiceDialogState = { conversationId: string; title: string; amount: string } | null;
export type InfoTab = 'media' | 'files' | 'links';
export type SettingsCategory =
  | 'account'
  | 'router'
  | 'usage'
  | 'updates'
  | 'notifications'
  | 'privacy'
  | 'data'
  | 'chat'
  | 'folders'
  | 'devices'
  | 'calls'
  | 'language'
  | 'advanced'
  | 'fabushi';

export type InferenceRouterStatus = {
  schemaVersion: 1;
  providers: Array<{
    id: InferenceProvider;
    label: string;
    available: boolean;
    authenticated: boolean;
    installed?: boolean;
    source: string;
  }>;
  sandboxes: Array<{ id: SandboxRuntime; label: string; available: boolean; source: string }>;
};

export type ProviderUsageSummary = {
  provider: string;
  requests: number;
  inputTokens: number;
  cachedInputTokens: number;
  outputTokens: number;
  reasoningTokens: number;
  totalTokens: number;
  lifetimeTokens: number;
  lastUsedAtMs: number | null;
};

export type UsageSummary = {
  totalTokens: number;
  lifetimeTokens?: number;
  events: number;
  source: string;
  updatedAtMs?: number | null;
  byProvider?: ProviderUsageSummary[];
};

export type DesktopMessengerPreferences = {
  showInfoPanel: boolean;
  messagePreview: boolean;
  autoPlayMedia: boolean;
  enterToSend: boolean;
  reducedMotion: boolean;
};

export type MessengerProjection = {
  version: 1;
  savedAtMs: number;
  actorId?: string;
  cursor?: string | null;
  activePeerKey?: string | null;
  legacyConversations?: ConversationSummary[];
  legacyBots?: BotSummary[];
  legacyGroups?: GroupSummary[];
  accountBots?: AccountBotMembership[];
  miniAppIdentityCatalog?: MarketplacePluginSummary[];
  selfActors: MessagingActor[];
  selfConversations: MessagingConversation[];
  selfMessages: Record<string, MessagingMessage[]>;
};

export interface LegacyCompatibilityCollections {
  conversations: ConversationSummary[];
  groups: GroupSummary[];
  selfActors: MessagingActor[];
  selfConversations: MessagingConversation[];
  selfMessages: Record<string, MessagingMessage[]>;
  selfStories: MessagingStory[];
  selfCommunities: MessagingCommunityState[];
  selfBotProfiles: MessagingBotProfile[];
  selfBotExecutions: MessagingBotExecution[];
  selfInvoices: MessagingInvoice[];
  selfOrders: MessagingOrder[];
  walletAccount: MessagingWalletAccount | null;
  walletEntries: MessagingLedgerEntry[];
  accountBots: AccountBotMembership[];
  marketplaceApps: MarketplacePluginSummary[];
  miniAppIdentityCatalog: MarketplacePluginSummary[];
  peers?: PeerItem[];
}
