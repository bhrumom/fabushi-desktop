import { Paperclip, Send, Square } from 'lucide-react';
import React, { type FormEvent, type KeyboardEvent } from 'react';
import styles from './grok-agent-composer.module.css';

export default function GrokAgentComposer({
  value,
  agentName,
  ready,
  busy,
  enterToSend,
  onChange,
  onSubmit,
  onAttach,
  onStop,
}: {
  value: string;
  agentName: string;
  ready: boolean;
  busy: boolean;
  enterToSend: boolean;
  onChange(value: string): void;
  onSubmit(event: FormEvent<HTMLFormElement>): void;
  onAttach(): void;
  onStop(): void;
}) {
  const hasText = value.trim().length > 0;
  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    const submitWithEnter = enterToSend && event.key === 'Enter' && !event.shiftKey;
    const submitWithShortcut = !enterToSend
      && event.key === 'Enter'
      && (event.metaKey || event.ctrlKey);
    if (!submitWithEnter && !submitWithShortcut) return;
    event.preventDefault();
    event.currentTarget.form?.requestSubmit();
  };

  return <form className={styles.root} data-testid="grok-agent-composer" onSubmit={onSubmit}>
    <button type="button" className={styles.attach} title="Attach files" aria-label="Attach files" onClick={onAttach}>
      <Paperclip size={18} />
    </button>
    <textarea
      data-testid="messenger-input"
      value={value}
      rows={1}
      aria-label={`Message ${agentName}`}
      placeholder={`Message ${agentName}`}
      onChange={(event) => onChange(event.target.value)}
      onKeyDown={handleKeyDown}
    />
    {hasText ? (
      <button data-testid="messenger-send" className={styles.primary} type="submit" disabled={!ready}>
        <Send size={17} />
      </button>
    ) : busy ? (
      <button data-testid="messenger-stop" className={styles.primary} type="button" title="Stop" aria-label="Stop" onClick={onStop}>
        <Square size={15} fill="currentColor" />
      </button>
    ) : (
      <span className={styles.primaryPlaceholder} aria-hidden="true" />
    )}
  </form>;
}
