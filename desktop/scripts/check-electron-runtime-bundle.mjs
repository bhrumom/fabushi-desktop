import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { createHash } from "node:crypto";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, "..");
const noticeSource = "resources/licenses/openclaw";
const noticeDestination = "licenses/openclaw";

function parseOptions(args) {
  if (args.length === 0) return { mode: "runtime" };
  if (args.length === 1 && args[0] === "--notice-only") {
    return { mode: "notice-only" };
  }
  if (args.length === 2 && args[0] === "--packaged-resources"
      && args[1].trim() !== "" && !args[1].startsWith("--")) {
    return { mode: "packaged-resources", packagedResources: path.resolve(args[1]) };
  }
  throw new Error("Invalid or conflicting options. Use no arguments, --notice-only, or --packaged-resources PATH.");
}

function readRegularFile(filename, label) {
  try {
    if (!fs.lstatSync(filename).isFile()) throw new Error("Expected a regular file");
    return fs.readFileSync(filename);
  } catch (cause) {
    throw new Error(`${label} is missing, unreadable or not a regular file: ${filename}`, { cause });
  }
}

function readJson(filename, label) {
  const bytes = readRegularFile(filename, label);
  try {
    return JSON.parse(bytes.toString("utf8"));
  } catch (cause) {
    throw new Error(`${label} is not valid JSON: ${filename}`, { cause });
  }
}

function gitBlobIdentity(bytes) {
  return createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
}

function readNotice(root, relativeDirectory, label) {
  let directory = root;
  try {
    for (const segment of relativeDirectory.split("/")) {
      directory = path.join(directory, segment);
      if (!fs.lstatSync(directory).isDirectory()) throw new Error("Expected a real directory");
    }
  } catch (cause) {
    throw new Error(`${label} directory is missing, unreadable or not a real directory: ${directory}`, { cause });
  }
  return readRegularFile(path.join(directory, "LICENSE"), label);
}

// This is a conservative destination admission, not electron-builder's glob or
// copy engine. Only portable literal relative paths can prove disjointness.
function literalResourcePath(value) {
  if (typeof value !== "string") return null;
  const slashes = value.replaceAll("\\", "/");
  if (slashes.startsWith("/") || !/^[A-Za-z0-9._ /-]*$/.test(slashes)) return null;
  for (const segment of slashes.split("/")) {
    if (segment === "" || segment === "." || segment === "..") continue;
    if (/[. ]$/.test(segment) || /^(?:con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(segment)) return null;
  }
  const normalized = path.posix.normalize(slashes).replace(/\/$/, "").toLowerCase();
  if (normalized === ".." || normalized.startsWith("../")) return null;
  return normalized;
}

function isDisjointNoticeResource(entry) {
  // Pinned getFileMatchers combines common and platform declarations. A string
  // uses the project-root default matcher and preserves its relative path. We
  // admit only a literal string, never infer safety from a glob or exclusion.
  const destination = typeof entry === "string"
    ? literalResourcePath(entry)
    : entry !== null && typeof entry === "object" && !Array.isArray(entry)
      ? literalResourcePath(entry.to == null ? "." : entry.to)
      : null;
  if (destination === null || destination === ".") return false;
  const protectedPath = `${noticeDestination}/LICENSE`.toLowerCase();
  return destination !== protectedPath
    && !protectedPath.startsWith(`${destination}/`)
    && !destination.startsWith(`${protectedPath}/`);
}

function checkNoticeMapping(packageJson) {
  const resources = packageJson?.build?.extraResources;
  if (!Array.isArray(resources)) {
    throw new Error("OpenClaw notice requires one common extraResources mapping");
  }
  const mappings = resources.filter((entry) => entry !== null && typeof entry === "object"
    && !Array.isArray(entry) && entry.from === noticeSource && entry.to === noticeDestination
    && Array.isArray(entry.filter) && entry.filter.length === 1 && entry.filter[0] === "LICENSE"
    && Object.keys(entry).sort().join(",") === "filter,from,to");
  if (mappings.length !== 1) {
    throw new Error(`OpenClaw notice requires one common extraResources mapping; found ${mappings.length}`);
  }
  const resourceSets = [["build.extraResources", resources]];
  for (const platform of ["mac", "mas", "win", "linux"]) {
    const platformResources = packageJson.build[platform]?.extraResources;
    if (platformResources != null) {
      resourceSets.push([`build.${platform}.extraResources`, Array.isArray(platformResources) ? platformResources : [platformResources]]);
    }
  }
  for (const [label, entries] of resourceSets) {
    for (const [index, entry] of entries.entries()) {
      if (entries === resources && entry === mappings[0]) continue;
      if (!isDisjointNoticeResource(entry)) {
        throw new Error(`OpenClaw notice extraResources mapping at ${label}[${index}] may write or obscure licenses/openclaw/LICENSE; disjointness is not proven`);
      }
    }
  }
}

function checkOpenClawNotice(options) {
  const sourceLock = readJson(
    path.resolve(desktopRoot, "..", "projects/openclaw-rust/upstream.lock.json"),
    "OpenClaw source lock",
  );
  const expectedBlob = sourceLock?.root_inputs?.LICENSE;
  const sha = /^[0-9a-f]{40}$/;
  if (sourceLock?.format_version !== 1 || sourceLock?.project_id !== "OCRM-001"
      || sourceLock?.upstream?.repository !== "openclaw/openclaw"
      || typeof sourceLock?.upstream?.commit !== "string" || !sha.test(sourceLock.upstream.commit)
      || typeof sourceLock?.upstream?.tree !== "string" || !sha.test(sourceLock.upstream.tree)
      || typeof expectedBlob !== "string" || !sha.test(expectedBlob)) {
    throw new Error("OpenClaw source lock has an invalid upstream or root_inputs.LICENSE identity");
  }
  const sourceBytes = readNotice(desktopRoot, noticeSource, "OpenClaw source notice");
  const sourceBlob = gitBlobIdentity(sourceBytes);
  if (sourceBlob !== expectedBlob) {
    throw new Error(`OpenClaw source notice Git blob differs from root_inputs.LICENSE: ${sourceBlob}`);
  }
  checkNoticeMapping(readJson(path.join(desktopRoot, "package.json"), "Desktop package configuration"));

  const evidence = {
    check: "ONP-v1",
    mode: options.mode,
    scope: "OpenClaw root notice inputs and common resource mapping only",
    source_repository: sourceLock.upstream.repository,
    source_commit: sourceLock.upstream.commit,
    source_tree: sourceLock.upstream.tree,
    source_blob: sourceBlob,
    source_bytes: sourceBytes.length,
    resource_from: `${noticeSource}/LICENSE`,
    resource_to: `${noticeDestination}/LICENSE`,
  };
  if (options.mode === "packaged-resources") {
    try {
      if (!fs.lstatSync(options.packagedResources).isDirectory()) throw new Error("Expected a directory");
    } catch (cause) {
      throw new Error(`Packaged Resources must be an existing directory: ${options.packagedResources}`, { cause });
    }
    const packagedFile = path.join(options.packagedResources, noticeDestination, "LICENSE");
    const packagedBytes = readNotice(options.packagedResources, noticeDestination, "OpenClaw packaged notice");
    if (gitBlobIdentity(packagedBytes) !== expectedBlob || !packagedBytes.equals(sourceBytes)) {
      throw new Error("OpenClaw packaged notice differs from the locked source notice");
    }
    evidence.scope = "OpenClaw root notice inputs, common mapping and supplied packaged Resources only";
    evidence.packaged_notice = packagedFile;
    evidence.packaged_blob = expectedBlob;
  }
  return evidence;
}

function checkRuntimeArtifacts() {
  const required = [
    ["Electron main", "dist/electron-main/main.cjs"],
    ["generic Mini App native carrier", "dist/electron-main/miniapp-native-carrier.cjs"],
    ["primary preload", "dist/electron-preload/preload.cjs"],
    ["sandboxed Mini App preload", "dist/electron-preload/preload-miniapp-tools.cjs"],
    ["Rust Coordinator carrier", "dist/node-agent-coordinator/main.cjs"],
    ["Renderer HTML", "dist/renderer/index.html"],
  ];

  const missing = required.filter(([, relative]) =>
    !fs.existsSync(path.join(desktopRoot, relative)),
  );

  if (missing.length > 0) {
    console.error("Desktop runtime artifact graph is incomplete:");
    for (const [label, relative] of missing) {
      console.error(`- ${label}: ${relative}`);
    }
    process.exit(1);
  }

  const exposed = [];
  vm.runInNewContext(fs.readFileSync(path.join(desktopRoot, "dist/electron-preload/preload-miniapp-tools.cjs"), "utf8"), {
    module: { exports: {} },
    require(name) {
      if (name !== "electron") throw new Error(`Mini App preload requires forbidden module ${name}`);
      return { contextBridge: { exposeInMainWorld: (name, bridge) => exposed.push({ name, bridge }) }, ipcRenderer: { invoke: async () => ({ ok: true }) } };
    },
  }, { timeout: 1000 });
  if (exposed.length !== 1 || exposed[0].name !== "fabushi" || Object.keys(exposed[0].bridge).join() !== "invoke") {
    throw new Error("Mini App preload must expose exactly fabushi.invoke");
  }

  const coordinatorCarrierPath = path.join(
    desktopRoot,
    "dist/node-agent-coordinator/main.cjs",
  );
  const coordinatorCarrier = fs.readFileSync(coordinatorCarrierPath, "utf8");
  if (!coordinatorCarrier.includes("parentPort")) {
    console.error("Rust Coordinator carrier does not read the utility-process parent port.");
    process.exit(1);
  }
  if (/require\(["']electron["']\)\.parentPort/.test(coordinatorCarrier)) {
    console.error("Rust Coordinator carrier incorrectly reads electron.parentPort instead of process.parentPort.");
    process.exit(1);
  }
  if (!coordinatorCarrier.includes("process.parentPort")) {
    console.error("Rust Coordinator carrier must read process.parentPort in the emitted bundle.");
    process.exit(1);
  }
  if (!coordinatorCarrier.includes("MAHAYANA_APP_HOST_BIN")) {
    console.error("Rust Coordinator carrier must bind the Mahayana app-host binary into the child environment.");
    process.exit(1);
  }
  if (!coordinatorCarrier.includes("mahayana-app-host")) {
    console.error("Rust Coordinator carrier must resolve the staged Mahayana app-host executable.");
    process.exit(1);
  }

  const productMain = fs.readFileSync(path.join(desktopRoot, "dist/electron-main/main.cjs"), "utf8");
  for (const forbidden of ["FABUSHI_DANDELION_BIN", "dandelion-default.toml", "dandelion.native.apple.v1"]) {
    if (productMain.includes(forbidden)) throw new Error(`Application bundle contains Mini App-owned code: ${forbidden}`);
  }
  for (const filename of ["dandelion", "dandelion-default.toml", "dandelion-build.json"]) {
    if (fs.existsSync(path.join(desktopRoot, "resources/bin", filename))) throw new Error(`Application contains Mini App-owned resource ${filename}`);
  }

  console.log(
    "Desktop runtime artifact graph passed: Electron main, preload, Rust Coordinator carrier + app-host binary wiring, and Renderer are co-staged.",
  );
}

try {
  const options = parseOptions(process.argv.slice(2));
  const evidence = checkOpenClawNotice(options);
  if (options.mode === "runtime") checkRuntimeArtifacts();
  console.log(JSON.stringify(evidence));
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
