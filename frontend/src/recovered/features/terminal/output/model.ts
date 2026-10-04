export type TerminalOutputStatus = "idle" | "running" | "exited" | "error";

export interface TerminalOutputSnapshot {
  sessionId: string;
  command: string;
  cwd: string | null;
  output: string;
  status: TerminalOutputStatus;
  exitCode: number | null;
}

function record(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function nonEmptyString(value: unknown): string | null {
  return typeof value === "string" && value.length > 0 ? value : null;
}

function finiteNumber(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

export function normalizeTerminalOutput(value: unknown): string {
  if (typeof value !== "string") return "";
  return value.replaceAll("\r\n", "\n").replaceAll("\r", "\n");
}

export function projectTerminalOutput(value: unknown): TerminalOutputSnapshot | null {
  const root = record(value);
  if (root == null) return null;

  const metadata = record(root.metadata);
  const currentCommand =
    record(root.currentCommand)
    ?? record(metadata?.currentCommand);

  const command =
    nonEmptyString(root.command)
    ?? nonEmptyString(currentCommand?.command);

  const numericId =
    finiteNumber(root.terminalInstanceId)
    ?? finiteNumber(root.terminal_instance_id);
  const sessionId =
    nonEmptyString(root.sessionId)
    ?? (numericId == null ? null : String(numericId))
    ?? nonEmptyString(root.terminalInstancePath)
    ?? nonEmptyString(root.terminal_instance_path);

  if (command == null || sessionId == null) return null;

  const exitCode = finiteNumber(root.exitCode ?? root.exit_code);
  const running =
    root.status === "running"
    || root.isRunning === true
    || root.isRunningInBackground === true;
  const failed =
    root.status === "error"
    || root.rejected === true
    || (exitCode != null && exitCode !== 0);

  const status: TerminalOutputStatus = running
    ? "running"
    : failed
      ? "error"
      : root.status === "idle"
        ? "idle"
        : "exited";

  return {
    sessionId,
    command,
    cwd:
      nonEmptyString(root.cwd)
      ?? nonEmptyString(root.cwdFull)
      ?? nonEmptyString(metadata?.cwd),
    output: normalizeTerminalOutput(
      root.outputRaw ?? root.output_raw ?? root.output ?? root.contents,
    ),
    status,
    exitCode,
  };
}
