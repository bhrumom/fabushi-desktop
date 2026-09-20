export type AgentNotificationKind = "completed" | "needs-input";

export interface AgentNotificationSnapshot {
  readonly id: string;
  readonly name: string;
  readonly isRunning: boolean;
  readonly awaitingReason: string | null;
  readonly lastMessageId: string | null;
  readonly lastMessagePreview: string | null;
  readonly notifyEnabled: boolean;
  readonly isHiddenFromSidebar: boolean;
}

export interface AgentNotificationDecision {
  readonly agentId: string;
  readonly agentName: string;
  readonly kind: AgentNotificationKind;
  readonly reason: string | null;
  readonly preview: string | null;
}

const DEFAULT_THROTTLE_MS = 5_000;
const BODY_LIMIT = 140;

function summarize(value: string | null | undefined): string {
  const clean = (value ?? "").replace(/\s+/gu, " ").trim();
  if (clean.length <= BODY_LIMIT) return clean;
  return `${clean.slice(0, BODY_LIMIT - 1).trimEnd()}…`;
}

export function buildAgentNotification(decision: AgentNotificationDecision): {
  readonly title: string;
  readonly body: string;
} {
  const name = decision.agentName.trim() || "智能体";
  if (decision.kind === "needs-input") {
    return {
      title: `${name} 需要你的确认`,
      body: summarize(decision.reason) || "任务正在等待你的输入。",
    };
  }
  return {
    title: `${name} 已完成本轮任务`,
    body: summarize(decision.preview) || "打开全球法布施查看结果。",
  };
}

export class AgentNotificationPolicy {
  private readonly prior = new Map<string, AgentNotificationSnapshot>();
  private readonly lastShownAt = new Map<string, number>();

  constructor(private readonly throttleMs = DEFAULT_THROTTLE_MS) {}

  seed(snapshots: readonly AgentNotificationSnapshot[]): void {
    for (const snapshot of snapshots) {
      if (!this.prior.has(snapshot.id)) this.prior.set(snapshot.id, snapshot);
    }
  }

  forgetAgent(agentId: string): void {
    this.prior.delete(agentId);
    this.lastShownAt.delete(agentId);
  }

  evaluate(
    snapshot: AgentNotificationSnapshot,
    environment: { readonly isWindowFocused: boolean; readonly nowMs: number },
  ): AgentNotificationDecision[] {
    const previous = this.prior.get(snapshot.id);
    this.prior.set(snapshot.id, snapshot);
    if (!previous) return [];

    const needsInput = previous.awaitingReason == null && snapshot.awaitingReason != null;
    const completed = previous.isRunning && !snapshot.isRunning && snapshot.awaitingReason == null;
    if (!needsInput && !completed) return [];
    if (snapshot.isHiddenFromSidebar || !snapshot.notifyEnabled || environment.isWindowFocused) return [];

    const lastShown = this.lastShownAt.get(snapshot.id);
    if (lastShown != null && environment.nowMs - lastShown < this.throttleMs) return [];
    this.lastShownAt.set(snapshot.id, environment.nowMs);

    return [{
      agentId: snapshot.id,
      agentName: snapshot.name,
      kind: needsInput ? "needs-input" : "completed",
      reason: needsInput ? snapshot.awaitingReason : null,
      preview: snapshot.lastMessagePreview,
    }];
  }
}
