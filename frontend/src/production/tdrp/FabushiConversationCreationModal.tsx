import React, { useState } from "react";
import { User, Users, Radio, Bot, Sparkles, X } from "lucide-react";

export type ConversationKind = "direct" | "group" | "channel" | "agent" | "hybrid";

export interface CreateConversationData {
  readonly kind: ConversationKind;
  readonly title: string;
  readonly description?: string;
  readonly memberIds?: readonly string[];
}

export interface FabushiConversationCreationModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onCreate: (data: CreateConversationData) => void;
}

const KIND_OPTIONS = [
  { id: "direct" as const, label: "私聊", desc: "与好友或同事直接沟通", icon: User },
  { id: "group" as const, label: "群组", desc: "多方参与的高效讨论群", icon: Users },
  { id: "channel" as const, label: "频道", desc: "单向广播与公开资讯发布", icon: Radio },
  { id: "agent" as const, label: "Agent 会话", desc: "专属智能体自动化协作", icon: Bot },
  { id: "hybrid" as const, label: "混合协同室", desc: "人类团队与多智能体混合空间", icon: Sparkles },
];

export const FabushiConversationCreationModal: React.FC<FabushiConversationCreationModalProps> = ({
  isOpen,
  onClose,
  onCreate,
}) => {
  const [selectedKind, setSelectedKind] = useState<ConversationKind>("direct");
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");

  if (!isOpen) return null;

  const handleConfirm = () => {
    const trimmedTitle = title.trim();
    if (!trimmedTitle) return;
    onCreate({
      kind: selectedKind,
      title: trimmedTitle,
      description: description.trim() || undefined,
    });
    setTitle("");
    setDescription("");
    onClose();
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      data-testid="fabushi-conversation-creation-modal"
      onClick={onClose}
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: "rgba(0, 0, 0, 0.65)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 9999,
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          background: "var(--cursor-bg-editor, #242429)",
          padding: "24px",
          borderRadius: "16px",
          width: "420px",
          maxWidth: "92vw",
          color: "var(--fabushi-text-primary, #ffffff)",
          boxShadow: "0 12px 36px rgba(0,0,0,0.5)",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "16px" }}>
          <h3 style={{ margin: 0, fontSize: "16px", fontWeight: 700 }}>新建会话</h3>
          <button
            type="button"
            onClick={onClose}
            aria-label="关闭"
            style={{ background: "none", border: 0, color: "var(--fabushi-text-secondary, #888)", cursor: "pointer" }}
          >
            <X size={18} />
          </button>
        </div>

        {/* Kind selector */}
        <div style={{ display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: "6px", marginBottom: "16px" }}>
          {KIND_OPTIONS.map((opt) => {
            const isSelected = selectedKind === opt.id;
            const Icon = opt.icon;
            return (
              <button
                key={opt.id}
                type="button"
                data-testid={`fabushi-create-kind-${opt.id}`}
                onClick={() => setSelectedKind(opt.id)}
                style={{
                  display: "flex",
                  flexDirection: "column",
                  alignItems: "center",
                  gap: "4px",
                  padding: "8px 4px",
                  borderRadius: "8px",
                  border: isSelected
                    ? "1.5px solid var(--fabushi-accent, #1084fe)"
                    : "1px solid var(--fabushi-border-weak, rgba(255,255,255,0.08))",
                  background: isSelected
                    ? "var(--fabushi-accent-subtle, rgba(16, 132, 254, 0.15))"
                    : "var(--cursor-bg-chrome, #1c1c21)",
                  color: isSelected ? "var(--fabushi-accent, #1084fe)" : "#aaa",
                  cursor: "pointer",
                  fontSize: "11px",
                  fontWeight: isSelected ? 600 : 400,
                  transition: "all 0.12s ease",
                }}
              >
                <Icon size={18} strokeWidth={isSelected ? 2.2 : 1.8} />
                <span>{opt.label}</span>
              </button>
            );
          })}
        </div>

        {/* Inputs */}
        <div style={{ marginBottom: "12px" }}>
          <label style={{ display: "block", fontSize: "12px", marginBottom: "6px", color: "var(--fabushi-text-secondary, rgba(255,255,255,0.7))" }}>
            名称
          </label>
          <input
            type="text"
            data-testid="fabushi-create-title-input"
            placeholder={
              selectedKind === "direct"
                ? "联系人姓名或用户名"
                : selectedKind === "channel"
                ? "频道名称"
                : selectedKind === "agent"
                ? "智能体名称"
                : "群组名称"
            }
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            autoFocus
            style={{
              width: "100%",
              padding: "8px 10px",
              borderRadius: "8px",
              border: "1px solid var(--fabushi-border-weak, #444)",
              background: "var(--cursor-bg-chrome, #18181c)",
              color: "#fff",
              fontSize: "13px",
            }}
          />
        </div>

        <div style={{ marginBottom: "18px" }}>
          <label style={{ display: "block", fontSize: "12px", marginBottom: "6px", color: "var(--fabushi-text-secondary, rgba(255,255,255,0.7))" }}>
            描述或备注（选填）
          </label>
          <input
            type="text"
            data-testid="fabushi-create-desc-input"
            placeholder="简要说明此会话的目标或主题"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            style={{
              width: "100%",
              padding: "8px 10px",
              borderRadius: "8px",
              border: "1px solid var(--fabushi-border-weak, #444)",
              background: "var(--cursor-bg-chrome, #18181c)",
              color: "#fff",
              fontSize: "13px",
            }}
          />
        </div>

        {/* Buttons */}
        <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
          <button
            type="button"
            onClick={onClose}
            style={{
              padding: "7px 16px",
              borderRadius: "8px",
              border: "1px solid var(--fabushi-border-weak, #555)",
              background: "transparent",
              color: "#ccc",
              fontSize: "13px",
              cursor: "pointer",
            }}
          >
            取消
          </button>
          <button
            type="button"
            data-testid="fabushi-create-confirm-btn"
            disabled={!title.trim()}
            onClick={handleConfirm}
            style={{
              padding: "7px 20px",
              borderRadius: "8px",
              border: 0,
              background: title.trim() ? "var(--fabushi-accent, #1084fe)" : "#555",
              color: "#ffffff",
              fontSize: "13px",
              fontWeight: 600,
              cursor: title.trim() ? "pointer" : "not-allowed",
              transition: "all 0.15s ease",
            }}
          >
            创建
          </button>
        </div>
      </div>
    </div>
  );
};
