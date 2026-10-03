export const VNC_VIEWER_VISIBLE_CHANNEL = "sand:vnc-viewer-visible";
export const VNC_HOST_KEY_CHANNEL = "sand:vnc-host-key";
export const VNC_SESSION_CHANNEL = "sand:vnc-session";
export const VNC_LIVENESS_CHANNEL = "sand:vnc-liveness";
export const VNC_PARTITION = "persist:sand-forever-box";
export const VNC_WEB_PREFERENCES = "contextIsolation=yes, nodeIntegration=no, sandbox=no";
export const VNC_STATUS_TIMEOUT_MS = 15_000;
export const VNC_CRASH_LIMIT = 3;
export const VNC_CRASH_WINDOW_MS = 60_000;
export const VNC_FOCUS_DELAY_MS = 32;
export const VNC_WARM_PREVIEW_LIMIT = 3;
export const DIRECT_MONITOR_LIMIT = 3;
export const COMPUTER_ACTIVE_HOLD_MS = 2_500;

export type ComputerPhase = "off" | "starting" | "sleeping" | "local" | "running" | "pulling";
export type ComputerReadState = "unknown" | "known" | "error";
export type ComputerHandoffResolution =
  | "waiting"
  | "handed_back"
  | "replied"
  | "dismissed"
  | string;

export interface ComputerHandoff {
  requestId: string;
  instruction: string;
  snapshotDataUrl?: string;
}

export interface ComputerStatusProjection {
  status: Record<string, unknown> | null;
  phase: ComputerPhase;
  readState: ComputerReadState;
  isStatusKnown: boolean;
  isStatusUnavailable: boolean;
  pullPercent: number | null;
  vncUrl: string | null;
  handoff: ComputerHandoff | null;
  windows: unknown[];
}

export interface ComputerMonitor {
  subagentId: string;
  title: string;
  vncUrl: string;
  handoff: ComputerHandoff | null;
}

export interface ComputerStageCopy {
  message: string;
  progressPercent: number | null;
  isBusy: boolean;
  hasRetry: boolean;
}

export interface VncLivenessReport {
  phase: "post_connect";
  stallMs: number;
  keys: number;
  clicks: number;
  moves: number;
  inBytes: number;
}

export interface VncSessionMessage {
  phase: "rfb_connect" | "rfb_disconnect" | "reconnect";
  clean?: boolean;
}

export interface ComputerCursor {
  x: number;
  y: number;
  type: "click" | "drag" | "move" | "scroll";
  seq: number;
  clickSeq: number;
  msSinceMove: number | null;
  lastMovedAtMs: number | null;
}

export interface ComputerCursorPresentation {
  isGliding: boolean;
  isVisible: boolean;
  press: { key: number; delayMs: number } | null;
}

function recordValue(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function nonnegativeNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && value >= 0;
}

export function projectHandoff(value: unknown): ComputerHandoff | null {
  const item = recordValue(value);
  if (
    item === null ||
    typeof item.requestId !== "string" ||
    typeof item.instruction !== "string"
  ) return null;
  return {
    requestId: item.requestId,
    instruction: item.instruction,
    ...(typeof item.snapshotDataUrl === "string" ? { snapshotDataUrl: item.snapshotDataUrl } : {}),
  };
}

export function projectComputerStatus(
  value: unknown,
  readState: ComputerReadState,
  isEnsureStarting = false,
): ComputerStatusProjection {
  const status = recordValue(value);
  const vncUrl =
    status?.state === "running" &&
    typeof status.vncUrl === "string" &&
    status.vncUrl.length > 0
      ? status.vncUrl
      : null;
  const pull = recordValue(status?.pull);

  let phase: ComputerPhase;
  if (pull !== null) phase = "pulling";
  else if (status?.state === "running") phase = vncUrl === null ? "local" : "running";
  else if (isEnsureStarting) phase = "starting";
  else if (status?.state === "hibernated") phase = "sleeping";
  else phase = "off";

  return {
    status,
    phase,
    readState,
    isStatusKnown: readState === "known",
    isStatusUnavailable: readState === "error",
    pullPercent: pull !== null && typeof pull.percent === "number" ? pull.percent : null,
    vncUrl,
    handoff: projectHandoff(status?.handoff),
    windows: vncUrl !== null && Array.isArray(status?.windows) ? status.windows : [],
  };
}

export function projectComputerMonitors(
  subagents: unknown,
  getStatus: (subagentId: string) => unknown,
): ComputerMonitor[] {
  if (!Array.isArray(subagents)) return [];
  const monitors: ComputerMonitor[] = [];
  for (const raw of subagents) {
    const subagent = recordValue(raw);
    if (
      subagent === null ||
      subagent.status !== "running" ||
      subagent.subagentType !== "computerUse" ||
      typeof subagent.subagentId !== "string"
    ) continue;

    const status = recordValue(getStatus(subagent.subagentId));
    if (
      status === null ||
      status.state !== "running" ||
      typeof status.vncUrl !== "string" ||
      status.vncUrl.length === 0
    ) continue;

    const title = typeof subagent.title === "string" ? subagent.title.trim() : "";
    monitors.push({
      subagentId: subagent.subagentId,
      title: title.length > 0 ? title : "Subagent",
      vncUrl: status.vncUrl,
      handoff: null,
    });
  }
  return monitors;
}

export function isComputerUseTaskActive(tasks: unknown): boolean {
  return Array.isArray(tasks) && tasks.some((raw) => recordValue(raw)?.subagentType === "computerUse");
}

export function computerStageCopy(input: {
  isScreenLoading: boolean;
  isScreenUnavailable: boolean;
  subjectLabel: string;
  emptyMessage?: string;
  isEmptyLoading: boolean;
  pullPercent: number | null;
}): ComputerStageCopy {
  if (input.isScreenLoading) {
    return {
      message: `Switching to ${input.subjectLabel}'s screen…`,
      progressPercent: null,
      isBusy: true,
      hasRetry: false,
    };
  }
  if (input.isScreenUnavailable) {
    return {
      message: `Can't reach ${input.subjectLabel}'s screen`,
      progressPercent: null,
      isBusy: false,
      hasRetry: true,
    };
  }
  if (input.pullPercent !== null) {
    return {
      message: "Setting up the computer",
      progressPercent: input.pullPercent,
      isBusy: true,
      hasRetry: false,
    };
  }
  return {
    message: input.emptyMessage ?? "Booting up the computer",
    progressPercent: null,
    isBusy: input.isEmptyLoading,
    hasRetry: false,
  };
}

export function isSpecialTreatmentVnc(value: string): boolean {
  try {
    return new URL(value).pathname.endsWith("/sand-special-treatment-v1/vnc.html");
  } catch {
    return false;
  }
}

export function vncDimensions(value: string): { width: number; height: number } {
  return isSpecialTreatmentVnc(value)
    ? { width: 2048, height: 2048 }
    : { width: 1280, height: 800 };
}

export function retainWarmVncSources(
  previous: string[],
  source: string | null,
  maxWarm = VNC_WARM_PREVIEW_LIMIT,
): string[] {
  if (source === null || previous[0] === source) return previous;
  const limit = Math.max(1, maxWarm);
  const next = [source, ...previous.filter((entry) => entry !== source)];
  return next.length > limit ? next.slice(0, limit) : next;
}

export function vncViewerUrl(value: string, interactive: boolean): string {
  const url = new URL(value);
  url.searchParams.set("autoconnect", "true");
  url.searchParams.set("resize", "scale");
  url.searchParams.set("reconnect", "true");
  if (interactive) url.searchParams.set("sandInteractive", "1");
  return url.toString();
}

export function vncIdentity(value: string): { host?: string; display?: string } {
  try {
    const url = new URL(value);
    const path = url.searchParams.get("path");
    if (path !== null) {
      const query = path.indexOf("?");
      if (query >= 0) {
        const token = new URLSearchParams(path.slice(query + 1)).get("token");
        if (token !== null && token.length > 0) {
          return { host: url.host, display: token };
        }
      }
    }
    return { host: url.host, display: "primary" };
  } catch {
    return {};
  }
}

export function parseVncSession(value: unknown): VncSessionMessage | null {
  if (typeof value !== "string") return null;
  try {
    const decoded: unknown = JSON.parse(value);
    const item = recordValue(decoded);
    if (
      item === null ||
      (item.phase !== "rfb_connect" && item.phase !== "rfb_disconnect" && item.phase !== "reconnect")
    ) return null;
    return {
      phase: item.phase,
      ...(typeof item.clean === "boolean" ? { clean: item.clean } : {}),
    };
  } catch {
    return null;
  }
}

export function parseVncLiveness(value: unknown): VncLivenessReport | null {
  const item = recordValue(value);
  if (item === null || item.phase !== "post_connect") return null;
  if (![item.stallMs, item.keys, item.clicks, item.moves, item.inBytes].every(nonnegativeNumber)) {
    return null;
  }
  return {
    phase: "post_connect",
    stallMs: item.stallMs as number,
    keys: item.keys as number,
    clicks: item.clicks as number,
    moves: item.moves as number,
    inBytes: item.inBytes as number,
  };
}

export function projectComputerCursor(
  value: unknown,
  previous: ComputerCursor | null,
  nowMs: number,
): ComputerCursor | null {
  const item = recordValue(value);
  if (
    item === null ||
    typeof item.agentId !== "string" ||
    (item.type !== "click" && item.type !== "drag" && item.type !== "move" && item.type !== "scroll") ||
    !nonnegativeNumber(item.x) ||
    !nonnegativeNumber(item.y)
  ) return null;

  const changed = previous !== null && (item.x !== previous.x || item.y !== previous.y);
  const lastMovedAtMs = changed ? nowMs : previous?.lastMovedAtMs ?? null;

  return {
    x: item.x,
    y: item.y,
    type: item.type,
    seq: (previous?.seq ?? 0) + 1,
    clickSeq: (previous?.clickSeq ?? 0) + (item.type === "click" ? 1 : 0),
    msSinceMove: lastMovedAtMs === null ? null : Math.max(0, nowMs - lastMovedAtMs),
    lastMovedAtMs,
  };
}

export function computerCursorPresentation(
  cursor: ComputerCursor | null,
  hasFrame: boolean,
): ComputerCursorPresentation {
  const visible = cursor !== null && hasFrame;
  return {
    isGliding: (cursor?.seq ?? 0) > 1,
    isVisible: visible,
    press:
      visible && cursor?.type === "click"
        ? {
            key: cursor.clickSeq,
            delayMs: cursor.msSinceMove === null ? 0 : Math.max(0, 500 - cursor.msSinceMove),
          }
        : null,
  };
}

export function firstSelectedMonitor(
  monitors: readonly ComputerMonitor[],
  requested: string | null,
): string | null {
  if (requested !== null && monitors.some((monitor) => monitor.subagentId === requested)) {
    return requested;
  }
  return monitors.find((monitor) => monitor.handoff !== null)?.subagentId
    ?? monitors[0]?.subagentId
    ?? null;
}

export function stepSelectedMonitor(
  monitors: readonly ComputerMonitor[],
  current: string | null,
  delta: -1 | 1,
): string | null {
  if (monitors.length < 2) return current;
  const found = monitors.findIndex((monitor) => monitor.subagentId === current);
  const base = found === -1 ? 0 : found;
  const nextIndex = (base + delta + monitors.length) % monitors.length;
  return monitors[nextIndex]?.subagentId ?? current;
}

export function handoffStatusLabel(
  status: ComputerHandoffResolution,
): { label: string; muted: boolean } {
  if (status === "waiting") return { label: "Action needed", muted: false };
  if (status === "handed_back") return { label: "Done", muted: false };
  if (status === "replied") return { label: "Answered", muted: false };
  if (status === "dismissed") return { label: "Skipped", muted: true };
  return { label: "Status unavailable", muted: true };
}
