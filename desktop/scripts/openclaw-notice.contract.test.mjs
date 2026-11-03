import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, "../..");
const productionChecker = path.join(scriptDir, "check-electron-runtime-bundle.mjs");
const sourceRelative = "desktop/resources/licenses/openclaw/LICENSE";
const attributesRelative = "desktop/resources/licenses/openclaw/.gitattributes";
const lockRelative = "projects/openclaw-rust/upstream.lock.json";
const sourceBytes = fs.readFileSync(path.join(repoRoot, sourceRelative));
const sourceLock = JSON.parse(fs.readFileSync(path.join(repoRoot, lockRelative), "utf8"));
const runtimeFiles = [
  "dist/electron-main/main.cjs",
  "dist/electron-main/miniapp-native-carrier.cjs",
  "dist/electron-preload/preload.cjs",
  "dist/electron-preload/preload-miniapp-tools.cjs",
  "dist/node-agent-coordinator/main.cjs",
  "dist/renderer/index.html",
];
const validPreload = 'require("electron").contextBridge.exposeInMainWorld("fabushi", { invoke() {} });\n';
const validCarrier = 'void process.parentPort; const env = "MAHAYANA_APP_HOST_BIN"; const binary = "mahayana-app-host";\n';

function write(filename, contents) {
  fs.mkdirSync(path.dirname(filename), { recursive: true });
  fs.writeFileSync(filename, contents);
}

function writeJson(filename, value) {
  write(filename, JSON.stringify(value, null, 2) + "\n");
}

function fixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "fabushi-onp-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const tree = {
    root,
    desktop: path.join(root, "desktop"),
    source: path.join(root, sourceRelative),
    attributes: path.join(root, attributesRelative),
    lock: path.join(root, lockRelative),
    package: path.join(root, "desktop/package.json"),
    checker: path.join(root, "desktop/scripts/check-electron-runtime-bundle.mjs"),
    resources: path.join(root, "installed/Package With Spaces/Resources"),
  };
  for (const [from, to] of [
    [productionChecker, tree.checker],
    [path.join(repoRoot, sourceRelative), tree.source],
    [path.join(repoRoot, attributesRelative), tree.attributes],
    [path.join(repoRoot, lockRelative), tree.lock],
    [path.join(repoRoot, "desktop/package.json"), tree.package],
  ]) {
    fs.mkdirSync(path.dirname(to), { recursive: true });
    fs.copyFileSync(from, to);
  }
  return tree;
}

// These byte fixtures exercise the existing checker, not an Electron build or
// installer. Copy the production checker unchanged so its actual CLI is used.
function stageRuntimeFixture(tree) {
  for (const relative of runtimeFiles) write(path.join(tree.desktop, relative), "// controlled artifact fixture\n");
  write(path.join(tree.desktop, "dist/electron-preload/preload-miniapp-tools.cjs"), validPreload);
  write(path.join(tree.desktop, "dist/node-agent-coordinator/main.cjs"), validCarrier);
}

function stagePackagedFixture(tree) {
  write(path.join(tree.resources, "licenses/openclaw/LICENSE"), sourceBytes);
}

function snapshot(root) {
  const entries = [];
  function visit(directory) {
    for (const name of fs.readdirSync(directory).sort()) {
      const filename = path.join(directory, name);
      const relative = path.relative(root, filename);
      const stat = fs.lstatSync(filename);
      if (stat.isDirectory()) {
        entries.push([relative, "directory"]);
        visit(filename);
      } else if (stat.isSymbolicLink()) {
        entries.push([relative, "symlink", fs.readlinkSync(filename)]);
      } else {
        entries.push([relative, "file", fs.readFileSync(filename).toString("base64")]);
      }
    }
  }
  visit(root);
  return entries;
}

function invoke(checker, args, cwd) {
  const result = spawnSync(process.execPath, [checker, ...args], {
    cwd,
    encoding: "utf8",
    timeout: 10_000,
  });
  assert.ifError(result.error);
  assert.equal(result.signal, null, `checker terminated by signal: ${result.stderr}`);
  return result;
}

function run(tree, args = []) {
  const before = snapshot(tree.root);
  const result = invoke(tree.checker, args, tree.root);
  assert.deepEqual(snapshot(tree.root), before, "checker must not create, repair or normalize its inputs");
  return result;
}

function accepted(result, mode) {
  assert.equal(result.status, 0, result.stderr);
  const reports = result.stdout.split(/\r?\n/).filter((line) => line.startsWith('{"check":"ONP-v1",'));
  assert.equal(reports.length, 1, result.stdout);
  const report = JSON.parse(reports[0]);
  assert.equal(report.mode, mode);
  assert.equal(report.source_blob, sourceLock.root_inputs.LICENSE);
  assert.equal(report.source_commit, sourceLock.upstream.commit);
  assert.equal(report.source_tree, sourceLock.upstream.tree);
  assert.equal(report.source_bytes, sourceBytes.length);
  return report;
}

function refused(result, reason) {
  assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stderr, reason);
  assert.doesNotMatch(result.stdout, /"check":"ONP-v1"/);
}

function changePackage(tree, change) {
  const config = JSON.parse(fs.readFileSync(tree.package, "utf8"));
  const index = config.build.extraResources.findIndex((entry) => entry.from === "resources/licenses/openclaw");
  assert.notEqual(index, -1, "production common mapping must exist before applying a negative control");
  change(config, index);
  writeJson(tree.package, config);
}

test("onp_real_notice_only_uses_locked_source_and_common_mapping", () => {
  const result = invoke(productionChecker, ["--notice-only"], repoRoot);
  const report = accepted(result, "notice-only");
  assert.equal(report.resource_from, "resources/licenses/openclaw/LICENSE");
  assert.equal(report.resource_to, "licenses/openclaw/LICENSE");
  assert.match(report.scope, /inputs and common resource mapping only/);
  assert.doesNotMatch(result.stdout, /Desktop runtime artifact graph passed/);
});

test("onp_notice_only_accepts_controlled_inputs_without_runtime_artifacts", (t) => {
  const tree = fixture(t);
  assert.equal(fs.existsSync(path.join(tree.desktop, "dist")), false);
  const result = run(tree, ["--notice-only"]);
  accepted(result, "notice-only");
  assert.doesNotMatch(result.stdout, /Desktop runtime artifact graph passed/);
});

test("onp_default_mode_retains_all_required_runtime_artifacts", (t) => {
  refused(run(fixture(t)), /Desktop runtime artifact graph is incomplete/);
  for (const relative of runtimeFiles) {
    const tree = fixture(t);
    stageRuntimeFixture(tree);
    fs.unlinkSync(path.join(tree.desktop, relative));
    const result = run(tree);
    refused(result, /Desktop runtime artifact graph is incomplete/);
    assert.ok(result.stderr.includes(relative), `missing artifact must be identified: ${relative}`);
  }
});

test("onp_default_mode_accepts_complete_controlled_runtime_and_notice", (t) => {
  const tree = fixture(t);
  stageRuntimeFixture(tree);
  const result = run(tree);
  accepted(result, "runtime");
  assert.match(result.stdout, /Desktop runtime artifact graph passed/);
});

test("onp_missing_source_notice_is_refused_without_repair", (t) => {
  const tree = fixture(t);
  fs.unlinkSync(tree.source);
  refused(run(tree, ["--notice-only"]), /OpenClaw source notice is missing, unreadable or not a regular file/);
  assert.equal(fs.existsSync(tree.source), false);
});

test("onp_changed_source_bytes_are_refused_without_normalization", (t) => {
  const mutations = [
    Buffer.from(sourceBytes.toString("utf8").replace("OpenClaw Foundation", "Different Copyright Holder")),
    sourceBytes.subarray(0, sourceBytes.length - 1),
    Buffer.from(sourceBytes.toString("utf8").replaceAll("\n", "\r\n")),
    Buffer.concat([Buffer.from([0xef, 0xbb, 0xbf]), sourceBytes]),
  ];
  for (const bytes of mutations) {
    const tree = fixture(t);
    write(tree.source, bytes);
    refused(run(tree, ["--notice-only"]), /OpenClaw source notice Git blob differs from root_inputs.LICENSE/);
  }
});

test("onp_non_file_source_notice_is_refused", (t) => {
  const tree = fixture(t);
  fs.unlinkSync(tree.source);
  fs.mkdirSync(tree.source);
  refused(run(tree, ["--notice-only"]), /OpenClaw source notice is missing, unreadable or not a regular file/);
  const linked = fixture(t);
  const otherDirectory = path.join(linked.root, "source-copy");
  write(path.join(otherDirectory, "LICENSE"), sourceBytes);
  fs.unlinkSync(linked.source);
  fs.rmdirSync(path.dirname(linked.source));
  fs.symlinkSync(otherDirectory, path.dirname(linked.source), "junction");
  refused(run(linked, ["--notice-only"]), /OpenClaw source notice directory.*not a real directory/);
});

test("onp_missing_malformed_and_invalid_lock_identity_are_refused", (t) => {
  for (const value of [undefined, null, 17, [], "invalid", "0".repeat(40)]) {
    const tree = fixture(t);
    const lock = JSON.parse(fs.readFileSync(tree.lock, "utf8"));
    lock.root_inputs.LICENSE = value;
    writeJson(tree.lock, lock);
    refused(run(tree, ["--notice-only"]), /invalid upstream or root_inputs.LICENSE identity|differs from root_inputs.LICENSE/);
  }
  const absent = fixture(t);
  fs.unlinkSync(absent.lock);
  refused(run(absent, ["--notice-only"]), /OpenClaw source lock is missing/);
  const malformed = fixture(t);
  write(malformed.lock, "{broken-json\n");
  refused(run(malformed, ["--notice-only"]), /OpenClaw source lock is not valid JSON/);
  for (const mutate of [
    (lock) => { lock.format_version = 2; },
    (lock) => { lock.project_id = "different-project"; },
    (lock) => { lock.upstream.repository = "different/repository"; },
    (lock) => { lock.upstream.commit = [lock.upstream.commit]; },
    (lock) => { lock.upstream.tree = [lock.upstream.tree]; },
    (lock) => { delete lock.upstream; },
  ]) {
    const tree = fixture(t);
    const lock = JSON.parse(fs.readFileSync(tree.lock, "utf8"));
    mutate(lock);
    writeJson(tree.lock, lock);
    refused(run(tree, ["--notice-only"]), /invalid upstream or root_inputs.LICENSE identity/);
  }
});

test("onp_root_license_identity_cannot_follow_discovery_candidate", (t) => {
  const tree = fixture(t);
  const lock = JSON.parse(fs.readFileSync(tree.lock, "utf8"));
  lock.live_candidate = { root_inputs: { LICENSE: "f".repeat(40) } };
  writeJson(tree.lock, lock);
  accepted(run(tree, ["--notice-only"]), "notice-only");
  lock.live_candidate.root_inputs.LICENSE = sourceLock.root_inputs.LICENSE;
  lock.root_inputs.LICENSE = "f".repeat(40);
  writeJson(tree.lock, lock);
  refused(run(tree, ["--notice-only"]), /differs from root_inputs.LICENSE/);
});

test("onp_missing_duplicate_wrong_and_broad_common_mappings_are_refused", (t) => {
  const mutations = [
    (config) => { delete config.build.extraResources; },
    (config, index) => { config.build.extraResources.splice(index, 1); },
    (config, index) => { config.build.extraResources.push({ ...config.build.extraResources[index] }); },
    (config, index) => { config.build.extraResources[index].from = "resources/licenses/other"; },
    (config, index) => { config.build.extraResources[index].to = "licenses/other"; },
    (config, index) => { config.build.extraResources[index].filter = ["**/*"]; },
    (config, index) => { config.build.extraResources[index].filter = ["LICENSE", "unexpected.txt"]; },
    (config, index) => { config.build.extraResources[index].filter = "LICENSE"; },
    (config, index) => { config.build.extraResources[index].unexpected = true; },
    (config, index) => { config.build.extraResources[index] = "resources/licenses/openclaw/**"; },
    (config, index) => {
      config.build.extraResources.push({ ...config.build.extraResources[index], from: "./resources/licenses/openclaw/", to: "./licenses/openclaw/" });
    },
  ];
  for (const mutate of mutations) {
    const tree = fixture(t);
    changePackage(tree, mutate);
    refused(run(tree, ["--notice-only"]), /OpenClaw notice.*extraResources mapping/);
  }
});

test("onp_platform_only_mapping_cannot_replace_the_common_mapping", (t) => {
  const tree = fixture(t);
  changePackage(tree, (config, index) => {
    config.build.mac.extraResources = [config.build.extraResources[index]];
    config.build.extraResources.splice(index, 1);
  });
  refused(run(tree, ["--notice-only"]), /requires one common extraResources mapping/);
});

test("onp_packaged_resources_consumes_exact_bytes_at_the_declared_destination", (t) => {
  const tree = fixture(t);
  stagePackagedFixture(tree);
  for (const directory of [tree.resources, path.relative(tree.root, tree.resources)]) {
    const result = run(tree, ["--packaged-resources", directory]);
    const report = accepted(result, "packaged-resources");
    assert.equal(report.packaged_notice, path.join(tree.resources, "licenses/openclaw/LICENSE"));
    assert.equal(report.packaged_blob, sourceLock.root_inputs.LICENSE);
    assert.match(report.scope, /supplied packaged Resources only/);
    assert.doesNotMatch(result.stdout, /Desktop runtime artifact graph passed/);
  }
});

test("onp_missing_changed_non_file_and_wrong_destination_packaged_notices_are_refused", (t) => {
  for (const mutation of ["missing", "changed", "directory", "wrong-destination", "linked-directory"]) {
    const tree = fixture(t);
    stagePackagedFixture(tree);
    const filename = path.join(tree.resources, "licenses/openclaw/LICENSE");
    fs.unlinkSync(filename);
    if (mutation === "changed") write(filename, Buffer.concat([sourceBytes, Buffer.from("changed\n")]));
    if (mutation === "directory") fs.mkdirSync(filename);
    if (mutation === "wrong-destination") write(path.join(tree.resources, "openclaw/LICENSE"), sourceBytes);
    if (mutation === "linked-directory") {
      fs.rmdirSync(path.dirname(filename));
      fs.symlinkSync(path.dirname(tree.source), path.dirname(filename), "junction");
    }
    refused(run(tree, ["--packaged-resources", tree.resources]), /OpenClaw packaged notice/);
  }
});

test("onp_packaged_resources_requires_an_existing_directory", (t) => {
  const tree = fixture(t);
  refused(run(tree, ["--packaged-resources", tree.resources]), /Packaged Resources must be an existing directory/);
  write(tree.resources, "not a directory\n");
  refused(run(tree, ["--packaged-resources", tree.resources]), /Packaged Resources must be an existing directory/);
});

test("onp_unknown_conflicting_and_incomplete_cli_options_are_refused", (t) => {
  const tree = fixture(t);
  stageRuntimeFixture(tree);
  stagePackagedFixture(tree);
  const argumentsList = [
    ["--notices-only"],
    ["--notice-only", "--notice-only"],
    ["--notice-only", "--packaged-resources", tree.resources],
    ["--packaged-resources", tree.resources, "--notice-only"],
    ["--packaged-resources"],
    ["--packaged-resources", ""],
    ["--packaged-resources", "   "],
    ["--packaged-resources", "--notice-only"],
    ["--packaged-resources", tree.resources, "extra"],
    ["--notice-only=1"],
    ["ordinary-positional"],
    ["--", "--notice-only"],
    [`--packaged-resources=${tree.resources}`],
  ];
  for (const args of argumentsList) refused(run(tree, args), /Invalid or conflicting options/);
});

test("onp_default_mode_retains_preload_and_coordinator_contracts", (t) => {
  const preloadPath = "dist/electron-preload/preload-miniapp-tools.cjs";
  const carrierPath = "dist/node-agent-coordinator/main.cjs";
  const mutations = [
    [preloadPath, "// no bridge\n", /must expose exactly fabushi.invoke/],
    [preloadPath, 'require("electron").contextBridge.exposeInMainWorld("other", { invoke() {} });', /must expose exactly fabushi.invoke/],
    [preloadPath, validPreload + validPreload, /must expose exactly fabushi.invoke/],
    [preloadPath, 'require("electron").contextBridge.exposeInMainWorld("fabushi", { invoke() {}, extra() {} });', /must expose exactly fabushi.invoke/],
    [preloadPath, 'require("node:fs");', /requires forbidden module node:fs/],
    [carrierPath, "// no carrier\n", /does not read the utility-process parent port/],
    [carrierPath, 'void require("electron").parentPort; const env = "MAHAYANA_APP_HOST_BIN"; const binary = "mahayana-app-host";', /incorrectly reads electron.parentPort/],
    [carrierPath, 'const parentPort = {}; const env = "MAHAYANA_APP_HOST_BIN"; const binary = "mahayana-app-host";', /must read process.parentPort/],
    [carrierPath, 'void process.parentPort; const binary = "mahayana-app-host";', /must bind the Mahayana app-host binary/],
    [carrierPath, 'void process.parentPort; const env = "MAHAYANA_APP_HOST_BIN";', /must resolve the staged Mahayana app-host executable/],
  ];
  for (const [relative, contents, reason] of mutations) {
    const tree = fixture(t);
    stageRuntimeFixture(tree);
    write(path.join(tree.desktop, relative), contents);
    refused(run(tree), reason);
  }
});

test("onp_default_mode_retains_forbidden_product_code_and_resources", (t) => {
  for (const forbidden of ["FABUSHI_DANDELION_BIN", "dandelion-default.toml", "dandelion.native.apple.v1"]) {
    const tree = fixture(t);
    stageRuntimeFixture(tree);
    write(path.join(tree.desktop, "dist/electron-main/main.cjs"), `// ${forbidden}\n`);
    refused(run(tree), /Application bundle contains Mini App-owned code/);
  }
  for (const filename of ["dandelion", "dandelion-default.toml", "dandelion-build.json"]) {
    const tree = fixture(t);
    stageRuntimeFixture(tree);
    write(path.join(tree.desktop, "resources/bin", filename), "controlled forbidden fixture\n");
    refused(run(tree), /Application contains Mini App-owned resource/);
  }
});

test("onp_default_mode_cannot_accept_runtime_without_valid_notice_inputs", (t) => {
  const absent = fixture(t);
  stageRuntimeFixture(absent);
  fs.unlinkSync(absent.source);
  refused(run(absent), /OpenClaw source notice is missing/);
  const missingMapping = fixture(t);
  stageRuntimeFixture(missingMapping);
  changePackage(missingMapping, (config, index) => { config.build.extraResources.splice(index, 1); });
  refused(run(missingMapping), /requires one common extraResources mapping/);
  const wrongIdentity = fixture(t);
  stageRuntimeFixture(wrongIdentity);
  const lock = JSON.parse(fs.readFileSync(wrongIdentity.lock, "utf8"));
  lock.root_inputs.LICENSE = "0".repeat(40);
  writeJson(wrongIdentity.lock, lock);
  refused(run(wrongIdentity), /differs from root_inputs.LICENSE/);
});

test("onp_competing_resource_writers_are_refused_across_common_and_platform_sets", (t) => {
  const overlay = "resources/notice-overlay";
  const competitors = [
    ["direct-file", { from: `${overlay}/LICENSE`, to: "licenses/openclaw/LICENSE" }],
    ["directory-ancestor", { from: overlay, to: "licenses", filter: ["openclaw/LICENSE"] }],
    ["broad-ancestor", { from: overlay, to: "licenses", filter: ["**/*"] }],
    ["default-root", { from: overlay, filter: ["licenses/openclaw/LICENSE"] }],
    ["explicit-root", { from: overlay, to: ".", filter: ["**/*"] }],
    ["case-and-backslash", { from: `${overlay}/LICENSE`, to: "Licenses\\OpenClaw\\license" }],
    ["dot-alias", { from: `${overlay}/LICENSE`, to: "unused/../licenses/./openclaw/LICENSE" }],
    ["protected-descendant", { from: `${overlay}/LICENSE`, to: "licenses/openclaw/LICENSE/child" }],
    ["directory-writer", { from: overlay, to: "licenses/openclaw", filter: ["LICENSE"] }],
    ["literal-string", "licenses/openclaw/LICENSE"],
    ["glob-string", "licenses/**"],
    ["exclude-only-string", "!unrelated/**"],
    ["unknown-macro", { from: `${overlay}/LICENSE`, to: "${env.ONP_NOTICE_DESTINATION}" }],
    ["absolute-target", { from: `${overlay}/LICENSE`, to: "/licenses/openclaw/LICENSE" }],
    ["escaping-target", { from: `${overlay}/LICENSE`, to: "../licenses/openclaw/LICENSE" }],
    ["windows-trailing-dot", { from: `${overlay}/LICENSE`, to: "licenses/openclaw/LICENSE." }],
  ];
  for (const scope of ["common", "mac", "mas", "win", "linux"]) {
    for (const [caseIndex, [label, entry]] of competitors.entries()) {
      const tree = fixture(t);
      const conflictingBytes = Buffer.from(`ONP controlled competing bytes: ${scope}/${label}\n`);
      assert.notDeepEqual(conflictingBytes, sourceBytes);
      for (const relative of [
        `${overlay}/LICENSE`, `${overlay}/openclaw/LICENSE`,
        `${overlay}/licenses/openclaw/LICENSE`, "licenses/openclaw/LICENSE",
      ]) write(path.join(tree.desktop, relative), conflictingBytes);
      changePackage(tree, (config, index) => {
        if (scope === "common") config.build.extraResources.push(entry);
        else config.build[scope].extraResources = caseIndex % 2 === 0 ? [entry] : entry;
        assert.deepEqual(config.build.extraResources[index], {
          from: "resources/licenses/openclaw", to: "licenses/openclaw", filter: ["LICENSE"],
        }, "the correct common mapping must remain beside the competing writer");
      });
      const result = run(tree, ["--notice-only"]);
      assert.equal(result.status, 1, `${scope}/${label}: ${result.stdout}\n${result.stderr}`);
      refused(result, /OpenClaw notice extraResources mapping.*disjointness is not proven/);
    }
  }
});

test("onp_disjoint_resource_mappings_preserve_all_platforms", (t) => {
  const tree = fixture(t);
  const originalResources = JSON.parse(fs.readFileSync(tree.package, "utf8")).build.extraResources;
  const payload = "resources/notice-disjoint/extra.txt";
  write(path.join(tree.desktop, payload), "controlled disjoint resource\n");
  write(path.join(tree.desktop, "platform-extra/readme.txt"), "controlled literal-pattern resource\n");
  changePackage(tree, (config) => {
    config.build.extraResources.push({ from: payload, to: "licenses/other/extra.txt" });
    config.build.extraResources.push({ from: payload, to: "licenses/openclaw/README" });
    config.build.extraResources.push("platform-extra/readme.txt");
    config.build.mac.extraResources = [{ from: payload, to: "unused/../platform/mac/extra.txt" }, "platform-extra/readme.txt"];
    config.build.mas.extraResources = { from: payload, to: "platform/mas/extra.txt" };
    config.build.win.extraResources = { from: payload, to: "Platform\\Win\\extra.txt" };
    config.build.linux.extraResources = "platform-extra/readme.txt";
    assert.deepEqual(config.build.extraResources.slice(0, originalResources.length), originalResources);
    for (const existing of ["resources/bin", "resources/asr", "resources/computer-control"]) {
      assert.ok(config.build.extraResources.some((entry) => entry.from === existing));
    }
  });
  accepted(run(tree, ["--notice-only"]), "notice-only");
  stageRuntimeFixture(tree);
  accepted(run(tree), "runtime");
});

test("onp_git_checkout_preserves_locked_notice_bytes_with_autocrlf", (t) => {
  const actualAttributes = fs.readFileSync(path.join(repoRoot, attributesRelative), "utf8").replaceAll("\r\n", "\n");
  assert.equal(actualAttributes, "LICENSE -text\n", "use the actual single tracked rule");
  assert.equal(sourceBytes.includes(Buffer.from("\r")), false, "the fixed source is LF text for the sensitivity control");

  function checkout(tree, retainAttributes) {
    if (!retainAttributes) fs.unlinkSync(tree.attributes);
    const gitDir = path.join(tree.root, ".git");
    const controls = path.join(tree.root, "git-fixture-controls");
    const emptyConfig = path.join(controls, "empty-config");
    const template = path.join(controls, "empty-template");
    write(emptyConfig, "");
    fs.mkdirSync(template, { recursive: true });
    // Never inherit a real repository, index, object store, config injection or
    // attribute source. All Git writes below name this exclusively owned tree.
    const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !/^GIT_/i.test(key)));
    Object.assign(env, {
      GIT_CONFIG_NOSYSTEM: "1", GIT_ATTR_NOSYSTEM: "1", GIT_TERMINAL_PROMPT: "0",
      GIT_CONFIG_GLOBAL: emptyConfig, GIT_CONFIG_SYSTEM: emptyConfig,
    });
    const prefix = [
      "--no-pager", `--git-dir=${gitDir}`, `--work-tree=${tree.root}`,
      "-c", `core.attributesFile=${emptyConfig}`, "-c", `core.hooksPath=${template}`,
    ];
    function git(args) {
      const result = spawnSync("git", [...prefix, ...args], { cwd: tree.root, env, timeout: 10_000 });
      assert.ifError(result.error);
      assert.equal(result.signal, null);
      assert.equal(result.status, 0, `owned Git ${args[0]} failed: ${result.stderr.toString("utf8")}`);
      return result.stdout;
    }
    git(["init", `--template=${template}`, "--object-format=sha1", "--initial-branch=onp-notice-fixture", tree.root]);
    const actualGitDir = git(["rev-parse", "--absolute-git-dir"]).toString("utf8").trim();
    const actualWorkTree = git(["rev-parse", "--show-toplevel"]).toString("utf8").trim();
    assert.equal(fs.realpathSync(actualGitDir), fs.realpathSync(gitDir));
    assert.equal(fs.realpathSync(actualWorkTree), fs.realpathSync(tree.root));
    git(["config", "--local", "core.autocrlf", "true"]);
    git(["config", "--local", "core.safecrlf", "false"]);
    assert.equal(git(["config", "--type=bool", "--get", "core.autocrlf"]).toString("utf8").trim(), "true");
    const tracked = retainAttributes ? [attributesRelative, sourceRelative] : [sourceRelative];
    git(["add", "--", ...tracked]);
    git(["ls-files", "--error-unmatch", "--", ...tracked]);
    assert.deepEqual(git(["show", `:${sourceRelative}`]), sourceBytes, "the index retains the exact upstream blob in both controls");
    if (retainAttributes) {
      assert.equal(git(["show", `:${attributesRelative}`]).toString("utf8"), actualAttributes);
    } else {
      assert.equal(git(["ls-files", "--", attributesRelative]).length, 0);
    }
    for (const relative of tracked) fs.unlinkSync(path.join(tree.root, relative));
    git(["checkout-index", "--force", "--", ...tracked]);
    return fs.readFileSync(tree.source);
  }

  const protectedTree = fixture(t);
  assert.deepEqual(checkout(protectedTree, true), sourceBytes);
  accepted(run(protectedTree, ["--notice-only"]), "notice-only");
  const controlTree = fixture(t);
  const converted = checkout(controlTree, false);
  assert.notDeepEqual(converted, sourceBytes, "the independent no-attribute checkout must actually change bytes");
  assert.deepEqual(converted, Buffer.from(sourceBytes.toString("utf8").replaceAll("\n", "\r\n")));
  refused(run(controlTree, ["--notice-only"]), /OpenClaw source notice Git blob differs from root_inputs.LICENSE/);
});
