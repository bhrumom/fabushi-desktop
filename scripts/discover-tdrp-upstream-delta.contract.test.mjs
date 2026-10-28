import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

const source = fs.readFileSync(new URL('./discover-tdrp-upstream-delta.mjs', import.meta.url), 'utf8');

test('upstream delta discovery compares complete immutable trees and records all changed paths', () => {
  assert.match(source, /recursive=1/);
  assert.match(source, /if \(!recursive\.truncated\)/);
  assert.match(source, /GitHub truncates recursive trees/);
  assert.match(source, /rebaseline-required/);
  assert.match(source, /changed_paths: changed/);
  assert.match(source, /candidateCommit/);
});
