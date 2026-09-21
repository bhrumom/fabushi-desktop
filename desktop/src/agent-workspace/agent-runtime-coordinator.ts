import type { AttachmentContext, ComputerStatus, RuntimeEvent } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MahayanaCommandBridgeDetail } from '../../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
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
  readonly richText?: string;
  readonly createdAtMs: number;
  readonly attachments?: readonly AttachmentContext[];
}

export interface AgentRuntimeCoordinatorHooks {
  onTranscriptChanged?(peerKey: string, thread: readonly AgentTranscriptSourceMessage[]): void;
  onOperationChanged?(peerKey: string): void;
  onComputerStatus?(status: ComputerStatus): void;
  onWorkspaceState?(key: string, value: unknown): void;
  onOperationStarted?(peerKey: string, operationId: string): void;
  onRequestFailed?(peerKey: string, requestId: string, message: string): void;
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
  private lastRecoveredGeneration = -1;
  private readonly peerByAgentId = new Map<string, string>();
  private readonly peerByConversationId = new Map<string, string>();
  private readonly turnStateByOperation = new Map<string, Extract<RuntimeEvent, { type: 'turn.state' }>['state']>();
  private readonly recoveryMessageByPeer = new Map<string, string>();

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

  bindAgentPeers(bindings: readonly { agentId: string; peerKey: string; conversationId?: string }[]): void {
    this.peerByAgentId.clear();
    this.peerByConversationId.clear();
    for (const binding of bindings) {
      if (binding.agentId.trim() && binding.peerKey.trim()) this.peerByAgentId.set(binding.agentId, binding.peerKey);
      if (binding.conversationId?.trim() && binding.peerKey.trim()) {
        this.peerByConversationId.set(binding.conversationId, binding.peerKey);
      }
    }
  }

  /**
   * Command bridge context is transport-facing: conversationKey is normally a
   * Rust conversation id (for example `codex:agent:research`), while the
   * workspace registry is keyed by the canonical UI peer (for example
   * `agent:research`). Resolve through the Agent directory bindings before
   * touching request ownership so one request can never be registered under
   * both keys.
   */
  private peerForCommandBridge(detail: MahayanaCommandBridgeDetail): string | null {
    const context = detail.context;
    const agentId = context?.agentId?.trim();
    if (agentId) {
      const peerKey = this.peerByAgentId.get(agentId);
      if (peerKey) return peerKey;
    }

    const conversationId = context?.conversationId?.trim() || context?.conversationKey?.trim();
    if (conversationId) {
      const peerKey = this.peerByConversationId.get(conversationId);
      if (peerKey) return peerKey;
    }

    const candidate = context?.conversationKey?.trim();
    if (!candidate) return null;
    return [...this.peerByAgentId.values()].includes(candidate) ? candidate : null;
  }

  beginLocalTurn(input: AgentLocalTurn): void {
    this.recoveryMessageByPeer.delete(input.peerKey);
    this.workspace.beginRequest(input.peerKey, input.requestId);
    this.transcripts.appendUserMessage(input.peerKey, {
      id: input.messageId,
      text: input.text,
      richText: input.richText,
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
    this.turnStateByOperation.delete(operationId);
    this.emitTranscript(finishedPeer);
    this.emitOperation(finishedPeer);
    this.hooks.onOperationTerminal?.(finishedPeer, operationId, status, message);
    return finishedPeer;
  }

  private recoverInterruptedOperations(event: Extract<RuntimeEvent, { type: 'host.lifecycle' }>): void {
    if (event.generation === this.lastRecoveredGeneration) return;
    this.lastRecoveredGeneration = event.generation;
    this.flushPendingDeltas();

    const operations = this.workspace.snapshot();
    const requests = this.workspace.requestSnapshot();
    const touched = new Set<string>();
    for (const [peerKey, operationId] of Object.entries(operations)) {
      if (this.turnStateByOperation.get(operationId) === 'waiting-user') {
        this.transcripts.appendAssistantTurnEvent(peerKey, {
          type: 'operation.interrupted',
          timestamp: event.timestamp,
          operationId,
        });
        this.transcripts.finishOperation(peerKey, operationId, 'interrupted');
        this.workspace.finishRuntimeOperation(operationId);
        this.turnStateByOperation.delete(operationId);
        touched.add(peerKey);
        this.hooks.onOperationTerminal?.(
          peerKey,
          operationId,
          'interrupted',
          event.reason || event.error || 'Agent runtime restarted while waiting for user approval.',
        );
        continue;
      }

      const recoveryMessageId = this.transcripts.prepareOperationRecovery(peerKey, operationId);
      if (recoveryMessageId) this.recoveryMessageByPeer.set(peerKey, recoveryMessageId);
      this.workspace.finishRuntimeOperation(operationId);
      this.turnStateByOperation.delete(operationId);
      touched.add(peerKey);
    }
    for (const [peerKey, requestId] of Object.entries(requests)) {
      this.workspace.cancelRequest(requestId);
      touched.add(peerKey);
      this.hooks.onRequestFailed?.(peerKey, requestId, event.reason || event.error || 'Agent runtime restarted.');
    }
    this.workspace.clearOperations({ preserveFinished: true });
    for (const peerKey of touched) {
      this.emitTranscript(peerKey);
      this.emitOperation(peerKey);
    }
  }

  handleCommandBridge(detail: MahayanaCommandBridgeDetail): boolean {
    if (detail.command.type !== 'chat.send') return false;
    const peerKey = this.peerForCommandBridge(detail);
    if (!peerKey) return false;

    const requestId = detail.command.requestId;
    if (detail.phase === 'dispatch') {
      if (!this.workspace.isBusy(peerKey)) this.workspace.beginRequest(peerKey, requestId);
      this.emitOperation(peerKey);
      return true;
    }
    if (detail.phase === 'accepted') {
      const operationId = detail.accepted.operationId;
      if (!operationId) return true;
      this.adoptOperation(requestId, operationId, peerKey);
      this.handle({
        type: 'operation.started',
        timestamp: new Date().toISOString(),
        operationId,
        label: '正在思考',
        interruptible: true,
      });
      return true;
    }

    this.workspace.cancelRequest(requestId);
    this.emitOperation(peerKey);
    this.hooks.onRequestFailed?.(peerKey, requestId, detail.error);
    return true;
  }

  handle(event: RuntimeEvent): boolean {
    if (
      event.type === 'chat.message'
      || event.type === 'agent.step'
      || event.type === 'approval.requested'
      || event.type === 'approval.resolved'
      || event.type === 'turn.state'
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
      case 'host.lifecycle': {
        if (['restarting', 'stopped', 'spawn-failed', 'protocol-error'].includes(event.lifecycle)) {
          this.recoverInterruptedOperations(event);
        }
        return true;
      }

      case 'agent.workspaceState': {
        this.hooks.onWorkspaceState?.(event.key, event.value);
        return true;
      }

      case 'computer.status': {
        this.hooks.onComputerStatus?.(event.status);
        return true;
      }

      case 'computer.snapshot':
      case 'computer.result': {
        if (!event.agentId) return false;
        const peerKey = this.peerByAgentId.get(event.agentId);
        if (!peerKey) return false;
        this.transcripts.appendComputerHandoff(peerKey, event);
        this.emitTranscript(peerKey);
        return true;
      }

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

      case 'turn.state': {
        const recoveryPeerKey = event.state === 'recovering'
          ? this.peerByConversationId.get(event.conversationId)
          : undefined;
        const peerKey = this.workspace.peerForOperation(event.operationId)
          ?? this.claimOperation(event.operationId, recoveryPeerKey);
        if (!peerKey) return this.workspace.isOperationFinished(event.operationId);
        if (event.state === 'recovering') {
          const messageId = this.recoveryMessageByPeer.get(peerKey);
          if (messageId) {
            this.transcripts.adoptRecoveredOperation(peerKey, messageId, event.operationId);
            this.recoveryMessageByPeer.delete(peerKey);
          }
        }
        this.turnStateByOperation.set(event.operationId, event.state);
        this.transcripts.applyTurnState(peerKey, event);
        this.emitTranscript(peerKey);
        if (event.state === 'completed') {
          return Boolean(this.finishOperation(event.operationId, 'completed'));
        }
        if (event.state === 'failed') {
          return Boolean(this.finishOperation(event.operationId, 'failed'));
        }
        if (event.state === 'cancelled') {
          return Boolean(this.finishOperation(event.operationId, 'interrupted'));
        }
        if (event.state === 'preparing') {
          this.hooks.onOperationStarted?.(peerKey, event.operationId);
        }
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
        const operationId = event.operationId ?? this.unambiguousRuntimeId();
        if (!operationId) return this.hasAgentWork();
        const peerKey = this.claimOperation(operationId);
        if (!peerKey) return this.hasAgentWork();
        this.appendAssistantEvent(peerKey, { ...event, operationId });
        return true;
      }

      case 'approval.requested': {
        if (!event.operationId) return false;
        if (this.workspace.isOperationFinished(event.operationId)) return true;
        const peerKey = this.claimOperation(event.operationId);
        if (!peerKey) return this.hasAgentWork();
        this.transcripts.appendApprovalRequested(peerKey, event);
        this.emitTranscript(peerKey);
        return true;
      }

      case 'approval.resolved': {
        if (!event.operationId) return false;
        const peerKey = this.workspace.peerForOperation(event.operationId)
          ?? this.claimOperation(event.operationId);
        if (!peerKey) return this.workspace.isOperationFinished(event.operationId);
        this.transcripts.resolveApproval(peerKey, event);
        this.emitTranscript(peerKey);
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
    this.turnStateByOperation.clear();
    this.recoveryMessageByPeer.clear();
    this.workspace.clearOperations();
  }

  dispose(): void {
    if (this.deltaTimer !== null) globalThis.clearTimeout(this.deltaTimer);
    this.deltaTimer = null;
    this.pendingDeltas.clear();
    this.turnStateByOperation.clear();
    this.recoveryMessageByPeer.clear();
  }
}