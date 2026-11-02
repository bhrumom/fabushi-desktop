import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
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

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((complete) => { resolve = complete; });
  return { promise, resolve };
}

function routerValue(reply: Reply): Settings {
  assert.ok(reply.ok, "the router RPC must succeed");
  assert.ok(typeof reply.value === "object" && reply.value != null && !Array.isArray(reply.value));
  return reply.value as Settings;
}

function assertRouterFailure(reply: Reply, detail?: string): void {
  assert.ok(!reply.ok, "the router RPC must report its failure");
  assert.equal(reply.failure.code, EDGE_HANDLER_FAILED);
  if (detail !== undefined) assert.equal(reply.failure.detail, detail);
}

test("router set waits for Host acknowledgement and mirrors its canonical provider", { timeout: 10_000 }, async (t) => {
  const f = fixture(t);
  f.store.setInferenceProvider("cursor");
  f.store.recordInferenceUsage("cursor", { inputTokens: 3 });
  const localUsage = f.store.getInferenceRouterUsage();
  const hostUsage = {
    ...localUsage,
    providers: { ...localUsage.providers, openrouter: { ...localUsage.providers.openrouter, requests: 7 } },
  };
  const entered = deferred<void>();
  const acknowledgement = deferred<Settings>();
  f.setHostWrite(async () => { entered.resolve(undefined); return await acknowledgement.promise; });
  const before = f.bytes();
  const pending = f.call("setInferenceRouter", { provider: "codex" });
  try {
    await entered.promise;
    assert.deepEqual(f.hostWrites, [{ inferenceProvider: "codex" }]);
    assert.deepEqual(f.bytes(), before, "pending Host IO cannot publish a speculative provider");
    assert.equal(f.reopen().getInferenceProvider(), "cursor");
    acknowledgement.resolve({ inferenceProvider: "openrouter", inferenceRouterUsage: hostUsage });
    const result = routerValue(await pending);
    assert.equal(result.provider, "openrouter", "the reply represents Host configuration, not the requested substitute");
    assert.deepEqual(result.usage, hostUsage, "Host usage takes precedence over the local diagnostic mirror");
    assert.equal(f.reopen().getInferenceProvider(), "openrouter");
    assert.deepEqual(f.reopen().getAgentDefaultModel(), legacyModel());
    assert.equal(f.hostWrites.length, 1);
  } finally {
    acknowledgement.resolve({ inferenceProvider: "openrouter", inferenceRouterUsage: hostUsage });
    await pending;
  }
});

test("router preserves all five configured providers and reconciles reads with diagnostic fallbacks", async (t) => {
  const f = fixture(t);
  f.store.recordInferenceUsage("cursor", { inputTokens: 11 });
  const usage = f.store.getInferenceRouterUsage();
  for (const provider of ["fabushi", "cursor", "claude-code", "codex", "openrouter"] as const) {
    const written = routerValue(await f.call("setInferenceRouter", { provider }));
    assert.equal(written.provider, provider);
    assert.deepEqual(f.hostWrites.at(-1), { inferenceProvider: provider });
    assert.equal(f.reopen().getInferenceProvider(), provider);
    assert.deepEqual(written.usage, usage, "an absent Host usage field retains the existing local fallback");
    const local = written.local as Record<string, Settings>;
    assert.deepEqual(Object.keys(local).sort(), ["claude-code", "codex"]);
    for (const status of Object.values(local)) {
      assert.equal(typeof status.installed, "boolean");
      assert.equal(typeof status.authenticated, "boolean");
      assert.ok(status.executablePath === null || typeof status.executablePath === "string");
    }

    f.store.setInferenceProvider(provider === "cursor" ? "codex" : "cursor");
    f.setHostRead(async () => ({ inferenceProvider: provider, inferenceRouterUsage: null }));
    const read = routerValue(await f.call("getInferenceRouter"));
    assert.equal(read.provider, provider, "fresh Host configuration replaces a differing local provider");
    assert.deepEqual(read.usage, usage, "a null Host usage field retains the existing local fallback");
    assert.equal(f.reopen().getInferenceProvider(), provider);
  }
  assert.equal(f.hostWrites.length, 5, "reads reconcile locally without replaying Host writes");
  assert.deepEqual(f.reopen().getAgentDefaultModel(), legacyModel());
});

test("router invalid requests and untrusted frames cause no Host or mirror effects", async (t) => {
  const f = fixture(t);
  const before = f.bytes();
  for (const provider of [undefined, null, false, 42, "", " ", "CODEX", "unknown-provider", [], {}]) {
    assertRouterFailure(await f.call("setInferenceRouter", { provider }), "Unknown inference provider.");
    assert.deepEqual(f.bytes(), before);
  }
  for (const event of [
    { sender: {}, senderFrame: f.trustedContents.mainFrame },
    { sender: f.trustedContents, senderFrame: {} },
  ]) {
    for (const method of ["getInferenceRouter", "setInferenceRouter"]) {
      const reply = await f.call(method, method === "setInferenceRouter" ? { provider: "codex" } : {}, event);
      assert.ok(!reply.ok);
      assert.equal(reply.failure.code, EDGE_UNTRUSTED_SENDER);
    }
  }
  assert.equal(f.hostWrites.length, 0);
  assert.equal(f.hostReads, 0);
  assert.deepEqual(f.bytes(), before);
  assert.equal(routerValue(await f.call("setInferenceRouter", { provider: "codex" })).provider, "codex");
  assert.equal(f.hostWrites.length, 1, "a legitimate call still reaches the same Host port");
});

test("router refuses malformed Host replies without local fallback and permits later reconciliation", async (t) => {
  const f = fixture(t);
  f.store.setInferenceProvider("openrouter");
  const malformedReplies: unknown[] = [
    null, undefined, false, 42, "codex", [], {},
    { inferenceProvider: undefined }, { inferenceProvider: null }, { inferenceProvider: false },
    { inferenceProvider: 42 }, { inferenceProvider: "" }, { inferenceProvider: "CODEX" },
    { inferenceProvider: "unknown-provider" }, Object.assign([], { inferenceProvider: "codex" }),
  ];
  for (const method of ["setInferenceRouter", "getInferenceRouter"]) {
    for (const malformed of malformedReplies) {
      const before = f.bytes();
      f.setHostWrite(async () => malformed as Settings | null);
      f.setHostRead(async () => malformed as Settings);
      assertRouterFailure(await f.call(method, method === "setInferenceRouter" ? { provider: "codex" } : {}));
      assert.deepEqual(f.bytes(), before, "a malformed acknowledgement cannot publish or clear the provider mirror");
      assert.equal(f.reopen().getInferenceProvider(), "openrouter");
      f.setHostRead(async () => ({ inferenceProvider: "openrouter" }));
      assert.equal(routerValue(await f.call("getInferenceRouter")).provider, "openrouter");
    }
  }
  assert.equal(f.hostWrites.length, malformedReplies.length, "invalid acknowledgements never trigger automatic retries");
});

test("router Host refusal releases a queued read and read rejection does not poison later calls", { timeout: 10_000 }, async (t) => {
  const f = fixture(t);
  f.store.setInferenceProvider("cursor");
  const before = f.bytes();
  const entered = deferred<void>();
  const refuse = deferred<void>();
  // This controlled refusal performs no commit. Transport-unknown outcomes have a separate oracle below.
  f.setHostWrite(async () => { entered.resolve(undefined); await refuse.promise; throw new Error("Host refused the provider update"); });
  f.setHostRead(async () => {
    assert.deepEqual(f.bytes(), before, "the refused write preserves mirror bytes before read reconciliation");
    return { inferenceProvider: "cursor" };
  });
  const write = f.call("setInferenceRouter", { provider: "codex" });
  let read: Promise<Reply> | undefined;
  try {
    await entered.promise;
    read = f.call("getInferenceRouter");
    await f.call("getComputerUseModel");
    assert.equal(f.hostReads, 0, "the next router read waits for the failed write's settlement");
    refuse.resolve(undefined);
    assertRouterFailure(await write, "Host refused the provider update");
    assert.equal(routerValue(await read).provider, "cursor");
    assert.equal(f.reopen().getInferenceProvider(), "cursor");
    assert.equal(f.hostWrites.length, 1);
    const beforeReadFailure = f.bytes();
    f.setHostRead(async () => { throw new Error("Host read unavailable"); });
    assertRouterFailure(await f.call("getInferenceRouter"), "Host read unavailable");
    assert.deepEqual(f.bytes(), beforeReadFailure);
    f.setHostRead(async () => ({ inferenceProvider: "fabushi" }));
    assert.equal(routerValue(await f.call("getInferenceRouter")).provider, "fabushi");
    assert.equal(f.reopen().getInferenceProvider(), "fabushi");
  } finally {
    refuse.resolve(undefined);
    await write;
    if (read !== undefined) await read;
  }
});

test("router unknown transport outcome can follow a Host commit and requires an explicit read", async (t) => {
  for (const outcome of ["null", "rejected"] as const) {
    const f = fixture(t);
    f.store.setInferenceProvider("cursor");
    const before = f.bytes();
    let hostProvider = "cursor";
    f.setHostWrite(async (update) => {
      hostProvider = update.inferenceProvider as string;
      if (outcome === "rejected") throw new Error("Host reply lost after commit");
      return null;
    });
    f.setHostRead(async () => ({ inferenceProvider: hostProvider }));
    assertRouterFailure(await f.call("setInferenceRouter", { provider: "codex" }));
    assert.equal(hostProvider, "codex", "unavailable acknowledgement must not be interpreted as Host rollback");
    assert.deepEqual(f.bytes(), before);
    assert.equal(f.reopen().getInferenceProvider(), "cursor");
    assert.equal(f.hostWrites.length, 1);
    assert.equal(f.hostReads, 0, "the failed write does not invent an automatic retry or recovery read");
    assert.equal(routerValue(await f.call("getInferenceRouter")).provider, "codex");
    assert.equal(f.reopen().getInferenceProvider(), "codex");
    assert.equal(f.hostWrites.length, 1, "explicit reconciliation never replays the unknown write");
  }
});

test("router local persistence failure leaves the Host commit intact and a later get repairs the mirror", async (t) => {
  const f = fixture(t);
  f.store.setInferenceProvider("cursor");
  const before = f.bytes();
  let hostProvider = "cursor";
  f.setHostWrite(async (update) => { hostProvider = update.inferenceProvider as string; return { inferenceProvider: hostProvider }; });
  f.setHostRead(async () => ({ inferenceProvider: hostProvider }));
  // Block the real temporary-file write, leaving the existing settings image readable.
  const blockedTemp = `${f.store.settingsPath}.${process.pid}.tmp`;
  mkdirSync(blockedTemp);
  try {
    assertRouterFailure(await f.call("setInferenceRouter", { provider: "codex" }));
    assert.equal(hostProvider, "codex");
    assert.equal(f.hostWrites.length, 1, "local failure cannot compensate or retry the acknowledged Host write");
    assert.deepEqual(f.bytes(), before);
    assert.equal(f.reopen().getInferenceProvider(), "cursor");
    assertRouterFailure(await f.call("getInferenceRouter"));
    assert.deepEqual(f.bytes(), before, "a failed read-side mirror commit is also observable");
  } finally {
    rmSync(blockedTemp, { recursive: true, force: true });
  }
  assert.equal(routerValue(await f.call("getInferenceRouter")).provider, "codex");
  assert.equal(f.reopen().getInferenceProvider(), "codex");
  assert.equal(f.hostWrites.length, 1);
  assert.deepEqual(f.reopen().getAgentDefaultModel(), legacyModel());
});

test("router orders a held get before set through mirror persistence", { timeout: 10_000 }, async (t) => {
  const f = fixture(t);
  const entered = deferred<void>();
  const acknowledgement = deferred<Settings>();
  f.setHostRead(async () => { entered.resolve(undefined); return await acknowledgement.promise; });
  f.setHostWrite(async (update) => {
    assert.equal(f.reopen().getInferenceProvider(), "openrouter", "the earlier read must finish its mirror commit before this Host write starts");
    return update;
  });
  const read = f.call("getInferenceRouter");
  let write: Promise<Reply> | undefined;
  try {
    await entered.promise;
    write = f.call("setInferenceRouter", { provider: "codex" });
    await f.call("getComputerUseModel");
    assert.equal(f.hostWrites.length, 0);
    acknowledgement.resolve({ inferenceProvider: "openrouter" });
    assert.equal(routerValue(await read).provider, "openrouter");
    assert.equal(routerValue(await write).provider, "codex");
    assert.deepEqual(f.hostWrites, [{ inferenceProvider: "codex" }]);
    assert.equal(f.reopen().getInferenceProvider(), "codex");
  } finally {
    acknowledgement.resolve({ inferenceProvider: "openrouter" });
    await read;
    if (write !== undefined) await write;
  }
});

test("router orders a held set before get through mirror persistence", { timeout: 10_000 }, async (t) => {
  const f = fixture(t);
  const entered = deferred<void>();
  const acknowledgement = deferred<Settings>();
  f.setHostWrite(async () => { entered.resolve(undefined); return await acknowledgement.promise; });
  f.setHostRead(async () => {
    assert.equal(f.reopen().getInferenceProvider(), "openrouter", "the acknowledged write must be mirrored before a later Host read starts");
    return { inferenceProvider: "fabushi" };
  });
  const write = f.call("setInferenceRouter", { provider: "codex" });
  let read: Promise<Reply> | undefined;
  try {
    await entered.promise;
    read = f.call("getInferenceRouter");
    await f.call("getComputerUseModel");
    assert.equal(f.hostReads, 0);
    acknowledgement.resolve({ inferenceProvider: "openrouter" });
    assert.equal(routerValue(await write).provider, "openrouter");
    assert.equal(routerValue(await read).provider, "fabushi");
    assert.equal(f.reopen().getInferenceProvider(), "fabushi");
    assert.equal(f.hostWrites.length, 1);
  } finally {
    acknowledgement.resolve({ inferenceProvider: "openrouter" });
    await write;
    if (read !== undefined) await read;
  }
});

test("router serializes two sets so a late acknowledgement cannot overwrite the later mirror", { timeout: 10_000 }, async (t) => {
  const f = fixture(t);
  const entered = deferred<void>();
  const acknowledgement = deferred<Settings>();
  let writes = 0;
  f.setHostWrite(async () => {
    writes += 1;
    if (writes === 1) { entered.resolve(undefined); return await acknowledgement.promise; }
    assert.equal(f.reopen().getInferenceProvider(), "openrouter", "the second write starts after the first acknowledgement is mirrored");
    return { inferenceProvider: "fabushi" };
  });
  const first = f.call("setInferenceRouter", { provider: "codex" });
  let second: Promise<Reply> | undefined;
  try {
    await entered.promise;
    second = f.call("setInferenceRouter", { provider: "claude-code" });
    await f.call("getComputerUseModel");
    assert.equal(writes, 1);
    acknowledgement.resolve({ inferenceProvider: "openrouter" });
    assert.equal(routerValue(await first).provider, "openrouter");
    assert.equal(routerValue(await second).provider, "fabushi");
    assert.deepEqual(f.hostWrites, [{ inferenceProvider: "codex" }, { inferenceProvider: "claude-code" }]);
    assert.equal(f.reopen().getInferenceProvider(), "fabushi");
  } finally {
    acknowledgement.resolve({ inferenceProvider: "openrouter" });
    await first;
    if (second !== undefined) await second;
  }
});

test("a pending router IPC does not order unrelated methods or another MainEdge instance", { timeout: 10_000 }, async (t) => {
  const first = fixture(t);
  const second = fixture(t);
  const entered = deferred<void>();
  const acknowledgement = deferred<Settings>();
  first.setHostWrite(async () => { entered.resolve(undefined); return await acknowledgement.promise; });
  const pending = first.call("setInferenceRouter", { provider: "codex" });
  try {
    await entered.promise;
    assert.deepEqual(await first.call("getComputerUseModel"), { ok: true, value: null });
    assert.equal(routerValue(await second.call("setInferenceRouter", { provider: "openrouter" })).provider, "openrouter");
    assert.equal(second.reopen().getInferenceProvider(), "openrouter");
    assert.equal(first.reopen().getInferenceProvider(), "cursor", "the first instance is still waiting for its own acknowledgement");
    acknowledgement.resolve({ inferenceProvider: "codex" });
    assert.equal(routerValue(await pending).provider, "codex");
    assert.equal(first.reopen().getInferenceProvider(), "codex");
  } finally {
    acknowledgement.resolve({ inferenceProvider: "codex" });
    await pending;
  }
});
