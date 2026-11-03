import type { CoordinatorSourceEventListener, RawPortCoordinatorSource } from "../../../runtime/coordinator-source";

// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=2291210 (title availability)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=2766045 (Agent Settings fields and notification toggle)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=2766182 (setAgentNotifyOnUpdates)

export interface AgentSettingsAgent {
  readonly id: string;
  readonly name: string;
  readonly title?: string;
  readonly description: string;
  readonly isGroup: boolean;
  readonly notifyOnUpdatesEnabled: boolean;
  readonly [key: string]: unknown;
}

export interface AgentSettingsProfile {
  readonly name: string;
  readonly title?: string;
  readonly description: string;
}

export interface AgentSettingsSource {
  updateAgent(args: { readonly id: string; readonly profile: AgentSettingsProfile }): Promise<unknown>;
  setAgentNotifyOnUpdates(args: { readonly id: string; readonly isEnabled: boolean }): Promise<unknown>;
  getSessionModelSelection(args: { readonly key: string }): Promise<unknown>;
  resetSessionModelSelection(args: {
    readonly key: string;
    readonly expectedSessionId: string;
    readonly expectedLifecycleRevision: string;
    readonly modelSelection: { readonly resetToDefault: true };
  }): Promise<unknown>;
  subscribe?(listener: CoordinatorSourceEventListener): { dispose(): void };
}

export interface SessionModelStatus {
  readonly sessionId: string;
  readonly lifecycleRevision: string;
  readonly supported: boolean;
  readonly override: { readonly provider: string; readonly modelId: string } | null;
  readonly default: { readonly provider: string; readonly modelId: string } | null;
  readonly locked: boolean;
  readonly canReset: boolean;
}

export interface AgentSettingsSnapshot {
  readonly agent: AgentSettingsAgent;
  readonly pending: "profile" | "notifications" | "model-selection" | null;
  readonly modelSelection: SessionModelStatus | null;
  readonly modelLoading: boolean;
  readonly error: unknown | null;
  readonly generation: number;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringValue(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function booleanValue(value: unknown): boolean | undefined {
  return typeof value === "boolean" ? value : undefined;
}

/** Projects listAgents/agent-upserted records into the exact settings fields. */
export function projectAgentSettingsAgent(value: unknown): AgentSettingsAgent | null {
  if (!isRecord(value) || typeof value.id !== "string" || typeof value.name !== "string") return null;
  const title = stringValue(value.title);
  return {
    ...value,
    id: value.id,
    name: value.name,
    ...(title === undefined ? {} : { title }),
    description: stringValue(value.description) ?? "",
    isGroup: value.isGroup === true,
    notifyOnUpdatesEnabled: booleanValue(value.notifyOnUpdatesEnabled) ?? false
  };
}

export function profileFromAgent(agent: AgentSettingsAgent): AgentSettingsProfile {
  return {
    name: agent.name,
    ...(agent.title === undefined ? {} : { title: agent.title }),
    description: agent.description
  };
}

function profileChanged(current: AgentSettingsProfile, next: AgentSettingsProfile): boolean {
  return current.name !== next.name || current.title !== next.title || current.description !== next.description;
}

function eventAgent(value: unknown): AgentSettingsAgent | null {
  const direct = projectAgentSettingsAgent(value);
  if (direct != null) return direct;
  if (!isRecord(value)) return null;
  return projectAgentSettingsAgent(value.agent);
}


function requireSessionModelStatus(value: unknown, expectedAgent: string): SessionModelStatus {
  if (!isRecord(value) || value.sessionId !== expectedAgent
      || typeof value.lifecycleRevision !== "string" || value.lifecycleRevision.length === 0
      || !isRecord(value.modelSelection)) {
    throw new Error("Session model status response is malformed");
  }
  const status = value.modelSelection;
  const pair = (candidate: unknown) => {
    if (candidate === null) return null;
    if (!isRecord(candidate) || typeof candidate.provider !== "string"
        || candidate.provider.trim() === "" || typeof candidate.modelId !== "string"
        || candidate.modelId.trim() === "") throw new Error("Session model pair is malformed");
    return { provider: candidate.provider, modelId: candidate.modelId };
  };
  if (status.version !== 1 || typeof status.supported !== "boolean"
      || typeof status.locked !== "boolean" || typeof status.canReset !== "boolean") {
    throw new Error("Session model status contract is malformed");
  }
  const current = pair(status.override);
  const configured = pair(status.default);
  if (status.supported !== (configured !== null) || (status.canReset && (
      !status.supported || status.locked || current === null))) {
    throw new Error("Session model status has inconsistent policy");
  }
  return {
    sessionId: value.sessionId,
    lifecycleRevision: value.lifecycleRevision,
    supported: status.supported,
    override: current,
    default: configured,
    locked: status.locked,
    canReset: status.canReset
  };
}

/**
 * Controller for the shipped Agent Settings section. It consumes authoritative
 * roster events and fences late mutation replies after selection/disposal.
 */
export function createAgentSettingsController(source: AgentSettingsSource, initialAgent: AgentSettingsAgent) {
  let current = initialAgent;
  let generation = 0;
  let pending: AgentSettingsSnapshot["pending"] = null;
  let modelSelection: SessionModelStatus | null = null;
  let modelLoading = false;
  let error: unknown | null = null;
  let disposed = false;
  let snapshot: AgentSettingsSnapshot = { agent: current, pending, modelSelection, modelLoading, error, generation };
  const listeners = new Set<() => void>();
  const subscription = source.subscribe?.({
    agents: (value) => {
      if (!Array.isArray(value)) return;
      const next = value.map(projectAgentSettingsAgent).find((agent): agent is AgentSettingsAgent => agent?.id === current.id);
      if (next != null) applyAuthoritative(next);
    },
    "agent-upserted": (value) => {
      const next = eventAgent(value);
      if (next?.id === current.id) applyAuthoritative(next);
    }
  });

  const emit = () => {
    if (disposed) return;
    snapshot = { agent: current, pending, modelSelection, modelLoading, error, generation };
    for (const listener of [...listeners]) listener();
  };
  function applyAuthoritative(next: AgentSettingsAgent): void {
    if (disposed || next.id !== current.id) return;
    current = next;
    error = null;
    emit();
  }


  async function loadSessionModelSelection(): Promise<void> {
    if (disposed || current.isGroup) return;
    const observed = generation;
    const id = current.id;
    modelLoading = true;
    error = null;
    emit();
    try {
      const result = requireSessionModelStatus(
        await source.getSessionModelSelection({ key: `agent:${id}:main` }), id
      );
      if (disposed || generation !== observed || current.id !== id) return;
      modelSelection = result;
    } catch (cause) {
      if (disposed || generation !== observed || current.id !== id) return;
      modelSelection = null;
      error = cause;
    } finally {
      if (!disposed && generation === observed && current.id === id) {
        modelLoading = false;
        emit();
      }
    }
  }
  async function resetSessionModelSelection(): Promise<boolean> {
    if (disposed || current.isGroup || pending != null || modelLoading
        || modelSelection == null || !modelSelection.canReset) return false;
    const observed = generation;
    const id = current.id;
    const captured = modelSelection;
    pending = "model-selection";
    error = null;
    emit();
    try {
      const response = await source.resetSessionModelSelection({
        key: `agent:${id}:main`,
        expectedSessionId: captured.sessionId,
        expectedLifecycleRevision: captured.lifecycleRevision,
        modelSelection: { resetToDefault: true }
      });
      if (disposed || generation !== observed || current.id !== id) return false;
      if (!isRecord(response) || response.ok !== true) {
        throw new Error("Session model reset was not acknowledged");
      }
      const settled = requireSessionModelStatus(response, id);
      if (settled.lifecycleRevision !== captured.lifecycleRevision || settled.override !== null) {
        throw new Error("Session model reset acknowledgement is inconsistent");
      }
      modelSelection = settled;
      emit();
      return true;
    } catch (cause) {
      if (!disposed && generation === observed && current.id === id) {
        error = cause;
        emit();
      }
      throw cause;
    } finally {
      if (!disposed && generation === observed && current.id === id) {
        pending = null;
        emit();
      }
    }
  }
  void loadSessionModelSelection();

  return {
    subscribe(listener: () => void): () => void {
      if (disposed) return () => {};
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    getSnapshot(): AgentSettingsSnapshot { return snapshot; },
    refreshSessionModelSelection: loadSessionModelSelection,
    resetSessionModelSelection,
    setAgent(next: AgentSettingsAgent): void {
      if (disposed || next.id === current.id && next === current) return;
      generation += 1;
      pending = null;
      modelSelection = null;
      modelLoading = false;
      error = null;
      current = next;
      emit();
      void loadSessionModelSelection();
    },
    async updateProfile(next: AgentSettingsProfile): Promise<boolean> {
      if (disposed || pending != null) return false;
      const normalized: AgentSettingsProfile = {
        name: next.name.trim(),
        ...(next.title === undefined ? {} : { title: next.title.trim() }),
        description: next.description.trim()
      };
      if (normalized.name.length === 0) return false;
      const before = profileFromAgent(current);
      if (!profileChanged(before, normalized)) return false;
      const requestGeneration = generation;
      pending = "profile";
      error = null;
      emit();
      try {
        const result = await source.updateAgent({ id: current.id, profile: normalized });
        if (disposed || requestGeneration !== generation) return false;
        const authoritative = projectAgentSettingsAgent(result);
        if (authoritative != null && authoritative.id === current.id) applyAuthoritative(authoritative);
        return true;
      } catch (cause) {
        if (!disposed && requestGeneration === generation) { error = cause; emit(); }
        throw cause;
      } finally {
        if (!disposed && requestGeneration === generation) { pending = null; emit(); }
      }
    },
    async setNotifications(isEnabled: boolean): Promise<boolean> {
      if (disposed || pending != null || current.isGroup || current.notifyOnUpdatesEnabled === isEnabled) return false;
      const requestGeneration = generation;
      pending = "notifications";
      error = null;
      emit();
      try {
        await source.setAgentNotifyOnUpdates({ id: current.id, isEnabled });
        if (disposed || requestGeneration !== generation) return false;
        current = { ...current, notifyOnUpdatesEnabled: isEnabled };
        emit();
        return true;
      } catch (cause) {
        if (!disposed && requestGeneration === generation) { error = cause; emit(); }
        throw cause;
      } finally {
        if (!disposed && requestGeneration === generation) { pending = null; emit(); }
      }
    },
    dispose(): void {
      if (disposed) return;
      disposed = true;
      generation += 1;
      pending = null;
      subscription?.dispose();
      listeners.clear();
    }
  };
}

export type AgentSettingsController = ReturnType<typeof createAgentSettingsController>;

export function coordinatorAgentSettingsSource(source: RawPortCoordinatorSource): AgentSettingsSource {
  return {
    updateAgent: (args) => source.updateAgent(args),
    setAgentNotifyOnUpdates: (args) => source.setAgentNotifyOnUpdates(args),
    getSessionModelSelection: (args) => source.getSessionModelSelection(args),
    resetSessionModelSelection: (args) => source.resetSessionModelSelection(args),
    subscribe: (listener) => source.subscribe(listener)
  };
}
