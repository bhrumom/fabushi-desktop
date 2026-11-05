/**
 * Conversation workspace composer with rich-text editing, voice dictation,
 * attachment staging, non-destructive typed action popover, and Human silent /
 * scheduled delivery options.
 *
 * Upstream & Open PR Alignment:
 * - Telegram Desktop `Telegram/SourceFiles/api/api_sending.cpp` (`InnerFillMessagePostFlags`
 *   silent/scheduled send options; `SendSimpleMedia` `action.clearDraft = false`).
 * - PR #111 (`origin/tdrp/human-silent-scheduled-current-main-20261018`):
 *   `onSendSilently` ("Send Without Sound") and `onScheduleSend` ("Schedule Message")
 *   via `SandMenuRoot` and future-time-validated schedule dialog.
 * - Blueprint 03 (`docs/specs/fusion-blueprints/03-unified-composer-and-typed-actions.md`):
 *   Unified composer contract (`onDirectSendTypedAction`, `onSubmitDirectMessage`,
 *   dynamic model/thinking effort selectors).
 */

import { useCallback, useEffect, useMemo, useRef, useState, type ChangeEvent, type DragEvent, type FormEvent } from "react";
import { COMPOSER_ATTACHMENT_LIMIT, attachmentBasename, formatAttachmentBytes, inferAttachmentKind, isComposerDraftEmpty, type ComposerDraft, type DraftAttachment } from "./model";
import type { DirectTypedActionPayload } from "./submission";
import { useVoiceSession, VoiceWaveform, type VoiceTranscriber } from "./voice";
import { ComposerReplyPill, replyComposerPlaceholder, type ComposerReplyTarget } from "./reply-preview";
import { PromptRichTextEditor, type PromptEditorControls, type PromptEditorProviders } from "./rich-text-editor";
import { SandIcon, SandIconButton } from "../../../ui/sand-kit-primitives";
import {
  SandMenuItem,
  SandMenuList,
  SandMenuPopup,
  SandMenuPortal,
  SandMenuPositioner,
  SandMenuRoot,
  SandMenuTrigger,
} from "../../../ui/sand-menu-primitives";
import { SandSpinner } from "../../../ui/sand-status-primitives";
import { TdrpAttachMenuPopover } from "../../../../production/tdrp/TdrpAttachMenuPopover";

// Immutable Mac voice carriers: index-UbX-y3il.js#byteOffset=4538599 (recording
// chip/waveform), 4539081 (Stop dictation), and 4543238 (mic action).
// Windows equivalents: 5703580, 5704161, and 5709324.

const PROMPT_ATTACH_CLASS = "sand-prompt-attach sand-2lah0s sand-i07v4r sand-uo9n5k sand-1vhj7fz sand-4b2ntj sand-1dsx48b sand-1hc1fzr sand-1lfpgzf sand-1ypdohk";
const PROMPT_MIC_EMPTY_CLASS = "sand-prompt-mic sand-2lah0s sand-jbqb8w sand-uo9n5k sand-19aaqeu sand-1dsx48b sand-1hc1fzr sand-1lfpgzf sand-19991ni sand-13dflua sand-12w9bfk sand-b51amx";
const PROMPT_MIC_PAYLOAD_CLASS = "sand-prompt-mic sand-2lah0s sand-jbqb8w sand-uo9n5k sand-19aaqeu sand-1dsx48b sand-1ypdohk sand-19991ni sand-13dflua sand-12w9bfk sand-b51amx sand-g01cxk";
const PROMPT_SEND_CLASS = "sand-prompt-send sand-2lah0s sand-mak4db sand-1tc92z3 sand-1hc1fzr sand-1p5hr7d sand-1lfpgzf sand-1ypdohk";
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=4540240 (Nl.glyphShown/glyphHidden opacity+scale classes)
// @evidence recovered/frontend/app/assets/index-UbX-y3il.js#byteOffset=5705592 (Windows prompt glyph state classes)
const COMPOSER_GLYPH_VISIBLE_CLASS = "sand-1hc1fzr sand-3oybdh";
const COMPOSER_GLYPH_HIDDEN_CLASS = "sand-g01cxk sand-1a33avv";
const RECORDING_CHIP_CLASS = "sand-recording-chip sand-9f619 sand-3nfvp2 sand-pkkfsy sand-1th6cxs sand-f6zju3 sand-16b7oty sand-cnij5n sand-o7x2bt sand-2lah0s sand-c342km sand-ng3xce sand-1i4c3av sand-i07v4r sand-1kj6vsg sand-1ypdohk sand-1k57tk5 sand-784prv sand-1t137rt sand-9v5kkp sand-1uczgqu sand-1725o6r sand-omy3lu";

function ComposerGlyph({ name, hidden = false }: { readonly name: "mic" | "arrow-up"; readonly hidden?: boolean }) {
  return <SandIcon className={hidden ? COMPOSER_GLYPH_HIDDEN_CLASS : COMPOSER_GLYPH_VISIBLE_CLASS} name={name} size="sm" style={{ lineHeight: 1 }} variant="filled" />;
}

export function formatDefaultScheduleValue(nowMs = Date.now()): string {
  const target = new Date(nowMs + 60 * 60 * 1000);
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${target.getFullYear()}-${pad(target.getMonth() + 1)}-${pad(target.getDate())}T${pad(target.getHours())}:${pad(target.getMinutes())}`;
}

// Immutable prompt editor keyboard contract: Escape cancels an active voice
// session or blurs the prompt when no voice session is active.
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=4738296 (e9n prompt editor; UTF-8 region SHA-256 0a96b805946245180330a9bffda0053e11015e9c8fe850dfa62361ab85acee35)
// @evidence recovered/frontend/app/assets/index-UbX-y3il.js#byteOffset=5705592 (e9n prompt editor; UTF-8 region SHA-256 6f8b718ba4e43704baea5078c9498184b98f82a3ad84d47bf1e5a9f516988f07)

export interface ConversationComposerProps {
  draft: ComposerDraft;
  disabled?: boolean;
  notice?: string | null;
  placeholder?: string;
  transcribeAudio: VoiceTranscriber;
  onChange(draft: ComposerDraft): void;
  onStageFiles(files: File[]): void | Promise<void>;
  onSubmit(): void | Promise<void>;
  onSendSilently?(): void | Promise<void>;
  onScheduleSend?(scheduledAtMs: number): void | Promise<void>;
  onSubmitWithOptions?(options?: {
    silent?: boolean;
    scheduledAtMs?: number;
    scheduleDate?: number;
    scheduledAt?: string;
  }): void | Promise<void>;
  silent?: boolean;
  scheduleDate?: number;
  scheduledAt?: string;
  activeModel?: string;
  availableModels?: ReadonlyArray<{ id: string; label: string }>;
  thinkingEffort?: "low" | "medium" | "high";
  onChangeModel?(modelId: string): void;
  onChangeThinkingEffort?(effort: "low" | "medium" | "high"): void;
  onDirectSendTypedAction?(payload: DirectTypedActionPayload): void | Promise<void>;
  onSubmitDirectMessage?(payload: string): void | Promise<void>;
  onRemoveAttachment?(attachment: DraftAttachment): void | Promise<void>;
  replyTarget?: ComposerReplyTarget;
  onClearReplyTarget?(): void;
  editorProviders?: PromptEditorProviders;
  scopeKey?: string;
  acceptedSendGeneration?: number;
  onOpenModal?: (mode: "poll" | "todo" | "invoice" | "bot" | "gift") => void;
}

export function extractClipboardFiles(items: DataTransferItemList | null | undefined): File[] {
  if (items == null) return [];
  const files: File[] = [];
  for (const item of Array.from(items)) {
    if (item.kind !== "file") continue;
    const file = item.getAsFile();
    if (file != null) files.push(file);
  }
  return files;
}

export function hasFileDragData(types: Iterable<string> | null | undefined): boolean {
  if (types == null) return false;
  for (const type of types) if (type === "Files") return true;
  return false;
}

export function selectComposerFiles(files: readonly File[], existingCount: number): File[] {
  const remaining = Math.max(0, COMPOSER_ATTACHMENT_LIMIT - existingCount);
  return files.slice(0, remaining);
}

export function ConversationComposer({
  acceptedSendGeneration = 0,
  draft,
  disabled = false,
  notice,
  placeholder = "Ask anything, or drop a file.",
  transcribeAudio,
  onChange,
  onClearReplyTarget,
  onRemoveAttachment,
  onStageFiles,
  onSubmit,
  onSendSilently,
  onScheduleSend,
  onSubmitWithOptions,
  activeModel,
  availableModels,
  thinkingEffort,
  onChangeModel,
  onChangeThinkingEffort,
  onDirectSendTypedAction,
  onSubmitDirectMessage,
  replyTarget,
  editorProviders,
  scopeKey,
  onOpenModal,
}: ConversationComposerProps) {
  const fileInput = useRef<HTMLInputElement>(null);
  const editorControls = useRef<PromptEditorControls | null>(null);
  const dragDepth = useRef(0);
  const [isDragOver, setIsDragOver] = useState(false);
  const [isAttachMenuOpen, setIsAttachMenuOpen] = useState(false);
  const [sendMenuOpen, setSendMenuOpen] = useState(false);
  const [scheduleOpen, setScheduleOpen] = useState(false);
  const [scheduleValue, setScheduleValue] = useState(() => formatDefaultScheduleValue());
  const [scheduleError, setScheduleError] = useState<string | null>(null);
  const draftRef = useRef(draft);
  const hasPayload = !isComposerDraftEmpty(draft);
  draftRef.current = draft;
  const voiceOptions = useMemo(() => ({ transcribe: transcribeAudio }), [transcribeAudio]);
  const voice = useVoiceSession(voiceOptions, scopeKey);
  const voiceBusy = voice.isRecording || voice.isProcessing || voice.isActivating;
  const canSend = hasPayload && !disabled && !voiceBusy;
  const hasSendOptions = onSendSilently != null || onScheduleSend != null || onSubmitWithOptions != null;
  const atLimit = draft.attachments.length >= COMPOSER_ATTACHMENT_LIMIT;

  useEffect(() => voice.controller.onFinal((text) => {
    editorControls.current?.insertText(text);
    const focusEditor = () => editorControls.current?.focus();
    if (typeof requestAnimationFrame === "function") requestAnimationFrame(focusEditor);
    else focusEditor();
  }), [voice.controller]);

  useEffect(() => {
    if (disabled && voiceBusy) voice.handleCancelClick();
  }, [disabled, voiceBusy, voice.handleCancelClick]);

  useEffect(() => {
    if (acceptedSendGeneration > 0) {
      editorControls.current?.clear();
      setSendMenuOpen(false);
      setScheduleOpen(false);
      setScheduleError(null);
    }
  }, [acceptedSendGeneration]);

  useEffect(() => {
    setSendMenuOpen(false);
    setScheduleOpen(false);
    setScheduleError(null);
  }, [scopeKey]);

  const cancelVoiceAndRefocus = useCallback(() => {
    voice.handleCancelClick();
    const focusEditor = () => editorControls.current?.focus();
    if (typeof requestAnimationFrame === "function") requestAnimationFrame(focusEditor);
    else focusEditor();
  }, [voice.handleCancelClick]);

  const receiveEditorControls = useCallback((controls: PromptEditorControls | null) => {
    editorControls.current = controls;
  }, []);

  const onEditorChange = useCallback((change: { readonly prompt: string; readonly richText?: string }) => {
    onChange({
      ...draftRef.current,
      prompt: change.prompt,
      richText: change.richText
    });
  }, [onChange]);

  const stageFiles = useCallback((files: readonly File[]) => {
    const accepted = selectComposerFiles(files, draft.attachments.length);
    if (accepted.length > 0) void onStageFiles(accepted);
  }, [draft.attachments.length, onStageFiles]);

  const stageSelectedFiles = (event: ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.currentTarget.files ?? []);
    event.currentTarget.value = "";
    stageFiles(files);
  };

  const hasDraggedFiles = useCallback((event: DragEvent<HTMLDivElement>) => hasFileDragData(event.dataTransfer?.types), []);
  const onDragEnter = useCallback((event: DragEvent<HTMLDivElement>) => {
    if (!hasDraggedFiles(event)) return;
    event.preventDefault();
    dragDepth.current += 1;
    setIsDragOver(true);
  }, [hasDraggedFiles]);
  const onDragOver = useCallback((event: DragEvent<HTMLDivElement>) => {
    if (!hasDraggedFiles(event)) return;
    event.preventDefault();
    if (event.dataTransfer != null) event.dataTransfer.dropEffect = "copy";
  }, [hasDraggedFiles]);
  const onDragLeave = useCallback((event: DragEvent<HTMLDivElement>) => {
    if (!hasDraggedFiles(event)) return;
    dragDepth.current = Math.max(0, dragDepth.current - 1);
    if (dragDepth.current === 0) setIsDragOver(false);
  }, [hasDraggedFiles]);
  const onDrop = useCallback((event: DragEvent<HTMLDivElement>) => {
    if (!hasDraggedFiles(event)) return;
    event.preventDefault();
    dragDepth.current = 0;
    setIsDragOver(false);
    stageFiles(Array.from(event.dataTransfer?.files ?? []));
  }, [hasDraggedFiles, stageFiles]);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (canSend) void onSubmit();
  };

  const triggerSilentSend = () => {
    if (!canSend) return;
    setSendMenuOpen(false);
    setScheduleOpen(false);
    setScheduleError(null);
    if (onSendSilently != null) {
      void onSendSilently();
    } else if (onSubmitWithOptions != null) {
      void onSubmitWithOptions({ silent: true });
    }
  };

  const submitScheduledSend = () => {
    if (!canSend || (onScheduleSend == null && onSubmitWithOptions == null)) return;
    const parsed = Date.parse(scheduleValue);
    if (!Number.isFinite(parsed) || parsed <= Date.now()) {
      setScheduleError("Choose a future date and time.");
      return;
    }
    setScheduleError(null);
    setScheduleOpen(false);
    if (onScheduleSend != null) {
      void onScheduleSend(parsed);
    } else if (onSubmitWithOptions != null) {
      void onSubmitWithOptions({
        scheduledAtMs: parsed,
        scheduleDate: Math.floor(parsed / 1000),
        scheduledAt: new Date(parsed).toISOString(),
      });
    }
  };

  return (
    <form className="sand-prompt-form" onSubmit={submit}>
      <div className="sand-prompt-shell" data-expanded={hasPayload || undefined} onDragEnter={onDragEnter} onDragLeave={onDragLeave} onDragOver={onDragOver} onDrop={onDrop}>
        {isDragOver ? <div aria-hidden="true" className="sand-chat-drop-overlay"><div className="sand-chat-drop-overlay__badge">Drop files to add to chat</div></div> : null}
        {notice ? <p aria-live="polite" className="sand-prompt-attachment-notice" role="status">{notice}</p> : null}
        {replyTarget == null || onClearReplyTarget == null ? null : <ComposerReplyPill onClear={onClearReplyTarget} target={replyTarget} />}
        {draft.attachments.length > 0 ? (
          <div aria-label="Attachments" className="sand-prompt-attachments" role="list">
            {draft.attachments.map((attachment, index) => {
              const label = attachment.name || attachmentBasename(attachment.path);
              return <span aria-label={label} className="sand-prompt-attachment" data-kind={inferAttachmentKind({ mimeType: attachment.mimeType, fileName: label, urlOrPath: attachment.path })} key={`${attachment.path}:${index}`} role="listitem">
                <span><strong>{attachment.name}</strong>{attachment.size == null ? null : <small>{formatAttachmentBytes(attachment.size)}</small>}</span>
                <button aria-label={`Remove ${label}`} onClick={() => {
                  onChange({ ...draft, attachments: draft.attachments.filter((candidate) => candidate !== attachment) });
                  void onRemoveAttachment?.(attachment);
                }} type="button"><SandIcon name="close" size="xs" /></button>
              </span>;
            })}
          </div>
        ) : null}
        <PromptRichTextEditor
          canSubmit={canSend}
          clearGeneration={acceptedSendGeneration}
          disabled={disabled}
          onChange={onEditorChange}
          onControls={receiveEditorControls}
          onEscape={() => {
            if (voiceBusy) cancelVoiceAndRefocus();
            else editorControls.current?.blur();
          }}
          onPasteFiles={stageFiles}
          onSubmit={onSubmit}
          placeholder={replyTarget == null ? placeholder : replyComposerPlaceholder(replyTarget.preview)}
          prompt={draft.prompt}
          providers={editorProviders}
          richText={draft.richText}
          scopeKey={scopeKey}
        />
        {voice.isRecording || voice.isActivating ? <span aria-live="polite" className="sand-prompt-voice-status" role="status">Listening…</span> : null}
        <div className="sand-prompt-actions-row">
          <div style={{ position: "relative", display: "inline-flex", alignItems: "center", gap: 6 }}>
            <SandIconButton
              aria-label="Attach file"
              className={PROMPT_ATTACH_CLASS}
              disabled={disabled || atLimit || voiceBusy}
              icon="plus"
              onClick={() => {
                if (onOpenModal != null) {
                  setIsAttachMenuOpen((open) => !open);
                } else {
                  fileInput.current?.click();
                }
              }}
              shape="circle"
              size="lg"
              type="button"
              variant="default"
            />
            <TdrpAttachMenuPopover
              isOpen={isAttachMenuOpen}
              onClose={() => setIsAttachMenuOpen(false)}
              onSelectPhotoOrFile={() => fileInput.current?.click()}
              onOpenModal={(mode) => {
                setIsAttachMenuOpen(false);
                onOpenModal?.(mode);
              }}
              onDirectSendTypedAction={onDirectSendTypedAction}
              onSubmitDirectMessage={onSubmitDirectMessage}
            />
            {availableModels != null ? (
              availableModels.length > 0 ? (
                <select
                  aria-label="Select model"
                  className="sand-prompt-model-select"
                  disabled={disabled || onChangeModel == null}
                  onChange={(event) => onChangeModel?.(event.currentTarget.value)}
                  value={activeModel ?? availableModels[0]?.id ?? ""}
                >
                  {availableModels.map((model) => (
                    <option key={model.id} value={model.id}>
                      {model.label}
                    </option>
                  ))}
                </select>
              ) : (
                <span aria-label="Model status" className="sand-prompt-model-unconfigured" role="status">
                  No models configured
                </span>
              )
            ) : null}
            {thinkingEffort != null && onChangeThinkingEffort != null ? (
              <select
                aria-label="Select thinking effort"
                className="sand-prompt-thinking-select"
                disabled={disabled}
                onChange={(event) =>
                  onChangeThinkingEffort(event.currentTarget.value as "low" | "medium" | "high")
                }
                value={thinkingEffort}
              >
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
              </select>
            ) : null}
          </div>
          <span className="sand-prompt-actions-trailing sand-prompt-cta-cluster sand-78zum5 sand-6s0dn4 sand-2lah0s">
            {voice.isRecording ? <button aria-label="Stop dictation" className={RECORDING_CHIP_CLASS} onClick={() => voice.handleStopClick()} onKeyDown={(event) => {
              if (event.key === "Escape") {
                event.preventDefault();
                cancelVoiceAndRefocus();
              }
            }} type="button">
              <span aria-hidden="true" className="sand-recording-chip__stop sand-1fsd2vl sand-170jfvy sand-2lah0s sand-1bl94mz sand-mak4db" />
              <span aria-hidden="true" className="sand-recording-chip__timer sand-2lah0s sand-fc7y3v sand-1yxxptd sand-1bignsj sand-ss6m8b">{voice.recordingDuration}</span>
              <span className="sand-recording-chip__waveform sand-1xp8n7a sand-18gnavp sand-2lah0s sand-78zum5 sand-6s0dn4"><VoiceWaveform stream={voice.stream} /></span>
            </button> : voice.isProcessing ? <span aria-label="Transcribing voice input…" className="sand-prompt-voice-processing sand-2lah0s sand-16w9d4f sand-1th6cxs sand-78zum5 sand-6s0dn4 sand-l56j7k" role="status"><SandSpinner ariaLabel="Transcribing voice input…" size={18} />Transcribing…</span> : <>
              {hasPayload ? <SandIconButton aria-label="Start voice input" className={PROMPT_MIC_PAYLOAD_CLASS} disabled={disabled || voiceBusy} icon="mic" onClick={() => voice.handleMicClick()} shape="circle" size="lg" type="button" variant="default" /> : null}
              {hasPayload ? <>
                <button aria-label="Send message" className={PROMPT_SEND_CLASS} disabled={!canSend} type="submit"><span className="sand-1n2onr6 sand-1kky2od sand-lup9mm"><ComposerGlyph hidden={hasPayload} name="mic" /><ComposerGlyph hidden={!hasPayload} name="arrow-up" /></span></button>
                {hasSendOptions ? <SandMenuRoot onOpenChange={setSendMenuOpen} open={sendMenuOpen}>
                  <SandMenuTrigger aria-label="Send options" className="sand-prompt-send-options" disabled={!canSend} type="button">
                    ▾
                  </SandMenuTrigger>
                  <SandMenuPortal>
                    <SandMenuPositioner align="end" sideOffset={8}>
                      <SandMenuPopup aria-label="Send options" style={{ minWidth: "200px" }}>
                        <SandMenuList>
                          {onSendSilently != null || onSubmitWithOptions != null ? <SandMenuItem onClick={triggerSilentSend}>Send Without Sound</SandMenuItem> : null}
                          {onScheduleSend != null || onSubmitWithOptions != null ? <SandMenuItem onClick={() => {
                            setSendMenuOpen(false);
                            setScheduleError(null);
                            setScheduleValue(formatDefaultScheduleValue());
                            setScheduleOpen(true);
                          }}>Schedule Message</SandMenuItem> : null}
                        </SandMenuList>
                      </SandMenuPopup>
                    </SandMenuPositioner>
                  </SandMenuPortal>
                </SandMenuRoot> : null}
              </> : <SandIconButton aria-label="Start voice input" className={PROMPT_MIC_EMPTY_CLASS} disabled={disabled} icon="mic" onClick={() => voice.handleMicClick()} shape="circle" size="lg" type="button" variant="default" />}
            </>}
          </span>
        </div>
        {scheduleOpen && (onScheduleSend != null || onSubmitWithOptions != null) ? <div aria-label="Schedule message" className="sand-prompt-schedule-popover" role="dialog">
          <label className="sand-prompt-schedule-label">
            <span>Schedule send time</span>
            <input
              aria-label="Scheduled send time"
              className="sand-prompt-schedule-input"
              onChange={(event) => {
                setScheduleValue(event.currentTarget.value);
                setScheduleError(null);
              }}
              type="datetime-local"
              value={scheduleValue}
            />
          </label>
          {scheduleError != null ? <p aria-live="polite" className="sand-prompt-schedule-error" role="alert">{scheduleError}</p> : null}
          <div className="sand-prompt-schedule-actions">
            <button
              className="sand-prompt-schedule-confirm"
              disabled={!canSend}
              onClick={submitScheduledSend}
              type="button"
            >
              Schedule
            </button>
            <button
              className="sand-prompt-schedule-cancel"
              onClick={() => {
                setScheduleOpen(false);
                setScheduleError(null);
              }}
              type="button"
            >
              Cancel
            </button>
          </div>
        </div> : null}
        {voice.error ? <p aria-live="polite" className="sand-prompt-voice-error" role="status">{voice.error.message}</p> : null}
        <input aria-hidden="true" className="sand-prompt-file-input" multiple onChange={stageSelectedFiles} ref={fileInput} tabIndex={-1} type="file" />
      </div>
    </form>
  );
}
