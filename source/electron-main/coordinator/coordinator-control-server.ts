import { randomUUID } from "node:crypto";

import {
  COORDINATOR_PROTOCOL_VERSION,
  parseCoordinatorFrame,
  type CoordinatorFrame,
  type CoordinatorControlExecutor,
  type CoordinatorControlSourceContext,
} from "../../shared/rpc/coordinator-port.js";
import { McpOriginRetiredError } from "../../shared/node/mcp/mcp-result-factory.js";

export const COORDINATOR_CONTROL_EVENT_FAMILIES = [
  "transport-connected",
  "transport-down",
  "agents-event",
  "agents-roster-seed",
] as const;

function createDeferred<T>(): {
  readonly promise: Promise<T>;
  readonly resolve: (value: T | PromiseLike<T>) => void;
} {
  let resolve!: (value: T | PromiseLike<T>) => void;
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

export function createCoordinatorControlServer(options: {
  readonly post: (frame: CoordinatorFrame) => void;
  readonly executors: Record<string, CoordinatorControlExecutor | undefined>;
  readonly onEvent: Record<string, (payload: unknown) => void>;
  readonly onProblem: (detail: string) => void;
  readonly reportFailure?: (leg: string, error: unknown) => void;
}) {
  let helloObserved = false;
  let settlement: any = null;
  const { promise: settled, resolve } = createDeferred<any>();
  const source: CoordinatorControlSourceContext = {
    sourceId: randomUUID(),
    assertCurrent: () => {
      if (settlement != null) throw new McpOriginRetiredError();
      return undefined;
    },
  };

  const reportFailure = (leg: string, error: unknown) => {
    try {
      options.reportFailure?.(leg, error);
    } catch {
      // Diagnostics must not escape the protocol settlement owner.
    }
  };

  const reportProblem = (detail: string) => {
    try {
      options.onProblem(detail);
    } catch (error) {
      reportFailure("problem-handler", error);
    }
  };

  const finish = (value: any) => {
    if (settlement != null) return;
    settlement = value;
    resolve(value);
  };

  const breach = (detail: string) => {
    if (settlement != null) return;
    // Retire before external diagnostics or posting can reenter this server.
    finish({ outcome: "protocol-breach", detail });
    reportProblem(detail);
    try {
      options.post({
        kind: "lifecycle",
        phase: "shutdown",
        reason: "protocol-error",
        detail,
      });
    } catch (error) {
      reportFailure("breach-post", error);
    }
  };

  const dispatch = (requestId: string, method: string, args: unknown) => {
    const executor = options.executors[method];
    void (async () => {
      if (executor == null) {
        return {
          status: "failed" as const,
          failure: {
            code: "main-unknown-command",
            message: `no control command named ${method}`,
          },
        };
      }
      try {
        return { status: "ok" as const, value: (await executor(args, source)) ?? null };
      } catch (error) {
        return {
          status: "failed" as const,
          failure: {
            code: "main-execution-failure",
            message: error instanceof Error ? error.message : String(error),
          },
        };
      }
    })().then((outcome) => {
      if (settlement == null) {
        try {
          options.post({ kind: "reply", requestId, outcome });
        } catch (error) {
          reportFailure("reply-post", error);
          breach(`failed to post control reply for ${requestId}`);
        }
      }
    });
  };

  const handleFrame = (frame: CoordinatorFrame) => {
    if (frame.kind === "lifecycle" && frame.phase === "hello") {
      if (helloObserved) {
        return breach("coordinator repeated hello on the control port");
      }
      helloObserved = true;
      try {
        options.post({
          kind: "lifecycle",
          phase: "ready",
          protocolVersion: COORDINATOR_PROTOCOL_VERSION,
        });
      } catch (error) {
        reportFailure("ready-post", error);
        return breach("failed to post control ready frame");
      }
      return;
    }
    if (frame.kind === "lifecycle" && frame.phase === "shutdown") {
      return finish({ outcome: "clean", reason: "shutdown" });
    }
    if (frame.kind === "lifecycle") {
      return breach("coordinator posted a main-direction ready frame");
    }
    if (frame.kind === "reply") {
      return breach("coordinator posted a main-direction reply frame");
    }
    if (!helloObserved) {
      return breach(`coordinator posted a ${frame.kind} frame before hello`);
    }
    if (frame.kind === "request") {
      return dispatch(frame.requestId, frame.method, frame.args);
    }
    if (frame.kind === "event") {
      if (
        !(COORDINATOR_CONTROL_EVENT_FAMILIES as readonly string[]).includes(
          frame.family,
        )
      ) {
        return breach(
          `coordinator emitted an undeclared control event family: ${frame.family}`,
        );
      }
      const onEvent = options.onEvent[frame.family];
      if (onEvent == null) {
        return breach(`control event family has no registered consumer: ${frame.family}`);
      }
      try {
        onEvent(frame.payload);
      } catch (error) {
        reportFailure("event-consumer", error);
        return breach(`control event consumer failed: ${frame.family}`);
      }
    }
  };

  return {
    handleMessage(value: unknown) {
      if (settlement != null) return;
      const intake = parseCoordinatorFrame(value);
      if (!intake.accepted) {
        return breach(
          `coordinator posted a malformed control frame: ${intake.rejection.detail}`,
        );
      }
      handleFrame(intake.frame);
    },
    handlePortClosed: () => finish({ outcome: "clean", reason: "port-closed" }),
    dispose() {
      if (settlement != null) return;
      finish({ outcome: "clean", reason: "disposed" });
      try {
        options.post({
          kind: "lifecycle",
          phase: "shutdown",
          reason: "requested",
          detail: null,
        });
      } catch (error) {
        reportFailure("dispose-post", error);
      }
    },
    settled,
  };
}
