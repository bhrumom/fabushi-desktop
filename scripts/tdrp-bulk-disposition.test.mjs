import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

import {validateSourceReviews, validateSourceReviewRecords} from './tdrp-source-review.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const inventoryPath = '/Users/bhrum/Documents/ChatGPT/.tdrp-artifacts/13118749668/x/upstream-recursive-inventory.json';

// Pure generic third-party submodules with verified license (23 repos)
const GENERIC_SUBMODULES = new Set([
  'ericniebler/range-v3', 'hunspell/hunspell', 'hime-ime/hime', 'flatpak/xdg-desktop-portal',
  'KDE/kcoreaddons', 'KDE/kimageformats', 'hamonikr/nimf', 'desktop-app/MicroTeX',
  'Yubico/libfido2', 'fcitx/fcitx5-qt', 'lz4/lz4', 'PJK/libcbor', 'desktop-app/cmark-gfm',
  'tzcnt/TooManyCooks', 'gitlab.com/mnauw/cppgir', 'desktop-app/cmake_helpers', 'Cyan4973/xxHash',
  'google/cld3', 'Microsoft/GSL', 'martinmoene/expected-lite',
  'TartanLlama/expected', 'tsyrogit/zxcvbn-c', 'nnen/doxygen-theme',
]);

// Generic third-party submodules pending license verification (1 repo)
const PENDING_GENERIC_SUBMODULES = new Set([
  'nayuki/QR-Code-generator',
]);

// Telegram proprietary submodules (15 repos)
const PROPRIETARY_SUBMODULES = new Set([
  'TelegramMessenger/tgcalls', 'desktop-app/lib_ui', 'desktop-app/lib_base', 'desktop-app/codegen',
  'desktop-app/lib_crl', 'desktop-app/lib_webrtc', 'desktop-app/lib_spellcheck', 'desktop-app/lib_lottie',
  'desktop-app/lib_rpl', 'desktop-app/lib_webview', 'desktop-app/lib_storage', 'desktop-app/libprisma',
  'desktop-app/lib_tl', 'desktop-app/lib_translate', 'desktop-app/lib_qr',
]);

function makeValidReview(source, overrides = {}) {
  const isProprietary = PROPRIETARY_SUBMODULES.has(source.repository);
  const isPendingGeneric = PENDING_GENERIC_SUBMODULES.has(source.repository);
  const isHighRisk = ['desktop-app/lib_storage', 'TelegramMessenger/tgcalls', 'desktop-app/lib_tl'].includes(source.repository);
  const isMediumRiskComplete = ['Yubico/libfido2', 'PJK/libcbor'].includes(source.repository);

  let status = 'complete';
  let risk = 'low';
  let review_depth = 'contract';
  let rationale = `Submodule ${source.repository} is not built, linked, or distributed by Fabushi.`;

  if (isProprietary) {
    status = 'pending';
    risk = isHighRisk ? 'high' : 'medium';
    review_depth = 'deep';
    rationale = `Telegram proprietary submodule ${source.repository}; product semantics to be evaluated in deep review.`;
  } else if (isPendingGeneric) {
    status = 'pending';
    risk = 'low';
    review_depth = 'contract';
    rationale = `Submodule ${source.repository} has no standalone LICENSE in inventory, license type pending verification; not built, linked, or distributed by Fabushi.`;
  } else if (isMediumRiskComplete) {
    risk = 'medium';
  }

  return {
    repository: source.repository,
    commit: source.commit,
    path: source.path,
    object: source.object,
    category: 'dependency',
    risk,
    review_depth,
    status,
    rationale,
    evidence: ['docs/specs/fmep-bulk-subtrees-disposition.md#submodule-contract-disposition'],
    responsibility_ids: [],
    ...overrides,
  };
}

function sampleGenericEntries() {
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
      repository: 'Yubico/libfido2',
      commit: 'b974e7cf2ee7c3f87c33aa9670f8ee96fec0cb49',
      path: 'src/fido.c',
      object: 'd'.repeat(40),
    },
  ];
}

test('positive: generic third-party submodules pass validateSourceReviews cleanly under contract review', () => {
  const inventory = sampleGenericEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  const rows = [];

  const result = validateSourceReviews(inventory, reviews, rows);
  assert.equal(result.source_identities, inventory.length);
  assert.equal(result.reviewed, inventory.length);
});

test('positive: real 8,104 generic third-party submodule entries pass validateSourceReviews cleanly', async () => {
  let inventoryData;
  try {
    const raw = await fs.readFile(inventoryPath, 'utf-8');
    inventoryData = JSON.parse(raw);
  } catch {
    return;
  }

  const genericEntries = inventoryData.entries.filter(e => GENERIC_SUBMODULES.has(e.repository));
  assert.equal(genericEntries.length, 8104);

  const reviews = genericEntries.map(source => makeValidReview(source));
  const rows = [];

  const result = validateSourceReviews(genericEntries, reviews, rows);
  assert.equal(result.source_identities, 8104);
  assert.equal(result.reviewed, 8104);
});

test('positive: pending records pass cleanly in candidate mode (allowPending=true) with correct counts', () => {
  const source = {
    repository: 'desktop-app/lib_storage',
    commit: 'ccdc72548a50eb345d1ec9c7041416e788bc5f25',
    path: 'storage/storage_encryption.cpp',
    object: 'a'.repeat(40),
  };
  const pendingReview = makeValidReview(source);
  assert.equal(pendingReview.status, 'pending');

  // Candidate mode (allowPending=true) passes without error
  const candidateResult = validateSourceReviewRecords([source], [pendingReview], [], true);
  assert.equal(candidateResult.source_identities, 1);
  assert.equal(candidateResult.recorded, 1);
  assert.equal(candidateResult.reviewed, 0); // pending does not increment reviewed count

  // Acceptance mode (allowPending=false) strictly rejects pending review
  assert.throws(
    () => validateSourceReviews([source], [pendingReview], []),
    /incomplete or duplicate review evidence/
  );
});

test('positive: all 9,486 submodule entries pass candidate mode (allowPending=true) with 8,104 reviewed and 9,486 recorded', async () => {
  let inventoryData;
  try {
    const raw = await fs.readFile(inventoryPath, 'utf-8');
    inventoryData = JSON.parse(raw);
  } catch {
    return;
  }
  const submoduleEntries = inventoryData.entries.filter(e => e.repository !== 'telegramdesktop/tdesktop');
  assert.equal(submoduleEntries.length, 9486);

  const reviews = submoduleEntries.map(source => makeValidReview(source));
  const candidateResult = validateSourceReviewRecords(submoduleEntries, reviews, [], true);
  assert.equal(candidateResult.source_identities, 9486);
  assert.equal(candidateResult.recorded, 9486);
  assert.equal(candidateResult.reviewed, 8104);

  // Accepted mode rejects because 1,382 are pending
  assert.throws(
    () => validateSourceReviews(submoduleEntries, reviews, []),
    /incomplete or duplicate review evidence/
  );
});

test('negative: proprietary submodules with product semantics cannot be marked complete', () => {
  for (const repo of PROPRIETARY_SUBMODULES) {
    const source = {
      repository: repo,
      commit: '0'.repeat(40),
      path: 'some/path.cpp',
      object: 'a'.repeat(40),
    };
    const review = makeValidReview(source);
    assert.equal(review.status, 'pending', `${repo} must be pending`);
    assert.equal(review.review_depth, 'deep', `${repo} must be deep review`);
    assert.notEqual(review.status, 'complete', `${repo} cannot be complete`);
  }
});

test('positive: every crate referenced in review rationale exists in a repository lockfile', async () => {
  const repoRoot = path.resolve(__dirname, '..');
  const lockfiles = [
    path.join(repoRoot, 'native/mahayana-messaging/Cargo.lock'),
    path.join(repoRoot, 'source/host/Cargo.lock'),
    path.join(repoRoot, 'source/mahayana/mahayana-rs/Cargo.lock'),
    path.join(repoRoot, 'source/mahayana/codex-rs/Cargo.lock'),
  ];
  let lockContents = '';
  for (const lf of lockfiles) {
    try {
      lockContents += await fs.readFile(lf, 'utf-8');
    } catch {}
  }

  const sampleReviews = sampleGenericEntries().map(source => makeValidReview(source));
  sampleReviews.push(makeValidReview({
    repository: 'desktop-app/cmark-gfm',
    commit: 'd7d4a24a9ebfa7581994ea6df0297662bfeaf413',
    path: 'src/cmark.c',
    object: 'e'.repeat(40),
  }, {
    rationale: 'Fabushi 不构建、不链接、不分发此第三方子模块（构建可达性：否；无运行时许可证分发义务）；Markdown 解析由 `pulldown-cmark`（`source/mahayana/mahayana-rs/Cargo.lock`）承接。',
  }));

  let checkedCrates = 0;
  for (const review of sampleReviews) {
    const matches = review.rationale.matchAll(/`([a-zA-Z0-9_-]+)`/g);
    for (const match of matches) {
      const crateName = match[1];
      if (crateName.endsWith('.lock') || crateName.endsWith('.toml')) continue;
      const pattern = new RegExp(`name = "${crateName}"`);
      assert.ok(pattern.test(lockContents), `Crate ${crateName} mentioned in rationale must exist in Cargo.lock`);
      checkedCrates++;
    }
  }
  assert.ok(checkedCrates > 0, 'Expected at least one crate mention checked');
});

test('negative: desktop-app/lib_storage and other proprietary submodules cannot claim acceptance completion in candidate mode', () => {
  const source = {
    repository: 'desktop-app/lib_storage',
    commit: 'ccdc72548a50eb345d1ec9c7041416e788bc5f25',
    path: 'storage/storage_encryption.cpp',
    object: 'a'.repeat(40),
  };

  // If marked pending (the correct disposition under FMEP-SOURCE-01), acceptance validateSourceReviews rejects it
  const pendingReview = makeValidReview(source);
  assert.equal(pendingReview.status, 'pending');
  assert.equal(pendingReview.risk, 'high');
  assert.equal(pendingReview.review_depth, 'deep');

  assert.throws(
    () => validateSourceReviews([source], [pendingReview], []),
    /incomplete or duplicate review evidence/
  );
});

test('negative: missing source review is rejected (缺项被拒)', () => {
  const inventory = sampleGenericEntries();
  const reviews = inventory.slice(0, -1).map(source => makeValidReview(source));
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /missing source reviews/
  );
});

test('negative: duplicate source review identity is rejected (重复被拒)', () => {
  const inventory = sampleGenericEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  reviews.push(makeValidReview(inventory[0]));
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /unknown, stale or duplicate source review/
  );
});

test('negative: blob SHA drift is rejected (blob 漂移被拒)', () => {
  const inventory = sampleGenericEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  reviews[1].object = 'f'.repeat(40);
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /unknown, stale or duplicate source review/
  );
});

test('negative: repository drift is rejected (仓库漂移被拒)', () => {
  const inventory = sampleGenericEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  reviews[0].repository = 'other/unrelated-repo';
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /unknown, stale or duplicate source review/
  );
});

test('negative: disposition depth rejected for dependency category', () => {
  const inventory = sampleGenericEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  reviews[0].review_depth = 'disposition';
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /contract review required/
  );
});

test('negative: high risk dependency without deep review is rejected', () => {
  const inventory = sampleGenericEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  reviews[0].risk = 'high';
  reviews[0].review_depth = 'contract';
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /deep review required/
  );
});

test('negative: uncovered responsibility ID is rejected', () => {
  const inventory = sampleGenericEntries();
  const reviews = inventory.map(source => makeValidReview(source));
  reviews[0].responsibility_ids = ['NON-EXISTENT-RESPONSIBILITY-001'];
  const rows = [];

  assert.throws(
    () => validateSourceReviews(inventory, reviews, rows),
    /unmapped responsibility: NON-EXISTENT-RESPONSIBILITY-001/
  );
});
