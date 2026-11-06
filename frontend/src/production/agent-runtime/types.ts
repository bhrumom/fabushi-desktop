import type { ProductionCoordinatorClient } from "../coordinator-client";

export type RuntimeDetailSectionId = "runtime" | "security" | "channels" | "nodes";

export type ThinkingLevel = "inherit" | "off" | "minimal" | "low" | "medium" | "high" | "xhigh";
export type VerboseLevel = "inherit" | "off" | "on" | "full";
export type ReasoningLevel = "inherit" | "off" | "on" | "stream";
export type ElevatedLevel = "inherit" | "off" | "on" | "ask" | "full";

export type ExecSecurity = "deny" | "allowlist" | "full";
export type ExecAsk = "off" | "on-miss" | "always";
export type ExecMode = "deny" | "allowlist" | "ask" | "auto" | "full";

export interface ExecAllowlistEntry {
  readonly id: string;
  readonly pattern: string;
  readonly description?: string;
  readonly lastUsedAt?: number;
}

export interface ExecApprovalsFile {
  readonly version: 1;
  readonly defaults?: {
    readonly security?: ExecSecurity;
    readonly ask?: ExecAsk;
    readonly askFallback?: "deny" | "allowlist";
    readonly autoAllowSkills?: boolean;
  };
  readonly allowlist?: readonly ExecAllowlistEntry[];
}

export interface ChannelStatusSummary {
  readonly id: string;
  readonly name: string;
  readonly protocol: string;
  readonly configured: boolean;
  readonly running: boolean;
  readonly connected: boolean;
  readonly lastError?: string | null;
  readonly dmPolicy: "open" | "allowlist" | "pairing";
  readonly groupPolicy: "open" | "allowlist" | "disabled";
  readonly activeChatsCount: number;
}

export interface NodeDescription {
  readonly nodeId: string;
  readonly name: string;
  readonly platform: string;
  readonly connected: boolean;
  readonly capabilities: readonly string[];
  readonly activeLeases: number;
}

export interface DevicePairingRequest {
  readonly id: string;
  readonly deviceName: string;
  readonly platform: string;
  readonly requestedAt: string;
  readonly status: "pending" | "approved" | "rejected";
}

export interface SessionUsageSnapshot {
  readonly sessionId: string;
  readonly key: string;
  readonly model: string;
  readonly modelProvider: string;
  readonly inputTokens: number;
  readonly outputTokens: number;
  readonly totalTokens: number;
  readonly contextTokens: number;
  readonly maxContextTokens: number;
  readonly thinkingLevel: ThinkingLevel;
  readonly verboseLevel: VerboseLevel;
  readonly reasoningLevel: ReasoningLevel;
  readonly elevatedLevel: ElevatedLevel;
  readonly childSessions: readonly string[];
  readonly subagentRole?: "orchestrator" | "leaf";
  readonly spawnDepth?: number;
}

export interface AgentRuntimeBridgeProps {
  readonly agentId: string;
  readonly client: ProductionCoordinatorClient | null;
}
