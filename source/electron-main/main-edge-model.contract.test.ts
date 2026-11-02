import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test, { type TestContext } from "node:test";

import { serveMainEdge, type MainEdgeWiringDeps } from "./main-edge-wiring.js";
import {
  EDGE_HANDLER_FAILED,
  EDGE_UNTRUSTED_SENDER,
  MAIN_METHOD_TABLE,
  mainRpcContract,
  methodChannel,
  serveEdge,
} from "./generated/main-rpc.js";
import {
  emptySettings,
  SAND_DOWNGRADE_MAX_FAST_MIGRATION_ID,
  SandSettingsStore,
} from "../shared/node/settings/sand-settings-store.js";

type Settings = Record<string, unknown>;
type Reply =
  | { ok: true; value: unknown }
  | { ok: false; failure: { code: string; detail: string } };
type IpcEvent = { sender: unknown; senderFrame: unknown };

const legacyModel = () => ({
  modelId: "Legacy/Existing@Case",
  maxMode: true,
  parameters: [{ id: "thinking", value: "true" }],
});
const boundModel = () => ({
  ...legacyModel(),
  modelId: "Vendor/Nested/Selected@Case",
  provider: "fabushi",
});

function fixture(t: TestContext) {
  const root = mkdtempSync(join(tmpdir(), "fabushi-model-bridge-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const store = new SandSettingsStore(join(root, "settings.json"));
  store.setAgentDefaultModel(legacyModel());
  const hostWrites: Settings[] = [];
  let hostReads = 0;
  let writeHost: (update: Settings) => Promise<Settings | null> = async (update) => update;
  let readHost: () => Promise<Settings> = async () => ({ agentDefaultModel: legacyModel() });
  let computerUseOverride: unknown;
  const trustedContents = { mainFrame: {} };
  const listeners = new Map<string, (event: IpcEvent, payload: unknown) => unknown>();
  const unexpected = (): never => { throw new Error("unrelated main-edge port was called"); };
  const recordStore = store as unknown as Settings;
  const served = serveMainEdge({
    readLiveUpdateService: () => null,
    readThemeController: () => null,
    readEgressTunnelController: () => null,
    settingsStore: recordStore,
    agentPrefsStore: recordStore,
    boxToggleStore: {},
    onboardingSeen: {},
    shell: {},
    boxRecovery: {},
    windowChrome: {},
    avatarImages: {},
    attachments: {},
    cursorAccount: {},
    experiments: {},
    syncHostSettingsToBox: async (update) => {
      // Observe the JSON boundary seen by Rust, including property omission.
      const wireUpdate = JSON.parse(JSON.stringify(update)) as Settings;
      hostWrites.push(wireUpdate);
      return await writeHost(wireUpdate);
    },
    readHostSettingsFromBox: async () => {
      hostReads += 1;
      return await readHost();
    },
    recordLocalToolApproval: async () => unexpected(),
    clearLocalToolApprovals: async () => unexpected(),
    getComputerUseModelOverride: () => computerUseOverride,
    fetchAvailableModels: unexpected,
    emitEgressTunnelChanged: unexpected,
    emitWebauthnProxyChanged: unexpected,
    ensureTranscriptionManager: async () => unexpected(),
    platform: process.platform,
    ipcMain: {
      handle: (channel, listener) => { listeners.set(channel, listener); },
      removeHandler: (channel) => { listeners.delete(channel); },
    },
    getTrustedContents: () => trustedContents,
    broadcast: unexpected,
    // This is the same generated dispatcher used by the production adapter.
    serveEdge: serveEdge as unknown as MainEdgeWiringDeps["serveEdge"],
    mainRpcContract,
    mainMethodTable: MAIN_METHOD_TABLE,
  }) as { dispose(): void };
  t.after(() => served.dispose());

  return {
    store,
    hostWrites,
    trustedContents,
    get hostReads() { return hostReads; },
    setHostWrite(port: typeof writeHost) { writeHost = port; },
    setHostRead(port: typeof readHost) { readHost = port; },
    setComputerUseOverride(value: unknown) { computerUseOverride = value; },
    bytes: () => readFileSync(store.settingsPath),
    reopen: () => new SandSettingsStore(store.settingsPath),
    async call(method: string, payload: Settings = {}, event: IpcEvent = {
      sender: trustedContents,
      senderFrame: trustedContents.mainFrame,
    }): Promise<Reply> {
      const listener = listeners.get(methodChannel(mainRpcContract.edge, method));
      assert.ok(listener, "the real served method must be registered");
      return await listener(event, payload) as Reply;
    },
  };
}

test("default-model RPC retains provider and mirrors only the acknowledged Host model", async (t) => {
  const f = fixture(t);
  const requested = { ...boundModel(), maxMode: false };
  const acknowledged = {
    ...requested,
    maxMode: true,
    hostCatalogRevision: "catalog-7",
    metadata: { label: "Host-selected metadata" },
    parameters: [{ id: "thinking", value: "true", hostMetadata: "retained" }],
  };
  let acknowledge!: (value: Settings) => void;
  const hostReply = new Promise<Settings>((resolve) => { acknowledge = resolve; });
  f.setHostWrite(async () => await hostReply);
  const before = f.bytes();
  const pending = f.call("setAgentDefaultModel", { model: requested });
  assert.deepEqual(f.hostWrites, [{ agentDefaultModel: requested }]);
  assert.deepEqual(f.bytes(), before, "waiting for the Host must not publish a speculative default");
  assert.deepEqual(f.reopen().getAgentDefaultModel(), legacyModel());

  acknowledge({ agentDefaultModel: acknowledged });
  assert.deepEqual(await pending, { ok: true, value: acknowledged });
  assert.deepEqual(f.store.getAgentDefaultModel(), acknowledged);
  assert.deepEqual(f.reopen().getAgentDefaultModel(), acknowledged);
  assert.deepEqual(JSON.parse(f.bytes().toString()).agentDefaultModel, acknowledged);
});

test("invalid provider bindings reach the Host unchanged and refusal preserves mirror bytes", async (t) => {
  const f = fixture(t);
  const before = f.bytes();
  // The controlled port supplies refusal, not a replacement provider validator.
  // Rust Settings contracts independently exercise the production validation.
  f.setHostWrite(async () => { throw new Error("Host rejected the provider binding"); });
  for (const provider of [null, 42, false, "", " ", "FABUSHI", "unknown-provider", [], {}]) {
    const model = { ...boundModel(), provider };
    const reply = await f.call("setAgentDefaultModel", { model });
    const written = f.hostWrites.at(-1)?.agentDefaultModel as Settings;
    assert.ok(Object.prototype.hasOwnProperty.call(written, "provider"));
    assert.deepEqual(written, model);
    assert.deepEqual(reply, {
      ok: false,
      failure: { code: EDGE_HANDLER_FAILED, detail: "Host rejected the provider binding" },
    });
    assert.deepEqual(f.bytes(), before);
  }
  assert.equal(f.hostWrites.length, 9, "each present binding must reach the existing owner");
  assert.deepEqual(f.reopen().getAgentDefaultModel(), legacyModel());
});

test("explicit undefined provider is refused before JSON can turn it into a legacy default", async (t) => {
  const f = fixture(t);
  const before = f.bytes();
  for (const model of [{ ...boundModel(), provider: undefined }, { provider: undefined }]) {
    assert.ok(Object.prototype.hasOwnProperty.call(model, "provider"));
    assert.deepEqual(await f.call("setAgentDefaultModel", { model }), {
      ok: false,
      failure: {
        code: EDGE_HANDLER_FAILED,
        detail: "An explicit default-model provider must have a JSON value.",
      },
    });
    assert.deepEqual(f.bytes(), before);
  }
  assert.equal(f.hostWrites.length, 0);
  assert.deepEqual(f.reopen().getAgentDefaultModel(), legacyModel());
});

test("unavailable Host write or read leaves the last acknowledged default intact", async (t) => {
  const f = fixture(t);
  f.store.setAgentDefaultModel(boundModel());
  const before = f.bytes();
  for (const writeHost of [
    async () => null,
    async (): Promise<Settings> => { throw new Error("Host transport unavailable"); },
  ]) {
    f.setHostWrite(writeHost);
    const reply = await f.call("setAgentDefaultModel", { model: legacyModel() });
    assert.equal(reply.ok, false);
    if (!reply.ok) assert.equal(reply.failure.code, EDGE_HANDLER_FAILED);
    assert.deepEqual(f.bytes(), before);
  }
  f.setHostRead(async () => { throw new Error("Host read unavailable"); });
  const readReply = await f.call("getAgentDefaultModel");
  assert.deepEqual(readReply, {
    ok: false,
    failure: { code: EDGE_HANDLER_FAILED, detail: "Host read unavailable" },
  });
  assert.deepEqual(f.bytes(), before);
  assert.deepEqual(f.reopen().getAgentDefaultModel(), boundModel());
});

test("Host reads preserve metadata and acknowledged absent or null defaults clear the mirror", async (t) => {
  const f = fixture(t);
  const acknowledged = { ...boundModel(), metadata: { displayName: "Bound model" } };
  f.setHostRead(async () => ({ agentDefaultModel: acknowledged }));
  assert.deepEqual(await f.call("getAgentDefaultModel"), { ok: true, value: acknowledged });
  assert.deepEqual(f.reopen().getAgentDefaultModel(), acknowledged);

  for (const hostSettings of [{}, { agentDefaultModel: null }]) {
    f.store.setAgentDefaultModel(acknowledged);
    f.setHostRead(async () => hostSettings);
    assert.deepEqual(await f.call("getAgentDefaultModel"), { ok: true, value: null });
    assert.equal(f.reopen().getAgentDefaultModel(), undefined);
    assert.equal(Object.prototype.hasOwnProperty.call(JSON.parse(f.bytes().toString()), "agentDefaultModel"), false);

    f.store.setAgentDefaultModel(acknowledged);
    f.setHostWrite(async () => hostSettings);
    assert.deepEqual(await f.call("setAgentDefaultModel", { model: boundModel() }), { ok: true, value: null });
    assert.equal(f.reopen().getAgentDefaultModel(), undefined);
  }
});

test("legacy unbound defaults and malformed base-shape behavior stay compatible", async (t) => {
  const f = fixture(t);
  const legacy = { ...legacyModel(), modelId: "Legacy/Next@Case" };
  assert.deepEqual(await f.call("setAgentDefaultModel", { model: legacy }), { ok: true, value: legacy });
  assert.deepEqual(f.hostWrites, [{ agentDefaultModel: legacy }]);
  assert.equal(Object.prototype.hasOwnProperty.call(f.hostWrites[0]!.agentDefaultModel, "provider"), false);
  assert.deepEqual(f.reopen().getAgentDefaultModel(), legacy);
  const before = f.bytes();
  for (const model of [null, {}, { ...boundModel(), parameters: null }, { ...boundModel(), modelId: "" }]) {
    assert.deepEqual(await f.call("setAgentDefaultModel", { model }), { ok: true, value: legacy });
    assert.deepEqual(f.bytes(), before);
  }
  assert.equal(f.hostWrites.length, 1, "base-shape compatibility refusals do not dispatch");
});

test("existing fast migration retains provider and model metadata through reopening", (t) => {
  const f = fixture(t);
  for (const selection of [boundModel(), legacyModel()]) {
    const stored = {
      ...selection,
      maxMode: false,
      hostCatalogRevision: "catalog-7",
      metadata: { displayName: "Stored model" },
      parameters: [
        { id: "fast", value: "true", hostMetadata: "fast label" },
        { id: "thinking", value: "true", hostMetadata: "thinking label" },
      ],
    };
    f.store.persist({ ...emptySettings(), settingsMigrations: [], agentDefaultModel: stored });
    const expected = {
      ...stored,
      maxMode: true,
      parameters: [
        { id: "fast", value: "false", hostMetadata: "fast label" },
        { id: "thinking", value: "true", hostMetadata: "thinking label" },
      ],
    };
    assert.deepEqual(f.reopen().getAgentDefaultModel(), expected);
    const migrated = JSON.parse(f.bytes().toString());
    assert.deepEqual(migrated.agentDefaultModel, expected);
    assert.deepEqual(migrated.settingsMigrations, [SAND_DOWNGRADE_MAX_FAST_MIGRATION_ID]);
    const after = f.bytes();
    assert.deepEqual(f.reopen().getAgentDefaultModel(), expected);
    assert.deepEqual(f.bytes(), after, "a second load does not repeat or alter the migration");
  }
});

test("account transitions clear bound model mirrors without changing legacy retention", async (t) => {
  const f = fixture(t);
  f.store.scopeToAccount("account-a");
  assert.equal((await f.call("setAgentDefaultModel", { model: boundModel() })).ok, true);
  f.store.scopeToAccount("account-a");
  assert.deepEqual(f.reopen().getAgentDefaultModel(), boundModel());
  f.store.scopeToAccount("account-b");
  assert.equal(f.reopen().getAgentDefaultModel(), undefined);
  assert.equal((await f.call("setAgentDefaultModel", { model: boundModel() })).ok, true);
  f.store.clearAccountScope();
  assert.equal(f.reopen().getAgentDefaultModel(), undefined);
  assert.equal(f.reopen().getMcpCustomInstructionsAccountScope(), undefined);
});

test("Computer Use keeps its existing parser, override and clear behavior", async (t) => {
  const f = fixture(t);
  f.store.setAgentDefaultModel(boundModel());
  const requested = { modelId: "Computer/Own@Model", maxMode: false, parameters: [], provider: "fabushi" };
  const expected = { modelId: requested.modelId, maxMode: false, parameters: [] };
  assert.deepEqual(await f.call("setComputerUseModel", { model: requested }), { ok: true, value: expected });
  assert.deepEqual(f.hostWrites, [{ computerUseModel: expected }]);
  assert.deepEqual(f.reopen().getComputerUseModel(), expected);
  assert.deepEqual(await f.call("setComputerUseModel", { model: { ...requested, modelId: " " } }), { ok: true, value: expected });
  assert.equal(f.hostWrites.length, 1);
  const override = { modelId: "Computer/Override", maxMode: true, parameters: [] };
  f.setComputerUseOverride(override);
  assert.deepEqual(await f.call("getComputerUseModel"), { ok: true, value: override });
  f.setComputerUseOverride(undefined);
  assert.deepEqual(await f.call("setComputerUseModel", { model: null }), { ok: true, value: null });
  assert.equal(f.reopen().getComputerUseModel(), undefined);
  assert.deepEqual(f.hostWrites.at(-1), { computerUseModel: null });
  assert.deepEqual(f.reopen().getAgentDefaultModel(), boundModel());
});

test("real appWindow transport and served trust refuse foreign or nested-frame model requests", async (t) => {
  const f = fixture(t);
  const before = f.bytes();
  for (const event of [
    { sender: {}, senderFrame: f.trustedContents.mainFrame },
    { sender: f.trustedContents, senderFrame: {} },
  ]) {
    for (const method of ["setAgentDefaultModel", "getAgentDefaultModel"]) {
      const reply = await f.call(method, { model: boundModel() }, event);
      assert.equal(reply.ok, false);
      if (!reply.ok) assert.equal(reply.failure.code, EDGE_UNTRUSTED_SENDER);
    }
  }
  assert.equal(f.hostWrites.length, 0);
  assert.equal(f.hostReads, 0);
  assert.deepEqual(f.bytes(), before);
  assert.deepEqual(await f.call("setAgentDefaultModel", { model: boundModel() }), { ok: true, value: boundModel() });
  assert.equal(f.hostWrites.length, 1, "a legitimate top-frame call still reaches Host Settings");
});
