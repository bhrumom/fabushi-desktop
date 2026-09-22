#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifest = JSON.parse(
  fs.readFileSync(path.join(root, "projects/grok-fabu-parity/architecture-manifest.json"), "utf8"),
);
const finalStatuses = new Set(["implemented", "not-applicable-noncode", "removed-extra"]);
const remaining = manifest.modules.filter((row) => !finalStatuses.has(row.status));

function domainOf(referencePath) {
  for (const prefix of [
    "frontend/src/recovered/features/",
    "source/host/extensions/",
    "source/host/runner/",
    "source/packages/",
    "source/electron-main/",
    "source/shared/",
    "source/electron-preload/",
    "source/node-agent-coordinator/",
    "source/host/",
  ]) {
    if (referencePath.startsWith(prefix)) return prefix.replace(/\/$/, "");
  }
  return referencePath.split("/").slice(0, 2).join("/");
}

const byDomain = {};
const byStatus = {};
const byLanguage = {};
for (const row of remaining) {
  const domain = domainOf(row.referencePath);
  byDomain[domain] ??= {};
  byDomain[domain][row.status] = (byDomain[domain][row.status] ?? 0) + 1;
  byStatus[row.status] = (byStatus[row.status] ?? 0) + 1;
  byLanguage[row.targetLanguage] = (byLanguage[row.targetLanguage] ?? 0) + 1;
}
const report = {
  frozenReference: manifest.frozenReference,
  moduleCount: manifest.modules.length,
  remainingCount: remaining.length,
  byStatus,
  byLanguage,
  byDomain,
  existingNeedsParity: remaining
    .filter((row) => row.status === "existing-needs-parity")
    .map(({ referencePath, targetPath, targetLanguage, owner, processOrPackage, notes }) => ({
      referencePath,
      targetPath,
      targetLanguage,
      owner,
      processOrPackage,
      notes: notes ?? null,
    })),
};
console.log(JSON.stringify(report, null, 2));
