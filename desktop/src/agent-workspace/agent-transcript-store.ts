import type { RuntimeEvent } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  assistantTurnPlainText,
  createAssistantTurn,
  reduceAssistantTurn,
  type AssistantTurn,
} from '../mahayana-assistant-turn';
import {
  projectTranscriptEntries,
  type TranscriptEntry,
  type TranscriptSourceMessage,
} from './transcript-model';

export interface AgentTranscriptSourceMessage extends TranscriptSourceMessage {
  readonly source: 'legacy';
}

export type AgentTranscriptTerminalStatus = 'completed' | 'failed' | 'interrupted';

function assistantTurnRank(message: AgentTranscriptSourceMessage): number {
  const turn = message.assistantTurn;
  const terminal = turn?.status === 'completed' ? 4 : turn?.status === 'failed' ? 3 : turn?.status === 'interrupted' ? 2 : 1;
  return (turn?.updatedAtMs ?? message.createdAtMs) * 10_000 + terminal * 1_000 + message.text.length;
}

function canonicalizeThread(messages: readonly AgentTranscriptSourceMessage[]): AgentTranscriptSourceMessage[] {
  const selectedByOperation = new Map<string, number>();
  messages.forEach((message, index) => {
    if (message.kind !== 'assistant-turn' || !message.operationId) return;
    const previousIndex = selectedByOperation.get(message.operationId);
    if (previousIndex == null || assistantTurnRank(message) >= assistantTurnRank(messages[previousIndex]!)) {
      selectedByOperation.set(message.operationId, index);
    }
  });

  const next = [...messages];
  for (const [operationId, index] of selectedByOperation) {
    const selected = next[index];
    if (!selected?.assistantTurn || assistantTurnPlainText(selected.assistantTurn).trim()) continue;
    const legacyFinal = [...messages].reverse().find((message) =>
      message.kind === 'message'
      && message.role === 'peer'
      && message.operationId === operationId
      && message.text.trim().length > 0,
    );
    if (!legacyFinal) continue;
    const assistantTurn: AssistantTurn = {
      ...selected.assistantTurn,
      updatedAtMs: Math.max(selected.assistantTurn.updatedAtMs, legacyFinal.createdAtMs),
      parts: [
        ...selected.assistantTurn.parts.filter((part) => part.kind !== 'text'),
        {
          id: `${operationId}:final-text`,
          kind: 'text',
          text: legacyFinal.text,
          status: 'completed',
        },
      ],
    };
    next[index] = { ...selected, text: legacyFinal.text, assistantTurn };
  }

  return next.filter((message, index) => {
    if (message.kind === 'assistant-turn' && message.operationId) {
      return selectedByOperation.get(message.operationId) === index;
    }
    if (
      message.kind === 'message'
      && message.role === 'peer'
      && message.operationId
      && selectedByOperation.has(message.operationId)
    ) return false;
    return true;
  });
}

function rekeyAssistantTurn(turn: AssistantTurn, operationId: string): AssistantTurn {
  if (turn.operationId === operationId) return turn;
  const previousPrefix = `${turn.operationId}:`;
  const nextPrefix = `${operationId}:`;
  return {
    ...turn,
    id: `assistant-turn:${operationId}`,
    operationId,
    parts: turn.parts.map((part) => ({
      ...part,
      id: part.id.startsWith(previousPrefix)
        ? `${nextPrefix}${part.id.slice(previousPrefix.length)}`
        : part.id,
    })),
  };
}

/**
 * Agent-owned transcript source store.
 *
 * Runtime events are reduced here, outside the Messenger renderer. The store
 * intentionally accepts the temporary legacy source shape only at its outer
 * boundary; views consume canonical TranscriptEntry projections.
 */
export class AgentTranscriptStore {
  private readonly threads = new Map<string, AgentTranscriptSourceMessage[]>();

  has(peerKey: string): boolean {
    return this.threads.has(peerKey);
  }

  clear(peerKey?: string): void {
    if (peerKey) this.threads.delete(peerKey);
    else this.threads.clear();
  }

  thread(peerKey: string): AgentTranscriptSourceMessage[] {
    return [...(this.threads.get(peerKey) ?? [])];
  }

  replace(peerKey: string, messages: readonly AgentTranscriptSourceMessage[]): AgentTranscriptSourceMessage[] {
    const next = canonicalizeThread(messages);
    this.threads.set(peerKey, next);
    return [...next];
  }

  hydrateEntries(peerKey: string, entries: readonly TranscriptEntry[]): AgentTranscriptSourceMessage[] {
    return this.replace(peerKey, entries.map((entry) => ({
      id: entry.id,
      source: 'legacy',
      role: entry.role,
      text: entry.text,
      ...(entry.richText ? { richText: entry.richText } : {}),
      createdAtMs: entry.createdAtMs,
      kind: entry.kind === 'tool-call' ? 'action' : entry.kind,
      ...(entry.operationId ? { operationId: entry.operationId } : {}),
      ...(entry.streaming ? { streaming: true } : {}),
      ...(entry.optimistic ? { optimistic: true } : {}),
      ...(entry.queued ? { queued: true } : {}),
      ...(entry.title ? { actionTitle: entry.title } : {}),
      ...(entry.detail ? { actionDetail: entry.detail } : {}),
      ...(entry.status ? { status: entry.status } : {}),
      ...(entry.assistantTurn ? { assistantTurn: entry.assistantTurn } : {}),
      ...(entry.attachments?.length ? { attachments: [...entry.attachments] } : {}),
      ...(entry.approval ? { approval: entry.approval } : {}),
      ...(entry.miniAppId ? { miniAppId: entry.miniAppId } : {}),
    })));
  }


  update(
    peerKey: string,
    updater: (current: AgentTranscriptSourceMessage[]) => AgentTranscriptSourceMessage[],
  ): AgentTranscriptSourceMessage[] {
    const current = this.thread(peerKey);
    const next = canonicalizeThread(updater(current));
    this.threads.set(peerKey, next);
    return [...next];
  }

  entries(peerKey: string): TranscriptEntry[] {
    return projectTranscriptEntries(this.threads.get(peerKey) ?? []);
  }

  userPromptBefore(peerKey: string, entryId: string): TranscriptEntry | undefined {
    const entries = this.entries(peerKey);
    const index = entries.findIndex((entry) => entry.id === entryId);
    if (index < 0) return undefined;
    for (let cursor = index - 1; cursor >= 0; cursor -= 1) {
      const entry = entries[cursor];
      if (entry.kind === 'message' && entry.role === 'me' && !entry.queued) return entry;
    }
    return undefined;
  }

  appendUserMessage(
    peerKey: string,
    input: {
      readonly id: string;
      readonly text: string;
      readonly richText?: string;
      readonly createdAtMs: number;
      readonly operationId?: string;
      readonly optimistic?: boolean;
      readonly queued?: boolean;
      readonly attachments?: AgentTranscriptSourceMessage['attachments'];
    },
  ): AgentTranscriptSourceMessage[] {
    const nextMessage: AgentTranscriptSourceMessage = {
      id: input.id,
      source: 'legacy',
      role: 'me',
      text: input.text,
      ...(input.richText ? { richText: input.richText } : {}),
      createdAtMs: input.createdAtMs,
      kind: 'message',
      optimistic: input.optimistic === true,
      queued: input.queued === true,
      ...(input.operationId ? { operationId: input.operationId } : {}),
      ...(input.attachments?.length ? { attachments: input.attachments } : {}),
    };
    return this.update(peerKey, (current) => {
      const index = current.findIndex((message) => message.id === input.id);
      if (index < 0) return [...current, nextMessage];
      return current.map((message, messageIndex) =>
        messageIndex === index ? { ...message, ...nextMessage } : message,
      );
    });
  }

  removeByIds(peerKey: string, ids: readonly string[]): AgentTranscriptSourceMessage[] {
    if (!ids.length) return this.thread(peerKey);
    const removed = new Set(ids);
    return this.update(peerKey, (current) => current.filter((message) => !removed.has(message.id)));
  }

  prepareRetry(
    peerKey: string,
    messageId: string,
    operationId?: string,
  ): AgentTranscriptSourceMessage[] {
    return this.update(peerKey, (current) => current.flatMap((message) => {
      if (message.id === messageId && message.role === 'me') {
        const retryMessage = { ...message };
        delete retryMessage.operationId;
        return [{
          ...retryMessage,
          optimistic: false,
          queued: false,
        }];
      }
      if (operationId && message.operationId === operationId) return [];
      return [message];
    }));
  }

  prepareOperationRecovery(peerKey: string, operationId: string): string | undefined {
    const message = [...this.thread(peerKey)].reverse().find((candidate) =>
      candidate.role === 'me'
      && candidate.kind === 'message'
      && candidate.operationId === operationId,
    );
    if (!message) return undefined;
    this.prepareRetry(peerKey, message.id, operationId);
    return message.id;
  }

  adoptRecoveredOperation(
    peerKey: string,
    messageId: string,
    operationId: string,
  ): AgentTranscriptSourceMessage[] {
    return this.update(peerKey, (current) => current.map((message) =>
      message.id === messageId && message.role === 'me' && message.kind === 'message'
        ? {
            ...message,
            operationId,
            optimistic: false,
            queued: false,
          }
        : message,
    ));
  }

  removeQueuedUserMessage(peerKey: string, messageId: string): AgentTranscriptSourceMessage[] {
    return this.update(peerKey, (current) => current.filter((message) =>
      message.id !== messageId || message.role !== 'me' || message.queued !== true,
    ));
  }

  markUserOperationAccepted(peerKey: string, operationId: string): AgentTranscriptSourceMessage[] {
    return this.update(peerKey, (current) => current.map((message) =>
      message.role === 'me'
      && message.kind === 'message'
      && message.operationId === operationId
      && (message.optimistic || message.queued)
        ? { ...message, optimistic: false, queued: false }
        : message,
    ));
  }

  reconcileUserMessage(
    peerKey: string,
    text: string,
    operationId?: string,
    now = Date.now(),
  ): AgentTranscriptSourceMessage[] {
    return this.update(peerKey, (current) => {
      const optimisticIndex = current.findIndex((message) =>
        message.role === 'me'
        && message.kind === 'message'
        && message.optimistic === true
        && message.text === text,
      );
      if (optimisticIndex >= 0) {
        return current.map((message, messageIndex) => messageIndex === optimisticIndex
          ? {
              ...message,
              optimistic: false,
              queued: false,
              ...(operationId ? { operationId } : {}),
            }
          : message);
      }
      if (current.some((message) =>
        message.role === 'me'
        && message.kind === 'message'
        && message.text === text
        && (!operationId || message.operationId === operationId))) return current;
      return [...current, {
        id: `${operationId ?? 'agent'}:user:${now}`,
        source: 'legacy',
        role: 'me',
        text,
        createdAtMs: now,
        kind: 'message',
        ...(operationId ? { operationId } : {}),
      }];
    });
  }

  adoptOperation(peerKey: string, requestId: string, operationId: string): AgentTranscriptSourceMessage[] {
    if (!requestId || requestId === operationId) return this.thread(peerKey);
    return this.update(peerKey, (current) => {
      const alreadyAuthoritative = current.some((message) =>
        message.kind === 'assistant-turn' && message.operationId === operationId,
      );
      return current.flatMap((message) => {
        if (message.operationId !== requestId) return [message];
        if (message.kind === 'assistant-turn' && message.assistantTurn) {
          if (alreadyAuthoritative) return [];
          const assistantTurn = rekeyAssistantTurn(message.assistantTurn, operationId);
          return [{
            ...message,
            id: `${operationId}:assistant-turn`,
            operationId,
            text: assistantTurnPlainText(assistantTurn),
            assistantTurn,
          }];
        }
        return [{ ...message, operationId }];
      });
    });
  }

  appendAssistantTurnEvent(peerKey: string, event: RuntimeEvent): AgentTranscriptSourceMessage[] {
    const operationId = 'operationId' in event && typeof event.operationId === 'string'
      ? event.operationId
      : undefined;
    if (!operationId) return this.thread(peerKey);
    return this.update(peerKey, (current) => {
      const index = current.findIndex((message) =>
        message.kind === 'assistant-turn' && message.operationId === operationId,
      );
      const existingTurn = index >= 0 ? current[index]?.assistantTurn : undefined;
      const assistantTurn = reduceAssistantTurn(
        existingTurn ?? createAssistantTurn(operationId),
        event,
      );
      const next: AgentTranscriptSourceMessage = {
        id: `${operationId}:assistant-turn`,
        source: 'legacy',
        role: 'peer',
        text: assistantTurnPlainText(assistantTurn),
        createdAtMs: assistantTurn.createdAtMs,
        kind: 'assistant-turn',
        operationId,
        streaming: assistantTurn.status === 'running',
        assistantTurn,
      };
      if (index < 0) return [...current, next];
      return current.map((message, messageIndex) =>
        messageIndex === index
          ? { ...message, ...next, createdAtMs: message.createdAtMs }
          : message,
      );
    });
  }

  applyTurnState(
    peerKey: string,
    event: Extract<RuntimeEvent, { type: 'turn.state' }>,
  ): AgentTranscriptSourceMessage[] {
    return this.update(peerKey, (current) => current.map((message) => {
      if (message.operationId !== event.operationId) return message;
      if (message.kind === 'assistant-turn') {
        return {
          ...message,
          status: event.state,
          streaming: event.state === 'streaming',
          queued: event.state === 'queued',
        };
      }
      if (message.role === 'me' && message.kind === 'message') {
        return {
          ...message,
          optimistic: event.state === 'accepted' || event.state === 'queued',
          queued: event.state === 'queued',
        };
      }
      return message;
    }));
  }

  appendComputerHandoff(
    peerKey: string,
    event: Extract<RuntimeEvent, { type: 'computer.snapshot' | 'computer.result' }>,
  ): AgentTranscriptSourceMessage[] {
    const createdAtMs = Number.isFinite(Date.parse(event.timestamp)) ? Date.parse(event.timestamp) : Date.now();
    const isSnapshot = event.type === 'computer.snapshot';
    const origin = isSnapshot ? event.origin : event.result.origin;
    const detail = isSnapshot
      ? `Captured this computer for the Agent (${origin}).`
      : `Completed ${event.result.actionsExecuted} computer action${event.result.actionsExecuted === 1 ? '' : 's'} (${origin}).`;
    const next: AgentTranscriptSourceMessage = {
      id: `computer:${event.requestId}:${event.type}`,
      source: 'legacy',
      role: 'peer',
      text: '',
      createdAtMs,
      kind: 'computer-handoff',
      actionTitle: isSnapshot ? 'Computer snapshot' : 'Computer action',
      actionDetail: detail,
      status: 'completed',
    };
    return this.update(peerKey, (current) => {
      const index = current.findIndex((message) => message.id === next.id);
      if (index < 0) return [...current, next];
      return current.map((message, messageIndex) => messageIndex === index ? next : message);
    });
  }

  appendApprovalRequested(
    peerKey: string,
    event: Extract<RuntimeEvent, { type: 'approval.requested' }>,
  ): AgentTranscriptSourceMessage[] {
    if (!event.operationId) return this.thread(peerKey);
    const approval = {
      approvalId: event.approvalId,
      miniAppId: event.miniAppId,
      capability: event.capability,
      reason: event.reason,
      kind: event.kind,
      subject: event.subject,
      detail: event.detail,
      proposedRule: event.proposedRule,
      location: event.location,
    };
    const id = `${event.operationId}:approval:${event.approvalId}`;
    const next: AgentTranscriptSourceMessage = {
      id,
      source: 'legacy',
      role: 'peer',
      text: event.reason,
      createdAtMs: Number.isFinite(Date.parse(event.timestamp)) ? Date.parse(event.timestamp) : Date.now(),
      kind: 'approval',
      operationId: event.operationId,
      actionTitle: event.subject || event.capability || 'Permission required',
      actionDetail: event.detail || event.reason,
      status: 'pending',
      approval,
    };
    return this.update(peerKey, (current) => {
      const index = current.findIndex((message) => message.id === id);
      if (index < 0) return [...current, next];
      return current.map((message, messageIndex) => messageIndex === index
        ? { ...message, ...next, createdAtMs: message.createdAtMs }
        : message);
    });
  }

  resolveApproval(
    peerKey: string,
    event: Extract<RuntimeEvent, { type: 'approval.resolved' }>,
  ): AgentTranscriptSourceMessage[] {
    return this.update(peerKey, (current) => current.map((message) => {
      if (message.kind !== 'approval' || message.approval?.approvalId !== event.approvalId) return message;
      return {
        ...message,
        status: 'completed',
        ...(event.operationId ? { operationId: event.operationId } : {}),
        approval: { ...message.approval, decision: event.decision },
      };
    }));
  }

  finishOperation(
    peerKey: string,
    operationId: string,
    terminalStatus: AgentTranscriptTerminalStatus = 'completed',
  ): AgentTranscriptSourceMessage[] {
    return this.update(peerKey, (current) => current
      .filter((message) => !(message.kind === 'thinking' && message.operationId === operationId))
      .map((message) => {
        if (
          message.kind === 'action'
          && message.operationId === operationId
          && message.actionStatus === 'running'
        ) {
          return { ...message, actionStatus: terminalStatus };
        }
        if (message.kind === 'message' && message.operationId === operationId && message.streaming) {
          return { ...message, streaming: false, optimistic: false };
        }
        return message;
      }));
  }
}
