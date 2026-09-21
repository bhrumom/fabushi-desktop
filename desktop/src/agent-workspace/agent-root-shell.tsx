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
import { marketplaceInstallAction, marketplaceInstallActionLabel } from '../../../frontend/apps/web/src/lib/marketplace-install-contract';
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
import extra from '../adapters/compatibility/compatibility.module.css';
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
import { CompatibilitySurface, buildCompatibilityPeers, compatibilityMessagingEnvelope, type CompatibilityPeerItem as PeerItem, type CompatibilityPeerKind as PeerKind, type CompatibilityPeerSource as PeerSource, type CompatibilitySection as MessengerSection } from '../adapters/compatibility/messaging-compatibility-adapter';
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
} from '../adapters/compatibility/compatibility-model';
import { useCompatibilityState } from '../adapters/compatibility/use-compatibility-state';
import { backgroundSyncLimit, cachedLegacyDisplayMessages, clearAccountScopedDesktopCaches, createTransport, defaultCommunityState, defaultProductHostSettings, formatTime, initialMessageRenderCount, initialSyncLimit, isDesktopUpdateState, matchesSection, messengerDraftsKey, messengerPreferencesKey, messengerSettingsKey, messengerSidebarWidthKey, persistAccountSyncCursor, persistMessengerProjection, projectionConversationLimit, projectionMessageLimit, readAccountSyncCursor, readDesktopMessengerPreferences, readDurableMessengerProjection, readMessengerProjection, sectionTitle, startupLegacyConversationId, upsertById, blobMediaUrl } from '../adapters/compatibility/compatibility-runtime';
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
  deleteMiniAppCloudStorage,
  listAccountAgentStore,
  readAccountAgentStoreObject,
  readAccountBots,
  readAccountMiniApps,
  readAccountSync,
  readMiniAppBotMessages,
  readMiniAppCloudStorage,
  reconcileAccountMiniApps,
  upsertAccountAgent,
  writeAccountAgentStoreObject,
  writeMiniAppCloudStorage,
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
import { CallCompatibilityAdapter } from '../features/calls/call-compatibility-adapter';
import { ContactCompatibilityAdapter } from '../features/contacts/contact-compatibility-adapter';
import { InvoiceCompatibilityDialog, PaymentCompatibilityAdapter, type PaymentUiState } from '../features/payments/payment-compatibility-adapter';
import { MiniAppCompatibilityDialog, MiniAppMarketplaceCompatibilityAdapter, generatedMiniAppPreview, miniAppCloudBridgeDocument, type MiniAppMarketplaceProps } from '../features/miniapps/miniapp-compatibility-adapter';
import { SettingsCompatibilityAdapter, SettingsNavigation, type SettingsWorkspaceProps } from '../features/settings/settings-compatibility-adapter';
import { TelegramAttachmentMenu, TelegramCommunityAdminDialog, TelegramEditMessageDialog, TelegramForwardMessageDialog, TelegramMessageContextMenu, TelegramNewConversationDialog, TelegramStoryViewer, TelegramStructuredMessageBody } from '../features/telegram/telegram-compatibility-adapter';

function toAgentTranscriptSources(
  messages: readonly DisplayMessage[],
): AgentTranscriptSourceMessage[] {
  return messages.map((message) => ({ ...message, source: 'legacy' as const }));
}

function isAgentPeer(peer: PeerItem): boolean {
  // Grok-style identity boundary: an Agent/Bot is a declared domain object,
  // never a contact whose title/id happens to contain "agent" or "assistant".
  return peer.kind === 'bot';
}

function localComputerLabel(): string {
  if (typeof navigator === 'undefined') return 'Fabushi 桌面端';
  const identity = `${navigator.platform || ''} ${navigator.userAgent || ''}`.toLowerCase();
  const platform = identity.includes('win')
    ? 'Windows'
    : identity.includes('mac')
      ? 'Mac'
      : identity.includes('linux')
        ? 'Linux'
        : '桌面端';
  return `Fabushi · ${platform}`;
}

function botMarkStateForPeer(
  peer: PeerItem,
  executions: MessagingBotExecution[],
  busy: boolean,
  hostReady: boolean,
): FabAvatarInputState {
  if (busy) return 'thinking';
  const identities = [peer.id, peer.actorId].filter((value): value is string => Boolean(value));
  const execution = [...executions]
    .sort((left, right) => (right.startedAtMs ?? 0) - (left.startedAtMs ?? 0))
    .find((candidate) => identities.includes(candidate.botId));
  if (!execution) return hostReady ? 'idle' : 'offline';
  switch (execution.state) {
    case 'queued': return 'thinking';
    case 'running': return 'working';
    case 'waitingForApproval': return 'waiting';
    case 'failed': return 'error';
    case 'cancelled': return 'offline';
    case 'completed': return 'success';
  }
}


export default function AgentRootShell() {
  const authTransport = useMemo(() => createTransport(), []);
  const localProjection = useMemo(() => readMessengerProjection(), []);
  const [startupProjection, setStartupProjection] = useState<MessengerProjection | null>(localProjection);
  const [projectionLookupComplete, setProjectionLookupComplete] = useState(Boolean(localProjection));
  const [authenticated, setAuthenticated] = useState<boolean | null>(null);
  const authTransitionEpoch = useRef(0);

  const resetToLogin = useCallback(async (revokeSession = true) => {
    // Invalidate any authStatus request that started before this explicit transition.
    const transitionEpoch = ++authTransitionEpoch.current;
    try {
      if (revokeSession) await authTransport.logout();
    } catch {
      // Product logout is best effort remotely; local account state is cleared below.
    } finally {
      await clearAccountScopedDesktopCaches();
      // A newer login may complete while logout/cache cleanup is in flight.
      if (transitionEpoch !== authTransitionEpoch.current) return;
      setStartupProjection(null);
      setProjectionLookupComplete(true);
      setAuthenticated(false);
    }
  }, [authTransport]);

  const handleHostAuthStateChange = useCallback((state: AuthState) => {
    if (state.loggedIn) {
      // Invalidate a signed-out authStatus snapshot that may still be clearing
      // account caches before it gets a chance to overwrite this fresh login.
      authTransitionEpoch.current += 1;
      // A fresh login must not remain behind the durable projection lookup.
      // The projection is optional startup data and can hydrate in the background.
      setProjectionLookupComplete(true);
      setAuthenticated(true);
    }
  }, []);

  useEffect(() => {
    if (localProjection) return;
    let closed = false;
    void readDurableMessengerProjection().then((projection) => {
      if (closed) return;
      setStartupProjection(projection);
      setProjectionLookupComplete(true);
    });
    return () => { closed = true; };
  }, [localProjection]);

  useEffect(() => {
    let closed = false;
    let retryTimer: number | undefined;
    const checkAuth = async () => {
      const requestEpoch = authTransitionEpoch.current;
      try {
        const state = await authTransport.authStatus();
        if (closed || requestEpoch !== authTransitionEpoch.current) return;
        if (!state.loggedIn) {
          await clearAccountScopedDesktopCaches();
          // Browser login can finish while the signed-out cache cleanup awaits.
          // Never let that older false snapshot replace the authenticated shell.
          if (closed || requestEpoch !== authTransitionEpoch.current) return;
          setStartupProjection(null);
        }
        setAuthenticated(state.loggedIn);
        if (!state.loggedIn) retryTimer = window.setTimeout(() => void checkAuth(), 900);
      } catch (cause) {
        if (closed) return;
        if (isTerminalAuthSessionFailure(cause)) {
          await resetToLogin(true);
          return;
        }
        // A transient Host/network failure must not replace a valid local-first shell
        // with a login/restore screen. Only an explicit loggedIn=false response signs out.
        retryTimer = window.setTimeout(() => void checkAuth(), 1_800);
      }
    };
    void checkAuth();
    return () => {
      closed = true;
      if (retryTimer) window.clearTimeout(retryTimer);
      void authTransport.close();
    };
  }, [authTransport, resetToLogin]);

  const showMessenger = projectionLookupComplete
    && authenticated !== false
    && (authenticated === true || Boolean(startupProjection));
  const showLogin = projectionLookupComplete && authenticated === false;
  return (
    <div className={styles.desktopRoot} data-testid="desktop-shell" data-local-first={showMessenger && authenticated !== true ? 'true' : undefined}>
      {showMessenger
        ? <AgentWorkspaceShell initialProjection={startupProjection} onLogout={() => resetToLogin(true)} />
        : showLogin
          ? <HostClient onAuthStateChange={handleHostAuthStateChange} />
          : <DesktopFastStartBootstrap />}
    </div>
  );
}

function DesktopFastStartBootstrap() {
  const bootstrapAgent: AgentSidebarItem = {
    key: 'agent:mahayana-assistant',
    peerKey: 'bootstrap:mahayana-assistant',
    id: 'mahayana-assistant',
    agentId: 'mahayana-assistant',
    name: 'Mahayana',
    description: 'Connecting to local Agent…',
    pinned: true,
    hidden: false,
    unread: 0,
    busy: false,
    isGroup: false,
    updatedAtMs: Date.now(),
  };
  const noop = () => {};
  return (
    <main
      className={`${styles.messenger} ${styles.fabushiUnified} ${styles.grokParity}`}
      data-testid="desktop-fast-start-bootstrap"
      aria-busy="true"
      aria-label="Fabushi is connecting"
      style={{ gridTemplateColumns: '300px minmax(420px,1fr)' }}
    >
      <aside className={styles.chatList}>
        <AgentSidebar
          agents={[bootstrapAgent]}
          activeKey={bootstrapAgent.key}
          query=""
          collapsed={false}
          hostReady={false}
          accountLabel="Connecting…"
          onQuery={noop}
          onOpen={noop}
          onNewAgent={noop}
          onToggleCollapsed={noop}
          onTogglePin={noop}
          onRename={noop}
          onHide={noop}
          onDuplicate={noop}
          onDelete={noop}
          onReorderPinned={noop}
          onBroadcast={noop}
          onOpenNetwork={noop}
          onOpenPlugins={noop}
          onOpenSettings={noop}
        />
      </aside>
      <section className={styles.chatWorkspace}>
        <div className={styles.chatEmpty}>
          <FabAvatar identity="fabushi:bootstrap:workspace" state="waking" size={72} label="Fabushi" />
          <strong>Mahayana</strong>
          <p>The Agent workspace is ready. Connecting to the local runtime…</p>
        </div>
      </section>
    </main>
  );
}

function AgentWorkspaceShell({ initialProjection, onLogout }: { initialProjection?: MessengerProjection | null; onLogout: () => Promise<void> }) {
  const transport = useMemo(() => createTransport(), []);
  const agentCoordinatorClient = useMemo(() => new AgentCoordinatorClient(transport), [transport]);
  const startupProjection = useMemo(() => initialProjection ?? readMessengerProjection(), [initialProjection]);
  const startupLegacyConversation = useMemo(() => startupLegacyConversationId(startupProjection), [startupProjection]);
  const selfHosted = useMemo(() => new SelfHostedMessagingClientV2(transport, { actorId: startupProjection?.actorId }), [transport, startupProjection]);
  const [hostReady, setHostReady] = useState(false);
  const [remoteAccountScope, setRemoteAccountScope] = useState<string | null>(null);
  const [initialLegacyHydrationMask, setInitialLegacyHydrationMask] = useState(0);
  const initialLegacyHydrationMaskRef = useRef(0);
  const initialLegacyHydrated = initialLegacyHydrationMask === 0b111;
  // Grok/Fabu parity: the primary desktop shell is Agent-first. Messenger,
  // Contacts, Channels, Calls and Payments remain backend capabilities but no
  // longer own top-level navigation.
  const [section, setSection] = useState<MessengerSection>('bots');
  const newAgentRequestPendingRef = useRef(false);
  const [pendingOpenAgentId, setPendingOpenAgentId] = useState<string | null>(null);
  const [activePeerKey, setActivePeerKey] = useState<string | null>(startupProjection?.activePeerKey ?? null);
  const legacyCompatibility = useCompatibilityState(
    startupProjection,
    startupLegacyConversation ? cachedLegacyDisplayMessages(startupLegacyConversation) : [],
  );
  const {
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
  } = legacyCompatibility;
  const shellView = useAgentShellViewState({
    initialMessageRenderCount,
    initialInfoOpen: readDesktopMessengerPreferences().showInfoPanel,
  });
  const {
    search, setSearch,
    sidebarWidth, setSidebarWidth,
    conversationSearchOpen, setConversationSearchOpen,
    agentConversationSearch, setAgentConversationSearch,
    messageRenderCount, setMessageRenderCount,
    infoOpen, setInfoOpen,
    narrowInfoOpen, setNarrowInfoOpen,
    wideInfoLayout,
    agentSettingsOpen, setAgentSettingsOpen,
    showScrollToLatest, setShowScrollToLatest,
  } = shellView;
  const [desktopUpdateState, setDesktopUpdateState] = useState<UpdateState | null>(null);
  const [desktopUpdateBusy, setDesktopUpdateBusy] = useState(false);
  const [desktopPreferences, setDesktopPreferences] = useState<DesktopMessengerPreferences>(() => readDesktopMessengerPreferences());
  const [settingsCategory, setSettingsCategory] = useState<SettingsCategory>('account');
  const settingsReturnSectionRef = useRef<MessengerSection>('chats');
  const [hostSettings, setHostSettings] = useState<ProductHostSettings>(defaultProductHostSettings);
  const [routerStatus, setRouterStatus] = useState<InferenceRouterStatus | null>(null);
  const [usageSummary, setUsageSummary] = useState<UsageSummary | null>(null);
  const [infoTab, setInfoTab] = useState<InfoTab>('media');
  // Compatibility Messenger/Mini App sends retain transport pending state.
  // Agent request/operation ownership is exclusively per-peer in the workspace controller.
  const [legacySendPending, setLegacySendPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const agentProductControllers = useAgentProductControllers({
    client: agentCoordinatorClient,
    accountScope: remoteAccountScope,
    initialAgents: startupProjection?.legacyBots ?? [],
    onError: setError,
    workflow: {
      onListed: (agentId, workflows) => {
        void mirrorAgentCloudSnapshot(agentId, FABU_AGENT_WORKFLOW_INDEX_PATH, {
          version: 1,
          workflows,
        });
      },
      onError: (_agentId, message) => setError(message),
    },
    mcp: { onError: setError },
    storeSync: {
      mirror: (agentId, objectPath, value) => mirrorAgentCloudSnapshot(agentId, objectPath, value),
      remove: (agentId, objectPath) => removeAgentCloudObject(agentId, objectPath),
      onError: (_agentId, message) => setError(message),
    },
    directory: {
      onListed: (agents) => {
        initialLegacyHydrationMaskRef.current |= 0b010;
        setInitialLegacyHydrationMask(initialLegacyHydrationMaskRef.current);
        for (const agent of agents) {
          void mirrorBotAgentCloud(agent).catch(() => {});
        }
      },
      onChanged: (action, agent) => {
        if (action === 'created' && newAgentRequestPendingRef.current) {
          newAgentRequestPendingRef.current = false;
          setPendingOpenAgentId(agent.agentId ?? agent.id);
        }
        if (action === 'deleted') {
          void invokeNativeDesktop('removeBotFromAccount', { botId: agent.id }).catch(() => {});
        } else {
          void mirrorBotAgentCloud(agent).catch(() => {});
        }
      },
      onError: setError,
    },
  });
  const agentPaletteController = agentProductControllers.palette;
  const agentSidebarController = agentProductControllers.sidebar;
  const agentNetworkController = agentProductControllers.network;
  const agentWorkflowController = agentProductControllers.workflow;
  const agentMcpController = agentProductControllers.mcp;
  const agentPrSuggestionController = agentProductControllers.pullRequests;
  const agentStoreSyncController = agentProductControllers.storeSync;
  const agentDirectoryController = agentProductControllers.directory;
  const agentPinnedOrder = agentSidebarController.pinnedOrder;
  const agentSidebarSections = agentSidebarController.sections;
  const agentSelectedKeys = agentSidebarController.selectedKeys;
  const bots = agentDirectoryController.agents;
  // AgentRootShell first-frame readiness is owned by the Agent domain, not by
  // legacy conversations/groups. A durable Agent projection is enough to render
  // and recover immediately while the authoritative directory refreshes in the
  // background; fresh accounts wait for the first Agent directory response.
  const initialAgentWorkspaceHydrated = hostReady
    && ((initialLegacyHydrationMask & 0b010) !== 0
      || bots.length > 0
      || Boolean(startupProjection?.legacyBots?.length));
  const [mutedPeerKeys, setMutedPeerKeys] = useState<Set<string>>(() => new Set());
  const [pinnedPeerKeys, setPinnedPeerKeys] = useState<Set<string>>(() => new Set());
  const [archivedPeerKeys, setArchivedPeerKeys] = useState<Set<string>>(() => new Set());
  const activePeerKeyRef = useRef<string | null>(null);
  const messagingCursorRef = useRef<string | null>(startupProjection?.cursor ?? null);
  const accountSyncCursorRef = useRef<string | null>(readAccountSyncCursor());
  const syncInFlightRef = useRef(false);
  const accountSyncInFlightRef = useRef(false);
  const typingStopTimerRef = useRef<number | null>(null);
  const peersRef = useRef<PeerItem[]>([]);
  const webRtcRef = useRef<FabushiWebRtcController | null>(null);
  const localVideoRef = useRef<HTMLVideoElement>(null);
  const remoteVideoRef = useRef<HTMLVideoElement>(null);
  const remoteAudioRef = useRef<HTMLAudioElement>(null);
  const mediaInputRef = useRef<HTMLInputElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const sessionResetInFlightRef = useRef(false);
  const messageAreaRef = useRef<HTMLDivElement | null>(null);
  const stickToLatestRef = useRef(true);
  const agentStoresRef = useRef(new Map<string, FabuAgentStore>());
  const agentComputer = useAgentComputerController({
    hostReady,
    hydrated: initialAgentWorkspaceHydrated,
    accountScope: remoteAccountScope,
    activePeerKey,
    transport,
    coordinatorClient: agentCoordinatorClient,
    label: localComputerLabel(),
    remoteControlEnabled: hostSettings.remoteControlEnabled,
    resolveAgentId: (requestedAgentId) => requestedAgentId === 'mahayana-assistant'
      || peersRef.current.some((peer) => isAgentPeer(peer)
        && (peer.agentId ?? peer.actorId ?? peer.id) === requestedAgentId)
      ? requestedAgentId
      : null,
    onError: setError,
  });

  const {
    controller: agentWorkspaceController,
    transcriptStore: agentTranscriptStore,
    coordinator: agentRuntimeCoordinator,
    submit: submitAgentWorkspace,
    openConversation: openAgentConversation,
    uploadAttachment: uploadAgentAttachment,
    resolveApproval: resolveAgentApproval,
    interrupt: interruptAgentWorkspace,
    revision: agentWorkspaceRevision,
    notify: notifyAgentWorkspaceState,
  } = useAgentWorkspaceRuntime({
    hostReady,
    coordinatorClient: agentCoordinatorClient,
    onError: (peerKey, message) => {
      if (peerKey === activePeerKeyRef.current) setError(message);
    },
    onComputerStatus: agentComputer.handleCapabilityStatus,
    onOperationStarted: (peerKey, operationId) => {
      mirrorAgentRuntimeCheckpoint(peerKey, 'running', operationId);
    },
    onOperationTerminal: (peerKey, operationId, status, message) => {
      mirrorAgentRuntimeCheckpoint(peerKey, status, operationId, message);
      mirrorAgentConversationSnapshot(peerKey);
      if (status === 'failed' && message && peerKey === activePeerKeyRef.current) {
        setError(message);
      }
    },
  });

  useEffect(() => {
    activePeerKeyRef.current = activePeerKey;
  }, [activePeerKey]);

  useEffect(() => {
    if (!error || !isTerminalAuthSessionFailure(error) || sessionResetInFlightRef.current) return;
    sessionResetInFlightRef.current = true;
    void onLogout()
      .catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))
      .finally(() => { sessionResetInFlightRef.current = false; });
  }, [error, onLogout]);

  useEffect(() => {
    try {
      window.localStorage.setItem(messengerPreferencesKey, JSON.stringify(desktopPreferences));
    } catch {
      // Desktop-only preferences are best effort.
    }
  }, [desktopPreferences]);

  useEffect(() => {
    if (startupProjection?.activePeerKey?.startsWith('selfhosted:')) {
      const conversationId = startupProjection.activePeerKey.slice('selfhosted:'.length);
      const cached = (startupProjection.selfMessages[conversationId] ?? []).filter((message) => !message.deleted);
      setMessages(cached.map(displaySelfMessage));
      return;
    }
    const legacyConversationId = startupLegacyConversationId(startupProjection);
    if (legacyConversationId) setMessages(cachedLegacyDisplayMessages(legacyConversationId));
  }, [startupProjection]);

  useEffect(() => {
    if (
      !selfConversations.length
      && !selfActors.length
      && !conversations.length
      && !bots.length
      && !groups.length
      && !accountBots.length
      && !miniAppIdentityCatalog.length
    ) return;
    const timer = window.setTimeout(() => {
      const conversationIds = new Set(
        [...selfConversations]
          .sort((left, right) => right.updatedAtMs - left.updatedAtMs)
          .slice(0, projectionConversationLimit)
          .map((conversation) => conversation.id),
      );
      const boundedMessages = Object.fromEntries(
        Object.entries(selfMessages)
          .filter(([conversationId]) => conversationIds.has(conversationId))
          .map(([conversationId, list]) => [conversationId, list.slice(-projectionMessageLimit)]),
      );
      persistMessengerProjection({
        version: 1,
        savedAtMs: Date.now(),
        actorId: selfHosted.actorId,
        cursor: messagingCursorRef.current,
        activePeerKey,
        legacyConversations: conversations,
        legacyBots: bots,
        legacyGroups: groups,
        accountBots,
        miniAppIdentityCatalog,
        selfActors,
        selfConversations: [...selfConversations]
          .sort((left, right) => right.updatedAtMs - left.updatedAtMs)
          .slice(0, projectionConversationLimit),
        selfMessages: boundedMessages,
      });
    }, 60);
    return () => window.clearTimeout(timer);
  }, [activePeerKey, accountBots, bots, conversations, groups, miniAppIdentityCatalog, selfActors, selfConversations, selfMessages, selfHosted.actorId]);

  function updateDesktopPreference<K extends keyof DesktopMessengerPreferences>(key: K, value: DesktopMessengerPreferences[K]) {
    setDesktopPreferences((current) => ({ ...current, [key]: value }));
    if (key === 'showInfoPanel') setInfoOpen(Boolean(value));
  }

  function updateHostSetting<K extends keyof ProductHostSettings>(key: K, value: ProductHostSettings[K]) {
    const settings = { ...hostSettings, [key]: value };
    setHostSettings(settings);
    void execute({ type: 'settings.update', requestId: nextRequestId('settings-update'), settings });
  }

  async function configureProviderSecret(provider: 'claude-code' | 'openrouter', value: string) {
    try {
      const name = provider === 'claude-code' ? 'inference/claude/api-key' : 'inference/openrouter/api-key';
      await invokeNativeDesktop('upsertSecrets', { name, value });
      if (hostSettings.inferenceProvider === provider) await invokeNativeDesktop('restartInferenceRouter');
      const status = await invokeNativeDesktop<InferenceRouterStatus>('getInferenceRouterStatus');
      setRouterStatus(status);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
      throw cause;
    }
  }

  async function removeProviderSecret(provider: 'claude-code' | 'openrouter') {
    try {
      const name = provider === 'claude-code' ? 'inference/claude/api-key' : 'inference/openrouter/api-key';
      await invokeNativeDesktop('removeSecrets', { name });
      const status = await invokeNativeDesktop<InferenceRouterStatus>('getInferenceRouterStatus');
      setRouterStatus(status);
      if (hostSettings.inferenceProvider === provider) updateHostSetting('inferenceProvider', 'fabushi');
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
      throw cause;
    }
  }

  useEffect(() => {
    const stored = Number(window.localStorage.getItem(messengerSidebarWidthKey));
    if (Number.isFinite(stored) && stored >= 84 && stored <= 460) setSidebarWidth(stored);
  }, []);

  useEffect(() => {
    try { window.localStorage.setItem(messengerSidebarWidthKey, String(Math.round(sidebarWidth))); } catch { /* best effort */ }
  }, [sidebarWidth]);

  useEffect(() => {
    let disposed = false;
    const acceptUpdateState = (payload: unknown) => {
      if (disposed || !isDesktopUpdateState(payload)) return;
      setDesktopUpdateState(payload);
      if (payload.type === 'available' || payload.type === 'upToDate' || payload.type === 'error') {
        setDesktopUpdateBusy(false);
      }
    };
    const unsubscribe = subscribeNativeDesktopEvents({ 'update-status': acceptUpdateState });
    void invokeNativeDesktop<UpdateState>('getUpdateStatus').then(acceptUpdateState).catch(() => {});
    const timer = window.setTimeout(() => {
      void invokeNativeDesktop<UpdateState>('checkForUpdates').then(acceptUpdateState).catch(() => {});
    }, 1_500);
    return () => {
      disposed = true;
      window.clearTimeout(timer);
      unsubscribe();
    };
  }, []);

  useEffect(() => {
    try {
      const stored = JSON.parse(window.localStorage.getItem(messengerDraftsKey) || '{}') as Record<string, unknown>;
      setDrafts(Object.fromEntries(Object.entries(stored).filter((entry): entry is [string, string] => typeof entry[1] === 'string')));
    } catch {
      setDrafts({});
    }
  }, []);

  useEffect(() => {
    try {
      window.localStorage.setItem(messengerDraftsKey, JSON.stringify(drafts));
    } catch {
      // Draft persistence is best-effort and must never block messaging.
    }
  }, [drafts]);

  useEffect(() => {
    if (!activePeerKey) return;
    const peer = peersRef.current.find((candidate) => candidate.key === activePeerKey);
    const isWorkspaceAgent = Boolean(
      peer
      && peer.source === 'legacy'
      && peer.kind !== 'group'
      && !peer.miniAppId
      && isAgentPeer(peer)
    );
    // Compatibility Messenger owns the renderer-global composer string.
    // Normal Agents render directly from AgentWorkspaceController and must
    // never copy their draft into this global state.
    if (!isWorkspaceAgent) setComposer(drafts[activePeerKey] ?? '');
    setSearch('');
    setConversationSearchOpen(false);
    setAgentConversationSearch('');
    setMessageRenderCount(initialMessageRenderCount);
  }, [activePeerKey]);

  useEffect(() => {
    try {
      const stored = JSON.parse(window.localStorage.getItem(messengerSettingsKey) || '{}') as {
        muted?: string[];
        pinned?: string[];
        archived?: string[];
      };
      setMutedPeerKeys(new Set(stored.muted ?? []));
      setPinnedPeerKeys(new Set(stored.pinned ?? []));
      setArchivedPeerKeys(new Set(stored.archived ?? []));
    } catch {
      // Ignore malformed old settings.
    }
  }, []);

  useEffect(() => {
    try {
      window.localStorage.setItem(messengerSettingsKey, JSON.stringify({
        muted: [...mutedPeerKeys],
        pinned: [...pinnedPeerKeys],
        archived: [...archivedPeerKeys],
      }));
    } catch {
      // Persistence is a convenience only.
    }
  }, [mutedPeerKeys, pinnedPeerKeys, archivedPeerKeys]);

  useEffect(() => {
    const controller = new FabushiWebRtcController(
      () => ({ deviceId: selfHosted.deviceId, sessionId: selfHosted.sessionId }),
      {
        onIdentity(identity) {
          selfHosted.actorId = identity.actorId;
          selfHosted.deviceId = identity.deviceId;
          selfHosted.sessionId = identity.sessionId;
        },
        onIncoming(call) {
          const peer = peersRef.current.find((candidate) =>
            candidate.conversationId === call.conversationId || candidate.actorId === call.fromActorId);
          setIncomingCall(call);
          setLocalCall({
            kind: call.kind,
            title: peer?.title ?? call.fromActorId,
            status: 'ringing',
            incoming: true,
            muted: false,
            videoEnabled: call.kind === 'video',
          });
        },
        onStatus(status, detail) {
          setLocalCall((current) => {
            if (!current) return current;
            if (status === 'ended') return null;
            return {
              ...current,
              status,
              error: status === 'failed' ? detail ?? 'WebRTC 连接失败' : current.error,
            };
          });
          if (status === 'connecting' || status === 'active') syncLocalCallMedia();
        },
        onRemoteStream(stream) {
          window.setTimeout(() => {
            const video = remoteVideoRef.current;
            const audio = remoteAudioRef.current;
            if (video) {
              video.srcObject = stream;
              if (stream) void video.play().catch(() => {});
            }
            if (audio) {
              audio.srcObject = stream;
              if (stream) void audio.play().catch(() => {});
            }
          }, 0);
        },
      },
    );
    webRtcRef.current = controller;
    return () => {
      if (webRtcRef.current === controller) webRtcRef.current = null;
      void controller.dispose();
    };
  }, [selfHosted]);

  useEffect(() => {
    let closed = false;
    const connection = agentCoordinatorClient.connect({
      config: { profileId: 'desktop-messenger-v2', mode: 'production' },
      onEvent: (event) => {
        if (!closed) handleRuntimeEvent(event);
      },
      onState: (state) => {
        if (closed) return;
        setHostReady(state.phase === 'ready');
        if (state.phase !== 'ready' || !state.recovered) return;
        void execute({ type: 'settings.get', requestId: nextRequestId('settings-recover') }).catch(() => {});
        refreshLegacy();
        const activeKey = activePeerKeyRef.current;
        const active = peersRef.current.find((peer) => peer.key === activeKey);
        if (active?.conversationId) {
          if (isAgentPeer(active) && !active.miniAppId) {
            void openAgentConversation(active.key, active.conversationId).catch(() => {});
          } else {
            void agentCoordinatorClient.openConversation(
              nextRequestId('conversation-recover'),
              active.conversationId,
            ).catch(() => {});
          }
        }
      },
    });
    const onCommandBridge = (event: Event) => {
      const detail = (event as CustomEvent<MahayanaCommandBridgeDetail>).detail;
      if (detail) agentRuntimeCoordinator.handleCommandBridge(detail);
    };
    window.addEventListener(MAHAYANA_COMMAND_EVENT_NAME, onCommandBridge);
    void connection.ready
      .then(async () => {
        if (closed) return;
        setHostReady(true);
        // First-frame identity hydration must not wait behind self-hosted sync or
        // account cursor reconciliation. These reads are lightweight and let
        // installed Mini App Bots (for example 全球法布施) appear immediately.
        void readAccountBots()
          .then((entries) => { if (!closed) setAccountBots(entries); })
          .catch(() => {});
        void readAccountMiniApps()
          .then((account) => {
            if (closed) return;
            const accountApps = accountMiniAppsAsMarketplaceSummaries(account);
            if (!accountApps.length) return;
            setMiniAppIdentityCatalog((current) => {
              const merged = new Map(current.map((app) => [app.pluginId, app]));
              for (const accountApp of accountApps) {
                const existing = merged.get(accountApp.pluginId);
                merged.set(accountApp.pluginId, {
                  ...existing,
                  ...accountApp,
                  bot: accountApp.bot ?? existing?.bot,
                  commands: accountApp.commands?.length ? accountApp.commands : existing?.commands,
                  surfaces: accountApp.surfaces?.length ? accountApp.surfaces : existing?.surfaces,
                });
              }
              return [...merged.values()];
            });
          })
          .catch(() => {});
        void execute({ type: 'settings.get', requestId: nextRequestId('settings-get') });
        refreshLegacy();
        if (startupLegacyConversation) {
          void execute({
            type: 'conversation.open',
            requestId: nextRequestId('conversation-open-startup'),
            conversationId: startupLegacyConversation,
          });
        }
        try {
          const account = await agentCoordinatorClient.authStatus().catch(() => null);
          const cachedActor = startupProjection?.selfActors.find((actor) => actor.id === selfHosted.actorId);
          const username = account?.user?.username?.trim() || cachedActor?.username;
          const displayName = account?.user?.nickname?.trim()
            || username
            || account?.user?.email?.trim()
            || cachedActor?.displayName
            || '当前用户';
          const identityScope = account?.user?.id ?? username ?? account?.user?.email?.trim() ?? startupProjection?.actorId;
          if (identityScope === undefined || identityScope === null || String(identityScope).trim() === '') {
            throw new Error('当前账号缺少电脑注册身份，无法启动后台在线状态');
          }
          setRemoteAccountScope(String(identityScope));
          // Contacts/Telegram/account-cursor reconciliation are compatibility
          // capabilities. They must never hold AgentRootShell, Agent Computer or
          // the Composer behind a legacy bootstrap round trip.
          void (async () => {
            await selfHosted.ensureCurrentActor(displayName, username);
            await selfHosted.sync(initialSyncLimit, messagingCursorRef.current);
            await synchronizeAccountState();
            void webRtcRef.current?.connect().catch(() => {});
          })().catch((cause: unknown) => {
            if (!closed) setError(cause instanceof Error ? cause.message : String(cause));
          });
        } catch (cause) {
          setError(cause instanceof Error ? cause.message : String(cause));
        }
      })
      .catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
    return () => {
      closed = true;
      window.removeEventListener(MAHAYANA_COMMAND_EVENT_NAME, onCommandBridge);
      void connection.dispose();
    };
  }, [agentCoordinatorClient, selfHosted, startupProjection]);

  useEffect(() => {
    if (!initialAgentWorkspaceHydrated) return;
    // MCP discovery belongs to the Agent Composer. Start it after the Agent
    // surface is usable, independently of compatibility conversation/group
    // hydration, and yield one frame so it cannot delay the first paint.
    const timer = window.setTimeout(() => {
      void agentMcpController.list().catch(() => {});
    }, 0);
    return () => window.clearTimeout(timer);
  }, [initialAgentWorkspaceHydrated, agentMcpController.list]);

  useEffect(() => {
    if (!hostReady || section !== 'settings' || !['router', 'usage'].includes(settingsCategory)) return;
    let disposed = false;
    void Promise.all([
      invokeNativeDesktop<InferenceRouterStatus>('getInferenceRouterStatus'),
      invokeNativeDesktop<UsageSummary>('getUsageSummary'),
    ]).then(([status, usage]) => {
      if (disposed) return;
      setRouterStatus(status);
      setUsageSummary(usage);
    }).catch((cause: unknown) => {
      if (!disposed) setError(cause instanceof Error ? cause.message : String(cause));
    });
    return () => { disposed = true; };
  }, [hostReady, section, settingsCategory]);

  useEffect(() => {
    if (section !== 'settings') return;
    const modal = document.querySelector<HTMLElement>('[data-testid="settings-modal-backdrop"] [role="dialog"]');
    const focusable = () => Array.from(modal?.querySelectorAll<HTMLElement>('button:not(:disabled), select:not(:disabled), input:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])') ?? []);
    window.requestAnimationFrame(() => modal?.querySelector<HTMLElement>('[data-testid="settings-close"]')?.focus());
    const handleSettingsKeys = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        closeSettings();
        return;
      }
      if (event.key !== 'Tab') return;
      const items = focusable();
      if (!items.length) return;
      const first = items[0];
      const last = items[items.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    window.addEventListener('keydown', handleSettingsKeys);
    return () => window.removeEventListener('keydown', handleSettingsKeys);
  }, [section]);

  useEffect(() => {
    if (!hostReady || section !== 'miniapps') return;
    const timer = window.setTimeout(() => {
      void refreshMiniApps(miniAppQuery);
    }, 250);
    return () => window.clearTimeout(timer);
  }, [hostReady, section, miniAppQuery]);

  useEffect(() => {
    if (!hostReady) return;
    let disposed = false;
    const synchronizeCompatibility = () => {
      if (disposed) return;
      void synchronizeAccountState();
      if (syncInFlightRef.current) return;
      syncInFlightRef.current = true;
      void selfHosted.sync(backgroundSyncLimit, messagingCursorRef.current)
        .catch(() => {})
        .finally(() => { syncInFlightRef.current = false; });
    };
    const refreshWhenForegrounded = () => {
      if (document.visibilityState === 'visible') synchronizeCompatibility();
    };
    const unsubscribeNative = subscribeNativeDesktopEvents({
      'account-state-changed': synchronizeCompatibility,
      'window-state': (payload) => {
        if ((payload as { focused?: boolean } | null)?.focused) synchronizeCompatibility();
      },
    });
    window.addEventListener('focus', refreshWhenForegrounded);
    window.addEventListener('online', synchronizeCompatibility);
    document.addEventListener('visibilitychange', refreshWhenForegrounded);
    return () => {
      disposed = true;
      unsubscribeNative();
      window.removeEventListener('focus', refreshWhenForegrounded);
      window.removeEventListener('online', synchronizeCompatibility);
      document.removeEventListener('visibilitychange', refreshWhenForegrounded);
    };
  }, [hostReady, selfHosted]);

  useEffect(() => () => {
    for (const timer of typingExpiryTimersRef.current.values()) window.clearTimeout(timer);
    typingExpiryTimersRef.current.clear();
  }, []);

  function nextRequestId(prefix: string): string {
    return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  }

  function execute(command: Parameters<MahayanaHostTransport['execute']>[0]) {
    return transport.execute(command).catch((cause: unknown) => {
      setError(cause instanceof Error ? cause.message : String(cause));
    });
  }

  function refreshLegacy(mask = 0) {
    const requests: Array<Promise<unknown>> = [];
    if ((mask & 0b001) === 0) requests.push(execute({ type: 'conversation.list', requestId: nextRequestId('conversation-list') }));
    if ((mask & 0b010) === 0) requests.push(agentDirectoryController.list());
    if ((mask & 0b100) === 0) requests.push(execute({ type: 'group.list', requestId: nextRequestId('group-list') }));
    void Promise.all(requests).catch(() => {});
  }

  function displaySelfMessage(message: MessagingMessage): DisplayMessage {
    return {
      id: message.id,
      source: 'selfhosted',
      role: message.senderId === selfHosted.actorId ? 'me' : 'peer',
      text: messagingText(message),
      createdAtMs: message.createdAtMs,
      pinned: message.pinned,
      reactions: message.reactions.map((reaction) => `${reaction.reaction} ${reaction.count}`),
      invoiceId: message.content.type === 'invoice'
        ? (message.content.data as { invoiceId?: string } | undefined)?.invoiceId
        : undefined,
      media: ['photo', 'video', 'document'].includes(message.content.type)
        ? (message.content.data as { media?: MessagingMediaRef } | undefined)?.media
        : undefined,
      mediaType: ['photo', 'video', 'document'].includes(message.content.type)
        ? message.content.type as 'photo' | 'video' | 'document'
        : undefined,
    };
  }

  function agentStoreFor(agentId: string): FabuAgentStore {
    const existing = agentStoresRef.current.get(agentId);
    if (existing) return existing;
    const store = new FabuAgentStore(agentId, {
      list: (id, prefix) => listAccountAgentStore(id, prefix),
      read: (id, path) => readAccountAgentStoreObject(id, path),
      write: (id, path, dataBase64, options) => writeAccountAgentStoreObject(id, path, dataBase64, options),
      delete: (id, path, baseEtag) => deleteAccountAgentStoreObject(id, path, baseEtag),
    });
    agentStoresRef.current.set(agentId, store);
    return store;
  }

  async function mirrorAgentCloudSnapshot(
    agentId: string,
    path: string,
    value: unknown,
  ): Promise<void> {
    if (!agentId.trim()) return;
    const store = agentStoreFor(agentId);
    try {
      await store.writeJson(path, value);
      await store.checkpointRoot();
    } catch {
      // The local Host remains authoritative while offline or during an
      // etag conflict. The content-addressed backend preserves conflicting
      // writes instead of silently overwriting another device.
    }
  }

  async function removeAgentCloudObject(agentId: string, path: string): Promise<void> {
    if (!agentId.trim()) return;
    try {
      const store = agentStoreFor(agentId);
      await store.delete(path);
      await store.checkpointRoot();
    } catch {
      // Deletion is best-effort; account sync/reconciliation can retry.
    }
  }

  function agentPeerForRuntimeKey(peerKey: string | null | undefined): PeerItem | undefined {
    if (!peerKey) return undefined;
    const peer = peersRef.current.find((candidate) => candidate.key === peerKey);
    return peer && isAgentPeer(peer) && !peer.miniAppId ? peer : undefined;
  }

  function mirrorAgentRuntimeCheckpoint(
    peerKey: string | null | undefined,
    status: 'running' | 'completed' | 'failed' | 'interrupted',
    operationId: string,
    message?: string,
  ): void {
    const peer = agentPeerForRuntimeKey(peerKey);
    if (!peer) return;
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    void mirrorAgentCloudSnapshot(agentId, FABU_AGENT_RUNTIME_CHECKPOINT_PATH, {
      schemaVersion: 1,
      agentId,
      conversationId: peer.conversationId,
      operationId,
      status,
      updatedAtMs: Date.now(),
      ...(message ? { message } : {}),
    });
  }

  function mirrorAgentConversationSnapshot(peerKey: string | null | undefined): void {
    const peer = agentPeerForRuntimeKey(peerKey);
    if (!peer?.conversationId) return;
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    window.setTimeout(() => {
      void mirrorAgentCloudSnapshot(agentId, fabuAgentConversationTranscriptPath(peer.conversationId!), {
        schemaVersion: 1,
        agentId,
        conversationId: peer.conversationId,
        entries: agentTranscriptStore.entries(peer.key),
        updatedAtMs: Date.now(),
      });
    }, 0);
  }

  async function restoreAgentConversationFromCloud(peer: PeerItem): Promise<boolean> {
    if (!isAgentPeer(peer) || peer.miniAppId || !peer.conversationId) return false;
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    try {
      const recovered = await restoreAgentStoreWorkspace(agentStoreFor(agentId), peer.conversationId);
      const controller = agentWorkspaceController;
      let changed = false;

      if (
        recovered.entries.length
        && !controller.operationForPeer(peer.key)
        && agentTranscriptStore.entries(peer.key).length === 0
      ) {
        agentTranscriptStore.hydrateEntries(peer.key, recovered.entries);
        changed = true;
      }

      if (recovered.attachments.length && controller.attachmentsForPeer(peer.key).length === 0) {
        controller.setAttachments(peer.key, recovered.attachments);
        changed = true;
      }

      if (changed) notifyAgentWorkspaceState();
      return recovered.entries.length > 0;
    } catch {
      return false;
    }
  }

  async function mirrorBotAgentCloud(bot: BotSummary): Promise<void> {
    const identity = projectFabuBotIdentity(bot);
    const profile = projectFabuAgentProfile({
      id: bot.id,
      agentId: bot.agentId,
      conversationId: bot.conversationId,
      name: bot.name,
      description: bot.description,
      title: bot.title,
      avatarShape: bot.avatarShape,
      avatarColor: bot.avatarColor,
      hidden: bot.hidden,
      notificationsEnabled: bot.notificationsEnabled,
      notifyOnUpdates: bot.notifyOnUpdates,
    });
    const settings = projectFabuAgentSettings({
      id: bot.id,
      hidden: bot.hidden,
      notificationsEnabled: bot.notificationsEnabled,
      notifyOnUpdates: bot.notifyOnUpdates,
    });
    await invokeNativeDesktop('addBotToAccount', {
      botId: identity.botId,
      bot: {
        ...bot,
        agentId: identity.agentId,
        displayName: bot.name,
      },
    });
    await upsertAccountAgent(identity.agentId, profile, {
      agentId: identity.agentId,
      name: profile.name,
      mode: 'default',
    });
    const store = agentStoreFor(identity.agentId);
    await Promise.all([
      store.writeProfile(profile),
      store.writeSettings(settings),
    ]);
    await store.checkpointRoot();
  }

  function showSelfConversation(conversationId: string) {
    setMessages((selfMessages[conversationId] ?? []).filter((message) => !message.deleted).map(displaySelfMessage));
  }

  function handleSelfHostedEvent(runtimeEvent: RuntimeEvent): boolean {
    const envelope = compatibilityMessagingEnvelope(runtimeEvent);
    if (!envelope) return false;
    const previousCursor = messagingCursorRef.current;
    if (envelope.cursor) messagingCursorRef.current = envelope.cursor;
    const event = envelope.event;
    switch (event.type) {
      case 'syncBatch': {
        const payload = event as unknown as {
          actors?: MessagingActor[];
          conversations?: MessagingConversation[];
          messages?: MessagingMessage[];
          invoices?: MessagingInvoice[];
          orders?: MessagingOrder[];
          stories?: MessagingStory[];
          communities?: MessagingCommunityState[];
          bots?: MessagingBotProfile[];
          botExecutions?: MessagingBotExecution[];
        };
        const grouped: Record<string, MessagingMessage[]> = {};
        for (const message of payload.messages ?? []) {
          (grouped[message.conversationId] ??= []).push(message);
        }
        for (const list of Object.values(grouped)) list.sort((a, b) => a.createdAtMs - b.createdAtMs);
        setSelfActors((current) => (payload.actors ?? []).reduce((items, actor) => upsertById(items, actor), previousCursor ? current : []));
        setSelfConversations((current) => (payload.conversations ?? []).reduce((items, conversation) => upsertById(items, conversation), previousCursor ? current : []));
        setSelfMessages((current) => {
          const next = previousCursor ? { ...current } : {};
          for (const [conversationId, incoming] of Object.entries(grouped)) {
            next[conversationId] = incoming.reduce((items, message) => upsertById(items, message), next[conversationId] ?? [])
              .sort((a, b) => a.createdAtMs - b.createdAtMs);
          }
          const active = activePeerKeyRef.current;
          if (active?.startsWith('selfhosted:')) {
            const conversationId = active.slice('selfhosted:'.length);
            setMessages((next[conversationId] ?? []).filter((message) => !message.deleted).map(displaySelfMessage));
          }
          return next;
        });
        setSelfInvoices((current) => (payload.invoices ?? []).reduce((items, item) => upsertById(items, item), previousCursor ? current : []));
        setSelfOrders((current) => (payload.orders ?? []).reduce((items, item) => upsertById(items, item), previousCursor ? current : []));
        setSelfStories((current) => (payload.stories ?? []).reduce((items, item) => upsertById(items, item), previousCursor ? current : []));
        setSelfCommunities((current) => (payload.communities ?? []).reduce((items, item) => [...items.filter((existing) => existing.conversationId !== item.conversationId), item], previousCursor ? current : []));
        setSelfBotProfiles((current) => (payload.bots ?? []).reduce((items, item) => [...items.filter((existing) => existing.actorId !== item.actorId), item], previousCursor ? current : []));
        setSelfBotExecutions((current) => (payload.botExecutions ?? []).reduce((items, item) => upsertById(items, item), previousCursor ? current : []));
        break;
      }
      case 'conversationChanged': {
        const conversation = (event as unknown as { conversation: MessagingConversation }).conversation;
        setSelfConversations((current) => {
          const existing = current.find((item) => item.id === conversation.id);
          return upsertById(current, existing
            ? {
                ...conversation,
                lastReadMessageId: existing.lastReadMessageId,
                unreadCount: existing.unreadCount,
                markedUnread: existing.markedUnread,
              }
            : conversation);
        });
        break;
      }
      case 'messageAdded':
      case 'messageChanged': {
        const message = (event as unknown as { message: MessagingMessage }).message;
        const isActiveConversation = activePeerKeyRef.current === `selfhosted:${message.conversationId}`;
        const isIncoming = message.senderId !== selfHosted.actorId;
        setSelfMessages((current) => {
          const list = upsertById(current[message.conversationId] ?? [], message)
            .sort((a, b) => a.createdAtMs - b.createdAtMs);
          const next = { ...current, [message.conversationId]: list };
          if (isActiveConversation) {
            setMessages(list.filter((item) => !item.deleted).map(displaySelfMessage));
          }
          return next;
        });
        if (event.type === 'messageAdded') {
          setSelfConversations((current) => current.map((conversation) => conversation.id === message.conversationId
            ? {
                ...conversation,
                lastMessageId: message.id,
                updatedAtMs: Math.max(conversation.updatedAtMs, message.createdAtMs),
                unreadCount: isIncoming && !isActiveConversation
                  ? conversation.unreadCount + 1
                  : conversation.unreadCount,
              }
            : conversation));
          if (isIncoming && isActiveConversation) {
            void selfHosted.markRead(message.conversationId, message.id).catch(() => {});
          }
        }
        setLegacySendPending(false);
        break;
      }
      case 'readChanged': {
        const payload = event as unknown as { conversationId: string; actorId: string; messageId: string };
        if (payload.actorId === selfHosted.actorId) {
          setSelfConversations((current) => current.map((conversation) => conversation.id === payload.conversationId
            ? {
                ...conversation,
                lastReadMessageId: payload.messageId,
                unreadCount: 0,
                markedUnread: false,
              }
            : conversation));
        }
        break;
      }
      case 'messagesDeleted': {
        const payload = event as unknown as { conversationId: string; messageIds: string[] };
        setSelfMessages((current) => {
          const list = (current[payload.conversationId] ?? []).filter((message) => !payload.messageIds.includes(message.id));
          if (activePeerKeyRef.current === `selfhosted:${payload.conversationId}`) {
            setMessages(list.map(displaySelfMessage));
          }
          return { ...current, [payload.conversationId]: list };
        });
        break;
      }
      case 'actorChanged': {
        const actor = (event as unknown as { actor: MessagingActor }).actor;
        setSelfActors((current) => upsertById(current, actor));
        break;
      }
      case 'typingChanged': {
        const payload = event as unknown as { conversationId: string; actorId: string; action?: string | null; expiresAtMs?: number | null };
        if (payload.actorId === selfHosted.actorId) break;
        const timerKey = `${payload.conversationId}\0${payload.actorId}`;
        const previousTimer = typingExpiryTimersRef.current.get(timerKey);
        if (previousTimer !== undefined) {
          window.clearTimeout(previousTimer);
          typingExpiryTimersRef.current.delete(timerKey);
        }
        const expiresAtMs = payload.expiresAtMs ?? 0;
        const active = Boolean(payload.action) && expiresAtMs > Date.now();
        setTypingByConversation((current) => {
          const actors = { ...(current[payload.conversationId] ?? {}) };
          if (active) actors[payload.actorId] = expiresAtMs;
          else delete actors[payload.actorId];
          const next = { ...current };
          if (Object.keys(actors).length) next[payload.conversationId] = actors;
          else delete next[payload.conversationId];
          return next;
        });
        if (active) {
          const timer = window.setTimeout(() => {
            typingExpiryTimersRef.current.delete(timerKey);
            setTypingByConversation((current) => {
              const actors = { ...(current[payload.conversationId] ?? {}) };
              if ((actors[payload.actorId] ?? 0) > Date.now()) return current;
              delete actors[payload.actorId];
              const next = { ...current };
              if (Object.keys(actors).length) next[payload.conversationId] = actors;
              else delete next[payload.conversationId];
              return next;
            });
          }, Math.max(0, expiresAtMs - Date.now()) + 25);
          typingExpiryTimersRef.current.set(timerKey, timer);
        }
        break;
      }
      case 'storyChanged': {
        const story = (event as unknown as { story: MessagingStory }).story;
        setSelfStories((current) => upsertById(current, story));
        setActiveStory((current) => current?.id === story.id ? story : current);
        break;
      }
      case 'storyDeleted': {
        const storyId = String((event as unknown as { storyId: string }).storyId);
        setSelfStories((current) => current.filter((story) => story.id !== storyId));
        setActiveStory((current) => current?.id === storyId ? null : current);
        break;
      }
      case 'communityChanged': {
        const community = (event as unknown as { community: MessagingCommunityState }).community;
        setSelfCommunities((current) => [...current.filter((item) => item.conversationId !== community.conversationId), community]);
        break;
      }
      case 'botChanged': {
        const payload = event as unknown as { profile?: MessagingBotProfile | null; execution?: MessagingBotExecution | null };
        if (payload.profile) setSelfBotProfiles((current) => [...current.filter((item) => item.actorId !== payload.profile!.actorId), payload.profile!]);
        if (payload.execution) setSelfBotExecutions((current) => upsertById(current, payload.execution!));
        break;
      }
      case 'invoiceChanged': {
        const invoice = (event as unknown as { invoice: MessagingInvoice }).invoice;
        setSelfInvoices((current) => upsertById(current, invoice));
        break;
      }
      case 'orderChanged': {
        const order = (event as unknown as { order: MessagingOrder }).order;
        setSelfOrders((current) => upsertById(current, order));
        void selfHosted.requestWalletStatus().catch(() => {});
        break;
      }
      case 'walletStatus': {
        const payload = event as unknown as { account?: MessagingWalletAccount | null; recentEntries?: MessagingLedgerEntry[] };
        setWalletAccount(payload.account ?? null);
        setWalletEntries(payload.recentEntries ?? []);
        break;
      }
      default:
        break;
    }
    return true;
  }

  function handleRuntimeEvent(event: RuntimeEvent) {
    if (handleSelfHostedEvent(event)) return;
    if (agentDirectoryController.handle(event)) return;
    if (agentNetworkController.handle(event)) return;
    if (agentStoreSyncController.handle(event)) return;
    if (agentWorkflowController.handle(event)) return;
    if (agentMcpController.handle(event)) return;
    // Normal Agent chat/operation events are owned by AgentRuntimeCoordinator.
    // The switch below remains only as a compatibility fallback for legacy
    // Messenger/Host event shapes that cannot be attributed to an Agent.
    if (agentRuntimeCoordinator.handle(event)) return;

    switch (event.type) {
      case 'host.ready':
        setHostReady(true);
        break;
      case 'conversation.listed':
        initialLegacyHydrationMaskRef.current |= 0b001;
        setInitialLegacyHydrationMask(initialLegacyHydrationMaskRef.current);
        setConversations(event.conversations);
        if (!activePeerKeyRef.current && event.conversations[0]) {
          const conversation = event.conversations[0];
          setActivePeerKey(`legacy:conversation:${conversation.id}`);
          setMessages(cachedLegacyDisplayMessages(conversation.id));
          void execute({ type: 'conversation.open', requestId: nextRequestId('conversation-open'), conversationId: conversation.id });
        }
        break;
      case 'conversation.opened': {
        const ownerPeer = peersRef.current.find((peer) =>
          peer.conversationId === event.conversationId && isAgentPeer(peer) && !peer.miniAppId,
        );
        const openedMessages: DisplayMessage[] = event.messages.map((message) => ({
          id: message.id,
          source: 'legacy',
          role: message.role === 'user' ? 'me' : 'peer',
          text: message.text,
          createdAtMs: message.createdAtMs,
          kind: 'message',
        }));
        if (ownerPeer) {
          const agentId = ownerPeer.agentId ?? ownerPeer.actorId ?? ownerPeer.id;
          void mirrorAgentCloudSnapshot(agentId, fabuAgentConversationTranscriptPath(event.conversationId), {
            schemaVersion: 1,
            agentId,
            conversationId: event.conversationId,
            entries: projectTranscriptEntries(openedMessages),
            updatedAtMs: Date.now(),
          });
          // Hydration is owned by the Agent whose conversation was opened.
          // A different visible Agent being busy must never suppress this
          // history, and an owner that is actively streaming must never be
          // overwritten by a late conversation.opened response.
          if (!agentWorkspaceController.operationForPeer(ownerPeer.key)) {
            const currentEntries = agentTranscriptStore.entries(ownerPeer.key);
            if (openedMessages.length || currentEntries.length === 0) {
              agentTranscriptStore.replace(ownerPeer.key, toAgentTranscriptSources(openedMessages));
              notifyAgentWorkspaceState();
            }
          }
          break;
        }
        if (agentWorkspaceController.operationForPeer(activePeerKeyRef.current)) break;
        if (activePeerKeyRef.current === `legacy:conversation:${event.conversationId}`
          || peersRef.current.some((peer) => peer.key === activePeerKeyRef.current
            && peer.source === 'legacy'
            && peer.conversationId === event.conversationId)) {
          setMessages(openedMessages);
        }
        break;
      }
      case 'group.listed':
        initialLegacyHydrationMaskRef.current |= 0b100;
        setInitialLegacyHydrationMask(initialLegacyHydrationMaskRef.current);
        setGroups(event.groups);
        break;
      case 'group.changed':
        setGroups((current) => event.action === 'deleted'
          ? current.filter((group) => group.id !== event.group.id)
          : upsertById(current, event.group));
        if (activePeerKeyRef.current === `legacy:group:${event.group.id}`) {
          setMessages(event.group.messages.map((message) => ({
            id: message.id,
            source: 'legacy',
            role: message.speaker.kind === 'user' ? 'me' : 'peer',
            text: message.content,
            createdAtMs: message.createdAtMs,
          })));
          setLegacySendPending(false);
        }
        break;
      case 'settings.changed':
        setHostSettings(event.settings);
        break;
      case 'chat.message':
        // Agent-owned chat is consumed above by AgentRuntimeCoordinator.
        // What reaches this branch belongs to compatibility Messenger surfaces.
        setMessages((current) => {
          if (event.role === 'user') {
            const optimisticIndex = current.findIndex((message) =>
              message.role === 'me' && message.optimistic === true && message.text === event.text,
            );
            if (optimisticIndex >= 0) {
              return current.map((message, messageIndex) => messageIndex === optimisticIndex
                ? { ...message, optimistic: false, operationId: event.operationId ?? message.operationId }
                : message);
            }
          } else if (event.operationId) {
            const streamingIndex = current.findIndex((message) =>
              message.role === 'peer'
              && message.kind === 'message'
              && message.operationId === event.operationId
              && message.streaming === true,
            );
            if (streamingIndex >= 0) {
              return current.map((message, messageIndex) => messageIndex === streamingIndex
                ? { ...message, text: event.text, streaming: false }
                : message);
            }
          }
          return [...current, {
            id: nextRequestId('message'),
            source: 'legacy',
            role: event.role === 'user' ? 'me' : 'peer',
            text: event.text,
            createdAtMs: Date.now(),
            kind: 'message',
            operationId: event.operationId,
            streaming: false,
          }];
        });
        break;
      case 'chat.delta':
        setMessages((current) => {
          const operationId = event.operationId;
          const index = current.findIndex((message) =>
            message.kind === 'message'
            && message.role === 'peer'
            && message.operationId === operationId
            && message.streaming === true,
          );
          if (index < 0) return [...current, {
            id: `${operationId ?? 'legacy'}:stream`,
            source: 'legacy',
            role: 'peer',
            text: event.delta,
            createdAtMs: Date.now(),
            kind: 'message',
            operationId,
            streaming: true,
          }];
          return current.map((message, messageIndex) => messageIndex === index
            ? { ...message, text: `${message.text}${event.delta}`, streaming: true }
            : message);
        });
        break;
      case 'operation.started':
      case 'model.routed':
      case 'agent.step':
      case 'operation.interrupted':
      case 'operation.completed':
        break;
      case 'miniapp.opened':
        if (event.html) {
          const title = miniAppIdentityCatalog.find((app) => app.pluginId === event.miniAppId)?.displayName ?? marketplaceApps.find((app) => app.pluginId === event.miniAppId)?.displayName ?? event.miniAppId;
          void showMiniAppDocument(event.miniAppId, title, event.html).catch((cause) => {
            setError(cause instanceof Error ? cause.message : String(cause));
          });
        }
        break;
      case 'operation.failed':
        // Agent failures are consumed by AgentRuntimeCoordinator. Preserve the
        // legacy error banner only for compatibility operations.
        setError(event.message);
        break;
      case 'host.closed':
        agentRuntimeCoordinator.resetOperations();
        notifyAgentWorkspaceState();
        setHostReady(false);
        break;
      default:
        break;
    }
  }

  useEffect(() => {
    if (section !== 'payments') return;
    void selfHosted.requestWalletStatus().catch((cause: unknown) => {
      setError(cause instanceof Error ? cause.message : String(cause));
    });
  }, [section, selfHosted]);

  const visibleStories = useMemo(() => selfStories
    .filter((story) => story.ownerId !== selfHosted.actorId && (story.pinnedToProfile || story.expiresAtMs > Date.now()))
    .sort((left, right) => right.createdAtMs - left.createdAtMs), [selfStories, selfHosted]);

  const peers = useMemo<PeerItem[]>(() => buildCompatibilityPeers({
    conversations,
    bots,
    accountBots,
    groups,
    selfActors,
    selfConversations,
    pinnedPeerKeys,
    archivedPeerKeys,
    miniAppIdentityCatalog,
    installedMiniApps,
    selfActorId: selfHosted.actorId,
  }), [conversations, bots, accountBots, groups, selfActors, selfConversations, pinnedPeerKeys, archivedPeerKeys, miniAppIdentityCatalog, installedMiniApps, selfHosted.actorId]);

  peersRef.current = peers;
  useEffect(() => {
    agentRuntimeCoordinator.bindAgentPeers(peers.flatMap((peer) => {
      if (!isAgentPeer(peer) || peer.miniAppId) return [];
      const agentId = peer.agentId ?? peer.actorId;
      return agentId ? [{ agentId, peerKey: peer.key }] : [];
    }));
  }, [agentRuntimeCoordinator, peers]);
  const agentActivityByPeer = Object.fromEntries(peers.map((peer) => {
    const thread = isAgentPeer(peer) && !peer.miniAppId
      ? agentTranscriptStore.thread(peer.key)
      : peer.key === activePeerKey
        ? messages
        : peer.miniAppId
          ? miniAppBotThreadsRef.current[peer.miniAppId] ?? []
          : [];
    const lastMessage = [...thread]
      .reverse()
      .find((message) => message.kind === 'message' && Boolean(message.text.trim()));
    const runningTurn = [...thread]
      .reverse()
      .find((message) => message.kind === 'assistant-turn' && message.assistantTurn?.status === 'running')
      ?.assistantTurn;
    const runningActivity = runningTurn
      ? [...runningTurn.parts].reverse().find((part) =>
        (part.kind === 'tool' || part.kind === 'activity') && part.status === 'running',
      )
      : undefined;
    const waitingActivity = runningTurn
      ? [...runningTurn.parts].reverse().find((part) =>
        part.kind === 'activity'
        && part.status === 'running'
        && (part.title === '等待授权' || part.title === '需要补充信息'),
      )
      : undefined;
    const isComposingMessage = peer.source === 'selfhosted'
      && Boolean(peer.conversationId)
      && Object.keys(typingByConversation[peer.conversationId!] ?? {}).length > 0;
    return [peer.key, {
      draftPrompt: peer.source === 'legacy' && peer.kind !== 'group' && !peer.miniAppId
        ? agentWorkspaceController.draftForPeer(peer.key)
        : drafts[peer.key],
      lastMessage: lastMessage?.text.trim().slice(0, 180),
      waitingReason: waitingActivity?.kind === 'activity'
        ? (waitingActivity.detail?.trim() || waitingActivity.title)
        : undefined,
      currentActivity: runningActivity && (runningActivity.kind === 'tool' || runningActivity.kind === 'activity')
        ? (runningActivity.detail?.trim() || runningActivity.title)
        : undefined,
      isComposingMessage,
    }] as const;
  }));
  const agentOperationSnapshot = useMemo(
    () => agentWorkspaceController.snapshot(),
    [agentWorkspaceRevision],
  );
  const agentRequestSnapshot = useMemo(
    () => agentWorkspaceController.requestSnapshot(),
    [agentWorkspaceRevision],
  );
  const agentBusyByPeer = Object.fromEntries(peers.flatMap((peer) => {
    const activityId = agentOperationSnapshot[peer.key] ?? agentRequestSnapshot[peer.key];
    return activityId ? [[peer.key, activityId] as const] : [];
  }));
  const legacyPinnedAgentKeys = [...new Set(peers
    .filter((peer) => peer.pinned && (peer.kind === 'bot' || peer.kind === 'group'))
    .map((peer) => agentWorkspaceKey(peer)))];
  const legacyPinnedAgentSignature = legacyPinnedAgentKeys.join('\u001f');
  const agentItems: AgentSidebarItem[] = projectAgentSidebarItems(
    peers,
    agentBusyByPeer,
    agentActivityByPeer,
    agentPinnedOrder,
  );
  const activePeer = peers.find((peer) => peer.key === activePeerKey) ?? null;
  const activeAgentBot: BotSummary | null = activePeer && isAgentPeer(activePeer) && !activePeer.miniAppId
    ? bots.find((bot) => bot.id === (activePeer.actorId ?? activePeer.id))
      ?? (() => {
        const membership = accountBots.find((entry) => entry.bot.id === (activePeer.actorId ?? activePeer.id));
        if (!membership) return null;
        const bot = membership.bot;
        return {
          id: bot.id,
          agentId: bot.agentId ?? activePeer.agentId ?? bot.id,
          name: bot.displayName ?? bot.username ?? activePeer.title,
          description: bot.description ?? '',
          title: bot.title ?? '',
          hidden: bot.hidden === true,
          ...(bot.avatar ? { avatar: bot.avatar } : {}),
          ...(bot.avatarShape ? { avatarShape: bot.avatarShape } : {}),
          ...(bot.avatarColor ? { avatarColor: bot.avatarColor } : {}),
          notificationsEnabled: bot.notificationsEnabled ?? true,
          notifyOnUpdates: bot.notifyOnUpdates ?? bot.notificationsEnabled ?? true,
          unread: bot.unread === true,
          ...(bot.conversationId ? { conversationId: bot.conversationId } : {}),
        } satisfies BotSummary;
      })()
      ?? {
        id: activePeer.actorId ?? activePeer.id,
        agentId: activePeer.agentId ?? activePeer.actorId ?? activePeer.id,
        name: activePeer.title,
        description: activePeer.subtitle,
        title: '',
        hidden: activePeer.hidden === true,
        notificationsEnabled: true,
        notifyOnUpdates: true,
        unread: activePeer.unread > 0,
        ...(activePeer.conversationId ? { conversationId: activePeer.conversationId } : {}),
      }
    : null;
  const activeAgentReply = activePeer && isAgentPeer(activePeer) && !activePeer.miniAppId
    ? agentWorkspaceController.replyForPeer(activePeer.key)
    : undefined;
  const {
    controller: agentSettingsController,
    snapshot: agentSettingsSnapshot,
  } = useAgentSettingsController(agentDirectoryController, activeAgentBot);
  const activeAgentKey = projectActiveAgentKey(activePeer);
  const activeAgentItem = activeAgentKey
    ? agentItems.find((item) => item.key === activeAgentKey) ?? null
    : null;
  const activeAgentPinned = activeAgentItem?.pinned === true;

  useEffect(() => {
    if (!initialAgentWorkspaceHydrated || !agentSidebarController.ready) return;
    agentSidebarController.adoptLegacyPinnedState(legacyPinnedAgentKeys);
  }, [
    initialAgentWorkspaceHydrated,
    agentSidebarController.ready,
    agentSidebarController.adoptLegacyPinnedState,
    legacyPinnedAgentSignature,
  ]);

  useEffect(() => {
    if (!pendingOpenAgentId) return;
    const peer = peers.find((candidate) =>
      candidate.kind === 'bot'
      && (candidate.agentId ?? candidate.actorId ?? candidate.id) === pendingOpenAgentId,
    );
    if (!peer) return;
    setPendingOpenAgentId(null);
    setSection('bots');
    void openPeer(peer);
  }, [pendingOpenAgentId, peers]);

  const activeTypingActors = activePeer?.source === 'selfhosted' && activePeer.conversationId
    ? Object.keys(typingByConversation[activePeer.conversationId] ?? {})
    : [];
  const sectionIsPeerList = ['chats', 'contacts', 'bots', 'groups', 'channels', 'saved', 'archive'].includes(section);
  const layoutInfoOpen = wideInfoLayout ? infoOpen : narrowInfoOpen;
  const infoPanelVisible = Boolean(layoutInfoOpen && activePeer && sectionIsPeerList);
  const infoPanelDocked = Boolean(infoPanelVisible && wideInfoLayout);
  const currentActor = selfActors.find((actor) => actor.id === selfHosted.actorId);
  const localComputerOnline = agentComputer.online;
  const localComputerStatus = agentComputer.status;
  const pendingRemoteAuthorization = agentComputer.state?.pendingAuthorization ?? null;
  // Normal Agent timelines render directly from AgentTranscriptStore. The
  // renderer-global messages array is now compatibility-only for Messenger,
  // groups and Mini Apps.
  const matchingMessages = messages;
  const renderedMessages = matchingMessages.slice(Math.max(0, matchingMessages.length - messageRenderCount));
  const botTranscriptMessages = renderedMessages;
  const allAgentTranscriptEntries: TranscriptEntry[] = activePeer && isAgentPeer(activePeer) && !activePeer.miniAppId
    ? agentTranscriptStore.entries(activePeer.key)
    : [];
  const agentTranscriptEntries: TranscriptEntry[] = allAgentTranscriptEntries.length || (activePeer && isAgentPeer(activePeer) && !activePeer.miniAppId)
    ? allAgentTranscriptEntries.slice(Math.max(0, allAgentTranscriptEntries.length - messageRenderCount))
    : activePeer && isAgentPeer(activePeer)
      ? projectTranscriptEntries(botTranscriptMessages)
      : [];
  const agentHasEarlierEntries = allAgentTranscriptEntries.length > agentTranscriptEntries.length;
  const activeAgentOperationId = activePeer
    ? agentOperationSnapshot[activePeer.key] ?? null
    : null;
  const activePeerBusy = Boolean(activePeer && (
    activeAgentOperationId
    || agentRequestSnapshot[activePeer.key]
  ));

  async function createAgent(): Promise<void> {
    setSection('bots');
    setSearch('');
    newAgentRequestPendingRef.current = true;
    let accepted: unknown;
    try {
      accepted = await agentDirectoryController.create({ name: 'New chat', description: '' });
    } catch {
      newAgentRequestPendingRef.current = false;
      return;
    }
    if (!accepted) newAgentRequestPendingRef.current = false;
  }

  function peerForAgent(item: AgentSidebarItem): PeerItem | undefined {
    return peersRef.current.find((peer) => peer.key === item.peerKey)
      ?? peersRef.current.find((peer) =>
        (peer.kind === 'bot' || peer.kind === 'group') && agentWorkspaceKey(peer) === item.key,
      );
  }

  async function renameAgent(item: AgentSidebarItem): Promise<void> {
    const peer = peerForAgent(item);
    if (!peer || peer.kind !== 'bot' || !peer.key.startsWith('legacy:bot:') && !peer.key.startsWith('legacy:conversation:')) {
      setError('Only locally managed Agents can be renamed from this shell.');
      return;
    }
    const name = window.prompt('Rename Agent', peer.title)?.trim();
    if (!name || name === peer.title) return;
    try {
      await agentDirectoryController.update(peer.actorId ?? peer.id, { name });
    } catch {
      return;
    }
  }

  async function duplicateAgent(item: AgentSidebarItem): Promise<void> {
    const peer = peerForAgent(item);
    const botId = peer?.source === 'legacy' && peer.kind === 'bot'
      ? peer.actorId ?? (peer.key.startsWith('legacy:bot:') ? peer.id : undefined)
      : undefined;
    if (!peer || !botId) {
      setError('Only locally managed Agents can be duplicated from this shell.');
      return;
    }
    try {
      await agentDirectoryController.duplicate(botId);
    } catch {
      return;
    }
  }

  async function deleteAgent(item: AgentSidebarItem, confirmDelete = true): Promise<void> {
    const peer = peerForAgent(item);
    const botId = peer?.source === 'legacy' && peer.kind === 'bot'
      ? peer.actorId ?? (peer.key.startsWith('legacy:bot:') ? peer.id : undefined)
      : undefined;
    if (!peer || !botId) {
      setError('Only locally managed Agents can be deleted from this shell.');
      return;
    }
    if (confirmDelete && !window.confirm(`Delete “${peer.title}”? The Agent store is retained for recovery.`)) return;
    agentWorkspaceController.clearPeer(peer.key);
    notifyAgentWorkspaceState();
    try {
      await agentDirectoryController.delete(botId);
    } catch {
      return;
    }
    if (peer.key === activePeerKeyRef.current) {
      activePeerKeyRef.current = null;
      setActivePeerKey(null);
      setMessages([]);
    }
  }

  async function hideAgent(item: AgentSidebarItem): Promise<void> {
    const peer = peerForAgent(item);
    const botId = peer?.source === 'legacy' && peer.kind === 'bot'
      ? peer.actorId ?? (peer.key.startsWith('legacy:bot:') ? peer.id : undefined)
      : undefined;
    if (!peer || !botId) {
      setError('Only locally managed Agents can be hidden from this shell.');
      return;
    }
    try {
      await agentDirectoryController.setHidden(botId, true);
    } catch {
      return;
    }
    if (peer.key === activePeerKeyRef.current) {
      setActivePeerKey(null);
      setMessages([]);
    }
  }

  function reorderPinnedAgents(
    moved: AgentSidebarItem,
    target: AgentSidebarItem,
    position: 'before' | 'after',
  ): void {
    agentSidebarController.reorderPinned(
      moved.key,
      target.key,
      position,
      agentItems.filter((item) => item.pinned).map((item) => item.key),
    );
  }

  function toggleAgentSelection(item: AgentSidebarItem): void {
    agentSidebarController.toggleSelection(item.key);
  }

  function rangeSelectAgent(item: AgentSidebarItem): void {
    const query = search.trim().toLocaleLowerCase();
    const orderedKeys = agentItems
      .filter((candidate) =>
        !candidate.hidden
        && (!query || `${candidate.name} ${candidate.description}`.toLocaleLowerCase().includes(query)),
      )
      .map((candidate) => candidate.key);
    agentSidebarController.rangeSelect(item.key, orderedKeys);
  }

  function clearAgentSelection(): void {
    agentSidebarController.clearSelection();
  }

  function createAgentSidebarSection(name: string, items: readonly AgentSidebarItem[]): void {
    agentSidebarController.createSection(name, items);
  }

  function moveAgentsToSection(items: readonly AgentSidebarItem[], sectionId: string): void {
    agentSidebarController.moveToSection(items, sectionId);
  }

  function moveAgentToSection(item: AgentSidebarItem, sectionId: string): void {
    agentSidebarController.moveToSection([item], sectionId);
  }

  function renameAgentSidebarSection(section: AgentSidebarSection): void {
    const name = window.prompt('Rename section', section.name)?.trim();
    if (!name || name === section.name) return;
    agentSidebarController.renameSection(section.id, name);
  }

  function deleteAgentSidebarSection(section: AgentSidebarSection): void {
    if (!window.confirm(`Delete “${section.name}”? Its Agents move to Unassigned.`)) return;
    agentSidebarController.removeSection(section.id);
  }

  async function deleteSelectedAgents(items: readonly AgentSidebarItem[]): Promise<void> {
    const deletable = items.filter((item) => !item.isGroup);
    if (!deletable.length) return;
    if (!window.confirm(`Delete ${deletable.length} selected Agent${deletable.length === 1 ? '' : 's'}? Agent stores are retained for recovery.`)) return;
    for (const item of deletable) {
      await deleteAgent(item, false);
    }
    clearAgentSelection();
  }

  function openAgent(item: AgentSidebarItem): void {
    const peer = peerForAgent(item);
    if (!peer) return;
    agentNetworkController.close();
    setSection('bots');
    void openPeer(peer);
  }

  function updateAgentComposer(peerKey: string, value: string, richText?: string) {
    agentWorkspaceController.setDraftDocument(peerKey, value, richText);
    notifyAgentWorkspaceState();
  }

  function updateComposer(value: string) {
    // Compatibility-only composer state for Messenger, Mini Apps and groups.
    setComposer(value);
    const activeKey = activePeerKeyRef.current;
    if (!activeKey) return;
    if (activeKey.startsWith('selfhosted:')) {
      const conversationId = activeKey.slice('selfhosted:'.length);
      if (typingStopTimerRef.current) window.clearTimeout(typingStopTimerRef.current);
      if (value.trim()) {
        void selfHosted.startTyping(conversationId).catch(() => {});
        typingStopTimerRef.current = window.setTimeout(() => {
          void selfHosted.stopTyping(conversationId).catch(() => {});
        }, 3_000);
      } else {
        void selfHosted.stopTyping(conversationId).catch(() => {});
      }
    }
    setDrafts((current) => {
      const next = { ...current };
      if (value) next[activeKey] = value;
      else delete next[activeKey];
      return next;
    });
  }

  function handleMessageAreaScroll(event: React.UIEvent<HTMLDivElement>) {
    const node = event.currentTarget;
    const distanceFromLatest = node.scrollHeight - node.scrollTop - node.clientHeight;
    const atLatest = distanceFromLatest < 96;
    stickToLatestRef.current = atLatest;
    setShowScrollToLatest(!atLatest);
  }

  function scrollToLatest() {
    const node = messageAreaRef.current;
    if (!node) return;
    stickToLatestRef.current = true;
    setShowScrollToLatest(false);
    node.scrollTo({ top: node.scrollHeight, behavior: 'smooth' });
  }

  function scrollToTranscriptEntry(entryId: string) {
    const root = messageAreaRef.current;
    if (!root) return;
    const target = Array.from(root.querySelectorAll<HTMLElement>('[data-transcript-entry-id]'))
      .find((element) => element.dataset.transcriptEntryId === entryId);
    target?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }

  async function copyBotMessage(message: BotTranscriptMessage): Promise<void> {
    try {
      await navigator.clipboard?.writeText(message.text);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : '复制消息失败');
    }
  }

  function clearAgentReply(peerKey: string): void {
    agentWorkspaceController.clearReply(peerKey);
    notifyAgentWorkspaceState();
  }

  function setReplyTarget(message: DisplayMessage): void {
    const peer = activePeer;
    if (peer && isAgentPeer(peer) && !peer.miniAppId) {
      agentWorkspaceController.setReply(peer.key, {
        id: message.id,
        role: message.role,
        text: message.text,
      });
      notifyAgentWorkspaceState();
      return;
    }
    setLegacyReplyTo(message);
  }

  function editBotMessage(message: BotTranscriptMessage | TranscriptEntry) {
    if (activePeer && isAgentPeer(activePeer) && !activePeer.miniAppId) {
      agentWorkspaceController.setDraftDocument(activePeer.key, message.text, 'richText' in message ? message.richText : undefined);
      notifyAgentWorkspaceState();
      clearAgentReply(activePeer.key);
      return;
    }
    updateComposer(message.text);
  }

  function enqueueAgentPrompt(
    peer: PeerItem,
    text: string,
    existingMessageId?: string,
    attachments: readonly AttachmentContext[] = [],
    replyContext?: AgentReplyContext,
    references: readonly AgentPromptReference[] = [],
    richText?: string,
  ) {    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    submitAgentWorkspace({
      ...(existingMessageId ? { messageId: existingMessageId } : {}),
      peerKey: peer.key,
      agentId,
      ...(peer.conversationId ? { conversationId: peer.conversationId } : {}),
      prompt: text,
      ...(richText ? { richText } : {}),
      ...(attachments.length ? { attachments: [...attachments] } : {}),
      ...(replyContext ? { replyTo: replyContext } : {}),
      ...(references.length ? { references: [...references] } : {}),
    });
  }

  function sendAgentMessage(event: FormEvent, peer: PeerItem): void {
    event.preventDefault();
    if (!isAgentPeer(peer) || peer.miniAppId || peer.kind === 'group') return;
    const text = agentWorkspaceController.draftForPeer(peer.key).trim();
    const attachments = agentWorkspaceController.attachmentsForPeer(peer.key);
    if (!text && !attachments.length) return;

    // Take the complete Agent-owned draft atomically. A failed/queued send is
    // restored by useAgentWorkspaceRuntime without overwriting a newer draft.
    const submittedDraft = agentWorkspaceController.takeDraft(peer.key);
    notifyAgentWorkspaceState();
    enqueueAgentPrompt(
      peer,
      submittedDraft.text.trim(),
      undefined,
      submittedDraft.attachments,
      submittedDraft.replyTo,
      submittedDraft.references ?? [],
      submittedDraft.richText,
    );
    setScheduledAtMs(undefined);
  }

  function regenerateBotMessage(message: BotTranscriptMessage | TranscriptEntry) {
    if (!activePeer || !isAgentPeer(activePeer) || activePeer.miniAppId) return;
    const prompt = agentTranscriptStore.userPromptBefore(activePeer.key, message.id);
    if (!prompt) {
      setError('找不到这条回复对应的用户消息。');
      return;
    }
    agentTranscriptStore.removeByIds(activePeer.key, [message.id]);
    notifyAgentWorkspaceState();
    enqueueAgentPrompt(activePeer, prompt.text, undefined, prompt.attachments ?? [], undefined, [], prompt.richText);
  }

  async function stopAgentOperation(): Promise<void> {
    try {
      await interruptAgentWorkspace(activePeerKeyRef.current);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  useEffect(() => {
    if (!activePeer || !isAgentPeer(activePeer) || !stickToLatestRef.current) return;
    const frame = window.requestAnimationFrame(() => {
      const node = messageAreaRef.current;
      if (node) node.scrollTop = node.scrollHeight;
    });
    return () => window.cancelAnimationFrame(frame);
  }, [activePeerKey, messages, agentWorkspaceRevision]);

  async function openPeer(peer: PeerItem) {
    activePeerKeyRef.current = peer.key;
    setActivePeerKey(peer.key);
    if (!isAgentPeer(peer)) setLegacySendPending(false);
    stickToLatestRef.current = true;
    setShowScrollToLatest(false);
    const isWorkspaceAgent = peer.source === 'legacy'
      && peer.kind !== 'group'
      && !peer.miniAppId
      && isAgentPeer(peer);
    if (!isWorkspaceAgent) setComposer(drafts[peer.key] ?? '');
    setSearch('');
    setMessageRenderCount(initialMessageRenderCount);
    setLegacyReplyTo(null);
    setError(null);
    setConversationSearchOpen(false);
    setAgentConversationSearch('');
    if (peer.miniAppId) {
      setMessages(miniAppBotThreadsRef.current[peer.miniAppId] ?? []);
      void loadMiniAppBotThread(peer.miniAppId).catch(() => {});
      return;
    }
    if (peer.source === 'selfhosted' && peer.conversationId) {
      showSelfConversation(peer.conversationId);
      const list = selfMessages[peer.conversationId] ?? [];
      const last = list.filter((message) => !message.deleted).at(-1);
      if (last) void selfHosted.markRead(peer.conversationId, last.id).catch(() => {});
      return;
    }
    if (isAgentPeer(peer)) {
      if (!agentTranscriptStore.has(peer.key) && peer.conversationId) {
        const cached = cachedLegacyDisplayMessages(peer.conversationId);
        if (cached.length) {
          agentTranscriptStore.replace(peer.key, toAgentTranscriptSources(cached));
        } else if (!(await restoreAgentConversationFromCloud(peer))) {
          agentTranscriptStore.replace(peer.key, []);
        }
      } else if (!agentTranscriptStore.has(peer.key)) {
        agentTranscriptStore.replace(peer.key, []);
      }
      notifyAgentWorkspaceState();
      if (peer.conversationId) {
        await openAgentConversation(peer.key, peer.conversationId).catch(() => {});
      }
      void agentWorkflowController.list(peer.agentId ?? peer.actorId ?? peer.id).catch(() => {});
      return;
    }
    if (peer.kind === 'group' && peer.groupId) {
      const group = groups.find((item) => item.id === peer.groupId);
      setMessages(group?.messages.map((message) => ({
        id: message.id,
        source: 'legacy',
        role: message.speaker.kind === 'user' ? 'me' : 'peer',
        text: message.content,
        createdAtMs: message.createdAtMs,
      })) ?? []);
      return;
    }
    if (peer.conversationId) {
      setMessages(cachedLegacyDisplayMessages(peer.conversationId));
      await agentCoordinatorClient.openConversation(nextRequestId('conversation-open'), peer.conversationId).catch((cause: unknown) => {
          setError(cause instanceof Error ? cause.message : String(cause));
        });
      return;
    }
    setMessages([]);
  }

  async function sendMessage(event: FormEvent) {
    event.preventDefault();
    if (!activePeer) return;
    const agentRequest = activePeer.source === 'legacy'
      && activePeer.kind !== 'group'
      && !activePeer.miniAppId
      && isAgentPeer(activePeer);
    if (agentRequest) {
      sendAgentMessage(event, activePeer);
      return;
    }
    const text = composer.trim();
    if (!text || legacySendPending) return;
    setLegacySendPending(true);
    updateComposer('');
    try {
      if (activePeer.miniAppId) {
        const createdAtMs = Date.now();
        const userMessage: DisplayMessage = {
          id: nextRequestId('miniapp-bot-user'),
          source: 'legacy',
          role: 'me',
          text,
          createdAtMs,
        };
        const pendingThread = [...(miniAppBotThreadsRef.current[activePeer.miniAppId] ?? []), userMessage];
        const thinkingMessage: DisplayMessage = {
          id: userMessage.id + ':thinking',
          source: 'legacy',
          role: 'peer',
          text: '',
          createdAtMs: Date.now(),
          kind: 'thinking',
          operationId: userMessage.id,
          actionTitle: '正在处理',
          actionDetail: '正在调用 Mini App…',
          actionStatus: 'running',
        };
        const pendingWithThinking = [...pendingThread, thinkingMessage];
        miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [activePeer.miniAppId]: pendingWithThinking };
        setMessages(pendingWithThinking);
        await appendMiniAppBotMessages(activePeer.miniAppId, [{
          messageId: userMessage.id,
          role: 'user',
          text: userMessage.text,
          createdAt: new Date(userMessage.createdAtMs).toISOString(),
        }]);
        const routed = await invokeNativeDesktop<Record<string, unknown>>('routeMiniAppInput', {
          pluginId: activePeer.miniAppId,
          input: text,
        });
        const executed = await executeDesktopMiniAppBotInput(activePeer.miniAppId, routed, text);
        const responseMessage: DisplayMessage = {
          id: nextRequestId('miniapp-bot-response'),
          source: 'legacy',
          role: 'peer',
          text: miniAppBotResponseText(executed),
          miniAppId: activePeer.miniAppId,
          createdAtMs: Date.now(),
        };
        const completedThread = [...pendingThread, responseMessage];
        miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [activePeer.miniAppId]: completedThread };
        if (activePeerKeyRef.current === activePeer.key) setMessages(completedThread);
        await appendMiniAppBotMessages(activePeer.miniAppId, [{
          messageId: responseMessage.id,
          role: 'assistant',
          text: responseMessage.text,
          payload: { miniAppId: activePeer.miniAppId },
          createdAt: new Date(responseMessage.createdAtMs).toISOString(),
        }]);
      } else if (activePeer.source === 'selfhosted' && activePeer.conversationId) {
        await selfHosted.sendText(activePeer.conversationId, text, {
          replyToMessageId: legacyReplyTo?.id,
          scheduledAtMs,
          silent: silentSend,
        });
      } else if (activePeer.kind === 'group' && activePeer.groupId) {
        await execute({ type: 'group.send', requestId: nextRequestId('group-send'), id: activePeer.groupId, text });
      } else {
        const requestId = nextRequestId('chat-send');
        const optimisticId = `optimistic:${requestId}`;
        setMessages((current) => [...current, {
          id: optimisticId,
          source: 'legacy',
          role: 'me',
          text,
          createdAtMs: Date.now(),
          kind: 'message',
          operationId: requestId,
          optimistic: true,
        }]);
        const accepted = await execute({
          type: 'chat.send',
          requestId,
          text,
          conversationId: activePeer.conversationId,
          agentId: activePeer.actorId,
        });
        if (!accepted) {
          setMessages((current) => current.filter((message) => message.id !== optimisticId));
          updateComposer(text);
          return;
        }
        if (accepted.operationId) {
          setMessages((current) => current.map((message) => message.id === optimisticId
            ? { ...message, operationId: accepted.operationId, optimistic: false, queued: false }
            : message));
        }
      }
      setLegacyReplyTo(null);
      setScheduledAtMs(undefined);
    } catch (cause) {
      if (activePeer.miniAppId) {
        const thread = (miniAppBotThreadsRef.current[activePeer.miniAppId] ?? [])
          .filter((message) => !(message.kind === 'thinking' && message.operationId));
        miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [activePeer.miniAppId]: thread };
        if (activePeerKeyRef.current === activePeer.key) setMessages(thread);
      }
      updateComposer(text);
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLegacySendPending(false);
    }
  }

  async function saveNewDialog() {
    if (!newDialog || !newDialog.name.trim()) return;
    try {
      if (newDialog.type === 'channel') {
        const conversation = await selfHosted.createChannel(newDialog.name, newDialog.description);
        setSelfConversations((current) => upsertById(current, conversation));
        setNewDialog(null);
        setSection('channels');
        await openPeer({
          key: `selfhosted:${conversation.id}`,
          id: conversation.id,
          source: 'selfhosted',
          conversationId: conversation.id,
          kind: 'channel',
          title: conversation.title,
          subtitle: conversation.description || '频道',
          unread: 0,
          pinned: false,
          archived: false,
          updatedAtMs: conversation.updatedAtMs,
        });
        return;
      }
      await execute({
        type: 'group.create',
        requestId: nextRequestId('group-create'),
        name: newDialog.name.trim(),
        description: '',
        memberIds: [...newDialog.selectedBotIds],
      });
      await execute({ type: 'group.list', requestId: nextRequestId('group-list-after-create') });
      setNewDialog(null);
      setSection('groups');
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function ensureSavedMessages() {
    try {
      const existing = selfConversations.find((conversation) => conversation.kind === 'savedMessages');
      const conversation = existing ?? await selfHosted.ensureSavedMessages();
      if (!existing) setSelfConversations((current) => upsertById(current, conversation));
      setSection('saved');
      await openPeer({
        key: `selfhosted:${conversation.id}`,
        id: conversation.id,
        source: 'selfhosted',
        conversationId: conversation.id,
        kind: 'saved',
        title: conversation.title,
        subtitle: '收藏消息',
        unread: conversation.unreadCount,
        pinned: conversation.pinned,
        archived: conversation.archived,
        updatedAtMs: conversation.updatedAtMs,
      });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }


  async function openStory(story: MessagingStory) {
    setActiveStory(story);
    if (story.ownerId !== selfHosted.actorId) {
      void selfHosted.viewStory(story.id).catch(() => {});
    }
  }

  async function reactToActiveStory(reaction: string) {
    if (!activeStory) return;
    try {
      await selfHosted.reactStory(activeStory.id, reaction);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function transcribeAgentVoice(peer: PeerItem, file: File): Promise<string> {
    if (!isAgentPeer(peer) || peer.miniAppId) throw new Error('Voice dictation is only available for Agents.');
    const validationError = validateAgentAttachment(file);
    if (validationError) throw new Error(validationError);
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    const stored = await uploadAgentAttachment(peer.key, {
      agentId,
      filename: file.name,
      mimeType: file.type || 'audio/webm',
      bytesBase64: await agentFileToBase64(file),
    });
    if (!stored.path) throw new Error('Voice recording was stored without a local path.');
    const result = await invokeNativeDesktop<Record<string, unknown>>('transcribeAudio', {
      path: stored.path,
      language: 'auto',
    });
    const nested = result?.result && typeof result.result === 'object'
      ? result.result as Record<string, unknown>
      : null;
    const text = typeof result?.text === 'string'
      ? result.text
      : typeof result?.transcript === 'string'
        ? result.transcript
        : typeof nested?.text === 'string'
          ? nested.text
          : '';
    if (text.trim()) return text.trim();
    if (result?.available === false) {
      throw new Error('Voice transcription is unavailable. Install the offline ASR model or enable a runtime transcription tool.');
    }
    throw new Error('Voice transcription returned no text.');
  }

  async function stageAgentFiles(peer: PeerItem, files: readonly File[]): Promise<void> {
    if (!isAgentPeer(peer) || peer.miniAppId || files.length === 0) return;
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    const controller = agentWorkspaceController;
    const alreadyStaged = controller.attachmentsForPeer(peer.key);
    const availableSlots = Math.max(0, AGENT_ATTACHMENT_LIMIT - alreadyStaged.length);
    if (availableSlots === 0) {
      setError(`You can attach up to ${AGENT_ATTACHMENT_LIMIT} files to one Agent draft.`);
      return;
    }

    controller.setUploading(peer.key, true);
    notifyAgentWorkspaceState();
    const staged: AttachmentContext[] = [];
    try {
      for (const file of files.slice(0, availableSlots)) {
        const validationError = validateAgentAttachment(file);
        if (validationError) {
          setError(validationError);
          continue;
        }
        try {
          const stored = await uploadAgentAttachment(peer.key, {
            agentId,
            filename: file.name,
            mimeType: file.type || undefined,
            bytesBase64: await agentFileToBase64(file),
          });
          staged.push(await enrichAgentAttachmentPreview(stored, file));
        } catch (cause) {
          setError(cause instanceof Error ? cause.message : String(cause));
        }
      }
      if (staged.length) {
        controller.appendAttachments(peer.key, staged);
        notifyAgentWorkspaceState();
        const nextAttachments = controller.attachmentsForPeer(peer.key);
        void mirrorAgentCloudSnapshot(agentId, FABU_AGENT_ATTACHMENT_INDEX_PATH, {
          schemaVersion: 1,
          agentId,
          attachments: nextAttachments.map((attachment) => ({
            id: attachment.id,
            name: attachment.name,
            path: attachment.path,
            mimeType: attachment.mimeType,
            sizeBytes: attachment.sizeBytes,
          })),
          updatedAtMs: Date.now(),
        });
      }
    } finally {
      controller.setUploading(peer.key, false);
      notifyAgentWorkspaceState();
    }
  }

  function removeAgentAttachment(peerKey: string, attachmentId: string): void {
    const controller = agentWorkspaceController;
    controller.removeAttachment(peerKey, attachmentId);
    notifyAgentWorkspaceState();
    const remaining = controller.attachmentsForPeer(peerKey);
    const peer = agentPeerForRuntimeKey(peerKey);
    if (!peer) return;
    const agentId = peer.agentId ?? peer.actorId ?? peer.id;
    void mirrorAgentCloudSnapshot(agentId, FABU_AGENT_ATTACHMENT_INDEX_PATH, {
      schemaVersion: 1,
      agentId,
      attachments: remaining.map((attachment) => ({
        id: attachment.id,
        name: attachment.name,
        path: attachment.path,
        mimeType: attachment.mimeType,
        sizeBytes: attachment.sizeBytes,
      })),
      updatedAtMs: Date.now(),
    });
  }

  async function sendAttachmentFile(file: File) {
    if (!activePeer?.conversationId || activePeer.source !== 'selfhosted') {
      setError('附件需要发送到 Fabushi 自建会话。');
      return;
    }
    setAttachmentMenuOpen(false);
    setLegacySendPending(true);
    setAttachmentProgress(`正在上传 ${file.name}…`);
    try {
      await selfHosted.sendAttachment(
        activePeer.conversationId,
        file,
        { replyToMessageId: legacyReplyTo?.id, scheduledAtMs, silent: silentSend },
        (uploaded, total) => setAttachmentProgress(`正在上传 ${file.name} · ${Math.round((uploaded / total) * 100)}%`),
      );
      setLegacyReplyTo(null);
      setScheduledAtMs(undefined);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLegacySendPending(false);
      setAttachmentProgress(null);
    }
  }

  async function sendPoll() {
    if (!activePeer?.conversationId || activePeer.source !== 'selfhosted') {
      setError('投票使用 Fabushi 自研消息协议；请先打开自建群组或频道。');
      return;
    }
    const question = window.prompt('投票问题');
    if (!question) return;
    const raw = window.prompt('选项（用 | 分隔，至少两个）', '选项一 | 选项二');
    if (!raw) return;
    try {
      await selfHosted.sendPoll(activePeer.conversationId, question, raw.split('|'));
      setAttachmentMenuOpen(false);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function sendLocation() {
    if (!activePeer?.conversationId || activePeer.source !== 'selfhosted') {
      setError('位置消息需要自建会话。');
      return;
    }
    if (!navigator.geolocation) {
      setError('当前平台没有提供定位能力。');
      return;
    }
    navigator.geolocation.getCurrentPosition(
      (position) => void selfHosted.sendContent(activePeer.conversationId!, {
        type: 'location',
        data: { latitude: position.coords.latitude, longitude: position.coords.longitude },
      }).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause))),
      (positionError) => setError(positionError.message),
    );
    setAttachmentMenuOpen(false);
  }

  function createInvoiceForActivePeer() {
  if (!activePeer?.conversationId || activePeer.source !== 'selfhosted') {
    setError('账单需要发送到 Fabushi 自建会话。');
    return;
  }
  setInvoiceDialog({ conversationId: activePeer.conversationId, title: '订单', amount: '9.99' });
}

async function saveInvoiceDialog() {
  if (!invoiceDialog) return;
  const title = invoiceDialog.title.trim();
  const amount = Number(invoiceDialog.amount);
  if (!title) {
    setError('账单名称不能为空。');
    return;
  }
  if (!Number.isFinite(amount) || amount <= 0) {
    setError('金额必须大于 0。');
    return;
  }
  try {
    await selfHosted.createInvoice({
      conversationId: invoiceDialog.conversationId,
      title,
      currency: 'USD',
      amountMinor: Math.round(amount * 100),
      providerId: 'fabushi-pay',
    });
    setInvoiceDialog(null);
  } catch (cause) {
    setError(cause instanceof Error ? cause.message : String(cause));
  }
}

  async function handleMessageAction(action: 'copy' | 'reply' | 'forward' | 'checkout' | 'edit' | 'delete' | 'react' | 'pin') {
    const target = messageMenu?.message;
    if (!target || !activePeer) return;
    setMessageMenu(null);
    if (action === 'copy') {
      await navigator.clipboard.writeText(target.text);
      return;
    }
    if (action === 'reply') {
      setReplyTarget(target);
      return;
    }
    if (action === 'forward') {
      if (activePeer.source !== 'selfhosted' || !activePeer.conversationId || target.source !== 'selfhosted') {
        setError('转发需要源消息已迁移到 Fabushi 自建协议。');
        return;
      }
      setForwardDialog({ sourceConversationId: activePeer.conversationId, message: target });
      return;
    }
    if (action === 'checkout') {
      if (target.source !== 'selfhosted' || !target.invoiceId) {
        setError('该账单尚未迁移到 Fabushi Pay。');
        return;
      }
      const invoice = selfInvoices.find((item) => item.id === target.invoiceId);
      if (invoice?.sellerId === selfHosted.actorId) {
        setError('不能支付自己创建的账单。');
        return;
      }
      try {
        await selfHosted.checkoutInvoice(target.invoiceId);
        await selfHosted.requestWalletStatus();
        setSection('payments');
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
      return;
    }
    if (activePeer.source !== 'selfhosted' || !activePeer.conversationId || target.source !== 'selfhosted') {
      setError('该消息来自旧会话适配器；修改类操作正在统一迁移到 Fabushi 自建协议。');
      return;
    }
    try {
      if (action === 'delete') await selfHosted.deleteMessages(activePeer.conversationId, [target.id]);
      if (action === 'react') await selfHosted.setReaction(activePeer.conversationId, target.id, '👍');
      if (action === 'pin') await selfHosted.pinMessage(activePeer.conversationId, target.id, !target.pinned);
      if (action === 'edit') {
        setEditDialog({
          conversationId: activePeer.conversationId,
          messageId: target.id,
          originalText: target.text,
          text: target.text,
        });
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function saveEditedMessage() {
    if (!editDialog) return;
    const text = editDialog.text.trim();
    if (!text || text === editDialog.originalText) {
      setEditDialog(null);
      return;
    }
    try {
      await selfHosted.editText(editDialog.conversationId, editDialog.messageId, text);
      setEditDialog(null);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function forwardToPeer(peer: PeerItem) {
    if (!forwardDialog || peer.source !== 'selfhosted' || !peer.conversationId) return;
    try {
      await selfHosted.forwardMessage(
        forwardDialog.sourceConversationId,
        forwardDialog.message.id,
        peer.conversationId,
      );
      setForwardDialog(null);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function refundOrder(orderId: string) {
    try {
      await selfHosted.refundOrder(orderId);
      await selfHosted.requestWalletStatus();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  function syncLocalCallMedia() {
    const stream = webRtcRef.current?.localMedia() ?? null;
    window.setTimeout(() => {
      if (localVideoRef.current) {
        localVideoRef.current.srcObject = stream;
        if (stream) void localVideoRef.current.play().catch(() => {});
      }
    }, 0);
  }

  async function openCommunityAdmin(peer: PeerItem) {
    if (peer.source !== 'selfhosted' || !peer.conversationId || !['group', 'channel'].includes(peer.kind)) {
      setError('群组/频道管理只对 Fabushi 自建社区开放。');
      return;
    }
    const existing = selfCommunities.find((community) => community.conversationId === peer.conversationId);
    if (!existing) {
      const created = defaultCommunityState(peer.conversationId, selfHosted.actorId);
      try {
        await selfHosted.updateCommunity(created);
        setSelfCommunities((current) => [...current.filter((item) => item.conversationId !== created.conversationId), created]);
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : String(cause));
        return;
      }
    }
    setCommunityDialogPeer(peer);
  }

  async function saveCommunity(community: MessagingCommunityState) {
    try {
      await selfHosted.updateCommunity(community);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function appendMiniAppCallExchange(miniAppId: string, input: string, visibleText = input): Promise<void> {
    const createdAtMs = Date.now();
    const userMessage: DisplayMessage = {
      id: nextRequestId('miniapp-call-user'),
      source: 'legacy',
      role: 'me',
      text: visibleText,
      createdAtMs,
    };
    const pendingThread = [...(miniAppBotThreadsRef.current[miniAppId] ?? []), userMessage];
    miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [miniAppId]: pendingThread };
    const active = peersRef.current.find((peer) => peer.key === activePeerKeyRef.current);
    if (active?.miniAppId === miniAppId) setMessages(pendingThread);
    await appendMiniAppBotMessages(miniAppId, [{
      messageId: userMessage.id,
      role: 'user',
      text: userMessage.text,
      createdAt: new Date(userMessage.createdAtMs).toISOString(),
      payload: { source: 'miniapp-call' },
    }]);
    const routed = await invokeNativeDesktop<Record<string, unknown>>('routeMiniAppInput', {
      pluginId: miniAppId,
      input,
    });
    const executed = await executeDesktopMiniAppBotInput(miniAppId, routed, input);
    const responseMessage: DisplayMessage = {
      id: nextRequestId('miniapp-call-response'),
      source: 'legacy',
      role: 'peer',
      text: miniAppBotResponseText(executed),
      createdAtMs: Date.now(),
    };
    const completedThread = [...pendingThread, responseMessage];
    miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [miniAppId]: completedThread };
    if (active?.miniAppId === miniAppId) setMessages(completedThread);
    await appendMiniAppBotMessages(miniAppId, [{
      messageId: responseMessage.id,
      role: 'assistant',
      text: responseMessage.text,
      createdAt: new Date(responseMessage.createdAtMs).toISOString(),
      payload: { source: 'miniapp-call' },
    }]);
  }

  async function runMiniAppCallCommand(session: MiniAppCallSession, command: string, args?: Record<string, unknown>): Promise<void> {
    const suffix = args && Object.keys(args).length ? ' ' + JSON.stringify(args) : '';
    const input = '/' + session.miniAppId + ':' + command + suffix;
    await appendMiniAppCallExchange(session.miniAppId, input, '通话服务 · /' + command);
  }

  async function saveMiniAppCallRecording(session: MiniAppCallSession, blob: Blob, mimeType: string): Promise<void> {
    const extension = mimeType.includes('mp4') ? 'mp4' : 'webm';
    const fileName = 'teleprompter-' + new Date().toISOString().replaceAll(':', '-') + '.' + extension;
    const file = new File([blob], fileName, { type: mimeType || 'video/webm' });
    setAttachmentProgress('正在保存 ' + fileName + '…');
    try {
      const media = await selfHosted.uploadBlob(file, (uploaded, total) => {
        setAttachmentProgress('正在保存 ' + fileName + ' · ' + Math.round((uploaded / Math.max(total, 1)) * 100) + '%');
      });
      const message: DisplayMessage = {
        id: nextRequestId('miniapp-call-video'),
        source: 'legacy',
        role: 'me',
        text: '口播录制视频',
        createdAtMs: Date.now(),
        media,
        mediaType: 'video',
      };
      const thread = [...(miniAppBotThreadsRef.current[session.miniAppId] ?? []), message];
      miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [session.miniAppId]: thread };
      const active = peersRef.current.find((peer) => peer.key === activePeerKeyRef.current);
      if (active?.miniAppId === session.miniAppId) setMessages(thread);
      await appendMiniAppBotMessages(session.miniAppId, [{
        messageId: message.id,
        role: 'user',
        text: message.text,
        createdAt: new Date(message.createdAtMs).toISOString(),
        payload: {
          source: 'miniapp-call-recording',
          miniAppId: session.miniAppId,
          callId: session.callId,
          mediaType: 'video',
          media,
        },
      }]);
    } finally {
      setAttachmentProgress(null);
    }
  }

  async function startCall(kind: 'voice' | 'video') {
    if (!activePeer) return;
    if (activePeer.miniAppId) {
      const program = activePeer.miniAppCalls?.[kind];
      if (!program) {
        setError('这个 Mini App 没有声明' + (kind === 'video' ? '视频' : '语音') + '通话程序。');
        return;
      }
      setError(null);
      try {
        let html: string | undefined;
        if (program.type === 'miniapp-surface') {
          const installed = installedMiniApps[activePeer.miniAppId] ?? await transport.pluginActive(activePeer.miniAppId);
          if (!installed) throw new Error('请先安装这个 Mini App，再使用它自定义的通话界面。');
          const document = await transport.pluginUiDocument(activePeer.miniAppId);
          html = prepareDesktopMiniAppWebMcpDocument(activePeer.miniAppId, document.html);
        }
        setMiniAppCall({
          callId: 'miniapp-call:' + crypto.randomUUID(),
          miniAppId: activePeer.miniAppId,
          title: activePeer.title,
          kind,
          program,
          html,
        });
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
      return;
    }
    if (activePeer.source !== 'selfhosted' || !activePeer.conversationId || !activePeer.actorId) {
      setError('远端通话只对已迁移到 Fabushi 自建协议的一对一联系人开放。');
      return;
    }
    const controller = webRtcRef.current;
    if (!controller) {
      setError('Fabushi WebRTC 控制器尚未准备好。');
      return;
    }
    setLocalCall({
      kind,
      title: activePeer.title,
      status: 'ringing',
      incoming: false,
      muted: false,
      videoEnabled: kind === 'video',
    });
    try {
      await controller.start({
        conversationId: activePeer.conversationId,
        peerActorId: activePeer.actorId,
        kind,
      });
    } catch (cause) {
      setLocalCall({
        kind,
        title: activePeer.title,
        status: 'failed',
        incoming: false,
        muted: false,
        videoEnabled: kind === 'video',
        error: cause instanceof Error ? cause.message : String(cause),
      });
    }
  }

  async function acceptIncomingCall() {
    const controller = webRtcRef.current;
    if (!controller || !incomingCall) return;
    try {
      const accepted = await controller.acceptIncoming();
      setIncomingCall(null);
      setLocalCall((current) => ({
        kind: accepted.kind,
        title: current?.title ?? accepted.peerActorId,
        status: 'connecting',
        incoming: true,
        muted: false,
        videoEnabled: accepted.kind === 'video',
      }));
      syncLocalCallMedia();
    } catch (cause) {
      setLocalCall((current) => current ? {
        ...current,
        status: 'failed',
        error: cause instanceof Error ? cause.message : String(cause),
      } : null);
    }
  }

  async function declineIncomingCall() {
    try {
      await webRtcRef.current?.declineIncoming();
    } finally {
      setIncomingCall(null);
      setLocalCall(null);
    }
  }

  async function endCall() {
    await webRtcRef.current?.hangup();
    setIncomingCall(null);
    setLocalCall(null);
  }

  async function toggleCallMute() {
    if (!localCall) return;
    const next = !localCall.muted;
    await webRtcRef.current?.setMuted(next);
    setLocalCall((current) => current ? { ...current, muted: next } : null);
  }

  async function toggleCallVideo() {
    if (!localCall || localCall.kind !== 'video') return;
    const next = !localCall.videoEnabled;
    await webRtcRef.current?.setVideoEnabled(next);
    setLocalCall((current) => current ? { ...current, videoEnabled: next } : null);
  }

  async function shareCallScreen() {
    try {
      await webRtcRef.current?.startScreenShare();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  function toggleLocalSet(setter: React.Dispatch<React.SetStateAction<Set<string>>>, key: string) {
    setter((current) => {
      const next = new Set(current);
      if (next.has(key)) next.delete(key); else next.add(key);
      return next;
    });
  }

  async function togglePinConversation(peer: PeerItem) {
    if (peer.source === 'selfhosted' && peer.conversationId) {
      await selfHosted.pinConversation(peer.conversationId, !peer.pinned);
      return;
    }
    toggleLocalSet(setPinnedPeerKeys, peer.key);
  }

  async function toggleArchiveConversation(peer: PeerItem) {
    if (peer.source === 'selfhosted' && peer.conversationId) {
      await selfHosted.archiveConversation(peer.conversationId, !peer.archived);
      return;
    }
    toggleLocalSet(setArchivedPeerKeys, peer.key);
  }

  async function toggleMuteConversation(peer: PeerItem) {
    if (peer.source === 'selfhosted' && peer.conversationId) {
      const muted = mutedPeerKeys.has(peer.key);
      await selfHosted.setConversationNotifications(peer.conversationId, muted ? undefined : Date.now() + 365 * 24 * 60 * 60 * 1000);
    }
    toggleLocalSet(setMutedPeerKeys, peer.key);
  }

  async function loadMiniAppBotThread(miniAppId: string): Promise<DisplayMessage[]> {
    const beforeRead = miniAppBotThreadsRef.current[miniAppId];
    const page = await readMiniAppBotMessages(miniAppId, '', 500);
    if (miniAppBotThreadsRef.current[miniAppId] !== beforeRead) return miniAppBotThreadsRef.current[miniAppId] ?? [];
    if (beforeRead?.some((message) => message.kind === 'thinking' || message.streaming)) return beforeRead;
    const thread = (page.messages ?? []).map((message): DisplayMessage => {
      const mediaTypeValue = message.payload?.mediaType;
      const mediaType = mediaTypeValue === 'photo' || mediaTypeValue === 'video' || mediaTypeValue === 'document'
        ? mediaTypeValue
        : undefined;
      const mediaValue = message.payload?.media;
      const media = mediaValue && typeof mediaValue === 'object' && !Array.isArray(mediaValue)
        ? mediaValue as unknown as MessagingMediaRef
        : undefined;
      return {
        id: message.messageId,
        role: message.role === 'user' ? 'me' : 'peer',
        text: message.text,
        createdAtMs: Number.isFinite(Date.parse(message.createdAt)) ? Date.parse(message.createdAt) : Date.now(),
        source: 'legacy',
        media,
        mediaType,
        miniAppId: message.role !== 'user' && message.payload?.miniAppId === miniAppId ? miniAppId : undefined,
      };
    });
    miniAppBotThreadsRef.current = { ...miniAppBotThreadsRef.current, [miniAppId]: thread };
    const active = peersRef.current.find((peer) => peer.key === activePeerKeyRef.current);
    if (active?.miniAppId === miniAppId) setMessages(thread);
    return thread;
  }

  async function synchronizeAccountState(): Promise<void> {
    if (accountSyncInFlightRef.current) return;
    accountSyncInFlightRef.current = true;
    try {
      let cursor = accountSyncCursorRef.current;
      let reconcileApps = false;
      let refreshBots = false;
      const changedMiniAppThreads = new Set<string>();
      for (let pageIndex = 0; pageIndex < 16; pageIndex += 1) {
        const envelope = await readAccountSync(cursor, 200);
        cursor = envelope.cursor;
        if (envelope.mode === 'snapshot') {
          reconcileApps = true;
          refreshBots = true;
        }
        for (const event of envelope.events ?? []) {
          if (event.type.startsWith('miniapp.') && !event.type.startsWith('miniapp.bot.message') && !event.type.startsWith('miniapp.content.')) {
            reconcileApps = true;
          }
          if (event.type === 'bot.added' || event.type === 'bot.updated' || event.type === 'bot.removed') refreshBots = true;
          if (event.type === 'miniapp.bot.message') {
            const miniAppId = typeof event.payload?.miniAppId === 'string' ? event.payload.miniAppId : '';
            if (miniAppId) changedMiniAppThreads.add(miniAppId);
          }
        }
        if (!envelope.hasMore) break;
      }
      accountSyncCursorRef.current = cursor;
      persistAccountSyncCursor(cursor);
      if (reconcileApps) {
        await reconcileAccountMiniApps();
        await refreshMiniApps(miniAppQuery, false);
      }
      if (refreshBots || accountBots.length === 0) {
        setAccountBots(await readAccountBots());
      }
      const active = peersRef.current.find((peer) => peer.key === activePeerKeyRef.current);
      if (active?.miniAppId && changedMiniAppThreads.has(active.miniAppId)) {
        await loadMiniAppBotThread(active.miniAppId);
      }
    } catch {
      // Offline/account bootstrap failures must not block the local Messenger.
      // The next periodic tick will retry from the last durable cursor.
    } finally {
      accountSyncInFlightRef.current = false;
    }
  }

  async function refreshMiniApps(query = miniAppQuery, reconcileAccount = true) {
    setMiniAppLoading(true);
    try {
      if (reconcileAccount) await reconcileAccountMiniApps().catch(() => undefined);
      const catalogPromise = transport.marketplaceBrowse(query);
      const identityCatalogPromise = query.trim() ? transport.marketplaceBrowse('') : catalogPromise;
      const accountCatalogPromise = readAccountMiniApps();
      const [catalogResult, identityCatalogResult, accountCatalogResult, installedResult] = await Promise.allSettled([
        catalogPromise,
        identityCatalogPromise,
        accountCatalogPromise,
        transport.pluginListInstalled(),
      ]);
      if (catalogResult.status === 'fulfilled') {
        setMarketplaceApps(catalogResult.value.plugins);
      } else {
        setError(catalogResult.reason instanceof Error ? catalogResult.reason.message : String(catalogResult.reason));
      }
      const discoveredIdentityCatalog = identityCatalogResult.status === 'fulfilled'
        ? identityCatalogResult.value.plugins
        : [];
      const accountIdentityCatalog = accountCatalogResult.status === 'fulfilled'
        ? accountMiniAppsAsMarketplaceSummaries(accountCatalogResult.value)
        : [];
      if (discoveredIdentityCatalog.length || accountIdentityCatalog.length) {
        const merged = new Map(discoveredIdentityCatalog.map((app) => [app.pluginId, app]));
        for (const accountApp of accountIdentityCatalog) {
          const discovered = merged.get(accountApp.pluginId);
          merged.set(accountApp.pluginId, {
            ...discovered,
            ...accountApp,
            bot: accountApp.bot ?? discovered?.bot,
            commands: accountApp.commands?.length ? accountApp.commands : discovered?.commands,
            surfaces: accountApp.surfaces?.length ? accountApp.surfaces : discovered?.surfaces,
          });
        }
        setMiniAppIdentityCatalog([...merged.values()]);
      } else if (identityCatalogResult.status === 'rejected') {
        setError(identityCatalogResult.reason instanceof Error ? identityCatalogResult.reason.message : String(identityCatalogResult.reason));
      } else if (accountCatalogResult.status === 'rejected') {
        setError(accountCatalogResult.reason instanceof Error ? accountCatalogResult.reason.message : String(accountCatalogResult.reason));
      }
      if (installedResult.status === 'fulfilled') {
        setInstalledMiniApps(Object.fromEntries(installedResult.value.plugins.map((plugin) => [plugin.pluginId, plugin])));
      } else {
        setError(installedResult.reason instanceof Error ? installedResult.reason.message : String(installedResult.reason));
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setMiniAppLoading(false);
    }
  }

  function setMiniAppBusyState(id: string, busy: boolean) {
    setMiniAppBusy((current) => {
      const next = new Set(current);
      if (busy) next.add(id); else next.delete(id);
      return next;
    });
  }

  async function installMiniApp(app: MarketplacePluginSummary) {
    setError(null);
    setMiniAppBusyState(app.pluginId, true);
    try {
      const release = await transport.marketplaceRelease(app.pluginId, app.latestVersion);
      if (release.releaseStatus && release.releaseStatus !== 'approved') {
        throw new Error(`Mini App release is not approved: ${release.releaseStatus}`);
      }
      if (release.releaseManifest?.protocol !== 'mahayana.external-release.v1') {
        throw new Error('Mini App release is missing a verified external release manifest');
      }
      const installContract = release.install
        ?? (release.releaseManifest.install as Record<string, unknown> | undefined);
      const installSource = installContract?.source as Record<string, unknown> | undefined;
      if (installContract?.protocol !== 'fabushi.marketplace.install.v1'
        || installContract.strategy !== 'github-immutable'
        || installSource?.marketplaceHostsPackage === true
        || typeof installSource?.repository !== 'string'
        || typeof installSource?.sourceRef !== 'string'
        || !installSource.sourceRef.trim()) {
        throw new Error('Mini App release is not a GitHub immutable package');
      }
      const pointer = await transport.pluginInstall(release.releaseManifest, 'desktop');
      setInstalledMiniApps((current) => ({ ...current, [app.pluginId]: pointer }));
      setMiniAppIdentityCatalog((current) => {
        const next = new Map(current.map((entry) => [entry.pluginId, entry]));
        const existing = next.get(app.pluginId);
        next.set(app.pluginId, existing ? { ...existing, ...app } : app);
        return [...next.values()];
      });
      try {
        await invokeNativeDesktop('addMiniAppToAccount', { pluginId: app.pluginId });
      } catch (cause) {
        await transport.pluginUninstall(app.pluginId).catch(() => undefined);
        setInstalledMiniApps((current) => {
          const next = { ...current };
          delete next[app.pluginId];
          return next;
        });
        throw cause;
      }
      const active = await transport.pluginActive(app.pluginId);
      if (!active) throw new Error('Mini App 安装完成后没有生成本地激活指针。');
      setInstalledMiniApps((current) => ({ ...current, [app.pluginId]: active }));
      setAccountBots(await readAccountBots().catch(() => []));
      await refreshMiniApps(miniAppQuery, false);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setMiniAppBusyState(app.pluginId, false);
    }
  }

  async function uninstallMiniApp(id: string) {
    setError(null);
    setMiniAppBusyState(id, true);
    try {
      await transport.pluginUninstall(id);
      await invokeNativeDesktop('removeMiniAppFromAccount', { pluginId: id });
      delete miniAppBotThreadsRef.current[id];
      if (miniApp?.id === id) setMiniApp(null);
      await refreshMiniApps(miniAppQuery);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setMiniAppBusyState(id, false);
    }
  }

  async function showMiniAppDocument(id: string, title: string, html: string) {
    const preparedHtml = miniAppCloudBridgeDocument(prepareDesktopMiniAppWebMcpDocument(id, html));
    const url = await window.fabushi.registerMiniAppDocument(id, preparedHtml);
    setMiniApp({ id, title, url });
  }

  async function openMiniApp(id: string) {
    setError(null);
    setMiniAppBusyState(id, true);
    try {
      const installed = installedMiniApps[id] ?? await transport.pluginActive(id);
      if (!installed) await reconcileAccountMiniApps().catch(() => undefined);
      const reconciledInstalled = installed ?? await transport.pluginActive(id);
      if (!reconciledInstalled) throw new Error('请先从在线 Mini App 市场安装此应用');
      const document = await transport.pluginUiDocument(id);
      const title = miniAppIdentityCatalog.find((app) => app.pluginId === id)?.displayName ?? marketplaceApps.find((app) => app.pluginId === id)?.displayName ?? id;
      await showMiniAppDocument(id, title, document.html);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setMiniAppBusyState(id, false);
    }
  }


  async function installDesktopUpdate(state: UpdateState) {
    if (!['available', 'downloading', 'staging', 'ready'].includes(state.type)) return;
    setDesktopUpdateBusy(true);
    try {
      const result = await invokeNativeDesktop<{ installed?: boolean; reason?: string }>('quitAndInstallUpdate', {
        expectedVersion: 'version' in state ? state.version : undefined,
      });
      if (result?.installed === false) throw new Error(result.reason || '无法开始桌面更新');
    } catch (cause) {
      setDesktopUpdateBusy(false);
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  function startSidebarResize(event: React.PointerEvent<HTMLDivElement>) {
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = sidebarWidth;
    const onMove = (moveEvent: PointerEvent) => {
      const next = Math.max(84, Math.min(460, startWidth + moveEvent.clientX - startX));
      setSidebarWidth(next);
    };
    const onUp = (upEvent: PointerEvent) => {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
      const finalWidth = Math.max(84, Math.min(460, startWidth + upEvent.clientX - startX));
      setSidebarWidth(finalWidth < 145 ? 88 : finalWidth);
    };
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  }

  function closeSettings() {
    setSection(settingsReturnSectionRef.current === 'settings' ? 'bots' : settingsReturnSectionRef.current);
  }

  return (
    <div
      className={`${styles.messenger} ${styles.fabushiUnified}`}
      data-agent-root-shell="true"
      data-agent-product-grid="true"
      data-testid="messenger-workspace"
      data-product-shell="agent"
      data-initial-host-hydrated={initialAgentWorkspaceHydrated ? 'true' : undefined}
      data-legacy-compatibility-hydrated={initialLegacyHydrated ? 'true' : undefined}
      data-sidebar-collapsed={sidebarWidth <= 112 || undefined}
      data-reduce-motion={desktopPreferences.reducedMotion || undefined}
      data-testid-ready-projection={startupProjection ? 'true' : undefined}
      style={{ gridTemplateColumns: infoPanelDocked ? `${sidebarWidth}px minmax(420px,1fr) 286px` : `${sidebarWidth}px minmax(420px,1fr)` }}
      onClick={() => { setMessageMenu(null); }}
    >
      <aside className={styles.chatList} data-testid="messenger-sidebar" data-collapsed={sidebarWidth <= 112 || undefined} onClick={(event) => event.stopPropagation()}>
        <AgentSidebar
          agents={agentItems}
          activeKey={activeAgentKey}
          query={search}
          collapsed={sidebarWidth <= 112}
          hostReady={hostReady}
          accountLabel={currentActor?.displayName ?? 'Account'}
          sections={agentSidebarSections}
          selectedKeys={agentSelectedKeys}
          onQuery={setSearch}
          onOpen={openAgent}
          onNewAgent={() => void createAgent()}
          onToggleCollapsed={() => setSidebarWidth((width) => width <= 112 ? 300 : 88)}
          onTogglePin={(item) => {
            agentSidebarController.togglePin(item.key);
          }}
          onRename={(item) => void renameAgent(item)}
          onHide={(item) => void hideAgent(item)}
          onDuplicate={(item) => void duplicateAgent(item)}
          onDelete={(item) => void deleteAgent(item)}
          onReorderPinned={reorderPinnedAgents}
          onToggleSelection={toggleAgentSelection}
          onRangeSelection={rangeSelectAgent}
          onClearSelection={clearAgentSelection}
          onDeleteSelected={(items) => void deleteSelectedAgents(items)}
          onMoveSelectedToSection={moveAgentsToSection}
          onCreateSection={createAgentSidebarSection}
          onToggleSection={(section) => agentSidebarController.toggleSection(section.id)}
          onRenameSection={renameAgentSidebarSection}
          onDeleteSection={deleteAgentSidebarSection}
          onMoveToSection={moveAgentToSection}
          onBroadcast={agentNetworkController.openBroadcast}
          onOpenNetwork={agentNetworkController.openNetwork}
          onOpenPlugins={() => {
            setSearch('');
            setSection('miniapps');
          }}
          onOpenSettings={() => {
            settingsReturnSectionRef.current = 'bots';
            setSection('settings');
          }}
        />
        <div
          className={styles.sidebarResizer}
          data-testid="sidebar-resizer"
          role="separator"
          aria-orientation="vertical"
          aria-label="调整会话栏宽度"
          onPointerDown={startSidebarResize}
          onDoubleClick={() => setSidebarWidth((width) => width <= 112 ? 330 : 88)}
        />
      </aside>

      <AgentCommandPalette
        open={agentPaletteController.open}
        agents={agentItems}
        query={agentPaletteController.query}
        onQuery={agentPaletteController.setQuery}
        onClose={agentPaletteController.close}
        onOpenAgent={openAgent}
        onNewAgent={() => void createAgent()}
        onNetwork={agentNetworkController.openNetwork}
        onBroadcast={agentNetworkController.openBroadcast}
        onPlugins={() => {
          setSearch('');
          setSection('miniapps');
        }}
        onSettings={() => {
          settingsReturnSectionRef.current = 'bots';
          setSection('settings');
        }}
        entries={allAgentTranscriptEntries}
        onOpenTranscriptEntry={scrollToTranscriptEntry}
        onConversationSearch={() => {
          setConversationSearchOpen(true);
          setAgentConversationSearch('');
        }}
        onComputer={() => {
          if (!activePeer || !isAgentPeer(activePeer) || activePeer.miniAppId) return;
          setAgentSettingsOpen(false);
          agentComputer.openForAgent(activePeer.agentId ?? activePeer.actorId ?? activePeer.id, 'command-palette');
          if (wideInfoLayout) setInfoOpen(true); else setNarrowInfoOpen(true);
        }}
      />

      <section className={styles.chatWorkspace}>
        <AgentNetwork
          open={agentNetworkController.open}
          agents={agentItems}
          groups={agentNetworkController.groups}
          peerMessagesByAgentId={agentNetworkController.peerMessagesByAgentId}
          activeKey={activeAgentKey}
          broadcastMode={agentNetworkController.broadcastMode}
          onClose={agentNetworkController.close}
          onOpenAgent={openAgent}
          onRefreshGroups={agentNetworkController.refreshGroups}
          onRefreshPeerHistory={agentNetworkController.refreshPeerHistory}
          onCreateGroup={agentNetworkController.createGroup}
          onUpdateGroup={agentNetworkController.updateGroup}
          onDeleteGroup={agentNetworkController.deleteGroup}
          onSendGroup={agentNetworkController.sendGroup}
          onSendPeer={agentNetworkController.sendPeer}
          onBroadcast={agentNetworkController.broadcast}
        />
        {agentNetworkController.open ? null : activePeer && sectionIsPeerList ? (
          <>
            {isAgentPeer(activePeer) && !activePeer.miniAppId ? (
              <AgentWorkspace
                title={activePeer.title}
                description={activePeer.subtitle}
                botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`}
                botState={botMarkStateForPeer(activePeer, selfBotExecutions, activePeerBusy, hostReady)}
                status={activeTypingActors.length
                  ? 'Typing…'
                  : activePeerBusy
                    ? 'Working…'
                    : `${activePeer.subtitle}${hostReady ? ' · Online' : ' · Connecting'}`}
                pinned={activeAgentPinned}
                searchActive={conversationSearchOpen}
                searchQuery={agentConversationSearch}
                computerActive={agentComputer.open}
                infoActive={layoutInfoOpen}
                onToggleSearch={() => {
                  const next = !conversationSearchOpen;
                  setConversationSearchOpen(next);
                  if (!next) setAgentConversationSearch('');
                }}
                onSearchQuery={setAgentConversationSearch}
                onSelectSearchResult={scrollToTranscriptEntry}
                onToggleComputer={() => {
                  setAgentSettingsOpen(false);
                  agentComputer.toggleForAgent(activePeer.agentId ?? activePeer.actorId ?? activePeer.id, 'agent-header');
                  if (wideInfoLayout) setInfoOpen(true); else setNarrowInfoOpen(true);
                }}
                onTogglePin={() => {
                  if (activeAgentKey) agentSidebarController.togglePin(activeAgentKey);
                }}
                onToggleInfo={() => wideInfoLayout ? setInfoOpen((value) => !value) : setNarrowInfoOpen((value) => !value)}
                entries={agentTranscriptEntries}
                activeOperationId={activeAgentOperationId}
                hasEarlierMessages={agentHasEarlierEntries}
                messageAreaRef={messageAreaRef}
                showScrollToLatest={showScrollToLatest}
                onLoadEarlier={() => setMessageRenderCount((count) => count + initialMessageRenderCount)}
                onOpenMiniApp={(id) => void openMiniApp(id)}
                onScroll={handleMessageAreaScroll}
                onScrollToLatest={scrollToLatest}
                onCopyMessage={(entry) => void copyBotMessage(entry as BotTranscriptMessage)}
                onRegenerate={(entry) => regenerateBotMessage(entry as BotTranscriptMessage)}
                onEdit={(entry) => editBotMessage(entry)}
                onResolveApproval={(approvalId, decision) => {
                  void resolveAgentApproval({ approvalId, decision }).catch((cause: unknown) => {
                    setError(cause instanceof Error ? cause.message : String(cause));
                  });
                }}
                onContextMenu={(event, entry) => {
                  if (entry.kind !== 'message' && entry.kind !== 'assistant-turn') return;
                  event.preventDefault();
                  event.stopPropagation();
                  setMessageMenu({
                    message: {
                      id: entry.id,
                      source: 'legacy',
                      role: entry.role,
                      text: entry.text,
                      createdAtMs: entry.createdAtMs,
                      kind: entry.kind,
                      ...(entry.operationId ? { operationId: entry.operationId } : {}),
                      ...(entry.streaming ? { streaming: true } : {}),
                      ...(entry.optimistic ? { optimistic: true } : {}),
                      ...(entry.queued ? { queued: true } : {}),
                      ...(entry.assistantTurn ? { assistantTurn: entry.assistantTurn } : {}),
                      ...(entry.attachments?.length ? { attachments: entry.attachments } : {}),
                    },
                    x: event.clientX,
                    y: event.clientY,
                  });
                }}
                notice={error ? <div className={styles.errorBanner} role="alert"><span>{error}</span><button type="button" onClick={() => setError(null)}><X size={14} /></button></div> : null}
                composerReplyTarget={activeAgentReply ? { id: activeAgentReply.id, label: '回复', text: activeAgentReply.text } : undefined}
                onClearComposerReply={() => clearAgentReply(activePeer.key)}
                composerAccessory={agentWorkspaceController.isUploading(activePeer.key)
                  ? <span className={extra.uploadProgress}>Uploading attachments…</span>
                  : null}
                composerValue={agentWorkspaceController.draftForPeer(activePeer.key)}
                composerRichText={agentWorkspaceController.richTextForPeer(activePeer.key)}
                composerReady={hostReady}
                composerBusy={Boolean(activeAgentOperationId)}
                composerUploading={agentWorkspaceController.isUploading(activePeer.key)}
                composerAttachments={agentWorkspaceController.attachmentsForPeer(activePeer.key)}
                composerMentionCandidates={[
                  ...agentItems
                    .filter((item) => item.key !== activePeer.key)
                    .map((item) => ({
                      id: item.agentId || item.key,
                      name: item.name,
                      description: item.description,
                      kind: 'agent' as const,
                    })),
                  ...agentMcpController.references.map((reference) => ({
                    id: reference.id,
                    name: reference.name,
                    description: reference.description,
                    kind: 'mcp' as const,
                  })),
                ]}
                composerWorkflowCandidates={(agentWorkflowController.workflowsByAgentId[activePeer.agentId ?? activePeer.actorId ?? activePeer.id] ?? [])
                  .filter((workflow) => workflow.isEnabledForAgent)
                  .map((workflow) => ({ id: workflow.id, name: workflow.name, description: workflow.description }))}
                composerPullRequestCandidates={agentPrSuggestionController.candidates}
                enterToSend={desktopPreferences.enterToSend}
                onComposerChange={(value, richText) => updateAgentComposer(activePeer.key, value, richText)}
                onComposerMention={(candidate) => {
                  agentWorkspaceController.upsertReference(activePeer.key, {
                    kind: candidate.kind === 'mcp' ? 'mcp' : 'agent',
                    id: candidate.id,
                    label: candidate.name,
                  });
                  notifyAgentWorkspaceState();
                }}
                onComposerWorkflowReference={(candidate) => {
                  agentWorkspaceController.upsertReference(activePeer.key, {
                    kind: 'workflow',
                    id: candidate.id,
                    label: candidate.name,
                  });
                  notifyAgentWorkspaceState();
                }}
                onComposerSubmit={(event) => sendAgentMessage(event, activePeer)}
                onComposerFiles={(files) => void stageAgentFiles(activePeer, files)}
                onRemoveComposerAttachment={(attachmentId) => removeAgentAttachment(activePeer.key, attachmentId)}
                onTranscribeVoice={(file) => transcribeAgentVoice(activePeer, file)}
                onStop={() => void stopAgentOperation()}
              />
            ) : (
              <>
            {isAgentPeer(activePeer) ? (
              <AgentHeader
                title={activePeer.title}
                description={activePeer.subtitle}
                botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`}
                botState={botMarkStateForPeer(activePeer, selfBotExecutions, activePeerBusy, hostReady)}
                status={activeTypingActors.length
                  ? 'Typing…'
                  : activePeerBusy
                    ? 'Working…'
                    : `${activePeer.subtitle}${hostReady ? ' · Online' : ' · Connecting'}`}
                pinned={activePeer.pinned}
                searchActive={conversationSearchOpen}
                computerActive={agentComputer.open}
                infoActive={layoutInfoOpen}
                miniAppTitle={activePeer.miniAppMenuButtonText}
                onOpenMiniApp={activePeer.miniAppId ? () => void openMiniApp(activePeer.miniAppId!) : undefined}
                onToggleSearch={() => {
                  const next = !conversationSearchOpen;
                  setConversationSearchOpen(next);
                  if (!next) setAgentConversationSearch('');
                  }}
                onToggleComputer={() => {
                  agentComputer.toggleForAgent(activePeer.agentId ?? activePeer.actorId ?? activePeer.id, 'compatibility-agent-header');
                  if (wideInfoLayout) setInfoOpen(true); else setNarrowInfoOpen(true);
                }}
                onTogglePin={() => void togglePinConversation(activePeer)}
                onToggleInfo={() => wideInfoLayout ? setInfoOpen((value) => !value) : setNarrowInfoOpen((value) => !value)}
              />
            ) : (
              <header className={styles.chatHeader}>
                <div className={styles.chatIdentity}>
                  <FabAvatar identity={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state="idle" size={40} className={styles.agentAvatarMark} label={activePeer.title} />
                  <div><strong>{activePeer.title}</strong><small data-testid="conversation-status">{activeTypingActors.length ? '正在输入…' : `${activePeer.subtitle}${hostReady ? ' · 在线' : ' · 正在连接'}`}</small></div>
                </div>
                <div className={styles.headerActions}>
                  <button type="button" title="搜索当前会话" data-active={conversationSearchOpen} onClick={() => {
                    const next = !conversationSearchOpen;
                    setConversationSearchOpen(next);
                    setSearch('');
                    }}><Search size={18} /></button>
                  <button type="button" title={activePeer.pinned ? '取消置顶' : '置顶'} onClick={() => void togglePinConversation(activePeer)}><Pin size={18} /></button>
                  <button type="button" title="资料" data-testid="conversation-info-toggle" data-active={layoutInfoOpen} onClick={() => wideInfoLayout ? setInfoOpen((value) => !value) : setNarrowInfoOpen((value) => !value)}><MoreVertical size={18} /></button>
                </div>
              </header>
            )}
            {conversationSearchOpen ? <AgentSearch
              entries={isAgentPeer(activePeer) ? agentTranscriptEntries : projectTranscriptEntries(renderedMessages)}
              query={agentConversationSearch}
              onQuery={setAgentConversationSearch}
              onClose={() => {
                setConversationSearchOpen(false);
                setAgentConversationSearch('');
              }}
              onSelect={scrollToTranscriptEntry}
            /> : null}
            {error ? <div className={styles.errorBanner} role="alert"><span>{error}</span><button type="button" onClick={() => setError(null)}><X size={14} /></button></div> : null}
            {isAgentPeer(activePeer) ? (
              <BotConversationView
                title={activePeer.title}
                description={activePeer.subtitle}
                botId={'peer:' + activePeer.kind + ':' + (activePeer.actorId ?? activePeer.id)}
                messages={botTranscriptMessages}
                activeOperationId={activeAgentOperationId}
                hasEarlierMessages={matchingMessages.length > renderedMessages.length}
                messageAreaRef={messageAreaRef}
                showScrollToLatest={showScrollToLatest}
                onOpenMiniApp={(id) => void openMiniApp(id)}
                onLoadEarlier={() => setMessageRenderCount((count) => count + initialMessageRenderCount)}
                onScroll={handleMessageAreaScroll}
                onScrollToLatest={scrollToLatest}
                onCopyMessage={(message) => void copyBotMessage(message)}
                onRegenerate={regenerateBotMessage}
                onEdit={editBotMessage}
                onContextMenu={(event, message) => {
                  event.preventDefault();
                  event.stopPropagation();
                  setMessageMenu({ message: message as DisplayMessage, x: event.clientX, y: event.clientY });
                }}
              />
            ) : (
            <div className={styles.messageArea} data-testid="message-list" data-agent-operation-id={activeAgentOperationId ?? undefined}>
              <div className={styles.dayDivider}>今天</div>
              {matchingMessages.length > renderedMessages.length ? <button type="button" data-testid="message-list-load-earlier" onClick={() => setMessageRenderCount((count) => count + initialMessageRenderCount)}>加载更早消息</button> : null}
              {renderedMessages.map((message) => message.kind === 'assistant-turn' && message.assistantTurn ? (
                <MahayanaAssistantTurnView
                  key={`${message.source}:${message.id}`}
                  turn={message.assistantTurn}
                  label={activePeer.title}
                  avatar={<FabAvatar
                    identity={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`}
                    state={message.assistantTurn.status === 'running' ? 'thinking' : message.assistantTurn.status === 'failed' ? 'error' : 'result'}
                    size={30}
                    className={styles.agentStreamAvatar}
                    label={activePeer.title}
                  />}
                />
              ) : message.kind === 'thinking' ? (
                <article key={`${message.source}:${message.id}`} className={styles.agentThinkingRow} data-testid="agent-thinking" data-operation-id={message.operationId}>
                  <FabAvatar identity={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state="thinking" size={30} className={styles.agentStreamAvatar} label={activePeer.title} />
                  <div><strong>{message.actionTitle ?? '正在思考'}</strong><span>大乘助手正在处理这条消息…</span></div>
                </article>
              ) : message.kind === 'action' ? (
                <article key={`${message.source}:${message.id}`} className={styles.agentActionRow} data-testid="agent-step" data-operation-id={message.operationId} data-status={message.actionStatus}>
                  <FabAvatar identity={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state={message.actionStatus === 'running' ? 'working' : message.actionStatus === 'failed' ? 'error' : 'result'} size={25} className={styles.agentStreamAvatar} label={activePeer.title} />
                  <div><strong>{message.actionTitle}</strong>{message.actionDetail ? <span>{message.actionDetail}</span> : null}</div>
                  <small>{message.actionStatus === 'running' ? '进行中' : message.actionStatus === 'failed' ? '失败' : '完成'}</small>
                </article>
              ) : (() => {
                const generatedPreview = message.role === 'peer'
                  ? generatedMiniAppPreview(message.text, message.operationId ?? message.id)
                  : null;
                return <article
                  key={`${message.source}:${message.id}`}
                  className={message.role === 'me' ? styles.messageMine : styles.messagePeer}
                  data-operation-id={message.operationId ?? undefined}
                  data-agent-id={`message-actions:${message.source}:${message.id}`}
                  data-agent-invoke="contextmenu"
                  data-agent-message-role={message.role}
                  onContextMenu={(event) => {
                    if (message.kind === 'assistant-turn' || message.kind === 'action' || message.kind === 'thinking') return;
                    event.preventDefault();
                    event.stopPropagation();
                    setMessageMenu({ message, x: event.clientX, y: event.clientY });
                  }}
                >
                  {message.pinned ? <Pin size={11} /> : null}
                  {message.mediaType === 'photo' && blobMediaUrl(message.media) ? <img className={extra.messageMedia} src={blobMediaUrl(message.media)} alt={message.media?.fileName ?? '图片'} /> : null}
                  {message.mediaType === 'video' && blobMediaUrl(message.media) ? <video className={extra.messageMedia} controls playsInline autoPlay={desktopPreferences.autoPlayMedia} src={blobMediaUrl(message.media)} /> : null}
                  {message.role === 'peer' ? <FabAvatar identity={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state="result" size={28} className={extra.messageAuthorAvatar} label={activePeer.title} /> : null}
                  {message.mediaType === 'document' && blobMediaUrl(message.media) ? <a className={extra.messageFile} href={blobMediaUrl(message.media)} download={message.media?.fileName}><FileText size={17} />{message.media?.fileName ?? '文件'}</a> : null}
                  {generatedPreview ? <div className={extra.generatedMiniAppCard} data-testid="generated-miniapp-card">
                    <AppWindow size={22} />
                    <div><strong>{generatedPreview.title}</strong><span>{generatedPreview.complete ? '小程序已生成，可以直接打开试玩。' : '正在生成可运行的小程序…'}</span></div>
                    {generatedPreview.complete ? <button type="button" data-testid="generated-miniapp-open" onClick={() => void showMiniAppDocument(generatedPreview.id, generatedPreview.title, generatedPreview.html)}>打开小程序</button> : null}
                  </div> : <TelegramStructuredMessageBody text={message.text} peers={peers} />}
                  <div className={extra.messageHoverActions} data-testid="message-hover-actions">
                    <button type="button" title="回复" aria-label="回复" onClick={() => setReplyTarget(message)}><span aria-hidden="true">↩</span></button>
                    <button type="button" title="复制" aria-label="复制" onClick={() => void navigator.clipboard.writeText(message.text)}><span aria-hidden="true">⧉</span></button>
                    <button type="button" title="更多" aria-label="更多" onClick={(event) => {
                      const rect = event.currentTarget.getBoundingClientRect();
                      setMessageMenu({ message, x: rect.right, y: rect.bottom + 4 });
                    }}><span aria-hidden="true">•••</span></button>
                  </div>
                  {message.reactions?.length ? <div className={extra.reactions}>{message.reactions.map((reaction) => <span key={reaction}>{reaction}</span>)}</div> : null}
                  <small>{formatTime(message.createdAtMs)} {message.role === 'me' ? <Check size={12} /> : null}</small>
                </article>;
              })())}
              {!matchingMessages.length ? <div className={styles.chatEmpty} data-testid="message-search-empty"><FabAvatar identity={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state={isAgentPeer(activePeer) ? botMarkStateForPeer(activePeer, selfBotExecutions, false, hostReady) : 'idle'} size={78} className={styles.agentAvatarMark} label={activePeer.title} /><strong>{activePeer.title}</strong><p>联系人、AI Bot、群组和频道使用同一个 Fabushi 消息产品层。</p></div> : null}
            </div>
            )}
            {legacyReplyTo ? <div className={extra.composerBanner} data-testid="reply-message-banner"><Reply size={15} /><div><strong>回复</strong><span>{legacyReplyTo.text}</span></div><button type="button" data-testid="reply-message-cancel" onClick={() => setLegacyReplyTo(null)}><X size={14} /></button></div> : null}
            {scheduledAtMs ? <div className={extra.composerBanner}><span>⏱</span><div><strong>定时发送</strong><span>{new Date(scheduledAtMs).toLocaleString()}</span></div><button type="button" onClick={() => setScheduledAtMs(undefined)}><X size={14} /></button></div> : null}
            {activePeer.miniAppId && composer.trimStart().startsWith('/') && activePeer.miniAppCommands?.length ? <div className={extra.composerBanner} data-testid="miniapp-bot-commands"><AppWindow size={15} /><div><strong>小程序命令</strong><span>{activePeer.miniAppCommands.map((command) => `/${command.name}`).join(' · ')}</span></div>{activePeer.miniAppCommands.slice(0, 4).map((command) => <button key={command.name} type="button" title={command.description} onClick={() => updateComposer(command.usage)}>{`/${command.name}`}</button>)}</div> : null}
              <form className={styles.composer} onSubmit={(event) => void sendMessage(event)}>
                <div className={extra.attachmentAnchor}>
                  <button type="button" title="附件" onClick={() => setAttachmentMenuOpen((value) => !value)}><Paperclip size={20} /></button>
                  {attachmentMenuOpen ? <TelegramAttachmentMenu onMedia={() => mediaInputRef.current?.click()} onFile={() => fileInputRef.current?.click()} onPoll={() => void sendPoll()} onLocation={() => void sendLocation()} onSchedule={() => {
                    const minutes = Number(window.prompt('多少分钟后发送？', '10'));
                    if (Number.isFinite(minutes) && minutes > 0) setScheduledAtMs(Date.now() + minutes * 60_000);
                    setAttachmentMenuOpen(false);
                  }} /> : null}
                </div>
                <input ref={mediaInputRef} type="file" accept="image/*,video/*" hidden onChange={(event) => { const file = event.currentTarget.files?.[0]; event.currentTarget.value = ''; if (file) void sendAttachmentFile(file); }} />
                <input ref={fileInputRef} type="file" hidden onChange={(event) => { const file = event.currentTarget.files?.[0]; event.currentTarget.value = ''; if (file) void sendAttachmentFile(file); }} />
                {attachmentProgress ? <span className={extra.uploadProgress}>{attachmentProgress}</span> : null}
                <textarea data-testid="messenger-input" value={composer} onChange={(event) => updateComposer(event.target.value)} onKeyDown={(event) => {
                  const submitWithEnter = desktopPreferences.enterToSend && event.key === 'Enter' && !event.shiftKey;
                  const submitWithShortcut = !desktopPreferences.enterToSend && event.key === 'Enter' && (event.metaKey || event.ctrlKey);
                  if (submitWithEnter || submitWithShortcut) { event.preventDefault(); event.currentTarget.form?.requestSubmit(); }
                }} placeholder="消息" rows={1} />
                {composer.trim()
                  ? <button data-testid="messenger-send" className={styles.sendButton} type="submit" disabled={!hostReady || legacySendPending}><Send size={19} /></button>
                  : null}
              </form>
              </>
            )}
          </>
        ) : (
          <>
            {error ? <div className={styles.errorBanner} role="alert"><span>{error}</span><button type="button" onClick={() => setError(null)}><X size={14} /></button></div> : null}
            <FeatureWorkspace section={section} onOpenMiniApp={openMiniApp} onInstallMiniApp={installMiniApp} onUninstallMiniApp={uninstallMiniApp} miniApps={marketplaceApps} installedMiniApps={installedMiniApps} miniAppQuery={miniAppQuery} onMiniAppQuery={setMiniAppQuery} miniAppLoading={miniAppLoading} miniAppBusy={miniAppBusy} onInvoice={() => void createInvoiceForActivePeer()} payment={{ account: walletAccount, entries: walletEntries, orders: selfOrders, invoices: selfInvoices, actorId: selfHosted.actorId }} onRefund={(orderId) => void refundOrder(orderId)} settings={{ category: settingsCategory, onCategory: setSettingsCategory, preferences: desktopPreferences, onPreference: updateDesktopPreference, actor: currentActor, actorId: selfHosted.actorId, hostSettings, onHostSetting: updateHostSetting, onConfigureProviderSecret: configureProviderSecret, onRemoveProviderSecret: removeProviderSecret, routerStatus, usageSummary, onInstallUpdate: installDesktopUpdate, onLogout }} />
          </>
        )}
      </section>

      {infoPanelVisible && activePeer ? (
        isAgentPeer(activePeer) && !activePeer.miniAppId ? (
          <AgentOverlays
            title={activePeer.title}
            description={activePeer.subtitle}
            botId={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`}
            botState={botMarkStateForPeer(activePeer, selfBotExecutions, activePeerBusy, hostReady)}
            pinned={activeAgentPinned}
            overlay={!wideInfoLayout}
            onClose={() => wideInfoLayout ? setInfoOpen(false) : setNarrowInfoOpen(false)}
            onSearch={() => {
              setConversationSearchOpen(true);
              setAgentConversationSearch('');
            }}
            onTogglePin={() => {
              if (activeAgentKey) agentSidebarController.togglePin(activeAgentKey);
            }}
            computer={{
              agentId: activePeer.agentId ?? activePeer.actorId ?? activePeer.id,
              open: agentComputer.open,
              label: localComputerLabel(),
              status: localComputerStatus,
              online: localComputerOnline,
              aiControlEnabled: hostSettings.aiComputerControlEnabled,
              remoteControlEnabled: hostSettings.remoteControlEnabled,
              state: agentComputer.state,
              capabilityStatus: agentComputer.capabilityStatus,
              onToggle: () => {
                setAgentSettingsOpen(false);
                agentComputer.toggleForAgent(activePeer.agentId ?? activePeer.actorId ?? activePeer.id, 'agent-overlay');
              },
              onRefreshPairingCode: agentComputer.refreshPairingCode,
              onApproveSession: agentComputer.approveSession,
              onDenySession: agentComputer.denySession,
              onDisconnect: agentComputer.disconnect,
              onToggleRemoteControl: () => updateHostSetting('remoteControlEnabled', !hostSettings.remoteControlEnabled),
              onOpenControlPage: () => agentComputer.openControlPage(activePeer.agentId ?? activePeer.actorId ?? activePeer.id),
            }}
            settings={{
              agentId: activePeer.agentId ?? activePeer.actorId ?? activePeer.id,
              open: agentSettingsOpen,
              value: agentSettingsSnapshot.value ?? {
                name: activePeer.title,
                title: '',
                description: activePeer.subtitle,
                avatarShape: activeAgentBot?.avatarShape ?? '',
                avatarColor: activeAgentBot?.avatarColor ?? '',
                notifyOnUpdatesEnabled: true,
                inferenceProvider: activeAgentBot?.inferenceProvider ?? 'account-default',
              },
              pending: agentSettingsSnapshot.pending,
              error: agentSettingsSnapshot.error,
              onToggle: () => {
                agentComputer.close();
                setAgentSettingsOpen((value) => !value);
              },
              onUpdateProfile: (profile) => agentSettingsController.updateProfile(profile),
              onSetNotifications: (enabled) => agentSettingsController.setNotifications(enabled),
              onSetInferenceProvider: (provider) => agentSettingsController.setInferenceProvider(provider),
            }}
          />
        ) : (
        activePeer.kind === 'contact' ? <ContactCompatibilityAdapter
          peer={activePeer}
          muted={mutedPeerKeys.has(activePeer.key)}
          infoTab={infoTab}
          overlay={!wideInfoLayout}
          onClose={() => wideInfoLayout ? setInfoOpen(false) : setNarrowInfoOpen(false)}
          onVoiceCall={() => void startCall('voice')}
          onVideoCall={() => void startCall('video')}
          onSearch={() => { setConversationSearchOpen(true); setAgentConversationSearch(''); }}
          onToggleMute={() => void toggleMuteConversation(activePeer)}
          onTogglePin={() => void togglePinConversation(activePeer)}
          onToggleArchive={() => { void toggleArchiveConversation(activePeer); setSection('archive'); }}
          onInfoTab={setInfoTab}
        /> : <aside className={styles.infoPanel} data-testid="messenger-info-panel" data-overlay={!wideInfoLayout || undefined}>
          <header><strong>资料</strong><button type="button" onClick={() => wideInfoLayout ? setInfoOpen(false) : setNarrowInfoOpen(false)}><X size={17} /></button></header>
          <div className={styles.profileCard}>
            <FabAvatar identity={`peer:${activePeer.kind}:${activePeer.actorId ?? activePeer.id}`} state={isAgentPeer(activePeer) ? botMarkStateForPeer(activePeer, selfBotExecutions, activePeerBusy, hostReady) : 'idle'} size={92} className={styles.agentProfileMark} label={activePeer.title} />
            <strong>{activePeer.title}</strong><small>{activePeer.subtitle}</small>
            <div className={styles.profileQuickActions} data-columns={isAgentPeer(activePeer) ? '4' : '3'}><button type="button" onClick={() => void startCall('voice')}><PhoneCall size={18} /><span>通话</span></button><button type="button" onClick={() => void startCall('video')}><Video size={18} /><span>视频</span></button><button type="button" onClick={() => { setConversationSearchOpen(true); setAgentConversationSearch(''); }}><Search size={18} /><span>搜索</span></button>{isAgentPeer(activePeer) ? <button type="button" data-testid="bot-computer-toggle" data-active={agentComputer.open} onClick={() => {
              agentComputer.toggleForAgent(activePeer.agentId ?? activePeer.actorId ?? activePeer.id, 'bot-profile');
            }}><Monitor size={18} /><span>电脑</span></button> : null}</div>
          </div>
          <div className={styles.profileActions}>
            <button type="button" onClick={() => void toggleMuteConversation(activePeer)}><BellOff size={17} /><span>{mutedPeerKeys.has(activePeer.key) ? '开启通知' : '静音通知'}</span></button>
            <button type="button" onClick={() => void togglePinConversation(activePeer)}><Pin size={17} /><span>{activePeer.pinned ? '取消置顶' : '置顶会话'}</span></button>
            <button type="button" onClick={() => { void toggleArchiveConversation(activePeer); setSection('archive'); }}><Archive size={17} /><span>{activePeer.archived ? '移出归档' : '归档会话'}</span></button>
            {activePeer.source === 'selfhosted' && ['group', 'channel'].includes(activePeer.kind) ? <button type="button" onClick={() => void openCommunityAdmin(activePeer)}><Settings size={17} /><span>管理群组/频道</span></button> : null}
          </div>
          {isAgentPeer(activePeer) && agentComputer.open ? <section className={styles.computerProfile} data-testid="bot-computer-panel">
            <header>
              <span className={styles.computerProfileIcon}><Monitor size={18} /></span>
              <span><strong>这台电脑</strong><small>{localComputerLabel()}</small></span>
              <i data-live={agentComputer.state?.channelOpen ? 'active' : localComputerOnline ? 'online' : 'offline'} />
            </header>
            <div className={styles.computerProfileStatus}>
              <span><small>设备状态</small><strong>{localComputerStatus}</strong></span>
              <span><small>已授权客户端</small><strong>{agentComputer.state?.clients.length ?? 0}</strong></span>
              <span><small>AI 操控</small><strong>{hostSettings.aiComputerControlEnabled ? '已允许' : '已关闭'}</strong></span>
            </div>
            <p>Fabushi 登录后会在后台保持设备在线；远控关闭时只能被同账号发现，不能读取画面或发送输入。</p>
            {hostSettings.remoteControlEnabled && agentComputer.state?.registration?.pairingCode ? <div className={styles.computerPairingCode}>
              <span><small>配对码</small><strong>{agentComputer.state.registration.pairingCode}</strong></span>
              <button type="button" onClick={() => agentComputer.refreshPairingCode()}>刷新</button>
            </div> : null}
            {pendingRemoteAuthorization ? <div className={styles.computerPairingCode} data-testid="remote-session-consent">
              <span><small>远控请求</small><strong>{pendingRemoteAuthorization.clientLabel || '已配对设备'}</strong></span>
              <button type="button" onClick={() => agentComputer.approveSession(pendingRemoteAuthorization.sessionId)}>允许本次连接</button>
              <button type="button" className={styles.computerDangerButton} onClick={() => agentComputer.denySession(pendingRemoteAuthorization.sessionId)}>拒绝</button>
            </div> : null}
            {agentComputer.state?.activeSessionId ? <button type="button" className={styles.computerDangerButton} onClick={() => agentComputer.disconnect()}>断开当前远控</button> : null}
            <div className={styles.computerProfileButtons}>
              <button type="button" data-enabled={hostSettings.remoteControlEnabled} onClick={() => updateHostSetting('remoteControlEnabled', !hostSettings.remoteControlEnabled)}>{hostSettings.remoteControlEnabled ? '关闭远程控制' : '开启远程控制'}</button>
              <button type="button" onClick={() => agentComputer.openControlPage(activePeer.agentId ?? activePeer.actorId ?? activePeer.id)}>打开控制页面</button>
            </div>
            {agentComputer.state?.error ? <small className={styles.computerProfileError}>{agentComputer.state.error}</small> : null}
          </section> : null}
          <nav className={styles.infoTabs}><button type="button" data-active={infoTab === 'media'} onClick={() => setInfoTab('media')}>媒体</button><button type="button" data-active={infoTab === 'files'} onClick={() => setInfoTab('files')}>文件</button><button type="button" data-active={infoTab === 'links'} onClick={() => setInfoTab('links')}>链接</button></nav>
          <div className={styles.infoContent}>{infoTab === 'media' ? <><Image size={30} /><strong>共享媒体</strong><p>图片、视频和动画按消息索引展示。</p></> : null}{infoTab === 'files' ? <><FileText size={30} /><strong>共享文件</strong><p>文档、音频和附件由 Rust 媒体层管理。</p></> : null}{infoTab === 'links' ? <><Link2 size={30} /><strong>共享链接</strong><p>富文本 URL 建立可搜索索引。</p></> : null}</div>
        </aside>
        )
      ) : null}

      {messageMenu ? <TelegramMessageContextMenu menu={messageMenu} onAction={(action) => void handleMessageAction(action)} /> : null}
      {forwardDialog ? <TelegramForwardMessageDialog message={forwardDialog.message} peers={peers.filter((peer) => peer.source === 'selfhosted' && Boolean(peer.conversationId) && peer.conversationId !== forwardDialog.sourceConversationId)} onClose={() => setForwardDialog(null)} onSelect={(peer) => void forwardToPeer(peer)} /> : null}
      {editDialog ? <TelegramEditMessageDialog value={editDialog.text} onChange={(text) => setEditDialog((current) => current ? { ...current, text } : current)} onClose={() => setEditDialog(null)} onSave={() => void saveEditedMessage()} /> : null}
      {invoiceDialog ? <InvoiceCompatibilityDialog dialog={invoiceDialog} onChange={setInvoiceDialog} onClose={() => setInvoiceDialog(null)} onSave={() => void saveInvoiceDialog()} /> : null}
      {newDialog ? <TelegramNewConversationDialog dialog={newDialog} bots={bots} onChange={setNewDialog} onClose={() => setNewDialog(null)} onSave={() => void saveNewDialog()} /> : null}
      {communityDialogPeer ? <TelegramCommunityAdminDialog peer={communityDialogPeer} community={selfCommunities.find((item) => item.conversationId === communityDialogPeer.conversationId) ?? defaultCommunityState(communityDialogPeer.conversationId!, selfHosted.actorId)} actors={selfActors} actorId={selfHosted.actorId} onClose={() => setCommunityDialogPeer(null)} onSave={(community) => void saveCommunity(community)} onSetMember={(conversationId, member) => void selfHosted.setCommunityMember(conversationId, member).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onCreateInvite={(community) => { const invite = { id: `invite:${crypto.randomUUID()}`, conversationId: community.conversationId, creatorId: selfHosted.actorId, token: crypto.randomUUID().replaceAll('-', ''), name: '邀请链接', createdAtMs: Date.now(), expiresAtMs: undefined, memberLimit: undefined, joinRequest: community.joinRequestRequired, revoked: false, joinedCount: 0 }; void selfHosted.createInviteLink(invite).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause))); }} onRevokeInvite={(conversationId, inviteId) => void selfHosted.revokeInviteLink(conversationId, inviteId).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onJoinDecision={(conversationId, requesterId, approved) => void selfHosted.respondCommunityJoin(conversationId, requesterId, approved).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onUpsertTopic={(topic) => void selfHosted.upsertForumTopic(topic).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} onDeleteTopic={(conversationId, topicId) => void selfHosted.deleteForumTopic(conversationId, topicId).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} /> : null}
      {activeStory ? <TelegramStoryViewer story={activeStory} owner={selfActors.find((actor) => actor.id === activeStory.ownerId)} own={activeStory.ownerId === selfHosted.actorId} onClose={() => setActiveStory(null)} onReact={(reaction) => void reactToActiveStory(reaction)} onDelete={() => void selfHosted.deleteStory(activeStory.id).then(() => setActiveStory(null)).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))} /> : null}
      {localCall ? <CallCompatibilityAdapter call={localCall} localVideoRef={localVideoRef} remoteVideoRef={remoteVideoRef} remoteAudioRef={remoteAudioRef} canAccept={Boolean(incomingCall)} onAccept={() => void acceptIncomingCall()} onDecline={() => void declineIncomingCall()} onMute={() => void toggleCallMute()} onVideo={() => void toggleCallVideo()} onShare={() => void shareCallScreen()} onEnd={() => void endCall()} /> : null}
      {miniAppCall ? <MiniAppCallDialog
        callId={miniAppCall.callId}
        miniAppId={miniAppCall.miniAppId}
        title={miniAppCall.title}
        kind={miniAppCall.kind}
        program={miniAppCall.program}
        html={miniAppCall.html}
        onCommand={(command, args) => runMiniAppCallCommand(miniAppCall, command, args)}
        onNaturalLanguage={miniAppCall.program.aiMode === 'optional'
          ? (input) => appendMiniAppCallExchange(miniAppCall.miniAppId, input, '通话语音/自然语言 · ' + input)
          : undefined}
        onSaveRecording={(blob, mimeType) => saveMiniAppCallRecording(miniAppCall, blob, mimeType)}
        onClose={() => setMiniAppCall(null)}
      /> : null}
      {miniApp ? <MiniAppCompatibilityDialog app={miniApp} onClose={() => setMiniApp(null)} /> : null}
      {section === 'settings' ? <div className={styles.settingsModalBackdrop} data-testid="settings-modal-backdrop" onMouseDown={closeSettings}>
        <section className={styles.settingsModal} role="dialog" aria-modal="true" aria-label="设置" onMouseDown={(event) => event.stopPropagation()}>
          <aside className={styles.settingsModalSidebar}>
            <header><strong>Settings</strong></header>
            <SettingsNavigation category={settingsCategory} onCategory={setSettingsCategory} />
          </aside>
          <div className={styles.settingsModalContent}>
            <button type="button" className={styles.settingsModalClose} data-testid="settings-close" aria-label="关闭设置" onClick={closeSettings}><X size={19} /></button>
            {error ? <div className={styles.settingsModalError} role="alert"><span>{error}</span><button type="button" aria-label="关闭错误" onClick={() => setError(null)}><X size={14} /></button></div> : null}
            <SettingsCompatibilityAdapter category={settingsCategory} onCategory={setSettingsCategory} preferences={desktopPreferences} onPreference={updateDesktopPreference} actor={currentActor} actorId={selfHosted.actorId} hostSettings={hostSettings} onHostSetting={updateHostSetting} onConfigureProviderSecret={configureProviderSecret} onRemoveProviderSecret={removeProviderSecret} routerStatus={routerStatus} usageSummary={usageSummary} onInstallUpdate={installDesktopUpdate} onLogout={onLogout} />
          </div>
        </section>
      </div> : null}
    </div>
  );
}

function FeatureWorkspace({ section, onInvoice, payment, onRefund, settings: _settings, ...miniAppProps }: { section: MessengerSection; onInvoice: () => void; payment: PaymentUiState; onRefund: (orderId: string) => void; settings: SettingsWorkspaceProps } & MiniAppMarketplaceProps) {
  return <CompatibilitySurface
    section={section}
    miniApps={<MiniAppMarketplaceCompatibilityAdapter {...miniAppProps} />}
    payments={<div className={styles.featureWorkspace}><WalletCards size={54} /><h2>Fabushi Pay</h2><p>自建余额、Invoice、Order、退款与外部 settlement 都由 Rust 账本结算。</p><PaymentCompatibilityAdapter payment={payment} onInvoice={onInvoice} onRefund={onRefund} /></div>}
    calls={<div className={styles.featureWorkspace}><Phone size={54} /><h2>通话</h2><p>本机媒体已接通，Rust realtime 已具备一对一/群组通话信令状态。</p></div>}
    settings={<div className={styles.featureWorkspace} aria-hidden="true" />}
    fallback={<div className={styles.featureWorkspace}><MessageCircle size={54} /><h2>{sectionTitle(section)}</h2><p>联系人、Bot、群组和频道正在统一到同一个 Fabushi Actor/Conversation 模型。</p></div>}
  />;
}