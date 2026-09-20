export type WidgetInteractionKind = "response" | "dismissal";

export interface WidgetResponseRecord {
  readonly kind: "response";
  readonly widgetId: string;
  readonly actionId: string | null;
  readonly value: unknown;
  readonly agentId: string | null;
  readonly conversationId: string | null;
  readonly createdAtMs: number;
}

export interface WidgetDismissalRecord {
  readonly kind: "dismissal";
  readonly widgetId: string;
  readonly agentId?: string | null;
  readonly conversationId?: string | null;
  readonly reason: string | null;
  readonly createdAtMs: number;
}

export type WidgetInteractionRecord = WidgetResponseRecord | WidgetDismissalRecord;

const STORAGE_KEY = "fabushi.widget-interactions.v1";
const MAX_RECORDS = 1_000;

function storage(): Storage | null {
  if (typeof window === "undefined") return null;
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function validRecord(value: unknown): value is WidgetInteractionRecord {
  if (!value || typeof value !== "object") return false;
  const item = value as Partial<WidgetInteractionRecord>;
  if (typeof item.widgetId !== "string" || !item.widgetId || !Number.isFinite(item.createdAtMs)) return false;
  return item.kind === "response" || item.kind === "dismissal";
}

function load(): WidgetInteractionRecord[] {
  const target = storage();
  if (!target) return [];
  try {
    const parsed = JSON.parse(target.getItem(STORAGE_KEY) ?? "[]") as unknown;
    return Array.isArray(parsed) ? parsed.filter(validRecord).slice(-MAX_RECORDS) : [];
  } catch {
    return [];
  }
}

function save(records: readonly WidgetInteractionRecord[]): void {
  const target = storage();
  if (!target) return;
  try {
    target.setItem(STORAGE_KEY, JSON.stringify(records.slice(-MAX_RECORDS)));
  } catch {
    // Widget interactions must not block the primary agent workflow when
    // persistence is unavailable or the browser quota is exhausted.
  }
}

export class FabushiWidgetInteractionStore {
  private records = load();
  private readonly listeners = new Set<(record: WidgetInteractionRecord) => void>();

  respond(input: {
    widgetId: string;
    actionId?: string | null;
    value?: unknown;
    agentId?: string | null;
    conversationId?: string | null;
  }): WidgetResponseRecord {
    const widgetId = input.widgetId.trim();
    if (!widgetId) throw new Error("Widget ID is required");
    const record: WidgetResponseRecord = {
      kind: "response",
      widgetId,
      actionId: input.actionId?.trim() || null,
      value: input.value ?? null,
      agentId: input.agentId?.trim() || null,
      conversationId: input.conversationId?.trim() || null,
      createdAtMs: Date.now(),
    };
    this.append(record);
    return record;
  }

  dismiss(input: {
    widgetId: string;
    agentId?: string | null;
    conversationId?: string | null;
    reason?: string | null;
  }): WidgetDismissalRecord {
    const widgetId = input.widgetId.trim();
    if (!widgetId) throw new Error("Widget ID is required");
    const record: WidgetDismissalRecord = {
      kind: "dismissal",
      widgetId,
      agentId: input.agentId?.trim() || null,
      conversationId: input.conversationId?.trim() || null,
      reason: input.reason?.replace(/\s+/gu, " ").trim().slice(0, 500) || null,
      createdAtMs: Date.now(),
    };
    this.append(record);
    return record;
  }

  latest(widgetId: string): WidgetInteractionRecord | null {
    for (let index = this.records.length - 1; index >= 0; index -= 1) {
      const record = this.records[index];
      if (record?.widgetId === widgetId) return record;
    }
    return null;
  }

  subscribe(listener: (record: WidgetInteractionRecord) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private append(record: WidgetInteractionRecord): void {
    this.records = [...this.records, record].slice(-MAX_RECORDS);
    save(this.records);
    for (const listener of this.listeners) listener(record);
  }
}
