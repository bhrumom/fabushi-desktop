import { LoaderCircle, Mic, Paperclip, Reply, Send, Square, X } from 'lucide-react';
import React, { useEffect, useMemo, useRef, useState, type DragEvent, type FormEvent } from 'react';
import AgentRichTextEditor, { type AgentRichTextEditorControls } from './agent-rich-text-editor';
import {
  emojiSuggestions,
  filterPullRequestSuggestions,
  type AgentPullRequestSuggestion,
} from './agent-composer-suggestion-provider';
import { AGENT_ATTACHMENT_LIMIT } from './agent-attachments';
import styles from './agent-composer.module.css';

export interface AgentComposerAttachment {
  id: string;
  name: string;
  sizeBytes?: number;
}

export interface AgentComposerReplyTarget {
  id: string;
  label: string;
  text: string;
}

export interface AgentComposerMentionCandidate {
  id: string;
  name: string;
  description?: string;
  kind?: 'agent' | 'mcp';
}

export interface AgentComposerWorkflowCandidate {
  id: string;
  name: string;
  description?: string;
}

function attachmentSize(sizeBytes?: number): string {
  if (sizeBytes == null || !Number.isFinite(sizeBytes)) return '';
  if (sizeBytes < 1024) return `${sizeBytes} B`;
  if (sizeBytes < 1024 * 1024) return `${Math.round(sizeBytes / 1024)} KB`;
  return `${(sizeBytes / 1024 / 1024).toFixed(1)} MB`;
}

export default function AgentComposer({
  value,
  richText,
  scopeKey,
  agentName,
  ready,
  busy,
  uploading = false,
  enterToSend,
  attachments = [],
  replyTarget,
  mentionCandidates = [],
  workflowCandidates = [],
  pullRequestCandidates = [],
  onClearReplyTarget,
  onMention,
  onWorkflowReference,
  onChange,
  onSubmit,
  onAttachFiles,
  onRemoveAttachment,
  onTranscribeVoice,
  onStop,
}: {
  value: string;
  richText?: string;
  scopeKey: string;
  agentName: string;
  ready: boolean;
  busy: boolean;
  uploading?: boolean;
  enterToSend: boolean;
  attachments?: readonly AgentComposerAttachment[];
  replyTarget?: AgentComposerReplyTarget;
  mentionCandidates?: readonly AgentComposerMentionCandidate[];
  workflowCandidates?: readonly AgentComposerWorkflowCandidate[];
  pullRequestCandidates?: readonly AgentPullRequestSuggestion[];
  onClearReplyTarget?(): void;
  onMention?(candidate: AgentComposerMentionCandidate): void;
  onWorkflowReference?(candidate: AgentComposerWorkflowCandidate): void;
  onChange(value: string, richText?: string): void;
  onSubmit(event: FormEvent<HTMLFormElement>): void;
  onAttachFiles(files: readonly File[]): void;
  onRemoveAttachment(id: string): void;
  onTranscribeVoice?(file: File): Promise<string>;
  onStop(): void;
}) {
  const hasText = value.trim().length > 0;
  const hasPayload = hasText || attachments.length > 0;
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const formRef = useRef<HTMLFormElement | null>(null);
  const editorControlsRef = useRef<AgentRichTextEditorControls | null>(null);
  const dragDepthRef = useRef(0);
  const [dragOver, setDragOver] = useState(false);
  const [voiceState, setVoiceState] = useState<'idle' | 'recording' | 'transcribing'>('idle');
  const [voiceError, setVoiceError] = useState<string | null>(null);
  const [mentionIndex, setMentionIndex] = useState(0);
  const [recentEmojiIds, setRecentEmojiIds] = useState<string[]>(() => {
    if (typeof window === 'undefined') return [];
    try {
      const parsed = JSON.parse(window.localStorage.getItem('fabushi.desktop.composer-recent-emoji.v1') || '[]');
      return Array.isArray(parsed) ? parsed.filter((value): value is string => typeof value === 'string').slice(0, 24) : [];
    } catch {
      return [];
    }
  });
  const canSend = hasPayload && ready && !uploading && voiceState === 'idle';
  const atAttachmentLimit = attachments.length >= AGENT_ATTACHMENT_LIMIT;
  const voiceRecorderRef = useRef<MediaRecorder | null>(null);
  const voiceStreamRef = useRef<MediaStream | null>(null);
  const voiceChunksRef = useRef<Blob[]>([]);

  const mentionMatch = /(?:^|\s)@([^@\n]{0,50})$/.exec(value);
  const mentionQuery = mentionMatch?.[1]?.trim().toLocaleLowerCase() ?? null;
  const mentionResults = useMemo(() => mentionQuery == null
    ? []
    : mentionCandidates
      .filter((candidate) => !mentionQuery || `${candidate.name} ${candidate.description ?? ''}`.toLocaleLowerCase().includes(mentionQuery))
      .slice(0, 8), [mentionCandidates, mentionQuery]);
  const workflowMatch = /(?:^|\s)\/([^/\n]{0,50})$/.exec(value);
  const workflowQuery = workflowMatch?.[1]?.trim().toLocaleLowerCase() ?? null;
  const workflowResults = useMemo(() => workflowQuery == null
    ? []
    : workflowCandidates
      .filter((candidate) => !workflowQuery || `${candidate.name} ${candidate.description ?? ''}`.toLocaleLowerCase().includes(workflowQuery))
      .slice(0, 8), [workflowCandidates, workflowQuery]);
  const emojiMatch = /(?:^|\s):([a-z0-9_+\-]{2,50})$/i.exec(value);
  const emojiQuery = emojiMatch?.[1] ?? null;
  const emojiResults = useMemo(
    () => emojiQuery == null ? [] : emojiSuggestions(emojiQuery, recentEmojiIds, 12),
    [emojiQuery, recentEmojiIds],
  );
  const pullRequestMatch = /(?:^|\s)#([^#\n]{0,50})$/.exec(value);
  const pullRequestQuery = pullRequestMatch?.[1] ?? null;
  const pullRequestResults = useMemo(
    () => pullRequestQuery == null ? [] : filterPullRequestSuggestions(pullRequestCandidates, pullRequestQuery),
    [pullRequestCandidates, pullRequestQuery],
  );
  const suggestionCount = mentionResults.length
    || workflowResults.length
    || emojiResults.length
    || pullRequestResults.length;

  useEffect(() => {
    setMentionIndex((index) => Math.min(index, Math.max(0, suggestionCount - 1)));
  }, [suggestionCount]);

  const insertMention = (candidate: AgentComposerMentionCandidate) => {
    const next = value.replace(/(^|\s)@([^@\n]{0,50})$/, (_match, prefix: string) => `${prefix}@${candidate.name} `);
    onChange(next);
    onMention?.(candidate);
    window.requestAnimationFrame(() => editorControlsRef.current?.focus());
  };

  const insertWorkflow = (candidate: AgentComposerWorkflowCandidate) => {
    const next = value.replace(/(^|\s)\/([^/\n]{0,50})$/, (_match, prefix: string) => `${prefix}/${candidate.name} `);
    onChange(next);
    onWorkflowReference?.(candidate);
    window.requestAnimationFrame(() => editorControlsRef.current?.focus());
  };

  const insertEmoji = (candidate: ReturnType<typeof emojiSuggestions>[number]) => {
    const tokenLength = 1 + (emojiMatch?.[1]?.length ?? 0);
    editorControlsRef.current?.replaceTokenWithEmoji(tokenLength, candidate.native);
    setRecentEmojiIds((current) => {
      const next = [candidate.id, ...current.filter((id) => id !== candidate.id)].slice(0, 24);
      try {
        window.localStorage.setItem('fabushi.desktop.composer-recent-emoji.v1', JSON.stringify(next));
      } catch {
        // Recency is only a local ranking hint.
      }
      return next;
    });
  };

  const insertPullRequest = (candidate: AgentPullRequestSuggestion) => {
    const tokenLength = 1 + (pullRequestMatch?.[1]?.length ?? 0);
    editorControlsRef.current?.replaceTokenWithPullRequest(tokenLength, candidate);
  };

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
          if (transcript) editorControlsRef.current?.insertText(transcript);
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

  const handleKeyDown = (event: globalThis.KeyboardEvent): boolean => {
    if (event.isComposing) return false;
    if (event.key === 'Escape' && voiceState === 'recording') {
      event.preventDefault();
      stopVoice();
      return true;
    }
    if (suggestionCount) {
      if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
        event.preventDefault();
        const delta = event.key === 'ArrowDown' ? 1 : -1;
        setMentionIndex((index) => (index + delta + suggestionCount) % suggestionCount);
        return true;
      }
      if ((event.key === 'Enter' || event.key === 'Tab') && mentionResults[mentionIndex]) {
        event.preventDefault();
        insertMention(mentionResults[mentionIndex]!);
        return true;
      }
      if ((event.key === 'Enter' || event.key === 'Tab') && workflowResults[mentionIndex]) {
        event.preventDefault();
        insertWorkflow(workflowResults[mentionIndex]!);
        return true;
      }
      if ((event.key === 'Enter' || event.key === 'Tab') && emojiResults[mentionIndex]) {
        event.preventDefault();
        insertEmoji(emojiResults[mentionIndex]!);
        return true;
      }
      if ((event.key === 'Enter' || event.key === 'Tab') && pullRequestResults[mentionIndex]) {
        event.preventDefault();
        insertPullRequest(pullRequestResults[mentionIndex]!);
        return true;
      }
      if (event.key === 'Escape') {
        event.preventDefault();
        if (mentionResults.length) onChange(value.replace(/(^|\s)@([^@\n]{0,50})$/, '$1'));
        else if (workflowResults.length) onChange(value.replace(/(^|\s)\/([^/\n]{0,50})$/, '$1'));
        else if (emojiResults.length) onChange(value.replace(/(^|\s):([a-z0-9_+\-]{2,50})$/i, '$1'));
        else onChange(value.replace(/(^|\s)#([^#\n]{0,50})$/, '$1'));
        return true;
      }
    }
    if (event.key === 'Escape') {
      event.preventDefault();
      editorControlsRef.current?.blur();
      return true;
    }
    const submitWithEnter = enterToSend && event.key === 'Enter' && !event.shiftKey;
    const submitWithShortcut = !enterToSend
      && event.key === 'Enter'
      && (event.metaKey || event.ctrlKey);
    if (!submitWithEnter && !submitWithShortcut) return false;
    event.preventDefault();
    if (canSend) formRef.current?.requestSubmit();
    return true;
  };

  const stageFiles = (files: readonly File[]) => {
    const remaining = Math.max(0, AGENT_ATTACHMENT_LIMIT - attachments.length);
    if (remaining === 0) return;
    const accepted = files.slice(0, remaining);
    if (accepted.length) onAttachFiles(accepted);
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
    if (files.length) stageFiles(files);
  };

  return <form
    ref={formRef}
    className={styles.root}
    data-testid="grok-agent-composer"
    data-drag-over={dragOver || undefined}
    onSubmit={(event) => {
      if (!canSend) {
        event.preventDefault();
        return;
      }
      onSubmit(event);
    }}
    onDragEnter={handleDragEnter}
    onDragLeave={handleDragLeave}
    onDragOver={(event) => {
      if (!hasDraggedFiles(event)) return;
      event.preventDefault();
      event.dataTransfer.dropEffect = 'copy';
    }}
    onDrop={handleDrop}
  >
    {dragOver ? <div className={styles.dropOverlay} aria-hidden="true">Drop files to add to {agentName}</div> : null}
    {replyTarget ? <div className={styles.replyTarget} data-testid="reply-message-banner">
      <Reply size={15} />
      <div><strong>{replyTarget.label}</strong><span>{replyTarget.text}</span></div>
      {onClearReplyTarget ? <button type="button" data-testid="reply-message-cancel" aria-label="Cancel reply" onClick={onClearReplyTarget}><X size={14} /></button> : null}
    </div> : null}
    {attachments.length ? <div className={styles.attachments} role="list" aria-label="Attachments">
      {attachments.map((attachment) => <span key={attachment.id} className={styles.attachment} role="listitem">
        <span><strong>{attachment.name}</strong>{attachmentSize(attachment.sizeBytes) ? <small>{attachmentSize(attachment.sizeBytes)}</small> : null}</span>
        <button type="button" aria-label={`Remove ${attachment.name}`} onClick={() => onRemoveAttachment(attachment.id)}><X size={13} /></button>
      </span>)}
    </div> : null}
    <button type="button" className={styles.attach} title="Attach files" aria-label="Attach files" disabled={!ready || uploading || voiceState !== 'idle' || atAttachmentLimit} onClick={() => fileInputRef.current?.click()}>
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
        if (files.length) stageFiles(files);
      }}
    />
    <div className={styles.editorWrap}>
      <AgentRichTextEditor
        key={scopeKey}
        prompt={value}
        richText={richText}
        scopeKey={scopeKey}
        disabled={!ready || uploading}
        placeholder={attachments.length ? `Add a message for ${agentName}` : `Message ${agentName}`}
        className={styles.editor}
        ariaLabel={`Message ${agentName}`}
        onChange={(draft) => onChange(draft.prompt, draft.richText)}
        onKeyDown={handleKeyDown}
        onPasteFiles={stageFiles}
        onControls={(controls) => { editorControlsRef.current = controls; }}
      />
      {mentionResults.length ? <div className={styles.mentions} role="listbox" aria-label="Mention an Agent">
        {mentionResults.map((candidate, index) => <button
          key={candidate.id}
          type="button"
          role="option"
          aria-selected={index === mentionIndex}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => insertMention(candidate)}
        >
          <span><strong>@{candidate.name}</strong>{candidate.description ? <small>{candidate.description}</small> : null}</span><small>{candidate.kind === 'mcp' ? 'MCP' : 'Agent'}</small>
        </button>)}
      </div> : null}
      {workflowResults.length ? <div className={styles.mentions} role="listbox" aria-label="Reference a workflow">
        {workflowResults.map((candidate, index) => <button
          key={candidate.id}
          type="button"
          role="option"
          aria-selected={index === mentionIndex}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => insertWorkflow(candidate)}
        >
          <span><strong>/{candidate.name}</strong>{candidate.description ? <small>{candidate.description}</small> : null}</span>
        </button>)}
      </div> : null}
      {emojiResults.length ? <div className={styles.mentions} role="listbox" aria-label="Insert emoji">
        {emojiResults.map((candidate, index) => <button
          key={candidate.id}
          type="button"
          role="option"
          aria-selected={index === mentionIndex}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => insertEmoji(candidate)}
        >
          <span><strong>{candidate.native} :{candidate.shortcodes[0] ?? candidate.id}:</strong><small>{candidate.name}</small></span>
        </button>)}
      </div> : null}
      {pullRequestResults.length ? <div className={styles.mentions} role="listbox" aria-label="Reference a pull request">
        {pullRequestResults.map((candidate, index) => <button
          key={candidate.url}
          type="button"
          role="option"
          aria-selected={index === mentionIndex}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => insertPullRequest(candidate)}
        >
          <span><strong>#{candidate.prNumber} {candidate.title}</strong>{candidate.repository ? <small>{candidate.repository}</small> : null}</span><small>PR</small>
        </button>)}
      </div> : null}
      {voiceState === 'recording' ? <span className={styles.voiceStatus}>Recording…</span> : voiceError ? <span className={styles.voiceError} title={voiceError}>Voice unavailable</span> : null}
    </div>
    {hasPayload ? (
      <button data-testid="messenger-send" className={styles.primary} type="submit" disabled={!canSend}>
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
