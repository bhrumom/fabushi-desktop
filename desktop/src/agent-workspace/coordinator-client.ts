import type { AttachmentContext } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MahayanaHostTransport } from '../../../frontend/apps/web/src/lib/mahayana-host/transport';
import { composeAgentPromptText, type AgentReplyContext } from './prompt-context';

type HostCommand = Parameters<MahayanaHostTransport['execute']>[0];

export interface AgentPromptRequest {
  readonly requestId: string;
  readonly text: string;
  readonly conversationId?: string;
  readonly agentId?: string;
  readonly attachments?: readonly AttachmentContext[];
  readonly replyTo?: AgentReplyContext;
}

export interface AgentAttachmentUpload {
  readonly requestId: string;
  readonly agentId: string;
  readonly filename: string;
  readonly mimeType?: string;
  readonly bytesBase64: string;
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
      text: composeAgentPromptText(request.text, request.replyTo),
      conversationId: request.conversationId,
      agentId: request.agentId,
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

  broadcast(requestId: string, message: string, targetIds?: readonly string[]) {
    return this.transport.execute({
      type: 'agent.broadcast',
      requestId,
      ...(targetIds?.length ? { targetIds: [...targetIds] } : {}),
      message,
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

  interrupt(operationId: string): Promise<void> {
    return this.transport.interrupt(operationId);
  }
}
