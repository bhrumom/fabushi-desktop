import type {
  ReactionActionController,
  ReactionActionScope,
} from "./reaction-actions.ts";

export interface TranscriptFeedBaseline {
  readonly agentId: string;
  readonly entries: readonly unknown[];
}

export interface TranscriptFeedUpdate {
  readonly agentId: string;
  readonly before?: unknown;
  readonly after: unknown;
}

export interface TranscriptFeedHandlers {
  onBaseline(input: TranscriptFeedBaseline): void;
  onAppended(input: { readonly agentId: string; readonly entry: unknown }): void;
  onUpdated(input: TranscriptFeedUpdate): void;
  onCleared(agentId: string): void;
}

export interface TranscriptFeedSource {
  observeEntriesFeed(
    handlers: TranscriptFeedHandlers,
  ): (() => void) | { dispose(): void };
}

export interface ReactionFeedAdapterOptions {
  readonly scope: ReactionActionScope;
  readonly feed: TranscriptFeedSource;
  readonly controller: Pick<
    ReactionActionController,
    "reconcile" | "clearAuthoritative" | "setScope"
  >;
}

export interface ReactionFeedAdapter {
  getScope(): ReactionActionScope;
  setScope(scope: ReactionActionScope): void;
  reconnect(): void;
  reset(): void;
  dispose(): void;
}

function copyScope(scope: ReactionActionScope): ReactionActionScope {
  return { accountSlot: scope.accountSlot, agentId: scope.agentId };
}

function sameScope(a: ReactionActionScope, b: ReactionActionScope): boolean {
  return a.accountSlot === b.accountSlot && a.agentId === b.agentId;
}

function entryRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function reactionFingerprint(value: unknown): string {
  if (!Array.isArray(value)) return "";
  const parts: string[] = [];
  for (const candidate of value) {
    const record = entryRecord(candidate);
    if (
      record != null
      && typeof record.emoji === "string"
      && record.emoji.length > 0
      && typeof record.by === "string"
      && record.by.length > 0
    ) {
      parts.push(`${record.emoji}\u0000${record.by}`);
    }
  }
  return parts.join("\u0001");
}

function release(
  subscription: (() => void) | { dispose(): void } | null,
): void {
  if (typeof subscription === "function") subscription();
  else subscription?.dispose();
}

export function createReactionFeedAdapter(
  options: ReactionFeedAdapterOptions,
): ReactionFeedAdapter {
  let scope = copyScope(options.scope);
  let generation = 0;
  let disposed = false;
  let subscription: (() => void) | { dispose(): void } | null = null;
  const fingerprints = new Map<string, string>();

  const detach = (): void => {
    release(subscription);
    subscription = null;
  };

  const accepts = (eventGeneration: number, agentId: string): boolean =>
    !disposed
    && eventGeneration === generation
    && scope.agentId != null
    && agentId === scope.agentId;

  const reconcileEntry = (
    eventGeneration: number,
    agentId: string,
    value: unknown,
  ): void => {
    if (!accepts(eventGeneration, agentId)) return;
    const record = entryRecord(value);
    const id = typeof record?.id === "string" && record.id.length > 0
      ? record.id
      : null;
    if (id == null) return;

    const raw = record?.reactions;
    const nextFingerprint = reactionFingerprint(raw);
    if (fingerprints.get(id) === nextFingerprint) return;
    if (!options.controller.reconcile(id, raw)) return;
    fingerprints.set(id, nextFingerprint);
  };

  const connect = (): void => {
    if (disposed || scope.agentId == null || subscription != null) return;
    const eventGeneration = generation;
    subscription = options.feed.observeEntriesFeed({
      onBaseline(input) {
        if (!accepts(eventGeneration, input.agentId)) return;
        for (const entry of input.entries) {
          reconcileEntry(eventGeneration, input.agentId, entry);
        }
      },
      onAppended(input) {
        reconcileEntry(eventGeneration, input.agentId, input.entry);
      },
      onUpdated(input) {
        reconcileEntry(eventGeneration, input.agentId, input.after);
      },
      onCleared(agentId) {
        if (!accepts(eventGeneration, agentId)) return;
        fingerprints.clear();
        options.controller.clearAuthoritative();
      },
    });
  };

  options.controller.setScope(scope);
  connect();

  return {
    getScope() {
      return copyScope(scope);
    },
    setScope(next) {
      if (disposed || sameScope(scope, next)) return;
      generation += 1;
      detach();
      fingerprints.clear();
      scope = copyScope(next);
      options.controller.setScope(scope);
      connect();
    },
    reconnect() {
      if (disposed) return;
      generation += 1;
      detach();
      fingerprints.clear();
      connect();
    },
    reset() {
      if (disposed) return;
      generation += 1;
      detach();
      fingerprints.clear();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      generation += 1;
      detach();
      fingerprints.clear();
    },
  };
}
