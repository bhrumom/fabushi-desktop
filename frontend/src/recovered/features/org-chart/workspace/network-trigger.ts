export interface AgentNetworkTriggerActions {
  readonly isAvailable: boolean;
  closeChooser(): void;
  openOrgChart(): void;
}

export const AGENT_NETWORK_TRIGGER = Object.freeze({
  ariaLabel: "Agent network",
  className: "sand-agents-sidebar__network",
  icon: "cube-nodes",
});

export type AgentNetworkTrigger = () => boolean;

export function createAgentNetworkTrigger(
  actions: AgentNetworkTriggerActions,
): AgentNetworkTrigger {
  return () => {
    if (!actions.isAvailable) return false;
    actions.closeChooser();
    actions.openOrgChart();
    return true;
  };
}
