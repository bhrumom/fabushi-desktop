import {test} from 'node:test';
import assert from 'node:assert/strict';
import {
  assertNoOpenSourceCandidate,
  readFrozenUpstreamCommit,
  validateSourceReviews as validate,
} from './tdrp-source-review.mjs';

const source = {
  repository: 'owner/repo',
  commit: 'a'.repeat(40),
  path: 'src/message.cpp',
  object: 'b'.repeat(40),
};
const secondSource = {
  repository: 'owner/dependency',
  commit: 'c'.repeat(40),
  path: 'include/message.h',
  object: 'd'.repeat(40),
};
const review = {
  ...source,
  category: 'product',
  risk: 'high',
  review_depth: 'deep',
  status: 'complete',
  rationale: 'state and recovery reviewed',
  evidence: ['dossier#message'],
  responsibility_ids: ['message'],
};
const rowFor = (identity, id = 'message') => ({
  responsibility_id: id,
  source_repository: identity.repository,
  source_commit: identity.commit,
  source_path: identity.path,
  source_blob_sha: identity.object,
});
const rows = [rowFor(source)];
const secondReview = {...review, ...secondSource, evidence: ['dossier#message-header']};

test('product deep review follows the primary source and returns source counts only', () => {
  assert.deepEqual(validate([source], [review], rows), {source_identities: 1, reviewed: 1});
});

for (const category of ['generated', 'dependency', 'resource', 'documentation', 'tooling']) {
  test(category + ' accepts evidenced non-deep disposition without product rows', () => {
    const disposition = {
      ...review,
      category,
      risk: 'low',
      review_depth: ['generated', 'dependency'].includes(category) ? 'contract' : 'disposition',
      responsibility_ids: [],
    };
    assert.equal(validate([source], [disposition], []).reviewed, 1);
  });
}

test('mixed UI and high-risk dependencies accept deep review with an exact source link', () => {
  for (const category of ['mixed-ui', 'dependency']) {
    assert.equal(validate([source], [{...review, category}], rows).reviewed, 1);
  }
});

test('one responsibility can explicitly cover additional exact recursive identities', () => {
  const linkedRows = [{...rows[0], additional_source_identities: [secondSource]}];
  assert.deepEqual(validate([source, secondSource], [review, secondReview], linkedRows), {
    source_identities: 2,
    reviewed: 2,
  });
});

test('one source can map to distinct responsibilities without duplicating source reviews', () => {
  const splitRows = [rows[0], rowFor(source, 'permissions')];
  const splitReview = {...review, responsibility_ids: ['message', 'permissions']};
  assert.equal(validate([source], [splitReview], splitRows).reviewed, 1);
});

const changedFields = {
  repository: 'owner/other-repo',
  commit: 'c'.repeat(40),
  path: 'src/other.cpp',
  object: 'd'.repeat(40),
};
for (const [field, value] of Object.entries(changedFields)) {
  test('reject stale review ' + field, () => {
    assert.throws(() => validate([source], [{...review, [field]: value}], rows),
      /unknown, stale or duplicate source review/);
  });
  test('an existing responsibility cannot close another source with different ' + field, () => {
    const other = {...source, [field]: value};
    assert.throws(() => validate([source, other], [review, {...review, ...other}], rows),
      /responsibility does not cover source identity/);
  });
}

for (const [name, patch, message] of [
  ['pending', {status: 'pending'}, /incomplete or duplicate review evidence/],
  ['missing evidence', {evidence: []}, /incomplete or duplicate review evidence/],
  ['blank evidence', {evidence: ['   ']}, /incomplete or duplicate review evidence/],
  ['duplicate evidence', {evidence: ['dossier#message', 'dossier#message']}, /incomplete or duplicate review evidence/],
  ['missing rationale', {rationale: ''}, /incomplete or duplicate review evidence/],
  ['shallow product', {review_depth: 'contract'}, /deep review required/],
  ['shallow mixed UI', {category: 'mixed-ui', review_depth: 'contract'}, /deep review required/],
  ['shallow high risk', {category: 'dependency', review_depth: 'contract'}, /deep review required/],
  ['unknown responsibility', {responsibility_ids: ['absent']}, /unmapped responsibility/],
  ['missing product mapping', {responsibility_ids: []}, /missing or duplicate responsibility links/],
  ['duplicate responsibility links', {responsibility_ids: ['message', 'message']}, /missing or duplicate responsibility links/],
  ['unknown category', {category: 'ignored'}, /unknown category/],
  ['unknown risk', {risk: 'safe'}, /unknown risk/],
  ['unknown review depth', {review_depth: 'summary'}, /unknown review depth/],
  ['dependency disposition', {category: 'dependency', risk: 'low', review_depth: 'disposition'}, /contract review required/],
  ['generated disposition', {category: 'generated', risk: 'low', review_depth: 'disposition'}, /contract review required/],
]) {
  test('reject ' + name, () => {
    assert.throws(() => validate([source], [{...review, ...patch}], rows), message);
  });
}

test('reject unrelated responsibility links even for non-product dispositions', () => {
  const disposition = {...review, category: 'documentation', risk: 'low', review_depth: 'disposition'};
  assert.throws(() => validate([source, secondSource], [disposition], [rowFor(secondSource)]),
    /responsibility does not cover source identity/);
});

test('reject duplicate inventory identities even when mount metadata differs', () => {
  assert.throws(() => validate([source, {...source, mount: 'another/mount'}], [review], rows),
    /duplicate inventory source identity/);
});

test('reject duplicate and missing source reviews', () => {
  assert.throws(() => validate([source], [review, review], rows), /duplicate source review/);
  assert.throws(() => validate([source], [], rows), /missing source reviews/);
});

test('reject undeclared review fields consistently with the strict schema', () => {
  assert.throws(() => validate([source], [{...review, accepted: true}], rows), /unknown source review field/);
});

test('additional source links never replace individual source review evidence', () => {
  const linkedRows = [{...rows[0], additional_source_identities: [secondSource]}];
  assert.throws(() => validate([source, secondSource], [review], linkedRows), /missing source reviews/);
});

test('reject duplicate responsibility IDs', () => {
  assert.throws(() => validate([source], [review], [rows[0], {...rows[0]}]),
    /duplicate responsibility id/);
});

test('reject responsibility primary sources absent from the inventory', () => {
  assert.throws(() => validate([source], [review], [rowFor(secondSource)]),
    /unknown responsibility source identity/);
});

test('reject unknown additional source identities', () => {
  const linkedRows = [{...rows[0], additional_source_identities: [secondSource]}];
  assert.throws(() => validate([source], [review], linkedRows), /unknown additional source identity/);
});

test('reject duplicate additional identities and repetition of the primary identity', () => {
  for (const additional of [[source], [secondSource, {...secondSource}]]) {
    assert.throws(() => validate([source, secondSource], [review, secondReview],
      [{...rows[0], additional_source_identities: additional}]), /duplicate responsibility source identity/);
  }
});

test('additional identities use the same exact four fields as the schema', () => {
  assert.throws(() => validate([source, secondSource], [review, secondReview],
    [{...rows[0], additional_source_identities: [{...secondSource, wildcard: true}]}]),
    /unknown additional source identity field/);
  for (const additional of [null, {}, 'owner/repo']) {
    assert.throws(() => validate([source], [review],
      [{...rows[0], additional_source_identities: additional}]), /additional source identities must be an array/);
  }
});

test('reject malformed Git identities at every input boundary', () => {
  for (const field of ['commit', 'object']) {
    for (const value of ['abc123', 'A'.repeat(40), 123, null]) {
      const invalid = {...source, [field]: value};
      assert.throws(() => validate([invalid], [review], rows), /invalid source identity: inventory/);
      assert.throws(() => validate([source], [{...review, [field]: value}], rows), /invalid source identity: review/);
      assert.throws(() => validate([source], [review], [rowFor(invalid)]), /invalid source identity: responsibility/);
      assert.throws(() => validate([source], [review],
        [{...rows[0], additional_source_identities: [invalid]}]), /invalid source identity: additional/);
    }
  }
});

test('reject missing repository or path identities instead of treating missing fields as equal', () => {
  for (const field of ['repository', 'path']) {
    const invalid = {...source, [field]: ''};
    assert.throws(() => validate([invalid], [{...review, [field]: ''}], [rowFor(invalid)]),
      /invalid source identity/);
  }
  assert.throws(() => validate([source], [review], [{responsibility_id: 'message'}]),
    /invalid source identity: responsibility/);
});

test('reject absent or malformed input collections', () => {
  for (const inventory of [undefined, [], {}]) {
    assert.throws(() => validate(inventory, [review], rows), /non-empty inventory required/);
  }
  assert.throws(() => validate([source], undefined, rows), /source reviews/);
  assert.throws(() => validate([source], [review], undefined), /responsibility rows required/);
  assert.throws(() => validate([source], [null], rows), /invalid source identity: review/);
  assert.throws(() => validate([source], [review], [{...rows[0], responsibility_id: ''}]),
    /invalid responsibility id/);
});

const frozenUpstream = {repository: 'owner/repo', commit: 'a'.repeat(40), tree: 'e'.repeat(40)};
const frozenCommitUrl = 'https://api.github.com/repos/owner/repo/git/commits/' + frozenUpstream.commit;
const frozenCommit = {sha: frozenUpstream.commit, tree: {sha: frozenUpstream.tree}, message: 'fixed research source'};

test('frozen upstream reads only the pinned commit even when dev advances', async () => {
  for (const movingHead of ['b'.repeat(40), 'c'.repeat(40)]) {
    const calls = [];
    const responses = new Map([
      [frozenCommitUrl, frozenCommit],
      ['https://api.github.com/repos/owner/repo/branches/dev', {commit: {sha: movingHead}}],
    ]);
    const result = await readFrozenUpstreamCommit({...frozenUpstream, branch: 'dev'}, async url => {
      calls.push(url);
      assert.ok(responses.has(url), 'unexpected upstream request: ' + url);
      return responses.get(url);
    });
    assert.strictEqual(result, frozenCommit);
    assert.deepEqual(calls, [frozenCommitUrl]);
  }
});

test('frozen upstream snapshots the selected identity before its asynchronous read', async () => {
  const selected = {...frozenUpstream};
  const result = await readFrozenUpstreamCommit(selected, async url => {
    assert.equal(url, frozenCommitUrl);
    selected.commit = 'b'.repeat(40);
    selected.tree = 'c'.repeat(40);
    return frozenCommit;
  });
  assert.strictEqual(result, frozenCommit);
});

test('frozen upstream rejects mismatched or missing returned commit and tree identities', async () => {
  for (const [response, message] of [
    [null, /frozen upstream commit mismatch/],
    [{...frozenCommit, sha: 'b'.repeat(40)}, /frozen upstream commit mismatch/],
    [{...frozenCommit, tree: undefined}, /frozen upstream tree mismatch/],
    [{...frozenCommit, tree: {sha: 'f'.repeat(40)}}, /frozen upstream tree mismatch/],
  ]) {
    await assert.rejects(readFrozenUpstreamCommit(frozenUpstream, async () => response), message);
  }
});

test('frozen upstream rejects missing or malformed pins before requesting data', async () => {
  const malformed = [null, {}];
  for (const field of ['repository', 'commit', 'tree']) {
    malformed.push({...frozenUpstream, [field]: undefined}, {...frozenUpstream, [field]: ''});
  }
  for (const field of ['commit', 'tree']) {
    malformed.push({...frozenUpstream, [field]: 'dev'}, {...frozenUpstream, [field]: 'A'.repeat(40)});
  }
  for (const repository of ['owner/repo/extra', 'https://github.com/owner/repo', 'owner/..', 'owner/repo?ref=dev', ' owner/repo']) {
    malformed.push({...frozenUpstream, repository});
  }
  let requests = 0;
  for (const upstream of malformed) {
    await assert.rejects(readFrozenUpstreamCommit(upstream, async () => {
      requests++;
      return frozenCommit;
    }), /invalid frozen upstream/);
  }
  assert.equal(requests, 0);
});

test('frozen upstream preserves retrieval failures and never falls back to a branch', async () => {
  const unavailable = new Error('exact commit unavailable');
  const calls = [];
  await assert.rejects(readFrozenUpstreamCommit(frozenUpstream, async url => {
    calls.push(url);
    throw unavailable;
  }), error => error === unavailable);
  assert.deepEqual(calls, [frozenCommitUrl]);
  await assert.rejects(readFrozenUpstreamCommit(frozenUpstream, null), /JSON reader required/);
});

test('source acceptance rejects any retained rebaseline candidate even if marked rejected or null', () => {
  for (const candidate of [{status: 'rejected'}, {status: 'complete'}, {}, null, false, 0, '']) {
    const ledger = {
      upstream_commit: frozenUpstream.commit,
      coverage: {source_entries_total: null, unknown: null, unread: null},
      rebaseline_candidate: candidate,
    };
    const before = structuredClone(ledger);
    assert.throws(() => assertNoOpenSourceCandidate(ledger), /cannot contain a rebaseline candidate/);
    assert.deepEqual(ledger, before);
  }
});

test('absence of a source candidate neither mutates unknown counts nor grants acceptance', () => {
  const ledger = {coverage: {source_entries_total: null, unknown: null, unread: null}};
  const before = structuredClone(ledger);
  assert.equal(assertNoOpenSourceCandidate(ledger), undefined);
  assert.deepEqual(ledger, before);
  for (const invalid of [null, undefined, []]) {
    assert.throws(() => assertNoOpenSourceCandidate(invalid), /source ledger object required/);
  }
});

