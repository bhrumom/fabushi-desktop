import React from "react";
import { AgentAvatar } from "../../recovered/features/conversation/workspace/agent-avatar";
import {
  PlusCircleIcon,
  GridAppsIcon,
  ChatBubbleIcon,
  UsersIcon,
  SearchIcon,
  HouseGatewayIcon,
  KeyboardIcon,
} from "./OpenClawIcons";

export type NavRailTab = "chat" | "apps" | "contacts" | "search" | "gateway";

export interface OpenClawNavigationRailProps {
  activeTab?: NavRailTab;
  onSelectTab?: (tab: NavRailTab) => void;
  onNewThread?: () => void;
  onOpenSearch?: () => void;
  onOpenShortcuts?: () => void;
  onOpenGateway?: () => void;
  onOpenSettings?: () => void;
  agent?: {
    id: string;
    name?: string;
    avatarDataUrl?: string | null;
    avatarShape?: string | null;
    avatarColor?: string | null;
    isGroup?: boolean;
    isSharedRoom?: boolean;
    memberIds?: readonly string[];
  } | null;
  agentAvatarUrl?: string | null;
  gatewayConnected?: boolean;
}

export function OpenClawNavigationRail({
  activeTab = "chat",
  onSelectTab,
  onNewThread,
  onOpenSearch,
  onOpenShortcuts,
  onOpenGateway,
  onOpenSettings,
  agent,
  gatewayConnected = true,
}: OpenClawNavigationRailProps) {
  const avatarKind = agent?.isSharedRoom ? "shared-room" : agent?.isGroup ? "group" : "agent";

  return (
    <aside className="oc-nav-rail" aria-label="OpenClaw primary navigation">
      <div className="oc-nav-rail__top">
        {/* Active agent avatar (Fabushi Authentic AgentAvatar) */}
        <div className="oc-nav-rail__avatar-wrapper" title={agent?.name ?? "Fabushi Assistant"}>
          <AgentAvatar
            agentId={agent?.id ?? "nav-agent"}
            kind={avatarKind}
            memberIds={agent?.memberIds}
            dataUrl={agent?.avatarDataUrl}
            color={agent?.avatarColor}
            shape={agent?.avatarShape}
            size="md"
          />
          <span
            className="oc-nav-rail__status-dot"
            style={{ backgroundColor: gatewayConnected ? "#27AE60" : "#E74C3C" }}
            aria-label={gatewayConnected ? "Gateway online" : "Gateway offline"}
          />
        </div>

        {/* Red + New Thread Button */}
        <button
          type="button"
          className="oc-nav-rail__btn"
          onClick={onNewThread}
          title="新会话 (New Thread)"
          aria-label="New Thread"
          style={{ padding: 0 }}
        >
          <PlusCircleIcon size={32} />
        </button>

        {/* 2x2 Grid Icon */}
        <button
          type="button"
          className={`oc-nav-rail__btn ${activeTab === "apps" ? "oc-nav-rail__btn--active" : ""}`}
          onClick={() => onSelectTab?.("apps")}
          title="工作区与应用 (Workspaces & Apps)"
          aria-label="Workspaces & Apps"
        >
          <GridAppsIcon size={20} />
        </button>

        {/* Chat Bubble Icon */}
        <button
          type="button"
          className={`oc-nav-rail__btn ${activeTab === "chat" ? "oc-nav-rail__btn--active" : ""}`}
          onClick={() => onSelectTab?.("chat")}
          title="智能体会话 (Threads & Sessions)"
          aria-label="Threads & Sessions"
        >
          <ChatBubbleIcon size={20} />
        </button>

        {/* Two people Icon */}
        <button
          type="button"
          className={`oc-nav-rail__btn ${activeTab === "contacts" ? "oc-nav-rail__btn--active" : ""}`}
          onClick={() => onSelectTab?.("contacts")}
          title="联系人与智能体 (Contacts & Agents)"
          aria-label="Contacts & Agents"
        >
          <UsersIcon size={20} />
        </button>

        {/* Search Magnifying Glass Icon */}
        <button
          type="button"
          className="oc-nav-rail__btn"
          onClick={onOpenSearch}
          title="搜索 (Command Palette / Search)"
          aria-label="Search"
        >
          <SearchIcon size={20} />
        </button>
      </div>

      <div className="oc-nav-rail__bottom">
        {/* Gateway House Icon */}
        <button
          type="button"
          className={`oc-nav-rail__btn ${activeTab === "gateway" ? "oc-nav-rail__btn--active" : ""}`}
          onClick={onOpenGateway}
          title="网关与节点状态 (Gateway & Node Host)"
          aria-label="Gateway Status"
        >
          <HouseGatewayIcon size={20} />
        </button>

        {/* Keyboard Shortcut Icon */}
        <button
          type="button"
          className="oc-nav-rail__btn"
          onClick={onOpenShortcuts}
          title="快捷键帮助 (Shortcuts)"
          aria-label="Shortcuts"
        >
          <KeyboardIcon size={20} />
        </button>

        {/* User Profile Avatar */}
        <button
          type="button"
          className="oc-nav-rail__btn"
          onClick={onOpenSettings}
          title="账户与偏好设置 (Account & Settings)"
          aria-label="Account & Settings"
          style={{ padding: 0 }}
        >
          <div
            style={{
              width: 30,
              height: 30,
              borderRadius: "50%",
              background: "linear-gradient(135deg, #3498db, #2c3e50)",
              color: "#fff",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: 12,
              fontWeight: 600,
            }}
          >
            U
          </div>
        </button>
      </div>
    </aside>
  );
}
