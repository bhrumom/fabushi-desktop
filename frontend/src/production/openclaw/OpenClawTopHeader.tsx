import React, { useState } from "react";
import { AgentAvatar } from "../../recovered/features/conversation/workspace/agent-avatar";
import {
  ShareIcon,
  LayoutSplitIcon,
  MoreHorizontalIcon,
  SidebarDetailIcon,
} from "./OpenClawIcons";

export interface OpenClawTopHeaderProps {
  title?: string;
  workspacePrefix?: string;
  agent?: {
    id: string;
    name?: string;
    avatarDataUrl?: string | null;
    avatarShape?: string | null;
    avatarColor?: string | null;
    currentActivity?: unknown | null;
    isRunning?: boolean;
    isComposingMessage?: boolean;
    awaitingUserResponse?: unknown | null;
    isGroup?: boolean;
    isSharedRoom?: boolean;
    memberIds?: readonly string[];
  } | null;
  onToggleSidebar?: () => void;
  canGoBack?: boolean;
  canGoForward?: boolean;
  onGoBack?: () => void;
  onGoForward?: () => void;
  onShare?: () => void;
  onToggleLayout?: () => void;
  isLayoutSplit?: boolean;
  onToggleDetails?: () => void;
  isDetailsOpen?: boolean;
  onRename?: () => void;
  onClearHistory?: () => void;
  onExportTranscript?: () => void;
  onOpenSettings?: () => void;
}

export function OpenClawTopHeader({
  title = "Main Session",
  workspacePrefix = "workspace",
  agent,
  onToggleSidebar,
  canGoBack = false,
  canGoForward = false,
  onGoBack,
  onGoForward,
  onShare,
  onToggleLayout,
  isLayoutSplit = false,
  onToggleDetails,
  isDetailsOpen = false,
  onRename,
  onClearHistory,
  onExportTranscript,
  onOpenSettings,
}: OpenClawTopHeaderProps) {
  const [moreMenuOpen, setMoreMenuOpen] = useState(false);
  const avatarKind = agent?.isSharedRoom ? "shared-room" : agent?.isGroup ? "group" : "agent";

  return (
    <header className="oc-top-header sand-chat-header" role="banner">
      <div className="oc-top-header__leading">
        {/* Agent Avatar (Fabushi Authentic AgentAvatar) */}
        <span className="sand-chat-header__avatar" style={{ display: "inline-flex", alignItems: "center" }}>
          <AgentAvatar
            agentId={agent?.id ?? "active-agent"}
            kind={avatarKind}
            memberIds={agent?.memberIds}
            dataUrl={agent?.avatarDataUrl}
            color={agent?.avatarColor}
            shape={agent?.avatarShape}
            currentActivity={agent?.currentActivity}
            isComposingMessage={agent?.isComposingMessage}
            isRunning={agent?.isRunning}
            awaitingUserResponse={agent?.awaitingUserResponse}
            size="sm"
          />
        </span>

        {/* Sidebar Toggle button if present */}
        {onToggleSidebar && (
          <button
            type="button"
            className="oc-top-header__nav-btn"
            onClick={onToggleSidebar}
            title="切换侧边栏 (Toggle Sidebar)"
            aria-label="Toggle Sidebar"
          >
            <span style={{ fontSize: 14 }}>☰</span>
          </button>
        )}

        {/* Back and Forward navigation arrows */}
        <div style={{ display: "flex", gap: 2 }}>
          <button
            type="button"
            className="oc-top-header__nav-btn"
            disabled={!canGoBack}
            onClick={onGoBack}
            title="后退 (Back)"
            aria-label="Back"
          >
            &lt;
          </button>
          <button
            type="button"
            className="oc-top-header__nav-btn"
            disabled={!canGoForward}
            onClick={onGoForward}
            title="前进 (Forward)"
            aria-label="Forward"
          >
            &gt;
          </button>
        </div>

        {/* Breadcrumb: 📁 workspace / Main Session */}
        <div className="oc-top-header__breadcrumb">
          <span style={{ marginRight: 2 }}>📁</span>
          <span>{workspacePrefix}</span>
          <span style={{ opacity: 0.5 }}>/</span>
          <span className="oc-top-header__breadcrumb-title">
            <span style={{ fontSize: 13, marginRight: 2 }}>📄</span>
            {title}
          </span>
        </div>
      </div>

      <div className="oc-top-header__actions">
        {/* 👥 分享 (Share) */}
        <button
          type="button"
          className="oc-top-header__action-btn"
          onClick={onShare}
          title="分享会话 (Share)"
        >
          <ShareIcon size={14} />
          <span>分享</span>
        </button>

        {/* 🪟 布局 (Layout) */}
        <button
          type="button"
          className={`oc-top-header__action-btn ${isLayoutSplit ? "oc-top-header__action-btn--active" : ""}`}
          onClick={onToggleLayout}
          title="切换布局 · 询问 OpenClaw (Layout)"
        >
          <LayoutSplitIcon size={14} />
          <span>布局</span>
        </button>

        {/* ... (More options) */}
        <div style={{ position: "relative" }}>
          <button
            type="button"
            className="oc-top-header__action-btn"
            onClick={() => setMoreMenuOpen(!moreMenuOpen)}
            title="更多操作 (More Options)"
            aria-haspopup="true"
            aria-expanded={moreMenuOpen}
          >
            <MoreHorizontalIcon size={14} />
          </button>

          {moreMenuOpen && (
            <div
              style={{
                position: "absolute",
                top: 32,
                right: 0,
                background: "var(--oc-card-bg)",
                border: "1px solid var(--oc-border-strong)",
                borderRadius: 8,
                boxShadow: "var(--oc-shadow-md)",
                padding: "4px 0",
                zIndex: 100,
                minWidth: 160,
              }}
              onMouseLeave={() => setMoreMenuOpen(false)}
            >
              {onRename && (
                <button
                  type="button"
                  style={{
                    display: "block",
                    width: "100%",
                    textAlign: "left",
                    padding: "6px 14px",
                    background: "none",
                    border: "none",
                    fontSize: 13,
                    color: "var(--oc-text-primary)",
                    cursor: "pointer",
                  }}
                  onClick={() => {
                    setMoreMenuOpen(false);
                    onRename();
                  }}
                >
                  重命名会话
                </button>
              )}
              {onExportTranscript && (
                <button
                  type="button"
                  style={{
                    display: "block",
                    width: "100%",
                    textAlign: "left",
                    padding: "6px 14px",
                    background: "none",
                    border: "none",
                    fontSize: 13,
                    color: "var(--oc-text-primary)",
                    cursor: "pointer",
                  }}
                  onClick={() => {
                    setMoreMenuOpen(false);
                    onExportTranscript();
                  }}
                >
                  导出记录 (Export)
                </button>
              )}
              {onOpenSettings && (
                <button
                  type="button"
                  style={{
                    display: "block",
                    width: "100%",
                    textAlign: "left",
                    padding: "6px 14px",
                    background: "none",
                    border: "none",
                    fontSize: 13,
                    color: "var(--oc-text-primary)",
                    cursor: "pointer",
                  }}
                  onClick={() => {
                    setMoreMenuOpen(false);
                    onOpenSettings();
                  }}
                >
                  会话配置 (Settings)
                </button>
              )}
              <hr style={{ margin: "4px 0", border: "none", borderTop: "1px solid var(--oc-border)" }} />
              {onClearHistory && (
                <button
                  type="button"
                  style={{
                    display: "block",
                    width: "100%",
                    textAlign: "left",
                    padding: "6px 14px",
                    background: "none",
                    border: "none",
                    fontSize: 13,
                    color: "#e74c3c",
                    cursor: "pointer",
                  }}
                  onClick={() => {
                    setMoreMenuOpen(false);
                    onClearHistory();
                  }}
                >
                  清除记录 (Clear)
                </button>
              )}
            </div>
          )}
        </div>

        {/* 详情 (Details) */}
        <button
          type="button"
          className={`oc-top-header__action-btn ${isDetailsOpen ? "oc-top-header__action-btn--active" : ""}`}
          onClick={onToggleDetails}
          data-testid="fabushi-runtime-details-trigger"
          title="智能体详情与运行时配置 (Details)"
        >
          <SidebarDetailIcon size={14} />
          <span>详情</span>
        </button>
      </div>
    </header>
  );
}
