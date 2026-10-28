import fs from 'node:fs';
import test from 'node:test';
import assert from 'node:assert/strict';

const source = fs.readFileSync(new URL('./discover-tdrp-candidate-root-inventory.mjs', import.meta.url), 'utf8');

test('candidate inventory binds an immutable checkout and records every root blob', () => {
  assert.match(source, /candidate\.commit/);
  assert.match(source, /git.*ls-tree.*-r.*-z/s);
  assert.ok(source.includes('[\\s\\S]+'), 'candidate path parser must accept newline-containing paths');
  assert.match(source, /root_non_directory_entries/);
  assert.match(source, /files/);
  assert.match(source, /recursive_non_directory_entries: null/);
  assert.match(source, /source_closure_ready: false/);
  assert.match(source, /accepted: false/);
});
