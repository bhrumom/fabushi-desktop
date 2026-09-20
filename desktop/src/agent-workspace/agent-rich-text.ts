import {
  agentPromptReferenceMarker,
  type AgentPromptReference,
  type AgentPromptReferenceKind,
} from './prompt-context';

type RichTextNode = {
  type: string;
  text?: string;
  attrs?: Record<string, unknown>;
  content?: RichTextNode[];
};

function escapeRegExp(value: string): string {
  return value.replace(/[\\^$.*+?()[\]{}|]/g, '\\$&');
}

function referenceNode(reference: AgentPromptReference): RichTextNode | null {
  if (reference.kind === 'agent' || reference.kind === 'mcp') {
    return { type: 'mention', attrs: { id: reference.id, label: reference.label, kind: reference.kind } };
  }
  if (reference.kind === 'workflow') {
    return { type: 'workflowReference', attrs: { id: reference.id, label: reference.label } };
  }
  return null;
}

function paragraphNodes(line: string, references: readonly AgentPromptReference[]): RichTextNode[] {
  const visible = references
    .map((reference) => ({ reference, marker: agentPromptReferenceMarker(reference) }))
    .filter(({ marker }) => marker.length > 0 && line.includes(marker))
    .sort((left, right) => right.marker.length - left.marker.length);
  if (!visible.length) return line ? [{ type: 'text', text: line }] : [];

  const pattern = new RegExp(visible.map(({ marker }) => escapeRegExp(marker)).join('|'), 'g');
  const byMarker = new Map(visible.map((item) => [item.marker, item.reference] as const));
  const nodes: RichTextNode[] = [];
  let cursor = 0;
  for (const match of line.matchAll(pattern)) {
    const index = match.index ?? 0;
    if (index > cursor) nodes.push({ type: 'text', text: line.slice(cursor, index) });
    const marker = match[0] ?? '';
    const reference = byMarker.get(marker);
    nodes.push((reference ? referenceNode(reference) : null) ?? { type: 'text', text: marker });
    cursor = index + marker.length;
  }
  if (cursor < line.length) nodes.push({ type: 'text', text: line.slice(cursor) });
  return nodes;
}

/**
 * TipTap-compatible Agent draft document.
 * Mahayana executes the normalized plain prompt; this preserves semantic
 * references so the Fabu editor can mount without another persistence migration.
 */
export function serializeAgentRichText(
  text: string,
  references: readonly AgentPromptReference[] = [],
): string | undefined {
  if (!text) return undefined;
  return JSON.stringify({
    type: 'doc',
    content: text.split('\n').map((line) => {
      const content = paragraphNodes(line, references);
      return { type: 'paragraph', ...(content.length ? { content } : {}) };
    }),
  });
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

export function isAgentRichText(value: string | undefined): boolean {
  if (!value) return false;
  try {
    const parsed: unknown = JSON.parse(value);
    return isRecord(parsed) && parsed.type === 'doc' && Array.isArray(parsed.content);
  } catch {
    return false;
  }
}

export function normalizeAgentRichText(
  richText: string | undefined,
  text: string,
  references: readonly AgentPromptReference[] = [],
): string | undefined {
  return isAgentRichText(richText) ? richText : serializeAgentRichText(text, references);
}

function promptReferenceKind(value: unknown): AgentPromptReferenceKind | null {
  return value === 'agent' || value === 'workflow' || value === 'mcp' || value === 'file' || value === 'link'
    ? value
    : null;
}

export function agentPromptReferencesFromRichText(value: string | undefined): AgentPromptReference[] {
  if (!isAgentRichText(value)) return [];
  const parsed = JSON.parse(value!) as RichTextNode;
  const result: AgentPromptReference[] = [];
  const visit = (node: RichTextNode) => {
    if (node.type === 'mention') {
      const id = typeof node.attrs?.id === 'string' ? node.attrs.id : '';
      const label = typeof node.attrs?.label === 'string' ? node.attrs.label : '';
      const kind = promptReferenceKind(node.attrs?.kind) ?? 'agent';
      if (id && label && (kind === 'agent' || kind === 'mcp')) result.push({ kind, id, label });
    } else if (node.type === 'workflowReference') {
      const id = typeof node.attrs?.id === 'string' ? node.attrs.id : '';
      const label = typeof node.attrs?.label === 'string' ? node.attrs.label : '';
      if (id && label) result.push({ kind: 'workflow', id, label });
    }
    for (const child of node.content ?? []) visit(child);
  };
  visit(parsed);
  return result.filter((reference, index, all) =>
    all.findIndex((candidate) => candidate.kind === reference.kind && candidate.id === reference.id) === index,
  );
}
