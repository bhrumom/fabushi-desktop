export const LOCAL_BROWSER_AGENT_LIMITS = Object.freeze({
  maxOperationIdLength: 128,
  maxElementRefLength: 64,
  maxTextInputLength: 16_384,
  maxSelectValues: 100,
  maxOptionValueLength: 4_096,
  maxSnapshotElements: 500,
  maxSnapshotFrames: 64,
  maxSnapshotTextLength: 32_768,
  maxWaitMs: 15_000,
  maxScrollDelta: 20_000,
  maxScreenshotBytes: 8 * 1024 * 1024,
  maxUploadBytes: 8 * 1024 * 1024,
});

export interface LocalBrowserAgentTarget {
  readonly conversationId: string;
  readonly workspaceId: string;
  readonly tabId: string;
  readonly expectedGeneration: number;
  readonly expectedControlEpoch: number;
  readonly expectedTaskId: string;
  readonly operationId: string;
}

export interface LocalBrowserAgentResolveTarget {
  readonly type: "resolve-target";
  readonly conversationId: string;
  readonly operationId: string;
  readonly tabId?: string;
}

export interface LocalBrowserAgentTabsAction {
  readonly type: "tabs";
  readonly conversationId: string;
  readonly operationId: string;
  readonly action: "list" | "new" | "select" | "close";
  readonly tabId?: string;
  readonly address?: string;
  readonly expectedControlEpoch?: number;
  readonly expectedTaskId?: string;
}

export type LocalBrowserAgentAction =
  | LocalBrowserAgentResolveTarget
  | LocalBrowserAgentTabsAction
  | (LocalBrowserAgentTarget & { readonly type: "snapshot" })
  | (LocalBrowserAgentTarget & { readonly type: "click"; readonly ref: string })
  | (LocalBrowserAgentTarget & { readonly type: "upload-file"; readonly ref: string })
  | (LocalBrowserAgentTarget & { readonly type: "fill"; readonly ref: string; readonly value: string })
  | (LocalBrowserAgentTarget & { readonly type: "select-option"; readonly ref: string; readonly values: readonly string[] })
  | (LocalBrowserAgentTarget & { readonly type: "type"; readonly ref: string; readonly text: string })
  | (LocalBrowserAgentTarget & { readonly type: "press-key"; readonly key: string })
  | (LocalBrowserAgentTarget & { readonly type: "scroll"; readonly deltaX: number; readonly deltaY: number })
  | (LocalBrowserAgentTarget & { readonly type: "drag"; readonly sourceRef: string; readonly targetRef?: string; readonly targetX?: number; readonly targetY?: number })
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
  readonly frameName: string | null;
  readonly frameId: number | null;
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

export interface LocalBrowserAgentTabSummary {
  readonly tabId: string;
  readonly title: string;
  readonly url: string;
  readonly generation: number;
  readonly active: boolean;
}

export type LocalBrowserAgentActionResult =
  | {
      readonly ok: true;
      readonly operationId: string;
      readonly workspaceId: string;
      readonly tabId: string;
      readonly generation: number;
      readonly controlEpoch: number;
      readonly taskId: string;
      readonly observedUrl: string;
      readonly observedTitle: string;
      readonly snapshot?: LocalBrowserStructuredSnapshot;
      readonly screenshotBase64?: string;
      readonly waited?: boolean;
      readonly tabs?: readonly LocalBrowserAgentTabSummary[];
      readonly uploadedFile?: { readonly name: string; readonly sizeBytes: number };
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

function parseBase(source: Record<string, unknown>): { conversationId: string; operationId: string } | null {
  const conversationId = boundedString(source.conversationId, 256);
  const operationId = boundedString(source.operationId, LOCAL_BROWSER_AGENT_LIMITS.maxOperationIdLength);
  return conversationId == null || operationId == null ? null : { conversationId, operationId };
}

export function parseLocalBrowserAgentAction(value: unknown): LocalBrowserAgentAction | null {
  const source = record(value);
  if (source == null || typeof source.type !== "string") return null;
  const base = parseBase(source);
  if (base == null) return null;

  if (source.type === "resolve-target") {
    const tabId = source.tabId === undefined ? undefined : boundedString(source.tabId, 128);
    if (source.tabId !== undefined && tabId == null) return null;
    return { type: "resolve-target", ...base, ...(tabId == null ? {} : { tabId }) };
  }

  if (source.type === "tabs") {
    if (!["list", "new", "select", "close"].includes(String(source.action))) return null;
    const action = source.action as LocalBrowserAgentTabsAction["action"];
    const tabId = source.tabId === undefined ? undefined : boundedString(source.tabId, 128);
    const address = source.address === undefined ? undefined : boundedString(source.address, 4096);
    if (source.tabId !== undefined && tabId == null) return null;
    if (source.address !== undefined && address == null) return null;
    if ((action === "select" || action === "close") && tabId == null) return null;
    const expectedControlEpoch = source.expectedControlEpoch === undefined
      ? undefined
      : finiteInteger(source.expectedControlEpoch, 1, Number.MAX_SAFE_INTEGER);
    const expectedTaskId = source.expectedTaskId === undefined ? undefined : boundedString(source.expectedTaskId, 128);
    if (source.expectedControlEpoch !== undefined && expectedControlEpoch == null) return null;
    if (source.expectedTaskId !== undefined && expectedTaskId == null) return null;
    if (action !== "list" && (expectedControlEpoch == null || expectedTaskId == null)) return null;
    return {
      type: "tabs",
      ...base,
      action,
      ...(tabId == null ? {} : { tabId }),
      ...(address == null ? {} : { address }),
      ...(expectedControlEpoch == null ? {} : { expectedControlEpoch }),
      ...(expectedTaskId == null ? {} : { expectedTaskId }),
    };
  }

  const workspaceId = boundedString(source.workspaceId, 128);
  const tabId = boundedString(source.tabId, 128);
  const expectedGeneration = finiteInteger(source.expectedGeneration, 0, Number.MAX_SAFE_INTEGER);
  const expectedControlEpoch = finiteInteger(source.expectedControlEpoch, 1, Number.MAX_SAFE_INTEGER);
  const expectedTaskId = boundedString(source.expectedTaskId, 128);
  if (workspaceId == null || tabId == null || expectedGeneration == null || expectedControlEpoch == null || expectedTaskId == null) return null;
  const target = { ...base, workspaceId, tabId, expectedGeneration, expectedControlEpoch, expectedTaskId };

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
    case "upload-file": {
      const ref = boundedString(source.ref, LOCAL_BROWSER_AGENT_LIMITS.maxElementRefLength);
      return ref == null ? null : { ...target, type: "upload-file", ref };
    }
    case "fill": {
      const ref = boundedString(source.ref, LOCAL_BROWSER_AGENT_LIMITS.maxElementRefLength);
      if (ref == null || typeof source.value !== "string" || source.value.length > LOCAL_BROWSER_AGENT_LIMITS.maxTextInputLength) return null;
      return { ...target, type: "fill", ref, value: source.value };
    }
    case "select-option": {
      const ref = boundedString(source.ref, LOCAL_BROWSER_AGENT_LIMITS.maxElementRefLength);
      if (ref == null || !Array.isArray(source.values) || source.values.length === 0 || source.values.length > LOCAL_BROWSER_AGENT_LIMITS.maxSelectValues) return null;
      const values = source.values.flatMap((value): string[] =>
        typeof value === "string" && value.length > 0 && value.length <= LOCAL_BROWSER_AGENT_LIMITS.maxOptionValueLength ? [value] : []
      );
      if (values.length !== source.values.length) return null;
      return { ...target, type: "select-option", ref, values };
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
    case "drag": {
      const sourceRef = boundedString(source.sourceRef, LOCAL_BROWSER_AGENT_LIMITS.maxElementRefLength);
      const targetRef = source.targetRef === undefined ? undefined : boundedString(source.targetRef, LOCAL_BROWSER_AGENT_LIMITS.maxElementRefLength);
      const targetX = source.targetX === undefined ? undefined : finiteInteger(source.targetX, 0, 16_384);
      const targetY = source.targetY === undefined ? undefined : finiteInteger(source.targetY, 0, 16_384);
      if (sourceRef == null) return null;
      if (source.targetRef !== undefined && targetRef == null) return null;
      if (source.targetX !== undefined && targetX == null) return null;
      if (source.targetY !== undefined && targetY == null) return null;
      const byRef = targetRef != null && targetX == null && targetY == null;
      const byPoint = targetRef == null && targetX != null && targetY != null;
      if (!byRef && !byPoint) return null;
      return {
        ...target,
        type: "drag",
        sourceRef,
        ...(targetRef == null ? {} : { targetRef }),
        ...(targetX == null ? {} : { targetX }),
        ...(targetY == null ? {} : { targetY }),
      };
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
