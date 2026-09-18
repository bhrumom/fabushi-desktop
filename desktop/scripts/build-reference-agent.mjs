import path from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

const here = path.dirname(fileURLToPath(import.meta.url));
const desktop = path.resolve(here, "..");
const repo = path.resolve(desktop, "..");
const entry = path.join(repo, "reference", "grok-bot-0.18", "source", "host", "runner", "sand-agent-runner.ts");
const outfile = path.join(desktop, "electron", "grok-reference-sand-runner.cjs");

await build({
  entryPoints: [entry],
  outfile,
  bundle: true,
  platform: "node",
  format: "cjs",
  target: "node26",
  packages: "external",
  sourcemap: false,
  minify: false,
  treeShaking: true,
  logLevel: "info",
  banner: { js: "'use strict';" },
});

console.log(`Built pinned SandAgentRunner bundle: ${outfile}`);
