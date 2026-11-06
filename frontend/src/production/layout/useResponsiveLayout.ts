import { useState, useEffect, useCallback, useRef } from "react";

/**
 * 3-Tier Responsive Layout State Machine (FSP-001 §17 & Blueprint 01 §4.2, §5.4).
 *
 * Upstream Source Alignment:
 * - Telegram Desktop: `Telegram/SourceFiles/window/window_adaptive.cpp` (`Adaptive::WindowLayout`:
 *   `OneColumn` / `Normal` / `ThreeColumn`, `isOneColumn()`, `isNormal()`, `isThreeColumn()`).
 * - OpenClaw: `ui/src/app-navigation.ts` & responsive split-view state transitions.
 */

export type ResponsiveLayoutTier = "wide" | "standard" | "compact";

export const RESPONSIVE_BREAKPOINT_COMPACT_MAX = 899;
export const RESPONSIVE_BREAKPOINT_STANDARD_MIN = 900;
export const RESPONSIVE_BREAKPOINT_STANDARD_MAX = 1199;
export const RESPONSIVE_BREAKPOINT_WIDE_MIN = 1200;

export interface UseResponsiveLayoutOptions {
  readonly initialWidth?: number;
  readonly initialHeight?: number;
  readonly defaultDrawerOpenInWide?: boolean;
  readonly defaultSidebarOpenInCompact?: boolean;
}

export interface ResponsiveLayoutState {
  readonly tier: ResponsiveLayoutTier;
  readonly tierClassName: "fabushi-tier-wide" | "fabushi-tier-standard" | "fabushi-tier-compact";
  readonly viewportWidth: number;
  readonly windowWidth: number;
  readonly windowHeight: number;
  readonly isCompact: boolean;
  readonly isStandard: boolean;
  readonly isWide: boolean;
  readonly isSidebarVisible: boolean;
  readonly isDrawerDocked: boolean;
  readonly isDrawerVisible: boolean;
  readonly setSidebarVisible: (visible: boolean) => void;
  readonly setDrawerVisible: (visible: boolean) => void;
  readonly toggleSidebar: () => void;
  readonly toggleDrawer: () => void;
}

export function resolveResponsiveLayoutTier(width: number): ResponsiveLayoutTier {
  if (!Number.isFinite(width) || width >= RESPONSIVE_BREAKPOINT_WIDE_MIN) {
    return "wide";
  }
  if (width >= RESPONSIVE_BREAKPOINT_STANDARD_MIN) {
    return "standard";
  }
  return "compact";
}

export function responsiveShellTierClassName(
  tier: ResponsiveLayoutTier,
): "fabushi-tier-wide" | "fabushi-tier-standard" | "fabushi-tier-compact" {
  switch (tier) {
    case "wide":
      return "fabushi-tier-wide";
    case "standard":
      return "fabushi-tier-standard";
    case "compact":
      return "fabushi-tier-compact";
  }
}

function readViewportDimensions(
  fallbackWidth = RESPONSIVE_BREAKPOINT_WIDE_MIN,
  fallbackHeight = 800,
): { width: number; height: number } {
  if (typeof window === "undefined") {
    return { width: fallbackWidth, height: fallbackHeight };
  }
  const width =
    typeof window.innerWidth === "number" && window.innerWidth > 0
      ? window.innerWidth
      : fallbackWidth;
  const height =
    typeof window.innerHeight === "number" && window.innerHeight > 0
      ? window.innerHeight
      : fallbackHeight;
  return { width, height };
}

export function useResponsiveLayout(
  options: UseResponsiveLayoutOptions = {},
): ResponsiveLayoutState {
  const {
    initialWidth = RESPONSIVE_BREAKPOINT_WIDE_MIN,
    initialHeight = 800,
    defaultDrawerOpenInWide = false,
    defaultSidebarOpenInCompact = false,
  } = options;

  const [dimensions, setDimensions] = useState<{ width: number; height: number }>(() =>
    readViewportDimensions(initialWidth, initialHeight),
  );

  const currentTier = resolveResponsiveLayoutTier(dimensions.width);
  const previousTierRef = useRef<ResponsiveLayoutTier>(currentTier);
  const dockedDrawerMemoryRef = useRef<boolean>(defaultDrawerOpenInWide);

  const [compactSidebarOpen, setCompactSidebarOpen] = useState<boolean>(
    defaultSidebarOpenInCompact,
  );
  const [drawerOpen, setDrawerOpen] = useState<boolean>(() =>
    currentTier === "wide" ? defaultDrawerOpenInWide : false,
  );

  useEffect(() => {
    const prevTier = previousTierRef.current;
    if (prevTier === currentTier) return;
    previousTierRef.current = currentTier;

    if (prevTier === "wide" && currentTier !== "wide") {
      // Collapse docked drawer when shrinking below 1200px to avoid unexpected overlay occlusion
      setDrawerOpen(false);
    } else if (prevTier !== "wide" && currentTier === "wide") {
      // Restore user's remembered wide-mode docked drawer preference
      setDrawerOpen(dockedDrawerMemoryRef.current);
    }

    if (currentTier !== "compact") {
      setCompactSidebarOpen(false);
    }
  }, [currentTier]);

  useEffect(() => {
    if (typeof window === "undefined") return;

    const syncViewport = () => {
      const next = readViewportDimensions(initialWidth, initialHeight);
      setDimensions((prev) =>
        prev.width === next.width && prev.height === next.height ? prev : next,
      );
    };

    syncViewport();

    window.addEventListener("resize", syncViewport, { passive: true });

    const mediaQueries: MediaQueryList[] = [];
    if (typeof window.matchMedia === "function") {
      const queries = [
        `(min-width: ${RESPONSIVE_BREAKPOINT_WIDE_MIN}px)`,
        `(min-width: ${RESPONSIVE_BREAKPOINT_STANDARD_MIN}px) and (max-width: ${RESPONSIVE_BREAKPOINT_STANDARD_MAX}px)`,
        `(max-width: ${RESPONSIVE_BREAKPOINT_COMPACT_MAX}px)`,
      ];
      for (const query of queries) {
        const mql = window.matchMedia(query);
        if (typeof mql.addEventListener === "function") {
          mql.addEventListener("change", syncViewport);
          mediaQueries.push(mql);
        }
      }
    }

    return () => {
      window.removeEventListener("resize", syncViewport);
      for (const mql of mediaQueries) {
        if (typeof mql.removeEventListener === "function") {
          mql.removeEventListener("change", syncViewport);
        }
      }
    };
  }, [initialHeight, initialWidth]);

  const setSidebarVisible = useCallback(
    (visible: boolean) => {
      if (currentTier === "compact") {
        setCompactSidebarOpen(visible);
        if (visible) {
          setDrawerOpen(false);
        }
      }
    },
    [currentTier],
  );

  const toggleSidebar = useCallback(() => {
    if (currentTier === "compact") {
      setCompactSidebarOpen((prev) => {
        const next = !prev;
        if (next) {
          setDrawerOpen(false);
        }
        return next;
      });
    }
  }, [currentTier]);

  const setDrawerVisible = useCallback(
    (visible: boolean) => {
      if (currentTier === "wide") {
        dockedDrawerMemoryRef.current = visible;
      }
      if (currentTier === "compact" && visible) {
        setCompactSidebarOpen(false);
      }
      setDrawerOpen(visible);
    },
    [currentTier],
  );

  const toggleDrawer = useCallback(() => {
    setDrawerOpen((prev) => {
      const next = !prev;
      if (currentTier === "wide") {
        dockedDrawerMemoryRef.current = next;
      }
      if (currentTier === "compact" && next) {
        setCompactSidebarOpen(false);
      }
      return next;
    });
  }, [currentTier]);

  const isSidebarVisible = currentTier === "compact" ? compactSidebarOpen : true;
  const isDrawerDocked = currentTier === "wide";

  return {
    tier: currentTier,
    tierClassName: responsiveShellTierClassName(currentTier),
    viewportWidth: dimensions.width,
    windowWidth: dimensions.width,
    windowHeight: dimensions.height,
    isCompact: currentTier === "compact",
    isStandard: currentTier === "standard",
    isWide: currentTier === "wide",
    isSidebarVisible,
    isDrawerDocked,
    isDrawerVisible: drawerOpen,
    setSidebarVisible,
    setDrawerVisible,
    toggleSidebar,
    toggleDrawer,
  };
}
