export interface FilterableAgent {
  readonly id: string;
  readonly name?: string;
  readonly isGroup?: boolean;
  readonly isUnread?: boolean;
  readonly unreadCount?: number;
  readonly isSubagent?: boolean;
  readonly isPinned?: boolean;
}

export function filterAgentsByFolder<T extends FilterableAgent>(
  agents: readonly T[],
  folderId: string,
  humanIdsSet: Set<string>
): T[] {
  switch (folderId) {
    case "all":
      return agents.filter((a) => !(a.isSubagent === true && !a.isPinned));
    case "unread":
      return agents.filter(
        (a) => (a.unreadCount != null && a.unreadCount > 0) || a.isUnread === true
      );
    case "direct":
      return agents.filter((a) => humanIdsSet.has(a.id) && !a.isGroup);
    case "groups":
      return agents.filter((a) => a.isGroup === true);
    case "channels":
      return agents.filter((a) => (a as { isChannel?: boolean }).isChannel === true);
    case "agents":
      return agents.filter((a) => !humanIdsSet.has(a.id) && !a.isGroup && !(a.isSubagent === true && !a.isPinned));
    default:
      return [...agents];
  }
}
