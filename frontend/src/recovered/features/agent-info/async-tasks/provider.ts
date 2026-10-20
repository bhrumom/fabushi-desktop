export type AsyncTaskKind = "subagent" | "shell" | "cloud-agent";

export interface AsyncTask {
  readonly kind: AsyncTaskKind;
  readonly id: string;
  readonly label: string;
  readonly status: "running";
  readonly startedAtMs: number;
  readonly detail?: string;
  readonly subagentType?: string;
}

export type AsyncTasksSnapshot =
  | { readonly status: "loading" }
  | { readonly status: "empty" }
  | { readonly status: "ready"; readonly value: readonly AsyncTask[] }
  | { readonly status: "failed"; readonly failure: unknown }
  | { readonly status: "failed"; readonly previous: readonly AsyncTask[]; readonly failure: unknown }
  | { readonly status: "unavailable"; readonly reason: string };

export interface AsyncTasksRequestOptions {
  readonly signal?: AbortSignal;
}

export interface AsyncTasksCoordinator {
  getAsyncTasks(
    args: { readonly id: string },
    options?: AsyncTasksRequestOptions,
  ): Promise<unknown>;
}

export interface AsyncTasksSnapshotHandle {
  get(): AsyncTasksSnapshot;
  subscribe(listener: () => void): () => void;
}

export interface AsyncTasksEvent {
  readonly parentAgentId: string;
  readonly tasks: readonly AsyncTask[];
}

export interface AsyncTasksProvider {
  snapshotsFor(agentId: string): AsyncTasksSnapshotHandle;
  refresh(agentId: string): void;
  connect(): void;
  noteReconnect(): void;
  reset(): void;
  dispose(): void;
  ingestAsyncTasksEvent(event: unknown): void;
}

type Entry = {
  agentId: string;
  current: readonly AsyncTask[] | null;
  failure: unknown | null;
  generation: number;
  loading: boolean;
  queuedRefresh: boolean;
  subscribers: number;
  promise: Promise<void> | null;
  snapshot: AsyncTasksSnapshot;
  listeners: Set<() => void>;
};

const LOADING: AsyncTasksSnapshot = Object.freeze({ status: "loading" });
const EMPTY: AsyncTasksSnapshot = Object.freeze({ status: "empty" });
const UNAVAILABLE_CODE = "source/capability-unavailable";

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function decodeTask(value: unknown): AsyncTask | null {
  if (!record(value)) return null;
  const kind = value.kind;
  if (kind !== "subagent" && kind !== "shell" && kind !== "cloud-agent") return null;
  if (typeof value.id !== "string" || value.id.length === 0) return null;
  if (typeof value.label !== "string" || value.label.length === 0) return null;
  if (value.status !== "running") return null;
  if (typeof value.startedAtMs !== "number" || !Number.isFinite(value.startedAtMs)) return null;
  if (value.detail !== undefined && typeof value.detail !== "string") return null;
  if (value.subagentType !== undefined && typeof value.subagentType !== "string") return null;
  const task: AsyncTask = {
    kind,
    id: value.id,
    label: value.label,
    status: "running",
    startedAtMs: value.startedAtMs,
  };
  return {
    ...task,
    ...(typeof value.detail === "string" ? { detail: value.detail } : {}),
    ...(typeof value.subagentType === "string" ? { subagentType: value.subagentType } : {}),
  };
}

function decodeTasks(value: unknown): readonly AsyncTask[] | null {
  if (!Array.isArray(value)) return null;
  const decoded: AsyncTask[] = [];
  for (const candidate of value) {
    const task = decodeTask(candidate);
    if (task === null) return null;
    decoded.push(task);
  }
  return decoded;
}

function codeOf(error: unknown): string | null {
  return record(error) && typeof error.code === "string" ? error.code : null;
}

function snapshotFor(entry: Entry): AsyncTasksSnapshot {
  if (entry.failure !== null) {
    if (codeOf(entry.failure) === UNAVAILABLE_CODE) {
      return { status: "unavailable", reason: UNAVAILABLE_CODE };
    }
    if (entry.current === null) return { status: "failed", failure: entry.failure };
    return { status: "failed", previous: entry.current, failure: entry.failure };
  }
  if (entry.current === null) return LOADING;
  if (entry.current.length > 0) return { status: "ready", value: entry.current };
  return entry.loading ? LOADING : EMPTY;
}

function decodeEvent(value: unknown): AsyncTasksEvent | null {
  if (!record(value)) return null;
  if (typeof value.parentAgentId !== "string" || value.parentAgentId.length === 0) return null;
  const tasks = decodeTasks(value.tasks);
  return tasks === null ? null : { parentAgentId: value.parentAgentId, tasks };
}

export function formatAsyncTaskTime(timestampMs: number, nowMs: number): string {
  if (!Number.isFinite(timestampMs) || timestampMs <= 0 || !Number.isFinite(nowMs)) return "";
  const totalSeconds = Math.floor(Math.max(0, nowMs - timestampMs) / 1_000);
  if (totalSeconds < 60) return "now";
  const minutes = Math.floor(totalSeconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months}mo ago`;
  return `${Math.floor(months / 12)}y ago`;
}

export function createAsyncTasksProvider(coordinator: AsyncTasksCoordinator): AsyncTasksProvider {
  const entries = new Map<string, Entry>();
  let online = false;
  let disposed = false;

  const emit = (entry: Entry): void => {
    if (disposed) return;
    entry.snapshot = snapshotFor(entry);
    for (const listener of Array.from(entry.listeners)) listener();
  };

  const getEntry = (agentId: string): Entry => {
    const found = entries.get(agentId);
    if (found !== undefined) return found;
    const created: Entry = {
      agentId,
      current: null,
      failure: null,
      generation: 0,
      loading: false,
      queuedRefresh: false,
      subscribers: 0,
      promise: null,
      snapshot: LOADING,
      listeners: new Set(),
    };
    entries.set(agentId, created);
    return created;
  };

  const startFetch = (entry: Entry): void => {
    if (disposed || !online || entry.loading) return;
    entry.loading = true;
    const generation = entry.generation;
    emit(entry);

    const request = coordinator
      .getAsyncTasks({ id: entry.agentId })
      .then((raw) => {
        const tasks = decodeTasks(raw);
        if (tasks === null) throw new Error("malformed async tasks reply");
        if (disposed || generation !== entry.generation) return;
        entry.current = tasks;
        entry.failure = null;
      })
      .catch((failure: unknown) => {
        if (!disposed && generation === entry.generation) entry.failure = failure;
      })
      .finally(() => {
        if (disposed || generation !== entry.generation) return;
        entry.loading = false;
        entry.promise = null;
        if (entry.queuedRefresh) {
          entry.queuedRefresh = false;
          startFetch(entry);
        } else {
          emit(entry);
        }
      });

    entry.promise = request;
  };

  const refreshEntry = (entry: Entry): void => {
    if (disposed || !online) return;
    if (entry.loading) {
      entry.queuedRefresh = true;
      return;
    }
    startFetch(entry);
  };

  const refreshSubscribed = (): void => {
    for (const entry of entries.values()) {
      if (entry.subscribers > 0) refreshEntry(entry);
    }
  };

  return {
    snapshotsFor(agentId) {
      const entry = getEntry(agentId);
      return {
        get: () => entry.snapshot,
        subscribe(listener) {
          if (disposed) return () => {};
          entry.subscribers += 1;
          entry.listeners.add(listener);
          if (entry.current === null || entry.failure !== null || entry.subscribers === 1) {
            refreshEntry(entry);
          }
          let active = true;
          return () => {
            if (!active) return;
            active = false;
            entry.subscribers = Math.max(0, entry.subscribers - 1);
            entry.listeners.delete(listener);
          };
        },
      };
    },

    refresh(agentId) {
      refreshEntry(getEntry(agentId));
    },

    connect() {
      if (disposed || online) return;
      online = true;
      refreshSubscribed();
    },

    noteReconnect() {
      if (disposed || !online) return;
      for (const entry of entries.values()) {
        if (entry.loading) {
          entry.generation += 1;
          entry.loading = false;
          entry.queuedRefresh = false;
          entry.promise = null;
        }
        if (entry.subscribers > 0) refreshEntry(entry);
      }
    },

    reset() {
      if (disposed) return;
      online = false;
      for (const entry of entries.values()) {
        entry.generation += 1;
        entry.current = null;
        entry.failure = null;
        entry.loading = false;
        entry.queuedRefresh = false;
        entry.promise = null;
        emit(entry);
      }
    },

    dispose() {
      if (disposed) return;
      disposed = true;
      online = false;
      for (const entry of entries.values()) {
        entry.generation += 1;
        entry.loading = false;
        entry.queuedRefresh = false;
        entry.promise = null;
        entry.listeners.clear();
      }
      entries.clear();
    },

    ingestAsyncTasksEvent(payload) {
      if (disposed) return;
      const event = decodeEvent(payload);
      if (event === null) return;
      const entry = getEntry(event.parentAgentId);
      entry.generation += 1;
      entry.loading = false;
      entry.queuedRefresh = false;
      entry.promise = null;
      entry.current = event.tasks;
      entry.failure = null;
      emit(entry);
    },
  };
}
