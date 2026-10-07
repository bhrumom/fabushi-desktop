import assert from "node:assert/strict";
import test from "node:test";

import {
  createConversationChildPageRequestController,
  type ConversationChildPage,
  type ConversationChildPageRequest
} from "./child-pagination.ts";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolveValue, rejectValue) => {
    resolve = resolveValue;
    reject = rejectValue;
  });
  return { promise, resolve, reject };
}

const page = (id: string): ConversationChildPage => ({ messageIds: [id] });

test("scope replacement aborts child-history requests and fences stale settlement", async () => {
  const pending = deferred<ConversationChildPage>();
  let signal: AbortSignal | null = null;
  const commits: string[] = [];
  const controller = createConversationChildPageRequestController({
    fetchPage: async (_request, requestSignal) => {
      signal = requestSignal;
      return pending.promise;
    },
    getBoundary: () => null,
    commitPage: (_direction, result) => commits.push(...result.messageIds)
  });

  controller.setScope("saved:one");
  const request = controller.loadAround("100");
  controller.setScope("saved:two");

  assert.equal(signal?.aborted, true);
  pending.resolve(page("stale"));
  await request;
  assert.deepEqual(commits, []);
  assert.equal(controller.getSnapshot().scopeKey, "saved:two");
});

test("same-around duplicate coalesces into exactly one retry after settlement", async () => {
  const requests: ConversationChildPageRequest[] = [];
  const pending = [deferred<ConversationChildPage>(), deferred<ConversationChildPage>(), deferred<ConversationChildPage>()];
  const commits: string[] = [];
  const controller = createConversationChildPageRequestController({
    fetchPage: (request) => {
      requests.push(request);
      return pending[requests.length - 1].promise;
    },
    getBoundary: () => null,
    commitPage: (_direction, result) => commits.push(...result.messageIds)
  });

  controller.setScope("saved:one");
  const first = controller.loadAround("100");
  const duplicate = controller.loadAround("100");
  assert.equal(first, duplicate);
  assert.equal(requests.length, 1);

  pending[0].resolve(page("first"));
  await first;
  await Promise.resolve();
  assert.equal(requests.length, 2);
  assert.equal(requests[1].direction, "around");
  assert.equal(requests[1].anchor, "100");

  pending[1].resolve(page("retry"));
  await pending[1].promise;
  await Promise.resolve();
  assert.deepEqual(commits, ["first", "retry"]);
});

test("stale before boundary is discarded and retried against the fresh boundary", async () => {
  let beforeBoundary = "m10";
  const requests: ConversationChildPageRequest[] = [];
  const pending = [deferred<ConversationChildPage>(), deferred<ConversationChildPage>()];
  const commits: string[] = [];
  const controller = createConversationChildPageRequestController({
    fetchPage: (request) => {
      requests.push(request);
      return pending[requests.length - 1].promise;
    },
    getBoundary: (direction) => direction === "before" ? beforeBoundary : null,
    commitPage: (_direction, result) => commits.push(...result.messageIds)
  });

  controller.setScope("saved:one");
  const first = controller.loadBefore();
  assert.equal(requests[0].anchor, "m10");

  beforeBoundary = "m9";
  pending[0].resolve(page("stale"));
  await first;
  await Promise.resolve();

  assert.deepEqual(commits, []);
  assert.equal(requests.length, 2);
  assert.equal(requests[1].anchor, "m9");

  beforeBoundary = "m8";
  pending[1].resolve(page("still-stale"));
  await pending[1].promise;
  await Promise.resolve();
  assert.deepEqual(commits, []);
  assert.equal(requests.length, 3);
  assert.equal(requests[2].anchor, "m8");

  pending[2].resolve(page("fresh"));
  await pending[2].promise;
  await Promise.resolve();
  assert.deepEqual(commits, ["fresh"]);
});

test("around supersedes an in-flight before request and prevents its commit", async () => {
  const before = deferred<ConversationChildPage>();
  const around = deferred<ConversationChildPage>();
  const signals: AbortSignal[] = [];
  const commits: string[] = [];
  let boundary = "m10";
  const controller = createConversationChildPageRequestController({
    fetchPage: (request, signal) => {
      signals.push(signal);
      return request.direction === "before" ? before.promise : around.promise;
    },
    getBoundary: (direction) => direction === "before" ? boundary : null,
    commitPage: (_direction, result) => commits.push(...result.messageIds)
  });

  controller.setScope("saved:one");
  const older = controller.loadBefore();
  const centered = controller.loadAround("m20");
  assert.equal(signals[0].aborted, true);

  before.resolve(page("stale-before"));
  await older;
  assert.deepEqual(commits, []);

  boundary = "m9";
  around.resolve(page("around"));
  await centered;
  assert.deepEqual(commits, ["around"]);
});
