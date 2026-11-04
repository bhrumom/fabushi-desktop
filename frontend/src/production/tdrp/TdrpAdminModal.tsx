import React from "react";
import { Shield, X } from "lucide-react";

export interface TdrpAdminModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly targetUserId: string;
  readonly onTargetUserIdChange: (id: string) => void;
  readonly banDuration: string;
  readonly onBanDurationChange: (duration: string) => void;
  readonly adminStatus: string | null;
  readonly onAdminAction: (action: "ban" | "restrict") => void;
}

export const TdrpAdminModal: React.FC<TdrpAdminModalProps> = ({
  isOpen,
  onClose,
  targetUserId,
  onTargetUserIdChange,
  banDuration,
  onBanDurationChange,
  adminStatus,
  onAdminAction,
}) => {
  if (!isOpen) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      data-testid="tdrp-admin-modal"
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
          padding: "20px 24px",
          borderRadius: "14px",
          width: "360px",
          color: "var(--fabushi-text-primary, #ffffff)",
          boxShadow: "0 10px 30px rgba(0,0,0,0.5)",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "14px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <Shield size={18} color="var(--fabushi-accent, #1084fe)" />
            <h3 style={{ margin: 0, fontSize: "15px", fontWeight: 600 }}>群管与成员权限</h3>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="关闭"
            style={{ background: "none", border: 0, color: "var(--fabushi-text-secondary, #888)", cursor: "pointer" }}
          >
            <X size={16} />
          </button>
        </div>
        <div style={{ fontSize: "12px", marginBottom: "8px" }}>目标用户 ID</div>
        <input
          type="text"
          data-testid="tdrp-admin-user-input"
          value={targetUserId}
          onChange={(e) => onTargetUserIdChange(e.target.value)}
          style={{
            width: "100%",
            padding: "6px 8px",
            borderRadius: "6px",
            border: "1px solid var(--sand-border-weak, #444)",
            background: "var(--cursor-bg-chrome, #18181c)",
            color: "#fff",
            fontSize: "12px",
            marginBottom: "12px",
          }}
        />
        <div style={{ fontSize: "12px", marginBottom: "8px" }}>时长</div>
        <select
          value={banDuration}
          onChange={(e) => onBanDurationChange(e.target.value)}
          style={{
            width: "100%",
            padding: "6px",
            borderRadius: "6px",
            border: "1px solid var(--sand-border-weak, #444)",
            background: "var(--cursor-bg-chrome, #18181c)",
            color: "#fff",
            fontSize: "12px",
            marginBottom: "16px",
          }}
        >
          <option value="3600">1 小时</option>
          <option value="86400">1 天</option>
          <option value="604800">1 周</option>
          <option value="0">永久</option>
        </select>
        {adminStatus && <div style={{ fontSize: "11px", color: "#69f0ae", marginBottom: "12px" }}>{adminStatus}</div>}
        <div style={{ display: "flex", gap: "8px" }}>
          <button
            type="button"
            data-testid="tdrp-admin-restrict-btn"
            onClick={() => onAdminAction("restrict")}
            style={{
              flex: 1,
              padding: "8px",
              borderRadius: "6px",
              border: 0,
              background: "#ff9800",
              color: "#fff",
              fontWeight: 600,
              fontSize: "12px",
              cursor: "pointer",
            }}
          >
            禁言限制
          </button>
          <button
            type="button"
            data-testid="tdrp-admin-ban-btn"
            onClick={() => onAdminAction("ban")}
            style={{
              flex: 1,
              padding: "8px",
              borderRadius: "6px",
              border: 0,
              background: "#f44336",
              color: "#fff",
              fontWeight: 600,
              fontSize: "12px",
              cursor: "pointer",
            }}
          >
            移出并封禁
          </button>
        </div>
      </div>
    </div>
  );
};
