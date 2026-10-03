export const TIMELINE_EVENT_TYPES = Object.freeze([
  "channel-connected",
  "channel-disconnected",
  "name-changed",
  "automation-changed",
] as const);

export type TimelineEventType = (typeof TIMELINE_EVENT_TYPES)[number];
export type TimelineEventProtocolKey = `event:${TimelineEventType}`;

export type TimelineEventData =
  | { readonly type: "name-changed"; readonly to: string }
  | { readonly type: "channel-connected"; readonly label: string }
  | { readonly type: "channel-disconnected"; readonly label: string }
  | {
      readonly type: "automation-changed";
      readonly automationId: string;
      readonly action: string;
      readonly automationName: string;
    };

export type AutomationChangedTimelineEvent = Extract<
  TimelineEventData,
  { readonly type: "automation-changed" }
>;

export interface TimelineEventMetadata {
  readonly protocolKey: TimelineEventProtocolKey;
  readonly entryKind: "event";
  readonly eventType: TimelineEventType;
  readonly placeholderHeight: 32;
  readonly icon: "calendar" | null;
  readonly sourcePath: string;
  readonly chunkFile: string;
}

const metadata = (
  eventType: TimelineEventType,
  sourcePath: string,
  chunkFile: string,
  icon: "calendar" | null = null,
): TimelineEventMetadata => ({
  protocolKey: `event:${eventType}`,
  entryKind: "event",
  eventType,
  placeholderHeight: 32,
  icon,
  sourcePath,
  chunkFile,
});

export const TIMELINE_EVENT_REGISTRY: Readonly<
  Record<TimelineEventProtocolKey, TimelineEventMetadata>
> = Object.freeze({
  "event:channel-connected": metadata(
    "channel-connected",
    "/src/electron-renderer/features/channels/cards/event/channel-connected/view.tsx",
    "view-BEocLLTG.js",
  ),
  "event:channel-disconnected": metadata(
    "channel-disconnected",
    "/src/electron-renderer/features/channels/cards/event/channel-disconnected/view.tsx",
    "view-BMD9fbyy.js",
  ),
  "event:name-changed": metadata(
    "name-changed",
    "/src/electron-renderer/features/chat/cards/event/name-changed/view.tsx",
    "view-LYKe8-aA.js",
  ),
  "event:automation-changed": metadata(
    "automation-changed",
    "/src/electron-renderer/features/agents/cards/event/automation-changed/view.tsx",
    "view-BXS10NUs.js",
    "calendar",
  ),
});

const AUTOMATION_ACTION_VERB: Readonly<Record<string, string>> = Object.freeze({
  created: "Created",
  updated: "Updated",
  enabled: "Enabled",
  disabled: "Disabled",
  deleted: "Deleted",
});

function record(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

export function projectTimelineEvent(value: unknown): TimelineEventData | null {
  const row = record(value);
  if (row == null) return null;

  if (row.type === "name-changed" && typeof row.to === "string") {
    return { type: "name-changed", to: row.to };
  }
  if (
    (row.type === "channel-connected" || row.type === "channel-disconnected")
    && typeof row.label === "string"
  ) {
    return { type: row.type, label: row.label };
  }
  if (
    row.type === "automation-changed"
    && typeof row.automationId === "string"
    && row.automationId.length > 0
    && typeof row.action === "string"
    && row.action.length > 0
    && typeof row.automationName === "string"
    && row.automationName.length > 0
  ) {
    return {
      type: "automation-changed",
      automationId: row.automationId,
      action: row.action,
      automationName: row.automationName,
    };
  }
  return null;
}

export function timelineEventProtocolKey(
  value: unknown,
): TimelineEventProtocolKey | null {
  const event = projectTimelineEvent(value);
  return event == null ? null : `event:${event.type}`;
}

export function timelineEventActionVerb(
  event: TimelineEventData,
): string | null {
  return event.type === "automation-changed"
    ? AUTOMATION_ACTION_VERB[event.action] ?? null
    : null;
}

export function timelineEventDescription(event: TimelineEventData): string {
  switch (event.type) {
    case "name-changed":
      return `Renamed to ${event.to}`;
    case "channel-connected":
      return `Connected to ${event.label}`;
    case "channel-disconnected":
      return `Disconnected from ${event.label}`;
    case "automation-changed":
      return `${timelineEventActionVerb(event) ?? "Changed"} automation "${event.automationName}"`;
  }
}

export interface TimelineEventRegistry {
  metadata(protocolKey: TimelineEventProtocolKey): TimelineEventMetadata;
}

export function createTimelineEventRegistry(): TimelineEventRegistry {
  return {
    metadata(protocolKey) {
      return TIMELINE_EVENT_REGISTRY[protocolKey];
    },
  };
}
