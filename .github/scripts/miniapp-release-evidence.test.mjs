import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const workflow = readFileSync(new URL('../workflows/release-macos-main.yml', import.meta.url), 'utf8');
test('packaged artifact upload allowlists diagnostics rather than private runtime data', () => {
  const block = workflow.split('- name: Upload packaged acceptance evidence')[1]?.split('- name: Write checksums')[0];
  assert.ok(block);
  const base = '${{ runner.temp }}/fabushi-candidate-acceptance';
  const paths = block.split('\n').map((line) => line.trim()).filter((line) => line.startsWith(base));
  assert.deepEqual(paths, ['screenshots', '*.json', '*.txt', '*.log', '*.zip', '*.mov', '*.mp4'].map((suffix) => `${base}/${suffix}`));
  assert.doesNotMatch(block, /app-data|credential|host-secrets|sand-secrets|\.sqlite/u);
});
test('native report test does not replace the signed app download policy or grant OS automation', () => {
  const harness = readFileSync(new URL('../../desktop/e2e/miniapp-packaged-acceptance.spec.ts', import.meta.url), 'utf8');
  const save = readFileSync(new URL('./miniapp-save-report.applescript', import.meta.url), 'utf8');
  assert.match(harness, /process\.env\.GITHUB_ACTIONS !== 'true'/u);
  assert.match(save, /unix id is appPid/u);
  assert.match(save, /REPORT_FILENAME_MISMATCH/u);
  assert.doesNotMatch(harness, /Browser\.setDownloadBehavior|setSavePath|showSaveDialog\s*=/u);
  assert.doesNotMatch(save, /tccutil|TCC\.db|AXIsProcessTrustedWithOptions/u);
});
test('diagnostic fixture uses the existing protected CI device identity contract', () => {
  const iteration = readFileSync(new URL('../workflows/miniapp-packaged-iteration.yml', import.meta.url), 'utf8');
  assert.ok(iteration.includes('DEVICE_ID: gha-${{ github.run_id }}-${{ github.run_attempt }}-macos-app'));
  assert.ok(iteration.includes('5d75920308fa17f5000308df1a0153309eb8e9ab'));
});
