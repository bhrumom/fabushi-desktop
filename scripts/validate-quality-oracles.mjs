#!/usr/bin/env node
/**
 * Fail-closed traceability gate for the canonical Conversation oracles.
 *
 * Normative oracle definitions live in
 *   projects/fabushi-communication-platform/quality/oracles/*.md
 * Executable cases live in
 *   source/host/tests/ora_turn_order_recon_contract.rs
 *
 * Each case declares its coverage with machine-readable annotations:
 *   #[test]
 *   // oracle: TURN-I01
 *   // regression: REG-TURN-FINAL-001
 *   fn ora_...() { ... }
 *
 * An invariant may be `covered` (has an executable case) or `pending` (declared
 * with a reason in oracle-execution-status.json). Anything else fails closed, so
 * an invariant can never silently disappear from the gate.
 *
 * Usage: node scripts/validate-quality-oracles.mjs [--report <path>]
 */

import fs from 'node:fs';
import path from 'node:path';

const ORACLE_DIR = 'projects/fabushi-communication-platform/quality/oracles';
const DOC_SOURCES = {
  'ORA-TURN-001': `${ORACLE_DIR}/conversation-turn-oracle.md`,
  'ORA-ORDER-001': `${ORACLE_DIR}/transcript-ordering-oracle.md`,
  'ORA-RECON-001': `${ORACLE_DIR}/reconciliation-oracle.md`,
};
const TEST_FILE = 'source/host/tests/ora_turn_order_recon_contract.rs';
const STATUS_FILE = `${ORACLE_DIR}/oracle-execution-status.json`;

const REQUIRED_REGRESSIONS = [
  'REG-TURN-FINAL-001',
  'REG-TURN-INTERMEDIATE-001',
  'REG-TRANSCRIPT-ORDER-001',
  'REG-TRANSCRIPT-REPLACE-001',
  'REG-TRANSCRIPT-DUP-001',
  'REG-CROSS-CONVERSATION-001',
];

const INVARIANT_PATTERN = /\b(?:TURN|ORD|REC)-I\d{2}\b/g;

const errors = [];
const fail = (message) => errors.push(message);

function readRequired(relativePath) {
  const absolute = path.resolve(process.cwd(), relativePath);
  if (!fs.existsSync(absolute)) {
    fail(`missing required file: ${relativePath}`);
    return null;
  }
  return fs.readFileSync(absolute, 'utf8');
}

// ---------------------------------------------------------------- normative set

const normative = new Map();
for (const [oracleId, relativePath] of Object.entries(DOC_SOURCES)) {
  const source = readRequired(relativePath);
  if (source == null) continue;
  const ids = new Set(source.match(INVARIANT_PATTERN) ?? []);
  if (ids.size === 0) {
    fail(`${relativePath} declares no invariant IDs; the oracle is unreadable`);
    continue;
  }
  for (const id of ids) {
    const owner = normative.get(id);
    if (owner != null && owner !== oracleId) {
      fail(`${id} is declared by both ${owner} and ${oracleId}`);
    }
    normative.set(id, oracleId);
  }
}

// ---------------------------------------------------------------- executable set

const testSource = readRequired(TEST_FILE);
const cases = [];

if (testSource != null) {
  const blocks = testSource.split('#[test]').slice(1);
  for (const block of blocks) {
    const lines = block.split(/\r?\n/);
    const oracles = [];
    const regressions = [];
    let name = null;
    for (const line of lines) {
      const oracle = line.match(/^\s*\/\/\s*oracle:\s*([A-Za-z0-9-]+)\s*$/);
      if (oracle != null) {
        oracles.push(oracle[1]);
        continue;
      }
      const regression = line.match(/^\s*\/\/\s*regression:\s*([A-Za-z0-9-]+)\s*$/);
      if (regression != null) {
        regressions.push(regression[1]);
        continue;
      }
      const fn = line.match(/^\s*(?:pub\s+)?fn\s+([A-Za-z0-9_]+)\s*\(/);
      if (fn != null) {
        name = fn[1];
        break;
      }
    }
    if (name == null) {
      fail(`${TEST_FILE} contains a #[test] block without a resolvable fn name`);
      continue;
    }
    cases.push({ name, oracles, regressions });
  }
  if (cases.length === 0) {
    fail(`${TEST_FILE} declares no executable oracle case`);
  }
}

const covered = new Map();
for (const testCase of cases) {
  if (testCase.oracles.length === 0) {
    fail(`test ${testCase.name} carries no // oracle: annotation`);
  }
  for (const id of testCase.oracles) {
    if (!normative.has(id)) {
      fail(`test ${testCase.name} claims unknown invariant ${id}`);
      continue;
    }
    const owners = covered.get(id) ?? [];
    owners.push(testCase.name);
    covered.set(id, owners);
  }
}

const seenNames = new Set();
for (const testCase of cases) {
  if (seenNames.has(testCase.name)) {
    fail(`duplicate test name ${testCase.name}`);
  }
  seenNames.add(testCase.name);
}

// ---------------------------------------------------------------- pending set

let status = { pending: {} };
const statusSource = readRequired(STATUS_FILE);
if (statusSource != null) {
  try {
    status = JSON.parse(statusSource);
  } catch (error) {
    fail(`${STATUS_FILE} is not valid JSON: ${error.message}`);
  }
}

const pending = new Map();
const pendingRecord = status.pending ?? {};
if (pendingRecord == null || typeof pendingRecord !== 'object' || Array.isArray(pendingRecord)) {
  fail(`${STATUS_FILE} must expose a "pending" object of invariantId -> reason`);
} else {
  for (const [id, reason] of Object.entries(pendingRecord)) {
    if (!normative.has(id)) {
      fail(`${STATUS_FILE} declares pending invariant ${id} which no oracle defines`);
      continue;
    }
    if (typeof reason !== 'string' || reason.trim().length === 0) {
      fail(`${STATUS_FILE} pending entry ${id} needs a non-empty reason`);
      continue;
    }
    pending.set(id, reason.trim());
  }
}

// ---------------------------------------------------------------- closure checks

for (const id of covered.keys()) {
  if (pending.has(id)) {
    fail(`${id} is both covered and pending; a covered invariant must not stay pending`);
  }
}

const uncovered = [...normative.keys()].filter((id) => !covered.has(id) && !pending.has(id));
for (const id of uncovered) {
  fail(`${id} (${normative.get(id)}) has no executable case and no pending disposition`);
}

const declaredRegressions = new Set(cases.flatMap((testCase) => testCase.regressions));
for (const regression of REQUIRED_REGRESSIONS) {
  if (!declaredRegressions.has(regression)) {
    fail(`${regression} has no lower-layer oracle case in ${TEST_FILE}`);
  }
}

for (const regression of declaredRegressions) {
  if (!REQUIRED_REGRESSIONS.includes(regression)) {
    fail(`unknown regression id ${regression} referenced by ${TEST_FILE}`);
  }
}

// ---------------------------------------------------------------- report

const report = {
  format_version: 1,
  head_sha: process.env.GITHUB_SHA ?? null,
  test_file: TEST_FILE,
  oracles: Object.keys(DOC_SOURCES),
  normative_invariants: normative.size,
  executable_cases: cases.length,
  covered: Object.fromEntries([...covered.entries()].sort(([a], [b]) => a.localeCompare(b))),
  pending: Object.fromEntries([...pending.entries()].sort(([a], [b]) => a.localeCompare(b))),
  regressions: [...declaredRegressions].sort(),
  uncovered: uncovered.sort(),
  status: errors.length === 0 ? 'pass' : 'fail',
  errors,
};

const reportFlagIndex = process.argv.indexOf('--report');
if (reportFlagIndex !== -1) {
  const reportPath = process.argv[reportFlagIndex + 1];
  if (!reportPath) {
    fail('--report requires a path');
  } else {
    fs.mkdirSync(path.dirname(path.resolve(process.cwd(), reportPath)), { recursive: true });
    fs.writeFileSync(
      path.resolve(process.cwd(), reportPath),
      `${JSON.stringify(report, null, 2)}\n`,
    );
  }
}

console.log(JSON.stringify(report, null, 2));

if (errors.length > 0) {
  console.error(`\nquality oracle traceability FAILED with ${errors.length} error(s):`);
  for (const error of errors) console.error(` - ${error}`);
  process.exit(1);
}

console.log(
  `\nquality oracle traceability passed: ${covered.size} covered, ${pending.size} pending, ` +
    `${cases.length} executable case(s), ${declaredRegressions.size} regression(s).`,
);
