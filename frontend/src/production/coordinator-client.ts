import type { CoordinatorPortBridge, TransferredCoordinatorPort } from "../recovered/contracts/desktop-bridge";
import {
  parseCoordinatorHumanTranscriptPageRequest,
  parseCoordinatorHumanRetryRequest,
  validateCoordinatorReply,
  parseCloudWorkerDoctorRequest,
  type CloudWorkerCapabilitiesProjection,
  type CloudWorkerDoctorProjection,
  type CloudWorkerDoctorRequest,
  type CloudWorkerStatusProjection,
  type CoordinatorAgentThreadRequest,
  type CoordinatorAgentThreadResponse,
  type CoordinatorTranscriptWindowRequest,
  type CoordinatorTranscriptWindowResponse,
  type TelegramAuthorizationProjection
} from "../../../source/shared/rpc/coordinator";

export const COORDINATOR_PROTOCOL_VERSION = 1;
export const COORDINATOR_TRANSPORT_STATE_FAMILY = "coordinator-transport-state";

type EventListener = (payload: unknown) => void;
type TransportState = "connected" | "down";
type TransportListener = (state: TransportState) => void;

interface RuntimeTransportObservation {
  readonly generation: number;
  readonly sequence: number;
  readonly action: "starting" | "terminal" | "connected" | "down";
}

interface PendingCall {
  method: string;
  resolve(value: unknown): void;
  reject(error: unknown): void;
}

export class CoordinatorCallError extends Error {
  constructor(readonly code: string, message: string, readonly transportKind?: string) {
    super(`${code}: ${message}`);
    this.name = "CoordinatorCallError";
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function runtimeTransportObservation(value: unknown): RuntimeTransportObservation | null {
  if (!isRecord(value)) return null;
  let action: RuntimeTransportObservation["action"];
  if (value.type === "host.lifecycle") {
    if (value.lifecycle === "running") action = "starting";
    else if (value.lifecycle === "stopped" || value.lifecycle === "circuit-open" || value.lifecycle === "spawn-failed") action = "terminal";
    else return null;
  } else if (value.type === "gateway.lifecycle") {
    if (value.lifecycle === "transport-connected") action = "connected";
    else if (value.lifecycle === "transport-down" || value.lifecycle === "discovery-timeout" || value.lifecycle === "rediscovery-failed") action = "down";
    else return null;
  } else return null;
  const { generation, sequence } = value;
  if (typeof generation !== "number" || !Number.isSafeInteger(generation) || generation <= 0
    || typeof sequence !== "number" || !Number.isSafeInteger(sequence) || sequence <= 0) return null;
  return { generation, sequence, action };
}

function validateReply(method: string, value: unknown): unknown {
  if (
    method.startsWith("feature.auth.telegram") ||
    [
      "getAgentTranscriptWindow",
      "getAgentThread",
      "getHumanConversationTranscriptTail",
      "retryHumanMessage",
      "cloudWorkers.status",
      "cloudWorkers.capabilities",
      "cloudWorkers.doctor",
    ].includes(method)
  ) {
    return validateCoordinatorReply(method, value);
  }
  if (["listAgents", "searchAgents", "getTrays", "listAllAutomations"].includes(method)) {
    if (!Array.isArray(value)) throw new Error(`${method} returned a malformed array reply`);
  }
  if (["openAgentTail", "getAgentTranscriptTail"].includes(method)) {
    if (!isRecord(value) || !Array.isArray(value.entries) || (value.nextBeforeSeq != null && typeof value.nextBeforeSeq !== "number")) {
      throw new Error(`${method} returned a malformed transcript page`);
    }
  }
  if (["getForeverBoxStatus", "ensureForeverBox"].includes(method) && value !== null) {
    if (!isRecord(value) || typeof value.agentId !== "string" || typeof value.state !== "string") {
      throw new Error(`${method} returned a malformed box status`);
    }
  }
  return value;
}

export interface CoordinatorCallAdmission {
  readonly isCurrent?: () => boolean;
  readonly isDispatchCurrent?: () => boolean;
  /** Marks possible Host admission before postMessage; a post failure is ambiguous. */
  readonly onRequestWillPost?: () => void;
}

export interface ProductionCoordinatorClient {
  readonly ready: Promise<void>;
  call<T = unknown>(method: string, args?: unknown, admission?: CoordinatorCallAdmission): Promise<T>;
  telegramAuthStatus(): Promise<TelegramAuthorizationProjection>;
  telegramSendCode(phoneNumber: string): Promise<TelegramAuthorizationProjection>;
  telegramSignIn(code: string): Promise<TelegramAuthorizationProjection>;
  telegramPassword(password: string): Promise<TelegramAuthorizationProjection>;
  telegramRegistration(firstName: string, lastName?: string): Promise<TelegramAuthorizationProjection>;
  telegramResendCode(reason?: string): Promise<TelegramAuthorizationProjection>;
  telegramCancelCode(): Promise<TelegramAuthorizationProjection>;
  telegramRequestPasswordRecovery(): Promise<TelegramAuthorizationProjection>;
  telegramCheckRecoveryPassword(code: string): Promise<TelegramAuthorizationProjection>;
  telegramSendVerifyEmailCode(email: string): Promise<TelegramAuthorizationProjection>;
  telegramVerifyEmail(code: string): Promise<TelegramAuthorizationProjection>;
  telegramRefreshPasswordChallenge(): Promise<TelegramAuthorizationProjection>;
  getAgentTranscriptWindow(args: CoordinatorTranscriptWindowRequest): Promise<CoordinatorTranscriptWindowResponse>;
  getAgentThread(args: CoordinatorAgentThreadRequest): Promise<CoordinatorAgentThreadResponse>;
  isEgressTunnelAvailable(): Promise<boolean>;
  cloudWorkersStatus(): Promise<CloudWorkerStatusProjection>;
  cloudWorkersCapabilities(): Promise<CloudWorkerCapabilitiesProjection>;
  cloudWorkersDoctor(args: CloudWorkerDoctorRequest): Promise<CloudWorkerDoctorProjection>;
  subscribe(family: string, listener: EventListener): () => void;
  subscribeTransport(listener: TransportListener): () => void;
  dispose(): void;
}

export type CoordinatorClient = ProductionCoordinatorClient;

let sharedCoordinatorClient: ProductionCoordinatorClient | null = null;

export function setSharedCoordinatorClient(client: ProductionCoordinatorClient | null): void {
  sharedCoordinatorClient = client;
}

export function getSharedCoordinatorClient(): ProductionCoordinatorClient | null {
  return sharedCoordinatorClient;
}

export function createCoordinatorClient(portBridge: CoordinatorPortBridge): ProductionCoordinatorClient | null {
  const eventListeners = new Map<string, Set<EventListener>>();
  const transportListeners = new Set<TransportListener>();
  const pending = new Map<string, PendingCall>();
  let port: TransferredCoordinatorPort | null = null;
  let nextRequestId = 0;
  let disposed = false;
  let serving = false;
  let currentTransport: TransportState | null = null;
  let runtimeGeneration: number | null = null;
  let runtimeSequence = 0;
  let runtimeTerminal = false;
  let runtimeState: TransportState | null = null;
  const publishTransport = (state: TransportState, newEpisode = false) => {
    if (currentTransport === state && !newEpisode) return;
    currentTransport = state;
    for (const listener of [...transportListeners]) listener(state);
  };
  const resetRuntimeEpoch = () => {
    runtimeGeneration = null;
    runtimeSequence = 0;
    runtimeTerminal = false;
    runtimeState = null;
  };
  const observeRuntimeTransport = (payload: unknown) => {
    // Informational/unknown frames cannot poison a meaningful observation's
    // generation or sequence, even when their counters are much larger.
    const observation = runtimeTransportObservation(payload);
    if (observation == null) return;
    const newerGeneration = runtimeGeneration == null || observation.generation > runtimeGeneration;
    if (runtimeGeneration != null && observation.generation < runtimeGeneration) return;
    if (!newerGeneration && observation.sequence <= runtimeSequence) return;
    if (!newerGeneration && runtimeTerminal
      && (observation.action === "connected" || observation.action === "starting")) return;
    if (newerGeneration) {
      resetRuntimeEpoch();
      runtimeGeneration = observation.generation;
    }
    runtimeSequence = observation.sequence;
    if (observation.action === "terminal") runtimeTerminal = true;
    const state = observation.action === "connected" ? "connected" : "down";
    const newEpisode = newerGeneration || runtimeState !== state;
    runtimeState = state;
    publishTransport(state, newEpisode);
  };
  let resolveReady = () => {};
  let rejectReady = (_reason: unknown) => {};
  const makeReady = () => new Promise<void>((resolve, reject) => {
    resolveReady = resolve;
    rejectReady = reject;
  });
  const ready = makeReady();
  let currentReady = ready;
  ready.catch(() => {});

  const rejectCalls = (reason: string) => {
    rejectReady(new Error(reason));
    for (const waiting of pending.values()) waiting.reject(new Error(`${waiting.method} failed: ${reason}`));
    pending.clear();
  };
  let claim: ReturnType<CoordinatorPortBridge["claim"]> = null;
  const disconnect = (expectedPort: TransferredCoordinatorPort, reason: string) => {
    if (disposed || port !== expectedPort) return;
    port = null;
    serving = false;
    rejectCalls(reason);
    publishTransport("down");
    currentReady = makeReady();
    currentReady.catch(() => {});
    claim?.request();
  };

  const handleMessage = (expectedPort: TransferredCoordinatorPort, value: unknown) => {
    if (port !== expectedPort || disposed) return;
    if (!isRecord(value) || typeof value.kind !== "string") {
      const reason = "coordinator posted a malformed frame";
      try {
        expectedPort.postMessage({ kind: "lifecycle", phase: "shutdown", reason: "protocol-error", detail: reason });
      } catch {
        return disconnect(expectedPort, `${reason}; protocol-error shutdown could not be sent`);
      }
      return disconnect(expectedPort, reason);
    }
    if (value.kind === "lifecycle" && value.phase === "ready") {
      if (value.protocolVersion !== COORDINATOR_PROTOCOL_VERSION) return disconnect(expectedPort, "coordinator protocol version mismatch");
      serving = true;
      resolveReady();
      // Protocol readiness remains the bootstrap fallback for runtime events
      // emitted before the Renderer handshake, not a replacement for newer state.
      if (runtimeGeneration == null) publishTransport("connected");
      return;
    }
    if (value.kind === "lifecycle" && value.phase === "shutdown") return disconnect(expectedPort, "coordinator requested shutdown");
    if (value.kind === "reply" && typeof value.requestId === "string" && isRecord(value.outcome)) {
      const waiting = pending.get(value.requestId);
      if (waiting == null) return;
      if (value.outcome.status === "ok") {
        pending.delete(value.requestId);
        try { waiting.resolve(validateReply(waiting.method, value.outcome.value)); }
        catch (error) { waiting.reject(error); }
      } else if (value.outcome.status === "failed") {
        pending.delete(value.requestId);
        if (!isRecord(value.outcome.failure)) {
          waiting.reject(new CoordinatorCallError(
            "protocol-error",
            "Coordinator returned a failed reply without a failure object",
          ));
          return;
        }
        waiting.reject(new CoordinatorCallError(
          typeof value.outcome.failure.code === "string" ? value.outcome.failure.code : "failed",
          typeof value.outcome.failure.message === "string" ? value.outcome.failure.message : "Coordinator request failed",
          typeof value.outcome.failure.transportKind === "string" ? value.outcome.failure.transportKind : undefined
        ));
      } else {
        pending.delete(value.requestId);
        waiting.reject(new CoordinatorCallError(
          "protocol-error",
          "Coordinator returned a reply with an invalid outcome status",
        ));
      }
      return;
    }
    if (value.kind === "reply") return disconnect(expectedPort, "coordinator posted a malformed reply frame");
    if (value.kind === "event" && typeof value.family === "string" && "payload" in value) {
      if (value.family === COORDINATOR_TRANSPORT_STATE_FAMILY && isRecord(value.payload) && (value.payload.state === "connected" || value.payload.state === "down")) {
        publishTransport(value.payload.state, true);
      } else if (value.family === "runtime") {
        observeRuntimeTransport(value.payload);
      }
      for (const listener of eventListeners.get(value.family) ?? []) listener(value.payload);
      return;
    }
    return disconnect(expectedPort, `coordinator posted an unsupported or malformed frame: ${value.kind}`);
  };

  claim = portBridge.claim({
    onPort(nextPort) {
      if (disposed) return nextPort.close();
      const replacesLivePort = port != null;
      if (replacesLivePort) rejectCalls("coordinator session replaced");
      const previousPort = port;
      port = nextPort;
      previousPort?.close();
      serving = false;
      resetRuntimeEpoch();
      if (replacesLivePort) {
        currentReady = makeReady();
        currentReady.catch(() => {});
        publishTransport("down");
      }
      nextPort.addEventListener("message", (event) => handleMessage(nextPort, event.data));
      nextPort.addEventListener("close", () => disconnect(nextPort, "coordinator port closed"));
      nextPort.start();
      try {
        nextPort.postMessage({ kind: "lifecycle", phase: "hello", protocolVersion: COORDINATOR_PROTOCOL_VERSION });
      } catch {
        try { nextPort.close(); } catch {
          // Disconnect below still owns local request/claim cleanup.
        }
        return disconnect(nextPort, "coordinator hello post failed");
      }
    }
  });
  if (claim == null) return null;
  claim.request();

  const call = async <T = unknown>(method: string, args: unknown = {}, admission?: CoordinatorCallAdmission): Promise<T> => {
    if (method === "getHumanConversationTranscriptTail" && parseCoordinatorHumanTranscriptPageRequest(args) == null) {
      throw new Error("getHumanConversationTranscriptTail requires a valid Human transcript cursor");
    }
    if (method === "retryHumanMessage" && parseCoordinatorHumanRetryRequest(args) == null) {
      throw new Error("retryHumanMessage requires only a valid conversationId and original clientNonce");
    }
    if (admission?.isCurrent?.() === false) throw new Error(`Coordinator request scope expired for ${method}`);
    if (admission?.isDispatchCurrent?.() === false) throw new CoordinatorCallError("request-retired", `${method} request is no longer current`);
    await currentReady;
    if (disposed || port == null || !serving) throw new Error(`coordinator is unavailable for ${method}`);
    // Reconnect can replace readiness after the renderer's initial ready has
    // resolved. Scope admission belongs immediately before the actual frame.
    if (admission?.isCurrent?.() === false) throw new Error(`Coordinator request scope expired for ${method}`);
    if (admission?.isDispatchCurrent?.() === false) throw new CoordinatorCallError("request-retired", `${method} request is no longer current`);
    const requestId = `r-${++nextRequestId}`;
    admission?.onRequestWillPost?.();
    const response = new Promise<unknown>((resolve, reject) => pending.set(requestId, { method, resolve, reject }));
    const expectedPort = port;
    try {
      expectedPort.postMessage({ kind: "request", requestId, method, args });
    } catch (error) {
      pending.delete(requestId);
      try { expectedPort.close(); } catch {
        // Disconnect below still owns local request/claim cleanup.
      }
      disconnect(expectedPort, "coordinator request post failed");
      throw error;
    }
    return (await response) as T;
  };

  return {
    ready,
    call,
    telegramAuthStatus: async () => await call("feature.auth.telegramStatus", {}) as TelegramAuthorizationProjection,
    telegramSendCode: async (phoneNumber) => await call("feature.auth.telegramSendCode", { phoneNumber }) as TelegramAuthorizationProjection,
    telegramSignIn: async (code) => await call("feature.auth.telegramSignIn", { code }) as TelegramAuthorizationProjection,
    telegramPassword: async (password) => await call("feature.auth.telegramPassword", { password }) as TelegramAuthorizationProjection,
    telegramRegistration: async (firstName, lastName) => await call("feature.auth.telegramRegistration", {
      firstName,
      ...(lastName == null ? {} : { lastName }),
    }) as TelegramAuthorizationProjection,
    telegramResendCode: async (reason) => await call("feature.auth.telegramResendCode", {
      ...(reason == null ? {} : { reason }),
    }) as TelegramAuthorizationProjection,
    telegramCancelCode: async () => await call("feature.auth.telegramCancelCode", {}) as TelegramAuthorizationProjection,
    telegramRequestPasswordRecovery: async () => await call("feature.auth.telegramRequestPasswordRecovery", {}) as TelegramAuthorizationProjection,
    telegramCheckRecoveryPassword: async (code) => await call("feature.auth.telegramCheckRecoveryPassword", { code }) as TelegramAuthorizationProjection,
    telegramSendVerifyEmailCode: async (email) => await call("feature.auth.telegramSendVerifyEmailCode", { email }) as TelegramAuthorizationProjection,
    telegramVerifyEmail: async (code) => await call("feature.auth.telegramVerifyEmail", { code }) as TelegramAuthorizationProjection,
    telegramRefreshPasswordChallenge: async () => await call("feature.auth.telegramRefreshPasswordChallenge", {}) as TelegramAuthorizationProjection,
    getAgentTranscriptWindow: async (args) => await call("getAgentTranscriptWindow", args) as CoordinatorTranscriptWindowResponse,
    getAgentThread: async (args) => await call("getAgentThread", args) as CoordinatorAgentThreadResponse,
    isEgressTunnelAvailable: async () => await call("isEgressTunnelAvailable") === true,
    cloudWorkersStatus: async () => await call("cloudWorkers.status") as CloudWorkerStatusProjection,
    cloudWorkersCapabilities: async () => await call("cloudWorkers.capabilities") as CloudWorkerCapabilitiesProjection,
    cloudWorkersDoctor: async (args) => {
      if (parseCloudWorkerDoctorRequest(args) == null) throw new Error("cloudWorkers.doctor request is malformed");
      return await call("cloudWorkers.doctor", args) as CloudWorkerDoctorProjection;
    },
    subscribe(family, listener) {
      const listeners = eventListeners.get(family) ?? new Set<EventListener>();
      listeners.add(listener);
      eventListeners.set(family, listeners);
      return () => listeners.delete(listener);
    },
    subscribeTransport(listener) {
      if (disposed) return () => {};
      transportListeners.add(listener);
      if (currentTransport != null) listener(currentTransport);
      return () => transportListeners.delete(listener);
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      const currentPort = port;
      try {
        currentPort?.postMessage({ kind: "lifecycle", phase: "shutdown", reason: "requested", detail: null });
      } catch {
        // Local disposal must still reclaim the claim and settle callers when
        // the endpoint is already unable to accept the shutdown frame.
      }
      claim.release();
      rejectCalls("coordinator source disposed");
      try {
        currentPort?.close();
      } catch {
        // Endpoint cleanup is best effort after local ownership is settled.
      }
      port = null;
      serving = false;
    }
  };
}

