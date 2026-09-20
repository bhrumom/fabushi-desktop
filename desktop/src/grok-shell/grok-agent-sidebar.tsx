import { Bot, ChevronsLeft, ChevronsRight, Copy, EyeOff, MoreHorizontal, Pencil, Pin, Plus, Search, Settings, Plug, Trash2 } from 'lucide-react';
import React, { useMemo, useState } from 'react';
import { BotMark, type BotMarkState } from '../../../frontend/apps/web/src/app/host/bot-mark';
import styles from './grok-agent-sidebar.module.css';

export type GrokAgentSidebarItem = {
  key: string;
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
};

export type GrokAgentSidebarProps = {
  agents: readonly GrokAgentSidebarItem[];
  activeKey: string | null;
  query: string;
  collapsed: boolean;
  hostReady: boolean;
  accountLabel: string;
  onQuery(value: string): void;
  onOpen(item: GrokAgentSidebarItem): void;
  onNewAgent(): void;
  onToggleCollapsed(): void;
  onTogglePin(item: GrokAgentSidebarItem): void;
  onRename(item: GrokAgentSidebarItem): void;
  onHide(item: GrokAgentSidebarItem): void;
  onDuplicate(item: GrokAgentSidebarItem): void;
  onDelete(item: GrokAgentSidebarItem): void;
  onOpenPlugins(): void;
  onOpenSettings(): void;
};

function activityState(item: GrokAgentSidebarItem, hostReady: boolean): BotMarkState {
  if (item.busy) return 'working';
  if (item.unread > 0) return 'notifying';
  return hostReady ? 'idle' : 'waking';
}

function relativeTime(updatedAtMs: number): string {
  if (!updatedAtMs) return '';
  const delta = Math.max(0, Date.now() - updatedAtMs);
  if (delta < 60_000) return 'now';
  const minutes = Math.round(delta / 60_000);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.round(minutes / 60);
  return hours < 24 ? `${hours}h` : `${Math.round(hours / 24)}d`;
}

function AgentRow({
  item,
  active,
  collapsed,
  hostReady,
  onOpen,
  onTogglePin,
  onRename,
  onHide,
  onDuplicate,
  onDelete,
}: {
  item: GrokAgentSidebarItem;
  active: boolean;
  collapsed: boolean;
  hostReady: boolean;
  onOpen(): void;
  onTogglePin(): void;
  onRename(): void;
  onHide(): void;
  onDuplicate(): void;
  onDelete(): void;
}) {
  const [menuOpen, setMenuOpen] = useState(false);
  return <div className={styles.rowWrap} data-active={active || undefined} data-pinned={item.pinned || undefined}>
    <button
      type="button"
      className={styles.row}
      aria-current={active ? 'page' : undefined}
      aria-label={item.name}
      data-testid={`peer-${item.key}`}
      data-agent-id={item.agentId}
      onClick={onOpen}
    >
      <span className={styles.avatar}>
        <BotMark
          botId={`grok-agent:${item.agentId}`}
          state={activityState(item, hostReady)}
          size={collapsed ? 36 : 38}
          label={item.name}
        />
        {item.busy ? <i className={styles.workingDot} title="Working" /> : item.unread ? <i className={styles.unreadDot} title="Unread activity" /> : null}
      </span>
      {collapsed ? null : <span className={styles.copy}>
        <span className={styles.nameLine}>
          <strong>{item.name}</strong>
          <time>{relativeTime(item.updatedAtMs)}</time>
        </span>
        <small>{item.busy ? 'Working…' : item.description || (item.isGroup ? 'Agent group' : 'Agent')}</small>
      </span>}
    </button>
    {collapsed ? null : <button
      type="button"
      className={styles.more}
      aria-label={`${item.name} actions`}
      aria-expanded={menuOpen}
      onClick={(event) => {
        event.stopPropagation();
        setMenuOpen((value) => !value);
      }}
    ><MoreHorizontal size={16} /></button>}
    {!collapsed && menuOpen ? <div className={styles.menu} role="menu" onClick={(event) => event.stopPropagation()}>
      <button type="button" role="menuitem" onClick={() => { setMenuOpen(false); onTogglePin(); }}><Pin size={14} />{item.pinned ? 'Unpin' : 'Pin'}</button>
      {!item.isGroup ? <button type="button" role="menuitem" onClick={() => { setMenuOpen(false); onRename(); }}><Pencil size={14} />Rename</button> : null}
      {!item.isGroup ? <button type="button" role="menuitem" onClick={() => { setMenuOpen(false); onDuplicate(); }}><Copy size={14} />Duplicate</button> : null}
      {!item.isGroup ? <button type="button" role="menuitem" onClick={() => { setMenuOpen(false); onHide(); }}><EyeOff size={14} />Hide</button> : null}
      {!item.isGroup ? <button type="button" role="menuitem" data-danger="true" onClick={() => { setMenuOpen(false); onDelete(); }}><Trash2 size={14} />Delete</button> : null}
    </div> : null}
  </div>;
}

export default function GrokAgentSidebar(props: GrokAgentSidebarProps) {
  const normalized = props.query.trim().toLocaleLowerCase();
  const visible = useMemo(
    () => props.agents.filter((item) => !item.hidden && (!normalized || `${item.name} ${item.description}`.toLocaleLowerCase().includes(normalized))),
    [props.agents, normalized],
  );
  const pinned = visible.filter((item) => item.pinned);
  const unpinned = visible.filter((item) => !item.pinned);
  const hiddenCount = props.agents.filter((item) => item.hidden).length;

  return <div className={styles.root} data-collapsed={props.collapsed || undefined}>
    <header className={styles.header}>
      <button type="button" className={styles.newButton} onClick={props.onNewAgent} title="New chat" data-testid="grok-new-agent">
        <Plus size={18} /><span>{props.collapsed ? null : 'New'}</span>
      </button>
      <button type="button" className={styles.collapseButton} onClick={props.onToggleCollapsed} title={props.collapsed ? 'Expand sidebar' : 'Collapse sidebar'}>
        {props.collapsed ? <ChevronsRight size={16} /> : <ChevronsLeft size={16} />}
      </button>
    </header>

    {props.collapsed ? null : <label className={styles.search}>
      <Search size={15} />
      <input value={props.query} onChange={(event) => props.onQuery(event.target.value)} placeholder="Search" aria-label="Search agents" />
    </label>}

    <div className={styles.list}>
      {pinned.length && !props.collapsed ? <div className={styles.sectionLabel}>Pinned</div> : null}
      {pinned.map((item) => <AgentRow key={item.key} item={item} active={item.key === props.activeKey} collapsed={props.collapsed} hostReady={props.hostReady} onOpen={() => props.onOpen(item)} onTogglePin={() => props.onTogglePin(item)} onRename={() => props.onRename(item)} onHide={() => props.onHide(item)} onDuplicate={() => props.onDuplicate(item)} onDelete={() => props.onDelete(item)} />)}
      {unpinned.length && !props.collapsed ? <div className={styles.sectionLabel}>Agents</div> : null}
      {unpinned.map((item) => <AgentRow key={item.key} item={item} active={item.key === props.activeKey} collapsed={props.collapsed} hostReady={props.hostReady} onOpen={() => props.onOpen(item)} onTogglePin={() => props.onTogglePin(item)} onRename={() => props.onRename(item)} onHide={() => props.onHide(item)} onDuplicate={() => props.onDuplicate(item)} onDelete={() => props.onDelete(item)} />)}
      {!visible.length && !props.collapsed ? <div className={styles.empty}><Bot size={24} /><strong>No agents yet</strong><small>Create a new chat to start an Agent.</small></div> : null}
    </div>

    <footer className={styles.footer}>
      {hiddenCount > 0 && !props.collapsed ? <div className={styles.hiddenHint}><EyeOff size={14} /><span>{hiddenCount} hidden bots</span></div> : null}
      <button type="button" onClick={props.onOpenPlugins} title="Plugins"><Plug size={17} />{props.collapsed ? null : <span>Plugins</span>}</button>
      <button type="button" onClick={props.onOpenSettings} title="Settings"><Settings size={17} />{props.collapsed ? null : <span>{props.accountLabel}</span>}</button>
    </footer>
  </div>;
}
