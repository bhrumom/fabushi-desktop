import { test } from 'node:test';
import assert from 'node:assert/strict';
import { canonicalNestedRepository, assertNestedGitlinkPins } from './tdrp-nested-pin-identity.mjs';

const sha = c => c.repeat(40);
const github = {parent: 'owner/direct@' + sha('a'), path: 'docs/theme', repository: 'owner/theme', commit: sha('b')};
const gitlab = {parent: 'desktop-app/cmake_helpers@' + sha('c'), path: 'external/glib/cppgir', repository: 'gitlab.com/mnauw/cppgir', commit: sha('d')};
const known = [github, gitlab];

test('nested identity matches GitHub and GitLab mounts bidirectionally', () => {
  assert.deepEqual(assertNestedGitlinkPins(known, [...known]), {locked: 2, observed: 2});
});
test('strict GitHub and GitLab submodule URL identities', () => {
  assert.equal(canonicalNestedRepository('https://github.com/owner/theme.git'), 'owner/theme');
  assert.equal(canonicalNestedRepository('https://gitlab.com/mnauw/cppgir.git'), 'gitlab.com/mnauw/cppgir');
  assert.equal(canonicalNestedRepository('git@github.com:owner/theme.git'), 'owner/theme');
  assert.equal(canonicalNestedRepository('ssh://git@gitlab.com/mnauw/cppgir.git'), 'gitlab.com/mnauw/cppgir');
  for (const url of ['https://github.com.evil.example/owner/theme.git', 'file:///tmp/cppgir', '../cppgir', 'https://gitlab.com/mnauw/extra/folder']) {
    assert.equal(canonicalNestedRepository(url), null);
  }
});
for (const [attribute, value] of Object.entries({
  parent: 'other/parent@' + sha('c'),
  path: 'external/glib/other',
  repository: 'gitlab.com/other/cppgir',
  commit: sha('e'),
})) {
  test('reject GitLab nested pin ' + attribute + ' mismatch', () => {
    const altered = {...gitlab, [attribute]: value};
    assert.throws(() => assertNestedGitlinkPins(known, [github, altered]));
  });
}
test('reject extra GitLab nested pin concealed alongside the expected pin', () => {
  assert.throws(() => assertNestedGitlinkPins(known, [
    ...known, {...gitlab, path: 'external/glib/extra', commit: sha('f')}
  ]), /unexpected nested gitlink/);
});
test('reject omitted GitLab nested pin', () => {
  assert.throws(() => assertNestedGitlinkPins(known, [github]), /missing nested gitlink/);
});
test('reject duplicate locked and observed mounts', () => {
  assert.throws(() => assertNestedGitlinkPins([...known, gitlab], known), /duplicate nested gitlink/);
  assert.throws(() => assertNestedGitlinkPins(known, [...known, gitlab]), /duplicate nested gitlink/);
});
test('reject malformed and unpinned nested identities', () => {
  assert.throws(() => assertNestedGitlinkPins(known, [github, {...gitlab, commit: 'floating'}]), /invalid nested gitlink/);
  assert.throws(() => assertNestedGitlinkPins(known, undefined), /arrays are required/);
});
