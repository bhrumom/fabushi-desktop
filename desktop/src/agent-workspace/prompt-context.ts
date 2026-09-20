export interface AgentPromptReference {
  readonly kind: 'agent';
  readonly id: string;
  readonly label: string;
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
    .filter((reference) => reference.kind === 'agent' && reference.id.trim() && reference.label.trim())
    .filter((reference, index, all) => all.findIndex((candidate) => candidate.id === reference.id) === index);
  if (stableReferences.length) {
    blocks.push(
      'Referenced Agents (stable ids from the composer):',
      ...stableReferences.map((reference) => `- @${reference.label} [agent:${reference.id}]`),
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
