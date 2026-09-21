import { Bot, Megaphone, Network, Plus, Trash2, Users, X } from 'lucide-react';
import React, { useEffect, useMemo, useRef, useState } from 'react';
import { FabButton, FabInput } from '../ui/primitives/fab-primitives';
import type { AgentPeerMessage, GroupSummary } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { agentMatchesGroupMember, indexAgentsByRuntimeOrSurfaceId, type AgentSidebarItem } from './agent-model';
import styles from './agent-network.module.css';

export interface AgentNetworkProps {
  open: boolean;
  agents: readonly AgentSidebarItem[];
  groups: readonly GroupSummary[];
  peerMessagesByAgentId: Readonly<Record<string, readonly AgentPeerMessage[]>>;
  activeKey: string | null;
  broadcastMode?: boolean;
  onClose(): void;
  onOpenAgent(agent: AgentSidebarItem): void;
  onRefreshGroups(): Promise<void>;
  onRefreshPeerHistory(agentId: string): Promise<void>;
  onCreateGroup(name: string, memberAgentIds: readonly string[]): Promise<void>;
  onUpdateGroup(id: string, patch: { name?: string; memberAgentIds?: readonly string[] }): Promise<void>;
  onDeleteGroup(id: string): Promise<void>;
  onSendGroup(id: string, message: string): Promise<void>;
  onSendPeer(fromAgentId: string, targetAgentId: string, message: string, priority?: boolean): Promise<void>;
  onBroadcast(message: string, targetAgentIds?: readonly string[]): Promise<void>;
}

function statusLabel(agent: AgentSidebarItem): string {
  if (agent.waitingReason?.trim()) return `Waiting for you · ${agent.waitingReason.trim()}`;
  if (agent.busy) return 'Working';
  if (agent.unread > 0) return 'Unread activity';
  return agent.lastMessage?.trim() || agent.description || 'Agent';
}

/**
 * Agent collaboration surface backed by Mahayana group.* / agent.broadcast.
 *
 * Runtime groups are never projected as Messenger peers here. A group is an
 * Agent-domain object with stable member Agent ids, while each member still
 * owns its own conversation, draft, operation and Computer context.
 */
export default function AgentNetwork({
  open,
  agents,
  groups,
  peerMessagesByAgentId,
  activeKey,
  broadcastMode = false,
  onClose,
  onOpenAgent,
  onRefreshGroups,
  onRefreshPeerHistory,
  onCreateGroup,
  onUpdateGroup,
  onDeleteGroup,
  onSendGroup,
  onSendPeer,
  onBroadcast,
}: AgentNetworkProps) {
  const [selected, setSelected] = useState<Set<string>>(() => new Set());
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const [message, setMessage] = useState('');
  const [sending, setSending] = useState(false);
  const [priority, setPriority] = useState(false);
  const [groupBusy, setGroupBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const refreshGroupsRef = useRef(onRefreshGroups);
  refreshGroupsRef.current = onRefreshGroups;

  const directAgents = useMemo(
    () => agents.filter((agent) => !agent.isGroup && !agent.hidden),
    [agents],
  );
  const agentByMemberId = useMemo(
    () => indexAgentsByRuntimeOrSurfaceId(directAgents),
    [directAgents],
  );
  const activeAgent = directAgents.find((agent) => agent.key === activeKey) ?? null;

  const recentNetworkMessages = useMemo(() => Object.values(peerMessagesByAgentId)
    .flatMap((items) => [...items])
    .sort((left, right) => right.createdAtMs - left.createdAtMs)
    .slice(0, 12), [peerMessagesByAgentId]);
  const coordinationStats = useMemo(() => ({
    working: directAgents.filter((agent) => agent.busy).length,
    waiting: directAgents.filter((agent) => Boolean(agent.waitingReason)).length,
    unread: directAgents.filter((agent) => agent.unread > 0).length,
  }), [directAgents]);
  const organizationEdges = useMemo(() => groups.flatMap((group) => group.memberIds
    .map((memberId) => ({
      id: `${group.id}:${memberId}`,
      groupId: group.id,
      groupName: group.name,
      agent: agentByMemberId.get(memberId),
    }))
    .filter((edge) => Boolean(edge.agent))), [agentByMemberId, groups]);

  useEffect(() => {
    if (!open) return;
    setError(null);
    void refreshGroupsRef.current().catch((cause) => setError(cause instanceof Error ? cause.message : String(cause)));
    if (!broadcastMode) {
      setSelected(new Set());
      setSelectedGroupId(null);
    }
  }, [broadcastMode, open]);

  useEffect(() => {
    if (!open || !activeAgent?.agentId) return;
    void onRefreshPeerHistory(activeAgent.agentId).catch((cause) => {
      setError(cause instanceof Error ? cause.message : String(cause));
    });
  }, [activeAgent?.agentId, onRefreshPeerHistory, open]);

  if (!open) return null;

  const selectedAgentIds = directAgents
    .filter((agent) => selected.has(agent.key) && (broadcastMode || agent.key !== activeKey))
    .map((agent) => agent.agentId);
  const selectedGroup = selectedGroupId ? groups.find((group) => group.id === selectedGroupId) ?? null : null;
  const directTarget = !broadcastMode && !selectedGroup && activeAgent && selectedAgentIds.length === 1
    ? directAgents.find((agent) => agent.agentId === selectedAgentIds[0] && agent.key !== activeAgent.key) ?? null
    : null;
  const visiblePeerMessages = activeAgent
    ? (peerMessagesByAgentId[activeAgent.agentId] ?? []).slice(-8)
    : [];

  const chooseGroup = (group: GroupSummary) => {
    const keys = directAgents
      .filter((agent) => group.memberIds.some((memberId) => agentMatchesGroupMember(agent, memberId)))
      .map((agent) => agent.key);
    setSelected(new Set(keys));
    setSelectedGroupId(group.id);
  };

  const toggleAgent = (agent: AgentSidebarItem) => {
    setSelectedGroupId(null);
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(agent.key)) next.delete(agent.key);
      else next.add(agent.key);
      return next;
    });
  };

  const createGroup = async () => {
    if (selectedAgentIds.length < 2 || groupBusy) return;
    const suggested = selectedAgentIds.map((id) => agentByMemberId.get(id)?.name).filter(Boolean).slice(0, 3).join(' + ');
    const name = window.prompt('Group name', suggested || 'Agent group')?.trim();
    if (!name) return;
    setGroupBusy(true);
    setError(null);
    try {
      await onCreateGroup(name, selectedAgentIds);
      await onRefreshGroups();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setGroupBusy(false);
    }
  };

  const renameGroup = async (group: GroupSummary) => {
    if (groupBusy) return;
    const name = window.prompt('Rename Agent group', group.name)?.trim();
    if (!name || name === group.name) return;
    setGroupBusy(true);
    setError(null);
    try {
      await onUpdateGroup(group.id, { name });
      await onRefreshGroups();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setGroupBusy(false);
    }
  };

  const deleteGroup = async (group: GroupSummary) => {
    if (groupBusy || !window.confirm(`Delete Agent group “${group.name}”? Agents and their histories will not be deleted.`)) return;
    setGroupBusy(true);
    setError(null);
    try {
      await onDeleteGroup(group.id);
      if (selectedGroupId === group.id) {
        setSelectedGroupId(null);
        setSelected(new Set());
      }
      await onRefreshGroups();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setGroupBusy(false);
    }
  };

  const submit = async () => {
    const trimmed = message.trim();
    if (!trimmed || sending) return;
    setSending(true);
    setError(null);
    try {
      if (selectedGroup) await onSendGroup(selectedGroup.id, trimmed);
      else if (directTarget && activeAgent) await onSendPeer(activeAgent.agentId, directTarget.agentId, trimmed, priority);
      else await onBroadcast(trimmed, selectedAgentIds.length ? selectedAgentIds : undefined);
      setMessage('');
      setPriority(false);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setSending(false);
    }
  };

  return <section className={styles.root} data-testid="grok-agent-network" aria-label="Agent network">
    <header className={styles.header}>
      <div>
        <Network size={18} />
        <span><strong>Agent Network</strong><small>{directAgents.length} agents · {groups.length} groups</small></span>
      </div>
      <FabButton variant="bare" type="button" onClick={onClose} aria-label="Close Agent network"><X size={17} /></FabButton>
    </header>

    <div className={styles.body}>
      <div className={styles.graph}>
        <section className={styles.orgGraph} data-testid="agent-org-graph" aria-label="Agent organization graph">
          <div className={styles.orgSummary}>
            <span><strong>{directAgents.length}</strong><small>Agents</small></span>
            <span><strong>{coordinationStats.working}</strong><small>Working</small></span>
            <span><strong>{coordinationStats.waiting}</strong><small>Needs input</small></span>
            <span><strong>{groups.length}</strong><small>Groups</small></span>
          </div>
          <div className={styles.orgTopology}>
            <div className={styles.orgHub}>
              <Network size={18} />
              <span><strong>{activeAgent?.name ?? 'Agent Network'}</strong><small>{activeAgent ? 'Current coordination hub' : 'Select an Agent to coordinate'}</small></span>
            </div>
            <div className={styles.orgEdges} aria-label="Organization relationships">
              {organizationEdges.slice(0, 18).map((edge) => <FabButton variant="bare"
                type="button"
                className={styles.orgEdge}
                key={edge.id}
                onClick={() => edge.agent && onOpenAgent(edge.agent)}
              >
                <span>{edge.groupName}</span><b aria-hidden="true">→</b><strong>{edge.agent?.name}</strong>
              </FabButton>)}
              {!organizationEdges.length ? <span className={styles.orgEmpty}>Create a group to map durable Agent relationships.</span> : null}
            </div>
          </div>
          {recentNetworkMessages.length ? <div className={styles.coordinationFeed} data-testid="agent-coordination-feed">
            {recentNetworkMessages.slice(0, 6).map((item) => <FabButton variant="bare" type="button" key={item.id} onClick={() => {
              const target = agentByMemberId.get(item.targetId);
              if (target) onOpenAgent(target);
            }}>
              <span><strong>{item.fromAgentName}</strong><b aria-hidden="true">→</b><strong>{item.targetName}</strong>{item.priority ? <em>Priority</em> : null}</span>
              <small>{item.text}</small>
            </FabButton>)}
          </div> : null}
        </section>

        <div className={styles.sectionHeading}><Bot size={15} /><span>Agents</span><small className={styles.headingMeta}>{coordinationStats.unread} unread</small></div>
        <div className={styles.nodes}>
          {directAgents.map((agent) => <article
            className={styles.node}
            data-active={agent.key === activeKey || undefined}
            data-busy={agent.busy || undefined}
            data-waiting={Boolean(agent.waitingReason) || undefined}
            key={agent.key}
          >
            <FabButton variant="bare" type="button" className={styles.openNode} onClick={() => onOpenAgent(agent)}>
              <span className={styles.nodeMark}><Bot size={17} /></span>
              <span><strong>{agent.name}</strong><small>{statusLabel(agent)}</small></span>
            </FabButton>
            <label className={styles.select}>
              <FabInput variant="bare"
                type="checkbox"
                checked={selected.has(agent.key)}
                disabled={!broadcastMode && agent.key === activeKey}
                onChange={() => toggleAgent(agent)}
              />
              <span>{!broadcastMode && agent.key === activeKey ? 'Current Agent' : broadcastMode ? 'Broadcast' : 'Select'}</span>
            </label>
          </article>)}
          {!directAgents.length ? <div className={styles.empty}>Create an Agent to build your network.</div> : null}
        </div>

        <div className={styles.sectionHeading}>
          <Users size={15} /><span>Groups</span>
          <FabButton variant="bare" type="button" className={styles.sectionAction} disabled={selectedAgentIds.length < 2 || groupBusy} onClick={() => void createGroup()}>
            <Plus size={13} />Create from selected
          </FabButton>
        </div>
        <div className={styles.nodes}>
          {groups.map((group) => {
            const memberNames = group.memberIds.map((id) => agentByMemberId.get(id)?.name ?? id).join(', ');
            return <article className={styles.node} data-active={selectedGroupId === group.id || undefined} key={group.id}>
              <FabButton variant="bare" type="button" className={styles.openNode} onClick={() => chooseGroup(group)}>
                <span className={styles.nodeMark}><Users size={17} /></span>
                <span><strong>{group.name}</strong><small>{group.memberIds.length} agents{memberNames ? ` · ${memberNames}` : ''}</small></span>
              </FabButton>
              <div className={styles.groupActions}>
                <FabButton variant="bare" type="button" onClick={() => void renameGroup(group)}>Rename</FabButton>
                <FabButton variant="bare" type="button" aria-label={`Delete ${group.name}`} onClick={() => void deleteGroup(group)}><Trash2 size={13} /></FabButton>
              </div>
            </article>;
          })}
          {!groups.length ? <div className={styles.empty}>Select at least two Agents to create a runtime Agent group.</div> : null}
        </div>
      </div>

      <aside className={styles.broadcast}>
        <div className={styles.broadcastTitle}><Megaphone size={16} /><strong>{selectedGroup ? selectedGroup.name : directTarget ? 'Agent handoff' : 'Broadcast'}</strong></div>
        <p>{selectedGroup
          ? `Send one group message through Mahayana to ${selectedGroup.memberIds.length} member Agents.`
          : directTarget && activeAgent
            ? `Send directly from ${activeAgent.name} to ${directTarget.name} through Mahayana agent.send. This is Agent-to-Agent context, not an owner broadcast.`
            : `Send one owner message to ${selectedAgentIds.length ? `${selectedAgentIds.length} selected Agent${selectedAgentIds.length === 1 ? '' : 's'}` : 'all Agents'}. Each Agent receives it as its own asynchronous turn.`}</p>
        <textarea
          autoFocus={broadcastMode}
          value={message}
          onChange={(event) => setMessage(event.target.value)}
          placeholder="Tell your agents what changed or what to do next…"
          rows={7}
        />
        {directTarget ? <label className={styles.select}><FabInput variant="bare" type="checkbox" checked={priority} onChange={(event) => setPriority(event.target.checked)} /><span>Priority handoff</span></label> : null}
        {visiblePeerMessages.length ? <div className={styles.nodes} aria-label="Recent Agent handoffs">
          {visiblePeerMessages.map((item) => <div className={styles.empty} key={item.id}><strong>{item.fromAgentName} → {item.targetName}</strong><span>{item.text}</span></div>)}
        </div> : null}
        {error ? <div className={styles.error} role="alert">{error}</div> : null}
        <FabButton variant="bare" type="button" className={styles.send} disabled={!message.trim() || sending || directAgents.length === 0} onClick={() => void submit()}>
          <Megaphone size={15} />{sending ? 'Sending…' : selectedGroup ? 'Send to group' : directTarget ? `Handoff to ${directTarget.name}` : selectedAgentIds.length ? 'Send to selected' : 'Broadcast to all'}
        </FabButton>
      </aside>
    </div>
  </section>;
}
