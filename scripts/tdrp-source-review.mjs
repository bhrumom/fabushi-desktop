// Source accounting only: this does not prove implementation or release acceptance.
export function validateSourceReviews(inventory, reviews, rows) {
  const key = x => JSON.stringify([x.repository, x.commit, x.path, x.object]);
  const expected = new Set(inventory.map(key));
  if (!expected.size || !Array.isArray(reviews)) throw Error('source reviews required');
  const seen = new Set();
  const ids = new Set(rows.map(x => x.responsibility_id));
  const text = x => typeof x === 'string' && x.trim().length > 0;
  for (const r of reviews) {
    const k = key(r);
    if (!expected.has(k) || seen.has(k)) throw Error('unknown, stale or duplicate source review');
    seen.add(k);
    if (!['product','mixed-ui','generated','dependency','resource','documentation','tooling'].includes(r.category)) throw Error('unknown category');
    if (!['low','medium','high'].includes(r.risk)) throw Error('unknown risk');
    if (!['deep','contract','disposition'].includes(r.review_depth)) throw Error('unknown review depth');
    if (r.status !== 'complete' || !text(r.rationale) || !Array.isArray(r.evidence) || !r.evidence.length || !r.evidence.every(text)) throw Error('incomplete review evidence');
    const product = ['product','mixed-ui'].includes(r.category);
    if ((product || r.risk === 'high') && r.review_depth !== 'deep') throw Error('deep review required');
    if (['generated','dependency'].includes(r.category) && r.review_depth === 'disposition') throw Error('contract review required');
    if (!Array.isArray(r.responsibility_ids) || (product && !r.responsibility_ids.length) || r.responsibility_ids.some(id => !ids.has(id))) throw Error('unmapped responsibility');
  }
  if (seen.size !== expected.size) throw Error('missing source reviews');
  return {source_identities: expected.size, reviewed: seen.size};
}
