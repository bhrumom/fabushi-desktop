import type { AttachmentContext, TurnLifecycleState } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { AssistantTurn } from '../mahayana-assistant-turn';

export type TranscriptEntryKind =
  | 'message'
  | 'assistant-turn'
  | 'thinking'
  | 'tool-call'
  | 'approval'
  | 'permission'
  | 'computer-handoff'
  | 'attachment'
  | 'timeline-event'
  | 'notice';

export type TranscriptEntryStatus =
  | TurnLifecycleState
  | 'pending'
  | 'running'
  | 'interrupted';

export type TranscriptApprovalDecision = 'allow-once' | 'allow-session' | 'deny';

export interface TranscriptApproval {
  readonly approvalId: string;
  readonly miniAppId?: string;
  readonly capability: string;
  readonly reason: string;
  readonly kind?: string;
  readonly subject?: string;
  readonly detail?: string;
  readonly proposedRule?: string;
  readonly location?: string;
  readonly decision?: TranscriptApprovalDecision;
}

export interface TranscriptEntry {
  readonly id: string;
  readonly kind: TranscriptEntryKind;
  readonly role: 'me' | 'peer';
  readonly text: string;
  readonly richText?: string;
  readonly createdAtMs: number;
  readonly operationId?: string;
  readonly streaming?: boolean;
  readonly optimistic?: boolean;
  readonly queued?: boolean;
  readonly title?: string;
  readonly detail?: string;
  readonly status?: TranscriptEntryStatus;
  readonly assistantTurn?: AssistantTurn;
  readonly attachments?: readonly AttachmentContext[];
  readonly approval?: TranscriptApproval;
  readonly miniAppId?: string;
}

export interface TranscriptSourceMessage {
  readonly id: string;
  readonly role: 'me' | 'peer';
  readonly text: string;
  readonly richText?: string;
  readonly createdAtMs: number;
  readonly kind?: TranscriptEntryKind | 'action';
  readonly operationId?: string;
  readonly streaming?: boolean;
  readonly optimistic?: boolean;
  readonly queued?: boolean;
  readonly actionTitle?: string;
  readonly actionDetail?: string;
  readonly actionStatus?: 'running' | 'completed' | 'failed' | 'interrupted';
  readonly status?: TranscriptEntryStatus;
  readonly assistantTurn?: AssistantTurn;
  readonly attachments?: readonly AttachmentContext[];
  readonly approval?: TranscriptApproval;
  readonly miniAppId?: string;
}

function sourceKind(message: TranscriptSourceMessage): TranscriptEntryKind {
  switch (message.kind) {
    case 'assistant-turn':
    case 'thinking':
    case 'approval':
    case 'permission':
    case 'computer-handoff':
    case 'attachment':
    case 'timeline-event':
    case 'notice':
      return message.kind;
    case 'action':
      return 'tool-call';
    default:
      return 'message';
  }
}

/**
 * Canonical renderer projection for an Agent timeline.
 *
 * Older message/action/thinking records are accepted only at this adapter
 * boundary. Agent workspace components consume TranscriptEntry so one runtime
 * turn cannot accidentally render through multiple overlapping message models.
 */
export function projectTranscriptEntries(
  messages: readonly TranscriptSourceMessage[],
): TranscriptEntry[] {
  return messages.map((message) => ({
    id: message.id,
    kind: sourceKind(message),
    role: message.role,
    text: message.text,
    ...(message.richText ? { richText: message.richText } : {}),
    createdAtMs: message.createdAtMs,
    ...(message.operationId ? { operationId: message.operationId } : {}),
    ...(message.streaming ? { streaming: true } : {}),
    ...(message.optimistic ? { optimistic: true } : {}),
    ...(message.queued ? { queued: true } : {}),
    ...(message.actionTitle ? { title: message.actionTitle } : {}),
    ...(message.actionDetail ? { detail: message.actionDetail } : {}),
    ...((message.status ?? message.actionStatus) ? { status: message.status ?? message.actionStatus } : {}),
    ...(message.assistantTurn ? { assistantTurn: message.assistantTurn } : {}),
    ...(message.attachments?.length ? { attachments: message.attachments } : {}),
    ...(message.approval ? { approval: message.approval } : {}),
    ...(message.miniAppId ? { miniAppId: message.miniAppId } : {}),
  }));
}
