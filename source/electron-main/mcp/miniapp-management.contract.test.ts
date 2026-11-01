import test from "node:test";
import assert from "node:assert/strict";
import { registerMiniAppManagement, type NativeMiniAppEntry } from "./miniapp-management.js";

function fixture() {
  const sender = {}, calls: string[] = [], handlers = new Map<string, (event: unknown, value?: unknown) => Promise<unknown>>();
  let account: string | null = "fixture-account-a", onChange = () => {};
  const entry: NativeMiniAppEntry = { appId: "org.test.app", publisherId: "test", version: "1.0.0",
    displayName: "Offline app", packageDigest: "a".repeat(64), sizeBytes: 100,
    offlineCore: true, installed: false, supported: true };
  let install: () => Promise<void> = async () => {};
  const service = registerMiniAppManagement({
    ipc: { handle(name, fn) { handlers.set(name, fn); } }, trustedSender: () => sender,
    accountScope: async () => account,
    subscribeAccount(fn) { onChange = fn; return () => { onChange = () => {}; }; },
    host: { list: async () => [{ ...entry, privateIgnored: "fixture-secret" }],
      installApproved: async (id, op) => { assert.match(op, /^[a-f0-9-]{36}$/); calls.push(`install:${id}`); await install(); },
      removeCode: async (id) => { calls.push(`remove:${id}`); } },
    windows: { launch: async (id, scope, stillCurrent) => {
      assert.equal(scope, account); assert.equal(await stillCurrent(), true); calls.push(`launch:${id}`);
    }, closeApp: (id) => { calls.push(`close:${id}`); }, closeAll: () => { calls.push("close-all"); } },
  });
  return { calls, service, entry,
    installWith(fn: () => Promise<void>) { install = fn; },
    change(next: string | null) { account = next; onChange(); },
    invoke(name: string, value?: unknown, event: unknown = { sender }) { return handlers.get(name)!(event, value); } };
}
test("native management denies untrusted sender/account and arbitrary paths or policy objects", async () => {
  const f = fixture();
  await assert.rejects(f.invoke("sand:miniapp-install", "org.test.app", { sender: {} }), /SENDER_DENIED/);
  for (const invalid of ["../path", "/absolute", { appId: "org.test.app", key: "untrusted" }]) {
    await assert.rejects(f.invoke("sand:miniapp-install", invalid), /ID_INVALID/);
  }
  f.change(null);
  await assert.rejects(f.invoke("sand:miniapp-list"), /ACCOUNT_REQUIRED/);
  assert.deepEqual(f.calls, ["close-all"]);
});
test("public projection does not leak account, resources or extra fields", async () => {
  const f = fixture();
  assert.deepEqual(await f.invoke("sand:miniapp-list"), { entries: [f.entry] });
  f.entry.sizeBytes = Number.MAX_SAFE_INTEGER;
  await assert.rejects(f.invoke("sand:miniapp-list"), /DESCRIPTOR_INVALID/);
});
test("account departure closes windows synchronously and fences late activation", async () => {
  const f = fixture();
  f.installWith(async () => { f.change("fixture-account-b"); });
  await assert.rejects(f.invoke("sand:miniapp-install", "org.test.app"), /SCOPE_CHANGED/);
  assert.deepEqual(f.calls, ["install:org.test.app", "close-all"]);
});
test("update hints retain installed identity and reject malformed metadata", async () => {
  const f = fixture();
  f.entry.installed = true;
  const update = { version: "1.1.0", packageDigest: "b".repeat(64) };
  f.entry.availableUpdate = update;
  assert.deepEqual(await f.invoke("sand:miniapp-list"), { entries: [f.entry] });
  for (const invalid of [
    { ...update, packageDigest: f.entry.packageDigest },
    { ...update, packageDigest: "invalid" },
    { ...update, version: "invalid" },
    { ...update, privatePath: "/private" },
  ]) {
    f.entry.availableUpdate = invalid;
    await assert.rejects(f.invoke("sand:miniapp-list"), /DESCRIPTOR_INVALID/);
  }
  f.entry.availableUpdate = update;
  f.entry.installed = false;
  await assert.rejects(f.invoke("sand:miniapp-list"), /DESCRIPTOR_INVALID/);
  f.service.dispose();
});
test("per-app serialization, code-only removal and disposal are explicit", async () => {
  const f = fixture();
  let release!: () => void;
  f.installWith(() => new Promise((resolve) => { release = resolve; }));
  const pending = f.invoke("sand:miniapp-install", "org.test.app");
  await new Promise((resolve) => setImmediate(resolve));
  await assert.rejects(f.invoke("sand:miniapp-launch", "org.test.app"), /BUSY/);
  release(); await pending;
  await f.invoke("sand:miniapp-remove-code", "org.test.app");
  assert.deepEqual(f.calls.slice(-2), ["close:org.test.app", "remove:org.test.app"]);
  f.service.dispose();
  await assert.rejects(f.invoke("sand:miniapp-list"), /CLOSED/);
});
test("unexpected Host errors are redacted and uncertain operations are not retried", async () => {
  const f = fixture();
  f.installWith(async () => { throw new Error("private-path credential fixture"); });
  await assert.rejects(f.invoke("sand:miniapp-install", "org.test.app"), /MINIAPP_OPERATION_FAILED/);
  assert.equal(f.calls.length, 1);
});
