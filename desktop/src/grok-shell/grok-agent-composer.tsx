import { LoaderCircle, Mic, Paperclip, Send, Square, X } from 'lucide-react';
import React, { useEffect, useRef, useState, type ClipboardEvent, type DragEvent, type FormEvent, type KeyboardEvent } from 'react';
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
  onTranscribeVoice,
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
  onTranscribeVoice?(file: File): Promise<string>;
  onStop(): void;
}) {
  const hasText = value.trim().length > 0;
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const dragDepthRef = useRef(0);
  const [dragOver, setDragOver] = useState(false);
  const [voiceState, setVoiceState] = useState<'idle' | 'recording' | 'transcribing'>('idle');
  const [voiceError, setVoiceError] = useState<string | null>(null);
  const voiceRecorderRef = useRef<MediaRecorder | null>(null);
  const voiceStreamRef = useRef<MediaStream | null>(null);
  const voiceChunksRef = useRef<Blob[]>([]);
  const valueRef = useRef(value);
  valueRef.current = value;

  const stopVoiceTracks = () => {
    for (const track of voiceStreamRef.current?.getTracks() ?? []) track.stop();
    voiceStreamRef.current = null;
  };

  const stopVoice = () => {
    const recorder = voiceRecorderRef.current;
    if (recorder && recorder.state !== 'inactive') recorder.stop();
  };

  const startVoice = async () => {
    if (!onTranscribeVoice || voiceState !== 'idle') return;
    setVoiceError(null);
    if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === 'undefined') {
      setVoiceError('Voice recording is unavailable on this device.');
      return;
    }
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
      voiceStreamRef.current = stream;
      const preferred = 'audio/webm;codecs=opus';
      const recorder = MediaRecorder.isTypeSupported(preferred)
        ? new MediaRecorder(stream, { mimeType: preferred })
        : new MediaRecorder(stream);
      voiceRecorderRef.current = recorder;
      voiceChunksRef.current = [];
      recorder.ondataavailable = (event) => {
        if (event.data.size > 0) voiceChunksRef.current.push(event.data);
      };
      recorder.onerror = () => {
        stopVoiceTracks();
        voiceRecorderRef.current = null;
        setVoiceState('idle');
        setVoiceError('Voice recording failed.');
      };
      recorder.onstop = () => {
        const chunks = [...voiceChunksRef.current];
        voiceChunksRef.current = [];
        const mimeType = recorder.mimeType || 'audio/webm';
        voiceRecorderRef.current = null;
        stopVoiceTracks();
        if (!chunks.length) {
          setVoiceState('idle');
          return;
        }
        const extension = mimeType.includes('ogg') ? 'ogg' : mimeType.includes('mp4') ? 'm4a' : 'webm';
        const file = new File(chunks, `voice-${Date.now()}.${extension}`, { type: mimeType });
        setVoiceState('transcribing');
        void onTranscribeVoice(file).then((text) => {
          const transcript = text.trim();
          if (transcript) {
            const current = valueRef.current.trimEnd();
            onChange(current ? `${current}\n${transcript}` : transcript);
          }
          setVoiceState('idle');
        }).catch((cause: unknown) => {
          setVoiceState('idle');
          setVoiceError(cause instanceof Error ? cause.message : String(cause));
        });
      };
      recorder.start(250);
      setVoiceState('recording');
    } catch (cause) {
      stopVoiceTracks();
      voiceRecorderRef.current = null;
      setVoiceState('idle');
      setVoiceError(cause instanceof Error ? cause.message : 'Microphone access was denied.');
    }
  };

  useEffect(() => () => {
    const recorder = voiceRecorderRef.current;
    if (recorder && recorder.state !== 'inactive') recorder.stop();
    stopVoiceTracks();
  }, []);

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
    <button type="button" className={styles.attach} title="Attach files" aria-label="Attach files" disabled={!ready || uploading || voiceState !== 'idle'} onClick={() => fileInputRef.current?.click()}>
      <Paperclip size={18} />
    </button>
    {onTranscribeVoice ? <button
      type="button"
      className={styles.voice}
      data-state={voiceState}
      title={voiceState === 'recording' ? 'Stop dictation' : voiceState === 'transcribing' ? 'Transcribing' : 'Dictate'}
      aria-label={voiceState === 'recording' ? 'Stop dictation' : voiceState === 'transcribing' ? 'Transcribing voice' : 'Start dictation'}
      disabled={!ready || uploading || voiceState === 'transcribing'}
      onClick={() => voiceState === 'recording' ? stopVoice() : void startVoice()}
    >
      {voiceState === 'transcribing' ? <LoaderCircle size={17} className={styles.spin} /> : voiceState === 'recording' ? <Square size={13} fill="currentColor" /> : <Mic size={17} />}
    </button> : null}
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
      onPaste={(event: ClipboardEvent<HTMLTextAreaElement>) => {
        const files = Array.from(event.clipboardData.items)
          .filter((item) => item.kind === 'file')
          .map((item) => item.getAsFile())
          .filter((file): file is File => file != null);
        if (!files.length) return;
        event.preventDefault();
        onAttachFiles(files);
      }}
    />
    {voiceState === 'recording' ? <span className={styles.voiceStatus}>Recording…</span> : voiceError ? <span className={styles.voiceError} title={voiceError}>Voice unavailable</span> : null}
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
