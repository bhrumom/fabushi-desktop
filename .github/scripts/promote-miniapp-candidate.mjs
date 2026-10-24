import { createHash } from 'node:crypto';
import { createReadStream, openSync, closeSync, mkdirSync, readdirSync, lstatSync, readFileSync, appendFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

const desktop = 'bhrumom/fabushi-desktop'; const backend = 'bhrumom/fabushi-backend';
const requiredSteps = ['Build and stage Mahayana desktop host', 'Package signed and notarized macOS installer', 'Notarize and staple DMG', 'Bind packaged candidate executable', 'Run real signed packaged acceptance', 'Run approved Mini App installed and OS-offline lifecycle acceptance'];
const requiredChecks = ['rust-host', 'renderer', 'platform-worker', 'Pressure profiler / macos-15', 'Pressure profiler / ubuntu-24.04', 'Pressure profiler / windows-2025', 'Focused Electron chat E2E', 'Renderer typecheck and build', 'bounded-validation', 'native-intents', 'publisher', 'electron-window', 'verifier'];
const fail = (code) => { throw Error(code); };
export { requiredSteps, requiredChecks };

export function verifyDeliveryChecksums(manifest, digests) {
  const expected = Object.keys(digests).filter((name) => /\.(dmg|zip)$/.test(name));
  const seen = new Set();
  for (const line of manifest.trim().split(/\r?\n/)) {
    const match = /^([a-f0-9]{64})  (?:\.\/)?([A-Za-z0-9._-]+\.(?:dmg|zip))$/.exec(line);
    if (!match || seen.has(match[2]) || digests[match[2]] !== match[1]) fail('PROMOTION_CHECKSUM_MANIFEST');
    seen.add(match[2]);
  }
  if (!expected.length || expected.length !== seen.size || expected.some((name) => !seen.has(name))) fail('PROMOTION_CHECKSUM_MANIFEST');
}

export function verifyPromotion({ source, artifact, candidateRun, candidateJobs, checks, factoryRun, factoryJobs, report }) {
  if (!/^[a-f0-9]{40}$/.test(source || '') || candidateRun.head_sha !== source || candidateRun.head_branch !== 'main' || candidateRun.event !== 'push' || candidateRun.status !== 'completed' || candidateRun.conclusion !== 'success' || candidateRun.path !== '.github/workflows/release-macos-main.yml' || candidateRun.repository?.full_name !== desktop || candidateRun.head_repository?.full_name !== desktop) fail('PROMOTION_CANDIDATE_DENIED');
  if (artifact.expired !== false || artifact.workflow_run?.id !== candidateRun.id || artifact.workflow_run?.head_sha !== source || artifact.name !== `fabushi-macos-${source.slice(0, 12)}` || !/^sha256:[a-f0-9]{64}$/.test(artifact.digest || '') || !Number.isSafeInteger(artifact.size_in_bytes) || artifact.size_in_bytes <= 0 || artifact.size_in_bytes > 1024 ** 3) fail('PROMOTION_ARTIFACT_DENIED');
  const steps = new Set(candidateJobs.flatMap((job) => job.steps || []).filter((step) => step.conclusion === 'success').map((step) => step.name));
  if (!requiredSteps.every((name) => steps.has(name)) || !requiredChecks.every((name) => { const found = checks.filter((check) => check.name === name); return found.length && found.every((check) => check.status === 'completed' && check.conclusion === 'success'); })) fail('PROMOTION_SOURCE_GATES_MISSING');
  if (factoryRun.head_branch !== 'main' || factoryRun.event !== 'workflow_dispatch' || factoryRun.status !== 'completed' || factoryRun.conclusion !== 'success' || factoryRun.path !== '.github/workflows/ci-account-factory-cloud.yml' || factoryRun.repository?.full_name !== backend || factoryRun.head_repository?.full_name !== backend || !factoryJobs.some((job) => job.name === 'desktop-miniapp' && job.status === 'completed' && job.conclusion === 'success')) fail('PROMOTION_FACTORY_RUN_DENIED');
  if (report.kind !== 'signed-desktop-miniapp-account-factory' || report.overall !== 'PASS' || report.consumerBackendSha !== factoryRun.head_sha || report.factoryBackendSha !== 'b562310d96a2c843e9b94f39c5506eed51b126d5' || report.factoryCoreSha !== 'c7d0e24b7561c6f5d1bb3c0e975f599643252504' || [report.desktopSourceSha, report.binarySourceSha, report.harnessSourceSha].some((value) => value !== source) || report.artifactId !== artifact.id || `sha256:${report.archiveSha256}` !== artifact.digest || String(report.runId) !== String(factoryRun.id) || String(report.runAttempt) !== String(factoryRun.run_attempt) || report.harnessExitCode !== 0 || report.pipeCleanupFailed === true || report.failureCode || report.cleanup === 'blocked') fail('PROMOTION_FACTORY_BINDING_DENIED');
  for (const name of ['acceptedFactorySource', 'realDistinctOrdinaryAccounts', 'signedRealAccountTransition', 'zeroOwnedResidueAndRevocation']) if (report.assertions?.[name] !== true) fail('PROMOTION_FACTORY_ASSERTION_MISSING');
  return true;
}

export function assertMonotonicVersion(version, releases, tag) {
  if (!/^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/.test(version || '')) fail('PROMOTION_VERSION_INVALID');
  const wanted = version.split('.').map(Number);
  if (wanted.some((value) => !Number.isSafeInteger(value) || value > 0xffffffff)) fail('PROMOTION_VERSION_INVALID');
  for (const release of releases) {
    if (release.draft || release.tag_name === tag) continue;
    const match = /(?:^|v)([0-9]+)\.([0-9]+)\.([0-9]+)(?:-|$)/.exec(release.tag_name || '');
    if (!match) continue;
    const prior = match.slice(1).map(Number);
    const different = wanted.findIndex((value, index) => value !== prior[index]);
    if (different < 0 || wanted[different] < prior[different]) fail('PROMOTION_VERSION_NOT_MONOTONIC');
  }
}

async function promote() {
  if (process.env.GITHUB_ACTIONS !== 'true' || process.env.GITHUB_REF !== 'refs/heads/main' || process.env.PROMOTION_PUBLISH !== 'true' || process.env.PROMOTION_DIAGNOSTIC === 'true') fail('PROMOTION_CONTEXT_DENIED');
  const source = process.env.PROMOTION_SOURCE_SHA; const archiveSha = process.env.PROMOTION_ARCHIVE_SHA256;
  const runId = process.env.PROMOTION_CANDIDATE_RUN_ID; const artifactId = process.env.PROMOTION_ARTIFACT_ID; const factoryId = process.env.PROMOTION_FACTORY_RUN_ID;
  if (!/^[a-f0-9]{40}$/.test(source || '') || !/^[a-f0-9]{64}$/.test(archiveSha || '') || ![runId, artifactId, factoryId].every((id) => /^[1-9][0-9]*$/.test(id || '') && Number.isSafeInteger(Number(id)))) fail('PROMOTION_INPUT_INVALID');
  const api = async (repo, path) => {
    const response = await fetch(`https://api.github.com/repos/${repo}/${path}`, { headers: { authorization: `Bearer ${process.env.GH_TOKEN}`, accept: 'application/vnd.github+json', 'user-agent': 'Fabushi-MiniApp-Promotion/1' }, redirect: 'error', signal: AbortSignal.timeout(30_000) });
    if (!response.ok) fail('PROMOTION_GITHUB_READ_FAILED'); return response.json();
  };
  const candidateRun = await api(desktop, `actions/runs/${runId}`); const artifact = await api(desktop, `actions/artifacts/${artifactId}`);
  const candidateJobs = (await api(desktop, `actions/runs/${runId}/jobs?per_page=100`)).jobs;
  const checks = (await api(desktop, `commits/${source}/check-runs?per_page=100`)).check_runs;
  const factoryRun = await api(backend, `actions/runs/${factoryId}`); const factoryJobs = (await api(backend, `actions/runs/${factoryId}/jobs?per_page=100`)).jobs;
  const evidenceList = (await api(backend, `actions/runs/${factoryId}/artifacts?per_page=100`)).artifacts.filter((item) => item.name === `desktop-miniapp-factory-${source}-${factoryId}-${factoryRun.run_attempt}`);
  if (evidenceList.length !== 1 || evidenceList[0].expired !== false || evidenceList[0].size_in_bytes <= 0 || evidenceList[0].size_in_bytes > 20 * 1024 * 1024 || !/^sha256:[a-f0-9]{64}$/.test(evidenceList[0].digest || '')) fail('PROMOTION_FACTORY_ARTIFACT_DENIED');
  const root = join(process.env.RUNNER_TEMP, `miniapp-promotion-${process.env.GITHUB_RUN_ID}-${process.env.GITHUB_RUN_ATTEMPT}`); mkdirSync(root, { mode: 0o700 });
  const hashFile = async (path) => { const hash = createHash('sha256'); for await (const chunk of createReadStream(path)) hash.update(chunk); return hash.digest('hex'); };
  const download = async (repo, item, name) => {
    const path = join(root, name); const fd = openSync(path, 'wx', 0o600);
    try { execFileSync('gh', ['api', `repos/${repo}/actions/artifacts/${item.id}/zip`], { stdio: ['ignore', fd, 'pipe'], timeout: 180_000 }); } finally { closeSync(fd); }
    if (lstatSync(path).size !== item.size_in_bytes || `sha256:${await hashFile(path)}` !== item.digest) fail('PROMOTION_ARCHIVE_INTEGRITY'); return path;
  };
  const proofZip = await download(backend, evidenceList[0], 'factory.zip');
  const proofNames = execFileSync('unzip', ['-Z1', proofZip], { encoding: 'utf8' }).trim().split('\n').filter((name) => ['desktop-evidence/factory-acceptance.json', 'factory-acceptance.json'].includes(name));
  if (proofNames.length !== 1) fail('PROMOTION_FACTORY_REPORT_AMBIGUOUS');
  const reportBytes = execFileSync('unzip', ['-p', proofZip, proofNames[0]], { maxBuffer: 64 * 1024 });
  const report = JSON.parse(reportBytes.toString('utf8'));
  verifyPromotion({ source, artifact, candidateRun, candidateJobs, checks, factoryRun, factoryJobs, report });
  if (artifact.id !== Number(artifactId) || artifact.digest !== `sha256:${archiveSha}`) fail('PROMOTION_INPUT_BINDING_DENIED');
  const candidateZip = await download(desktop, artifact, 'candidate.zip');
  const members = execFileSync('unzip', ['-Z1', candidateZip], { encoding: 'utf8' }).trim().split('\n');
  if (!members.length || members.length > 32 || new Set(members).size !== members.length || members.some((name) => name.includes('/') || name.includes('\\') || name.startsWith('.') || !/^(?:[A-Za-z0-9._-]+\.(?:dmg|zip|blockmap)|latest-mac\.yml|SHA256SUMS\.txt)$/.test(name))) fail('PROMOTION_DELIVERY_MEMBERS_DENIED');
  const delivery = join(root, 'delivery'); mkdirSync(delivery);
  execFileSync('unzip', ['-q', candidateZip, '-d', delivery], { stdio: 'pipe' });
  const assets = readdirSync(delivery).map((name) => join(delivery, name));
  if (assets.some((path) => !lstatSync(path).isFile())) fail('PROMOTION_DELIVERY_MEMBER_TYPE');
  const pkg = await api(desktop, `contents/desktop/package.json?ref=${source}`); const version = JSON.parse(Buffer.from(pkg.content, 'base64').toString('utf8')).version;
  const tag = `macos-main-v${version}-${source.slice(0, 12)}`;
  assertMonotonicVersion(version, [], tag);
  const names = assets.map((path) => path.slice(delivery.length + 1));
  if (!names.includes(`fabushi-${version}-macos-arm64.zip`) || !names.includes('latest-mac.yml') || !names.includes('SHA256SUMS.txt') || !names.some((name) => name.endsWith('.dmg'))) fail('PROMOTION_DELIVERY_INCOMPLETE');
  const digests = Object.fromEntries(await Promise.all(assets.map(async (path) => [path.slice(delivery.length + 1), await hashFile(path)])));
  verifyDeliveryChecksums(readFileSync(join(delivery, 'SHA256SUMS.txt'), 'utf8'), digests);
  const yml = readFileSync(join(delivery, 'latest-mac.yml'), 'utf8');
  let updateVersion = yml.split('\n').find((item) => item.startsWith('version:'))?.slice('version:'.length).trim();
  if (updateVersion && ['"', "'"].includes(updateVersion[0]) && updateVersion.at(-1) === updateVersion[0]) updateVersion = updateVersion.slice(1, -1);
  if (updateVersion !== version) fail('PROMOTION_UPDATER_VERSION');
  const releases = [];
  for (let page = 1; page <= 10; page++) { const batch = await api(desktop, `releases?per_page=100&page=${page}`); releases.push(...batch); if (batch.length < 100) break; if (page === 10) fail('PROMOTION_RELEASE_HISTORY_LIMIT'); }
  assertMonotonicVersion(version, releases, tag);
  let existing = releases.find((release) => release.tag_name === tag);
  if (!existing) {
    const notes = `Signed/notarized Desktop ${version}, source ${source}. Global Dandelion0.1.0 proprietary offline packet workbench; no native LAN/WAN/hotspot claims. Production chat/install/offline lifecycle: https://github.com/${desktop}/actions/runs/${runId}. Real isolated factory accounts, signed window isolation and zero-residue cleanup: https://github.com/${backend}/actions/runs/${factoryId}. Published bytes are exactly the accepted artifact, with no rebuild. Full native/hardware/mobile OMA acceptance remains separate.`;
    execFileSync('gh', ['release', 'create', tag, ...assets, '--repo', desktop, '--target', source, '--title', `Fabushi macOS ${version} — ${source.slice(0, 12)}`, '--notes', notes, '--prerelease', '--draft'], { stdio: 'inherit' });
    const createdReleases = await api(desktop, 'releases?per_page=100&page=1');
    existing = createdReleases.find((release) => release.tag_name === tag && release.draft === true);
    if (!existing) fail('PROMOTION_RELEASE_READBACK_FAILED');
  }
  if (existing.target_commitish !== source || existing.assets.length !== assets.length) fail('PROMOTION_EXISTING_CONFLICT');
  for (const asset of assets) { const name = asset.slice(delivery.length + 1); const digest = `sha256:${await hashFile(asset)}`; if (!existing.assets.some((item) => item.name === name && item.digest === digest)) fail('PROMOTION_EXISTING_CONFLICT'); }
  // A partial or ambiguous upload remains private. Never delete/recreate it.
  if (existing.draft) execFileSync('gh', ['release', 'edit', tag, '--repo', desktop, '--draft=false', '--prerelease'], { stdio: 'inherit' });
  appendFileSync(process.env.GITHUB_STEP_SUMMARY, `Verified immutable promotion: https://github.com/${desktop}/releases/tag/${tag}\nBinary source: ${source}; archive SHA256: ${archiveSha}; factory run: ${factoryId}.\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try { await promote(); } catch (error) { console.error(/^PROMOTION_[A-Z_]+$/.test(error.message || '') ? error.message : 'PROMOTION_FAILED'); process.exitCode = 1; }
}
