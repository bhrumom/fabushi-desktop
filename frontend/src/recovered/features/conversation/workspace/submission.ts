/**
 * The shipped send journal owns optimistic send, offline queueing, reconnect
 * flush, and stale-generation fencing around the coordinator send boundary.
 * @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5674804
 */

export type ComposerSubmissionPhase = "pending" | "queued" | "failed" | "sent" | "cancelled";

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
}

export type ComposerSubmissionRecord<Submission extends ComposerSubmission = ComposerSubmission> = Submission & {
  phase: Exclude<ComposerSubmissionPhase, "cancelled">;
};

export interface ComposerSubmissionQueue<Submission extends ComposerSubmission = ComposerSubmission> {
  submit(input: Submission): { nonce: string; completion: Promise<ComposerSubmissionPhase> };
  discard(nonce: string): void;
  cancelQueued(nonce: string): boolean;
  flush(): void;
  snapshot(): readonly ComposerSubmissionRecord<Submission>[];
  /** Retire renderer tracking without cancelling any request already owned by Host. */
  reset(): void;
  dispose(): void;
}

interface QueueOptions<Submission extends ComposerSubmission> {
  isTransportDown(): boolean;
  send(input: Submission): Promise<void>;
  now?(): number;
  onPhase?(input: Submission & { phase: ComposerSubmissionPhase }): void;
  onFailure?(input: ComposerSubmissionRecord<Submission>, error: unknown): void;
}

interface PendingRecord<Submission extends ComposerSubmission> {
  input: Submission;
  phase: Exclude<ComposerSubmissionPhase, "cancelled">;
  resolve(value: ComposerSubmissionPhase): void;
  completion: Promise<ComposerSubmissionPhase>;
}

/**
 * Small renderer-side boundary around the coordinator's sendPrompt contract.
 * It intentionally does not invent a stop-generation method: only queued
 * sends can be cancelled because that is the action exposed by the shipped
 * send journal and the current coordinator contract.
 */
export function createComposerSubmissionQueue<Submission extends ComposerSubmission = ComposerSubmission>(options: QueueOptions<Submission>): ComposerSubmissionQueue<Submission> {
  const records = new Map<string, PendingRecord<Submission>>();
  const activeByAgent = new Map<string, string>();
  let generation = 0;
  let disposed = false;

  const notify = (record: PendingRecord<Submission>, phase: PendingRecord<Submission>["phase"]) => {
    record.phase = phase;
    options.onPhase?.({ ...record.input, phase });
  };

  const finish = (record: PendingRecord<Submission>, phase: ComposerSubmissionPhase) => {
    if (records.get(record.input.nonce) !== record) return;
    if (phase === "sent" || phase === "cancelled") records.delete(record.input.nonce);
    if (activeByAgent.get(record.input.agentId) === record.input.nonce) activeByAgent.delete(record.input.agentId);
    record.resolve(phase);
    if (phase !== "cancelled") options.onPhase?.({ ...record.input, phase });
  };

  const run = (record: PendingRecord<Submission>, runGeneration: number) => {
    if (disposed || generation !== runGeneration || records.get(record.input.nonce) !== record) return;
    if (options.isTransportDown()) {
      notify(record, "queued");
      return;
    }
    activeByAgent.set(record.input.agentId, record.input.nonce);
    notify(record, "pending");
    void options.send(record.input).then(() => {
      if (disposed || generation !== runGeneration || records.get(record.input.nonce) !== record) return;
      finish(record, "sent");
      flushAgent(record.input.agentId, runGeneration);
    }).catch((error: unknown) => {
      if (disposed || generation !== runGeneration || records.get(record.input.nonce) !== record) return;
      notify(record, "failed");
      if (activeByAgent.get(record.input.agentId) === record.input.nonce) activeByAgent.delete(record.input.agentId);
      record.resolve("failed");
      options.onFailure?.({ ...record.input, phase: "failed" }, error);
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

  const retireTracking = () => {
    generation += 1;
    activeByAgent.clear();
    for (const record of records.values()) record.resolve("cancelled");
    records.clear();
  };

  return {
    submit(input) {
      if (disposed) return { nonce: input.nonce, completion: Promise.resolve("cancelled") };
      let resolve!: (value: ComposerSubmissionPhase) => void;
      const completion = new Promise<ComposerSubmissionPhase>((done) => { resolve = done; });
      const record: PendingRecord<Submission> = { input, phase: "pending", resolve, completion };
      records.set(input.nonce, record);
      const queued = options.isTransportDown() || activeByAgent.has(input.agentId);
      if (queued) notify(record, "queued");
      else run(record, generation);
      return { nonce: input.nonce, completion };
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
      options.onPhase?.({ ...record.input, phase: "cancelled" });
      return true;
    },
    flush,
    snapshot() {
      return [...records.values()].map((record) => ({ ...record.input, phase: record.phase }));
    },
    reset() {
      if (!disposed) retireTracking();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      retireTracking();
    }
  };
}
