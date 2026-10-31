export interface CoordinatorTranscriptEntry {
  readonly id: string;
  readonly kind: string;
  readonly [key: string]: unknown;
}

export interface CoordinatorTranscriptWindowRequest {
  readonly id: string;
  readonly beforeSeq?: number;
  readonly limit?: number;
}

export interface CoordinatorTranscriptWindowResponse {
  readonly entries: readonly CoordinatorTranscriptEntry[];
  readonly nextBeforeSeq?: number;
  readonly threadCounts: Readonly<Record<string, number>>;
}

export interface CoordinatorAgentThreadRequest {
  readonly id: string;
  readonly rootId: string;
}

export interface CoordinatorAgentThreadResponse {
  readonly entries: readonly CoordinatorTranscriptEntry[];
}

export type TelegramCodeDeliveryType =
  | "telegramMessage"
  | "sms"
  | "call"
  | "flashCall"
  | "missedCall"
  | "email"
  | "fragment"
  | "firebaseAndroid"
  | "firebaseIos"
  | "unknown";

export type TelegramAuthorizationStateProjection =
  | { readonly type: "waitParameters" }
  | { readonly type: "waitPhoneNumber" }
  | {
      readonly type: "waitCode";
      readonly phoneNumber: string;
      readonly deliveryType: TelegramCodeDeliveryType;
      readonly codeLength: number;
      readonly timeoutSeconds?: number | null;
    }
  | { readonly type: "waitOtherDeviceConfirmation"; readonly link: string }
  | { readonly type: "waitRegistration"; readonly termsOfServiceId?: string | null }
  | {
      readonly type: "waitPassword";
      readonly passwordHint: string;
      readonly hasRecoveryEmail: boolean;
      readonly recoveryEmailPattern?: string | null;
    }
  | { readonly type: "ready" }
  | { readonly type: "loggingOut" }
  | { readonly type: "closing" }
  | { readonly type: "closed" };

export interface TelegramAuthIssueProjection {
  readonly code: string;
  readonly category: string;
  readonly action: string;
  readonly recoverable: boolean;
  readonly retryAfterSeconds?: number | null;
}

export interface TelegramAuthorizationProjection {
  readonly authorizationState: TelegramAuthorizationStateProjection;
  readonly selfUserId?: number | null;
  readonly error?: TelegramAuthIssueProjection;
  readonly backoff?: { readonly retryAfterSeconds: number };
  readonly [key: string]: unknown;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isTranscriptEntry(value: unknown): value is CoordinatorTranscriptEntry {
  return isRecord(value) && typeof value.id === "string" && value.id.length > 0 && typeof value.kind === "string" && value.kind.length > 0;
}

function isFiniteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

const TELEGRAM_CODE_DELIVERY_TYPES = new Set<TelegramCodeDeliveryType>([
  "telegramMessage",
  "sms",
  "call",
  "flashCall",
  "missedCall",
  "email",
  "fragment",
  "firebaseAndroid",
  "firebaseIos",
  "unknown",
]);
const MAX_TELEGRAM_AUTH_BACKOFF_SECONDS = 7 * 24 * 60 * 60;

function isBoundedString(value: unknown, maxLength: number): value is string {
  return typeof value === "string" && value.length > 0 && value.length <= maxLength;
}

function isNullableBoundedString(value: unknown, maxLength: number): boolean {
  return value == null || (typeof value === "string" && value.length <= maxLength);
}

function isTelegramAuthorizationState(value: unknown): value is TelegramAuthorizationStateProjection {
  if (!isRecord(value) || typeof value.type !== "string") return false;
  switch (value.type) {
    case "waitParameters":
    case "waitPhoneNumber":
    case "ready":
    case "loggingOut":
    case "closing":
    case "closed":
      return true;
    case "waitCode":
      return isBoundedString(value.phoneNumber, 64)
        && typeof value.deliveryType === "string"
        && TELEGRAM_CODE_DELIVERY_TYPES.has(value.deliveryType as TelegramCodeDeliveryType)
        && typeof value.codeLength === "number"
        && Number.isSafeInteger(value.codeLength)
        && value.codeLength >= 0
        && (value.codeLength as number) <= 64
        && (value.timeoutSeconds == null
          || (typeof value.timeoutSeconds === "number"
            && Number.isSafeInteger(value.timeoutSeconds)
            && value.timeoutSeconds >= 0
            && value.timeoutSeconds <= MAX_TELEGRAM_AUTH_BACKOFF_SECONDS));
    case "waitOtherDeviceConfirmation":
      return isBoundedString(value.link, 4096);
    case "waitRegistration":
      return isNullableBoundedString(value.termsOfServiceId, 1024);
    case "waitPassword":
      return typeof value.passwordHint === "string"
        && value.passwordHint.length <= 1024
        && typeof value.hasRecoveryEmail === "boolean"
        && isNullableBoundedString(value.recoveryEmailPattern, 1024);
    default:
      return false;
  }
}

function isTelegramAuthIssue(value: unknown): value is TelegramAuthIssueProjection {
  if (!isRecord(value)) return false;
  if (!isBoundedString(value.code, 128)
    || !isBoundedString(value.category, 64)
    || !isBoundedString(value.action, 128)
    || typeof value.recoverable !== "boolean") return false;
  return value.retryAfterSeconds == null
    || (typeof value.retryAfterSeconds === "number"
      && Number.isSafeInteger(value.retryAfterSeconds)
      && value.retryAfterSeconds >= 1
      && value.retryAfterSeconds <= MAX_TELEGRAM_AUTH_BACKOFF_SECONDS);
}

export function parseCoordinatorTelegramAuthorizationProjection(value: unknown): TelegramAuthorizationProjection | null {
  if (!isRecord(value) || !isTelegramAuthorizationState(value.authorizationState)) return null;
  if (value.selfUserId !== undefined
    && value.selfUserId !== null
    && (typeof value.selfUserId !== "number" || !Number.isSafeInteger(value.selfUserId) || value.selfUserId <= 0)) return null;
  if (value.error !== undefined && !isTelegramAuthIssue(value.error)) return null;
  if (value.backoff !== undefined) {
    if (!isRecord(value.backoff)
      || typeof value.backoff.retryAfterSeconds !== "number"
      || !Number.isSafeInteger(value.backoff.retryAfterSeconds)
      || value.backoff.retryAfterSeconds < 1
      || value.backoff.retryAfterSeconds > MAX_TELEGRAM_AUTH_BACKOFF_SECONDS) return null;
  }
  return value as unknown as TelegramAuthorizationProjection;
}

export function parseCoordinatorTranscriptWindowResponse(value: unknown): CoordinatorTranscriptWindowResponse | null {
  if (!isRecord(value) || !Array.isArray(value.entries) || !value.entries.every(isTranscriptEntry) || !isRecord(value.threadCounts)) return null;
  const threadCounts: Record<string, number> = {};
  for (const [rootId, count] of Object.entries(value.threadCounts)) {
    if (rootId.length === 0 || !isFiniteNumber(count)) return null;
    threadCounts[rootId] = count;
  }
  if (value.nextBeforeSeq !== undefined && !isFiniteNumber(value.nextBeforeSeq)) return null;
  return {
    entries: value.entries,
    ...(value.nextBeforeSeq === undefined ? {} : { nextBeforeSeq: value.nextBeforeSeq }),
    threadCounts,
  };
}

export function parseCoordinatorTranscriptWindowRequest(value: unknown): CoordinatorTranscriptWindowRequest | null {
  if (!isRecord(value)) return null;
  const id = value.id;
  const beforeSeq = value.beforeSeq;
  const limit = value.limit;
  if (typeof id !== "string" || id.length === 0) return null;
  if (beforeSeq !== undefined && (!isFiniteNumber(beforeSeq) || !Number.isInteger(beforeSeq))) return null;
  if (limit !== undefined && !isFiniteNumber(limit)) return null;
  return {
    id,
    ...(beforeSeq === undefined ? {} : { beforeSeq }),
    ...(limit === undefined ? {} : { limit }),
  };
}

export function parseCoordinatorAgentThreadResponse(value: unknown): CoordinatorAgentThreadResponse | null {
  return isRecord(value) && Array.isArray(value.entries) && value.entries.every(isTranscriptEntry)
    ? { entries: value.entries }
    : null;
}

export function parseCoordinatorAgentThreadRequest(value: unknown): CoordinatorAgentThreadRequest | null {
  return isRecord(value) && typeof value.id === "string" && value.id.length > 0 && typeof value.rootId === "string" && value.rootId.length > 0
    ? { id: value.id, rootId: value.rootId }
    : null;
}

export function validateCoordinatorReply(method: string, value: unknown): unknown {
  if (method.startsWith("feature.auth.telegram") && parseCoordinatorTelegramAuthorizationProjection(value) == null) {
    throw new Error(`${method} returned a malformed Telegram authorization projection`);
  }
  if (method === "getAgentTranscriptWindow" && parseCoordinatorTranscriptWindowResponse(value) == null) {
    throw new Error("getAgentTranscriptWindow returned a malformed transcript window");
  }
  if (method === "getAgentThread" && parseCoordinatorAgentThreadResponse(value) == null) {
    throw new Error("getAgentThread returned a malformed agent thread");
  }
  return value;
}

export const COORDINATOR_METHOD_TABLE = {
  "feature.auth.telegramStatus": { args: "none", reply: "record" },
  "feature.auth.telegramSendCode": { args: "object", reply: "record" },
  "feature.auth.telegramSignIn": { args: "object", reply: "record" },
  "feature.auth.telegramPassword": { args: "object", reply: "record" },
  "feature.auth.telegramRegistration": { args: "object", reply: "record" },
  "feature.auth.telegramResendCode": { args: "object", reply: "record" },
  "feature.auth.telegramCancelCode": { args: "none", reply: "record" },
  "feature.auth.telegramRequestPasswordRecovery": { args: "none", reply: "record" },
  "feature.auth.telegramCheckRecoveryPassword": { args: "object", reply: "record" },
  "feature.auth.telegramSendVerifyEmailCode": { args: "object", reply: "record" },
  "feature.auth.telegramVerifyEmail": { args: "object", reply: "record" },
  "feature.auth.telegramRefreshPasswordChallenge": { args: "none", reply: "record" },
  getAgentTranscriptWindow: { args: "object", reply: "transcript-window" },
  getAgentThread: { args: "object", reply: "agent-thread" },
  getAgentTranscriptTail: { args: "object", reply: "transcript-page" },
  openAgentTail: { args: "object", reply: "transcript-page" },
  sendPrompt: { args: "object", reply: "send-result" },
  interruptAgent: { args: "object", reply: "interrupt-result" },
  promptAcceptanceStatus: { args: "object", reply: "acceptance-lookup" },
  respondToWidget: { args: "object", reply: "record-or-null" },
  resolveAutoReviewApproval: { args: "object", reply: "void" },
  resolveLocalToolPermission: { args: "object", reply: "void" },
  dismissWidget: { args: "object", reply: "record" },
  submitSecret: { args: "object", reply: "void" },
  reactToMessage: { args: "object", reply: "void" },
  listAgents: { args: "none", reply: "array" },
  countAgents: { args: "none", reply: "count" },
  searchAgents: { args: "object", reply: "array" },
  searchMedia: { args: "object", reply: "array" },
  createAgent: { args: "object", reply: "record" },
  createGroup: { args: "object", reply: "record" },
  setGroupMembers: { args: "object", reply: "record-or-null" },
  updateAgent: { args: "object", reply: "record-or-null" },
  deleteAgents: { args: "object", reply: "record" },
  duplicateAgent: { args: "object", reply: "record" },
  kickstartAgent: { args: "object", reply: "record-or-null" },
  requestDiskSaverAudit: { args: "object", reply: "record-or-null" },
  broadcastToAgents: { args: "object", reply: "record" },
  getCloudAgentInfo: { args: "object", reply: "record-or-null" },
  getListenerIntegrations: { args: "none", reply: "record" },
  getListenerConnectUrl: { args: "object", reply: "connect-url" },
  setAgentUnread: { args: "object", reply: "void" },
  setAgentHiddenFromSidebar: { args: "object", reply: "void" },
  setAgentNotificationsEnabled: { args: "object", reply: "void" },
  setAgentNotifyOnUpdates: { args: "object", reply: "void" },
  setAgentAvatarBytes: { args: "object", reply: "record-or-null" },
  getAgentAvatar: { args: "object", reply: "record" },
  getAgentWorkflows: { args: "object", reply: "array" },
  createAgentWorkflow: { args: "object", reply: "array" },
  updateAgentWorkflow: { args: "object", reply: "array" },
  setAgentWorkflowEnabled: { args: "object", reply: "array" },
  deleteAgentWorkflow: { args: "object", reply: "array" },
  runAgentWorkflowNow: { args: "object", reply: "void" },
  importAgentWorkflowText: { args: "object", reply: "import-result" },
  importAgentWorkflowUrl: { args: "object", reply: "import-result" },
  portAgentLocalSkills: { args: "object", reply: "import-result" },
  getConversationOutline: { args: "object", reply: "array" },
  skillsCatalog: { args: "none", reply: "array" },
  syncPluginSkills: { args: "none", reply: "array" },
  getPluginSyncStatus: { args: "none", reply: "record" },
  listRoutedMcpTools: { args: "none", reply: "array" },
  executeRoutedMcpTool: { args: "object", reply: "record" },
  getSkillPublishTargets: { args: "none", reply: "record" },
  publishSkill: { args: "object", reply: "record" },
  resyncPublishedSkill: { args: "object", reply: "record" },
  unpublishSkill: { args: "object", reply: "record" },
  getSubagents: { args: "object", reply: "array" },
  getAsyncTasks: { args: "object", reply: "array" },
  getForeverBoxStatus: { args: "object", reply: "box-status" },
  ensureForeverBox: { args: "object", reply: "box-status" },
  handBackForeverBox: { args: "object", reply: "void" },
  startTeachRecording: { args: "object", reply: "record" },
  stopTeachRecording: { args: "object", reply: "record" },
  getTeachRecordingStatus: { args: "none", reply: "record" },
  getTrays: { args: "none", reply: "array" },
  dismissTray: { args: "object", reply: "void" },
  clearTrays: { args: "none", reply: "void" },
  getAgentChannels: { args: "object", reply: "channels-view" },
  connectChannel: { args: "object", reply: "channels-view" },
  disconnectChannel: { args: "object", reply: "channels-view" },
  refreshChannel: { args: "object", reply: "channels-view" },
  getBoxSecretsStatus: { args: "none", reply: "box-secrets" },
  getAgentAutomations: { args: "object", reply: "array" },
  listAllAutomations: { args: "none", reply: "array" },
  isAgentNetworkEnabled: { args: "none", reply: "boolean" },
  isGlobalSearchEnabled: { args: "none", reply: "boolean" },
  isEgressTunnelAvailable: { args: "none", reply: "boolean" },
  getSharingState: { args: "none", reply: "record" },
  createRoomFromAgent: { args: "object", reply: "record" },
  createRoomInvite: { args: "object", reply: "record" },
  joinSharedRoom: { args: "object", reply: "record" },
  respondToRoomJoinRequest: { args: "object", reply: "record" },
  createSharedRoom: { args: "object", reply: "record" },
  addOwnAgentToSharedRoom: { args: "object", reply: "record" },
  removeOwnAgentFromSharedRoom: { args: "object", reply: "record" },
  setSharedRoomTyping: { args: "object", reply: "void" },
  leaveSharedRoom: { args: "object", reply: "record" },
  setAgentAutomationEnabled: { args: "object", reply: "array" },
  createAgentAutomation: { args: "object", reply: "array" },
  updateAgentAutomation: { args: "object", reply: "array" },
  deleteAgentAutomation: { args: "object", reply: "array" },
  runAgentAutomationNow: { args: "object", reply: "void" }
} as const;

export type CoordinatorMethod = keyof typeof COORDINATOR_METHOD_TABLE;

export function isCoordinatorMethod(name: string): name is CoordinatorMethod {
  return Object.hasOwn(COORDINATOR_METHOD_TABLE, name);
}
