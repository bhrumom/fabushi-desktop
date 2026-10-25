#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';

const root = process.cwd();
const lock = JSON.parse(await fs.readFile(path.join(root, 'projects/telegram-desktop-rust/upstream.lock.json'), 'utf8'));
const repository = lock.upstream.repository;
const headers = {
  Accept: 'application/vnd.github+json',
  'User-Agent': 'fabushi-tdrp-upstream-delta',
};
if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;

async function get(url) {
  const response = await fetch(url, { headers });
  if (!response.ok) throw new Error(`GET ${url} -> ${response.status}`);
  return response.json();
}

async function treeEntries(commit) {
  const recursive = await get(`https://api.github.com/repos/${repository}/git/trees/${commit}?recursive=1`);
  if (!recursive.truncated) return recursive.tree;

  // GitHub truncates recursive trees above its response limit. Walk each
  // subtree independently so a truncated response can never be mistaken for
  // a complete source baseline.
  const output = [];
  async function walk(treeSha, prefix = '') {
    const tree = await get(`https://api.github.com/repos/${repository}/git/trees/${treeSha}`);
    for (const entry of tree.tree ?? []) {
      const fullPath = prefix ? `${prefix}/${entry.path}` : entry.path;
      if (entry.type === 'tree') await walk(entry.sha, fullPath);
      else output.push({ ...entry, path: fullPath });
    }
  }
  const rootTreeSha = recursive.sha ?? (await get(`https://api.github.com/repos/${repository}/git/commits/${commit}`)).tree?.sha;
  if (!/^[0-9a-f]{40}$/.test(rootTreeSha ?? '')) throw new Error(`truncated tree ${commit} did not expose a root tree SHA`);
  await walk(rootTreeSha);
  return output;
}

const branch = await get(`https://api.github.com/repos/${repository}/branches/dev`);
const candidateCommit = branch.commit?.sha;
if (!/^[0-9a-f]{40}$/.test(candidateCommit ?? '')) throw new Error('dev branch did not return an immutable commit SHA');
const candidateTree = branch.commit?.commit?.tree?.sha;
if (!/^[0-9a-f]{40}$/.test(candidateTree ?? '')) throw new Error('dev branch did not return an immutable tree SHA');

const [baselineEntries, candidateEntries] = await Promise.all([
  treeEntries(lock.upstream.commit),
  treeEntries(candidateCommit),
]);
const baseline = new Map(baselineEntries.filter(entry => entry.type !== 'tree').map(entry => [entry.path, entry]));
const candidate = new Map(candidateEntries.filter(entry => entry.type !== 'tree').map(entry => [entry.path, entry]));
const paths = [...new Set([...baseline.keys(), ...candidate.keys()])].sort();
const changed = [];
for (const file of paths) {
  const before = baseline.get(file);
  const after = candidate.get(file);
  if (!before) changed.push({ path: file, status: 'added', candidate_sha: after.sha, mode: after.mode, type: after.type });
  else if (!after) changed.push({ path: file, status: 'removed', baseline_sha: before.sha, mode: before.mode, type: before.type });
  else if (before.sha !== after.sha || before.mode !== after.mode || before.type !== after.type) {
    changed.push({ path: file, status: 'modified', baseline_sha: before.sha, candidate_sha: after.sha, mode: after.mode, type: after.type });
  }
}

const report = {
  project_id: 'TDRP-001',
  spec_revision: 9,
  repository,
  baseline: { commit: lock.upstream.commit, tree: lock.upstream.tree, non_directory_entries: baseline.size },
  candidate: { ref: 'refs/heads/dev', commit: candidateCommit, tree: candidateTree, non_directory_entries: candidate.size },
  changed_paths: changed,
  counts: {
    added: changed.filter(item => item.status === 'added').length,
    removed: changed.filter(item => item.status === 'removed').length,
    modified: changed.filter(item => item.status === 'modified').length,
    changed_total: changed.length,
  },
  status: candidateCommit === lock.upstream.commit ? 'unchanged' : 'rebaseline-required',
  generated_at: new Date().toISOString(),
};

const outDir = path.join(root, 'artifacts/tdrp-authority');
await fs.mkdir(outDir, { recursive: true });
await fs.writeFile(path.join(outDir, 'upstream-delta.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ status: report.status, baseline: report.baseline, candidate: report.candidate, counts: report.counts }, null, 2));
