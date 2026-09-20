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
  | 'pending'
  | 'running'
  | 'completed'
  | 'failed'
  | 'interrupted';

export interface TranscriptEntry {
  readonly id: string;
  readonly kind: TranscriptEntryKind;
  readonly role: 'me' | 'peer';
  readonly text: string;
  readonly createdAtMs: number;
  readonly operationId?: string;
  readonly streaming?: boolean;
  readonly optimistic?: boolean;
  readonly queued?: boolean;
  readonly title?: string;
  readonly detail?: string;
  readonly status?: TranscriptEntryStatus;
  readonly assistantTurn?: AssistantTurn;
  readonly miniAppId?: string;
}

export interface TranscriptSourceMessage {
  readonly id: string;
  readonly role: 'me' | 'peer';
  readonly text: string;
  readonly createdAtMs: number;
  readonly kind?: 'message' | 'assistant-turn' | 'action' | 'thinking';
  readonly operationId?: string;
  readonly streaming?: boolean;
  readonly optimistic?: boolean;
  readonly queued?: boolean;
  readonly actionTitle?: string;
  readonly actionDetail?: string;
  readonly actionStatus?: 'running' | 'completed' | 'failed' | 'interrupted';
  readonly assistantTurn?: AssistantTurn;
  readonly miniAppId?: string;
}

function sourceKind(message: TranscriptSourceMessage): TranscriptEntryKind {
  switch (message.kind) {
    case 'assistant-turn':
      return 'assistant-turn';
    case 'thinking':
      return 'thinking';
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
    createdAtMs: message.createdAtMs,
    ...(message.operationId ? { operationId: message.operationId } : {}),
    ...(message.streaming ? { streaming: true } : {}),
    ...(message.optimistic ? { optimistic: true } : {}),
    ...(message.queued ? { queued: true } : {}),
    ...(message.actionTitle ? { title: message.actionTitle } : {}),
    ...(message.actionDetail ? { detail: message.actionDetail } : {}),
    ...(message.actionStatus ? { status: message.actionStatus } : {}),
    ...(message.assistantTurn ? { assistantTurn: message.assistantTurn } : {}),
    ...(message.miniAppId ? { miniAppId: message.miniAppId } : {}),
  }));
}
