/**
 * Contract verification for account-scoped composer dispatch fencing
 * (`composer-dispatch-scope.ts`) and NMR-50i forward retry settlement
 * (`ForwardMessageDialog.tsx`).
 *
 * Ported from PR #190 (`origin/checkpoint/tdrp-nmr50-recovery-20bf9475-r1`)
 * and aligned with Blueprint 03 (`docs/specs/fusion-blueprints/03-unified-composer-and-typed-actions.md`).
 */

import { describe, expect, it } from "vitest";
import { createComposerSubmissionQueue, type ComposerSubmission } from "../recovered/features/conversation/workspace/submission";
import { CoordinatorClient, type HostTransport } from "./coordinator-client";
import {
  bindComposerSubmissionScope,
  sendScopedComposerSubmission,
  type ComposerDispatchScope,
  type ScopedComposerSubmission,
} from "./composer-dispatch-scope";
import { createForwardDialogModel, type ForwardRecipientOption } from "./ForwardMessageDialog";

function makeSubmission(overrides: Partial<ComposerSubmission> = {}): ComposerSubmission {
  return {
    nonce: "nonce-1",
    agentId: "peer-1",
    prompt: "hello",
    attachments: [],
    createdAtMs: 1_710_000_000_000,
    ...overrides,
  };
}

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("composer dispatch account scope contract", () => {
  it("binds the submission origin at enqueue time and blocks stale queued dispatches after an account switch", async () => {
    let scope: ComposerDispatchScope = { accountId: "acct-a", generation: 1 };
    let transportDown = true;
    const calls: { method: string; params: Record<string, unknown> }[] = [];
    const queue = createComposerSubmissionQueue<ScopedComposerSubmission>({
      isTransportDown: () => transportDown,
      send: (submission) =>
        sendScopedComposerSubmission({
          client: {
            async call<T = unknown>(method: string, params: Record<string, unknown>): Promise<T> {
              calls.push({ method, params });
              return { message_id: "host-msg-1" } as T;
            },
          },
          submission,
          isHumanConversation: true,
        }),
    });

    const submitted = queue.submit(bindComposerSubmissionScope(makeSubmission(), () => scope));
    scope = { accountId: "acct-b", generation: 2 };
    transportDown = false;
    queue.flush();

    await expect(submitted.completion).resolves.toBe("failed");
    expect(calls).toEqual([]);
  });

  it("retires Human attachment upload before the second Host IPC call when the account switches mid-batch", async () => {
    let scope: ComposerDispatchScope = { accountId: "acct-a", generation: 1 };
    const firstUpload = deferred<{ file_path: string; file_name: string }>();
    const calls: { method: string; params: Record<string, unknown> }[] = [];
    const submission = bindComposerSubmissionScope(
      makeSubmission({
        attachments: [
          { path: "/tmp/one.png", name: "one.png" },
          { path: "/tmp/two.pdf", name: "two.pdf" },
        ],
      }),
      () => scope
    );

    const dispatched = sendScopedComposerSubmission({
      client: {
        async call<T = unknown>(method: string, params: Record<string, unknown>): Promise<T> {
          calls.push({ method, params });
          if (calls.length === 1) {
            return (await firstUpload.promise) as T;
          }
          throw new Error(`unexpected Host IPC after account switch: ${method}`);
        },
      },
      submission,
      isHumanConversation: true,
    });

    expect(calls).toEqual([
      { method: "media.upload", params: { path: "/tmp/one.png", mime_type: "image/png" } },
    ]);

    scope = { accountId: "acct-b", generation: 2 };
    firstUpload.resolve({ file_path: "/uploaded/one.png", file_name: "one.png" });

    await expect(dispatched).rejects.toMatchObject({
      code: "request-retired",
      message: "Composer dispatch retired before media.upload.",
    });
    expect(calls).toHaveLength(1);
  });

  it("retires Human send before messages.send goes on the wire when ready resolves after an account switch", async () => {
    const readyGate = deferred<void>();
    const posted: Record<string, unknown>[] = [];
    let account = "acct-a";
    let generation = 1;
    const transport: HostTransport = {
      postMessage(message) {
        posted.push(message);
      },
      subscribe() {
        return () => undefined;
      },
    };
    const client = new CoordinatorClient(transport, {
      ready: () => readyGate.promise,
      defaultTimeoutMs: 250,
    });
    const submission = bindComposerSubmissionScope(makeSubmission(), () => ({
      accountId: account,
      generation,
    }));

    const dispatched = sendScopedComposerSubmission({
      client,
      submission,
      isHumanConversation: true,
    });

    account = "acct-b";
    generation = 2;
    readyGate.resolve();

    await expect(dispatched).rejects.toMatchObject({
      code: "request-retired",
      message: "Coordinator request retired before messages.send.",
    });
    expect(posted).toEqual([]);
  });

  it("retires Agent sendPrompt before the Host IPC call goes on the wire after an account switch", async () => {
    const readyGate = deferred<void>();
    const posted: Record<string, unknown>[] = [];
    let scope: ComposerDispatchScope = { accountId: "acct-a", generation: 1 };
    const transport: HostTransport = {
      postMessage(message) {
        posted.push(message);
      },
      subscribe() {
        return () => undefined;
      },
    };
    const client = new CoordinatorClient(transport, {
      ready: () => readyGate.promise,
      defaultTimeoutMs: 250,
    });
    const submission = bindComposerSubmissionScope(
      makeSubmission({
        agentId: "agent-alpha",
        prompt: "run diagnostics",
        richText: "{\"type\":\"doc\"}",
        isFork: true,
      }),
      () => scope
    );

    const dispatched = sendScopedComposerSubmission({
      client,
      submission,
      isHumanConversation: false,
    });

    scope = { accountId: "acct-b", generation: 2 };
    readyGate.resolve();

    await expect(dispatched).rejects.toMatchObject({
      code: "request-retired",
      message: "Coordinator request retired before sendPrompt.",
    });
    expect(posted).toEqual([]);
  });
});

describe("ForwardDialogModel account scope and NMR-50i retry settlement contract", () => {
  const recipients: readonly ForwardRecipientOption[] = [
    { id: "peer-a", title: "Alpha", kind: "human" },
    { id: "peer-b", title: "Beta", kind: "human" },
  ];

  it("preserves previously confirmed recipients in the final settlement after retrying only failed recipients", async () => {
    const scope: ComposerDispatchScope = { accountId: "acct-a", generation: 1 };
    const calls: string[] = [];
    const settledBatches: string[][] = [];
    let closedCount = 0;
    let failBeta = true;

    const model = createForwardDialogModel({
      source: {
        messageId: "msg-10",
        fromPeerId: "peer-origin",
        authorLabel: "Origin",
        previewText: "ship it",
      },
      recipients,
      initialRecipientIds: ["peer-a", "peer-b"],
      readDispatchScope: () => scope,
      client: {
        async call<T = unknown>(_method: string, params: Record<string, unknown>): Promise<T> {
          const toPeerId = String(params.to_peer_id);
          calls.push(toPeerId);
          if (toPeerId === "peer-b" && failBeta) {
            throw new Error("beta temporary failure");
          }
          return {
            message: {
              id: `fwd-${toPeerId}`,
              peer_id: toPeerId,
              text: "ship it",
              created_at: "2026-10-16T00:00:00.000Z",
            },
          } as T;
        },
      },
      onSettled(settlements) {
        settledBatches.push(settlements.map((item) => `${item.toPeerId}:${item.messageId}`));
      },
      onClosed() {
        closedCount += 1;
      },
    });

    const firstAttempt = await model.submit();
    expect(firstAttempt.map((item) => item.toPeerId)).toEqual(["peer-a"]);
    expect(model.snapshot().selectedRecipientIds).toEqual(["peer-b"]);
    expect(settledBatches).toEqual([]);
    expect(closedCount).toBe(0);

    failBeta = false;
    const retryAttempt = await model.submit();
    expect(calls).toEqual(["peer-a", "peer-b", "peer-b"]);
    expect(retryAttempt.map((item) => `${item.toPeerId}:${item.messageId}`)).toEqual([
      "peer-a:fwd-peer-a",
      "peer-b:fwd-peer-b",
    ]);
    expect(settledBatches).toEqual([["peer-a:fwd-peer-a", "peer-b:fwd-peer-b"]]);
    expect(closedCount).toBe(1);
  });

  it("retires remaining recipient forwards when the account scope changes mid-batch", async () => {
    let scope: ComposerDispatchScope = { accountId: "acct-a", generation: 1 };
    const firstRecipientGate = deferred<void>();
    const calls: string[] = [];
    let closedCount = 0;

    const model = createForwardDialogModel({
      source: {
        messageId: "msg-11",
        fromPeerId: "peer-origin",
        authorLabel: "Origin",
        previewText: "confidential",
      },
      recipients,
      initialRecipientIds: ["peer-a", "peer-b"],
      readDispatchScope: () => scope,
      client: {
        async call<T = unknown>(_method: string, params: Record<string, unknown>): Promise<T> {
          const toPeerId = String(params.to_peer_id);
          calls.push(toPeerId);
          if (toPeerId === "peer-a") {
            await firstRecipientGate.promise;
          }
          return {
            message: {
              id: `fwd-${toPeerId}`,
              peer_id: toPeerId,
              text: "confidential",
              created_at: "2026-10-16T00:00:00.000Z",
            },
          } as T;
        },
      },
      onClosed() {
        closedCount += 1;
      },
    });

    const pendingSubmit = model.submit();
    expect(calls).toEqual(["peer-a"]);

    scope = { accountId: "acct-b", generation: 2 };
    firstRecipientGate.resolve();

    const settlements = await pendingSubmit;
    expect(settlements).toEqual([]);
    expect(calls).toEqual(["peer-a"]);
    expect(closedCount).toBe(0);
  });
});
