export type MessageReactionKind = "like" | "dislike" | "heart" | "laugh" | "insightful";
export type FeedbackRating = "up" | "down";

export interface MessageReactionRecord {
  readonly conversationId: string;
  readonly messageId: string;
  readonly reaction: MessageReactionKind;
  readonly active: boolean;
  readonly updatedAtMs: number;
}

export interface FeedbackRecord {
  readonly id: string;
  readonly conversationId: string | null;
  readonly messageId: string;
  readonly rating: FeedbackRating;
  readonly note: string | null;
  readonly createdAtMs: number;
}

type InteractionState = {
  reactions: MessageReactionRecord[];
  feedback: FeedbackRecord[];
};

const STORAGE_KEY = "fabushi.interactions.v1";
const MAX_REACTIONS = 2_000;
const MAX_FEEDBACK = 1_000;

function storage(): Storage | null {
  if (typeof window === "undefined") return null;
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function normalizeState(value: unknown): InteractionState {
  if (!value || typeof value !== "object") return { reactions: [], feedback: [] };
  const candidate = value as Partial<InteractionState>;
  const reactions = Array.isArray(candidate.reactions)
    ? candidate.reactions.filter((item): item is MessageReactionRecord =>
        Boolean(
          item
          && typeof item.conversationId === "string"
          && typeof item.messageId === "string"
          && ["like", "dislike", "heart", "laugh", "insightful"].includes(item.reaction)
          && typeof item.active === "boolean"
          && Number.isFinite(item.updatedAtMs),
        ),
      ).slice(-MAX_REACTIONS)
    : [];
  const feedback = Array.isArray(candidate.feedback)
    ? candidate.feedback.filter((item): item is FeedbackRecord =>
        Boolean(
          item
          && typeof item.id === "string"
          && typeof item.messageId === "string"
          && (item.conversationId === null || typeof item.conversationId === "string")
          && (item.rating === "up" || item.rating === "down")
          && (item.note === null || typeof item.note === "string")
          && Number.isFinite(item.createdAtMs),
        ),
      ).slice(-MAX_FEEDBACK)
    : [];
  return { reactions, feedback };
}

function loadState(): InteractionState {
  const target = storage();
  if (!target) return { reactions: [], feedback: [] };
  try {
    return normalizeState(JSON.parse(target.getItem(STORAGE_KEY) ?? "null"));
  } catch {
    return { reactions: [], feedback: [] };
  }
}

function saveState(state: InteractionState): void {
  const target = storage();
  if (!target) return;
  try {
    target.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // Interaction state must never block the primary agent workflow if the
    // browser denies or exhausts local persistence.
  }
}

function reactionKey(record: Pick<MessageReactionRecord, "conversationId" | "messageId" | "reaction">): string {
  return `${record.conversationId}\u001f${record.messageId}\u001f${record.reaction}`;
}

export class FabushiInteractionStore {
  private state = loadState();
  private readonly listeners = new Set<() => void>();

  listReactions(conversationId: string, messageId: string): MessageReactionRecord[] {
    return this.state.reactions.filter(
      (item) => item.conversationId === conversationId && item.messageId === messageId,
    );
  }

  setReaction(input: {
    conversationId: string;
    messageId: string;
    reaction: MessageReactionKind;
    active?: boolean;
  }): MessageReactionRecord {
    const conversationId = input.conversationId.trim();
    const messageId = input.messageId.trim();
    if (!conversationId || !messageId) throw new Error("Conversation and message IDs are required");
    const record: MessageReactionRecord = {
      conversationId,
      messageId,
      reaction: input.reaction,
      active: input.active ?? true,
      updatedAtMs: Date.now(),
    };
    const key = reactionKey(record);
    this.state = {
      ...this.state,
      reactions: [
        ...this.state.reactions.filter((item) => reactionKey(item) !== key),
        record,
      ].slice(-MAX_REACTIONS),
    };
    this.commit();
    return record;
  }

  feedbackFor(messageId: string): FeedbackRecord | null {
    for (let index = this.state.feedback.length - 1; index >= 0; index -= 1) {
      const record = this.state.feedback[index];
      if (record?.messageId === messageId) return record;
    }
    return null;
  }

  recordFeedback(input: {
    conversationId?: string | null;
    messageId: string;
    rating: FeedbackRating;
    note?: string | null;
  }): FeedbackRecord {
    const messageId = input.messageId.trim();
    if (!messageId) throw new Error("Message ID is required");
    const note = input.note?.replace(/\s+/gu, " ").trim().slice(0, 1_000) || null;
    const record: FeedbackRecord = {
      id: typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
        ? crypto.randomUUID()
        : `feedback-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`,
      conversationId: input.conversationId?.trim() || null,
      messageId,
      rating: input.rating,
      note,
      createdAtMs: Date.now(),
    };
    this.state = {
      ...this.state,
      feedback: [...this.state.feedback, record].slice(-MAX_FEEDBACK),
    };
    this.commit();
    return record;
  }

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private commit(): void {
    saveState(this.state);
    for (const listener of this.listeners) listener();
  }
}
