import type { AttachmentContext } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { AGENT_ATTACHMENT_LIMIT } from './agent-attachments';

const storageKey = 'fabushi.agent-workspace.drafts.v1';

export interface PersistedAgentDraft {
  text: string;
  attachments: AttachmentContext[];
}

export type PersistedAgentDrafts = Record<string, PersistedAgentDraft>;

function validAttachment(value: unknown): value is AttachmentContext {
  if (!value || typeof value !== 'object') return false;
  const attachment = value as Partial<AttachmentContext>;
  return typeof attachment.id === 'string'
    && typeof attachment.name === 'string'
    && (attachment.path == null || typeof attachment.path === 'string')
    && (attachment.text == null || typeof attachment.text === 'string')
    && (attachment.mimeType == null || typeof attachment.mimeType === 'string')
    && (attachment.sizeBytes == null || typeof attachment.sizeBytes === 'number');
}

export function readAgentWorkspaceDrafts(): PersistedAgentDrafts {
  if (typeof window === 'undefined') return {};
  try {
    const raw = window.localStorage.getItem(storageKey);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== 'object') return {};
    const result: PersistedAgentDrafts = {};
    for (const [peerKey, value] of Object.entries(parsed as Record<string, unknown>)) {
      if (!value || typeof value !== 'object') continue;
      const draft = value as { text?: unknown; attachments?: unknown };
      const text = typeof draft.text === 'string' ? draft.text : '';
      const attachments = Array.isArray(draft.attachments)
        ? draft.attachments.filter(validAttachment).slice(0, AGENT_ATTACHMENT_LIMIT)
        : [];
      if (text || attachments.length) result[peerKey] = { text, attachments };
    }
    return result;
  } catch {
    return {};
  }
}

export function persistAgentWorkspaceDrafts(
  textByPeer: Readonly<Record<string, string>>,
  attachmentsByPeer: Readonly<Record<string, readonly AttachmentContext[]>>,
): void {
  if (typeof window === 'undefined') return;
  const peerKeys = new Set([...Object.keys(textByPeer), ...Object.keys(attachmentsByPeer)]);
  const snapshot: PersistedAgentDrafts = {};
  for (const peerKey of peerKeys) {
    const text = textByPeer[peerKey] ?? '';
    const attachments = [...(attachmentsByPeer[peerKey] ?? [])].slice(0, AGENT_ATTACHMENT_LIMIT);
    if (text || attachments.length) snapshot[peerKey] = { text, attachments };
  }
  try {
    if (Object.keys(snapshot).length) {
      window.localStorage.setItem(storageKey, JSON.stringify(snapshot));
    } else {
      window.localStorage.removeItem(storageKey);
    }
  } catch {
    // Draft recovery is best effort when local storage is unavailable.
  }
}
