import type { ErrorTray, TrayAction } from "../mahayana-host/contracts";

export type ErrorTrayEvent =
  | { type: "pushed"; tray: ErrorTray }
  | { type: "dismissed"; id: string }
  | { type: "cleared" };

export interface EnqueueErrorOptions {
  readonly agentId: string;
  readonly title: string;
  readonly detail?: string;
  readonly requestId?: string;
  readonly errorKind?: "provider_overloaded";
  readonly rawDetail?: string;
  readonly actions?: readonly TrayAction[];
  readonly dedupeKey?: string;
  readonly count?: number;
}

const MAX_VISIBLE_TRAYS = 20;

export class ErrorTrayQueue {
  private items: ErrorTray[] = [];
  private readonly subscribers = new Set<(event: ErrorTrayEvent) => void>();

  getTrays(): readonly ErrorTray[] {
    return this.items;
  }

  pushError(options: EnqueueErrorOptions): ErrorTray {
    const existingIndex = options.dedupeKey
      ? this.items.findIndex((item) => item.kind === "error" && item.dedupeKey === options.dedupeKey)
      : -1;
    const existing = existingIndex >= 0 ? this.items[existingIndex] : undefined;
    const tray: ErrorTray = {
      kind: "error",
      id: existing?.id ?? crypto.randomUUID(),
      agentId: options.agentId,
      title: options.title,
      detail: options.detail,
      requestId: options.requestId,
      createdAt: Date.now(),
      ...(options.errorKind ? { errorKind: options.errorKind } : {}),
      ...(options.rawDetail ? { rawDetail: options.rawDetail } : {}),
      ...(options.actions?.length ? { actions: [...options.actions] } : {}),
      ...(options.dedupeKey ? {
        dedupeKey: options.dedupeKey,
        count: options.count ?? (existing?.count ?? 0) + 1,
      } : {}),
    };

    if (existingIndex >= 0) {
      this.items = this.items.map((item, index) => index === existingIndex ? tray : item);
    } else {
      this.items = [...this.items, tray];
    }
    this.publish({ type: "pushed", tray });
    this.trimOverflow();
    return tray;
  }

  dismiss(id: string): boolean {
    const exists = this.items.some((item) => item.id === id);
    if (!exists) return false;
    this.items = this.items.filter((item) => item.id !== id);
    this.publish({ type: "dismissed", id });
    return true;
  }

  clearAll(): void {
    if (!this.items.length) return;
    this.items = [];
    this.publish({ type: "cleared" });
  }

  clearForAgent(agentId: string): void {
    const removed = this.items.filter((item) => item.agentId === agentId);
    if (!removed.length) return;
    this.items = this.items.filter((item) => item.agentId !== agentId);
    for (const item of removed) this.publish({ type: "dismissed", id: item.id });
  }

  subscribe(listener: (event: ErrorTrayEvent) => void): () => void {
    this.subscribers.add(listener);
    return () => this.subscribers.delete(listener);
  }

  private trimOverflow(): void {
    const excess = this.items.length - MAX_VISIBLE_TRAYS;
    if (excess <= 0) return;
    const removed = this.items.slice(0, excess);
    this.items = this.items.slice(excess);
    for (const item of removed) this.publish({ type: "dismissed", id: item.id });
  }

  private publish(event: ErrorTrayEvent): void {
    for (const listener of this.subscribers) listener(event);
  }
}
