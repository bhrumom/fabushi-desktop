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

export function assertNoOpenSourceCandidate(ledger) {
  if (!ledger || typeof ledger !== 'object' || Array.isArray(ledger)) {
    throw Error('source ledger object required');
  }
  if (Object.hasOwn(ledger, 'rebaseline_candidate')) {
    throw Error('accepted source accounting cannot contain a rebaseline candidate');
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

export function validateSourceReviews(inventory, reviews, rows) {
  if (!Array.isArray(inventory) || !inventory.length || !Array.isArray(reviews)) {
    throw Error('source reviews and non-empty inventory required');
  }
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

  const seen = new Set();
  for (const review of reviews) {
    const key = identityKey(review, 'review');
    if (Object.keys(review).some(field => !reviewFields.includes(field))) throw Error('unknown source review field');
    if (!expected.has(key) || seen.has(key)) throw Error('unknown, stale or duplicate source review: ' + key);
    seen.add(key);
    if (!['product', 'mixed-ui', 'generated', 'dependency', 'resource', 'documentation', 'tooling'].includes(review.category)) throw Error('unknown category');
    if (!['low', 'medium', 'high'].includes(review.risk)) throw Error('unknown risk');
    if (!['deep', 'contract', 'disposition'].includes(review.review_depth)) throw Error('unknown review depth');
    if (review.status !== 'complete' || !text(review.rationale) ||
        !uniqueTextArray(review.evidence) || !review.evidence.length) {
      throw Error('incomplete or duplicate review evidence');
    }
    const product = ['product', 'mixed-ui'].includes(review.category);
    if ((product || review.risk === 'high') && review.review_depth !== 'deep') throw Error('deep review required');
    if (['generated', 'dependency'].includes(review.category) && review.review_depth === 'disposition') throw Error('contract review required');
    if (!uniqueTextArray(review.responsibility_ids) || (product && !review.responsibility_ids.length)) {
      throw Error('missing or duplicate responsibility links');
    }
    for (const id of review.responsibility_ids) {
      const related = sourcesByResponsibility.get(id);
      if (!related) throw Error('unmapped responsibility: ' + id);
      if (!related.has(key)) throw Error('responsibility does not cover source identity: ' + id + ' ' + key);
    }
  }
  if (seen.size !== expected.size) throw Error('missing source reviews');
  return {source_identities: expected.size, reviewed: seen.size};
}
