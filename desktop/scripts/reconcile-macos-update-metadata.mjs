import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { chmod, lstat, mkdtemp, readFile, realpath, rename, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { gunzipSync } from "node:zlib";

// NMR-43: use the same external blockmap producer as the locked packager.
// Reviewed upstream source: electron-userland/electron-builder@
// 512a57ec9bcda593d3e0970bd2b9a33a63beeb57, packages/app-builder-lib/src/
// targets/blockmap/blockmap.ts; npm-ci's 26.15.3 remains the executed authority.
// An omitted outFile would append a map to the signed input; never omit it.
const require = createRequire(import.meta.url);
const { buildBlockMap } = require("app-builder-lib/out/targets/blockmap/blockmap.js");
const yaml = require("js-yaml");
const desktopRoot = fileURLToPath(new URL("../", import.meta.url));
const expectedPackagerVersion = "26.15.3";
const metadataName = "latest-mac.yml";
const maximumDecodedBlockmapBytes = 64 * 1024 * 1024;

function fail(code) { throw new Error(code); }
function record(value) { return value !== null && typeof value === "object" && !Array.isArray(value); }
function validDigest(value) {
  return typeof value === "string" && /^[A-Za-z0-9+/]{86}==$/.test(value)
    && Buffer.from(value, "base64").length === 64
    && Buffer.from(value, "base64").toString("base64") === value;
}

async function requireLockedGenerator() {
  const lock = JSON.parse(await readFile(path.join(desktopRoot, "package-lock.json"), "utf8"));
  const installed = require("app-builder-lib/package.json");
  const locked = lock.packages?.["node_modules/app-builder-lib"];
  const lockedYaml = lock.packages?.["node_modules/js-yaml"];
  if (locked?.version !== expectedPackagerVersion || installed.version !== locked.version
      || require("js-yaml/package.json").version !== lockedYaml?.version
      || typeof buildBlockMap !== "function") fail("MACOS_METADATA_GENERATOR_NOT_LOCKED");
  return { version: installed.version, integrity: locked.integrity };
}

async function regularFile(filename, limit = Number.MAX_SAFE_INTEGER) {
  let stat;
  try { stat = await lstat(filename); } catch { fail("MACOS_METADATA_FILE_MISSING"); }
  if (!stat.isFile() || stat.isSymbolicLink() || !Number.isSafeInteger(stat.size)
      || stat.size <= 0 || stat.size > limit) fail("MACOS_METADATA_FILE_INVALID");
  return stat;
}

function unchanged(before, after) {
  return after.isFile() && !after.isSymbolicLink()
    && ["dev", "ino", "size", "mtimeMs", "ctimeMs"].every((key) => before[key] === after[key]);
}

async function hashes(filename) {
  const sha512 = createHash("sha512");
  const sha256 = createHash("sha256");
  let size = 0;
  for await (const chunk of createReadStream(filename)) {
    size += chunk.length;
    sha512.update(chunk);
    sha256.update(chunk);
  }
  return { size, sha512: sha512.digest("base64"), sha256: sha256.digest("hex") };
}

async function preflight({ directory, version, targetSha }) {
  if (typeof version !== "string"
      || !/^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z]+(?:[.-][0-9A-Za-z]+)*)?(?:\+[0-9A-Za-z]+(?:[.-][0-9A-Za-z]+)*)?$/.test(version)
      || typeof targetSha !== "string" || !/^[0-9a-f]{40}$/.test(targetSha)
      || typeof directory !== "string" || !directory || directory.includes("\0")) fail("MACOS_METADATA_INPUT_INVALID");
  const generator = await requireLockedGenerator();
  const inputDirectory = path.resolve(directory);
  const directoryStat = await lstat(inputDirectory);
  if (!directoryStat.isDirectory() || directoryStat.isSymbolicLink()) fail("MACOS_METADATA_DIRECTORY_INVALID");
  const root = await realpath(inputDirectory);
  const metadataPath = path.join(root, metadataName);
  const metadataStat = await regularFile(metadataPath, 128 * 1024);
  const metadataBytes = await readFile(metadataPath);
  let metadata;
  try { metadata = yaml.load(metadataBytes.toString("utf8"), { schema: yaml.JSON_SCHEMA }); }
  catch { fail("MACOS_METADATA_MALFORMED"); }
  const names = [`fabushi-${version}-macos-arm64.zip`, `fabushi-${version}-macos-arm64.dmg`];
  if (!record(metadata) || metadata.version !== version) fail("MACOS_METADATA_VERSION_MISMATCH");
  if (!Array.isArray(metadata.files) || metadata.files.length !== names.length
      || metadata.path !== names[0] || !validDigest(metadata.sha512)) fail("MACOS_METADATA_ENTRIES_INVALID");
  const seen = new Set();
  for (const entry of metadata.files) {
    if (!record(entry) || !names.includes(entry.url) || seen.has(entry.url)
        || !validDigest(entry.sha512) || !Number.isSafeInteger(entry.size) || entry.size <= 0) {
      fail("MACOS_METADATA_ENTRIES_INVALID");
    }
    seen.add(entry.url);
  }
  const assets = [];
  for (const name of names) {
    const filename = path.join(root, name);
    const sidecar = `${filename}.blockmap`;
    assets.push({ name, filename, sidecar,
      stat: await regularFile(filename), sidecarStat: await regularFile(sidecar, 64 * 1024 * 1024) });
  }
  // Every input (including both existing sidecars) is inspected before any
  // temporary/output write. Failed preflight leaves the candidate untouched.
  return { root, version, targetSha, generator, metadataPath, metadataStat, metadataBytes, metadata, assets };
}

async function requireUnchangedInputs(input) {
  for (const [filename, before] of [
    [input.metadataPath, input.metadataStat],
    ...input.assets.flatMap((asset) => [[asset.filename, asset.stat], [asset.sidecar, asset.sidecarStat]]),
  ]) {
    if (!unchanged(before, await lstat(filename))) fail("MACOS_METADATA_INPUT_CHANGED");
  }
  if (!(await readFile(input.metadataPath)).equals(input.metadataBytes)) fail("MACOS_METADATA_INPUT_CHANGED");
}

async function generateFinalMaps(input, temporary) {
  const results = [];
  for (const asset of input.assets) {
    const mapPath = path.join(temporary, `${asset.name}.blockmap`);
    const generated = await buildBlockMap(asset.filename, "gzip", mapPath);
    const identity = await hashes(asset.filename);
    if (!unchanged(asset.stat, await lstat(asset.filename)) || generated.size !== identity.size
        || generated.sha512 !== identity.sha512) fail("MACOS_METADATA_INPUT_CHANGED");
    const blockmap = await hashes(mapPath);
    results.push({ ...asset, mapPath, identity, blockmap });
  }
  return results;
}

async function decodedBlockmap(filename) {
  try {
    return gunzipSync(await readFile(filename), { maxOutputLength: maximumDecodedBlockmapBytes });
  } catch { fail("MACOS_METADATA_BLOCKMAP_MISMATCH"); }
}

async function verifyPrepared(input, temporary) {
  const results = await generateFinalMaps(input, temporary);
  for (const asset of results) {
    const entry = input.metadata.files.find((file) => file.url === asset.name);
    if (entry.size !== asset.identity.size || entry.sha512 !== asset.identity.sha512) fail("MACOS_METADATA_ASSET_MISMATCH");
    // The gzip envelope (including its OS byte) and compression level can
    // differ across builders. Require identical bounded map content, while
    // receipts identify the actual sidecar accepted by this read-only check.
    if (!(await decodedBlockmap(asset.sidecar)).equals(await decodedBlockmap(asset.mapPath))) fail("MACOS_METADATA_BLOCKMAP_MISMATCH");
    asset.blockmap = await hashes(asset.sidecar);
  }
  if (input.metadata.sha512 !== results[0].identity.sha512) fail("MACOS_METADATA_LEGACY_HASH_MISMATCH");
  if (Object.hasOwn(input.metadata, "size") && input.metadata.size !== results[0].identity.size) fail("MACOS_METADATA_LEGACY_SIZE_MISMATCH");
  await requireUnchangedInputs(input);
  return {
    kind: "fabushi-final-macos-update-metadata",
    sourceSha: input.targetSha,
    version: input.version,
    generator: input.generator,
    metadata: { name: metadataName, ...(await hashes(input.metadataPath)) },
    assets: results.map((asset) => ({ name: asset.name, ...asset.identity,
      blockmap: { name: `${asset.name}.blockmap`, ...asset.blockmap } })),
  };
}

export async function verifyMacUpdateMetadata(options) {
  const input = await preflight(options);
  const temporary = await mkdtemp(path.join(tmpdir(), "fabushi-macos-metadata-verify-"));
  try { return await verifyPrepared(input, temporary); }
  finally { await rm(temporary, { recursive: true, force: true }); }
}

export async function reconcileMacUpdateMetadata(options) {
  const input = await preflight(options);
  // Same-directory temporary files make each rename atomic. Metadata is the
  // last commit point; interruption cannot produce a success receipt. Repeating
  // reconciliation regenerates all maps and repairs any partial prior attempt.
  const temporary = await mkdtemp(path.join(input.root, ".fabushi-macos-metadata-"));
  try {
    const results = await generateFinalMaps(input, temporary);
    const next = { ...input.metadata,
      files: input.metadata.files.map((entry) => {
        const asset = results.find((item) => item.name === entry.url);
        return { ...entry, sha512: asset.identity.sha512, size: asset.identity.size };
      }),
      path: results[0].name, sha512: results[0].identity.sha512,
    };
    if (Object.hasOwn(next, "size")) next.size = results[0].identity.size;
    const stagedMetadata = path.join(temporary, metadataName);
    await writeFile(stagedMetadata, yaml.dump(next, { schema: yaml.JSON_SCHEMA, noRefs: true, lineWidth: -1 }),
      { flag: "wx", mode: input.metadataStat.mode & 0o777 });
    for (const asset of results) await chmod(asset.mapPath, asset.sidecarStat.mode & 0o777);
    await requireUnchangedInputs(input);
    for (const asset of results) await rename(asset.mapPath, asset.sidecar);
    await rename(stagedMetadata, input.metadataPath);
  } finally { await rm(temporary, { recursive: true, force: true }); }
  // Read the actual published metadata and sidecars afresh and rebuild expected
  // maps independently. Only this successful verification permits a receipt.
  return await verifyMacUpdateMetadata(options);
}

function argumentsFrom(argv) {
  const options = {};
  const keys = new Map([["--directory", "directory"], ["--version", "version"],
    ["--target-sha", "targetSha"], ["--receipt", "receipt"]]);
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (flag === "--verify" && !options.verify) { options.verify = true; continue; }
    const key = keys.get(flag);
    if (!key || Object.hasOwn(options, key) || !argv[index + 1] || argv[index + 1].startsWith("--")) fail("MACOS_METADATA_ARGUMENTS_INVALID");
    options[key] = argv[++index];
  }
  if (!options.directory || !options.version || !options.targetSha
      || (!options.verify && !options.receipt)) fail("MACOS_METADATA_ARGUMENTS_INVALID");
  return options;
}

async function runCli() {
  const options = argumentsFrom(process.argv.slice(2));
  const root = await realpath(path.resolve(options.directory));
  if (options.receipt) {
    const receiptPath = path.resolve(options.receipt);
    // Keep evidence separate from immutable delivery files. Verify mode itself
    // writes no candidate files; an explicitly requested receipt is an output.
    const receiptParent = await realpath(path.dirname(receiptPath));
    if (receiptParent === root || receiptParent.startsWith(`${root}${path.sep}`)) fail("MACOS_METADATA_RECEIPT_PATH_INVALID");
    try {
      const receiptStat = await lstat(receiptPath);
      if (!receiptStat.isFile() || receiptStat.isSymbolicLink()) fail("MACOS_METADATA_RECEIPT_PATH_INVALID");
    }
    catch (error) { if (error.code !== "ENOENT") throw error; }
  }
  const result = await (options.verify ? verifyMacUpdateMetadata(options) : reconcileMacUpdateMetadata(options));
  const serialized = `${JSON.stringify(result, null, 2)}\n`;
  if (options.receipt) {
    const destination = path.resolve(options.receipt);
    const temporary = await mkdtemp(path.join(path.dirname(destination), ".fabushi-metadata-receipt-"));
    try {
      const staged = path.join(temporary, "receipt.json");
      await writeFile(staged, serialized, { flag: "wx" });
      await rename(staged, destination);
    } finally { await rm(temporary, { recursive: true, force: true }); }
  }
  process.stdout.write(serialized);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  runCli().catch((error) => {
    process.stderr.write(`${error?.message || "MACOS_METADATA_FAILED"}\n`);
    process.exitCode = 1;
  });
}
