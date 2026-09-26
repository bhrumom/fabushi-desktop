export type ToolResultCardKind = "file-edit" | "file-write" | "shell";
export type ToolResultCardStatus =
  | "running"
  | "success"
  | "error"
  | "denied"
  | "rejected"
  | "cancelled"
  | "background";

export interface ToolResultCardSnapshot {
  kind: ToolResultCardKind;
  toolCallId: string | null;
  status: ToolResultCardStatus;
  path: string | null;
  command: string | null;
  workingDirectory: string | null;
  summary: string;
  output: string;
  diff: string;
  isStreaming: boolean;
  isBackground: boolean;
}

type UnknownRecord = Record<string, unknown>;

function objectValue(value: unknown): UnknownRecord | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as UnknownRecord
    : null;
}

function nonEmptyString(value: unknown): string | null {
  return typeof value === "string" && value.length > 0 ? value : null;
}

function pickString(...values: unknown[]): string | null {
  for (const value of values) {
    const candidate = nonEmptyString(value);
    if (candidate !== null) return candidate;
  }
  return null;
}

function normalizedText(value: unknown): string {
  return typeof value === "string"
    ? value.replaceAll("\r\n", "\n").replaceAll("\r", "\n")
    : "";
}

function nestedBranch(source: UnknownRecord | null, names: readonly string[]): UnknownRecord | null {
  if (source === null) return null;
  for (const name of names) {
    const candidate = objectValue(source[name]);
    if (candidate !== null) return candidate;
  }
  return null;
}

const RESULT_BRANCHES = [
  "success",
  "failure",
  "timeout",
  "rejected",
  "permission_denied",
  "permissionDenied",
  "error",
  "spawn_error",
  "spawnError",
  "no_space",
  "noSpace",
  "file_not_found",
  "fileNotFound",
  "read_permission_denied",
  "readPermissionDenied",
  "write_permission_denied",
  "writePermissionDenied",
] as const;

function resultPayload(result: UnknownRecord | null): UnknownRecord | null {
  return nestedBranch(result, RESULT_BRANCHES);
}

function statusName(value: unknown): ToolResultCardStatus | null {
  const name = nonEmptyString(value)?.toLowerCase();
  if (name === "running" || name === "streaming" || name === "pending") return "running";
  if (name === "success" || name === "done" || name === "completed" || name === "exited") return "success";
  if (name === "background" || name === "backgrounded") return "background";
  if (name === "denied" || name === "permission-denied" || name === "permission_denied") return "denied";
  if (name === "rejected" || name === "declined") return "rejected";
  if (name === "cancelled" || name === "canceled" || name === "aborted") return "cancelled";
  if (name === "error" || name === "failed" || name === "failure" || name === "timeout") return "error";
  return null;
}

function inferKind(
  input: UnknownRecord,
  args: UnknownRecord | null,
  result: UnknownRecord | null,
): ToolResultCardKind | null {
  const label = pickString(input.kind, input.tool, input.toolName, input.tool_name)?.toLowerCase() ?? "";
  if (label.includes("shell") || label.includes("bash") || label.includes("terminal")) return "shell";
  if (label.includes("write")) return "file-write";
  if (label.includes("edit")) return "file-edit";

  if (pickString(input.command, args?.command) !== null) return "shell";
  if (
    pickString(args?.fileText, args?.file_text) !== null ||
    result?.linesCreated !== undefined ||
    result?.lines_created !== undefined
  ) return "file-write";
  if (
    args?.edits !== undefined ||
    args?.streamContent !== undefined ||
    args?.stream_content !== undefined ||
    result?.diffString !== undefined ||
    result?.diff_string !== undefined ||
    result?.linesAdded !== undefined ||
    result?.lines_added !== undefined
  ) return "file-edit";
  return null;
}

function resultStatus(kind: ToolResultCardKind, result: UnknownRecord | null): ToolResultCardStatus | null {
  if (result === null) return null;

  if (kind === "shell") {
    if (result.permission_denied !== undefined || result.permissionDenied !== undefined) return "denied";
    if (result.rejected !== undefined) return "rejected";
    const failure = objectValue(result.failure);
    if (
      failure !== null &&
      (failure.aborted === true ||
        nonEmptyString(failure.abortReason) !== null ||
        nonEmptyString(failure.abort_reason) !== null)
    ) return "cancelled";
    if (
      result.timeout !== undefined ||
      result.spawn_error !== undefined ||
      result.spawnError !== undefined ||
      result.failure !== undefined
    ) return "error";
    if (result.success !== undefined) {
      return result.is_background === true || result.isBackground === true ? "background" : "success";
    }
    return null;
  }

  if (
    result.permission_denied !== undefined ||
    result.permissionDenied !== undefined ||
    result.read_permission_denied !== undefined ||
    result.readPermissionDenied !== undefined ||
    result.write_permission_denied !== undefined ||
    result.writePermissionDenied !== undefined
  ) return "denied";
  if (result.rejected !== undefined) return "rejected";
  if (
    result.error !== undefined ||
    result.file_not_found !== undefined ||
    result.fileNotFound !== undefined ||
    result.no_space !== undefined ||
    result.noSpace !== undefined
  ) return "error";
  if (result.success !== undefined) return "success";
  return null;
}

function finalOutput(kind: ToolResultCardKind, result: UnknownRecord | null, input: UnknownRecord): string {
  const payload = resultPayload(result);
  if (kind === "shell") {
    return normalizedText(pickString(
      payload?.interleavedOutput,
      payload?.interleaved_output,
      payload?.stdout,
      payload?.stderr,
      input.output,
      input.contents,
    ));
  }
  return normalizedText(pickString(payload?.output, payload?.message, input.output));
}

function finalDiff(result: UnknownRecord | null, input: UnknownRecord): string {
  const payload = resultPayload(result);
  return normalizedText(pickString(
    payload?.diffString,
    payload?.diff_string,
    payload?.diff,
    payload?.patch,
    input.diff,
    input.diff_string,
  ));
}

function finalSummary(result: UnknownRecord | null, input: UnknownRecord): string {
  const payload = resultPayload(result);
  return pickString(
    payload?.error,
    payload?.reason,
    payload?.message,
    input.summary,
    input.error,
    input.reason,
  ) ?? "";
}

export function projectToolResultCard(
  value: unknown,
  hints: { kind?: ToolResultCardKind; toolCallId?: string } = {},
): ToolResultCardSnapshot | null {
  const input = objectValue(value);
  if (input === null) return null;

  const args = objectValue(input.args);
  const result = objectValue(input.result) ?? input;
  const kind = hints.kind ?? inferKind(input, args, result);
  if (kind === null) return null;

  const payload = resultPayload(result);
  const toolCallId = pickString(
    hints.toolCallId,
    input.toolCallId,
    input.tool_call_id,
    args?.toolCallId,
    args?.tool_call_id,
  );
  const command = kind === "shell"
    ? pickString(input.command, args?.command, payload?.command)
    : null;
  const path = kind === "shell"
    ? null
    : pickString(input.path, args?.path, payload?.path);
  const workingDirectory = kind === "shell"
    ? pickString(
        input.workingDirectory,
        input.working_directory,
        args?.workingDirectory,
        args?.working_directory,
        payload?.workingDirectory,
        payload?.working_directory,
      )
    : null;

  const isBackground =
    input.isBackground === true ||
    input.is_background === true ||
    args?.isBackground === true ||
    args?.is_background === true ||
    result.isBackground === true ||
    result.is_background === true ||
    result.backgrounded !== undefined;

  const stream = objectValue(input.delta) ?? objectValue(input.stream) ?? input;
  const streamOutput = kind === "shell"
    ? normalizedText(pickString(
        stream.stdoutDelta,
        stream.stdout_delta,
        stream.stderrDelta,
        stream.stderr_delta,
        objectValue(stream.stdout)?.content,
        objectValue(stream.stderr)?.content,
      ))
    : normalizedText(pickString(input.streamContentDelta, input.stream_content_delta));

  const mapped = statusName(input.status) ?? resultStatus(kind, result);
  const noTerminalPayload = result === input && resultPayload(result) === null;
  let status: ToolResultCardStatus;
  if (mapped !== null) status = mapped === "success" && isBackground ? "background" : mapped;
  else status = noTerminalPayload || streamOutput.length > 0 ? "running" : "error";

  return {
    kind,
    toolCallId,
    status,
    path,
    command,
    workingDirectory,
    summary: finalSummary(result, input),
    output: streamOutput.length > 0 ? streamOutput : finalOutput(kind, result, input),
    diff: finalDiff(result, input),
    isStreaming:
      input.isStreaming === true ||
      input.is_streaming === true ||
      streamOutput.length > 0,
    isBackground,
  };
}

export function mergeToolResultCard(
  previous: ToolResultCardSnapshot,
  update: unknown,
): ToolResultCardSnapshot {
  const next = projectToolResultCard(update, {
    kind: previous.kind,
    ...(previous.toolCallId === null ? {} : { toolCallId: previous.toolCallId }),
  });
  if (next === null) return previous;

  const raw = objectValue(update);
  const deltaShape =
    raw?.delta !== undefined ||
    raw?.stream !== undefined ||
    raw?.stdout_delta !== undefined ||
    raw?.stderr_delta !== undefined ||
    raw?.stream_content_delta !== undefined;
  const append = next.isStreaming && next.output.length > 0 && deltaShape;

  return {
    ...next,
    output: append ? previous.output + next.output : next.output || previous.output,
    diff: next.diff || previous.diff,
    toolCallId: next.toolCallId ?? previous.toolCallId,
  };
}
