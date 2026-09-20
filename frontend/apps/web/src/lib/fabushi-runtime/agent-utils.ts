export const AGENT_MESSAGE_LIMIT = 8_000;

const MODE_LABELS: Readonly<Record<string, string>> = {
  agent: "Agent",
  plan: "Plan",
  debug: "Debug",
  chat: "Ask",
  ask: "Ask",
  multitask: "Multitask",
};

/**
 * Product-level execution policy shared by every Mahayana chat turn.
 *
 * Keep this deliberately short: it is injected through modeStatement on every
 * request, so it must steer tool choice without consuming a meaningful share
 * of the model context window.
 */
export const EFFICIENT_AGENT_RUN_POLICY = [
  "Prefer event-driven work over polling when an event source exists; use a schedule only when no reliable event can wake the task.",
  "Prefer connected Connector/MCP/API tools over browser or computer UI automation when they can complete the same action.",
  "If progress is blocked or you cannot make further useful progress, report the blocker immediately and state the exact human input or action needed.",
  "Keep event filters narrow and preserve approval requirements for destructive, external, publish, send, delete, purchase, or production-changing actions.",
].join(" ");

export function buildModeTransitionNote(mode: string): string {
  const label = MODE_LABELS[mode] ?? (mode.trim() || "Agent");
  return `Active mode: ${label}. Apply this mode to the current task from this point forward. ${EFFICIENT_AGENT_RUN_POLICY}`;
}

export function normalizeAutomationSchedule(value: string): string {
  return value.trim().split(/\s+/u).filter(Boolean).join(" ");
}

export function clampAgentMessage(value: string): string {
  return value.trim().slice(0, AGENT_MESSAGE_LIMIT);
}

export function estimateTextTokens(value: string): number {
  if (!value) return 0;
  let estimate = 0;
  for (const char of value) {
    const codePoint = char.codePointAt(0) ?? 0;
    if (/\s/u.test(char)) estimate += 0.05;
    else if (
      (codePoint >= 0x3400 && codePoint <= 0x9fff) ||
      (codePoint >= 0x3040 && codePoint <= 0x30ff) ||
      (codePoint >= 0xac00 && codePoint <= 0xd7af) ||
      codePoint > 0xffff
    ) estimate += 1;
    else estimate += 0.25;
  }
  return Math.max(1, Math.ceil(estimate));
}

type NumberedLine = { lineNumber?: number; text: string };
type LineNumberOptions = {
  compact?: boolean;
  prefixStyle?: "pipe" | "label";
  sparseEvery?: number;
};

export function formatNumberedLines(
  options: LineNumberOptions,
  source: string | readonly NumberedLine[],
  firstLine = 1,
): string {
  const rows: NumberedLine[] = typeof source === "string"
    ? source.split("\n").map((text, index) => ({ lineNumber: firstLine + index, text }))
    : [...source];
  const width = Math.max(4, String(rows.at(-1)?.lineNumber ?? firstLine).length);
  return rows.map((row) => {
    if (!Number.isInteger(row.lineNumber)) return "…";
    const line = row.lineNumber as number;
    if (options.sparseEvery && line % options.sparseEvery !== 0) return row.text;
    if (options.prefixStyle === "label") return `L${line}:${row.text}`;
    const number = String(line).padStart(options.compact ? 1 : width, " ");
    return `${number}${options.compact ? "  " : " | "}${row.text}`;
  }).join("\n");
}
