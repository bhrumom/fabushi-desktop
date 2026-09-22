import type {
  ExpectedLocalExecProcessIdentity,
  LocalExecProcessIdentity,
} from "../../shared/local-exec-process-identity.js";

export interface CoordinatorGatewayControl {
  resolveConnection(): unknown | Promise<unknown>;
  mintLocalExecDaemonCredential?(): unknown | Promise<unknown>;
}

export interface CoordinatorMcpControl {
  listRoutedMcpTools(): unknown | Promise<unknown>;
  executeRoutedMcpTool(request: unknown): unknown | Promise<unknown>;
}

export interface CoordinatorWebAuthnPromptControl {
  requestConsent(args: unknown): unknown | Promise<unknown>;
  requestPin(args: unknown): unknown | Promise<unknown>;
  update(status: string): unknown | Promise<unknown>;
  finish(): unknown | Promise<unknown>;
}

export interface CoordinatorLocalExecControl {
  spawnLocalExecDaemon(args: {
    readonly logPath: string;
    readonly env: Readonly<Record<string, string>>;
  }): LocalExecProcessIdentity | Promise<LocalExecProcessIdentity>;
  terminateProcess(identity: LocalExecProcessIdentity): boolean | Promise<boolean>;
  isProcessAlive(pid: number): boolean | Promise<boolean>;
  getProcessIdentity(
    expected: ExpectedLocalExecProcessIdentity,
  ): LocalExecProcessIdentity | null | Promise<LocalExecProcessIdentity | null>;
  waitLocalExecDaemonExit?(
    identity: LocalExecProcessIdentity,
  ): unknown | Promise<unknown>;
}

export interface CoordinatorTelemetryControl {
  getRpcTraceWindowTraceparent?(): string | null;
  reportTransportStage?(report: unknown): void;
  reportGatewayCommandSpan?(report: unknown): void;
  reportGatewayReachability?(report: unknown): void;
  reportGatewayDnsDiagnostic?(report: unknown): void;
  reportProcessCrash?(report: unknown): void;
}

export interface CoordinatorControlDependencies {
  readonly gateway: CoordinatorGatewayControl;
  readonly mcp: CoordinatorMcpControl;
  readonly webauthnPrompt: CoordinatorWebAuthnPromptControl;
  readonly localExec: CoordinatorLocalExecControl;
  readonly telemetry?: CoordinatorTelemetryControl;
}

export const COORDINATOR_CONTROL_METHODS = Object.freeze([
  "resolveGatewayConnection",
  "listRoutedMcpTools",
  "executeRoutedMcpTool",
  "mintLocalExecDaemonCredential",
  "requestWebAuthnConsent",
  "requestWebAuthnPin",
  "updateWebAuthnConsent",
  "finishWebAuthnConsent",
  "spawnLocalExecDaemon",
  "terminateProcess",
  "isProcessAlive",
  "getProcessIdentity",
  "waitLocalExecDaemonExit",
  "getRpcTraceWindowTraceparent",
  "reportTransportStage",
  "reportGatewayCommandSpan",
  "reportGatewayReachability",
  "reportGatewayDnsDiagnostic",
  "reportProcessCrash",
] as const);

type CoordinatorControlMethod = typeof COORDINATOR_CONTROL_METHODS[number];
export type CoordinatorControlExecutor =
  (args: unknown) => unknown | Promise<unknown>;
export type CoordinatorControlExecutors =
  Readonly<Record<CoordinatorControlMethod, CoordinatorControlExecutor>>;

function requireRecord(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new TypeError(`${label} must be an object`);
  }
  return value as Record<string, unknown>;
}

function requireString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`${label} must be a non-empty string`);
  }
  return value;
}

function requirePositiveInteger(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value <= 0) {
    throw new TypeError(`${label} must be a positive integer`);
  }
  return value;
}

function parseEnvironment(value: unknown): Readonly<Record<string, string>> {
  const record = requireRecord(value, "spawnLocalExecDaemon.env");
  for (const [key, entry] of Object.entries(record)) {
    if (key.length === 0 || typeof entry !== "string") {
      throw new TypeError(
        "spawnLocalExecDaemon.env must contain only string entries",
      );
    }
  }
  return record as Record<string, string>;
}

function parseIdentity(value: unknown, label: string): LocalExecProcessIdentity {
  const record = requireRecord(value, label);
  return {
    pid: requirePositiveInteger(record.pid, `${label}.pid`),
    startEpochMs: requirePositiveInteger(
      record.startEpochMs,
      `${label}.startEpochMs`,
    ),
    command: requireString(record.command, `${label}.command`),
    entryRealpath: requireString(
      record.entryRealpath,
      `${label}.entryRealpath`,
    ),
    generationToken: requireString(
      record.generationToken,
      `${label}.generationToken`,
    ),
  };
}

function parseExpectedIdentity(value: unknown): ExpectedLocalExecProcessIdentity {
  const record = requireRecord(value, "getProcessIdentity");
  const startEpochMs = record.startEpochMs === undefined
    ? undefined
    : requirePositiveInteger(
        record.startEpochMs,
        "getProcessIdentity.startEpochMs",
      );
  const command = record.command === undefined
    ? undefined
    : requireString(record.command, "getProcessIdentity.command");
  const discoveryStartedAt = record.discoveryStartedAt === undefined
    ? undefined
    : requirePositiveInteger(
        record.discoveryStartedAt,
        "getProcessIdentity.discoveryStartedAt",
      );
  return {
    pid: requirePositiveInteger(record.pid, "getProcessIdentity.pid"),
    entryRealpath: requireString(
      record.entryRealpath,
      "getProcessIdentity.entryRealpath",
    ),
    generationToken: requireString(
      record.generationToken,
      "getProcessIdentity.generationToken",
    ),
    ...(startEpochMs === undefined ? {} : { startEpochMs }),
    ...(command === undefined ? {} : { command }),
    ...(discoveryStartedAt === undefined ? {} : { discoveryStartedAt }),
  };
}

/**
 * Frozen Grok Coordinator control-plane surface. Business execution stays in
 * Mahayana Host/Runner. Electron main owns only platform/native operations,
 * user-facing consent, and observability adapters.
 */
export function createCoordinatorControlExecutors(
  dependencies: CoordinatorControlDependencies,
): CoordinatorControlExecutors {
  const telemetry = dependencies.telemetry;
  return {
    resolveGatewayConnection: () => dependencies.gateway.resolveConnection(),
    listRoutedMcpTools: () => dependencies.mcp.listRoutedMcpTools(),
    executeRoutedMcpTool: (request) =>
      dependencies.mcp.executeRoutedMcpTool(request),
    mintLocalExecDaemonCredential: async () =>
      (await dependencies.gateway.mintLocalExecDaemonCredential?.()) ?? null,
    requestWebAuthnConsent: (args) =>
      dependencies.webauthnPrompt.requestConsent(args),
    requestWebAuthnPin: (args) =>
      dependencies.webauthnPrompt.requestPin(args),
    updateWebAuthnConsent: (args) => {
      const record = requireRecord(args, "updateWebAuthnConsent");
      return dependencies.webauthnPrompt.update(
        requireString(record.status, "updateWebAuthnConsent.status"),
      );
    },
    finishWebAuthnConsent: () => dependencies.webauthnPrompt.finish(),
    spawnLocalExecDaemon: (args) => {
      const record = requireRecord(args, "spawnLocalExecDaemon");
      return dependencies.localExec.spawnLocalExecDaemon({
        logPath: requireString(
          record.logPath,
          "spawnLocalExecDaemon.logPath",
        ),
        env: parseEnvironment(record.env),
      });
    },
    terminateProcess: async (args) => {
      const record = requireRecord(args, "terminateProcess");
      const identity = parseIdentity(
        record.identity,
        "terminateProcess.identity",
      );
      return {
        terminated: await dependencies.localExec.terminateProcess(identity),
      };
    },
    isProcessAlive: (args) => {
      const record = requireRecord(args, "isProcessAlive");
      return dependencies.localExec.isProcessAlive(
        requirePositiveInteger(record.pid, "isProcessAlive.pid"),
      );
    },
    getProcessIdentity: (args) =>
      dependencies.localExec.getProcessIdentity(parseExpectedIdentity(args)),
    waitLocalExecDaemonExit: (args) => {
      if (dependencies.localExec.waitLocalExecDaemonExit === undefined) {
        throw new Error("local-exec daemon exit tracking is unavailable");
      }
      return dependencies.localExec.waitLocalExecDaemonExit(
        parseIdentity(args, "waitLocalExecDaemonExit"),
      );
    },
    getRpcTraceWindowTraceparent: () =>
      telemetry?.getRpcTraceWindowTraceparent?.() ?? null,
    reportTransportStage: (report) =>
      telemetry?.reportTransportStage?.(report),
    reportGatewayCommandSpan: (report) =>
      telemetry?.reportGatewayCommandSpan?.(report),
    reportGatewayReachability: (report) =>
      telemetry?.reportGatewayReachability?.(report),
    reportGatewayDnsDiagnostic: (report) =>
      telemetry?.reportGatewayDnsDiagnostic?.(report),
    reportProcessCrash: (report) =>
      telemetry?.reportProcessCrash?.(report),
  };
}
