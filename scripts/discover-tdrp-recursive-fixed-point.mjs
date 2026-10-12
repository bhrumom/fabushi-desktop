#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { spawnSync } from 'node:child_process';

const fail = (ok, message) => { if (!ok) throw new Error(message); };
const run = (cmd, args, opts = {}) => {
  const result = spawnSync(cmd, args, { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, ...opts });
  if (result.status !== 0) {
    throw new Error(cmd + ' ' + args.join(' ') + ' failed ' + result.status + '\n' + (result.stderr || ''));
  }
  return result.stdout;
};
const tryRun = (cmd, args, opts = {}) => spawnSync(cmd, args, { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, ...opts });

const normalizeRepo = (value) => value.replace(/\.git$/, '').replace(/\/$/, '');
const resolveUrl = (parent, value) => {
  const parentRepo = normalizeRepo(parent);
  if (value.startsWith('../') || value.startsWith('./')) return normalizeRepo(new URL(value, parentRepo + '/').toString());
  if (value.startsWith('git@github.com:')) return normalizeRepo('https://github.com/' + value.slice('git@github.com:'.length));
  if (/^https?:\/\//.test(value)) return normalizeRepo(value);
  if (/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(value)) return 'https://github.com/' + value;
  return normalizeRepo(value);
};
const cloneUrl = (repo) => normalizeRepo(repo) + (normalizeRepo(repo).includes('github.com/') ? '.git' : '');
const authorityKey = (repo, commit) => normalizeRepo(repo) + '@' + commit;

const readReports = async (dir) => {
  const files = (await fs.readdir(dir, { recursive: true })).filter((file) =>
    /recursive-(root|child|next)-authority-shard-\d+\.json$/.test(file) ||
    file === 'recursive-closure-authorities.json'
  );
  const reports = [];
  for (const file of files) {
    const json = JSON.parse(await fs.readFile(path.join(dir, file), 'utf8'));
    if (file === 'recursive-closure-authorities.json') reports.push(...(json.authorities || []));
    else reports.push(...(json.authorities || []));
  }
  return reports;
};

const scannedDirs = (process.env.SCANNED_REPORT_DIRS ||
  'artifacts/tdrp-layer-root:artifacts/tdrp-layer-child:artifacts/tdrp-layer-next')
  .split(':')
  .filter(Boolean);

const scanned = new Map();
for (const dir of scannedDirs) {
  for (const report of await readReports(dir)) {
    const key = authorityKey(report.repository, report.commit);
    if (scanned.has(key)) {
      throw new Error('duplicate authority across scanned layers: ' + key);
    }
    scanned.set(key, report);
  }
}
fail(scanned.size > 0, 'no scanned recursive authority reports found');

const initialScannedAuthorities = scanned.size;
const occurrences = new Map();
const pending = new Map();

const addOccurrence = (key, occurrence) => {
  const list = occurrences.get(key) || [];
  const signature = JSON.stringify(occurrence);
  if (!list.some((item) => JSON.stringify(item) === signature)) list.push(occurrence);
  occurrences.set(key, list);
};

const enqueueLink = (parentReport, gitlink) => {
  fail(gitlink.url, 'gitlink missing url at ' + parentReport.repository + ' ' + gitlink.path);
  fail(/^[0-9a-f]{40}$/.test(gitlink.object || ''), 'gitlink missing immutable commit at ' + parentReport.repository + ' ' + gitlink.path);
  const repository = resolveUrl(parentReport.repository, gitlink.url);
  const commit = gitlink.object;
  const key = authorityKey(repository, commit);
  const occurrence = {
    parent_repository: parentReport.repository,
    parent_commit: parentReport.commit,
    path: gitlink.path,
    raw_url: gitlink.url,
    branch: gitlink.branch || null
  };
  addOccurrence(key, occurrence);
  if (scanned.has(key)) return;
  if (!pending.has(key)) pending.set(key, { repository, commit });
};

for (const report of scanned.values()) {
  for (const gitlink of report.gitlinks || []) enqueueLink(report, gitlink);
}

const initialUnresolvedChildAuthorities = pending.size;
const tempRoot = await fs.mkdtemp(path.join(process.env.RUNNER_TEMP || os.tmpdir(), 'tdrp-recursive-fixed-point-'));

const acquisitionPattern = [
  'git[[:space:]]+(clone|fetch|submodule)',
  'curl[[:space:]]',
  'wget[[:space:]]',
  '(^|[;&|[:space:]])(iwr|Invoke-WebRequest)[[:space:]]',
  'git\\+https?://',
  '^[[:space:]]*source[[:space:]]*:',
  'source-(url|commit|tag|branch):',
  'GIT_REPOSITORY',
  'GIT_TAG',
  'URL[[:space:]]+https?://',
  'python(3(\\.[0-9]+)*)?[[:space:]]+-m[[:space:]]+pip[[:space:]]+install',
  'pip(3)?[[:space:]]+install',
  'pacman[[:space:]]+-S',
  'apt(-get)?[[:space:]]+(install|update)',
  'dnf[[:space:]][^#]*install',
  'brew[[:space:]]+install',
  'cargo[[:space:]]+install',
  'uses:[[:space:]]',
  '^[[:space:]]*FROM[[:space:]]',
  'https?://[^[:space:]"\\x27]+\\.git'
].join('|');

const parseTree = (raw) => raw.split(/\r?\n/).filter(Boolean).map((line) => {
  const match = line.match(/^([0-9]{6})\s+(\S+)\s+([0-9a-f]{40})\t(.+)$/);
  fail(match, 'unparseable ls-tree line: ' + line);
  return { mode: match[1], type: match[2], object: match[3], path: match[4] };
});

const parseGitmodules = (raw) => {
  const out = [];
  let current = null;
  for (const line of raw.split(/\r?\n/)) {
    const header = line.match(/^\s*\[submodule\s+"(.+)"\]\s*$/);
    if (header) {
      if (current) out.push(current);
      current = { name: header[1], path: null, url: null, branch: null };
      continue;
    }
    if (!current) continue;
    const pathMatch = line.match(/^\s*path\s*=\s*(.+?)\s*$/);
    const urlMatch = line.match(/^\s*url\s*=\s*(.+?)\s*$/);
    const branchMatch = line.match(/^\s*branch\s*=\s*(.+?)\s*$/);
    if (pathMatch) current.path = pathMatch[1];
    if (urlMatch) current.url = urlMatch[1];
    if (branchMatch) current.branch = branchMatch[1];
  }
  if (current) out.push(current);
  return out;
};

const selectedPaths = (entries, regex) =>
  entries.filter((entry) => entry.type === 'blob' && regex.test(entry.path)).map((entry) => entry.path).sort();

const scanAuthority = (authority, ordinal) => {
  const dir = path.join(tempRoot, 'closure-' + String(ordinal).padStart(4, '0'));
  run('git', ['clone', '--filter=blob:none', '--no-checkout', '--quiet', cloneUrl(authority.repository), dir]);
  run('git', ['-C', dir, 'checkout', '--detach', '--quiet', authority.commit]);
  const checkout = run('git', ['-C', dir, 'rev-parse', 'HEAD']).trim();
  fail(checkout === authority.commit, 'checkout drift ' + authority.repository + '@' + authority.commit);

  const entries = parseTree(run('git', ['-C', dir, 'ls-tree', '-r', 'HEAD']));
  const rawGitlinks = entries.filter((entry) => entry.mode === '160000' || entry.type === 'commit');
  const gitmodulesRead = tryRun('git', ['-C', dir, 'show', 'HEAD:.gitmodules']);
  const gitmodules = gitmodulesRead.status === 0 ? parseGitmodules(gitmodulesRead.stdout) : [];
  const gitlinks = rawGitlinks.map((gitlink) => {
    const module = gitmodules.find((item) => item.path === gitlink.path);
    return { ...gitlink, url: module?.url || null, branch: module?.branch || null };
  });
  for (const gitlink of gitlinks) fail(gitlink.url, authority.repository + ' gitlink lacks .gitmodules URL ' + gitlink.path);

  const grep = tryRun('git', ['-C', dir, 'grep', '-n', '-I', '-E', acquisitionPattern, 'HEAD', '--', '.']);
  fail(grep.status === 0 || grep.status === 1, 'acquisition grep failed ' + authority.repository);
  const attr = tryRun('git', ['-C', dir, 'show', 'HEAD:.gitattributes']);
  const lfs = tryRun('git', ['-C', dir, 'grep', '-n', '-I', '-F', 'version https://git-lfs.github.com/spec/v1', 'HEAD', '--', '.']);
  fail(lfs.status === 0 || lfs.status === 1, 'lfs scan failed ' + authority.repository);

  return {
    id: 'CLOSURE-' + String(ordinal).padStart(4, '0'),
    repository: authority.repository,
    commit: authority.commit,
    checkout_commit: checkout,
    occurrences: occurrences.get(authorityKey(authority.repository, authority.commit)) || [],
    tree_entries_non_directory: entries.length,
    gitlinks,
    gitmodules,
    acquisition_candidates: (grep.stdout || '').split(/\r?\n/).filter(Boolean).sort(),
    patches: selectedPaths(entries, /\.(patch|diff)$/i),
    lfs_attribute_detected: /filter=lfs|diff=lfs|merge=lfs/.test(attr.status === 0 ? attr.stdout : ''),
    lfs_pointers: (lfs.stdout || '').split(/\r?\n/).filter(Boolean).sort(),
    generated_input_contracts: selectedPaths(entries, /(^|\/)(generate|generator|generators|gen)(\/|_|\.)|\.(gn|gni|gyp|gypi|bzl)$|(^|\/)CMakeLists\.txt$/i),
    resource_inputs: selectedPaths(entries, /(^|\/)(resources?|assets?|locales?|translations?|icons?|testdata)(\/|$)/i),
    tool_and_package_inputs: selectedPaths(entries, /(^|\/)(tools?|scripts?|build|infra)(\/|$)|(^|\/)(Cargo\.lock|package-lock\.json|pnpm-lock\.yaml|yarn\.lock|Pipfile\.lock|poetry\.lock|requirements[^/]*\.txt|vcpkg\.json|conanfile[^/]*|Dockerfile|Makefile|WORKSPACE(?:\.bazel)?|MODULE\.bazel)$/i),
    recursive_disposition_status: 'open'
  };
};

const newlyScanned = [];
let ordinal = 0;
while (pending.size > 0) {
  const nextKey = [...pending.keys()].sort()[0];
  const authority = pending.get(nextKey);
  pending.delete(nextKey);
  if (scanned.has(nextKey)) continue;

  ordinal += 1;
  const report = scanAuthority(authority, ordinal);
  scanned.set(nextKey, report);
  newlyScanned.push(report);
  console.log(JSON.stringify({
    id: report.id,
    repository: report.repository,
    commit: report.commit,
    entries: report.tree_entries_non_directory,
    gitlinks: report.gitlinks.length,
    acquisitions: report.acquisition_candidates.length
  }));

  for (const gitlink of report.gitlinks) enqueueLink(report, gitlink);
}

const unresolved = [];
for (const report of scanned.values()) {
  for (const gitlink of report.gitlinks || []) {
    fail(gitlink.url, 'gitlink missing url at fixed point ' + report.repository + ' ' + gitlink.path);
    const key = authorityKey(resolveUrl(report.repository, gitlink.url), gitlink.object);
    if (!scanned.has(key)) unresolved.push({
      parent_repository: report.repository,
      parent_commit: report.commit,
      path: gitlink.path,
      child_key: key
    });
  }
}
fail(unresolved.length === 0, 'recursive fixed point still has unresolved gitlink authorities: ' + unresolved.length);

const reduceTotals = (reports) => reports.reduce((totals, report) => {
  totals.authorities += 1;
  totals.tree_entries += report.tree_entries_non_directory;
  totals.gitlinks += (report.gitlinks || []).length;
  totals.acquisition_candidates += (report.acquisition_candidates || []).length;
  totals.patches += (report.patches || []).length;
  totals.lfs_pointers += (report.lfs_pointers || []).length;
  totals.generated_inputs += (report.generated_input_contracts || []).length;
  totals.resource_inputs += (report.resource_inputs || []).length;
  totals.tool_inputs += (report.tool_and_package_inputs || []).length;
  return totals;
}, { authorities: 0, tree_entries: 0, gitlinks: 0, acquisition_candidates: 0, patches: 0, lfs_pointers: 0, generated_inputs: 0, resource_inputs: 0, tool_inputs: 0 });

const allReports = [...scanned.values()];
const outDir = 'artifacts/tdrp-recursive-closure';
await fs.mkdir(outDir, { recursive: true });
await fs.writeFile(path.join(outDir, 'recursive-closure-authorities.json'), JSON.stringify({
  project_id: 'TDRP-001',
  spec_revision: 9,
  target_commit: process.env.GITHUB_SHA,
  authorities: newlyScanned
}, null, 2) + '\n');

const allTotals = reduceTotals(allReports);
const newTotals = reduceTotals(newlyScanned);
await fs.writeFile(path.join(outDir, 'recursive-closure-aggregate.json'), JSON.stringify({
  project_id: 'TDRP-001',
  spec_revision: 9,
  target_commit: process.env.GITHUB_SHA,
  scanned_report_dirs: scannedDirs,
  initial_scanned_authorities: initialScannedAuthorities,
  initial_unresolved_child_authorities: initialUnresolvedChildAuthorities,
  recursively_scanned_new_authorities: newlyScanned.length,
  final_scanned_authorities: scanned.size,
  unresolved_gitlink_authorities: 0,
  gitlink_fixed_point_status: 'closed',
  new_totals: newTotals,
  all_discovered_totals: allTotals,
  recursive_disposition_status: 'open',
  acquisition_disposition_status: 'open-build-reachability-required',
  generated_resource_tool_disposition_status: 'open',
  source_closure_ready: false,
  closure_status: 'gitlink-fixed-point-closed-disposition-pending'
}, null, 2) + '\n');

console.log(JSON.stringify({
  initial_scanned_authorities: initialScannedAuthorities,
  initial_unresolved_child_authorities: initialUnresolvedChildAuthorities,
  recursively_scanned_new_authorities: newlyScanned.length,
  final_scanned_authorities: scanned.size,
  unresolved_gitlink_authorities: 0,
  gitlink_fixed_point_status: 'closed',
  recursive_disposition_status: 'open',
  source_closure_ready: false,
  new_totals: newTotals
}, null, 2));
