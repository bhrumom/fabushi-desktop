export type AgentPromptReferenceKind = 'agent' | 'workflow' | 'mcp' | 'pull-request' | 'file' | 'link';

export interface AgentPromptReference {
  readonly kind: AgentPromptReferenceKind;
  readonly id: string;
  readonly label: string;
}

export function agentPromptReferenceMarker(reference: AgentPromptReference): string {
  if (reference.kind === 'agent') return `@${reference.label}`;
  if (reference.kind === 'workflow') return `/${reference.label}`;
  if (reference.kind === 'mcp') return `@${reference.label}`;
  if (reference.kind === 'pull-request') return `#${reference.label.replace(/^#/, '')}`;
  return reference.label;
}

export interface AgentReplyContext {
  readonly id: string;
  readonly role: 'me' | 'peer';
  readonly text: string;
}

export function composeAgentPromptText(
  text: string,
  replyTo?: AgentReplyContext,
  references: readonly AgentPromptReference[] = [],
): string {
  const blocks: string[] = [];
  const stableReferences = references
    .filter((reference) => reference.id.trim() && reference.label.trim())
    .filter((reference, index, all) => all.findIndex((candidate) => candidate.kind === reference.kind && candidate.id === reference.id) === index);
  if (stableReferences.length) {
    blocks.push(
      'Referenced context (stable ids from the composer):',
      ...stableReferences.map((reference) => `- ${agentPromptReferenceMarker(reference)} [${reference.kind}:${reference.id}]`),
      '',
    );
  }
  if (replyTo) {
    const quoted = replyTo.text.trim().slice(0, 8_000);
    if (quoted) {
      const speaker = replyTo.role === 'me' ? 'user' : 'assistant';
      blocks.push(
        `Reply context (previous ${speaker} message, for conversational reference):`,
        '---',
        quoted,
        '---',
        '',
      );
    }
  }
  blocks.push(text);
  return blocks.join('\n');
}
