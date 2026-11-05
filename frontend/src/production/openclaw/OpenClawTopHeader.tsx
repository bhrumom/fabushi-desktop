import React, { useState } from "react";
import { Phone, Video, Search, Pin, Hash } from "lucide-react";
import { AgentAvatar } from "../../recovered/features/conversation/workspace/agent-avatar";
import {
  ShareIcon,
  LayoutSplitIcon,
  MoreHorizontalIcon,
  SidebarDetailIcon,
} from "./OpenClawIcons";

/**
 * Unified Conversation Top Header (Blueprint 02 §4.1, §5.1).
 *
 * Eliminates the `activeIsHuman` dual-header split (`sand-chat-header` vs `OpenClawTopHeader`)
 * so Human direct chats, Supergroups, Channels, Forum Topics, and Autonomous Agents share a single
 * canonical header skeleton.
 *
 * Upstream Source Alignment:
 * - Telegram Desktop: `Telegram/SourceFiles/history/view/history_view_top_bar_widget.cpp`
 *   (`HistoryView::TopBarWidget`: unified peer title, online/connecting subtitle, `_call`, `_search`,
 *   `_infoToggle`, `_menuToggle`).
 * - OpenClaw: `ui/src/pages/chat/chat-assistant-identity.ts` &
 *   `ui/src/pages/chat/components/chat-activity-headline-selection.ts`.
 * - Invariant: Share button is only rendered/enabled when a genuine share capability (`canShare` or
 *   authenticated `shareUrl` / `inviteLink` / `onShare` handler) is available; never falls back to browser location URLs.
 */

export type UnifiedHeaderPeerKind = "human" | "agent" | "group" | "channel" | "topic";
export type UnifiedHeaderOnlineState = "online" | "offline" | "connecting";

export interface OpenClawTopHeaderProps {
  title?: string;
  workspacePrefix?: string;
  peerKind?: UnifiedHeaderPeerKind;
  statusSubtitle?: string;
  onlineState?: UnifiedHeaderOnlineState;
  topicTitle?: string | null;
  pinnedBannerTitle?: string;
  onClickPinnedBanner?: () => void;
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
    raw?: Record<string, unknown> | null;
  } | null;
  onToggleSidebar?: () => void;
  canGoBack?: boolean;
  canGoForward?: boolean;
  onGoBack?: () => void;
  onGoForward?: () => void;
  onStartAudioCall?: () => void;
  onStartVideoCall?: () => void;
  onOpenSearch?: () => void;
  canShare?: boolean;
  shareUrl?: string | null;
  onShare?: () => void;
  onToggleLayout?: () => void;
  isLayoutSplit?: boolean;
  onToggleDetails?: () => void;
  isDetailsOpen?: boolean;
  onToggleInspector?: () => void;
  isInspectorOpen?: boolean;
  onRename?: () => void;
  onClearHistory?: () => void;
  onExportTranscript?: () => void;
  onOpenSettings?: () => void;
  trailingExtras?: React.ReactNode;
}

function resolveGenuineShareUrl(
  explicitShareUrl: string | null | undefined,
  raw: Record<string, unknown> | null | undefined,
): string | null {
  if (typeof explicitShareUrl === "string" && explicitShareUrl.trim().length > 0) {
    return explicitShareUrl.trim();
  }
  if (raw && typeof raw === "object") {
    const shareCandidate = raw.shareUrl ?? raw.inviteLink;
    if (typeof shareCandidate === "string" && shareCandidate.trim().length > 0) {
      return shareCandidate.trim();
    }
  }
  return null;
}

function resolveDefaultPeerKind(
  explicitPeerKind: UnifiedHeaderPeerKind | undefined,
  agent: OpenClawTopHeaderProps["agent"],
): UnifiedHeaderPeerKind {
  if (explicitPeerKind) return explicitPeerKind;
  if (agent?.isGroup || agent?.isSharedRoom) return "group";
  return "agent";
}

function peerKindBadgeLabel(peerKind: UnifiedHeaderPeerKind): string {
  switch (peerKind) {
    case "human":
      return "Human";
    case "group":
      return "群组";
    case "channel":
      return "频道";
    case "topic":
      return "话题";
    case "agent":
      return "Agent";
  }
}

const menuItemStyle: React.CSSProperties = {
  display: "block",
  width: "100%",
  textAlign: "left",
  padding: "6px 14px",
  background: "none",
  border: "none",
  fontSize: 13,
  color: "var(--fabushi-text-primary, var(--oc-text-primary))",
  cursor: "pointer",
};

export function OpenClawTopHeader({
  title = "Main Session",
  workspacePrefix: _workspacePrefix = "workspace",
  peerKind,
  statusSubtitle,
  onlineState,
  topicTitle,
  pinnedBannerTitle,
  onClickPinnedBanner,
  agent,
  onToggleSidebar,
  canGoBack = false,
  canGoForward = false,
  onGoBack,
  onGoForward,
  onStartAudioCall,
  onStartVideoCall,
  onOpenSearch,
  canShare,
  shareUrl,
  onShare,
  onToggleLayout,
  isLayoutSplit = false,
  onToggleDetails,
  isDetailsOpen = false,
  onToggleInspector,
  isInspectorOpen,
  onRename,
  onClearHistory,
  onExportTranscript,
  onOpenSettings,
  trailingExtras,
}: OpenClawTopHeaderProps) {
  const [moreMenuOpen, setMoreMenuOpen] = useState(false);

  const resolvedPeerKind = resolveDefaultPeerKind(peerKind, agent);
  const avatarKind = agent?.isSharedRoom
    ? "shared-room"
    : agent?.isGroup || resolvedPeerKind === "group" || resolvedPeerKind === "channel"
      ? "group"
      : "agent";

  const genuineShareUrl = resolveGenuineShareUrl(shareUrl, agent?.raw ?? null);
  const isShareEnabled =
    canShare !== undefined ? canShare : Boolean(genuineShareUrl) || Boolean(onShare);

  const handleShareClick = () => {
    if (onShare) {
      onShare();
      return;
    }
    if (
      genuineShareUrl &&
      typeof navigator !== "undefined" &&
      navigator.clipboard?.writeText
    ) {
      void navigator.clipboard.writeText(genuineShareUrl);
    }
  };

  const resolvedInspectorToggle = onToggleInspector ?? onToggleDetails;
  const resolvedInspectorOpen = isInspectorOpen ?? isDetailsOpen;

  const onlineDotColor =
    onlineState === "online"
      ? "var(--fabushi-status-completed, #16a34a)"
      : onlineState === "connecting"
        ? "var(--fabushi-status-waiting-connection, #d97706)"
        : onlineState === "offline"
          ? "var(--fabushi-text-muted, #8e887e)"
          : null;

  return (
    <header
      className="oc-top-header sand-chat-header fabushi-conversation-header"
      role="banner"
      aria-label="会话顶栏"
      data-peer-kind={resolvedPeerKind}
    >
      <div className="oc-top-header__leading">
        {/* Sidebar Toggle button if present (Compact / Mobile navigation) */}
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

        {/* Navigation controls (only rendered when genuine callbacks and history state exist) */}
        {((onGoBack && canGoBack) || (onGoForward && canGoForward)) && (
          <div style={{ display: "flex", gap: 2 }}>
            {onGoBack && canGoBack && (
              <button
                type="button"
                className="oc-top-header__nav-btn"
                onClick={onGoBack}
                title="后退 (Back)"
                aria-label="Back"
              >
                &lt;
              </button>
            )}
            {onGoForward && canGoForward && (
              <button
                type="button"
                className="oc-top-header__nav-btn"
                onClick={onGoForward}
                title="前进 (Forward)"
                aria-label="Forward"
              >
                &gt;
              </button>
            )}
          </div>
        )}

        {/* Unified Peer / Agent Avatar */}
        <span
          className="sand-chat-header__avatar"
          style={{ position: "relative", display: "inline-flex", alignItems: "center" }}
        >
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
          {onlineDotColor && (
            <span
              data-testid="fabushi-header-online-dot"
              data-online-state={onlineState}
              style={{
                position: "absolute",
                right: -1,
                bottom: -1,
                width: 8,
                height: 8,
                borderRadius: "50%",
                background: onlineDotColor,
                border: "1.5px solid var(--fabushi-bg-surface, #ffffff)",
              }}
            />
          )}
        </span>

        {/* Genuine Conversation Title, Peer Kind & Presence Subtitle */}
        <div
          className="oc-top-header__identity sand-chat-header__identity"
          style={{
            display: "flex",
            flexDirection: "column",
            justifyContent: "center",
            minWidth: 0,
            marginLeft: 4,
          }}
        >
          <div style={{ display: "inline-flex", alignItems: "center", gap: 6, minWidth: 0 }}>
            <span
              className="oc-top-header__title"
              id="sand-conversation-heading"
              style={{
                color: "var(--fabushi-text-primary, var(--oc-text-primary))",
                fontWeight: 600,
                fontSize: 14,
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}
            >
              {title}
            </span>
            <small
              data-testid="fabushi-header-peer-kind"
              style={{
                fontSize: 10,
                fontWeight: 600,
                padding: "1px 6px",
                borderRadius: 6,
                background: "var(--fabushi-card-hover, rgba(128, 128, 128, 0.12))",
                color: "var(--fabushi-text-secondary, var(--oc-text-secondary))",
                flexShrink: 0,
              }}
            >
              {peerKindBadgeLabel(resolvedPeerKind)}
            </small>
            {topicTitle && (
              <span
                data-testid="fabushi-header-topic-pill"
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: 2,
                  fontSize: 11,
                  color: "var(--fabushi-accent-primary, #1084fe)",
                  fontWeight: 500,
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                }}
              >
                <Hash size={11} />
                <span>{topicTitle}</span>
              </span>
            )}
          </div>

          {statusSubtitle && (
            <span
              className="fabushi-header-subtitle"
              data-testid="fabushi-header-status-subtitle"
              aria-live="polite"
              style={{
                fontSize: 11,
                lineHeight: "14px",
                color: "var(--fabushi-text-secondary, var(--oc-text-secondary))",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}
            >
              {statusSubtitle}
            </span>
          )}
        </div>
      </div>

      {/* Center Context Indicator: Pinned Banner Quick Pill */}
      {pinnedBannerTitle && (
        <button
          type="button"
          className="fabushi-header-pinned-pill"
          data-testid="fabushi-header-pinned-pill"
          onClick={onClickPinnedBanner}
          title={pinnedBannerTitle}
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 5,
            maxWidth: 260,
            padding: "3px 10px",
            borderRadius: 12,
            border: "1px solid var(--fabushi-border, rgba(128, 128, 128, 0.15))",
            background: "var(--fabushi-bg-surface, rgba(128, 128, 128, 0.06))",
            color: "var(--fabushi-text-secondary, #555558)",
            fontSize: 11.5,
            cursor: onClickPinnedBanner ? "pointer" : "default",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          <Pin size={12} style={{ flexShrink: 0, color: "var(--fabushi-accent-primary, #1084fe)" }} />
          <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {pinnedBannerTitle}
          </span>
        </button>
      )}

      <div className="oc-top-header__actions">
        {trailingExtras}

        {/* Audio Call */}
        {onStartAudioCall && (
          <button
            type="button"
            className="oc-top-header__action-btn fabushi-header-action-btn"
            data-testid="fabushi-header-audio-call-btn"
            onClick={onStartAudioCall}
            title="发起语音通话 (Audio Call)"
            aria-label="Audio Call"
          >
            <Phone size={14} />
          </button>
        )}

        {/* Video Call */}
        {onStartVideoCall && (
          <button
            type="button"
            className="oc-top-header__action-btn fabushi-header-action-btn"
            data-testid="fabushi-header-video-call-btn"
            onClick={onStartVideoCall}
            title="发起视频通话 (Video Call)"
            aria-label="Video Call"
          >
            <Video size={14} />
          </button>
        )}

        {/* Find in Chat / Search */}
        {onOpenSearch && (
          <button
            type="button"
            className="oc-top-header__action-btn fabushi-header-action-btn"
            data-testid="fabushi-header-search-btn"
            onClick={onOpenSearch}
            title="搜索会话内容 (Search)"
            aria-label="Search in Conversation"
          >
            <Search size={14} />
          </button>
        )}

        {/* 👥 分享 (Share) - only rendered when genuine shareUrl/inviteLink or handler is available */}
        {isShareEnabled && (onShare || genuineShareUrl) && (
          <button
            type="button"
            className="oc-top-header__action-btn fabushi-header-action-btn"
            data-testid="fabushi-header-share-btn"
            onClick={handleShareClick}
            title="分享会话链接 (Share)"
            aria-label="Share"
          >
            <ShareIcon size={14} />
            <span>分享</span>
          </button>
        )}

        {/* 🪟 布局 (Layout) */}
        {onToggleLayout && (
          <button
            type="button"
            className={`oc-top-header__action-btn ${isLayoutSplit ? "oc-top-header__action-btn--active" : ""}`}
            onClick={onToggleLayout}
            title="切换布局 · 询问 OpenClaw (Layout)"
            aria-label="Toggle Layout"
          >
            <LayoutSplitIcon size={14} />
            <span>布局</span>
          </button>
        )}

        {/* ... (More options) */}
        {(onRename || onExportTranscript || onOpenSettings || onClearHistory) && (
          <div style={{ position: "relative" }}>
            <button
              type="button"
              className="oc-top-header__action-btn fabushi-header-action-btn"
              onClick={() => setMoreMenuOpen(!moreMenuOpen)}
              title="更多操作 (More Options)"
              aria-haspopup="true"
              aria-expanded={moreMenuOpen}
              aria-label="More Options"
            >
              <MoreHorizontalIcon size={14} />
            </button>

            {moreMenuOpen && (
              <div
                style={{
                  position: "absolute",
                  top: 32,
                  right: 0,
                  background: "var(--fabushi-card-bg, var(--oc-card-bg))",
                  border: "1px solid var(--fabushi-border-strong, var(--oc-border-strong))",
                  borderRadius: 8,
                  boxShadow: "var(--fabushi-shadow-md, var(--oc-shadow-md))",
                  padding: "4px 0",
                  zIndex: 100,
                  minWidth: 160,
                }}
                onMouseLeave={() => setMoreMenuOpen(false)}
              >
                {onRename && (
                  <button
                    type="button"
                    style={menuItemStyle}
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
                    style={menuItemStyle}
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
                    style={menuItemStyle}
                    onClick={() => {
                      setMoreMenuOpen(false);
                      onOpenSettings();
                    }}
                  >
                    会话配置 (Settings)
                  </button>
                )}
                {onClearHistory && (
                  <>
                    {(onRename || onExportTranscript || onOpenSettings) && (
                      <hr
                        style={{
                          margin: "4px 0",
                          border: "none",
                          borderTop: "1px solid var(--fabushi-border, var(--oc-border))",
                        }}
                      />
                    )}
                    <button
                      type="button"
                      style={{ ...menuItemStyle, color: "var(--fabushi-status-failed, #e74c3c)" }}
                      onClick={() => {
                        setMoreMenuOpen(false);
                        onClearHistory();
                      }}
                    >
                      清除记录 (Clear)
                    </button>
                  </>
                )}
              </div>
            )}
          </div>
        )}

        {/* 详情 / Inspector (Details) */}
        {resolvedInspectorToggle && (
          <button
            type="button"
            className={`oc-top-header__action-btn fabushi-header-action-btn ${
              resolvedInspectorOpen ? "oc-top-header__action-btn--active fabushi-header-action-btn--active" : ""
            }`}
            onClick={resolvedInspectorToggle}
            data-testid="fabushi-runtime-details-trigger"
            title="会话详情与上下文检查器 (Inspector)"
            aria-label="Agent Details"
            aria-expanded={resolvedInspectorOpen}
          >
            <SidebarDetailIcon size={14} />
            <span>详情</span>
          </button>
        )}
      </div>
    </header>
  );
}

export const UnifiedConversationHeader = OpenClawTopHeader;
export type UnifiedConversationHeaderProps = OpenClawTopHeaderProps;
