import type { RuntimeEvent } from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MahayanaGatewayEventEnvelope } from '../../frontend/apps/web/src/lib/mahayana-host/gateway-events';

export type AssistantTurnStatus = 'running' | 'completed' | 'failed' | 'interrupted';
export type AssistantPartStatus = 'streaming' | 'running' | 'completed' | 'failed';

export type AssistantTurnPart =
  | {
      id: string;
      kind: 'reasoning';
      text: string;
      status: 'streaming' | 'completed';
    }
  | {
      id: string;
      kind: 'text';
      text: string;
      status: 'streaming' | 'completed';
    }
  | {
      id: string;
      kind: 'tool';
      title: string;
      detail?: string;
      status: 'running' | 'completed' | 'failed';
    }
  | {
      id: string;
      kind: 'activity';
      title: string;
      detail?: string;
      status: 'running' | 'completed' | 'failed';
    };

export type AssistantTurn = {
  id: string;
  operationId: string;
  createdAtMs: number;
  updatedAtMs: number;
  status: AssistantTurnStatus;
  parts: AssistantTurnPart[];
};

export type AssistantTurnEvent = RuntimeEvent | MahayanaGatewayEventEnvelope;

function partText(value: unknown): string {
  return typeof value === 'string' ? value : '';
}

function partId(turn: AssistantTurn, suffix: string): string {
  return `${turn.operationId}:${suffix}`;
}

function replaceOrAppendPart(turn: AssistantTurn, nextPart: AssistantTurnPart): AssistantTurnPart[] {
  const index = turn.parts.findIndex((part) => part.id === nextPart.id);
  if (index < 0) return [...turn.parts, nextPart];
  return turn.parts.map((part, partIndex) => partIndex === index ? nextPart : part);
}

function appendStreamingText(
  turn: AssistantTurn,
  kind: 'text' | 'reasoning',
  delta: string,
): AssistantTurnPart[] {
  if (!delta) return turn.parts;
  const tail = turn.parts.at(-1);
  if (tail?.kind === kind && tail.status === 'streaming') {
    return [
      ...turn.parts.slice(0, -1),
      { ...tail, text: `${tail.text}${delta}` },
    ];
  }
  return [
    ...turn.parts,
    {
      id: partId(turn, `${kind}:${turn.parts.length}`),
      kind,
      text: delta,
      status: 'streaming',
    },
  ];
}

function sealStreamingParts(parts: AssistantTurnPart[]): AssistantTurnPart[] {
  return parts.map((part) => {
    if (part.kind === 'text' || part.kind === 'reasoning') {
      return part.status === 'streaming' ? { ...part, status: 'completed' as const } : part;
    }
    return part;
  });
}

function visibleText(parts: AssistantTurnPart[]): string {
  return parts
    .filter((part): part is Extract<AssistantTurnPart, { kind: 'text' }> => part.kind === 'text')
    .map((part) => part.text)
    .join('');
}

/**
 * Reconcile a legacy final assistant message without collapsing the ordered
 * transcript. Hermes can seal interim assistant text, run tools, and then
 * continue speaking in the same turn. The old renderer used to replace every
 * text fragment with the final message, which destroyed that ordering.
 */
function reconcileLegacyFinalText(turn: AssistantTurn, finalText: string): AssistantTurnPart[] {
  const parts = sealStreamingParts(turn.parts);
  if (!finalText) return parts;

  const emittedText = visibleText(parts);
  if (!emittedText) {
    return [
      ...parts,
      {
        id: partId(turn, 'final-text'),
        kind: 'text',
        text: finalText,
        status: 'completed',
      },
    ];
  }

  // The final message commonly repeats the complete token stream. Keep the
  // already-ordered parts instead of duplicating the response.
  if (finalText === emittedText || emittedText.endsWith(finalText)) return parts;

  // If the final message is the complete answer and the current transcript is
  // its prefix, append only the not-yet-seen suffix. This preserves any tool
  // parts that occurred between earlier and later assistant text.
  if (finalText.startsWith(emittedText)) {
    const suffix = finalText.slice(emittedText.length);
    return suffix
      ? [
          ...parts,
          {
            id: partId(turn, 'final-text-suffix'),
            kind: 'text',
            text: suffix,
            status: 'completed',
          },
        ]
      : parts;
  }

  // Some legacy providers emit only the post-tool completion here. Preserve
  // the earlier ordered transcript and append that completion as the next part.
  return [
    ...parts,
    {
      id: partId(turn, 'final-text'),
      kind: 'text',
      text: finalText,
      status: 'completed',
    },
  ];
}

function legacyOperationId(event: RuntimeEvent): string | undefined {
  return 'operationId' in event && typeof event.operationId === 'string'
    ? event.operationId
    : undefined;
}

export function createAssistantTurn(operationId: string, now = Date.now()): AssistantTurn {
  return {
    id: `assistant-turn:${operationId}`,
    operationId,
    createdAtMs: now,
    updatedAtMs: now,
    status: 'running',
    parts: [],
  };
}

function reduceLegacyRuntimeEvent(turn: AssistantTurn, event: RuntimeEvent, now: number): AssistantTurn {
  const operationId = legacyOperationId(event);
  if (operationId && operationId !== turn.operationId) return turn;

  switch (event.type) {
    case 'operation.started':
      return {
        ...turn,
        updatedAtMs: now,
        status: 'running',
        parts: replaceOrAppendPart(turn, {
          id: partId(turn, 'thinking'),
          kind: 'reasoning',
          text: event.label && event.label !== 'chat-response' ? event.label : '正在思考',
          status: 'streaming',
        }),
      };
    case 'model.routed':
      // Model routing is operational metadata. Hermes-style chat does not
      // expose provider/router selection as a separate transcript row.
      return { ...turn, updatedAtMs: now };
    case 'agent.step':
      return {
        ...turn,
        updatedAtMs: now,
        parts: replaceOrAppendPart(turn, {
          id: partId(turn, `step:${event.stepId}`),
          kind: 'tool',
          title: event.title,
          detail: event.detail,
          status: event.status,
        }),
      };
    case 'chat.delta':
      return {
        ...turn,
        updatedAtMs: now,
        status: 'running',
        parts: appendStreamingText(
          { ...turn, parts: sealStreamingParts(turn.parts) },
          'text',
          event.delta,
        ),
      };
    case 'chat.message':
      if (event.role !== 'assistant') return turn;
      return {
        ...turn,
        updatedAtMs: now,
        status: 'completed',
        parts: reconcileLegacyFinalText(turn, event.text),
      };
    case 'operation.completed':
      return {
        ...turn,
        updatedAtMs: now,
        status: 'completed',
        parts: sealStreamingParts(turn.parts),
      };
    case 'operation.interrupted':
      return {
        ...turn,
        updatedAtMs: now,
        status: 'interrupted',
        parts: sealStreamingParts(turn.parts),
      };
    case 'operation.failed':
      return {
        ...turn,
        updatedAtMs: now,
        status: 'failed',
        parts: [
          ...sealStreamingParts(turn.parts),
          {
            id: partId(turn, 'failure'),
            kind: 'activity',
            title: '执行失败',
            detail: event.message,
            status: 'failed',
          },
        ],
      };
    default:
      return turn;
  }
}

function reduceGatewayEvent(turn: AssistantTurn, event: MahayanaGatewayEventEnvelope, now: number): AssistantTurn {
  if (event.turnId !== turn.operationId) return turn;
  const payload = event.payload;
  switch (event.type) {
    case 'message.start':
      return { ...turn, updatedAtMs: now, status: 'running' };
    case 'message.delta':
      return {
        ...turn,
        updatedAtMs: now,
        status: 'running',
        parts: appendStreamingText(turn, 'text', partText(payload.text ?? payload.delta)),
      };
    case 'reasoning.delta':
    case 'thinking.delta':
      return {
        ...turn,
        updatedAtMs: now,
        status: 'running',
        parts: appendStreamingText(turn, 'reasoning', partText(payload.text ?? payload.delta)),
      };
    case 'message.interim':
      return {
        ...turn,
        updatedAtMs: now,
        parts: sealStreamingParts(turn.parts),
      };
    case 'tool.generating':
    case 'tool.start': {
      const toolId = partText(payload.tool_id ?? payload.toolId ?? payload.id) || `tool:${event.seq}`;
      const name = partText(payload.name ?? payload.tool_name ?? payload.toolName) || '工具';
      return {
        ...turn,
        updatedAtMs: now,
        parts: replaceOrAppendPart(turn, {
          id: partId(turn, `tool:${toolId}`),
          kind: 'tool',
          title: name,
          detail: partText(payload.description ?? payload.detail) || undefined,
          status: 'running',
        }),
      };
    }
    case 'tool.complete': {
      const toolId = partText(payload.tool_id ?? payload.toolId ?? payload.id) || `tool:${event.seq}`;
      const existing = turn.parts.find((part) => part.id === partId(turn, `tool:${toolId}`));
      return {
        ...turn,
        updatedAtMs: now,
        parts: replaceOrAppendPart(turn, {
          id: partId(turn, `tool:${toolId}`),
          kind: 'tool',
          title: existing && (existing.kind === 'tool' || existing.kind === 'activity')
            ? existing.title
            : partText(payload.name ?? payload.tool_name ?? payload.toolName) || '工具',
          detail: partText(payload.result ?? payload.detail) || (existing && 'detail' in existing ? existing.detail : undefined),
          status: payload.error ? 'failed' : 'completed',
        }),
      };
    }
    case 'approval.request':
      return {
        ...turn,
        updatedAtMs: now,
        parts: replaceOrAppendPart(turn, {
          id: partId(turn, `approval:${partText(payload.request_id ?? payload.requestId ?? payload.id) || event.seq}`),
          kind: 'activity',
          title: partText(payload.title) || '等待授权',
          detail: partText(payload.description ?? payload.detail) || undefined,
          status: 'running',
        }),
      };
    case 'clarify.request':
      return {
        ...turn,
        updatedAtMs: now,
        parts: replaceOrAppendPart(turn, {
          id: partId(turn, `clarify:${partText(payload.request_id ?? payload.requestId ?? payload.id) || event.seq}`),
          kind: 'activity',
          title: '需要补充信息',
          detail: partText(payload.question ?? payload.detail) || undefined,
          status: 'running',
        }),
      };
    case 'subagent.start':
    case 'subagent.progress':
    case 'subagent.complete': {
      const subagentId = partText(payload.subagent_id ?? payload.subagentId ?? payload.id) || `subagent:${event.seq}`;
      return {
        ...turn,
        updatedAtMs: now,
        parts: replaceOrAppendPart(turn, {
          id: partId(turn, `subagent:${subagentId}`),
          kind: 'activity',
          title: partText(payload.title ?? payload.name) || '子智能体',
          detail: partText(payload.detail ?? payload.status) || undefined,
          status: event.type === 'subagent.complete' ? 'completed' : 'running',
        }),
      };
    }
    case 'message.complete': {
      const status = partText(payload.status);
      const completedStatus: AssistantTurnStatus = status === 'error'
        ? 'failed'
        : status === 'interrupted'
          ? 'interrupted'
          : 'completed';
      const finalText = partText(payload.text ?? payload.content);
      const parts = sealStreamingParts(turn.parts);
      const emittedText = visibleText(parts);
      const completedParts = !finalText || finalText === emittedText || emittedText.endsWith(finalText)
        ? parts
        : finalText.startsWith(emittedText)
          ? [
              ...parts,
              {
                id: partId(turn, `complete:${event.seq}`),
                kind: 'text' as const,
                text: finalText.slice(emittedText.length),
                status: 'completed' as const,
              },
            ].filter((part) => part.kind !== 'text' || part.text.length > 0)
          : [
              ...parts,
              {
                id: partId(turn, `complete:${event.seq}`),
                kind: 'text' as const,
                text: finalText,
                status: 'completed' as const,
              },
            ];
      return {
        ...turn,
        updatedAtMs: now,
        status: completedStatus,
        parts: completedParts,
      };
    }
    default:
      return turn;
  }
}

export function reduceAssistantTurn(
  turn: AssistantTurn,
  event: AssistantTurnEvent,
  now = Date.now(),
): AssistantTurn {
  return 'protocolVersion' in event
    ? reduceGatewayEvent(turn, event, now)
    : reduceLegacyRuntimeEvent(turn, event, now);
}

export function assistantTurnPlainText(turn: AssistantTurn): string {
  return turn.parts
    .filter((part): part is Extract<AssistantTurnPart, { kind: 'text' }> => part.kind === 'text')
    .map((part) => part.text)
    .join('\n\n')
    .trim();
}

export function assistantTurnHasVisibleContent(turn: AssistantTurn): boolean {
  return turn.parts.some((part) => {
    if (part.kind === 'text' || part.kind === 'reasoning') return Boolean(part.text.trim());
    return Boolean(part.title.trim() || part.detail?.trim());
  });
}
