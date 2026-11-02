import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, lstat, mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

// NMR-AC-30 exercises the installed packager, not a replacement glob evaluator.
// electron-builder 26.15.3 release source:
// https://github.com/electron-userland/electron-builder/commit/512a57ec9bcda593d3e0970bd2b9a33a63beeb57
// PlatformPackager.getExtraFileMatchers/doPack call fileMatcher.getFileMatchers
// and copyFiles. The default extra-resource transformer is null. These tests
// qualify that copy boundary; they do not build/sign an installer or simulate
// Windows ACLs. Both filename conventions execute on the Actions runner's OS.
const desktopRoot = fileURLToPath(new URL("../", import.meta.url));
const require = createRequire(import.meta.url);
const { getFileMatchers, copyFiles } = require("app-builder-lib/out/fileMatcher.js");
const installedPackager = require("app-builder-lib/package.json");
const lock = JSON.parse(await readFile(path.join(desktopRoot, "package-lock.json"), "utf8"));
const lockedPackager = lock.packages["node_modules/app-builder-lib"];
assert.equal(lockedPackager?.version, "26.15.3", "review the resource-copy API when updating the locked packager");
assert.equal(installedPackager.version, lockedPackager.version, "use the npm-ci-installed locked implementation");
assert.equal(typeof getFileMatchers, "function");
assert.equal(typeof copyFiles, "function");

const productionPackage = JSON.parse(await readFile(path.join(desktopRoot, "package.json"), "utf8"));
const binFileSets = productionPackage.build.extraResources.filter(
  (entry) => entry !== null && typeof entry === "object" && entry.from === "resources/bin",
);
assert.equal(binFileSets.length, 1, "the canonical executable resource slot has exactly one FileSet");
const productionFileSet = binFileSets[0];
assert.equal(productionFileSet.to, "bin");

const copyImplementationSha256 = createHash("sha256")
  .update(await readFile(require.resolve("app-builder-lib/out/fileMatcher.js")))
  .digest("hex");
const executableBases = [
  "mahayana-app-host",
  "mahayana-node-agent-coordinator",
  "box-exec-daemon",
  "fabushi-network-worker",
];

function fixtureBytes(relativePath) {
  return Buffer.concat([
    Buffer.from(`NMR-39 retained fixture: ${relativePath}\n`, "utf8"),
    Buffer.from([0x00, 0xff, 0x01, 0x80, 0x7f]),
  ]);
}

async function putFixture(sourceBin, relativePath, mode) {
  const target = path.join(sourceBin, relativePath);
  await mkdir(path.dirname(target), { recursive: true });
  await writeFile(target, fixtureBytes(relativePath));
  await chmod(target, mode);
}

async function snapshotTree(root, relative = "") {
  const absolute = path.join(root, relative);
  const stat = await lstat(absolute);
  const entry = { path: relative, mode: stat.mode & 0o7777 };
  if (stat.isFile()) {
    return [{ ...entry, type: "file", bytes: (await readFile(absolute)).toString("hex") }];
  }
  assert.ok(stat.isDirectory(), `unexpected fixture entry type: ${relative}`);
  const entries = [{ ...entry, type: "directory" }];
  for (const name of (await readdir(absolute)).sort()) {
    entries.push(...await snapshotTree(root, relative === "" ? name : `${relative}/${name}`));
  }
  return entries;
}

async function copyResourceSlot(stageRoot, packageResources, fileSet) {
  await mkdir(packageResources, { recursive: true });
  const matchers = getFileMatchers(
    { extraResources: [fileSet] },
    "extraResources",
    packageResources,
    {
      // The production FileSet has literal paths/names and no ${...} macros.
      macroExpander: (value) => value,
      customBuildOptions: {},
      defaultSrc: stageRoot,
      globalOutDir: path.dirname(packageResources),
    },
  );
  assert.equal(matchers?.length, 1);
  await copyFiles(matchers, null);
  return await snapshotTree(packageResources);
}

async function setup(t) {
  const root = await mkdtemp(path.join(tmpdir(), "fabushi-native-package-"));
  t.after(async () => { await rm(root, { recursive: true, force: true }); });
  const stageRoot = path.join(root, "staging");
  const sourceBin = path.join(stageRoot, "resources", "bin");
  await mkdir(sourceBin, { recursive: true });
  t.diagnostic(JSON.stringify({
    packager: installedPackager.name,
    version: installedPackager.version,
    integrity: lockedPackager.integrity,
    copyImplementationSha256,
    runnerPlatform: process.platform,
  }));
  return { root, stageRoot, sourceBin };
}

for (const [label, suffix] of [["Unix", ""], ["Windows-named", ".exe"]]) {
  test(`NMR-39 ${label}: actual resource copy retains only canonical executable bytes and permissions`, async (t) => {
    const { root, stageRoot, sourceBin } = await setup(t);
    const allowedNames = executableBases.map((name) => `${name}${suffix}`);
    for (const [index, name] of allowedNames.entries()) {
      // These modes are already distributable under builder-util's documented
      // copy normalization and therefore must survive the real copy unchanged.
      await putFixture(sourceBin, name, index % 2 === 0 ? 0o755 : 0o555);
    }
    const excluded = [
      "libmahayana_runtime.so",
      "libmahayana_runtime.dylib",
      "libmahayana_runtime.a",
      "mahayana_runtime.dll",
      "libfabushi_telegram_runtime.so",
      "libfabushi_telegram_runtime.dylib",
      "fabushi_telegram_runtime.dll",
      "legacy-helper",
      "unknown-helper.exe",
      "mahayana-app-host.backup",
      "mahayana-app-host.exe.old",
      ".retained-private-file",
      `nested/${allowedNames[0]}`,
      "nested/deeper/libmahayana_runtime.so",
    ];
    for (const [index, name] of excluded.entries()) {
      await putFixture(sourceBin, name, index % 2 === 0 ? 0o700 : 0o600);
    }
    const original = await snapshotTree(sourceBin);

    // Causal control: the same real copy implementation with the historical
    // unrestricted FileSet does include the sentinels. No hidden built-in FFI
    // exclusion is allowed to make this production-filter oracle vacuous.
    const control = await copyResourceSlot(stageRoot, path.join(root, "control", "resources"), {
      from: productionFileSet.from,
      to: productionFileSet.to,
    });
    assert.deepEqual(
      control.filter((entry) => entry.type === "file").map((entry) => entry.path).sort(),
      [...allowedNames, ...excluded].map((name) => `bin/${name}`).sort(),
    );

    const copied = await copyResourceSlot(stageRoot, path.join(root, "package", "resources"), productionFileSet);
    assert.deepEqual(
      copied.filter((entry) => entry.type === "file").map((entry) => entry.path).sort(),
      allowedNames.map((name) => `bin/${name}`).sort(),
    );
    assert.deepEqual(
      copied.filter((entry) => entry.type === "directory").map((entry) => entry.path).sort(),
      ["", "bin"],
      "no excluded nested resource directory is created",
    );
    for (const name of allowedNames) {
      const before = original.find((entry) => entry.path === name);
      const after = copied.find((entry) => entry.path === `bin/${name}`);
      assert.equal(after?.bytes, before?.bytes, `${name} payload bytes`);
      assert.equal(after?.mode, before?.mode, `${name} permissions on this runner`);
    }
    assert.deepEqual(await snapshotTree(sourceBin), original, "all source bytes, permissions and names survive both copies");
  });
}

test("NMR-39 an approved executable name used as a directory cannot smuggle nested payload", async (t) => {
  const { root, stageRoot, sourceBin } = await setup(t);
  // FileMatcher adds <name>/**/* to literal patterns without a dot. Include
  // every approved Unix and Windows name as a directory to exercise that real
  // behavior and the production nested-file exclusion, not just a random path.
  const approvedDirectoryNames = executableBases.flatMap((name) => [name, `${name}.exe`]);
  for (const name of approvedDirectoryNames) {
    await putFixture(sourceBin, `${name}/legacy-runtime.so`, 0o700);
    await putFixture(sourceBin, `${name}/nested/unknown.exe`, 0o600);
  }
  const original = await snapshotTree(sourceBin);
  const control = await copyResourceSlot(stageRoot, path.join(root, "control", "resources"), {
    from: productionFileSet.from,
    to: productionFileSet.to,
  });
  assert.equal(control.filter((entry) => entry.type === "file").length, approvedDirectoryNames.length * 2);

  const copied = await copyResourceSlot(stageRoot, path.join(root, "package", "resources"), productionFileSet);
  assert.deepEqual(copied.map((entry) => entry.path), [""], "no directory or nested file enters the package slot");
  assert.deepEqual(await snapshotTree(sourceBin), original, "malformed staging directories are retained without mutation");
});
