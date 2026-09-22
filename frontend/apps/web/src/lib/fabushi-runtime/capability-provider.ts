export type CloudAgentStatus = "queued" | "running" | "finished" | "error" | "expired" | "unknown";

export interface CloudAgentInfo {
  readonly id: string;
  readonly agentId?: string;
  readonly runId?: string;
  readonly conversationId?: string;
  readonly name?: string;
  readonly available: boolean;
  readonly provider: "fabushi-platform" | "local-device" | null;
  readonly status: CloudAgentStatus;
  readonly rawStatus?: string;
  readonly model?: string | null;
  readonly inputTokens?: number;
  readonly outputTokens?: number;
  readonly toolCallCount?: number;
  readonly startedAt?: string | null;
  readonly completedAt?: string | null;
  readonly failedAt?: string | null;
  readonly errorCode?: string | null;
  readonly errorMessage?: string | null;
  readonly reason: string | null;
}

export interface ForeverBoxStatus {
  readonly agentId: string;
  readonly boxId: string | null;
  readonly status: "ready" | "released" | "unavailable";
  readonly provider: "local-device" | "fabushi-desktop" | null;
  readonly createdAtMs: number | null;
  readonly updatedAtMs: number;
  readonly reason: string | null;
}

export interface BoxSecretsStatus {
  readonly agentId: string;
  readonly boxId: string | null;
  readonly configured: boolean;
  readonly secretCount: number;
  readonly provider: "local-device" | "fabushi-desktop" | null;
}

type ProviderState = {
  boxes: Record<string, { boxId: string; createdAtMs: number; updatedAtMs: number }>;
};

const STORAGE_KEY = "fabushi.capability-provider.v1";

function storage(): Storage | null {
  if (typeof window === "undefined") return null;
  try { return window.localStorage; } catch { return null; }
}

function loadState(): ProviderState {
  const target = storage();
  if (!target) return { boxes: {} };
  try {
    const parsed = JSON.parse(target.getItem(STORAGE_KEY) ?? "null") as Partial<ProviderState> | null;
    return parsed && parsed.boxes && typeof parsed.boxes === "object" ? { boxes: parsed.boxes } : { boxes: {} };
  } catch {
    return { boxes: {} };
  }
}

function saveState(state: ProviderState): void {
  try { storage()?.setItem(STORAGE_KEY, JSON.stringify(state)); } catch { /* best effort */ }
}

function stableLocalBoxId(agentId: string): string {
  let hash = 2166136261;
  for (const character of agentId) {
    hash ^= character.codePointAt(0) ?? 0;
    hash = Math.imul(hash, 16777619);
  }
  return `local-box-${(hash >>> 0).toString(36)}`;
}

export class FabushiCapabilityProvider {
  private state = loadState();

  getCloudAgentInfo(id: string): CloudAgentInfo {
    const clean = id.trim();
    if (!clean) throw new Error("Cloud agent or run ID is required");
    return {
      id: clean,
      agentId: clean,
      available: false,
      provider: null,
      status: "unknown",
      reason: "No cloud run was resolved; Fabushi will use local execution when available.",
    };
  }

  getForeverBoxStatus(agentId: string): ForeverBoxStatus {
    const clean = agentId.trim();
    if (!clean) throw new Error("Agent ID is required");
    const box = this.state.boxes[clean];
    return box
      ? {
          agentId: clean,
          boxId: box.boxId,
          status: "ready",
          provider: "local-device",
          createdAtMs: box.createdAtMs,
          updatedAtMs: box.updatedAtMs,
          reason: null,
        }
      : {
          agentId: clean,
          boxId: null,
          status: "released",
          provider: null,
          createdAtMs: null,
          updatedAtMs: Date.now(),
          reason: "No persistent local workspace has been provisioned for this agent.",
        };
  }

  ensureForeverBox(agentId: string): ForeverBoxStatus {
    const clean = agentId.trim();
    if (!clean) throw new Error("Agent ID is required");
    const now = Date.now();
    const current = this.state.boxes[clean];
    const next = current ?? { boxId: stableLocalBoxId(clean), createdAtMs: now, updatedAtMs: now };
    next.updatedAtMs = now;
    this.state = { ...this.state, boxes: { ...this.state.boxes, [clean]: next } };
    saveState(this.state);
    return this.getForeverBoxStatus(clean);
  }

  handBackForeverBox(agentId: string): ForeverBoxStatus {
    const clean = agentId.trim();
    if (!clean) throw new Error("Agent ID is required");
    const boxes = { ...this.state.boxes };
    delete boxes[clean];
    this.state = { ...this.state, boxes };
    saveState(this.state);
    return this.getForeverBoxStatus(clean);
  }

  getBoxSecretsStatus(agentId: string): BoxSecretsStatus {
    const box = this.getForeverBoxStatus(agentId);
    return {
      agentId: box.agentId,
      boxId: box.boxId,
      configured: false,
      secretCount: 0,
      provider: box.provider,
    };
  }

  isAgentNetworkEnabled(_agentId: string): boolean {
    // No policy adapter currently proves that arbitrary agent network egress is
    // enabled. Be conservative rather than claiming an unenforced permission.
    return false;
  }

  isGlobalSearchEnabled(): boolean {
    // Message/media global search is implemented by the Mahayana Host itself,
    // so this is a built-in product capability rather than a remote-provider flag.
    return true;
  }

  isEgressTunnelAvailable(): boolean {
    // A dedicated egress tunnel provider is not configured by the local fallback.
    // Return false rather than treating ordinary network access as a managed tunnel.
    return false;
  }
}
