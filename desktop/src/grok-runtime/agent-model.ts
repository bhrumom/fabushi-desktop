export interface GrokAgentPeerProjection {
  key: string;
  id: string;
  actorId?: string;
  agentId?: string;
  conversationId?: string;
  kind: string;
  hidden?: boolean;
  title: string;
  subtitle: string;
  pinned: boolean;
  unread: number;
  updatedAtMs: number;
}

export interface GrokAgentActivityProjection {
  draftPrompt?: string;
  lastMessage?: string;
  waitingReason?: string;
  isComposingMessage?: boolean;
}

export interface GrokAgentSidebarItem {
  /** Stable Agent/group identity used by the Agent-first shell. */
  key: string;
  /** Current compatibility peer used to open the underlying conversation. */
  peerKey: string;
  id: string;
  agentId: string;
  name: string;
  description: string;
  pinned: boolean;
  hidden: boolean;
  unread: number;
  busy: boolean;
  isGroup: boolean;
  updatedAtMs: number;
  /** Fabu/Grok row detail precedence: draft -> waiting -> working -> last message. */
  draftPrompt?: string;
  lastMessage?: string;
  waitingReason?: string;
  isComposingMessage?: boolean;
}

export function grokAgentKey(peer: GrokAgentPeerProjection): string {
  return peer.kind === 'group'
    ? `group:${peer.id}`
    : `agent:${peer.agentId ?? peer.actorId ?? peer.id}`;
}

function mergeActivity(
  preferred: GrokAgentSidebarItem,
  fallback: GrokAgentSidebarItem,
): GrokAgentSidebarItem {
  return {
    ...preferred,
    pinned: preferred.pinned || fallback.pinned,
    hidden: preferred.hidden && fallback.hidden,
    unread: Math.max(preferred.unread, fallback.unread),
    busy: preferred.busy || fallback.busy,
    updatedAtMs: Math.max(preferred.updatedAtMs, fallback.updatedAtMs),
    draftPrompt: preferred.draftPrompt || fallback.draftPrompt,
    lastMessage: preferred.lastMessage || fallback.lastMessage,
    waitingReason: preferred.waitingReason || fallback.waitingReason,
    isComposingMessage: preferred.isComposingMessage || fallback.isComposingMessage,
  };
}

function pinnedRank(key: string, pinnedOrder: readonly string[]): number {
  const index = pinnedOrder.indexOf(key);
  return index < 0 ? Number.MAX_SAFE_INTEGER : index;
}

/**
 * Mirrors Fabu's renderer-model boundary: the visible sidebar consumes an
 * Agent projection, never the raw transport/conversation list.
 *
 * One runtime Agent can momentarily have more than one compatibility peer
 * while account Bot membership and conversation hydration converge. Collapse
 * those peers here so the UI never flickers or duplicates the Agent.
 */
export function projectGrokAgentSidebarItems(
  peers: readonly GrokAgentPeerProjection[],
  operationByPeer: Readonly<Record<string, string>>,
  activityByPeer: Readonly<Record<string, GrokAgentActivityProjection>> = {},
  pinnedOrder: readonly string[] = [],
): GrokAgentSidebarItem[] {
  const visiblePeers = peers.filter((peer) => peer.kind === 'bot' || peer.kind === 'group');
  const peerByKey = new Map(visiblePeers.map((peer) => [peer.key, peer] as const));
  const items = new Map<string, GrokAgentSidebarItem>();

  for (const peer of visiblePeers) {
    const key = grokAgentKey(peer);
    const existing = items.get(key);
    const activity = activityByPeer[peer.key] ?? {};
    const candidate: GrokAgentSidebarItem = {
      key,
      peerKey: peer.key,
      id: peer.id,
      agentId: peer.agentId ?? peer.actorId ?? peer.id,
      name: peer.title,
      description: peer.subtitle,
      pinned: peer.pinned,
      hidden: peer.hidden === true,
      unread: peer.unread,
      busy: Boolean(operationByPeer[peer.key]),
      isGroup: peer.kind === 'group',
      updatedAtMs: peer.updatedAtMs,
      ...activity,
    };

    if (!existing) {
      items.set(key, candidate);
      continue;
    }

    const candidateHasConversation = Boolean(peer.conversationId);
    const existingHasConversation = Boolean(peerByKey.get(existing.peerKey)?.conversationId);
    const candidateWins = (candidateHasConversation && !existingHasConversation)
      || (candidateHasConversation === existingHasConversation
        && ((candidate.pinned && !existing.pinned)
          || candidate.updatedAtMs > existing.updatedAtMs));

    items.set(
      key,
      candidateWins
        ? mergeActivity(candidate, existing)
        : mergeActivity(existing, candidate),
    );
  }

  return [...items.values()].sort((left, right) => {
    if (left.pinned !== right.pinned) return left.pinned ? -1 : 1;
    if (left.pinned && right.pinned) {
      const rankDelta = pinnedRank(left.key, pinnedOrder) - pinnedRank(right.key, pinnedOrder);
      if (rankDelta !== 0) return rankDelta;
    }
    return right.updatedAtMs - left.updatedAtMs;
  });
}

export function projectActiveGrokAgentKey(
  peer: GrokAgentPeerProjection | null | undefined,
): string | null {
  return peer && (peer.kind === 'bot' || peer.kind === 'group')
    ? grokAgentKey(peer)
    : null;
}
