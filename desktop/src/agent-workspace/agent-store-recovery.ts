import type { AttachmentContext } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  FABU_AGENT_ATTACHMENT_INDEX_PATH,
  FABU_AGENT_RUNTIME_CHECKPOINT_PATH,
  FabuAgentStore,
  fabuAgentConversationTranscriptPath,
} from '../fabu-runtime/agent-store';
import type { TranscriptEntry } from './transcript-model';

export type AgentStoreCheckpointStatus = 'running' | 'completed' | 'failed' | 'interrupted';

export interface AgentStoreRuntimeCheckpoint {
  readonly schemaVersion?: number;
  readonly agentId?: string;
  readonly conversationId?: string;
  readonly operationId?: string;
  readonly status?: AgentStoreCheckpointStatus;
  readonly updatedAtMs?: number;
  readonly message?: string;
}

export interface AgentStoreWorkspaceRecovery {
  readonly entries: readonly TranscriptEntry[];
  readonly attachments: readonly AttachmentContext[];
  readonly checkpoint?: AgentStoreRuntimeCheckpoint;
}

function validAttachment(value: unknown): value is AttachmentContext {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const attachment = value as Partial<AttachmentContext>;
  return typeof attachment.id === 'string'
    && attachment.id.trim().length > 0
    && typeof attachment.name === 'string'
    && attachment.name.trim().length > 0
    && (attachment.path == null || typeof attachment.path === 'string')
    && (attachment.mimeType == null || typeof attachment.mimeType === 'string')
    && (attachment.sizeBytes == null || (typeof attachment.sizeBytes === 'number' && Number.isFinite(attachment.sizeBytes)));
}

function validCheckpoint(
  value: unknown,
  agentId: string,
  conversationId: string,
): AgentStoreRuntimeCheckpoint | undefined {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return undefined;
  const checkpoint = value as AgentStoreRuntimeCheckpoint;
  if (checkpoint.agentId && checkpoint.agentId !== agentId) return undefined;
  if (checkpoint.conversationId && checkpoint.conversationId !== conversationId) return undefined;
  if (checkpoint.status && !['running', 'completed', 'failed', 'interrupted'].includes(checkpoint.status)) return undefined;
  return checkpoint;
}

function runningCheckpointNotice(
  checkpoint: AgentStoreRuntimeCheckpoint | undefined,
): TranscriptEntry | undefined {
  if (checkpoint?.status !== 'running' || !checkpoint.operationId) return undefined;
  return {
    id: `agent-store-recovery:${checkpoint.operationId}`,
    kind: 'notice',
    role: 'peer',
    text: '',
    createdAtMs: typeof checkpoint.updatedAtMs === 'number' && Number.isFinite(checkpoint.updatedAtMs) ? checkpoint.updatedAtMs : Date.now(),
    operationId: checkpoint.operationId,
    title: 'Previous device run',
    detail: checkpoint.message?.trim()
      || 'The last saved checkpoint was still running on another device. This device did not adopt that operation; retry or continue the turn to recover safely.',
    status: 'interrupted',
  };
}

/**
 * Restore the Agent-owned workspace projection from one immutable root.
 *
 * The runtime checkpoint is deliberately not adopted into the local operation
 * registry. A checkpoint can originate on another device, so reusing its
 * operation id as locally active would create a false global operation pointer.
 */
export async function restoreAgentStoreWorkspace(
  store: FabuAgentStore,
  conversationId: string,
): Promise<AgentStoreWorkspaceRecovery> {
  await store.restoreRoot();
  await store.materializeRoot();
  const transcriptPath = fabuAgentConversationTranscriptPath(conversationId);

  const transcript = store.hasRootPath(transcriptPath)
    ? await store.readJson<{
        schemaVersion?: number;
        agentId?: string;
        conversationId?: string;
        entries?: TranscriptEntry[];
      }>(transcriptPath).catch(() => null)
    : null;

  const attachmentIndex = store.hasRootPath(FABU_AGENT_ATTACHMENT_INDEX_PATH)
    ? await store.readJson<{
        schemaVersion?: number;
        agentId?: string;
        attachments?: unknown[];
      }>(FABU_AGENT_ATTACHMENT_INDEX_PATH).catch(() => null)
    : null;

  const rawCheckpoint = store.hasRootPath(FABU_AGENT_RUNTIME_CHECKPOINT_PATH)
    ? await store.readJson<AgentStoreRuntimeCheckpoint>(FABU_AGENT_RUNTIME_CHECKPOINT_PATH).catch(() => null)
    : null;

  const transcriptMatches = transcript
    && (!transcript.agentId || transcript.agentId === store.agentId)
    && (!transcript.conversationId || transcript.conversationId === conversationId);
  const entries = transcriptMatches && Array.isArray(transcript.entries)
    ? transcript.entries
    : [];

  const attachments = attachmentIndex
    && (!attachmentIndex.agentId || attachmentIndex.agentId === store.agentId)
    && Array.isArray(attachmentIndex.attachments)
    ? attachmentIndex.attachments.filter(validAttachment)
    : [];

  const checkpoint = validCheckpoint(rawCheckpoint, store.agentId, conversationId);
  const recoveryNotice = runningCheckpointNotice(checkpoint);

  return {
    entries: recoveryNotice
      ? [...entries.filter((entry) => entry.id !== recoveryNotice.id), recoveryNotice]
      : entries,
    attachments,
    ...(checkpoint ? { checkpoint } : {}),
  };
}
