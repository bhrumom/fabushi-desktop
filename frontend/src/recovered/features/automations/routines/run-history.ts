export type RoutineRunStatus = "running" | "ok" | "error";

export interface RoutineRun {
  readonly id: string;
  readonly status: RoutineRunStatus;
  readonly startedAt: number;
  readonly detail?: string | null;
  readonly event?: string | null;
}

export interface RoutineRunPresentation {
  readonly id: string;
  readonly title?: string;
  readonly timestampLabel: string;
  readonly status: RoutineRunStatus;
  readonly ariaLabel: "Running" | "Succeeded" | "Failed";
  readonly iconName: "loading" | "check" | "close";
  readonly statusRole?: "status";
}

export interface RoutineRunHistoryPresentation {
  readonly empty: boolean;
  readonly rows: readonly RoutineRunPresentation[];
}

const MONTHS = Object.freeze([
  "Jan", "Feb", "Mar", "Apr", "May", "Jun",
  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
] as const);
const WEEKDAYS = Object.freeze([
  "Sunday", "Monday", "Tuesday", "Wednesday",
  "Thursday", "Friday", "Saturday",
] as const);

interface ZonedParts {
  year: number;
  month: number;
  day: number;
  weekday: number;
  hour: number;
  minute: number;
}

function zonedParts(timestamp: number, timeZone?: string): ZonedParts {
  const formatter = new Intl.DateTimeFormat("en-US", {
    ...(timeZone == null ? {} : { timeZone }),
    year: "numeric",
    month: "numeric",
    day: "numeric",
    weekday: "long",
    hour: "numeric",
    minute: "2-digit",
    hour12: false,
  });
  const map = Object.fromEntries(
    formatter.formatToParts(new Date(timestamp))
      .filter((part) => part.type !== "literal")
      .map((part) => [part.type, part.value]),
  );
  const weekday = WEEKDAYS.indexOf(
    map.weekday as (typeof WEEKDAYS)[number],
  );
  return {
    year: Number(map.year),
    month: Number(map.month),
    day: Number(map.day),
    weekday: weekday < 0 ? new Date(timestamp).getDay() : weekday,
    hour: Number(map.hour) % 24,
    minute: Number(map.minute),
  };
}

function dayKey(timestamp: number, timeZone?: string): number {
  const parts = zonedParts(timestamp, timeZone);
  return Date.UTC(parts.year, parts.month - 1, parts.day);
}

function timeLabel(parts: ZonedParts): string {
  const hour = parts.hour % 12 || 12;
  return `${hour}:${String(parts.minute).padStart(2, "0")} ${parts.hour < 12 ? "AM" : "PM"}`;
}

function capitalized(value: string): string {
  return value.charAt(0).toUpperCase() + value.slice(1);
}

export function formatRoutineRunTimestamp(
  startedAt: number,
  now: number,
  timeZone?: string,
): string {
  const delta = startedAt - now;
  if (delta > 0 && delta < 3_600_000) {
    return capitalized(`in ${Math.ceil(delta / 60_000)} min`);
  }
  if (delta <= 0 && -delta < 60_000) return "Just now";
  if (delta <= 0 && -delta < 3_600_000) {
    return `${Math.floor(-delta / 60_000)} min ago`;
  }

  const current = zonedParts(now, timeZone);
  const started = zonedParts(startedAt, timeZone);
  const dayDelta = Math.round(
    (dayKey(startedAt, timeZone) - dayKey(now, timeZone)) / 86_400_000,
  );
  const clock = timeLabel(started);

  if (dayDelta === 0) return `Today at ${clock}`;
  if (dayDelta === 1) return `Tomorrow at ${clock}`;
  if (dayDelta === -1) return `Yesterday at ${clock}`;
  if (dayDelta > 1 && dayDelta < 7) {
    return `${WEEKDAYS[started.weekday]} at ${clock}`;
  }
  if (dayDelta < -1 && dayDelta > -7) {
    return `Last ${WEEKDAYS[started.weekday]} at ${clock}`;
  }

  const date = `${MONTHS[started.month - 1]} ${started.day}`;
  return started.year === current.year
    ? `${date} at ${clock}`
    : `${date}, ${started.year} at ${clock}`;
}

export function presentRoutineRun(
  run: RoutineRun,
  now: number,
  timeZone?: string,
): RoutineRunPresentation {
  const title = run.detail ?? run.event ?? undefined;
  const common = {
    id: run.id,
    ...(title == null ? {} : { title }),
    timestampLabel: formatRoutineRunTimestamp(run.startedAt, now, timeZone),
    status: run.status,
  };

  switch (run.status) {
    case "running":
      return {
        ...common,
        ariaLabel: "Running",
        iconName: "loading",
        statusRole: "status",
      };
    case "ok":
      return { ...common, ariaLabel: "Succeeded", iconName: "check" };
    case "error":
      return { ...common, ariaLabel: "Failed", iconName: "close" };
  }
}

export function presentRoutineRunHistory(
  runs: readonly RoutineRun[],
  now: number,
  timeZone?: string,
): RoutineRunHistoryPresentation {
  return runs.length === 0
    ? { empty: true, rows: [] }
    : {
        empty: false,
        rows: runs.map((run) => presentRoutineRun(run, now, timeZone)),
      };
}
