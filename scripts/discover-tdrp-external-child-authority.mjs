#!/usr/bin/env node
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const fail = (ok, message) => {
  if (!ok) throw new Error(message);
};
const run = (cmd, args, options = {}) => {
  const result = spawnSync(cmd, args, {
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
    ...options,
  });
  if (result.status !== 0) {
    throw new Error(
      `${cmd} ${args.join(' ')} failed with ${result.status}\nstdout:\n${result.stdout || ''}\nstderr:\n${result.stderr || ''}`,
    );
  }
  return result.stdout;
};
const git = (cwd, ...args) => run('git', ['-C', cwd, ...args]);
const tryGit = (cwd, ...args) =>
  spawnSync('git', ['-C', cwd, ...args], {
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  });

const targets = [
  {
    id: 'tg-owt-libyuv',
    repository: 'gitlab.com/chromiumsrc/libyuv',
    clone_url: 'https://gitlab.com/chromiumsrc/libyuv.git',
    commit: '04821d1e7d60845525e8db55c7bcd41ef5be9406',
  },
  {
    id: 'xcb-util-m4',
    repository: 'gitlab.freedesktop.org/xorg/util/xcb-util-m4',
    clone_url: 'https://gitlab.freedesktop.org/xorg/util/xcb-util-m4.git',
    commit: 'c617eee22ae5c285e79e81ec39ce96862fd3262f',
  },
  {
    id: 'crashpad-mini-chromium',
    repository: 'chromium.googlesource.com/chromium/mini_chromium',
    clone_url: 'https://chromium.googlesource.com/chromium/mini_chromium',
    commit: '14b219d5d503e305a6d853e64de201659cfcbe2d',
  },
];

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
  'https://github\\.com/.+\\.git',
  'https://gitlab\\.com/.+\\.git',
  'chromium\\.googlesource\\.com',
  'gitlab\\.freedesktop\\.org',
].join('|');

function parseTree(raw) {
  return raw
    .split(/\r?\n/)
    .filter(Boolean)
    .map((line) => {
      const match = line.match(/^([0-9]{6})\s+(\S+)\s+([0-9a-f]{40})\t(.+)$/);
      fail(match, 'unparseable git ls-tree line: ' + line);
      return { mode: match[1], type: match[2], object: match[3], path: match[4] };
    });
}

function parseGitmodules(raw) {
  const modules = [];
  let current = null;
  for (const line of raw.split(/\r?\n/)) {
    const header = line.match(/^\s*\[submodule\s+"(.+)"\]\s*$/);
    if (header) {
      if (current) modules.push(current);
      current = { name: header[1], path: null, url: null };
      continue;
    }
    if (!current) continue;
    const pathMatch = line.match(/^\s*path\s*=\s*(.+)\s*$/);
    const urlMatch = line.match(/^\s*url\s*=\s*(.+)\s*$/);
    if (pathMatch) current.path = pathMatch[1];
    if (urlMatch) current.url = urlMatch[1];
  }
  if (current) modules.push(current);
  return modules;
}

function selectedPaths(entries, predicate) {
  return entries.filter((entry) => entry.type === 'blob' && predicate(entry.path)).map((entry) => entry.path);
}

const tempRoot = await fs.mkdtemp(path.join(process.env.RUNNER_TEMP || os.tmpdir(), 'tdrp-external-authority-'));
const report = {
  project_id: 'TDRP-001',
  spec_revision: 9,
  target_commit: process.env.GITHUB_SHA || null,
  discovery_kind: 'recursive-external-child-authority',
  targets: [],
};
const candidateLines = [];

for (const target of targets) {
  const targetDir = path.join(tempRoot, target.id);
  run('git', ['clone', '--filter=blob:none', '--no-checkout', '--quiet', target.clone_url, targetDir]);
  git(targetDir, 'checkout', '--detach', '--quiet', target.commit);
  const actualCommit = git(targetDir, 'rev-parse', 'HEAD').trim();
  fail(actualCommit === target.commit, `${target.id} checkout drift: ${actualCommit}`);

  const entries = parseTree(git(targetDir, 'ls-tree', '-r', 'HEAD'));
  const gitlinks = entries.filter((entry) => entry.mode === '160000' || entry.type === 'commit');

  const gitmodulesProbe = tryGit(targetDir, 'show', 'HEAD:.gitmodules');
  const gitmodulesRaw = gitmodulesProbe.status === 0 ? gitmodulesProbe.stdout : '';
  const gitmodules = gitmodulesRaw ? parseGitmodules(gitmodulesRaw) : [];
  for (const link of gitlinks) {
    const module = gitmodules.find((item) => item.path === link.path);
    fail(module?.url, `${target.id} gitlink lacks .gitmodules URL: ${link.path}`);
  }

  const grep = spawnSync(
    'git',
    ['-C', targetDir, 'grep', '-n', '-I', '-E', acquisitionPattern, 'HEAD', '--', '.'],
    { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
  );
  fail(grep.status === 0 || grep.status === 1, `${target.id} acquisition scan failed: ${grep.stderr || ''}`);
  const matches = (grep.stdout || '')
    .split(/\r?\n/)
    .filter(Boolean)
    .sort();
  for (const match of matches) candidateLines.push(`${target.id}:${match}`);

  const attributesProbe = tryGit(targetDir, 'show', 'HEAD:.gitattributes');
  const attributes = attributesProbe.status === 0 ? attributesProbe.stdout : '';
  const lfsAttributeDetected = /filter=lfs|diff=lfs|merge=lfs/.test(attributes);

  const lfsPointerProbe = spawnSync(
    'git',
    ['-C', targetDir, 'grep', '-n', '-I', '-F', 'version https://git-lfs.github.com/spec/v1', 'HEAD', '--', '.'],
    { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
  );
  fail(lfsPointerProbe.status === 0 || lfsPointerProbe.status === 1, `${target.id} LFS pointer scan failed`);
  const lfsPointers = (lfsPointerProbe.stdout || '').split(/\r?\n/).filter(Boolean).sort();

  const patchPaths = selectedPaths(entries, (p) => /\.(patch|diff)$/i.test(p));
  const lockAndPackagePaths = selectedPaths(
    entries,
    (p) =>
      /(^|\/)(Cargo\.lock|package-lock\.json|pnpm-lock\.yaml|yarn\.lock|Pipfile\.lock|poetry\.lock|requirements[^/]*\.txt|vcpkg\.json|conanfile[^/]*|WORKSPACE(?:\.bazel)?|MODULE\.bazel)$/i.test(p),
  );
  const generatedInputPaths = selectedPaths(
    entries,
    (p) =>
      /(^|\/)(generate|generator|generators|gen)(\/|_|\.)/i.test(p) ||
      /\.(gn|gni|gyp|gypi|bzl)$/i.test(p) ||
      /(^|\/)CMakeLists\.txt$/i.test(p),
  );
  const resourcePaths = selectedPaths(
    entries,
    (p) => /(^|\/)(resources?|assets?|locales?|translations?|icons?|testdata)(\/|$)/i.test(p),
  );
  const toolPaths = selectedPaths(
    entries,
    (p) => /(^|\/)(tools?|scripts?|build)(\/|$)/i.test(p) || /(^|\/)(Dockerfile|Makefile)$/i.test(p),
  );

  report.targets.push({
    ...target,
    actual_commit: actualCommit,
    tree_entries_non_directory: entries.length,
    gitlinks: gitlinks.map((link) => ({
      ...link,
      url: gitmodules.find((item) => item.path === link.path)?.url || null,
    })),
    gitmodules,
    acquisition_candidate_count: matches.length,
    acquisition_candidates: matches,
    patches: patchPaths,
    lfs_attribute_detected: lfsAttributeDetected,
    lfs_pointers: lfsPointers,
    lock_and_package_inputs: lockAndPackagePaths,
    generated_input_contracts: generatedInputPaths,
    resource_inputs: resourcePaths,
    tool_and_build_inputs: toolPaths,
  });
}

const artifactDir = path.join(process.cwd(), 'artifacts', 'tdrp-authority');
await fs.mkdir(artifactDir, { recursive: true });
await fs.writeFile(
  path.join(artifactDir, 'external-recursive-child-discovery.json'),
  JSON.stringify(report, null, 2) + '\n',
);
await fs.writeFile(
  path.join(artifactDir, 'external-recursive-child-acquisition-candidates.txt'),
  candidateLines.join('\n') + (candidateLines.length ? '\n' : ''),
);

console.log(JSON.stringify({
  target_commit: report.target_commit,
  targets: report.targets.map((target) => ({
    id: target.id,
    repository: target.repository,
    commit: target.actual_commit,
    entries: target.tree_entries_non_directory,
    gitlinks: target.gitlinks.length,
    acquisition_candidates: target.acquisition_candidate_count,
    patches: target.patches.length,
    lfs_attribute_detected: target.lfs_attribute_detected,
    lfs_pointers: target.lfs_pointers.length,
    lock_and_package_inputs: target.lock_and_package_inputs.length,
    generated_input_contracts: target.generated_input_contracts.length,
    resource_inputs: target.resource_inputs.length,
    tool_and_build_inputs: target.tool_and_build_inputs.length,
  })),
}, null, 2));
