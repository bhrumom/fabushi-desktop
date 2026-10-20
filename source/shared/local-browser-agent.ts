export const LOCAL_BROWSER_AGENT_LIMITS = Object.freeze({
  maxOperationIdLength: 128,
  maxElementRefLength: 64,
  maxTextInputLength: 16_384,
  maxSnapshotElements: 500,
  maxSnapshotTextLength: 32_768,
  maxWaitMs: 15_000,
  maxScrollDelta: 20_000,
  maxScreenshotBytes: 8 * 1024 * 1024,
});

export interface LocalBrowserAgentTarget {
  readonly conversationId: string;
  readonly workspaceId: string;
  readonly tabId: string;
  readonly expectedGeneration: number;
  readonly operationId: string;
}

export type LocalBrowserAgentAction =
  | (LocalBrowserAgentTarget & { readonly type: "snapshot" })
  | (LocalBrowserAgentTarget & { readonly type: "click"; readonly ref: string })
  | (LocalBrowserAgentTarget & { readonly type: "fill"; readonly ref: string; readonly value: string })
  | (LocalBrowserAgentTarget & { readonly type: "type"; readonly ref: string; readonly text: string })
  | (LocalBrowserAgentTarget & { readonly type: "press-key"; readonly key: string })
  | (LocalBrowserAgentTarget & { readonly type: "scroll"; readonly deltaX: number; readonly deltaY: number })
  | (LocalBrowserAgentTarget & { readonly type: "screenshot" })
  | (LocalBrowserAgentTarget & { readonly type: "wait"; readonly text?: string; readonly ref?: string; readonly timeoutMs?: number })
  | (LocalBrowserAgentTarget & { readonly type: "navigate"; readonly address: string })
  | (LocalBrowserAgentTarget & { readonly type: "back" | "forward" | "reload" });

export interface LocalBrowserSnapshotElement {
  readonly ref: string;
  readonly tag: string;
  readonly role: string | null;
  readonly name: string;
  readonly text: string;
  readonly href: string | null;
  readonly inputType: string | null;
  readonly disabled: boolean;
  readonly editable: boolean;
  readonly checked: boolean | null;
  readonly selected: boolean | null;
  readonly frameUrl: string | null;
}

export interface LocalBrowserStructuredSnapshot {
  readonly workspaceId: string;
  readonly tabId: string;
  readonly generation: number;
  readonly url: string;
  readonly title: string;
  readonly text: string;
  readonly elements: readonly LocalBrowserSnapshotElement[];
}

export type LocalBrowserAgentActionResult =
  | {
      readonly ok: true;
      readonly operationId: string;
      readonly workspaceId: string;
      readonly tabId: string;
      readonly generation: number;
      readonly observedUrl: string;
      readonly observedTitle: string;
      readonly snapshot?: LocalBrowserStructuredSnapshot;
      readonly screenshotBase64?: string;
      readonly waited?: boolean;
    }
  | {
      readonly ok: false;
      readonly operationId: string;
      readonly code: string;
      readonly message: string;
    };

function record(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function boundedString(value: unknown, max: number): string | null {
  if (typeof value !== "string") return null;
  const normalized = value.trim();
  return normalized.length > 0 && normalized.length <= max ? normalized : null;
}

function finiteInteger(value: unknown, min: number, max: number): number | null {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= min && value <= max
    ? value
    : null;
}

export function parseLocalBrowserAgentAction(value: unknown): LocalBrowserAgentAction | null {
  const source = record(value);
  if (source == null || typeof source.type !== "string") return null;
  const conversationId = boundedString(source.conversationId, 256);
  const workspaceId = boundedString(source.workspaceId, 128);
  const tabId = boundedString(source.tabId, 128);
  const operationId = boundedString(source.operationId, LOCAL_BROWSER_AGENT_LIMITS.maxOperationIdLength);
  const expectedGeneration = finiteInteger(source.expectedGeneration, 0, Number.MAX_SAFE_INTEGER);
  if (conversationId == null || workspaceId == null || tabId == null || operationId == null || expectedGeneration == null) return null;
  const target = { conversationId, workspaceId, tabId, operationId, expectedGeneration };
  switch (source.type) {
    case "snapshot":
    case "screenshot":
    case "back":
    case "forward":
    case "reload":
      return { ...target, type: source.type };
    case "click": {
      const ref = boundedString(source.ref, LOCAL_BROWSER_AGENT_LIMITS.maxElementRefLength);
      return ref == null ? null : { ...target, type: "click", ref };
    }
    case "fill": {
      const ref = boundedString(source.ref, LOCAL_BROWSER_AGENT_LIMITS.maxElementRefLength);
      if (ref == null || typeof source.value !== "string" || source.value.length > LOCAL_BROWSER_AGENT_LIMITS.maxTextInputLength) return null;
      return { ...target, type: "fill", ref, value: source.value };
    }
    case "type": {
      const ref = boundedString(source.ref, LOCAL_BROWSER_AGENT_LIMITS.maxElementRefLength);
      if (ref == null || typeof source.text !== "string" || source.text.length > LOCAL_BROWSER_AGENT_LIMITS.maxTextInputLength) return null;
      return { ...target, type: "type", ref, text: source.text };
    }
    case "press-key": {
      const key = boundedString(source.key, 64);
      return key == null ? null : { ...target, type: "press-key", key };
    }
    case "scroll": {
      const deltaX = typeof source.deltaX === "number" && Number.isFinite(source.deltaX) ? source.deltaX : null;
      const deltaY = typeof source.deltaY === "number" && Number.isFinite(source.deltaY) ? source.deltaY : null;
      if (deltaX == null || deltaY == null || Math.abs(deltaX) > LOCAL_BROWSER_AGENT_LIMITS.maxScrollDelta || Math.abs(deltaY) > LOCAL_BROWSER_AGENT_LIMITS.maxScrollDelta) return null;
      return { ...target, type: "scroll", deltaX, deltaY };
    }
    case "wait": {
      const text = source.text === undefined ? undefined : boundedString(source.text, 1024);
      const ref = source.ref === undefined ? undefined : boundedString(source.ref, LOCAL_BROWSER_AGENT_LIMITS.maxElementRefLength);
      const timeoutMs = source.timeoutMs === undefined ? undefined : finiteInteger(source.timeoutMs, 1, LOCAL_BROWSER_AGENT_LIMITS.maxWaitMs);
      if (source.text !== undefined && text == null) return null;
      if (source.ref !== undefined && ref == null) return null;
      if (source.timeoutMs !== undefined && timeoutMs == null) return null;
      if (text == null && ref == null) return null;
      return { ...target, type: "wait", ...(text == null ? {} : { text }), ...(ref == null ? {} : { ref }), ...(timeoutMs == null ? {} : { timeoutMs }) };
    }
    case "navigate": {
      if (typeof source.address !== "string" || source.address.length === 0 || source.address.length > 4096) return null;
      return { ...target, type: "navigate", address: source.address };
    }
    default:
      return null;
  }
}
