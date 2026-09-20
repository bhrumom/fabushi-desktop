import { AgentOperationRegistry, type AgentOperationSnapshot } from '../grok-runtime/agent-operation-registry';

export interface AgentWorkspaceDraftSnapshot {
  readonly [peerKey: string]: string;
}

/**
 * Renderer-facing state owner for the Agent workspace.
 *
 * The view layer must not know how request ids become runtime operation ids.
 * Keeping that adoption state here mirrors Fabu's renderer/controller boundary
 * and prevents an active-chat pointer from becoming the source of truth.
 */
export class AgentWorkspaceController {
  private readonly operations = new AgentOperationRegistry();
  private readonly drafts = new Map<string, string>();

  beginRequest(peerKey: string, requestId: string): void {
    this.operations.beginRequest(peerKey, requestId);
  }

  cancelRequest(requestId: string): string | null {
    return this.operations.cancelRequest(requestId);
  }

  adoptOperation(
    requestId: string | null | undefined,
    operationId: string,
    fallbackPeerKey?: string | null,
  ): string | null {
    return this.operations.adoptOperation(requestId, operationId, fallbackPeerKey);
  }

  claimOperation(operationId: string, peerKey: string): string {
    return this.operations.claimOperation(operationId, peerKey);
  }

  finishOperation(operationId: string): string | null {
    return this.operations.finishOperation(operationId);
  }

  operationForPeer(peerKey: string | null | undefined): string | null {
    return this.operations.operationForPeer(peerKey);
  }

  requestForPeer(peerKey: string | null | undefined): string | null {
    return this.operations.requestForPeer(peerKey);
  }

  peerForOperation(operationId: string | null | undefined): string | null {
    return this.operations.peerForOperation(operationId);
  }

  peerForRequest(requestId: string | null | undefined): string | null {
    return this.operations.peerForRequest(requestId);
  }

  isBusy(peerKey: string | null | undefined): boolean {
    return this.operations.isBusy(peerKey);
  }

  snapshot(): AgentOperationSnapshot {
    return this.operations.snapshot();
  }

  clearPeer(peerKey: string): void {
    this.operations.clearPeer(peerKey);
    this.drafts.delete(peerKey);
  }

  clear(): void {
    this.operations.clear();
    this.drafts.clear();
  }

  setDraft(peerKey: string, value: string): void {
    const normalized = value;
    if (normalized) this.drafts.set(peerKey, normalized);
    else this.drafts.delete(peerKey);
  }

  hydrateDrafts(snapshot: Readonly<Record<string, string>>): void {
    this.drafts.clear();
    for (const [peerKey, value] of Object.entries(snapshot)) {
      if (peerKey && value) this.drafts.set(peerKey, value);
    }
  }

  draftForPeer(peerKey: string | null | undefined): string {
    return peerKey ? this.drafts.get(peerKey) ?? '' : '';
  }

  draftSnapshot(): AgentWorkspaceDraftSnapshot {
    return Object.freeze(Object.fromEntries(this.drafts));
  }
}
