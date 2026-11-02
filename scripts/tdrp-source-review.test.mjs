import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {readRecoveredCandidateDocuments, main as sourceCandidateMain} from './validate-tdrp-source-candidate.mjs';
import {
  assertNoOpenSourceCandidate,
  assertSourceAcquisitionClosure,
  validateSourceCandidate,
  validateResponsibilitySources,
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

function closedAcquisitionFixture() {
  const upstream = {repository: 'owner/repo', commit: 'a'.repeat(40), tree: 'b'.repeat(40)};
  return {
    lock: {
      upstream,
      acceptance: {accepted: false, baseline_ready: false},
      build_time_acquisition_inventory: {
        path: 'projects/telegram-desktop-rust/inventory/build-time-acquisitions.json',
        upstream_repository: upstream.repository, upstream_commit: upstream.commit, upstream_tree: upstream.tree,
        closure_status: 'complete', disposition_closure_status: 'complete', recursive_authority_closure_status: 'complete',
        candidate_lines_pending_classification: 0, recursive_child_unique_external_nested_scans_pending: 0,
      },
    },
    acquisition: {
      upstream_repository: upstream.repository, upstream_commit: upstream.commit, upstream_tree: upstream.tree,
      current_source_status: 'complete',
      discovery: {
        disposition_closure_status: 'complete', recursive_authority_closure_status: 'complete', evidence_status: 'complete',
        candidate_lines_pending_classification: 0, recursive_child_unique_external_nested_scans_pending: 0,
      },
      recursive_build_reachability: {status: 'complete', accepted_upstream_commit: upstream.commit},
    },
  };
}

test('closed acquisition state is only a necessary guard and never grants source or product acceptance', () => {
  const f = closedAcquisitionFixture();
  const before = structuredClone(f);
  assert.equal(assertSourceAcquisitionClosure(f.lock, f.acquisition), undefined);
  assert.deepEqual(f, before);
  assert.equal(f.lock.acceptance.accepted, false);
  assert.equal(f.lock.acceptance.baseline_ready, false);
});

test('complete source reviews cannot bypass acquisition candidates or a candidate JSON pointer', () => {
  assert.equal(validate([source], [review], rows).reviewed, 1);
  for (const value of [null, false, {status: 'complete'}]) {
    for (const owner of ['lock', 'acquisition']) {
      const f = closedAcquisitionFixture();
      f[owner].rebaseline_candidate = value;
      assert.throws(() => assertSourceAcquisitionClosure(f.lock, f.acquisition), /candidate marker remains/);
    }
  }
  for (const value of ['/rebaseline_candidate', '/another-authority', null]) {
    const f = closedAcquisitionFixture();
    f.lock.build_time_acquisition_inventory.json_pointer = value;
    assert.throws(() => assertSourceAcquisitionClosure(f.lock, f.acquisition), /current root authority/);
  }
});

test('accepted acquisition guard rejects open partial historical missing and invented completion states', () => {
  const paths = [
    ['lock', 'build_time_acquisition_inventory', 'closure_status'],
    ['lock', 'build_time_acquisition_inventory', 'disposition_closure_status'],
    ['lock', 'build_time_acquisition_inventory', 'recursive_authority_closure_status'],
    ['acquisition', 'current_source_status'],
    ['acquisition', 'discovery', 'disposition_closure_status'],
    ['acquisition', 'discovery', 'recursive_authority_closure_status'],
    ['acquisition', 'discovery', 'evidence_status'],
    ['acquisition', 'recursive_build_reachability', 'status'],
  ];
  for (const fields of paths) {
    for (const value of ['open', 'partial-audited-disposition-open', 'historical-pending-fixed-u-revalidation',
      'pending-current-head-revalidation', 'unaccepted', 'unknown', 'done', '', null, undefined]) {
      const f = closedAcquisitionFixture();
      const owner = fields.slice(0, -1).reduce((item, field) => item[field], f);
      if (value === undefined) delete owner[fields.at(-1)];
      else owner[fields.at(-1)] = value;
      assert.throws(() => assertSourceAcquisitionClosure(f.lock, f.acquisition), /must be explicitly complete/);
    }
  }
});

test('accepted acquisition guard requires integer zero pending counts in both actual authority and lock', () => {
  for (const scope of ['pointer', 'discovery']) {
    for (const field of ['candidate_lines_pending_classification', 'recursive_child_unique_external_nested_scans_pending']) {
      for (const value of [1, -1, 0.5, '0', null, undefined]) {
        const f = closedAcquisitionFixture();
        const owner = scope === 'pointer' ? f.lock.build_time_acquisition_inventory : f.acquisition.discovery;
        if (value === undefined) delete owner[field];
        else owner[field] = value;
        assert.throws(() => assertSourceAcquisitionClosure(f.lock, f.acquisition), /must be integer zero/);
      }
    }
  }
});

test('accepted acquisition guard binds the actual source and reachability to fixed U', () => {
  for (const [scope, field, value] of [
    ['acquisition', 'upstream_repository', 'owner/other'],
    ['acquisition', 'upstream_commit', 'c'.repeat(40)],
    ['acquisition', 'upstream_tree', 'c'.repeat(40)],
    ['pointer', 'upstream_commit', 'c'.repeat(40)],
    ['pointer', 'upstream_tree', 'c'.repeat(40)],
    ['reachability', 'accepted_upstream_commit', 'c'.repeat(40)],
  ]) {
    const f = closedAcquisitionFixture();
    const owner = scope === 'pointer' ? f.lock.build_time_acquisition_inventory
      : scope === 'reachability' ? f.acquisition.recursive_build_reachability : f.acquisition;
    owner[field] = value;
    assert.throws(() => assertSourceAcquisitionClosure(f.lock, f.acquisition), /identity|drift/);
  }
});

test('complete review records cannot override explicit unresolved generated LFS build or legal evidence', () => {
  assert.equal(validate([source], [review], rows).reviewed, 1);
  for (const field of ['build_time_acquisitions_complete', 'generated_inputs_complete',
    'lfs_object_closure_complete', 'legal_review_performed']) {
    for (const scope of ['pointer', 'acquisition', 'discovery', 'reachability', 'acceptance']) {
      const f = closedAcquisitionFixture();
      const owner = scope === 'pointer' ? f.lock.build_time_acquisition_inventory
        : scope === 'discovery' ? f.acquisition.discovery
        : scope === 'reachability' ? f.acquisition.recursive_build_reachability
        : scope === 'acceptance' ? f.lock.acceptance : f.acquisition;
      owner[field] = false;
      assert.throws(() => assertSourceAcquisitionClosure(f.lock, f.acquisition), /remains unresolved/);
    }
  }
});

test('responsibility mapping accepts root and recursive primary sources at their own pinned commits', () => {
  const dependency = {...secondSource, path: source.path};
  const gitlabDependency = {...dependency, repository: 'gitlab.com/owner/dependency'};
  const inventory = [source, dependency, gitlabDependency];
  const mapped = [rowFor(source), rowFor(dependency, 'dependency'), rowFor(gitlabDependency, 'gitlab')];
  assert.deepEqual(validateResponsibilitySources(inventory, mapped), {
    source_identities: 3, responsibilities: 3,
  });
  // This is a pre-acceptance mapping check: source_reviews need not yet exist,
  // but the accepted review gate must still reject incomplete review coverage.
  assert.throws(() => validate(inventory, undefined, mapped), /source reviews/);
  assert.deepEqual(validate(inventory, [
    review,
    {...review, ...dependency, responsibility_ids: ['dependency']},
    {...review, ...gitlabDependency, responsibility_ids: ['gitlab']},
  ], mapped), {source_identities: 3, reviewed: 3});
});

for (const [field, value] of Object.entries({
  repository: 'unrelated/repository',
  commit: 'e'.repeat(40),
  path: 'unacquired/message.h',
  object: 'f'.repeat(40),
})) {
  test('responsibility mapping rejects recursive primary ' + field + ' drift', () => {
    assert.throws(() => validateResponsibilitySources([source, secondSource],
      [rowFor(source), rowFor({...secondSource, [field]: value}, 'dependency')]),
    /unknown responsibility source identity/);
  });
}

test('a row cannot declare its own recursive inventory membership', () => {
  assert.throws(() => validateResponsibilitySources([source], [rowFor(secondSource)]),
    /unknown responsibility source identity/);
  const foreign = {...source, repository: 'unrelated/repository'};
  assert.throws(() => validateResponsibilitySources([source], [rowFor(foreign)]),
    /unknown responsibility source identity/);
});

test('pre-acceptance mapping retains strict additional identity and uniqueness checks', () => {
  const linked = {...rows[0], additional_source_identities: [secondSource]};
  assert.deepEqual(validateResponsibilitySources([source, secondSource], [linked]), {
    source_identities: 2, responsibilities: 1,
  });
  assert.throws(() => validateResponsibilitySources([source], [linked]), /unknown additional source identity/);
  assert.throws(() => validateResponsibilitySources([source, secondSource],
    [{...linked, additional_source_identities: [{...secondSource, wildcard: true}]}]),
  /unknown additional source identity field/);
  assert.throws(() => validateResponsibilitySources([source, secondSource],
    [{...linked, additional_source_identities: [secondSource, {...secondSource}]}]),
  /duplicate responsibility source identity/);
  assert.throws(() => validateResponsibilitySources([source],
    [{...rows[0], additional_source_identities: [source]}]), /duplicate responsibility source identity/);
  assert.throws(() => validateResponsibilitySources([source], [rows[0], {...rows[0]}]),
    /duplicate responsibility id/);
  assert.throws(() => validateResponsibilitySources([source, {...source, mount: 'other'}], rows),
    /duplicate inventory source identity/);
});

test('pre-acceptance mapping rejects missing collections and malformed Git identities', () => {
  for (const inventory of [undefined, [], {}]) {
    assert.throws(() => validateResponsibilitySources(inventory, rows), /non-empty inventory required/);
  }
  assert.throws(() => validateResponsibilitySources([source], undefined), /responsibility rows required/);
  for (const field of ['commit', 'object']) {
    const invalid = {...secondSource, [field]: 'not-a-git-oid'};
    assert.throws(() => validateResponsibilitySources([source, invalid], rows), /invalid source identity: inventory/);
    assert.throws(() => validateResponsibilitySources([source, secondSource], [rowFor(invalid)]),
      /invalid source identity: responsibility/);
  }
});

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


function candidateFixture() {
  const upstream = {repository: 'telegramdesktop/tdesktop',
    commit: 'f985723b552b30fe1c17d377d57f44e61a15a8c4',
    tree: '91f2b083ab2877c0a17d07868eabec362a5cfca6'};
  const cmake = {repository: 'desktop-app/cmake_helpers', commit: '7ceca3019b4750d7a5acb2de0faffd523ee14095'};
  const cppgirCommit = '47cf94f83b54cda59018135601e19d7fb0c77776';
  const expectedCommit = '95b9cb015fa17baa749c2b396b335906e1596a9e';
  const primary = {repository: upstream.repository, commit: upstream.commit,
    path: 'Telegram/build/prepare/prepare.py', object: 'b'.repeat(40), mode: '100644', type: 'blob'};
  const generator = {...cmake, path: 'generator.cmake', object: 'f'.repeat(40), mode: '100644', type: 'blob'};
  const opaque = {...cmake, path: 'modules/compiler.dll', object: 'c'.repeat(40), mode: '100644', type: 'blob'};
  const rootEntries = [
    {path: primary.path, object: primary.object, mode: '100644', type: 'blob'},
    {path: 'cmake', object: cmake.commit, mode: '160000', type: 'commit'},
  ];
  const entries = [
    primary, {...rootEntries[1], repository: upstream.repository, commit: upstream.commit},
    {...cmake, path: 'external/glib/cppgir', object: cppgirCommit, mode: '160000', type: 'commit'},
    {repository: 'gitlab.com/mnauw/cppgir', commit: cppgirCommit, path: 'CMakeLists.txt',
      object: 'e'.repeat(40), mode: '100644', type: 'blob'},
    generator, opaque,
    {repository: 'gitlab.com/mnauw/cppgir', commit: cppgirCommit, path: 'expected-lite',
      object: expectedCommit, mode: '160000', type: 'commit'},
    {repository: 'martinmoene/expected-lite', commit: expectedCommit, path: 'include/nonstd/expected.hpp',
      object: 'd'.repeat(40), mode: '100644', type: 'blob'},
  ];
  const falseClaims = Object.fromEntries([
    'baseline_accepted', 'source_closure_ready', 'build_time_acquisitions_complete',
    'generated_inputs_complete', 'lfs_object_closure_complete', 'semantic_review_performed',
    'legal_review_performed', 'product_accepted', 'independent_acceptance',
  ].map(key => [key, false]));
  const artifact = {repository: 'bhrumom/fabushi-desktop', head_sha: '1'.repeat(40),
    run_id: 101, artifact_id: 102, digest: 'sha256:' + '2'.repeat(64)};
  const exportHeader = {format: 'TDRP_FIXED_U_BUILD_INPUT_EXPORT_V1',
    project_id: 'TDRP-001', spec_revision: 9, target_commit: artifact.head_sha,
    fixed_upstream: upstream, complete: true, status: 'success', ...falseClaims};
  const scopes = {'legacy-root-paths': 1, 'pinned-cmake-component': 1};
  const contexts = [
    {...primary, observation_id: 'root-observation', source_line: 3,
      raw_line: 'git clone https://github.com/example/dependency.git', scan_scope: 'legacy-root-paths',
      ref_observations: [{literal_commit: null}]},
    {...generator, observation_id: 'cmake-observation', source_line: 4,
      raw_line: 'git clone https://github.com/example/generator.git', scan_scope: 'pinned-cmake-component',
      ref_observations: [{literal_commit: '4'.repeat(40)}]},
  ];
  const buildInputs = {
    artifact,
    report: {...exportHeader, source_commands_executed: false, upstream_refs_resolved: false,
      selected_text_source_read_complete: true, selected_source_read_complete: false,
      selected_text_source_count: 2, selected_source_count: 3, read_source_files: 2,
      opaque_inputs_not_read: 1, context_observations: 2, unresolved_ref_observations: 1,
      candidate_lines_total: 2, candidate_lines_by_scope: scopes,
      git_inventory_counts: {source_entries: entries.length, root_non_directory_entries: rootEntries.length}},
    candidates: {...exportHeader, candidates: contexts.map(context => ({...context,
      disposition_status: 'pending-review'})), disposition_complete: false,
      total_candidate_lines: 2, counts_by_scan_scope: scopes},
    contexts: {...exportHeader, contexts, source_files: [primary, generator].map(entry => ({
      ...entry, utf8_complete: true, semantic_review_complete: false,
    })), unread_inputs: [{...opaque, disposition: 'registered-opaque-input-unread',
      source_bytes_read: false, lexical_scan_complete: false,
      semantic_review_complete: false, legal_review_complete: false}]},
    delta: {...exportHeader, records: []},
  };
  const current = {status: 'unaccepted-build-input-observations',
    upstream_repository: upstream.repository, upstream_commit: upstream.commit,
    upstream_tree: upstream.tree, evidence: {...artifact}, classification_status: 'pending-source-review',
    total_candidate_lines: 2, counts_by_scope: {...scopes}, text_files_read: 2, opaque_inputs_unread: 1,
    context_observations: 2, unresolved_ref_observations: 1, ...falseClaims};
  const pointer = {path: 'projects/telegram-desktop-rust/inventory/build-time-acquisitions.json',
    json_pointer: '/rebaseline_candidate', upstream_commit: upstream.commit,
    status: current.status, evidence: {...artifact}};
  const header = {project_id: 'TDRP-001', spec_revision: 9};
  const history = {...header, accepted_upstream_commit: '9'.repeat(40),
    current_source_status: 'historical-pending-fixed-u-revalidation',
    candidate_evidence: {upstream_commit: upstream.commit, path: pointer.path,
      json_pointer: pointer.json_pointer, evidence: {...artifact}}};
  const node = (repository_id, repository_url, commit, tree, depth) => ({
    repository_id, repository_url, commit, tree, depth, status: 'success', fetch_verified: true,
    tree_enumerated_completely: true, recursive_git_trees_enumerated_completely: true,
  });
  const repositories = [
    node('repo-000', 'https://github.com/telegramdesktop/tdesktop.git', upstream.commit, upstream.tree, 0),
    node('repo-001', 'https://github.com/desktop-app/cmake_helpers.git', cmake.commit, 'a'.repeat(40), 1),
    node('repo-002', 'https://gitlab.com/mnauw/cppgir.git', cppgirCommit, 'b'.repeat(40), 2),
    node('repo-003', 'https://github.com/martinmoene/expected-lite.git', expectedCommit, 'c'.repeat(40), 3),
  ];
  const edge = (index, declared_path) => {
    const parent = repositories[index], child = repositories[index + 1];
    return {edge_id: 'edge-' + index, status: 'attested', declared_path,
      declared_url: child.repository_url, child_repository_id: child.repository_id,
      child_commit: child.commit, child_tree: child.tree, child_depth: child.depth, gitlink: child.commit,
      parent_repository_id: parent.repository_id, parent_commit: parent.commit,
      parent_tree: parent.tree, parent_url: parent.repository_url,
      gitmodules_blob: ['2a71e861fab57dfd74d66e787867f2ede203a099',
        'c92fbea3467f72e188cba4cbfe51c8c68a7e2e77', 'e0c91502bcb2798a8a57bd85e49c76e0ef7275a3'][index]};
  };
  const acquisitionManifest = {...header, format: 'TDRP_FIXED_UPSTREAM_GIT_V1',
    root: upstream, root_repository_id: 'repo-000', status: 'success',
    canonical_inventory_complete: true, git_trees_enumerated_completely: true,
    root_commit_tree_verified: true, canonical_entries_emitted: entries.length,
    recursive_non_directory_entries: entries.length, ...falseClaims,
    repositories, repository_count: repositories.length,
    edges: [edge(0, 'cmake'), edge(1, 'external/glib/cppgir'), edge(2, 'expected-lite')], edge_count: 3,
    cppgir_component_proof: {child: {...repositories[2]}, child_tree_enumerated_completely: true,
      nested_gitlinks: [{path: 'expected-lite', object: expectedCommit, mode: '160000', type: 'commit'}]},
  };
  return {
    lock: {...header, upstream: {...upstream, accepted: false},
      acceptance: {accepted: false, baseline_ready: false},
      direct_gitlinks: [{path: 'cmake', repository: cmake.repository, commit: cmake.commit}],
      known_nested_gitlinks: [{parent: cmake.repository + '@' + cmake.commit,
        path: 'external/glib/cppgir', repository: 'gitlab.com/mnauw/cppgir', commit: cppgirCommit}],
      observed_root_inventory: {root_non_directory_count: rootEntries.length, direct_gitlink_count: 1},
      coverage: {root_source_entries_total: rootEntries.length, recursive_source_entries_total: entries.length,
        unknown_minimum: null, unread_minimum: null, omitted_known: 0},
      build_time_acquisition_inventory: pointer},
    index: {...header, upstream: {...upstream, baseline_identity_accepted: false},
      inventory: {root_non_directory_entries: rootEntries.length, recursive_non_directory_entries: entries.length,
        unknown_minimum: null, unread_minimum: null, omitted_known: 0,
        semantic_read_complete: false, responsibility_decomposition_complete: false}},
    ledger: {...header, format_version: 3, upstream_commit: upstream.commit,
      coverage: {source_entries_total: entries.length, responsibilities_total: 1,
        unknown: null, unread: null, omitted: 0, unmapped_responsibilities: 0},
      rows: [rowFor(primary)], rebaseline_candidate: {
        upstream_commit: upstream.commit, status: 'unaccepted-inventory-open',
        direct_gitlinks_checked: 1, nested_gitlinks_discovered: 2,
        nested_gitlinks_resolved: 2, nested_gitlinks_blocked: 0,
        source_responsibility_rows: 'Fixture identity observations only',
        unknown: null, unread: null, omitted_known: 0,
        cppgir_identity: {declared_by: cmake.repository + '@' + cmake.commit +
          ':.gitmodules#c92fbea3467f72e188cba4cbfe51c8c68a7e2e77', repository: 'https://gitlab.com/mnauw/cppgir.git',
          commit: cppgirCommit, git_tree_verified: true, reason: 'Fixture Git proof only'},
      }},
    acquisitions: {...header, upstream_commit: '9'.repeat(40),
      current_source_status: 'historical-pending-fixed-u-revalidation', rebaseline_candidate: current,
      discovery: {candidate_lines_total: 331, candidate_lines_classified: 331}},
    mutableAuthorities: structuredClone(history), toolAuthorities: structuredClone(history),
    rootInventory: {...header, ...upstream, entries: rootEntries,
      inventory_status: 'unaccepted-git-inventory-candidate', baseline_accepted: false, source_closure_ready: false},
    recursiveInventory: {...header, root: {...upstream}, entries,
      counts: {entries: entries.length}, inventory_status: 'unaccepted-git-inventory-candidate',
      baseline_accepted: false, source_closure_ready: false},
    inventoryReport: {...header, root: {...upstream}, status: 'success',
      canonical_inventory_complete: true, git_trees_enumerated_completely: true, root_commit_tree_verified: true,
      recursive_non_directory_entries: entries.length, root_non_directory_entries: rootEntries.length, ...falseClaims},
    acquisitionManifest, buildInputs,
  };
}

test('focused candidate checks actual source arrays and preserves unestablished counters without acceptance', () => {
  const fixture = candidateFixture();
  const before = structuredClone(fixture);
  const result = validateSourceCandidate(fixture);
  assert.equal(result.source_entries_total, fixture.recursiveInventory.entries.length);
  assert.equal(result.responsibility_rows, 1);
  assert.equal(result.lexical_observations, 2);
  assert.deepEqual(result.counts_by_scope, {'legacy-root-paths': 1, 'pinned-cmake-component': 1});
  assert.equal(result.unknown, null);
  assert.equal(result.unread, null);
  assert.equal(result.source_closure_ready, false);
  assert.equal(result.independent_acceptance, false);
  assert.deepEqual(fixture, before);
  assert.throws(() => assertNoOpenSourceCandidate(fixture.ledger), /cannot contain a rebaseline candidate/);
});

test('candidate evidence object key order does not change its immutable identity', () => {
  const fixture = candidateFixture();
  fixture.lock.build_time_acquisition_inventory.evidence =
    Object.fromEntries(Object.entries(fixture.buildInputs.artifact).reverse());
  assert.equal(validateSourceCandidate(fixture).status, 'unaccepted-candidate-consistent');
});

test('candidate checks authored review subset against the full inventory without granting closure', () => {
  const fixture = candidateFixture();
  const row = fixture.ledger.rows[0];
  fixture.ledger.source_reviews = [{...review, repository: row.source_repository,
    commit: row.source_commit, path: row.source_path, object: row.source_blob_sha}];
  const result = validateSourceCandidate(fixture);
  assert.equal(result.authored_source_review_records, 1);
  assert.equal(result.completed_source_review_records, 1);
  assert.equal(result.source_entries_total, fixture.recursiveInventory.entries.length);
  assert.equal(result.source_closure_ready, false);
  assert.equal(result.baseline_accepted, false);
  assert.equal(result.unknown, null);
  assert.equal(result.unread, null);
  assert.throws(() => validate(fixture.recursiveInventory.entries,
    fixture.ledger.source_reviews, fixture.ledger.rows), /missing source reviews/);
  assert.throws(() => assertNoOpenSourceCandidate(fixture.ledger), /rebaseline candidate/);
});

test('candidate admits schema-compatible pending work without counting it as reviewed', () => {
  const fixture = candidateFixture();
  const row = fixture.ledger.rows[0];
  fixture.ledger.source_reviews = [{...review, repository: row.source_repository,
    commit: row.source_commit, path: row.source_path, object: row.source_blob_sha,
    status: 'pending', review_depth: 'disposition', evidence: [], responsibility_ids: []}];
  const result = validateSourceCandidate(fixture);
  assert.equal(result.authored_source_review_records, 1);
  assert.equal(result.completed_source_review_records, 0);
  assert.equal(result.source_closure_ready, false);
  assert.throws(() => validate(fixture.recursiveInventory.entries,
    fixture.ledger.source_reviews, fixture.ledger.rows), /incomplete or duplicate review evidence/);
  fixture.ledger.source_reviews[0].object = '0'.repeat(40);
  assert.throws(() => validateSourceCandidate(fixture), /unknown, stale or duplicate source review/);
});

for (const [name, patch, reason] of [
  ['stale object', {object: '0'.repeat(40)}, /unknown, stale or duplicate source review/],
  ['missing evidence', {evidence: []}, /incomplete or duplicate review evidence/],
  ['shallow mixed UI', {category: 'mixed-ui', review_depth: 'contract'}, /deep review required/],
  ['unrelated responsibility', {responsibility_ids: ['absent']}, /unmapped responsibility/],
]) {
  test('candidate rejects authored review ' + name, () => {
    const fixture = candidateFixture();
    const row = fixture.ledger.rows[0];
    fixture.ledger.source_reviews = [{...review, repository: row.source_repository,
      commit: row.source_commit, path: row.source_path, object: row.source_blob_sha, ...patch}];
    assert.throws(() => validateSourceCandidate(fixture), reason);
  });
}

test('candidate rejects duplicate or nonarray authored reviews', () => {
  const fixture = candidateFixture();
  const row = fixture.ledger.rows[0];
  const record = {...review, repository: row.source_repository,
    commit: row.source_commit, path: row.source_path, object: row.source_blob_sha};
  fixture.ledger.source_reviews = [record, record];
  assert.throws(() => validateSourceCandidate(fixture), /duplicate source review/);
  fixture.ledger.source_reviews = null;
  assert.throws(() => validateSourceCandidate(fixture), /source reviews and non-empty inventory required/);
});

test('a consistently reduced graph cannot omit the separately required cppgir child', () => {
  const fixture = candidateFixture();
  fixture.recursiveInventory.entries.splice(-2);
  const count = fixture.recursiveInventory.entries.length;
  fixture.recursiveInventory.counts.entries = count;
  fixture.lock.coverage.recursive_source_entries_total = count;
  fixture.index.inventory.recursive_non_directory_entries = count;
  fixture.ledger.coverage.source_entries_total = count;
  fixture.inventoryReport.recursive_non_directory_entries = count;
  fixture.buildInputs.report.git_inventory_counts.source_entries = count;
  fixture.ledger.rebaseline_candidate.nested_gitlinks_discovered = 1;
  fixture.ledger.rebaseline_candidate.nested_gitlinks_resolved = 1;
  const manifest = fixture.acquisitionManifest;
  manifest.canonical_entries_emitted = count;
  manifest.recursive_non_directory_entries = count;
  manifest.repositories.pop();
  manifest.repository_count--;
  manifest.edges.pop();
  manifest.edge_count--;
  assert.throws(() => validateSourceCandidate(fixture), /cppgir expected-lite child proof/);
});

const sourceCandidateFailures = [
  ['missing ledger U', f => { delete f.ledger.upstream_commit; }, /ledger upstream/],
  ['candidate U drift', f => { f.ledger.rebaseline_candidate.upstream_commit = '8'.repeat(40); }, /candidate upstream/],
  ['index tree drift', f => { f.index.upstream.tree = '8'.repeat(40); }, /identity differs/],
  ['missing lock U', f => { delete f.lock.upstream.commit; }, /fixed upstream/],
  ['null candidate marker', f => { f.ledger.rebaseline_candidate = null; }, /strict rebaseline/],
  ['candidate extra acceptance claim', f => { f.ledger.rebaseline_candidate.accepted = true; }, /strict rebaseline/],
  ['candidate accepted status', f => { f.ledger.rebaseline_candidate.status = 'accepted'; }, /candidate upstream\/status/],
  ['candidate missing observation field', f => { delete f.ledger.rebaseline_candidate.cppgir_identity; }, /strict rebaseline/],
  ['true upstream acceptance', f => { f.lock.upstream.accepted = true; }, /explicit false/],
  ['missing false acceptance', f => { delete f.lock.acceptance.baseline_ready; }, /explicit false/],
  ['false semantic completion claim', f => { f.index.inventory.semantic_read_complete = true; }, /explicit false/],
  ['null coerced to zero', f => { f.ledger.coverage.unknown = 0; }, /must all be null/],
  ['old numeric minimum', f => { f.lock.coverage.unread_minimum = 16145; }, /must all be null/],
  ['missing pending counter', f => { delete f.index.inventory.unknown_minimum; }, /must all be null/],
  ['numeric string denominator', f => { f.ledger.coverage.source_entries_total = '6'; }, /recursive denominator/],
  ['claimed denominator drift', f => { f.lock.coverage.recursive_source_entries_total++; }, /recursive denominator/],
  ['actual inventory omission', f => { f.recursiveInventory.entries.pop(); }, /recursive denominator/],
  ['root denominator drift', f => { f.index.inventory.root_non_directory_entries++; }, /root denominator/],
  ['duplicate actual source', f => { f.recursiveInventory.entries.push({...f.recursiveInventory.entries[0]}); }, /duplicate or conflicting/],
  ['conflicting object at same source path', f => {
    f.recursiveInventory.entries.push({...f.recursiveInventory.entries[0], object: '8'.repeat(40)});
  }, /duplicate or conflicting/],
  ['missing root identity in recursive array', f => { f.rootInventory.entries[0].object = '8'.repeat(40); }, /root source missing/],
  ['wrong nested count', f => { f.ledger.rebaseline_candidate.nested_gitlinks_resolved = 0; }, /Git observations/],
  ['cppgir mirror claim', f => { f.ledger.rebaseline_candidate.cppgir_identity.repository = 'https://github.com/mirror/cppgir'; }, /original-host/],
  ['missing manifest', f => { delete f.acquisitionManifest; }, /acquisitionManifest object/],
  ['same-count stale direct commit', f => { f.lock.direct_gitlinks[0].commit = '8'.repeat(40); }, /direct gitlink exact tuple/],
  ['same-count wrong direct repository', f => { f.lock.direct_gitlinks[0].repository = 'mirror/cmake'; }, /direct gitlink exact tuple/],
  ['same-count stale nested parent', f => { f.lock.known_nested_gitlinks[0].parent = 'desktop-app/cmake_helpers@' + '8'.repeat(40); }, /known nested gitlink exact tuple/],
  ['same-count wrong nested path', f => { f.lock.known_nested_gitlinks[0].path = 'external/cppgir'; }, /known nested gitlink exact tuple/],
  ['stale cppgir declaration parent', f => {
    f.ledger.rebaseline_candidate.cppgir_identity.declared_by = 'desktop-app/cmake_helpers@' +
      '8'.repeat(40) + ':.gitmodules#c92fbea3467f72e188cba4cbfe51c8c68a7e2e77';
  }, /cppgir declaration provenance/],
  ['wrong cppgir declaration blob', f => {
    f.ledger.rebaseline_candidate.cppgir_identity.declared_by =
      f.ledger.rebaseline_candidate.cppgir_identity.declared_by.replace(/#[0-9a-f]{40}$/, '#' + '8'.repeat(40));
  }, /cppgir declaration provenance/],
  ['missing cppgir manifest declaration blob', f => {
    delete f.acquisitionManifest.edges[1].gitmodules_blob;
  }, /cppgir declaration provenance/],
  ['missing direct tuple', f => { f.lock.direct_gitlinks = []; }, /direct gitlink cardinality/],
  ['duplicate nested tuple', f => { f.lock.known_nested_gitlinks.push({...f.lock.known_nested_gitlinks[0]}); }, /known nested gitlink cardinality/],
  ['expected-lite incorrectly mixed into first nested pins', f => {
    f.lock.known_nested_gitlinks.push({parent: 'gitlab.com/mnauw/cppgir@' + f.ledger.rebaseline_candidate.cppgir_identity.commit,
      path: 'expected-lite', repository: 'martinmoene/expected-lite', commit: '95b9cb015fa17baa749c2b396b335906e1596a9e'});
  }, /known nested gitlink cardinality/],
  ['missing manifest child edge', f => { f.acquisitionManifest.edges.pop(); }, /graph cardinality/],
  ['manifest parent identity drift', f => { f.acquisitionManifest.edges[1].parent_commit = '8'.repeat(40); }, /edge parent\/child/],
  ['manifest declared repository drift', f => { f.acquisitionManifest.edges[2].declared_url = 'https://github.com/mirror/expected-lite.git'; }, /edge parent\/child/],
  ['missing explicit cppgir child proof', f => { delete f.acquisitionManifest.cppgir_component_proof; }, /cppgir expected-lite/],
  ['child proof commit drift', f => { f.acquisitionManifest.cppgir_component_proof.nested_gitlinks[0].object = '8'.repeat(40); }, /cppgir expected-lite/],
  ['unread child tree', f => { f.acquisitionManifest.repositories[3].tree_enumerated_completely = false; }, /incomplete acquisition repository/],
  ['responsibility count drift', f => { f.ledger.coverage.responsibilities_total = 0; }, /responsibility count/],
  ['duplicate responsibility', f => { f.ledger.rows.push({...f.ledger.rows[0]}); f.ledger.coverage.responsibilities_total++; }, /responsibility id/],
  ['stale responsibility object', f => { f.ledger.rows[0].source_blob_sha = '8'.repeat(40); }, /primary identity/],
  ['unknown additional identity', f => {
    f.ledger.rows[0].additional_source_identities = [{...source, object: '8'.repeat(40)}];
  }, /additional responsibility/],
  ['export target H drift', f => { f.buildInputs.report.target_commit = '8'.repeat(40); }, /export identity/],
  ['export claims source acceptance', f => { f.buildInputs.report.source_closure_ready = true; }, /overclaims/],
  ['export claims source execution', f => { f.buildInputs.report.source_commands_executed = true; }, /read-only/],
  ['opaque source falsely read', f => { f.buildInputs.contexts.unread_inputs[0].source_bytes_read = true; }, /explicitly unread/],
  ['read completion ignores opaque input', f => { f.buildInputs.report.selected_source_read_complete = true; }, /read\/unread accounting/],
  ['duplicate lexical observation', f => { f.buildInputs.candidates.candidates.push({...f.buildInputs.candidates.candidates[0]}); }, /unique pending/],
  ['changed observed source line', f => { f.buildInputs.candidates.candidates[0].source_line++; }, /unique pending/],
  ['classified old occurrence relabelled current', f => { f.buildInputs.candidates.candidates[0].disposition_status = 'classified'; }, /unique pending/],
  ['expanded observations renamed root scope', f => { f.buildInputs.report.candidate_lines_by_scope = {'legacy-root-paths': 2}; }, /count\/scope/],
  ['unresolved refs labelled zero', f => { f.buildInputs.report.unresolved_ref_observations = 0; }, /unresolved reference/],
  ['old deep authority U relabelled current', f => { f.acquisitions.upstream_commit = f.lock.upstream.commit; }, /original upstream/],
  ['current build pointer points at old discovery', f => { f.lock.build_time_acquisition_inventory.json_pointer = '/discovery'; }, /explicit current/],
  ['current candidate count reuses 331', f => { f.acquisitions.rebaseline_candidate.total_candidate_lines = 331; }, /actual export/],
  ['current candidate adds accepted true', f => { f.acquisitions.rebaseline_candidate.accepted = true; }, /overclaims/],
  ['mutable authority U relabelled current', f => { f.mutableAuthorities.accepted_upstream_commit = f.lock.upstream.commit; }, /relabelled/],
  ['tools falsely current-complete', f => { f.toolAuthorities.current_source_status = 'complete'; }, /relabelled/],
];
for (const [name, mutate, message] of sourceCandidateFailures) {
  test('focused source candidate rejects ' + name, () => {
    const fixture = candidateFixture();
    mutate(fixture);
    assert.throws(() => validateSourceCandidate(fixture), message);
  });
}

test('historical classified totals do not supply missing current identities or counters', () => {
  const fixture = candidateFixture();
  delete fixture.acquisitions.rebaseline_candidate;
  assert.throws(() => validateSourceCandidate(fixture), /current build candidate pointer/);
  const second = candidateFixture();
  delete second.ledger.rebaseline_candidate;
  assert.throws(() => validateSourceCandidate(second), /strict rebaseline/);
});


async function recoveredCandidateFixture(t) {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'tdrp-candidate-recovery-'));
  t.after(() => fs.rm(directory, {recursive: true, force: true}));
  const target = 'a'.repeat(40);
  const name = 'fixed-input-export/documents/fixture.json';
  const bytes = Buffer.from('{"fixture":"complete"}\n');
  const archive = Buffer.from('Hermetic archive digest fixture; extraction is owned by the existing recovery helper.\n');
  const hash = value => createHash('sha256').update(value).digest('hex');
  const encode = value => Buffer.from(JSON.stringify(value, null, 2) + '\n');
  const pin = {repository: 'bhrumom/fabushi-desktop', head_sha: 'b'.repeat(40),
    run_id: 11, artifact_id: 12, bytes: archive.length, name: 'fixture-artifact',
    digest: 'sha256:' + hash(archive), files: {[name]: {bytes: bytes.length, sha256: hash(bytes)}}};
  const base = 'https://api.github.com/repos/bhrumom/fabushi-desktop/actions/artifacts/12';
  const metadata = {id: 12, name: pin.name, expired: false, digest: pin.digest,
    size_in_bytes: pin.bytes, url: base, archive_download_url: base + '/zip',
    workflow_run: {id: 11, head_sha: pin.head_sha}};
  const metadataBytes = encode(metadata);
  const index = {
    schema: 'tdrp-protocol-artifact-recovery/v1', complete: true,
    expected: Object.fromEntries(['repository', 'head_sha', 'artifact_id', 'run_id', 'digest'].map(key => [key, pin[key]])),
    recovery_execution: {GITHUB_REPOSITORY: pin.repository, GITHUB_SHA: target},
    archive: {path: 'artifact.zip', complete: true, verified: true, bytes: archive.length, sha256: hash(archive)},
    metadata: {path: 'metadata.json', verified: true, bytes: metadataBytes.length, sha256: hash(metadataBytes)},
    members: [{relative_path: name, member_name: name, raw_path: 'raw/' + name,
      kind: 'file', complete: true, bytes_read: bytes.length, declared_bytes: bytes.length,
      sha256: hash(bytes), digest_scope: 'complete member'}],
    completed_members: 1, uncompressed_bytes_read: bytes.length, declared_uncompressed_bytes: bytes.length,
  };
  await fs.mkdir(path.dirname(path.join(directory, 'raw', name)), {recursive: true});
  await fs.writeFile(path.join(directory, 'raw', name), bytes);
  await fs.writeFile(path.join(directory, 'artifact.zip'), archive);
  await fs.writeFile(path.join(directory, 'metadata.json'), metadataBytes);
  const writeIndex = () => fs.writeFile(path.join(directory, 'index.json'), encode(index));
  await writeIndex();
  return {directory, target, pin, index, metadata, name, hash, encode, writeIndex};
}

test('candidate recovery selects exact indexed members and preserves producer versus new target H', async t => {
  const fixture = await recoveredCandidateFixture(t);
  const result = await readRecoveredCandidateDocuments(fixture.directory, fixture.pin, fixture.target);
  assert.deepEqual(result.documents[fixture.name], {fixture: 'complete'});
  assert.equal(result.artifact.head_sha, fixture.pin.head_sha);
  assert.notEqual(result.artifact.head_sha, fixture.target);
});

test('candidate recovery rejects a nested old recovery execution as the new H', async t => {
  const f = await recoveredCandidateFixture(t);
  f.index.recovery_execution.GITHUB_SHA = f.pin.head_sha;
  await f.writeIndex();
  await assert.rejects(readRecoveredCandidateDocuments(f.directory, f.pin, f.target), /another target H/);
});

test('candidate recovery rejects mismatched artifact identity and incomplete members', async t => {
  const f = await recoveredCandidateFixture(t);
  f.index.expected.artifact_id++;
  await f.writeIndex();
  await assert.rejects(readRecoveredCandidateDocuments(f.directory, f.pin, f.target), /producer identity/);
  f.index.expected.artifact_id--;
  f.index.members[0].complete = false;
  await f.writeIndex();
  await assert.rejects(readRecoveredCandidateDocuments(f.directory, f.pin, f.target), /incomplete recovery member/);
});

test('candidate recovery rehashes complete ZIP bytes', async t => {
  const f = await recoveredCandidateFixture(t);
  await fs.writeFile(path.join(f.directory, 'artifact.zip'), Buffer.alloc(f.pin.bytes, 0));
  await assert.rejects(readRecoveredCandidateDocuments(f.directory, f.pin, f.target), /ZIP size\/digest/);
});

test('candidate recovery rehashes exact member bytes and cannot substitute another prefix', async t => {
  const f = await recoveredCandidateFixture(t);
  await fs.writeFile(path.join(f.directory, 'raw', f.name), Buffer.alloc(f.pin.files[f.name].bytes, 32));
  await assert.rejects(readRecoveredCandidateDocuments(f.directory, f.pin, f.target), /member bytes\/digest/);
  const other = 'old-recovery/' + f.name;
  f.index.members[0].relative_path = other;
  f.index.members[0].member_name = other;
  f.index.members[0].raw_path = 'raw/' + other;
  await f.writeIndex();
  await assert.rejects(readRecoveredCandidateDocuments(f.directory, f.pin, f.target), /exact pinned member missing/);
});

test('candidate recovery rejects duplicate member paths and traversal', async t => {
  const f = await recoveredCandidateFixture(t);
  f.index.members.push({...f.index.members[0]});
  f.index.completed_members = 2;
  await f.writeIndex();
  await assert.rejects(readRecoveredCandidateDocuments(f.directory, f.pin, f.target), /duplicated/);
  f.index.members.pop();
  f.index.completed_members = 1;
  f.index.members[0].relative_path = '../outside';
  f.index.members[0].raw_path = 'raw/../outside';
  await f.writeIndex();
  await assert.rejects(readRecoveredCandidateDocuments(f.directory, f.pin, f.target), /invalid/);
});

test('candidate recovery rejects a metadata producer mismatch even with a matching metadata digest', async t => {
  const f = await recoveredCandidateFixture(t);
  f.metadata.workflow_run.head_sha = 'c'.repeat(40);
  const bytes = f.encode(f.metadata);
  f.index.metadata.bytes = bytes.length;
  f.index.metadata.sha256 = f.hash(bytes);
  await fs.writeFile(path.join(f.directory, 'metadata.json'), bytes);
  await f.writeIndex();
  await assert.rejects(readRecoveredCandidateDocuments(f.directory, f.pin, f.target), /metadata producer/);
});

test('candidate recovery does not follow a selected member symlink', async t => {
  const f = await recoveredCandidateFixture(t);
  const file = path.join(f.directory, 'raw', f.name);
  const outside = path.join(f.directory, 'outside.json');
  await fs.rename(file, outside);
  await fs.symlink(outside, file);
  await assert.rejects(readRecoveredCandidateDocuments(f.directory, f.pin, f.target), /symlink/);
});

test('focused candidate CLI rejects local execution before reading inputs', async t => {
  const before = process.env.GITHUB_ACTIONS;
  t.after(() => { if (before === undefined) delete process.env.GITHUB_ACTIONS; else process.env.GITHUB_ACTIONS = before; });
  process.env.GITHUB_ACTIONS = 'false';
  await assert.rejects(sourceCandidateMain([]), /only in GitHub Actions/);
});
