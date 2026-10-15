import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { execFile } from 'node:child_process';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { readFileSync, writeFileSync, mkdirSync, symlinkSync, truncateSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const exec = promisify(execFile);
const runtime = process.env.G11_RUNTIME;
const evidence = process.env.G11_EVIDENCE;
mkdirSync(evidence, { recursive: true });
const report = { sourceSha: process.env.EXACT_SHA, runId: process.env.GITHUB_RUN_ID,
  command: 'node .github/scripts/fdn-g11-real-browser.mjs', environment: 'GitHub Actions ubuntu-24.04; disposable managed Chromium; loopback CDP',
  scope: 'shipping driver only; outer Rust Host/box transport and extension not exercised', cases: [] };
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
let browserProcess, browser, server;
async function check(name, fn) {
  const start = Date.now();
  try { await fn(); report.cases.push({ name, status: 'passed', exitCode: 0, durationMs: Date.now() - start }); }
  catch (error) { report.cases.push({ name, status: 'failed', exitCode: 1, error: error.message, durationMs: Date.now() - start }); }
  console.log(`${report.cases.at(-1).status}: ${name}`);
}
try {
  const source = readFileSync('source/host/src/runner/tools/sand_browser_driver_source.rs', 'utf8');
  const marker = 'pub const SAND_BROWSER_DRIVER_SOURCE: &str = r#"';
  assert.equal(source.split(marker).length, 2);
  const payload = source.split(marker)[1];
  const end = payload.lastIndexOf('"#;');
  assert.ok(end > 0);
  const driverSource = payload.slice(0, end);
  report.driverSha256 = sha(driverSource);
  report.driverSourceFileSha256 = sha(source);
  const driverPath = join(runtime, 'shipping-driver.mjs');
  writeFileSync(driverPath, driverSource);
  const { chromium } = await import(pathToFileURL(join(runtime, 'node_modules/playwright-core/index.mjs')).href);
  report.playwrightVersion = JSON.parse(readFileSync(join(runtime, 'node_modules/playwright-core/package.json'))).version;
  assert.equal(report.playwrightVersion, '1.62.1');
  server = createServer((req, res) => {
    if (req.url === '/download') {
      res.writeHead(200, { 'Content-Type': 'application/octet-stream', 'Content-Disposition': 'attachment; filename="fixture.txt"' });
      return res.end('g11-download-bytes');
    }
    res.setHeader('Content-Type', 'text/html');
    if (req.url === '/frame') return res.end('<input aria-label="Frame input"><button onclick="this.textContent=\'frame-clicked\'">Frame button</button>');
    res.end(`<title>${req.url}</title><h1>${req.url}</h1><input aria-label="Main input"><input type="file" aria-label="Upload"><a href="/download">Download fixture</a><iframe name="fixture-frame" src="/frame"></iframe>`);
  });
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const origin = `http://127.0.0.1:${server.address().port}`;
  const profile = join(runtime, 'disposable-profile');
  const display = 77;
  const cdpPort = 9477;
  rmSync(`/tmp/.sand-browser/views-${display}.json`, { force: true });
  browserProcess = spawn(chromium.executablePath(), ['--headless=new', '--no-sandbox', '--disable-dev-shm-usage', '--remote-debugging-address=127.0.0.1', `--remote-debugging-port=${cdpPort}`, `--user-data-dir=${profile}`, 'about:blank'], { stdio: 'ignore' });
  for (let attempt = 0; attempt < 100; attempt++) {
    try { if ((await fetch(`http://127.0.0.1:${cdpPort}/json/version`)).ok) break; } catch {}
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  browser = await chromium.connectOverCDP(`http://127.0.0.1:${cdpPort}`);
  report.chromiumVersion = browser.version();
  const context = browser.contexts()[0];
  async function call(request) {
    const args = { display, cdpPort, ...request };
    const { stdout, stderr } = await exec(process.execPath, [driverPath, Buffer.from(JSON.stringify(args)).toString('base64')], { timeout: 100000, maxBuffer: 1024 * 1024 });
    const line = stdout.split('\n').find(line => line.startsWith('__SAND_BROWSER_RESULT__'));
    assert.ok(line, `driver result missing: ${stderr}`);
    return JSON.parse(line.slice('__SAND_BROWSER_RESULT__'.length));
  }
  const ok = result => { assert.equal(result.ok, true, result.error); return result; };
  const denied = (result, pattern) => { assert.equal(result.ok, false, 'unsafe action was accepted'); assert.match(result.error, pattern); };
  async function observePrimary(fn) {
    const observer = await chromium.connectOverCDP(`http://127.0.0.1:${cdpPort}`);
    try {
      const ctx = observer.contexts()[0];
      const state = JSON.parse(readFileSync(`/tmp/.sand-browser/views-${display}.json`));
      for (const page of ctx.pages()) {
        const session = await ctx.newCDPSession(page);
        const { targetInfo } = await session.send('Target.getTargetInfo'); await session.detach();
        if (targetInfo.targetId === state.views.primary) return await fn(page);
      }
      throw new Error('Exact claimed target missing from real observer');
    } finally { await observer.close(); }
  }
  let current;
  const snapshot = async extra => ok(await call({ op: 'snapshot', viewId: 'primary', ...extra }));
  const refFor = (snap, label) => {
    const line = snap.data.split('\n').find(line => line.includes(label));
    assert.ok(line, `missing ref for ${label}`);
    const match = line.match(/\[ref=([^\]]+)\]/);
    assert.ok(match, `unrecognized ref: ${line}`);
    return match[1];
  };
  await check('real navigate and exact driver execution', async () => {
    current = ok(await call({ op: 'navigate', newTab: true, viewId: 'primary', url: origin + '/one' }));
    assert.equal(current.url, origin + '/one');
    assert.ok(current.generation);
  });
  await check('navigation generation rotation', async () => {
    const old = current.generation;
    current = ok(await call({ op: 'navigate', viewId: 'primary', expectedGeneration: old, url: origin + '/two' }));
    assert.notEqual(current.generation, old);
  });
  await check('refresh generation rotation', async () => {
    const old = current.generation;
    current = ok(await call({ op: 'refresh', viewId: 'primary', expectedGeneration: old }));
    assert.equal(current.url, origin + '/two'); assert.notEqual(current.generation, old);
  });
  await check('back and forward history generation rotation', async () => {
    const old = current.generation;
    current = ok(await call({ op: 'back', viewId: 'primary', expectedGeneration: old }));
    assert.equal(current.url, origin + '/one'); assert.notEqual(current.generation, old);
    const backGeneration = current.generation;
    current = ok(await call({ op: 'forward', viewId: 'primary', expectedGeneration: backGeneration }));
    assert.equal(current.url, origin + '/two'); assert.notEqual(current.generation, backGeneration);
  });
  const reset = ok(await call({ op: 'snapshot', viewId: 'primary' }));
  current = ok(await call({ op: 'navigate', viewId: 'primary', expectedGeneration: reset.generation, url: origin + '/two' }));
  await check('stale and missing generation reject with zero navigation', async () => {
    const before = await snapshot();
    for (const expectedGeneration of [undefined, 'stale-generation']) {
      denied(await call({ op: 'navigate', viewId: 'primary', expectedGeneration, url: origin + '/forbidden' }), /expectedGeneration|Stale browser view/);
      const after = await snapshot(); assert.equal(after.url, before.url); assert.equal(after.generation, before.generation);
    }
  });
  await check('same-URL tabs retain distinct exact target claims', async () => {
    const secondary = ok(await call({ op: 'navigate', newTab: true, viewId: 'secondary', url: origin + '/two' }));
    const primary = await snapshot();
    const refs = JSON.parse(readFileSync(`/tmp/.sand-browser/views-${display}.json`));
    assert.notEqual(refs.views.primary, refs.views.secondary);
    assert.notEqual(primary.generation, secondary.generation);
    denied(await call({ op: 'refresh', viewId: 'primary', expectedGeneration: secondary.generation }), /Stale browser view/);
    assert.equal((await snapshot()).generation, primary.generation);
  });
  await check('iframe snapshot fill and exact frame targeting', async () => {
    const snap = await snapshot({ frameName: 'fixture-frame', frameUrl: origin + '/frame' });
    ok(await call({ op: 'fill', viewId: 'primary', expectedGeneration: snap.generation, frameName: 'fixture-frame', frameUrl: origin + '/frame', ref: refFor(snap, 'Frame input'), value: 'g11-frame-value' }));
    await observePrimary(async page => {
      assert.equal(await page.frame({ name: 'fixture-frame' }).getByLabel('Frame input').inputValue(), 'g11-frame-value');
    });
    denied(await call({ op: 'snapshot', viewId: 'primary', frameName: 'missing-frame' }), /No browser frame/);
  });
  await check('tracked real download bytes path and digest', async () => {
    const snap = await snapshot();
    const result = ok(await call({ op: 'download', viewId: 'primary', expectedGeneration: snap.generation, ref: refFor(snap, 'Download fixture') }));
    const tracked = JSON.parse(result.data);
    assert.ok(tracked.path.startsWith('/workspace/.fabushi-downloads/'));
    const bytes = readFileSync(tracked.path);
    assert.equal(bytes.toString(), 'g11-download-bytes'); assert.equal(tracked.sha256, sha(bytes)); assert.equal(tracked.sizeBytes, bytes.length);
  });
  await check('authorized real upload and restricted paths', async () => {
    writeFileSync('/workspace/g11-allowed.txt', 'allowed-upload');
    writeFileSync('/tmp/g11-outside.txt', 'outside-upload');
    rmSync('/workspace/g11-escape', { force: true }); symlinkSync('/tmp/g11-outside.txt', '/workspace/g11-escape');
    writeFileSync('/workspace/g11-large.bin', ''); truncateSync('/workspace/g11-large.bin', 26 * 1024 * 1024);
    const snap = await snapshot(); const ref = refFor(snap, 'Upload');
    const upload = filePath => call({ op: 'upload_file', viewId: 'primary', expectedGeneration: snap.generation, ref, filePath });
    ok(await upload('/workspace/g11-allowed.txt'));
    for (const file of ['/tmp/g11-outside.txt', '/workspace/g11-escape', '/workspace/g11-large.bin']) denied(await upload(file), /escapes|limit/);
    await observePrimary(async page => {
      const file = await page.locator('input[type=file]').evaluate(input => ({ name: input.files[0]?.name, size: input.files[0]?.size }));
      assert.equal(file.name, 'g11-allowed.txt'); assert.equal(file.size, Buffer.byteLength('allowed-upload'));
    });
  });
  await check('cookie storage and browser-profile CDP export is denied', async () => {
    const snap = await snapshot();
    for (const method of ['Network.getCookies', 'Network.getAllCookies', 'Storage.getCookies', 'Browser.getVersion', 'Target.getTargets']) denied(await call({ op: 'cdp', viewId: 'primary', expectedGeneration: snap.generation, method }), /denied/);
  });
  await check('indirect cookie/session-secret export is denied', async () => {
    await context.addCookies([{ name: 'g11_synthetic_secret', value: 'synthetic-never-production', url: origin }]);
    const snap = await snapshot();
    const result = await call({ op: 'cdp', viewId: 'primary', expectedGeneration: snap.generation, method: 'Runtime.evaluate', params: { expression: 'document.cookie', returnByValue: true } });
    // Do not copy the returned cookie into evidence even though it is synthetic.
    assert.equal(result.ok, false, 'Runtime.evaluate can export document.cookie; security gate remains blocked');
  });
  await check('tab close and reuse rejects old view generation', async () => {
    const before = await snapshot();
    ok(await call({ op: 'tabs', action: 'close' }));
    const replacement = ok(await call({ op: 'navigate', newTab: true, viewId: 'primary', url: origin + '/two' }));
    assert.notEqual(replacement.generation, before.generation);
    denied(await call({ op: 'refresh', viewId: 'primary', expectedGeneration: before.generation }), /Stale browser view/);
    assert.equal((await snapshot()).generation, replacement.generation);
  });
} catch (error) {
  report.setupFailure = error.message;
} finally {
  await browser?.close().catch(() => {});
  browserProcess?.kill('SIGTERM');
  server?.close();
  report.passed = report.cases.filter(c => c.status === 'passed').length;
  report.failed = report.cases.filter(c => c.status === 'failed').length;
  report.executed = report.cases.length; report.skipped = 0;
  report.exitCode = report.setupFailure || report.failed || report.executed !== 12 ? 1 : 0;
  writeFileSync(join(evidence, 'results.json'), JSON.stringify(report, null, 2));
  if (process.env.GITHUB_STEP_SUMMARY) writeFileSync(process.env.GITHUB_STEP_SUMMARY, `G11 ${report.sourceSha}: ${report.passed} passed, ${report.failed} failed, ${report.executed} executed; exit ${report.exitCode}. Setup: ${report.setupFailure || 'ok'}. Shipping driver only; outer Host/box and extension remain separate.\n`, { flag: 'a' });
  process.exitCode = report.exitCode;
}
