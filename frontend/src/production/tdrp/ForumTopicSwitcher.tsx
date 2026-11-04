import React from "react";
import type { ForumTopicItem } from "../topic-workspace-projection";

export interface ForumTopicSwitcherProps {
  readonly visibleTopics: readonly ForumTopicItem[];
  readonly activeTopicId: string | null;
  readonly onSelectTopic: (topicId: string | null) => void;
}

export function ForumTopicSwitcher({
  visibleTopics,
  activeTopicId,
  onSelectTopic,
}: ForumTopicSwitcherProps): React.JSX.Element | null {
  if (visibleTopics.length === 0) return null;
  return (
    <nav className="sand-forum-topic-switcher" data-testid="forum-topic-switcher" aria-label="Forum topics">
      <button
        type="button"
        className={activeTopicId == null ? "sand-topic-chip active" : "sand-topic-chip"}
        aria-selected={activeTopicId == null}
        onClick={() => onSelectTopic(null)}
      >
        General
      </button>
      {visibleTopics.map((topic) => (
        <button
          key={topic.id}
          type="button"
          className={activeTopicId === topic.id ? "sand-topic-chip active" : "sand-topic-chip"}
          aria-selected={activeTopicId === topic.id}
          onClick={() => onSelectTopic(topic.id)}
        >
          {topic.title}
          {topic.closed ? " (Closed)" : null}
        </button>
      ))}
    </nav>
  );
}
