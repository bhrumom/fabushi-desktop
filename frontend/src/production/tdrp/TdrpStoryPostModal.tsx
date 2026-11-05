import React, { useState } from "react";
import { X } from "lucide-react";

export interface StoryPublishPayload {
  readonly content: string;
  readonly privacy: "all" | "contacts" | "closeFriends" | "selected";
  readonly expirePeriod: number;
  readonly pinnedToTop: boolean;
  readonly noforwards: boolean;
}

export interface TdrpStoryPostModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onPublish: (payload: StoryPublishPayload) => void;
}

export const TdrpStoryPostModal: React.FC<TdrpStoryPostModalProps> = ({
  isOpen,
  onClose,
  onPublish,
}) => {
  const [content, setContent] = useState("");
  const [privacy, setPrivacy] = useState<"all" | "contacts" | "closeFriends" | "selected">("all");
  const [expirePeriod, setExpirePeriod] = useState<number>(24);
  const [pinnedToTop, setPinnedToTop] = useState(false);
  const [noforwards, setNoforwards] = useState(false);

  if (!isOpen) return null;

  const handleConfirm = () => {
    const trimmed = content.trim();
    if (!trimmed) return;
    onPublish({
      content: trimmed,
      privacy,
      expirePeriod,
      pinnedToTop,
      noforwards,
    });
    setContent("");
    onClose();
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      data-testid="tdrp-post-story-modal"
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
          padding: "20px",
          borderRadius: "14px",
          width: "320px",
          color: "var(--sand-text-primary, #ffffff)",
          border: "1px solid var(--sand-border-weak, rgba(128,128,128,0.2))",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "10px" }}>
          <h4 style={{ margin: 0, fontSize: "14px", fontWeight: 700 }}>发布动态 (Post Story)</h4>
          <button type="button" onClick={onClose} aria-label="关闭" style={{ background: "none", border: 0, color: "#888", cursor: "pointer" }}>
            <X size={16} />
          </button>
        </div>

        <textarea
          data-testid="tdrp-post-story-input"
          rows={3}
          placeholder="分享此时此刻的想法..."
          value={content}
          onChange={(e) => setContent(e.target.value)}
          style={{
            width: "100%",
            padding: "8px",
            borderRadius: "8px",
            border: "1px solid var(--sand-border-weak, #444)",
            background: "var(--sand-surface-muted, rgba(128,128,128,0.1))",
            color: "inherit",
            fontSize: "12px",
            resize: "none",
            marginBottom: "10px",
            boxSizing: "border-box",
          }}
        />

        {/* Privacy selector */}
        <div style={{ fontSize: "11px", fontWeight: 600, marginBottom: "4px" }}>可见范围 (Privacy)</div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "4px", marginBottom: "10px" }}>
          {[
            { id: "all" as const, label: "所有人" },
            { id: "contacts" as const, label: "联系人" },
            { id: "closeFriends" as const, label: "挚友" },
            { id: "selected" as const, label: "指定人" },
          ].map((p) => (
            <button
              key={p.id}
              type="button"
              onClick={() => setPrivacy(p.id)}
              style={{
                padding: "4px 2px",
                borderRadius: "6px",
                border: privacy === p.id ? "1px solid #1084fe" : "1px solid var(--sand-border-weak, #444)",
                background: privacy === p.id ? "rgba(16, 132, 254, 0.15)" : "transparent",
                color: privacy === p.id ? "#1084fe" : "inherit",
                fontSize: "10px",
                cursor: "pointer",
              }}
            >
              {p.label}
            </button>
          ))}
        </div>

        {/* Expire Period */}
        <div style={{ fontSize: "11px", fontWeight: 600, marginBottom: "4px" }}>有效期 (Period)</div>
        <div style={{ display: "flex", gap: "6px", marginBottom: "10px" }}>
          {[6, 12, 24, 48].map((hrs) => (
            <button
              key={hrs}
              type="button"
              onClick={() => setExpirePeriod(hrs)}
              style={{
                flex: 1,
                padding: "4px",
                borderRadius: "6px",
                border: expirePeriod === hrs ? "1px solid #1084fe" : "1px solid var(--sand-border-weak, #444)",
                background: expirePeriod === hrs ? "rgba(16, 132, 254, 0.15)" : "transparent",
                color: expirePeriod === hrs ? "#1084fe" : "inherit",
                fontSize: "10px",
                cursor: "pointer",
              }}
            >
              {hrs}小时
            </button>
          ))}
        </div>

        {/* Checkbox rules */}
        <div style={{ display: "flex", flexDirection: "column", gap: "6px", marginBottom: "12px", fontSize: "11px" }}>
          <label style={{ display: "flex", alignItems: "center", gap: "6px", cursor: "pointer" }}>
            <input type="checkbox" checked={pinnedToTop} onChange={(e) => setPinnedToTop(e.target.checked)} />
            <span>置顶到主页 (pinnedToTop)</span>
          </label>
          <label style={{ display: "flex", alignItems: "center", gap: "6px", cursor: "pointer" }}>
            <input type="checkbox" checked={noforwards} onChange={(e) => setNoforwards(e.target.checked)} />
            <span>禁止转发与保存 (noforwards)</span>
          </label>
        </div>

        <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
          <button
            type="button"
            onClick={onClose}
            style={{ padding: "5px 12px", borderRadius: "6px", border: "1px solid #555", background: "transparent", color: "inherit", cursor: "pointer", fontSize: "12px" }}
          >
            取消
          </button>
          <button
            type="button"
            data-testid="tdrp-post-story-confirm"
            onClick={handleConfirm}
            style={{ padding: "5px 14px", borderRadius: "6px", border: 0, background: "#1084fe", color: "#fff", fontWeight: 600, cursor: "pointer", fontSize: "12px" }}
          >
            发布
          </button>
        </div>
      </div>
    </div>
  );
};
