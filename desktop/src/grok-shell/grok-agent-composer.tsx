import { Paperclip, Send, Square, X } from 'lucide-react';
import React, { useRef, useState, type DragEvent, type FormEvent, type KeyboardEvent } from 'react';
import styles from './grok-agent-composer.module.css';

export interface GrokComposerAttachment {
  id: string;
  name: string;
  sizeBytes?: number;
}

function attachmentSize(sizeBytes?: number): string {
  if (sizeBytes == null || !Number.isFinite(sizeBytes)) return '';
  if (sizeBytes < 1024) return `${sizeBytes} B`;
  if (sizeBytes < 1024 * 1024) return `${Math.round(sizeBytes / 1024)} KB`;
  return `${(sizeBytes / 1024 / 1024).toFixed(1)} MB`;
}

export default function GrokAgentComposer({
  value,
  agentName,
  ready,
  busy,
  uploading = false,
  enterToSend,
  attachments = [],
  onChange,
  onSubmit,
  onAttachFiles,
  onRemoveAttachment,
  onStop,
}: {
  value: string;
  agentName: string;
  ready: boolean;
  busy: boolean;
  uploading?: boolean;
  enterToSend: boolean;
  attachments?: readonly GrokComposerAttachment[];
  onChange(value: string): void;
  onSubmit(event: FormEvent<HTMLFormElement>): void;
  onAttachFiles(files: readonly File[]): void;
  onRemoveAttachment(id: string): void;
  onStop(): void;
}) {
  const hasText = value.trim().length > 0;
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const dragDepthRef = useRef(0);
  const [dragOver, setDragOver] = useState(false);

  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    const submitWithEnter = enterToSend && event.key === 'Enter' && !event.shiftKey;
    const submitWithShortcut = !enterToSend
      && event.key === 'Enter'
      && (event.metaKey || event.ctrlKey);
    if (!submitWithEnter && !submitWithShortcut) return;
    event.preventDefault();
    event.currentTarget.form?.requestSubmit();
  };

  const hasDraggedFiles = (event: DragEvent<HTMLElement>) =>
    Array.from(event.dataTransfer.types).includes('Files');

  const handleDragEnter = (event: DragEvent<HTMLFormElement>) => {
    if (!hasDraggedFiles(event)) return;
    event.preventDefault();
    dragDepthRef.current += 1;
    setDragOver(true);
  };

  const handleDragLeave = (event: DragEvent<HTMLFormElement>) => {
    if (!hasDraggedFiles(event)) return;
    dragDepthRef.current = Math.max(0, dragDepthRef.current - 1);
    if (dragDepthRef.current === 0) setDragOver(false);
  };

  const handleDrop = (event: DragEvent<HTMLFormElement>) => {
    if (!hasDraggedFiles(event)) return;
    event.preventDefault();
    dragDepthRef.current = 0;
    setDragOver(false);
    const files = Array.from(event.dataTransfer.files ?? []);
    if (files.length) onAttachFiles(files);
  };

  return <form
    className={styles.root}
    data-testid="grok-agent-composer"
    data-drag-over={dragOver || undefined}
    onSubmit={onSubmit}
    onDragEnter={handleDragEnter}
    onDragLeave={handleDragLeave}
    onDragOver={(event) => {
      if (hasDraggedFiles(event)) event.preventDefault();
    }}
    onDrop={handleDrop}
  >
    {dragOver ? <div className={styles.dropOverlay} aria-hidden="true">Drop files to add to {agentName}</div> : null}
    {attachments.length ? <div className={styles.attachments} role="list" aria-label="Attachments">
      {attachments.map((attachment) => <span key={attachment.id} className={styles.attachment} role="listitem">
        <span><strong>{attachment.name}</strong>{attachmentSize(attachment.sizeBytes) ? <small>{attachmentSize(attachment.sizeBytes)}</small> : null}</span>
        <button type="button" aria-label={`Remove ${attachment.name}`} onClick={() => onRemoveAttachment(attachment.id)}><X size={13} /></button>
      </span>)}
    </div> : null}
    <button type="button" className={styles.attach} title="Attach files" aria-label="Attach files" disabled={!ready || uploading} onClick={() => fileInputRef.current?.click()}>
      <Paperclip size={18} />
    </button>
    <input
      ref={fileInputRef}
      className={styles.fileInput}
      type="file"
      multiple
      tabIndex={-1}
      aria-hidden="true"
      onChange={(event) => {
        const files = Array.from(event.currentTarget.files ?? []);
        event.currentTarget.value = '';
        if (files.length) onAttachFiles(files);
      }}
    />
    <textarea
      data-testid="messenger-input"
      value={value}
      rows={1}
      aria-label={`Message ${agentName}`}
      placeholder={attachments.length ? `Add a message for ${agentName}` : `Message ${agentName}`}
      onChange={(event) => onChange(event.target.value)}
      onKeyDown={handleKeyDown}
    />
    {hasText ? (
      <button data-testid="messenger-send" className={styles.primary} type="submit" disabled={!ready || uploading}>
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
