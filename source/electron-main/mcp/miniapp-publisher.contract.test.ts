import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { DandelionCandidate, registerMiniAppPublisher } from "./miniapp-publisher.js";

function setup(options: {
  bytes?: Uint8Array; networkFailure?: boolean; switchDuringPick?: boolean;
  token?: string | null; pickerFailure?: boolean;
  pick?: () => Promise<Uint8Array | null>;
  respond?: (url: string, init: RequestInit | undefined, call: number) => Response | Promise<Response>;
} = {}) {
  const sender = {};
  const handlers = new Map<string, (event: unknown) => Promise<unknown>>();
  const calls: { url: string; method: string; headers: Headers; body: unknown }[] = [];
  let token = options.token === undefined ? "ephemeral-fixture-token" : options.token;
  const handle = registerMiniAppPublisher({
    ipc: { handle(name, handler) { handlers.set(name, handler); } }, trustedSender: () => sender,
    accessToken: async () => token,
    selectPackage: async () => { if (options.pickerFailure) throw new Error("/private/fixture-user-path bearer fixture-secret"); if (options.switchDuringPick) token = "different-account-fixture"; return options.pick ? options.pick() : options.bytes ?? null; },
    fetch: async (url, init) => {
      calls.push({ url: String(url), method: init?.method ?? "GET", headers: new Headers(init?.headers), body: init?.body });
      assert.equal(init?.credentials, "omit");
      assert.equal(init?.cache, "no-store");
      assert.equal(init?.referrerPolicy, "no-referrer");
      if (options.networkFailure) throw new Error("fixture uncertain write");
      assert.equal(init?.redirect, "manual");
      if (options.respond) return options.respond(String(url), init, calls.length);
      if (String(url).endsWith("namespaces")) return Response.json({ namespaces: [{ namespace: "bhrumom", privateIgnoredField: "fixture" }] });
      if (String(url).endsWith("releases") && init?.method === "GET") return Response.json({ releases: [] });
      if (String(url).endsWith("/submit")) return Response.json({ release: { state: "pending_review" } });
      return Response.json({ release: { state: "draft" }, artifact: {} });
    },
  });
  return { calls, handle, setToken: (value: string | null) => { token = value; }, invoke: (name: string, event: unknown = { sender }) => handlers.get(name)!(event) };
}
test("untrusted sender and missing account cannot reach publisher service", async () => {
  const p = setup();
  await assert.rejects(p.invoke("sand:marketplace-publisher-claim", { sender: {} }), /SENDER_DENIED/);
  assert.equal(p.calls.length, 0);
  const anonymous = setup({ token: null });
  await assert.rejects(anonymous.invoke("sand:marketplace-publisher-owned"), /ACCOUNT_REQUIRED/);
  assert.equal(anonymous.calls.length, 0);
});
test("account departure during native file selection and unreviewed bytes fail closed", async () => {
  const changed = setup({ bytes: new Uint8Array([1]), switchDuringPick: true });
  await assert.rejects(changed.invoke("sand:marketplace-submit-dandelion"), /ACCOUNT_CHANGED/);
  assert.equal(changed.calls.length, 0);
  const invalid = setup({ bytes: new Uint8Array([1]) });
  await assert.rejects(invalid.invoke("sand:marketplace-submit-dandelion"), /UNREVIEWED_PACKAGE/);
  assert.equal(invalid.calls.length, 0);
});
test("namespace projection excludes upstream private fields; no credential returned", async () => {
  const p = setup();
  assert.deepEqual(await p.invoke("sand:marketplace-publisher-owned"), { namespaces: ["bhrumom"] });
  assert.equal(p.calls[0]!.headers.get("Authorization"), "Bearer ephemeral-fixture-token");
  p.handle.dispose();
  await assert.rejects(p.invoke("sand:marketplace-publisher-owned"), /CLOSED/);
});
test("uncertain namespace mutation is never retried", async () => {
  const p = setup({ networkFailure: true });
  await assert.rejects(p.invoke("sand:marketplace-publisher-claim"), /UNCERTAIN_NO_RETRY/);
  assert.equal(p.calls.length, 1);
});
test("native file/account diagnostics never escape the publisher IPC boundary", async () => {
  const p = setup({ pickerFailure: true });
  await assert.rejects(p.invoke("sand:marketplace-submit-dandelion"), { message: "PUBLISHER_OPERATION_FAILED" });
  assert.equal(p.calls.length, 0);
});
test("actual production-signed candidate follows genuine account draft/upload/submit order", async () => {
  const filename = process.env.FABUSHI_SIGNED_DANDELION_FIXTURE;
  assert.ok(filename, "Actions must provide exact production-signed fixture; never skip");
  const bytes = new Uint8Array(readFileSync(filename));
  const p = setup({ bytes });
  assert.deepEqual(await p.invoke("sand:marketplace-submit-dandelion"), { state: "pending_review", digest: DandelionCandidate.digest });
  assert.equal(p.calls.length, 5);
  const upload = p.calls.find((call) => call.url.endsWith("/artifacts"))!;
  assert.equal(upload.headers.get("x-fabushi-artifact-format"), "fabapp");
  assert.deepEqual(upload.body, bytes);
  const draft = JSON.parse(p.calls[2]!.body as string).descriptor;
  assert.equal(draft.runtime.license, "MIT OR Apache-2.0");
  assert.equal(draft.runtime.nativeTransferAvailable, false);
  assert.equal(draft.release.bundleDigest, DandelionCandidate.digest);
});

test("cancelled selection performs no write and releases the publication lock", async () => {
  let picks = 0;
  const p = setup({ pick: async () => { picks += 1; return null; } });
  assert.deepEqual(await p.invoke("sand:marketplace-submit-dandelion"), { state: "cancelled" });
  assert.deepEqual(await p.invoke("sand:marketplace-submit-dandelion"), { state: "cancelled" });
  assert.equal(picks, 2);
  assert.equal(p.calls.length, 0);
});

test("selected byte bounds reject empty and oversized input before hashing or writes", async () => {
  for (const size of [0, 25 * 1024 * 1024 + 1]) {
    const p = setup({ bytes: new Uint8Array(size) });
    await assert.rejects(p.invoke("sand:marketplace-submit-dandelion"), { message: "PUBLISHER_PACKAGE_LIMIT" });
    assert.equal(p.calls.length, 0);
  }
  const inclusive = setup({ bytes: new Uint8Array(25 * 1024 * 1024) });
  await assert.rejects(inclusive.invoke("sand:marketplace-submit-dandelion"), { message: "PUBLISHER_UNREVIEWED_PACKAGE" });
  assert.equal(inclusive.calls.length, 0, "inclusive limit reaches digest validation, never upload");
});

test("pending selection is exclusive and account departure or disposal fences its result", async (t) => {
  for (const change of ["token", "sign-out", "dispose"] as const) {
    await t.test(change, async () => {
      let picked!: (value: Uint8Array | null) => void;
      let entered!: () => void;
      const started = new Promise<void>((resolve) => { entered = resolve; });
      const selection = new Promise<Uint8Array | null>((resolve) => { picked = resolve; });
      const p = setup({ pick: () => { entered(); return selection; } });
      const pending = p.invoke("sand:marketplace-submit-dandelion");
      const rejected = assert.rejects(pending, { message: "PUBLISHER_ACCOUNT_CHANGED" });
      await started;
      await assert.rejects(p.invoke("sand:marketplace-submit-dandelion"), { message: "PUBLISHER_BUSY" });
      if (change === "dispose") p.handle.dispose();
      else p.setToken(change === "sign-out" ? null : "changed-fixture-token");
      picked(null); // Cancellation cannot bypass the account/disposal fence.
      await rejected;
      assert.equal(p.calls.length, 0);
      if (change === "dispose") await assert.rejects(p.invoke("sand:marketplace-publisher-claim"), { message: "PUBLISHER_CLOSED" });
      else {
        p.setToken("restored-fixture-token");
        assert.deepEqual(await p.invoke("sand:marketplace-submit-dandelion"), { state: "cancelled" });
      }
    });
  }
});

test("namespace projection enforces syntax and the inclusive 128-character bound", async () => {
  const accepted = ["bhrumom", "a".repeat(128)];
  const p = setup({ respond: () => Response.json({ namespaces: [
    ...accepted.map((namespace) => ({ namespace })),
    ...["", "a".repeat(129), "../escape", "Upper", "-prefix", "a\nb", "a\n", "中文", 3, null]
      .map((namespace) => ({ namespace })),
  ], privateField: "not projected" }) });
  assert.deepEqual(await p.invoke("sand:marketplace-publisher-owned"), { namespaces: accepted });
});

test("response byte limits cancel oversized streams and malformed responses stay safe", async () => {
  const json = JSON.stringify({ namespaces: [{ namespace: "bhrumom" }] });
  const inclusive = setup({ respond: () => new Response(json.padEnd(1024 * 1024, " ")) });
  assert.deepEqual(await inclusive.invoke("sand:marketplace-publisher-owned"), { namespaces: ["bhrumom"] });
  let cancelled = false;
  const oversized = setup({ respond: () => new Response(new ReadableStream<Uint8Array>({
    start(controller) { controller.enqueue(new Uint8Array(1024 * 1024 + 1)); },
    cancel() { cancelled = true; },
  })) });
  await assert.rejects(oversized.invoke("sand:marketplace-publisher-owned"), { message: "PUBLISHER_RESPONSE_LIMIT" });
  assert.equal(cancelled, true);
  for (const respond of [() => new Response("{broken"), () => new Response(null)]) {
    const malformed = setup({ respond });
    await assert.rejects(malformed.invoke("sand:marketplace-publisher-owned"), { message: "PUBLISHER_INVALID_RESPONSE" });
  }
  const rejected = setup({ respond: () => new Response("private upstream diagnostics", { status: 403 }) });
  await assert.rejects(rejected.invoke("sand:marketplace-publisher-claim"), { message: "PUBLISHER_HTTP_403" });
  assert.equal(rejected.calls.length, 1);
});

function signedPublisherBytes(): Uint8Array {
  const filename = process.env.FABUSHI_SIGNED_DANDELION_FIXTURE;
  assert.ok(filename, "Actions must supply the immutable signed candidate; no digest override");
  return new Uint8Array(readFileSync(filename));
}

test("token changes and disposal while a response body is pending reject its eventual result", async (t) => {
  for (const change of ["token", "dispose"] as const) {
    await t.test(change, async () => {
      let controller!: ReadableStreamDefaultController<Uint8Array>;
      let entered!: () => void;
      const reading = new Promise<void>((resolve) => { entered = resolve; });
      const p = setup({ respond: () => new Response(new ReadableStream<Uint8Array>({
        start(value) { controller = value; },
        pull() { entered(); },
      })) });
      const pending = p.invoke("sand:marketplace-publisher-claim");
      const rejected = assert.rejects(pending, { message: "PUBLISHER_ACCOUNT_CHANGED" });
      await reading;
      if (change === "token") p.setToken("changed-during-body-fixture");
      else p.handle.dispose();
      controller.enqueue(new TextEncoder().encode("{}"));
      controller.close();
      await rejected;
      assert.equal(p.calls.length, 1, "an already sent mutation is never automatically retried");
    });
  }
});

test("token changes and disposal at each response fence stop subsequent publication mutations", async (t) => {
  const bytes = signedPublisherBytes();
  for (const change of ["token", "dispose"] as const) {
    for (let boundary = 1; boundary <= 5; boundary += 1) {
      await t.test(`${change} after request ${boundary}`, async () => {
        const p = setup({ bytes, respond: (url, init, call) => {
          if (call === boundary) {
            if (change === "token") p.setToken("replacement-fixture-token");
            else p.handle.dispose();
          }
          if (url.endsWith("namespaces")) return Response.json({ namespaces: [{ namespace: "bhrumom" }] });
          if (url.endsWith("releases") && init?.method === "GET") return Response.json({ releases: [] });
          if (url.endsWith("/submit")) return Response.json({ release: { state: "pending_review" } });
          return Response.json({ release: { state: "draft" } });
        } });
        await assert.rejects(p.invoke("sand:marketplace-submit-dandelion"), { message: "PUBLISHER_ACCOUNT_CHANGED" });
        assert.equal(p.calls.length, boundary, "no next request, retry or fabricated success");
      });
    }
  }
});

test("immutable conflicts and frozen releases cannot reach artifact upload or submit", async (t) => {
  const bytes = signedPublisherBytes();
  for (const state of ["conflicting-digest", "rejected", "yanked"] as const) {
    await t.test(state, async () => {
      const p = setup({ bytes, respond: (url) => url.endsWith("namespaces")
        ? Response.json({ namespaces: [{ namespace: "bhrumom" }] })
        : Response.json({ releases: [{ namespace: "bhrumom", name: "global-dandelion", kind: "web-miniapp", version: "0.2.1",
          state: state === "conflicting-digest" ? "draft" : state,
          descriptor: { release: { bundleDigest: state === "conflicting-digest" ? "0".repeat(64) : DandelionCandidate.digest } },
        }] }) });
      await assert.rejects(p.invoke("sand:marketplace-submit-dandelion"), {
        message: state === "conflicting-digest" ? "PUBLISHER_IMMUTABLE_CONFLICT" : "PUBLISHER_RELEASE_FROZEN",
      });
      assert.equal(p.calls.length, 2);
      assert.ok(p.calls.every((call) => call.method === "GET"));
    });
  }
});
