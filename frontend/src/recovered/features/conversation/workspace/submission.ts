/**
 * The shipped send journal owns optimistic send, offline queueing, reconnect
 * flush, and stale-generation fencing around the coordinator send boundary.
 * @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5674804
 */

export type ComposerSubmissionPhase = "pending" | "queued" | "failed" | "sent" | "cancelled";

/** Explicit retry belongs to its transcript row, not the current composer. */
export function composerSubmissionBusyState(input: { phase: ComposerSubmissionPhase; isRetry?: boolean }): boolean | undefined {
  if (input.isRetry) return undefined;
  return input.phase === "pending";
}

export interface ComposerSubmission {
  nonce: string;
  agentId: string;
  prompt: string;
  /** Serialized Tiptap JSON; absent for an empty/plain compatibility draft. */
  richText?: string;
  attachments: readonly { path: string; name: string }[];
  createdAtMs: number;
  replyToId?: string;
  isFork?: boolean;
  /** Existing queue-owned Human input; never reconstructed from a projection. */
  humanDispatch?: { requestPosted: boolean; committedAttachments: { path: string; name: string }[] };
}

export interface ComposerSubmissionAdmission {
  readonly isCurrent?: () => boolean;
}

export interface ComposerSubmissionRecord extends ComposerSubmission {
  phase: Exclude<ComposerSubmissionPhase, "cancelled">;
  readonly isRetry?: boolean;
  readonly isCurrentAttempt?: () => boolean;
}

export interface ComposerSubmissionQueue {
  submit(input: ComposerSubmission): { nonce: string; completion: Promise<ComposerSubmissionPhase> };
  retryFailed(nonce: string, admission?: ComposerSubmissionAdmission): { nonce: string; completion: Promise<ComposerSubmissionPhase> } | null;
  discard(nonce: string): void;
  cancelQueued(nonce: string): boolean;
  flush(): void;
  snapshot(): readonly ComposerSubmissionRecord[];
  /** Abandons a prior account's renderer records; it does not cancel Host work. */
  reset(): void;
  dispose(): void;
}

interface QueueOptions {
  isTransportDown(): boolean;
  send(input: ComposerSubmission, admission?: ComposerSubmissionAdmission): Promise<void>;
  now?(): number;
  onPhase?(input: ComposerSubmission & { phase: ComposerSubmissionPhase; isRetry?: boolean; isCurrentAttempt?: () => boolean }): void;
  onFailure?(input: ComposerSubmissionRecord, error: unknown): void;
}

interface PendingRecord {
  input: ComposerSubmission;
  phase: Exclude<ComposerSubmissionPhase, "cancelled">;
  resolve(value: ComposerSubmissionPhase): void;
  completion: Promise<ComposerSubmissionPhase>;
  admission?: ComposerSubmissionAdmission;
  isRetry?: boolean;
}

/**
 * Small renderer-side boundary around the coordinator's sendPrompt contract.
 * It intentionally does not invent a stop-generation method: only queued
 * sends can be cancelled because that is the action exposed by the shipped
 * send journal and the current coordinator contract.
 */
export function createComposerSubmissionQueue(options: QueueOptions): ComposerSubmissionQueue {
  const records = new Map<string, PendingRecord>();
  const activeByAgent = new Map<string, string>();
  let generation = 0;
  let disposed = false;

  const notify = (record: PendingRecord, phase: PendingRecord["phase"]) => {
    record.phase = phase;
    if (record.admission?.isCurrent?.() !== false) options.onPhase?.({ ...record.input, phase,
      ...(record.isRetry ? { isRetry: true } : {}),
      ...(record.admission?.isCurrent == null ? {} : { isCurrentAttempt: record.admission.isCurrent }) });
  };

  const finish = (record: PendingRecord, phase: ComposerSubmissionPhase) => {
    if (records.get(record.input.nonce) !== record) return;
    if (phase === "sent" || phase === "cancelled") records.delete(record.input.nonce);
    if (activeByAgent.get(record.input.agentId) === record.input.nonce) activeByAgent.delete(record.input.agentId);
    record.resolve(phase);
    if (phase !== "cancelled" && record.admission?.isCurrent?.() !== false) options.onPhase?.({ ...record.input, phase,
      ...(record.isRetry ? { isRetry: true } : {}),
      ...(record.admission?.isCurrent == null ? {} : { isCurrentAttempt: record.admission.isCurrent }) });
  };

  const run = (record: PendingRecord, runGeneration: number) => {
    if (disposed || generation !== runGeneration || records.get(record.input.nonce) !== record) return;
    if (record.admission?.isCurrent?.() === false) {
      record.phase = "failed";
      record.resolve("cancelled");
      return;
    }
    if (options.isTransportDown()) {
      notify(record, "queued");
      return;
    }
    activeByAgent.set(record.input.agentId, record.input.nonce);
    notify(record, "pending");
    void options.send(record.input, record.admission).then(() => {
      if (disposed || generation !== runGeneration || records.get(record.input.nonce) !== record) return;
      finish(record, "sent");
      flushAgent(record.input.agentId, runGeneration);
    }).catch((error: unknown) => {
      if (disposed || generation !== runGeneration || records.get(record.input.nonce) !== record) return;
      notify(record, "failed");
      if (activeByAgent.get(record.input.agentId) === record.input.nonce) activeByAgent.delete(record.input.agentId);
      record.resolve("failed");
      if (record.admission?.isCurrent?.() !== false) options.onFailure?.({ ...record.input, phase: "failed",
        ...(record.isRetry ? { isRetry: true } : {}),
        ...(record.admission?.isCurrent == null ? {} : { isCurrentAttempt: record.admission.isCurrent }) }, error);
    });
  };

  const flushAgent = (agentId: string, runGeneration: number) => {
    if (disposed || generation !== runGeneration || options.isTransportDown() || activeByAgent.has(agentId)) return;
    const next = [...records.values()].find((record) => record.input.agentId === agentId && record.phase === "queued");
    if (next != null) run(next, runGeneration);
  };

  const flush = () => {
    if (disposed || options.isTransportDown()) return;
    const runGeneration = generation;
    for (const record of records.values()) {
      if (record.phase === "queued" && !activeByAgent.has(record.input.agentId)) flushAgent(record.input.agentId, runGeneration);
    }
  };

  const reset = () => {
    generation += 1;
    activeByAgent.clear();
    for (const record of records.values()) record.resolve("cancelled");
    records.clear();
  };

  const submit = (input: ComposerSubmission, admission?: ComposerSubmissionAdmission, isRetry = false):
    { nonce: string; completion: Promise<ComposerSubmissionPhase> } => {
      if (disposed) return { nonce: input.nonce, completion: Promise.resolve("cancelled") };
      let resolve!: (value: ComposerSubmissionPhase) => void;
      const completion = new Promise<ComposerSubmissionPhase>((done) => { resolve = done; });
      const record: PendingRecord = { input, phase: "pending", resolve, completion, admission, isRetry };
      records.set(input.nonce, record);
      const queued = options.isTransportDown() || activeByAgent.has(input.agentId);
      if (queued) notify(record, "queued");
      else run(record, generation);
      return { nonce: input.nonce, completion };
  };

  return {
    submit,
    retryFailed(nonce, admission) {
      if (disposed) return null;
      const prior = records.get(nonce);
      if (prior == null) return null;
      if (prior.phase === "pending" || prior.phase === "queued") return { nonce, completion: prior.completion };
      return prior.phase === "failed" ? submit(prior.input, admission, true) : null;
    },
    discard(nonce) {
      const record = records.get(nonce);
      if (record == null || record.phase !== "failed") return;
      records.delete(nonce);
      record.resolve("cancelled");
    },
    cancelQueued(nonce) {
      const record = records.get(nonce);
      if (record == null || record.phase !== "queued") return false;
      records.delete(nonce);
      record.resolve("cancelled");
      if (record.admission?.isCurrent?.() !== false) options.onPhase?.({ ...record.input, phase: "cancelled",
        ...(record.isRetry ? { isRetry: true } : {}),
        ...(record.admission?.isCurrent == null ? {} : { isCurrentAttempt: record.admission.isCurrent }) });
      return true;
    },
    flush,
    snapshot() {
      return [...records.values()].map((record) => ({ ...record.input, phase: record.phase }));
    },
    reset() {
      if (!disposed) reset();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      reset();
    }
  };
}
