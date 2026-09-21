import React, { useState } from 'react';
import { FabButton, FabDialog, FabDialogActions, FabInput } from '../ui/primitives/fab-primitives';
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
    {sectionDraft ? <FabDialog
      label="Create section"
      onClose={closeSectionDialog}
      onSubmit={submitSectionDialog}
    >
      <strong>Create section</strong>
      <FabInput
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
      />
      <FabDialogActions>
        <FabButton type="button" variant="ghost" onClick={closeSectionDialog}>Cancel</FabButton>
        <FabButton type="submit" variant="primary" disabled={!sectionDraft.name.trim()}>Create</FabButton>
      </FabDialogActions>
    </FabDialog> : null}
  </>;
}
