#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const manifestPath = path.join(root, 'projects/grok-fabu-parity/architecture-manifest.json');
const strict = process.argv.includes('--strict') || process.env.GROK_ARCH_STRICT === '1';

function fail(message) {
  console.error(`[grok-architecture] ERROR: ${message}`);
  process.exitCode = 1;
}

const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
if (manifest.schemaVersion !== 1) fail(`unsupported schemaVersion ${manifest.schemaVersion}`);
if (manifest.frozenReference?.commit !== 'a9f633e09d49a85829b8236331b9e21f7e612634') {
  fail('frozen Grok reference SHA changed');
}
if (!Array.isArray(manifest.modules) || manifest.modules.length !== manifest.moduleCount) {
  fail('moduleCount does not match modules length');
}

const refs = new Set();
const targets = new Map();
const allowedDevelopment = new Set(['planned', 'existing-needs-parity', 'implemented', 'not-applicable']);
const finalAllowed = new Set(['implemented', 'not-applicable']);
const requiredDomains = new Map([
  ['frontend/', 0],
  ['source/electron-main/', 0],
  ['source/electron-preload/', 0],
  ['source/node-agent-coordinator/', 0],
  ['source/host/', 0],
  ['source/shared/', 0],
  ['source/packages/', 0],
]);

for (const row of manifest.modules ?? []) {
  if (!row.referencePath || !row.targetPath || !row.targetLanguage || !row.owner || !row.status) {
    fail(`incomplete manifest row: ${JSON.stringify(row)}`);
    continue;
  }
  if (refs.has(row.referencePath)) fail(`duplicate referencePath ${row.referencePath}`);
  refs.add(row.referencePath);
  if (!allowedDevelopment.has(row.status)) fail(`invalid status ${row.status} for ${row.referencePath}`);
  if (strict && !finalAllowed.has(row.status)) fail(`final gate: ${row.referencePath} is ${row.status}`);
  if (row.status === 'not-applicable' && (!Array.isArray(row.evidence) || row.evidence.length === 0)) {
    fail(`not-applicable row requires evidence: ${row.referencePath}`);
  }
  if (row.status === 'implemented') {
    const target = path.resolve(root, row.targetPath);
    if (!target.startsWith(root + path.sep) || !fs.existsSync(target)) {
      fail(`implemented target missing: ${row.targetPath} for ${row.referencePath}`);
    }
  }
  const prior = targets.get(row.targetPath);
  if (prior && prior !== row.referencePath) {
    fail(`targetPath collision: ${row.targetPath} maps both ${prior} and ${row.referencePath}`);
  } else {
    targets.set(row.targetPath, row.referencePath);
  }
  for (const domain of requiredDomains.keys()) {
    if (row.referencePath.startsWith(domain)) requiredDomains.set(domain, requiredDomains.get(domain) + 1);
  }
}
for (const [domain, count] of requiredDomains) {
  if (count === 0) fail(`frozen inventory has no entries for required domain ${domain}`);
}

const expectedCounts = {
  'frontend/': 280,
  'source/electron-main/': 184,
  'source/electron-preload/': 16,
  'source/node-agent-coordinator/': 24,
  'source/host/': 471,
  'source/shared/': 165,
  'source/packages/': 852,
};
for (const [domain, expected] of Object.entries(expectedCounts)) {
  const actual = requiredDomains.get(domain);
  if (actual !== expected) fail(`${domain} expected ${expected} frozen modules, found ${actual}`);
}

if (strict) {
  const forbiddenParallelRoots = [
    'desktop/src',
    'desktop/electron',
    'frontend/apps/web',
    'third_party/mahayana',
  ];
  for (const legacyRoot of forbiddenParallelRoots) {
    if (fs.existsSync(path.join(root, legacyRoot))) {
      fail(`final gate: legacy parallel runtime root still exists: ${legacyRoot}`);
    }
  }
}

if (process.exitCode) process.exit(process.exitCode);
console.log(`[grok-architecture] PASS modules=${manifest.moduleCount} strict=${strict}`);
