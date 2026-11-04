import { useEffect, useMemo, useState } from "react";
import type { ConversationTranscriptEntry } from "../../recovered/features/conversation/workspace/model";
import { filterTranscriptEntriesByTopic } from "../transcript-projection";
import {
  resolveCommunityComposerGuard,
  resolvePinnedMessageBanner,
  selectVisibleForumTopics,
  type ForumTopicItem,
} from "../topic-workspace-projection";
import type { RendererAgent } from "../model";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import type { PinnedMessageBannerState } from "./ConversationPinnedBanner";

export interface CommunityComposerGuardState {
  readonly canSend: boolean;
  readonly bannerKind: "none" | "cooldown" | "restricted" | "closed" | string;
  readonly remainingCooldownSeconds: number;
  readonly message: string | null;
}

export interface UseTdrpTopicWorkspaceOptions {
  readonly activeAgent: RendererAgent | null;
  readonly activeAgentId: string;
  readonly client: ProductionCoordinatorClient | null;
  readonly entries: readonly ConversationTranscriptEntry[];
  readonly lastSentAtMsRef: React.MutableRefObject<number | undefined>;
}

export interface TdrpTopicWorkspaceResult {
  readonly activeTopicId: string | null;
  readonly setActiveTopicId: React.Dispatch<React.SetStateAction<string | null>>;
  readonly conversationTopics: readonly ForumTopicItem[];
  readonly setConversationTopics: React.Dispatch<React.SetStateAction<readonly ForumTopicItem[]>>;
  readonly conversationPins: readonly string[];
  readonly setConversationPins: React.Dispatch<React.SetStateAction<readonly string[]>>;
  readonly visibleTopics: readonly ForumTopicItem[];
  readonly pinnedBanner: PinnedMessageBannerState;
  readonly activeTopic: ForumTopicItem | null;
  readonly composerGuard: CommunityComposerGuardState;
  readonly displayedEntries: readonly ConversationTranscriptEntry[];
}

export function useTdrpTopicWorkspace({
  activeAgent,
  activeAgentId,
  client,
  entries,
  lastSentAtMsRef,
}: UseTdrpTopicWorkspaceOptions): TdrpTopicWorkspaceResult {
  const [activeTopicId, setActiveTopicId] = useState<string | null>(null);
  const [conversationTopics, setConversationTopics] = useState<readonly ForumTopicItem[]>([]);
  const [conversationPins, setConversationPins] = useState<readonly string[]>([]);

  const rawMetadata = (activeAgent?.raw ?? {}) as Record<string, unknown>;

  const rawTopics = useMemo((): readonly ForumTopicItem[] => {
    if (Array.isArray(rawMetadata.topics)) return rawMetadata.topics as ForumTopicItem[];
    if (Array.isArray(rawMetadata.forumTopics)) return rawMetadata.forumTopics as ForumTopicItem[];
    return [];
  }, [rawMetadata.topics, rawMetadata.forumTopics]);

  const rawPins = useMemo((): readonly string[] => {
    if (Array.isArray(rawMetadata.pins)) return rawMetadata.pins as string[];
    if (Array.isArray(rawMetadata.pinnedMessageIds)) return rawMetadata.pinnedMessageIds as string[];
    return [];
  }, [rawMetadata.pins, rawMetadata.pinnedMessageIds]);

  const canManageTopics = Boolean(
    rawMetadata.canManageTopics === true
    || (rawMetadata.permissions as any)?.canManageTopics === true
    || rawMetadata.memberStatus === "creator"
    || rawMetadata.memberStatus === "administrator"
    || rawMetadata.memberStatus === "owner"
  );

  const effectiveTopics = conversationTopics.length > 0 ? conversationTopics : rawTopics;
  const visibleTopics = useMemo(() => {
    return selectVisibleForumTopics(effectiveTopics, canManageTopics);
  }, [effectiveTopics, canManageTopics]);

  useEffect(() => {
    setActiveTopicId(null);
    setConversationTopics([]);
    setConversationPins([]);
  }, [activeAgentId]);

  useEffect(() => {
    if (client == null || activeAgent == null) return;
    const isForum = rawMetadata.isForum === true;
    const isCommunity = rawMetadata.conversationKind === "group" || rawMetadata.conversationKind === "channel";
    if (!isForum && !isCommunity) return;
    const agentId = activeAgent.id;
    let cancelled = false;
    if (isForum && rawTopics.length === 0) {
      client.call("listConversationTopics", { conversationId: agentId })
        .then((res) => {
          if (!cancelled && Array.isArray(res)) setConversationTopics(res as ForumTopicItem[]);
        })
        .catch(() => {});
    }
    if (isCommunity && rawPins.length === 0) {
      client.call("listConversationPins", { conversationId: agentId, ...(activeTopicId ? { topicId: activeTopicId } : {}) })
        .then((res) => {
          if (!cancelled && Array.isArray(res)) setConversationPins(res as string[]);
        })
        .catch(() => {});
    }
    return () => { cancelled = true; };
  }, [client, activeAgent?.id, activeTopicId, rawMetadata.isForum, rawMetadata.conversationKind, rawTopics.length, rawPins.length]);

  const effectivePins = conversationPins.length > 0 ? conversationPins : rawPins;
  const pinnedBanner = useMemo(() => {
    return resolvePinnedMessageBanner(effectivePins, entries as any, activeTopicId);
  }, [effectivePins, entries, activeTopicId]);

  const activeTopic = useMemo(() => {
    return visibleTopics.find((t) => t.id === activeTopicId) ?? null;
  }, [visibleTopics, activeTopicId]);

  const composerGuard = useMemo((): CommunityComposerGuardState => {
    if (activeAgent == null) {
      return { canSend: true, bannerKind: "none", remainingCooldownSeconds: 0, message: null };
    }
    const convKind = typeof rawMetadata.conversationKind === "string"
      ? rawMetadata.conversationKind
      : undefined;
    const memberStatus = typeof rawMetadata.memberStatus === "string" ? rawMetadata.memberStatus : undefined;
    const canPostMessages = typeof rawMetadata.canPostMessages === "boolean"
      ? rawMetadata.canPostMessages
      : (rawMetadata.permissions as any)?.canPostMessages;
    const canSendMessages = typeof rawMetadata.canSendMessages === "boolean"
      ? rawMetadata.canSendMessages
      : (rawMetadata.permissions as any)?.canSendMessages;
    const slowModeSeconds = typeof rawMetadata.slowModeSeconds === "number"
      ? rawMetadata.slowModeSeconds
      : undefined;

    return resolveCommunityComposerGuard({
      conversationKind: convKind,
      memberStatus,
      canPostMessages,
      canSendMessages,
      slowModeSeconds,
      lastSentAtMs: typeof rawMetadata.lastSentAtMs === "number" ? rawMetadata.lastSentAtMs : lastSentAtMsRef.current,
      activeTopicClosed: activeTopic?.closed === true || rawMetadata.activeTopicClosed === true,
      canManageTopics,
    });
  }, [activeAgent, rawMetadata, activeTopic, canManageTopics, lastSentAtMsRef]);

  const displayedEntries = useMemo(() => {
    if (visibleTopics.length > 0) {
      return filterTranscriptEntriesByTopic(entries, activeTopicId);
    }
    return entries;
  }, [entries, visibleTopics.length, activeTopicId]);

  return {
    activeTopicId,
    setActiveTopicId,
    conversationTopics,
    setConversationTopics,
    conversationPins,
    setConversationPins,
    visibleTopics,
    pinnedBanner,
    activeTopic,
    composerGuard,
    displayedEntries,
  };
}
