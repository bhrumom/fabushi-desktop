import { siteHref } from "../site-url";
import type {
  ApprovalResolution,
  AuthProvider,
  AuthProviderId,
  AuthState,
  BrowserLoginAttempt,
  BrowserLoginPollResult,
  BrowserLoginReopenResult,
  CommandAccepted,
  ConversationSummary,
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
import { MockMahayanaHostTransport } from "./mock-transport";

type JsonRecord = Record<string, unknown>;

type MahayanaWasmBridge = {
  initialize(): Promise<unknown>;
  createRuntime(configJson?: string): Promise<number>;
  execute(runtimeId: number, commandJson: string): Promise<string>;
  executeProduct(runtimeId: number, commandJson: string): Promise<string>;
  receive(runtimeId: number): Promise<string | null>;
  closeRuntime(runtimeId: number): Promise<void>;
};

declare global {
  interface Window {
    mahayanaWasm?: MahayanaWasmBridge;
  }
}

const MAHAYANA_CONVERSATION_ID = "mahayana-ai:agent:assistant";

function isRecord(value: unknown): value is JsonRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function now(): string {
  return new Date().toISOString();
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

function productData(value: unknown): unknown {
  if (!isRecord(value) || value.ok !== true || !Object.prototype.hasOwnProperty.call(value, "data")) {
    throw new Error(
      isRecord(value) && typeof value.message === "string"
        ? value.message
        : "Mahayana product command failed",
    );
  }
  return value.data;
}

function unwrappedProductData(value: unknown): unknown {
  if (isRecord(value) && value.ok === true) return productData(value);
  return value;
}

function authStateFromProduct(value: unknown): AuthState {
  const valueData = unwrappedProductData(value);
  const data = isRecord(valueData) ? valueData : {};
  const nested = isRecord(data.auth) ? data.auth : data;
  const rawUser = isRecord(nested.user) ? nested.user : undefined;
  const user = rawUser
    ? {
        id:
          typeof rawUser.id === "string" || typeof rawUser.id === "number"
            ? rawUser.id
            : typeof rawUser.userId === "string" || typeof rawUser.userId === "number"
              ? rawUser.userId
              : undefined,
        username: typeof rawUser.username === "string" ? rawUser.username : undefined,
        nickname: typeof rawUser.nickname === "string" ? rawUser.nickname : undefined,
        email: typeof rawUser.email === "string" ? rawUser.email : undefined,
        avatar: typeof rawUser.avatar === "string" ? rawUser.avatar : undefined,
      }
    : undefined;
  return {
    loggedIn: nested.loggedIn === true,
    provider: typeof nested.provider === "string" ? nested.provider : undefined,
    user,
  };
}

function authProviderFromProduct(value: unknown): AuthProvider[] {
  const valueData = unwrappedProductData(value);
  const data = isRecord(valueData) ? valueData : undefined;
  const providers = Array.isArray(data?.providers) ? data.providers : Array.isArray(valueData) ? valueData : [];
  return providers.flatMap((candidate) => {
    if (!isRecord(candidate)) return [];
    const id = candidate.id;
    if (!(["google", "apple", "microsoft", "github"] as string[]).includes(String(id))) return [];
    return [{
      id: id as AuthProviderId,
      displayName: typeof candidate.displayName === "string" ? candidate.displayName : String(id),
      enabled: candidate.enabled !== false,
    }];
  });
}

function capabilityKind(value: unknown): "agent" | "bot" | "plugin" | "miniApp" | "application" | "contact" {
  const normalized = String(value ?? "agent").toLowerCase();
  if (normalized === "miniapp") return "miniApp";
  if (normalized === "bot") return "bot";
  if (normalized === "plugin") return "plugin";
  if (normalized === "application") return "application";
  if (normalized === "contact") return "contact";
  return "agent";
}

function capabilityAvailability(value: unknown): "ready" | "permissionRequired" | "unavailable" {
  const normalized = String(value ?? "ready").toLowerCase();
  if (normalized === "permissionrequired") return "permissionRequired";
  if (normalized === "unavailable") return "unavailable";
  return "ready";
}

function browserStatus(value: unknown): BrowserLoginPollResult["status"] {
  const normalized = String(value ?? "failed").toLowerCase();
  if (normalized === "pending" || normalized === "completed" || normalized === "expired" || normalized === "cancelled") {
    return normalized;
  }
  return "failed";
}

function browserLoginStartFromProduct(value: unknown): BrowserLoginAttempt {
  const data = unwrappedProductData(value);
  if (!isRecord(data) || typeof data.attemptId !== "string" || typeof data.loginUrl !== "string") {
    throw new Error("Mahayana 浏览器登录没有返回有效的登录尝试");
  }
  return {
    attemptId: data.attemptId,
    loginUrl: data.loginUrl,
    expiresAt: typeof data.expiresAt === "number" ? data.expiresAt : undefined,
    pollAfterMs: typeof data.pollAfterMs === "number" ? data.pollAfterMs : undefined,
  };
}

function browserLoginPollFromProduct(value: unknown): BrowserLoginPollResult {
  const data = unwrappedProductData(value);
  if (!isRecord(data)) return { status: "failed" };
  return {
    status: browserStatus(data.status),
    provider: typeof data.provider === "string" ? data.provider : undefined,
    auth: isRecord(data.auth) ? authStateFromProduct(data.auth) : undefined,
  };
}

function browserLoginReopenFromProduct(value: unknown): BrowserLoginReopenResult {
  const data = unwrappedProductData(value);
  if (!isRecord(data)) return { status: "failed" };
  return {
    status: browserStatus(data.status),
    attemptId: typeof data.attemptId === "string" ? data.attemptId : undefined,
    loginUrl: typeof data.loginUrl === "string" ? data.loginUrl : undefined,
    pollAfterMs: typeof data.pollAfterMs === "number" ? data.pollAfterMs : undefined,
  };
}

function runtimeInfoFromStatus(status: JsonRecord): HostInfo {
  const runtimeVersion = typeof status.modelRuntimeVersion === "number"
    ? `mahayana-web-wasm-${status.modelRuntimeVersion}`
    : "mahayana-web-wasm";
  return {
    runtimeVersion,
    protocolVersion: "1",
    platform: "wasm",
  };
}

function runtimeCommand(command: JsonRecord): string {
  return JSON.stringify(command);
}

let wasmBridgePromise: Promise<MahayanaWasmBridge> | null = null;

async function loadWasmBridge(): Promise<MahayanaWasmBridge> {
  if (typeof window === "undefined") {
    throw new Error("Mahayana WebAssembly Host requires a browser window");
  }
  if (window.mahayanaWasm) return window.mahayanaWasm;
  if (!wasmBridgePromise) {
    wasmBridgePromise = (async () => {
      const bootstrapUrl = siteHref("/mahayana-wasm/bootstrap.js");
      await import(/* webpackIgnore: true */ bootstrapUrl);
      const bridge = window.mahayanaWasm;
      if (!bridge) throw new Error("Mahayana WebAssembly bundle did not initialize");
      return bridge;
    })();
  }
  return wasmBridgePromise;
}

function hostConversationSummary(value: unknown): ConversationSummary | null {
  if (!isRecord(value)) return null;
  const peer = isRecord(value.peer) ? value.peer : {};
  const peerType = typeof peer.type === "string" ? peer.type : "agent";
  const kind = peerType === "miniApp" ? "miniapp" : peerType.toLowerCase();
  return {
    id: typeof value.id === "string" ? value.id : "",
    title: typeof value.title === "string" ? value.title : "Mahayana（大乘 AI）",
    kind,
    pinned: value.pinned === true,
    unreadCount: typeof value.unreadCount === "number" ? value.unreadCount : 0,
    updatedAtMs: typeof value.updatedAtMs === "number" ? value.updatedAtMs : 0,
  };
}

function hostMessage(value: unknown): {
  id: string;
  role: "user" | "assistant";
  text: string;
  createdAtMs: number;
} | null {
  if (!isRecord(value)) return null;
  const role = value.role === "user" ? "user" : "assistant";
  return {
    id: typeof value.id === "string" ? value.id : `${role}-${Date.now()}`,
    role,
    text: typeof value.text === "string" ? value.text : "",
    createdAtMs: typeof value.createdAtMs === "number" ? value.createdAtMs : Date.now(),
  };
}

/**
 * Browser production transport.
 *
 * The WebAssembly runtime owns account credentials, conversation history,
 * operation state, model turns, and browser-local Tool calls. The small Mock
 * transport is retained only for the settings/marketplace surfaces that are
 * not yet represented by the WebAssembly protocol; the Mahayana conversation
 * and authentication paths never fall back to it.
 */
export class WasmMahayanaHostTransport implements MahayanaHostTransport {
  private readonly surfaceFallback = new MockMahayanaHostTransport({ authenticated: false });
  private readonly listeners = new Set<RuntimeEventListener>();
  private bridge: MahayanaWasmBridge | null = null;
  private runtimeId: number | null = null;
  private initialized = false;
  private closed = false;
  private receiveTimer: ReturnType<typeof setInterval> | null = null;
  private receivePromise: Promise<void> | null = null;
  private info: HostInfo = {
    runtimeVersion: "mahayana-web-wasm",
    protocolVersion: "1",
    platform: "wasm",
  };

  constructor() {
    this.surfaceFallback.subscribe((event) => {
      if (event.type !== "host.ready" && event.type !== "host.closed") this.emit(event);
    });
  }

  subscribe(listener: RuntimeEventListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  async initialize(config: HostConfig): Promise<HostInfo> {
    if (this.initialized && !this.closed) return this.info;
    this.closed = false;
    this.bridge = await loadWasmBridge();
    await this.bridge.initialize();
    const configuredModel = typeof process !== "undefined"
      ? process.env.NEXT_PUBLIC_MAHAYANA_MODEL?.trim()
      : undefined;
    const configuredResponsesBaseUrl = typeof process !== "undefined"
      ? process.env.NEXT_PUBLIC_MAHAYANA_RESPONSES_BASE_URL?.trim()
      : undefined;
    const configuredProductBaseUrl = typeof process !== "undefined"
      ? process.env.NEXT_PUBLIC_MAHAYANA_API_BASE_URL?.trim()
        || process.env.NEXT_PUBLIC_FABUSHI_API_BASE_URL?.trim()
        || process.env.NEXT_PUBLIC_API_BASE_URL?.trim()
      : undefined;
    this.runtimeId = await this.bridge.createRuntime(JSON.stringify({
      model: configuredModel || undefined,
      responsesBaseUrl: configuredResponsesBaseUrl || undefined,
      productBaseUrl: configuredProductBaseUrl || undefined,
      maxModelTurns: 16,
    }));
    const status = await this.executeWasm({ "@type": "mahayana.runtime.status" });
    this.info = runtimeInfoFromStatus(status);
    await this.surfaceFallback.initialize({ ...config, mode: "test" });
    this.initialized = true;
    this.emit({ type: "host.ready", timestamp: now(), info: this.info });
    await this.drainRuntimeEvents();
    this.receiveTimer = setInterval(() => void this.drainRuntimeEvents(), 30);
    return this.info;
  }

  async execute(command: RuntimeCommand): Promise<CommandAccepted> {
    this.assertReady();
    switch (command.type) {
      case "conversation.list": {
        const response = await this.executeWasm({ "@type": "mahayana.conversation.list" });
        const conversations = Array.isArray(response.data)
          ? response.data.flatMap((candidate) => {
              const summary = hostConversationSummary(candidate);
              return summary ? [summary] : [];
            })
          : [];
        this.emit({ type: "conversation.listed", timestamp: now(), conversations });
        return { requestId: command.requestId };
      }
      case "conversation.open": {
        const response = await this.executeWasm({
          "@type": "mahayana.conversation.history",
          conversationId: command.conversationId,
          limit: 500,
        });
        const messages = isRecord(response) && Array.isArray(response.data)
          ? response.data.flatMap((candidate) => {
              const message = hostMessage(candidate);
              return message ? [message] : [];
            })
          : [];
        this.emit({
          type: "conversation.opened",
          timestamp: now(),
          conversationId: command.conversationId,
          messages,
        });
        return { requestId: command.requestId };
      }
      case "capability.list": {
        const response = await this.executeWasm({
          "@type": "mahayana.capability.list",
          query: command.query || undefined,
        });
        const capabilities = Array.isArray(response.data)
          ? response.data.flatMap((candidate) => {
              if (!isRecord(candidate)) return [];
              return [{
                id: String(candidate.id ?? ""),
                title: String(candidate.title ?? "Mahayana（大乘 AI）"),
                kind: capabilityKind(candidate.kind),
                mention: String(candidate.mention ?? "@agent.mahayana"),
                conversationId: String(candidate.conversationId ?? MAHAYANA_CONVERSATION_ID),
                provider: String(candidate.provider ?? "mahayana-ai"),
                pluginId: typeof candidate.pluginId === "string" ? candidate.pluginId : undefined,
                description: String(candidate.description ?? "大乘共享智能代理"),
                requiredPermissions: Array.isArray(candidate.requiredPermissions) ? candidate.requiredPermissions.filter((item): item is string => typeof item === "string") : [],
                availability: capabilityAvailability(candidate.availability),
                unavailableReason: typeof candidate.unavailableReason === "string" ? candidate.unavailableReason : undefined,
              }];
            })
          : [];
        this.emit({ type: "capability.listed", timestamp: now(), capabilities });
        return { requestId: command.requestId };
      }
      case "chat.send":
        return await this.sendMahayanaMessage(command);
      default:
        return await this.surfaceFallback.execute(command);
    }
  }

  private async sendMahayanaMessage(
    command: Extract<RuntimeCommand, { type: "chat.send" }>,
  ): Promise<CommandAccepted> {
    const conversationId = command.conversationId || MAHAYANA_CONVERSATION_ID;
    if (conversationId !== MAHAYANA_CONVERSATION_ID) {
      return await this.surfaceFallback.execute(command);
    }
    const accepted = await this.executeWasm({
      "@type": "mahayana.conversation.send",
      conversationId,
      text: command.text,
      clientMessageId: command.requestId,
      hidden: false,
    });
    const operationId = typeof accepted.operationId === "string" ? accepted.operationId : "";
    if (!operationId) throw new Error("Mahayana Web Agent did not return an operation id");
    this.emit({
      type: "chat.message",
      timestamp: now(),
      role: "user",
      text: command.text,
      operationId,
    });
    this.emit({
      type: "operation.started",
      timestamp: now(),
      operationId,
      label: "Mahayana 多步骤工作流",
      interruptible: true,
    });
    this.emit({
      type: "model.routed",
      timestamp: now(),
      operationId,
      provider: "dacheng-responses-web-wasm",
      model: typeof process !== "undefined" ? process.env.NEXT_PUBLIC_MAHAYANA_MODEL?.trim() || "deepseek-chat" : "deepseek-chat",
      mode: command.mode ?? "agent",
    });
    await this.drainRuntimeEvents();
    return { requestId: command.requestId, operationId };
  }

  private async executeWasm(command: JsonRecord): Promise<JsonRecord> {
    this.assertReadyOrInitializing();
    if (!this.bridge || this.runtimeId === null) throw new Error("Mahayana WebAssembly runtime is not ready");
    const raw = await this.bridge.execute(this.runtimeId, runtimeCommand(command));
    const parsed: unknown = JSON.parse(raw);
    if (!isRecord(parsed) || parsed.ok !== true || !isRecord(parsed.data)) {
      throw new Error(isRecord(parsed) && typeof parsed.message === "string" ? parsed.message : "Mahayana runtime command failed");
    }
    return parsed.data;
  }

  private async executeProduct(command: JsonRecord): Promise<unknown> {
    this.assertReadyOrInitializing();
    if (!this.bridge || this.runtimeId === null) throw new Error("Mahayana WebAssembly runtime is not ready");
    const raw = await this.bridge.executeProduct(this.runtimeId, runtimeCommand(command));
    return productData(JSON.parse(raw));
  }

  private async drainRuntimeEvents(): Promise<void> {
    if (this.receivePromise || !this.bridge || this.runtimeId === null || this.closed) return;
    this.receivePromise = (async () => {
      if (!this.bridge || this.runtimeId === null) return;
      while (!this.closed) {
        const raw = await this.bridge.receive(this.runtimeId);
        if (!raw) break;
        const parsed: unknown = JSON.parse(raw);
        if (!isRecord(parsed) || parsed.ok !== true) continue;
        this.translateRuntimeEvent(parsed.data);
      }
    })().catch((cause) => {
      if (!this.closed) this.emit({
        type: "operation.failed",
        timestamp: now(),
        operationId: "mahayana-web-runtime",
        code: "runtime_receive_failed",
        message: errorMessage(cause),
      });
    }).finally(() => {
      this.receivePromise = null;
    });
    await this.receivePromise;
  }

  private translateRuntimeEvent(value: unknown): void {
    if (!isRecord(value) || typeof value["@type"] !== "string") return;
    const type = value["@type"];
    switch (type) {
      case "mahayana.runtime.ready":
        return;
      case "mahayana.message.delta":
        if (typeof value.operationId === "string" && typeof value.delta === "string") {
          this.emit({ type: "chat.delta", timestamp: now(), operationId: value.operationId, delta: value.delta });
        }
        return;
      case "mahayana.message.completed": {
        const message = hostMessage(value.message);
        if (message && typeof value.operationId === "string") {
          this.emit({ type: "chat.message", timestamp: now(), role: message.role, text: message.text, operationId: value.operationId });
        }
        return;
      }
      case "mahayana.model.usage.updated": {
        const usage = isRecord(value.usage) ? value.usage : {};
        const last = isRecord(usage.last) ? usage.last : usage;
        if (typeof value.operationId === "string") {
          this.emit({
            type: "usage.updated",
            timestamp: now(),
            operationId: value.operationId,
            inputTokens: Number(last.inputTokens ?? 0),
            cachedInputTokens: Number(last.cachedInputTokens ?? 0),
            outputTokens: Number(last.outputTokens ?? 0),
            reasoningTokens: Number(last.reasoningOutputTokens ?? 0),
            totalTokens: Number(last.totalTokens ?? 0),
          });
        }
        return;
      }
      case "mahayana.agent.activity":
        if (typeof value.operationId === "string") {
          this.emit({
            type: "agent.step",
            timestamp: now(),
            operationId: value.operationId,
            stepId: String(value.stepId ?? `${value.operationId}:step`),
            kind: String(value.kind ?? "agent"),
            title: String(value.title ?? "Mahayana 正在工作"),
            detail: typeof value.detail === "string" ? value.detail : undefined,
            status: value.status === "failed" ? "failed" : value.status === "completed" ? "completed" : "running",
          });
        }
        return;
      case "mahayana.plugin.progress":
        if (typeof value.operationId === "string") {
          this.emit({
            type: "agent.step",
            timestamp: now(),
            operationId: value.operationId,
            stepId: `${String(value.pluginId ?? "plugin")}:${String(value.tool ?? "tool")}`,
            kind: "tool",
            title: String(value.tool ?? "执行工具"),
            detail: typeof value.message === "string" ? value.message : undefined,
            status: Number(value.total ?? 0) > 0 && Number(value.progress ?? 0) >= Number(value.total ?? 0) ? "completed" : "running",
            progress: Number(value.progress ?? 0),
            total: Number(value.total ?? 0),
          });
        }
        return;
      case "mahayana.operation.completed":
        if (typeof value.operationId === "string") this.emit({ type: "operation.completed", timestamp: now(), operationId: value.operationId });
        return;
      case "mahayana.operation.failed":
        if (typeof value.operationId === "string") this.emit({
          type: "operation.failed",
          timestamp: now(),
          operationId: value.operationId,
          code: String(value.code ?? "model_inference_failed"),
          message: String(value.message ?? "Mahayana Agent failed"),
        });
        return;
      case "mahayana.provider.degraded":
        this.emit({
          type: "agent.step",
          timestamp: now(),
          operationId: typeof value.operationId === "string" ? value.operationId : undefined,
          stepId: `provider:${String(value.provider ?? "model")}`,
          kind: "provider",
          title: "模型服务提示",
          detail: String(value.message ?? "模型服务暂时降级"),
          status: "failed",
        });
        return;
      default:
        return;
    }
  }

  async marketplaceBrowse(query?: string): Promise<MarketplaceBrowseResult> { return await this.surfaceFallback.marketplaceBrowse(query); }
  async marketplaceRelease(pluginId: string, version: string): Promise<MarketplaceReleaseMetadata> { return await this.surfaceFallback.marketplaceRelease(pluginId, version); }
  async pluginInstall(release: Record<string, unknown>, platform?: string): Promise<InstalledPluginPointer> { return await this.surfaceFallback.pluginInstall(release, platform); }
  async pluginUninstall(pluginId: string): Promise<PluginUninstallResult> { return await this.surfaceFallback.pluginUninstall(pluginId); }
  async pluginRollback(pluginId: string): Promise<InstalledPluginPointer | null> { return await this.surfaceFallback.pluginRollback(pluginId); }
  async pluginActive(pluginId: string): Promise<InstalledPluginPointer | null> { return await this.surfaceFallback.pluginActive(pluginId); }
  async pluginListInstalled(): Promise<InstalledPluginList> { return await this.surfaceFallback.pluginListInstalled(); }
  async pluginUiDocument(pluginId: string): Promise<PluginUiDocument> { return await this.surfaceFallback.pluginUiDocument(pluginId); }

  async authStatus(): Promise<AuthState> {
    return authStateFromProduct(await this.executeProduct({ "@type": "mahayana.auth.status" }));
  }

  async authProviders(): Promise<AuthProvider[]> {
    return authProviderFromProduct(await this.executeProduct({ "@type": "mahayana.auth.oauth.providers" }));
  }

  async browserLoginStart(): Promise<BrowserLoginAttempt> {
    return browserLoginStartFromProduct(await this.executeProduct({
      "@type": "mahayana.auth.browser.start",
      platform: "web",
      deviceId: "fabushi-web-wasm",
    }));
  }

  async browserLoginPoll(attemptId: string): Promise<BrowserLoginPollResult> {
    return browserLoginPollFromProduct(await this.executeProduct({ "@type": "mahayana.auth.browser.poll", attemptId }));
  }

  async browserLoginCancel(attemptId: string): Promise<BrowserLoginPollResult> {
    return browserLoginPollFromProduct(await this.executeProduct({ "@type": "mahayana.auth.browser.cancel", attemptId }));
  }

  async browserLoginReopen(attemptId: string): Promise<BrowserLoginReopenResult> {
    return browserLoginReopenFromProduct(await this.executeProduct({ "@type": "mahayana.auth.browser.reopen", attemptId }));
  }

  async oauthStart(provider: AuthProviderId): Promise<OAuthAttempt> {
    throw new Error(`请在 Fabushi 浏览器登录页选择 ${provider}，Web Host 不单独发起 OAuth`);
  }

  async oauthPoll(_attemptId: string): Promise<OAuthPollResult> {
    return { status: "failed" };
  }

  async openExternal(url: string): Promise<void> {
    if (url.startsWith("about:blank#fabushi-test-")) return;
    const popup = window.open(url, "fabushi-auth", "popup,width=520,height=720");
    if (!popup) throw new Error("浏览器窗口被拦截，请允许弹出窗口后重试");
  }

  async openSystemSettings(pane: "screen-recording" | "accessibility"): Promise<void> {
    return await this.surfaceFallback.openSystemSettings(pane);
  }

  async windowFocused(): Promise<boolean> { return await this.surfaceFallback.windowFocused(); }
  async showNotification(title: string, body: string): Promise<void> { return await this.surfaceFallback.showNotification(title, body); }

  async passwordLogin(username: string, password: string): Promise<AuthState> {
    return authStateFromProduct(await this.executeProduct({
      "@type": "mahayana.auth.password.login",
      username,
      password,
      deviceId: "fabushi-web-wasm",
    }));
  }

  async logout(): Promise<AuthState> {
    const state = authStateFromProduct(await this.executeProduct({ "@type": "mahayana.auth.logout" }));
    await this.surfaceFallback.logout();
    return state;
  }

  async interrupt(operationId: string): Promise<void> {
    await this.executeWasm({ "@type": "mahayana.operation.interrupt", operationId });
    this.emit({ type: "operation.interrupted", timestamp: now(), operationId });
  }

  async resolveApproval(resolution: ApprovalResolution): Promise<void> {
    await this.surfaceFallback.resolveApproval(resolution);
  }

  async close(): Promise<void> {
    if (this.closed) return;
    this.closed = true;
    if (this.receiveTimer) clearInterval(this.receiveTimer);
    this.receiveTimer = null;
    await this.surfaceFallback.close();
    if (this.bridge && this.runtimeId !== null) await this.bridge.closeRuntime(this.runtimeId);
    this.runtimeId = null;
    this.initialized = false;
    this.emit({ type: "host.closed", timestamp: now() });
  }

  private assertReady(): void {
    if (!this.initialized || this.closed || this.runtimeId === null) throw new Error("Mahayana WebAssembly Host is not ready");
  }

  private assertReadyOrInitializing(): void {
    if (!this.bridge || this.runtimeId === null || this.closed) throw new Error("Mahayana WebAssembly Host is not ready");
  }

  private emit(event: RuntimeEvent): void {
    for (const listener of this.listeners) listener(event);
  }
}
