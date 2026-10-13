import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { DandelionCandidate, registerMiniAppPublisher } from "./miniapp-publisher.js";

function setup(options: { bytes?: Uint8Array; networkFailure?: boolean; switchDuringPick?: boolean; token?: string | null; pickerFailure?: boolean } = {}) {
  const sender = {};
  const handlers = new Map<string, (event: unknown) => Promise<unknown>>();
  const calls: { url: string; method: string; headers: Headers; body: unknown }[] = [];
  let token = options.token === undefined ? "ephemeral-fixture-token" : options.token;
  const handle = registerMiniAppPublisher({
    ipc: { handle(name, handler) { handlers.set(name, handler); } }, trustedSender: () => sender,
    accessToken: async () => token,
    selectPackage: async () => { if (options.pickerFailure) throw new Error("/private/fixture-user-path bearer fixture-secret"); if (options.switchDuringPick) token = "different-account-fixture"; return options.bytes ?? null; },
    fetch: async (url, init) => {
      calls.push({ url: String(url), method: init?.method ?? "GET", headers: new Headers(init?.headers), body: init?.body });
      assert.equal(init?.credentials, "omit");
      assert.equal(init?.cache, "no-store");
      assert.equal(init?.referrerPolicy, "no-referrer");
      if (options.networkFailure) throw new Error("fixture uncertain write");
      assert.equal(init?.redirect, "manual");
      if (String(url).endsWith("namespaces")) return Response.json({ namespaces: [{ namespace: "bhrumom", privateIgnoredField: "fixture" }] });
      if (String(url).endsWith("releases") && init?.method === "GET") return Response.json({ releases: [] });
      if (String(url).endsWith("/submit")) return Response.json({ release: { state: "pending_review" } });
      return Response.json({ release: { state: "draft" }, artifact: {} });
    },
  });
  return { calls, handle, invoke: (name: string, event: unknown = { sender }) => handlers.get(name)!(event) };
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
  assert.equal(draft.runtime.license, "LicenseRef-Proprietary");
  assert.equal(draft.runtime.nativeTransferAvailable, false);
  assert.equal(draft.release.bundleDigest, DandelionCandidate.digest);
});
