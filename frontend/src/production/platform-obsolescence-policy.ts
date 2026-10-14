export interface PlatformObsolescencePersistence {
  readLastDismissedDate(): Promise<string | null>;
  writeLastDismissedDate(date: string): Promise<void>;
}

export interface ClientPersistenceLike {
  read(key: string): Promise<string | null>;
  write(key: string, value: string): Promise<void>;
}

export interface PlatformObsolescenceSnapshot {
  readonly cutoffDate: string;
  readonly today: string;
  readonly lastDismissedDate: string | null;
  readonly snoozeDays: 7 | 30 | 90;
  readonly hidden: boolean;
  readonly phase: "soon" | "outdated";
}

const DAY_MS = 24 * 60 * 60 * 1000;
export const PLATFORM_OBSOLESCENCE_PERSISTENCE_KEY = "ui/platform-support/outdated-hidden";

function civilDay(value: string): number | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (match == null) return null;
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const time = Date.UTC(year, month - 1, day);
  const date = new Date(time);
  if (date.getUTCFullYear() !== year || date.getUTCMonth() !== month - 1 || date.getUTCDate() !== day) return null;
  return Math.floor(time / DAY_MS);
}

export function platformObsolescenceSnoozeDays(cutoffDate: string, today: string, lastDismissedDate: string): 7 | 30 | 90 {
  const cutoff = civilDay(cutoffDate);
  const current = civilDay(today);
  const dismissed = civilDay(lastDismissedDate);
  if (cutoff == null || current == null || dismissed == null) return 30;
  if (current > cutoff && dismissed <= cutoff) return 7;
  if (current <= cutoff) return 30;
  return 90;
}

/**
 * Exact source-neutral adaptation of the accepted 7/30/90-day suppression
 * policy. Invalid or future persistence must fail open so a support warning is
 * never hidden by corrupt state.
 */
export function projectPlatformObsolescence(input: {
  cutoffDate: string;
  today: string;
  lastDismissedDate: string | null;
}): PlatformObsolescenceSnapshot {
  const cutoff = civilDay(input.cutoffDate);
  const current = civilDay(input.today);
  if (cutoff == null || current == null) throw new TypeError("cutoffDate and today must be valid YYYY-MM-DD civil dates");

  const dismissed = input.lastDismissedDate == null ? null : civilDay(input.lastDismissedDate);
  const validDismissed = dismissed != null && dismissed <= current;
  const snoozeDays = validDismissed
    ? platformObsolescenceSnoozeDays(input.cutoffDate, input.today, input.lastDismissedDate!)
    : current <= cutoff ? 30 : 90;
  const skippedDays = validDismissed ? current - dismissed : Number.POSITIVE_INFINITY;
  return {
    cutoffDate: input.cutoffDate,
    today: input.today,
    lastDismissedDate: validDismissed ? input.lastDismissedDate : null,
    snoozeDays,
    hidden: validDismissed && skippedDays < snoozeDays,
    phase: current <= cutoff ? "soon" : "outdated",
  };
}

export function createPlatformObsolescencePersistence(
  persistence: ClientPersistenceLike,
  key = PLATFORM_OBSOLESCENCE_PERSISTENCE_KEY,
): PlatformObsolescencePersistence {
  return {
    readLastDismissedDate: () => persistence.read(key),
    writeLastDismissedDate: (date) => {
      if (civilDay(date) == null) return Promise.reject(new TypeError("dismissed date must be YYYY-MM-DD"));
      return persistence.write(key, date);
    },
  };
}
