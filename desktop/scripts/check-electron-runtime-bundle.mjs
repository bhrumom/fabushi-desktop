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

console.log(
  "Desktop runtime artifact graph passed: Electron main, preload, Rust Coordinator carrier, and Renderer are co-staged.",
);
