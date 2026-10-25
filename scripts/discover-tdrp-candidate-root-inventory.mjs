#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
const candidate = JSON.parse(await fs.readFile(
  path.join(root, 'projects/telegram-desktop-rust/upstream.candidate.json'),
  'utf8',
));
const fail = (ok, message) => { if (!ok) throw new Error(message); };
const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, {
    encoding: 'utf8',
    maxBuffer: 256 * 1024 * 1024,
    ...options,
  });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(' ')} failed ${result.status}\n${result.stderr || ''}`);
  }
  return result.stdout;
};

fail(candidate.repository === 'telegramdesktop/tdesktop', 'unexpected candidate repository');
fail(/^[0-9a-f]{40}$/.test(candidate.commit), 'candidate commit must be immutable');
fail(/^[0-9a-f]{40}$/.test(candidate.tree), 'candidate tree must be immutable');

const checkoutRoot = process.env.TDRP_CANDIDATE_CHECKOUT || path.join(
  process.env.RUNNER_TEMP || '/tmp',
  `tdrp-candidate-${candidate.commit}`,
);
await fs.rm(checkoutRoot, { recursive: true, force: true });
run('git', [
  'clone', '--filter=blob:none', '--no-checkout', '--quiet',
  `https://github.com/${candidate.repository}.git`, checkoutRoot,
]);
run('git', ['-C', checkoutRoot, 'checkout', '--detach', '--quiet', candidate.commit]);
const actualCommit = run('git', ['-C', checkoutRoot, 'rev-parse', 'HEAD']).trim();
fail(actualCommit === candidate.commit, `candidate checkout drift: ${actualCommit}`);

const entries = run('git', ['-C', checkoutRoot, 'ls-tree', '-r', '-z', 'HEAD'])
  .split('\0')
  .filter(Boolean)
  .map((line) => {
    // `git ls-tree -z` permits any byte except NUL in a path, including newlines.
    // Match the complete NUL-delimited record rather than relying on dot matching.
    const match = line.match(/^([0-9]{6}) (\w+) ([0-9a-f]{40})\t([\s\S]+)$/);
    fail(match, `unparseable candidate tree entry: ${line}`);
    const [, mode, type, blob, entryPath] = match;
    return { mode, type, path: entryPath, blob };
  });
const files = entries.filter((entry) => entry.type === 'blob');
const gitlinks = entries.filter((entry) => entry.mode === '160000' || entry.type === 'commit');
const report = {
  project_id: 'TDRP-001',
  spec_revision: 9,
  repository: candidate.repository,
  ref: candidate.ref,
  commit: candidate.commit,
  tree: candidate.tree,
  checkout_commit: actualCommit,
  root_non_directory_entries: entries.length,
  root_blob_entries: files.length,
  direct_gitlinks: gitlinks.length,
  files,
  gitlinks,
  recursive_non_directory_entries: null,
  recursive_closure_status: 'root-complete-recursive-pending',
  source_closure_ready: false,
  accepted: false,
  generated_at: new Date().toISOString(),
};

const outputDir = path.join(root, 'artifacts/tdrp-authority');
await fs.mkdir(outputDir, { recursive: true });
await fs.writeFile(
  path.join(outputDir, 'candidate-root-inventory.json'),
  `${JSON.stringify(report, null, 2)}\n`,
);
console.log(JSON.stringify({
  commit: report.commit,
  root_non_directory_entries: report.root_non_directory_entries,
  root_blob_entries: report.root_blob_entries,
  direct_gitlinks: report.direct_gitlinks,
  recursive_closure_status: report.recursive_closure_status,
}, null, 2));
