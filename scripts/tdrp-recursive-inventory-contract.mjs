import { createHash } from 'node:crypto';

const assert = (ok, message) => {
  if (!ok) throw new Error('G-INVENTORY ' + message);
};
const sha = value => typeof value === 'string' && /^[0-9a-f]{40}$/.test(value);
const key = row => [row.scope, row.repository, row.commit, row.parent || '', row.mount || '', row.path].join('\0');
const componentKey = row => [row.parent || '', row.path || row.mount, row.repository, row.commit].join('\0');

const sourceDisplayPath = row => row.scope === 'root'
  ? row.path
  : (row.mount + '::' + row.path);
const sameOptional = (a, b) => (a ?? null) === (b ?? null);

export function checkSourceDispositionPrefix({ rows, prefix, entries }) {
  assert(Number.isInteger(prefix?.first_order) && prefix.first_order === 1, 'source-dispositions prefix must start at order 1');
  assert(Number.isInteger(prefix?.last_order) && prefix.last_order >= 0, 'source-dispositions prefix last order missing');
  assert(prefix.entries === prefix.last_order, 'source-dispositions prefix entry count drift');
  assert(Array.isArray(rows), 'source-dispositions rows must be an array');
  assert(rows.length === prefix.entries, 'source-dispositions row count differs from deterministic prefix');
  assert(prefix.last_order <= entries.length, 'source-dispositions prefix exceeds recursive inventory');
  const orders = new Set();
  for (const row of rows) {
    assert(Number.isInteger(row.recursive_order)
      && row.recursive_order >= 1
      && row.recursive_order <= prefix.last_order,
    'source-dispositions row order outside deterministic prefix: ' + row.recursive_order);
    assert(!orders.has(row.recursive_order), 'duplicate source-dispositions recursive order: ' + row.recursive_order);
    orders.add(row.recursive_order);
    const expected = entries[row.recursive_order - 1];
    assert(expected, 'missing recursive source identity at order ' + row.recursive_order);
    assert(row.source_path === sourceDisplayPath(expected),
      'source-dispositions deterministic path drift at order ' + row.recursive_order
      + ': expected ' + sourceDisplayPath(expected) + ', recorded ' + row.source_path);
    assert(row.source_blob_sha === expected.object,
      'source-dispositions deterministic blob drift at order ' + row.recursive_order + ': ' + row.source_path);
    if (expected.scope !== 'root') {
      const identity = row.source_identity;
      assert(identity && typeof identity === 'object',
        'component source-disposition missing source_identity at order ' + row.recursive_order);
      for (const field of ['repository','commit','scope','path','object','mode','type']) {
        assert(identity[field] === expected[field],
          'component source-disposition ' + field + ' drift at order ' + row.recursive_order);
      }
      for (const field of ['parent','mount','size']) {
        assert(sameOptional(identity[field], expected[field]),
          'component source-disposition ' + field + ' drift at order ' + row.recursive_order);
      }
    } else if (row.source_identity) {
      const identity = row.source_identity;
      for (const field of ['repository','commit','scope','path','object','mode','type']) {
        assert(identity[field] === expected[field],
          'root source-disposition ' + field + ' drift at order ' + row.recursive_order);
      }
      for (const field of ['parent','mount','size']) {
        assert(sameOptional(identity[field], expected[field]),
          'root source-disposition ' + field + ' drift at order ' + row.recursive_order);
      }
    }
    assert(row.read_complete === true && row.responsibility_decomposition_complete === true,
      'source-dispositions prefix contains unread/incomplete row at order ' + row.recursive_order);
  }
  for (let order = 1; order <= prefix.last_order; order++) {
    assert(orders.has(order), 'source-dispositions deterministic prefix has a gap at order ' + order);
  }
  return { read_through: prefix.last_order, orders };
}

/**
 * Compare the live, recursively enumerated Git objects against all three
 * durable TDRP stores. Only GitHub Actions may execute this verification.
 * The upstream commit and every gitlink pin are immutable source identities;
 * every record retains its own path, mode and blob/gitlink object SHA.
 */
export function checkRecursiveInventory({
  lock, inventoryIndex, ledger, entries, directComponents,
  githubNestedComponents, cppgirTreeEntries, cppgirNonDirectoryEntries,
  cppgirChild
}) {
  const observed = lock.observed_recursive_inventory;
  assert(observed && Array.isArray(observed.direct_component_counts), 'missing durable direct component census');
  assert(Array.isArray(observed.github_nested_component_counts), 'missing durable nested component census');
  assert(Array.isArray(entries) && entries.length > 0, 'missing source entries');
  const identity = new Set();
  const counts = { root: 0, direct: 0, githubNested: 0, gitlab: 0, cppgirChild: 0 };
  const pinnedDirect = new Map(lock.direct_gitlinks.map(p => [p.path, p]));
  const pinnedNested = new Map(lock.known_nested_gitlinks.map(p => [p.parent + '\0' + p.path, p]));
  assert(pinnedDirect.size === lock.direct_gitlinks.length, 'duplicate direct gitlink pins');
  assert(pinnedNested.size === lock.known_nested_gitlinks.length, 'duplicate nested gitlink pins');
  assert(observed.direct_component_counts.length === pinnedDirect.size, 'missing direct component counts');
  assert(observed.github_nested_component_counts.length === lock.known_nested_gitlinks.filter(p => !p.repository.startsWith('gitlab.com/')).length, 'missing GitHub nested component counts');
  const directActual = new Map(directComponents.map(p => [p.mount, p]));
  const directPinned = new Map(observed.direct_component_counts.map(p => [p.mount, p]));
  const nestedActual = new Map(githubNestedComponents.map(p => [p.parent + '\0' + p.path, p]));
  const nestedPinned = new Map(observed.github_nested_component_counts.map(p => [p.parent + '\0' + p.path, p]));
  assert(directActual.size === directComponents.length && directPinned.size === observed.direct_component_counts.length, 'duplicate direct component mounts');
  assert(nestedActual.size === githubNestedComponents.length && nestedPinned.size === observed.github_nested_component_counts.length, 'duplicate nested component mounts');
  assert(directActual.size === pinnedDirect.size, 'actual direct component cardinality drift');
  assert(nestedActual.size === observed.github_nested_component_counts.length, 'actual nested component cardinality drift');
  for (const [mount, pin] of pinnedDirect) {
    const a = directActual.get(mount);
    const e = directPinned.get(mount);
    assert(a && e && a.repository === pin.repository && e.repository === pin.repository && a.commit === pin.commit && e.commit === pin.commit && a.entries === e.entries, 'direct component identity/count mismatch: ' + mount);
  }
  for (const [mount, e] of nestedPinned) {
    const a = nestedActual.get(mount);
    const pin = pinnedNested.get(mount);
    assert(a && pin && a.repository === pin.repository && e.repository === pin.repository && a.commit === pin.commit && e.commit === pin.commit && a.entries === e.entries, 'nested component identity/count mismatch: ' + mount);
  }
  const gitlabPin = lock.known_nested_gitlinks.find(p => p.repository === 'gitlab.com/mnauw/cppgir');
  assert(gitlabPin && observed.gitlab_cppgir.repository === gitlabPin.repository && observed.gitlab_cppgir.commit === gitlabPin.commit, 'cppgir pin mismatch');
  assert(cppgirTreeEntries === observed.gitlab_cppgir.tree_entries && cppgirNonDirectoryEntries === observed.gitlab_cppgir.non_directory_entries, 'cppgir tree/non-directory count drift');
  assert(cppgirChild.path === observed.cppgir_child.path && cppgirChild.repository === observed.cppgir_child.repository && cppgirChild.commit === observed.cppgir_child.commit && cppgirChild.entries === observed.cppgir_child.entries, 'cppgir child identity/count drift');
  const scopes = new Set(['root', 'direct-gitlink', 'nested-gitlink', 'nested-gitlink-child']);
  const rootLinks = new Map();
  const childLinks = new Map();
  const seenBySource = new Map();
  for (const e of entries) {
    assert(scopes.has(e.scope), 'unknown source scope: ' + e.scope);
    assert(sha(e.commit) && sha(e.object) && typeof e.path === 'string' && e.path.length > 0 && !e.path.startsWith('/') && !e.path.split('/').includes('..'), 'invalid path/blob identity');
    assert(e.type === 'blob' || e.type === 'commit', 'non-file object in source inventory');
    assert(e.type !== 'commit' || e.mode === '160000', 'gitlink mode mismatch');
    const identityKey = key(e);
    assert(!identity.has(identityKey), 'duplicate source path identity: ' + identityKey);
    identity.add(identityKey);
    if (e.scope === 'root') {
      counts.root++;
      assert(e.repository === lock.upstream.repository && e.commit === lock.upstream.commit, 'root source identity drift');
      if (e.mode === '160000') rootLinks.set(e.path, e.object);
    } else if (e.scope === 'direct-gitlink') {
      counts.direct++;
      const pin = pinnedDirect.get(e.mount);
      assert(pin && e.repository === pin.repository && e.commit === pin.commit, 'direct source provenance drift');
      if (e.mode === '160000') childLinks.set(e.repository + '@' + e.commit + '\0' + e.path, e.object);
    } else if (e.scope === 'nested-gitlink') {
      const pin = pinnedNested.get(e.parent + '\0' + e.mount);
      assert(pin && e.repository === pin.repository && e.commit === pin.commit, 'nested source provenance drift');
      if (e.repository === 'gitlab.com/mnauw/cppgir') {
        counts.gitlab++;
        if (e.mode === '160000') childLinks.set(e.repository + '@' + e.commit + '\0' + e.path, e.object);
      } else counts.githubNested++;
    } else {
      counts.cppgirChild++;
      assert(e.repository === cppgirChild.repository && e.commit === cppgirChild.commit && e.mount === cppgirChild.path && e.parent === gitlabPin.repository + '@' + gitlabPin.commit, 'cppgir child provenance drift');
    }
    const sourceGroup = [e.scope, e.parent || '', e.mount || ''].join('\0');
    seenBySource.set(sourceGroup, (seenBySource.get(sourceGroup) || 0) + 1);
  }
  assert(rootLinks.size === pinnedDirect.size, 'root gitlink path set drift');
  for (const [path, pin] of pinnedDirect) assert(rootLinks.get(path) === pin.commit, 'root gitlink blob SHA drift: ' + path);
  for (const pin of lock.known_nested_gitlinks) {
    assert(childLinks.get(pin.parent + '\0' + pin.path) === pin.commit, 'nested gitlink path/blob SHA drift: ' + pin.parent + '/' + pin.path);
  }
  assert(childLinks.get(gitlabPin.repository + '@' + gitlabPin.commit + '\0' + cppgirChild.path) === cppgirChild.commit, 'cppgir child gitlink SHA drift');
  assert(counts.root === lock.observed_root_inventory.root_non_directory_count && counts.root === observed.root_non_directory_entries && counts.root === inventoryIndex.inventory.root_non_directory_entries && counts.root === lock.coverage.root_source_entries_total, 'root count disagrees with persisted census');
  const actualCount = (scope, parent, mount) => seenBySource.get([scope, parent || '', mount || ''].join('\0')) || 0;
  for (const p of directComponents) assert(actualCount('direct-gitlink', '', p.mount) === p.entries, 'direct path-set count drift: ' + p.mount);
  for (const p of githubNestedComponents) assert(actualCount('nested-gitlink', p.parent, p.path) === p.entries, 'nested path-set count drift: ' + p.parent + '/' + p.path);
  assert(counts.gitlab === cppgirNonDirectoryEntries && counts.cppgirChild === cppgirChild.entries, 'GitLab child path-set count drift');
  assert(counts.direct === directComponents.reduce((n,p) => n + p.entries, 0), 'direct source group sum drift');
  assert(counts.githubNested === githubNestedComponents.reduce((n,p) => n + p.entries, 0), 'GitHub nested source group sum drift');
  const total = counts.root + counts.direct + counts.githubNested + counts.gitlab + counts.cppgirChild;
  assert(total === entries.length && total === observed.total_recursive_non_directory_entries, 'actual recursive total does not match component census');
  assert(total === lock.coverage.recursive_source_entries_total && total === inventoryIndex.inventory.recursive_non_directory_entries && total === ledger.coverage.source_entries_total, 'actual recursive total differs from lock/inventory/ledger');
  for (const [a,b,c,d,label] of [
    [lock.coverage.unknown_minimum,inventoryIndex.inventory.unknown_minimum,ledger.coverage.unknown,total,'unknown'],
    [lock.coverage.unread_minimum,inventoryIndex.inventory.unread_minimum,ledger.coverage.unread,total,'unread'],
    [lock.coverage.omitted_known,inventoryIndex.inventory.omitted_known,ledger.coverage.omitted,total,'omitted']
  ]) assert(Number.isInteger(a) && a >= 0 && a <= d && a === b && a === c, label + ' three-store accounting drift');
  const formattedCount = value => value.toLocaleString('en-US');
  const narrative = inventoryIndex.note;
  assert(typeof narrative === 'string' && narrative.includes('unread_minimum is ' + formattedCount(inventoryIndex.inventory.unread_minimum)), 'inventory narrative unread count drift');
  assert(typeof narrative === 'string' && narrative.includes('unknown remains ' + formattedCount(inventoryIndex.inventory.unknown_minimum)), 'inventory narrative unknown count drift');
  assert(typeof narrative === 'string' && narrative.includes('omitted remains ' + formattedCount(inventoryIndex.inventory.omitted_known)), 'inventory narrative omitted count drift');
  const inventoryNarrative = inventoryIndex.inventory.note;
  assert(typeof inventoryNarrative === 'string' && inventoryNarrative.includes('unknown ' + formattedCount(inventoryIndex.inventory.unknown_minimum)), 'inventory detail narrative unknown count drift');
  assert(typeof inventoryNarrative === 'string' && inventoryNarrative.includes('unread ' + formattedCount(inventoryIndex.inventory.unread_minimum)), 'inventory detail narrative unread count drift');
  assert(typeof inventoryNarrative === 'string' && inventoryNarrative.includes('omitted ' + formattedCount(inventoryIndex.inventory.omitted_known)), 'inventory detail narrative omitted count drift');
  const rows = [...entries].sort((a,b) => key(a).localeCompare(key(b))).map(e => [key(e),e.mode,e.type,e.object].join('\0'));
  return { total, counts, identity_sha256: createHash('sha256').update(rows.join('\n')+'\n').digest('hex') };
}
