import { COORDINATOR_PROTOCOL_VERSION, parseCoordinatorFrame, type CoordinatorFrame, type CoordinatorReplyOutcome } from "../../shared/rpc/coordinator-port.js";
import { COORDINATOR_MAIN_METHOD_TABLE } from "../../shared/rpc/coordinator-main.js";

export class CoordinatorMainPortCallError extends Error { constructor(readonly code: string, message: string) { super(`${code}: ${message}`); this.name = "CoordinatorMainPortCallError"; } }

export type CallLifetimeGuard = () => void;

interface PendingMainPortCall {
  readonly method: string;
  readonly assertLifetime?: CallLifetimeGuard;
  resolve(value: unknown): void;
  reject(error: unknown): void;
}

interface MainPortCall extends PendingMainPortCall {
  readonly args: unknown;
}

export function createCoordinatorMainPortClient(endpoint: { post(frame: CoordinatorFrame): void; close(): void }) {
  let phase: "awaiting-ready" | "serving" | "settled" = "awaiting-ready";
  let nextRequestId = 0;
  const pending = new Map<string, PendingMainPortCall>();
  const queued: MainPortCall[] = [];
  const { promise: settled, resolve: resolveSettled } = Promise.withResolvers<any>();

  const assertLive = (): void => {
    if (phase === "settled") throw new Error("coordinator main data port is no longer live");
  };
  const settle = (settlement: any, shutdown?: CoordinatorFrame) => {
    if (phase === "settled") return;
    // Publish the terminal phase before calling an endpoint that can reenter.
    phase = "settled";
    for (const call of queued.splice(0)) call.reject(new Error(`coordinator main data port settled (${settlement.outcome}) before the ${call.method} request was posted`));
    for (const call of pending.values()) call.reject(new Error(`coordinator main data port settled (${settlement.outcome}) before the ${call.method} reply`));
    pending.clear();
    if (shutdown != null) {
      try { endpoint.post(shutdown); } catch { /* local settlement must not depend on a live endpoint */ }
    }
    try { endpoint.close(); } catch { /* local settlement must not depend on endpoint cleanup */ }
    resolveSettled(settlement);
  };
  const breach = (detail: string) => settle(
    { outcome: "protocol-breach", detail },
    { kind: "lifecycle", phase: "shutdown", reason: "protocol-error", detail },
  );
  const permitsCall = (call: PendingMainPortCall): boolean => {
    try {
      call.assertLifetime?.();
      return true;
    } catch {
      // A retired caller does not own the shared port or other callers' cleanup.
      call.reject(new Error("coordinator main data call lifetime is no longer current"));
      return false;
    }
  };
  const postCall = (call: MainPortCall) => {
    if (!permitsCall(call)) return;
    if (phase !== "serving") {
      call.reject(new Error(`coordinator main data port is no longer ready for the ${call.method} request`));
      return;
    }
    const requestId = `m-${++nextRequestId}`;
    pending.set(requestId, { method: call.method, assertLifetime: call.assertLifetime, resolve: call.resolve, reject: call.reject });
    try {
      endpoint.post({ kind: "request", requestId, method: call.method, args: call.args });
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      settle({ outcome: "post-failed", detail });
    }
  };
  const call = (method: string, args: unknown, assertLifetime?: CallLifetimeGuard) => new Promise<unknown>((resolve, reject) => {
    const request: MainPortCall = { method, args, assertLifetime, resolve, reject };
    if (!permitsCall(request)) return;
    if (phase === "settled") {
      reject(new Error(`coordinator main data port settled before ${method} was posted`));
      return;
    }
    if (phase === "serving") postCall(request);
    else queued.push(request);
  });
  const settleReply = (requestId: string, outcome: CoordinatorReplyOutcome) => {
    const waiting = pending.get(requestId);
    if (waiting == null) return;
    pending.delete(requestId);
    if (!permitsCall(waiting)) return;
    if (phase === "settled") {
      waiting.reject(new Error(`coordinator main data port settled before the ${waiting.method} reply`));
      return;
    }
    if (outcome.status === "ok") waiting.resolve(outcome.value);
    else waiting.reject(new CoordinatorMainPortCallError(outcome.failure.code, outcome.failure.message));
  };
  const handleFrame = (frame: CoordinatorFrame) => {
    if (frame.kind === "lifecycle" && frame.phase === "shutdown") {
      settle(frame.reason === "requested" ? { outcome: "coordinator-shutdown" } : { outcome: "protocol-breach", detail: frame.detail ?? "peer reported a breach" });
      return;
    }
    if (frame.kind === "lifecycle" && frame.phase === "ready") {
      if (phase !== "awaiting-ready") return breach("ready repeated on a live main-data session");
      if (frame.protocolVersion !== COORDINATOR_PROTOCOL_VERSION) return breach(`ready.protocolVersion ${frame.protocolVersion} is not the supported ${COORDINATOR_PROTOCOL_VERSION}`);
      phase = "serving";
      for (const request of queued.splice(0)) postCall(request);
      return;
    }
    if (frame.kind === "reply") {
      settleReply(frame.requestId, frame.outcome);
      return;
    }
    if (frame.kind === "event") return;
    breach(`coordinator posted a client-direction ${frame.kind} frame`);
  };
  const createLegs = (assertLifetime?: CallLifetimeGuard) => {
    const legs: Record<string, (first?: unknown) => Promise<unknown>> = {};
    for (const [name, wire] of Object.entries(COORDINATOR_MAIN_METHOD_TABLE)) {
      legs[name] = wire.args === "none" ? () => call(name, {}, assertLifetime) : (first) => call(name, first, assertLifetime);
    }
    return legs;
  };

  endpoint.post({ kind: "lifecycle", phase: "hello", protocolVersion: COORDINATOR_PROTOCOL_VERSION });
  return {
    legs: createLegs(),
    bindLifetime: (assertLifetime: CallLifetimeGuard) => createLegs(assertLifetime),
    assertLive,
    handleMessage(value: unknown) {
      if (phase === "settled") return;
      const intake = parseCoordinatorFrame(value);
      if (!intake.accepted) return breach(intake.rejection.detail);
      handleFrame(intake.frame);
    },
    handlePortClosed: () => settle({ outcome: "port-closed" }),
    dispose() {
      settle(
        { outcome: "shutdown-requested" },
        { kind: "lifecycle", phase: "shutdown", reason: "requested", detail: null },
      );
    },
    settled,
  };
}
