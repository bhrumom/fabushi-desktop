import React, { useState, useRef, useEffect } from "react";
import {
  ShieldGateIcon,
  MicrophoneIcon,
  SendArrowUpIcon,
} from "./OpenClawIcons";

export interface OpenClawCleanComposerProps {
  draft: string | { prompt: string; attachments?: any[]; [key: string]: any };
  onChange: (value: any) => void;
  onSubmit: () => void;
  disabled?: boolean;
  placeholder?: string;
  onStageFiles?: (files?: any) => void;
  onCreatePoll?: () => void;
  onCreateTodoList?: () => void;
  onCreateInvoice?: () => void;
  securityGate?: "deny" | "allowlist" | "full";
  onChangeSecurityGate?: (next: "deny" | "allowlist" | "full") => void;
  model?: string;
  onChangeModel?: (model: string) => void;
  thinkingLevel?: string;
  onChangeThinkingLevel?: (level: string) => void;
  onTranscribeAudio?: (audio?: any, mimeType?: any, language?: any) => any;
  isTranscribing?: boolean;
}

const AVAILABLE_MODELS = [
  { id: "gpt-6-astra", label: "GPT-6 Astra" },
  { id: "gemini-1.5-pro", label: "Gemini 1.5 Pro" },
  { id: "claude-3.5-sonnet", label: "Claude 3.5 Sonnet" },
  { id: "sand-local", label: "Local Sand" },
];

const THINKING_LEVELS = ["Low", "Medium", "High", "Default"];

export function OpenClawCleanComposer({
  draft,
  onChange,
  onSubmit,
  disabled = false,
  placeholder = "给 Assistant 发消息",
  onStageFiles,
  onCreatePoll,
  onCreateTodoList,
  onCreateInvoice,
  securityGate = "allowlist",
  onChangeSecurityGate,
  model = "GPT-6 Astra",
  onChangeModel,
  thinkingLevel = "Medium",
  onChangeThinkingLevel,
  onTranscribeAudio,
  isTranscribing = false,
}: OpenClawCleanComposerProps) {
  const [attachMenuOpen, setAttachMenuOpen] = useState(false);
  const [modelMenuOpen, setModelMenuOpen] = useState(false);
  const [thinkingMenuOpen, setThinkingMenuOpen] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const draftText = typeof draft === "string" ? draft : (draft?.prompt ?? "");

  const handleDraftChange = (nextText: string) => {
    if (typeof draft === "string") {
      onChange(nextText);
    } else {
      onChange({ ...draft, prompt: nextText });
    }
  };

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 180)}px`;
    }
  }, [draftText]);

  const handleKeyDown = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      if (!disabled && draftText.trim().length > 0) {
        onSubmit();
      }
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

      {/* Multiline textarea */}
      <textarea
        ref={textareaRef}
        className="oc-clean-composer__textarea"
        value={draftText}
        onChange={(e) => handleDraftChange(e.target.value)}
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
                setAttachMenuOpen(!attachMenuOpen);
                setModelMenuOpen(false);
                setThinkingMenuOpen(false);
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
                      onStageFiles();
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
                    if (onCreatePoll) onCreatePoll();
                    else handleDraftChange(`${draftText ? `${draftText}\n` : ""}/poll 投票主题 选项A 选项B`);
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
                    if (onCreateTodoList) onCreateTodoList();
                    else handleDraftChange(`${draftText ? `${draftText}\n` : ""}/todo 待办事项`);
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
                    if (onCreateInvoice) onCreateInvoice();
                    else handleDraftChange(`${draftText ? `${draftText}\n` : ""}/invoice 账单明细 金额`);
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

        {/* Trailing tools: Model pill, Thinking pill, Mic, Send */}
        <div className="oc-clean-composer__right-tools">
          {/* Model selector pill */}
          <div style={{ position: "relative" }}>
            <button
              type="button"
              className="oc-pill-select"
              onClick={() => {
                setModelMenuOpen(!modelMenuOpen);
                setThinkingMenuOpen(false);
                setAttachMenuOpen(false);
              }}
              title="选择推理模型"
              aria-haspopup="true"
              aria-expanded={modelMenuOpen}
            >
              <span>{model}</span>
              <span style={{ fontSize: 9, opacity: 0.6 }}>▼</span>
            </button>

            {modelMenuOpen && (
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
                {AVAILABLE_MODELS.map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    style={{
                      display: "block",
                      width: "100%",
                      textAlign: "left",
                      padding: "6px 12px",
                      background: model === item.label ? "var(--oc-card-hover)" : "none",
                      border: "none",
                      fontSize: 12.5,
                      fontWeight: model === item.label ? 600 : 400,
                      color: "var(--oc-text-primary)",
                      cursor: "pointer",
                    }}
                    onClick={() => {
                      onChangeModel?.(item.label);
                      setModelMenuOpen(false);
                    }}
                  >
                    {item.label}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Thinking effort pill */}
          <div style={{ position: "relative" }}>
            <button
              type="button"
              className="oc-pill-select"
              onClick={() => {
                setThinkingMenuOpen(!thinkingMenuOpen);
                setModelMenuOpen(false);
                setAttachMenuOpen(false);
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
                {THINKING_LEVELS.map((level) => (
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

          {/* Microphone button */}
          <button
            type="button"
            className="oc-clean-composer__tool-btn"
            onClick={() => onTranscribeAudio?.()}
            title={isTranscribing ? "正在语音输入..." : "语音听写 (Dictation)"}
            aria-label="Microphone"
            style={{ color: isTranscribing ? "#e74c3c" : undefined }}
          >
            <MicrophoneIcon size={16} />
          </button>

          {/* Send button (up arrow circle) */}
          <button
            type="button"
            className="oc-clean-composer__send-btn"
            disabled={disabled || draftText.trim().length === 0}
            onClick={onSubmit}
            title="发送消息 (Send)"
            aria-label="Send"
          >
            <SendArrowUpIcon size={16} />
          </button>
        </div>
      </div>
    </div>
  );
}
