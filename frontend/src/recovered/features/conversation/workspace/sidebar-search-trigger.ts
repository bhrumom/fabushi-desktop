export interface SidebarSearchTriggerActions {
  openSearch(): void;
}

export interface SidebarSearchKeyEvent {
  readonly key: string;
  readonly defaultPrevented: boolean;
  readonly metaKey: boolean;
  readonly ctrlKey: boolean;
  readonly altKey: boolean;
  preventDefault(): void;
  stopPropagation(): void;
}

export const SIDEBAR_SEARCH_TRIGGER = Object.freeze({
  ariaLabel: "Search",
  className: "sand-agents-sidebar__search",
  icon: "search",
  label: "Search",
});

export interface SidebarSearchTrigger {
  onClick(): void;
  onKeyDown(event: SidebarSearchKeyEvent): void;
}

export function createSidebarSearchTrigger(
  actions: SidebarSearchTriggerActions,
): SidebarSearchTrigger {
  return {
    onClick() {
      actions.openSearch();
    },
    onKeyDown(event) {
      if (event.key === "Delete" || event.key === "Backspace") {
        event.stopPropagation();
        return;
      }

      const isPlainPrintableKey =
        !event.defaultPrevented
        && event.key.length === 1
        && event.key !== " "
        && !event.metaKey
        && !event.ctrlKey
        && !event.altKey;

      if (!isPlainPrintableKey) return;
      event.preventDefault();
      actions.openSearch();
    },
  };
}
