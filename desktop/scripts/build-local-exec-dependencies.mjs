// LDB-v1: the existing desktop build owns two separately qualified native stages.
// There is deliberately no cross-process native-output cache: a manifest cannot
// prove that an uncaptured compiler/SDK installation has stayed unchanged.
// Repeated calls in this owner process reuse only rehashed inputs/outputs and a
// fresh native load. Verification below is structural and never builds or runs.
import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import { lstat, mkdir, mkdtemp, readFile, readdir, realpath, rename, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const require = createRequire(import.meta.url);
const SELF = fileURLToPath(import.meta.url);
const MANIFEST = "local-exec-dependencies.json";
const ELECTRON_VERSION = "43.4.0";
const GYP_VERSION = "12.4.0";
const STOCK_PATCH = "d94979694bc3b1854ba0030be33e9941df9867f99d956117b704e9cfaca37a7d";
const APPLIED_PATCH = "8f7eb83b314277458e41d0096c03d58849c2921ee8705b14376b712e1eb647d4";
const PACKAGES = [
  { name: "tree-sitter", version: "0.21.1", integrity: "sha512-7dxoA6kYvtgWw80265MyqJlkRl4yawIjO7S5MigytjELkX43fV2WsAXzsNfO7sBpPPCF5Gp0+XzHk0DwLCq3xQ==", licenseFile: "LICENSE" },
  { name: "tree-sitter-bash", version: "0.21.0", integrity: "sha512-UuXf+wliu1mmS/O2Iz7OQghExM4a+lk+GaVPndZVpAJnFuzanaN33UcHOsrmngHxaOXHz5JSZrwp6i2qM/PKag==", licenseFile: "LICENSE" },
  { name: "node-addon-api", version: "8.5.0", integrity: "sha512-/bRZty2mXUIFY/xU5HLvveNHlswNJej+RnxBjOMkidWfwZzgTbPG1E3K5TOxRLOR+5hX7bSofy8yf1hZevMS8A==", licenseFile: "LICENSE.md" },
  { name: "node-gyp-build", version: "4.8.4", integrity: "sha512-LA4ZjwlnUblHVgq0oBF3Jl/6h/Nvs5fzBLwdEF4nuxnFdsfajde4WfxtJr3CaiH+F6ewcIB/q4jQ4UzPyid+CQ==", licenseFile: "LICENSE" },
  { name: "web-tree-sitter", version: "0.21.0", integrity: "sha512-iJ+QJ6ikN9D9cG7Kh6q3KtAstYFUQbYZ8OjuPEJYWfj2kLrmp5I3C2n6WjE1Y3jvj7nJbkcrJytJGWUEhCxn+g==", licenseFile: "LICENSE" },
].map(p => Object.freeze({ ...p, resolved: "https://registry.npmjs.org/" + p.name + "/-/" + p.name + "-" + p.version + ".tgz", license: "MIT" }));
const NATIVE_PATHS = [
  "tree-sitter/build/Release/tree_sitter_runtime_binding.node",
  "tree-sitter-bash/build/Release/tree_sitter_bash_binding.node",
];
const SUPPORT = new Set(PACKAGES.slice(2).map(p => p.name));
const activeBuilds = new Map();
const qualifiedHere = new Map();

function check(condition, message) {
  if (!condition) throw new Error("LDB-v1: " + message);
}
function hash(bytes) { return createHash("sha256").update(bytes).digest("hex"); }
function same(a, b) { return JSON.stringify(a) === JSON.stringify(b); }
function safeRelative(value) {
  return typeof value === "string" && value.length > 0 && !value.includes("\\")
    && !value.includes("\0") && !value.includes(":") && !path.posix.isAbsolute(value)
    && value.split("/").every(p => p && p !== "." && p !== "..");
}
function sorted(files) { return files.sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0); }
function fileIdentity(relative, bytes) { return { path: relative, bytes: bytes.length, sha256: hash(bytes) }; }
function assertFiles(files, label) {
  check(Array.isArray(files) && files.length > 0, label + " is empty");
  let previous = "";
  for (const f of files) {
    check(f && same(Object.keys(f).sort(), ["bytes", "path", "sha256"]), label + " has an invalid file record");
    check(safeRelative(f.path) && f.path > previous, label + " paths are unsafe, duplicated or unsorted");
    check(Number.isSafeInteger(f.bytes) && f.bytes >= 0 && /^[a-f0-9]{64}$/.test(f.sha256), label + " has an invalid file digest");
    previous = f.path;
  }
}
async function json(file) { return JSON.parse(await readFile(file, "utf8")); }
async function regularBytes(file) {
  const info = await lstat(file);
  check(info.isFile() && !info.isSymbolicLink(), "expected a regular file: " + path.basename(file));
  return readFile(file);
}
async function inventory(root, { omitManifest = false, source = false } = {}) {
  check((await lstat(root)).isDirectory() && !(await lstat(root)).isSymbolicLink(), "stage/package root must be an ordinary directory");
  const result = [];
  async function visit(relative) {
    for (const entry of await readdir(path.join(root, relative), { withFileTypes: true })) {
      const name = relative ? relative + "/" + entry.name : entry.name;
      if (omitManifest && name === MANIFEST) continue;
      if (source && (name === "build" || name === "prebuilds" || entry.name === "node_modules" || entry.name === "__pycache__" || entry.name.endsWith(".pyc"))) continue;
      check(safeRelative(name) && !entry.isSymbolicLink(), "nonportable path or symlink in stage/source: " + name);
      if (entry.isDirectory()) {
        const before = result.length;
        await visit(name);
        check(source || result.length > before, "undeclared empty stage directory: " + name);
      }
      else {
        check(entry.isFile(), "non-regular stage/source member: " + name);
        result.push(fileIdentity(name, await regularBytes(path.join(root, name))));
      }
    }
  }
  await visit("");
  return sorted(result);
}
function assertRuntime(runtime, domain, desktopPackage) {
  check(domain === "node" || domain === "electron", "unknown dependency domain");
  check(runtime && same(Object.keys(runtime).sort(), ["arch", "modules", "node", "platform", "version"]), "invalid runtime identity");
  for (const k of ["version", "node"]) check(/^\d+\.\d+\.\d+$/.test(runtime[k]), "invalid runtime " + k);
  check(/^\d+$/.test(runtime.modules), "invalid runtime modules ABI");
  check(runtime.platform === process.platform && runtime.arch === process.arch, "runtime platform/architecture mismatch");
  check(desktopPackage.devDependencies?.electron === ELECTRON_VERSION, "desktop Electron pin drifted");
  if (domain === "electron") check(runtime.version === ELECTRON_VERSION, "Electron runtime version mismatch");
  else check(runtime.version === runtime.node, "Node runtime version mismatch");
}
function headersIdentity(runtime, domain) {
  const origin = domain === "electron" ? "https://www.electronjs.org/headers" : "https://nodejs.org/dist";
  const base = origin + "/v" + runtime.version + "/";
  return { url: base + "node-v" + runtime.version + "-headers.tar.gz", version: runtime.version, checksum_url: base + "SHASUMS256.txt" };
}
export function validateHeaderChecksum(text, filename, archive) {
  const matches = text.split(/\r?\n/).map(line => line.trim().split(/\s+/))
    .filter(parts => parts.length === 2 && parts[1].replace(/^\.\//, "").replace(/^\*/, "") === filename);
  check(matches.length === 1 && /^[a-fA-F0-9]{64}$/.test(matches[0][0]), "missing or duplicate official header checksum");
  check(hash(archive) === matches[0][0].toLowerCase(), "official header archive checksum mismatch");
  return hash(archive);
}
function headerNumber(source, name) {
  const matches = [...source.matchAll(new RegExp("^\\s*#\\s*define\\s+" + name + "\\s+(\\d+)\\s*(?:/[/\\*].*)?$", "gm"))];
  check(matches.length === 1, "missing or ambiguous header " + name);
  return matches[0][1];
}
function validateHeaderRuntime(source, runtime) {
  check(["NODE_MAJOR_VERSION", "NODE_MINOR_VERSION", "NODE_PATCH_VERSION"].map(k => headerNumber(source, k)).join(".") === runtime.node,
    "official headers disagree with embedded Node version");
  check(headerNumber(source, "NODE_MODULE_VERSION") === runtime.modules, "official headers disagree with runtime modules ABI");
}
function packageFiles(files, name) {
  return files.filter(f => f.path.startsWith(name + "/") || (SUPPORT.has(name) && f.path.startsWith("node_modules/" + name + "/")));
}
function lockIdentity(lock, pinned) {
  const found = lock.packages?.["node_modules/" + pinned.name];
  check(found && ["version", "resolved", "integrity", "license"].every(k => found[k] === pinned[k]), "locked package identity drift: " + pinned.name);
  return { name: pinned.name, version: found.version, resolved: found.resolved, integrity: found.integrity, license: found.license };
}

/** Structural artifact check only. This never executes a native load or fetches/builds anything. */
export async function verifyLocalExecDependencyStage({ stageRoot, desktopRoot, domain }) {
  const receipt = JSON.parse(await regularBytes(path.join(stageRoot, MANIFEST)));
  check(receipt?.contract === "LDB-v1" && receipt.domain === domain, "dependency stage domain/contract mismatch");
  const desktopPackage = await json(path.join(desktopRoot, "package.json"));
  assertRuntime(receipt.runtime, domain, desktopPackage);
  const lockBytes = await regularBytes(path.join(desktopRoot, "package-lock.json"));
  check(receipt.package_lock_sha256 === hash(lockBytes), "dependency stage lock identity mismatch");
  const lock = JSON.parse(lockBytes);
  assertFiles(receipt.build_inputs, "build inputs");
  check(receipt.build_input_sha256 === hash(JSON.stringify(receipt.build_inputs)), "build-input digest mismatch");
  const builder = receipt.build_inputs.find(f => f.path === "scripts/build-local-exec-dependencies.mjs");
  check(builder && same(builder, fileIdentity(builder.path, await regularBytes(SELF))), "dependency builder identity mismatch");
  assertFiles(receipt.stage_files, "stage files");
  const actualFiles = await inventory(stageRoot, { omitManifest: true });
  check(same(receipt.stage_files, actualFiles), "stage membership or file bytes mismatch");
  const byPath = new Map(actualFiles.map(f => [f.path, f]));
  const inputsByPath = new Map(receipt.build_inputs.map(f => [f.path, f]));
  for (const file of ["package.json", "package-lock.json"]) {
    check(same(inputsByPath.get(file), fileIdentity(file, await regularBytes(path.join(desktopRoot, file)))), "declared desktop build input mismatch: " + file);
  }
  for (const input of ["runtime/host-node", "runtime/" + domain + "-executable", "configuration/host-node-config.json", "configuration/build-environment.json",
    "tool/node_modules/node-gyp/package.json", "tool/node_modules/node-gyp/bin/node-gyp.js", "tool/node_modules/node-gyp/lib/install.js",
    "tool/node_modules/node-gyp/lib/download.js", "tool/node_modules/node-gyp/lib/process-release.js"]) {
    check(inputsByPath.get(input)?.bytes > 0, "missing causal build input: " + input);
  }
  check(same(inputsByPath.get("configuration/runtime.json"), fileIdentity("configuration/runtime.json", Buffer.from(JSON.stringify(receipt.runtime)))), "runtime build input mismatch");
  check(Array.isArray(receipt.packages) && receipt.packages.length === PACKAGES.length, "incomplete package set");
  for (let i = 0; i < PACKAGES.length; i++) {
    const expected = PACKAGES[i];
    const p = receipt.packages[i];
    check(p && same({ name: p.name, version: p.version, resolved: p.resolved, integrity: p.integrity, license: p.license }, lockIdentity(lock, expected)),
      "dependency package receipt mismatch: " + expected.name);
    assertFiles(p.files, "package files");
    check(same(p.files, packageFiles(actualFiles, expected.name)), "package membership mismatch: " + expected.name);
    const installed = await json(path.join(stageRoot, expected.name, "package.json"));
    check(installed.name === expected.name && installed.version === expected.version, "staged package version mismatch: " + expected.name);
    check(byPath.get(expected.name + "/" + expected.licenseFile)?.bytes > 0, "missing package license: " + expected.name);
    for (const staged of p.files.filter(f => f.path.startsWith(expected.name + "/") && !f.path.startsWith(expected.name + "/build/"))) {
      const inputPath = "source/" + staged.path;
      const input = inputsByPath.get(inputPath);
      if (staged.path === "tree-sitter/binding.gyp") check(input?.sha256 === STOCK_PATCH && input.bytes > 0, "missing exact stock patch input");
      else check(same(input, { ...staged, path: inputPath }), "missing or changed staged source input: " + staged.path);
    }
    check(p.files.some(f => /\/(?:README(?:\.[^/]*)?)$/i.test(f.path) && f.bytes > 0), "missing package documentation: " + expected.name);
    if (SUPPORT.has(expected.name)) {
      const original = p.files.filter(f => f.path.startsWith(expected.name + "/"));
      const duplicate = p.files.filter(f => f.path.startsWith("node_modules/" + expected.name + "/"))
        .map(f => ({ ...f, path: f.path.slice("node_modules/".length) }));
      check(same(original, duplicate), "support package copy mismatch: " + expected.name);
    }
  }
  for (const source of ["tree-sitter/src/binding.cc", "tree-sitter/vendor/tree-sitter/lib/src/lib.c", "tree-sitter-bash/src/parser.c", "tree-sitter-bash/src/scanner.c"]) {
    check(byPath.get(source)?.bytes > 0, "missing native package source: " + source);
  }
  check(byPath.get("tree-sitter/binding.gyp")?.sha256 === APPLIED_PATCH, "isolated tree-sitter patch identity mismatch");
  check(!actualFiles.some(f => /^(tree-sitter|tree-sitter-bash)\/(?:prebuilds|build\/Debug)\//.test(f.path)), "unqualified native fallback members");
  const nativeFiles = actualFiles.filter(f => f.path.endsWith(".node"));
  check(same(nativeFiles.map(f => f.path), NATIVE_PATHS.slice().sort()), "unexpected or missing native outputs");
  check(same(receipt.native_outputs, nativeFiles) && nativeFiles.every(f => f.bytes > 0), "native output identity mismatch");
  const expectedHeaders = headersIdentity(receipt.runtime, domain);
  check(receipt.headers && Object.entries(expectedHeaders).every(([k, v]) => receipt.headers[k] === v), "official header origin/version mismatch");
  const archivePath = ".headers/node-v" + receipt.runtime.version + "-headers.tar.gz";
  const checksumPath = ".headers/SHASUMS256.txt";
  const archive = await regularBytes(path.join(stageRoot, archivePath));
  const checksum = await regularBytes(path.join(stageRoot, checksumPath));
  check(receipt.headers.archive_sha256 === validateHeaderChecksum(checksum.toString("utf8"), path.posix.basename(archivePath), archive),
    "header archive receipt mismatch");
  check(receipt.headers.checksum_sha256 === hash(checksum), "header checksum document mismatch");
  assertFiles(receipt.headers.files, "header files");
  check(same(receipt.headers.files, actualFiles.filter(f => f.path.startsWith(".headers/"))), "header membership mismatch");
  check(same(receipt.build_inputs.filter(f => f.path.startsWith("headers/")), headerInputs(receipt.headers)), "header build input mismatch");
  const headerRoot = ".headers/" + receipt.runtime.version;
  validateHeaderRuntime(await readFile(path.join(stageRoot, headerRoot, "include/node/node_version.h"), "utf8"), receipt.runtime);
  check(byPath.get(headerRoot + "/include/node/config.gypi")?.bytes > 0, "missing runtime config.gypi");
  if (receipt.runtime.platform === "win32") {
    const library = await regularBytes(path.join(stageRoot, headerRoot, receipt.runtime.arch, "node.lib"));
    const libraryName = "win-" + (receipt.runtime.arch === "ia32" ? "x86" : receipt.runtime.arch) + "/node.lib";
    validateHeaderChecksum(checksum.toString("utf8"), libraryName, library);
  }
  check(receipt.load_probe && receipt.load_probe.exit === 0 && receipt.load_probe.parser_language_loaded === true
    && same(receipt.load_probe.runtime, receipt.runtime), "missing qualified load-probe receipt");
  return receipt;
}

// All commands use argument arrays, bounded output, no shell, and exact child
// handles. Never signal a numeric PID/process group after the child exits.
export function runOwnedChild(command, args, { cwd, env, timeoutMs = 300000, maxOutputBytes = 4 * 1024 * 1024, forward = false } = {}) {
  return new Promise((resolve, reject) => {
    let child;
    let exited = false;
    let closed = false;
    let failure;
    let stdout = "";
    let stderr = "";
    let bytes = 0;
    let termTimer;
    let closeTimer;
    let deadline;
    const fail = error => {
      failure ??= error;
      if (child?.pid && !exited && !closed) child.kill("SIGTERM");
      termTimer ??= setTimeout(() => {
        if (child?.pid && !exited && !closed) child.kill("SIGKILL");
      }, 1500);
      closeTimer ??= setTimeout(() => {
        if (closed) return;
        clearTimeout(deadline);
        clearTimeout(termTimer);
        const error = new Error("LDB-v1: owned child did not close after termination", { cause: failure });
        error.ownedChildUnsettled = true;
        reject(error);
      }, 5000);
    };
    try { child = spawn(command, args, { cwd, env, shell: false, stdio: ["ignore", "pipe", "pipe"], windowsHide: true }); }
    catch (error) { reject(error); return; }
    const collect = (chunk, stream) => {
      bytes += chunk.length;
      if (bytes > maxOutputBytes) { fail(new Error("LDB-v1: owned child exceeded output limit")); return; }
      if (stream === "stdout") stdout += chunk;
      else stderr += chunk;
      if (forward) (stream === "stdout" ? process.stdout : process.stderr).write(chunk);
    };
    child.stdout.on("data", chunk => collect(chunk, "stdout"));
    child.stderr.on("data", chunk => collect(chunk, "stderr"));
    child.stdout.on("error", fail);
    child.stderr.on("error", fail);
    child.once("error", error => fail(error));
    child.once("exit", () => { exited = true; clearTimeout(termTimer); });
    child.once("close", (code, signal) => {
      closed = true;
      clearTimeout(deadline);
      clearTimeout(termTimer);
      clearTimeout(closeTimer);
      if (failure) reject(failure);
      else if (code !== 0 || signal !== null) reject(new Error("LDB-v1: owned child failed (exit " + code + ", signal " + signal + ")\n" + stderr));
      else resolve({ stdout, stderr, exit: 0 });
    });
    deadline = setTimeout(() => fail(new Error("LDB-v1: owned child deadline exceeded")), timeoutMs);
  });
}
function cleanEnvironment(domain) {
  const env = {};
  const allowed = /^(?:PATH|PATHEXT|SYSTEMROOT|WINDIR|COMSPEC|HOME|USERPROFILE|APPDATA|LOCALAPPDATA|TEMP|TMP|TMPDIR|PROGRAMFILES(?:\(X86\))?|PROGRAMW6432|NUMBER_OF_PROCESSORS|VSINSTALLDIR|VCINSTALLDIR|WINDOWSSDKDIR|WINDOWSSDKVERSION|INCLUDE|LIB|LIBPATH|SDKROOT|DEVELOPER_DIR|CC|CXX|AR|LD|RANLIB|PYTHON|NODE_GYP_FORCE_PYTHON|HTTP_PROXY|HTTPS_PROXY|NO_PROXY)$/i;
  for (const [key, value] of Object.entries(process.env)) if (allowed.test(key) && value !== undefined) env[key] = value;
  env.PYTHONDONTWRITEBYTECODE = "1";
  if (domain === "electron") env.ELECTRON_RUN_AS_NODE = "1";
  return env;
}
const RUNTIME_PROGRAM = 'process.stdout.write(JSON.stringify({version:process.versions.electron||process.versions.node,node:process.versions.node,modules:process.versions.modules,platform:process.platform,arch:process.arch}));';
async function runtimeIdentity(executable, domain, desktopPackage, cwd) {
  const { stdout } = await runOwnedChild(executable, ["-e", RUNTIME_PROGRAM], { cwd, env: cleanEnvironment(domain), timeoutMs: 30000, maxOutputBytes: 65536 });
  const runtime = JSON.parse(stdout);
  assertRuntime(runtime, domain, desktopPackage);
  return runtime;
}
const LOAD_PROGRAM = String.raw`
const assert = require("node:assert/strict");
const path = require("node:path");
const fs = require("node:fs");
const root = process.argv[1];
const expected = JSON.parse(process.argv[2]);
const runtime = {version:process.versions.electron||process.versions.node,node:process.versions.node,modules:process.versions.modules,platform:process.platform,arch:process.arch};
assert.deepEqual(runtime, expected);
const Parser = require(path.join(root, "tree-sitter"));
const bash = require(path.join(root, "tree-sitter-bash"));
const parser = new Parser();
parser.setLanguage(bash);
const tree = parser.parse("printf '%s\\n' ldb_native_probe");
assert.equal(tree.rootNode.type, "program");
assert.equal(tree.rootNode.hasError, false);
assert.equal(tree.rootNode.namedChildren[0].type, "command");
const native = Object.keys(require.cache).filter(p => p.endsWith(".node")).map(p => fs.realpathSync(p)).sort();
const required = ["tree-sitter/build/Release/tree_sitter_runtime_binding.node", "tree-sitter-bash/build/Release/tree_sitter_bash_binding.node"].map(p => fs.realpathSync(path.join(root,p))).sort();
assert.deepEqual(native, required);
process.stdout.write(JSON.stringify({runtime,exit:0,parser_language_loaded:true}));
`;
async function nativeProbe(stageRoot, target) {
  const { stdout } = await runOwnedChild(target.executable, ["-e", LOAD_PROGRAM, stageRoot, JSON.stringify(target.runtime)], {
    cwd: stageRoot, env: cleanEnvironment(target.domain), timeoutMs: 30000, maxOutputBytes: 65536,
  });
  const result = JSON.parse(stdout);
  check(same(result, { runtime: target.runtime, exit: 0, parser_language_loaded: true }), "native parser load did not produce the required observation");
  return result;
}
async function targetRuntimes(desktopRoot, desktopPackage) {
  const electronPackageRoot = path.join(desktopRoot, "node_modules", "electron");
  check((await json(path.join(electronPackageRoot, "package.json"))).version === ELECTRON_VERSION, "installed Electron version drifted");
  // electron/index.js is the package's canonical installed executable selector.
  const electronExecutable = require(electronPackageRoot);
  check(typeof electronExecutable === "string" && path.isAbsolute(electronExecutable), "Electron executable is unavailable");
  await regularBytes(electronExecutable);
  return [
    { domain: "node", executable: process.execPath, runtime: await runtimeIdentity(process.execPath, "node", desktopPackage, desktopRoot) },
    { domain: "electron", executable: electronExecutable, runtime: await runtimeIdentity(electronExecutable, "electron", desktopPackage, desktopRoot) },
  ];
}

/** A separate execution claim, used by the builder and Actions native contracts. */
export async function verifyLocalExecDependencyLoad({ stageRoot, desktopRoot, domain }) {
  const receipt = await verifyLocalExecDependencyStage({ stageRoot, desktopRoot, domain });
  const targets = await targetRuntimes(desktopRoot, await json(path.join(desktopRoot, "package.json")));
  const target = targets.find(t => t.domain === domain);
  check(same(receipt.runtime, target.runtime), "stage does not match the actual selected runtime");
  await nativeProbe(stageRoot, target);
  return receipt;
}

export function patchTreeSitterBinding(source) {
  check(hash(source) === STOCK_PATCH, "refusing unexpected stock tree-sitter binding.gyp");
  const transformed = (() => {
      const cxxstd = [
        '    "v8_enable_31bit_smis_on_64bit_arch%": 0,',
        '    "cxxstd%": "<!(node -p \\"parseInt(process.env.npm_config_target ?? process.versions.node) < 22 ? \'c++17\' : \'c++20\'\\")",',
        "",
      ].join("\n");
      return source
        .replace(`      "cflags": [\n        "-std=c++17"\n      ],\n      "cflags_cc": [\n        "-std=c++17"\n      ],\n`, "")
        .replace('"CLANG_CXX_LANGUAGE_STANDARD": "c++17"', '"CLANG_CXX_LANGUAGE_STANDARD": "<(cxxstd)"')
        .replace('"/std:c++17"', '"/std:<(cxxstd)"')
        .replace('          "cflags_cc": [\n            "-Wno-cast-function-type"\n', '          "cflags_cc": [\n            "-std=<(cxxstd)",\n            "-Wno-cast-function-type",\n')
        .replace('    "v8_enable_31bit_smis_on_64bit_arch%": 0,\n', cxxstd);
  })();
  check(hash(transformed) === APPLIED_PATCH, "tree-sitter patch output identity drifted");
  return transformed;
}


async function installedInputs(desktopRoot, target, gypRoot) {
  const lockBytes = await regularBytes(path.join(desktopRoot, "package-lock.json"));
  const lock = JSON.parse(lockBytes);
  const desktopPackage = await json(path.join(desktopRoot, "package.json"));
  check(desktopPackage.devDependencies?.["node-addon-api"] === "8.5.0" && desktopPackage.overrides?.["node-addon-api"] === "8.5.0",
    "node-addon-api direct pin and override are required");
  check(desktopPackage.devDependencies?.["node-gyp"] === GYP_VERSION, "node-gyp direct pin drifted");
  check((await json(path.join(gypRoot, "package.json"))).version === GYP_VERSION, "installed node-gyp version drifted");
  const files = [
    fileIdentity("scripts/build-local-exec-dependencies.mjs", await regularBytes(SELF)),
    fileIdentity("package.json", await regularBytes(path.join(desktopRoot, "package.json"))),
    fileIdentity("package-lock.json", lockBytes),
    fileIdentity("runtime/host-node", await regularBytes(process.execPath)),
    fileIdentity("runtime/" + target.domain + "-executable", await regularBytes(target.executable)),
    fileIdentity("configuration/runtime.json", Buffer.from(JSON.stringify(target.runtime))),
    fileIdentity("configuration/host-node-config.json", Buffer.from(JSON.stringify(process.config))),
    fileIdentity("configuration/build-environment.json", Buffer.from(JSON.stringify(Object.entries(cleanEnvironment("node")).sort(([a], [b]) => a.localeCompare(b))))),
    fileIdentity("configuration/cache-policy.txt", Buffer.from("LDB-v1: only qualified reuse within this build-owner process; no cross-process native-output cache\n")),
  ];
  const sources = new Map();
  for (const pinned of PACKAGES) {
    lockIdentity(lock, pinned);
    const root = path.join(desktopRoot, "node_modules", pinned.name);
    const pkg = await json(path.join(root, "package.json"));
    check(pkg.name === pinned.name && pkg.version === pinned.version, "installed package identity mismatch: " + pinned.name);
    const members = await inventory(root, { source: true });
    check(members.some(f => f.path === pinned.licenseFile && f.bytes > 0), "installed package license missing: " + pinned.name);
    if (pinned.name === "tree-sitter") {
      check(members.find(f => f.path === "binding.gyp")?.sha256 === STOCK_PATCH, "installed tree-sitter must remain stock");
      check(members.some(f => f.path === "vendor/tree-sitter/lib/src/unicode/LICENSE" && f.bytes > 0), "missing vendored Unicode license");
    }
    sources.set(pinned.name, { root, members });
    files.push(...members.map(f => ({ ...f, path: "source/" + pinned.name + "/" + f.path })));
  }
  // Follow the actual existing node-gyp dependency resolution, including nested
  // installed versions. This does not install or mutate any tool/package.
  const visited = new Set();
  async function toolClosure(root) {
    const relative = path.relative(desktopRoot, root).split(path.sep).join("/");
    check(safeRelative(relative) && relative.startsWith("node_modules/"), "build tool escaped the desktop installation");
    if (visited.has(relative)) return;
    visited.add(relative);
    const pkg = await json(path.join(root, "package.json"));
    check(lock.packages?.[relative]?.version === pkg.version, "installed build-tool lock mismatch: " + relative);
    files.push(...(await inventory(root, { source: true })).map(f => ({ ...f, path: "tool/" + relative + "/" + f.path })));
    for (const dependency of Object.keys(pkg.dependencies ?? {}).sort()) {
      let directory = root;
      let found;
      while (directory.startsWith(desktopRoot + path.sep) || directory === desktopRoot) {
        const candidate = path.join(directory, "node_modules", dependency);
        try { await lstat(path.join(candidate, "package.json")); found = candidate; break; }
        catch (error) { if (error.code !== "ENOENT" && error.code !== "ENOTDIR") throw error; }
        if (directory === desktopRoot) break;
        directory = path.dirname(directory);
      }
      check(found, "missing installed build-tool dependency: " + dependency);
      await toolClosure(found);
    }
  }
  await toolClosure(gypRoot);
  return { files: sorted(files), sources, lock, lockSha: hash(lockBytes) };
}
async function copySource(source, destination) {
  await mkdir(destination, { recursive: true });
  for (const member of source.members) {
    const bytes = await regularBytes(path.join(source.root, member.path));
    check(same(fileIdentity(member.path, bytes), member), "package source changed while copying: " + member.path);
    const output = path.join(destination, member.path);
    await mkdir(path.dirname(output), { recursive: true });
    await writeFile(output, bytes);
  }
}
const DOWNLOAD_PROGRAM = String.raw`
const fs = require("node:fs/promises");
const path = require("node:path");
const {download} = require(path.join(process.argv[1], "lib/download.js"));
(async () => {
  const requested = process.argv[2];
  const destination = process.argv[3];
  const maximum = Number(process.argv[4]);
  const response = await download({version:"12.4.0",opts:{}}, requested);
  if (response.status !== 200) throw new Error("Official header HTTP status " + response.status);
  const final = new URL(response.url);
  if (final.protocol !== "https:" || !["nodejs.org","www.electronjs.org","electronjs.org","artifacts.electronjs.org"].includes(final.hostname))
    throw new Error("Unexpected official-header redirect");
  const chunks = [];
  let size = 0;
  for await (const chunk of response.body) {
    size += chunk.length;
    if (size > maximum) throw new Error("Official-header download exceeded limit");
    chunks.push(chunk);
  }
  await fs.writeFile(destination, Buffer.concat(chunks));
  process.stdout.write(JSON.stringify({requested,final:response.url,bytes:size}), () => process.exit(0));
})().catch(error => { console.error(error); process.exit(1); });
`;
async function downloadHeaders(candidate, target, desktopRoot, gypRoot) {
  const expected = headersIdentity(target.runtime, target.domain);
  const headerDir = path.join(candidate, ".headers");
  await mkdir(headerDir);
  const downloads = [];
  for (const [url, name, maximum] of [
    [expected.checksum_url, "SHASUMS256.txt", 2 * 1024 * 1024],
    [expected.url, "node-v" + target.runtime.version + "-headers.tar.gz", 128 * 1024 * 1024],
  ]) {
    const { stdout } = await runOwnedChild(process.execPath, ["-e", DOWNLOAD_PROGRAM, gypRoot, url, path.join(headerDir, name), String(maximum)], {
      cwd: desktopRoot, env: cleanEnvironment("node"), timeoutMs: 180000, maxOutputBytes: 65536,
    });
    downloads.push(JSON.parse(stdout));
  }
  await writeFile(path.join(headerDir, "downloads.json"), JSON.stringify(downloads, null, 2) + "\n");
  const archivePath = path.join(headerDir, path.posix.basename(expected.url));
  const checksum = await regularBytes(path.join(headerDir, "SHASUMS256.txt"));
  const archive = await regularBytes(archivePath);
  const archiveSha = validateHeaderChecksum(checksum.toString("utf8"), path.basename(archivePath), archive);
  const base = target.domain === "electron" ? "https://www.electronjs.org/headers" : "https://nodejs.org/dist";
  const gyp = path.join(gypRoot, "bin/node-gyp.js");
  await runOwnedChild(process.execPath, [gyp, "install", "--target=" + target.runtime.version, "--arch=" + target.runtime.arch,
    "--dist-url=" + base, "--devdir=" + headerDir, "--tarball=" + archivePath, "--node_gyp_internal_noretry"], {
    cwd: desktopRoot, env: cleanEnvironment("node"), timeoutMs: 300000, forward: true,
  });
  const installed = path.join(headerDir, target.runtime.version);
  validateHeaderRuntime(await readFile(path.join(installed, "include/node/node_version.h"), "utf8"), target.runtime);
  // node-gyp falls back to process.config on malformed downloaded config.gypi.
  // Parse with this exact installed node-gyp's own parser before compiling.
  const { parseConfigGypi } = require(path.join(gypRoot, "lib/create-config-gypi.js"));
  const config = parseConfigGypi(await readFile(path.join(installed, "include/node/config.gypi"), "utf8"));
  check(config && typeof config.variables === "object", "invalid official runtime config.gypi");
  if (target.runtime.platform === "win32") {
    const library = await regularBytes(path.join(installed, target.runtime.arch, "node.lib"));
    validateHeaderChecksum(checksum.toString("utf8"), "win-" + (target.runtime.arch === "ia32" ? "x86" : target.runtime.arch) + "/node.lib", library);
  }
  const members = (await inventory(headerDir)).map(f => ({ ...f, path: ".headers/" + f.path }));
  return { ...expected, archive_sha256: archiveSha, checksum_sha256: hash(checksum), files: members };
}
function headerInputs(headers) {
  return headers.files.map(f => ({ ...f, path: "headers/" + f.path.slice(".headers/".length) }));
}
// The capture supplies an existing, invocation-owned directory inside its
// artifact. Keep publication on the stage filesystem and never follow a
// redirected work root. Failures retain this exact directory, including hidden
// headers and any previous stage, for the capture's bounded artifact archive.
export async function withOwnedNativeBuildWork({ stageRoot, domain, workRoot }, build) {
  check(domain === "node" || domain === "electron", "unknown build work domain");
  const stageParent = path.dirname(path.resolve(stageRoot));
  await mkdir(stageParent, { recursive: true });
  check(workRoot === undefined || (typeof workRoot === "string" && path.isAbsolute(workRoot)),
    "native build evidence root must be absolute");
  const parent = workRoot === undefined ? stageParent : path.resolve(workRoot);
  const owner = await lstat(parent);
  check(owner.isDirectory() && !owner.isSymbolicLink() && await realpath(parent) === parent,
    "native build evidence root must be an ordinary owned directory");
  check(owner.dev === (await lstat(stageParent)).dev, "native build evidence root crosses the stage filesystem");
  const work = await mkdtemp(path.join(parent, "ldb-" + domain + "-"));
  try {
    const result = await build(work);
    await rm(work, { recursive: true });
    return result;
  } catch (cause) {
    // An unsettled child may still hold these files. No deletion, group kill,
    // directory search or transfer of ownership is performed here.
    const error = cause instanceof Error ? cause : new Error(String(cause));
    error.message += "\nLDB-v1 retained owned build evidence at " + work;
    throw error;
  }
}

async function buildStage(desktopRoot, target, gypRoot, workRoot) {
  const stageRoot = path.join(desktopRoot, "dist", target.domain === "node" ? "node-deps" : "deps");
  const before = await installedInputs(desktopRoot, target, gypRoot);
  const prior = qualifiedHere.get(stageRoot);
  if (prior) {
    const receipt = await verifyLocalExecDependencyStage({ desktopRoot, stageRoot, domain: target.domain });
    const current = sorted([...before.files, ...headerInputs(receipt.headers)]);
    if (same(receipt.runtime, target.runtime) && same(current, receipt.build_inputs)
        && same(receipt, prior)) {
      await nativeProbe(stageRoot, target);
      return receipt;
    }
  }
  const receipt = await withOwnedNativeBuildWork({ stageRoot, domain: target.domain, workRoot }, async (work) => {
    const candidate = path.join(work, "candidate");
    await mkdir(candidate);
    let published = false;
    try {
      for (const pinned of PACKAGES) {
        await copySource(before.sources.get(pinned.name), path.join(candidate, pinned.name));
        if (SUPPORT.has(pinned.name)) await copySource(before.sources.get(pinned.name), path.join(candidate, "node_modules", pinned.name));
      }
      const binding = path.join(candidate, "tree-sitter/binding.gyp");
      await writeFile(binding, patchTreeSitterBinding(await readFile(binding, "utf8")));
      const headers = await downloadHeaders(candidate, target, desktopRoot, gypRoot);
      const nativeHeaderRoot = path.join(candidate, ".headers", target.runtime.version);
      const env = cleanEnvironment("node");
      env.npm_config_build_from_source = "true";
      env.npm_config_runtime = target.domain;
      env.npm_config_target = target.runtime.version;
      // binding.gyp invokes "node" for node-addon-api and the approved C++ patch.
      // Refuse a PATH-selected Node different from the actual build interpreter.
      const pathNode = await runOwnedChild("node", ["-p", "process.execPath"], { cwd: desktopRoot, env, timeoutMs: 30000, maxOutputBytes: 65536 });
      check(hash(await regularBytes(pathNode.stdout.trim())) === hash(await regularBytes(process.execPath)), "PATH node differs from build Node");
      for (const [index, name] of ["tree-sitter", "tree-sitter-bash"].entries()) {
        const packageRoot = path.join(candidate, name);
        await runOwnedChild(process.execPath, [path.join(gypRoot, "bin/node-gyp.js"), "rebuild", "--directory=" + packageRoot,
          "--release", "--target=" + target.runtime.version, "--arch=" + target.runtime.arch, "--nodedir=" + nativeHeaderRoot,
          "--devdir=" + path.join(candidate, ".headers")], { cwd: candidate, env, timeoutMs: 600000, forward: true });
        const generated = await readFile(path.join(packageRoot, "build/config.gypi"), "utf8");
        const config = JSON.parse(generated.replace(/^#.*\n/, ""));
        check(path.resolve(config.variables.nodedir) === nativeHeaderRoot && config.variables.target_arch === target.runtime.arch,
          "node-gyp used an unowned header directory or architecture");
        const native = await regularBytes(path.join(candidate, NATIVE_PATHS[index]));
        check(native.length > 0, "native build output is empty");
        // Preserve sources/legal artifacts, discard only compiler intermediates and
        // alternative native lookup locations, then retain the exact built binary.
        await rm(path.join(packageRoot, "build"), { recursive: true });
        await mkdir(path.dirname(path.join(candidate, NATIVE_PATHS[index])), { recursive: true });
        await writeFile(path.join(candidate, NATIVE_PATHS[index]), native);
      }
      const after = await installedInputs(desktopRoot, target, gypRoot);
      check(same(before.files, after.files), "installed inputs changed during native build");
      const currentHeaders = (await inventory(path.join(candidate, ".headers"))).map(f => ({ ...f, path: ".headers/" + f.path }));
      check(same(currentHeaders, headers.files), "headers changed during native build");
      const stageFiles = await inventory(candidate);
      const buildInputs = sorted([...before.files, ...headerInputs(headers)]);
      const receipt = {
        contract: "LDB-v1", domain: target.domain, runtime: target.runtime,
        package_lock_sha256: before.lockSha, build_input_sha256: hash(JSON.stringify(buildInputs)), build_inputs: buildInputs,
        packages: PACKAGES.map(p => ({ ...lockIdentity(before.lock, p), files: packageFiles(stageFiles, p.name) })),
        headers, native_outputs: stageFiles.filter(f => f.path.endsWith(".node")), stage_files: stageFiles,
        load_probe: await nativeProbe(candidate, target),
      };
      await writeFile(path.join(candidate, MANIFEST), JSON.stringify(receipt, null, 2) + "\n");
      await verifyLocalExecDependencyStage({ stageRoot: candidate, desktopRoot, domain: target.domain });
      const previous = path.join(work, "previous");
      let replaced = false;
      try { await rename(stageRoot, previous); replaced = true; }
      catch (error) { if (error.code !== "ENOENT") throw error; }
      try { await rename(candidate, stageRoot); }
      catch (error) {
        if (replaced) {
          try { await rename(previous, stageRoot); }
          catch (restoreError) { throw new AggregateError([error, restoreError], "LDB-v1: publication and previous-stage restoration both failed; retained " + work); }
        }
        throw error;
      }
      published = true;
      // Qualify the final published path before handing it to packaging. On a
      // relocation/load failure, keep the failed candidate and restore the prior
      // complete stage; never silently accept a partially published replacement.
      try { await nativeProbe(stageRoot, target); }
      catch (error) {
        await rename(stageRoot, path.join(work, "failed-published"));
        if (replaced) await rename(previous, stageRoot);
        published = false;
        throw error;
      }
      return receipt;
    } catch (error) {
      if (published) qualifiedHere.delete(stageRoot);
      throw error;
    }
  });
  qualifiedHere.set(stageRoot, receipt);
  return receipt;
}

export async function stageLocalExecDependencies({ desktopRoot, workRoot }) {
  desktopRoot = path.resolve(desktopRoot);
  const active = activeBuilds.get(desktopRoot);
  if (active) {
    check(active.workRoot === workRoot, "concurrent native build requested a different evidence root");
    return active.build;
  }
  const build = (async () => {
    check(!process.versions.electron, "the desktop native build owner must run under ordinary Node");
    const desktopPackage = await json(path.join(desktopRoot, "package.json"));
    const targets = await targetRuntimes(desktopRoot, desktopPackage);
    const gypRoot = path.join(desktopRoot, "node_modules", "node-gyp");
    const result = {};
    for (const target of targets) result[target.domain] = await buildStage(desktopRoot, target, gypRoot, workRoot);
    return result;
  })();
  activeBuilds.set(desktopRoot, { build, workRoot });
  try { return await build; }
  finally { activeBuilds.delete(desktopRoot); }
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const desktopRoot = path.resolve(path.dirname(SELF), "..");
  const result = await stageLocalExecDependencies({ desktopRoot, workRoot: process.env.FABUSHI_LOCAL_EXEC_BUILD_WORK_ROOT });
  console.log(JSON.stringify({ contract: "LDB-v1", node: result.node.runtime, electron: result.electron.runtime }));
}
