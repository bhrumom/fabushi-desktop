import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

const candidate = JSON.parse(fs.readFileSync(new URL('../projects/telegram-desktop-rust/upstream.candidate.json', import.meta.url), 'utf8'));

test('current TDRP upstream candidate remains explicitly unaccepted until recursive closure', () => {
  assert.equal(candidate.project_id, 'TDRP-001');
  assert.equal(candidate.spec_revision, 9);
  assert.match(candidate.commit, /^[0-9a-f]{40}$/);
  assert.match(candidate.tree, /^[0-9a-f]{40}$/);
  assert.equal(candidate.accepted, false);
  assert.equal(candidate.baseline_ready, false);
  assert.equal(candidate.semantic_read_complete, false);
  assert.equal(candidate.responsibility_decomposition_complete, false);
  assert.equal(candidate.recursive_non_directory_entries, null);
  assert.equal(candidate.recursive_closure_status, 'pending');
  assert.equal(candidate.delta_from_locked_baseline.changed_total, 590);
});
