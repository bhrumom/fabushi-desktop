import {
  AppWindow,
  Archive,
  BellOff,
  Bot,
  Bookmark,
  Check,
  CloudDownload,
  Copy,
  Edit3,
  FileText,
  Forward,
  Image,
  Link2,
  MapPin,
  MessageCircle,
  Mic,
  Monitor,
  MoreVertical,
  Paperclip,
  Phone,
  PhoneCall,
  Pin,
  Plus,
  Radio,
  Reply,
  Search,
  Send,
  Settings,
  ShoppingBag,
  Smile,
  SquarePen,
  Trash2,
  UserPlus,
  Users,
  Video,
  WalletCards,
  X,
} from 'lucide-react';
import React, { useCallback, useEffect, useMemo, useRef, useState, type FormEvent } from 'react';
import HostClient from '../../../frontend/apps/web/src/app/host/host-client';
import FabAvatar, { type FabAvatarInputState } from '../ui/avatar/fab-avatar';
import type {
  AttachmentContext,
  AuthState,
  BotSummary,
  ConversationSummary,
  GroupSummary,
  InferenceProvider,
  ProductHostSettings,
  RuntimeEvent,
  SandboxRuntime,
  UpdateState,
} from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  ElectronMahayanaHostTransport,
  isElectronMahayanaHostAvailable,
  MAHAYANA_ACCOUNT_SESSION_RESET_EVENT,
  MAHAYANA_COMMAND_EVENT_NAME,
  readCachedConversationMessages,
  type MahayanaCommandBridgeDetail,
} from '../../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import { MockMahayanaHostTransport } from '../../../frontend/apps/web/src/lib/mahayana-host/mock-transport';
import { invokeNativeDesktop, subscribeNativeDesktopEvents } from '../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import type { InstalledPluginPointer, MahayanaHostTransport, MarketplacePluginSummary } from '../../../frontend/apps/web/src/lib/mahayana-host/transport';
import {
  SelfHostedMessagingClientV2,
  messagingText,
  type MessagingActor,
  type MessagingBotExecution,
  type MessagingBotProfile,
  type MessagingCommunityMember,
  type MessagingCommunityState,
  type MessagingConversation,
  type MessagingForumTopic,
  type MessagingInvoice,
  type MessagingLedgerEntry,
  type MessagingMediaRef,
  type MessagingMessage,
  type MessagingOrder,
  type MessagingStory,
  type MessagingWalletAccount,
} from '../selfhosted-messaging-client-v2';
import styles from '../messaging-shell.module.css';
import extra from '../adapters/legacy-messaging/legacy-messaging-shell.module.css';
import {
  FabushiWebRtcController,
  type IncomingFabushiCall,
  type WebRtcCallStatus,
} from '../webrtc-call-controller';
import { isTerminalAuthSessionFailure } from '../auth-session';
import {
  miniAppBotResponseText,
  type MiniAppBotCallProgram,
} from '../miniapp-bot-projection';
import { MiniAppCallDialog } from '../miniapp-call-dialog';
import { executeDesktopMiniAppBotInput, prepareDesktopMiniAppWebMcpDocument } from '../miniapp-webmcp-host';
import BotConversationView from '../bot-conversation-view';
import type { BotTranscriptMessage } from '../bot-conversation-view';
import AgentWorkspace from './agent-workspace';
import { CompatibilitySurface, buildCompatibilityPeers, compatibilityMessagingEnvelope, type CompatibilityPeerItem as PeerItem, type CompatibilityPeerKind as PeerKind, type CompatibilityPeerSource as PeerSource, type CompatibilitySection as MessengerSection } from './messenger-compatibility-adapter';
import type {
  DesktopMessengerPreferences,
  DisplayMessage,
  EditDialogState,
  ForwardDialogState,
  InfoTab,
  InferenceRouterStatus,
  InvoiceDialogState,
  LocalCall,
  MessageMenu,
  MessengerProjection,
  MiniAppCallSession,
  NewDialog,
  SettingsCategory,
  UsageSummary,
} from '../adapters/legacy-messaging/legacy-messaging-model';
import { useLegacyMessagingCompatibilityState } from '../adapters/legacy-messaging/use-legacy-messaging-compatibility-state';
import AgentOverlays from './agent-overlays';
import { AgentCoordinatorClient } from './coordinator-client';
import type { AgentTranscriptSourceMessage } from './agent-transcript-store';
import { useAgentWorkspaceRuntime } from './use-agent-workspace-runtime';
import { useAgentComputerController } from './use-agent-computer-controller';
import { projectTranscriptEntries, type TranscriptEntry } from './transcript-model';
import {
  AGENT_ATTACHMENT_LIMIT,
  agentFileToBase64,
  enrichAgentAttachmentPreview,
  validateAgentAttachment,
} from './agent-attachments';
import type { AgentPromptReference, AgentReplyContext } from './prompt-context';
import type { AgentSidebarSection } from './agent-sidebar-state';
import { useAgentSettingsController } from './use-agent-settings-controller';
import { useAgentProductControllers } from './use-agent-product-controllers';
import { useAgentShellViewState } from './use-agent-shell-view-state';
import {
  accountMiniAppsAsMarketplaceSummaries,
  appendMiniAppBotMessages,
  deleteAccountAgentStoreObject,
  listAccountAgentStore,
  readAccountAgentStoreObject,
  readAccountBots,
  readAccountMiniApps,
  readAccountSync,
  readMiniAppBotMessages,
  reconcileAccountMiniApps,
  upsertAccountAgent,
  writeAccountAgentStoreObject,
  type AccountBotMembership,
} from '../account-sync-client';
import { projectFabuAgentProfile, projectFabuAgentSettings, projectFabuBotIdentity } from '../fabu-runtime/agent-domain';
import {
  FABU_AGENT_ATTACHMENT_INDEX_PATH,
  FABU_AGENT_RUNTIME_CHECKPOINT_PATH,
  FABU_AGENT_WORKFLOW_INDEX_PATH,
  FabuAgentStore,
  fabuAgentConversationTranscriptPath,
} from '../fabu-runtime/agent-store';
import { restoreAgentStoreWorkspace } from './agent-store-recovery';
import AgentSidebar from './agent-sidebar';
import { agentWorkspaceKey, projectActiveAgentKey, projectAgentSidebarItems, type AgentSidebarItem } from './agent-model';
import AgentSearch from './agent-search';
import AgentHeader from './agent-header';
import AgentNetwork from './agent-network';
import AgentCommandPalette from './agent-command-palette';
import { MahayanaAssistantTurnView } from '../mahayana-assistant-turn-view';
import type { AssistantTurn } from '../mahayana-assistant-turn';
import { CallDialog } from '../adapters/calls/call-compatibility-adapter';
import {
  MiniAppDialog,
  MiniAppMarketplaceWorkspace,
  miniAppCloudBridgeDocument,
  function FeatureWorkspace({ section, onInvoice, payment, onRefund, settings: _settings, ...miniAppProps }: { section: MessengerSection; onInvoice: () => void; payment: PaymentUiState; onRefund: (orderId: string) => void; settings: SettingsWorkspaceProps } & MiniAppMarketplaceProps) {
  return <CompatibilitySurface
    section={section}
    miniApps={<MiniAppMarketplaceWorkspace {...miniAppProps} />}
    payments={<div className={styles.featureWorkspace}><WalletCards size={54} /><h2>Fabushi Pay</h2><p>自建余额、Invoice、Order、退款与外部 settlement 都由 Rust 账本结算。</p><PaymentOverview payment={payment} onInvoice={onInvoice} onRefund={onRefund} /></div>}
    calls={<div className={styles.featureWorkspace}><Phone size={54} /><h2>通话</h2><p>本机媒体已接通，Rust realtime 已具备一对一/群组通话信令状态。</p></div>}
    settings={<div className={styles.featureWorkspace} aria-hidden="true" />}
    fallback={<div className={styles.featureWorkspace}><MessageCircle size={54} /><h2>{sectionTitle(section)}</h2><p>联系人、Bot、群组和频道正在统一到同一个 Fabushi Actor/Conversation 模型。</p></div>}
  />;
}