export interface AgentReplyContext {
  readonly id: string;
  readonly role: 'me' | 'peer';
  readonly text: string;
}

export function composeAgentPromptText(text: string, replyTo?: AgentReplyContext): string {
  if (!replyTo) return text;
  const quoted = replyTo.text.trim().slice(0, 8_000);
  if (!quoted) return text;
  const speaker = replyTo.role === 'me' ? 'user' : 'assistant';
  return [
    `Reply context (previous ${speaker} message, for conversational reference):`,
    '---',
    quoted,
    '---',
    '',
    text,
  ].join('\n');
}
