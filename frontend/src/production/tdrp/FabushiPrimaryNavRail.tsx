import React from "react";
import {
  MessageSquare,
  Users,
  Bot,
  CheckSquare,
  Monitor,
  Zap,
  ShoppingBag,
  Settings,
} from "lucide-react";

export type FabushiPrimaryNavDomain =
  | "messages"
  | "contacts"
  | "agents"
  | "tasks"
  | "computer"
  | "automations"
  | "marketplace"
  | "settings";

export interface FabushiPrimaryNavRailProps {
  readonly activeDomain: FabushiPrimaryNavDomain;
  readonly onSelectDomain: (domain: FabushiPrimaryNavDomain) => void;
  readonly unreadMessagesCount?: number;
  readonly onOpenSettings?: () => void;
  readonly onOpenMarketplace?: () => void;
  readonly userAvatarText?: string;
}

interface NavItemDef {
  readonly id: FabushiPrimaryNavDomain;
  readonly label: string;
  readonly icon: React.ComponentType<{ size?: number; strokeWidth?: number; color?: string }>;
  readonly badge?: number;
}

export const FabushiPrimaryNavRail: React.FC<FabushiPrimaryNavRailProps> = ({
  activeDomain,
  onSelectDomain,
  unreadMessagesCount = 0,
  onOpenSettings,
  onOpenMarketplace,
  userAvatarText = "F",
}) => {
  const topNavItems: readonly NavItemDef[] = [
    { id: "messages", label: "消息", icon: MessageSquare, badge: unreadMessagesCount },
    { id: "contacts", label: "联系人", icon: Users },
    { id: "agents", label: "Agents", icon: Bot },
    { id: "tasks", label: "任务", icon: CheckSquare },
    { id: "computer", label: "Computer", icon: Monitor },
    { id: "automations", label: "自动化", icon: Zap },
    { id: "marketplace", label: "插件市场", icon: ShoppingBag },
  ];

  const handleItemClick = (domain: FabushiPrimaryNavDomain) => {
    if (domain === "marketplace" && onOpenMarketplace) {
      onOpenMarketplace();
    }
    if (domain === "settings" && onOpenSettings) {
      onOpenSettings();
    }
    onSelectDomain(domain);
  };

  return (
    <nav
      className="fabushi-primary-nav-rail"
      data-testid="fabushi-primary-nav-rail"
      aria-label="主要导航"
      style={{
        width: "52px",
        minWidth: "52px",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "12px 0 10px 0",
        background: "var(--fabushi-surface-rail, var(--cursor-bg-chrome, #16161a))",
        borderRight: "1px solid var(--fabushi-border-weak, var(--sand-border-weak, rgba(255, 255, 255, 0.08)))",
        userSelect: "none",
        zIndex: 10,
      }}
    >
      {/* Top primary product domains */}
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "8px", width: "100%" }}>
        {/* Fabushi Brand Mark */}
        <div
          title="全球法布施 (Fabushi)"
          style={{
            width: "34px",
            height: "34px",
            borderRadius: "10px",
            background: "linear-gradient(135deg, #1084fe, #00c972)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "#ffffff",
            fontWeight: 700,
            fontSize: "15px",
            marginBottom: "10px",
            cursor: "pointer",
          }}
          onClick={() => handleItemClick("messages")}
        >
          {userAvatarText[0] || "F"}
        </div>

        {topNavItems.map((item) => {
          const isActive = activeDomain === item.id;
          const IconComponent = item.icon;
          return (
            <button
              key={item.id}
              type="button"
              data-nav-domain={item.id}
              data-testid={`fabushi-nav-${item.id}`}
              title={item.label}
              aria-label={item.label}
              aria-current={isActive ? "page" : undefined}
              onClick={() => handleItemClick(item.id)}
              style={{
                position: "relative",
                width: "40px",
                height: "40px",
                borderRadius: "10px",
                border: 0,
                background: isActive
                  ? "var(--fabushi-accent-subtle, rgba(16, 132, 254, 0.15))"
                  : "transparent",
                color: isActive
                  ? "var(--fabushi-accent, #1084fe)"
                  : "var(--fabushi-text-secondary, rgba(255, 255, 255, 0.6))",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                cursor: "pointer",
                transition: "all 0.15s ease",
              }}
            >
              <IconComponent size={20} strokeWidth={isActive ? 2.2 : 1.8} />

              {/* Unread badge */}
              {item.badge != null && item.badge > 0 && (
                <span
                  data-badge="unread"
                  style={{
                    position: "absolute",
                    top: "4px",
                    right: "4px",
                    minWidth: "14px",
                    height: "14px",
                    padding: "0 3px",
                    borderRadius: "7px",
                    background: "var(--fabushi-danger, #ff263c)",
                    color: "#ffffff",
                    fontSize: "9px",
                    fontWeight: 700,
                    lineHeight: "14px",
                    textAlign: "center",
                  }}
                >
                  {item.badge > 99 ? "99+" : item.badge}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {/* Bottom settings item */}
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", width: "100%" }}>
        <button
          type="button"
          data-nav-domain="settings"
          data-testid="fabushi-nav-settings"
          title="账号与设置"
          aria-label="账号与设置"
          aria-current={activeDomain === "settings" ? "page" : undefined}
          onClick={() => handleItemClick("settings")}
          style={{
            width: "40px",
            height: "40px",
            borderRadius: "10px",
            border: 0,
            background: activeDomain === "settings"
              ? "var(--fabushi-accent-subtle, rgba(16, 132, 254, 0.15))"
              : "transparent",
            color: activeDomain === "settings"
              ? "var(--fabushi-accent, #1084fe)"
              : "var(--fabushi-text-secondary, rgba(255, 255, 255, 0.6))",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            cursor: "pointer",
            transition: "all 0.15s ease",
          }}
        >
          <Settings size={20} strokeWidth={activeDomain === "settings" ? 2.2 : 1.8} />
        </button>
      </div>
    </nav>
  );
};
