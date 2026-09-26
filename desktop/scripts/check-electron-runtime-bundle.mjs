import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, "..");

const required = [
  ["Electron main", "dist/electron-main/main.cjs"],
  ["primary preload", "dist/electron-preload/preload.cjs"],
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

console.log(
  "Desktop runtime artifact graph passed: Electron main, preload, Rust Coordinator carrier, and Renderer are co-staged.",
);
