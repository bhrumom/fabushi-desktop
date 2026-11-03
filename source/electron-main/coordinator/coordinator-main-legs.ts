import { COORDINATOR_MAIN_METHOD_TABLE } from "../../shared/rpc/coordinator-main.js";
import { createCoordinatorMainPortClient } from "./coordinator-main-port-client.js";

interface MessagePortLike { postMessage(frame: unknown): void; close(): void; on(event: "message" | "close", listener: (event: any) => void): void; start(): void }
type CoordinatorMainPortClient = ReturnType<typeof createCoordinatorMainPortClient>;

export interface CapturedCoordinatorMainSession {
  readonly legs: CoordinatorMainPortClient["legs"];
  assertCurrent(): void;
}

export interface CoordinatorMainPortOwnership {
  assertCurrent(): void;
  revoke(): void;
}

export function createCoordinatorMainLegs(options: { readonly onProblem: (problem: string) => void; readonly reportFailure?: (leg: string, error: unknown) => void }) {
  let current: CoordinatorMainPortClient | null = null;
  let disposed = false;
  let adoptionAttempt: object | null = null;
  const methods: Record<string, (first?: unknown) => Promise<unknown>> = {};
  for (const name of Object.keys(COORDINATOR_MAIN_METHOD_TABLE)) methods[name] = (first) => current == null ? Promise.reject(new Error(`Sand coordinator main-data leg "${name}" has no live coordinator session (coordinator not launched).`)) : current.legs[name]!(first);

  const reportFailure = (leg: string, error: unknown) => {
    try { options.reportFailure?.(leg, error); } catch { /* reporting cannot prevent owned-port cleanup */ }
  };
  const assertCurrentClient = (client: CoordinatorMainPortClient): void => {
    if (disposed || current !== client) throw new Error("coordinator main data session is no longer current");
    client.assertLive();
  };
  const revokeCurrent = () => {
    adoptionAttempt = null;
    const previous = current;
    current = null;
    previous?.dispose();
  };
  return {
    legs: methods,
    captureCurrent(assertLifetime: () => void): CapturedCoordinatorMainSession | null {
      const client = current;
      if (client == null) return null;
      const assertCurrent = (): void => {
        assertCurrentClient(client);
        try { assertLifetime(); } catch { throw new Error("coordinator main data session lifetime is no longer current"); }
        // The supplied assertion is opaque and may itself call a collaborator.
        assertCurrentClient(client);
      };
      try { assertCurrent(); } catch { return null; }
      return { legs: client.bindLifetime(assertCurrent), assertCurrent };
    },
    adoptPort(port: MessagePortLike): CoordinatorMainPortOwnership | null {
      const closePort = () => {
        try { port.close(); } catch (error) { reportFailure("close", error); }
      };
      if (disposed) {
        closePort();
        return null;
      }
      const attempt = {};
      adoptionAttempt = attempt;
      const previous = current;
      let client: CoordinatorMainPortClient;
      try {
        client = createCoordinatorMainPortClient({
          post: (frame) => {
            try {
              port.postMessage(frame);
            } catch (error) {
              reportFailure("leg-post", error);
              throw error;
            }
          },
          close: closePort,
        });
      } catch {
        if (adoptionAttempt === attempt) adoptionAttempt = null;
        closePort();
        return null;
      }

      const discardCandidate = (): null => {
        if (adoptionAttempt === attempt) adoptionAttempt = null;
        if (current === client) current = null;
        client.dispose();
        return null;
      };
      const ownsLiveCandidate = (): boolean => {
        if (disposed || adoptionAttempt !== attempt) return false;
        try { client.assertLive(); return true; } catch { return false; }
      };
      void client.settled.then((settlement) => {
        if (current === client) current = null;
        if (adoptionAttempt === attempt) adoptionAttempt = null;
        if (settlement.outcome === "protocol-breach") options.onProblem(`main-data session breach: ${settlement.detail}`);
      });
      if (!ownsLiveCandidate()) return discardCandidate();

      // Keep a valid previous client until the candidate is fully set up. Every
      // native callback can revoke this attempt or install a newer candidate.
      try {
        port.on("message", (event) => client.handleMessage(event.data));
        if (!ownsLiveCandidate()) return discardCandidate();
        port.on("close", () => client.handlePortClosed());
        if (!ownsLiveCandidate()) return discardCandidate();
        port.start();
      } catch (error) {
        discardCandidate();
        reportFailure("leg-adopt", error);
        return null;
      }
      if (!ownsLiveCandidate()) return discardCandidate();

      current = client;
      previous?.dispose();
      if (!ownsLiveCandidate() || current !== client) return discardCandidate();

      let revoked = false;
      return {
        assertCurrent: () => assertCurrentClient(client),
        revoke() {
          if (revoked) return;
          revoked = true;
          if (adoptionAttempt === attempt) adoptionAttempt = null;
          if (current === client) current = null;
          client.dispose();
        },
      };
    },
    revoke() { if (!disposed) revokeCurrent(); },
    dispose() { if (disposed) return; disposed = true; revokeCurrent(); },
  };
}
