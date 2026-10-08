import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const lock = JSON.parse(fs.readFileSync('projects/telegram-desktop-rust/upstream.lock.json', 'utf8'));
const accepted = lock.upstream.commit;
const work = process.env.TDRP_UPSTREAM_WORK || path.join(process.env.RUNNER_TEMP || '/tmp', 'tdesktop-acquisition-scan');
const manifestDir = process.argv[2] || 'projects/telegram-desktop-rust/inventory/source-attestation-manifests';
const outDir = 'artifacts/tdrp-authority';
fs.mkdirSync(outDir, { recursive: true });

function run(cmd, args) {
  return execFileSync(cmd, args, { encoding: 'utf8' }).trimEnd();
}
function symbolFor(sourcePath) {
  return path.basename(sourcePath).replace(/@(?:2x|3x)(?=\.)/, '').replace(/\.[^.]+$/, '');
}
function grepSymbol(symbol) {
  try {
    return run('git', ['-C', work, 'grep', '-n', '-F', symbol, '--', 'Telegram/SourceFiles']);
  } catch (error) {
    if (error?.status === 1) return '';
    throw error;
  }
}

const manifests = fs.readdirSync(manifestDir)
  .filter(name => /^\d+-\d+\.json$/.test(name))
  .sort((a, b) => Number(a.split('-')[0]) - Number(b.split('-')[0]));

for (const name of manifests) {
  const manifest = JSON.parse(fs.readFileSync(path.join(manifestDir, name), 'utf8'));
  if (manifest.accepted_upstream !== accepted) throw new Error(name + ': accepted upstream drift');
  if (!Number.isInteger(manifest.start) || !Number.isInteger(manifest.end) || manifest.start > manifest.end) throw new Error(name + ': invalid bounds');
  if (!Array.isArray(manifest.entries) || manifest.entries.length !== manifest.end - manifest.start + 1) throw new Error(name + ': entry count mismatch');
  for (let i = 0; i < manifest.entries.length; i++) {
    const entry = manifest.entries[i];
    if (entry.order !== manifest.start + i) throw new Error(name + ': non-contiguous order at ' + entry.order);
    if (!/^[0-9a-f]{40}$/.test(entry.blob)) throw new Error(name + ': invalid blob at ' + entry.order);
  }

  const range = manifest.start + '-' + manifest.end;
  const binary = ['accepted_upstream=' + accepted];
  const reachability = ['accepted_upstream=' + accepted];
  const trace = ['accepted_upstream=' + accepted];
  const seenSymbols = new Set();

  for (const entry of manifest.entries) {
    const actual = run('git', ['-C', work, 'hash-object', entry.path]);
    if (actual !== entry.blob) throw new Error(range + ': blob mismatch order ' + entry.order + ' expected=' + entry.blob + ' actual=' + actual);
    const absolute = path.join(work, entry.path);
    const size = fs.statSync(absolute).size;
    const kind = run('file', ['-b', absolute]);
    binary.push([entry.order, entry.path, actual, size, kind].join('|'));

    const symbol = entry.consumer_symbol || symbolFor(entry.path);
    if (!seenSymbols.has(symbol)) {
      seenSymbols.add(symbol);
      const hits = grepSymbol(symbol);
      reachability.push('symbol=' + symbol);
      if (hits) {
        reachability.push(hits);
        trace.push(hits);
      }
    }
  }

  fs.writeFileSync(path.join(outDir, 'source-binary-evidence-' + range + '.txt'), binary.join('\n') + '\n');
  fs.writeFileSync(path.join(outDir, 'source-consumer-reachability-' + range + '.txt'), reachability.join('\n') + '\n');
  fs.writeFileSync(path.join(outDir, 'source-consumer-trace-' + range + '.txt'), trace.join('\n') + '\n');
  console.log('attested source range ' + range + ' entries=' + manifest.entries.length + ' symbols=' + seenSymbols.size);
}
