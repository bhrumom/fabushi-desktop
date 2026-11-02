import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test, { type TestContext } from "node:test";

import {
  createBackendMarketplaceClient,
  type BackendPluginEntry,
} from "../../packages/cursor-plugins/backend-marketplace-client.js";
import { DefaultPluginCacheManager } from "../../packages/cursor-plugins/cursor-marketplace.js";
import {
  createHostPluginSkillsLoader,
  HOST_PLUGIN_SKILLS_CURRENT_USER_TIMEOUT_MS,
  HOST_PLUGIN_SKILLS_EFFECTIVE_PLUGINS_TIMEOUT_MS,
  type HostPluginSkillsLoaderDeps,
} from "./host-plugin-skills-loader.js";

type ClientFactory = NonNullable<HostPluginSkillsLoaderDeps["createMarketplaceClient"]>;

async function fixture(
  t: TestContext,
  response: () => Promise<unknown>,
  createMarketplaceClient?: ClientFactory,
) {
  const sandRootDir = await mkdtemp(join(tmpdir(), "fabushi-host-plugin-skills-"));
  t.after(async () => await rm(sandRootDir, { recursive: true, force: true }));
  let listings = 0;
  let currentUserLookups = 0;
  const load = createHostPluginSkillsLoader({
    sandRootDir,
    getAccessToken: async () => "fixture-token",
    getMachineId: () => "fixture-machine",
    peekAccessToken: async () => null,
    isSparsePluginClonesEnabled: () => false,
    log: () => {},
    dashboardClient: {
      async getEffectiveUserPlugins(request, options) {
        assert.deepEqual(request, {});
        assert.equal(options.timeoutMs, HOST_PLUGIN_SKILLS_EFFECTIVE_PLUGINS_TIMEOUT_MS);
        listings += 1;
        return await response();
      },
      async getMe(request, options) {
        assert.deepEqual(request, {});
        assert.equal(options.timeoutMs, HOST_PLUGIN_SKILLS_CURRENT_USER_TIMEOUT_MS);
        currentUserLookups += 1;
        return { userId: 8n };
      },
    },
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

test("Host authorization failures use the canonical DB ID when the slug matches another plugin's DB ID", async (t) => {
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
  assert.deepEqual(result.authBlocked, [{ pluginId: "41", pluginName: "7", marketplaceName: "cursor-public" }]);
  assert.equal(result.plugins[0]?.identifier.pluginDbId, "41");
  assert.match(result.plugins[0]?.loadError ?? "", /Permission denied/);
  assert.equal(result.plugins[1]?.identifier.pluginDbId, "7");
  assert.equal(result.plugins[1]?.loadError, undefined);
  assert.deepEqual(result.plugins[1]?.skills, []);
  assert.equal(await readFile(deniedOldSkill, "utf8"), "Previous committed skill bytes.\n");
  assert.equal(await readFile(healthyOldSkill, "utf8"), "Previous committed skill bytes.\n");
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
