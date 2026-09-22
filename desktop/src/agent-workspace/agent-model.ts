import {
  conversationIdentityOf,
  isAgentIdentity,
  type AgentProjectionKind,
} from './conversation-identity';

export interface AgentPeerProjection {
  key: string;
  id: string;
  actorId?: string;
  agentId?: string;
  conversationId?: string;
  kind: AgentProjectionKind;
  hidden?: boolean;
  title: string;
  subtitle: string;
  pinned: boolean;
  unread: number;
  updatedAtMs: number;
}

export interface AgentActivityProjection {
  draftPrompt?: string;
  lastMessage?: string;
  waitingReason?: string;
  currentActivity?: string;
  isComposingMessage?: boolean;
}

export interface AgentSidebarItem {
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
  /** Fabu row detail precedence: draft -> waiting -> working -> last message. */
  draftPrompt?: string;
  lastMessage?: string;
  waitingReason?: string;
  currentActivity?: string;
  isComposingMessage?: boolean;
}

export function agentWorkspaceKey(peer: AgentPeerProjection): string {
  const identity = conversationIdentityOf(peer);
  if (!isAgentIdentity(identity)) {
    throw new Error(`Agent workspace received non-Agent identity: ${identity.type}`);
  }
  return peer.kind === 'group'
    ? `group:${peer.id}`
    : `agent:${identity.agentId}`;
}

export function agentMatchesGroupMember(agent: AgentSidebarItem, memberId: string): boolean {
  return memberId === agent.agentId || memberId === agent.id;
}

export function indexAgentsByRuntimeOrSurfaceId(
  agents: readonly AgentSidebarItem[],
): ReadonlyMap<string, AgentSidebarItem> {
  const index = new Map<string, AgentSidebarItem>();
  for (const agent of agents) {
    index.set(agent.agentId, agent);
    index.set(agent.id, agent);
  }
  return index;
}

function mergeActivity(
  preferred: AgentSidebarItem,
  fallback: AgentSidebarItem,
): AgentSidebarItem {
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
    currentActivity: preferred.currentActivity || fallback.currentActivity,
    isComposingMessage: preferred.isComposingMessage || fallback.isComposingMessage,
  };
}

function pinnedRank(key: string, pinnedOrder: readonly string[]): number {
  const index = pinnedOrder.indexOf(key);
  return index < 0 ? Number.MAX_SAFE_INTEGER : index;
}

/**
 * Product-owned renderer-model boundary for Agent navigation.
 *
 * The visible sidebar consumes stable Agent projections, never raw Messenger
 * peers. Compatibility peers can temporarily duplicate one runtime Agent while
 * account membership and conversation hydration converge; collapse them here.
 */
export function projectAgentSidebarItems(
  peers: readonly AgentPeerProjection[],
  operationByPeer: Readonly<Record<string, string>>,
  activityByPeer: Readonly<Record<string, AgentActivityProjection>> = {},
  pinnedOrder: readonly string[] = [],
): AgentSidebarItem[] {
  const visiblePeers = peers.filter((peer) => isAgentIdentity(conversationIdentityOf(peer)));
  const peerByKey = new Map(visiblePeers.map((peer) => [peer.key, peer] as const));
  const items = new Map<string, AgentSidebarItem>();

  for (const peer of visiblePeers) {
    const key = agentWorkspaceKey(peer);
    const existing = items.get(key);
    const activity = activityByPeer[peer.key] ?? {};
    const candidate: AgentSidebarItem = {
      key,
      peerKey: peer.key,
      id: peer.id,
      agentId: peer.agentId ?? peer.actorId ?? peer.id,
      name: peer.title,
      description: peer.subtitle,
      pinned: pinnedOrder.includes(key),
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

export function projectActiveAgentKey(
  peer: AgentPeerProjection | null | undefined,
): string | null {
  return peer && isAgentIdentity(conversationIdentityOf(peer))
    ? agentWorkspaceKey(peer)
    : null;
}
