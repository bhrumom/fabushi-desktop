import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';

test('recursive child external scan references are checked for membership', () => {
  const source = fs.readFileSync(new URL('./validate-tdrp-revision9-authority.mjs', import.meta.url), 'utf8');
  assert.match(
    source,
    /if \(child\.external_scan_id\) fail\(externalScanIds\.has\(child\.external_scan_id\),['"]recursive child references unknown external scan:/,
  );
  assert.match(source, /const inventoryIndex=await readJson\('projects\/telegram-desktop-rust\/inventory\/index\.json'\)/);
  assert.match(source, /lock unknown coverage exceeds inventory baseline/);
  assert.match(source, /lock unread coverage exceeds inventory baseline/);
  assert.match(source, /lock\.coverage\.unknown_minimum<=inventoryIndex\.inventory\?\.unknown_minimum/);
  assert.match(source, /lock\.coverage\.unread_minimum<=inventoryIndex\.inventory\?\.unread_minimum/);
});
