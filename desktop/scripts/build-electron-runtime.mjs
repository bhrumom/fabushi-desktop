import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";
import { createHash } from "node:crypto";
import { stageLocalExecDependencies } from "./build-local-exec-dependencies.mjs";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(desktopRoot, "..");
const outMain = path.join(desktopRoot, "dist", "electron-main");
const outPreload = path.join(desktopRoot, "dist", "electron-preload");
const outCoordinator = path.join(desktopRoot, "dist", "node-agent-coordinator");
const outLocalExec = path.join(desktopRoot, "dist", "local-exec-daemon");

await Promise.all([
  fs.mkdir(outMain, { recursive: true }),
  fs.mkdir(outPreload, { recursive: true }),
  fs.mkdir(outCoordinator, { recursive: true }),
  fs.mkdir(outLocalExec, { recursive: true }),
]);

const common = {
  bundle: true,
  platform: "node",
  target: "node24",
  format: "cjs",
  mainFields: ["module", "main"],
  sourcemap: true,
  logLevel: "info",
  absWorkingDir: desktopRoot,
  nodePaths: [path.join(desktopRoot, "node_modules")],
  // Keep the installed LGPL library separately loadable, with its complete
  // source and notices staged below. It is a required dependency, not a stub.
  external: ["electron", "jschardet"],
  banner: {
    js: 'const __fabushiImportMetaUrl = require("node:url").pathToFileURL(__filename).href;',
  },
  define: {
    "import.meta.url": "__fabushiImportMetaUrl",
  },
};

await build({
  ...common,
  entryPoints: [path.join(repoRoot, "source", "electron-main", "entry.ts")],
  outfile: path.join(outMain, "main.cjs"),
});

for (const [entry, outfile] of [
  ["runtime/primary.ts", "preload.cjs"],
  ["runtime/primary.ts", "preload-sand-dev.cjs"],
  ["runtime/dev-controls.ts", "preload-dev-controls.cjs"],
  ["runtime/vnc.ts", "preload-vnc.cjs"],
  ["runtime/webview.ts", "preload-webview.cjs"],
]) {
  await build({
    ...common,
    entryPoints: [path.join(repoRoot, "source", "electron-preload", entry)],
    outfile: path.join(outPreload, outfile),
  });
}

await build({
  ...common,
  entryPoints: [
    path.join(
      repoRoot,
      "source",
      "electron-main",
      "coordinator",
      "rust-coordinator-carrier.ts",
    ),
  ],
  outfile: path.join(outCoordinator, "main.cjs"),
});

console.log("built Fabushi product main/preload and Rust Coordinator carrier");

await build({
  ...common,
  entryPoints: [path.join(repoRoot, "source", "electron-main", "mcp", "miniapp-native-carrier.ts")],
  outfile: path.join(outMain, "miniapp-native-carrier.cjs"),
});

// Sandboxed Electron preloads cannot load the main runtime's node:url banner.
// This bundle imports only electron and exposes the bounded Mini App IPC bridge.
await build({
  ...common,
  banner: {},
  entryPoints: [path.join(repoRoot, "source", "electron-preload", "preload-miniapp-tools.ts")],
  outfile: path.join(outPreload, "preload-miniapp-tools.cjs"),
});


function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function regularFiles(root, prefix = "") {
  const result = [];
  for (const entry of (await fs.readdir(path.join(root, prefix), { withFileTypes: true }))
    .sort((left, right) => left.name < right.name ? -1 : left.name > right.name ? 1 : 0)) {
    const relative = path.posix.join(prefix, entry.name);
    const absolute = path.join(root, relative);
    const stat = await fs.lstat(absolute);
    if (stat.isSymbolicLink()) throw new Error("LocalExec package contains a symbolic link: " + relative);
    if (stat.isDirectory()) result.push(...await regularFiles(root, relative));
    else if (stat.isFile()) {
      const bytes = await fs.readFile(absolute);
      result.push({ path: relative, bytes: bytes.length, sha256: sha256(bytes) });
    } else throw new Error("LocalExec package contains a non-regular input: " + relative);
  }
  return result;
}

async function stageLocalExecLibrary() {
  const source = path.join(desktopRoot, "node_modules", "jschardet");
  if (!(await fs.lstat(source)).isDirectory() || await fs.realpath(source) !== source) {
    throw new Error("jschardet must be a real installed package directory");
  }
  const metadata = JSON.parse(await fs.readFile(path.join(source, "package.json"), "utf8"));
  const lockBytes = await fs.readFile(path.join(desktopRoot, "package-lock.json"));
  const lock = JSON.parse(lockBytes.toString("utf8"));
  const pinned = lock.packages?.["node_modules/jschardet"];
  if (metadata.name !== "jschardet" || metadata.version !== "3.1.4"
      || pinned?.version !== metadata.version || pinned.license !== "LGPL-2.1+"
      || lock.packages?.[""]?.dependencies?.jschardet !== metadata.version) {
    throw new Error("jschardet installed package or exact dependency identity drifted");
  }
  const files = await regularFiles(source);
  for (const required of ["package.json", "LICENSE", "CONTRIBUTORS", "index.js", "src/index.js"]) {
    if (!files.some(file => file.path === required && file.bytes > 0)) {
      throw new Error("jschardet source/notice artifact is missing: " + required);
    }
  }
  const parent = path.join(desktopRoot, "dist", "node_modules");
  await fs.mkdir(parent, { recursive: true });
  if (await fs.realpath(parent) !== parent) throw new Error("LocalExec library output parent is not owned");
  const temporary = await fs.mkdtemp(path.join(parent, ".jschardet-"));
  const candidate = path.join(temporary, "jschardet");
  const destination = path.join(parent, "jschardet");
  try {
    await fs.cp(source, candidate, { recursive: true, dereference: false, errorOnExist: true });
    const copied = await regularFiles(candidate);
    const sourceAfter = await regularFiles(source);
    if (JSON.stringify(files) !== JSON.stringify(copied)
        || JSON.stringify(files) !== JSON.stringify(sourceAfter)) {
      throw new Error("jschardet source changed or the complete staged copy differs");
    }
    await fs.rm(destination, { recursive: true, force: true });
    await fs.rename(candidate, destination);
  } finally {
    await fs.rm(temporary, { recursive: true, force: true });
  }
  return {
    name: metadata.name, version: metadata.version, license: pinned.license,
    resolved: pinned.resolved, integrity: pinned.integrity,
    package_lock_sha256: sha256(lockBytes),
    directory: "dist/node_modules/jschardet",
    source_and_notices_retained: true,
    files: files.map(file => ({ ...file, path: "dist/node_modules/jschardet/" + file.path })),
  };
}


async function stageLocalExecLegalInputs() {
  const source = path.join(desktopRoot, "resources", "licenses", "local-exec");
  if (!(await fs.lstat(source)).isDirectory() || await fs.realpath(source) !== source
      || await fs.realpath(outLocalExec) !== outLocalExec) {
    throw new Error("LocalExec legal/source roots must be real owned directories");
  }
  const before = await regularFiles(source);
  const manifest = JSON.parse(await fs.readFile(path.join(source, "source-inputs.json"), "utf8"));
  const lock = JSON.parse(await fs.readFile(path.join(desktopRoot, "package-lock.json"), "utf8"));
  const expectedNames = ["iconv-lite", "ignore", "jschardet", "re2js", "tree-sitter",
    "tree-sitter-bash", "node-addon-api", "node-gyp-build", "web-tree-sitter", "safer-buffer"].sort();
  if (manifest.contract !== "LDB-v1-legal-inputs" || !Array.isArray(manifest.packages)
      || JSON.stringify(manifest.packages.map(item => item.name).sort()) !== JSON.stringify(expectedNames)) {
    throw new Error("LocalExec legal/source input set is incomplete");
  }
  for (const item of manifest.packages) {
    const pinned = lock.packages?.["node_modules/" + item.name];
    if (pinned?.version !== item.version || pinned.resolved !== item.resolved
        || pinned.integrity !== item.integrity || pinned.license !== item.declared_license
        || !Array.isArray(item.inspected_source_artifacts) || item.inspected_source_artifacts.length === 0) {
      throw new Error("LocalExec legal/source input differs from lock: " + item.name);
    }
    for (const artifact of item.inspected_source_artifacts) {
      if (typeof artifact.file !== "string" || artifact.file.startsWith("/") || artifact.file.includes("\\")
          || artifact.file.split("/").some(part => part === "" || part === "." || part === "..")) {
        throw new Error("LocalExec legal/source path leaves its declared root");
      }
      const bytes = await fs.readFile(path.join(source, artifact.file));
      const blob = createHash("sha1").update("blob " + bytes.length + "\0").update(bytes).digest("hex");
      if (bytes.length !== artifact.bytes || sha256(bytes) !== artifact.sha256 || blob !== artifact.git_blob) {
        throw new Error("LocalExec legal/source artifact identity drifted: " + artifact.file);
      }
    }
  }
  const destination = path.join(outLocalExec, "licenses");
  await fs.rm(destination, { recursive: true, force: true });
  await fs.cp(source, destination, { recursive: true, dereference: false, errorOnExist: true });
  const copied = await regularFiles(destination);
  if (JSON.stringify(before) !== JSON.stringify(copied)
      || JSON.stringify(before) !== JSON.stringify(await regularFiles(source))) {
    throw new Error("LocalExec legal/source inputs changed or staged copy differs");
  }
  return {
    manifest: "dist/local-exec-daemon/licenses/source-inputs.json",
    files: copied.map(file => ({ ...file, path: "dist/local-exec-daemon/licenses/" + file.path })),
    acceptance: "Original source/notice bytes and package identities only; final packaged legal acceptance remains separate.",
  };
}

// The production Coordinator already selects this entry. Native parser stages
// are required even for file requests because the default executor loads its
// parser at module evaluation. Build each actual runtime domain separately.
const nativeStages = await stageLocalExecDependencies({
  desktopRoot, workRoot: process.env.FABUSHI_LOCAL_EXEC_BUILD_WORK_ROOT,
});
const externalLibrary = await stageLocalExecLibrary();
const legalInputs = await stageLocalExecLegalInputs();
const daemonBuild = await build({
  ...common,
  entryPoints: [path.join(repoRoot, "source", "local-exec-daemon", "main.ts")],
  outfile: path.join(outLocalExec, "main.cjs"),
  metafile: true,
});
const daemonMetafile = JSON.stringify(daemonBuild.metafile, null, 2) + "\n";
await fs.writeFile(path.join(outLocalExec, "build-metafile.json"), daemonMetafile);
const daemonBytes = await fs.readFile(path.join(outLocalExec, "main.cjs"));
await fs.writeFile(path.join(outLocalExec, "build-manifest.json"), JSON.stringify({
  contract: "LDB-v1",
  entry: "source/local-exec-daemon/main.ts",
  bundle: "dist/local-exec-daemon/main.cjs",
  bundle_sha256: sha256(daemonBytes),
  metafile: "dist/local-exec-daemon/build-metafile.json",
  metafile_sha256: sha256(Buffer.from(daemonMetafile)),
  node_dependencies: {
    manifest: "dist/node-deps/local-exec-dependencies.json",
    runtime: nativeStages.node.runtime,
  },
  electron_dependencies: {
    manifest: "dist/deps/local-exec-dependencies.json",
    runtime: nativeStages.electron.runtime,
  },
  external_library: externalLibrary,
  legal_inputs: legalInputs,
  scope: "Actual selected daemon and separate native runtime stages; legacy executor business migration remains open.",
}, null, 2) + "\n");
console.log("built selected LocalExec daemon with qualified Node/Electron parser stages");

// Compile the selected desktop boundary contract entries with the same module
// resolution and Node target as the shipping Electron runtime.
if (process.argv.includes("--plugin-skills-contracts")) {
  const outContracts = path.join(desktopRoot, ".tmp-grok-electron", "ocrm-plugin");
  await fs.mkdir(outContracts, { recursive: true });
  for (const [entry, filename] of [
    ["source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts", "host-plugin-skills-loader.contract.test.cjs"],
    ["source/packages/cursor-plugins/loader.contract.test.ts", "loader.contract.test.cjs"],
    ["source/electron-main/coordinator/coordinator-control-server.contract.test.ts", "coordinator-control-server.contract.test.cjs"],
    ["source/electron-main/coordinator/coordinator-executors.test.ts", "coordinator-executors.test.cjs"],
    ["source/electron-main/mcp/mcp-runtime.contract.test.ts", "mcp-runtime.contract.test.cjs"],
    ["source/electron-main/mcp/official-mcp.contract.test.ts", "official-mcp.contract.test.cjs"],
    ["source/shared/node/cursor-backend/backend-mcp-origin.contract.test.ts", "backend-mcp-origin.contract.test.cjs"],
    ["source/electron-main/main-edge-model.contract.test.ts", "main-edge-model.contract.test.cjs"],
    ["source/electron-main/coordinator/coordinator-account-runtime.contract.test.ts", "coordinator-account-runtime.contract.test.cjs"],
    ["source/electron-main/coordinator/coordinator-main-port-client.contract.test.ts", "coordinator-main-port-client.contract.test.cjs"],
    ["source/electron-main/coordinator/coordinator-runtime.test.ts", "coordinator-runtime.test.cjs"],
    ["source/electron-main/adapters/attachment-gateway.contract.test.ts", "attachment-gateway.contract.test.cjs"],
    ["source/local-exec-daemon/production-outcome.contract.test.ts", "production-outcome.contract.test.cjs"],
  ]) {
    await build({
      ...common,
      entryPoints: [path.join(repoRoot, entry)],
      outfile: path.join(outContracts, filename),
    });
  }
}

// This boundary needs the same-H Rust Coordinator. Keep it isolated from the
// renderer-only contract entries, which do not own a native test executable.
if (process.argv.includes("--local-exec-identity-contracts")) {
  const outContracts = path.join(desktopRoot, ".tmp-grok-electron", "ocrm-local-exec-identity");
  await fs.mkdir(outContracts, { recursive: true });
  const result = await build({
    ...common,
    entryPoints: [path.join(repoRoot, "source/electron-main/coordinator/local-exec-process-identity.contract.test.ts")],
    outfile: path.join(outContracts, "local-exec-process-identity.contract.test.cjs"),
    metafile: true,
  });
  await fs.writeFile(path.join(outContracts, "build-metafile.json"), JSON.stringify(result.metafile, null, 2) + "\n");

  const startup = await build({
    ...common,
    entryPoints: [path.join(repoRoot, "source/local-exec-daemon/startup.contract.test.ts")],
    outfile: path.join(outContracts, "startup.contract.test.cjs"),
    metafile: true,
  });
  await fs.writeFile(path.join(outContracts, "startup-build-metafile.json"), JSON.stringify(startup.metafile, null, 2) + "\n");
}
