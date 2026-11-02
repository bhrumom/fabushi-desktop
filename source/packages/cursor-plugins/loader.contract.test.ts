import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
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
  return new ObservedCacheManager(undefined, { cacheRoot: root });
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
