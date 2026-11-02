// Source accounting only: this does not prove implementation or release acceptance.
const text = value => typeof value === 'string' && value.trim().length > 0;
const oid = value => typeof value === 'string' && /^[0-9a-f]{40}$/.test(value);
const identityFields = ['repository', 'commit', 'path', 'object'];
const reviewFields = [...identityFields, 'category', 'risk', 'review_depth', 'status', 'rationale', 'evidence', 'responsibility_ids'];

function githubRepository(value) {
  if (typeof value !== 'string' || !/^[A-Za-z0-9][A-Za-z0-9-]*\/[A-Za-z0-9_.-]+$/.test(value)) return false;
  return !['.', '..'].includes(value.split('/')[1]);
}

export async function readFrozenUpstreamCommit(upstream, getJson) {
  if (!upstream || !githubRepository(upstream.repository) ||
      !oid(upstream.commit) || !oid(upstream.tree)) {
    throw Error('invalid frozen upstream repository, commit or tree identity');
  }
  if (typeof getJson !== 'function') throw Error('frozen upstream JSON reader required');
  const {repository, commit: expectedCommit, tree: expectedTree} = upstream;
  const commit = await getJson('https://api.github.com/repos/' + repository +
    '/git/commits/' + expectedCommit);
  if (commit?.sha !== expectedCommit) throw Error('frozen upstream commit mismatch');
  if (commit.tree?.sha !== expectedTree) throw Error('frozen upstream tree mismatch');
  return commit;
}

// Reviewed build-input contract for the one frozen U. This is a local identity
// and consumer guard, not acquisition closure, a source-read claim or acceptance.
export function assertFixedQtSourceContract({upstream, tree, evidence, prepare, docker, snap}) {
  const check = (ok, message) => {
    if (!ok) throw Error('fixed-U Qt source contract: ' + message);
  };
  check(upstream?.repository === 'telegramdesktop/tdesktop' &&
    upstream.commit === 'f985723b552b30fe1c17d377d57f44e61a15a8c4' &&
    upstream.tree === '91f2b083ab2877c0a17d07868eabec362a5cfca6',
  'reviewed upstream identity mismatch');
  check(tree?.truncated === false && tree.sha === upstream.tree && Array.isArray(tree.tree),
    'complete frozen source tree required');
  check(Array.isArray(evidence), 'source evidence array required');
  const required = [
    ['Telegram/build/prepare/prepare.py', 'a9f3dd2d9e4020df958ab5af942376177d66c78b'],
    ['Telegram/build/docker/centos_env/Dockerfile', '3fea9e6866e0ea6351f8a13ebb825ebf49cadfc2'],
    ['snap/snapcraft.yaml', 'a181d828f25d901303f7221824ac30b0ff5e773a'],
  ];
  for (const [sourcePath, blob] of required) {
    const recorded = evidence.filter(item => item?.path === sourcePath);
    check(recorded.length === 1 && recorded[0].blob === blob,
      'missing, duplicate or stale evidence: ' + sourcePath);
    const actual = tree.tree.filter(item => item?.path === sourcePath);
    check(actual.length === 1 && actual[0].type === 'blob' &&
      actual[0].mode === '100644' && actual[0].sha === blob,
    'missing, duplicate or changed source blob: ' + sourcePath);
  }
  check([prepare, docker, snap].every(value => typeof value === 'string'), 'source text required');
  for (const exact of [
    'git submodule update --init --recursive --progress qtbase qtimageformats qtsvg',
    'git submodule update --init --recursive --progress qtbase qtimageformats qtshadertools qtsvg',
  ]) check(prepare.includes(exact), 'prepare Qt submodule selection drift: ' + exact);
  check(docker.includes('qtbase qtdeclarative qtwayland qtimageformats qtsvg qtshadertools'),
    'Docker Qt submodule selection drift');
  for (const module of ['qtbase', 'qtdeclarative', 'qtimageformats', 'qtshadertools', 'qtsvg', 'qtwayland']) {
    check(snap.includes('      - ' + module), 'Snap Qt submodule selection missing ' + module);
  }
  const windowsStage = prepare.match(/stage\('qt6_windows7',\s*"""[\s\S]*?\n"""\)/)?.[0] ?? '';
  check(windowsStage.includes('git clone https://github.com/desktop-app/qt6_windows7_patches.git qt6_windows7'),
    'Windows compatibility acquisition drift');
  check(windowsStage.includes('git checkout 9ce1ffe051161a22f7e78f715b042c773f868e59'),
    'Windows compatibility pin drift');
  check(!prepare.includes('qttools'), 'prepare unexpectedly initializes qttools');
  check(!docker.includes('qttools'), 'Docker unexpectedly initializes qttools');
  check(!snap.includes('      - qttools'), 'Snap unexpectedly initializes qttools');
}

export function assertNoOpenSourceCandidate(ledger) {
  if (!ledger || typeof ledger !== 'object' || Array.isArray(ledger)) {
    throw Error('source ledger object required');
  }
  if (Object.hasOwn(ledger, 'rebaseline_candidate')) {
    throw Error('accepted source accounting cannot contain a rebaseline candidate');
  }
}

// Necessary accepted-mode conditions only; this returns no acceptance verdict.
// Detailed external/generated/LFS/legal evidence remains independently required.
export function assertSourceAcquisitionClosure(lock, acquisition) {
  const check = (ok, message) => {
    if (!ok) throw Error('source acquisition closure: ' + message);
  };
  const object = value => value != null && typeof value === 'object' && !Array.isArray(value);
  check(object(lock) && object(acquisition), 'lock and acquisition objects required');
  const upstream = lock.upstream;
  check(object(upstream) && githubRepository(upstream.repository) &&
    oid(upstream.commit) && oid(upstream.tree), 'frozen upstream identity required');
  const pointer = lock.build_time_acquisition_inventory;
  const discovery = acquisition.discovery;
  const reachability = acquisition.recursive_build_reachability;
  check(object(pointer) && object(discovery) && object(reachability), 'current authority structures required');
  for (const authority of [lock, acquisition, pointer, discovery, reachability]) {
    check(!Object.hasOwn(authority, 'rebaseline_candidate'), 'candidate marker remains');
  }
  check(pointer.path === 'projects/telegram-desktop-rust/inventory/build-time-acquisitions.json' &&
    (!Object.hasOwn(pointer, 'json_pointer') || pointer.json_pointer === ''),
    'acquisition pointer must select the current root authority');
  check(acquisition.upstream_repository === upstream.repository &&
    acquisition.upstream_commit === upstream.commit &&
    reachability.accepted_upstream_commit === upstream.commit,
    'authority differs from frozen upstream identity');
  for (const authority of [acquisition, pointer]) {
    if (Object.hasOwn(authority, 'upstream_tree')) check(authority.upstream_tree === upstream.tree, 'upstream tree drift');
    if (Object.hasOwn(authority, 'upstream_commit')) check(authority.upstream_commit === upstream.commit, 'upstream commit drift');
    if (Object.hasOwn(authority, 'upstream_repository')) check(authority.upstream_repository === upstream.repository, 'upstream repository drift');
  }
  for (const [authority, fields] of [
    [pointer, ['closure_status', 'disposition_closure_status', 'recursive_authority_closure_status']],
    [discovery, ['disposition_closure_status', 'recursive_authority_closure_status', 'evidence_status']],
    [acquisition, ['current_source_status']],
    [reachability, ['status']],
  ]) {
    for (const field of fields) check(authority[field] === 'complete', field + ' must be explicitly complete');
  }
  for (const authority of [pointer, discovery]) {
    for (const field of ['candidate_lines_pending_classification', 'recursive_child_unique_external_nested_scans_pending']) {
      check(Number.isSafeInteger(authority[field]) && authority[field] === 0, field + ' must be integer zero');
    }
  }
  for (const authority of [pointer, acquisition, discovery, reachability, lock.acceptance ?? {}]) {
    for (const field of ['status', 'closure_status', 'disposition_closure_status',
      'recursive_authority_closure_status', 'evidence_status', 'current_source_status']) {
      if (Object.hasOwn(authority, field)) check(authority[field] === 'complete', field + ' remains unresolved');
    }
    for (const field of ['build_time_acquisitions_complete', 'generated_inputs_complete',
      'lfs_object_closure_complete', 'legal_review_performed']) {
      if (Object.hasOwn(authority, field)) check(authority[field] === true, field + ' remains unresolved');
    }
  }
}

function identityKey(source, context) {
  if (!source || typeof source !== 'object' || Array.isArray(source) ||
      !text(source.repository) || !oid(source.commit) ||
      !text(source.path) || !oid(source.object)) {
    throw Error('invalid source identity: ' + context);
  }
  return JSON.stringify(identityFields.map(field => source[field]));
}

function uniqueTextArray(values) {
  return Array.isArray(values) && values.every(text) && new Set(values).size === values.length;
}

function responsibilitySourceRelations(inventory, rows) {
  if (!Array.isArray(inventory) || !inventory.length) throw Error('non-empty inventory required');
  if (!Array.isArray(rows)) throw Error('responsibility rows required');

  const expected = new Set();
  for (const source of inventory) {
    const key = identityKey(source, 'inventory');
    if (expected.has(key)) throw Error('duplicate inventory source identity: ' + key);
    expected.add(key);
  }

  const sourcesByResponsibility = new Map();
  for (const row of rows) {
    if (!row || !text(row.responsibility_id)) throw Error('invalid responsibility id');
    const id = row.responsibility_id;
    if (sourcesByResponsibility.has(id)) throw Error('duplicate responsibility id: ' + id);
    const primary = identityKey({
      repository: row.source_repository,
      commit: row.source_commit,
      path: row.source_path,
      object: row.source_blob_sha,
    }, 'responsibility ' + id);
    if (!expected.has(primary)) throw Error('unknown responsibility source identity: ' + id);
    const related = new Set([primary]);
    const additional = Object.hasOwn(row, 'additional_source_identities')
      ? row.additional_source_identities : [];
    if (!Array.isArray(additional)) throw Error('additional source identities must be an array: ' + id);
    for (const source of additional) {
      const key = identityKey(source, 'additional source for ' + id);
      if (Object.keys(source).some(field => !identityFields.includes(field))) {
        throw Error('unknown additional source identity field: ' + id);
      }
      if (!expected.has(key)) throw Error('unknown additional source identity: ' + id);
      if (related.has(key)) throw Error('duplicate responsibility source identity: ' + id);
      related.add(key);
    }
    sourcesByResponsibility.set(id, related);
  }

  return {expected, sourcesByResponsibility};
}

// The inventory must come from the caller's pinned recursive acquisition, not
// from fetching the identities asserted by a row. Mapping is checked even when
// review coverage is incomplete; these counts grant no acceptance status.
export function validateResponsibilitySources(inventory, rows) {
  const {expected, sourcesByResponsibility} = responsibilitySourceRelations(inventory, rows);
  return {source_identities: expected.size, responsibilities: sourcesByResponsibility.size};
}

function validateSourceReviewRecords(inventory, reviews, rows, allowPending = false) {
  if (!Array.isArray(inventory) || !inventory.length || !Array.isArray(reviews)) {
    throw Error('source reviews and non-empty inventory required');
  }
  const {expected, sourcesByResponsibility} = responsibilitySourceRelations(inventory, rows);

  const seen = new Set();
  let completed = 0;
  for (const review of reviews) {
    const key = identityKey(review, 'review');
    if (Object.keys(review).some(field => !reviewFields.includes(field))) throw Error('unknown source review field');
    if (!expected.has(key) || seen.has(key)) throw Error('unknown, stale or duplicate source review: ' + key);
    seen.add(key);
    if (!['product', 'mixed-ui', 'generated', 'dependency', 'resource', 'documentation', 'tooling'].includes(review.category)) throw Error('unknown category');
    if (!['low', 'medium', 'high'].includes(review.risk)) throw Error('unknown risk');
    if (!['deep', 'contract', 'disposition'].includes(review.review_depth)) throw Error('unknown review depth');
    const pending = allowPending && review.status === 'pending';
    if ((!pending && review.status !== 'complete') || !text(review.rationale) ||
        !uniqueTextArray(review.evidence) || (!pending && !review.evidence.length)) {
      throw Error('incomplete or duplicate review evidence');
    }
    const product = ['product', 'mixed-ui'].includes(review.category);
    if (!pending && (product || review.risk === 'high') && review.review_depth !== 'deep') throw Error('deep review required');
    if (!pending && ['generated', 'dependency'].includes(review.category) && review.review_depth === 'disposition') throw Error('contract review required');
    if (!uniqueTextArray(review.responsibility_ids) || (!pending && product && !review.responsibility_ids.length)) {
      throw Error('missing or duplicate responsibility links');
    }
    for (const id of review.responsibility_ids) {
      const related = sourcesByResponsibility.get(id);
      if (!related) throw Error('unmapped responsibility: ' + id);
      if (!related.has(key)) throw Error('responsibility does not cover source identity: ' + id + ' ' + key);
    }
    if (!pending) completed++;
  }
  return {source_identities: expected.size, reviewed: completed, recorded: seen.size};
}

// Acceptance always requires the complete identity set. Candidate checks below
// reuse the same record/relationship rules without promoting a reviewed subset.
export function validateSourceReviews(inventory, reviews, rows) {
  const result = validateSourceReviewRecords(inventory, reviews, rows);
  if (result.reviewed !== result.source_identities) throw Error('missing source reviews');
  return {source_identities: result.source_identities, reviewed: result.reviewed};
}


// Focused candidate consistency only. Successful return is not source acceptance;
// callers must recover and hash-verify the fixed Git/export artifacts first.
export function validateSourceCandidate({
  lock, index, ledger, acquisitions, mutableAuthorities, toolAuthorities,
  rootInventory, recursiveInventory, inventoryReport, acquisitionManifest, buildInputs,
}) {
  const check = (ok, message) => {
    if (!ok) throw Error('source candidate: ' + message);
  };
  const object = value => value && typeof value === 'object' && !Array.isArray(value);
  const integer = value => Number.isSafeInteger(value) && value >= 0;
  const same = (left, right) => object(left) && object(right) &&
    Object.keys(left).length === Object.keys(right).length &&
    Object.entries(left).every(([key, value]) => Object.hasOwn(right, key) && right[key] === value);
  const falseClaims = [
    'baseline_accepted', 'source_closure_ready', 'build_time_acquisitions_complete',
    'generated_inputs_complete', 'lfs_object_closure_complete', 'semantic_review_performed',
    'legal_review_performed', 'product_accepted', 'independent_acceptance',
  ];
  for (const [name, value] of Object.entries({
    lock, index, ledger, acquisitions, mutableAuthorities, toolAuthorities,
    rootInventory, recursiveInventory, inventoryReport, acquisitionManifest, buildInputs,
  })) check(object(value), name + ' object required');
  for (const document of [lock, index, ledger, acquisitions, mutableAuthorities, toolAuthorities]) {
    check(document.project_id === 'TDRP-001' && document.spec_revision === 9,
      'project/revision mismatch');
  }
  const upstream = lock.upstream;
  check(object(upstream) && githubRepository(upstream.repository) &&
    oid(upstream.commit) && oid(upstream.tree), 'fixed upstream identity required');
  const matchesUpstream = value => object(value) &&
    value.repository === upstream.repository && value.commit === upstream.commit &&
    value.tree === upstream.tree;
  check(matchesUpstream(index.upstream) && matchesUpstream(rootInventory) &&
    matchesUpstream(recursiveInventory.root), 'fixed upstream identity differs between files');
  check(ledger.upstream_commit === upstream.commit && ledger.format_version === 3,
    'ledger upstream/revision mismatch');
  const candidate = ledger.rebaseline_candidate;
  const candidateFields = [
    'upstream_commit', 'status', 'direct_gitlinks_checked', 'nested_gitlinks_discovered',
    'nested_gitlinks_resolved', 'nested_gitlinks_blocked', 'source_responsibility_rows',
    'unknown', 'unread', 'omitted_known', 'cppgir_identity',
  ];
  check(object(candidate) && candidateFields.every(key => Object.hasOwn(candidate, key)) &&
    Object.keys(candidate).every(key => candidateFields.includes(key)),
    'strict rebaseline candidate object required');
  check(candidate.upstream_commit === upstream.commit &&
    candidate.status === 'unaccepted-inventory-open' && text(candidate.source_responsibility_rows),
    'candidate upstream/status mismatch');
  for (const value of [
    upstream.accepted, lock.acceptance?.accepted, lock.acceptance?.baseline_ready,
    index.upstream?.baseline_identity_accepted, index.inventory?.semantic_read_complete,
    index.inventory?.responsibility_decomposition_complete,
  ]) check(value === false, 'candidate must retain explicit false acceptance/read flags');
  for (const value of [
    ledger.coverage?.unknown, ledger.coverage?.unread, candidate.unknown, candidate.unread,
    lock.coverage?.unknown_minimum, lock.coverage?.unread_minimum,
    index.inventory?.unknown_minimum, index.inventory?.unread_minimum,
  ]) check(value === null, 'current candidate review counters must all be null');
  for (const value of [
    ledger.coverage?.omitted, ledger.coverage?.unmapped_responsibilities,
    lock.coverage?.omitted_known, index.inventory?.omitted_known, candidate.omitted_known,
  ]) check(integer(value), 'omitted/unmapped counts must be nonnegative integers');
  check(ledger.coverage.omitted === lock.coverage.omitted_known &&
    ledger.coverage.omitted === index.inventory.omitted_known &&
    ledger.coverage.omitted === candidate.omitted_known, 'known omission counts disagree');

  check(Array.isArray(rootInventory.entries) && rootInventory.entries.length > 0 &&
    Array.isArray(recursiveInventory.entries) && recursiveInventory.entries.length > 0,
    'actual nonempty inventory entries required');
  for (const document of [rootInventory, recursiveInventory]) {
    check(document.project_id === 'TDRP-001' && document.spec_revision === 9 &&
      document.inventory_status === 'unaccepted-git-inventory-candidate' &&
      document.baseline_accepted === false && document.source_closure_ready === false,
      'inventory candidate header mismatch');
  }
  check(matchesUpstream(inventoryReport.root) && inventoryReport.status === 'success' &&
    inventoryReport.canonical_inventory_complete === true &&
    inventoryReport.git_trees_enumerated_completely === true &&
    inventoryReport.root_commit_tree_verified === true,
    'complete fixed Git inventory report required');
  for (const field of falseClaims) check(inventoryReport[field] === false,
    'Git inventory report overclaims ' + field);
  const exact = new Set(), paths = new Set(), rootKeys = new Set();
  let recursiveLinks = 0, rootLinks = 0;
  for (const entry of recursiveInventory.entries) {
    const key = identityKey(entry, 'candidate recursive inventory');
    const location = JSON.stringify([entry.repository, entry.commit, entry.path]);
    check(!exact.has(key) && !paths.has(location), 'duplicate or conflicting inventory source');
    exact.add(key);
    paths.add(location);
    if (entry.type === 'commit' && entry.mode === '160000') recursiveLinks++;
    if (entry.repository === upstream.repository && entry.commit === upstream.commit) {
      rootKeys.add(key);
      if (entry.type === 'commit' && entry.mode === '160000') rootLinks++;
    }
  }
  const declaredRootKeys = new Set();
  for (const entry of rootInventory.entries) {
    const key = identityKey({...entry, repository: upstream.repository, commit: upstream.commit},
      'candidate root inventory');
    check(!declaredRootKeys.has(key), 'duplicate root inventory source');
    declaredRootKeys.add(key);
    check(rootKeys.has(key), 'root source missing from recursive inventory');
  }
  check(rootKeys.size === declaredRootKeys.size, 'root projection differs from recursive inventory');
  const sourceCount = recursiveInventory.entries.length;
  const rootCount = rootInventory.entries.length;
  for (const value of [
    lock.coverage?.recursive_source_entries_total, index.inventory?.recursive_non_directory_entries,
    ledger.coverage?.source_entries_total, recursiveInventory.counts?.entries,
    inventoryReport.recursive_non_directory_entries,
  ]) check(integer(value) && value === sourceCount, 'recursive denominator differs from actual inventory');
  for (const value of [
    lock.coverage?.root_source_entries_total, index.inventory?.root_non_directory_entries,
    lock.observed_root_inventory?.root_non_directory_count, inventoryReport.root_non_directory_entries,
  ]) check(integer(value) && value === rootCount, 'root denominator differs from actual inventory');
  check(lock.observed_root_inventory?.direct_gitlink_count === rootLinks &&
    candidate.direct_gitlinks_checked === rootLinks &&
    candidate.nested_gitlinks_discovered === recursiveLinks - rootLinks &&
    candidate.nested_gitlinks_resolved === candidate.nested_gitlinks_discovered &&
    candidate.nested_gitlinks_blocked === 0,
    'candidate Git observations differ from the complete inventory');
  for (const key of ['direct_gitlinks_checked', 'nested_gitlinks_discovered',
    'nested_gitlinks_resolved', 'nested_gitlinks_blocked']) {
    check(integer(candidate[key]), 'candidate Git count must be a nonnegative integer');
  }
  const cppgir = candidate.cppgir_identity;
  const cppgirFields = ['declared_by', 'repository', 'commit', 'git_tree_verified', 'reason'];
  check(object(cppgir) && cppgirFields.every(key => Object.hasOwn(cppgir, key)) &&
    Object.keys(cppgir).every(key => cppgirFields.includes(key)) &&
    text(cppgir.declared_by) && text(cppgir.reason) &&
    cppgir.repository === 'https://gitlab.com/mnauw/cppgir.git' &&
    oid(cppgir.commit) && cppgir.git_tree_verified === true,
    'candidate cppgir identity must retain verified original-host evidence');
  check(recursiveInventory.entries.some(entry =>
    entry.repository === 'gitlab.com/mnauw/cppgir' && entry.commit === cppgir.commit),
    'candidate cppgir identity absent from actual inventory');

  // Keep the existing lock semantics: direct_gitlinks are root edges, and
  // known_nested_gitlinks are edges immediately inside direct components.
  // The separately proven cppgir child closes the only deeper fixed-U edge.
  const manifest = acquisitionManifest;
  check(matchesUpstream(manifest.root) && manifest.status === 'success' &&
    manifest.format === 'TDRP_FIXED_UPSTREAM_GIT_V1' &&
    manifest.canonical_inventory_complete === true &&
    manifest.git_trees_enumerated_completely === true &&
    manifest.root_commit_tree_verified === true &&
    manifest.canonical_entries_emitted === sourceCount &&
    manifest.recursive_non_directory_entries === sourceCount,
    'complete fixed Git acquisition manifest required');
  for (const field of falseClaims) check(manifest[field] === false,
    'Git acquisition manifest overclaims ' + field);
  check(Array.isArray(manifest.repositories) && Array.isArray(manifest.edges) &&
    manifest.repository_count === manifest.repositories.length &&
    manifest.edge_count === manifest.edges.length && manifest.edges.length === recursiveLinks,
    'acquisition graph cardinality differs from inventory');
  const canonicalRepository = url => {
    const match = typeof url === 'string' &&
      /^https:\/\/(github\.com|gitlab\.com)\/([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+?)(?:\.git)?$/.exec(url);
    check(match && match[2].split('/').every(part => !['.', '..'].includes(part)),
      'unsupported acquisition repository URL');
    return (match[1] === 'gitlab.com' ? 'gitlab.com/' : '') + match[2];
  };
  const nodes = new Map(), nodesByIdentity = new Map();
  for (const node of manifest.repositories) {
    check(object(node) && text(node.repository_id) && !nodes.has(node.repository_id) &&
      oid(node.commit) && oid(node.tree) && integer(node.depth) &&
      node.status === 'success' && node.fetch_verified === true &&
      node.tree_enumerated_completely === true && node.recursive_git_trees_enumerated_completely === true,
      'invalid, duplicate or incomplete acquisition repository');
    const repository = canonicalRepository(node.repository_url);
    const key = JSON.stringify([repository, node.commit]);
    check(!nodesByIdentity.has(key), 'duplicate acquisition repository identity');
    const record = {...node, repository, sourceEntries: 0};
    nodes.set(node.repository_id, record);
    nodesByIdentity.set(key, record);
  }
  const rootNode = nodes.get(manifest.root_repository_id);
  check(rootNode?.repository === upstream.repository && rootNode.commit === upstream.commit &&
    rootNode.tree === upstream.tree && rootNode.depth === 0,
    'acquisition graph root identity mismatch');
  const inventoryLinks = new Map();
  for (const entry of recursiveInventory.entries) {
    const node = nodesByIdentity.get(JSON.stringify([entry.repository, entry.commit]));
    check(node, 'inventory source lacks an acquisition repository');
    node.sourceEntries++;
    if (entry.type === 'commit' && entry.mode === '160000') {
      inventoryLinks.set(JSON.stringify([entry.repository, entry.commit, entry.path]), entry.object);
    }
  }
  for (const node of nodes.values()) check(node.sourceEntries > 0,
    'acquisition repository has no inventory entries');
  const edges = [], edgeIds = new Set(), linkKeys = new Set(), reached = new Set([rootNode.repository_id]);
  for (const edge of manifest.edges) {
    check(object(edge), 'acquisition edge object required');
    const parent = nodes.get(edge.parent_repository_id), child = nodes.get(edge.child_repository_id);
    check(text(edge.edge_id) && !edgeIds.has(edge.edge_id) && parent && child &&
      edge.status === 'attested' && text(edge.declared_path) &&
      edge.parent_commit === parent.commit && edge.parent_tree === parent.tree &&
      canonicalRepository(edge.parent_url) === parent.repository &&
      edge.child_commit === child.commit && edge.child_tree === child.tree &&
      edge.gitlink === child.commit && canonicalRepository(edge.declared_url) === child.repository &&
      edge.child_depth === child.depth && child.depth === parent.depth + 1,
      'acquisition edge parent/child identity mismatch');
    const key = JSON.stringify([parent.repository, parent.commit, edge.declared_path]);
    check(!linkKeys.has(key) && inventoryLinks.get(key) === child.commit,
      'acquisition edge absent from actual gitlink inventory');
    edgeIds.add(edge.edge_id);
    linkKeys.add(key);
    reached.add(child.repository_id);
    edges.push({parent: parent.repository + '@' + parent.commit, path: edge.declared_path,
      repository: child.repository, commit: child.commit, depth: child.depth,
      parent_id: parent.repository_id, child_id: child.repository_id, declared_url: edge.declared_url,
      gitmodules_blob: edge.gitmodules_blob});
  }
  check(linkKeys.size === inventoryLinks.size && reached.size === nodes.size,
    'acquisition graph omits a gitlink or repository');
  const exactPins = (declared, observed, fields, label) => {
    check(Array.isArray(declared) && declared.length === observed.length,
      label + ' cardinality mismatch');
    const expected = new Set(observed.map(edge => JSON.stringify(fields.map(field => edge[field]))));
    const seen = new Set();
    for (const pin of declared) {
      check(object(pin), label + ' pin object required');
      const key = JSON.stringify(fields.map(field => pin[field]));
      check(!seen.has(key) && expected.has(key), label + ' exact tuple mismatch');
      seen.add(key);
    }
    check(seen.size === expected.size, label + ' incomplete exact coverage');
  };
  exactPins(lock.direct_gitlinks, edges.filter(edge => edge.depth === 1),
    ['path', 'repository', 'commit'], 'direct gitlink');
  exactPins(lock.known_nested_gitlinks, edges.filter(edge => edge.depth === 2),
    ['parent', 'path', 'repository', 'commit'], 'known nested gitlink');
  const cppgirParents = edges.filter(edge => edge.depth === 2 &&
    edge.repository === 'gitlab.com/mnauw/cppgir' && edge.commit === cppgir.commit);
  check(cppgirParents.length === 1 && oid(cppgirParents[0].gitmodules_blob) &&
    cppgir.declared_by === cppgirParents[0].parent + ':.gitmodules#' + cppgirParents[0].gitmodules_blob,
    'candidate cppgir declaration provenance differs from its exact manifest edge');
  const deeper = edges.filter(edge => edge.depth > 2), proof = manifest.cppgir_component_proof;
  check(deeper.length === 1 && deeper[0].depth === 3 &&
    deeper[0].parent === 'gitlab.com/mnauw/cppgir@' + cppgir.commit &&
    deeper[0].path === 'expected-lite' && deeper[0].repository === 'martinmoene/expected-lite' &&
    deeper[0].declared_url === 'https://github.com/martinmoene/expected-lite.git' &&
    proof?.child?.repository_url === cppgir.repository && proof.child.commit === cppgir.commit &&
    proof.child.tree === nodes.get(deeper[0].parent_id).tree && proof.child.fetch_verified === true &&
    proof.child_tree_enumerated_completely === true && Array.isArray(proof.nested_gitlinks) &&
    proof.nested_gitlinks.length === 1 && proof.nested_gitlinks[0].path === deeper[0].path &&
    proof.nested_gitlinks[0].object === deeper[0].commit &&
    proof.nested_gitlinks[0].mode === '160000' && proof.nested_gitlinks[0].type === 'commit',
    'separate cppgir expected-lite child proof is missing or differs');
  check(Array.isArray(ledger.rows) && integer(ledger.coverage.responsibilities_total) &&
    ledger.coverage.responsibilities_total === ledger.rows.length, 'responsibility count mismatch');
  const ids = new Set();
  for (const row of ledger.rows) {
    check(object(row) && text(row.responsibility_id) && !ids.has(row.responsibility_id),
      'duplicate or missing responsibility id');
    ids.add(row.responsibility_id);
    const primary = identityKey({repository: row.source_repository, commit: row.source_commit,
      path: row.source_path, object: row.source_blob_sha}, 'candidate responsibility');
    check(exact.has(primary), 'responsibility primary identity absent from current inventory: ' +
      row.responsibility_id + ' ' + primary);
    const related = new Set([primary]);
    const additional = Object.hasOwn(row, 'additional_source_identities') ? row.additional_source_identities : [];
    check(Array.isArray(additional), 'additional responsibility identities must be an array');
    for (const source of additional) {
      const key = identityKey(source, 'candidate additional responsibility');
      check(Object.keys(source).every(field => identityFields.includes(field)) &&
        exact.has(key) && !related.has(key), 'unknown or duplicate additional responsibility identity: ' +
        row.responsibility_id + ' ' + key);
      related.add(key);
    }
  }

  const authoredReviews = Object.hasOwn(ledger, 'source_reviews')
    ? validateSourceReviewRecords(recursiveInventory.entries, ledger.source_reviews, ledger.rows, true)
    : {recorded: 0, reviewed: 0};

  const {report, candidates, contexts, delta, artifact} = buildInputs;
  check(object(artifact) && artifact.repository === 'bhrumom/fabushi-desktop' &&
    oid(artifact.head_sha) && integer(artifact.run_id) && artifact.run_id > 0 &&
    integer(artifact.artifact_id) && artifact.artifact_id > 0 &&
    typeof artifact.digest === 'string' && /^sha256:[0-9a-f]{64}$/.test(artifact.digest),
    'build input artifact identity required');
  for (const document of [report, candidates, contexts, delta]) {
    check(object(document) && document.format === 'TDRP_FIXED_U_BUILD_INPUT_EXPORT_V1' &&
      document.project_id === 'TDRP-001' && document.spec_revision === 9 &&
      document.target_commit === artifact.head_sha && matchesUpstream(document.fixed_upstream) &&
      document.status === 'success' && document.complete === true,
      'build input export identity/completion mismatch');
    for (const field of falseClaims) check(document[field] === false, 'build input export overclaims ' + field);
  }
  check(report.source_commands_executed === false && report.upstream_refs_resolved === false &&
    report.selected_text_source_read_complete === true && candidates.disposition_complete === false,
    'source export must remain read-only and disposition-open');
  check(report.git_inventory_counts?.source_entries === sourceCount &&
    report.git_inventory_counts?.root_non_directory_entries === rootCount,
    'export source denominator differs from actual inventory');
  check(Array.isArray(contexts.source_files) && Array.isArray(contexts.unread_inputs) &&
    Array.isArray(contexts.contexts) && Array.isArray(candidates.candidates) && Array.isArray(delta.records),
    'export observation arrays required');
  const readKeys = new Set(), unreadKeys = new Set();
  for (const source of contexts.source_files) {
    const key = identityKey(source, 'export read source');
    check(exact.has(key) && !readKeys.has(key) && source.utf8_complete === true &&
      source.semantic_review_complete === false, 'read source identity/completion mismatch');
    readKeys.add(key);
  }
  for (const source of contexts.unread_inputs) {
    const key = identityKey(source, 'export unread source');
    check(exact.has(key) && !readKeys.has(key) && !unreadKeys.has(key) &&
      source.disposition === 'registered-opaque-input-unread' &&
      source.source_bytes_read === false && source.lexical_scan_complete === false &&
      source.semantic_review_complete === false && source.legal_review_complete === false,
      'opaque input must remain explicitly unread');
    unreadKeys.add(key);
  }
  check(report.read_source_files === readKeys.size && report.selected_text_source_count === readKeys.size &&
    report.opaque_inputs_not_read === unreadKeys.size &&
    report.selected_source_count === readKeys.size + unreadKeys.size &&
    report.selected_source_read_complete === (unreadKeys.size === 0),
    'export read/unread accounting mismatch');
  const observations = new Map(), scopeCounts = {};
  for (const context of contexts.contexts) {
    const key = identityKey(context, 'export context');
    check(readKeys.has(key) && text(context.observation_id) && !observations.has(context.observation_id) &&
      integer(context.source_line) && context.source_line > 0 && typeof context.raw_line === 'string' &&
      Array.isArray(context.ref_observations), 'invalid or duplicate export context');
    observations.set(context.observation_id, context);
  }
  const candidateIds = new Set();
  for (const observation of candidates.candidates) {
    const key = identityKey(observation, 'export acquisition observation');
    const context = observations.get(observation.observation_id);
    check(readKeys.has(key) && context && !candidateIds.has(observation.observation_id) &&
      identityKey(context, 'matching export context') === key &&
      observation.source_line === context.source_line && observation.raw_line === context.raw_line &&
      observation.scan_scope === context.scan_scope &&
      ['legacy-root-paths', 'explicit-root-entrypoint', 'pinned-cmake-component'].includes(observation.scan_scope) &&
      observation.disposition_status === 'pending-review',
      'candidate observation is not a unique pending source context');
    candidateIds.add(observation.observation_id);
    scopeCounts[observation.scan_scope] = (scopeCounts[observation.scan_scope] || 0) + 1;
  }
  const equalCounts = (left, right) => object(left) && object(right) &&
    Object.keys(left).length === Object.keys(right).length &&
    Object.entries(left).every(([key, value]) => integer(value) && right[key] === value);
  check(candidates.total_candidate_lines === candidateIds.size &&
    report.candidate_lines_total === candidateIds.size &&
    equalCounts(candidates.counts_by_scan_scope, scopeCounts) &&
    equalCounts(report.candidate_lines_by_scope, scopeCounts) &&
    report.context_observations === observations.size, 'lexical observation count/scope mismatch');
  const unresolved = contexts.contexts.reduce((total, context) => total +
    context.ref_observations.filter(ref => ref.literal_commit === null).length, 0);
  check(report.unresolved_ref_observations === unresolved, 'unresolved reference count mismatch');

  const current = acquisitions.rebaseline_candidate;
  check(acquisitions.current_source_status === 'historical-pending-fixed-u-revalidation' &&
    oid(acquisitions.upstream_commit) && acquisitions.upstream_commit !== upstream.commit,
    'historical build authorities must retain their original upstream');
  check(object(current) && current.status === 'unaccepted-build-input-observations' &&
    current.upstream_repository === upstream.repository && current.upstream_commit === upstream.commit &&
    current.upstream_tree === upstream.tree && same(current.evidence, artifact) &&
    current.classification_status === 'pending-source-review',
    'current build candidate pointer/identity mismatch');
  for (const field of falseClaims) check(current[field] === false, 'current build candidate overclaims ' + field);
  for (const field of ['accepted', 'baseline_ready']) {
    if (Object.hasOwn(current, field)) check(current[field] === false,
      'current build candidate overclaims ' + field);
  }
  check(current.total_candidate_lines === candidateIds.size && equalCounts(current.counts_by_scope, scopeCounts) &&
    current.text_files_read === readKeys.size && current.opaque_inputs_unread === unreadKeys.size &&
    current.context_observations === observations.size && current.unresolved_ref_observations === unresolved,
    'current build candidate differs from actual export observations');
  const pointer = lock.build_time_acquisition_inventory;
  check(pointer?.path === 'projects/telegram-desktop-rust/inventory/build-time-acquisitions.json' &&
    pointer.json_pointer === '/rebaseline_candidate' && pointer.upstream_commit === upstream.commit &&
    pointer.status === 'unaccepted-build-input-observations' && same(pointer.evidence, artifact),
    'lock must select the explicit current build candidate');
  for (const document of [mutableAuthorities, toolAuthorities]) {
    check(document.current_source_status === 'historical-pending-fixed-u-revalidation' &&
      oid(document.accepted_upstream_commit) && document.accepted_upstream_commit !== upstream.commit,
      'historical mutable/tool authority was relabelled as current');
    check(document.candidate_evidence?.upstream_commit === upstream.commit &&
      document.candidate_evidence?.path === pointer.path &&
      document.candidate_evidence?.json_pointer === '/rebaseline_candidate' &&
      same(document.candidate_evidence?.evidence, artifact),
      'historical mutable/tool candidate evidence mismatch');
  }
  return {
    status: 'unaccepted-candidate-consistent', upstream_commit: upstream.commit,
    source_entries_total: sourceCount, root_source_entries_total: rootCount,
    responsibility_rows: ledger.rows.length,
    authored_source_review_records: authoredReviews.recorded,
    completed_source_review_records: authoredReviews.reviewed,
    unknown: null, unread: null,
    lexical_observations: candidateIds.size, counts_by_scope: scopeCounts,
    text_files_read: readKeys.size, opaque_inputs_unread: unreadKeys.size,
    unresolved_ref_observations: unresolved,
    baseline_accepted: false, source_closure_ready: false, product_accepted: false,
    independent_acceptance: false,
  };
}
