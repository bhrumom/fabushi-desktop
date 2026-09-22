export interface HiddenAgentSummary {
  id: string;
  name: string;
}

export interface HiddenChatsOverlayModel {
  hiddenAgents: readonly HiddenAgentSummary[];
  isOpen: boolean;
  onClose(): void;
  onOpenAgent(agentId: string): void;
  onUnhide(agentId: string): void;
}

export function hiddenChatNameId(agentId: string): string {
  return `sand-hidden-chat-${agentId}-name`;
}

export function hiddenChatsEmptyLabel(
  hiddenAgents: readonly HiddenAgentSummary[],
): string | null {
  return hiddenAgents.length === 0 ? "No hidden bots" : null;
}

export function openHiddenChat(
  onClose: () => void,
  onOpenAgent: (agentId: string) => void,
  agentId: string,
): void {
  // Dismissal is intentionally synchronous so the details surface never opens
  // behind the chooser.
  onClose();
  onOpenAgent(agentId);
}
