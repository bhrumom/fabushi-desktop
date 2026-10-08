import test from 'node:test';
import assert from 'node:assert/strict';
import { checkRecursiveInventory } from './tdrp-recursive-inventory-contract.mjs';

const s = digit => digit.repeat(40);
function fixture() {
  const root = 'upstream/root', a = 'vendor/a', nested = 'vendor/nested', cpp = 'gitlab.com/mnauw/cppgir';
  const parent = a + '@' + s('2'), cppParent = cpp + '@' + s('4');
  const lock = {
    upstream: { repository: root, commit: s('1') },
    observed_root_inventory: { root_non_directory_count: 1 },
    direct_gitlinks: [{ path: 'vendor/a', repository: a, commit: s('2') }],
    known_nested_gitlinks: [
      { parent, path: 'docs', repository: nested, commit: s('3') },
      { parent, path: 'cppgir', repository: cpp, commit: s('4') }
    ],
    coverage: { root_source_entries_total: 1, recursive_source_entries_total: 6, unknown_minimum: 6, unread_minimum: 5, omitted_known: 0 },
    observed_recursive_inventory: {
      root_non_directory_entries: 1, direct_component_counts: [{ mount:'vendor/a',repository:a,commit:s('2'),entries:2 }],
      github_nested_component_counts: [{ parent,path:'docs',repository:nested,commit:s('3'),entries:1 }],
      gitlab_cppgir: { repository:cpp,commit:s('4'),tree_entries:1,non_directory_entries:1 },
      cppgir_child: { path:'expected-lite',repository:'vendor/child',commit:s('5'),entries:1 },
      total_recursive_non_directory_entries:6
    }
  };
  const entries = [
    { scope:'root',repository:root,commit:s('1'),path:'vendor/a',mode:'160000',type:'commit',object:s('2') },
    { scope:'direct-gitlink',repository:a,commit:s('2'),mount:'vendor/a',path:'docs',mode:'160000',type:'commit',object:s('3') },
    { scope:'direct-gitlink',repository:a,commit:s('2'),mount:'vendor/a',path:'cppgir',mode:'160000',type:'commit',object:s('4') },
    { scope:'nested-gitlink',repository:nested,commit:s('3'),parent,path:'README',mount:'docs',mode:'100644',type:'blob',object:s('6') },
    { scope:'nested-gitlink',repository:cpp,commit:s('4'),parent,mount:'cppgir',path:'expected-lite',mode:'160000',type:'commit',object:s('5') },
    { scope:'nested-gitlink-child',repository:'vendor/child',commit:s('5'),parent:cppParent,mount:'expected-lite',path:'README',mode:'100644',type:'blob',object:s('7') }
  ];
  return {
    lock,
    inventoryIndex: {inventory:{root_non_directory_entries:1,recursive_non_directory_entries:6,unknown_minimum:6,unread_minimum:5,omitted_known:0}},
    ledger: {coverage:{source_entries_total:6,unknown:6,unread:5,omitted:0}},
    entries,
    directComponents:lock.observed_recursive_inventory.direct_component_counts,
    githubNestedComponents:lock.observed_recursive_inventory.github_nested_component_counts,
    cppgirTreeEntries:1,
    cppgirNonDirectoryEntries:1,
    cppgirChild:lock.observed_recursive_inventory.cppgir_child
  };
}
test('the full root, direct, GitHub nested and GitLab grandchild inventory is accepted', () => {
  const result = checkRecursiveInventory(fixture());
  assert.equal(result.total,6);
  assert.equal(result.counts.cppgirChild,1);
  assert.match(result.identity_sha256,/^[0-9a-f]{64}$/);
});
test('a stale denominator cannot pass ordinary validation', () => {
  const f=fixture();
  f.lock.coverage.recursive_source_entries_total=5;
  f.inventoryIndex.inventory.recursive_non_directory_entries=5;
  f.ledger.coverage.source_entries_total=5;
  assert.throws(() => checkRecursiveInventory(f),/actual recursive total/);
});
test('an xxHash-style per-component addition cannot pass stale component pins', () => {
  const f=fixture();
  f.entries.push({scope:'direct-gitlink',repository:'vendor/a',commit:s('2'),mount:'vendor/a',path:'new-file',mode:'100644',type:'blob',object:s('8')});
  assert.throws(() => checkRecursiveInventory(f),/actual recursive total|direct path-set count drift|direct source group sum drift/);
});
test('gitlink SHA changes or a missing nested child fail closed', () => {
  const f=fixture();
  f.entries[2].object=s('9');
  assert.throws(() => checkRecursiveInventory(f),/nested gitlink path\/blob SHA drift/);
  const g=fixture();
  g.entries.pop();
  assert.throws(() => checkRecursiveInventory(g),/GitLab child path-set count drift/);
});
test('duplicate paths and invalid blob identities fail closed', () => {
  const f=fixture();f.entries.push({...f.entries[3]});
  assert.throws(() => checkRecursiveInventory(f),/duplicate source path identity/);
  const g=fixture();g.entries[3].object='unverified';
  assert.throws(() => checkRecursiveInventory(g),/invalid path\/blob identity/);
});
test('the three persisted accounting stores must agree without pretending unread is zero', () => {
  const f=fixture();f.inventoryIndex.inventory.unread_minimum=4;
  assert.throws(() => checkRecursiveInventory(f),/unread three-store accounting drift/);
});
