import React, { useState } from "react";
import { X, Phone, Video, Mic, Search, Shield, Download, Lock, Star, Gift, FileText, CheckSquare, BarChart2 } from "lucide-react";
import type { ProductionCoordinatorClient } from "../coordinator-client";

export interface TdrpProfileDetailsDrawerProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly activeAgent: {
    readonly id: string;
    readonly name: string;
    readonly isGroup?: boolean;
    readonly isHuman?: boolean;
  };
  readonly client?: ProductionCoordinatorClient | null;
  readonly starsBalance?: number;
  readonly onStartCall: (isVideo?: boolean) => void;
  readonly onStartGroupCall: () => void;
  readonly onOpenSearch?: () => void;
  readonly onOpenAdminModal: () => void;
  readonly onOpenExportModal: () => void;
  readonly onOpenStarsModal: () => void;
  readonly onOpenSecurityModal: () => void;
}

export const TdrpProfileDetailsDrawer: React.FC<TdrpProfileDetailsDrawerProps> = ({
  isOpen,
  onClose,
  activeAgent,
  starsBalance = 1250,
  onStartCall,
  onStartGroupCall,
  onOpenSearch,
  onOpenAdminModal,
  onOpenExportModal,
  onOpenStarsModal,
  onOpenSecurityModal,
}) => {
  const [activeTab, setActiveTab] = useState<"media" | "polls" | "todos" | "settings">("media");

  if (!isOpen) return null;

  const rowStyle: React.CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "10px 14px",
    borderRadius: "10px",
    background: "var(--sand-surface-muted, rgba(128, 128, 128, 0.05))",
    border: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.12))",
    fontSize: "13px",
    cursor: "pointer",
    transition: "background 0.12s ease",
  };

  const actionBtnStyle: React.CSSProperties = {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: "4px",
    padding: "8px",
    borderRadius: "8px",
    border: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.12))",
    background: "var(--bg-elevated, #ffffff)",
    color: "var(--text-strong, #211e1a)",
    fontSize: "11px",
    fontWeight: 500,
    cursor: "pointer",
    flex: 1,
  };

  return (
    <aside
      className="sand-info-pane tdrp-profile-details-drawer"
      data-testid="tdrp-profile-details-drawer"
      data-open="true"
      style={{
        width: "320px",
        minWidth: "300px",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        background: "var(--bg-elevated, #ffffff)",
        borderLeft: "1px solid var(--border, #e8e4dc)",
        overflowY: "auto",
        zIndex: 50,
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "12px 16px",
          borderBottom: "1px solid var(--border, #e8e4dc)",
        }}
      >
        <span style={{ fontWeight: 700, fontSize: "15px", color: "var(--text-strong, #211e1a)" }}>详情</span>
        <button
          type="button"
          data-testid="tdrp-drawer-close"
          onClick={onClose}
          style={{ background: "none", border: 0, cursor: "pointer", color: "var(--muted, #6e6960)" }}
        >
          <X size={18} />
        </button>
      </div>

      <div style={{ padding: "16px", display: "flex", flexDirection: "column", gap: "16px" }}>
        {/* Avatar & Hero */}
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", textAlign: "center", gap: "6px" }}>
          <div
            style={{
              width: "64px",
              height: "64px",
              borderRadius: "50%",
              background: "linear-gradient(135deg, #1084fe, #d9534f)",
              color: "#ffffff",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: "24px",
              fontWeight: 700,
              boxShadow: "0 4px 12px rgba(0, 0, 0, 0.1)",
            }}
          >
            {activeAgent.name.slice(0, 2)}
          </div>
          <span style={{ fontSize: "16px", fontWeight: 700, color: "var(--text-strong, #211e1a)" }}>
            {activeAgent.name}
          </span>
          <span style={{ fontSize: "12px", color: "var(--muted, #6e6960)" }}>
            {activeAgent.isGroup ? "群组会话 · 多方协同" : activeAgent.isHuman ? "联系人 · 在线" : "智能体 · 就绪"}
          </span>
        </div>

        {/* 4 Telegram Quick Action Buttons */}
        <div style={{ display: "flex", gap: "8px" }}>
          <button type="button" data-testid="tdrp-drawer-voice" style={actionBtnStyle} onClick={() => onStartCall(false)}>
            <Phone size={16} style={{ color: "#1084fe" }} />
            <span>语音</span>
          </button>
          <button type="button" data-testid="tdrp-drawer-video" style={actionBtnStyle} onClick={() => onStartCall(true)}>
            <Video size={16} style={{ color: "#1084fe" }} />
            <span>视频</span>
          </button>
          {activeAgent.isGroup ? (
            <button type="button" data-testid="tdrp-drawer-group-call" style={actionBtnStyle} onClick={onStartGroupCall}>
              <Mic size={16} style={{ color: "#00c972" }} />
              <span>语音房</span>
            </button>
          ) : (
            <button type="button" data-testid="tdrp-drawer-search" style={actionBtnStyle} onClick={onOpenSearch}>
              <Search size={16} style={{ color: "#6e6960" }} />
              <span>搜索</span>
            </button>
          )}
        </div>

        {/* Telegram Stars Balance Card */}
        <div
          data-testid="tdrp-drawer-stars-card"
          style={{
            padding: "12px 14px",
            borderRadius: "12px",
            background: "linear-gradient(135deg, rgba(234, 179, 8, 0.12), rgba(249, 115, 22, 0.08))",
            border: "1px solid rgba(234, 179, 8, 0.3)",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <Star size={18} style={{ color: "#eab308" }} fill="#eab308" />
            <div>
              <div style={{ fontWeight: 700, fontSize: "13px", color: "var(--text-strong, #211e1a)" }}>
                {starsBalance.toLocaleString()} Stars
              </div>
              <div style={{ fontSize: "11px", color: "var(--muted, #6e6960)" }}>官方星标余额</div>
            </div>
          </div>
          <button
            type="button"
            data-testid="tdrp-drawer-send-gift"
            onClick={onOpenStarsModal}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: "4px",
              padding: "5px 10px",
              borderRadius: "8px",
              border: 0,
              background: "#eab308",
              color: "#000000",
              fontSize: "11px",
              fontWeight: 700,
              cursor: "pointer",
            }}
          >
            <Gift size={13} />
            <span>赠送礼物</span>
          </button>
        </div>

        {/* Administration, Export & Security Rows */}
        <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
          <div style={rowStyle} data-testid="tdrp-drawer-admin" onClick={onOpenAdminModal}>
            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              <Shield size={16} style={{ color: "#1084fe" }} />
              <span>群组管理与权限</span>
            </div>
            <span style={{ fontSize: "11px", color: "var(--muted, #6e6960)" }}>管理</span>
          </div>

          <div style={rowStyle} data-testid="tdrp-drawer-export" onClick={onOpenExportModal}>
            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              <Download size={16} style={{ color: "#2e7d32" }} />
              <span>导出聊天记录</span>
            </div>
            <span style={{ fontSize: "11px", color: "var(--muted, #6e6960)" }}>HTML / JSON</span>
          </div>

          <div style={rowStyle} data-testid="tdrp-drawer-security" onClick={onOpenSecurityModal}>
            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              <Lock size={16} style={{ color: "#d9534f" }} />
              <span>云密码与隐私安全</span>
            </div>
            <span style={{ fontSize: "11px", color: "var(--muted, #6e6960)" }}>2FA / Passkey</span>
          </div>
        </div>

        {/* Shared Media & Runtime Tabs */}
        <div style={{ borderTop: "1px solid var(--border, #e8e4dc)", paddingTop: "12px" }}>
          <div style={{ display: "flex", gap: "4px", marginBottom: "10px" }}>
            {[
              { id: "media", label: "媒体", icon: FileText },
              { id: "polls", label: "投票", icon: BarChart2 },
              { id: "todos", label: "待办", icon: CheckSquare },
            ].map((tab) => (
              <button
                key={tab.id}
                type="button"
                onClick={() => setActiveTab(tab.id as any)}
                style={{
                  flex: 1,
                  padding: "5px 0",
                  fontSize: "11px",
                  fontWeight: activeTab === tab.id ? 700 : 500,
                  borderRadius: "6px",
                  border: 0,
                  background: activeTab === tab.id ? "var(--sand-surface-muted, rgba(120, 120, 128, 0.14))" : "transparent",
                  color: activeTab === tab.id ? "#1084fe" : "var(--muted, #6e6960)",
                  cursor: "pointer",
                }}
              >
                {tab.label}
              </button>
            ))}
          </div>

          <div style={{ fontSize: "12px", color: "var(--muted, #6e6960)", textAlign: "center", padding: "16px 0" }}>
            {activeTab === "media" && "暂无共享媒体文件"}
            {activeTab === "polls" && "暂无历史投票记录"}
            {activeTab === "todos" && "暂无已归档待办清单"}
          </div>
        </div>
      </div>
    </aside>
  );
};
