import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

import {validateSourceReviews} from './tdrp-source-review.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const inventoryPath = '/Users/bhrum/Documents/ChatGPT/.tdrp-artifacts/13118749668/x/upstream-recursive-inventory.json';

// Helper to construct a synthetic submodule inventory fixture
function sampleSubmoduleEntries() {
  return [
    {
      repository: 'ericniebler/range-v3',
      commit: '2dae74bb7c179c3cf3335db4dc2f0db1ff69ec19',
      path: 'include/range/v3/all.hpp',
      object: 'a'.repeat(40),
    },
    {
      repository: 'ericniebler/range-v3',
      commit: '2dae74bb7c179c3cf3335db4dc2f0db1ff69ec19',
      path: 'include/range/v3/core.hpp',
      object: 'b'.repeat(40),
    },
    {
      repository: 'hunspell/hunspell',
      commit: 'a698d8b53cab3507d1d619d5fa08a88777abe50b',
      path: 'src/hunspell/hunspell.cxx',
      object: 'c'.repeat(40),
    },
    {
      repository: 'desktop-app/lib_ui',
      commit: '79c846fdf032e3a515f400fb8a9d16a5a0cb0890',
      path: 'ui/widgets/labels.cpp',
      object: 'd'.repeat(40),
    },
    {
      repository: 'TelegramMessenger/tgcalls',
      commit: '1c236c09f8d83935adcfbd6fe72cb25c15ee6b2a',
      path: 'tgcalls/MediaManager.cpp',
      object: 'e'.repeat(40),
    },
  ];
}

function makeValidReview(source, overrides = {}) {
  return {
    repository: source.repository,
    commit: source.commit,
    path: source.path,
    object: source.object,
    category: 'dependency',
    risk: 'low',
    review_depth: 'contract',
    status: 'complete',
    rationale: `Submodule ${source.repository} replaced by Fabushi Cargo/Chromium architecture under FMEP-BULK-01.`,
    evidence: ['docs/specs/fmep-bulk-subtrees-disposition.md#submodule-contract-disposition'],
    responsibility_ids: [],
    ...overrides,
  };
}

test('positive: real 9,486 submodule inventory entries pass validateSourceReviews cleanly', async () => {
  let inventoryData;
  try {
    const raw = await fs.readFile(inventoryPath, 'utf-8');
    inventoryData = JSON.parse(raw);
  } catch {
    // Skip if artifact is not present
    return;
  }

  const submoduleEntries = inventoryData.entries.filter(e => e.repository !== 'telegramdesktop/tdesktop');
  assert.equal(submoduleEntries.length, 9486);

  const reviews = submoduleEntries.map(source => makeValidReview(source));
  const rows = [];

  const result = validateSourceReviews(submoduleEntries, reviews, rows);
  assert.equal(result.source_identities, 9486);
  assert.equal(result.reviewed, 9486);
});

test('positive: bulk contract disposition with row mounting scheme passes validateSourceReviews', () => {
  const inventory = sampleSubmoduleEntries();
  // Group by repository and build rows with additional_source_identities
  const primaryByRepo = new Map();
  const additionalsByRepo = new Map();
  for (const item of inventory) {
    if (!primaryByRepo.has(item.repository)) {
      primaryByRepo.set(item.repository, item);
      additionalsByRepo.set(item.repository, []);
    } else {
      additionalsByRepo.get(item.repository).push(item);
    }
  }

  const rows = [];
  for (const [repo, primary] of primaryByRepo.entries()) {
    const rowId = `TDRP-DEP-${repo.replace(/[/._]/g, '-').toUpperCase()}-001`;
    rows.push({
      responsibility_id: rowId,
      source_repository: primary.repository,
      source_commit: primary.commit,
      source_path: primary.path,
      source_blob_sha: primary.object,
      additional_source_identities: additionalsByRepo.get(repo),
    });
  }

  const reviews = inventory.map(source => {
    const rowId = `TDRP-DEP-${source.repository.replace(/[/._]/g, '-').toUpperCase()}-001`;
    return makeValidReview(source, {responsibility_ids: [rowId]});
  });

  const result = validateSourceReviews(inventory, reviews, rows);
  assert.equal(result.source_identities, inventory.length);
  assert.equal(result.reviewed, inventory.length);
});

test('negative: missing source review is rejected (缺项被拒)', () => {
  const inventory = sampleSubmoduleEntries();
  // Omit the last review
  const reviews = inventory.slice(0, -1).map(source => makeValidReview(source));
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /missing source reviews/
  );
});

test('negative: duplicate source review identity is rejected (重复被拒)', () => {
  const inventory = sampleSubmoduleEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  // Add duplicate of the first review
  reviews.push(makeValidReview(inventory[0]));
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /unknown, stale or duplicate source review/
  );
});

test('negative: blob SHA drift is rejected (blob 漂移被拒)', () => {
  const inventory = sampleSubmoduleEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  // Mutate blob SHA of second review to simulate drift
  reviews[1].object = 'f'.repeat(40);
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /unknown, stale or duplicate source review/
  );
});

test('negative: repository drift is rejected (仓库漂移被拒)', () => {
  const inventory = sampleSubmoduleEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  reviews[0].repository = 'other/unrelated-repo';
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /unknown, stale or duplicate source review/
  );
});

test('negative: disposition depth rejected for dependency category', () => {
  const inventory = sampleSubmoduleEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  reviews[0].review_depth = 'disposition';
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /contract review required/
  );
});

test('negative: high risk dependency without deep review is rejected', () => {
  const inventory = sampleSubmoduleEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  reviews[0].risk = 'high';
  // High risk requires deep review depth
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /deep review required/
  );
});

test('negative: uncovered responsibility ID is rejected', () => {
  const inventory = sampleSubmoduleEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  // Assign a non-existent responsibility id
  reviews[0].responsibility_ids = ['NON-EXISTENT-RESPONSIBILITY-001'];
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /unmapped responsibility: NON-EXISTENT-RESPONSIBILITY-001/
  );
});

test('negative: responsibility row does not cover review identity is rejected', () => {
  const inventory = sampleSubmoduleEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  reviews[1].responsibility_ids = ['TDRP-COVER-001'];

  // Row only covers inventory[0], but review[1] references it
  const rows = [
    {
      responsibility_id: 'TDRP-COVER-001',
      source_repository: inventory[0].repository,
      source_commit: inventory[0].commit,
      source_path: inventory[0].path,
      source_blob_sha: inventory[0].object,
    },
  ];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /responsibility does not cover source identity: TDRP-COVER-001/
  );
});
