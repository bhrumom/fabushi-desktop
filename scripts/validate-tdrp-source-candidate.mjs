#!/usr/bin/env node
// Read-only focused candidate consistency. Downloads/extraction belong to the
// existing tdrp-summarize-protocol-artifact.py recovery step in GitHub Actions.
import fs from 'node:fs/promises';
import {constants} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {validateSourceCandidate} from './tdrp-source-review.mjs';

const REPOSITORY = 'bhrumom/fabushi-desktop';
const MAX_JSON_BYTES = 16 * 1024 * 1024;
export const SOURCE_CANDIDATE_INPUTS = {
  git: {
    repository: REPOSITORY,
    head_sha: '9f77bb2389cba6a65d589eb309d1392501ed308d',
    run_id: 41397005716, artifact_id: 13118749668, bytes: 4582365,
    name: 'tdrp-fixed-upstream-git-9f77bb2389cba6a65d589eb309d1392501ed308d',
    digest: 'sha256:77501e120e4df95742c028a7b35e1e7e4f570b0a7b2a2d6842999a70be8f9d60',
    files: {
      'authority-report.json': {bytes: 3897, sha256: '8b6a70b05030f2a389cfb78b740ea5eddcbf941afd5bd5974dbebfd90b7627ad'},
      'upstream-root-inventory.json': {bytes: 1431097, sha256: '6330dccd9f5e9502b89b3e0890ee1aa079d6f03510497514b44f901b2379fb2f'},
      'upstream-recursive-inventory.json': {bytes: 6010540, sha256: 'd2aa4fd1a6f02b496b44f4a759880905cd2b186adda824217685742e477c9a4b'},
      'upstream-git-acquisition.json': {bytes: 5444621, sha256: 'd5c62dcd1a1397c2e2c633527db69409d227a2471a3713b483dc15a70333897b'},
    },
  },
  export: {
    repository: REPOSITORY,
    head_sha: '1cd59bb01513624d26e12f77bfab66729ab72694',
    run_id: 41401970993, artifact_id: 13121721589, bytes: 9756692,
    name: 'tdrp-fixed-build-inputs-1cd59bb01513624d26e12f77bfab66729ab72694',
    digest: 'sha256:7d246df47d044b89cc5eb6c9287f7ddbf5ffa3d95631be83ac18d123d4b47fdc',
    files: {
      'fixed-input-export/documents/build-time-acquisition-candidates.json': {bytes: 201915, sha256: '03a0ddbf4b69515f0161e804a4ba8c93e2e615747e9e506f0336d2483bd04e4a'},
      'fixed-input-export/documents/fixed-u-build-input-contexts.json': {bytes: 1138162, sha256: '8da7109d597530bae9bfe770cb49773f4637371b22bcec18d15654832d1b5a11'},
      'fixed-input-export/documents/fixed-u-build-input-delta.json': {bytes: 368807, sha256: 'c7ef3e019de515e08539b3a8976408981af8910528d80447fda8487e48f2e518'},
      'fixed-input-export/documents/fixed-u-build-input-report.json': {bytes: 7229, sha256: 'fb6feccfff369ed510b99c2280314ebe94ab5121da17a422689aa20da537a6b3'},
    },
  },
};

const check = (ok, message) => { if (!ok) throw Error('candidate evidence: ' + message); };
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const artifactIdentity = pin => Object.fromEntries(
  ['repository', 'head_sha', 'artifact_id', 'run_id', 'digest'].map(key => [key, pin[key]]));
const equalRecord = (left, right) => left && typeof left === 'object' && !Array.isArray(left) &&
  Object.keys(left).length === Object.keys(right).length &&
  Object.entries(right).every(([key, value]) => left[key] === value);
const safeMember = value => typeof value === 'string' && value.length > 0 && value.length <= 1024 &&
  !/[\\\u0000-\u001f\u007f]/.test(value) &&
  value.split('/').length <= 32 && value.split('/').every(part => part && part !== '.' && part !== '..');

async function noSymlinks(file) {
  let current = path.resolve(file);
  for (;;) {
    check(!(await fs.lstat(current)).isSymbolicLink(), 'symlink in evidence path');
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
}

async function readBounded(file, maximum) {
  await noSymlinks(file);
  const handle = await fs.open(file, constants.O_RDONLY | (constants.O_NOFOLLOW || 0));
  try {
    const stat = await handle.stat();
    check(stat.isFile() && stat.size >= 0 && stat.size <= maximum, 'file is not bounded regular evidence');
    const bytes = await handle.readFile();
    check(bytes.length === stat.size && bytes.length <= maximum, 'evidence size changed or exceeds bound');
    return bytes;
  } finally {
    await handle.close();
  }
}

function parseJson(bytes) {
  const text = new TextDecoder('utf-8', {fatal: true}).decode(bytes);
  return JSON.parse(text);
}

// The pin parameter is injectable for hermetic Actions tests. Production main
// always supplies SOURCE_CANDIDATE_INPUTS; the CLI has no pin override option.
export async function readRecoveredCandidateDocuments(directory, pin, targetSha) {
  const index = parseJson(await readBounded(path.join(directory, 'index.json'), 4 * 1024 * 1024));
  check(index.schema === 'tdrp-protocol-artifact-recovery/v1' && index.complete === true,
    'complete recovery index required');
  check(equalRecord(index.expected, artifactIdentity(pin)), 'artifact/producer identity mismatch');
  check(index.recovery_execution?.GITHUB_REPOSITORY === REPOSITORY &&
    index.recovery_execution?.GITHUB_SHA === targetSha, 'recovery belongs to another target H');
  check(index.archive?.path === 'artifact.zip' && index.archive.complete === true &&
    index.archive.verified === true && index.archive.bytes === pin.bytes &&
    'sha256:' + index.archive.sha256 === pin.digest, 'complete pinned archive index required');
  const archive = await readBounded(path.join(directory, 'artifact.zip'), pin.bytes);
  check(archive.length === pin.bytes && 'sha256:' + sha256(archive) === pin.digest,
    'complete ZIP size/digest mismatch');
  check(index.metadata?.path === 'metadata.json' && index.metadata.verified === true,
    'verified metadata index required');
  const metadataBytes = await readBounded(path.join(directory, 'metadata.json'), 1024 * 1024);
  check(metadataBytes.length === index.metadata.bytes && sha256(metadataBytes) === index.metadata.sha256,
    'metadata bytes/digest mismatch');
  const metadata = parseJson(metadataBytes);
  const base = 'https://api.github.com/repos/' + REPOSITORY + '/actions/artifacts/' + pin.artifact_id;
  check(metadata.id === pin.artifact_id && metadata.name === pin.name && metadata.expired === false &&
    metadata.digest === pin.digest && metadata.size_in_bytes === pin.bytes &&
    metadata.url === base && metadata.archive_download_url === base + '/zip' &&
    metadata.workflow_run?.id === pin.run_id && metadata.workflow_run?.head_sha === pin.head_sha,
    'metadata producer/artifact mismatch');
  check(Array.isArray(index.members) && index.members.length > 0 && index.members.length <= 4096 &&
    index.completed_members === index.members.length, 'recovery member accounting incomplete');
  const members = new Map();
  let total = 0;
  for (const member of index.members) {
    const name = member.relative_path;
    check(safeMember(name) && !members.has(name) && member.complete === true &&
      member.raw_path === 'raw/' + name && Number.isSafeInteger(member.bytes_read) && member.bytes_read >= 0 &&
      member.bytes_read === member.declared_bytes, 'invalid, duplicated or incomplete recovery member');
    check((member.kind === 'file' && /^[0-9a-f]{64}$/.test(member.sha256 || '') &&
      member.digest_scope === 'complete member') ||
      (member.kind === 'directory' && member.bytes_read === 0), 'recovery member digest incomplete');
    members.set(name, member);
    total += member.bytes_read;
  }
  check(Number.isSafeInteger(total) && total === index.uncompressed_bytes_read &&
    total === index.declared_uncompressed_bytes, 'recovery total byte accounting mismatch');
  const documents = {};
  for (const [name, filePin] of Object.entries(pin.files)) {
    check(safeMember(name), 'unsafe pinned member name');
    const member = members.get(name);
    check(member?.kind === 'file' && member.member_name === name &&
      member.bytes_read === filePin.bytes && member.sha256 === filePin.sha256,
      'exact pinned member missing or altered: ' + name);
    const bytes = await readBounded(path.join(directory, 'raw', name), MAX_JSON_BYTES);
    check(bytes.length === filePin.bytes && sha256(bytes) === filePin.sha256,
      'pinned member bytes/digest mismatch: ' + name);
    documents[name] = parseJson(bytes);
  }
  return {documents, artifact: artifactIdentity(pin)};
}

export async function main(argv = process.argv.slice(2)) {
  check(process.env.GITHUB_ACTIONS === 'true', 'candidate verification may execute only in GitHub Actions');
  const options = {};
  for (let i = 0; i < argv.length; i += 2) {
    check(['--git-recovery-dir', '--export-recovery-dir', '--output'].includes(argv[i]) &&
      typeof argv[i + 1] === 'string' && !options[argv[i]], 'invalid or duplicate CLI option');
    options[argv[i]] = argv[i + 1];
  }
  check(Object.keys(options).length === 3, 'three candidate input/output options required');
  const targetSha = process.env.GITHUB_SHA;
  check(process.env.GITHUB_REPOSITORY === REPOSITORY && /^[0-9a-f]{40}$/.test(targetSha || '') &&
    (!process.env.TDRP_TARGET_SHA || process.env.TDRP_TARGET_SHA === targetSha),
    'exact Actions repository/head required');
  const git = await readRecoveredCandidateDocuments(options['--git-recovery-dir'], SOURCE_CANDIDATE_INPUTS.git, targetSha);
  const exported = await readRecoveredCandidateDocuments(options['--export-recovery-dir'], SOURCE_CANDIDATE_INPUTS.export, targetSha);
  const names = [
    'upstream.lock.json', 'inventory/index.json', 'parity-ledger.json',
    'inventory/build-time-acquisitions.json', 'inventory/root-mutable-authorities.json',
    'inventory/root-tool-authorities.json',
  ];
  const local = [];
  for (const name of names) {
    local.push(parseJson(await readBounded(path.join('projects/telegram-desktop-rust', name), MAX_JSON_BYTES)));
  }
  const [lock, index, ledger, acquisitions, mutableAuthorities, toolAuthorities] = local;
  const exportedDocument = name => exported.documents['fixed-input-export/documents/' + name];
  const result = validateSourceCandidate({
    lock, index, ledger, acquisitions, mutableAuthorities, toolAuthorities,
    rootInventory: git.documents['upstream-root-inventory.json'],
    recursiveInventory: git.documents['upstream-recursive-inventory.json'],
    inventoryReport: git.documents['authority-report.json'],
    acquisitionManifest: git.documents['upstream-git-acquisition.json'],
    buildInputs: {artifact: exported.artifact,
      report: exportedDocument('fixed-u-build-input-report.json'),
      candidates: exportedDocument('build-time-acquisition-candidates.json'),
      contexts: exportedDocument('fixed-u-build-input-contexts.json'),
      delta: exportedDocument('fixed-u-build-input-delta.json')},
  });
  const output = options['--output'];
  await noSymlinks(path.dirname(output));
  const report = {...result, target_commit: targetSha,
    reused_git_artifact: git.artifact, reused_build_input_artifact: exported.artifact};
  await fs.writeFile(output, JSON.stringify(report, null, 2) + '\n', {flag: 'wx'});
  console.log(JSON.stringify(report));
  return 0;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => {
    const message = String(error?.message || error).replace(/[\u0000-\u001f\u007f]/g, '?').replaceAll('::', ': :');
    console.error('candidate verification failed: ' + message.slice(0, 1600));
    process.exitCode = 1;
  });
}

