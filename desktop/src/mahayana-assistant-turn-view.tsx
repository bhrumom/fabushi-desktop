import type { ReactNode } from 'react';
import type { AssistantTurn, AssistantTurnPart } from './mahayana-assistant-turn';

function activityStatusLabel(status: Extract<AssistantTurnPart, { kind: 'tool' | 'activity' }>['status']): string {
  if (status === 'running') return '进行中';
  if (status === 'failed') return '失败';
  return '完成';
}

function renderPart(part: AssistantTurnPart): ReactNode {
  if (part.kind === 'reasoning') {
    return (
      <div
        className="mahayana-assistant-turn__reasoning"
        data-part-kind="reasoning"
        data-streaming={part.status === 'streaming' || undefined}
        key={part.id}
      >
        {part.text}
      </div>
    );
  }

  if (part.kind === 'text') {
    return (
      <p
        className="mahayana-assistant-turn__text"
        data-part-kind="text"
        data-streaming={part.status === 'streaming' || undefined}
        key={part.id}
      >
        {part.text}
      </p>
    );
  }

  return (
    <div
      className="mahayana-assistant-turn__activity"
      data-part-kind={part.kind}
      data-status={part.status}
      key={part.id}
    >
      <i aria-hidden="true" />
      <div>
        <strong>{part.title}</strong>
        {part.detail ? <span>{part.detail}</span> : null}
      </div>
      <small>{activityStatusLabel(part.status)}</small>
    </div>
  );
}

export type MahayanaAssistantTurnViewProps = {
  turn: AssistantTurn;
  avatar?: ReactNode;
  label?: string;
};

/**
 * Presentation-only renderer for one assistant turn.
 *
 * The Rust gateway/session store is authoritative. This component never starts
 * tools, changes approvals, rewrites sequence numbers, or persists agent state;
 * it only renders the already ordered projection supplied by the transcript.
 */
export function MahayanaAssistantTurnView({
  turn,
  avatar,
  label = 'Mahayana',
}: MahayanaAssistantTurnViewProps) {
  return (
    <article
      className="mahayana-assistant-turn"
      data-testid="mahayana-assistant-turn"
      data-operation-id={turn.operationId}
      data-status={turn.status}
      aria-label={`${label} 回复`}
    >
      <div className="mahayana-assistant-turn__avatar" aria-hidden={!avatar || undefined}>
        {avatar}
      </div>
      <div className="mahayana-assistant-turn__body">
        {turn.parts.map(renderPart)}
        {turn.status === 'failed' || turn.status === 'interrupted' ? (
          <span className="mahayana-assistant-turn__meta" data-turn-status={turn.status}>
            {turn.status === 'failed' ? '失败' : '已暂停'}
          </span>
        ) : null}
      </div>
    </article>
  );
}

export default MahayanaAssistantTurnView;
