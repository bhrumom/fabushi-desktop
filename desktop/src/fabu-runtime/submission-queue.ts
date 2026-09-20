import type { AttachmentContext } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { AgentPromptReference, AgentReplyContext } from '../agent-workspace/prompt-context';
export type AgentSubmissionPhase = 'pending' | 'queued' | 'failed' | 'sent' | 'cancelled';

export interface AgentSubmission {
  nonce: string;
  messageId: string;
  peerKey: string;
  agentId: string;
  prompt: string;
  attachments?: readonly AttachmentContext[];
  references?: readonly AgentPromptReference[];
  replyTo?: AgentReplyContext;
  createdAtMs: number;
}

export interface AgentSubmissionRecord extends AgentSubmission {
  phase: Exclude<AgentSubmissionPhase, 'cancelled'>;
}

export interface AgentSubmissionQueue {
  submit(input: AgentSubmission): { nonce: string; completion: Promise<AgentSubmissionPhase> };
  cancelQueued(nonce: string): boolean;
  discard(nonce: string): void;
  flush(): void;
  snapshot(): readonly AgentSubmissionRecord[];
  dispose(): void;
}

interface QueueOptions {
  isBlocked(input: AgentSubmission): boolean;
  send(input: AgentSubmission): Promise<void>;
  onPhase?(input: AgentSubmission & { phase: AgentSubmissionPhase }): void;
  onFailure?(input: AgentSubmissionRecord, error: unknown): void;
}

interface PendingRecord {
  input: AgentSubmission;
  phase: Exclude<AgentSubmissionPhase, 'cancelled'>;
  resolve(value: AgentSubmissionPhase): void;
  completion: Promise<AgentSubmissionPhase>;
}

/**
 * Port of Fabu's composer send journal.
 *
 * Submission acceptance and Agent generation are deliberately different
 * lifecycles: queued/pending/sent describes transport delivery, while the
 * conversation renderer owns thinking/running/completed.
 */
export function createAgentSubmissionQueue(options: QueueOptions): AgentSubmissionQueue {
  const records = new Map<string, PendingRecord>();
  const activeByAgent = new Map<string, string>();
  let generation = 0;
  let disposed = false;

  const notify = (record: PendingRecord, phase: PendingRecord['phase']) => {
    record.phase = phase;
    options.onPhase?.({ ...record.input, phase });
  };

  const finish = (record: PendingRecord, phase: AgentSubmissionPhase) => {
    if (records.get(record.input.nonce) !== record) return;
    if (phase === 'sent' || phase === 'cancelled') records.delete(record.input.nonce);
    if (activeByAgent.get(record.input.agentId) === record.input.nonce) {
      activeByAgent.delete(record.input.agentId);
    }
    record.resolve(phase);
    if (phase !== 'cancelled') options.onPhase?.({ ...record.input, phase });
  };

  const flushAgent = (agentId: string, runGeneration: number) => {
    if (disposed || generation !== runGeneration || activeByAgent.has(agentId)) return;
    const next = [...records.values()].find((record) =>
      record.input.agentId === agentId && record.phase === 'queued' && !options.isBlocked(record.input),
    );
    if (next) run(next, runGeneration);
  };

  const run = (record: PendingRecord, runGeneration: number) => {
    if (disposed || generation !== runGeneration || records.get(record.input.nonce) !== record) return;
    if (options.isBlocked(record.input)) {
      notify(record, 'queued');
      return;
    }
    activeByAgent.set(record.input.agentId, record.input.nonce);
    notify(record, 'pending');
    void options.send(record.input).then(() => {
      if (disposed || generation !== runGeneration || records.get(record.input.nonce) !== record) return;
      finish(record, 'sent');
      flushAgent(record.input.agentId, runGeneration);
    }).catch((error: unknown) => {
      if (disposed || generation !== runGeneration || records.get(record.input.nonce) !== record) return;
      notify(record, 'failed');
      if (activeByAgent.get(record.input.agentId) === record.input.nonce) {
        activeByAgent.delete(record.input.agentId);
      }
      record.resolve('failed');
      options.onFailure?.({ ...record.input, phase: 'failed' }, error);
    });
  };

  return {
    submit(input) {
      if (disposed) return { nonce: input.nonce, completion: Promise.resolve('cancelled') };
      let resolve!: (value: AgentSubmissionPhase) => void;
      const completion = new Promise<AgentSubmissionPhase>((done) => { resolve = done; });
      const record: PendingRecord = { input, phase: 'pending', resolve, completion };
      records.set(input.nonce, record);
      const queued = options.isBlocked(input) || activeByAgent.has(input.agentId);
      if (queued) notify(record, 'queued');
      else run(record, generation);
      return { nonce: input.nonce, completion };
    },
    cancelQueued(nonce) {
      const record = records.get(nonce);
      if (!record || record.phase !== 'queued') return false;
      records.delete(nonce);
      record.resolve('cancelled');
      options.onPhase?.({ ...record.input, phase: 'cancelled' });
      return true;
    },
    discard(nonce) {
      const record = records.get(nonce);
      if (!record || record.phase !== 'failed') return;
      records.delete(nonce);
      record.resolve('cancelled');
    },
    flush() {
      if (disposed) return;
      const runGeneration = generation;
      const agentIds = new Set([...records.values()].map((record) => record.input.agentId));
      for (const agentId of agentIds) flushAgent(agentId, runGeneration);
    },
    snapshot() {
      return [...records.values()].map((record) => ({ ...record.input, phase: record.phase }));
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      generation += 1;
      activeByAgent.clear();
      for (const record of records.values()) record.resolve('cancelled');
      records.clear();
    },
  };
}
