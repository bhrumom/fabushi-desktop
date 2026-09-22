export type AgentOperationSnapshot = Readonly<Record<string, string>>;
export type AgentRequestSnapshot = Readonly<Record<string, string>>;

/**
 * Tracks Agent transport ownership by peer instead of globally.
 *
 * Grok/Fabu lets one Agent keep working while the user opens or sends work to
 * another Agent. The registry therefore serializes only within one Agent peer
 * and keeps request-id -> operation-id adoption explicit.
 */
export class AgentOperationRegistry {
  private readonly requestByPeer = new Map<string, string>();
  private readonly peerByRequest = new Map<string, string>();
  private readonly operationByPeer = new Map<string, string>();
  private readonly peerByOperation = new Map<string, string>();

  beginRequest(peerKey: string, requestId: string): void {
    if (this.isBusy(peerKey)) throw new Error(`Agent peer is already busy: ${peerKey}`);
    this.requestByPeer.set(peerKey, requestId);
    this.peerByRequest.set(requestId, peerKey);
  }

  cancelRequest(requestId: string): string | null {
    const peerKey = this.peerByRequest.get(requestId) ?? null;
    if (!peerKey) return null;
    this.peerByRequest.delete(requestId);
    if (this.requestByPeer.get(peerKey) === requestId) this.requestByPeer.delete(peerKey);
    return peerKey;
  }

  adoptOperation(requestId: string | null | undefined, operationId: string, fallbackPeerKey?: string | null): string | null {
    const peerKey = (requestId ? this.peerByRequest.get(requestId) : undefined)
      ?? fallbackPeerKey
      ?? this.peerByOperation.get(operationId)
      ?? null;
    if (!peerKey) return null;
    if (requestId) this.cancelRequest(requestId);

    // A normal turn.state event carries the authoritative conversation id.
    // If an earlier compatibility event tentatively claimed this operation
    // for a different peer, repair both directions before rebinding it.
    const previousPeer = this.peerByOperation.get(operationId);
    if (
      previousPeer
      && previousPeer !== peerKey
      && this.operationByPeer.get(previousPeer) === operationId
    ) {
      this.operationByPeer.delete(previousPeer);
    }

    const previousOperation = this.operationByPeer.get(peerKey);
    if (previousOperation && previousOperation !== operationId) {
      this.peerByOperation.delete(previousOperation);
    }
    this.operationByPeer.set(peerKey, operationId);
    this.peerByOperation.set(operationId, peerKey);
    return peerKey;
  }

  claimOperation(operationId: string, peerKey: string): string {
    this.adoptOperation(this.requestByPeer.get(peerKey), operationId, peerKey);
    return peerKey;
  }

  finishOperation(operationId: string): string | null {
    const peerKey = this.peerByOperation.get(operationId) ?? null;
    if (!peerKey) return null;
    this.peerByOperation.delete(operationId);
    if (this.operationByPeer.get(peerKey) === operationId) this.operationByPeer.delete(peerKey);
    return peerKey;
  }

  operationForPeer(peerKey: string | null | undefined): string | null {
    if (!peerKey) return null;
    return this.operationByPeer.get(peerKey) ?? null;
  }

  requestForPeer(peerKey: string | null | undefined): string | null {
    if (!peerKey) return null;
    return this.requestByPeer.get(peerKey) ?? null;
  }

  peerForOperation(operationId: string | null | undefined): string | null {
    if (!operationId) return null;
    return this.peerByOperation.get(operationId) ?? null;
  }

  peerForRequest(requestId: string | null | undefined): string | null {
    if (!requestId) return null;
    return this.peerByRequest.get(requestId) ?? null;
  }

  isBusy(peerKey: string | null | undefined): boolean {
    if (!peerKey) return false;
    return this.operationByPeer.has(peerKey) || this.requestByPeer.has(peerKey);
  }

  snapshot(): AgentOperationSnapshot {
    return Object.freeze(Object.fromEntries(this.operationByPeer));
  }

  requestSnapshot(): AgentRequestSnapshot {
    return Object.freeze(Object.fromEntries(this.requestByPeer));
  }

  clearPeer(peerKey: string): void {
    const requestId = this.requestByPeer.get(peerKey);
    if (requestId) this.cancelRequest(requestId);
    const operationId = this.operationByPeer.get(peerKey);
    if (operationId) this.finishOperation(operationId);
  }

  clear(): void {
    this.requestByPeer.clear();
    this.peerByRequest.clear();
    this.operationByPeer.clear();
    this.peerByOperation.clear();
  }
}
