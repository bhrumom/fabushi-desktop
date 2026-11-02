import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFile, lstat, mkdir, mkdtemp, readFile, readdir, readlink, rm, symlink, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { reconcileMacUpdateMetadata, verifyMacUpdateMetadata } from "./reconcile-macos-update-metadata.mjs";

const require = createRequire(import.meta.url);
const { buildBlockMap } = require("app-builder-lib/out/targets/blockmap/blockmap.js");
const yaml = require("js-yaml");
const version = "1.2.78";
// A synthetic source identity tests receipt binding; this is not a release SHA.
const targetSha = "1234567890abcdef1234567890abcdef12345678";
const cli = fileURLToPath(new URL("./reconcile-macos-update-metadata.mjs", import.meta.url));
const desktopRoot = fileURLToPath(new URL("../", import.meta.url));

function content(length, salt) {
  return Buffer.from(Array.from({ length }, (_, index) => (index * 31 + (index >>> 9) + salt) & 255));
}

async function setup(t) {
  const root = await mkdtemp(path.join(tmpdir(), "fabushi-macos-metadata-case-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const directory = path.join(root, "release");
  await mkdir(directory);
  const zip = `fabushi-${version}-macos-arm64.zip`;
  const dmg = `fabushi-${version}-macos-arm64.dmg`;
  await writeFile(path.join(directory, zip), content(66000, 9));
  await writeFile(path.join(directory, dmg), content(97000, 47));
  const files = [];
  for (const name of [zip, dmg]) {
    const file = path.join(directory, name);
    const generated = await buildBlockMap(file, "gzip", `${file}.blockmap`);
    files.push({ url: name, ...generated, retainedField: `retained-${name}` });
  }
  const metadata = {
    version, files, path: zip, sha512: files[0].sha512,
    releaseDate: "2026-11-01T17:00:02.582Z", releaseNotes: "Retain the declared release notes.",
  };
  const metadataPath = path.join(directory, "latest-mac.yml");
  await writeFile(metadataPath, yaml.dump(metadata));
  return { root, directory, zip, dmg, metadata, metadataPath, options: { directory, version, targetSha } };
}

async function writeMetadata(fixture) {
  await writeFile(fixture.metadataPath, yaml.dump(fixture.metadata));
}

async function snapshot(directory, relative = "") {
  const filename = path.join(directory, relative);
  const stat = await lstat(filename);
  const row = { path: relative, mode: stat.mode & 0o7777 };
  if (stat.isSymbolicLink()) return [{ ...row, kind: "symlink", target: await readlink(filename) }];
  if (stat.isFile()) return [{ ...row, kind: "file", bytes: (await readFile(filename)).toString("base64") }];
  assert.ok(stat.isDirectory());
  const rows = [{ ...row, kind: "directory" }];
  for (const name of (await readdir(filename)).sort()) rows.push(...await snapshot(directory, path.join(relative, name)));
  return rows;
}

async function rejectWithoutMutation(fixture, error, options = fixture.options) {
  const before = await snapshot(fixture.directory);
  await assert.rejects(reconcileMacUpdateMetadata(options), error);
  assert.deepEqual(await snapshot(fixture.directory), before);
}

test("final ZIP and DMG metadata uses the locked generator and verify leaves all candidate bytes unchanged", async (t) => {
  const f = await setup(t);
  const before = await snapshot(f.directory);
  const report = await verifyMacUpdateMetadata(f.options);
  assert.equal(report.sourceSha, targetSha);
  assert.equal(report.generator.version, "26.15.3");
  assert.equal(report.assets.length, 2);
  assert.deepEqual(report.assets.map((asset) => asset.name), [f.zip, f.dmg]);
  assert.deepEqual(await snapshot(f.directory), before);
  t.diagnostic(JSON.stringify({ generator: report.generator, boundary: "actual external blockmap generation; synthetic package bytes" }));
});

test("ticket append rejects pre-staple metadata then reconciles final hashes, sizes and maps without altering packages", async (t) => {
  const f = await setup(t);
  const dmg = path.join(f.directory, f.dmg);
  const originalMap = await readFile(`${dmg}.blockmap`);
  const zip = await readFile(path.join(f.directory, f.zip));
  // Simulates the observed 2143-byte size increase, without calling Apple tools.
  await appendFile(dmg, content(2143, 111));
  const finalDmg = await readFile(dmg);
  const before = await snapshot(f.directory);
  await assert.rejects(verifyMacUpdateMetadata(f.options), /MACOS_METADATA_ASSET_MISMATCH/);
  assert.deepEqual(await snapshot(f.directory), before);
  const receipt = await reconcileMacUpdateMetadata(f.options);
  const updated = yaml.load(await readFile(f.metadataPath, "utf8"));
  assert.equal(updated.files[1].size, 99143);
  assert.equal(updated.files[1].sha512, createHash("sha512").update(finalDmg).digest("base64"));
  assert.equal(updated.path, f.zip);
  assert.equal(updated.sha512, createHash("sha512").update(zip).digest("base64"));
  assert.equal(updated.releaseDate, f.metadata.releaseDate);
  assert.equal(updated.releaseNotes, f.metadata.releaseNotes);
  assert.equal(updated.files[1].retainedField, f.metadata.files[1].retainedField);
  assert.ok(!(await readFile(`${dmg}.blockmap`)).equals(originalMap));
  assert.deepEqual(await readFile(dmg), finalDmg);
  assert.deepEqual(await readFile(path.join(f.directory, f.zip)), zip);
  assert.deepEqual(await verifyMacUpdateMetadata(f.options), receipt);
});

test("a same-size byte change with manually refreshed hash still rejects its stale differential map", async (t) => {
  const f = await setup(t);
  const dmg = path.join(f.directory, f.dmg);
  const bytes = await readFile(dmg);
  bytes[43000] ^= 0xff;
  await writeFile(dmg, bytes);
  await assert.rejects(verifyMacUpdateMetadata(f.options), /MACOS_METADATA_ASSET_MISMATCH/);
  f.metadata.files[1].sha512 = createHash("sha512").update(bytes).digest("base64");
  await writeMetadata(f);
  const before = await snapshot(f.directory);
  await assert.rejects(verifyMacUpdateMetadata(f.options), /MACOS_METADATA_BLOCKMAP_MISMATCH/);
  assert.deepEqual(await snapshot(f.directory), before);
  await reconcileMacUpdateMetadata(f.options);
  await verifyMacUpdateMetadata(f.options);
  assert.deepEqual(await readFile(dmg), bytes);
});

test("a nonempty but unrelated blockmap is rejected and rebuilt from its own package", async (t) => {
  const f = await setup(t);
  await writeFile(path.join(f.directory, `${f.dmg}.blockmap`), await readFile(path.join(f.directory, `${f.zip}.blockmap`)));
  await assert.rejects(verifyMacUpdateMetadata(f.options), /MACOS_METADATA_BLOCKMAP_MISMATCH/);
  const assetsBefore = await Promise.all([f.zip, f.dmg].map((name) => readFile(path.join(f.directory, name))));
  await reconcileMacUpdateMetadata(f.options);
  assert.deepEqual(await Promise.all([f.zip, f.dmg].map((name) => readFile(path.join(f.directory, name)))), assetsBefore);
});

test("missing package, sidecar or metadata fails preflight without touching the remaining files", async (t) => {
  for (const missing of ["package", "sidecar", "metadata"]) {
    const f = await setup(t);
    await rm(missing === "metadata" ? f.metadataPath : path.join(f.directory, `${f.dmg}${missing === "sidecar" ? ".blockmap" : ""}`));
    await rejectWithoutMutation(f, /MACOS_METADATA_FILE_MISSING/);
  }
});

test("duplicate file entries cannot omit one of the required package identities", async (t) => {
  const f = await setup(t);
  f.metadata.files[1] = { ...f.metadata.files[0] };
  await writeMetadata(f);
  await rejectWithoutMutation(f, /MACOS_METADATA_ENTRIES_INVALID/);
});

test("a metadata or expected-version mismatch cannot rewrite another release", async (t) => {
  const f = await setup(t);
  await rejectWithoutMutation(f, /MACOS_METADATA_VERSION_MISMATCH/, { ...f.options, version: "1.2.79" });
  f.metadata.version = "1.2.77";
  await writeMetadata(f);
  await rejectWithoutMutation(f, /MACOS_METADATA_VERSION_MISMATCH/);
});

test("file and legacy paths cannot escape the canonical arm64 release filenames", async (t) => {
  for (const field of ["file", "legacy"]) {
    const f = await setup(t);
    if (field === "file") f.metadata.files[1].url = `../${f.dmg}`;
    else f.metadata.path = `../${f.zip}`;
    await writeMetadata(f);
    await rejectWithoutMutation(f, /MACOS_METADATA_ENTRIES_INVALID/);
  }
});

test("symlinked package, sidecar and metadata inputs are refused before writes", async (t) => {
  for (const kind of ["package", "sidecar", "metadata"]) {
    const f = await setup(t);
    const input = kind === "metadata" ? f.metadataPath : path.join(f.directory, `${f.dmg}${kind === "sidecar" ? ".blockmap" : ""}`);
    const outside = path.join(f.root, `external-${kind}`);
    const bytes = await readFile(input);
    await writeFile(outside, bytes);
    await rm(input);
    await symlink(outside, input);
    await rejectWithoutMutation(f, /MACOS_METADATA_FILE_INVALID/);
    assert.deepEqual(await readFile(outside), bytes);
  }
});

test("directories and empty files cannot stand in for regular package inputs", async (t) => {
  for (const kind of ["directory", "empty"]) {
    const f = await setup(t);
    const input = path.join(f.directory, `${f.dmg}.blockmap`);
    await rm(input);
    if (kind === "directory") await mkdir(input);
    else await writeFile(input, "");
    await rejectWithoutMutation(f, /MACOS_METADATA_FILE_INVALID/);
  }
});

test("duplicate YAML keys and malformed digest fields fail before sidecar mutation", async (t) => {
  const f = await setup(t);
  await appendFile(f.metadataPath, `version: ${version}\n`);
  await rejectWithoutMutation(f, /MACOS_METADATA_MALFORMED/);
  f.metadata.files[1].sha512 = "not-a-digest";
  await writeMetadata(f);
  await rejectWithoutMutation(f, /MACOS_METADATA_ENTRIES_INVALID/);
});

test("receipt identity must be an exact complete source SHA", async (t) => {
  const f = await setup(t);
  for (const invalid of [undefined, "main", targetSha.slice(0, 12)]) {
    await rejectWithoutMutation(f, /MACOS_METADATA_INPUT_INVALID/, { ...f.options, targetSha: invalid });
  }
});

test("partly replaced sidecars from an interrupted reconciliation remain repairable", async (t) => {
  const f = await setup(t);
  const dmg = path.join(f.directory, f.dmg);
  await appendFile(dmg, content(2143, 39));
  // A previous attempt replaced this sidecar but had not committed metadata.
  await buildBlockMap(dmg, "gzip", `${dmg}.blockmap`);
  await assert.rejects(verifyMacUpdateMetadata(f.options), /MACOS_METADATA_ASSET_MISMATCH/);
  const packageBefore = await readFile(dmg);
  const first = await reconcileMacUpdateMetadata(f.options);
  const second = await reconcileMacUpdateMetadata(f.options);
  assert.deepEqual(second, first);
  assert.deepEqual(await readFile(dmg), packageBefore);
});

test("legacy ZIP hash is checked independently and reconciles without changing release fields", async (t) => {
  const f = await setup(t);
  f.metadata.sha512 = f.metadata.files[1].sha512;
  await writeMetadata(f);
  await assert.rejects(verifyMacUpdateMetadata(f.options), /MACOS_METADATA_LEGACY_HASH_MISMATCH/);
  await reconcileMacUpdateMetadata(f.options);
  const actual = yaml.load(await readFile(f.metadataPath, "utf8"));
  assert.equal(actual.sha512, actual.files[0].sha512);
  assert.equal(actual.releaseNotes, f.metadata.releaseNotes);
});

test("production CLI writes an exact-source receipt only on success and exposes a read-only verify mode", async (t) => {
  const f = await setup(t);
  await appendFile(path.join(f.directory, f.dmg), content(2143, 6));
  const receipt = path.join(f.root, "receipt.json");
  const argumentsBase = [cli, "--directory", f.directory, "--version", version, "--target-sha", targetSha];
  const call = (args) => spawnSync(process.execPath, args, { cwd: desktopRoot, encoding: "utf8" });
  const success = call([...argumentsBase, "--receipt", receipt]);
  assert.equal(success.status, 0, success.stderr);
  const report = JSON.parse(await readFile(receipt, "utf8"));
  assert.equal(report.sourceSha, targetSha);
  assert.equal(report.assets[1].size, 99143);
  assert.deepEqual(JSON.parse(success.stdout), report);
  const before = await snapshot(f.directory);
  const verified = call([...argumentsBase, "--verify"]);
  assert.equal(verified.status, 0, verified.stderr);
  assert.deepEqual(JSON.parse(verified.stdout), report);
  assert.deepEqual(await snapshot(f.directory), before);
  const neverWritten = path.join(f.root, "failed-receipt.json");
  const failed = call([...argumentsBase, "--receipt", neverWritten, "--unknown"]);
  assert.equal(failed.status, 1);
  assert.match(failed.stderr, /MACOS_METADATA_ARGUMENTS_INVALID/);
  await assert.rejects(lstat(neverWritten), { code: "ENOENT" });
  assert.deepEqual(await snapshot(f.directory), before);
  const unsafe = call([...argumentsBase, "--receipt", f.metadataPath]);
  assert.equal(unsafe.status, 1);
  assert.match(unsafe.stderr, /MACOS_METADATA_RECEIPT_PATH_INVALID/);
  assert.deepEqual(await snapshot(f.directory), before);
});
