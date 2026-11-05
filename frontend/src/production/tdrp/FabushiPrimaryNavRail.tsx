import React, { useRef, useState, useCallback, useEffect } from "react";
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

/**
 * Canonical Primary Left Navigation Rail (Blueprint 01 §4.1, §5.1).
 *
 * Upstream Source Alignment:
 * - OpenClaw: `ui/src/app-navigation.ts` (`SIDEBAR_NAV_ROUTES`, hub route highlighting).
 * - Telegram Desktop: `Telegram/SourceFiles/window/window_main_menu.cpp` & `ui/widgets/side_bar_button.cpp`.
 * - Invariant: Clicking `tasks` unconditionally dispatches `"tasks"` regardless of `activeIsHuman` or `activeAgent`.
 */

export type FabushiPrimaryNavDomain =
  | "messages"
  | "contacts"
  | "agents"
  | "tasks"
  | "computer"
  | "automations"
  | "marketplace"
  | "settings";

export const FABUSHI_PRIMARY_NAV_DOMAINS: readonly FabushiPrimaryNavDomain[] = [
  "messages",
  "contacts",
  "agents",
  "tasks",
  "computer",
  "automations",
  "marketplace",
  "settings",
];

export interface FabushiPrimaryNavRailProps {
  readonly activeDomain: FabushiPrimaryNavDomain;
  readonly onSelectDomain: (domain: FabushiPrimaryNavDomain) => void;
  readonly unreadMessagesCount?: number;
  readonly pendingTasksCount?: number;
  readonly hasWaitingAuth?: boolean;
  readonly onOpenTasks?: () => void;
  readonly onOpenSettings?: () => void;
  readonly onOpenMarketplace?: () => void;
  readonly userAvatarText?: string;
  readonly isCompact?: boolean;
}

interface NavItemDef {
  readonly id: FabushiPrimaryNavDomain;
  readonly label: string;
  readonly shortcutHint: string;
  readonly icon: React.ComponentType<{ size?: number; strokeWidth?: number; color?: string }>;
  readonly badge?: number;
  readonly warningDot?: boolean;
}

export const FabushiPrimaryNavRail: React.FC<FabushiPrimaryNavRailProps> = ({
  activeDomain,
  onSelectDomain,
  unreadMessagesCount = 0,
  pendingTasksCount = 0,
  hasWaitingAuth = false,
  onOpenTasks,
  onOpenSettings,
  onOpenMarketplace,
  userAvatarText = "F",
  isCompact = false,
}) => {
  const topNavItems: readonly NavItemDef[] = [
    {
      id: "messages",
      label: "消息",
      shortcutHint: "⌘1",
      icon: MessageSquare,
      badge: unreadMessagesCount,
    },
    {
      id: "contacts",
      label: "联系人",
      shortcutHint: "⌘2",
      icon: Users,
    },
    {
      id: "agents",
      label: "Agents",
      shortcutHint: "⌘3",
      icon: Bot,
    },
    {
      id: "tasks",
      label: "任务",
      shortcutHint: "⌘4",
      icon: CheckSquare,
      badge: pendingTasksCount,
    },
    {
      id: "computer",
      label: "Computer",
      shortcutHint: "⌘5",
      icon: Monitor,
    },
    {
      id: "automations",
      label: "自动化",
      shortcutHint: "⌘6",
      icon: Zap,
    },
    {
      id: "marketplace",
      label: "插件市场",
      shortcutHint: "⌘7",
      icon: ShoppingBag,
    },
  ];

  const settingsItem: NavItemDef = {
    id: "settings",
    label: "账号与设置",
    shortcutHint: "⌘,",
    icon: Settings,
    warningDot: hasWaitingAuth,
  };

  const allItems: readonly NavItemDef[] = [...topNavItems, settingsItem];
  const buttonRefs = useRef<Map<FabushiPrimaryNavDomain, HTMLButtonElement | null>>(new Map());
  const [focusedDomain, setFocusedDomain] = useState<FabushiPrimaryNavDomain>(activeDomain);

  useEffect(() => {
    setFocusedDomain(activeDomain);
  }, [activeDomain]);

  const handleItemClick = useCallback(
    (domain: FabushiPrimaryNavDomain) => {
      setFocusedDomain(domain);
      if (domain === "tasks" && onOpenTasks) {
        onOpenTasks();
      }
      if (domain === "marketplace" && onOpenMarketplace) {
        onOpenMarketplace();
      }
      if (domain === "settings" && onOpenSettings) {
        onOpenSettings();
      }
      // Unconditionally dispatch domain selection (including "tasks" regardless of activeAgent / activeIsHuman)
      onSelectDomain(domain);
    },
    [onOpenMarketplace, onOpenSettings, onOpenTasks, onSelectDomain],
  );

  const focusDomainAtIndex = useCallback(
    (index: number) => {
      const normalizedIndex = ((index % allItems.length) + allItems.length) % allItems.length;
      const target = allItems[normalizedIndex];
      if (!target) return;
      setFocusedDomain(target.id);
      const btn = buttonRefs.current.get(target.id);
      btn?.focus();
    },
    [allItems],
  );

  const handleRailKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLElement>) => {
      const currentIndex = allItems.findIndex((item) => item.id === focusedDomain);
      const baseIndex = currentIndex >= 0 ? currentIndex : 0;

      if (event.key === "ArrowDown") {
        event.preventDefault();
        focusDomainAtIndex(baseIndex + 1);
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        focusDomainAtIndex(baseIndex - 1);
      } else if (event.key === "Home") {
        event.preventDefault();
        focusDomainAtIndex(0);
      } else if (event.key === "End") {
        event.preventDefault();
        focusDomainAtIndex(allItems.length - 1);
      }
    },
    [allItems, focusDomainAtIndex, focusedDomain],
  );

  const railWidth = isCompact ? "48px" : "52px";
  const buttonSize = isCompact ? "36px" : "38px";

  const renderNavButton = (item: NavItemDef) => {
    const isActive = activeDomain === item.id;
    const isRovingFocusTarget = focusedDomain === item.id;
    const IconComponent = item.icon;

    return (
      <button
        key={item.id}
        ref={(el) => {
          if (el) {
            buttonRefs.current.set(item.id, el);
          } else {
            buttonRefs.current.delete(item.id);
          }
        }}
        type="button"
        className={`fabushi-primary-nav-rail__btn${isActive ? " fabushi-primary-nav-rail__btn--active" : ""}`}
        data-nav-domain={item.id}
        data-testid={`fabushi-nav-${item.id}`}
        title={`${item.label} (${item.shortcutHint})`}
        aria-label={item.label}
        aria-current={isActive ? "page" : undefined}
        tabIndex={isRovingFocusTarget ? 0 : -1}
        onFocus={() => setFocusedDomain(item.id)}
        onClick={() => handleItemClick(item.id)}
        style={{
          position: "relative",
          width: buttonSize,
          height: buttonSize,
          borderRadius: "10px",
          border: 0,
          background: isActive ? "rgba(16, 132, 254, 0.12)" : "transparent",
          color: isActive
            ? "var(--fabushi-accent-primary, #1084fe)"
            : "var(--fabushi-text-secondary, var(--sand-text-secondary, #52525b))",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          cursor: "pointer",
          transition: "all 0.15s cubic-bezier(0.4, 0, 0.2, 1)",
          boxSizing: "border-box",
        }}
      >
        <IconComponent size={19} strokeWidth={isActive ? 2.2 : 1.8} />

        {/* Unread / Pending Count Badge (zero layout shift on increment) */}
        {item.badge != null && item.badge > 0 && (
          <span
            data-badge="unread"
            aria-live="polite"
            aria-atomic="true"
            style={{
              position: "absolute",
              top: "3px",
              right: "3px",
              minWidth: "14px",
              height: "14px",
              padding: "0 3px",
              borderRadius: "7px",
              background: "var(--fabushi-accent-coral, #ff263c)",
              color: "#ffffff",
              fontSize: "9px",
              fontWeight: 700,
              lineHeight: "14px",
              textAlign: "center",
              pointerEvents: "none",
              boxSizing: "border-box",
            }}
          >
            {item.badge > 99 ? "99+" : item.badge}
          </span>
        )}

        {/* Waiting Auth Warning Indicator */}
        {item.warningDot && (item.badge == null || item.badge <= 0) && (
          <span
            data-badge="warning-auth"
            aria-label="需要认证授权"
            style={{
              position: "absolute",
              top: "5px",
              right: "5px",
              width: "8px",
              height: "8px",
              borderRadius: "50%",
              background: "var(--fabushi-status-waiting-auth, #d97706)",
              pointerEvents: "none",
            }}
          />
        )}
      </button>
    );
  };

  return (
    <nav
      role="navigation"
      className={`fabushi-primary-nav-rail${isCompact ? " fabushi-primary-nav-rail--compact" : ""}`}
      data-testid="fabushi-primary-nav-rail"
      aria-label="主要导航"
      onKeyDown={handleRailKeyDown}
      style={{
        width: railWidth,
        minWidth: railWidth,
        height: "100%",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "38px 0 12px 0",
        background:
          "var(--fabushi-bg-sidebar, var(--sand-surface-backdrop, var(--cursor-bg-chrome, #16161a)))",
        borderRight:
          "1px solid var(--fabushi-border, var(--sand-border-weak, rgba(128, 128, 128, 0.15)))",
        userSelect: "none",
        zIndex: 10,
        boxSizing: "border-box",
      }}
    >
      {/* Top primary product domains */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: "8px",
          width: "100%",
        }}
      >
        {/* Fabushi Brand Mark: centered 32x32 monogram */}
        <button
          type="button"
          title="全球法布施 (Fabushi)"
          aria-label="全球法布施 (Fabushi)"
          tabIndex={-1}
          onClick={() => handleItemClick("messages")}
          style={{
            width: "32px",
            height: "32px",
            borderRadius: "10px",
            border: 0,
            padding: 0,
            background: "linear-gradient(135deg, #1084fe, #00c972)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "#ffffff",
            fontWeight: 800,
            fontSize: "14px",
            marginBottom: "8px",
            cursor: "pointer",
            flexShrink: 0,
            boxShadow: "0 2px 8px rgba(16, 132, 254, 0.25)",
          }}
        >
          {userAvatarText[0] || "F"}
        </button>

        {topNavItems.map(renderNavButton)}
      </div>

      {/* Bottom settings item */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          width: "100%",
        }}
      >
        {renderNavButton(settingsItem)}
      </div>
    </nav>
  );
};
