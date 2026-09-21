import React, { useState } from 'react';
import { FabButton, FabDialog, FabDialogActions, FabInput } from '../ui/primitives/fab-primitives';
import GrokAgentSidebar, {
  type GrokAgentSidebarItem,
  type GrokAgentSidebarProps,
} from '../grok-shell/grok-agent-sidebar';
import type { AgentSidebarSection } from './agent-sidebar-state';

export type AgentSidebarItem = GrokAgentSidebarItem;

type AgentSidebarDialog =
  | { readonly kind: 'create-agent'; readonly name: string }
  | { readonly kind: 'rename-agent'; readonly item: GrokAgentSidebarItem; readonly name: string }
  | { readonly kind: 'delete-agents'; readonly items: readonly GrokAgentSidebarItem[] }
  | { readonly kind: 'create-section'; readonly items: readonly GrokAgentSidebarItem[]; readonly name: string }
  | { readonly kind: 'rename-section'; readonly section: AgentSidebarSection; readonly name: string }
  | { readonly kind: 'delete-section'; readonly section: AgentSidebarSection };

export interface AgentSidebarProps extends Omit<
  GrokAgentSidebarProps,
  | 'onNewAgent'
  | 'onRename'
  | 'onDelete'
  | 'onDeleteSelected'
  | 'onCreateSection'
  | 'onRenameSection'
  | 'onDeleteSection'
> {
  onCreateAgent?(name: string): void | Promise<void>;
  onRenameAgent?(item: GrokAgentSidebarItem, name: string): void | Promise<void>;
  onDeleteAgents?(items: readonly GrokAgentSidebarItem[]): void | Promise<void>;
  onCreateSection?(name: string, items: readonly GrokAgentSidebarItem[]): void;
  onRenameSection?(section: AgentSidebarSection, name: string): void;
  onDeleteSection?(section: AgentSidebarSection): void;
}

/**
 * Stable Agent-workspace boundary for primary desktop navigation mutations.
 *
 * Naming and destructive confirmation live in application-owned FabDialog
 * surfaces. The root shell receives only validated, confirmed domain intents;
 * browser prompt/confirm APIs are never part of Agent product state.
 */
export default function AgentSidebar({
  onCreateAgent,
  onRenameAgent,
  onDeleteAgents,
  onCreateSection,
  onRenameSection,
  onDeleteSection,
  ...props
}: AgentSidebarProps) {
  const [dialog, setDialog] = useState<AgentSidebarDialog | null>(null);

  function closeDialog(): void {
    setDialog(null);
  }

  function updateDialogName(name: string): void {
    setDialog((current) => {
      if (!current || !('name' in current)) return current;
      return { ...current, name };
    });
  }

  function submitDialog(event: React.FormEvent<HTMLFormElement>): void {
    event.preventDefault();
    if (!dialog) return;

    if (dialog.kind === 'create-agent') {
      const name = dialog.name.trim();
      if (!name) return;
      closeDialog();
      void onCreateAgent?.(name);
      return;
    }
    if (dialog.kind === 'rename-agent') {
      const name = dialog.name.trim();
      if (!name || name === dialog.item.name) return;
      closeDialog();
      void onRenameAgent?.(dialog.item, name);
      return;
    }
    if (dialog.kind === 'delete-agents') {
      closeDialog();
      void onDeleteAgents?.(dialog.items);
      return;
    }
    if (dialog.kind === 'create-section') {
      const name = dialog.name.trim();
      if (!name) return;
      closeDialog();
      onCreateSection?.(name, dialog.items);
      return;
    }
    if (dialog.kind === 'rename-section') {
      const name = dialog.name.trim();
      if (!name || name === dialog.section.name) return;
      closeDialog();
      onRenameSection?.(dialog.section, name);
      return;
    }

    closeDialog();
    onDeleteSection?.(dialog.section);
  }

  const textDialog = dialog && 'name' in dialog ? dialog : null;
  const dialogLabel = dialog?.kind === 'create-agent'
    ? 'Create Agent'
    : dialog?.kind === 'rename-agent'
      ? 'Rename Agent'
      : dialog?.kind === 'delete-agents'
        ? dialog.items.length === 1 ? 'Delete Agent' : 'Delete Agents'
        : dialog?.kind === 'create-section'
          ? 'Create section'
          : dialog?.kind === 'rename-section'
            ? 'Rename section'
            : dialog?.kind === 'delete-section'
              ? 'Delete section'
              : '';

  return <>
    <GrokAgentSidebar
      {...props}
      onNewAgent={() => setDialog({ kind: 'create-agent', name: 'New Agent' })}
      onRename={(item) => setDialog({ kind: 'rename-agent', item, name: item.name })}
      onDelete={(item) => setDialog({ kind: 'delete-agents', items: [item] })}
      onDeleteSelected={(items) => setDialog({ kind: 'delete-agents', items: [...items] })}
      onCreateSection={onCreateSection
        ? (items) => setDialog({ kind: 'create-section', items: [...items], name: 'New section' })
        : undefined}
      onRenameSection={onRenameSection
        ? (section) => setDialog({ kind: 'rename-section', section, name: section.name })
        : undefined}
      onDeleteSection={onDeleteSection
        ? (section) => setDialog({ kind: 'delete-section', section })
        : undefined}
    />
    {dialog ? <FabDialog
      label={dialogLabel}
      onClose={closeDialog}
      onSubmit={submitDialog}
    >
      <strong>{dialogLabel}</strong>
      {textDialog ? <FabInput
        autoFocus
        aria-label={
          dialog.kind === 'create-agent' || dialog.kind === 'rename-agent'
            ? 'Agent name'
            : 'Section name'
        }
        value={textDialog.name}
        onChange={(event) => updateDialogName(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === 'Escape') {
            event.preventDefault();
            closeDialog();
          }
        }}
      /> : dialog.kind === 'delete-agents'
        ? <p>{dialog.items.length === 1
          ? `Delete Agent “${dialog.items[0]?.name ?? ''}”?`
          : `Delete ${dialog.items.length} selected Agents?`}</p>
        : <p>{`Delete section “${dialog.section.name}”?`}</p>}
      <FabDialogActions>
        <FabButton type="button" variant="ghost" onClick={closeDialog}>Cancel</FabButton>
        <FabButton
          type="submit"
          variant={dialog.kind === 'delete-agents' || dialog.kind === 'delete-section' ? 'danger' : 'primary'}
          disabled={textDialog ? !textDialog.name.trim() : false}
        >{dialog.kind === 'delete-agents' || dialog.kind === 'delete-section'
          ? 'Delete'
          : dialog.kind === 'rename-agent' || dialog.kind === 'rename-section'
            ? 'Rename'
            : 'Create'}</FabButton>
      </FabDialogActions>
    </FabDialog> : null}
  </>;
}
