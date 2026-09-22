#!/usr/bin/env node
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const referenceRoot = path.resolve(process.argv[2] ?? "");
if (!referenceRoot || !fs.existsSync(referenceRoot)) {
  throw new Error("usage: node scripts/import-grok-reference.mjs <checked-out-grok-reference-root>");
}

const manifestPath = path.join(root, "projects/grok-fabu-parity/architecture-manifest.json");
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const frozenSha = "a9f633e09d49a85829b8236331b9e21f7e612634";
if (manifest.frozenReference?.commit !== frozenSha) {
  throw new Error(`refusing to import unexpected Grok SHA ${manifest.frozenReference?.commit}`);
}

function gitBlobSha(bytes) {
  return crypto
    .createHash("sha1")
    .update(Buffer.from(`blob ${bytes.length}\0`))
    .update(bytes)
    .digest("hex");
}

const finalStatuses = new Set(["implemented", "not-applicable-noncode", "removed-extra"]);
const copyLanguages = new Set(["typescript", "react-typescript"]);
const copied = [];
for (const row of manifest.modules) {
  if (finalStatuses.has(row.status)) continue;
  if (!copyLanguages.has(row.targetLanguage)) continue;

  const source = path.resolve(referenceRoot, row.referencePath);
  const target = path.resolve(root, row.targetPath);
  if (!source.startsWith(referenceRoot + path.sep) || !target.startsWith(root + path.sep)) {
    throw new Error(`unsafe path in manifest: ${row.referencePath} -> ${row.targetPath}`);
  }
  if (!fs.existsSync(source)) {
    throw new Error(`reference source missing: ${row.referencePath}`);
  }

  const bytes = fs.readFileSync(source);
  const actualReferenceSha = gitBlobSha(bytes);
  if (actualReferenceSha !== row.referenceBlobSha) {
    throw new Error(
      `frozen reference mismatch for ${row.referencePath}: expected ${row.referenceBlobSha}, found ${actualReferenceSha}`,
    );
  }

  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, bytes);
  const targetSha = gitBlobSha(fs.readFileSync(target));
  if (targetSha !== row.referenceBlobSha) {
    throw new Error(`copied target mismatch for ${row.targetPath}`);
  }

  row.status = "implemented";
  row.parityMode = "reference-copy";
  row.behavioralEvidence = [row.targetPath];
  row.testEvidence = [
    "scripts/check-grok-architecture.mjs",
    ".github/workflows/grok-architecture-migrate.yml",
  ];
  row.notes =
    "Byte-identical frozen Grok 0.18 source restored at the mapped production path; Git blob SHA is enforced by the architecture gate.";
  copied.push(row.referencePath);
}

for (const relative of ["frontend/tsconfig.json", "source/tsconfig.json"]) {
  const source = path.join(referenceRoot, relative);
  if (!fs.existsSync(source)) continue;
  const target = path.join(root, relative);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.copyFileSync(source, target);
}

fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + "\n");
const counts = Object.create(null);
for (const row of manifest.modules) counts[row.status] = (counts[row.status] ?? 0) + 1;
console.log(JSON.stringify({ copied: copied.length, counts }, null, 2));
