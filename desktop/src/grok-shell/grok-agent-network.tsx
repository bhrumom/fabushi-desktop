import { Bot, Megaphone, Network, Users, X } from 'lucide-react';
import React, { useEffect, useMemo, useState } from 'react';
import type { GrokAgentSidebarItem } from '../grok-runtime/agent-model';
import styles from './grok-agent-network.module.css';

export type GrokAgentNetworkProps = {
  open: boolean;
  agents: readonly GrokAgentSidebarItem[];
  activeKey: string | null;
  broadcastMode?: boolean;
  onClose(): void;
  onOpenAgent(agent: GrokAgentSidebarItem): void;
  onBroadcast(message: string, targetAgentIds?: string[]): Promise<void>;
};

function statusLabel(agent: GrokAgentSidebarItem): string {
  if (agent.waitingReason?.trim()) return `Waiting for you · ${agent.waitingReason.trim()}`;
  if (agent.busy) return 'Working';
  if (agent.unread > 0) return 'Unread activity';
  return agent.lastMessage?.trim() || agent.description || (agent.isGroup ? 'Agent group' : 'Agent');
}

export default function GrokAgentNetwork({
  open,
  agents,
  activeKey,
  broadcastMode = false,
  onClose,
  onOpenAgent,
  onBroadcast,
}: GrokAgentNetworkProps) {
  const [selected, setSelected] = useState<Set<string>>(() => new Set());
  const [message, setMessage] = useState('');
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const directAgents = useMemo(() => agents.filter((agent) => !agent.isGroup && !agent.hidden), [agents]);
  const groups = useMemo(() => agents.filter((agent) => agent.isGroup && !agent.hidden), [agents]);

  useEffect(() => {
    if (!open) return;
    setError(null);
    if (!broadcastMode) setSelected(new Set());
  }, [broadcastMode, open]);

  if (!open) return null;

  const selectedAgentIds = directAgents
    .filter((agent) => selected.has(agent.key))
    .map((agent) => agent.agentId);
  const submit = async () => {
    const trimmed = message.trim();
    if (!trimmed || sending) return;
    setSending(true);
    setError(null);
    try {
      await onBroadcast(trimmed, selectedAgentIds.length ? selectedAgentIds : undefined);
      setMessage('');
      setSelected(new Set());
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
      <button type="button" onClick={onClose} aria-label="Close Agent network"><X size={17} /></button>
    </header>

    <div className={styles.body}>
      <div className={styles.graph}>
        <div className={styles.sectionHeading}><Bot size={15} /><span>Agents</span></div>
        <div className={styles.nodes}>
          {directAgents.map((agent) => <article
            className={styles.node}
            data-active={agent.key === activeKey || undefined}
            data-busy={agent.busy || undefined}
            data-waiting={Boolean(agent.waitingReason) || undefined}
            key={agent.key}
          >
            <button type="button" className={styles.openNode} onClick={() => onOpenAgent(agent)}>
              <span className={styles.nodeMark}><Bot size={17} /></span>
              <span><strong>{agent.name}</strong><small>{statusLabel(agent)}</small></span>
            </button>
            <label className={styles.select}>
              <input
                type="checkbox"
                checked={selected.has(agent.key)}
                onChange={() => setSelected((current) => {
                  const next = new Set(current);
                  if (next.has(agent.key)) next.delete(agent.key);
                  else next.add(agent.key);
                  return next;
                })}
              />
              <span>Broadcast</span>
            </label>
          </article>)}
          {!directAgents.length ? <div className={styles.empty}>Create an Agent to build your network.</div> : null}
        </div>

        {groups.length ? <>
          <div className={styles.sectionHeading}><Users size={15} /><span>Groups</span></div>
          <div className={styles.nodes}>
            {groups.map((group) => <article className={styles.node} data-active={group.key === activeKey || undefined} key={group.key}>
              <button type="button" className={styles.openNode} onClick={() => onOpenAgent(group)}>
                <span className={styles.nodeMark}><Users size={17} /></span>
                <span><strong>{group.name}</strong><small>{statusLabel(group)}</small></span>
              </button>
            </article>)}
          </div>
        </> : null}
      </div>

      <aside className={styles.broadcast}>
        <div className={styles.broadcastTitle}><Megaphone size={16} /><strong>Broadcast</strong></div>
        <p>Send one owner message to {selectedAgentIds.length ? `${selectedAgentIds.length} selected Agent${selectedAgentIds.length === 1 ? '' : 's'}` : 'all Agents'}. Each Agent receives it as its own asynchronous turn.</p>
        <textarea
          autoFocus={broadcastMode}
          value={message}
          onChange={(event) => setMessage(event.target.value)}
          placeholder="Tell your agents what changed or what to do next…"
          rows={7}
        />
        {error ? <div className={styles.error} role="alert">{error}</div> : null}
        <button type="button" className={styles.send} disabled={!message.trim() || sending || directAgents.length === 0} onClick={() => void submit()}>
          <Megaphone size={15} />{sending ? 'Sending…' : selectedAgentIds.length ? 'Send to selected' : 'Broadcast to all'}
        </button>
      </aside>
    </div>
  </section>;
}
