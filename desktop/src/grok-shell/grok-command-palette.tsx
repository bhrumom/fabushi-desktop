import { Bot, Command, FileText, Link2, Megaphone, MessageSquareText, Monitor, Network, Plug, Plus, Search, Settings } from 'lucide-react';
import React, { useEffect, useMemo, useState } from 'react';
import type { GrokAgentSidebarItem } from './grok-agent-sidebar';
import type { TranscriptEntry } from '../agent-workspace/transcript-model';
import styles from './grok-command-palette.module.css';

type CommandItem =
  | { kind: 'agent'; key: string; label: string; detail: string; agent: GrokAgentSidebarItem }
  | { kind: 'command' | 'resource'; key: string; label: string; detail: string; icon: React.ReactNode; run(): void };

export default function GrokCommandPalette({
  open,
  agents,
  query,
  onQuery,
  onClose,
  onOpenAgent,
  onNewAgent,
  onNetwork,
  onBroadcast,
  onPlugins,
  onSettings,
  entries = [],
  onOpenTranscriptEntry,
  onConversationSearch,
  onComputer,
}: {
  open: boolean;
  agents: readonly GrokAgentSidebarItem[];
  query: string;
  onQuery(value: string): void;
  onClose(): void;
  onOpenAgent(agent: GrokAgentSidebarItem): void;
  onNewAgent(): void;
  onNetwork(): void;
  onBroadcast(): void;
  onPlugins(): void;
  onSettings(): void;
  entries?: readonly TranscriptEntry[];
  onOpenTranscriptEntry?(entryId: string): void;
  onConversationSearch?(): void;
  onComputer?(): void;
}) {
  const [selected, setSelected] = useState(0);
  const items = useMemo<CommandItem[]>(() => {
    const commands: CommandItem[] = [
      { kind: 'command', key: 'new', label: 'New chat', detail: 'Create a new Agent', icon: <Plus size={16} />, run: onNewAgent },
      { kind: 'command', key: 'find', label: 'Find in conversation', detail: 'Search the current Agent transcript', icon: <Search size={16} />, run: () => onConversationSearch?.() },
      { kind: 'command', key: 'computer', label: 'Computer', detail: 'Open the current Agent computer', icon: <Monitor size={16} />, run: () => onComputer?.() },
      { kind: 'command', key: 'network', label: 'Agent Network', detail: 'View Agents and groups', icon: <Network size={16} />, run: onNetwork },
      { kind: 'command', key: 'broadcast', label: 'Broadcast to agents', detail: 'Send one owner message to multiple Agents', icon: <Megaphone size={16} />, run: onBroadcast },
      { kind: 'command', key: 'plugins', label: 'Plugins', detail: 'Open installed apps and plugins', icon: <Plug size={16} />, run: onPlugins },
      { kind: 'command', key: 'settings', label: 'Settings', detail: 'Open Fabushi settings', icon: <Settings size={16} />, run: onSettings },
    ];
    const agentItems: CommandItem[] = agents
      .filter((agent) => !agent.hidden)
      .map((agent) => ({
        kind: 'agent',
        key: `agent:${agent.key}`,
        label: agent.name,
        detail: agent.busy ? 'Working…' : agent.description || 'Agent',
        agent,
      }));

    const resourceItems: CommandItem[] = [];
    for (const entry of entries.slice(-240)) {
      const label = entry.text.trim().replace(/\s+/g, ' ');
      if (label && (entry.kind === 'message' || entry.kind === 'assistant-turn')) {
        resourceItems.push({
          kind: 'resource',
          key: `message:${entry.id}`,
          label: label.slice(0, 90),
          detail: entry.role === 'me' ? 'Your message' : 'Agent message',
          icon: <MessageSquareText size={16} />,
          run: () => onOpenTranscriptEntry?.(entry.id),
        });
      }
      for (const attachment of entry.attachments ?? []) {
        resourceItems.push({
          kind: 'resource',
          key: `file:${entry.id}:${attachment.id}`,
          label: attachment.name,
          detail: 'Attachment',
          icon: <FileText size={16} />,
          run: () => onOpenTranscriptEntry?.(entry.id),
        });
      }
      for (const match of entry.text.matchAll(/https?:\/\/[^\s)\]}>"']+/g)) {
        const url = match[0];
        resourceItems.push({
          kind: 'resource',
          key: `link:${entry.id}:${url}`,
          label: url.length > 90 ? `${url.slice(0, 87)}…` : url,
          detail: 'Link in conversation',
          icon: <Link2 size={16} />,
          run: () => onOpenTranscriptEntry?.(entry.id),
        });
      }
    }

    const normalized = query.trim().toLocaleLowerCase();
    const all = normalized ? [...commands, ...agentItems, ...resourceItems] : [...commands, ...agentItems];
    return normalized
      ? all.filter((item) => `${item.label} ${item.detail}`.toLocaleLowerCase().includes(normalized))
      : all;
  }, [agents, entries, onBroadcast, onComputer, onConversationSearch, onNetwork, onNewAgent, onOpenTranscriptEntry, onPlugins, onSettings, query]);

  useEffect(() => {
    if (open) setSelected(0);
  }, [open, query]);

  if (!open) return null;

  const run = (item: CommandItem | undefined) => {
    if (!item) return;
    onClose();
    if (item.kind === 'agent') onOpenAgent(item.agent);
    else item.run();
  };

  return <div className={styles.backdrop} role="presentation" onMouseDown={onClose}>
    <section className={styles.palette} role="dialog" aria-modal="true" aria-label="Command palette" onMouseDown={(event) => event.stopPropagation()}>
      <label className={styles.inputRow}>
        <Command size={18} />
        <input
          autoFocus
          value={query}
          onChange={(event) => onQuery(event.target.value)}
          placeholder="Search agents or run a command"
          onKeyDown={(event) => {
            if (event.key === 'Escape') {
              event.preventDefault();
              onClose();
            } else if (event.key === 'ArrowDown') {
              event.preventDefault();
              setSelected((value) => items.length ? (value + 1) % items.length : 0);
            } else if (event.key === 'ArrowUp') {
              event.preventDefault();
              setSelected((value) => items.length ? (value - 1 + items.length) % items.length : 0);
            } else if (event.key === 'Enter') {
              event.preventDefault();
              run(items[selected]);
            }
          }}
        />
        <kbd>⌘K</kbd>
      </label>
      <div className={styles.list} role="listbox" aria-label="Commands">
        {items.map((item, index) => <button
          type="button"
          role="option"
          aria-selected={selected === index}
          key={item.key}
          onMouseEnter={() => setSelected(index)}
          onClick={() => run(item)}
        >
          <span className={styles.icon}>{item.kind === 'agent' ? <Bot size={16} /> : item.icon}</span>
          <span className={styles.copy}><strong>{item.label}</strong><small>{item.detail}</small></span>
          {item.kind === 'agent' && item.agent.busy ? <i className={styles.working} title="Working" /> : null}
        </button>)}
        {!items.length ? <div className={styles.empty}>No matching agents or commands.</div> : null}
      </div>
    </section>
  </div>;
}
