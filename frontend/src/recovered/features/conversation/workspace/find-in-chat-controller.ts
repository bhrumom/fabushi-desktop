export type FindableTranscriptEntry =
  | { readonly id: string; readonly kind: "message"; readonly text: string }
  | { readonly id: string; readonly kind: "notice"; readonly text: string }
  | {
      readonly id: string;
      readonly kind: "send-message";
      readonly message:
        | { readonly type: "text"; readonly content: string }
        | { readonly type: "widget"; readonly widget: { readonly prompt: string } }
        | {
            readonly type: "email-draft";
            readonly draft: { readonly subject: string; readonly body: string };
          }
        | { readonly type: "slack-draft"; readonly draft: { readonly body: string } }
        | { readonly type: string; readonly [key: string]: unknown };
    }
  | { readonly id: string; readonly kind: string };

export interface FindInChatScope {
  accountSlot: string | null;
  agentId: string | null;
}

export interface FindInChatMatch {
  entryId: string;
  occurrence: number;
}

export interface FindInChatTranscriptHandle {
  scrollToEntryWithoutHighlight(entryId: string): boolean;
  subscribeViewCommits(listener: () => void): () => void;
}

export interface FindInChatSnapshot {
  readonly generation: number;
  readonly scope: FindInChatScope;
  readonly query: string;
  readonly matches: readonly FindInChatMatch[];
  readonly current: FindInChatMatch | null;
}

export interface FindInChatController {
  getSnapshot(): FindInChatSnapshot;
  subscribe(listener: () => void): () => void;
  replaceEntries(entries: readonly FindableTranscriptEntry[]): void;
  setQuery(query: string): void;
  step(delta: 1 | -1): FindInChatMatch | null;
  close(): void;
  setScope(accountSlot: string | null, agentId: string | null): void;
  dispose(): void;
}

function copyScope(scope: FindInChatScope): FindInChatScope {
  return { accountSlot: scope.accountSlot, agentId: scope.agentId };
}

export function findInChatSearchText(
  entry: FindableTranscriptEntry,
): string {
  if (entry.kind === "message" || entry.kind === "notice") {
    return typeof (entry as { text?: unknown }).text === "string"
      ? (entry as { text: string }).text
      : "";
  }
  if (entry.kind !== "send-message") return "";

  const message = (entry as Extract<
    FindableTranscriptEntry,
    { kind: "send-message" }
  >).message;
  switch (message.type) {
    case "text":
      return typeof (message as { content?: unknown }).content === "string"
        ? (message as { content: string }).content
        : "";
    case "widget":
      return typeof (message as { widget?: { prompt?: unknown } }).widget?.prompt === "string"
        ? (message as { widget: { prompt: string } }).widget.prompt
        : "";
    case "email-draft": {
      const draft = (message as { draft?: { subject?: unknown; body?: unknown } }).draft;
      return typeof draft?.subject === "string" && typeof draft.body === "string"
        ? `${draft.subject}\n${draft.body}`
        : "";
    }
    case "slack-draft": {
      const draft = (message as { draft?: { body?: unknown } }).draft;
      return typeof draft?.body === "string" ? draft.body : "";
    }
    default:
      return "";
  }
}

function matchesFor(
  entries: readonly FindableTranscriptEntry[],
  query: string,
): FindInChatMatch[] {
  if (query.trim().length === 0) return [];
  const needle = query.toLowerCase();
  const result: FindInChatMatch[] = [];

  for (const entry of entries) {
    const text = findInChatSearchText(entry).toLowerCase();
    let at = text.indexOf(needle);
    let occurrence = 0;
    while (at >= 0) {
      result.push({ entryId: entry.id, occurrence });
      occurrence += 1;
      at = text.indexOf(needle, at + needle.length);
    }
  }
  return result;
}

function sameMatch(
  left: FindInChatMatch | null,
  right: FindInChatMatch | null,
): boolean {
  return left?.entryId === right?.entryId
    && left?.occurrence === right?.occurrence;
}

export function createFindInChatController(options: {
  scope?: FindInChatScope;
  onNavigate?(match: FindInChatMatch, index: number): void;
} = {}): FindInChatController {
  let scope = copyScope(options.scope ?? {
    accountSlot: null,
    agentId: null,
  });
  let entries: readonly FindableTranscriptEntry[] = [];
  let query = "";
  let matches: readonly FindInChatMatch[] = [];
  let current: FindInChatMatch | null = null;
  let generation = 0;
  let disposed = false;
  const listeners = new Set<() => void>();
  let snapshot: FindInChatSnapshot;

  const refresh = (): void => {
    snapshot = {
      generation,
      scope: copyScope(scope),
      query,
      matches,
      current,
    };
  };
  const emit = (): void => {
    if (!disposed) {
      for (const listener of Array.from(listeners)) listener();
    }
  };
  const recompute = (): void => {
    const next = matchesFor(entries, query);
    current = next.find((match) => sameMatch(match, current)) ?? null;
    matches = next;
  };

  refresh();

  return {
    getSnapshot: () => snapshot,
    subscribe(listener) {
      if (disposed) return () => {};
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    replaceEntries(next) {
      if (disposed) return;
      entries = [...next];
      recompute();
      refresh();
      emit();
    },
    setQuery(next) {
      if (disposed || query === next) return;
      query = next;
      current = null;
      recompute();
      refresh();
      emit();
    },
    step(delta) {
      if (disposed || matches.length === 0) return null;
      const currentIndex =
        current == null
          ? matches.length - 1
          : Math.max(
              0,
              matches.findIndex((match) => sameMatch(match, current)),
            );
      const index =
        (currentIndex + delta + matches.length) % matches.length;
      current = matches[index] ?? null;
      if (current != null) options.onNavigate?.(current, index);
      refresh();
      emit();
      return current;
    },
    close() {
      if (disposed) return;
      query = "";
      matches = [];
      current = null;
      refresh();
      emit();
    },
    setScope(accountSlot, agentId) {
      if (
        disposed
        || (
          scope.accountSlot === accountSlot
          && scope.agentId === agentId
        )
      ) {
        return;
      }
      generation += 1;
      scope = { accountSlot, agentId };
      entries = [];
      query = "";
      matches = [];
      current = null;
      refresh();
      emit();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      generation += 1;
      entries = [];
      query = "";
      matches = [];
      current = null;
      refresh();
      listeners.clear();
    },
  };
}
