import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, "..");

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

console.log(
  "Desktop runtime artifact graph passed: Electron main, preload, Rust Coordinator carrier + app-host binary wiring, and Renderer are co-staged.",
);

const productMain = fs.readFileSync(path.join(desktopRoot, "dist/electron-main/main.cjs"), "utf8");
for (const forbidden of ["FABUSHI_DANDELION_BIN", "dandelion-default.toml", "dandelion.native.apple.v1"]) {
  if (productMain.includes(forbidden)) throw new Error(`Application bundle contains Mini App-owned code: ${forbidden}`);
}
for (const filename of ["dandelion", "dandelion-default.toml", "dandelion-build.json"]) {
  if (fs.existsSync(path.join(desktopRoot, "resources/bin", filename))) throw new Error(`Application contains Mini App-owned resource ${filename}`);
}
