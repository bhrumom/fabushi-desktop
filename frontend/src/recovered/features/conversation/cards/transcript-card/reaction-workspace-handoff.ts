import {
  createReactionActionController,
  createReactToMessageTransport,
  type ReactToMessageInput,
  type ReactionActionController,
  type ReactionActionScope,
  type ReactionAuthoritativeUpdate,
  type ReactionCallSource,
} from "./reaction-actions.ts";
import type { TranscriptFeedSource } from "./reaction-feed.ts";

export interface ScopedReactionWorkspacePair {
  readonly feed: TranscriptFeedSource;
  readonly controller: ReactionActionController;
}

export interface ScopedReactionWorkspacePairOptions {
  readonly scope: ReactionActionScope;
  readonly feed: TranscriptFeedSource | null | undefined;
  readonly source: ReactionCallSource | null | undefined;
  readonly onReacted: (input: ReactToMessageInput) => void;
  readonly onAuthoritativeReactions?: (update: ReactionAuthoritativeUpdate) => void;
  readonly onAuthoritativeCleared?: (scope: ReactionActionScope) => void;
}

function isFeed(value: TranscriptFeedSource | null | undefined): value is TranscriptFeedSource {
  return typeof value?.observeEntriesFeed === "function";
}

function isSource(value: ReactionCallSource | null | undefined): value is ReactionCallSource {
  return typeof value?.call === "function";
}

export function createScopedReactionWorkspacePair(
  options: ScopedReactionWorkspacePairOptions,
): ScopedReactionWorkspacePair | null {
  if (!isFeed(options.feed) || !isSource(options.source)) return null;

  return {
    feed: options.feed,
    controller: createReactionActionController({
      scope: options.scope,
      transport: createReactToMessageTransport(options.source),
      onReacted: options.onReacted,
      onAuthoritativeReactions: options.onAuthoritativeReactions,
      onAuthoritativeCleared: options.onAuthoritativeCleared,
    }),
  };
}
