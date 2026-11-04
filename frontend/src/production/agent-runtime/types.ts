export type RuntimeDetailSectionId = "runtime" | "security" | "channels" | "nodes";

export type ChannelId =
  | "telegram"
  | "discord"
  | "slack"
  | "whatsapp"
  | "feishu"
  | "matrix"
  | "signal"
  | "line"
  | "msteams"
  | "googlechat"
  | "mattermost"
  | "irc"
  | "twitch"
  | "nostr"
  | "zalo"
  | "bluebubbles"
  | "nextcloudtalk"
  | "synologychat";

export type ChannelStatus = "connected" | "disconnected" | "connecting" | "error";
export type DmPolicy = "open" | "allowlist" | "pairing";
export type GroupPolicy = "open" | "allowlist" | "disabled";

export interface ChannelItem {
  readonly id: ChannelId;
  readonly name: string;
  readonly protocol: string;
  readonly status: ChannelStatus;
  readonly dmPolicy: DmPolicy;
  readonly groupPolicy: GroupPolicy;
  readonly activeChatsCount: number;
  readonly webhookUrl?: string;
  readonly tokenMasked?: string;
}

export interface NodeHostCapability {
  readonly id: "system.run" | "browser.proxy" | "canvas";
  readonly name: string;
  readonly description: string;
  readonly enabled: boolean;
  readonly activeExecutions: number;
}

export interface NodeHostLease {
  readonly id: string;
  readonly clientId: string;
  readonly capability: string;
  readonly grantedAt: string;
  readonly ttlRemainingSeconds: number;
}

export interface DevicePairingEntry {
  readonly id: string;
  readonly deviceName: string;
  readonly platform: "ios" | "android" | "macos" | "linux" | "windows" | "web";
  readonly requestedAt: string;
  readonly status: "approved" | "pending" | "rejected";
}

export interface SecurityPolicyState {
  readonly shellAllowlist: boolean;
  readonly ssrfProtection: boolean;
  readonly commandObfuscationDetector: boolean;
  readonly skillSourceScanner: boolean;
  readonly viralTaintStatus: "clean" | "monitored" | "tainted" | "quarantined";
  readonly taintTurnsCount: number;
  readonly quarantinedPayloads: number;
}

export interface MultimodalVoiceState {
  readonly audioTranscriptionEngine: string;
  readonly imageDocInspectionEngine: string;
  readonly videoAnalysisEngine: string;
  readonly talkVoiceEngine: string;
  readonly talkSpeedRate: number;
  readonly talkPitch: number;
}

export interface SessionRuntimeStats {
  readonly sessionId: string;
  readonly modelName: string;
  readonly provider: string;
  readonly usedTokens: number;
  readonly maxTokens: number;
  readonly subagentQueueCount: number;
  readonly branchesCount: number;
  readonly memorySnippetsCount: number;
}
