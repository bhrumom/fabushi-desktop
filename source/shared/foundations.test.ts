import assert from "node:assert/strict";
import test from "node:test";

import {
  availableUpdateTracks,
  coerceToEnabledTrack,
  resolveEffectiveTrack,
  resolveReleaseTrackGate,
  selectableUpdateTracks,
} from "./update-track.js";
import {
  accountCacheScope,
  getAccessTokenExpiryMs,
  getAuthClientId,
  isTokenExpiringSoon,
  parseJwtPayload,
} from "./node/cursor-token.js";
import {
  isSameSandBoxMigrationOperation,
  parseSandBoxMigrationOperationId,
} from "./box-migration.js";
import { WriteEpoch } from "./write-epoch.js";
import { BoxSettingsField } from "./host-settings-field.js";
import { createReleaseMetadata } from "../electron-main/update/release-metadata.js";
import { createDesktopAccountAuthorizer } from "../electron-main/account/account-authorization.js";
import { createSandRecreateCommands } from "../electron-main/box/box-recreate-commands.js";
import { createDesktopHostSettingsFields } from "../electron-main/prefs/host-settings-fields.js";

function jwt(payload: Record<string, unknown>): string {
  return `x.${Buffer.from(JSON.stringify(payload)).toString("base64url")}.y`;
}

test("update track policy disables nightly and gates dogfood", () => {
  assert.deepEqual(selectableUpdateTracks(false), ["stable"]);
  assert.deepEqual(selectableUpdateTracks(true), ["stable", "dogfood"]);
  assert.deepEqual(availableUpdateTracks({ unlockInternalTracks: false, effectiveTrack: "dogfood" }), ["stable", "dogfood"]);
  assert.equal(coerceToEnabledTrack("nightly"), "stable");
  assert.deepEqual(resolveReleaseTrackGate({ releaseTrack: "dogfood", unlockInternalTracks: true }), {
    managedTrack: "dogfood",
    unlockInternalTracks: true,
  });
  assert.equal(resolveEffectiveTrack({ managedTrack: "nightly" }), "stable");
});

test("cursor token helpers scope accounts and validate expiry", () => {
  const token = jwt({ sub: "account-1", email: "a@example.com", exp: 4_000_000_000 });
  assert.deepEqual(parseJwtPayload(token), { sub: "account-1", email: "a@example.com", exp: 4_000_000_000 });
  assert.equal(accountCacheScope(token).length, 64);
  assert.equal(accountCacheScope(token), accountCacheScope(jwt({ sub: "account-1" })));
  assert.equal(getAccessTokenExpiryMs(token), 4_000_000_000_000);
  assert.equal(isTokenExpiringSoon(token, 1_000), false);
  assert.equal(isTokenExpiringSoon("malformed"), true);
  assert.notEqual(getAuthClientId("https://api2.cursor.sh"), getAuthClientId("http://localhost:3000"));
});

test("box migration ids compare only valid operation identities", () => {
  assert.deepEqual(parseSandBoxMigrationOperationId("op-1"), { value: "op-1" });
  assert.equal(parseSandBoxMigrationOperationId(""), null);
  assert.equal(isSameSandBoxMigrationOperation({ value: "a" }, { value: "a" }), true);
  assert.equal(isSameSandBoxMigrationOperation({ value: "a" }, { value: "b" }), false);
  assert.equal(isSameSandBoxMigrationOperation(null, { value: "a" }), false);
});

test("write epoch marks reads stale during and after writes and reset fences old settlers", () => {
  const epoch = new WriteEpoch();
  const before = epoch.snapshot();
  const settle = epoch.begin();
  assert.equal(epoch.isStale(before), true);
  const during = epoch.snapshot();
  settle();
  assert.equal(epoch.isStale(during), true);

  const staleSettle = epoch.begin();
  epoch.reset();
  const afterReset = epoch.snapshot();
  staleSettle();
  assert.equal(epoch.isStale(afterReset), false);
});

test("BoxSettingsField mirrors local values and rejects stale reads", async () => {
  let readable = true;
  let remote: { flag?: boolean } = {};
  let local: boolean | undefined = true;
  const field = new BoxSettingsField(
    {
      isReadable: () => readable,
      read: async () => remote,
      write: async (update) => {
        remote = { ...remote, ...update };
        return remote;
      },
    },
    "flag",
    {
      read: () => local,
      write: (value) => { local = value; },
      clear: () => { local = undefined; },
    },
  );

  assert.equal(await field.absorbFromBox(), "cleared");
  local = true;
  assert.deepEqual(await field.apply(false), { status: "persisted", value: false });
  assert.equal(local, false);
  assert.equal(await field.reconcile(), false);
  readable = false;
  assert.equal(await field.reconcile(), false);
});

test("release metadata and account authorization preserve policy boundaries", async () => {
  const metadata = createReleaseMetadata({
    packageVersion: "9.1.2",
    packageTrack: "dogfood",
    isLabBuild: false,
    app: { getVersion: () => "fallback", isPackaged: true },
    env: {},
    platform: "darwin",
  });
  assert.deepEqual(metadata.readAppReleaseMetadata(), { version: "9.1.2", buildDefaultTrack: "dogfood" });
  assert.equal(await metadata.computeUpdateDisabledReasonLive(), null);

  let scoped: string | null = null;
  let abandoned = 0;
  const authorizer = createDesktopAccountAuthorizer({
    hasExistingDurableData: async () => false,
    store: {
      getMcpCustomInstructionsAccountScope: () => "old-scope",
      scopeToAccount: (scope) => { scoped = scope; },
    },
    abandonForeignOnboardingMirror: () => { abandoned += 1; },
  });
  assert.equal(await authorizer("account-token", { isStartup: true }), true);
  assert.equal(scoped, accountCacheScope("account-token"));
  assert.equal(abandoned, 1);
});

test("recreate commands record trackable and untrackable accepted operations", async () => {
  const accepted: Array<{ value: string } | null> = [];
  const commands = createSandRecreateCommands<{ reason: string }>({
    connector: {
      recreate: async () => ({ status: "started", operationId: { value: "op-1" } }),
      forceRecreate: async () => ({ status: "started-untrackable" }),
    },
    noteRecreateAccepted: (operationId) => accepted.push(operationId),
  });
  assert.deepEqual(await commands.recreateComputer({ reason: "test" }), { status: "started", operationId: { value: "op-1" } });
  assert.deepEqual(await commands.forceRecreateComputer(), { status: "started-untrackable" });
  assert.deepEqual(accepted, [{ value: "op-1" }, null]);
});

test("desktop host settings field absorbs box values and clears account mirror", async () => {
  let box = { hasSeenOnboarding: true };
  let mirror: boolean | undefined = false;
  const reports: Array<Record<string, string>> = [];
  const fields = createDesktopHostSettingsFields({
    read: async () => box,
    write: async (update) => {
      box = { ...box, ...update };
      return box;
    },
    store: {
      getHasSeenOnboarding: () => mirror,
      setHasSeenOnboarding: (value) => { mirror = value; },
      clearHasSeenOnboarding: () => { mirror = undefined; },
    },
    reportPersistence: (event) => reports.push(event),
  });
  fields.onTransportConnected();
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.equal(mirror, true);
  assert.equal(reports[0]?.outcome, "ok");
  fields.onAccountDeparted();
  assert.equal(mirror, undefined);
});
