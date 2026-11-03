import assert from "node:assert/strict";
import { promises as fsPromises } from "node:fs";
import { lstat, mkdir, mkdtemp, readFile, rename, rm, symlink, writeFile } from "node:fs/promises";
import { syncBuiltinESMExports } from "node:module";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";
import test, { type TestContext } from "node:test";

import {
  createBackendMarketplaceClient,
  type BackendPluginEntry,
} from "../../packages/cursor-plugins/backend-marketplace-client.js";
import { DefaultPluginCacheManager } from "../../packages/cursor-plugins/cursor-marketplace.js";
import { buildOriginTokenGitConfig } from "../../packages/cursor-plugins/origin-git-auth.js";
import type { FabushiSession } from "../../product/fabushi/fabushi-account-policy.js";
import { FabushiAuthService } from "../../product/fabushi/fabushi-account-service.js";
import { getSandInferenceBackendUrl } from "../../shared/node/cursor-backend/cursor-inference.js";
import { COORDINATOR_PROTOCOL_VERSION, type CoordinatorFrame } from "../../shared/rpc/coordinator-port.js";
import {
  ACCESS_TOKEN_SECRET_KEY,
  REFRESH_TOKEN_SECRET_KEY,
  SandCursorAuthService,
} from "../account/cursor-auth.js";
import { createCoordinatorControlServer } from "../coordinator/coordinator-control-server.js";
import {
  createHostPluginSkillsLoader,
  HOST_PLUGIN_SKILLS_CURRENT_USER_TIMEOUT_MS,
  HOST_PLUGIN_SKILLS_EFFECTIVE_PLUGINS_TIMEOUT_MS,
  type HostPluginSkillsDashboardClient,
  type HostPluginSkillsLoadResult,
  type HostPluginSkillsLoaderDeps,
} from "./host-plugin-skills-loader.js";

type ClientFactory = NonNullable<HostPluginSkillsLoaderDeps["createMarketplaceClient"]>;
type IdentityFixture = {
  readonly getAccessToken?: HostPluginSkillsLoaderDeps["getAccessToken"];
  readonly peekAccessToken?: HostPluginSkillsLoaderDeps["peekAccessToken"];
  readonly getCurrentUser?: (accessToken: string) => ReturnType<HostPluginSkillsDashboardClient["getMe"]>;
  readonly log?: (message: string) => void;
};

async function fixture(
  t: TestContext,
  response: (accessToken: string) => Promise<unknown>,
  createMarketplaceClient?: ClientFactory,
  identity: IdentityFixture = {},
  rootComponent?: string,
) {
  const fixtureRoot = await mkdtemp(join(tmpdir(), "fabushi-host-plugin-skills-"));
  const sandRootDir = rootComponent === undefined ? fixtureRoot : join(fixtureRoot, rootComponent);
  t.after(async () => await rm(fixtureRoot, { recursive: true, force: true }));
  let listings = 0;
  let currentUserLookups = 0;
  const load = createHostPluginSkillsLoader({
    sandRootDir,
    getAccessToken: identity.getAccessToken ?? (async () => "fixture-token"),
    getMachineId: () => "fixture-machine",
    peekAccessToken: identity.peekAccessToken ?? (async () => "fixture-token"),
    isSparsePluginClonesEnabled: () => false,
    log: identity.log ?? (() => {}),
    createDashboardClient: (credentials) => ({
      async getEffectiveUserPlugins(request, options) {
        assert.deepEqual(request, {});
        assert.equal(options.timeoutMs, HOST_PLUGIN_SKILLS_EFFECTIVE_PLUGINS_TIMEOUT_MS);
        listings += 1;
        return await response(await credentials.getAccessToken({ backendUrl: getSandInferenceBackendUrl() }));
      },
      async getMe(request, options) {
        assert.deepEqual(request, {});
        assert.equal(options.timeoutMs, HOST_PLUGIN_SKILLS_CURRENT_USER_TIMEOUT_MS);
        currentUserLookups += 1;
        const accessToken = await credentials.getAccessToken({ backendUrl: getSandInferenceBackendUrl() });
        return identity.getCurrentUser == null ? { userId: 8n } : await identity.getCurrentUser(accessToken);
      },
    }),
    ...(createMarketplaceClient === undefined ? {} : { createMarketplaceClient }),
  });
  return {
    load,
    cache: new DefaultPluginCacheManager(undefined, { cacheRoot: join(sandRootDir, "plugins", "cache") }),
    calls: () => ({ listings, currentUserLookups }),
  };
}

function effectivePlugin(id: unknown, name: string, inlineContentJson?: string) {
  return {
    isEnabled: true,
    plugin: {
      id, name, updatedAt: 1,
      publisher: { ownerUserId: 8n },
      marketplace: { id: 3n, name: "cursor-public", teamId: 5n },
    },
    ...(inlineContentJson === undefined ? {} : { inlineContentJson }),
  };
}

async function seedOldVersion(cache: DefaultPluginCacheManager, pluginId: string): Promise<string> {
  const key = { marketplaceSlug: "cursor-public", pluginId, version: "previous-version" };
  const directory = cache.getCacheDir(key);
  await mkdir(directory, { recursive: true });
  const skillPath = join(directory, "SKILL.md");
  await writeFile(skillPath, "Previous committed skill bytes.\n");
  await cache.markCacheComplete(key);
  return skillPath;
}

function withInstall(
  install: (entry: BackendPluginEntry, targetDir: string, actual: () => Promise<void>) => Promise<void>,
): ClientFactory {
  return (getEffectiveUserPlugins, options) => {
    const client = createBackendMarketplaceClient(getEffectiveUserPlugins, options);
    return {
      listEnabledPlugins: (userId, teamId) => client.listEnabledPlugins(userId, teamId),
      installPlugin: (entry, targetDir) => install(entry, targetDir, () => client.installPlugin(entry, targetDir)),
    };
  };
}

test("Host rejects an unavailable SDK source but returns an authoritative empty snapshot", async (t) => {
  let unavailable = true;
  const host = await fixture(t, async () => {
    if (unavailable) throw new Error("effective listing offline");
    return { plugins: [] };
  });
  const oldSkill = await seedOldVersion(host.cache, "installed");
  const oldBytes = await readFile(oldSkill, "utf8");

  await assert.rejects(host.load(), /Plugin Skills marketplace source is unavailable/);
  assert.deepEqual(host.calls(), { listings: 1, currentUserLookups: 0 });
  assert.equal(await readFile(oldSkill, "utf8"), oldBytes);

  unavailable = false;
  assert.deepEqual(await host.load(), {
    plugins: [], authBlocked: [], listedPluginIds: [], listedCacheKeys: [],
    publisherFacts: {}, currentUserId: 8,
  });
  assert.deepEqual(host.calls(), { listings: 2, currentUserLookups: 1 });
  assert.equal(await readFile(oldSkill, "utf8"), oldBytes, "only Rust may prune after committing this empty snapshot");
});

test("Host rejects malformed raw listing shapes before materializing any entry", async (t) => {
  let response: unknown;
  let installs = 0;
  const host = await fixture(t, async () => response, withInstall(async (_entry, _targetDir, actual) => {
    installs += 1;
    await actual();
  }));
  const valid = effectivePlugin("valid", "valid", "{}");
  const malformed: ReadonlyArray<readonly [string, unknown]> = [
    ["missing response", undefined],
    ["null response", null],
    ["array response", []],
    ["missing plugins", {}],
    ["null plugins", { plugins: null }],
    ["non-array plugins", { plugins: {} }],
    ["non-array marketplaces", { plugins: [], marketplaces: {} }],
    ["null entry", { plugins: [valid, null] }],
    ["array entry", { plugins: [valid, []] }],
    ["missing enabled state", { plugins: [valid, { plugin: { id: "other" } }] }],
    ["non-boolean enabled state", { plugins: [valid, { isEnabled: "true", plugin: { id: "other" } }] }],
    ["missing enabled plugin", { plugins: [valid, { isEnabled: true }] }],
    ["array enabled plugin", { plugins: [valid, { isEnabled: true, plugin: [] }] }],
  ];
  for (const [label, value] of malformed) {
    response = value;
    await assert.rejects(host.load(), /source is unavailable/, label);
  }
  assert.equal(installs, 0);
  assert.equal(host.calls().currentUserLookups, 0);
});

test("Host rejects malformed or duplicate enabled IDs before SDK filtering", async (t) => {
  let response: unknown;
  let installs = 0;
  const host = await fixture(t, async () => response, withInstall(async (_entry, _targetDir, actual) => {
    installs += 1;
    await actual();
  }));
  const malformedIds: ReadonlyArray<readonly [string, unknown]> = [
    ["missing", undefined], ["null", null], ["empty", ""], ["space only", " "],
    ["leading whitespace", " db"], ["trailing whitespace", "db\t"],
    ["zero", 0], ["negative", -1], ["fraction", 1.5], ["NaN", Number.NaN],
    ["infinity", Number.POSITIVE_INFINITY], ["unsafe number", Number.MAX_SAFE_INTEGER + 1],
    ["zero bigint", 0n], ["negative bigint", -1n], ["object", {}], ["array", ["db"]],
  ];
  for (const [label, id] of malformedIds) {
    response = { plugins: [effectivePlugin("valid", "valid", "{}"), effectivePlugin(id, "invalid")] };
    await assert.rejects(host.load(), /source is unavailable/, label);
  }
  for (const duplicate of ["41", 41, 41n]) {
    response = { plugins: [effectivePlugin(41, "first", "{}"), effectivePlugin(duplicate, "second", "{}")] };
    await assert.rejects(host.load(), /source is unavailable/, "duplicate canonical enabled identity");
  }
  assert.equal(installs, 0);
  assert.equal(host.calls().currentUserLookups, 0);
});

test("Host captures enabled IDs before the real SDK omits inline content or nameless entries", async (t) => {
  const nameless = effectivePlugin("42", "");
  const host = await fixture(t, async () => ({
    plugins: [
      effectivePlugin(41, "unmaterialized"),
      nameless,
      effectivePlugin(43n, "materialized", "{}"),
      { isEnabled: false },
      { isEnabled: false, plugin: { id: "disabled" } },
    ],
  }));
  const result = await host.load();

  assert.deepEqual(result.listedPluginIds, ["41", "42", "43"]);
  assert.equal(result.plugins.length, 1);
  assert.equal(result.plugins[0]?.identifier.pluginDbId, "43");
  assert.deepEqual(result.plugins[0]?.skills, []);
  assert.equal(result.plugins[0]?.loadError, undefined);
  assert.deepEqual(result.listedCacheKeys, [{ marketplaceSlug: "cursor-public", pluginId: "materialized" }]);
  assert.deepEqual(result.authBlocked, []);
  assert.deepEqual(result.publisherFacts["41"], { publisherUserId: 8, marketplaceTeamId: 5 });
  assert.equal(result.publisherFacts["disabled"], undefined);
});

test("Host returns the actual SDK install path for a zero-skill plugin with dotted cache keys", async (t) => {
  const commandBytes = "Review the current request without changing unrelated files.";
  const entry = effectivePlugin("db-zero-skill", "review.tools", JSON.stringify({
    commands: [{ name: "review", content: commandBytes, isActive: true }],
  }));
  entry.plugin.marketplace.name = "market.place";
  const host = await fixture(t, async () => ({ plugins: [entry] }));
  const result = await host.load();

  assert.deepEqual(result.listedPluginIds, ["db-zero-skill"]);
  assert.deepEqual(result.listedCacheKeys, [
    { marketplaceSlug: "market.place", pluginId: "review.tools" },
  ]);
  assert.deepEqual(result.authBlocked, []);
  assert.equal(result.plugins.length, 1);
  const installed = result.plugins[0]!;
  assert.equal(installed.identifier.pluginDbId, "db-zero-skill");
  assert.ok(installed.identifier.version);
  assert.equal(installed.loadError, undefined);
  assert.deepEqual(installed.skills, []);
  assert.equal(
    dirname(installed.installPath),
    join(host.cache.cacheRoot, "market_place", "review_tools"),
    "the materializer's real directories differ from raw listing keys",
  );
  assert.equal(installed.installPath, host.cache.getCacheDir({
    marketplaceSlug: "market.place", pluginId: "review.tools", version: installed.identifier.version,
  }));
  assert.equal(await readFile(join(installed.installPath, "commands", "review.md"), "utf8"), commandBytes);
  assert.equal(await readFile(join(installed.installPath, ".cache-complete"), "utf8"), "");
});

test("Host late failed materialization preserves a zero-skill winner and a fresh cache hit", { timeout: 10_000 }, async (t) => {
  const commandBytes = "Review the completed plugin without changing its files.";
  const entry = effectivePlugin("db-zero-custody", "review.tools", JSON.stringify({
    commands: [{ name: "review", content: commandBytes, isActive: true }],
  }));
  entry.plugin.marketplace.name = "market.place";
  const entered = Promise.withResolvers<void>();
  const release = Promise.withResolvers<void>();
  const stages: string[] = [];
  let installs = 0;
  const host = await fixture(t, async () => ({ plugins: [entry] }), withInstall(async (_entry, stage, actual) => {
    stages.push(stage);
    installs += 1;
    await actual();
    if (installs === 1) {
      await writeFile(join(stage, "commands", "review.md"), "older uncommitted command");
      entered.resolve();
      await release.promise;
      throw new Error("late Host install failure after caller retirement");
    }
  }));
  const oldSkill = await seedOldVersion(host.cache, "retained");
  type Reply = Extract<CoordinatorFrame, { kind: "reply" }>;
  const pending = new Map<string, (reply: Reply) => void>();
  const problems: string[] = [];
  let olderReplied = false;
  // The shipping control server dispatches B while A's remote executor still
  // runs. The Rust timeout/retired-waiter oracle remains a separate control.
  const server = createCoordinatorControlServer({
    post(frame) {
      if (frame.kind !== "reply") return;
      if (frame.requestId === "custody-older") olderReplied = true;
      const receive = pending.get(frame.requestId);
      assert.ok(receive, `unexpected control reply ${frame.requestId}`);
      receive(frame);
    },
    executors: { loadHostPluginSkills: () => host.load() },
    onEvent: {},
    onProblem: (detail) => { problems.push(detail); },
  });
  t.after(() => server.dispose());
  server.handleMessage({ kind: "lifecycle", phase: "hello", protocolVersion: COORDINATOR_PROTOCOL_VERSION });
  const request = (requestId: string): Promise<HostPluginSkillsLoadResult> => {
    const reply = Promise.withResolvers<Reply>();
    pending.set(requestId, reply.resolve);
    server.handleMessage({ kind: "request", requestId, method: "loadHostPluginSkills", args: {} });
    return reply.promise.then((frame) => {
      assert.ok(frame.outcome.status === "ok", "the actual Host executor must return per-entry failures as snapshot metadata");
      return frame.outcome.value as HostPluginSkillsLoadResult;
    });
  };
  const older = request("custody-older");
  try {
    await Promise.race([
      entered.promise,
      older.then(() => { throw new Error("The first Host materialization must remain held."); }),
    ]);
    const winner = await request("custody-winner");
    assert.equal(olderReplied, false, "the real control server must not serialize independent materializers");
    assert.deepEqual(winner.listedPluginIds, ["db-zero-custody"]);
    assert.deepEqual(winner.listedCacheKeys, [{ marketplaceSlug: "market.place", pluginId: "review.tools" }]);
    assert.deepEqual(winner.authBlocked, []);
    assert.equal(winner.plugins.length, 1);
    const installed = winner.plugins[0]!;
    assert.equal(installed.identifier.pluginDbId, "db-zero-custody");
    assert.ok(installed.identifier.version);
    assert.equal(installed.loadError, undefined);
    assert.deepEqual(installed.skills, []);
    const key = { marketplaceSlug: "market.place", pluginId: "review.tools", version: installed.identifier.version };
    assert.equal(installed.installPath, host.cache.getCacheDir(key));
    assert.equal(dirname(installed.installPath), join(host.cache.cacheRoot, "market_place", "review_tools"));
    const manifestBytes = JSON.stringify({ name: "review.tools", commands: ["commands/review.md"] }, null, 2);
    const assertWinnerBytes = async () => {
      assert.equal(await readFile(join(installed.installPath, "commands", "review.md"), "utf8"), commandBytes);
      assert.equal(await readFile(join(installed.installPath, ".cursor-plugin", "plugin.json"), "utf8"), manifestBytes);
      assert.equal(await readFile(join(installed.installPath, ".cache-complete"), "utf8"), "");
      assert.equal(await host.cache.isCached(key), true);
    };
    await assertWinnerBytes();
    assert.equal(stages.length, 2);
    assert.notEqual(stages[0], stages[1]);
    for (const stage of stages) assert.equal(dirname(stage), dirname(host.cache.cacheRoot));

    release.resolve();
    const failed = await older;
    assert.deepEqual(failed.listedPluginIds, ["db-zero-custody"]);
    assert.deepEqual(failed.authBlocked, []);
    assert.equal(failed.plugins[0]?.identifier.pluginDbId, "db-zero-custody");
    assert.match(failed.plugins[0]?.loadError ?? "", /late Host install failure/);
    assert.equal(failed.plugins[0]?.installPath, "");
    assert.deepEqual(failed.plugins[0]?.skills, []);
    await assertWinnerBytes();
    for (const stage of stages) await assert.rejects(lstat(stage), { code: "ENOENT" });

    const fresh = await request("custody-fresh");
    assert.deepEqual(fresh.listedPluginIds, ["db-zero-custody"]);
    assert.deepEqual(fresh.authBlocked, []);
    assert.equal(fresh.plugins[0]?.identifier.pluginDbId, "db-zero-custody");
    assert.equal(fresh.plugins[0]?.installPath, installed.installPath);
    assert.equal(fresh.plugins[0]?.loadError, undefined);
    assert.deepEqual(fresh.plugins[0]?.skills, []);
    await assertWinnerBytes();
    assert.equal(installs, 2, "a fresh actual Host call must hit the completed SDK cache");
    assert.equal(await readFile(oldSkill, "utf8"), "Previous committed skill bytes.\n");
    assert.deepEqual(problems, []);
  } finally {
    release.resolve();
    await older;
  }
});

test("Host preserves opaque strings and bigint IDs above the safe-number range exactly", async (t) => {
  const ids = ["01", "Db/Identity: A", "case", "CASE", 1, Number.MAX_SAFE_INTEGER, 18_446_744_073_709_551_615n];
  const host = await fixture(t, async () => ({
    plugins: ids.map((id, index) => effectivePlugin(id, `unmaterialized-${index}`)),
  }));
  const result = await host.load();

  assert.deepEqual(result.listedPluginIds, [
    "01", "Db/Identity: A", "case", "CASE", "1", "9007199254740991", "18446744073709551615",
  ]);
  assert.deepEqual(result.plugins, []);
  assert.deepEqual(result.authBlocked, []);
  assert.deepEqual(result.publisherFacts["18446744073709551615"], { publisherUserId: 8, marketplaceTeamId: 5 });
});

test("local install permission failures never become remote authorization blocks", async (t) => {
  const host = await fixture(t, async () => ({
    plugins: [effectivePlugin("41", "7", "{}"), effectivePlugin("7", "healthy", "{}")],
  }), withInstall(async (entry, _targetDir, actual) => {
    if (entry.pluginDbId === "41") throw new Error("Permission denied (publickey).");
    await actual();
  }));
  const deniedOldSkill = await seedOldVersion(host.cache, "7");
  const healthyOldSkill = await seedOldVersion(host.cache, "healthy");
  const result = await host.load();

  assert.deepEqual(result.listedPluginIds, ["41", "7"]);
  assert.deepEqual(result.authBlocked, [], "install/cache permission errors are not remote Git authorization decisions");
  assert.equal(result.plugins[0]?.identifier.pluginDbId, "41");
  assert.match(result.plugins[0]?.loadError ?? "", /Permission denied/);
  assert.equal(result.plugins[1]?.identifier.pluginDbId, "7");
  assert.equal(result.plugins[1]?.loadError, undefined);
  assert.deepEqual(result.plugins[1]?.skills, []);
  assert.equal(await readFile(deniedOldSkill, "utf8"), "Previous committed skill bytes.\n");
  assert.equal(await readFile(healthyOldSkill, "utf8"), "Previous committed skill bytes.\n");
});

test("Host authorization failures use the canonical DB ID when the slug matches another plugin's DB ID", async (t) => {
  const host = await fixture(t, async () => ({
    plugins: [effectivePlugin("41", "7", "{}"), effectivePlugin("7", "healthy", "{}")],
  }), withInstall(async (entry, _targetDir, actual) => {
    if (entry.pluginDbId === "41") throw new Error("git clone failed: Permission denied (publickey).");
    await actual();
  }));
  const deniedOldSkill = await seedOldVersion(host.cache, "7");
  const healthyOldSkill = await seedOldVersion(host.cache, "healthy");
  const result = await host.load();

  assert.deepEqual(result.listedPluginIds, ["41", "7"]);
  assert.deepEqual(result.authBlocked, [{ pluginId: "41", pluginName: "7", marketplaceName: "cursor-public" }]);
  assert.equal(result.plugins[0]?.identifier.pluginDbId, "41");
  assert.match(result.plugins[0]?.loadError ?? "", /git clone failed: Permission denied/);
  assert.equal(result.plugins[1]?.identifier.pluginDbId, "7");
  assert.equal(result.plugins[1]?.loadError, undefined);
  assert.deepEqual(result.plugins[1]?.skills, []);
  assert.equal(await readFile(deniedOldSkill, "utf8"), "Previous committed skill bytes.\n");
  assert.equal(await readFile(healthyOldSkill, "utf8"), "Previous committed skill bytes.\n");
});

test("Host staging cleanup failure preserves the canonical authorization denial", async (t) => {
  for (const authDenied of [true, false]) {
    const installMessage = authDenied ? "git clone failed: Permission denied (publickey)." : "remote endpoint disconnected";
    const logs: string[] = [];
    let stagingDir = "";
    let retainedParent = "";
    const host = await fixture(t, async () => ({
      plugins: [effectivePlugin("41", "7", "{}")],
    }), withInstall(async (_entry, stage, actual) => {
      stagingDir = stage;
      await actual();
      // Only this fixture's exclusive plugins parent moves. Making its old name
      // a file forces rm(stage) to fail with ENOTDIR even under privileged CI.
      const stagingParent = dirname(stage);
      retainedParent = `${stagingParent}-retained`;
      await rename(stagingParent, retainedParent);
      await writeFile(stagingParent, "owned fixture parent is unavailable");
      throw new Error(installMessage);
    }), { log: (message) => { logs.push(message); } }, authDenied ? undefined : "permission denied");
    await seedOldVersion(host.cache, "7");
    const result = await host.load();

    assert.deepEqual(result.listedPluginIds, ["41"]);
    assert.deepEqual(result.authBlocked, authDenied
      ? [{ pluginId: "41", pluginName: "7", marketplaceName: "cursor-public" }]
      : [], "a cleanup error/path cannot create or hide a canonical Git-auth denial");
    assert.equal(result.plugins.length, 1);
    const failed = result.plugins[0]!;
    assert.equal(failed.identifier.pluginDbId, "41");
    assert.equal(failed.loadError?.split("\n")[0], installMessage);
    assert.match(failed.loadError ?? "", /Plugin cache staging cleanup failed; residual state unknown/);
    assert.match(failed.loadError ?? "", /ENOTDIR/);
    assert.doesNotMatch(failed.loadError ?? "", /\.plugin-cache-install-/);
    if (!authDenied) {
      assert.match(stagingDir, /permission denied/);
      assert.doesNotMatch(failed.loadError ?? "", /permission denied/i);
    }
    assert.equal(failed.installPath, "");
    assert.deepEqual(failed.skills, []);
    assert.equal(dirname(stagingDir), dirname(host.cache.cacheRoot));
    assert.equal(await readFile(dirname(stagingDir), "utf8"), "owned fixture parent is unavailable");
    await assert.rejects(lstat(stagingDir), { code: "ENOTDIR" });
    const retainedStage = join(retainedParent, basename(stagingDir));
    assert.equal(await readFile(join(retainedStage, ".cursor-plugin", "plugin.json"), "utf8"), JSON.stringify({ name: "7" }, null, 2));
    await assert.rejects(lstat(join(retainedStage, ".cache-complete")), { code: "ENOENT" });
    assert.equal(await host.cache.isCached({ marketplaceSlug: "cursor-public", pluginId: "7", version: failed.identifier.version }), false);
    assert.equal(await readFile(join(retainedParent, "cache", "cursor-public", "7", "previous-version", "SKILL.md"), "utf8"), "Previous committed skill bytes.\n");
    assert.ok(logs.some((message) => message.includes(installMessage) && message.includes("residual state unknown")));
  }
});

// Controlled negatives exercise actual Host consumption of cache failures;
// only removal of the exact owned stage receives an injected I/O fault.
test("Host cleanup I/O failures preserve canonical remote denial and recovery", async (t) => {
  for (const cleanupCode of ["ETIMEDOUT", "EACCES"] as const) {
    for (const authDenied of [true, false]) {
      const primaryError = new Error(authDenied
        ? "git clone failed: Permission denied (publickey)."
        : "remote endpoint disconnected");
      const cleanupError = Object.assign(new Error("controlled cleanup failure"), { code: cleanupCode });
      let stagingDir = "";
      let installs = 0;
      let cleanupCalls = 0;
      const host = await fixture(t, async () => ({
        plugins: [effectivePlugin("41", "7", "{}")],
      }), withInstall(async (_entry, stage, actual) => {
        installs += 1;
        stagingDir = stage;
        await actual();
        if (installs === 1) throw primaryError;
      }));
      const oldSkill = await seedOldVersion(host.cache, "7");
      const originalRm = fsPromises.rm;
      const rmMock = t.mock.method(fsPromises, "rm", async (...args: Parameters<typeof originalRm>): Promise<void> => {
        if (stagingDir !== "" && args[0] === stagingDir) {
          cleanupCalls += 1;
          throw cleanupError;
        }
        return originalRm(...args);
      });
      let retainedStage = "";
      try {
        syncBuiltinESMExports();
        const result = await host.load();
        retainedStage = stagingDir;
        assert.notEqual(retainedStage, "");
        assert.equal(cleanupCalls, 1);
        assert.deepEqual(result.listedPluginIds, ["41"]);
        assert.deepEqual(result.authBlocked, authDenied
          ? [{ pluginId: "41", pluginName: "7", marketplaceName: "cursor-public" }]
          : [], cleanupCode);
        assert.equal(result.plugins.length, 1);
        const failed = result.plugins[0]!;
        assert.equal(failed.identifier.pluginDbId, "41");
        assert.equal(failed.installPath, "");
        assert.deepEqual(failed.skills, []);
        assert.equal(failed.loadError?.split("\n")[0], primaryError.message);
        assert.match(failed.loadError ?? "", /Plugin cache staging cleanup failed; residual state unknown/);
        assert.ok(failed.loadError?.includes(`I/O code: ${cleanupCode}`));
        assert.equal(await readFile(join(retainedStage, ".cursor-plugin", "plugin.json"), "utf8"), JSON.stringify({ name: "7" }, null, 2));
        await assert.rejects(lstat(join(retainedStage, ".cache-complete")), { code: "ENOENT" });
        assert.equal(await host.cache.isCached({ marketplaceSlug: "cursor-public", pluginId: "7", version: failed.identifier.version }), false);
        assert.equal(await readFile(oldSkill, "utf8"), "Previous committed skill bytes.\n");
      } finally {
        rmMock.mock.restore();
        syncBuiltinESMExports();
      }
      await originalRm(retainedStage, { recursive: true, force: true });
      const recovered = await host.load();
      assert.deepEqual(recovered.authBlocked, []);
      assert.equal(recovered.plugins.length, 1);
      const installed = recovered.plugins[0]!;
      assert.equal(installed.identifier.pluginDbId, "41");
      assert.equal(installed.loadError, undefined);
      assert.notEqual(installed.installPath, "");
      assert.equal(await readFile(join(installed.installPath, ".cache-complete"), "utf8"), "");
      assert.equal(installs, 2);
      assert.equal(await readFile(oldSkill, "utf8"), "Previous committed skill bytes.\n");
    }
  }
});

test("Host cache-owned materialization failures cannot create authorization denials", async (t) => {
  for (const conflict of ["initial", "during-install", "reserved-marker"] as const) {
    const protectedBytes = "protected cache collision bytes";
    let cache!: DefaultPluginCacheManager;
    let targetDir = "";
    let stagingDir = "";
    let protectedPath = "";
    let installs = 0;
    const host = await fixture(t, async () => ({
      plugins: [effectivePlugin("41", "7", "{}")],
    }), (getEffectiveUserPlugins, options) => {
      const client = createBackendMarketplaceClient(getEffectiveUserPlugins, options);
      return {
        async listEnabledPlugins(userId, teamId) {
          const listed = await client.listEnabledPlugins(userId, teamId);
          const plugin = listed.plugins[0];
          assert.ok(plugin);
          assert.equal(plugin.pluginDbId, "41");
          targetDir = cache.getCacheDir({
            marketplaceSlug: plugin.marketplace!.name, pluginId: plugin.pluginId, version: plugin.version,
          });
          if (conflict === "initial") {
            await mkdir(targetDir, { recursive: true });
            protectedPath = join(targetDir, "foreign.txt");
            await writeFile(protectedPath, protectedBytes);
          }
          return listed;
        },
        async installPlugin(plugin, stage) {
          installs += 1;
          stagingDir = stage;
          await client.installPlugin(plugin, stage);
          if (conflict === "during-install") {
            await mkdir(targetDir, { recursive: true });
            protectedPath = join(targetDir, "foreign.txt");
            await writeFile(protectedPath, protectedBytes);
          } else if (conflict === "reserved-marker") {
            // Exclusive marker creation must reject the reserved name without
            // replacing this foreign file or turning its path into a denial.
            protectedPath = join(dirname(cache.cacheRoot), "protected-marker.txt");
            await writeFile(protectedPath, protectedBytes);
            await symlink(protectedPath, join(stage, ".cache-complete"), "file");
          }
        },
      };
    }, {}, "git permission denied");
    cache = host.cache;
    const oldSkill = await seedOldVersion(cache, "7");
    const result = await host.load();

    assert.match(targetDir, /permission denied/);
    assert.equal(installs, conflict === "initial" ? 0 : 1, conflict);
    assert.deepEqual(result.listedPluginIds, ["41"]);
    assert.deepEqual(result.authBlocked, [], conflict);
    assert.equal(result.plugins.length, 1);
    const failed = result.plugins[0]!;
    assert.equal(failed.identifier.pluginDbId, "41");
    assert.ok(failed.identifier.version);
    assert.equal(failed.installPath, "");
    assert.deepEqual(failed.skills, []);
    assert.match(failed.loadError ?? "", conflict === "reserved-marker"
      ? /Plugin cache completion marker creation failed \(I\/O code: EEXIST\)/
      : /Plugin cache target is incomplete/);
    assert.doesNotMatch(failed.loadError ?? "", /permission denied/i);
    assert.equal(await readFile(protectedPath, "utf8"), protectedBytes);
    assert.equal(await readFile(oldSkill, "utf8"), "Previous committed skill bytes.\n");
    assert.equal(await readFile(join(dirname(oldSkill), ".cache-complete"), "utf8"), "");
    assert.equal(await cache.isCached({ marketplaceSlug: "cursor-public", pluginId: "7", version: failed.identifier.version }), false);
    await assert.rejects(lstat(join(targetDir, ".cache-complete")), { code: "ENOENT" });
    if (stagingDir !== "") await assert.rejects(lstat(stagingDir), { code: "ENOENT" });
    if (conflict === "reserved-marker") await assert.rejects(lstat(targetDir), { code: "ENOENT" });
  }
});

test("Host refuses an SDK auth denial without canonical identity instead of guessing from a slug or name", async (t) => {
  let failureId: string | undefined;
  const host = await fixture(t, async () => ({ plugins: [effectivePlugin("7", "7")] }), (getResponse, options) => {
    const client = createBackendMarketplaceClient(getResponse, options);
    return {
      async listEnabledPlugins(userId, teamId) {
        const listed = await client.listEnabledPlugins(userId, teamId);
        return {
          ...listed,
          listFailures: [{
            pluginName: "7", pluginId: "7", pluginDbId: failureId,
            errorMessage: "Repository not found", errorType: "clone" as const,
          }],
        };
      },
      async installPlugin() { assert.fail("a version-resolution denial cannot install content"); },
    };
  });
  for (const id of [undefined, "", " 7", "7 "]) {
    failureId = id;
    await assert.rejects(host.load(), /authorization failure has no valid canonical plugin ID/);
  }
});

test("remote Git clone authorization denial retains its canonical ID as an auth block", async (t) => {
  const host = await fixture(t, async () => ({ plugins: [effectivePlugin("41", "7")] }), (getResponse, options) => {
    const client = createBackendMarketplaceClient(getResponse, options);
    return {
      async listEnabledPlugins(userId, teamId) {
        const listed = await client.listEnabledPlugins(userId, teamId);
        return {
          ...listed,
          listFailures: [{
            pluginName: "7", pluginId: "7", pluginDbId: "41",
            errorMessage: "Repository not found", errorType: "clone" as const,
          }],
        };
      },
      async installPlugin() { assert.fail("a version-resolution denial cannot install content"); },
    };
  });
  const oldSkill = await seedOldVersion(host.cache, "7");
  const result = await host.load();
  assert.deepEqual(result.listedPluginIds, ["41"]);
  assert.deepEqual(result.authBlocked, [{ pluginId: "41", pluginName: "7" }]);
  assert.equal(await readFile(oldSkill, "utf8"), "Previous committed skill bytes.\n");
});

test("Host defers SDK pruning even after a complete successful materialization", async (t) => {
  const host = await fixture(t, async () => ({ plugins: [effectivePlugin("db-current", "current", "{}")] }));
  const oldSkill = await seedOldVersion(host.cache, "current");
  const result = await host.load();

  assert.equal(result.plugins.length, 1);
  assert.equal(result.plugins[0]?.identifier.pluginDbId, "db-current");
  assert.equal(result.plugins[0]?.loadError, undefined);
  assert.deepEqual(result.plugins[0]?.skills, []);
  assert.deepEqual(result.authBlocked, []);
  assert.equal(await readFile(oldSkill, "utf8"), "Previous committed skill bytes.\n");
  assert.equal((await host.cache.listCachedVersions({ marketplaceSlug: "cursor-public", pluginId: "current" })).length, 2);
});

function accountToken(subject: string | undefined, renewal: number): string {
  return `fixture.${Buffer.from(JSON.stringify({ ...(subject === undefined ? {} : { sub: subject }), renewal })).toString("base64url")}.signature`;
}

test("one Host loader refreshes publisher identity for consecutive accounts", async (t) => {
  const tokenA = accountToken("publisher-a", 1);
  const tokenB = accountToken("consumer-b", 1);
  let currentToken = tokenA;
  let tokenReads = 0;
  const observedListingCredentials: string[] = [];
  const observedIdentityCredentials: string[] = [];
  const host = await fixture(t, async (token) => {
    observedListingCredentials.push(token);
    return { plugins: [effectivePlugin("shared", "shared", "{}")] };
  }, undefined, {
    getAccessToken: async () => { tokenReads += 1; return currentToken; },
    peekAccessToken: async () => currentToken,
    getCurrentUser: async (token) => {
      observedIdentityCredentials.push(token);
      return { userId: token === tokenA ? 8n : 9n };
    },
  });

  const first = await host.load();
  currentToken = tokenB;
  const second = await host.load();

  assert.equal(first.currentUserId, 8);
  assert.equal(second.currentUserId, 9);
  assert.deepEqual(first.listedPluginIds, ["shared"]);
  assert.deepEqual(second.listedPluginIds, first.listedPluginIds);
  assert.deepEqual(second.publisherFacts, first.publisherFacts, "installed publisher identity need not change when the consuming account changes");
  assert.deepEqual(observedListingCredentials, [tokenA, tokenB]);
  assert.deepEqual(observedIdentityCredentials, [tokenA, tokenB]);
  assert.equal(tokenReads, 2, "each pass captures one credential for all SDK requests");
  assert.deepEqual(host.calls(), { listings: 2, currentUserLookups: 2 });
});

test("Host refuses an actual Sand account read superseded by logout and login", { timeout: 10_000 }, async (t) => {
  for (const ordering of ["peek-before-logout-login", "login-before-peek"] as const) {
    await t.test(ordering, async (t) => {
      const token = (subject: string) => `fixture.${Buffer.from(JSON.stringify({
        sub: subject,
        exp: Math.floor(Date.now() / 1_000) + 3_600,
      })).toString("base64url")}.signature`;
      const tokenA = token("publisher-a");
      const tokenB = token("consumer-b");
      const stored = new Map([
        [ACCESS_TOKEN_SECRET_KEY, tokenA],
        [REFRESH_TOKEN_SECRET_KEY, "refresh-a"],
      ]);
      const entered = Promise.withResolvers<void>();
      const release = Promise.withResolvers<void>();
      const loginEntered = Promise.withResolvers<void>();
      const finishLogin = Promise.withResolvers<void>();
      t.after(() => { release.resolve(); finishLogin.resolve(); });
      let holdNextAccessRead = false;
      let pauseFinalPeek = false;
      const auth = new SandCursorAuthService({
        openExternal: async () => {},
        secrets: {
          async readSecret(key) {
            const value = stored.get(key) ?? null;
            if (key === ACCESS_TOKEN_SECRET_KEY && holdNextAccessRead) {
              holdNextAccessRead = false;
              entered.resolve();
              await release.promise;
            }
            return value;
          },
          async writeSecret(key, value) { stored.set(key, value); },
          async deleteSecret(key) { stored.delete(key); },
          isEncryptedStorageAvailable: () => true,
        },
        waitForEncryptedStorage: async (isAvailable) => { assert.ok(isAvailable()); },
        createLoginManager: () => ({
          startLogin: () => ({ metadata: { uuid: "fixture-login", verifier: "fixture-verifier" }, loginUrl: "https://auth.example.test/login" }),
          waitForResult: async () => {
            loginEntered.resolve();
            if (ordering === "login-before-peek") await finishLogin.promise;
            return { accessToken: tokenB, refreshToken: "refresh-b" };
          },
        }),
        fetchOAuthToken: async () => { throw new Error("Unexpired fixture credentials must not refresh over the network."); },
      });
      const host = await fixture(t, async () => ({ plugins: [effectivePlugin("shared", "shared", "{}")] }), undefined, {
        getAccessToken: (options) => auth.getValidAccessToken(options),
        peekAccessToken: () => {
          if (pauseFinalPeek) {
            pauseFinalPeek = false;
            holdNextAccessRead = true;
          }
          return auth.peekAccessToken();
        },
        getCurrentUser: async (accessToken) => ({ userId: accessToken === tokenA ? 8n : 9n }),
      });
      assert.equal((await host.load()).currentUserId, 8);
      let login: ReturnType<typeof auth.login> | undefined;
      if (ordering === "login-before-peek") {
        login = auth.login();
        await loginEntered.promise;
      }
      pauseFinalPeek = true;
      const refused = assert.rejects(host.load(), /account context changed or became unavailable/);
      await Promise.race([
        entered.promise,
        refused.then(() => { throw new Error("The stale pass must reach the actual held account read."); }),
      ]);
      try {
        if (login === undefined) {
          assert.equal((await auth.logout()).kind, "logged-out");
          login = auth.login();
        } else {
          finishLogin.resolve();
        }
        const signedIn = await login;
        assert.equal(signedIn.kind, "logged-in");
        assert.equal(signedIn.kind === "logged-in" ? signedIn.authId : undefined, "consumer-b");
        assert.equal(await auth.peekAccessToken(), tokenB, "the new account is active before the held A read finishes");
      } finally {
        release.resolve();
        finishLogin.resolve();
      }
      await refused;
      assert.equal((await host.load()).currentUserId, 9, "the same loader accepts a fresh B pass after refusing stale A");
      assert.deepEqual(host.calls(), { listings: 3, currentUserLookups: 3 });
    });
  }
});

test("a Host identity lookup failure or invalid ID cannot reuse a prior successful identity", async (t) => {
  let identity: { userId?: unknown } | null | undefined = { userId: 8n };
  let failIdentity = false;
  const logs: string[] = [];
  const host = await fixture(t, async () => ({ plugins: [effectivePlugin("shared", "shared", "{}")] }), undefined, {
    log: (message) => logs.push(message),
    getCurrentUser: async () => {
      if (failIdentity) throw new Error("fixture-sensitive-transport-detail");
      return identity;
    },
  });

  assert.equal((await host.load()).currentUserId, 8);
  failIdentity = true;
  const failedIdentity = await host.load();
  assert.equal(failedIdentity.currentUserId, null);
  assert.deepEqual(failedIdentity.listedPluginIds, ["shared"]);
  assert.equal(failedIdentity.plugins.length, 1, "unknown publisher identity does not erase a valid authoritative listing");
  assert.ok(logs.some((message) => message.includes("could not resolve the signed-in user")));
  assert.ok(logs.every((message) => !message.includes("fixture-sensitive-transport-detail")));

  failIdentity = false;
  for (const invalid of [undefined, null, {}, { userId: "8" }, { userId: 0 }, { userId: -1 }, { userId: 1.5 }, { userId: Number.MAX_SAFE_INTEGER + 1 }, { userId: 0n }]) {
    identity = invalid;
    assert.equal((await host.load()).currentUserId, null);
  }
  identity = { userId: 9n };
  assert.equal((await host.load()).currentUserId, 9, "fresh identity lookup can recover on the same loader");
});

test("Host pins listing identity and Git credentials then refuses an account change during the pass", async (t) => {
  const tokenA = accountToken("publisher-a", 1);
  const tokenB = accountToken("consumer-b", 1);
  let currentToken = tokenA;
  const listingCredentials: string[] = [];
  const identityCredentials: string[] = [];
  let gitConfig: Parameters<ClientFactory>[1]["extraGitConfig"];
  const host = await fixture(t, async (token) => {
    listingCredentials.push(token);
    currentToken = tokenB;
    return { plugins: [effectivePlugin("shared", "shared", "{}")] };
  }, (getResponse, options) => {
    gitConfig = options.extraGitConfig;
    return createBackendMarketplaceClient(getResponse, options);
  }, {
    getAccessToken: async () => currentToken,
    peekAccessToken: async () => currentToken,
    getCurrentUser: async (token) => {
      identityCredentials.push(token);
      return { userId: 8n };
    },
  });
  const oldSkill = await seedOldVersion(host.cache, "shared");
  const oldBytes = await readFile(oldSkill);

  await assert.rejects(host.load(), /account context changed or became unavailable/);
  assert.deepEqual(listingCredentials, [tokenA]);
  assert.deepEqual(identityCredentials, [tokenA], "getMe must not switch to the new account after listing");
  assert.deepEqual(gitConfig, buildOriginTokenGitConfig(tokenA));
  assert.deepEqual(await readFile(oldSkill), oldBytes, "a rejected pass cannot prune a previously materialized version");
});

test("Host rejects account departure and unreadable current account after materialization", async (t) => {
  for (const currentAccount of ["departed", "unreadable"] as const) {
    await t.test(currentAccount, async (t) => {
      let materialized = false;
      const host = await fixture(t, async () => ({ plugins: [effectivePlugin("shared", "shared", "{}")] }), withInstall(async (_entry, _targetDir, actual) => {
        await actual();
        materialized = true;
      }), {
        peekAccessToken: async () => {
          assert.ok(materialized, "currentness is checked after the actual SDK materialization");
          if (currentAccount === "unreadable") throw new Error("fixture-sensitive-current-token-error");
          return null;
        },
      });
      await assert.rejects(host.load(), (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.equal(error.message, "Plugin Skills account context changed or became unavailable during refresh.");
        return true;
      });
      assert.ok(materialized, "the refusal must follow a completed SDK materialization");
    });
  }
});

test("Host accepts same-account renewal without reusing or mixing the pass credential", async (t) => {
  const firstToken = accountToken("publisher-a", 1);
  const renewedToken = accountToken("publisher-a", 2);
  let currentToken = firstToken;
  const host = await fixture(t, async (token) => {
    assert.equal(token, firstToken);
    currentToken = renewedToken;
    return { plugins: [] };
  }, undefined, {
    getAccessToken: async () => currentToken,
    peekAccessToken: async () => currentToken,
    getCurrentUser: async (token) => {
      assert.equal(token, firstToken);
      return { userId: 8n };
    },
  });

  assert.equal((await host.load()).currentUserId, 8);
});

test("Host does not equate different tokens with empty missing or ambiguous account subjects", async (t) => {
  for (const subject of [undefined, "", " ", " publisher-a "]) {
    const firstToken = accountToken(subject, 1);
    let currentToken = firstToken;
    const host = await fixture(t, async () => {
      currentToken = accountToken(subject, 2);
      return { plugins: [] };
    }, undefined, {
      getAccessToken: async () => currentToken,
      peekAccessToken: async () => currentToken,
    });

    await assert.rejects(host.load(), /account context changed or became unavailable/);
  }
});

test("Host refuses a backend-context change before SDK credentials are consumed or after materialization", async (t) => {
  const previousBackend = process.env.FABUSHI_API_BASE_URL;
  t.after(() => {
    if (previousBackend === undefined) delete process.env.FABUSHI_API_BASE_URL;
    else process.env.FABUSHI_API_BASE_URL = previousBackend;
  });
  for (const changeAt of ["credential-capture", "materialization"] as const) {
    process.env.FABUSHI_API_BASE_URL = "http://127.0.0.1:42101";
    let listed = false;
    let materialized = false;
    const host = await fixture(t, async () => {
      listed = true;
      return { plugins: [effectivePlugin("shared", "shared", "{}")] };
    }, withInstall(async (_entry, _targetDir, actual) => {
      await actual();
      materialized = true;
      process.env.FABUSHI_API_BASE_URL = "http://127.0.0.1:42102";
    }), {
      getAccessToken: async ({ backendUrl }) => {
        assert.equal(backendUrl, "http://127.0.0.1:42101/");
        if (changeAt === "credential-capture") process.env.FABUSHI_API_BASE_URL = "http://127.0.0.1:42102";
        return "fixture-token";
      },
    });

    await assert.rejects(host.load(), /account context changed or became unavailable/);
    assert.equal(listed, changeAt === "materialization");
    assert.equal(materialized, changeAt === "materialization");
  }
});

test("Host refuses the previous Fabushi account while public browser login is pending", { timeout: 10_000 }, async (t) => {
  const issuer = "http://127.0.0.1:42103";
  const previousBackend = process.env.FABUSHI_API_BASE_URL;
  process.env.FABUSHI_API_BASE_URL = issuer;
  t.after(() => {
    if (previousBackend === undefined) delete process.env.FABUSHI_API_BASE_URL;
    else process.env.FABUSHI_API_BASE_URL = previousBackend;
  });
  const credentialDirectory = await mkdtemp(join(tmpdir(), "fabushi-plugin-account-"));
  t.after(async () => await rm(credentialDirectory, { recursive: true, force: true }));
  const hostAccessCredentialFile = join(credentialDirectory, "host-access.json");
  const now = Date.now();
  const tokenA = accountToken("publisher-a", 1);
  const tokenB = accountToken("consumer-b", 1);
  const sessionA: FabushiSession = {
    issuer,
    accessToken: tokenA,
    refreshToken: "fixture-refresh-a",
    accessTokenExpiresAt: now + 3_600_000,
    refreshTokenExpiresAt: now + 7_200_000,
    sessionId: "browser:fixture-a",
    deviceId: "fixture-machine",
    username: "publisher-a@example.test",
    userId: 8,
  };
  const sessionB: FabushiSession = {
    ...sessionA,
    accessToken: tokenB,
    refreshToken: "fixture-refresh-b",
    sessionId: "browser:fixture-b",
    username: "consumer-b@example.test",
    userId: 9,
  };
  const materialized = Promise.withResolvers<void>();
  const finishMaterialization = Promise.withResolvers<void>();
  const pollEntered = Promise.withResolvers<void>();
  const finishLogin = Promise.withResolvers<void>();
  t.after(() => { finishMaterialization.resolve(); finishLogin.resolve(); });
  let pollCompleted = false;
  const requests: string[] = [];
  const loginUrl = `${issuer}/login/fixture-b`;
  const response = (body: unknown) => new Response(JSON.stringify(body), {
    headers: { "Content-Type": "application/json" },
  });
  const auth = new FabushiAuthService({
    openExternal: async (url) => { assert.equal(url, loginUrl); },
    env: { FABUSHI_API_BASE_URL: issuer },
    machineId: "fixture-machine",
    hostAccessCredentialFile,
    now: () => now,
    fetchImpl: async (input, init) => {
      assert.ok(typeof input === "string");
      const url = new URL(input);
      assert.equal(url.origin, issuer);
      requests.push(url.pathname);
      if (url.pathname === "/api/auth/browser/start") {
        assert.equal(init?.method, "POST");
        assert.equal(init?.body, JSON.stringify({ deviceId: "fixture-machine", platform: "desktop" }));
        return response({ attemptId: "fixture-b", loginUrl, pollSecret: "fixture-poll-secret", expiresAt: now + 60_000 });
      }
      if (url.pathname === "/api/auth/browser/attempts/fixture-b") {
        assert.equal(init?.method, "POST");
        assert.equal(init?.body, JSON.stringify({ pollSecret: "fixture-poll-secret" }));
        pollEntered.resolve();
        await finishLogin.promise;
        pollCompleted = true;
        return response({ status: "completed", session: sessionB });
      }
      if (url.pathname === "/api/auth/user-info") {
        assert.equal(new Headers(init?.headers).get("Authorization"), `Bearer ${tokenB}`);
        return response({ id: 9, username: sessionB.username });
      }
      throw new Error("Unexpected Fabushi account network request in the held-login contract.");
    },
  });
  // Only secret-storage I/O is substituted. currentSession, login/runLogin,
  // identity/epoch checks and atomic Host credential-file writes remain real.
  const storage = auth as unknown as {
    readStoredSession(): Promise<FabushiSession | null>;
    saveStoredSession(session: FabushiSession): Promise<void>;
  };
  let storedSession = sessionA;
  let storedReads = 0;
  storage.readStoredSession = async () => { storedReads += 1; return storedSession; };
  storage.saveStoredSession = async (session) => { storedSession = session; };
  const listingCredentials: string[] = [];
  const identityCredentials: string[] = [];
  const host = await fixture(t, async (accessToken) => {
    listingCredentials.push(accessToken);
    return { plugins: [effectivePlugin("shared", "shared", "{}")] };
  }, withInstall(async (_entry, _targetDir, actual) => {
    await actual();
    materialized.resolve();
    await finishMaterialization.promise;
  }), {
    getAccessToken: (options) => auth.getValidAccessToken(options),
    peekAccessToken: () => auth.peekAccessToken(),
    getCurrentUser: async (accessToken) => {
      identityCredentials.push(accessToken);
      assert.ok(accessToken === tokenA || accessToken === tokenB);
      return { userId: accessToken === tokenA ? 8n : 9n };
    },
  });
  const oldSkill = await seedOldVersion(host.cache, "shared");
  const oldBytes = await readFile(oldSkill);
  assert.equal(await auth.peekAccessToken(), tokenA, "ordinary stable Fabushi reads remain available");
  const credentialBytesA = await readFile(hostAccessCredentialFile);
  assert.equal(JSON.parse(credentialBytesA.toString("utf8")).accessToken, tokenA);

  let login: ReturnType<typeof auth.login> | undefined;
  const stalePass = host.load();
  const refused = assert.rejects(stalePass, /account context changed or became unavailable/);
  try {
    await Promise.race([
      materialized.promise,
      refused.then(() => { throw new Error("The A pass must reach actual SDK materialization before B login."); }),
    ]);
    login = auth.login();
    await Promise.race([
      pollEntered.promise,
      login.then(() => { throw new Error("The public Fabushi login must reach the held browser poll."); }),
    ]);
    const readsBeforePendingPeek = storedReads;
    finishMaterialization.resolve();
    await refused;
    assert.equal(pollCompleted, false, "the old A pass must be refused before B credentials publish");
    assert.equal(await auth.peekAccessToken(), null, "an active browser login cannot authorize the previous account");
    assert.equal(storedReads, readsBeforePendingPeek, "pending-login peeks must not read or republish stored A credentials");
    assert.deepEqual(await readFile(hostAccessCredentialFile), credentialBytesA);
    assert.deepEqual(await readFile(oldSkill), oldBytes, "refusing stale account authority must preserve previous plugin bytes");

    finishLogin.resolve();
    const signedIn = await login;
    assert.equal(signedIn.kind, "logged-in");
    assert.equal(signedIn.kind === "logged-in" ? signedIn.authId : undefined, "9");
    assert.equal(await auth.peekAccessToken(), tokenB);
    const fresh = await host.load();
    assert.equal(fresh.currentUserId, 9, "the same loader accepts the new identity after public login settles");
    assert.deepEqual(fresh.listedPluginIds, ["shared"]);
    assert.equal(fresh.plugins[0]?.identifier.pluginDbId, "shared");
    assert.deepEqual(listingCredentials, [tokenA, tokenB]);
    assert.deepEqual(identityCredentials, [tokenA, tokenB]);
    assert.deepEqual(host.calls(), { listings: 2, currentUserLookups: 2 });
    assert.deepEqual(JSON.parse(await readFile(hostAccessCredentialFile, "utf8")), {
      version: 1,
      issuer,
      accessToken: tokenB,
      expiresAtMs: sessionB.accessTokenExpiresAt,
      sessionId: sessionB.sessionId,
      deviceId: sessionB.deviceId,
      userId: sessionB.userId,
    });
    assert.deepEqual(requests, ["/api/auth/browser/start", "/api/auth/browser/attempts/fixture-b", "/api/auth/user-info"]);
  } finally {
    finishMaterialization.resolve();
    finishLogin.resolve();
    await Promise.allSettled([stalePass, ...(login === undefined ? [] : [login])]);
  }
});
