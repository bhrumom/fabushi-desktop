/**
 * Canonical Profile & Context Inspector Drawer Adapter (`TdrpProfileDetailsDrawer`)
 *
 * Upstream comparative anchors studied before modification:
 * - Telegram Desktop:
 *   - `Telegram/SourceFiles/info/profile/info_profile_widget.cpp` (`Info::Profile::Widget`)
 *   - `Telegram/SourceFiles/info/media/info_media_widget.cpp` (`Info::Media::Type`)
 *   - `Telegram/SourceFiles/api/api_credits.cpp` (real Stars balance & ledger state)
 * - Blueprint 04 (`04-unified-context-inspector-drawer.md` §2.1 BUG-04-03):
 *   - Eradicates hardcoded fake Stars balance defaults; binds strictly to real `starsBalance ?? 0`
 *     with explicit ledger sync status.
 *   - Delegates to unified `ContextInspector` while preserving all `tdrp-drawer-*` testids and props.
 */

import React from "react";
import { X, Phone, Video, Mic, Search, Shield, Download, Lock, Star, Gift } from "lucide-react";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import {
  ContextInspector,
  deriveE2ESafetyFingerprint,
  type ContextInspectorTabId,
} from "../../recovered/features/conversation/workspace/context-inspector";

export interface TdrpProfileDetailsDrawerProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly activeAgent: {
    readonly id: string;
    readonly name: string;
    readonly isGroup?: boolean;
    readonly isHuman?: boolean;
    readonly isChannel?: boolean;
    readonly title?: string;
    readonly description?: string;
  };
  readonly accountScopeGeneration?: number;
  readonly client?: ProductionCoordinatorClient | null;
  /** Strictly bound to authoritative ADR-002 Stars ledger; never defaults to a hardcoded placeholder. */
  readonly starsBalance?: number;
  readonly isStarsLedgerLoaded?: boolean;
  readonly initialTab?: ContextInspectorTabId;
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
  accountScopeGeneration = 0,
  client = null,
  starsBalance,
  isStarsLedgerLoaded = false,
  initialTab = "profile",
  onStartCall,
  onStartGroupCall,
  onOpenSearch,
  onOpenAdminModal,
  onOpenExportModal,
  onOpenStarsModal,
  onOpenSecurityModal,
}) => {
  if (!isOpen) return null;

  const resolvedStarsBalance = starsBalance ?? 0;
  const safetyFingerprint = deriveE2ESafetyFingerprint(activeAgent.id, "local-self");

  const rowStyle: React.CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "10px 14px",
    borderRadius: "10px",
    background: "var(--fabushi-bg-elevated, rgba(128, 128, 128, 0.05))",
    border: "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.12))",
    fontSize: "13px",
    cursor: "pointer",
  };

  const actionBtnStyle: React.CSSProperties = {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: "4px",
    padding: "8px",
    borderRadius: "8px",
    border: "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.12))",
    background: "var(--fabushi-bg-surface, #ffffff)",
    color: "var(--fabushi-text-primary, #211e1a)",
    fontSize: "11px",
    fontWeight: 500,
    cursor: "pointer",
    flex: 1,
  };

  return (
    <div
      className="tdrp-profile-details-drawer"
      data-testid="tdrp-profile-details-drawer"
      data-open="true"
      style={{ height: "100%", display: "flex" }}
    >
      <ContextInspector
        isOpen={isOpen}
        onClose={onClose}
        activeAgent={activeAgent}
        accountScopeGeneration={accountScopeGeneration}
        client={client}
        starsBalance={resolvedStarsBalance}
        initialTab={initialTab}
        onStartCall={(isVideo) => onStartCall(isVideo)}
        onStartGroupCall={onStartGroupCall}
        onOpenSearch={onOpenSearch}
        onOpenAdminModal={onOpenAdminModal}
        onOpenExportModal={onOpenExportModal}
        onOpenStarsModal={onOpenStarsModal}
        onOpenSecurityModal={onOpenSecurityModal}
        customTabRenderers={{
          profile: () => (
            <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
              {/* Quick Close & Action Compatibility Strip */}
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <span style={{ fontSize: "12px", fontWeight: 600, opacity: 0.75 }}>
                  {activeAgent.isGroup
                    ? "群组会话 · 多方协同"
                    : activeAgent.isHuman
                    ? "联系人 · 在线"
                    : "智能体 · 就绪"}
                </span>
                <button
                  type="button"
                  data-testid="tdrp-drawer-close"
                  onClick={onClose}
                  aria-label="关闭详情抽屉"
                  style={{
                    background: "none",
                    border: 0,
                    cursor: "pointer",
                    color: "var(--fabushi-text-muted, #6e6960)",
                  }}
                >
                  <X size={16} />
                </button>
              </div>

              {/* Telegram Quick Action Buttons */}
              <div style={{ display: "flex", gap: "8px" }}>
                <button
                  type="button"
                  data-testid="tdrp-drawer-voice"
                  style={actionBtnStyle}
                  onClick={() => onStartCall(false)}
                >
                  <Phone size={16} style={{ color: "#1084fe" }} />
                  <span>语音</span>
                </button>
                <button
                  type="button"
                  data-testid="tdrp-drawer-video"
                  style={actionBtnStyle}
                  onClick={() => onStartCall(true)}
                >
                  <Video size={16} style={{ color: "#1084fe" }} />
                  <span>视频</span>
                </button>
                {activeAgent.isGroup ? (
                  <button
                    type="button"
                    data-testid="tdrp-drawer-group-call"
                    style={actionBtnStyle}
                    onClick={onStartGroupCall}
                  >
                    <Mic size={16} style={{ color: "#00c972" }} />
                    <span>语音房</span>
                  </button>
                ) : (
                  <button
                    type="button"
                    data-testid="tdrp-drawer-search"
                    style={actionBtnStyle}
                    onClick={onOpenSearch}
                  >
                    <Search size={16} style={{ color: "#6e6960" }} />
                    <span>搜索</span>
                  </button>
                )}
              </div>

              {/* Authoritative ADR-002 Stars Balance Card */}
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
                    <div style={{ fontWeight: 700, fontSize: "13px" }}>
                      {resolvedStarsBalance.toLocaleString()} Stars
                    </div>
                    <div style={{ fontSize: "11px", opacity: 0.72 }}>
                      {isStarsLedgerLoaded || starsBalance !== undefined
                        ? "官方星标余额 (已同步)"
                        : "官方星标余额 (待同步账本)"}
                    </div>
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

              {/* E2E Safety Fingerprint */}
              <div style={rowStyle}>
                <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                  <Lock size={15} style={{ color: "#10b981" }} />
                  <span>端到端加密安全指纹</span>
                </div>
                <span style={{ fontSize: "13px" }}>{safetyFingerprint.grid.join(" ")}</span>
              </div>

              {/* Administration, Export & Security Rows */}
              <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                <div style={rowStyle} data-testid="tdrp-drawer-admin" onClick={onOpenAdminModal}>
                  <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <Shield size={16} style={{ color: "#1084fe" }} />
                    <span>群组管理与权限</span>
                  </div>
                  <span style={{ fontSize: "11px", opacity: 0.7 }}>管理</span>
                </div>

                <div style={rowStyle} data-testid="tdrp-drawer-export" onClick={onOpenExportModal}>
                  <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <Download size={16} style={{ color: "#2e7d32" }} />
                    <span>导出聊天记录</span>
                  </div>
                  <span style={{ fontSize: "11px", opacity: 0.7 }}>HTML / JSON</span>
                </div>

                <div style={rowStyle} data-testid="tdrp-drawer-security" onClick={onOpenSecurityModal}>
                  <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <Lock size={16} style={{ color: "#d9534f" }} />
                    <span>云密码与隐私安全</span>
                  </div>
                  <span style={{ fontSize: "11px", opacity: 0.7 }}>2FA / Passkey</span>
                </div>
              </div>
            </div>
          ),
        }}
      />
    </div>
  );
};
