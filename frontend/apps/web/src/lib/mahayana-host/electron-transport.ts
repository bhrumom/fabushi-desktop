import type {
  ApprovalResolution,
  AuthProvider,
  AuthProviderId,
  AuthState,
  BrowserLoginAttempt,
  BrowserLoginPollResult,
  BrowserLoginReopenResult,
  CommandAccepted,
  HostConfig,
  HostInfo,
  OAuthAttempt,
  OAuthPollResult,
  RuntimeCommand,
  RuntimeEvent,
} from "./contracts";
import type {
  InstalledPluginList,
  InstalledPluginPointer,
  MahayanaHostTransport,
  MarketplaceBrowseResult,
  MarketplaceReleaseMetadata,
  PluginUiDocument,
  PluginUninstallResult,
  RuntimeEventListener,
} from "./transport";

type MahayanaElectronBridge = {
  contractVersion: number;
  invoke<T>(method: string, params?: Record<string, unknown>): Promise<T>;
  subscribe?(listener: RuntimeEventListener): () => void;
};

type ElectronShellBridge = {
  contractVersion: number;
  notify(title: string, body: string): Promise<boolean | void>;
  openExternal(url: string): Promise<boolean | void>;
  openSystemSettings(pane: "screen-recording" | "accessibility"): Promise<boolean | void>;
  windowFocused(): Promise<boolean>;
  registerMiniAppDocument(pluginId: string, html: string): Promise<string>;
};

declare global {
  interface Window {
    fabushi: ElectronShellBridge;
    mahayana?: MahayanaElectronBridge;
  }
}

const ELECTRON_EDGE_CONTRACT_VERSION = 1;
const CONVERSATION_JOURNAL_KEY = "fabushi.desktop.mahayana-conversation-journal.v1";
const CONVERSATION_JOURNAL_VERSION = 1;
const CONVERSATION_JOURNAL_LIMIT = 80;
const CONVERSATION_MESSAGE_LIMIT = 240;
const CONVERSATION_EQUIVALENCE_WINDOW_MS = 60_000;

export const MAHAYANA_RUNTIME_EVENT_NAME = "fabushi:mahayana-runtime-event";
export const MAHAYANA_COMMAND_EVENT_NAME = "fabushi:mahayana-command";
export const MAHAYANA_ACCOUNT_SESSION_RESET_EVENT = "fabushi:mahayana-account-session-reset";

export type MahayanaCommandBridgeContext = {
  conversationKey?: string;
  conversationId?: string;
  agentId?: string;
};

export type MahayanaCommandBridgeDetail =
  | {
      phase: "dispatch";
      command: RuntimeCommand;
      context?: MahayanaCommandBridgeContext;
    }
  | {
      phase: "accepted";
      command: RuntimeCommand;
      accepted: CommandAccepted;
      context?: MahayanaCommandBridgeContext;
    }
  | {
      phase: "failed";
      command: RuntimeCommand;
      error: string;
      context?: MahayanaCommandBridgeContext;
    };

export type ConversationJournalMessage = {
  id: string;
  role: "user" | "assistant";
  text: string;
  createdAtMs: number;
  streaming?: boolean;
};

type ConversationJournal = {
  version: 1;
  conversations: Record<string, ConversationJournalMessage[]>;
};

const idle = (milliseconds = 10) =>
  new Promise<void>((resolve) => setTimeout(resolve, milliseconds));

function isMiniAppConversation(kind: string): boolean {
  return kind.trim().toLocaleLowerCase() === "miniapp";
}

function miniAppIdForConversation(conversation: { id: string; title: string }): string {
  const exactTitle = conversation.title.trim().toLocaleLowerCase();
  const knownByTitle: Record<string, string> = {
    "bot father": "bot-father",
    "chatgpt 自动确认": "chatgpt-auto-confirm",
    "全球法布施": "global-dharma",
    "法流记忆卡": "faliu-flashcards",
    "平台发布": "platform-publish",
    "hermes 安装器": "hermes-installer",
    "大乘助手": "mahayana-assistant",
  };
  const known = knownByTitle[exactTitle];
  if (known) return known;

  const cleanId = conversation.id.trim();
  const namespaced = cleanId.match(/(?:^|:)([a-z0-9][a-z0-9-]*)$/i)?.[1];
  return namespaced || cleanId;
}

function bridgeContextForCommand(command: RuntimeCommand): MahayanaCommandBridgeContext | undefined {
  if (command.type === "chat.send") {
    const conversationKey = command.conversationId || command.agentId || "mahayana-assistant";
    return {
      conversationKey,
      conversationId: command.conversationId,
      agentId: command.agentId || "mahayana-assistant",
    };
  }
  if (command.type === "conversation.open") {
    return {
      conversationKey: command.conversationId,
      conversationId: command.conversationId,
    };
  }
  return undefined;
}

function normalizeAgentCommand(command: RuntimeCommand): RuntimeCommand {
  if (command.type !== "chat.send") return command;
  return {
    ...command,
    mode: "agent",
  };
}

function dispatchWindowBridgeEvent<T>(name: string, detail: T): void {
  if (typeof window === "undefined" || typeof window.dispatchEvent !== "function") return;
  window.dispatchEvent(new CustomEvent<T>(name, { detail }));
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function eventTimestampMs(timestamp: string): number {
  const parsed = Date.parse(timestamp);
  return Number.isFinite(parsed) ? parsed : Date.now();
}

function isConversationJournalMessage(value: unknown): value is ConversationJournalMessage {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const candidate = value as Partial<ConversationJournalMessage>;
  return (
    typeof candidate.id === "string" &&
    (candidate.role === "user" || candidate.role === "assistant") &&
    typeof candidate.text === "string" &&
    typeof candidate.createdAtMs === "number" &&
    Number.isFinite(candidate.createdAtMs) &&
    (candidate.streaming === undefined || typeof candidate.streaming === "boolean")
  );
}

function emptyConversationJournal(): ConversationJournal {
  return { version: CONVERSATION_JOURNAL_VERSION, conversations: {} };
}

function readConversationJournal(): ConversationJournal {
  if (typeof window === "undefined") return emptyConversationJournal();
  try {
    const parsed = JSON.parse(window.localStorage.getItem(CONVERSATION_JOURNAL_KEY) || "null") as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return emptyConversationJournal();
    const candidate = parsed as Partial<ConversationJournal>;
    if (
      candidate.version !== CONVERSATION_JOURNAL_VERSION ||
      !candidate.conversations ||
      typeof candidate.conversations !== "object" ||
      Array.isArray(candidate.conversations)
    ) {
      return emptyConversationJournal();
    }

    const conversations = Object.fromEntries(
      Object.entries(candidate.conversations)
        .filter(([conversationId, messages]) => Boolean(conversationId) && Array.isArray(messages))
        .map(([conversationId, messages]) => [
          conversationId,
          (messages as unknown[])
            .filter(isConversationJournalMessage)
            .sort((left, right) => left.createdAtMs - right.createdAtMs)
            .slice(-CONVERSATION_MESSAGE_LIMIT),
        ])
        .filter(([, messages]) => (messages as ConversationJournalMessage[]).length > 0)
        .slice(-CONVERSATION_JOURNAL_LIMIT),
    );
    return { version: CONVERSATION_JOURNAL_VERSION, conversations };
  } catch {
    return emptyConversationJournal();
  }
}


export function readCachedConversationMessages(conversationId: string): ConversationJournalMessage[] {
  const id = conversationId.trim();
  if (!id) return [];
  return (readConversationJournal().conversations[id] ?? []).map((message) => ({ ...message }));
}

function persistConversationJournal(journal: ConversationJournal): void {
  if (typeof window === "undefined") return;
  try {
    const conversations = Object.fromEntries(
      Object.entries(journal.conversations)
        .filter(([, messages]) => messages.length > 0)
        .sort(([, left], [, right]) =>
          (right.at(-1)?.createdAtMs ?? 0) - (left.at(-1)?.createdAtMs ?? 0),
        )
        .slice(0, CONVERSATION_JOURNAL_LIMIT)
        .map(([conversationId, messages]) => [
          conversationId,
          messages
            .slice(-CONVERSATION_MESSAGE_LIMIT)
            .map((message) => ({ ...message })),
        ]),
    );
    if (Object.keys(conversations).length === 0) {
      window.localStorage.removeItem(CONVERSATION_JOURNAL_KEY);
      return;
    }
    window.localStorage.setItem(
      CONVERSATION_JOURNAL_KEY,
      JSON.stringify({ version: CONVERSATION_JOURNAL_VERSION, conversations }),
    );
  } catch {
    // Conversation recovery is a local-first cache. Host persistence remains
    // authoritative and storage pressure must never block a live Agent turn.
  }
}

function equivalentConversationMessage(
  left: ConversationJournalMessage,
  right: ConversationJournalMessage,
): boolean {
  if (left.id === right.id) return true;
  return (
    left.role === right.role &&
    left.text === right.text &&
    Math.abs(left.createdAtMs - right.createdAtMs) <= CONVERSATION_EQUIVALENCE_WINDOW_MS
  );
}

function mergeConversationMessages(
  current: ConversationJournalMessage[],
  incoming: ConversationJournalMessage[],
): ConversationJournalMessage[] {
  const merged = current.map((message) => ({ ...message }));
  for (const message of incoming) {
    const index = merged.findIndex((candidate) => equivalentConversationMessage(candidate, message));
    if (index >= 0) {
      merged[index] = {
        ...merged[index],
        ...message,
        id: merged[index].id || message.id,
        createdAtMs: Math.min(merged[index].createdAtMs, message.createdAtMs),
      };
    } else {
      merged.push({ ...message });
    }
  }
  return merged
    .sort((left, right) => left.createdAtMs - right.createdAtMs)
    .slice(-CONVERSATION_MESSAGE_LIMIT);
}

export function isElectronMahayanaHostAvailable(): boolean {
  return (
    typeof window !== "undefined" &&
    window.mahayana?.contractVersion === ELECTRON_EDGE_CONTRACT_VERSION &&
    typeof window.mahayana.invoke === "function"
  );
}

function mahayanaBridge(): MahayanaElectronBridge {
  if (!isElectronMahayanaHostAvailable() || !window.mahayana) {
    throw new Error("Electron Mahayana Host is not available or uses an unsupported bridge contract");
  }
  return window.mahayana;
}

function shellBridge(): ElectronShellBridge {
  if (typeof window === "undefined" || window.fabushi?.contractVersion !== ELECTRON_EDGE_CONTRACT_VERSION) {
    throw new Error("Electron shell bridge is not available or uses an unsupported contract");
  }
  return window.fabushi;
}

export class ElectronMahayanaHostTransport implements MahayanaHostTransport {
  private readonly listeners = new Set<RuntimeEventListener>();
  private readonly miniAppConversations = new Map<string, string>();
  private readonly conversationJournal = readConversationJournal();
  private readonly requestConversations = new Map<string, string>();
  private readonly operationConversations = new Map<string, string>();
  private readonly ignoredRequests = new Set<string>();
  private readonly ignoredOperations = new Set<string>();
  private activeConversationId: string | null = null;
  private suppressUnscopedRuntime = false;
  private closed = false;
  private pumping = false;
  private unsubscribeBridge: (() => void) | null = null;
  private unsubscribeCommandObserver: (() => void) | null = null;
  private unsubscribeAccountSessionReset: (() => void) | null = null;
  private journalPersistTimer: ReturnType<typeof setTimeout> | null = null;
  private discardJournalOnClose = false;

  constructor() {
    if (typeof window !== "undefined") {
      const onAccountSessionReset = () => {
        this.discardJournalOnClose = true;
        this.discardConversationJournal();
      };
      window.addEventListener(MAHAYANA_ACCOUNT_SESSION_RESET_EVENT, onAccountSessionReset);
      this.unsubscribeAccountSessionReset = () => {
        window.removeEventListener(MAHAYANA_ACCOUNT_SESSION_RESET_EVENT, onAccountSessionReset);
      };
    }
  }

  subscribe(listener: RuntimeEventListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  async initialize(_config: HostConfig): Promise<HostInfo> {
    const info = await mahayanaBridge().invoke<HostInfo>("feature.info");
    this.closed = false;
    this.attachCommandObserver();
    this.attachRuntimeEvents();
    // The main process owns the long-lived event pump and may observe the
    // one-shot host.ready event before the renderer subscribes. A successful
    // feature.info handshake proves the Host is ready, so replay that state
    // locally after the push subscription is attached. A later native
    // host.ready event is harmless because readiness updates are idempotent.
    this.dispatchEvent({
      type: "host.ready",
      timestamp: new Date().toISOString(),
      info,
    });
    return info;
  }

  async execute(command: RuntimeCommand): Promise<CommandAccepted> {
    const normalizedCommand = normalizeAgentCommand(command);
    const context = bridgeContextForCommand(normalizedCommand);
    dispatchWindowBridgeEvent<MahayanaCommandBridgeDetail>(MAHAYANA_COMMAND_EVENT_NAME, {
      phase: "dispatch",
      command: normalizedCommand,
      context,
    });

    try {
      let accepted: CommandAccepted;
      if (normalizedCommand.type === "conversation.open") {
        const miniAppId = this.miniAppConversations.get(normalizedCommand.conversationId);
        if (miniAppId) {
          accepted = await mahayanaBridge().invoke<CommandAccepted>("feature.execute", {
            command: {
              type: "miniapp.open",
              requestId: normalizedCommand.requestId,
              miniAppId,
            },
          });
        } else {
          accepted = await mahayanaBridge().invoke<CommandAccepted>("feature.execute", {
            command: normalizedCommand,
          });
        }
      } else {
        accepted = await mahayanaBridge().invoke<CommandAccepted>("feature.execute", {
          command: normalizedCommand,
        });
      }

      dispatchWindowBridgeEvent<MahayanaCommandBridgeDetail>(MAHAYANA_COMMAND_EVENT_NAME, {
        phase: "accepted",
        command: normalizedCommand,
        accepted,
        context,
      });
      if (
        normalizedCommand.type === "conversation.open" &&
        !this.miniAppConversations.has(normalizedCommand.conversationId)
      ) {
        this.refreshConversationList("after-open");
      }
      return accepted;
    } catch (error) {
      dispatchWindowBridgeEvent<MahayanaCommandBridgeDetail>(MAHAYANA_COMMAND_EVENT_NAME, {
        phase: "failed",
        command: normalizedCommand,
        error: errorMessage(error),
        context,
      });
      throw error;
    }
  }

  marketplaceBrowse(query?: string): Promise<MarketplaceBrowseResult> {
    return mahayanaBridge().invoke<MarketplaceBrowseResult>("feature.marketplace.browse", {
      query: query?.trim() || undefined,
      platform: "desktop",
    });
  }

  marketplaceRelease(pluginId: string, version: string): Promise<MarketplaceReleaseMetadata> {
    return mahayanaBridge().invoke<MarketplaceReleaseMetadata>("feature.marketplace.release", { pluginId, version });
  }

  pluginInstall(
    release: Record<string, unknown>,
    platform = "desktop",
  ): Promise<InstalledPluginPointer> {
    return mahayanaBridge().invoke<InstalledPluginPointer>("feature.plugin.install", { release, platform });
  }

  pluginUninstall(pluginId: string): Promise<PluginUninstallResult> {
    return mahayanaBridge().invoke<PluginUninstallResult>("feature.plugin.uninstall", { pluginId });
  }

  pluginRollback(pluginId: string): Promise<InstalledPluginPointer | null> {
    return mahayanaBridge().invoke<InstalledPluginPointer | null>("feature.plugin.rollback", { pluginId });
  }

  pluginActive(pluginId: string): Promise<InstalledPluginPointer | null> {
    return mahayanaBridge().invoke<InstalledPluginPointer | null>("feature.plugin.active", { pluginId });
  }

  pluginListInstalled(): Promise<InstalledPluginList> {
    return mahayanaBridge().invoke<InstalledPluginList>("feature.plugin.listInstalled");
  }

  pluginUiDocument(pluginId: string): Promise<PluginUiDocument> {
    return mahayanaBridge().invoke<PluginUiDocument>("feature.plugin.uiDocument", { pluginId });
  }

  authStatus(): Promise<AuthState> {
    return mahayanaBridge().invoke<AuthState>("feature.auth.status");
  }

  authProviders(): Promise<AuthProvider[]> {
    return mahayanaBridge().invoke<AuthProvider[]>("feature.auth.providers");
  }

  browserLoginStart(): Promise<BrowserLoginAttempt> {
    return mahayanaBridge().invoke<BrowserLoginAttempt>("feature.auth.browserStart");
  }

  browserLoginPoll(attemptId: string): Promise<BrowserLoginPollResult> {
    return mahayanaBridge().invoke<BrowserLoginPollResult>("feature.auth.browserPoll", { attemptId });
  }

  browserLoginCancel(attemptId: string): Promise<BrowserLoginPollResult> {
    return mahayanaBridge().invoke<BrowserLoginPollResult>("feature.auth.browserCancel", { attemptId });
  }

  browserLoginReopen(attemptId: string): Promise<BrowserLoginReopenResult> {
    return mahayanaBridge().invoke<BrowserLoginReopenResult>("feature.auth.browserReopen", { attemptId });
  }

  passwordLogin(username: string, password: string): Promise<AuthState> {
    return mahayanaBridge().invoke<AuthState>("feature.auth.passwordLogin", { username, password });
  }

  oauthStart(provider: AuthProviderId): Promise<OAuthAttempt> {
    return mahayanaBridge().invoke<OAuthAttempt>("feature.auth.oauthStart", { provider });
  }

  oauthPoll(attemptId: string): Promise<OAuthPollResult> {
    return mahayanaBridge().invoke<OAuthPollResult>("feature.auth.oauthPoll", { attemptId });
  }

  logout(): Promise<AuthState> {
    return mahayanaBridge().invoke<AuthState>("feature.auth.logout");
  }

  openExternal(url: string): Promise<void> {
    // The deterministic Rust FeatureHost test backend returns this inert URL
    // for OAuth. Keep the full Electron -> IPC -> Rust login path in E2E
    // without asking the runner OS to launch a browser. Production OAuth URLs
    // still pass through the hardened Electron external-navigation bridge.
    if (
      url.startsWith("about:blank#fabushi-test-oauth-") ||
      url.startsWith("about:blank#fabushi-test-browser-login")
    ) {
      return Promise.resolve();
    }
    return shellBridge().openExternal(url).then(() => undefined);
  }

  openSystemSettings(pane: "screen-recording" | "accessibility"): Promise<void> {
    return shellBridge().openSystemSettings(pane).then(() => undefined);
  }

  windowFocused(): Promise<boolean> {
    return shellBridge().windowFocused();
  }

  showNotification(title: string, body: string): Promise<void> {
    return shellBridge().notify(title, body).then(() => undefined);
  }

  interrupt(operationId: string): Promise<void> {
    return mahayanaBridge().invoke<void>("feature.interrupt", { operationId });
  }

  resolveApproval(resolution: ApprovalResolution): Promise<void> {
    return mahayanaBridge().invoke<void>("feature.approval.resolve", { resolution });
  }

  async close(): Promise<void> {
    this.closed = true;
    this.unsubscribeBridge?.();
    this.unsubscribeBridge = null;
    this.unsubscribeCommandObserver?.();
    this.unsubscribeCommandObserver = null;
    this.unsubscribeAccountSessionReset?.();
    this.unsubscribeAccountSessionReset = null;
    this.miniAppConversations.clear();
    if (this.discardJournalOnClose) this.discardConversationJournal();
    else this.flushConversationJournal();
  }

  private dispatchToListeners(event: RuntimeEvent): void {
    dispatchWindowBridgeEvent<RuntimeEvent>(MAHAYANA_RUNTIME_EVENT_NAME, event);
    for (const listener of this.listeners) listener(event);
  }

  private dispatchEvent(event: RuntimeEvent): void {
    if (event.type === "conversation.listed") {
      const messengerConversations = event.conversations.filter((conversation) => {
        if (!isMiniAppConversation(conversation.kind)) return true;
        this.miniAppConversations.set(
          conversation.id,
          miniAppIdForConversation(conversation),
        );
        return false;
      });
      const normalizedEvent: RuntimeEvent = {
        ...event,
        conversations: messengerConversations,
      };
      this.dispatchToListeners(normalizedEvent);
      return;
    }

    if (event.type === "conversation.opened") {
      this.activeConversationId = event.conversationId;
      const cached = this.conversationJournal.conversations[event.conversationId] ?? [];
      const merged = mergeConversationMessages(cached, event.messages);
      this.setConversationMessages(event.conversationId, merged, true);
      this.dispatchToListeners({ ...event, messages: merged });
      return;
    }

    if (event.type === "chat.message") {
      const conversationId = this.conversationIdForEvent(event.operationId);
      if (conversationId) {
        const createdAtMs = eventTimestampMs(event.timestamp);
        const current = this.conversationJournal.conversations[conversationId] ?? [];
        const streaming = [...current]
          .reverse()
          .find((message) => message.role === "assistant" && message.streaming && message.id.startsWith(`${event.operationId ?? ""}:stream`));
        const baseId = event.operationId
          ? `${event.operationId}:${event.role}`
          : `${event.role}:${createdAtMs}`;
        let id = streaming?.id ?? baseId;
        if (!streaming) {
          let suffix = 1;
          while (current.some((message) => message.id === id)) {
            id = `${baseId}:${suffix}`;
            suffix += 1;
          }
        }
        this.appendConversationMessage(conversationId, {
          id,
          role: event.role,
          text: event.text,
          createdAtMs,
          streaming: false,
        }, true);
        if (event.role === "assistant") {
          this.refreshConversationList("assistant-message");
        }
      }
    } else if (event.type === "chat.delta") {
      const conversationId = this.conversationIdForEvent(event.operationId);
      if (conversationId) {
        const createdAtMs = eventTimestampMs(event.timestamp);
        const current = this.conversationJournal.conversations[conversationId] ?? [];
        const existing = [...current]
          .reverse()
          .find((message) => message.role === "assistant" && message.streaming && message.id.startsWith(`${event.operationId}:stream`));
        const id = existing?.id ?? `${event.operationId}:stream`;
        this.appendConversationMessage(conversationId, {
          id,
          role: "assistant",
          text: `${existing?.text ?? ""}${event.delta}`,
          createdAtMs: existing?.createdAtMs ?? createdAtMs,
          streaming: true,
        }, false);
      }
    }

    this.dispatchToListeners(event);

    if (
      event.type === "operation.completed" ||
      event.type === "operation.failed" ||
      event.type === "operation.interrupted"
    ) {
      if (this.ignoredOperations.delete(event.operationId)) this.refreshUnscopedSuppression();
    }
  }

  private refreshConversationList(reason: string): void {
    if (this.closed) return;
    const requestId = `conversation-list-${reason}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    void mahayanaBridge().invoke<CommandAccepted>("feature.execute", {
      command: { type: "conversation.list", requestId },
    }).catch((error: unknown) => {
      console.error(`Failed to refresh conversation list after ${reason}`, error);
    });
  }

  private attachRuntimeEvents(): void {
    this.unsubscribeBridge?.();
    this.unsubscribeBridge = null;

    const electronBridge = mahayanaBridge();
    if (typeof electronBridge.subscribe === "function") {
      this.unsubscribeBridge = electronBridge.subscribe((event) => this.dispatchEvent(event));
      return;
    }

    // Compatibility fallback for older Tauri/Electron bundles which have not
    // yet adopted the native edge event channel.
    this.startEventPump();
  }

  private attachCommandObserver(): void {
    this.unsubscribeCommandObserver?.();
    if (typeof window === "undefined") return;
    const listener = (event: Event) => {
      const detail = (event as CustomEvent<MahayanaCommandBridgeDetail>).detail;
      if (detail) this.observeCommand(detail);
    };
    window.addEventListener(MAHAYANA_COMMAND_EVENT_NAME, listener);
    this.unsubscribeCommandObserver = () => window.removeEventListener(MAHAYANA_COMMAND_EVENT_NAME, listener);
  }

  private observeCommand(detail: MahayanaCommandBridgeDetail): void {
    const command = detail.command;
    if (command.type === "conversation.open") {
      if (detail.phase === "dispatch") this.activeConversationId = command.conversationId;
      return;
    }
    if (command.type !== "chat.send") return;

    const requestId = command.requestId;
    const isSelfHostedProjection = detail.context?.conversationKey?.startsWith("selfhosted:") ?? false;
    const conversationId = command.conversationId || detail.context?.conversationId || this.activeConversationId;

    if (detail.phase === "dispatch") {
      if (isSelfHostedProjection || !conversationId) {
        this.ignoredRequests.add(requestId);
        this.refreshUnscopedSuppression();
        return;
      }
      this.activeConversationId = conversationId;
      this.requestConversations.set(requestId, conversationId);
      this.appendConversationMessage(conversationId, {
        id: `request:${requestId}:user`,
        role: "user",
        text: command.text,
        createdAtMs: Date.now(),
      }, true);
      return;
    }

    if (detail.phase === "accepted") {
      const operationId = detail.accepted.operationId;
      if (this.ignoredRequests.delete(requestId)) {
        if (operationId) this.ignoredOperations.add(operationId);
        this.refreshUnscopedSuppression();
        return;
      }
      const mappedConversationId = this.requestConversations.get(requestId) || conversationId;
      this.requestConversations.delete(requestId);
      if (operationId && mappedConversationId) {
        this.operationConversations.set(operationId, mappedConversationId);
        this.trimOperationMappings();
      }
      return;
    }

    this.requestConversations.delete(requestId);
    this.ignoredRequests.delete(requestId);
    this.refreshUnscopedSuppression();
  }

  private conversationIdForEvent(operationId?: string): string | null {
    if (operationId) {
      if (this.ignoredOperations.has(operationId)) return null;
      const mapped = this.operationConversations.get(operationId);
      if (mapped) return mapped;
    }
    if (this.suppressUnscopedRuntime) return null;
    return this.activeConversationId;
  }

  private appendConversationMessage(
    conversationId: string,
    message: ConversationJournalMessage,
    persistImmediately: boolean,
  ): void {
    const current = this.conversationJournal.conversations[conversationId] ?? [];
    const merged = mergeConversationMessages(current, [message]);
    this.setConversationMessages(conversationId, merged, persistImmediately);
  }

  private setConversationMessages(
    conversationId: string,
    messages: ConversationJournalMessage[],
    persistImmediately: boolean,
  ): void {
    this.conversationJournal.conversations[conversationId] = messages.slice(-CONVERSATION_MESSAGE_LIMIT);
    if (persistImmediately) this.flushConversationJournal();
    else this.scheduleConversationJournalPersist();
  }

  private scheduleConversationJournalPersist(): void {
    if (this.journalPersistTimer) return;
    this.journalPersistTimer = setTimeout(() => {
      this.journalPersistTimer = null;
      persistConversationJournal(this.conversationJournal);
    }, 120);
  }

  private flushConversationJournal(): void {
    if (this.journalPersistTimer) {
      clearTimeout(this.journalPersistTimer);
      this.journalPersistTimer = null;
    }
    persistConversationJournal(this.conversationJournal);
  }

  private discardConversationJournal(): void {
    if (this.journalPersistTimer) {
      clearTimeout(this.journalPersistTimer);
      this.journalPersistTimer = null;
    }
    this.conversationJournal.conversations = {};
    if (typeof window !== "undefined") {
      try {
        window.localStorage.removeItem(CONVERSATION_JOURNAL_KEY);
      } catch {
        // Logout/session revocation must still clear in-memory account data.
      }
    }
  }

  private refreshUnscopedSuppression(): void {
    this.suppressUnscopedRuntime = this.ignoredRequests.size > 0 || this.ignoredOperations.size > 0;
  }

  private trimOperationMappings(): void {
    while (this.operationConversations.size > 256) {
      const oldest = this.operationConversations.keys().next().value as string | undefined;
      if (!oldest) break;
      this.operationConversations.delete(oldest);
    }
  }

  private startEventPump(): void {
    if (this.pumping) return;
    this.pumping = true;
    void this.pumpEvents();
  }

  private async pumpEvents(): Promise<void> {
    try {
      while (!this.closed) {
        try {
          const event = await mahayanaBridge().invoke<RuntimeEvent | null>("feature.receive");
          if (event) this.dispatchEvent(event);
          else await idle(10);
        } catch (error) {
          if (this.closed) break;
          console.error("Electron Mahayana Host event pump failed", error);
          await idle(100);
        }
      }
    } finally {
      this.pumping = false;
    }
  }
}
