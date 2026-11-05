import React, { useState } from "react";
import { FolderPlus, X } from "lucide-react";

export interface ChatFilterModel {
  readonly id: string;
  readonly title: string;
  readonly iconEmoji?: string;
  readonly colorIndex?: number;
  readonly flags: number;
  readonly alwaysPeers?: readonly string[];
  readonly neverPeers?: readonly string[];
  readonly pinnedPeers?: readonly string[];
}

export const CHAT_FILTER_FLAGS = {
  Contacts: 1 << 0,      // 0x01
  NonContacts: 1 << 1,   // 0x02
  Groups: 1 << 2,        // 0x04
  Channels: 1 << 3,      // 0x08
  Bots: 1 << 4,          // 0x10
  NoMuted: 1 << 5,       // 0x20
  NoRead: 1 << 6,        // 0x40
  NoArchived: 1 << 7,    // 0x80
  Chatlist: 1 << 8,      // 0x100
  HasMyLinks: 1 << 9,    // 0x200
  StaticTitle: 1 << 10,  // 0x400
  NewChats: 1 << 11,     // 0x800
  ExistingChats: 1 << 12,// 0x1000
} as const;

export interface TdrpChatFilterEditModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onSave: (filter: ChatFilterModel) => void;
}

export const TdrpChatFilterEditModal: React.FC<TdrpChatFilterEditModalProps> = ({
  isOpen,
  onClose,
  onSave,
}) => {
  const [title, setTitle] = useState("");
  const [colorIndex, setColorIndex] = useState(0);
  const [flags, setFlags] = useState<number>(
    CHAT_FILTER_FLAGS.Contacts | CHAT_FILTER_FLAGS.Groups
  );
  const [alwaysInput, setAlwaysInput] = useState("");
  const [neverInput, setNeverInput] = useState("");

  if (!isOpen) return null;

  const toggleFlag = (flag: number) => {
    setFlags((prev) => (prev & flag ? prev & ~flag : prev | flag));
  };

  const handleSave = () => {
    const trimmedTitle = title.trim();
    if (!trimmedTitle) return;
    const filter: ChatFilterModel = {
      id: `filter_${Date.now()}`,
      title: trimmedTitle,
      colorIndex,
      flags,
      alwaysPeers: alwaysInput ? alwaysInput.split(",").map((s) => s.trim()).filter(Boolean) : [],
      neverPeers: neverInput ? neverInput.split(",").map((s) => s.trim()).filter(Boolean) : [],
      pinnedPeers: [],
    };
    onSave(filter);
    setTitle("");
    onClose();
  };

  const flagOptions = [
    { flag: CHAT_FILTER_FLAGS.Contacts, label: "联系人 (Contacts)" },
    { flag: CHAT_FILTER_FLAGS.NonContacts, label: "非联系人 (Non-contacts)" },
    { flag: CHAT_FILTER_FLAGS.Groups, label: "群组 (Groups)" },
    { flag: CHAT_FILTER_FLAGS.Channels, label: "频道 (Channels)" },
    { flag: CHAT_FILTER_FLAGS.Bots, label: "智能体与机器人 (Bots)" },
    { flag: CHAT_FILTER_FLAGS.NoMuted, label: "排除免打扰 (No Muted)" },
    { flag: CHAT_FILTER_FLAGS.NoRead, label: "仅保留未读 (No Read)" },
    { flag: CHAT_FILTER_FLAGS.NoArchived, label: "排除已归档 (No Archived)" },
    { flag: CHAT_FILTER_FLAGS.NewChats, label: "新会话 (New Chats)" },
    { flag: CHAT_FILTER_FLAGS.ExistingChats, label: "已有会话 (Existing Chats)" },
    { flag: CHAT_FILTER_FLAGS.Chatlist, label: "可分享对话夹 (Chatlist)" },
  ];

  return (
    <div
      role="dialog"
      aria-modal="true"
      data-testid="tdrp-chat-filter-modal"
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
          background: "var(--sand-surface-backdrop, #242429)",
          padding: "20px 24px",
          borderRadius: "16px",
          width: "380px",
          maxHeight: "88vh",
          overflowY: "auto",
          color: "var(--sand-text-primary, #ffffff)",
          boxShadow: "0 12px 36px rgba(0,0,0,0.45)",
          border: "1px solid var(--sand-border-weak, rgba(128,128,128,0.2))",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "14px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <FolderPlus size={18} color="#1084fe" />
            <h3 style={{ margin: 0, fontSize: "15px", fontWeight: 700 }}>新建对话分组 (Chat Filter)</h3>
          </div>
          <button type="button" onClick={onClose} aria-label="关闭" style={{ background: "none", border: 0, color: "#888", cursor: "pointer" }}>
            <X size={16} />
          </button>
        </div>

        <div style={{ fontSize: "12px", marginBottom: "6px", fontWeight: 600 }}>分组名称 (Title)</div>
        <input
          type="text"
          data-testid="tdrp-filter-title-input"
          maxLength={16}
          placeholder="例如: 重点项目 / 工作协同"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          style={{
            width: "100%",
            padding: "7px 10px",
            borderRadius: "8px",
            border: "1px solid var(--sand-border-weak, #444)",
            background: "var(--sand-surface-muted, rgba(128,128,128,0.1))",
            color: "inherit",
            fontSize: "12px",
            marginBottom: "12px",
            boxSizing: "border-box",
          }}
        />

        <div style={{ fontSize: "12px", marginBottom: "6px", fontWeight: 600 }}>颜色标记 (Color Tag 0..7)</div>
        <div style={{ display: "flex", gap: "6px", marginBottom: "14px" }}>
          {[0, 1, 2, 3, 4, 5, 6, 7].map((idx) => {
            const colors = ["#1084fe", "#00c972", "#ff9800", "#f44336", "#9c27b0", "#00bcd4", "#795548", "#607d8b"];
            const isSel = colorIndex === idx;
            return (
              <button
                key={idx}
                type="button"
                onClick={() => setColorIndex(idx)}
                style={{
                  width: "22px",
                  height: "22px",
                  borderRadius: "50%",
                  background: colors[idx],
                  border: isSel ? "2px solid #ffffff" : "2px solid transparent",
                  outline: isSel ? `2px solid ${colors[idx]}` : "none",
                  cursor: "pointer",
                }}
              />
            );
          })}
        </div>

        <div style={{ fontSize: "12px", marginBottom: "8px", fontWeight: 600 }}>包含/排除规则 (Upstream Flags)</div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr", gap: "6px", marginBottom: "14px" }}>
          {flagOptions.map((opt) => (
            <label
              key={opt.flag}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "8px",
                fontSize: "12px",
                cursor: "pointer",
              }}
            >
              <input
                type="checkbox"
                checked={Boolean(flags & opt.flag)}
                onChange={() => toggleFlag(opt.flag)}
              />
              <span>{opt.label}</span>
            </label>
          ))}
        </div>

        <div style={{ fontSize: "12px", marginBottom: "4px", fontWeight: 600 }}>始终包含 (Always Peers)</div>
        <input
          type="text"
          placeholder="以逗号分隔 Peer ID"
          value={alwaysInput}
          onChange={(e) => setAlwaysInput(e.target.value)}
          style={{
            width: "100%",
            padding: "6px 8px",
            borderRadius: "6px",
            border: "1px solid var(--sand-border-weak, #444)",
            background: "var(--sand-surface-muted, rgba(128,128,128,0.1))",
            color: "inherit",
            fontSize: "11px",
            marginBottom: "10px",
            boxSizing: "border-box",
          }}
        />

        <div style={{ fontSize: "12px", marginBottom: "4px", fontWeight: 600 }}>从不包含 (Never Peers)</div>
        <input
          type="text"
          placeholder="以逗号分隔 Peer ID"
          value={neverInput}
          onChange={(e) => setNeverInput(e.target.value)}
          style={{
            width: "100%",
            padding: "6px 8px",
            borderRadius: "6px",
            border: "1px solid var(--sand-border-weak, #444)",
            background: "var(--sand-surface-muted, rgba(128,128,128,0.1))",
            color: "inherit",
            fontSize: "11px",
            marginBottom: "16px",
            boxSizing: "border-box",
          }}
        />

        <button
          type="button"
          data-testid="tdrp-filter-save-btn"
          onClick={handleSave}
          style={{
            width: "100%",
            padding: "8px",
            borderRadius: "8px",
            border: 0,
            background: "#1084fe",
            color: "#fff",
            fontWeight: 700,
            fontSize: "13px",
            cursor: "pointer",
          }}
        >
          保存分组
        </button>
      </div>
    </div>
  );
};
