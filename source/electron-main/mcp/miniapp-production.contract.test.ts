import { test } from "node:test";
import assert from "node:assert/strict";
import { registerProductionMiniApps } from "./miniapp-production.js";
import type { ProductionServiceContext } from "../main-production-services.js";

function fixture() {
  const sender = {};
  const handlers = new Map<string, (event: unknown, id?: unknown) => Promise<unknown>>();
  const listeners = new Set<() => void>();
  const calls: string[] = [];
  const entry = { appId: "org.fabushi.global-dandelion", publisherId: "bhrumom", displayName: "Global Dandelion", version: "0.1.0", packageDigest: "a".repeat(64), sizeBytes: 512, offlineCore: true, installed: true, supported: true };
  const context = {
    native: { app: { commandLine: {} } },
    getTrustedContents: () => sender,
    requireAccount: () => ({
      getStatus: async () => ({ kind: "logged-in", authId: "real-user-1" }),
      subscribe: (listener: () => void) => { listeners.add(listener); return () => { listeners.delete(listener); }; },
    }),
    coordinatorLegs: { legs: {
      listInstalledMiniApps: async () => ({ items: [entry] }),
      listApprovedMiniApps: async () => { throw new Error("offline"); },
      installApprovedMiniApp: async (args: Record<string, unknown>) => { calls.push("install"); assert.equal(args.appId, entry.appId); assert.match(String(args.operationId), /^[0-9a-f-]{36}$/); },
      uninstallMiniApp: async () => { calls.push("remove"); },
    } },
  } as unknown as ProductionServiceContext;
  const handle = registerProductionMiniApps(context, { handle: (name, fn) => { handlers.set(name, fn); } });
  return { context, handle, listeners, calls, entry, invoke: (name: string, id?: unknown) => handlers.get(name)!({ sender }, id), handlers };
}

test("production keeps installed metadata usable when catalog is offline", async () => {
  const f = fixture();
  assert.deepEqual(await f.invoke("sand:miniapp-list"), { entries: [f.entry] });
  await f.invoke("sand:miniapp-install", f.entry.appId);
  assert.deepEqual(f.calls, ["install"]);
  f.handle.dispose();
  assert.equal(f.listeners.size, 0);
  await assert.rejects(f.invoke("sand:miniapp-list"), /MINIAPP_CLOSED/);
});

test("production never substitutes unsandboxed or absent sessions for launch", async () => {
  const f = fixture();
  await assert.rejects(f.invoke("sand:miniapp-launch", f.entry.appId), /MINIAPP_UNSUPPORTED_HOST/);
  assert.deepEqual(f.calls, []);
  await assert.rejects(f.handlers.get("sand:miniapp-list")!({ sender: {} }), /MINIAPP_SENDER_DENIED/);
  f.handle.dispose();
});

test("installed ready or quarantined truth wins newer catalog metadata", async () => {
  for (const supported of [true, false]) {
    const f = fixture();
    f.entry.supported = supported;
    const catalogOnly = { ...f.entry, appId: "org.fabushi.other", installed: false };
    f.context.coordinatorLegs.legs.listApprovedMiniApps = async () => ({ items: [
      { ...f.entry, version: "0.2.0", packageDigest: "b".repeat(64), supported: true, installed: true },
      catalogOnly,
    ] });
    assert.deepEqual(await f.invoke("sand:miniapp-list"), { entries: [f.entry, catalogOnly] });
    f.handle.dispose();
  }
});

test("account notification fences an in-flight native catalog response", async () => {
  const f = fixture();
  let resolve!: (value: unknown) => void;
  f.context.coordinatorLegs.legs.listInstalledMiniApps = async () => new Promise((done) => { resolve = done; });
  const pending = f.invoke("sand:miniapp-list");
  while (!resolve) await new Promise((done) => setImmediate(done));
  for (const listener of f.listeners) listener();
  resolve({ items: [f.entry] });
  await assert.rejects(pending, /MINIAPP_SCOPE_CHANGED/);
  f.handle.dispose();
});

test("production refuses Host resources from a different verified account", async () => {
  const f = fixture();
  f.handle.dispose();
  let opened = 0;
  Object.assign(f.context.native, {
    session: { fromPartition() { throw new Error("must not access partition"); } },
    BrowserWindow: class { constructor() { opened += 1; } },
  });
  f.context.coordinatorLegs.legs.getInstalledMiniAppResources = async (raw) => {
    const args = raw as Record<string, unknown>;
    assert.deepEqual(Object.keys(args).sort(), ["appId", "operationId"]);
    assert.equal(args.appId, f.entry.appId);
    return { sameScopeHash: "b".repeat(64), descriptor: {}, resources: [] };
  };
  const registration = registerProductionMiniApps(f.context, { handle: (name, fn) => { f.handlers.set(name, fn); } });
  await assert.rejects(f.invoke("sand:miniapp-launch", f.entry.appId), /MINIAPP_SCOPE_CHANGED/);
  assert.equal(opened, 0);
  registration.dispose();
});
