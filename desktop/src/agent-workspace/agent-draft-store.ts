import type { AttachmentContext } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { AGENT_ATTACHMENT_LIMIT } from './agent-attachments';
import type { AgentPromptReference, AgentReplyContext } from './prompt-context';
import { normalizeAgentRichText } from './agent-rich-text';

export const AGENT_WORKSPACE_DRAFT_STORAGE_KEY = 'fabushi.agent-workspace.drafts.v1';
const storageKey = AGENT_WORKSPACE_DRAFT_STORAGE_KEY;

export interface PersistedAgentDraft {
  text: string;
  /** Serialized TipTap-compatible Agent-owned document. Mahayana executes text. */
  richText?: string;
  attachments: AttachmentContext[];
  references?: AgentPromptReference[];
  replyTo?: AgentReplyContext;
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

function validPromptReference(value: unknown): value is AgentPromptReference {
  if (!value || typeof value !== 'object') return false;
  const reference = value as Partial<AgentPromptReference>;
  return ['agent', 'workflow', 'mcp', 'file', 'link'].includes(String(reference.kind))
    && typeof reference.id === 'string'
    && reference.id.trim().length > 0
    && typeof reference.label === 'string'
    && reference.label.trim().length > 0;
}

function validReplyContext(value: unknown): value is AgentReplyContext {
  if (!value || typeof value !== 'object') return false;
  const reply = value as Partial<AgentReplyContext>;
  return typeof reply.id === 'string'
    && (reply.role === 'me' || reply.role === 'peer')
    && typeof reply.text === 'string';
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
      const draft = value as { text?: unknown; richText?: unknown; attachments?: unknown; references?: unknown; replyTo?: unknown };
      const text = typeof draft.text === 'string' ? draft.text : '';
      const attachments = Array.isArray(draft.attachments)
        ? draft.attachments.filter(validAttachment).slice(0, AGENT_ATTACHMENT_LIMIT)
        : [];
      const references = Array.isArray(draft.references)
        ? draft.references.filter(validPromptReference).slice(0, 32)
        : [];
      const replyTo = validReplyContext(draft.replyTo) ? draft.replyTo : undefined;
      const richText = normalizeAgentRichText(
        typeof draft.richText === 'string' ? draft.richText : undefined,
        text,
        references,
      );
      if (text || attachments.length || references.length || replyTo) {
        result[peerKey] = {
          text,
          ...(richText ? { richText } : {}),
          attachments,
          ...(references.length ? { references } : {}),
          ...(replyTo ? { replyTo } : {}),
        };
      }
    }
    return result;
  } catch {
    return {};
  }
}

export function persistAgentWorkspaceDrafts(
  textByPeer: Readonly<Record<string, string>>,
  attachmentsByPeer: Readonly<Record<string, readonly AttachmentContext[]>>,
  replyByPeer: Readonly<Record<string, AgentReplyContext>> = {},
  referencesByPeer: Readonly<Record<string, readonly AgentPromptReference[]>> = {},
  richTextByPeer: Readonly<Record<string, string>> = {},
): void {
  if (typeof window === 'undefined') return;
  const peerKeys = new Set([
    ...Object.keys(textByPeer),
    ...Object.keys(attachmentsByPeer),
    ...Object.keys(replyByPeer),
    ...Object.keys(referencesByPeer),
    ...Object.keys(richTextByPeer),
  ]);
  const snapshot: PersistedAgentDrafts = {};
  for (const peerKey of peerKeys) {
    const text = textByPeer[peerKey] ?? '';
    const attachments = [...(attachmentsByPeer[peerKey] ?? [])].slice(0, AGENT_ATTACHMENT_LIMIT);
    const replyTo = replyByPeer[peerKey];
    const references = [...(referencesByPeer[peerKey] ?? [])].slice(0, 32);
    const richText = normalizeAgentRichText(richTextByPeer[peerKey], text, references);
    if (text || attachments.length || references.length || replyTo) {
      snapshot[peerKey] = {
        text,
        ...(richText ? { richText } : {}),
        attachments,
        ...(references.length ? { references } : {}),
        ...(replyTo ? { replyTo } : {}),
      };
    }
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
