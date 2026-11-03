import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
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
import {
  ACCESS_TOKEN_SECRET_KEY,
  REFRESH_TOKEN_SECRET_KEY,
  SandCursorAuthService,
} from "../account/cursor-auth.js";
import {
  createHostPluginSkillsLoader,
  HOST_PLUGIN_SKILLS_CURRENT_USER_TIMEOUT_MS,
  HOST_PLUGIN_SKILLS_EFFECTIVE_PLUGINS_TIMEOUT_MS,
  type HostPluginSkillsDashboardClient,
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
) {
  const sandRootDir = await mkdtemp(join(tmpdir(), "fabushi-host-plugin-skills-"));
  t.after(async () => await rm(sandRootDir, { recursive: true, force: true }));
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
  assert.deepEqual(result.authBlocked, [{
    pluginId: "41", pluginName: "7",
  }]);
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
