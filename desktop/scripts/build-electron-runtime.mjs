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

console.log("built Grok-shaped Electron main/preload and Rust Coordinator carrier");
