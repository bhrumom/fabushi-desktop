export type ConversationChildIdentity =
  | { readonly kind: "topic"; readonly rootMessageId: string }
  | { readonly kind: "savedSublist"; readonly participantId: string }
  | { readonly kind: "conversation"; readonly conversationId: string };

export interface ConversationDestination {
  readonly conversationId: string;
  readonly child: ConversationChildIdentity | null;
}

export interface LegacyTopicDestination {
  readonly conversationId: string;
  readonly legacyTopicId: string;
}

export type LegacyTopicRootResolver = (
  conversationId: string,
  legacyTopicId: string
) => string | null;

export interface ConversationChildSelectionSnapshot {
  readonly selected: ConversationDestination | null;
}

export interface ConversationChildSelectionController {
  getSnapshot(): ConversationChildSelectionSnapshot;
  subscribe(listener: () => void): () => void;
  select(destination: ConversationDestination): boolean;
  selectLegacyTopic(destination: LegacyTopicDestination): boolean;
  clearDestroyedChild(conversationId: string, child: ConversationChildIdentity): boolean;
  clear(): void;
  dispose(): void;
}

export interface ConversationChildSelectionControllerOptions {
  resolveLegacyTopicRoot?: LegacyTopicRootResolver;
}

function validId(value: string): boolean {
  const trimmed = value.trim();
  return trimmed.length > 0 && trimmed.length <= 200;
}

export function isConversationDestinationValid(destination: ConversationDestination): boolean {
  if (!validId(destination.conversationId)) return false;
  const child = destination.child;
  if (child == null) return true;
  switch (child.kind) {
    case "topic":
      return validId(child.rootMessageId);
    case "savedSublist":
      return validId(child.participantId);
    case "conversation":
      return validId(child.conversationId) && child.conversationId !== destination.conversationId;
  }
}

export function conversationDestinationScopeKey(destination: ConversationDestination): string {
  const parent = destination.conversationId.trim();
  const child = destination.child;
  if (child == null) return `conversation:${parent}:root`;
  switch (child.kind) {
    case "topic":
      return `conversation:${parent}:topic:${child.rootMessageId.trim()}`;
    case "savedSublist":
      return `conversation:${parent}:saved:${child.participantId.trim()}`;
    case "conversation":
      return `conversation:${parent}:child:${child.conversationId.trim()}`;
  }
}

function sameChild(left: ConversationChildIdentity, right: ConversationChildIdentity): boolean {
  if (left.kind !== right.kind) return false;
  switch (left.kind) {
    case "topic":
      return right.kind === "topic" && left.rootMessageId === right.rootMessageId;
    case "savedSublist":
      return right.kind === "savedSublist" && left.participantId === right.participantId;
    case "conversation":
      return right.kind === "conversation" && left.conversationId === right.conversationId;
  }
}

export function createConversationChildSelectionController(
  options: ConversationChildSelectionControllerOptions = {}
): ConversationChildSelectionController {
  const listeners = new Set<() => void>();
  let disposed = false;
  let selected: ConversationDestination | null = null;
  let snapshot: ConversationChildSelectionSnapshot = { selected };

  const emit = () => {
    if (disposed) return;
    snapshot = { selected };
    for (const listener of [...listeners]) listener();
  };

  return {
    getSnapshot: () => snapshot,
    subscribe(listener) {
      if (disposed) return () => {};
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    select(destination) {
      if (disposed || !isConversationDestinationValid(destination)) return false;
      selected = destination;
      emit();
      return true;
    },
    selectLegacyTopic(destination) {
      if (disposed || !validId(destination.conversationId) || !validId(destination.legacyTopicId)) {
        return false;
      }
      const rootMessageId = options.resolveLegacyTopicRoot?.(
        destination.conversationId.trim(),
        destination.legacyTopicId.trim()
      )?.trim() ?? "";
      if (!validId(rootMessageId)) return false;
      selected = {
        conversationId: destination.conversationId.trim(),
        child: { kind: "topic", rootMessageId }
      };
      emit();
      return true;
    },
    clearDestroyedChild(conversationId, child) {
      if (
        disposed
        || selected == null
        || selected.conversationId !== conversationId
        || selected.child == null
        || !sameChild(selected.child, child)
      ) {
        return false;
      }
      selected = null;
      emit();
      return true;
    },
    clear() {
      if (disposed || selected == null) return;
      selected = null;
      emit();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      selected = null;
      listeners.clear();
    }
  };
}
