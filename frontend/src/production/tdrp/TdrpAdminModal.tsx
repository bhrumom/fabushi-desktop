import React, { useState } from "react";
import { Shield, X, UserCheck, Lock } from "lucide-react";

export interface TdrpAdminModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly targetUserId: string;
  readonly onTargetUserIdChange: (id: string) => void;
  readonly banDuration: string;
  readonly onBanDurationChange: (duration: string) => void;
  readonly adminStatus: string | null;
  readonly onAdminAction: (action: "ban" | "restrict", payload?: { rank?: string; rights?: number; restrictions?: number }) => void;
}

// Upstream data_chat_participant_status.h ChatAdminRight
export const CHAT_ADMIN_RIGHTS = {
  ChangeInfo: 1 << 0,
  PostMessages: 1 << 1,
  EditMessages: 1 << 2,
  DeleteMessages: 1 << 3,
  BanUsers: 1 << 4,
  InviteByLinkOrAdd: 1 << 5,
  PinMessages: 1 << 7,
  AddAdmins: 1 << 9,
  Anonymous: 1 << 10,
  ManageCall: 1 << 11,
  ManageTopics: 1 << 13,
  ManageDirect: 1 << 17,
} as const;

// Upstream data_chat_participant_status.h ChatRestriction
export const CHAT_RESTRICTIONS = {
  SendMessages: 1 << 0,
  SendStickers: 1 << 3,
  SendGifs: 1 << 4,
  SendGames: 1 << 5,
  SendInline: 1 << 6,
  EmbedLinks: 1 << 7,
  SendPolls: 1 << 8,
  ChangeInfo: 1 << 10,
  InviteUsers: 1 << 15,
  PinMessages: 1 << 17,
  CreateTopics: 1 << 18,
  SendPhotos: 1 << 19,
  SendVideos: 1 << 20,
  SendVideoMessages: 1 << 21,
  SendMusic: 1 << 22,
  SendVoiceMessages: 1 << 23,
  SendFiles: 1 << 24,
} as const;

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
  const [activeTab, setActiveTab] = useState<"restrict" | "admin">("restrict");
  const [customRank, setCustomRank] = useState("");
  const [adminRights, setAdminRights] = useState<number>(
    CHAT_ADMIN_RIGHTS.DeleteMessages | CHAT_ADMIN_RIGHTS.BanUsers | CHAT_ADMIN_RIGHTS.InviteByLinkOrAdd
  );
  const [restrictions, setRestrictions] = useState<number>(
    CHAT_RESTRICTIONS.SendMessages | CHAT_RESTRICTIONS.SendPhotos | CHAT_RESTRICTIONS.SendFiles
  );

  if (!isOpen) return null;

  const toggleRight = (val: number) => {
    setAdminRights((prev) => (prev & val ? prev & ~val : prev | val));
  };

  const toggleRestriction = (val: number) => {
    setRestrictions((prev) => (prev & val ? prev & ~val : prev | val));
  };

  const adminRightOptions = [
    { bit: CHAT_ADMIN_RIGHTS.ChangeInfo, label: "修改群资料 (ChangeInfo)" },
    { bit: CHAT_ADMIN_RIGHTS.PostMessages, label: "发布消息 (PostMessages)" },
    { bit: CHAT_ADMIN_RIGHTS.EditMessages, label: "编辑消息 (EditMessages)" },
    { bit: CHAT_ADMIN_RIGHTS.DeleteMessages, label: "删除消息 (DeleteMessages)" },
    { bit: CHAT_ADMIN_RIGHTS.BanUsers, label: "封禁成员 (BanUsers)" },
    { bit: CHAT_ADMIN_RIGHTS.InviteByLinkOrAdd, label: "邀请成员 (InviteByLinkOrAdd)" },
    { bit: CHAT_ADMIN_RIGHTS.PinMessages, label: "置顶消息 (PinMessages)" },
    { bit: CHAT_ADMIN_RIGHTS.ManageTopics, label: "管理话题 (ManageTopics)" },
    { bit: CHAT_ADMIN_RIGHTS.ManageCall, label: "管理语音视频房 (ManageCall)" },
    { bit: CHAT_ADMIN_RIGHTS.AddAdmins, label: "任命新管理员 (AddAdmins)" },
    { bit: CHAT_ADMIN_RIGHTS.Anonymous, label: "匿名管理员 (Anonymous)" },
    { bit: CHAT_ADMIN_RIGHTS.ManageDirect, label: "直连管理 (ManageDirect)" },
  ];

  const restrictionOptions = [
    { bit: CHAT_RESTRICTIONS.SendMessages, label: "发送文字消息 (SendMessages)" },
    { bit: CHAT_RESTRICTIONS.SendPhotos, label: "发送图片 (SendPhotos)" },
    { bit: CHAT_RESTRICTIONS.SendVideos, label: "发送视频 (SendVideos)" },
    { bit: CHAT_RESTRICTIONS.SendVoiceMessages, label: "发送语音 (SendVoiceMessages)" },
    { bit: CHAT_RESTRICTIONS.SendFiles, label: "发送文件 (SendFiles)" },
    { bit: CHAT_RESTRICTIONS.SendPolls, label: "发起投票 (SendPolls)" },
    { bit: CHAT_RESTRICTIONS.EmbedLinks, label: "嵌入链接预览 (EmbedLinks)" },
    { bit: CHAT_RESTRICTIONS.CreateTopics, label: "创建话题 (CreateTopics)" },
    { bit: CHAT_RESTRICTIONS.InviteUsers, label: "邀请成员 (InviteUsers)" },
    { bit: CHAT_RESTRICTIONS.PinMessages, label: "置顶消息 (PinMessages)" },
  ];

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
          background: "var(--sand-surface-backdrop, #242429)",
          padding: "20px 24px",
          borderRadius: "16px",
          width: "400px",
          maxHeight: "88vh",
          overflowY: "auto",
          color: "var(--sand-text-primary, #ffffff)",
          boxShadow: "0 12px 36px rgba(0,0,0,0.5)",
          border: "1px solid var(--sand-border-weak, rgba(128,128,128,0.2))",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <Shield size={18} color="#1084fe" />
            <h3 style={{ margin: 0, fontSize: "15px", fontWeight: 700 }}>群管与成员权限</h3>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="关闭"
            style={{ background: "none", border: 0, color: "var(--sand-text-secondary, #888)", cursor: "pointer" }}
          >
            <X size={16} />
          </button>
        </div>

        {/* Tab switch */}
        <div style={{ display: "flex", gap: "6px", marginBottom: "14px" }}>
          <button
            type="button"
            onClick={() => setActiveTab("restrict")}
            style={{
              flex: 1,
              padding: "6px",
              borderRadius: "8px",
              border: activeTab === "restrict" ? "1px solid #ff9800" : "1px solid var(--sand-border-weak, #444)",
              background: activeTab === "restrict" ? "rgba(255, 152, 0, 0.15)" : "transparent",
              color: activeTab === "restrict" ? "#ff9800" : "inherit",
              fontSize: "12px",
              fontWeight: 600,
              cursor: "pointer",
            }}
          >
            禁言与限制 (Restrictions)
          </button>
          <button
            type="button"
            onClick={() => setActiveTab("admin")}
            style={{
              flex: 1,
              padding: "6px",
              borderRadius: "8px",
              border: activeTab === "admin" ? "1px solid #1084fe" : "1px solid var(--sand-border-weak, #444)",
              background: activeTab === "admin" ? "rgba(16, 132, 254, 0.15)" : "transparent",
              color: activeTab === "admin" ? "#1084fe" : "inherit",
              fontSize: "12px",
              fontWeight: 600,
              cursor: "pointer",
            }}
          >
            管理员权限 (Admin Rights)
          </button>
        </div>

        <div style={{ fontSize: "12px", marginBottom: "4px", fontWeight: 600 }}>目标用户 ID</div>
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
            background: "var(--sand-surface-muted, rgba(128,128,128,0.1))",
            color: "inherit",
            fontSize: "12px",
            marginBottom: "10px",
            boxSizing: "border-box",
          }}
        />

        {activeTab === "restrict" ? (
          <>
            <div style={{ fontSize: "12px", marginBottom: "4px", fontWeight: 600 }}>限制时长 (untilDate)</div>
            <select
              value={banDuration}
              onChange={(e) => onBanDurationChange(e.target.value)}
              style={{
                width: "100%",
                padding: "6px",
                borderRadius: "6px",
                border: "1px solid var(--sand-border-weak, #444)",
                background: "var(--sand-surface-muted, rgba(128,128,128,0.1))",
                color: "inherit",
                fontSize: "12px",
                marginBottom: "12px",
                boxSizing: "border-box",
              }}
            >
              <option value="3600">1 小时</option>
              <option value="86400">1 天</option>
              <option value="604800">1 周</option>
              <option value="2592000">1 个月</option>
              <option value="0">永久限制</option>
            </select>

            <div style={{ fontSize: "12px", marginBottom: "6px", fontWeight: 600 }}>禁止的权限列表 (ChatRestrictions)</div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr", gap: "5px", marginBottom: "14px" }}>
              {restrictionOptions.map((opt) => (
                <label key={opt.bit} style={{ display: "flex", alignItems: "center", gap: "6px", fontSize: "11px", cursor: "pointer" }}>
                  <input type="checkbox" checked={Boolean(restrictions & opt.bit)} onChange={() => toggleRestriction(opt.bit)} />
                  <span>{opt.label}</span>
                </label>
              ))}
            </div>

            {adminStatus && <div style={{ fontSize: "11px", color: "#69f0ae", marginBottom: "10px" }}>{adminStatus}</div>}
            <div style={{ display: "flex", gap: "8px" }}>
              <button
                type="button"
                data-testid="tdrp-admin-restrict-btn"
                onClick={() => onAdminAction("restrict", { restrictions })}
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
                应用禁言限制
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
          </>
        ) : (
          <>
            <div style={{ fontSize: "12px", marginBottom: "4px", fontWeight: 600 }}>自定义头衔称号 (Custom Rank &lt;=16 chars)</div>
            <input
              type="text"
              maxLength={16}
              placeholder="例如: 技术顾问 / 社区管家"
              value={customRank}
              onChange={(e) => setCustomRank(e.target.value)}
              style={{
                width: "100%",
                padding: "6px 8px",
                borderRadius: "6px",
                border: "1px solid var(--sand-border-weak, #444)",
                background: "var(--sand-surface-muted, rgba(128,128,128,0.1))",
                color: "inherit",
                fontSize: "12px",
                marginBottom: "12px",
                boxSizing: "border-box",
              }}
            />

            <div style={{ fontSize: "12px", marginBottom: "6px", fontWeight: 600 }}>授予的管理员权限 (ChatAdminRights)</div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr", gap: "5px", marginBottom: "14px" }}>
              {adminRightOptions.map((opt) => (
                <label key={opt.bit} style={{ display: "flex", alignItems: "center", gap: "6px", fontSize: "11px", cursor: "pointer" }}>
                  <input type="checkbox" checked={Boolean(adminRights & opt.bit)} onChange={() => toggleRight(opt.bit)} />
                  <span>{opt.label}</span>
                </label>
              ))}
            </div>

            {adminStatus && <div style={{ fontSize: "11px", color: "#69f0ae", marginBottom: "10px" }}>{adminStatus}</div>}
            <button
              type="button"
              data-testid="tdrp-admin-grant-btn"
              onClick={() => onAdminAction("restrict", { rank: customRank, rights: adminRights })}
              style={{
                width: "100%",
                padding: "8px",
                borderRadius: "6px",
                border: 0,
                background: "#1084fe",
                color: "#fff",
                fontWeight: 700,
                fontSize: "12px",
                cursor: "pointer",
              }}
            >
              保存管理员权限与称号
            </button>
          </>
        )}
      </div>
    </div>
  );
};
