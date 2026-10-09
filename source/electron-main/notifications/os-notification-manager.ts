import {
  SandOsNotificationDecider,
  buildNotificationContent,
  toNotificationSnapshot,
  type NotificationAgent,
  type NotificationTransition,
} from "../../shared/os-notification.js";

export interface DesktopNotificationPort {
  on(event: "click", listener: () => void): void;
  on(event: "reply", listener: (event: unknown, reply: string) => void): void;
  on(event: "action", listener: (event: unknown, index: number) => void): void;
  once(event: "close", listener: () => void): void;
  show(): void;
  close(): void;
}

export interface DesktopNotificationOptions {
  readonly title: string;
  readonly body: string;
  readonly silent: boolean;
  readonly urgency: "critical" | "normal";
  readonly hasReply?: boolean;
  readonly replyPlaceholder?: string;
  readonly actions?: readonly { readonly type: "button"; readonly text: string }[];
}

export interface NotificationWindowPort {
  isFocused(): boolean;
  isMinimized(): boolean;
  restore(): void;
  show(): void;
  focus(): void;
}

export type DesktopNotificationScope =
  | { readonly kind: "agent"; readonly agentId: string }
  | {
      readonly kind: "conversation";
      readonly accountId: string;
      readonly conversationId: string;
      readonly childId?: string | null;
    };

export interface ScopedDesktopNotification {
  readonly scope: DesktopNotificationScope;
  readonly title: string;
  readonly body: string;
  readonly silent: boolean;
  readonly urgency: "critical" | "normal";
  readonly onActivate: () => void;
  readonly onReply?: (reply: string) => void | Promise<void>;
  readonly replyPlaceholder?: string;
  readonly onMarkRead?: () => void | Promise<void>;
  readonly markReadLabel?: string;
}

function scopeKey(scope: DesktopNotificationScope): string {
  return scope.kind === "agent"
    ? JSON.stringify(["agent", scope.agentId])
    : JSON.stringify(["conversation", scope.accountId, scope.conversationId, scope.childId ?? null]);
}

export class SandOsNotificationManager {
  private decider = new SandOsNotificationDecider();
  private readonly activeByScope = new Map<string, Set<DesktopNotificationPort>>();
  private hasSeededBaseline = false;
  private preSeedDeltas: Array<{ readonly agent: NotificationAgent }> = [];

  constructor(private readonly deps: {
    readonly getWindow: () => NotificationWindowPort | null;
    readonly isSupported: () => boolean;
    readonly createNotification: (options: DesktopNotificationOptions) => DesktopNotificationPort;
    readonly openAgent: (agentId: string) => void;
    readonly reportActionFailure?: (operation: "reply" | "mark-read", error: unknown) => void;
    readonly now?: () => number;
  }) {}

  handleAgentsEvent(event: { readonly agents: readonly NotificationAgent[] }): void {
    const window = this.deps.getWindow();
    if (window == null || !this.deps.isSupported()) return;
    const transitions = this.decider.decide({ agents: event.agents.map(toNotificationSnapshot), isWindowFocused: window.isFocused(), nowMs: (this.deps.now ?? Date.now)() });
    this.flushPreSeedDeltas();
    for (const transition of transitions) this.show(transition);
  }

  handleAgentUpsertedEvent(event: { readonly agent: NotificationAgent }): void {
    if (!this.hasSeededBaseline) { this.preSeedDeltas.push(event); return; }
    this.processDelta(event);
  }

  seedBaseline(agents: readonly NotificationAgent[]): void {
    this.decider.seedBaseline(agents.map(toNotificationSnapshot));
    this.flushPreSeedDeltas();
  }

  forget(agentId: string): void {
    this.decider.forget(agentId);
    this.clearScope({ kind: "agent", agentId });
  }

  showScoped(input: ScopedDesktopNotification): boolean {
    if (!this.deps.isSupported()) return false;
    const notification = this.deps.createNotification({
      title: input.title,
      body: input.body,
      silent: input.silent,
      urgency: input.urgency,
      ...(input.onReply == null
        ? {}
        : {
            hasReply: true,
            replyPlaceholder: input.replyPlaceholder ?? "Reply",
          }),
      ...(input.onMarkRead == null
        ? {}
        : {
            actions: [{ type: "button" as const, text: input.markReadLabel ?? "Mark as Read" }],
          }),
    });
    const key = scopeKey(input.scope);
    let active = this.activeByScope.get(key);
    if (active == null) {
      active = new Set<DesktopNotificationPort>();
      this.activeByScope.set(key, active);
    }
    active.add(notification);
    notification.on("click", () => {
      this.focusWindow();
      input.onActivate();
    });
    if (input.onReply != null) {
      notification.on("reply", (_event, reply) => {
        const text = reply.trim();
        if (!text) return;
        this.runScopedAction(input.scope, "reply", () => input.onReply!(text));
      });
    }
    if (input.onMarkRead != null) {
      notification.on("action", (_event, index) => {
        if (index !== 0) return;
        this.runScopedAction(input.scope, "mark-read", input.onMarkRead!);
      });
    }
    notification.once("close", () => {
      const current = this.activeByScope.get(key);
      if (current == null) return;
      current.delete(notification);
      if (current.size === 0) this.activeByScope.delete(key);
    });
    notification.show();
    return true;
  }

  clearScope(scope: DesktopNotificationScope): void {
    const key = scopeKey(scope);
    const active = this.activeByScope.get(key);
    if (active == null) return;
    this.activeByScope.delete(key);
    for (const notification of [...active]) notification.close();
  }

  reset(): void {
    this.decider = new SandOsNotificationDecider();
    this.hasSeededBaseline = false;
    this.preSeedDeltas = [];
    const active = [...this.activeByScope.values()].flatMap((notifications) => [...notifications]);
    this.activeByScope.clear();
    for (const notification of active) notification.close();
  }

  private processDelta(event: { readonly agent: NotificationAgent }): void {
    const snapshot = toNotificationSnapshot(event.agent);
    const window = this.deps.getWindow();
    if (window == null || !this.deps.isSupported()) { this.decider.observeAgent(snapshot); return; }
    for (const transition of this.decider.decideAgent(snapshot, { isWindowFocused: window.isFocused(), nowMs: (this.deps.now ?? Date.now)() })) this.show(transition);
  }

  private flushPreSeedDeltas(): void {
    if (this.hasSeededBaseline) return;
    this.hasSeededBaseline = true;
    const buffered = this.preSeedDeltas;
    this.preSeedDeltas = [];
    for (const event of buffered) this.processDelta(event);
  }

  private show(transition: NotificationTransition): void {
    const { title, body } = buildNotificationContent(transition);
    this.showScoped({
      scope: { kind: "agent", agentId: transition.agentId },
      title,
      body,
      silent: transition.kind === "agent-done",
      urgency: transition.kind === "agent-needs-input" ? "critical" : "normal",
      onActivate: () => this.deps.openAgent(transition.agentId),
    });
  }

  private runScopedAction(
    scope: DesktopNotificationScope,
    operation: "reply" | "mark-read",
    action: () => void | Promise<void>,
  ): void {
    try {
      const result = action();
      if (result == null || typeof (result as PromiseLike<void>).then !== "function") {
        this.clearScope(scope);
        return;
      }
      void Promise.resolve(result).then(
        () => this.clearScope(scope),
        (error) => this.deps.reportActionFailure?.(operation, error),
      );
    } catch (error) {
      this.deps.reportActionFailure?.(operation, error);
    }
  }

  private focusWindow(): void {
    const window = this.deps.getWindow();
    if (window == null) return;
    if (window.isMinimized()) window.restore();
    window.show();
    window.focus();
  }
}
