import type { InferenceProvider } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { ApprovalResolution, AttachmentContext, AuthState, HostConfig, HostInfo, RuntimeEvent } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MahayanaHostTransport } from '../../../frontend/apps/web/src/lib/mahayana-host/transport';
import { composeAgentPromptText, type AgentPromptReference, type AgentReplyContext } from './prompt-context';

type HostCommand = Parameters<MahayanaHostTransport['execute']>[0];

export interface AgentPromptRequest {
  readonly requestId: string;
  readonly text: string;
  readonly conversationId?: string;
  readonly agentId?: string;
  readonly clientMessageId?: string;
  readonly retryOfMessageId?: string;
  readonly attachments?: readonly AttachmentContext[];
  readonly references?: readonly AgentPromptReference[];
  readonly replyTo?: AgentReplyContext;
}

export interface AgentAttachmentUpload {
  readonly requestId: string;
  readonly agentId: string;
  readonly filename: string;
  readonly mimeType?: string;
  readonly bytesBase64: string;
}

export type AgentCoordinatorConnectionPhase = 'connecting' | 'ready' | 'recovering' | 'closed';

export interface AgentCoordinatorConnectionState {
  readonly phase: AgentCoordinatorConnectionPhase;
  readonly generation: number;
  readonly attempt: number;
  readonly recovered: boolean;
  readonly error?: string;
}

export interface AgentCoordinatorConnection {
  readonly ready: Promise<HostInfo>;
  dispose(): Promise<void>;
}

export interface AgentCoordinatorConnectionOptions {
  readonly config: HostConfig;
  readonly onEvent: (event: RuntimeEvent) => void;
  readonly onState?: (state: AgentCoordinatorConnectionState) => void;
}

/**
 * Narrow renderer -> Mahayana boundary for Agent lifecycle operations.
 *
 * Messenger compatibility code may still use the host transport for social
 * features, but Agent workspace code goes through this client so reconnect,
 * request adoption and event-family handling can migrate here without touching
 * the React surface.
 */
export class AgentCoordinatorClient {
  constructor(private readonly transport: MahayanaHostTransport) {}

  /**
   * Own the long-lived transport subscription + Host initialization lifecycle.
   * React surfaces receive projected events, but do not directly subscribe to
   * or close the Host transport.
   */
  connect(options: AgentCoordinatorConnectionOptions): AgentCoordinatorConnection {
    let disposed = false;
    let everReady = false;
    let observedGeneration = 0;
    let readyGeneration = 0;
    let retryAttempt = 0;
    let retryTimer: ReturnType<typeof globalThis.setTimeout> | null = null;
    let handshake: Promise<void> | null = null;
    let resolveFirstReady: (info: HostInfo) => void = () => {};
    const ready = new Promise<HostInfo>((resolve) => { resolveFirstReady = resolve; });

    const emitState = (
      phase: AgentCoordinatorConnectionPhase,
      error?: unknown,
      recovered = everReady,
    ) => {
      options.onState?.({
        phase,
        generation: observedGeneration,
        attempt: retryAttempt,
        recovered,
        ...(error ? { error: error instanceof Error ? error.message : String(error) } : {}),
      });
    };

    const clearRetry = () => {
      if (retryTimer !== null) globalThis.clearTimeout(retryTimer);
      retryTimer = null;
    };

    const retryDelay = () => Math.min(3_000, [100, 250, 500, 1_000, 2_000, 3_000][Math.min(retryAttempt, 5)] ?? 3_000);

    const scheduleRecovery = (cause?: unknown) => {
      if (disposed || retryTimer !== null) return;
      emitState(everReady ? 'recovering' : 'connecting', cause);
      const delay = retryDelay();
      retryAttempt += 1;
      retryTimer = globalThis.setTimeout(() => {
        retryTimer = null;
        void performHandshake();
      }, delay);
    };

    const performHandshake = async () => {
      if (disposed || handshake) return handshake ?? Promise.resolve();
      emitState(everReady ? 'recovering' : 'connecting');
      const current = this.transport.initialize(options.config)
        .then((info) => {
          if (disposed) return;
          clearRetry();
          retryAttempt = 0;
          readyGeneration = Math.max(readyGeneration, observedGeneration);
          const recovered = everReady;
          if (!everReady) resolveFirstReady(info);
          everReady = true;
          emitState('ready', undefined, recovered);
        })
        .catch((cause: unknown) => {
          if (!disposed) scheduleRecovery(cause);
        })
        .finally(() => {
          if (handshake === current) handshake = null;
        });
      handshake = current;
      return current;
    };

    const unsubscribe = this.transport.subscribe((event) => {
      if (disposed) return;
      if (event.type === 'host.lifecycle') {
        observedGeneration = Math.max(observedGeneration, event.generation);
        const unavailable = ['restarting', 'stopped', 'spawn-failed', 'protocol-error', 'request-timeout'].includes(event.lifecycle);
        if (unavailable) scheduleRecovery(event.error || event.reason || `Host ${event.lifecycle}`);
        if (event.lifecycle === 'running' && event.generation > readyGeneration) {
          clearRetry();
          void performHandshake();
        }
      }
      options.onEvent(event);
    });

    void performHandshake();
    return {
      ready,
      dispose: async () => {
        if (disposed) return;
        disposed = true;
        clearRetry();
        unsubscribe();
        emitState('closed');
        await this.transport.close();
      },
    };
  }

  authStatus(): Promise<AuthState> {
    return this.transport.authStatus();
  }

  send(request: AgentPromptRequest) {
    return this.transport.execute({
      type: 'chat.send',
      requestId: request.requestId,
      text: composeAgentPromptText(
        request.text.trim() || (request.attachments?.length ? 'Please review the attached file(s).' : request.text),
        request.replyTo,
        request.references,
      ),
      conversationId: request.conversationId,
      agentId: request.agentId,
      ...(request.clientMessageId ? { clientMessageId: request.clientMessageId } : {}),
      ...(request.retryOfMessageId ? { retryOfMessageId: request.retryOfMessageId } : {}),
      ...(request.attachments?.length ? { attachments: [...request.attachments] } : {}),
    } as HostCommand);
  }

  openConversation(requestId: string, conversationId: string) {
    return this.transport.execute({
      type: 'conversation.open',
      requestId,
      conversationId,
    } as HostCommand);
  }

  readWorkspaceState(requestId: string, key: string) {
    return this.transport.execute({
      type: 'agent.workspaceState.get',
      requestId,
      key,
    } as HostCommand);
  }

  writeWorkspaceState(requestId: string, key: string, value: unknown) {
    return this.transport.execute({
      type: 'agent.workspaceState.set',
      requestId,
      key,
      value,
    } as HostCommand);
  }

  listAgents(requestId: string) {
    return this.transport.execute({
      type: 'bot.list',
      requestId,
    } as HostCommand);
  }

  createAgent(requestId: string, input: { name: string; description?: string }) {
    return this.transport.execute({
      type: 'bot.create',
      requestId,
      name: input.name,
      description: input.description ?? '',
    } as HostCommand);
  }

  updateAgent(
    requestId: string,
    id: string,
    patch: {
      name?: string;
      title?: string;
      description?: string;
      avatar?: string;
      avatarShape?: string;
      avatarColor?: string;
      notifyOnUpdates?: boolean;
      notificationsEnabled?: boolean;
      inferenceProvider?: InferenceProvider;
      clearInferenceProvider?: boolean;
    },
  ) {
    return this.transport.execute({
      type: 'bot.update',
      requestId,
      id,
      ...patch,
    } as HostCommand);
  }

  duplicateAgent(requestId: string, id: string) {
    return this.transport.execute({
      type: 'bot.clone',
      requestId,
      id,
    } as HostCommand);
  }

  deleteAgent(requestId: string, id: string) {
    return this.transport.execute({
      type: 'bot.delete',
      requestId,
      id,
    } as HostCommand);
  }

  setAgentHidden(requestId: string, id: string, hidden: boolean) {
    return this.transport.execute({
      type: 'bot.setHidden',
      requestId,
      id,
      hidden,
    } as HostCommand);
  }

  sendAgentPeer(
    requestId: string,
    fromAgentId: string,
    targetId: string,
    text: string,
    priority = false,
  ) {
    return this.transport.execute({
      type: 'agent.send',
      requestId,
      fromAgentId,
      targetId,
      text,
      priority,
    } as HostCommand);
  }

  listAgentPeerHistory(requestId: string, agentId: string, limit = 100) {
    return this.transport.execute({
      type: 'agent.peerHistory',
      requestId,
      agentId,
      limit,
    } as HostCommand);
  }

  broadcast(requestId: string, message: string, targetIds?: readonly string[]) {
    return this.transport.execute({
      type: 'agent.broadcast',
      requestId,
      ...(targetIds?.length ? { targetIds: [...targetIds] } : {}),
      message,
    } as HostCommand);
  }

  listGroups(requestId: string) {
    return this.transport.execute({
      type: 'group.list',
      requestId,
    } as HostCommand);
  }

  createGroup(
    requestId: string,
    name: string,
    memberAgentIds: readonly string[],
    description = '',
  ) {
    return this.transport.execute({
      type: 'group.create',
      requestId,
      name,
      description,
      memberIds: [...memberAgentIds],
    } as HostCommand);
  }

  updateGroup(
    requestId: string,
    id: string,
    patch: { name?: string; description?: string; memberAgentIds?: readonly string[] },
  ) {
    return this.transport.execute({
      type: 'group.update',
      requestId,
      id,
      ...(patch.name !== undefined ? { name: patch.name } : {}),
      ...(patch.description !== undefined ? { description: patch.description } : {}),
      ...(patch.memberAgentIds !== undefined ? { memberIds: [...patch.memberAgentIds] } : {}),
    } as HostCommand);
  }

  deleteGroup(requestId: string, id: string) {
    return this.transport.execute({
      type: 'group.delete',
      requestId,
      id,
    } as HostCommand);
  }

  sendGroup(requestId: string, id: string, message: string) {
    return this.transport.execute({
      type: 'group.send',
      requestId,
      id,
      text: message,
    } as HostCommand);
  }

  listWorkflows(requestId: string, agentId: string) {
    return this.transport.execute({
      type: 'workflow.list',
      requestId,
      agentId,
    } as HostCommand);
  }

  listMcpServers(requestId: string) {
    return this.transport.execute({
      type: 'mcp.list',
      requestId,
    } as HostCommand);
  }

  refreshMcpServers(requestId: string) {
    return this.transport.execute({
      type: 'mcp.refresh',
      requestId,
    } as HostCommand);
  }

  mcpOauthLogin(requestId: string, server: string) {
    return this.transport.execute({
      type: 'mcp.oauthLogin',
      requestId,
      server,
    } as HostCommand);
  }

  mcpOauthLogout(requestId: string, server: string) {
    return this.transport.execute({
      type: 'mcp.oauthLogout',
      requestId,
      server,
    } as HostCommand);
  }

  removeMcpServer(requestId: string, server: string) {
    return this.transport.execute({
      type: 'mcp.remove',
      requestId,
      server,
    } as HostCommand);
  }

  setMcpCustomInstructions(requestId: string, server: string, instructions: string) {
    return this.transport.execute({
      type: 'mcp.setCustomInstructions',
      requestId,
      server,
      instructions,
    } as HostCommand);
  }

  setMcpToolDisabled(requestId: string, server: string, tool: string, disabled: boolean) {
    return this.transport.execute({
      type: 'mcp.setToolDisabled',
      requestId,
      server,
      tool,
      disabled,
    } as HostCommand);
  }

  async callMcpTool(
    requestId: string,
    server: string,
    tool: string,
    argumentsValue: unknown = {},
    timeoutMs = 10_000,
  ): Promise<unknown> {
    let cancelWait = () => {};
    const result = new Promise<unknown>((resolve, reject) => {
      const timeout = globalThis.setTimeout(() => {
        unsubscribe();
        reject(new Error(`Timed out waiting for MCP tool result: ${server}/${tool}`));
      }, timeoutMs);
      const unsubscribe = this.transport.subscribe((event) => {
        if (
          event.type !== 'mcp.toolResult'
          || event.requestId !== requestId
          || event.server !== server
          || event.tool !== tool
        ) return;
        globalThis.clearTimeout(timeout);
        unsubscribe();
        resolve(event.result);
      });
      cancelWait = () => {
        globalThis.clearTimeout(timeout);
        unsubscribe();
      };
    });

    try {
      await this.transport.execute({
        type: 'mcp.toolCall',
        requestId,
        server,
        tool,
        arguments: argumentsValue,
      } as HostCommand);
    } catch (cause) {
      cancelWait();
      void result.catch(() => {});
      throw cause;
    }
    return result;
  }

  listMemory(requestId: string, agentId: string, limit = 1000) {
    return this.transport.execute({
      type: 'memory.list',
      requestId,
      agentId,
      limit,
    } as HostCommand);
  }

  refreshComputerStatus(requestId: string) {
    return this.transport.execute({
      type: 'computer.status',
      requestId,
    } as HostCommand);
  }

  takeComputerControl(requestId: string, agentId: string, leaseId: string) {
    return this.transport.execute({
      type: 'computer.takeControl',
      requestId,
      agentId,
      leaseId,
    } as HostCommand);
  }

  releaseComputerControl(requestId: string, agentId: string, leaseId: string) {
    return this.transport.execute({
      type: 'computer.releaseControl',
      requestId,
      agentId,
      leaseId,
    } as HostCommand);
  }

  async uploadAttachment(input: AgentAttachmentUpload, timeoutMs = 20_000): Promise<AttachmentContext> {
    let cancelWait = () => {};
    const stored = new Promise<AttachmentContext>((resolve, reject) => {
      let unsubscribe = () => {};
      const timeout = globalThis.setTimeout(() => {
        unsubscribe();
        reject(new Error(`Timed out while storing ${input.filename} for Agent ${input.agentId}.`));
      }, timeoutMs);
      cancelWait = () => {
        globalThis.clearTimeout(timeout);
        unsubscribe();
      };
      unsubscribe = this.transport.subscribe((event) => {
        if (
          event.type !== 'attachment.stored'
          || event.attachment.agentId !== input.agentId
          || event.attachment.name !== input.filename
        ) return;
        cancelWait();
        resolve({
          id: event.attachment.id,
          name: event.attachment.name,
          mimeType: event.attachment.mimeType,
          path: event.attachment.path,
          sizeBytes: event.attachment.sizeBytes,
        });
      });
    });

    try {
      await this.transport.execute({
        type: 'attachment.upload',
        requestId: input.requestId,
        agentId: input.agentId,
        filename: input.filename,
        mimeType: input.mimeType,
        bytesBase64: input.bytesBase64,
      } as HostCommand);
    } catch (cause) {
      cancelWait();
      void stored.catch(() => {});
      throw cause;
    }
    return stored;
  }

  resolveApproval(resolution: ApprovalResolution): Promise<void> {
    return this.transport.resolveApproval(resolution);
  }

  interrupt(operationId: string): Promise<void> {
    return this.transport.interrupt(operationId);
  }
}