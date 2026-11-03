// LDB-v1 contracts execute only in GitHub Actions. Structural fixtures do not
// claim native validity; the first contract separately loads both real stages.
import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { copyFile, lstat, mkdir, mkdtemp, readFile, readdir, realpath, rm, symlink, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  assembleLocalExecDependencyCandidate, patchTreeSitterBinding, runOwnedChild, withOwnedNativeBuildWork,
  verifyLocalExecDependencyLoad, verifyLocalExecDependencyStage,
} from "./build-local-exec-dependencies.mjs";
import { createLocalExecDependencyFixture, STOCK_TREE_SITTER_BINDING } from "./local-exec-dependencies.fixture.mjs";

const scriptsRoot = path.dirname(fileURLToPath(import.meta.url));
const realDesktopRoot = path.resolve(scriptsRoot, "..");
const manifestName = "local-exec-dependencies.json";
function hash(bytes) { return createHash("sha256").update(bytes).digest("hex"); }
function sort(files) { return files.sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0); }
async function fixture(t, domain = "node") {
  const desktopRoot = await mkdtemp(path.join(os.tmpdir(), "ldb-contract-"));
  t.after(() => rm(desktopRoot, { recursive: true, force: true }));
  await mkdir(path.join(desktopRoot, "scripts"));
  for (const file of ["package.json", "package-lock.json"]) await copyFile(path.join(realDesktopRoot, file), path.join(desktopRoot, file));
  await copyFile(path.join(scriptsRoot, "build-local-exec-dependencies.mjs"), path.join(desktopRoot, "scripts/build-local-exec-dependencies.mjs"));
  const receipt = createLocalExecDependencyFixture({ desktopRoot, domain });
  const stageRoot = path.join(desktopRoot, "dist", domain === "node" ? "node-deps" : "deps");
  return { desktopRoot, stageRoot, domain, receipt };
}
async function save(f) {
  f.receipt.build_input_sha256 = hash(JSON.stringify(f.receipt.build_inputs));
  await writeFile(path.join(f.stageRoot, manifestName), JSON.stringify(f.receipt, null, 2) + "\n");
}
async function scan(root, relative = "") {
  const files = [];
  for (const name of await readdir(path.join(root, relative))) {
    const file = relative ? relative + "/" + name : name;
    if (file === manifestName) continue;
    const stat = await lstat(path.join(root, file));
    if (stat.isDirectory()) files.push(...await scan(root, file));
    else {
      assert.equal(stat.isFile(), true);
      const bytes = await readFile(path.join(root, file));
      files.push({ path: file, bytes: bytes.length, sha256: hash(bytes) });
    }
  }
  return sort(files);
}
// Recompute output declarations after a deliberate mutation. Deeper negative
// controls therefore cannot pass merely because the outer file digest changed.
async function refreshOutputs(f, { headerInputs = false } = {}) {
  const files = await scan(f.stageRoot);
  f.receipt.stage_files = files;
  f.receipt.native_outputs = files.filter(p => p.path.endsWith(".node"));
  f.receipt.headers.files = files.filter(p => p.path.startsWith(".headers/"));
  for (const p of f.receipt.packages) {
    p.files = files.filter(file => file.path.startsWith(p.name + "/") || file.path.startsWith("node_modules/" + p.name + "/"));
  }
  if (headerInputs) {
    f.receipt.build_inputs = sort([
      ...f.receipt.build_inputs.filter(p => !p.path.startsWith("headers/")),
      ...f.receipt.headers.files.map(p => ({ ...p, path: "headers/" + p.path.slice(".headers/".length) })),
    ]);
  }
  await save(f);
}
function verify(f) { return verifyLocalExecDependencyStage(f); }


async function assemblyFixture(t, domain = "node") {
  const original = await fixture(t, domain);
  const work = await realpath(await mkdtemp(path.join(original.desktopRoot, "assembly-work-")));
  const buildRoot = path.join(work, "build");
  await mkdir(buildRoot);
  // Controlled bytes exercise assembly only. The ordinary first contract still
  // requires two real, compiled and loaded native domains in Actions.
  for (const file of original.receipt.stage_files) {
    const output = path.join(buildRoot, file.path);
    await mkdir(path.dirname(output), { recursive: true });
    await copyFile(path.join(original.stageRoot, file.path), output);
  }
  const sources = new Map();
  for (const pkg of original.receipt.packages) {
    const root = path.join(work, "sources", pkg.name);
    const prefix = "source/" + pkg.name + "/";
    const members = original.receipt.build_inputs.filter(file => file.path.startsWith(prefix))
      .map(file => ({ ...file, path: file.path.slice(prefix.length) }));
    for (const member of members) {
      const bytes = pkg.name === "tree-sitter" && member.path === "binding.gyp"
        ? Buffer.from(STOCK_TREE_SITTER_BINDING)
        : await readFile(path.join(original.stageRoot, pkg.name, member.path));
      assert.deepEqual({ path: member.path, bytes: bytes.length, sha256: hash(bytes) }, member);
      await mkdir(path.dirname(path.join(root, member.path)), { recursive: true });
      await writeFile(path.join(root, member.path), bytes);
    }
    sources.set(pkg.name, { root, members });
  }
  return {
    original, work, buildRoot, sources,
    headers: structuredClone(original.receipt.headers),
    nativeOutputs: structuredClone(original.receipt.native_outputs),
  };
}
async function assembleFixture(f) {
  const stageRoot = await assembleLocalExecDependencyCandidate(f);
  const assembled = { ...f.original, stageRoot, receipt: structuredClone(f.original.receipt) };
  await save(assembled);
  return assembled;
}

test("LDB real Node and Electron stages load the parser under their actual runtimes", { timeout: 120000 }, async () => {
  const node = await verifyLocalExecDependencyLoad({
    desktopRoot: realDesktopRoot, stageRoot: path.join(realDesktopRoot, "dist/node-deps"), domain: "node",
  });
  const electron = await verifyLocalExecDependencyLoad({
    desktopRoot: realDesktopRoot, stageRoot: path.join(realDesktopRoot, "dist/deps"), domain: "electron",
  });
  assert.equal(node.runtime.version, process.versions.node);
  assert.equal(electron.runtime.version, "43.4.0");
  assert.notEqual(node.domain, electron.domain);
  assert.notEqual(node.build_input_sha256, electron.build_input_sha256);
  assert.equal(node.load_probe.parser_language_loaded, true);
  assert.equal(electron.load_probe.parser_language_loaded, true);
});


test("LDB fresh assembly preserves declared inputs and excludes compiler byproducts", async t => {
  for (const domain of ["node", "electron"]) {
    const f = await assemblyFixture(t, domain);
    const byproducts = [
      "tree-sitter/node_modules/node-addon-api/node_addon_api.Makefile",
      "tree-sitter/build/Makefile",
      "tree-sitter-bash/build/Release/obj.target/compiler.o",
      "node_modules/node-addon-api/generated-build.mk",
      "generated-build.json",
    ];
    for (const file of byproducts) {
      await mkdir(path.dirname(path.join(f.buildRoot, file)), { recursive: true });
      await writeFile(path.join(f.buildRoot, file), "controlled compiler byproduct: " + file);
    }
    const assembled = await assembleFixture(f);
    assert.deepEqual(await scan(assembled.stageRoot), f.original.receipt.stage_files);
    assert.deepEqual(await verify(assembled), f.original.receipt);
    for (const file of byproducts) {
      await assert.rejects(lstat(path.join(assembled.stageRoot, file)), { code: "ENOENT" });
      assert.equal(await readFile(path.join(f.buildRoot, file), "utf8"), "controlled compiler byproduct: " + file);
    }
    for (const file of [...f.headers.files, ...f.nativeOutputs]) {
      assert.deepEqual(await readFile(path.join(assembled.stageRoot, file.path)),
        await readFile(path.join(f.buildRoot, file.path)));
    }
    assert.deepEqual(await verify(f.original), f.original.receipt);
  }
});

test("LDB assembled stage rejects generated sources behind rewritten receipts", async t => {
  const f = await assemblyFixture(t);
  const assembled = await assembleFixture(f);
  const byproduct = "tree-sitter/node_modules/node-addon-api/node_addon_api.Makefile";
  await mkdir(path.dirname(path.join(assembled.stageRoot, byproduct)), { recursive: true });
  await writeFile(path.join(assembled.stageRoot, byproduct), "generated makefile, not declared package source");
  await refreshOutputs(assembled);
  await assert.rejects(verify(assembled), /missing or changed staged source input: tree-sitter\/node_modules\/node-addon-api\/node_addon_api.Makefile/);
  assert.deepEqual(await verify(f.original), f.original.receipt);
});

test("LDB fresh assembly refuses changed source header and native inputs", async t => {
  for (const kind of ["source", "header", "native"]) {
    const f = await assemblyFixture(t);
    const changed = kind === "source"
      ? path.join(f.sources.get("tree-sitter-bash").root, "src/parser.c")
      : path.join(f.buildRoot, kind === "header" ? ".headers/SHASUMS256.txt" : f.nativeOutputs[0].path);
    await writeFile(changed, "changed after identity capture");
    await assert.rejects(assembleLocalExecDependencyCandidate(f),
      kind === "source" ? /package source changed while copying/
        : kind === "header" ? /headers changed during native build/
        : /declared native build input changed/);
    assert.equal(await readFile(changed, "utf8"), "changed after identity capture");
    assert.deepEqual(await verify(f.original), f.original.receipt);
  }
});

test("LDB fresh assembly refuses unsafe or incomplete declared build inputs", async t => {
  for (const kind of ["header-traversal", "native-traversal", "native-missing", "native-unexpected", "source-unexpected"]) {
    const f = await assemblyFixture(t);
    if (kind === "header-traversal") f.headers.files[0].path = "../outside";
    if (kind === "native-traversal") f.nativeOutputs[0].path = "../outside.node";
    if (kind === "native-missing") f.nativeOutputs.pop();
    if (kind === "native-unexpected") f.nativeOutputs[0].path += ".other";
    if (kind === "source-unexpected") f.sources.set("unexpected", f.sources.get("tree-sitter"));
    await assert.rejects(assembleLocalExecDependencyCandidate(f),
      /unsafe, duplicated or unsorted|unexpected or missing declared native build outputs|incomplete or unexpected native package source set/);
    assert.deepEqual(await verify(f.original), f.original.receipt);
  }
});

test("LDB fresh assembly refuses redirected inputs and occupied candidates", async t => {
  for (const kind of ["source", "header", "native", "candidate"]) {
    const f = await assemblyFixture(t);
    if (kind === "candidate") {
      const candidate = path.join(f.work, "candidate");
      await mkdir(candidate);
      await writeFile(path.join(candidate, "marker"), "existing candidate");
      await assert.rejects(assembleLocalExecDependencyCandidate(f), { code: "EEXIST" });
      assert.equal(await readFile(path.join(candidate, "marker"), "utf8"), "existing candidate");
    } else {
      const relative = kind === "header" ? ".headers"
        : kind === "native" ? path.dirname(f.nativeOutputs[0].path) : "tree-sitter-bash/src";
      const destination = kind === "source"
        ? path.join(f.sources.get("tree-sitter-bash").root, "src")
        : path.join(f.buildRoot, relative);
      await rm(destination, { recursive: true });
      await symlink(path.join(f.original.stageRoot, relative), destination,
        process.platform === "win32" ? "junction" : "dir");
      await assert.rejects(assembleLocalExecDependencyCandidate(f),
        kind === "source" ? /redirected package source input/
          : kind === "header" ? /ordinary directory/
          : /redirected declared native build input/);
    }
    assert.deepEqual(await verify(f.original), f.original.receipt);
  }
});

test("LDB structural checking accepts complete synthetic stages without installed dependencies", async t => {
  for (const domain of ["node", "electron"]) {
    const f = await fixture(t, domain);
    await assert.rejects(lstat(path.join(f.desktopRoot, "node_modules")), { code: "ENOENT" });
    assert.deepEqual(await verify(f), f.receipt);
  }
});

test("LDB structural checking refuses wrong-domain stages without fallback", async t => {
  const f = await fixture(t, "electron");
  await assert.rejects(verify({ ...f, domain: "node" }), /domain\/contract mismatch/);
});

test("LDB structural checking refuses absent native outputs even with a rewritten receipt", async t => {
  const f = await fixture(t);
  await rm(path.join(f.stageRoot, f.receipt.native_outputs[0].path));
  await refreshOutputs(f);
  await assert.rejects(verify(f), /unexpected or missing native outputs/);
});

test("LDB structural checking detects changed native bytes", async t => {
  const f = await fixture(t);
  const file = path.join(f.stageRoot, f.receipt.native_outputs[0].path);
  const bytes = await readFile(file);
  bytes[0] ^= 1;
  await writeFile(file, bytes);
  await assert.rejects(verify(f), /membership or file bytes mismatch/);
});

test("LDB structural checking rejects undeclared stage members", async t => {
  const f = await fixture(t);
  await writeFile(path.join(f.stageRoot, "unexpected.bin"), "not in the manifest");
  await assert.rejects(verify(f), /membership or file bytes mismatch/);
  await rm(path.join(f.stageRoot, "unexpected.bin"));
  await mkdir(path.join(f.stageRoot, "undeclared-empty"));
  await assert.rejects(verify(f), /undeclared empty stage directory/);
});

test("LDB structural checking rejects a symlink or junction escape", async t => {
  const f = await fixture(t);
  const outside = path.join(f.desktopRoot, "outside");
  await mkdir(outside);
  await writeFile(path.join(outside, "outside.txt"), "must not be traversed");
  await symlink(outside, path.join(f.stageRoot, "escape"), process.platform === "win32" ? "junction" : "dir");
  await assert.rejects(verify(f), /symlink/);
  assert.equal(await readFile(path.join(outside, "outside.txt"), "utf8"), "must not be traversed");
});

test("LDB structural checking rejects traversal and duplicate manifest paths", async t => {
  const f = await fixture(t);
  const original = structuredClone(f.receipt.stage_files);
  f.receipt.stage_files[0].path = "../outside";
  await save(f);
  await assert.rejects(verify(f), /unsafe, duplicated or unsorted/);
  f.receipt.stage_files = [original[0], ...original];
  await save(f);
  await assert.rejects(verify(f), /unsafe, duplicated or unsorted/);
});

test("LDB structural checking rejects stale lock and builder identities", async t => {
  const f = await fixture(t);
  const lockPath = path.join(f.desktopRoot, "package-lock.json");
  const lock = await readFile(lockPath);
  await writeFile(lockPath, Buffer.concat([lock, Buffer.from("\n")]));
  await assert.rejects(verify(f), /lock identity mismatch/);
  await writeFile(lockPath, lock);
  f.receipt.build_inputs.find(p => p.path === "scripts/build-local-exec-dependencies.mjs").sha256 = "0".repeat(64);
  await save(f);
  await assert.rejects(verify(f), /builder identity mismatch/);
});

test("LDB structural checking rejects missing causal source inputs", async t => {
  const f = await fixture(t);
  f.receipt.build_inputs = f.receipt.build_inputs.filter(p => p.path !== "source/tree-sitter-bash/src/parser.c");
  await save(f);
  await assert.rejects(verify(f), /missing or changed staged source input/);
});

test("LDB structural checking detects changed package source behind rewritten output hashes", async t => {
  const f = await fixture(t);
  await writeFile(path.join(f.stageRoot, "tree-sitter-bash/src/parser.c"), "altered source\n");
  await refreshOutputs(f);
  await assert.rejects(verify(f), /missing or changed staged source input/);
});

test("LDB structural checking rejects missing package license artifacts", async t => {
  const f = await fixture(t);
  await rm(path.join(f.stageRoot, "tree-sitter/LICENSE"));
  await refreshOutputs(f);
  await assert.rejects(verify(f), /missing package license/);
});

test("LDB structural checking rejects unqualified native fallback locations", async t => {
  const f = await fixture(t);
  const extra = path.join(f.stageRoot, "tree-sitter/prebuilds/fixture");
  await mkdir(extra, { recursive: true });
  await writeFile(path.join(extra, "runtime.node"), "unqualified native output");
  await refreshOutputs(f);
  // Give the invented source a matching input to reach the fallback policy.
  const file = f.receipt.stage_files.find(p => p.path.endsWith("prebuilds/fixture/runtime.node"));
  f.receipt.build_inputs.push({ ...file, path: "source/" + file.path });
  sort(f.receipt.build_inputs);
  await save(f);
  await assert.rejects(verify(f), /unqualified native fallback members/);
});

test("LDB structural checking rejects altered official archive checksums", async t => {
  const f = await fixture(t);
  const archivePath = ".headers/node-v" + f.receipt.runtime.version + "-headers.tar.gz";
  const altered = Buffer.from("altered archive even with internally updated output hashes");
  await writeFile(path.join(f.stageRoot, archivePath), altered);
  f.receipt.headers.archive_sha256 = hash(altered);
  await refreshOutputs(f, { headerInputs: true });
  await assert.rejects(verify(f), /official header archive checksum mismatch/);
});

test("LDB structural checking rejects mismatched runtime header ABI", async t => {
  const f = await fixture(t);
  const header = path.join(f.stageRoot, ".headers", f.receipt.runtime.version, "include/node/node_version.h");
  const source = await readFile(header, "utf8");
  await writeFile(header, source.replace(/NODE_MODULE_VERSION \d+/, "NODE_MODULE_VERSION 999999"));
  await refreshOutputs(f, { headerInputs: true });
  await assert.rejects(verify(f), /headers disagree with runtime modules ABI/);
});

test("LDB structural checking rejects a failed parser load receipt", async t => {
  const f = await fixture(t);
  f.receipt.load_probe.exit = 1;
  f.receipt.load_probe.parser_language_loaded = false;
  await save(f);
  await assert.rejects(verify(f), /missing qualified load-probe receipt/);
});

test("LDB stock-only C++ patch checks both exact input and output identities", () => {
  const patched = patchTreeSitterBinding(STOCK_TREE_SITTER_BINDING);
  assert.equal(hash(STOCK_TREE_SITTER_BINDING), "d94979694bc3b1854ba0030be33e9941df9867f99d956117b704e9cfaca37a7d");
  assert.equal(hash(patched), "8f7eb83b314277458e41d0096c03d58849c2921ee8705b14376b712e1eb647d4");
  assert.throws(() => patchTreeSitterBinding(STOCK_TREE_SITTER_BINDING + "\n"), /unexpected stock/);
  assert.throws(() => patchTreeSitterBinding(patched), /unexpected stock/);
});

test("LDB owned child runner rejects spawn errors and nonzero close", { timeout: 10000 }, async t => {
  const f = await fixture(t);
  await assert.rejects(runOwnedChild(path.join(f.desktopRoot, "missing-executable"), [], { timeoutMs: 1000 }), /ENOENT/);
  await assert.rejects(runOwnedChild(process.execPath, ["-e", "process.exit(7)"], { timeoutMs: 5000 }), /exit 7/);
  const result = await runOwnedChild(process.execPath, ["-e", "process.stdout.write('observed-close')"], { timeoutMs: 5000 });
  assert.equal(result.stdout, "observed-close");
  assert.equal(result.exit, 0);
});

test("LDB owned child deadline cleanup leaves an unrelated process alive", { timeout: 15000 }, async t => {
  const sibling = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], { stdio: "ignore" });
  await once(sibling, "spawn");
  t.after(async () => {
    if (sibling.exitCode === null && sibling.signalCode === null) {
      const closed = once(sibling, "close");
      sibling.kill("SIGKILL");
      await closed;
    }
  });
  await assert.rejects(runOwnedChild(process.execPath, ["-e", "setInterval(() => {}, 1000)"], { timeoutMs: 300 }), /deadline exceeded/);
  assert.equal(sibling.exitCode, null);
  assert.equal(sibling.signalCode, null);
});

test("LDB owned child runner waits for close without signaling an exited child", { timeout: 15000 }, async t => {
  const f = await fixture(t);
  const marker = path.join(f.desktopRoot, "descendant-finished");
  const grandchild = "setTimeout(() => require('node:fs').writeFileSync(process.argv[1], 'finished'), 6500)";
  const parent = [
    "const {spawn}=require('node:child_process');",
    "spawn(process.execPath,['-e',process.argv[1],process.argv[2]],{stdio:['ignore',process.stdout,process.stderr]});",
    "process.exit(0);",
  ].join("");
  await assert.rejects(runOwnedChild(process.execPath, ["-e", parent, grandchild, marker], { timeoutMs: 5000 }), /deadline exceeded/);
  assert.equal(await readFile(marker, "utf8"), "finished");
});

// This is an actual filesystem/owned-child failure control for the build-work
// boundary, not a synthetic native compile or parser-load claim.
test("LDB owned native build failure retains only its bound work root", { timeout: 15000 }, async t => {
  const f = await fixture(t);
  const workRoot = path.join(f.desktopRoot, "evidence", "owned-work");
  await mkdir(workRoot, { recursive: true });
  const outside = path.join(f.desktopRoot, "unrelated-build");
  await mkdir(outside);
  await writeFile(path.join(outside, "marker"), "unrelated");
  let failedWork;
  await assert.rejects(withOwnedNativeBuildWork({ stageRoot: f.stageRoot, domain: "node", workRoot }, async work => {
    failedWork = work;
    assert.equal(path.dirname(work), workRoot);
    const child = [
      "const fs=require('node:fs'),p=require('node:path'),root=process.argv[1];",
      "fs.mkdirSync(p.join(root,'candidate','.headers'),{recursive:true});",
      "fs.writeFileSync(p.join(root,'candidate','.headers','archive.tar.gz'),'owned-header');",
      "fs.mkdirSync(p.join(root,'previous'));",
      "fs.writeFileSync(p.join(root,'previous','runtime.node'),'prior-native');",
      "process.exit(17);",
    ].join("");
    await runOwnedChild(process.execPath, ["-e", child, work], { timeoutMs: 5000 });
  }), error => /exit 17/.test(error.message) && error.message.includes(failedWork));
  assert.equal(await readFile(path.join(failedWork, "candidate/.headers/archive.tar.gz"), "utf8"), "owned-header");
  assert.equal(await readFile(path.join(failedWork, "previous/runtime.node"), "utf8"), "prior-native");
  assert.equal(await readFile(path.join(outside, "marker"), "utf8"), "unrelated");
  assert.deepEqual(await readdir(workRoot), [path.basename(failedWork)]);
  let successfulWork;
  assert.equal(await withOwnedNativeBuildWork({ stageRoot: f.stageRoot, domain: "node", workRoot }, async work => {
    successfulWork = work;
    await writeFile(path.join(work, "complete"), "settled");
    return "qualified";
  }), "qualified");
  await assert.rejects(lstat(successfulWork), { code: "ENOENT" });
  assert.deepEqual(await readdir(workRoot), [path.basename(failedWork)]);
  const link = path.join(f.desktopRoot, "redirected-work");
  await symlink(workRoot, link, process.platform === "win32" ? "junction" : "dir");
  await assert.rejects(withOwnedNativeBuildWork({ stageRoot: f.stageRoot, domain: "node", workRoot: link }, async () => {
    assert.fail("redirected work root must not start a build");
  }), /ordinary owned directory/);
  assert.equal(await readFile(path.join(outside, "marker"), "utf8"), "unrelated");
});
