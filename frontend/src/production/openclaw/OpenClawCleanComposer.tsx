/**
 * OpenClaw Clean Composer with strict CJK/IME composition protection,
 * dynamic provider model catalog indicator, silent/scheduled delivery options,
 * and non-destructive typed action triggers.
 *
 * Upstream & Open PR Alignment:
 * - OpenClaw `src/agents/model-catalog.ts`: Dynamic provider model discovery;
 *   never falls back to hardcoded fake models when no provider is configured.
 * - Telegram Desktop `Telegram/SourceFiles/api/api_sending.cpp` (`InnerFillMessagePostFlags`
 *   silent/scheduled send flags; `SendSimpleMedia` `action.clearDraft = false`
 *   non-destructive typed actions).
 * - PR #111 (`origin/tdrp/human-silent-scheduled-current-main-20261018`):
 *   Silent ("Send Without Sound") and future-validated scheduled delivery.
 * - Blueprint 03 (`docs/specs/fusion-blueprints/03-unified-composer-and-typed-actions.md`):
 *   Section 2.4 (`OpenClawCleanComposer.tsx` IME guard, dynamic models, direct typed actions).
 */

import React, { useState, useRef, useEffect } from "react";
import type { ComposerDraft, DraftAttachment } from "../../recovered/features/conversation/workspace/model";
import type { DirectTypedActionPayload } from "../../recovered/features/conversation/workspace/submission";
import { formatDefaultScheduleValue } from "../../recovered/features/conversation/workspace/composer";
import {
  ShieldGateIcon,
  MicrophoneIcon,
  SendArrowUpIcon,
} from "./OpenClawIcons";

export type OpenClawComposerDraftInput =
  | string
  | ComposerDraft
  | {
      readonly prompt: string;
      readonly richText?: string;
      readonly attachments?: readonly DraftAttachment[];
    };

export interface OpenClawSendOptions {
  readonly silent?: boolean;
  readonly scheduledAtMs?: number;
  readonly scheduleDate?: number;
  readonly scheduledAt?: string;
}

export interface OpenClawCleanComposerProps {
  draft: OpenClawComposerDraftInput;
  onChange: ((value: string) => void) | ((value: OpenClawComposerDraftInput) => void);
  onSubmit: (options?: OpenClawSendOptions) => void | Promise<void>;
  disabled?: boolean;
  placeholder?: string;
  onStageFiles?: (files?: File[]) => void | Promise<void>;
  onOpenAttachPopover?: () => void;
  onCreatePoll?: () => void;
  onCreateTodoList?: () => void;
  onCreateInvoice?: () => void;
  onSubmitPollOrTodo?: (payload: DirectTypedActionPayload | string) => Promise<void> | void;
  onSubmitDirectMessage?: (payload: string) => Promise<void> | void;
  onSendSilently?: () => void | Promise<void>;
  onScheduleSend?: (scheduledAtMs: number) => void | Promise<void>;
  silent?: boolean;
  scheduleDate?: number;
  scheduledAt?: string;
  securityGate?: "deny" | "allowlist" | "full";
  onChangeSecurityGate?: (next: "deny" | "allowlist" | "full") => void;
  model?: string;
  activeModel?: string;
  onChangeModel?: (model: string) => void;
  availableModels?: ReadonlyArray<{ id: string; label: string }>;
  thinkingLevel?: string;
  onChangeThinkingLevel?: (level: string) => void;
  onTranscribeAudio?: (
    audio?: Uint8Array,
    mimeType?: string,
    language?: string
  ) => Promise<{ text: string }> | void;
  isTranscribing?: boolean;
}

export function OpenClawCleanComposer({
  draft,
  onChange,
  onSubmit,
  disabled = false,
  placeholder = "给 Assistant 发消息",
  onStageFiles,
  onOpenAttachPopover,
  onCreatePoll,
  onCreateTodoList,
  onCreateInvoice,
  onSubmitPollOrTodo,
  onSubmitDirectMessage,
  onSendSilently,
  onScheduleSend,
  securityGate = "allowlist",
  onChangeSecurityGate,
  model,
  activeModel,
  onChangeModel,
  availableModels = [],
  thinkingLevel,
  onChangeThinkingLevel,
  onTranscribeAudio,
  isTranscribing = false,
}: OpenClawCleanComposerProps) {
  const [attachMenuOpen, setAttachMenuOpen] = useState(false);
  const [modelMenuOpen, setModelMenuOpen] = useState(false);
  const [thinkingMenuOpen, setThinkingMenuOpen] = useState(false);
  const [sendOptionsOpen, setSendOptionsOpen] = useState(false);
  const [scheduleDialogOpen, setScheduleDialogOpen] = useState(false);
  const [scheduleValue, setScheduleValue] = useState(() => formatDefaultScheduleValue());
  const [scheduleError, setScheduleError] = useState<string | null>(null);
  const isComposingRef = useRef(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const draftText = typeof draft === "string" ? draft : (draft?.prompt ?? "");
  const hasAttachments =
    typeof draft !== "string" && Array.isArray(draft?.attachments) && draft.attachments.length > 0;
  const canSend = !disabled && (draftText.trim().length > 0 || hasAttachments);
  const resolvedModelLabel = activeModel ?? model ?? availableModels[0]?.label ?? "";

  const handleDraftChange = (nextText: string) => {
    const notifyChange = onChange as (value: OpenClawComposerDraftInput) => void;
    if (typeof draft === "string") {
      notifyChange(nextText);
    } else {
      notifyChange({ ...draft, prompt: nextText });
    }
  };

  const appendToDraftNonDestructively = (snippet: string) => {
    const trimmedExisting = draftText.trimEnd();
    handleDraftChange(trimmedExisting.length > 0 ? `${trimmedExisting}\n${snippet}` : snippet);
  };

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 180)}px`;
    }
  }, [draftText]);

  const handleKeyDown = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Strict CJK / IME composition guard (Blueprint 03 Section 2.4)
    if (event.nativeEvent.isComposing || isComposingRef.current || event.keyCode === 229) {
      return;
    }
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      if (canSend) {
        void onSubmit();
      }
    }
  };

  const handleSilentSend = () => {
    if (!canSend) return;
    setSendOptionsOpen(false);
    setScheduleDialogOpen(false);
    setScheduleError(null);
    if (onSendSilently != null) {
      void onSendSilently();
    } else {
      void onSubmit({ silent: true });
    }
  };

  const handleConfirmSchedule = () => {
    if (!canSend) return;
    const parsed = Date.parse(scheduleValue);
    if (!Number.isFinite(parsed) || parsed <= Date.now()) {
      setScheduleError("Choose a future date and time.");
      return;
    }
    setScheduleError(null);
    setScheduleDialogOpen(false);
    if (onScheduleSend != null) {
      void onScheduleSend(parsed);
    } else {
      void onSubmit({
        scheduledAtMs: parsed,
        scheduleDate: Math.floor(parsed / 1000),
        scheduledAt: new Date(parsed).toISOString(),
      });
    }
  };

  const cycleSecurityGate = () => {
    if (!onChangeSecurityGate) return;
    const next = securityGate === "deny" ? "allowlist" : securityGate === "allowlist" ? "full" : "deny";
    onChangeSecurityGate(next);
  };

  return (
    <div className="oc-clean-composer sand-prompt-form" role="group" aria-label="Composer card">
      {/* Hidden file input for attachment picking */}
      <input
        ref={fileInputRef}
        type="file"
        multiple
        style={{ display: "none" }}
        onChange={(e) => {
          if (e.target.files && e.target.files.length > 0 && onStageFiles) {
            onStageFiles(Array.from(e.target.files));
          }
        }}
      />

      {/* Multiline textarea with strict IME composition tracking */}
      <textarea
        ref={textareaRef}
        className="oc-clean-composer__textarea"
        value={draftText}
        onChange={(e) => handleDraftChange(e.target.value)}
        onCompositionStart={() => {
          isComposingRef.current = true;
        }}
        onCompositionEnd={() => {
          isComposingRef.current = false;
        }}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        rows={1}
      />

      {/* Composer toolbar */}
      <div className="oc-clean-composer__toolbar">
        {/* Leading tools: + attach menu and Shield */}
        <div className="oc-clean-composer__left-tools">
          <div style={{ position: "relative" }}>
            <button
              type="button"
              className="oc-clean-composer__tool-btn"
              onClick={() => {
                const nextOpen = !attachMenuOpen;
                setAttachMenuOpen(nextOpen);
                setModelMenuOpen(false);
                setThinkingMenuOpen(false);
                setSendOptionsOpen(false);
                if (nextOpen) {
                  onOpenAttachPopover?.();
                }
              }}
              title="添加附件或扩展操作 (+)"
              aria-label="Add attachment"
              aria-haspopup="true"
              aria-expanded={attachMenuOpen}
            >
              <span style={{ fontSize: 18, fontWeight: 500 }}>+</span>
            </button>

            {attachMenuOpen && (
              <div
                style={{
                  position: "absolute",
                  bottom: 36,
                  left: 0,
                  background: "var(--oc-card-bg)",
                  border: "1px solid var(--oc-border-strong)",
                  borderRadius: 10,
                  boxShadow: "var(--oc-shadow-md)",
                  padding: "6px 0",
                  zIndex: 100,
                  minWidth: 170,
                }}
                onMouseLeave={() => setAttachMenuOpen(false)}
              >
                <button
                  type="button"
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    width: "100%",
                    textAlign: "left",
                    padding: "7px 14px",
                    background: "none",
                    border: "none",
                    fontSize: 13,
                    color: "var(--oc-text-primary)",
                    cursor: "pointer",
                  }}
                  onClick={() => {
                    setAttachMenuOpen(false);
                    if (fileInputRef.current) {
                      fileInputRef.current.click();
                    } else if (onStageFiles) {
                      void onStageFiles();
                    }
                  }}
                >
                  <span>📷</span>
                  <span>照片与文件</span>
                </button>
                <button
                  type="button"
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    width: "100%",
                    textAlign: "left",
                    padding: "7px 14px",
                    background: "none",
                    border: "none",
                    fontSize: 13,
                    color: "var(--oc-text-primary)",
                    cursor: "pointer",
                  }}
                  onClick={() => {
                    setAttachMenuOpen(false);
                    if (onCreatePoll) {
                      onCreatePoll();
                    } else if (onSubmitPollOrTodo) {
                      void onSubmitPollOrTodo("/poll 投票主题 选项A 选项B");
                    } else if (onSubmitDirectMessage) {
                      void onSubmitDirectMessage("/poll 投票主题 选项A 选项B");
                    } else {
                      appendToDraftNonDestructively("/poll 投票主题 选项A 选项B");
                    }
                  }}
                >
                  <span>📊</span>
                  <span>发起投票 (Poll)</span>
                </button>
                <button
                  type="button"
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    width: "100%",
                    textAlign: "left",
                    padding: "7px 14px",
                    background: "none",
                    border: "none",
                    fontSize: 13,
                    color: "var(--oc-text-primary)",
                    cursor: "pointer",
                  }}
                  onClick={() => {
                    setAttachMenuOpen(false);
                    if (onCreateTodoList) {
                      onCreateTodoList();
                    } else if (onSubmitPollOrTodo) {
                      void onSubmitPollOrTodo("/todo 待办事项");
                    } else if (onSubmitDirectMessage) {
                      void onSubmitDirectMessage("/todo 待办事项");
                    } else {
                      appendToDraftNonDestructively("/todo 待办事项");
                    }
                  }}
                >
                  <span>☑️</span>
                  <span>待办清单 (Todo)</span>
                </button>
                <button
                  type="button"
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    width: "100%",
                    textAlign: "left",
                    padding: "7px 14px",
                    background: "none",
                    border: "none",
                    fontSize: 13,
                    color: "var(--oc-text-primary)",
                    cursor: "pointer",
                  }}
                  onClick={() => {
                    setAttachMenuOpen(false);
                    if (onCreateInvoice) {
                      onCreateInvoice();
                    } else if (onSubmitDirectMessage) {
                      void onSubmitDirectMessage("/invoice 账单明细 金额");
                    } else {
                      appendToDraftNonDestructively("/invoice 账单明细 金额");
                    }
                  }}
                >
                  <span>💳</span>
                  <span>收款账单 (Invoice)</span>
                </button>
              </div>
            )}
          </div>

          <button
            type="button"
            className="oc-clean-composer__tool-btn"
            onClick={cycleSecurityGate}
            title={`安全门策略: ${securityGate === "full" ? "全部放行 (Full)" : securityGate === "allowlist" ? "白名单放行 (Allowlist)" : "严格拦截 (Deny)"}`}
            aria-label="Security gate policy"
          >
            <ShieldGateIcon status={securityGate} size={18} />
          </button>
        </div>

        {/* Trailing tools: Model pill, Thinking pill, Mic, Send + Send options */}
        <div className="oc-clean-composer__right-tools">
          {/* Dynamic Model selector pill or explicit unconfigured indicator */}
          {availableModels.length > 0 ? (
            <div style={{ position: "relative" }}>
              <button
                type="button"
                className="oc-pill-select"
                disabled={disabled || !onChangeModel}
                onClick={() => {
                  setModelMenuOpen(!modelMenuOpen);
                  setThinkingMenuOpen(false);
                  setAttachMenuOpen(false);
                  setSendOptionsOpen(false);
                }}
                title="选择推理模型"
                aria-haspopup="true"
                aria-expanded={modelMenuOpen}
              >
                <span>{resolvedModelLabel}</span>
                <span style={{ fontSize: 9, opacity: 0.6 }}>▼</span>
              </button>

              {modelMenuOpen && onChangeModel && (
                <div
                  style={{
                    position: "absolute",
                    bottom: 34,
                    right: 0,
                    background: "var(--oc-card-bg)",
                    border: "1px solid var(--oc-border-strong)",
                    borderRadius: 8,
                    boxShadow: "var(--oc-shadow-md)",
                    padding: "4px 0",
                    zIndex: 100,
                    minWidth: 150,
                  }}
                  onMouseLeave={() => setModelMenuOpen(false)}
                >
                  {availableModels.map((item) => {
                    const isSelected = resolvedModelLabel === item.label || resolvedModelLabel === item.id;
                    return (
                      <button
                        key={item.id}
                        type="button"
                        style={{
                          display: "block",
                          width: "100%",
                          textAlign: "left",
                          padding: "6px 12px",
                          background: isSelected ? "var(--oc-card-hover)" : "none",
                          border: "none",
                          fontSize: 12.5,
                          fontWeight: isSelected ? 600 : 400,
                          color: "var(--oc-text-primary)",
                          cursor: "pointer",
                        }}
                        onClick={() => {
                          onChangeModel(item.label);
                          setModelMenuOpen(false);
                        }}
                      >
                        {item.label}
                      </button>
                    );
                  })}
                </div>
              )}
            </div>
          ) : (
            <button
              type="button"
              className="oc-pill-select"
              data-testid="oc-model-unconfigured"
              disabled
              title="未配置推理模型 (No models configured)"
              aria-label="No models configured"
              style={{ opacity: 0.6, cursor: "not-allowed" }}
            >
              <span>{model && model.trim().length > 0 ? model : "未配置模型"}</span>
            </button>
          )}

          {/* Thinking effort pill (only rendered when thinking level is configured via owner) */}
          {onChangeThinkingLevel && thinkingLevel && (
            <div style={{ position: "relative" }}>
              <button
                type="button"
                className="oc-pill-select"
                onClick={() => {
                  setThinkingMenuOpen(!thinkingMenuOpen);
                  setModelMenuOpen(false);
                  setAttachMenuOpen(false);
                  setSendOptionsOpen(false);
                }}
                title="设置思考推理深度 (Thinking Effort)"
                aria-haspopup="true"
                aria-expanded={thinkingMenuOpen}
              >
                <span>{thinkingLevel}</span>
                <span style={{ fontSize: 9, opacity: 0.6 }}>▼</span>
              </button>

              {thinkingMenuOpen && (
                <div
                  style={{
                    position: "absolute",
                    bottom: 34,
                    right: 0,
                    background: "var(--oc-card-bg)",
                    border: "1px solid var(--oc-border-strong)",
                    borderRadius: 8,
                    boxShadow: "var(--oc-shadow-md)",
                    padding: "4px 0",
                    zIndex: 100,
                    minWidth: 120,
                  }}
                  onMouseLeave={() => setThinkingMenuOpen(false)}
                >
                  {["Low", "Medium", "High", "Default"].map((level) => (
                    <button
                      key={level}
                      type="button"
                      style={{
                        display: "block",
                        width: "100%",
                        textAlign: "left",
                        padding: "6px 12px",
                        background: thinkingLevel === level ? "var(--oc-card-hover)" : "none",
                        border: "none",
                        fontSize: 12.5,
                        fontWeight: thinkingLevel === level ? 600 : 400,
                        color: "var(--oc-text-primary)",
                        cursor: "pointer",
                      }}
                      onClick={() => {
                        onChangeThinkingLevel?.(level);
                        setThinkingMenuOpen(false);
                      }}
                    >
                      {level}
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Microphone button */}
          <button
            type="button"
            className="oc-clean-composer__tool-btn"
            onClick={() => {
              void onTranscribeAudio?.();
            }}
            title={isTranscribing ? "正在语音输入..." : "语音听写 (Dictation)"}
            aria-label="Microphone"
            style={{ color: isTranscribing ? "#e74c3c" : undefined }}
          >
            <MicrophoneIcon size={16} />
          </button>

          {/* Send button + Silent/Scheduled Send options */}
          <div style={{ position: "relative", display: "inline-flex", alignItems: "center", gap: 4 }}>
            <button
              type="button"
              className="oc-clean-composer__send-btn"
              disabled={!canSend}
              onClick={() => {
                void onSubmit();
              }}
              title="发送消息 (Send)"
              aria-label="Send"
            >
              <SendArrowUpIcon size={16} />
            </button>
            <button
              type="button"
              className="oc-clean-composer__tool-btn"
              disabled={!canSend}
              onClick={() => {
                setSendOptionsOpen((open) => !open);
                setAttachMenuOpen(false);
                setModelMenuOpen(false);
                setThinkingMenuOpen(false);
              }}
              title="发送选项: 静音发送 / 定时发送 (Send options)"
              aria-label="Send options"
              aria-haspopup="true"
              aria-expanded={sendOptionsOpen}
              style={{ padding: "2px 6px", fontSize: 11 }}
            >
              ▾
            </button>

            {sendOptionsOpen && (
              <div
                role="menu"
                aria-label="Send options"
                style={{
                  position: "absolute",
                  bottom: 36,
                  right: 0,
                  background: "var(--oc-card-bg)",
                  border: "1px solid var(--oc-border-strong)",
                  borderRadius: 10,
                  boxShadow: "var(--oc-shadow-md)",
                  padding: "6px 0",
                  zIndex: 100,
                  minWidth: 190,
                }}
              >
                <button
                  type="button"
                  role="menuitem"
                  style={{
                    display: "block",
                    width: "100%",
                    textAlign: "left",
                    padding: "7px 14px",
                    background: "none",
                    border: "none",
                    fontSize: 13,
                    color: "var(--oc-text-primary)",
                    cursor: "pointer",
                  }}
                  onClick={handleSilentSend}
                >
                  🔕 静音发送 (Send Without Sound)
                </button>
                <button
                  type="button"
                  role="menuitem"
                  style={{
                    display: "block",
                    width: "100%",
                    textAlign: "left",
                    padding: "7px 14px",
                    background: "none",
                    border: "none",
                    fontSize: 13,
                    color: "var(--oc-text-primary)",
                    cursor: "pointer",
                  }}
                  onClick={() => {
                    setSendOptionsOpen(false);
                    setScheduleError(null);
                    setScheduleValue(formatDefaultScheduleValue());
                    setScheduleDialogOpen(true);
                  }}
                >
                  🕒 定时发送 (Schedule Message)
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {scheduleDialogOpen && (
        <div
          aria-label="Schedule message"
          className="sand-prompt-schedule-popover"
          role="dialog"
          style={{
            marginTop: 8,
            padding: "10px 12px",
            borderRadius: 10,
            border: "1px solid var(--oc-border-strong)",
            background: "var(--oc-card-bg)",
          }}
        >
          <label style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 12.5 }}>
            <span>定时发送时间</span>
            <input
              aria-label="Scheduled send time"
              className="sand-prompt-schedule-input"
              type="datetime-local"
              value={scheduleValue}
              onChange={(event) => {
                setScheduleValue(event.currentTarget.value);
                setScheduleError(null);
              }}
            />
          </label>
          {scheduleError != null && (
            <p role="alert" style={{ margin: "6px 0 0", fontSize: 12, color: "#e74c3c" }}>
              {scheduleError}
            </p>
          )}
          <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
            <button
              type="button"
              className="sand-prompt-schedule-confirm"
              disabled={!canSend}
              onClick={handleConfirmSchedule}
            >
              Schedule
            </button>
            <button
              type="button"
              className="sand-prompt-schedule-cancel"
              onClick={() => {
                setScheduleDialogOpen(false);
                setScheduleError(null);
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
