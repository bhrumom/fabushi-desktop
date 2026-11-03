import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { createHash } from "node:crypto";
import { verifyLocalExecDependencyStage } from "./build-local-exec-dependencies.mjs";

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
    ["selected LocalExec daemon", "dist/local-exec-daemon/main.cjs"],
    ["LocalExec entry input manifest", "dist/local-exec-daemon/build-metafile.json"],
    ["LocalExec build identity", "dist/local-exec-daemon/build-manifest.json"],
    ["LocalExec source notices", "dist/local-exec-daemon/licenses/source-inputs.json"],
    ["LocalExec Node parser stage", "dist/node-deps/local-exec-dependencies.json"],
    ["LocalExec Electron parser stage", "dist/deps/local-exec-dependencies.json"],
    ["separate charset library", "dist/node_modules/jschardet/package.json"],
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


}


function artifactBytes(relative) {
  if (typeof relative !== "string" || relative === "" || relative.includes("\\")
      || relative.startsWith("/") || path.posix.normalize(relative) !== relative
      || relative.split("/").some(part => part === "" || part === "." || part === "..")) {
    throw new Error("Invalid LocalExec artifact path");
  }
  const parts = relative.split("/");
  let directory = desktopRoot;
  for (const part of parts.slice(0, -1)) {
    directory = path.join(directory, part);
    if (!fs.lstatSync(directory).isDirectory()) throw new Error("LocalExec artifact parent is not a real directory: " + relative);
  }
  return readRegularFile(path.join(desktopRoot, relative), "LocalExec artifact");
}

function digestBytes(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function artifactJson(relative) {
  return JSON.parse(artifactBytes(relative).toString("utf8"));
}

function sameRuntime(actual, declared) {
  return actual != null && declared != null
    && ["version", "node", "modules", "platform", "arch"].every(key =>
      typeof actual[key] === "string" && actual[key] === declared[key]);
}

function actualLibraryFiles(root, prefix = "") {
  const files = [];
  for (const name of fs.readdirSync(path.join(root, prefix)).sort()) {
    const relative = path.posix.join(prefix, name);
    const absolute = path.join(root, relative);
    const stat = fs.lstatSync(absolute);
    if (stat.isDirectory()) files.push(...actualLibraryFiles(root, relative));
    else if (stat.isFile()) files.push(relative);
    else throw new Error("LocalExec library has a non-regular or symbolic member: " + relative);
  }
  return files;
}


function checkLocalExecLegalInputs(manifest, lock, packageJson) {
  const legal = manifest.legal_inputs;
  const prefix = "dist/local-exec-daemon/licenses/";
  if (legal?.manifest !== prefix + "source-inputs.json" || !Array.isArray(legal.files) || legal.files.length === 0) {
    throw new Error("LocalExec source/notice manifest is missing");
  }
  const declared = [];
  for (const file of legal.files) {
    if (typeof file.path !== "string" || !file.path.startsWith(prefix)) {
      throw new Error("LocalExec source/notice member leaves its root");
    }
    const bytes = artifactBytes(file.path);
    if (file.bytes !== bytes.length || file.sha256 !== digestBytes(bytes)) {
      throw new Error("LocalExec source/notice member differs: " + file.path);
    }
    declared.push(file.path.slice(prefix.length));
  }
  const actual = actualLibraryFiles(path.join(desktopRoot, prefix));
  if (new Set(declared).size !== declared.length
      || JSON.stringify([...declared].sort()) !== JSON.stringify(actual.sort())) {
    throw new Error("LocalExec source/notice membership is incomplete or duplicated");
  }
  const inputs = artifactJson(legal.manifest);
  const expectedNames = ["iconv-lite", "ignore", "jschardet", "re2js", "tree-sitter",
    "tree-sitter-bash", "node-addon-api", "node-gyp-build", "web-tree-sitter", "safer-buffer"].sort();
  if (inputs.contract !== "LDB-v1-legal-inputs" || !Array.isArray(inputs.packages)
      || JSON.stringify(inputs.packages.map(item => item.name).sort()) !== JSON.stringify(expectedNames)) {
    throw new Error("LocalExec legal/source package set differs from the required closure");
  }
  const memberSet = new Set(declared);
  for (const item of inputs.packages) {
    const pinned = lock.packages?.["node_modules/" + item.name];
    if (pinned?.version !== item.version || pinned.resolved !== item.resolved
        || pinned.integrity !== item.integrity || pinned.license !== item.declared_license
        || !Array.isArray(item.inspected_source_artifacts) || item.inspected_source_artifacts.length === 0) {
      throw new Error("LocalExec legal/source package identity differs from lock: " + item.name);
    }
    for (const artifact of item.inspected_source_artifacts) {
      if (!memberSet.has(artifact.file)) throw new Error("LocalExec legal/source artifact is absent");
      const bytes = artifactBytes(prefix + artifact.file);
      if (bytes.length !== artifact.bytes || digestBytes(bytes) !== artifact.sha256
          || gitBlobIdentity(bytes) !== artifact.git_blob) {
        throw new Error("LocalExec legal/source identity drifted: " + artifact.file);
      }
    }
  }

  const common = packageJson.build?.extraResources;
  const mappings = Array.isArray(common) ? common.filter(entry => entry !== null
    && typeof entry === "object" && !Array.isArray(entry)
    && entry.from === "resources/licenses/local-exec" && entry.to === "licenses/local-exec"
    && JSON.stringify(entry.filter) === '["**/*"]'
    && Object.keys(entry).sort().join(",") === "filter,from,to") : [];
  if (mappings.length !== 1) throw new Error("LocalExec source notices require one common resource mapping");
  const resourceSets = [common];
  for (const platform of ["mac", "mas", "win", "linux"]) {
    const entries = packageJson.build[platform]?.extraResources;
    if (entries != null) resourceSets.push(Array.isArray(entries) ? entries : [entries]);
  }
  for (const entries of resourceSets) for (const entry of entries) {
    if (entries === common && entry === mappings[0]) continue;
    const destination = typeof entry === "string" ? literalResourcePath(entry)
      : entry !== null && typeof entry === "object" && !Array.isArray(entry)
        ? literalResourcePath(entry.to == null ? "." : entry.to) : null;
    const protectedDirectory = "licenses/local-exec";
    if (destination === null || destination === "." || destination === protectedDirectory
        || protectedDirectory.startsWith(destination + "/")
        || destination.startsWith(protectedDirectory + "/")) {
      throw new Error("Another resource mapping may overwrite LocalExec source notices");
    }
  }
}

async function checkLocalExecArtifacts() {
  const manifest = artifactJson("dist/local-exec-daemon/build-manifest.json");
  if (manifest?.contract !== "LDB-v1" || manifest.entry !== "source/local-exec-daemon/main.ts"
      || manifest.bundle !== "dist/local-exec-daemon/main.cjs"
      || manifest.metafile !== "dist/local-exec-daemon/build-metafile.json"
      || manifest.node_dependencies?.manifest !== "dist/node-deps/local-exec-dependencies.json"
      || manifest.electron_dependencies?.manifest !== "dist/deps/local-exec-dependencies.json") {
    throw new Error("LocalExec selected entry/build identity is invalid");
  }
  const bundle = artifactBytes(manifest.bundle);
  const metaBytes = artifactBytes(manifest.metafile);
  if (digestBytes(bundle) !== manifest.bundle_sha256 || digestBytes(metaBytes) !== manifest.metafile_sha256) {
    throw new Error("LocalExec emitted entry or input manifest differs from its build identity");
  }
  const metadata = JSON.parse(metaBytes.toString("utf8"));
  const output = metadata.outputs?.[manifest.bundle];
  if (output?.entryPoint !== "../source/local-exec-daemon/main.ts" || output.bytes !== bundle.length) {
    throw new Error("LocalExec input manifest does not produce the selected daemon entry");
  }
  for (const input of [
    "../source/local-exec-daemon/main.ts",
    "../source/local-exec-daemon/production-executor.ts",
    "../source/host/local-exec/local-exec-daemon.ts",
    "../source/host/local-exec/local-exec-provider.ts",
    "../source/host/local-exec/local-exec-native-file-access.ts",
  ]) {
    if (!Object.hasOwn(metadata.inputs ?? {}, input)) {
      throw new Error("LocalExec production consumer is absent from emitted inputs: " + input);
    }
  }
  if (!(output.imports ?? []).some(item => item.path === "jschardet" && item.external === true)
      || Object.keys(metadata.inputs ?? {}).some(input => /(?:^|\/)node_modules\/jschardet\//.test(input))) {
    throw new Error("LocalExec charset library must remain a separate required module");
  }

  const packageJson = readJson(path.join(desktopRoot, "package.json"), "Desktop package configuration");
  for (const required of ["dist/deps/**", "dist/node-deps/**", "dist/node_modules/jschardet/**"]) {
    if (!Array.isArray(packageJson.build?.asarUnpack) || !packageJson.build.asarUnpack.includes(required)) {
      throw new Error("LocalExec packaged native/source asset mapping is missing: " + required);
    }
  }
  const node = await verifyLocalExecDependencyStage({
    stageRoot: path.join(desktopRoot, "dist/node-deps"), desktopRoot, domain: "node",
  });
  const electron = await verifyLocalExecDependencyStage({
    stageRoot: path.join(desktopRoot, "dist/deps"), desktopRoot, domain: "electron",
  });
  if (!sameRuntime(node.runtime, manifest.node_dependencies.runtime)
      || !sameRuntime(electron.runtime, manifest.electron_dependencies.runtime)) {
    throw new Error("LocalExec entry and parser runtime domains disagree");
  }

  const library = manifest.external_library;
  const lockBytes = readRegularFile(path.join(desktopRoot, "package-lock.json"), "Desktop dependency lock");
  const lock = JSON.parse(lockBytes.toString("utf8"));
  checkLocalExecLegalInputs(manifest, lock, packageJson);
  const locked = lock.packages?.["node_modules/jschardet"];
  if (library?.name !== "jschardet" || library.version !== "3.1.4"
      || library.license !== "LGPL-2.1+" || library.directory !== "dist/node_modules/jschardet"
      || library.source_and_notices_retained !== true
      || library.package_lock_sha256 !== digestBytes(lockBytes)
      || locked?.version !== library.version || locked.license !== library.license
      || locked.resolved !== library.resolved || locked.integrity !== library.integrity
      || packageJson.dependencies?.jschardet !== library.version
      || !Array.isArray(library.files) || library.files.length === 0) {
    throw new Error("LocalExec separate charset source/package identity differs from the lock");
  }
  const prefix = library.directory + "/";
  const declared = [];
  for (const file of library.files) {
    if (typeof file.path !== "string" || !file.path.startsWith(prefix)) {
      throw new Error("LocalExec charset source member leaves its package");
    }
    const bytes = artifactBytes(file.path);
    if (!Number.isSafeInteger(file.bytes) || file.bytes !== bytes.length
        || !/^[0-9a-f]{64}$/.test(file.sha256) || file.sha256 !== digestBytes(bytes)) {
      throw new Error("LocalExec charset source member differs: " + file.path);
    }
    declared.push(file.path.slice(prefix.length));
  }
  const members = actualLibraryFiles(path.join(desktopRoot, library.directory));
  if (new Set(declared).size !== declared.length
      || JSON.stringify([...declared].sort()) !== JSON.stringify(members.sort())) {
    throw new Error("LocalExec charset source membership is incomplete or duplicated");
  }
  for (const required of ["package.json", "LICENSE", "CONTRIBUTORS", "index.js", "src/index.js"]) {
    if (!declared.includes(required) || artifactBytes(prefix + required).length === 0) {
      throw new Error("LocalExec charset source/notice is missing: " + required);
    }
  }
  const libraryPackage = artifactJson(prefix + "package.json");
  if (libraryPackage.name !== library.name || libraryPackage.version !== library.version) {
    throw new Error("LocalExec charset installed package identity differs");
  }
  console.log("LocalExec selected daemon, separate runtime manifests and source assets passed structural checks; execution requires the exact-H native/startup receipts.");
}

try {
  const options = parseOptions(process.argv.slice(2));
  const evidence = checkOpenClawNotice(options);
  if (options.mode === "runtime") {
    checkRuntimeArtifacts();
    await checkLocalExecArtifacts();
    console.log("Desktop runtime artifact graph passed: Electron main, preload, Rust Coordinator carrier + app-host binary wiring, selected LocalExec daemon and Renderer are co-staged.");
  }
  console.log(JSON.stringify(evidence));
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
