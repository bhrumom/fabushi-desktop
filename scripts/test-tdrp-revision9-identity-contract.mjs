#!/usr/bin/env node
// Revision-9 identity separation and non-acceptance contract; runs on GitHub Actions only.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
const root = 'projects/telegram-desktop-rust/';
const read = p => JSON.parse(readFileSync(root + p, 'utf8'));
const lock = read('upstream.lock.json');
const ledger = read('parity-ledger.json');
const acquisitions = read('inventory/build-time-acquisitions.json');
const inventory = read('inventory/index.json');
const validator = readFileSync('scripts/validate-tdrp-revision9-authority.mjs', 'utf8');
const history = lock.previous_historical_baseline.commit;
const candidate = lock.upstream.commit;
assert.match(history, /^[a-f0-9]{40}$/);
assert.match(candidate, /^[a-f0-9]{40}$/);
assert.notEqual(history, candidate, 'discovery must not alias history');
assert.equal(ledger.upstream_commit, history);
assert.ok(ledger.rows.length > 0);
for (const row of ledger.rows) assert.equal(row.source_commit, history);
assert.equal(acquisitions.upstream_commit, history);
assert.equal(acquisitions.upstream_repository, lock.upstream.repository);
assert.equal(inventory.upstream.commit, candidate);
assert.equal(inventory.upstream.baseline_identity_accepted, false);
assert.equal(lock.upstream.accepted, false);
assert.equal(lock.acceptance.accepted, false);
assert.equal(lock.acceptance.baseline_ready, false);
assert.equal(lock.coverage.recursive_source_entries_total, null);
assert.equal(lock.coverage.unknown_minimum, null);
assert.equal(lock.coverage.unread_minimum, null);
// Ensure the runtime accepted-gate checks genuinely use candidate-owned evidence.
const gates = [
  'candidateLedger=await readJson(candidateLedgerPath)',
  'candidateLedger.upstream_commit===lock.upstream.commit',
  'candidateLedger.rows.every(row=>row.source_commit===lock.upstream.commit)',
  'candidateLedger.coverage?.source_entries_total===lock.coverage.recursive_source_entries_total',
  'candidateAcquisitionInventory=await readJson(candidatePath)',
  'candidateAcquisitionInventory.upstream_commit===lock.upstream.commit',
  'candidateAcquisitionInventory.recursive_build_reachability?.accepted_upstream_commit===lock.upstream.commit',
  'acquisitionInventory.upstream_commit===historicalCommit',
  'ledger.upstream_commit===historicalLedgerCommit',
  'lock.upstream.accepted===false && lock.acceptance?.baseline_ready===false && lock.acceptance?.accepted===false'
];
for (const gate of gates) assert.ok(validator.includes(gate), 'validator missing mandatory fail-closed oracle: ' + gate);
console.log('Revision-9 historical/candidate identity and fail-closed gate wiring: PASS');
