import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import test from "node:test";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { SAND_DATA_ROOT_ENV } from "../../shared/node/sand-data-root.js";
import {
  bootstrapDesktopUserData,
  SAND_APP_DATA_DIR_ENV,
  SAND_TEMP_DIR_ENV,
} from "./desktop-user-data-bootstrap.js";

function fixture(env: NodeJS.ProcessEnv = {}) {
  const paths: Array<[string, string]> = [];
  const current: Record<string, string> = {
    appData: "/default/app-data",
    userData: "/default/user-data",
  };
  return {
    paths,
    current,
    app: {
      isPackaged: true,
      setPath(name: "appData" | "userData" | "sessionData" | "temp", path: string) {
        paths.push([name, path]);
        current[name] = path;
      },
      getPath(name: "appData" | "userData") { return current[name]!; },
    },
    env,
  };
}

test("appData override is opt-in and leaves ordinary desktop defaults unchanged", () => {
  const f = fixture({ FABUSHI_APP_DATA: "/existing/host-data", [SAND_DATA_ROOT_ENV]: "/existing/profile-data" });
  assert.equal(bootstrapDesktopUserData({ app: f.app, isLabBuild: false, platform: "darwin", env: f.env }), null);
  assert.deepEqual(f.paths, []);
  assert.equal(f.current.appData, "/default/app-data");
  assert.equal(f.env.FABUSHI_APP_DATA, "/existing/host-data");
  assert.equal(f.env[SAND_DATA_ROOT_ENV], "/existing/profile-data");
});

test("explicit RAM appData, userData and sessionData paths are applied before startup consumers", (t) => {
  const ramRoot = mkdtempSync(join(tmpdir(), "fabushi-miniapp-ram-contract-"));
  t.after(() => rmSync(ramRoot, { recursive: true, force: true }));
  const f = fixture({
    [SAND_APP_DATA_DIR_ENV]: join(ramRoot, "app-data"),
    [SAND_TEMP_DIR_ENV]: join(ramRoot, "tmp"),
    SAND_USER_DATA_DIR: join(ramRoot, "user-data"),
  });
  assert.equal(bootstrapDesktopUserData({ app: f.app, isLabBuild: false, platform: "darwin", env: f.env }), join(ramRoot, "user-data"));
  assert.deepEqual(f.paths, [
    ["appData", join(ramRoot, "app-data")],
    ["temp", join(ramRoot, "tmp")],
    ["userData", join(ramRoot, "user-data")],
    ["sessionData", join(ramRoot, "user-data")],
  ]);
  assert.equal(f.env.FABUSHI_APP_DATA, join(ramRoot, "app-data", "fabushi-runtime"));
  assert.equal(f.env[SAND_DATA_ROOT_ENV], join(ramRoot, "app-data", "sand-data"));
});

test("relative temp override fails closed instead of falling back to disk", () => {
  const f = fixture({ [SAND_TEMP_DIR_ENV]: "relative-temp-path" });
  assert.throws(
    () => bootstrapDesktopUserData({ app: f.app, isLabBuild: false, platform: "darwin", env: f.env }),
    /must be an absolute path/,
  );
  assert.deepEqual(f.paths, []);
});

test("relative appData override fails closed instead of silently using persistent storage", () => {
  const f = fixture({ [SAND_APP_DATA_DIR_ENV]: "relative-ram-path" });
  assert.throws(
    () => bootstrapDesktopUserData({ app: f.app, isLabBuild: false, platform: "darwin", env: f.env }),
    /must be an absolute path/,
  );
  assert.deepEqual(f.paths, []);
});
