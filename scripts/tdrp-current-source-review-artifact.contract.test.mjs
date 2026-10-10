import test from 'node:test';
import assert from 'node:assert/strict';
import {
  canonicalSourcePath,
  gitBlobSha,
  safeEvidenceName,
  selectCurrentFirstUnread,
} from './tdrp-current-source-review-artifact.mjs';

test('gitBlobSha matches the canonical Git object identity for hello newline', () => {
  assert.equal(
    gitBlobSha(Buffer.from('hello\n', 'utf8')),
    'ce013625030ba8dba906f756967f9e9ca394464a',
  );
});

test('canonical source paths distinguish root and mounted component entries', () => {
  assert.equal(canonicalSourcePath({ scope: 'root', path: 'Telegram/a.txt' }), 'Telegram/a.txt');
  assert.equal(
    canonicalSourcePath({ scope: 'direct-gitlink', mount: 'Telegram/ThirdParty/hime', path: 'data/a.pdf' }),
    'Telegram/ThirdParty/hime::data/a.pdf',
  );
});

test('selectCurrentFirstUnread binds the persisted cursor to the exact recursive entry', () => {
  const index = { rebaseline: { first_unread_order: 2, first_unread_path: 'vendor/a::data/a.pdf' } };
  const recursiveInventory = { entries: [
    { scope: 'root', path: 'vendor/a' },
    { scope: 'direct-gitlink', mount: 'vendor/a', path: 'data/a.pdf', repository: 'o/r', object: '1'.repeat(40) },
  ] };
  const selected = selectCurrentFirstUnread(index, recursiveInventory);
  assert.equal(selected.order, 2);
  assert.equal(selected.entry.object, '1'.repeat(40));
  assert.throws(
    () => selectCurrentFirstUnread(
      { rebaseline: { first_unread_order: 2, first_unread_path: 'vendor/a::wrong.pdf' } },
      recursiveInventory,
    ),
    /first unread path drift/,
  );
});

test('safeEvidenceName cannot escape the authority artifact directory', () => {
  assert.equal(safeEvidenceName(7660, '../../array license.pdf'), '07660-array_license.pdf');
});
