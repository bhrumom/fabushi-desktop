export interface SidebarProfileActionActions {
  openProfile(agentId: string): void;
}

export const SIDEBAR_PROFILE_ACTION = Object.freeze({
  icon: "pencil",
  label: "Edit Profile",
});

export interface SidebarProfileAction {
  onSelect(agentId: string): void;
}

export function createSidebarProfileAction(
  actions: SidebarProfileActionActions,
): SidebarProfileAction {
  return {
    onSelect(agentId) {
      actions.openProfile(agentId);
    },
  };
}
