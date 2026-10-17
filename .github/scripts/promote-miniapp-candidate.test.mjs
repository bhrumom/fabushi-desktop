import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { assertMonotonicVersion, verifyPromotion, verifyDeliveryChecksums, requiredSteps, requiredChecks } from './promote-miniapp-candidate.mjs';

function fixture() {
  const source = 'a'.repeat(40); const archive = 'b'.repeat(64); const consumer = 'c'.repeat(40);
  return { source, artifact: { id: 123, expired: false, workflow_run: { id: 456, head_sha: source }, name: `fabushi-macos-${source.slice(0, 12)}`, digest: `sha256:${archive}`, size_in_bytes: 100 }, candidateRun: { id: 456, head_sha: source, head_branch: 'main', event: 'push', status: 'completed', conclusion: 'success', path: '.github/workflows/release-macos-main.yml', repository: { full_name: 'bhrumom/fabushi-desktop' }, head_repository: { full_name: 'bhrumom/fabushi-desktop' } }, candidateJobs: [{ steps: requiredSteps.map((name) => ({ name, conclusion: 'success' })) }], checks: requiredChecks.map((name) => ({ name, status: 'completed', conclusion: 'success' })), factoryRun: { id: 789, run_attempt: 2, head_sha: consumer, head_branch: 'main', event: 'workflow_dispatch', status: 'completed', conclusion: 'success', path: '.github/workflows/ci-account-factory-cloud.yml', repository: { full_name: 'bhrumom/fabushi-backend' }, head_repository: { full_name: 'bhrumom/fabushi-backend' } }, factoryJobs: [{ name: 'desktop-miniapp', status: 'completed', conclusion: 'success' }], report: { kind: 'signed-desktop-miniapp-account-factory', overall: 'PASS', consumerBackendSha: consumer, factoryBackendSha: 'b562310d96a2c843e9b94f39c5506eed51b126d5', factoryCoreSha: 'c7d0e24b7561c6f5d1bb3c0e975f599643252504', desktopSourceSha: source, binarySourceSha: source, harnessSourceSha: source, artifactId: 123, archiveSha256: archive, runId: '789', runAttempt: '2', harnessExitCode: 0, assertions: { acceptedFactorySource: true, realDistinctOrdinaryAccounts: true, signedRealAccountTransition: true, zeroOwnedResidueAndRevocation: true } } };
}
test('only the same immutable main binary accepted by both independent gates promotes', () => {
  assert.equal(verifyPromotion(fixture()), true);
  for (const mutate of [
    (v) => { v.candidateRun.head_branch = 'codex/candidate'; }, (v) => { v.candidateRun.conclusion = 'failure'; },
    (v) => { v.candidateRun.head_repository.full_name = 'outside/fork'; }, (v) => { v.artifact.expired = true; },
    (v) => { v.artifact.workflow_run.id++; }, (v) => { v.factoryRun.conclusion = 'cancelled'; },
    (v) => { v.factoryRun.path = '.github/workflows/arbitrary.yml'; }, (v) => { v.factoryJobs[0].conclusion = 'skipped'; },
    (v) => { v.report.binarySourceSha = 'd'.repeat(40); }, (v) => { v.report.consumerBackendSha = 'd'.repeat(40); },
    (v) => { v.report.archiveSha256 = 'd'.repeat(64); }, (v) => { v.report.runAttempt = '1'; },
    (v) => { v.report.cleanup = 'blocked'; }, (v) => { v.report.pipeCleanupFailed = true; },
    (v) => { v.report.harnessExitCode = 1; }, (v) => { v.report.overall = 'FAIL'; },
  ]) { const value = fixture(); mutate(value); assert.throws(() => verifyPromotion(value), /PROMOTION_/); }
});
test('missing or skipped source, production lifecycle and cleanup assertions cannot pass', () => {
  for (const step of requiredSteps) { const v = fixture(); v.candidateJobs[0].steps.find((item) => item.name === step).conclusion = 'skipped'; assert.throws(() => verifyPromotion(v), /PROMOTION_SOURCE_GATES_MISSING/); }
  for (const name of requiredChecks) { const v = fixture(); v.checks = v.checks.filter((item) => item.name !== name); assert.throws(() => verifyPromotion(v), /PROMOTION_SOURCE_GATES_MISSING/); }
  for (const name of Object.keys(fixture().report.assertions)) { const v = fixture(); delete v.report.assertions[name]; assert.throws(() => verifyPromotion(v), /PROMOTION_FACTORY_ASSERTION_MISSING/); }
});
test('public Desktop version must strictly exceed every other published version', () => {
  assertMonotonicVersion('1.2.77', [{ tag_name: 'macos-main-v1.2.76-old' }], 'new');
  assertMonotonicVersion('1.2.77', [{ tag_name: 'same' }], 'same');
  for (const prior of ['1.2.77', '1.2.78', '1.3.0', '2.0.0']) assert.throws(() => assertMonotonicVersion('1.2.77', [{ tag_name: `macos-main-v${prior}-old` }], 'new'), /NOT_MONOTONIC/);
  for (const version of ['01.2.77', '1.2', '../release', '1.2.9007199254740993']) assert.throws(() => assertMonotonicVersion(version, [], 'new'), /VERSION_INVALID/);
});
test('promotion never rebuilds or deletes an existing release on uncertain outcome', () => {
  const source = readFileSync(new URL('./promote-miniapp-candidate.mjs', import.meta.url), 'utf8');
  assert.doesNotMatch(source, /release', 'delete|--clobber|--cleanup-tag|notarytool|npm.*build/u);
  assert.match(source, /PROMOTION_EXISTING_CONFLICT/u);
  assert.match(source, /artifact\.digest !== `sha256:\$\{archiveSha\}`/u);
  assert.doesNotMatch(source, /releases\/tags\/\$\{tag\}/u);
  assert.match(source, /releases\?per_page=100&page=1/u);
  assert.match(source, /PROMOTION_RELEASE_READBACK_FAILED/u);
});
test('published installer checksum manifest exactly covers the immutable delivery assets', () => {
  const digest = 'a'.repeat(64); const other = 'b'.repeat(64);
  const assets = { 'fabushi.zip': digest, 'fabushi.dmg': other, 'latest-mac.yml': digest };
  const valid = `${digest}  ./fabushi.zip\n${other}  ./fabushi.dmg\n`;
  verifyDeliveryChecksums(valid, assets);
  for (const invalid of ['', `${digest}  ./fabushi.zip`, `${valid}${digest}  fabushi.zip\n`, valid.replace(other, digest), valid.replace('fabushi.zip', '../fabushi.zip'), valid.replace('fabushi.zip', 'outside.zip')]) assert.throws(() => verifyDeliveryChecksums(invalid, assets), /PROMOTION_CHECKSUM_MANIFEST/);
});

test('promotion binds the isolated factory to the current deployed Backend/Core sources', () => {
  const value = fixture();
  assert.equal(value.report.factoryBackendSha, 'b562310d96a2c843e9b94f39c5506eed51b126d5');
  assert.equal(value.report.factoryCoreSha, 'c7d0e24b7561c6f5d1bb3c0e975f599643252504');
  for (const mutate of [
    (v) => { v.report.factoryBackendSha = '4ccf4f31ce035b19a0b697828f091d320e04b77b'; },
    (v) => { v.report.factoryCoreSha = 'd'.repeat(40); },
  ]) { const candidate = fixture(); mutate(candidate); assert.throws(() => verifyPromotion(candidate), /PROMOTION_FACTORY_BINDING_DENIED/); }
});
