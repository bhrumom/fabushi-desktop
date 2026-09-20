import type { MahayanaHostTransport } from '../../../frontend/apps/web/src/lib/mahayana-host/transport';

type HostCommand = Parameters<MahayanaHostTransport['execute']>[0];

export interface AgentPromptRequest {
  readonly requestId: string;
  readonly text: string;
  readonly conversationId?: string;
  readonly agentId?: string;
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

  send(request: AgentPromptRequest) {
    return this.transport.execute({
      type: 'chat.send',
      requestId: request.requestId,
      text: request.text,
      conversationId: request.conversationId,
      agentId: request.agentId,
    } as HostCommand);
  }

  openConversation(requestId: string, conversationId: string) {
    return this.transport.execute({
      type: 'conversation.open',
      requestId,
      conversationId,
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

  interrupt(operationId: string): Promise<void> {
    return this.transport.interrupt(operationId);
  }
}
