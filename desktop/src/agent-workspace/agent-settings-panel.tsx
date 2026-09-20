import React, { useEffect, useState, type KeyboardEvent } from 'react';
import type { AgentSettingsPending, AgentSettingsProfileUpdate, AgentSettingsProfileValue } from './agent-settings-controller';
export type { AgentSettingsProfileUpdate, AgentSettingsProfileValue } from './agent-settings-controller';
import styles from './agent-settings-panel.module.css';

export interface AgentSettingsPanelProps {
  readonly agentId: string;
  readonly value: AgentSettingsProfileValue;
  readonly pending: AgentSettingsPending;
  readonly error: string | null;
  readonly onUpdateProfile: (profile: AgentSettingsProfileUpdate) => Promise<unknown>;
  readonly onSetNotifications: (enabled: boolean) => Promise<unknown>;
}

function EditableField({
  label,
  value,
  placeholder,
  multiline = false,
  required = false,
  disabled = false,
  onCommit,
}: {
  label: string;
  value: string;
  placeholder: string;
  multiline?: boolean;
  required?: boolean;
  disabled?: boolean;
  onCommit(value: string): void;
}) {
  const [draft, setDraft] = useState(value);
  const [focused, setFocused] = useState(false);
  useEffect(() => {
    if (!focused) setDraft(value);
  }, [focused, value]);

  const commit = () => {
    setFocused(false);
    const normalized = draft.trim();
    if ((required && !normalized) || normalized === value) {
      setDraft(value);
      return;
    }
    onCommit(normalized);
  };
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      setDraft(value);
      setFocused(false);
      event.currentTarget.blur();
    } else if (!multiline && event.key === 'Enter') {
      event.preventDefault();
      event.currentTarget.blur();
    }
  };
  const common = {
    'aria-label': label,
    value: draft,
    placeholder,
    disabled,
    spellCheck: false,
    onFocus: () => setFocused(true),
    onBlur: commit,
    onKeyDown,
    onChange: (event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => setDraft(event.currentTarget.value),
  };
  return multiline ? <textarea {...common} rows={4} /> : <input {...common} type="text" />;
}

/**
 * Agent-owned profile/settings surface matching Fabu's settings boundary.
 *
 * The panel edits Agent presentation through the Agent runtime and leaves
 * account/CAS mirroring to the authoritative bot.changed projection.
 */
export default function AgentSettingsPanel(props: AgentSettingsPanelProps) {
  const updateProfile = (field: 'name' | 'title' | 'description', value: string) => {
    if (props.pending) return;
    const next: AgentSettingsProfileUpdate = {
      name: props.value.name,
      ...(props.value.title === undefined ? {} : { title: props.value.title }),
      description: props.value.description,
      [field]: value,
    };
    void props.onUpdateProfile(next);
  };

  const toggleNotifications = () => {
    if (props.pending) return;
    void props.onSetNotifications(!props.value.notifyOnUpdatesEnabled);
  };

  return <section className={styles.root} aria-label="Agent settings" data-agent-id={props.agentId} data-pending={props.pending ?? undefined}>
    <header><strong>Agent settings</strong><small>Agent-owned profile and notifications</small></header>
    <label><span>Name</span><EditableField label="Agent name" value={props.value.name} required disabled={props.pending != null} placeholder="Agent name" onCommit={(value) => void updateProfile('name', value)} /></label>
    {props.value.title === undefined ? null : <label><span>Title</span><EditableField label="Agent title" value={props.value.title} disabled={props.pending != null} placeholder="Describe what your Agent does" onCommit={(value) => void updateProfile('title', value)} /></label>}
    <label><span>Description</span><EditableField label="Agent description" value={props.value.description} multiline disabled={props.pending != null} placeholder="What this Agent is for" onCommit={(value) => void updateProfile('description', value)} /></label>
    <div className={styles.row}>
      <span><strong>Notifications</strong><small>Get notified when this Agent finishes or needs input</small></span>
      <button type="button" role="switch" aria-checked={props.value.notifyOnUpdatesEnabled} disabled={props.pending != null} onClick={toggleNotifications}>
        {props.value.notifyOnUpdatesEnabled ? 'On' : 'Off'}
      </button>
    </div>
    {props.error ? <div className={styles.error} role="status" aria-live="polite">{props.error}</div> : null}
  </section>;
}
