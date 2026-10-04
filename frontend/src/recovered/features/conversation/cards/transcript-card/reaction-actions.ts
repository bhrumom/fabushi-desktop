export const SAND_REACTION_SELF = "me" as const;
export const QUICK_REACTION_EMOJIS = Object.freeze(["👍", "👎", "❤️", "😂", "🎉", "😮"] as const);

export interface TranscriptReaction {
  readonly emoji: string;
  readonly by: string;
}

export interface TranscriptReactionProjection {
  readonly reactions: readonly TranscriptReaction[];
  readonly myReactions: ReadonlySet<string>;
}

export interface ReactToMessageInput {
  readonly entryId: string;
  readonly emoji: string;
  readonly agentId: string;
}

export interface ReactToMessageTransport {
  reactToMessage(input: ReactToMessageInput): Promise<void>;
}

export interface ReactionCallSource {
  call(method: "reactToMessage", args: ReactToMessageInput): Promise<unknown>;
}

export interface ReactionActionScope {
  readonly accountSlot: string | null;
  readonly agentId: string | null;
}

export interface ReactionAuthoritativeUpdate {
  readonly scope: ReactionActionScope;
  readonly entryId: string;
  readonly projection: TranscriptReactionProjection;
}

export interface ReactionActionControllerOptions {
  readonly scope: ReactionActionScope;
  readonly transport: ReactToMessageTransport;
  readonly onReacted: (input: ReactToMessageInput) => void;
  readonly onAuthoritativeReactions?: (update: ReactionAuthoritativeUpdate) => void;
  readonly onAuthoritativeCleared?: (scope: ReactionActionScope) => void;
}

export interface ReactionActionController {
  getScope(): ReactionActionScope;
  react(entryId: string, emoji: string): boolean;
  reconcile(entryId: string, value: unknown): boolean;
  clearAuthoritative(): boolean;
  setScope(scope: ReactionActionScope): void;
  dispose(): void;
}

function copyScope(scope: ReactionActionScope): ReactionActionScope {
  return { accountSlot: scope.accountSlot, agentId: scope.agentId };
}

function sameScope(a: ReactionActionScope, b: ReactionActionScope): boolean {
  return a.accountSlot === b.accountSlot && a.agentId === b.agentId;
}

export function projectTranscriptReactions(value: unknown): TranscriptReactionProjection {
  if (!Array.isArray(value)) {
    return { reactions: [], myReactions: new Set<string>() };
  }

  const reactions: TranscriptReaction[] = [];
  const myReactions = new Set<string>();
  for (const candidate of value) {
    if (typeof candidate !== "object" || candidate === null || Array.isArray(candidate)) continue;
    const record = candidate as Record<string, unknown>;
    if (
      typeof record.emoji !== "string"
      || record.emoji.length === 0
      || typeof record.by !== "string"
      || record.by.length === 0
    ) continue;

    const reaction = { emoji: record.emoji, by: record.by };
    reactions.push(reaction);
    if (reaction.by === SAND_REACTION_SELF) myReactions.add(reaction.emoji);
  }
  return { reactions, myReactions };
}

export function createReactToMessageTransport(source: ReactionCallSource): ReactToMessageTransport {
  return {
    async reactToMessage(input) {
      await source.call("reactToMessage", input);
    },
  };
}

export function createReactionActionController(
  options: ReactionActionControllerOptions,
): ReactionActionController {
  let scope = copyScope(options.scope);
  let generation = 0;
  let disposed = false;

  return {
    getScope() {
      return copyScope(scope);
    },
    react(entryId, emoji) {
      const agentId = scope.agentId;
      if (disposed || agentId == null || entryId.length === 0 || emoji.length === 0) return false;

      const input = { entryId, emoji, agentId };
      const requestGeneration = generation;
      const requestScope = copyScope(scope);
      options.onReacted(input);

      void options.transport.reactToMessage(input).catch(() => {
        // The product deliberately keeps optimistic reaction UI and does not
        // surface transport errors. Generation/scope fencing prevents a late
        // failure from belonging to a replacement conversation.
        if (disposed || requestGeneration !== generation || !sameScope(requestScope, scope)) return;
      });
      return true;
    },
    reconcile(entryId, value) {
      if (disposed || scope.agentId == null || entryId.length === 0) return false;
      options.onAuthoritativeReactions?.({
        scope: copyScope(scope),
        entryId,
        projection: projectTranscriptReactions(value),
      });
      return true;
    },
    clearAuthoritative() {
      if (disposed || scope.agentId == null) return false;
      options.onAuthoritativeCleared?.(copyScope(scope));
      return true;
    },
    setScope(next) {
      if (disposed || sameScope(scope, next)) return;
      scope = copyScope(next);
      generation += 1;
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      generation += 1;
    },
  };
}
