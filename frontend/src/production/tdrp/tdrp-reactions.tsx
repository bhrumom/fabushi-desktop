import React, { useCallback } from "react";
import type { ConversationTranscriptEntry } from "../../recovered/features/conversation/workspace/model";
import type { ReactionAuthoritativeUpdate } from "../../recovered/features/conversation/cards/transcript-card/reaction-actions";
import type { TranscriptMessageReactionPillsProps } from "../../recovered/features/conversation/workspace/transcript";
import type { TranscriptMessageReactionSlotProps } from "../../recovered/features/conversation/cards/transcript-card/message-actions";
import { MessageReactionAction, ReactionPills } from "../../recovered/features/conversation/cards/transcript-card/reaction-picker";
import type { RendererAgent } from "../model";

export function applyAuthoritativeReactionUpdate(
  entries: readonly ConversationTranscriptEntry[],
  entryId: string,
  projection: Pick<ReactionAuthoritativeUpdate, "projection">["projection"],
): readonly ConversationTranscriptEntry[] {
  let changed = false;
  const next = entries.map((entry) => {
    if (entry.id !== entryId) return entry;
    changed = true;
    if (entry.kind === "message") return { ...entry, reactions: projection.reactions, myReactions: projection.myReactions };
    if (entry.kind === "send-message") return { ...entry, reactions: projection.reactions, myReactions: projection.myReactions };
    return entry;
  });
  return changed ? next : entries;
}

export function clearAuthoritativeReactions(
  entries: readonly ConversationTranscriptEntry[]
): readonly ConversationTranscriptEntry[] {
  let changed = false;
  const next = entries.map((entry) => {
    if (entry.kind !== "message" && entry.kind !== "send-message") return entry;
    if (entry.reactions == null && entry.myReactions == null) return entry;
    changed = true;
    return { ...entry, reactions: [], myReactions: new Set<string>() };
  });
  return changed ? next : entries;
}

export interface UseTdrpReactionRenderersOptions {
  readonly activeIsHuman: boolean;
  readonly activeAgentId: string;
  readonly agentsRef: React.MutableRefObject<RendererAgent[]>;
  readonly humanReactionTransport: any;
  readonly reactionRoot: any;
  readonly accountScopeGenerationRef: React.MutableRefObject<number>;
  readonly transcriptConversationGenerationsRef: React.MutableRefObject<Map<string, number>>;
}

export function useTdrpReactionRenderers({
  activeIsHuman,
  activeAgentId,
  agentsRef,
  humanReactionTransport,
  reactionRoot,
  accountScopeGenerationRef,
  transcriptConversationGenerationsRef,
}: UseTdrpReactionRenderersOptions) {
  const resolveReactionReactorName = useCallback(
    (reactor: string) => agentsRef.current.find((agent) => agent.id === reactor)?.name ?? "the agent",
    [agentsRef]
  );

  const renderReactionPills = useCallback(
    (props: TranscriptMessageReactionPillsProps) => {
      if (!props.isDeliveryActionable || (props.entry.kind !== "message" && props.entry.kind !== "send-message" && props.entry.kind !== "user-attachment")) return null;
      const reactions = "reactions" in props.entry && Array.isArray(props.entry.reactions) ? props.entry.reactions : [];
      if (activeIsHuman) {
        if (humanReactionTransport == null || activeAgentId.length === 0) return null;
        return (
          <ReactionPills
            key={`${accountScopeGenerationRef.current}:${transcriptConversationGenerationsRef.current.get(activeAgentId) ?? 0}:${props.entry.id}`}
            agentId={activeAgentId}
            entryId={props.entry.id}
            reactions={reactions}
            resolveReactorName={resolveReactionReactorName}
            transport={humanReactionTransport}
          />
        );
      }
      if (reactionRoot == null) return null;
      return (
        <ReactionPills
          agentId={activeAgentId.length > 0 ? activeAgentId : null}
          controller={reactionRoot.pair.controller}
          entryId={props.entry.id}
          reactions={reactions}
          resolveReactorName={resolveReactionReactorName}
        />
      );
    },
    [accountScopeGenerationRef, activeAgentId, activeIsHuman, humanReactionTransport, reactionRoot, resolveReactionReactorName, transcriptConversationGenerationsRef]
  );

  const renderReactionActions = useCallback(
    (props: TranscriptMessageReactionSlotProps) => {
      if (props.isReadOnly || !props.isDeliveryActionable) return null;
      if (props.entry.kind !== "message" && props.entry.kind !== "send-message" && props.entry.kind !== "user-attachment") return null;
      const reactions = "reactions" in props.entry && Array.isArray(props.entry.reactions) ? props.entry.reactions : [];
      const myReactions = "myReactions" in props.entry && props.entry.myReactions != null
        ? props.entry.myReactions
        : new Set(reactions.filter((reaction) => reaction.by === "me").map((reaction) => reaction.emoji));
      if (activeIsHuman) {
        if (humanReactionTransport == null || activeAgentId.length === 0) return null;
        return (
          <MessageReactionAction
            key={`${accountScopeGenerationRef.current}:${transcriptConversationGenerationsRef.current.get(activeAgentId) ?? 0}:${props.entry.id}`}
            agentId={activeAgentId}
            entryId={props.entry.id}
            myReactions={myReactions}
            onExpandPicker={() => undefined}
            onOpenChange={props.onOpenChange}
            transport={humanReactionTransport}
          />
        );
      }
      if (reactionRoot == null) return null;
      return (
        <MessageReactionAction
          agentId={activeAgentId.length > 0 ? activeAgentId : null}
          controller={reactionRoot.pair.controller}
          entryId={props.entry.id}
          myReactions={myReactions}
          onExpandPicker={() => undefined}
          onOpenChange={props.onOpenChange}
        />
      );
    },
    [accountScopeGenerationRef, activeAgentId, activeIsHuman, humanReactionTransport, reactionRoot, transcriptConversationGenerationsRef]
  );

  return {
    resolveReactionReactorName,
    renderReactionPills,
    renderReactionActions,
  };
}
