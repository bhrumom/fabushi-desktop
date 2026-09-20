export type HostStatus = "idle" | "initializing" | "ready" | "closed";

export interface HostConfig {
  profileId: string;
  mode: "test" | "production";
}

export interface HostInfo {
  runtimeVersion: string;
  protocolVersion: "1";
  platform: "mock" | "electron" | "tauri" | "wasm" | "flutter";
}

export interface AuthUser {
  id?: string | number;
  username?: string;
  nickname?: string;
  email?: string;
  avatar?: string;
}

export interface AuthState {
  loggedIn: boolean;
  provider?: string;
  user?: AuthUser;
}

export type AuthProviderId = "google" | "apple" | "microsoft" | "github";

export interface AuthProvider {
  id: AuthProviderId;
  displayName: string;
  enabled: boolean;
}

export interface BrowserLoginAttempt {
  attemptId: string;
  loginUrl: string;
  expiresAt?: number;
  pollAfterMs?: number;
}

export interface BrowserLoginPollResult {
  status: "pending" | "completed" | "expired" | "cancelled" | "failed";
  provider?: string;
  auth?: AuthState;
}

export interface BrowserLoginReopenResult {
  status: "pending" | "completed" | "expired" | "cancelled" | "failed";
  attemptId?: string;
  loginUrl?: string;
  pollAfterMs?: number;
}

export interface OAuthAttempt {
  attemptId: string;
  provider: AuthProviderId;
  authorizationUrl: string;
  expiresAt?: number;
}

export interface OAuthPollResult {
  status: "pending" | "completed" | "expired" | "cancelled" | "failed";
  auth?: AuthState;
}

interface CommandBase {
  requestId: string;
}

export type AgentMode = "agent" | "ask" | "plan";

export interface AttachmentContext {
  id: string;
  name: string;
  mimeType?: string;
  text?: string;
  path?: string;
  sizeBytes?: number;
}

export interface AttachmentStored {
  id: string;
  agentId: string;
  name: string;
  path: string;
  mimeType?: string;
  sizeBytes: number;
  hash: string;
}

export interface AttachmentTextResult {
  path: string;
  kind: "text" | "binary";
  text?: string;
  truncated: boolean;
  bytes: number;
}

export interface AttachmentChunkResult {
  path: string;
  bytesBase64: string;
  totalSize: number;
  mime?: string;
}

export interface AttachmentImageResult {
  path: string;
  dataUrl: string;
  width?: number;
  height?: number;
}

export type ListenerPlatform =
  | "slack"
  | "github"
  | "git"
  | "teams"
  | "linear"
  | "sentry"
  | "pagerduty";

export interface ScheduleAutomationTrigger {
  kind: "schedule";
  schedule: string;
}

export interface EventAutomationTrigger {
  kind: "event";
  source: ListenerPlatform;
  event: string;
  filter?: string;
}

export type AutomationTrigger = ScheduleAutomationTrigger | EventAutomationTrigger;

export type ConnectorStatus =
  | "connected"
  | "disconnected"
  | "connecting"
  | "authRequired"
  | "error"
  | "disabledByTeamAdminPolicy";

export interface ConnectorToolSummary {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  requiresApproval?: boolean;
}

export interface ConnectorAccountSummary {
  id: string;
  label: string;
  status: ConnectorStatus;
  email?: string;
  teamManaged?: boolean;
  error?: string;
}

export interface ConnectorSummary {
  id: string;
  displayName: string;
  description: string;
  status: ConnectorStatus;
  isTeam: boolean;
  canAddAccount: boolean;
  transport: "http" | "sse" | "command";
  source?: string;
  teammateCount?: number;
  accounts: ConnectorAccountSummary[];
  tools: ConnectorToolSummary[];
}

export type SkillSource = "private" | "team" | "public";
export type SkillPublishState = "local" | "published" | "synced" | "managed";

export interface SkillSummary {
  id: string;
  name: string;
  description: string;
  useWhen: string;
  instructions: string;
  source: SkillSource;
  publishState: SkillPublishState;
  ownerAgentId?: string;
  teamId?: string;
  teamName?: string;
  readOnly?: boolean;
  updatedAtMs: number;
}

export interface SkillTeamSummary {
  id: string;
  name: string;
}

export interface BotSummary {
  id: string;
  name: string;
  description: string;
  title: string;
  hidden: boolean;
  avatar?: string;
  avatarShape?: string;
  avatarColor?: string;
  notificationsEnabled: boolean;
  notifyOnUpdates: boolean;
  unread: boolean;
  conversationId?: string;
}

export type GroupSpeaker =
  | { kind: "user"; name?: string }
  | { kind: "member"; id: string; name: string };

export interface GroupMessage {
  id: string;
  speaker: GroupSpeaker;
  content: string;
  createdAtMs: number;
}

export interface GroupSummary {
  id: string;
  name: string;
  description: string;
  memberIds: string[];
  messages: GroupMessage[];
  createdAtMs: number;
  updatedAtMs: number;
}

export interface AgentPeerMessage {
  id: string;
  fromAgentId: string;
  fromAgentName: string;
  targetId: string;
  targetName: string;
  text: string;
  priority: boolean;
  createdAtMs: number;
}

export interface AgentBroadcastResult {
  total: number;
  scheduled: number;
}

export type SubagentStatus = "running" | "done" | "error" | "aborted";
export interface SubagentSummary {
  id: string;
  parentAgentId: string;
  subagentType: string;
  title: string;
  status: SubagentStatus;
  startedAtMs: number;
  updatedAtMs: number;
  detail?: string;
}

export type AsyncTaskKind = "subagent" | "shell" | "cloud-agent";
export interface AsyncTaskSummary {
  kind: AsyncTaskKind;
  id: string;
  parentAgentId: string;
  label: string;
  status: "running";
  startedAtMs: number;
  detail?: string;
  subagentType?: string;
  resourceId?: string;
}

export type TeachEntryPoint = "screen_hover" | "composer_menu" | "fullscreen_title_bar";
export interface TeachRecordingStatus {
  state: "idle" | "recording";
  agentId?: string;
  startedAtMs?: number;
  maxDurationMs: number;
}
export interface TeachRecordingResult {
  agentId: string;
  videoPath: string;
  startedAtMs: number;
  endedAtMs: number;
  durationMs: number;
  saved: boolean;
}

export type ComputerControlOrigin = "local-ui" | "remote-mobile" | "ai";
export const COMPUTER_CONTROL_PROTOCOL_VERSION = 1 as const;
export type ComputerTargetKind = "desktop" | "window" | "browser-tab";
export interface ComputerControlTarget {
  protocolVersion: typeof COMPUTER_CONTROL_PROTOCOL_VERSION;
  kind: ComputerTargetKind;
  deviceId?: string;
  windowId?: string;
  browserSessionId?: string;
  browserTargetId?: string;
  tabId?: string;
  generation: number;
}
export const localDesktopComputerTarget = (): ComputerControlTarget => ({
  protocolVersion: COMPUTER_CONTROL_PROTOCOL_VERSION,
  kind: "desktop",
  generation: 0,
});
export type ComputerActionKind = "screenshot" | "click" | "move" | "drag" | "type" | "key" | "scroll" | "wait";
export type ComputerMouseButton = "left" | "right" | "middle";
export type ComputerScrollDirection = "up" | "down" | "left" | "right";
export interface ComputerPoint { x: number; y: number; }
export interface ComputerAction {
  action: ComputerActionKind;
  x?: number;
  y?: number;
  x2?: number;
  y2?: number;
  path?: ComputerPoint[];
  text?: string;
  key?: string;
  button?: ComputerMouseButton;
  count?: number;
  direction?: ComputerScrollDirection;
  amount?: number;
  durationMs?: number;
  description?: string;
}
export interface ComputerStatus {
  platform: string;
  available: boolean;
  captureSupported: boolean;
  inputSupported: boolean;
  accessibilityGranted: boolean;
  screenRecordingGranted: boolean;
  localExecutionEnabled: boolean;
  routeEgressLocally: boolean;
  remoteControlEnabled: boolean;
  aiControlEnabled: boolean;
}
export interface ComputerSnapshot {
  capturedAtMs: number;
  dataUrl: string;
  width?: number;
  height?: number;
}
export interface ComputerActionResult {
  origin: ComputerControlOrigin;
  actionsExecuted: number;
  snapshot: ComputerSnapshot;
}

export type RemoteComputerProvider = "fabushi-webrtc" | "rustdesk-sidecar";
export type RemoteComputerPlatform = "windows" | "macos" | "linux" | "android" | "ios" | "web" | "unknown";
export type RemoteComputerCapability =
  | "remote-desktop"
  | "input"
  | "clipboard"
  | "file-transfer"
  | "display"
  | "audio"
  | "session-management";

export interface RemoteComputerRegistration {
  deviceId: string;
  label: string;
  pairingCode: string;
  pairingExpiresAt: number;
}
export interface RemoteComputerClient {
  clientId: string;
  label: string;
  pairedAt: number;
  lastSeenAt: number;
}
export interface RemoteComputerPermissions {
  display: boolean;
  input: boolean;
  clipboard: boolean;
  fileTransfer: boolean;
  audio: boolean;
}

export interface RemoteComputerSession {
  sessionId: string;
  clientId: string;
  clientLabel?: string;
  state: "pending" | "active" | "closed";
  createdAt?: number;
  expiresAt: number;
  generation?: number;
  permissions: RemoteComputerPermissions;
}
export interface RemoteComputerSignal {
  signalId: number;
  senderRole: "desktop" | "mobile";
  kind: "offer" | "answer" | "ice" | "ready" | "close";
  payload: unknown;
  createdAt: number;
}

export type MemoryKind = "profile" | "log";

export interface MemoryRecord {
  id: string;
  content: string;
  createdAt: number;
  kind: MemoryKind;
}

export type TrayAction =
  | { kind: "open-url"; label: string; url: string }
  | { kind: "switch-model" }
  | { kind: "dashboard-action"; label: string; action: string; args: Record<string, string>; successMessage?: string };

export interface ErrorTray {
  kind: "error";
  id: string;
  agentId: string;
  title: string;
  detail?: string;
  requestId?: string;
  createdAt: number;
  errorKind?: "provider_overloaded";
  rawDetail?: string;
  actions?: TrayAction[];
  dedupeKey?: string;
  count?: number;
}

export type WorkflowSource = "workflow" | "managed" | "plugin" | "automation";

export interface WorkflowTrigger {
  schedule: string;
  isEnabled: boolean;
}

export interface WorkflowSummary {
  id: string;
  name: string;
  description: string;
  body: string;
  trigger?: WorkflowTrigger;
  sourceRef?: string;
  source: WorkflowSource;
  pluginId?: string;
  publishedByCurrentUser: boolean;
  isEnabledForAgent: boolean;
  disableModelInvocation?: boolean;
  scheduleDescription?: string;
  createdAt: number;
  lastRunAt?: number;
  nextRunAt?: number;
  helperScripts: string[];
  filePath: string;
}

export interface SearchMessageMatch {
  agentId: string;
  agentName: string;
  conversationId: string;
  entryId: string;
  role: "user" | "assistant";
  timestampMs: number;
  snippet: string;
}

export interface SearchMediaMatch {
  agentId: string;
  agentName: string;
  path: string;
  name: string;
  mimeType?: string;
  sizeBytes: number;
  timestampMs: number;
}

export type LocalToolPermission = "never" | "ask" | "always";
export type InferenceProvider = "fabushi" | "codex" | "claude-code" | "openrouter";
export type SandboxRuntime = "host" | "local-docker";
export type AutoReviewBehavior = "allow" | "ask";
export interface AutoReviewRule {
  id: string;
  behavior: AutoReviewBehavior;
  text: string;
}
export interface ProductHostSettings {
  notifications: boolean;
  autoUpdateWhenIdle: boolean;
  localExecution: boolean;
  routeEgressLocally: boolean;
  securityKeys: boolean;
  webauthnProxyEnabled: boolean;
  localToolPermission: LocalToolPermission;
  remoteControlEnabled: boolean;
  aiComputerControlEnabled: boolean;
  autoReviewRules: AutoReviewRule[];
  inferenceProvider: InferenceProvider;
  sandboxRuntime: SandboxRuntime;
}

export interface ListenerIntegrationSummary {
  platform: ListenerPlatform;
  displayName: string;
  blurb: string;
  isConnected: boolean;
  accountLabel?: string;
  error?: string;
}

export type DraftSendState =
  | "editable"
  | "sending"
  | "sent"
  | "discarded"
  | "failed";

export interface EmailDraft {
  kind: "email";
  id: string;
  from?: string;
  to: string[];
  cc?: string[];
  subject: string;
  body: string;
  status: DraftSendState;
  error?: string;
}

export interface SlackDraft {
  kind: "slack";
  id: string;
  workspace?: string;
  target: string;
  thread?: string;
  body: string;
  status: DraftSendState;
  error?: string;
}

export type MessageDraft = EmailDraft | SlackDraft;

export interface SpreadsheetSheet {
  name: string;
  rows: string[][];
}

export interface EventCard {
  source: ListenerPlatform;
  event: string;
  title: string;
  summary: string;
  url?: string;
  actor?: string;
  fields?: Array<{ label: string; value: string }>;
  occurredAtMs?: number;
}

export type TranscriptCard =
  | { kind: "emailDraft"; draft: EmailDraft }
  | { kind: "slackDraft"; draft: SlackDraft }
  | {
      kind: "secretRequest";
      requestId: string;
      label: string;
      description?: string;
      provided: boolean;
    }
  | {
      kind: "listenerConnect";
      platform: ListenerPlatform;
      reason?: string;
      connected: boolean;
      pending?: boolean;
    }
  | { kind: "event"; event: EventCard }
  | {
      kind: "pdf";
      name: string;
      url?: string;
      dataBase64?: string;
      pageCount?: number;
    }
  | {
      kind: "spreadsheet";
      name: string;
      sheets: SpreadsheetSheet[];
    }
  | {
      kind: "miniApp";
      miniAppId: string;
      name: string;
      description?: string;
      html: string;
    }
  | {
      kind: "deliverable";
      deliverable: MiniAppDeliverable;
    };

export type UpdateState =
  | { type: "loading" }
  | {
      type: "disabled";
      reason:
        | "not-packaged"
        | "lab-build"
        | "unsupported-platform"
        | "disabled-by-env";
    }
  | { type: "checking" }
  | { type: "available"; version: string; notes?: string }
  | { type: "downloading"; version: string; progress?: number }
  | { type: "staging"; version: string }
  | { type: "ready"; version: string }
  | { type: "upToDate"; version: string }
  | { type: "error"; message: string };

export type RuntimeCommand =
  | (CommandBase & {
      type: "chat.send";
      text: string;
      agentId?: string;
      conversationId?: string;
      mode?: AgentMode;
      modeStatement?: string;
      model?: string;
      attachments?: AttachmentContext[];
    })
  | (CommandBase & { type: "conversation.list"; query?: string })
  | (CommandBase & { type: "conversation.open"; conversationId: string })
  | (CommandBase & { type: "capability.list"; query?: string })
  | (CommandBase & { type: "automation.list"; agentId?: string })
  | (CommandBase & {
      type: "automation.upsert";
      id?: string;
      agentId?: string;
      name: string;
      prompt: string;
      schedule: string;
      trigger?: AutomationTrigger;
      enabled?: boolean;
    })
  | (CommandBase & { type: "automation.setEnabled"; id: string; agentId?: string; enabled: boolean })
  | (CommandBase & { type: "automation.delete"; id: string; agentId?: string })
  | (CommandBase & { type: "automation.run"; id: string; agentId?: string })
  | (CommandBase & { type: "marketplace.install"; miniAppId: string })
  | (CommandBase & { type: "miniapp.open"; miniAppId: string })
  | (CommandBase & {
      type: "capability.request";
      miniAppId: string;
      capability: string;
      reason: string;
    })
  | (CommandBase & { type: "connector.list" })
  | (CommandBase & {
      type: "connector.connect";
      connectorId: string;
      accountLabel?: string;
    })
  | (CommandBase & {
      type: "connector.renameAccount";
      connectorId: string;
      accountId: string;
      label: string;
    })
  | (CommandBase & {
      type: "connector.removeAccount";
      connectorId: string;
      accountId: string;
    })
  | (CommandBase & {
      type: "connector.setToolEnabled";
      connectorId: string;
      toolId: string;
      enabled: boolean;
    })
  | (CommandBase & { type: "skill.list"; agentId?: string })
  | (CommandBase & {
      type: "skill.upsert";
      id?: string;
      name: string;
      description: string;
      useWhen: string;
      instructions: string;
      ownerAgentId?: string;
    })
  | (CommandBase & { type: "skill.delete"; id: string })
  | (CommandBase & { type: "skill.publish"; id: string; teamId: string })
  | (CommandBase & { type: "skill.unpublish"; id: string })
  | (CommandBase & { type: "skill.sync"; id: string })
  | (CommandBase & { type: "bot.list" })
  | (CommandBase & {
      type: "bot.create";
      name: string;
      description?: string;
      title?: string;
      avatar?: string;
      avatarShape?: string;
      avatarColor?: string;
    })
  | (CommandBase & {
      type: "bot.update";
      id: string;
      name?: string;
      description?: string;
      title?: string;
      avatar?: string;
      avatarShape?: string;
      avatarColor?: string;
      notificationsEnabled?: boolean;
      notifyOnUpdates?: boolean;
      unread?: boolean;
    })
  | (CommandBase & { type: "bot.clone"; id: string })
  | (CommandBase & { type: "bot.delete"; id: string })
  | (CommandBase & { type: "bot.setHidden"; id: string; hidden: boolean })
  | (CommandBase & { type: "group.list" })
  | (CommandBase & {
      type: "group.create";
      name: string;
      description?: string;
      memberIds: string[];
    })
  | (CommandBase & {
      type: "group.update";
      id: string;
      name?: string;
      description?: string;
      memberIds?: string[];
    })
  | (CommandBase & { type: "group.delete"; id: string })
  | (CommandBase & { type: "group.send"; id: string; text: string })
  | (CommandBase & { type: "agent.send"; fromAgentId: string; targetId: string; text: string; priority?: boolean })
  | (CommandBase & { type: "agent.broadcast"; targetIds?: string[]; message: string })
  | (CommandBase & { type: "agent.peerHistory"; agentId: string; limit?: number })
  | (CommandBase & { type: "subagent.list"; agentId: string })
  | (CommandBase & { type: "asyncTask.list"; agentId: string })
  | (CommandBase & { type: "teach.status" })
  | (CommandBase & { type: "teach.start"; agentId: string; entryPoint: TeachEntryPoint })
  | (CommandBase & { type: "teach.stop"; agentId: string; save: boolean })
  | (CommandBase & { type: "computer.status" })
  | (CommandBase & { type: "computer.screenshot"; origin?: ComputerControlOrigin; sessionId?: string; target?: ComputerControlTarget })
  | (CommandBase & { type: "computer.action"; origin?: ComputerControlOrigin; agentId?: string; sessionId?: string; target?: ComputerControlTarget; action: ComputerAction; then?: ComputerAction[] })
  | (CommandBase & {
      type: "remoteComputer.register";
      deviceId: string;
      label: string;
      provider: RemoteComputerProvider;
      platform: RemoteComputerPlatform;
      appVersion: string;
      capabilities: RemoteComputerCapability[];
    })
  | (CommandBase & { type: "remoteComputer.heartbeat"; deviceId: string })
  | (CommandBase & { type: "remoteComputer.clients"; deviceId: string })
  | (CommandBase & { type: "remoteComputer.clientRevoke"; deviceId: string; clientId: string })
  | (CommandBase & { type: "remoteComputer.sessions"; deviceId: string })
  | (CommandBase & { type: "remoteComputer.sessionActivate"; deviceId: string; sessionId: string })
  | (CommandBase & { type: "remoteComputer.sessionClose"; deviceId: string; sessionId: string })
  | (CommandBase & { type: "remoteComputer.signal"; deviceId: string; sessionId: string; kind: RemoteComputerSignal["kind"]; payload: unknown })
  | (CommandBase & { type: "remoteComputer.signalDrain"; deviceId: string; sessionId: string; afterSignalId?: number })
  | (CommandBase & { type: "memory.list"; agentId: string; limit?: number })
  | (CommandBase & { type: "memory.add"; agentId: string; content: string; kind: MemoryKind })
  | (CommandBase & { type: "memory.remove"; agentId: string; id: string })
  | (CommandBase & { type: "memory.clear"; agentId: string })
  | (CommandBase & { type: "tray.list" })
  | (CommandBase & { type: "tray.dismiss"; id: string })
  | (CommandBase & { type: "tray.clear" })
  | (CommandBase & { type: "tray.clearForAgent"; agentId: string })
  | (CommandBase & { type: "workflow.list"; agentId: string })
  | (CommandBase & {
      type: "workflow.upsert";
      agentId: string;
      id?: string;
      name: string;
      description?: string;
      body: string;
      trigger?: WorkflowTrigger;
      sourceRef?: string;
    })
  | (CommandBase & { type: "workflow.setEnabled"; agentId: string; id: string; enabled: boolean })
  | (CommandBase & { type: "workflow.delete"; agentId: string; id: string })
  | (CommandBase & { type: "workflow.run"; agentId: string; id: string })
  | (CommandBase & { type: "workflow.importMarkdown"; agentId: string; markdown: string; fallbackName?: string })
  | (CommandBase & { type: "workflow.importLiveSource"; agentId: string; source: string; fallbackName?: string })
  | (CommandBase & { type: "attachment.upload"; agentId: string; filename: string; mimeType?: string; bytesBase64: string })
  | (CommandBase & { type: "attachment.readText"; agentId: string; path: string })
  | (CommandBase & { type: "attachment.readChunk"; agentId: string; path: string; offset: number; length: number })
  | (CommandBase & { type: "attachment.readImage"; agentId: string; path: string })
  | (CommandBase & { type: "search.messages"; query: string; limit?: number })
  | (CommandBase & { type: "search.media"; query?: string; limit?: number })
  | (CommandBase & { type: "mcp.list" })
  | (CommandBase & { type: "mcp.apps" })
  | (CommandBase & { type: "mcp.oauthLogin"; server: string })
  | (CommandBase & { type: "mcp.oauthLogout"; server: string })
  | (CommandBase & { type: "mcp.remove"; server: string })
  | (CommandBase & { type: "mcp.setCustomInstructions"; server: string; instructions: string })
  | (CommandBase & { type: "mcp.setToolDisabled"; server: string; tool: string; disabled: boolean })
  | (CommandBase & { type: "mcp.refresh" })
  | (CommandBase & { type: "mcp.toolCall"; server: string; tool: string; arguments?: unknown })
  | (CommandBase & { type: "settings.get" })
  | (CommandBase & { type: "settings.update"; settings: ProductHostSettings })
  | (CommandBase & { type: "audit.list"; agentId: string; limit?: number })
  | (CommandBase & {
      type: "draft.resolve";
      draft: MessageDraft;
      action: "send" | "discard";
    })
  | (CommandBase & {
      type: "secret.provide";
      secretRequestId: string;
      value: string;
    })
  | (CommandBase & {
      type: "widget.respond";
      widgetId: string;
      agentId: string;
      conversationId?: string;
      actionId?: string;
      value?: unknown;
    })
  | (CommandBase & {
      type: "widget.dismiss";
      widgetId: string;
      agentId: string;
      conversationId?: string;
      reason?: string;
    })
  | (CommandBase & { type: "listener.list" })
  | (CommandBase & { type: "listener.connect"; platform: ListenerPlatform })
  | (CommandBase & { type: "listener.disconnect"; platform: ListenerPlatform })
  | (CommandBase & { type: "update.status" })
  | (CommandBase & { type: "update.check" })
  | (CommandBase & { type: "update.install" })
  | (CommandBase & { type: "runtime.longTask"; label: string })
  | (CommandBase & { type: "session.clear" });

export interface CommandAccepted {
  requestId: string;
  operationId?: string;
}

export interface WidgetInteractionSummary {
  widgetId: string;
  agentId: string;
  conversationId?: string;
  actionId?: string;
  value?: unknown;
  status: "responded" | "dismissed";
  reason?: string;
  createdAtMs: number;
}

export interface ConversationSummary {
  id: string;
  title: string;
  kind: string;
  pinned: boolean;
  unreadCount: number;
  updatedAtMs: number;
}

export interface CapabilitySummary {
  id: string;
  title: string;
  kind: "agent" | "bot" | "plugin" | "miniApp" | "application" | "contact";
  mention: string;
  conversationId: string;
  provider: string;
  pluginId?: string;
  description: string;
  requiredPermissions: string[];
  availability: "ready" | "permissionRequired" | "unavailable";
  unavailableReason?: string;
}

export interface AutomationSummary {
  id: string;
  agentId?: string;
  name: string;
  prompt: string;
  schedule: string;
  trigger?: AutomationTrigger;
  enabled: boolean;
  createdAtMs: number;
  lastRunAtMs?: number;
  nextRunAtMs?: number;
}

interface EventBase {
  timestamp: string;
}

export type RuntimeEvent =
  | (EventBase & { type: "host.ready"; info: HostInfo })
  | (EventBase & {
      type: "chat.message";
      role: "user" | "assistant";
      text: string;
      operationId?: string;
    })
  | (EventBase & {
      type: "chat.delta";
      operationId: string;
      delta: string;
    })
  | (EventBase & {
      type: "transcript.card";
      entryId: string;
      operationId?: string;
      card: TranscriptCard;
    })
  | (EventBase & {
      type: "draft.changed";
      draftId: string;
      status: DraftSendState;
      error?: string;
    })
  | (EventBase & { type: "secret.provided"; secretRequestId: string })
  | (EventBase & { type: "widget.changed"; interaction: WidgetInteractionSummary })
  | (EventBase & {
      type: "conversation.listed";
      conversations: ConversationSummary[];
    })
  | (EventBase & {
      type: "conversation.opened";
      conversationId: string;
      messages: Array<{
        id: string;
        role: "user" | "assistant";
        text: string;
        createdAtMs: number;
      }>;
    })
  | (EventBase & {
      type: "capability.listed";
      capabilities: CapabilitySummary[];
    })
  | (EventBase & {
      type: "automation.listed";
      automations: AutomationSummary[];
    })
  | (EventBase & {
      type: "automation.changed";
      action: "created" | "updated" | "paused" | "resumed" | "deleted" | "running";
      automation: AutomationSummary;
    })
  | (EventBase & { type: "connector.listed"; connectors: ConnectorSummary[] })
  | (EventBase & {
      type: "connector.changed";
      action: "connected" | "disconnected" | "updated" | "removed" | "toolChanged" | "failed";
      connector: ConnectorSummary;
    })
  | (EventBase & {
      type: "connector.oauthRequested";
      connectorId: string;
      authorizationUrl: string;
    })
  | (EventBase & {
      type: "skill.listed";
      skills: SkillSummary[];
      teams: SkillTeamSummary[];
    })
  | (EventBase & {
      type: "skill.changed";
      action: "created" | "updated" | "deleted" | "published" | "unpublished" | "synced";
      skill: SkillSummary;
    })
  | (EventBase & { type: "bot.listed"; bots: BotSummary[] })
  | (EventBase & {
      type: "bot.changed";
      action: "created" | "updated" | "cloned" | "deleted";
      bot: BotSummary;
    })
  | (EventBase & { type: "group.listed"; groups: GroupSummary[] })
  | (EventBase & {
      type: "group.changed";
      action: string;
      group: GroupSummary;
    })
  | (EventBase & {
      type: "group.delta";
      groupId: string;
      memberId: string;
      memberName: string;
      operationId: string;
      delta: string;
    })
  | (EventBase & { type: "agent.peerMessage"; message: AgentPeerMessage })
  | (EventBase & { type: "agent.peerHistory"; agentId: string; messages: AgentPeerMessage[] })
  | (EventBase & { type: "agent.broadcasted"; result: AgentBroadcastResult })
  | (EventBase & { type: "agent.backgroundStarted"; agentId: string; agentName: string; operationId: string; source: string })
  | (EventBase & { type: "agent.backgroundDelta"; agentId: string; agentName: string; operationId: string; source: string; delta: string })
  | (EventBase & { type: "agent.backgroundMessage"; agentId: string; agentName: string; operationId: string; source: string; text: string })
  | (EventBase & { type: "agent.backgroundFinished"; agentId: string; agentName: string; operationId: string; source: string; error?: string })
  | (EventBase & { type: "subagent.listed"; agentId: string; subagents: SubagentSummary[] })
  | (EventBase & { type: "subagent.changed"; subagent: SubagentSummary })
  | (EventBase & { type: "asyncTask.listed"; agentId: string; tasks: AsyncTaskSummary[] })
  | (EventBase & { type: "asyncTask.changed"; agentId: string; tasks: AsyncTaskSummary[] })
  | (EventBase & { type: "teach.changed"; status: TeachRecordingStatus; result?: TeachRecordingResult })
  | (EventBase & { type: "computer.status"; requestId: string; status: ComputerStatus })
  | (EventBase & { type: "computer.snapshot"; requestId: string; origin: ComputerControlOrigin; snapshot: ComputerSnapshot })
  | (EventBase & { type: "computer.result"; requestId: string; result: ComputerActionResult })
  | (EventBase & { type: "remoteComputer.changed"; requestId: string; action: string; data: unknown })
  | (EventBase & { type: "memory.listed"; agentId: string; memories: MemoryRecord[]; count: number; location?: string })
  | (EventBase & { type: "memory.changed"; agentId: string; action: string; memory?: MemoryRecord })
  | (EventBase & { type: "tray.listed"; trays: ErrorTray[] })
  | (EventBase & { type: "tray.changed"; action: "pushed" | "dismissed" | "cleared"; tray?: ErrorTray; id?: string })
  | (EventBase & { type: "workflow.listed"; agentId: string; workflows: WorkflowSummary[] })
  | (EventBase & { type: "workflow.changed"; agentId: string; action: string; workflow?: WorkflowSummary; id?: string })
  | (EventBase & { type: "attachment.stored"; attachment: AttachmentStored })
  | (EventBase & { type: "attachment.text"; result: AttachmentTextResult })
  | (EventBase & { type: "attachment.chunk"; result: AttachmentChunkResult })
  | (EventBase & { type: "attachment.image"; result: AttachmentImageResult })
  | (EventBase & { type: "search.messages"; query: string; matches: SearchMessageMatch[] })
  | (EventBase & { type: "search.media"; query: string; matches: SearchMediaMatch[] })
  | (EventBase & { type: "mcp.listed"; servers: unknown[] })
  | (EventBase & { type: "mcp.apps"; apps: unknown[] })
  | (EventBase & { type: "mcp.oauth"; server: string; authorizationUrl?: string; removed: boolean })
  | (EventBase & { type: "mcp.refreshed" })
  | (EventBase & { type: "mcp.toolResult"; server: string; tool: string; result: unknown })
  | (EventBase & { type: "settings.changed"; settings: ProductHostSettings })
  | (EventBase & { type: "audit.listed"; agentId: string; records: unknown[] })
  | (EventBase & {
      type: "listener.listed";
      integrations: ListenerIntegrationSummary[];
    })
  | (EventBase & {
      type: "listener.changed";
      integration: ListenerIntegrationSummary;
    })
  | (EventBase & { type: "update.changed"; state: UpdateState })
  | (EventBase & {
      type: "agent.step";
      operationId?: string;
      stepId: string;
      kind: string;
      title: string;
      detail?: string;
      status: "running" | "completed" | "failed";
      progress?: number;
      total?: number;
    })
  | (EventBase & {
      type: "model.routed";
      operationId: string;
      provider: string;
      model: string;
      mode: AgentMode;
    })
  | (EventBase & {
      type: "usage.updated";
      operationId: string;
      inputTokens: number;
      cachedInputTokens: number;
      outputTokens: number;
      reasoningTokens: number;
      totalTokens: number;
      contextWindow?: number;
    })
  | (EventBase & {
      type: "marketplace.installed";
      miniAppId: string;
      version: string;
    })
  | (EventBase & { type: "miniapp.opened"; miniAppId: string; html?: string })
  | (EventBase & {
      type: "approval.requested";
      approvalId: string;
      miniAppId: string;
      capability: string;
      reason: string;
      kind?: "command" | "connectedService" | "automation" | "capability";
      subject?: string;
      detail?: string;
      proposedRule?: string;
      location?: "local" | "agent" | "cloud";
    })
  | (EventBase & {
      type: "approval.resolved";
      approvalId: string;
      decision: "allow-once" | "allow-session" | "deny";
    })
  | (EventBase & {
      type: "operation.started";
      operationId: string;
      label: string;
      interruptible: boolean;
    })
  | (EventBase & { type: "operation.interrupted"; operationId: string })
  | (EventBase & { type: "operation.completed"; operationId: string })
  | (EventBase & {
      type: "operation.failed";
      operationId: string;
      code: string;
      message: string;
    })
  | (EventBase & {
      type: "artifact.delivered";
      operationId?: string;
      artifact: MiniAppDeliverable;
    })
  | (EventBase & { type: "session.cleared" })
  | (EventBase & { type: "host.closed" });

export interface MiniAppDeliverable {
  id: string;
  title: string;
  version: string;
  description: string;
  icon?: string;
  entryHtml: string;
  files?: Record<string, string>;
  createdAtMs: number;
}

export type ApprovalRequestedEvent = Extract<
  RuntimeEvent,
  { type: "approval.requested" }
>;

export interface ApprovalResolution {
  approvalId: string;
  decision: "allow-once" | "allow-session" | "deny";
}
