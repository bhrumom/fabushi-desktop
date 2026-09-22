import type {
  AgentBroadcastResult,
  AsyncTaskSummary,
  AutomationSummary,
  AutomationTrigger,
  BotSummary,
  CommandAccepted,
  ConversationSummary,
  ErrorTray,
  GroupSummary,
  ListenerIntegrationSummary,
  ListenerPlatform,
  ProductHostSettings,
  RuntimeCommand,
  RuntimeEvent,
  SearchMediaMatch,
  SearchMessageMatch,
  SkillSummary,
  SkillTeamSummary,
  SubagentSummary,
  TeachEntryPoint,
  TeachRecordingResult,
  TeachRecordingStatus,
  WorkflowSummary,
  WorkflowTrigger,
} from "./contracts";
import type { MahayanaHostTransport, RuntimeEventListener } from "./transport";
import {
  FabushiWidgetInteractionStore,
  type WidgetDismissalRecord,
  type WidgetResponseRecord,
} from "../fabushi-runtime/widget-interactions";
import {
  FabushiCapabilityProvider,
  type BoxSecretsStatus,
  type CloudAgentInfo,
  type ForeverBoxStatus,
} from "../fabushi-runtime/capability-provider";
import {
  invokeNativeDesktop,
  requestNativeDiskSaverAudit,
  type NativeDiskAudit,
} from "../fabushi-runtime/native-desktop";
import {
  LocalCollaborationProvider,
  PlatformCollaborationProvider,
  type CollaborationEvent,
  type SharedRoomInvite,
  type SharedRoomJoinRequest,
  type SharedRoomSummary,
  type SharingState,
} from "../fabushi-runtime/collaboration";
import {
  FabushiInteractionStore,
  type FeedbackRating,
  type FeedbackRecord,
  type MessageReactionKind,
  type MessageReactionRecord,
} from "../fabushi-runtime/interactions";

type RuntimeEventOf<TType extends RuntimeEvent["type"]> = Extract<RuntimeEvent, { type: TType }>;
type EventMatcher<TEvent extends RuntimeEvent> = (event: RuntimeEvent) => event is TEvent;

type PendingEvent = {
  readonly matches: (event: RuntimeEvent) => boolean;
  readonly resolve: (event: RuntimeEvent) => void;
  readonly reject: (cause: unknown) => void;
  readonly timer: ReturnType<typeof setTimeout>;
};

const DEFAULT_EVENT_TIMEOUT_MS = 15_000;

function normalizedSearch(value: string): string {
  return value.normalize("NFKC").trim().toLocaleLowerCase();
}

function includesSearch(value: string | undefined, query: string): boolean {
  return normalizedSearch(value ?? "").includes(query);
}

function shouldUseLocalCollaborationFallback(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error ?? "");
  return /native desktop bridge is unavailable|transport|host process|econn|enet|network|offline|timed?\s*out|http 5\d\d/i.test(message);
}

function connectorIdForListener(platform: ListenerPlatform): string | null {
  switch (platform) {
    case "slack": return "slack";
    case "github": return "github";
    case "git": return "git";
    case "teams": return "teams";
    case "linear": return "linear";
    case "sentry": return "sentry";
    case "pagerduty": return "pagerduty";
    default: return null;
  }
}

export class MahayanaCoordinator {
  private readonly listeners = new Set<RuntimeEventListener>();
  private readonly pending = new Set<PendingEvent>();
  private unsubscribeTransport: (() => void) | null = null;
  private readonly receipts = new Map<string, CommandAccepted>();
  private readonly interactions = new FabushiInteractionStore();
  private readonly collaboration = new LocalCollaborationProvider();
  private readonly platformCollaboration = new PlatformCollaborationProvider();
  private readonly widgets = new FabushiWidgetInteractionStore();
  private readonly capabilityProvider = new FabushiCapabilityProvider();
  private sequence = 0;
  private disposed = false;

  constructor(private readonly transport: MahayanaHostTransport) {}

  subscribe(listener: RuntimeEventListener): () => void {
    this.assertActive();
    this.ensureTransportSubscription();
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  execute(command: RuntimeCommand): Promise<CommandAccepted> {
    this.assertActive();
    return this.transport.execute(command).then((accepted) => {
      this.receipts.set(accepted.requestId, accepted);
      if (this.receipts.size > 512) {
        const oldest = this.receipts.keys().next().value;
        if (typeof oldest === "string") this.receipts.delete(oldest);
      }
      return accepted;
    });
  }

  promptAcceptanceStatus(requestId: string): CommandAccepted | null {
    return this.receipts.get(requestId) ?? null;
  }

  async respondToWidget(input: {
    widgetId: string;
    actionId?: string | null;
    value?: unknown;
    agentId?: string | null;
    conversationId?: string | null;
    prompt?: string | null;
  }): Promise<WidgetResponseRecord> {
    const record = this.widgets.respond(input);
    if (record.agentId) {
      await this.execute({
        type: "widget.respond",
        requestId: this.requestId("widget-respond"),
        widgetId: record.widgetId,
        agentId: record.agentId,
        ...(record.conversationId ? { conversationId: record.conversationId } : {}),
        ...(record.actionId ? { actionId: record.actionId } : {}),
        ...(record.value !== null ? { value: record.value } : {}),
      });
    } else if (input.prompt?.trim()) {
      throw new Error("Widget prompt routing requires an agent ID");
    }
    return record;
  }

  async dismissWidget(input: {
    widgetId: string;
    agentId?: string | null;
    conversationId?: string | null;
    reason?: string | null;
  }): Promise<WidgetDismissalRecord> {
    const record = this.widgets.dismiss(input);
    if (record.agentId) {
      await this.execute({
        type: "widget.dismiss",
        requestId: this.requestId("widget-dismiss"),
        widgetId: record.widgetId,
        agentId: record.agentId,
        ...(record.conversationId ? { conversationId: record.conversationId } : {}),
        ...(record.reason ? { reason: record.reason } : {}),
      });
    }
    return record;
  }

  kickstartAgent(agentId: string, prompt = "Continue from your current task and report meaningful progress."): Promise<CommandAccepted> {
    const cleanAgentId = agentId.trim();
    if (!cleanAgentId) throw new Error("Agent ID is required");
    const text = prompt.trim();
    if (!text) throw new Error("Kickstart prompt is required");
    return this.sendPrompt({ text, agentId: cleanAgentId });
  }

  requestDiskSaverAudit(): Promise<NativeDiskAudit | null> {
    return requestNativeDiskSaverAudit();
  }

  reactToMessage(input: {
    conversationId: string;
    messageId: string;
    reaction: MessageReactionKind;
    active?: boolean;
  }): MessageReactionRecord {
    return this.interactions.setReaction(input);
  }

  listMessageReactions(conversationId: string, messageId: string): MessageReactionRecord[] {
    return this.interactions.listReactions(conversationId, messageId);
  }

  voteFeedback(input: {
    conversationId?: string | null;
    messageId: string;
    rating: FeedbackRating;
    note?: string | null;
  }): FeedbackRecord {
    return this.interactions.recordFeedback(input);
  }

  feedbackFor(messageId: string): FeedbackRecord | null {
    return this.interactions.feedbackFor(messageId);
  }

  async getCloudAgentInfo(id: string): Promise<CloudAgentInfo> {
    const clean = id.trim();
    if (!clean) throw new Error("Cloud agent or run ID is required");
    try {
      return await invokeNativeDesktop<CloudAgentInfo>("getCloudAgentInfo", {
        bcId: clean,
        includeFiles: false,
      });
    } catch {
      return this.capabilityProvider.getCloudAgentInfo(clean);
    }
  }

  async getForeverBoxStatus(agentId: string): Promise<ForeverBoxStatus> {
    await this.requireAgent(agentId);
    try {
      return await invokeNativeDesktop<ForeverBoxStatus>("getForeverBoxStatus", { agentId });
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.capabilityProvider.getForeverBoxStatus(agentId);
    }
  }

  async ensureForeverBox(agentId: string): Promise<ForeverBoxStatus> {
    await this.requireAgent(agentId);
    try {
      return await invokeNativeDesktop<ForeverBoxStatus>("ensureForeverBox", { agentId });
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.capabilityProvider.ensureForeverBox(agentId);
    }
  }

  async handBackForeverBox(agentId: string): Promise<ForeverBoxStatus> {
    await this.requireAgent(agentId);
    try {
      return await invokeNativeDesktop<ForeverBoxStatus>("handBackForeverBox", { agentId });
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.capabilityProvider.handBackForeverBox(agentId);
    }
  }

  async getBoxSecretsStatus(agentId: string): Promise<BoxSecretsStatus> {
    await this.requireAgent(agentId);
    try {
      return await invokeNativeDesktop<BoxSecretsStatus>("getBoxSecretsStatus", { agentId });
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.capabilityProvider.getBoxSecretsStatus(agentId);
    }
  }

  async isAgentNetworkEnabled(agentId: string): Promise<boolean> {
    await this.requireAgent(agentId);
    try {
      return await invokeNativeDesktop<boolean>("isAgentNetworkEnabled", { agentId });
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.capabilityProvider.isAgentNetworkEnabled(agentId);
    }
  }

  isGlobalSearchEnabled(): boolean {
    return this.capabilityProvider.isGlobalSearchEnabled();
  }

  async isEgressTunnelAvailable(): Promise<boolean> {
    try {
      const status = await invokeNativeDesktop<{ available?: boolean }>("getEgressTunnelStatus");
      return status.available === true;
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.capabilityProvider.isEgressTunnelAvailable();
    }
  }

  async getSharingState(agentId?: string): Promise<SharingState> {
    try {
      return await this.platformCollaboration.refresh(agentId);
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.collaboration.getSharingState(agentId);
    }
  }

  async createRoomFromAgent(agentId: string, name?: string): Promise<SharedRoomSummary> {
    try {
      return await this.platformCollaboration.createRoomFromAgent(agentId, name);
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.collaboration.createRoomFromAgent(agentId, name);
    }
  }

  async createRoomInvite(roomId: string): Promise<SharedRoomInvite> {
    try {
      return await this.platformCollaboration.createRoomInvite(roomId);
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.collaboration.createRoomInvite(roomId);
    }
  }

  async joinSharedRoom(token: string, agentId: string, displayName?: string): Promise<SharedRoomJoinRequest> {
    try {
      return await this.platformCollaboration.joinSharedRoom(token, agentId, displayName);
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.collaboration.joinSharedRoom(token, agentId, displayName);
    }
  }

  async respondToRoomJoinRequest(requestId: string, accept: boolean): Promise<SharedRoomJoinRequest> {
    try {
      return await this.platformCollaboration.respondToRoomJoinRequest(requestId, accept);
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.collaboration.respondToRoomJoinRequest(requestId, accept);
    }
  }

  async createSharedRoom(
    name: string,
    memberAgentIds: readonly string[],
    ownerAgentId: string | null = null,
  ): Promise<SharedRoomSummary> {
    try {
      return await this.platformCollaboration.createSharedRoom(name, memberAgentIds, ownerAgentId);
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.collaboration.createSharedRoom(name, memberAgentIds, ownerAgentId);
    }
  }

  async addOwnAgentToSharedRoom(roomId: string, agentId: string): Promise<SharedRoomSummary> {
    try {
      return await this.platformCollaboration.addOwnAgentToSharedRoom(roomId, agentId);
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.collaboration.addOwnAgentToSharedRoom(roomId, agentId);
    }
  }

  async removeOwnAgentFromSharedRoom(roomId: string, agentId: string): Promise<SharedRoomSummary | null> {
    try {
      return await this.platformCollaboration.removeOwnAgentFromSharedRoom(roomId, agentId);
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.collaboration.removeOwnAgentFromSharedRoom(roomId, agentId);
    }
  }

  async setSharedRoomTyping(roomId: string, participantId: string, isTyping: boolean): Promise<void> {
    try {
      await this.platformCollaboration.setSharedRoomTyping(roomId, participantId, isTyping);
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      this.collaboration.setSharedRoomTyping(roomId, participantId, isTyping);
    }
  }

  async leaveSharedRoom(roomId: string, agentId: string): Promise<SharedRoomSummary | null> {
    try {
      return await this.platformCollaboration.leaveSharedRoom(roomId, agentId);
    } catch (error) {
      if (!shouldUseLocalCollaborationFallback(error)) throw error;
      return this.collaboration.leaveSharedRoom(roomId, agentId);
    }
  }

  subscribeCollaboration(listener: (event: CollaborationEvent) => void): () => void {
    const unsubscribePlatform = this.platformCollaboration.subscribe(listener);
    const unsubscribeLocal = this.collaboration.subscribe(listener);
    return () => {
      unsubscribePlatform();
      unsubscribeLocal();
    };
  }

  requestId(scope: string): string {
    this.sequence += 1;
    const safeScope = scope.replace(/[^a-zA-Z0-9._-]+/g, "-").replace(/^-+|-+$/g, "") || "request";
    return `${safeScope}-${Date.now().toString(36)}-${this.sequence.toString(36)}`;
  }

  async request<TEvent extends RuntimeEvent>(
    command: RuntimeCommand,
    matches: EventMatcher<TEvent>,
    timeoutMs = DEFAULT_EVENT_TIMEOUT_MS,
  ): Promise<TEvent> {
    this.assertActive();
    this.ensureTransportSubscription();
    return new Promise<TEvent>((resolve, reject) => {
      const pending: PendingEvent = {
        matches,
        resolve: (event) => {
          if (!matches(event)) return;
          this.pending.delete(pending);
          clearTimeout(pending.timer);
          resolve(event);
        },
        reject: (cause) => {
          this.pending.delete(pending);
          clearTimeout(pending.timer);
          reject(cause);
        },
        timer: setTimeout(() => {
          pending.reject(new Error(`Timed out waiting for ${command.type}`));
        }, Math.max(1_000, timeoutMs)),
      };
      this.pending.add(pending);
      void this.execute(command).catch((cause) => pending.reject(cause));
    });
  }

  async listConversations(query?: string): Promise<ConversationSummary[]> {
    const event = await this.request(
      { type: "conversation.list", requestId: this.requestId("conversations-list"), query },
      (candidate): candidate is RuntimeEventOf<"conversation.listed"> => candidate.type === "conversation.listed",
    );
    return event.conversations;
  }

  async openConversation(conversationId: string): Promise<RuntimeEventOf<"conversation.opened">> {
    return this.request(
      { type: "conversation.open", requestId: this.requestId("conversation-open"), conversationId },
      (candidate): candidate is RuntimeEventOf<"conversation.opened"> =>
        candidate.type === "conversation.opened" && candidate.conversationId === conversationId,
    );
  }

  async getAgentTranscriptTail(agentId: string, limit = 30): Promise<RuntimeEventOf<"conversation.opened">["messages"]> {
    const bot = (await this.listAgents()).find((candidate) => candidate.id === agentId);
    if (!bot?.conversationId) return [];
    const opened = await this.openConversation(bot.conversationId);
    return opened.messages.slice(-Math.max(1, Math.min(200, limit)));
  }

  async openAgentTail(agentId: string): Promise<RuntimeEventOf<"conversation.opened"> | null> {
    const bot = (await this.listAgents()).find((candidate) => candidate.id === agentId);
    return bot?.conversationId ? this.openConversation(bot.conversationId) : null;
  }

  async getConversationOutline(conversationId: string): Promise<Array<{
    id: string;
    role: "user" | "assistant";
    preview: string;
    createdAtMs: number;
  }>> {
    const opened = await this.openConversation(conversationId);
    return opened.messages.map((message) => ({
      id: message.id,
      role: message.role,
      preview: message.text.replace(/\s+/gu, " ").trim().slice(0, 180),
      createdAtMs: message.createdAtMs,
    }));
  }

  async listAgents(): Promise<BotSummary[]> {
    const event = await this.request(
      { type: "bot.list", requestId: this.requestId("agents-list") },
      (candidate): candidate is RuntimeEventOf<"bot.listed"> => candidate.type === "bot.listed",
    );
    return event.bots;
  }

  async countAgents(): Promise<number> {
    return (await this.listAgents()).length;
  }

  async searchAgents(query: string): Promise<BotSummary[]> {
    const bots = await this.listAgents();
    const needle = normalizedSearch(query);
    if (!needle) return bots;
    return bots.filter((bot) =>
      includesSearch(bot.name, needle)
      || includesSearch(bot.title, needle)
      || includesSearch(bot.description, needle),
    );
  }

  async createAgent(input: {
    name: string;
    description?: string;
    title?: string;
    avatarShape?: string;
    avatarColor?: string;
  }): Promise<BotSummary> {
    const expectedName = normalizedSearch(input.name);
    const event = await this.request(
      { type: "bot.create", requestId: this.requestId("agent-create"), ...input },
      (candidate): candidate is RuntimeEventOf<"bot.changed"> =>
        candidate.type === "bot.changed"
        && candidate.action === "created"
        && normalizedSearch(candidate.bot.name) === expectedName,
    );
    return event.bot;
  }

  async updateAgent(
    id: string,
    patch: Omit<Extract<RuntimeCommand, { type: "bot.update" }>, "type" | "requestId" | "id">,
  ): Promise<BotSummary> {
    const event = await this.request(
      { type: "bot.update", requestId: this.requestId("agent-update"), id, ...patch },
      (candidate): candidate is RuntimeEventOf<"bot.changed"> =>
        candidate.type === "bot.changed"
        && candidate.action === "updated"
        && candidate.bot.id === id,
    );
    return event.bot;
  }

  async deleteAgents(ids: readonly string[]): Promise<void> {
    await Promise.all(ids.map(async (id) => {
      await this.request(
        { type: "bot.delete", requestId: this.requestId("agent-delete"), id },
        (candidate): candidate is RuntimeEventOf<"bot.changed"> =>
          candidate.type === "bot.changed"
          && candidate.action === "deleted"
          && candidate.bot.id === id,
      );
    }));
  }

  async duplicateAgent(id: string): Promise<BotSummary> {
    const event = await this.request(
      { type: "bot.clone", requestId: this.requestId("agent-clone"), id },
      (candidate): candidate is RuntimeEventOf<"bot.changed"> =>
        candidate.type === "bot.changed" && candidate.action === "cloned" && candidate.bot.id !== id,
    );
    return event.bot;
  }

  setAgentHidden(id: string, hidden: boolean): Promise<CommandAccepted> {
    return this.execute({ type: "bot.setHidden", requestId: this.requestId("agent-hidden"), id, hidden });
  }

  setAgentHiddenFromSidebar(id: string, hidden: boolean): Promise<CommandAccepted> {
    return this.setAgentHidden(id, hidden);
  }

  setAgentAvatarBytes(id: string, bytes: Uint8Array, mimeType = "image/png"): Promise<CommandAccepted> {
    if (!new Set(["image/png", "image/jpeg", "image/webp", "image/gif"]).has(mimeType)) {
      throw new Error(`Unsupported avatar MIME type: ${mimeType}`);
    }
    if (!bytes.byteLength || bytes.byteLength > 2 * 1024 * 1024) {
      throw new Error("Avatar must be between 1 byte and 2 MiB");
    }
    let binary = "";
    for (let offset = 0; offset < bytes.length; offset += 0x8000) {
      binary += String.fromCharCode(...bytes.subarray(offset, Math.min(bytes.length, offset + 0x8000)));
    }
    return this.execute({
      type: "bot.update",
      requestId: this.requestId("agent-avatar"),
      id,
      avatar: `data:${mimeType};base64,${btoa(binary)}`,
    });
  }

  async getAgentAvatar(id: string): Promise<string | null> {
    return (await this.listAgents()).find((candidate) => candidate.id === id)?.avatar ?? null;
  }

  setAgentNotificationsEnabled(id: string, notificationsEnabled: boolean): Promise<CommandAccepted> {
    return this.execute({
      type: "bot.update",
      requestId: this.requestId("agent-notifications"),
      id,
      notificationsEnabled,
    });
  }

  setAgentNotifyOnUpdates(id: string, notifyOnUpdates: boolean): Promise<CommandAccepted> {
    return this.execute({
      type: "bot.update",
      requestId: this.requestId("agent-update-notifications"),
      id,
      notifyOnUpdates,
    });
  }

  setAgentUnread(id: string, unread: boolean): Promise<CommandAccepted> {
    return this.execute({
      type: "bot.update",
      requestId: this.requestId("agent-unread"),
      id,
      unread,
    });
  }

  interruptAgentRun(operationId: string): Promise<void> {
    return this.transport.interrupt(operationId);
  }

  async broadcastToAgents(targetIds: readonly string[] | undefined, message: string): Promise<AgentBroadcastResult> {
    const event = await this.request(
      {
        type: "agent.broadcast",
        requestId: this.requestId("agent-broadcast"),
        targetIds: targetIds ? [...targetIds] : undefined,
        message,
      },
      (candidate): candidate is RuntimeEventOf<"agent.broadcasted"> => candidate.type === "agent.broadcasted",
    );
    return event.result;
  }

  resolveAutoReviewApproval(approvalId: string, decision: "allow-once" | "allow-session" | "deny"): Promise<void> {
    return this.transport.resolveApproval({ approvalId, decision });
  }

  resolveLocalToolPermission(approvalId: string, decision: "allow-once" | "allow-session" | "deny"): Promise<void> {
    return this.transport.resolveApproval({ approvalId, decision });
  }

  submitSecret(secretRequestId: string, value: string): Promise<CommandAccepted> {
    return this.execute({
      type: "secret.provide",
      requestId: this.requestId("secret-submit"),
      secretRequestId,
      value,
    });
  }

  async listGroups(): Promise<GroupSummary[]> {
    const event = await this.request(
      { type: "group.list", requestId: this.requestId("groups-list") },
      (candidate): candidate is RuntimeEventOf<"group.listed"> => candidate.type === "group.listed",
    );
    return event.groups;
  }

  async createGroup(input: { name: string; description?: string; memberIds: string[] }): Promise<GroupSummary> {
    const event = await this.request(
      { type: "group.create", requestId: this.requestId("group-create"), ...input },
      (candidate): candidate is RuntimeEventOf<"group.changed"> =>
        candidate.type === "group.changed" && candidate.action === "created",
    );
    return event.group;
  }

  async setGroupMembers(id: string, memberIds: string[]): Promise<GroupSummary> {
    const event = await this.request(
      { type: "group.update", requestId: this.requestId("group-members"), id, memberIds },
      (candidate): candidate is RuntimeEventOf<"group.changed"> =>
        candidate.type === "group.changed" && candidate.group.id === id,
    );
    return event.group;
  }

  async listAutomations(agentId?: string): Promise<AutomationSummary[]> {
    const event = await this.request(
      { type: "automation.list", requestId: this.requestId("automations-list"), agentId },
      (candidate): candidate is RuntimeEventOf<"automation.listed"> => candidate.type === "automation.listed",
    );
    return event.automations;
  }

  upsertAutomation(input: {
    id?: string;
    agentId?: string;
    name: string;
    prompt: string;
    schedule: string;
    trigger?: AutomationTrigger;
    enabled?: boolean;
  }): Promise<CommandAccepted> {
    return this.execute({ type: "automation.upsert", requestId: this.requestId("automation-upsert"), ...input });
  }

  setAutomationEnabled(id: string, enabled: boolean, agentId?: string): Promise<CommandAccepted> {
    return this.execute({ type: "automation.setEnabled", requestId: this.requestId("automation-enabled"), id, agentId, enabled });
  }

  deleteAutomation(id: string, agentId?: string): Promise<CommandAccepted> {
    return this.execute({ type: "automation.delete", requestId: this.requestId("automation-delete"), id, agentId });
  }

  runAutomationNow(id: string, agentId?: string): Promise<CommandAccepted> {
    return this.execute({ type: "automation.run", requestId: this.requestId("automation-run"), id, agentId });
  }

  getAgentAutomations(agentId: string): Promise<AutomationSummary[]> {
    return this.listAutomations(agentId);
  }

  createAgentAutomation(
    agentId: string,
    input: Omit<Extract<RuntimeCommand, { type: "automation.upsert" }>, "type" | "requestId" | "id" | "agentId">,
  ): Promise<CommandAccepted> {
    return this.upsertAutomation({ ...input, agentId });
  }

  updateAgentAutomation(
    agentId: string,
    id: string,
    input: Omit<Extract<RuntimeCommand, { type: "automation.upsert" }>, "type" | "requestId" | "id" | "agentId">,
  ): Promise<CommandAccepted> {
    return this.upsertAutomation({ ...input, id, agentId });
  }

  setAgentAutomationEnabled(agentId: string, id: string, enabled: boolean): Promise<CommandAccepted> {
    return this.setAutomationEnabled(id, enabled, agentId);
  }

  deleteAgentAutomation(agentId: string, id: string): Promise<CommandAccepted> {
    return this.deleteAutomation(id, agentId);
  }

  runAgentAutomationNow(agentId: string, id: string): Promise<CommandAccepted> {
    return this.runAutomationNow(id, agentId);
  }

  listAllAutomations(): Promise<AutomationSummary[]> {
    return this.listAutomations();
  }

  async listWorkflows(agentId: string): Promise<WorkflowSummary[]> {
    const event = await this.request(
      { type: "workflow.list", requestId: this.requestId("workflows-list"), agentId },
      (candidate): candidate is RuntimeEventOf<"workflow.listed"> =>
        candidate.type === "workflow.listed" && candidate.agentId === agentId,
    );
    return event.workflows;
  }

  getAgentWorkflows(agentId: string): Promise<WorkflowSummary[]> {
    return this.listWorkflows(agentId);
  }

  async upsertWorkflow(input: {
    agentId: string;
    id?: string;
    name: string;
    description?: string;
    body: string;
    trigger?: WorkflowTrigger;
    sourceRef?: string;
  }): Promise<WorkflowSummary | undefined> {
    const event = await this.request(
      { type: "workflow.upsert", requestId: this.requestId("workflow-upsert"), ...input },
      (candidate): candidate is RuntimeEventOf<"workflow.changed"> =>
        candidate.type === "workflow.changed"
        && candidate.agentId === input.agentId
        && (input.id == null || candidate.workflow?.id === input.id),
    );
    return event.workflow;
  }

  createAgentWorkflow(input: {
    agentId: string;
    name: string;
    description?: string;
    body: string;
    trigger?: WorkflowTrigger;
    sourceRef?: string;
  }): Promise<WorkflowSummary | undefined> {
    return this.upsertWorkflow(input);
  }

  updateAgentWorkflow(input: {
    agentId: string;
    id: string;
    name: string;
    description?: string;
    body: string;
    trigger?: WorkflowTrigger;
    sourceRef?: string;
  }): Promise<WorkflowSummary | undefined> {
    return this.upsertWorkflow(input);
  }

  setWorkflowEnabled(agentId: string, id: string, enabled: boolean): Promise<CommandAccepted> {
    return this.execute({ type: "workflow.setEnabled", requestId: this.requestId("workflow-enabled"), agentId, id, enabled });
  }

  setAgentWorkflowEnabled(agentId: string, id: string, enabled: boolean): Promise<CommandAccepted> {
    return this.setWorkflowEnabled(agentId, id, enabled);
  }

  deleteWorkflow(agentId: string, id: string): Promise<CommandAccepted> {
    return this.execute({ type: "workflow.delete", requestId: this.requestId("workflow-delete"), agentId, id });
  }

  deleteAgentWorkflow(agentId: string, id: string): Promise<CommandAccepted> {
    return this.deleteWorkflow(agentId, id);
  }

  runWorkflowNow(agentId: string, id: string): Promise<CommandAccepted> {
    return this.execute({ type: "workflow.run", requestId: this.requestId("workflow-run"), agentId, id });
  }

  runAgentWorkflowNow(agentId: string, id: string): Promise<CommandAccepted> {
    return this.runWorkflowNow(agentId, id);
  }

  importWorkflowText(agentId: string, markdown: string, fallbackName?: string): Promise<CommandAccepted> {
    return this.execute({
      type: "workflow.importMarkdown",
      requestId: this.requestId("workflow-import-text"),
      agentId,
      markdown,
      fallbackName,
    });
  }

  importAgentWorkflowText(agentId: string, markdown: string, fallbackName?: string): Promise<CommandAccepted> {
    return this.importWorkflowText(agentId, markdown, fallbackName);
  }

  importWorkflowUrl(agentId: string, source: string, fallbackName?: string): Promise<CommandAccepted> {
    return this.execute({
      type: "workflow.importLiveSource",
      requestId: this.requestId("workflow-import-source"),
      agentId,
      source,
      fallbackName,
    });
  }

  importAgentWorkflowUrl(agentId: string, source: string, fallbackName?: string): Promise<CommandAccepted> {
    return this.importWorkflowUrl(agentId, source, fallbackName);
  }

  async listSkills(agentId?: string): Promise<{ skills: SkillSummary[]; teams: SkillTeamSummary[] }> {
    const event = await this.request(
      { type: "skill.list", requestId: this.requestId("skills-list"), agentId },
      (candidate): candidate is RuntimeEventOf<"skill.listed"> => candidate.type === "skill.listed",
    );
    return { skills: event.skills, teams: event.teams };
  }

  async skillsCatalog(agentId?: string): Promise<SkillSummary[]> {
    return (await this.listSkills(agentId)).skills;
  }

  async portAgentLocalSkills(agentId: string): Promise<{
    agentId: string;
    exportedAtMs: number;
    skills: SkillSummary[];
  }> {
    await this.requireAgent(agentId);
    const { skills } = await this.listSkills(agentId);
    return {
      agentId,
      exportedAtMs: Date.now(),
      skills: skills.filter((skill) => skill.ownerAgentId === agentId && skill.source === "private"),
    };
  }

  async syncPluginSkills(agentId?: string): Promise<{
    attempted: number;
    accepted: number;
    skipped: number;
  }> {
    const { skills } = await this.listSkills(agentId);
    const candidates = skills.filter(
      (skill) => skill.source !== "private" && skill.publishState === "published",
    );
    const results = await Promise.allSettled(
      candidates.map((skill) => this.resyncPublishedSkill(skill.id)),
    );
    const accepted = results.filter((result) => result.status === "fulfilled").length;
    return { attempted: candidates.length, accepted, skipped: skills.length - candidates.length };
  }

  async getPluginSyncStatus(agentId?: string): Promise<{
    total: number;
    synced: number;
    outOfSync: number;
    privateSkills: number;
  }> {
    const { skills } = await this.listSkills(agentId);
    return {
      total: skills.length,
      synced: skills.filter((skill) => skill.publishState === "synced").length,
      outOfSync: skills.filter((skill) => skill.publishState === "published").length,
      privateSkills: skills.filter((skill) => skill.source === "private").length,
    };
  }

  async getSkillPublishTargets(): Promise<Array<{
    id: string;
    name: string;
    kind: "team";
  }>> {
    const { teams } = await this.listSkills();
    return teams.map((team) => ({ id: team.id, name: team.name, kind: "team" as const }));
  }

  publishSkill(id: string, teamId: string): Promise<CommandAccepted> {
    return this.execute({ type: "skill.publish", requestId: this.requestId("skill-publish"), id, teamId });
  }

  resyncPublishedSkill(id: string): Promise<CommandAccepted> {
    return this.execute({ type: "skill.sync", requestId: this.requestId("skill-sync"), id });
  }

  unpublishSkill(id: string): Promise<CommandAccepted> {
    return this.execute({ type: "skill.unpublish", requestId: this.requestId("skill-unpublish"), id });
  }

  async listSubagents(agentId: string): Promise<SubagentSummary[]> {
    const event = await this.request(
      { type: "subagent.list", requestId: this.requestId("subagents-list"), agentId },
      (candidate): candidate is RuntimeEventOf<"subagent.listed"> =>
        candidate.type === "subagent.listed" && candidate.agentId === agentId,
    );
    return event.subagents;
  }

  getSubagents(agentId: string): Promise<SubagentSummary[]> {
    return this.listSubagents(agentId);
  }

  async listAsyncTasks(agentId: string): Promise<AsyncTaskSummary[]> {
    const event = await this.request(
      { type: "asyncTask.list", requestId: this.requestId("tasks-list"), agentId },
      (candidate): candidate is RuntimeEventOf<"asyncTask.listed"> =>
        candidate.type === "asyncTask.listed" && candidate.agentId === agentId,
    );
    return event.tasks;
  }

  getAsyncTasks(agentId: string): Promise<AsyncTaskSummary[]> {
    return this.listAsyncTasks(agentId);
  }

  async getTeachStatus(): Promise<{ status: TeachRecordingStatus; result?: TeachRecordingResult }> {
    const event = await this.request(
      { type: "teach.status", requestId: this.requestId("teach-status") },
      (candidate): candidate is RuntimeEventOf<"teach.changed"> => candidate.type === "teach.changed",
    );
    return { status: event.status, result: event.result };
  }

  getTeachRecordingStatus(): Promise<{ status: TeachRecordingStatus; result?: TeachRecordingResult }> {
    return this.getTeachStatus();
  }

  startTeachRecording(agentId: string, entryPoint: TeachEntryPoint): Promise<CommandAccepted> {
    return this.execute({ type: "teach.start", requestId: this.requestId("teach-start"), agentId, entryPoint });
  }

  stopTeachRecording(agentId: string, save: boolean): Promise<CommandAccepted> {
    return this.execute({ type: "teach.stop", requestId: this.requestId("teach-stop"), agentId, save });
  }

  async listTrays(): Promise<ErrorTray[]> {
    const event = await this.request(
      { type: "tray.list", requestId: this.requestId("trays-list") },
      (candidate): candidate is RuntimeEventOf<"tray.listed"> => candidate.type === "tray.listed",
    );
    return event.trays;
  }

  getTrays(): Promise<ErrorTray[]> {
    return this.listTrays();
  }

  dismissTray(id: string): Promise<CommandAccepted> {
    return this.execute({ type: "tray.dismiss", requestId: this.requestId("tray-dismiss"), id });
  }

  clearTrays(): Promise<CommandAccepted> {
    return this.execute({ type: "tray.clear", requestId: this.requestId("trays-clear") });
  }

  async listListenerIntegrations(): Promise<ListenerIntegrationSummary[]> {
    const event = await this.request(
      { type: "listener.list", requestId: this.requestId("listeners-list") },
      (candidate): candidate is RuntimeEventOf<"listener.listed"> => candidate.type === "listener.listed",
    );
    return event.integrations;
  }

  getListenerIntegrations(): Promise<ListenerIntegrationSummary[]> {
    return this.listListenerIntegrations();
  }

  getAgentChannels(_agentId?: string): Promise<ListenerIntegrationSummary[]> {
    return this.listListenerIntegrations();
  }

  connectChannel(platform: ListenerPlatform): Promise<CommandAccepted> {
    return this.execute({ type: "listener.connect", requestId: this.requestId("channel-connect"), platform });
  }

  async getListenerConnectUrl(platform: ListenerPlatform): Promise<string | null> {
    const connectorId = connectorIdForListener(platform);
    type ConnectOutcome =
      | RuntimeEventOf<"listener.changed">
      | RuntimeEventOf<"connector.changed">
      | RuntimeEventOf<"connector.oauthRequested">;
    const event = await this.request<ConnectOutcome>(
      { type: "listener.connect", requestId: this.requestId("listener-connect-url"), platform },
      (candidate): candidate is ConnectOutcome => {
        if (candidate.type === "listener.changed") return candidate.integration.platform === platform;
        if (!connectorId) return false;
        if (candidate.type === "connector.changed") return candidate.connector.id === connectorId;
        return candidate.type === "connector.oauthRequested" && candidate.connectorId === connectorId;
      },
    );
    return event.type === "connector.oauthRequested" ? event.authorizationUrl : null;
  }

  refreshChannel(platform: ListenerPlatform): Promise<CommandAccepted> {
    return this.connectChannel(platform);
  }

  async disconnectChannel(platform: ListenerPlatform): Promise<ListenerIntegrationSummary> {
    const event = await this.request(
      { type: "listener.disconnect", requestId: this.requestId("channel-disconnect"), platform },
      (candidate): candidate is RuntimeEventOf<"listener.changed"> =>
        candidate.type === "listener.changed"
        && candidate.integration.platform === platform
        && candidate.integration.isConnected === false,
    );
    return event.integration;
  }

  async getSettings(): Promise<ProductHostSettings> {
    const event = await this.request(
      { type: "settings.get", requestId: this.requestId("settings-get") },
      (candidate): candidate is RuntimeEventOf<"settings.changed"> => candidate.type === "settings.changed",
    );
    return event.settings;
  }

  async searchMessages(query: string, limit = 50): Promise<SearchMessageMatch[]> {
    const expected = normalizedSearch(query);
    const event = await this.request(
      { type: "search.messages", requestId: this.requestId("messages-search"), query, limit },
      (candidate): candidate is RuntimeEventOf<"search.messages"> =>
        candidate.type === "search.messages" && normalizedSearch(candidate.query) === expected,
    );
    return event.matches;
  }

  async searchMedia(query = "", limit = 50): Promise<SearchMediaMatch[]> {
    const expected = normalizedSearch(query);
    const event = await this.request(
      { type: "search.media", requestId: this.requestId("media-search"), query, limit },
      (candidate): candidate is RuntimeEventOf<"search.media"> =>
        candidate.type === "search.media" && normalizedSearch(candidate.query) === expected,
    );
    return event.matches;
  }

  sendPrompt(input: Omit<Extract<RuntimeCommand, { type: "chat.send" }>, "type" | "requestId">): Promise<CommandAccepted> {
    return this.execute({ type: "chat.send", requestId: this.requestId("prompt"), ...input });
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.unsubscribeTransport?.();
    this.unsubscribeTransport = null;
    this.collaboration.close();
    this.platformCollaboration.close();
    this.listeners.clear();
    const cause = new Error("Mahayana coordinator disposed");
    for (const pending of [...this.pending]) pending.reject(cause);
    this.pending.clear();
  }

  private async requireAgent(agentId: string): Promise<BotSummary> {
    const clean = agentId.trim();
    if (!clean) throw new Error("Agent ID is required");
    const agent = (await this.listAgents()).find((candidate) => candidate.id === clean);
    if (!agent) throw new Error(`Unknown agent: ${clean}`);
    return agent;
  }

  private ensureTransportSubscription(): void {
    if (this.unsubscribeTransport || this.disposed) return;
    this.unsubscribeTransport = this.transport.subscribe((event) => this.acceptEvent(event));
  }

  private acceptEvent(event: RuntimeEvent): void {
    if (this.disposed) return;
    for (const listener of this.listeners) listener(event);
    for (const pending of [...this.pending]) {
      if (pending.matches(event)) pending.resolve(event);
    }
  }

  private assertActive(): void {
    if (this.disposed) throw new Error("Mahayana coordinator is disposed");
  }
}
