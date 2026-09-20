import React, { useState } from 'react';
import GrokAgentSidebar, {
  type GrokAgentSidebarItem,
  type GrokAgentSidebarProps,
} from '../grok-shell/grok-agent-sidebar';

export type AgentSidebarItem = GrokAgentSidebarItem;

export interface AgentSidebarProps extends Omit<GrokAgentSidebarProps, 'onCreateSection'> {
  onCreateSection?(name: string, items: readonly GrokAgentSidebarItem[]): void;
}

/**
 * Stable Agent-workspace boundary for the primary desktop navigation.
 *
 * Section naming lives here rather than in the Messenger compatibility shell.
 * Electron does not give the primary Agent workspace a reliable browser prompt
 * lifecycle, so the Agent surface owns an application dialog and submits the
 * durable section mutation through useAgentSidebarController.
 */
export default function AgentSidebar({ onCreateSection, ...props }: AgentSidebarProps) {
  const [sectionDraft, setSectionDraft] = useState<{
    items: readonly GrokAgentSidebarItem[];
    name: string;
  } | null>(null);

  function closeSectionDialog(): void {
    setSectionDraft(null);
  }

  function submitSectionDialog(event: React.FormEvent<HTMLFormElement>): void {
    event.preventDefault();
    if (!sectionDraft) return;
    const name = sectionDraft.name.trim();
    if (!name) return;
    onCreateSection?.(name, sectionDraft.items);
    closeSectionDialog();
  }

  return <>
    <GrokAgentSidebar
      {...props}
      onCreateSection={onCreateSection
        ? (items) => setSectionDraft({ items: [...items], name: 'New section' })
        : undefined}
    />
    {sectionDraft ? <div
      role="dialog"
      aria-modal="true"
      aria-label="Create section"
      data-testid="agent-section-dialog"
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 1200,
        display: 'grid',
        placeItems: 'center',
        background: 'rgba(0, 0, 0, 0.54)',
      }}
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) closeSectionDialog();
      }}
    >
      <form
        onSubmit={submitSectionDialog}
        style={{
          width: 'min(360px, calc(100vw - 40px))',
          display: 'grid',
          gap: 12,
          padding: 18,
          borderRadius: 14,
          border: '1px solid rgba(255,255,255,.10)',
          background: '#1b1b20',
          boxShadow: '0 18px 60px rgba(0,0,0,.45)',
        }}
      >
        <strong>Create section</strong>
        <input
          autoFocus
          aria-label="Section name"
          value={sectionDraft.name}
          onChange={(event) => setSectionDraft((current) => current
            ? { ...current, name: event.target.value }
            : current)}
          onKeyDown={(event) => {
            if (event.key === 'Escape') {
              event.preventDefault();
              closeSectionDialog();
            }
          }}
          style={{
            width: '100%',
            boxSizing: 'border-box',
            borderRadius: 10,
            border: '1px solid rgba(255,255,255,.12)',
            background: 'rgba(255,255,255,.05)',
            color: 'inherit',
            padding: '9px 11px',
          }}
        />
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8 }}>
          <button type="button" onClick={closeSectionDialog}>Cancel</button>
          <button type="submit" disabled={!sectionDraft.name.trim()}>Create</button>
        </div>
      </form>
    </div> : null}
  </>;
}
