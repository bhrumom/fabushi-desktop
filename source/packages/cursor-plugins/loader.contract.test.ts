import assert from "node:assert/strict";
import { promises as fsPromises } from "node:fs";
import { lstat, mkdir, mkdtemp, readdir, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { syncBuiltinESMExports } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test, { type TestContext } from "node:test";

import type { BackendPluginEntry } from "./backend-marketplace-client.js";
import { DefaultPluginCacheManager } from "./cursor-marketplace.js";
import { loadFromMarketplaceSource, type MarketplaceLoaderClient } from "./loader.js";

type PruneArgs = Parameters<DefaultPluginCacheManager["pruneOldVersions"]>[0];

class ObservedCacheManager extends DefaultPluginCacheManager {
  readonly prunes: PruneArgs[] = [];

  override async pruneOldVersions(args: PruneArgs): Promise<void> {
    this.prunes.push({ ...args, keepVersions: [...args.keepVersions] });
    await super.pruneOldVersions(args);
  }
}

async function cacheFixture(t: TestContext): Promise<ObservedCacheManager> {
  const root = await mkdtemp(join(tmpdir(), "fabushi-plugin-loader-"));
  t.after(async () => await rm(root, { recursive: true, force: true }));
  return new ObservedCacheManager(undefined, { cacheRoot: join(root, "cache") });
}

function entry(overrides: Partial<BackendPluginEntry> = {}): BackendPluginEntry {
  return {
    pluginId: "calendar-slug",
    pluginDbId: "db-calendar",
    name: "calendar-slug",
    version: "new-version",
    downloadUrl: "fixture://calendar",
    marketplace: { id: "marketplace-db", name: "cursor-public" },
    ...overrides,
  };
}

function cacheKey(plugin: BackendPluginEntry) {
  return {
    marketplaceSlug: plugin.marketplace!.name,
    pluginId: plugin.pluginId,
    version: plugin.version,
  };
}

async function writeSkill(directory: string, name: string): Promise<string> {
  await mkdir(directory, { recursive: true });
  const path = join(directory, "SKILL.md");
  await writeFile(path, `---\nname: ${name}\ndescription: Fixture skill\n---\n\n${name} instructions.\n`);
  return path;
}

async function seedOldVersion(cache: DefaultPluginCacheManager, plugin: BackendPluginEntry): Promise<string> {
  const key = {
    marketplaceSlug: plugin.marketplace!.name,
    pluginId: plugin.pluginId,
    version: "old-version",
  };
  const path = await writeSkill(cache.getCacheDir(key), "previous-skill");
  await cache.markCacheComplete(key);
  return path;
}

test("SDK distinguishes an unavailable source from an authoritative empty listing", async (t) => {
  const cache = await cacheFixture(t);
  const oldSkill = await seedOldVersion(cache, entry());
  const oldBytes = await readFile(oldSkill, "utf8");
  let unavailable = true;
  let listings = 0;
  const client: MarketplaceLoaderClient = {
    async listEnabledPlugins() {
      if (unavailable) throw new Error("effective listing offline");
      return { plugins: [] };
    },
    async installPlugin() { assert.fail("empty or failed listings cannot install a plugin"); },
  };
  const options = {
    client,
    userId: "fixture-user",
    cacheManager: cache,
    pruneOldVersions: true,
    onPluginsListed: async () => { listings += 1; },
  };

  assert.deepEqual(await loadFromMarketplaceSource(options), {
    plugins: [], failures: [], sourceUnavailable: true,
  });
  assert.equal(listings, 0);
  assert.deepEqual(cache.prunes, []);
  assert.equal(await readFile(oldSkill, "utf8"), oldBytes);

  unavailable = false;
  assert.deepEqual(await loadFromMarketplaceSource(options), {
    plugins: [], failures: [], sourceUnavailable: false,
  });
  assert.equal(listings, 1);
  assert.deepEqual(cache.prunes, []);
});

for (const [label, pruneOldVersions, expectedPrunes] of [
  ["explicit false defers both pruning sites", false, 0],
  ["omitted option preserves the default per-plugin prune", undefined, 1],
  ["explicit true preserves both pruning sites", true, 2],
] as const) {
  test(`SDK ${label}`, async (t) => {
    const cache = await cacheFixture(t);
    const plugin = entry();
    const oldSkill = await seedOldVersion(cache, plugin);
    const oldBytes = await readFile(oldSkill, "utf8");
    let installs = 0;
    const result = await loadFromMarketplaceSource({
      client: {
        async listEnabledPlugins() { return { plugins: [plugin] }; },
        async installPlugin(_entry, targetDir) {
          installs += 1;
          await writeSkill(targetDir, "fresh-skill");
        },
      },
      userId: "fixture-user",
      cacheManager: cache,
      ...(pruneOldVersions === undefined ? {} : { pruneOldVersions }),
    });

    assert.equal(result.sourceUnavailable, false);
    assert.deepEqual(result.failures, []);
    assert.equal(installs, 1);
    assert.equal(result.plugins.length, 1);
    assert.equal(result.plugins[0]?.identifier.sourceInfo.pluginDbId, "db-calendar");
    assert.equal(result.plugins[0]?.skills[0]?.name, "fresh-skill");
    assert.deepEqual(cache.prunes, Array.from({ length: expectedPrunes }, () => ({
      marketplaceSlug: "cursor-public", pluginId: "calendar-slug", keepVersions: ["new-version"],
    })));
    if (pruneOldVersions === false) {
      assert.equal(await readFile(oldSkill, "utf8"), oldBytes);
    } else {
      await assert.rejects(readFile(oldSkill), { code: "ENOENT" });
    }
    assert.equal(await cache.isCached({
      marketplaceSlug: "cursor-public", pluginId: plugin.pluginId, version: plugin.version,
    }), true);
  });
}

test("SDK missing-marketplace failures preserve the supplied canonical DB ID", async (t) => {
  const cache = await cacheFixture(t);
  const plugin = entry({ pluginDbId: "opaque:001", marketplace: undefined });
  const reported: BackendPluginEntry[] = [];
  const result = await loadFromMarketplaceSource({
    client: {
      async listEnabledPlugins() { return { plugins: [plugin] }; },
      async installPlugin() { assert.fail("missing marketplace must fail before installation"); },
    },
    userId: "fixture-user",
    cacheManager: cache,
    pruneOldVersions: false,
    onCursorError: (failed) => { reported.push(failed); },
  });

  assert.equal(result.sourceUnavailable, false);
  assert.equal(result.failures.length, 1);
  assert.equal(result.failures[0]?.pluginDbId, "opaque:001");
  assert.equal(result.failures[0]?.pluginId, "calendar-slug");
  assert.equal(result.failures[0]?.errorType, "manifest");
  assert.deepEqual(reported, [plugin]);
  assert.deepEqual(cache.prunes, []);
});

test("SDK installation failure retains canonical identity and old files during a partial pass", async (t) => {
  const cache = await cacheFixture(t);
  const denied = entry({ pluginId: "7", name: "7", pluginDbId: "41" });
  const healthy = entry({ pluginId: "healthy", name: "healthy", pluginDbId: "7" });
  const oldDeniedSkill = await seedOldVersion(cache, denied);
  const oldHealthySkill = await seedOldVersion(cache, healthy);
  const reported: BackendPluginEntry[] = [];
  const result = await loadFromMarketplaceSource({
    client: {
      async listEnabledPlugins() { return { plugins: [denied, healthy] }; },
      async installPlugin(plugin, targetDir) {
        if (plugin.pluginDbId === "41") throw new Error("Permission denied (publickey).");
        await writeSkill(targetDir, "healthy-skill");
      },
    },
    userId: "fixture-user",
    cacheManager: cache,
    pruneOldVersions: false,
    onCursorError: (failed) => { reported.push(failed); },
  });

  assert.equal(result.sourceUnavailable, false);
  assert.equal(result.failures.length, 1);
  assert.equal(result.failures[0]?.pluginDbId, "41");
  assert.equal(result.failures[0]?.pluginId, "7");
  assert.deepEqual(reported, [denied]);
  assert.equal(result.plugins[0]?.identifier.sourceInfo.pluginDbId, "41");
  assert.match(result.plugins[0]?.loadError ?? "", /Permission denied/);
  assert.equal(result.plugins[1]?.identifier.sourceInfo.pluginDbId, "7");
  assert.equal(result.plugins[1]?.skills[0]?.name, "healthy-skill");
  assert.deepEqual(cache.prunes, []);
  assert.match(await readFile(oldDeniedSkill, "utf8"), /previous-skill instructions/);
  assert.match(await readFile(oldHealthySkill, "utf8"), /previous-skill instructions/);
  assert.equal(await cache.isCached({
    marketplaceSlug: "cursor-public", pluginId: denied.pluginId, version: denied.version,
  }), false);
});

test("failed cache publication removes only its staging directory", async (t) => {
  const cache = await cacheFixture(t);
  const plugin = entry();
  const targetDir = cache.getCacheDir(cacheKey(plugin));
  let stagingDir = "";
  const result = await loadFromMarketplaceSource({
    client: {
      async listEnabledPlugins() { return { plugins: [plugin] }; },
      async installPlugin(_entry, stage) {
        stagingDir = stage;
        await writeSkill(stage, "staged-skill");
        // Introduce the collision after staging is owned, so the failure must
        // preserve the foreign target and actually clean the allocated stage.
        await mkdir(targetDir, { recursive: true });
        await writeFile(join(targetDir, "preserve.txt"), "existing target must survive");
      },
    },
    userId: "fixture-user",
    cacheManager: cache,
    pruneOldVersions: false,
  });
  assert.equal(result.failures.length, 1);
  assert.match(result.failures[0]?.errorMessage ?? "", /Plugin cache target is incomplete/);
  assert.equal(await readFile(join(targetDir, "preserve.txt"), "utf8"), "existing target must survive");
  assert.deepEqual(await readdir(dirname(targetDir)), [plugin.version]);
  assert.notEqual(stagingDir, "");
  await assert.rejects(lstat(stagingDir), { code: "ENOENT" });
  await assert.rejects(lstat(join(targetDir, ".cache-complete")), { code: "ENOENT" });
  assert.equal(result.plugins[0]?.installPath, "");
  assert.deepEqual(cache.prunes, []);
});

for (const code of ["EACCES", "EPERM", "EROFS"] as const) {
  test(`filesystem ${code} during cache installation is an install failure, not clone authentication`, async (t) => {
    const cache = await cacheFixture(t);
    const plugin = entry();
    const filesystemError = Object.assign(new Error("permission denied while publishing plugin cache"), { code });
    const result = await loadFromMarketplaceSource({
      client: {
        async listEnabledPlugins() { return { plugins: [plugin] }; },
        async installPlugin() { throw filesystemError; },
      },
      userId: "fixture-user",
      cacheManager: cache,
      pruneOldVersions: false,
    });
    assert.equal(result.failures.length, 1);
    assert.equal(result.failures[0]?.errorType, "install");
    assert.match(result.failures[0]?.errorMessage ?? "", /permission denied/i);
    assert.equal(await cache.isCached(cacheKey(plugin)), false);
  });
}

// Controlled negatives target the real stage's removal; they do not establish
// live filesystem timeout or marketplace compatibility.
test("SDK cleanup I/O failures preserve the primary load category and raw causes", async (t) => {
  for (const cleanupCode of ["ETIMEDOUT", "EACCES"] as const) {
    const denial = new Error("git clone failed: Permission denied (publickey).");
    for (const [primaryError, expectedType] of [
      [denial, "clone"],
      [new Error("remote endpoint disconnected"), "unknown"],
      [new AggregateError([denial], `${denial.message} Parent diagnostics ETIMEDOUT.`, { cause: denial }), "clone"],
      [undefined, "unknown"],
    ] as const) {
      const cache = await cacheFixture(t);
      const plugin = entry({ pluginId: "7", name: "7", pluginDbId: "41" });
      const oldSkill = await seedOldVersion(cache, plugin);
      let stagingDir = "";
      let installs = 0;
      let cleanupCalls = 0;
      const observedErrors: Error[] = [];
      const cleanupError = Object.assign(new Error("controlled cleanup failure"), { code: cleanupCode });
      const client: MarketplaceLoaderClient = {
        async listEnabledPlugins() { return { plugins: [plugin] }; },
        async installPlugin(_entry, stage) {
          installs += 1;
          stagingDir = stage;
          await writeSkill(stage, installs === 1 ? "retained-stage" : "recovered-skill");
          if (installs === 1) throw primaryError;
        },
      };
      const load = () => loadFromMarketplaceSource({
        client, userId: "fixture-user", cacheManager: cache, pruneOldVersions: false,
        onCursorError: (_entry, error) => { observedErrors.push(error); },
      });
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
        const result = await load();
        retainedStage = stagingDir;
        assert.notEqual(retainedStage, "");
        assert.equal(cleanupCalls, 1);
        assert.equal(result.failures.length, 1);
        assert.equal(result.failures[0]?.pluginDbId, "41");
        assert.equal(result.failures[0]?.errorType, expectedType, cleanupCode);
        assert.equal(result.plugins[0]?.installPath, "");
        assert.equal(observedErrors.length, 1);
        const observedError = observedErrors[0];
        assert.ok(observedError instanceof AggregateError);
        assert.equal(observedError.cause, primaryError);
        assert.equal(observedError.errors[0], primaryError);
        assert.equal(observedError.errors[1], cleanupError);
        assert.equal(result.failures[0]?.errorMessage, observedError.message);
        assert.match(observedError.message, /Plugin cache staging cleanup failed; residual state unknown/);
        assert.ok(observedError.message.includes(`I/O code: ${cleanupCode}`));
        assert.match(await readFile(join(retainedStage, "SKILL.md"), "utf8"), /retained-stage instructions/);
        await assert.rejects(lstat(join(retainedStage, ".cache-complete")), { code: "ENOENT" });
        await assert.rejects(lstat(cache.getCacheDir(cacheKey(plugin))), { code: "ENOENT" });
        assert.match(await readFile(oldSkill, "utf8"), /previous-skill instructions/);
        assert.deepEqual(cache.prunes, []);
      } finally {
        rmMock.mock.restore();
        syncBuiltinESMExports();
      }
      await originalRm(retainedStage, { recursive: true, force: true });
      const recovered = await load();
      assert.deepEqual(recovered.failures, []);
      assert.equal(recovered.plugins[0]?.skills[0]?.name, "recovered-skill");
      assert.equal(await cache.isCached(cacheKey(plugin)), true);
      assert.equal(installs, 2);
      assert.match(await readFile(oldSkill, "utf8"), /previous-skill instructions/);
    }
  }
});

test("an older failed install cannot remove a successor published cache directory", async (t) => {
  const cache = await cacheFixture(t);
  const plugin = entry();
  const olderEntered = Promise.withResolvers<void>();
  const releaseOlder = Promise.withResolvers<void>();
  const older: MarketplaceLoaderClient = {
    async listEnabledPlugins() { return { plugins: [plugin] }; },
    async installPlugin(_entry, targetDir) {
      olderEntered.resolve();
      await releaseOlder.promise;
      await writeSkill(targetDir, "older-partial");
      throw new Error("older install failed after timeout");
    },
  };
  const newer: MarketplaceLoaderClient = {
    async listEnabledPlugins() { return { plugins: [plugin] }; },
    async installPlugin(_entry, targetDir) { await writeSkill(targetDir, "successor-skill"); },
  };
  const olderPass = loadFromMarketplaceSource({ client: older, userId: "fixture-user", cacheManager: cache, pruneOldVersions: false });
  await olderEntered.promise;
  const newerPass = await loadFromMarketplaceSource({ client: newer, userId: "fixture-user", cacheManager: cache, pruneOldVersions: false });
  assert.equal(newerPass.failures.length, 0);
  assert.equal(newerPass.plugins[0]?.skills[0]?.name, "successor-skill");
  releaseOlder.resolve();
  const olderResult = await olderPass;
  assert.equal(olderResult.failures.length, 1);
  assert.equal(await cache.isCached({ marketplaceSlug: "cursor-public", pluginId: plugin.pluginId, version: plugin.version }), true);
  const published = cache.getCacheDir({ marketplaceSlug: "cursor-public", pluginId: plugin.pluginId, version: plugin.version });
  assert.match(await readFile(join(published, "SKILL.md"), "utf8"), /successor-skill instructions/);
});

test("SDK version-resolution failures reach the Host with their canonical DB ID", async (t) => {
  const cache = await cacheFixture(t);
  const failure = {
    pluginName: "7", pluginId: "7", pluginDbId: "41", marketplaceName: "cursor-public",
    errorMessage: "Repository not found", errorType: "clone" as const,
  };
  const result = await loadFromMarketplaceSource({
    client: {
      async listEnabledPlugins() { return { plugins: [], listFailures: [failure] }; },
      async installPlugin() { assert.fail("unresolved versions cannot be installed"); },
    },
    userId: "fixture-user",
    cacheManager: cache,
    pruneOldVersions: false,
  });

  assert.equal(result.sourceUnavailable, false);
  assert.deepEqual(result.failures, [failure]);
  assert.equal(result.plugins[0]?.identifier.sourceInfo.pluginDbId, "41");
  assert.equal(result.plugins[0]?.loadError, failure.errorMessage);
  assert.deepEqual(cache.prunes, []);
});

test("SDK late failed materialization preserves the completed winner and a fresh cache hit", { timeout: 10_000 }, async (t) => {
  const cache = await cacheFixture(t);
  const plugin = entry();
  const key = cacheKey(plugin);
  const targetDir = cache.getCacheDir(key);
  const entered = Promise.withResolvers<void>();
  const release = Promise.withResolvers<void>();
  const stages: string[] = [];
  let installs = 0;
  const client: MarketplaceLoaderClient = {
    async listEnabledPlugins() { return { plugins: [plugin] }; },
    async installPlugin(_plugin, stagingDir) {
      stages.push(stagingDir);
      installs += 1;
      if (installs === 1) {
        await writeSkill(stagingDir, "unfinished-skill");
        entered.resolve();
        await release.promise;
        throw new Error("late install failure after caller retirement");
      }
      await writeSkill(stagingDir, "winning-skill");
    },
  };
  const load = () => loadFromMarketplaceSource({
    client, userId: "fixture-user", pruneOldVersions: false,
    cacheManager: new DefaultPluginCacheManager(undefined, { cacheRoot: cache.cacheRoot }),
  });
  const older = load();
  try {
    await Promise.race([
      entered.promise,
      older.then(() => { throw new Error("The first load must remain inside installation."); }),
    ]);
    assert.equal(await cache.isCached(key), false);
    const winner = await load();
    const winnerBytes = "---\nname: winning-skill\ndescription: Fixture skill\n---\n\nwinning-skill instructions.\n";
    assert.deepEqual(winner.failures, []);
    assert.equal(winner.plugins[0]?.installPath, targetDir);
    assert.equal(winner.plugins[0]?.skills[0]?.path, "SKILL.md");
    assert.equal(winner.plugins[0]?.skills[0]?.content, winnerBytes);
    assert.equal(await readFile(join(targetDir, "SKILL.md"), "utf8"), winnerBytes);
    assert.equal(await readFile(join(targetDir, ".cache-complete"), "utf8"), "");
    assert.equal(stages.length, 2);
    assert.notEqual(stages[0], stages[1]);
    for (const stage of stages) {
      assert.equal(dirname(stage), dirname(cache.cacheRoot), "Rust pruning cannot traverse an in-progress sibling");
      assert.notEqual(stage, targetDir);
    }

    release.resolve();
    const failed = await older;
    assert.equal(failed.failures.length, 1);
    assert.equal(failed.failures[0]?.pluginDbId, plugin.pluginDbId);
    assert.match(failed.plugins[0]?.loadError ?? "", /late install failure/);
    assert.equal(failed.plugins[0]?.installPath, "");
    assert.equal(await readFile(join(targetDir, "SKILL.md"), "utf8"), winnerBytes);
    assert.equal(await readFile(join(targetDir, ".cache-complete"), "utf8"), "");
    assert.equal(await cache.isCached(key), true);
    for (const stage of stages) await assert.rejects(lstat(stage), { code: "ENOENT" });

    const fresh = await load();
    assert.deepEqual(fresh.failures, []);
    assert.equal(fresh.plugins[0]?.installPath, targetDir);
    assert.equal(fresh.plugins[0]?.skills[0]?.path, "SKILL.md");
    assert.equal(fresh.plugins[0]?.skills[0]?.content, winnerBytes);
    assert.equal(installs, 2, "a later refresh must consume the completed cache without reinstalling");
  } finally {
    release.resolve();
    await older;
  }
});

test("SDK concurrent successful materializations reuse the first completed directory", { timeout: 10_000 }, async (t) => {
  const cache = await cacheFixture(t);
  const plugin = entry();
  const targetDir = cache.getCacheDir(cacheKey(plugin));
  const entered = Promise.withResolvers<void>();
  const release = Promise.withResolvers<void>();
  const stages: string[] = [];
  let installs = 0;
  const client: MarketplaceLoaderClient = {
    async listEnabledPlugins() { return { plugins: [plugin] }; },
    async installPlugin(_plugin, stagingDir) {
      stages.push(stagingDir);
      installs += 1;
      if (installs === 1) {
        await writeSkill(stagingDir, "late-skill");
        await writeFile(join(stagingDir, "late-only.txt"), "must not enter the winner");
        entered.resolve();
        await release.promise;
      } else {
        await writeSkill(stagingDir, "winning-skill");
      }
    },
  };
  const load = () => loadFromMarketplaceSource({ client, userId: "fixture-user", cacheManager: cache, pruneOldVersions: false });
  const older = load();
  try {
    await Promise.race([
      entered.promise,
      older.then(() => { throw new Error("The earlier successful attempt must remain held."); }),
    ]);
    const winner = await load();
    const winnerBytes = "---\nname: winning-skill\ndescription: Fixture skill\n---\n\nwinning-skill instructions.\n";
    assert.deepEqual(winner.failures, []);
    assert.equal(winner.plugins[0]?.installPath, targetDir);
    assert.equal(winner.plugins[0]?.skills[0]?.content, winnerBytes);
    release.resolve();
    const late = await older;
    assert.deepEqual(late.failures, []);
    assert.equal(late.plugins[0]?.installPath, targetDir);
    assert.equal(late.plugins[0]?.skills[0]?.path, "SKILL.md");
    assert.equal(late.plugins[0]?.skills[0]?.content, winnerBytes);
    assert.equal(await readFile(join(targetDir, "SKILL.md"), "utf8"), winnerBytes);
    assert.equal(await readFile(join(targetDir, ".cache-complete"), "utf8"), "");
    await assert.rejects(lstat(join(targetDir, "late-only.txt")), { code: "ENOENT" });
    for (const stage of stages) await assert.rejects(lstat(stage), { code: "ENOENT" });
    const fresh = await load();
    assert.deepEqual(fresh.failures, []);
    assert.equal(fresh.plugins[0]?.skills[0]?.content, winnerBytes);
    assert.equal(installs, 2);
  } finally {
    release.resolve();
    await older;
  }
});

test("SDK incomplete and non-directory cache targets are preserved without installation", async (t) => {
  for (const kind of ["incomplete", "not-directory", "directory-marker", "target-symlink", "marker-symlink"] as const) {
    const cache = await cacheFixture(t);
    const plugin = entry();
    const key = cacheKey(plugin);
    const targetDir = cache.getCacheDir(key);
    await mkdir(dirname(targetDir), { recursive: true });
    let protectedPath = targetDir;
    let protectedLink: string | undefined;
    let linkedMarker: string | undefined;
    if (kind === "target-symlink") {
      const linkedDirectory = join(dirname(cache.cacheRoot), "linked-content");
      await mkdir(linkedDirectory);
      linkedMarker = join(linkedDirectory, ".cache-complete");
      await writeFile(linkedMarker, "linked completion-marker bytes");
      await symlink(linkedDirectory, targetDir, "dir");
      protectedLink = targetDir;
      protectedPath = join(linkedDirectory, "foreign.txt");
    } else if (kind !== "not-directory") {
      await mkdir(targetDir);
      protectedPath = join(targetDir, "foreign.txt");
      if (kind === "directory-marker") {
        await mkdir(join(targetDir, ".cache-complete"));
        protectedPath = join(targetDir, ".cache-complete", "foreign.txt");
      } else if (kind === "marker-symlink") {
        protectedPath = join(dirname(cache.cacheRoot), "linked-marker");
        protectedLink = join(targetDir, ".cache-complete");
        await symlink(protectedPath, protectedLink, "file");
      }
    }
    await writeFile(protectedPath, "foreign collision bytes");
    let installs = 0;
    const result = await loadFromMarketplaceSource({
      client: {
        async listEnabledPlugins() { return { plugins: [plugin] }; },
        async installPlugin() { installs += 1; },
      },
      userId: "fixture-user", cacheManager: cache, pruneOldVersions: false,
    });
    assert.equal(installs, 0, kind);
    assert.equal(result.failures.length, 1, kind);
    assert.equal(result.failures[0]?.pluginDbId, plugin.pluginDbId);
    assert.match(result.plugins[0]?.loadError ?? "", /Plugin cache target is/);
    assert.equal(result.plugins[0]?.installPath, "");
    assert.equal(await readFile(protectedPath, "utf8"), "foreign collision bytes");
    if (protectedLink !== undefined) assert.equal((await lstat(protectedLink)).isSymbolicLink(), true);
    if (linkedMarker !== undefined) assert.equal(await readFile(linkedMarker, "utf8"), "linked completion-marker bytes");
    assert.equal(await cache.isCached(key), false);
    assert.deepEqual(cache.prunes, []);
  }
});

test("SDK a target appearing during materialization is preserved without publication", async (t) => {
  const cache = await cacheFixture(t);
  const plugin = entry();
  const key = cacheKey(plugin);
  const targetDir = cache.getCacheDir(key);
  let stagingDir = "";
  const result = await loadFromMarketplaceSource({
    client: {
      async listEnabledPlugins() { return { plugins: [plugin] }; },
      async installPlugin(_plugin, stage) {
        stagingDir = stage;
        await writeSkill(stage, "unpublished-skill");
        await mkdir(targetDir, { recursive: true });
        await writeFile(join(targetDir, "foreign.txt"), "a different incomplete installation");
      },
    },
    userId: "fixture-user", cacheManager: cache, pruneOldVersions: false,
  });
  assert.equal(result.failures.length, 1);
  assert.match(result.plugins[0]?.loadError ?? "", /Plugin cache target is incomplete/);
  assert.equal(result.plugins[0]?.installPath, "");
  assert.equal(await readFile(join(targetDir, "foreign.txt"), "utf8"), "a different incomplete installation");
  await assert.rejects(lstat(join(targetDir, "SKILL.md")), { code: "ENOENT" });
  await assert.rejects(lstat(join(targetDir, ".cache-complete")), { code: "ENOENT" });
  await assert.rejects(lstat(stagingDir), { code: "ENOENT" });
  assert.equal(await cache.isCached(key), false);
  assert.deepEqual(cache.prunes, []);
});

test("SDK a failed materialization removes only its owned staging directory", async (t) => {
  const cache = await cacheFixture(t);
  const plugin = entry();
  const key = cacheKey(plugin);
  const oldSkill = await seedOldVersion(cache, plugin);
  const oldBytes = await readFile(oldSkill, "utf8");
  const unrelated = await mkdtemp(join(dirname(cache.cacheRoot), ".plugin-cache-install-"));
  await writeFile(join(unrelated, "foreign.txt"), "another attempt's bytes");
  let stagingDir = "";
  const result = await loadFromMarketplaceSource({
    client: {
      async listEnabledPlugins() { return { plugins: [plugin] }; },
      async installPlugin(_plugin, stage) {
        stagingDir = stage;
        await writeSkill(stage, "partial-skill");
        throw new Error("install failed after writing partial bytes");
      },
    },
    userId: "fixture-user", cacheManager: cache, pruneOldVersions: false,
  });
  assert.equal(result.failures.length, 1);
  assert.equal(result.failures[0]?.pluginDbId, plugin.pluginDbId);
  assert.match(result.plugins[0]?.loadError ?? "", /install failed after writing partial bytes/);
  assert.equal(result.plugins[0]?.installPath, "");
  assert.deepEqual(result.plugins[0]?.skills, []);
  assert.equal(dirname(stagingDir), dirname(cache.cacheRoot));
  assert.notEqual(stagingDir, unrelated);
  await assert.rejects(lstat(stagingDir), { code: "ENOENT" });
  await assert.rejects(lstat(cache.getCacheDir(key)), { code: "ENOENT" });
  assert.equal(await cache.isCached(key), false);
  assert.equal(await readFile(oldSkill, "utf8"), oldBytes);
  assert.equal(await readFile(join(unrelated, "foreign.txt"), "utf8"), "another attempt's bytes");
  assert.deepEqual(cache.prunes, []);
});

test("SDK marker and publication failures do not report a successful materialization", async (t) => {
  for (const failureAt of ["marker", "parent"] as const) {
    const cache = await cacheFixture(t);
    const plugin = entry();
    const key = cacheKey(plugin);
    const targetDir = cache.getCacheDir(key);
    let stagingDir = "";
    const result = await loadFromMarketplaceSource({
      client: {
        async listEnabledPlugins() { return { plugins: [plugin] }; },
        async installPlugin(_plugin, stage) {
          stagingDir = stage;
          await writeSkill(stage, "unpublished-skill");
          if (failureAt === "marker") {
            await writeFile(join(stage, ".cache-complete"), "plugin-provided marker");
          } else {
            await mkdir(dirname(dirname(targetDir)), { recursive: true });
            await writeFile(dirname(targetDir), "foreign final parent");
          }
        },
      },
      userId: "fixture-user", cacheManager: cache, pruneOldVersions: false,
    });
    assert.equal(result.failures.length, 1, failureAt);
    assert.equal(result.failures[0]?.pluginDbId, plugin.pluginDbId);
    assert.ok(result.plugins[0]?.loadError, failureAt);
    assert.equal(result.plugins[0]?.installPath, "");
    assert.deepEqual(result.plugins[0]?.skills, []);
    assert.equal(await cache.isCached(key), false);
    await assert.rejects(lstat(stagingDir), { code: "ENOENT" });
    if (failureAt === "parent") {
      assert.equal(await readFile(dirname(targetDir), "utf8"), "foreign final parent");
    } else {
      await assert.rejects(lstat(targetDir), { code: "ENOENT" });
    }
    assert.deepEqual(cache.prunes, []);
  }
});
