import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(desktopRoot, "..");
const outMain = path.join(desktopRoot, "dist", "electron-main");
const outPreload = path.join(desktopRoot, "dist", "electron-preload");
const outCoordinator = path.join(desktopRoot, "dist", "node-agent-coordinator");

await Promise.all([
  fs.mkdir(outMain, { recursive: true }),
  fs.mkdir(outPreload, { recursive: true }),
  fs.mkdir(outCoordinator, { recursive: true }),
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
  external: ["electron"],
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

// Compile the actual SDK/Host-loader contract entries with the same module
// resolution and Node target as the shipping Electron runtime.
if (process.argv.includes("--plugin-skills-contracts")) {
  const outContracts = path.join(desktopRoot, ".tmp-grok-electron", "ocrm-plugin");
  await fs.mkdir(outContracts, { recursive: true });
  for (const [entry, filename] of [
    ["source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts", "host-plugin-skills-loader.contract.test.cjs"],
    ["source/packages/cursor-plugins/loader.contract.test.ts", "loader.contract.test.cjs"],
    ["source/electron-main/coordinator/coordinator-control-server.contract.test.ts", "coordinator-control-server.contract.test.cjs"],
  ]) {
    await build({
      ...common,
      entryPoints: [path.join(repoRoot, entry)],
      outfile: path.join(outContracts, filename),
    });
  }
}
