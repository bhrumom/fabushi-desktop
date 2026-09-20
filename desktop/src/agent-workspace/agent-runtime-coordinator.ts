import type { AttachmentContext, RuntimeEvent } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { AgentWorkspaceController } from './agent-workspace-controller';
import {
  AgentTranscriptStore,
  type AgentTranscriptSourceMessage,
  type AgentTranscriptTerminalStatus,
} from './agent-transcript-store';

type AgentDeltaEvent = Extract<RuntimeEvent, { type: 'chat.delta' }> & { operationId: string };

export interface AgentLocalTurn {
  readonly peerKey: string;
  readonly requestId: string;
  readonly messageId: string;
  readonly text: string;
  readonly createdAtMs: number;
  readonly attachments?: readonly AttachmentContext[];
}

export interface AgentRuntimeCoordinatorHooks {
  onTranscriptChanged?(peerKey: string, thread: readonly AgentTranscriptSourceMessage[]): void;
  onOperationChanged?(peerKey: string): void;
  onOperationStarted?(peerKey: string, operationId: string): void;
  onOperationTerminal?(
    peerKey: string,
    operationId: string,
    status: AgentTranscriptTerminalStatus,
    message?: string,
  ): void;
}

/**
 * Agent runtime event owner for the desktop renderer.
 *
 * Fabu/Grok keeps request adoption, streaming backpressure and transcript
 * reduction outside the React surface. This class provides the same boundary
 * for Mahayana: the renderer can still handle compatibility Messenger event
 * families, but normal Agent chat/operation events are claimed and reduced
 * here without consulting the currently visible Agent.
 */
export class AgentRuntimeCoordinator {
  private readonly pendingDeltas = new Map<string, AgentDeltaEvent>();
  private deltaTimer: ReturnType<typeof globalThis.setTimeout> | null = null;

  constructor(
    private readonly workspace: AgentWorkspaceController,
    private readonly transcripts: AgentTranscriptStore,
    private readonly hooks: AgentRuntimeCoordinatorHooks = {},
  ) {}

  private emitTranscript(peerKey: string): void {
    this.hooks.onTranscriptChanged?.(peerKey, this.transcripts.thread(peerKey));
  }

  private emitOperation(peerKey: string): void {
    this.hooks.onOperationChanged?.(peerKey);
  }

  private knownRuntimeIds(): string[] {
    return [...new Set([
      ...Object.values(this.workspace.snapshot()),
      ...Object.values(this.workspace.requestSnapshot()),
    ])];
  }

  private unambiguousRuntimeId(): string | undefined {
    const ids = this.knownRuntimeIds();
    return ids.length === 1 ? ids[0] : undefined;
  }

  private hasAgentWork(): boolean {
    return this.knownRuntimeIds().length > 0;
  }

  beginLocalTurn(input: AgentLocalTurn): void {
    this.workspace.beginRequest(input.peerKey, input.requestId);
    this.transcripts.appendUserMessage(input.peerKey, {
      id: input.messageId,
      text: input.text,
      createdAtMs: input.createdAtMs,
      operationId: input.requestId,
      optimistic: true,
      queued: false,
      attachments: input.attachments,
    });
    this.transcripts.appendAssistantTurnEvent(input.peerKey, {
      type: 'operation.started',
      timestamp: new Date(input.createdAtMs).toISOString(),
      operationId: input.requestId,
      label: '正在思考',
      interruptible: true,
    });
    this.emitTranscript(input.peerKey);
    this.emitOperation(input.peerKey);
  }

  cancelLocalTurn(peerKey: string, requestId: string, messageId: string): void {
    this.transcripts.removeByIds(peerKey, [messageId, `${requestId}:assistant-turn`]);
    this.workspace.cancelRequest(requestId);
    this.emitTranscript(peerKey);
    this.emitOperation(peerKey);
  }

  adoptOperation(
    requestId: string | null | undefined,
    operationId: string,
    fallbackPeerKey?: string | null,
  ): string | null {
    const peerKey = (requestId ? this.workspace.peerForRequest(requestId) : null)
      ?? fallbackPeerKey
      ?? this.workspace.peerForOperation(operationId);
    if (!peerKey) return null;
    if (requestId && requestId !== operationId) {
      this.transcripts.adoptOperation(peerKey, requestId, operationId);
    }
    this.workspace.adoptOperation(requestId, operationId, peerKey);
    this.transcripts.markUserOperationAccepted(peerKey, operationId);
    this.emitTranscript(peerKey);
    this.emitOperation(peerKey);
    return peerKey;
  }

  claimOperation(operationId: string, fallbackPeerKey?: string | null): string | null {
    if (!operationId || this.workspace.isOperationFinished(operationId)) return null;
    const alreadyOwned = this.workspace.peerForOperation(operationId);
    if (alreadyOwned) return alreadyOwned;

    const fallback = this.workspace.peerForRequest(operationId)
      ?? fallbackPeerKey
      ?? this.workspace.onlyPendingPeer();
    const requestId = fallback ? this.workspace.requestForPeer(fallback) : null;
    const peerKey = this.workspace.claimRuntimeOperation(operationId, fallback);
    if (!peerKey) return null;

    if (requestId && requestId !== operationId) {
      this.transcripts.adoptOperation(peerKey, requestId, operationId);
    }
    this.transcripts.markUserOperationAccepted(peerKey, operationId);
    this.emitTranscript(peerKey);
    this.emitOperation(peerKey);
    return peerKey;
  }

  private appendAssistantEvent(peerKey: string, event: RuntimeEvent): void {
    this.transcripts.appendAssistantTurnEvent(peerKey, event);
    this.emitTranscript(peerKey);
  }

  private scheduleDeltaFlush(): void {
    if (this.deltaTimer !== null) return;
    this.deltaTimer = globalThis.setTimeout(() => {
      this.deltaTimer = null;
      this.flushPendingDeltas();
    }, 16);
  }

  private queueDelta(event: AgentDeltaEvent): void {
    const current = this.pendingDeltas.get(event.operationId);
    this.pendingDeltas.set(event.operationId, current
      ? { ...event, delta: `${current.delta}${event.delta}` }
      : event);
    this.scheduleDeltaFlush();
  }

  flushPendingDeltas(operationId?: string): void {
    const ids = operationId ? [operationId] : [...this.pendingDeltas.keys()];
    for (const id of ids) {
      const event = this.pendingDeltas.get(id);
      if (!event) continue;
      this.pendingDeltas.delete(id);
      const peerKey = this.workspace.peerForRuntimeId(id);
      if (peerKey) this.appendAssistantEvent(peerKey, event);
    }
    if (this.pendingDeltas.size === 0 && this.deltaTimer !== null) {
      globalThis.clearTimeout(this.deltaTimer);
      this.deltaTimer = null;
    }
  }

  private finishOperation(
    operationId: string,
    status: AgentTranscriptTerminalStatus,
    message?: string,
  ): string | null {
    this.flushPendingDeltas(operationId);
    const peerKey = this.workspace.peerForOperation(operationId);
    if (!peerKey) return null;
    const finishedPeer = this.workspace.finishRuntimeOperation(operationId);
    if (!finishedPeer) return null;
    this.transcripts.finishOperation(finishedPeer, operationId, status);
    this.emitTranscript(finishedPeer);
    this.emitOperation(finishedPeer);
    this.hooks.onOperationTerminal?.(finishedPeer, operationId, status, message);
    return finishedPeer;
  }

  handle(event: RuntimeEvent): boolean {
    if (
      event.type === 'chat.message'
      || event.type === 'agent.step'
      || event.type === 'operation.completed'
      || event.type === 'operation.failed'
      || event.type === 'operation.interrupted'
    ) {
      const operationId = ('operationId' in event && typeof event.operationId === 'string'
        ? event.operationId
        : this.unambiguousRuntimeId()) ?? undefined;
      if (operationId) this.flushPendingDeltas(operationId);
    }

    switch (event.type) {
      case 'chat.message': {
        const operationId = event.operationId ?? this.unambiguousRuntimeId();
        if (event.role === 'assistant') {
          if (!operationId) return this.hasAgentWork();
          if (this.workspace.isOperationFinished(operationId)) return true;
          const peerKey = this.claimOperation(operationId);
          if (!peerKey) return this.hasAgentWork();
          this.appendAssistantEvent(peerKey, { ...event, operationId });
          return true;
        }

        if (!operationId) return false;
        const peerKey = this.claimOperation(operationId);
        if (!peerKey) return false;
        this.transcripts.reconcileUserMessage(peerKey, event.text, operationId);
        this.emitTranscript(peerKey);
        return true;
      }

      case 'chat.delta': {
        const operationId = event.operationId ?? this.unambiguousRuntimeId();
        if (!operationId) return this.hasAgentWork();
        if (this.workspace.isOperationFinished(operationId)) return true;
        const peerKey = this.claimOperation(operationId);
        if (!peerKey) return this.hasAgentWork();
        this.queueDelta({ ...event, operationId });
        return true;
      }

      case 'operation.started': {
        const peerKey = this.claimOperation(event.operationId);
        if (!peerKey) return false;
        this.appendAssistantEvent(peerKey, event);
        this.hooks.onOperationStarted?.(peerKey, event.operationId);
        return true;
      }

      case 'model.routed':
      case 'agent.step': {
        const peerKey = this.claimOperation(event.operationId);
        if (!peerKey) return false;
        this.appendAssistantEvent(peerKey, event);
        return true;
      }

      case 'operation.interrupted': {
        const peerKey = this.workspace.peerForOperation(event.operationId);
        if (!peerKey) return this.workspace.isOperationFinished(event.operationId);
        this.appendAssistantEvent(peerKey, event);
        return Boolean(this.finishOperation(event.operationId, 'interrupted'));
      }

      case 'operation.completed': {
        const peerKey = this.workspace.peerForOperation(event.operationId);
        if (!peerKey) return this.workspace.isOperationFinished(event.operationId);
        this.appendAssistantEvent(peerKey, event);
        return Boolean(this.finishOperation(event.operationId, 'completed'));
      }

      case 'operation.failed': {
        const peerKey = this.workspace.peerForOperation(event.operationId);
        if (!peerKey) return this.workspace.isOperationFinished(event.operationId);
        this.appendAssistantEvent(peerKey, event);
        return Boolean(this.finishOperation(event.operationId, 'failed', event.message));
      }

      default:
        return false;
    }
  }

  resetOperations(): void {
    if (this.deltaTimer !== null) globalThis.clearTimeout(this.deltaTimer);
    this.deltaTimer = null;
    this.pendingDeltas.clear();
    this.workspace.clear();
  }

  dispose(): void {
    if (this.deltaTimer !== null) globalThis.clearTimeout(this.deltaTimer);
    this.deltaTimer = null;
    this.pendingDeltas.clear();
  }
}
