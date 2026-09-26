export const GROUP_INFO_PANE_ROUTE = "overview" as const;
export const GROUP_INFO_PANE_HEADER = Object.freeze({
  ariaLabel: "Conversation details",
  closeLabel: "Close details",
  sectionLabel: "Members",
});

export interface GroupRouteAgent {
  readonly id: string;
  readonly isGroup: boolean;
  readonly raw: Readonly<Record<string, unknown>> & {
    readonly isSharedRoom?: boolean;
  };
}

export interface GroupInfoPaneRoute {
  readonly route: typeof GROUP_INFO_PANE_ROUTE;
  readonly agentId: string;
  readonly accountKey: string;
  readonly accountGeneration: number;
  readonly header: typeof GROUP_INFO_PANE_HEADER;
  readonly onOpenAgentChat: (agentId: string) => void;
}

export interface GroupInfoPaneRouteInput {
  readonly agent: GroupRouteAgent | null;
  readonly accountKey: string | null;
  readonly accountGeneration: number;
  readonly onOpenAgentChat: (agentId: string) => void;
}

export function isLocalGroupAgent(
  agent: Pick<GroupRouteAgent, "isGroup" | "raw">,
): boolean {
  return agent.isGroup && agent.raw.isSharedRoom !== true;
}

export function projectGroupInfoPaneRoute(
  input: GroupInfoPaneRouteInput,
): GroupInfoPaneRoute | null {
  const { agent, accountKey, accountGeneration, onOpenAgentChat } = input;
  if (
    agent == null
    || agent.id.length === 0
    || !isLocalGroupAgent(agent)
    || accountKey == null
    || accountKey.length === 0
    || !Number.isInteger(accountGeneration)
    || accountGeneration < 0
  ) {
    return null;
  }

  return {
    route: GROUP_INFO_PANE_ROUTE,
    agentId: agent.id,
    accountKey,
    accountGeneration,
    header: GROUP_INFO_PANE_HEADER,
    onOpenAgentChat,
  };
}

export function isCurrentGroupInfoPaneRoute(
  route: GroupInfoPaneRoute | null,
  scope: Pick<GroupInfoPaneRouteInput, "agent" | "accountKey" | "accountGeneration">,
): boolean {
  return route != null
    && scope.agent != null
    && isLocalGroupAgent(scope.agent)
    && route.agentId === scope.agent.id
    && route.accountKey === scope.accountKey
    && route.accountGeneration === scope.accountGeneration;
}
