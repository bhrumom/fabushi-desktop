import {
  Bot,
  ChevronDown,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Copy,
  EyeOff,
  FolderPlus,
  Megaphone,
  MoreHorizontal,
  Network,
  Pencil,
  Pin,
  Plug,
  Plus,
  Search,
  Settings,
  Trash2,
  X,
} from 'lucide-react';
import React, { useMemo, useState } from 'react';
import FabAvatar, { type FabAvatarState } from '../ui/avatar/fab-avatar';
import {
  AGENT_SIDEBAR_UNASSIGNED_ID,
  projectAgentSidebarSections,
  type AgentSidebarSection,
} from '../agent-workspace/agent-sidebar-state';
import type { GrokAgentSidebarItem } from '../grok-runtime/agent-model';
import styles from './grok-agent-sidebar.module.css';

export type { GrokAgentSidebarItem } from '../grok-runtime/agent-model';

export type GrokAgentSidebarProps = {
  agents: readonly GrokAgentSidebarItem[];
  activeKey: string | null;
  query: string;
  collapsed: boolean;
  hostReady: boolean;
  accountLabel: string;
  sections?: readonly AgentSidebarSection[];
  selectedKeys?: readonly string[];
  onQuery(value: string): void;
  onOpen(item: GrokAgentSidebarItem): void;
  onNewAgent(): void;
  onToggleCollapsed(): void;
  onTogglePin(item: GrokAgentSidebarItem): void;
  onRename(item: GrokAgentSidebarItem): void;
  onHide(item: GrokAgentSidebarItem): void;
  onDuplicate(item: GrokAgentSidebarItem): void;
  onDelete(item: GrokAgentSidebarItem): void;
  onReorderPinned(moved: GrokAgentSidebarItem, target: GrokAgentSidebarItem, position: 'before' | 'after'): void;
  onBroadcast(): void;
  onOpenNetwork(): void;
  onOpenPlugins(): void;
  onOpenSettings(): void;
  onToggleSelection?(item: GrokAgentSidebarItem): void;
  onRangeSelection?(item: GrokAgentSidebarItem): void;
  onClearSelection?(): void;
  onDeleteSelected?(items: readonly GrokAgentSidebarItem[]): void;
  onMoveSelectedToSection?(items: readonly GrokAgentSidebarItem[], sectionId: string): void;
  onCreateSection?(items: readonly GrokAgentSidebarItem[]): void;
  onToggleSection?(section: AgentSidebarSection): void;
  onRenameSection?(section: AgentSidebarSection): void;
  onDeleteSection?(section: AgentSidebarSection): void;
  onMoveToSection?(item: GrokAgentSidebarItem, sectionId: string): void;
};

function activityState(item: GrokAgentSidebarItem, hostReady: boolean): FabAvatarState {
  if (item.busy) return 'working';
  if (item.waitingReason) return 'waiting';
  if (item.unread > 0) return 'waiting';
  return hostReady ? 'idle' : 'offline';
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
  selected,
  selectionEnabled,
  collapsed,
  hostReady,
  onOpen,
  onToggleSelection,
  onRangeSelection,
  onTogglePin,
  onRename,
  onHide,
  onDuplicate,
  onDelete,
  onReorderPinned,
}: {
  item: GrokAgentSidebarItem;
  active: boolean;
  selected: boolean;
  selectionEnabled: boolean;
  collapsed: boolean;
  hostReady: boolean;
  onOpen(): void;
  onToggleSelection?(): void;
  onRangeSelection?(): void;
  onTogglePin(): void;
  onRename(): void;
  onHide(): void;
  onDuplicate(): void;
  onDelete(): void;
  onReorderPinned(movedKey: string, targetKey: string, position: 'before' | 'after'): void;
}) {
  const [menuOpen, setMenuOpen] = useState(false);
  return <div
    className={styles.rowWrap}
    data-active={active || undefined}
    data-pinned={item.pinned || undefined}
    data-selected={selected || undefined}
  >
    <button
      type="button"
      className={styles.row}
      aria-current={active ? 'page' : undefined}
      aria-pressed={selectionEnabled ? selected : undefined}
      aria-label={item.name}
      data-testid={`peer-${item.peerKey}`}
      data-agent-key={item.key}
      data-agent-id={item.agentId}
      onClick={(event) => {
        if ((event.metaKey || event.ctrlKey) && onToggleSelection) {
          event.preventDefault();
          onToggleSelection();
          return;
        }
        if (event.shiftKey && onRangeSelection) {
          event.preventDefault();
          onRangeSelection();
          return;
        }
        if (selectionEnabled && onToggleSelection) {
          onToggleSelection();
          return;
        }
        onOpen();
      }}
      draggable
      onDragStart={(event) => {
        event.dataTransfer.effectAllowed = 'move';
        event.dataTransfer.setData('text/x-grok-agent-key', item.key);
      }}
      onDragOver={(event) => {
        if (!item.pinned) return;
        event.preventDefault();
        event.dataTransfer.dropEffect = 'move';
      }}
      onDrop={(event) => {
        if (!item.pinned) return;
        const movedKey = event.dataTransfer.getData('text/x-grok-agent-key');
        if (!movedKey || movedKey === item.key) return;
        event.preventDefault();
        const bounds = event.currentTarget.getBoundingClientRect();
        const position = event.clientY < bounds.top + bounds.height / 2 ? 'before' : 'after';
        onReorderPinned(movedKey, item.key, position);
      }}
      onDoubleClick={(event) => {
        if (item.isGroup) return;
        event.preventDefault();
        event.stopPropagation();
        onRename();
      }}
    >
      <span className={styles.avatar}>
        <FabAvatar
          identity={`agent:${item.agentId}`}
          state={activityState(item, hostReady)}
          size={collapsed ? 36 : 38}
          label={item.name}
          active={active && item.busy}
        />
        {item.busy ? <i className={styles.workingDot} title="Working" /> : item.unread ? <i className={styles.unreadDot} title="Unread activity" /> : null}
      </span>
      {collapsed ? null : <span className={styles.copy}>
        <span className={styles.nameLine}>
          <strong>{item.name}</strong>
          <time>{relativeTime(item.updatedAtMs)}</time>
        </span>
        <small data-state={item.waitingReason ? 'waiting' : item.busy ? 'working' : item.unread ? 'unread' : undefined}>{
          item.draftPrompt?.trim()
            ? `Draft: ${item.draftPrompt.trim()}`
            : item.waitingReason?.trim()
              ? `Waiting for you: ${item.waitingReason.trim()}`
              : item.busy
                ? item.currentActivity?.trim()
                  ? `Working… · ${item.currentActivity.trim()}`
                  : item.lastMessage?.trim() ? `Working… · ${item.lastMessage.trim()}` : 'Working…'
                : item.isComposingMessage
                  ? 'Composing…'
                  : item.lastMessage?.trim() || item.description || (item.isGroup ? 'Agent group' : 'Agent')
        }</small>
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

function SidebarSectionHeader({
  name,
  count,
  collapsed,
  synthetic,
  onToggle,
  onRename,
  onDelete,
  onCreate,
  onDropAgent,
}: {
  name: string;
  count: number;
  collapsed: boolean;
  synthetic: boolean;
  onToggle?(): void;
  onRename?(): void;
  onDelete?(): void;
  onCreate?(): void;
  onDropAgent?(key: string): void;
}) {
  return <div
    className={styles.sectionHeader}
    data-synthetic={synthetic || undefined}
    onDragOver={(event) => {
      if (!onDropAgent) return;
      if (!event.dataTransfer.types.includes('text/x-grok-agent-key')) return;
      event.preventDefault();
      event.dataTransfer.dropEffect = 'move';
    }}
    onDrop={(event) => {
      if (!onDropAgent) return;
      const key = event.dataTransfer.getData('text/x-grok-agent-key');
      if (!key) return;
      event.preventDefault();
      onDropAgent(key);
    }}
  >
    <button type="button" className={styles.sectionToggle} onClick={onToggle} disabled={!onToggle} aria-expanded={!collapsed}>
      {collapsed ? <ChevronRight size={13} /> : <ChevronDown size={13} />}
      <span>{name}</span><small>{count}</small>
    </button>
    <span className={styles.sectionActions}>
      {onCreate ? <button type="button" title="New section" aria-label="New section" onClick={onCreate}><FolderPlus size={13} /></button> : null}
      {!synthetic && onRename ? <button type="button" title="Rename section" aria-label={`Rename ${name}`} onClick={onRename}><Pencil size={12} /></button> : null}
      {!synthetic && onDelete ? <button type="button" title="Delete section" aria-label={`Delete ${name}`} onClick={onDelete}><Trash2 size={12} /></button> : null}
    </span>
  </div>;
}

export default function GrokAgentSidebar(props: GrokAgentSidebarProps) {
  const normalized = props.query.trim().toLocaleLowerCase();
  const visible = useMemo(
    () => props.agents.filter((item) => !item.hidden && (!normalized || `${item.name} ${item.description}`.toLocaleLowerCase().includes(normalized))),
    [props.agents, normalized],
  );
  const pinned = visible.filter((item) => item.pinned);
  const sections = useMemo(
    () => projectAgentSidebarSections(visible, props.sections ?? []),
    [visible, props.sections],
  );
  const hiddenCount = props.agents.filter((item) => item.hidden).length;
  const selectedSet = useMemo(() => new Set(props.selectedKeys ?? []), [props.selectedKeys]);
  const selectedItems = visible.filter((item) => selectedSet.has(item.key));
  const selectionEnabled = selectedItems.length > 0;
  const movableSelected = selectedItems;

  const renderRow = (item: GrokAgentSidebarItem) => <AgentRow
    key={item.key}
    item={item}
    active={item.key === props.activeKey}
    selected={selectedSet.has(item.key)}
    selectionEnabled={selectionEnabled}
    collapsed={props.collapsed}
    hostReady={props.hostReady}
    onOpen={() => props.onOpen(item)}
    onToggleSelection={props.onToggleSelection ? () => props.onToggleSelection!(item) : undefined}
    onRangeSelection={props.onRangeSelection ? () => props.onRangeSelection!(item) : undefined}
    onTogglePin={() => props.onTogglePin(item)}
    onRename={() => props.onRename(item)}
    onHide={() => props.onHide(item)}
    onDuplicate={() => props.onDuplicate(item)}
    onDelete={() => props.onDelete(item)}
    onReorderPinned={(movedKey, targetKey, position) => {
      const moved = props.agents.find((candidate) => candidate.key === movedKey);
      const target = props.agents.find((candidate) => candidate.key === targetKey);
      if (moved && target) props.onReorderPinned(moved, target, position);
    }}
  />;

  return <div className={styles.root} data-collapsed={props.collapsed || undefined}>
    <header className={styles.header}>
      <div className={styles.headerActions}>
        {!props.collapsed ? <button type="button" className={styles.headerIcon} onClick={props.onBroadcast} title="Broadcast to agents" aria-label="Broadcast to agents"><Megaphone size={16} /></button> : null}
        {!props.collapsed ? <button type="button" className={styles.headerIcon} onClick={props.onOpenNetwork} title="Agent network" aria-label="Agent network"><Network size={16} /></button> : null}
        <button type="button" className={styles.newButton} onClick={props.onNewAgent} title="New chat" data-testid="grok-new-agent">
          <Plus size={18} /><span>{props.collapsed ? null : 'New'}</span>
        </button>
      </div>
      <button type="button" className={styles.collapseButton} onClick={props.onToggleCollapsed} title={props.collapsed ? 'Expand sidebar' : 'Collapse sidebar'}>
        {props.collapsed ? <ChevronsRight size={16} /> : <ChevronsLeft size={16} />}
      </button>
    </header>

    {props.collapsed ? null : <label className={styles.search}>
      <Search size={15} />
      <input value={props.query} onChange={(event) => props.onQuery(event.target.value)} placeholder="Search" aria-label="Search agents" />
    </label>}

    {!props.collapsed && selectionEnabled ? <div className={styles.selectionBar} data-testid="agent-selection-bar">
      <strong>{selectedItems.length} selected</strong>
      <button type="button" onClick={() => props.onCreateSection?.(movableSelected)} disabled={!props.onCreateSection}><FolderPlus size={13} />Section</button>
      <select
        aria-label="Move selected agents"
        defaultValue=""
        disabled={!movableSelected.length || !props.onMoveSelectedToSection}
        onChange={(event) => {
          if (!event.target.value) return;
          props.onMoveSelectedToSection?.(movableSelected, event.target.value);
          event.target.value = '';
        }}
      >
        <option value="">Move…</option>
        {(props.sections ?? []).map((section) => <option key={section.id} value={section.id}>{section.name}</option>)}
        <option value={AGENT_SIDEBAR_UNASSIGNED_ID}>Unassigned</option>
      </select>
      <button type="button" data-danger="true" aria-label="Delete selected agents" onClick={() => props.onDeleteSelected?.(selectedItems)} disabled={!props.onDeleteSelected}><Trash2 size={13} /></button>
      <button type="button" aria-label="Clear selection" onClick={props.onClearSelection}><X size={13} /></button>
    </div> : null}

    <div className={styles.list}>
      {pinned.length && !props.collapsed ? <div className={styles.sectionLabel}>Pinned</div> : null}
      {pinned.map(renderRow)}
      {props.collapsed ? sections.flatMap((section) => section.agents).map(renderRow) : sections.map((section) => {
        const source = (props.sections ?? []).find((candidate) => candidate.id === section.id);
        return <section className={styles.section} key={section.id} data-section-id={section.id}>
          <SidebarSectionHeader
            name={section.name}
            count={section.agents.length}
            collapsed={section.isCollapsed}
            synthetic={section.isSynthetic}
            onToggle={source && props.onToggleSection ? () => props.onToggleSection!(source) : undefined}
            onRename={source && props.onRenameSection ? () => props.onRenameSection!(source) : undefined}
            onDelete={source && props.onDeleteSection ? () => props.onDeleteSection!(source) : undefined}
            onCreate={section.isSynthetic && props.onCreateSection ? () => props.onCreateSection!([]) : undefined}
            onDropAgent={props.onMoveToSection ? (key) => {
              const item = props.agents.find((candidate) => candidate.key === key);
              if (item) props.onMoveToSection!(item, section.id);
            } : undefined}
          />
          {section.isCollapsed ? null : section.agents.map(renderRow)}
        </section>;
      })}
      {!visible.length && !props.collapsed ? <div className={styles.empty}><Bot size={24} /><strong>No agents yet</strong><small>Create a new chat to start an Agent.</small></div> : null}
    </div>

    <footer className={styles.footer}>
      {hiddenCount > 0 && !props.collapsed ? <div className={styles.hiddenHint}><EyeOff size={14} /><span>{hiddenCount} hidden bots</span></div> : null}
      <button type="button" onClick={props.onOpenPlugins} title="Plugins"><Plug size={17} />{props.collapsed ? null : <span>Plugins</span>}</button>
      <button type="button" onClick={props.onOpenSettings} title="Settings"><Settings size={17} />{props.collapsed ? null : <span>{props.accountLabel}</span>}</button>
    </footer>
  </div>;
}
