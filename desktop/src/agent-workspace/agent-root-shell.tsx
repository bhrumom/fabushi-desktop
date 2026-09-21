import { X } from 'lucide-react';
import React, { useCallback, useEffect, useMemo, useRef, useState, type FormEvent } from 'react';
import type {
  BotSummary,
  ProductHostSettings,
  RuntimeEvent,
} from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  MAHAYANA_COMMAND_EVENT_NAME,
  type MahayanaCommandBridgeDetail,
} from '../../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import type { MahayanaHostTransport } from '../../../frontend/apps/web/src/lib/mahayana-host/transport';
import AgentCommandPalette from './agent-command-palette';
import AgentNetwork from './agent-network';
import AgentOverlays from './agent-overlays';
import AgentSidebar from './agent-sidebar';
import type { AgentSidebarSection } from './agent-sidebar-state';
import {
  projectAgentSidebarItems,
  type AgentPeerProjection,
  type AgentSidebarItem,
} from './agent-model';
import AgentWorkspace from './agent-workspace';
import { AgentCoordinatorClient } from './coordinator-client';
import { useAgentComputerController } from './use-agent-computer-controller';
import { useAgentProductControllers } from './use-agent-product-controllers';
import { useAgentSettingsController } from './use-agent-settings-controller';
import { useAgentShellViewState } from './use-agent-shell-view-state';
import { useAgentWorkspaceRuntime } from './use-agent-workspace-runtime';
import { mirrorAgentStoreObject, removeAgentStoreObject } from './agent-store-mirror';
import {
  AGENT_ATTACHMENT_LIMIT,
  agentFileToBase64,
  enrichAgentAttachmentPreview,
  validateAgentAttachment,
} from './agent-attachments';
import type { TranscriptEntry } from './transcript-model';
import { desktopComputerLabel, createDesktopAgentTransport } from '../bridge/agent-host';
import FabAvatar, { type FabAvatarInputState } from '../ui/avatar/fab-avatar';
import { FabIconButton } from '../ui/primitives/fab-primitives';
import MiniAppCompatibilityAdapter from '../features/miniapps/miniapp-compatibility-adapter';
import SettingsCompatibilityAdapter from '../features/settings/settings-compatibility-adapter';
import ContactsCompatibilityAdapter from '../features/contacts/contacts-compatibility-adapter';
import TelegramCompatibilityAdapter from '../features/telegram/telegram-compatibility-adapter';
import PaymentsCompatibilityAdapter from '../features/payments/payments-compatibility-adapter';
import CallsCompatibilityAdapter from '../features/calls/calls-compatibility-adapter';
import styles from './agent-root-shell.module.css';

type CompatibilitySurface = 'agents' | 'contacts' | 'telegram' | 'miniapps' | 'payments' | 'calls' | 'settings';

const defaultHostSettings: ProductHostSettings = {
  notifications: true,
  autoUpdateWhenIdle: true,
  localExecution: true,
  routeEgressLocally: false,
  securityKeys: false,
  webauthnProxyEnabled: false,
  localToolPermission: 'ask',
  remoteControlEnabled: false,
  aiComputerControlEnabled: true,
  autoReviewRules: [],
  inferenceProvider: 'fabushi',
  sandboxRuntime: 'host',
};

function requestId(prefix: string): string {
  return `${prefix}:${Date.now().toString(36)}:${crypto.randomUUID()}`;
}

function agentPeer(bot: BotSummary): AgentPeerProjection {
  const agentId = bot.agentId || bot.id;
  return {
    key: `agent:${agentId}`,
    id: bot.id,
    actorId: bot.id,
    agentId,
    ...(bot.conversationId ? { conversationId: bot.conversationId } : {}),
    kind: 'bot',
    title: bot.name,
    subtitle: bot.description || bot.title || 'Agent',
    pinned: false,
    hidden: bot.hidden === true,
    unread: bot.unread ? 1 : 0,
    updatedAtMs: Date.now(),
  };
}

function agentAvatarState(busy: boolean, hostReady: boolean): FabAvatarInputState {
  if (busy) return 'working';
  return hostReady ? 'idle' : 'offline';
}

function transcriptSources(event: Extract<RuntimeEvent, { type: 'conversation.opened' }>) {
  return event.messages.map((message) => ({
    id: message.id,
    source: 'legacy' as const,
    role: message.role === 'user' ? 'me' as const : 'peer' as const,
    text: message.text,
    createdAtMs: message.createdAtMs,
    kind: 'message' as const,
  }));
}

export default function AgentRootShell({
  onLogout,
  transport: suppliedTransport,
}: {
  readonly onLogout: () => Promise<void>;
  readonly transport?: MahayanaHostTransport;
}) {
  const transport = useMemo(() => suppliedTransport ?? createDesktopAgentTransport(), [suppliedTransport]);
  const coordinatorClient = useMemo(() => new AgentCoordinatorClient(transport), [transport]);
  const [hostReady, setHostReady] = useState(false);
  const [directoryReady, setDirectoryReady] = useState(false);
  const [accountScope, setAccountScope] = useState<string | null>(null);
  const [activePeerKey, setActivePeerKey] = useState<string | null>(null);
  const [surface, setSurface] = useState<CompatibilitySurface>('agents');
  const [error, setError] = useState<string | null>(null);
  const [hostSettings, setHostSettings] = useState<ProductHostSettings>(defaultHostSettings);
  const messageAreaRef = useRef<HTMLDivElement>(null);
  const peersRef = useRef<AgentPeerProjection[]>([]);
  const computerStatusRef = useRef<(status: Parameters<ReturnType<typeof useAgentComputerController>['handleCapabilityStatus']>[0]) => void>(() => {});

  const view = useAgentShellViewState({
    initialSidebarWidth: 300,
    initialMessageRenderCount: 240,
    initialInfoOpen: false,
  });

  const product = useAgentProductControllers({
    client: coordinatorClient,
    accountScope,
    onError: setError,
    storeSync: {
      mirror(agentId, path, value) {
        return mirrorAgentStoreObject(agentId, path, value).catch((cause) => {
          setError(cause instanceof Error ? cause.message : String(cause));
        });
      },
      remove(agentId, path) {
        return removeAgentStoreObject(agentId, path).catch((cause) => {
          setError(cause instanceof Error ? cause.message : String(cause));
        });
      },
    },
    directory: {
      onListed() {
        setDirectoryReady(true);
      },
      onChanged() {
        setDirectoryReady(true);
      },
      onError: setError,
    },
  });

  const runtime = useAgentWorkspaceRuntime({
    hostReady,
    coordinatorClient,
    onError(_peerKey, message) {
      setError(message);
    },
    onComputerStatus(status) {
      computerStatusRef.current(status);
    },
  });

  const peers = useMemo(() => product.directory.agents.map(agentPeer), [product.directory.agents]);
  peersRef.current = peers;

  useEffect(() => {
    runtime.coordinator.bindAgentPeers(peers.map((peer) => ({
      agentId: peer.agentId || peer.id,
      peerKey: peer.key,
    })));
  }, [peers, runtime.coordinator]);

  const operationSnapshot = useMemo(
    () => runtime.controller.snapshot(),
    [runtime.controller, runtime.revision],
  );
  const requestSnapshot = useMemo(
    () => runtime.controller.requestSnapshot(),
    [runtime.controller, runtime.revision],
  );
  const activityByPeer = useMemo(() => Object.fromEntries(peers.map((peer) => {
    const entries = runtime.transcriptStore.entries(peer.key);
    const lastMessage = [...entries].reverse().find((entry) =>
      (entry.kind === 'message' || entry.kind === 'assistant-turn') && Boolean(entry.text.trim()),
    );
    return [peer.key, {
      draftPrompt: runtime.controller.draftForPeer(peer.key),
      lastMessage: lastMessage?.text.trim().slice(0, 180),
    }];
  })), [peers, runtime.controller, runtime.revision, runtime.transcriptStore]);

  const agentItems = useMemo(
    () => projectAgentSidebarItems(peers, { ...requestSnapshot, ...operationSnapshot }, activityByPeer, product.sidebar.pinnedOrder),
    [activityByPeer, operationSnapshot, peers, product.sidebar.pinnedOrder, requestSnapshot],
  );

  useEffect(() => {
    if (!product.sidebar.ready || !agentItems.length) return;
    const mahayana = agentItems.find((item) => item.agentId === 'mahayana-assistant') ?? agentItems[0];
    if (mahayana) product.sidebar.adoptLegacyPinnedState([mahayana.key]);
  }, [agentItems, product.sidebar.adoptLegacyPinnedState, product.sidebar.ready]);

  const activePeer = peers.find((peer) => peer.key === activePeerKey) ?? null;
  const activeAgent = activePeer
    ? product.directory.agents.find((bot) => (bot.agentId || bot.id) === (activePeer.agentId || activePeer.id)) ?? null
    : null;
  const activeItem = activePeer ? agentItems.find((item) => item.peerKey === activePeer.key) ?? null : null;
  const activeEntriesAll = activePeer ? runtime.transcriptStore.entries(activePeer.key) : [];
  const activeEntries = activeEntriesAll.slice(Math.max(0, activeEntriesAll.length - view.messageRenderCount));
  const activeOperationId = activePeer ? operationSnapshot[activePeer.key] ?? null : null;
  const activeBusy = Boolean(activePeer && (activeOperationId || requestSnapshot[activePeer.key]));
  const activePinned = activeItem?.pinned === true;

  const settings = useAgentSettingsController(product.directory, activeAgent);

  const computer = useAgentComputerController({
    hostReady,
    hydrated: directoryReady,
    accountScope,
    activePeerKey,
    transport,
    coordinatorClient,
    label: desktopComputerLabel(),
    remoteControlEnabled: hostSettings.remoteControlEnabled,
    resolveAgentId(requestedAgentId) {
      return agentItems.some((item) => item.agentId === requestedAgentId)
        ? requestedAgentId
        : null;
    },
    onError: setError,
  });
  computerStatusRef.current = computer.handleCapabilityStatus;

  const eventHandlerRef = useRef<(event: RuntimeEvent) => void>(() => {});
  eventHandlerRef.current = (event) => {
    if (product.directory.handle(event)) return;
    if (product.sidebar.handle(event)) return;
    if (product.network.handle(event)) return;
    if (product.storeSync.handle(event)) return;
    if (product.workflow.handle(event)) return;
    if (product.mcp.handle(event)) return;
    if (computer.handleRuntimeEvent(event)) return;
    if (event.type === 'turn.state' && event.state === 'waiting-user') {
      const waitingPeer = peersRef.current.find(
        (candidate) => runtime.controller.operationForPeer(candidate.key) === event.operationId,
      );
      if (waitingPeer?.key === activePeerKey) {
        computer.openForAgent(waitingPeer.agentId || waitingPeer.id, 'waiting-user');
        view.setAgentSettingsOpen(false);
        if (view.wideInfoLayout) view.setInfoOpen(true);
        else view.setNarrowInfoOpen(true);
      }
    }
    if (runtime.coordinator.handle(event)) return;

    if (event.type === 'conversation.opened') {
      const peer = peersRef.current.find((candidate) => candidate.conversationId === event.conversationId);
      if (!peer || runtime.controller.operationForPeer(peer.key)) return;
      runtime.transcriptStore.replace(peer.key, transcriptSources(event));
      runtime.notify();
      return;
    }
    if (event.type === 'settings.changed') {
      setHostSettings(event.settings);
      return;
    }
    if (event.type === 'host.ready') {
      setHostReady(true);
      return;
    }
    if (event.type === 'host.closed') {
      setHostReady(false);
      runtime.coordinator.resetOperations();
      runtime.notify();
      return;
    }
    if (event.type === 'operation.failed') {
      setError(event.message);
    }
  };

  useEffect(() => {
    let closed = false;
    const connection = coordinatorClient.connect({
      config: { profileId: 'desktop-agent-root-v1', mode: 'production' },
      onEvent(event) {
        if (!closed) eventHandlerRef.current(event);
      },
      onState(state) {
        if (closed) return;
        setHostReady(state.phase === 'ready');
      },
    });
    const onBridge = (event: Event) => {
      const detail = (event as CustomEvent<MahayanaCommandBridgeDetail>).detail;
      if (detail) runtime.coordinator.handleCommandBridge(detail);
    };
    window.addEventListener(MAHAYANA_COMMAND_EVENT_NAME, onBridge);

    void connection.ready.then(async () => {
      if (closed) return;
      setHostReady(true);
      try {
        const auth = await coordinatorClient.authStatus();
        const identity = auth.user?.id ?? auth.user?.username ?? auth.user?.email ?? 'local';
        setAccountScope(String(identity));
      } catch {
        setAccountScope('local');
      }
      await product.directory.list().catch(() => undefined);
      void product.network.refreshGroups().catch(() => undefined);
      void product.mcp.list().catch(() => undefined);
      void transport.execute({ type: 'settings.get', requestId: requestId('settings-get') } as Parameters<MahayanaHostTransport['execute']>[0]).catch(() => undefined);
    }).catch((cause: unknown) => {
      if (!closed) setError(cause instanceof Error ? cause.message : String(cause));
    });

    return () => {
      closed = true;
      window.removeEventListener(MAHAYANA_COMMAND_EVENT_NAME, onBridge);
      void connection.dispose();
    };
  }, [
    coordinatorClient,
    product.directory.list,
    product.mcp.list,
    product.network.refreshGroups,
    runtime.coordinator,
    transport,
  ]);

  useEffect(() => {
    if (activePeerKey || !agentItems.length) return;
    const first = agentItems.find((item) => item.agentId === 'mahayana-assistant') ?? agentItems[0];
    if (!first) return;
    setActivePeerKey(first.peerKey);
  }, [activePeerKey, agentItems]);

  useEffect(() => {
    if (!hostReady || !activePeer) return;
    if (activePeer.conversationId) {
      void runtime.openConversation(activePeer.key, activePeer.conversationId).catch((cause) => {
        setError(cause instanceof Error ? cause.message : String(cause));
      });
    }
    void product.workflow.list(activePeer.agentId || activePeer.id).catch(() => undefined);
  }, [activePeer?.key, activePeer?.conversationId, hostReady]);


  useEffect(() => {
    if (!activePeer) return;
    const timer = window.setTimeout(() => {
      const node = messageAreaRef.current;
      if (node) node.scrollTop = node.scrollHeight;
    }, 0);
    return () => window.clearTimeout(timer);
  }, [activePeer?.key, runtime.revision]);

  const openAgent = useCallback((item: AgentSidebarItem) => {
    setSurface('agents');
    setActivePeerKey(item.peerKey);
    product.sidebar.clearSelection();
  }, [product.sidebar.clearSelection]);

  async function createAgent(): Promise<void> {
    const name = window.prompt('Agent name', 'New Agent')?.trim();
    if (!name) return;
    await product.directory.create({ name, description: '' });
    await product.directory.list();
  }

  async function renameAgent(item: AgentSidebarItem): Promise<void> {
    const bot = product.directory.agents.find((candidate) => (candidate.agentId || candidate.id) === item.agentId);
    if (!bot) return;
    const name = window.prompt('Rename Agent', bot.name)?.trim();
    if (!name || name === bot.name) return;
    await product.directory.update(bot.id, { name });
  }

  async function duplicateAgent(item: AgentSidebarItem): Promise<void> {
    const bot = product.directory.agents.find((candidate) => (candidate.agentId || candidate.id) === item.agentId);
    if (!bot) return;
    await product.directory.duplicate(bot.id);
    await product.directory.list();
  }

  async function deleteAgent(item: AgentSidebarItem): Promise<void> {
    const bot = product.directory.agents.find((candidate) => (candidate.agentId || candidate.id) === item.agentId);
    if (!bot || !window.confirm(`Delete Agent “${item.name}”?`)) return;
    runtime.controller.clearPeer(item.peerKey);
    runtime.transcriptStore.clear(item.peerKey);
    runtime.notify();
    await product.directory.delete(bot.id);
    if (activePeerKey === item.peerKey) setActivePeerKey(null);
  }

  async function hideAgent(item: AgentSidebarItem): Promise<void> {
    const bot = product.directory.agents.find((candidate) => (candidate.agentId || candidate.id) === item.agentId);
    if (bot) await product.directory.setHidden(bot.id, true);
  }

  function updateAgentComposer(peerKey: string, value: string, richText?: string): void {
    runtime.controller.setDraftDocument(peerKey, value, richText);
    runtime.controller.pruneReferences(peerKey, value);
    runtime.notify();
  }

  function enqueueAgentPrompt(
    peer: AgentPeerProjection,
    text: string,
    attachments = runtime.controller.attachmentsForPeer(peer.key),
    richText = runtime.controller.richTextForPeer(peer.key),
  ): void {
    runtime.submit({
      peerKey: peer.key,
      agentId: peer.agentId || peer.id,
      ...(peer.conversationId ? { conversationId: peer.conversationId } : {}),
      prompt: text,
      ...(richText ? { richText } : {}),
      ...(attachments.length ? { attachments: [...attachments] } : {}),
      ...(runtime.controller.replyForPeer(peer.key) ? { replyTo: runtime.controller.replyForPeer(peer.key) } : {}),
      ...(runtime.controller.referencesForPeer(peer.key).length
        ? { references: [...runtime.controller.referencesForPeer(peer.key)] }
        : {}),
    });
  }

  function sendAgentMessage(event: FormEvent<HTMLFormElement>): void {
    event.preventDefault();
    if (!activePeer) return;
    const draft = runtime.controller.takeDraft(activePeer.key);
    if (!draft.text.trim() && !draft.attachments.length) return;
    runtime.notify();
    runtime.submit({
      peerKey: activePeer.key,
      agentId: activePeer.agentId || activePeer.id,
      ...(activePeer.conversationId ? { conversationId: activePeer.conversationId } : {}),
      prompt: draft.text.trim(),
      ...(draft.richText ? { richText: draft.richText } : {}),
      ...(draft.attachments.length ? { attachments: [...draft.attachments] } : {}),
      ...(draft.replyTo ? { replyTo: draft.replyTo } : {}),
      ...(draft.references?.length ? { references: [...draft.references] } : {}),
    });
  }

  async function stageAgentFiles(files: readonly File[]): Promise<void> {
    if (!activePeer || !files.length) return;
    const peerKey = activePeer.key;
    const agentId = activePeer.agentId || activePeer.id;
    const available = Math.max(0, AGENT_ATTACHMENT_LIMIT - runtime.controller.attachmentsForPeer(peerKey).length);
    if (!available) {
      setError(`You can attach up to ${AGENT_ATTACHMENT_LIMIT} files to one Agent draft.`);
      return;
    }
    runtime.controller.setUploading(peerKey, true);
    runtime.notify();
    const staged = [];
    try {
      for (const file of files.slice(0, available)) {
        const validation = validateAgentAttachment(file);
        if (validation) {
          setError(validation);
          continue;
        }
        const stored = await runtime.uploadAttachment(peerKey, {
          agentId,
          filename: file.name,
          mimeType: file.type || undefined,
          bytesBase64: await agentFileToBase64(file),
        });
        staged.push(await enrichAgentAttachmentPreview(stored, file));
      }
      if (staged.length) runtime.controller.appendAttachments(peerKey, staged);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      runtime.controller.setUploading(peerKey, false);
      runtime.notify();
    }
  }

  function removeAttachment(id: string): void {
    if (!activePeer) return;
    runtime.controller.removeAttachment(activePeer.key, id);
    runtime.notify();
  }

  function regenerate(entry: TranscriptEntry): void {
    if (!activePeer) return;
    const prompt = runtime.transcriptStore.userPromptBefore(activePeer.key, entry.id);
    if (!prompt) {
      setError('No user prompt was found for this response.');
      return;
    }
    runtime.transcriptStore.removeByIds(activePeer.key, [entry.id]);
    runtime.notify();
    runtime.submit({
      peerKey: activePeer.key,
      agentId: activePeer.agentId || activePeer.id,
      ...(activePeer.conversationId ? { conversationId: activePeer.conversationId } : {}),
      prompt: prompt.text,
      ...(prompt.richText ? { richText: prompt.richText } : {}),
      ...(prompt.attachments?.length ? { attachments: prompt.attachments } : {}),
    });
  }

  function edit(entry: TranscriptEntry): void {
    if (!activePeer) return;
    const prompt = entry.role === 'me' ? entry : runtime.transcriptStore.userPromptBefore(activePeer.key, entry.id);
    if (!prompt) return;
    runtime.controller.setDraftDocument(activePeer.key, prompt.text, prompt.richText);
    runtime.notify();
  }

  function onTranscriptScroll(event: React.UIEvent<HTMLDivElement>): void {
    const node = event.currentTarget;
    view.setShowScrollToLatest(node.scrollHeight - node.scrollTop - node.clientHeight > 96);
  }

  function scrollToLatest(): void {
    const node = messageAreaRef.current;
    if (node) node.scrollTop = node.scrollHeight;
    view.setShowScrollToLatest(false);
  }

  function scrollToEntry(entryId: string): void {
    document.querySelector<HTMLElement>(`[data-transcript-entry-id="${CSS.escape(entryId)}"]`)?.scrollIntoView({ block: 'center' });
  }

  function updateHostSetting<K extends keyof ProductHostSettings>(key: K, value: ProductHostSettings[K]): void {
    const next = { ...hostSettings, [key]: value };
    setHostSettings(next);
    void transport.execute({
      type: 'settings.update',
      requestId: requestId('settings-update'),
      settings: next,
    } as Parameters<MahayanaHostTransport['execute']>[0]).catch((cause: unknown) => {
      setError(cause instanceof Error ? cause.message : String(cause));
    });
  }

  const infoVisible = Boolean(activePeer && (view.wideInfoLayout ? view.infoOpen : view.narrowInfoOpen));
  const infoDocked = Boolean(infoVisible && view.wideInfoLayout);
  const collapsed = view.sidebarWidth <= 112;

  const feature = surface === 'contacts'
    ? <ContactsCompatibilityAdapter transport={transport} onClose={() => setSurface('agents')} />
    : surface === 'telegram'
      ? <TelegramCompatibilityAdapter transport={transport} onClose={() => setSurface('agents')} />
      : surface === 'miniapps'
        ? <MiniAppCompatibilityAdapter transport={transport} onClose={() => setSurface('agents')} />
        : surface === 'payments'
          ? <PaymentsCompatibilityAdapter transport={transport} onClose={() => setSurface('agents')} />
          : surface === 'calls'
            ? <CallsCompatibilityAdapter transport={transport} onClose={() => setSurface('agents')} />
            : surface === 'settings'
              ? <SettingsCompatibilityAdapter transport={transport} onClose={() => setSurface('agents')} onLogout={onLogout} />
              : null;

  return <main
    className={styles.root}
    data-testid="messenger-workspace"
    data-agent-root-shell="true"
    data-agent-product-grid="true"
    data-product-shell="agent"
    data-initial-host-hydrated={hostReady && directoryReady ? 'true' : undefined}
    data-sidebar-collapsed={collapsed || undefined}
    data-info-docked={infoDocked || undefined}
  >
    <aside className={styles.sidebar} data-testid="messenger-sidebar" data-collapsed={collapsed || undefined}>
      <AgentSidebar
        agents={agentItems}
        activeKey={activeItem?.key ?? null}
        query={view.search}
        collapsed={collapsed}
        hostReady={hostReady}
        accountLabel="Account"
        sections={product.sidebar.sections}
        selectedKeys={product.sidebar.selectedKeys}
        onQuery={view.setSearch}
        onOpen={openAgent}
        onNewAgent={() => void createAgent()}
        onToggleCollapsed={() => view.setSidebarWidth((width) => width <= 112 ? 300 : 88)}
        onTogglePin={(item) => product.sidebar.togglePin(item.key)}
        onRename={(item) => void renameAgent(item)}
        onHide={(item) => void hideAgent(item)}
        onDuplicate={(item) => void duplicateAgent(item)}
        onDelete={(item) => void deleteAgent(item)}
        onReorderPinned={(moved, target, position) => product.sidebar.reorderPinned(
          moved.key,
          target.key,
          position,
          agentItems.filter((item) => item.pinned).map((item) => item.key),
        )}
        onToggleSelection={(item) => product.sidebar.toggleSelection(item.key)}
        onRangeSelection={(item) => product.sidebar.rangeSelect(item.key, agentItems.map((candidate) => candidate.key))}
        onClearSelection={product.sidebar.clearSelection}
        onDeleteSelected={(items) => void Promise.all(items.map((item) => deleteAgent(item))).then(() => product.sidebar.clearSelection())}
        onMoveSelectedToSection={(items, sectionId) => {
          product.sidebar.moveToSection(items.map((item) => ({ key: item.key, pinned: item.pinned })), sectionId);
          product.sidebar.clearSelection();
        }}
        onCreateSection={(name, items) => {
          product.sidebar.createSection(name, items.map((item) => ({ key: item.key, pinned: item.pinned })));
          product.sidebar.clearSelection();
        }}
        onToggleSection={(section) => product.sidebar.toggleSection(section.id)}
        onRenameSection={(section: AgentSidebarSection) => {
          const name = window.prompt('Rename section', section.name)?.trim();
          if (name) product.sidebar.renameSection(section.id, name);
        }}
        onDeleteSection={(section: AgentSidebarSection) => {
          if (window.confirm(`Delete section “${section.name}”?`)) product.sidebar.removeSection(section.id);
        }}
        onMoveToSection={(item, sectionId) => product.sidebar.moveToSection([{ key: item.key, pinned: item.pinned }], sectionId)}
        onBroadcast={() => {
          setSurface('agents');
          product.network.openBroadcast();
        }}
        onOpenNetwork={() => {
          setSurface('agents');
          product.network.openNetwork();
        }}
        onOpenPlugins={() => setSurface('miniapps')}
        onOpenContacts={() => setSurface('contacts')}
        onOpenTelegram={() => setSurface('telegram')}
        onOpenPayments={() => setSurface('payments')}
        onOpenCalls={() => setSurface('calls')}
        onOpenSettings={() => setSurface('settings')}
      />
    </aside>

    {surface !== 'agents' ? <div className={styles.compatibility}>{feature}</div> : <>
      <AgentCommandPalette
        open={product.palette.open}
        agents={agentItems}
        query={product.palette.query}
        onQuery={product.palette.setQuery}
        onClose={product.palette.close}
        onOpenAgent={openAgent}
        onNewAgent={() => void createAgent()}
        onNetwork={product.network.openNetwork}
        onBroadcast={product.network.openBroadcast}
        onPlugins={() => setSurface('miniapps')}
        onSettings={() => setSurface('settings')}
        entries={activeEntriesAll}
        onOpenTranscriptEntry={scrollToEntry}
        onConversationSearch={() => {
          view.setConversationSearchOpen(true);
          view.setAgentConversationSearch('');
        }}
        onComputer={() => {
          if (!activePeer) return;
          view.setAgentSettingsOpen(false);
          computer.openForAgent(activePeer.agentId || activePeer.id, 'command-palette');
          if (view.wideInfoLayout) view.setInfoOpen(true);
          else view.setNarrowInfoOpen(true);
        }}
      />

      <section className={styles.workspace}>
        <AgentNetwork
          open={product.network.open}
          agents={agentItems}
          groups={product.network.groups}
          peerMessagesByAgentId={product.network.peerMessagesByAgentId}
          activeKey={activeItem?.key ?? null}
          broadcastMode={product.network.broadcastMode}
          onClose={product.network.close}
          onOpenAgent={openAgent}
          onRefreshGroups={product.network.refreshGroups}
          onRefreshPeerHistory={product.network.refreshPeerHistory}
          onCreateGroup={product.network.createGroup}
          onUpdateGroup={product.network.updateGroup}
          onDeleteGroup={product.network.deleteGroup}
          onSendGroup={product.network.sendGroup}
          onSendPeer={product.network.sendPeer}
          onBroadcast={product.network.broadcast}
        />

        {product.network.open ? null : activePeer ? <AgentWorkspace
          title={activePeer.title}
          description={activePeer.subtitle}
          botId={`agent:${activePeer.agentId || activePeer.id}`}
          botState={agentAvatarState(activeBusy, hostReady)}
          status={activeBusy ? 'Working…' : hostReady ? `${activePeer.subtitle} · Online` : 'Connecting…'}
          pinned={activePinned}
          searchActive={view.conversationSearchOpen}
          searchQuery={view.agentConversationSearch}
          computerActive={computer.open}
          infoActive={infoVisible}
          onToggleSearch={() => {
            view.setConversationSearchOpen((open) => !open);
            view.setAgentConversationSearch('');
          }}
          onSearchQuery={view.setAgentConversationSearch}
          onSelectSearchResult={scrollToEntry}
          onToggleComputer={() => {
            view.setAgentSettingsOpen(false);
            computer.toggleForAgent(activePeer.agentId || activePeer.id, 'agent-header');
            if (view.wideInfoLayout) view.setInfoOpen(true);
            else view.setNarrowInfoOpen(true);
          }}
          onTogglePin={() => {
            if (activeItem) product.sidebar.togglePin(activeItem.key);
          }}
          onToggleInfo={() => {
            if (view.wideInfoLayout) view.setInfoOpen((open) => !open);
            else view.setNarrowInfoOpen((open) => !open);
          }}
          entries={activeEntries}
          activeOperationId={activeOperationId}
          hasEarlierMessages={activeEntriesAll.length > activeEntries.length}
          messageAreaRef={messageAreaRef}
          showScrollToLatest={view.showScrollToLatest}
          onLoadEarlier={() => view.setMessageRenderCount((count) => count + 240)}
          onScroll={onTranscriptScroll}
          onScrollToLatest={scrollToLatest}
          onCopyMessage={(entry) => void navigator.clipboard.writeText(entry.text)}
          onRegenerate={regenerate}
          onEdit={edit}
          onResolveApproval={(approvalId, decision) => {
            void runtime.resolveApproval({ approvalId, decision }).catch((cause) => {
              setError(cause instanceof Error ? cause.message : String(cause));
            });
          }}
          notice={error ? <div className={styles.error} role="alert">
            <span>{error}</span>
            <FabIconButton label="Dismiss error" onClick={() => setError(null)}><X size={14} /></FabIconButton>
          </div> : null}
          composerReplyTarget={runtime.controller.replyForPeer(activePeer.key)
            ? {
                id: runtime.controller.replyForPeer(activePeer.key)!.id,
                label: 'Reply',
                text: runtime.controller.replyForPeer(activePeer.key)!.text,
              }
            : undefined}
          onClearComposerReply={() => {
            runtime.controller.clearReply(activePeer.key);
            runtime.notify();
          }}
          composerValue={runtime.controller.draftForPeer(activePeer.key)}
          composerRichText={runtime.controller.richTextForPeer(activePeer.key)}
          composerReady={hostReady}
          composerBusy={activeBusy}
          composerUploading={runtime.controller.isUploading(activePeer.key)}
          composerAttachments={runtime.controller.attachmentsForPeer(activePeer.key)}
          composerMentionCandidates={[
            ...agentItems
              .filter((item) => item.peerKey !== activePeer.key)
              .map((item) => ({
                id: item.agentId,
                name: item.name,
                description: item.description,
                kind: 'agent' as const,
              })),
            ...product.mcp.references.map((reference) => ({
              id: reference.id,
              name: reference.name,
              description: reference.description,
              kind: 'mcp' as const,
            })),
          ]}
          composerWorkflowCandidates={(product.workflow.workflowsByAgentId[activePeer.agentId || activePeer.id] ?? [])
            .filter((workflow) => workflow.isEnabledForAgent)
            .map((workflow) => ({ id: workflow.id, name: workflow.name, description: workflow.description }))}
          composerPullRequestCandidates={product.pullRequests.candidates}
          enterToSend
          onComposerChange={(value, richText) => updateAgentComposer(activePeer.key, value, richText)}
          onComposerMention={(candidate) => {
            runtime.controller.upsertReference(activePeer.key, {
              kind: candidate.kind === 'mcp' ? 'mcp' : 'agent',
              id: candidate.id,
              label: candidate.name,
            });
            runtime.notify();
          }}
          onComposerWorkflowReference={(candidate) => {
            runtime.controller.upsertReference(activePeer.key, {
              kind: 'workflow',
              id: candidate.id,
              label: candidate.name,
            });
            runtime.notify();
          }}
          onComposerSubmit={sendAgentMessage}
          onComposerFiles={(files) => void stageAgentFiles(files)}
          onRemoveComposerAttachment={removeAttachment}
          onStop={() => void runtime.interrupt(activePeer.key).catch((cause) => {
            setError(cause instanceof Error ? cause.message : String(cause));
          })}
        /> : <div className={styles.empty}>
          <FabAvatar identity="fabushi:empty" state={hostReady ? 'idle' : 'offline'} size={72} label="Fabushi" />
          <strong>No Agent selected</strong>
          <p>Create or select an Agent to begin.</p>
        </div>}
      </section>

      {infoVisible && activePeer ? <AgentOverlays
        title={activePeer.title}
        description={activePeer.subtitle}
        botId={`agent:${activePeer.agentId || activePeer.id}`}
        botState={agentAvatarState(activeBusy, hostReady)}
        pinned={activePinned}
        overlay={!view.wideInfoLayout}
        onClose={() => view.wideInfoLayout ? view.setInfoOpen(false) : view.setNarrowInfoOpen(false)}
        onSearch={() => {
          view.setConversationSearchOpen(true);
          view.setAgentConversationSearch('');
        }}
        onTogglePin={() => {
          if (activeItem) product.sidebar.togglePin(activeItem.key);
        }}
        computer={{
          agentId: activePeer.agentId || activePeer.id,
          open: computer.open,
          label: desktopComputerLabel(),
          status: computer.status,
          online: computer.online,
          aiControlEnabled: hostSettings.aiComputerControlEnabled,
          remoteControlEnabled: hostSettings.remoteControlEnabled,
          state: computer.state,
          capabilityStatus: computer.capabilityStatus,
          control: computer.control,
          onToggle: () => {
            view.setAgentSettingsOpen(false);
            computer.toggleForAgent(activePeer.agentId || activePeer.id, 'agent-overlay');
          },
          onRefreshPairingCode: computer.refreshPairingCode,
          onApproveSession: computer.approveSession,
          onDenySession: computer.denySession,
          onDisconnect: computer.disconnect,
          onTakeControl: () => computer.takeControl(
            activePeer.agentId || activePeer.id,
            activeOperationId,
          ),
          onReleaseControl: computer.releaseControl,
          onToggleRemoteControl: () => updateHostSetting('remoteControlEnabled', !hostSettings.remoteControlEnabled),
          onOpenControlPage: () => computer.openControlPage(activePeer.agentId || activePeer.id),
        }}
        settings={{
          agentId: activePeer.agentId || activePeer.id,
          open: view.agentSettingsOpen,
          value: settings.snapshot.value ?? {
            name: activePeer.title,
            title: '',
            description: activePeer.subtitle,
            avatarShape: activeAgent?.avatarShape ?? '',
            avatarColor: activeAgent?.avatarColor ?? '',
            notifyOnUpdatesEnabled: true,
            inferenceProvider: activeAgent?.inferenceProvider ?? 'account-default',
          },
          pending: settings.snapshot.pending,
          error: settings.snapshot.error,
          onToggle: () => {
            computer.close();
            view.setAgentSettingsOpen((open) => !open);
          },
          onUpdateProfile: settings.controller.updateProfile,
          onSetNotifications: settings.controller.setNotifications,
          onSetInferenceProvider: settings.controller.setInferenceProvider,
        }}
      /> : null}
    </>}
  </main>;
}
