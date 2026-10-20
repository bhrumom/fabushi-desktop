import { chromium, expect, test, type Browser, type Locator, type Page } from '@playwright/test';
import { execFileSync, spawn, type ChildProcess } from 'node:child_process';
import { createHash } from 'node:crypto';
import { lstat, mkdir, mkdtemp, readFile, readdir, rm, stat, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

const enabled = process.env.MINIAPP_REAL_ACCEPTANCE === '1';
const executable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || '';
const sourceSha = process.env.OBF_SOURCE_SHA?.trim() || '';
const expectedSourceSha = process.env.OBF_EXPECTED_SOURCE_SHA?.trim() || sourceSha;
const evidenceRoot = process.env.MINIAPP_EVIDENCE_DIR?.trim()
  || path.join(tmpdir(), `fabushi-miniapp-acceptance-${sourceSha.slice(0, 12) || 'unknown'}`);
const appId = 'org.fabushi.global-dandelion';
const packageVersion = process.env.MINIAPP_PACKAGE_VERSION || '0.3.3';
const packageDigest = process.env.MINIAPP_PACKAGE_DIGEST || '4f53dd5df129adb41feadf86016a5923ba792a9a7636a37694965ea4ad28f4db';
const preferenceKey = 'global-dandelion-display-v1';
const maxInputBytes = 16 * 1024 * 1024;
const harnessSource = process.env.FABUSHI_MINIAPP_HARNESS_SOURCE || process.env.GITHUB_SHA || sourceSha;

async function verifyCiBootstrapInput(sessionPath: string): Promise<void> {
  expect(sessionPath, 'ordinary CI session bootstrap input is required').toBeTruthy();
  const info = await lstat(sessionPath);
  expect(info.mode & 0o777, 'bootstrap transport must be owner-private').toBe(0o600);
  expect(info.uid).toBe(process.getuid!());
  if (process.env.FABUSHI_CI_SESSION_DELIVERY === 'private-fifo') {
    expect(process.env.FABUSHI_ACCOUNT_ENVIRONMENT).toBe('ci-isolated');
    expect(sessionPath.startsWith(`${process.env.DFC_RAM_VOLUME}/factory-private/`)).toBe(true);
    expect(info.isFIFO(), 'factory credentials must not become a disk JSON session').toBe(true);
  } else {
    expect(process.env.FABUSHI_CI_SESSION_DELIVERY || 'private-file').toBe('private-file');
    expect(info.isFile()).toBe(true);
    expect(info.size).toBeGreaterThan(0);
    expect(info.size).toBeLessThanOrEqual(64 * 1024);
  }
}

type Target = { readonly id?: string; readonly type?: string; readonly url?: string };
type NetworkService = { readonly name: string; readonly wasEnabled: boolean };

function isMainRenderer(url: string): boolean {
  if (url.startsWith('app://bundle/')) return true;
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'file:' && parsed.pathname.endsWith('/dist/renderer/index.html');
  } catch { return false; }
}

async function waitForRenderer(appDataDir: string): Promise<{ page: Page; browser: Browser }> {
  const deadline = Date.now() + 120_000;
  let lastError = 'no CDP target';
  while (Date.now() < deadline) {
    try {
      const activePort = await readFile(path.join(appDataDir, 'DevToolsActivePort'), 'utf8');
      const port = Number(activePort.trim().split(/\r?\n/u)[0]);
      if (!Number.isInteger(port) || port < 1 || port > 65_535) throw new Error('invalid packaged CDP port');
      const response = await fetch(`http://127.0.0.1:${port}/json/list`, { signal: AbortSignal.timeout(2_000) });
      if (!response.ok) throw new Error(`CDP target registry HTTP ${response.status}`);
      const payload: unknown = await response.json();
      const targets = Array.isArray(payload)
        ? payload.filter((item): item is Target => item != null && typeof item === 'object')
        : [];
      if (!targets.some((target) => target.type === 'page' && typeof target.url === 'string' && isMainRenderer(target.url))) {
        await new Promise((resolve) => setTimeout(resolve, 250));
        continue;
      }
      const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`, { timeout: 30_000 });
      const page = browser.contexts().flatMap((context) => context.pages()).find((candidate) => isMainRenderer(candidate.url()));
      if (page) return { page, browser };
      await browser.close().catch(() => undefined);
      lastError = 'packaged renderer target was not exposed by CDP';
    } catch (error) {
      lastError = error instanceof Error ? error.message : 'CDP probe failed';
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Signed production renderer did not become available: ${lastError}`);
}

async function launchSignedApp(appDataDir: string, ciSessionFile = process.env.FABUSHI_CI_ACCOUNT_SESSION_FILE): Promise<{ process: ChildProcess; browser: Browser; page: Page }> {
  const child = spawn(executable, ['--remote-debugging-address=127.0.0.1', '--remote-debugging-port=0'], {
    env: {
      ...process.env,
      FABUSHI_APP_DATA: appDataDir,
      SAND_USER_DATA_DIR: appDataDir,
      OBF_SOURCE_SHA: sourceSha,
      FABUSHI_FEATURE_HOST_MODE: 'production',
      FABUSHI_E2E: '0',
      FABUSHI_CI_ACCOUNT_SESSION_FILE: ciSessionFile,
    },
    stdio: ['ignore', 'ignore', 'ignore'],
  });
  const earlyExit = new Promise<never>((_, reject) => child.once('exit', (code, signal) => {
    reject(new Error(`Signed app exited before renderer binding (code=${code ?? 'null'}, signal=${signal ?? 'null'}).`));
  }));
  try {
    const binding = await Promise.race([waitForRenderer(appDataDir), earlyExit]);
    return { process: child, ...binding };
  } catch (error) {
    if (child.exitCode == null && child.signalCode == null) child.kill('SIGTERM');
    throw error;
  }
}

async function quitSignedApp(child: ChildProcess, browser: Browser): Promise<void> {
  await browser.close().catch(() => undefined);
  if (child.exitCode != null || child.signalCode != null) return;
  child.kill('SIGTERM');
  const exited = await Promise.race([
    new Promise<boolean>((resolve) => child.once('exit', () => resolve(true))),
    new Promise<boolean>((resolve) => setTimeout(() => resolve(false), 10_000)),
  ]);
  if (!exited && child.exitCode == null && child.signalCode == null) {
    child.kill('SIGKILL');
    await Promise.race([
      new Promise<void>((resolve) => child.once('exit', () => resolve())),
      new Promise<void>((resolve) => setTimeout(resolve, 5_000)),
    ]);
  }
}

async function waitForProductionLogin(page: Page): Promise<void> {
  type LoginPhase = 'onboarding' | 'login' | 'browser-waiting' | 'ready' | 'waiting';
  const workspace = page.getByTestId('messenger-workspace');
  const signInLanding = page.getByRole('main', { name: 'Fabushi' });
  const onboarding = page.locator('.sand-onboarding[data-step]');
  const readAccountKind = async (): Promise<string | null> => page.evaluate(async () => {
    const desktop = (window as typeof window & {
      desktop?: { cursorAccount?: { getStatus?: () => Promise<{ kind?: unknown }> } };
    }).desktop;
    if (typeof desktop?.cursorAccount?.getStatus !== 'function') return null;
    try {
      const status = await desktop.cursorAccount.getStatus();
      return typeof status?.kind === 'string' ? status.kind : null;
    } catch { return null; }
  });
  const readPhase = async (): Promise<LoginPhase> => {
    if (!(await workspace.count())) return 'waiting';
    const accountKind = await readAccountKind();
    if (accountKind === 'logged-in') {
      if (await onboarding.count()) return 'onboarding';
      if (await signInLanding.count()) return 'waiting';
      return 'ready';
    }
    if (accountKind === 'logging-in') return 'browser-waiting';
    if (accountKind === 'logged-out') return 'login';
    if (await signInLanding.count()) {
      return await signInLanding.getByRole('button', { name: 'Sign in', exact: true }).count() ? 'login' : 'browser-waiting';
    }
    return 'waiting';
  };
  const hostHydrated = async (): Promise<boolean> => {
    if (!(await page.getByRole('status', { name: 'Connected' }).first().isVisible().catch(() => false))) return false;
    const sidebar = page.locator('aside[aria-label="Agents"]');
    return await sidebar.isVisible().catch(() => false) && await sidebar.locator('button.sand-agent-item').count() > 0;
  };
  await expect(workspace).toBeVisible({ timeout: 35_000 });
  await expect(workspace).toHaveAttribute('data-product-shell', 'agent');
  await expect(workspace).toHaveAttribute('data-runtime', 'electron');
  for (let attempt = 0; attempt < 18; attempt += 1) {
    await expect.poll(readPhase, { timeout: 15_000 }).not.toBe('waiting');
    const phase = await readPhase();
    if (phase === 'onboarding') {
      const step = await onboarding.getAttribute('data-step');
      if (step === 'meet' || step === 'computer-demo' || step === 'jobs' || step === 'tools') {
        await onboarding.getByRole('button', { name: 'Next', exact: true }).last().click();
        continue;
      }
      if (step === 'create') {
        const name = onboarding.getByPlaceholder('New Bot');
        if (!(await name.inputValue()).trim()) await name.fill(`Mini App Acceptance ${Date.now()}`);
        await onboarding.getByRole('button', { name: 'Get started', exact: true }).click();
        continue;
      }
      if (step === 'hand-off') {
        if (await onboarding.getByRole('button', { name: 'Try again', exact: true }).count()) {
          throw new Error('Production signed-in onboarding could not complete its native hand-off.');
        }
        await expect.poll(readPhase, { timeout: 90_000 }).not.toBe('onboarding');
        continue;
      }
      throw new Error(`Unexpected production onboarding step: ${step ?? 'unknown'}`);
    }
    if (phase === 'login') {
      await signInLanding.getByRole('button', { name: 'Sign in', exact: true }).click();
      continue;
    }
    if (phase === 'browser-waiting') {
      await expect.poll(readPhase, {
        timeout: 45_000,
        message: 'Packaged acceptance needs the workflow-provisioned production CI login; no test auth fallback is allowed.',
      }).not.toBe('browser-waiting');
      continue;
    }
    if (phase === 'ready') {
      await expect(signInLanding).toHaveCount(0);
      await expect(onboarding).toHaveCount(0);
      await expect.poll(hostHydrated, { timeout: 45_000 }).toBe(true);
      return;
    }
  }
  throw new Error('Packaged app did not finish production login/onboarding and Coordinator hydration.');
}

async function openPlugins(page: Page): Promise<{ dialog: Locator; section: Locator }> {
  await page.getByRole('button', { name: 'Plugins', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Plugins' });
  await expect(dialog).toBeVisible({ timeout: 20_000 });
  const section = dialog.locator('section[aria-label="离线小程序"]');
  await expect(section).toBeVisible();
  await section.getByRole('button', { name: '刷新市场与安装状态' }).click();
  return { dialog, section };
}

async function refreshUntilCatalogItem(page: Page): Promise<{ dialog: Locator; section: Locator; row: Locator }> {
  const { dialog, section } = await openPlugins(page);
  const row = section.locator('li').filter({ hasText: packageDigest });
  await expect(row, 'public approved catalog must expose the exact pinned package').toHaveCount(1, { timeout: 60_000 });
  await expect(row).toContainText(packageVersion);
  await expect(row).toContainText('bhrumom');
  return { dialog, section, row };
}

async function findMiniAppPage(browser: Browser): Promise<Page> {
  await expect.poll(async () => browser.contexts().flatMap((context) => context.pages())
    .filter((page) => page.url().startsWith('fabushi-miniapp://')).length,
  { timeout: 30_000, message: 'Host launch intent must open the real isolated Mini App window.' }).toBeGreaterThan(0);
  const page = browser.contexts().flatMap((context) => context.pages()).find((candidate) => candidate.url().startsWith('fabushi-miniapp://'));
  if (!page) throw new Error('Isolated Mini App target disappeared after launch.');
  return page;
}

function bytesLabel(size: number): string {
  return size < 1024 ? `${size} B` : size < 1024 ** 2 ? `${(size / 1024).toFixed(1)} KiB` : `${(size / 1024 ** 2).toFixed(1)} MiB`;
}

async function verifyWasmFileReport(page: Page, inputFile: string, inputBytes: number): Promise<void> {
  await expect(page.locator('#status')).toContainText('本地 Rust 内核已就绪。', { timeout: 45_000 });
  const fileInput = page.locator('#files');
  await expect(fileInput, 'the signed app must provide its genuine Web file input').toHaveCount(1);
  await fileInput.setInputFiles(inputFile);
  await expect(page.locator('#selection-summary')).toContainText('1 个文件');
  await expect(page.getByRole('button', { name: '开始离线校验', exact: true })).toBeEnabled();
  await page.getByRole('button', { name: '开始离线校验', exact: true }).click();
  await expect(page.locator('#status')).toHaveText('本地校验完成。没有网络发送；校验和不代表身份认证。', { timeout: 120_000 });
  await expect(page.locator('#file-count')).toHaveText('1');
  const packetCount = Number(await page.locator('#packet-count').textContent());
  expect(Number.isSafeInteger(packetCount) && packetCount > 0, 'Wasm round-trip must produce verified packet(s)').toBe(true);
  await expect(page.locator('#byte-count')).toHaveText(bytesLabel(inputBytes));
  const result = page.getByRole('list', { name: '文件结果' });
  await expect(result).toContainText(path.basename(inputFile));
  await expect(result).toContainText('完整校验');
  await expect(page.getByRole('button', { name: '导出报告', exact: true })).toBeEnabled();
}

async function verifyBundledNativeHost(page: Page): Promise<boolean> {
  if (process.env.MINIAPP_REQUIRE_NATIVE_ENGINE !== '1') return false;
  const observed = await page.evaluate(async () => {
    const bridge = (globalThis as unknown as { fabushi?: { invoke(tool: string, args?: Record<string, unknown>): Promise<{ ok: boolean; data?: Record<string, unknown> }> } }).fabushi;
    if (!bridge) throw new Error('INSTALLED_NATIVE_BRIDGE_MISSING');
    return {
      capabilities: await bridge.invoke('offline.capabilities'),
      config: await bridge.invoke('config.check'),
      diagnostic: await bridge.invoke('diagnostics.selfTest'),
    };
  });
  expect(observed.capabilities.ok).toBe(true);
  const capabilities = observed.capabilities.data?.capabilities as string[];
  for (const capability of ['selection', 'scan', 'transfer', 'config', 'diagnostics']) expect(capabilities).toContain(capability);
  expect(capabilities).not.toContain('hotspot');
  expect(capabilities).not.toContain('platform');
  expect(observed.config.ok).toBe(true);
  expect((observed.config.data?.scanner as Record<string, unknown>).max_files).toBe(128);
  expect((observed.config.data?.transport as Record<string, unknown>).mode).toBe('loopback');
  expect(observed.diagnostic.ok).toBe(true);
  expect(observed.diagnostic.data?.status).toBe('completed');
  expect(observed.diagnostic.data?.verified_payload_bytes).toBe(65537);
  return true;
}

async function exportAndVerifyReport(page: Page, appPid: number, downloadDir: string, inputFile: string, inputBytes: number): Promise<void> {
  if (process.platform !== 'darwin' || process.env.GITHUB_ACTIONS !== 'true') throw new Error('Actions-only native Save panel gate');
  const reportDir = await mkdtemp(path.join(downloadDir, 'report-'));
    await page.getByRole('button', { name: '导出报告', exact: true }).click();
    execFileSync('/usr/bin/osascript', [path.resolve('../.github/scripts/miniapp-save-report.applescript'), String(appPid), reportDir], {
      encoding: 'utf8', timeout: 45_000, stdio: ['ignore', 'pipe', 'pipe'],
    });
    const reportFilename = 'global-dandelion-local-report.json';
    const reportPath = path.join(reportDir, reportFilename);
    await expect.poll(async () => {
      try { return (await stat(reportPath)).isFile(); }
      catch { return false; }
    }, { timeout: 10_000, message: 'completed native download must materialize its report file' }).toBe(true);
    const reportStat = await stat(reportPath);
    expect(reportStat.size).toBeGreaterThan(0);
    expect(reportStat.size).toBeLessThanOrEqual(1024 * 1024);
    expect(await readdir(reportDir)).toContain(reportFilename);
    const report: unknown = JSON.parse(await readFile(reportPath, 'utf8'));
    expect(report).toMatchObject({
      schemaVersion: 1,
      appVersion: '0.3.3',
      verification: 'local-packet-roundtrip',
      networkSent: false,
      status: 'complete',
      verifiedBytes: inputBytes,
    });
    expect(report).toEqual(expect.objectContaining({ files: expect.any(Array), packets: expect.any(Number) }));
    const data = report as {
      files: Array<{ name: unknown; size: unknown; packets: unknown; verified: unknown }>;
      packets: number;
    };
    expect(data.files).toHaveLength(1);
    expect(data.files[0].name).toBe(path.basename(inputFile));
    expect(data.files[0].size).toBe(inputBytes);
    expect(data.files[0].verified).toBe(true);
    expect(Number.isSafeInteger(data.files[0].packets) && Number(data.files[0].packets) > 0).toBe(true);
    expect(data.packets).toBe(data.files[0].packets);
}

async function setAndVerifyDisplayPreference(page: Page, shouldBeChecked: boolean): Promise<void> {
  const disclosure = page.getByText('能力、数据与来源', { exact: true });
  const preference = page.getByRole('checkbox', { name: '记住显示偏好', exact: true });
  if (!(await preference.isVisible())) await disclosure.click();
  await expect(preference).toBeVisible();
  if (shouldBeChecked) await preference.check();
  else await expect(preference).toBeChecked();
  if (shouldBeChecked) {
    await expect(page.locator('#storage-status')).toHaveText('已保存当前小程序显示偏好，不包含文件信息。');
  }
  await expect.poll(() => page.evaluate((key) => localStorage.getItem(key), preferenceKey)).toBe('true');
}

function networksetup(args: readonly string[]): string {
  return execFileSync('sudo', ['-n', '/usr/sbin/networksetup', ...args], {
    encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], timeout: 15_000,
  }).trim();
}

function enabledNetworkServices(): NetworkService[] {
  const lines = networksetup(['-listallnetworkservices']).split(/\r?\n/u).slice(1);
  return lines.flatMap((line) => {
    const wasEnabled = !line.startsWith('*');
    const name = line.replace(/^\*\s*/u, '').trim();
    return name.length > 0 && wasEnabled ? [{ name, wasEnabled }] : [];
  });
}

function disableRunnerNetwork(): NetworkService[] {
  if (process.platform !== 'darwin' || process.env.GITHUB_ACTIONS !== 'true') {
    throw new Error('Offline cold-launch acceptance requires the macOS GitHub Actions runner.');
  }
  const services = enabledNetworkServices();
  if (services.length === 0) throw new Error('No enabled macOS network service was found; refusing to claim OS offline state.');
  const changed: NetworkService[] = [];
  try {
    for (const service of services) {
      networksetup(['-setnetworkserviceenabled', service.name, 'off']);
      changed.push(service);
    }
    if (enabledNetworkServices().length !== 0) throw new Error('One or more macOS network services remained enabled.');
    return changed;
  } catch (error) {
    restoreRunnerNetwork(changed);
    throw error;
  }
}

function restoreRunnerNetwork(services: readonly NetworkService[]): void {
  const failures: string[] = [];
  for (const service of [...services].reverse()) {
    try { networksetup(['-setnetworkserviceenabled', service.name, service.wasEnabled ? 'on' : 'off']); }
    catch { failures.push(service.name); }
  }
  if (failures.length > 0) throw new Error(`Failed to restore ${failures.length} runner network service(s).`);
}

async function expectInternetUnavailable(): Promise<void> {
  let reached = false;
  try {
    await fetch('https://api.ombhrum.com/v1/extensions', { signal: AbortSignal.timeout(5_000), cache: 'no-store' });
    reached = true;
  } catch { /* OS networking must make this real outbound probe fail. */ }
  expect(reached, 'offline phase must disable OS networking, not only Playwright context networking').toBe(false);
}

test.describe('signed production Mini App install and offline lifecycle', () => {
  test.describe.configure({ retries: 0 });
  test.skip(!enabled, 'Set MINIAPP_REAL_ACCEPTANCE=1 only in the signed macOS packaged Actions job.');

  test('real Plugins install, Wasm report, offline cold launch, code removal and online reinstall', async () => {
    test.setTimeout(20 * 60_000);
    expect(process.platform, 'MU-07 currently requires the signed macOS candidate').toBe('darwin');
    expect(process.env.GITHUB_ACTIONS, 'this test may only alter network services on an Actions runner').toBe('true');
    expect(executable, 'FABUSHI_ELECTRON_EXECUTABLE is required').toBeTruthy();
    expect(sourceSha).toMatch(/^[0-9a-f]{40}$/u);
    expect(expectedSourceSha).toMatch(/^[0-9a-f]{40}$/u);
    expect(sourceSha, 'the signed app must match the exact CI source revision').toBe(expectedSourceSha);
    expect(process.env.FABUSHI_FEATURE_HOST_MODE || '').toBe('production');
    expect(process.env.FABUSHI_E2E || '').not.toBe('1');
    const ciSessionPath = process.env.FABUSHI_CI_ACCOUNT_SESSION_FILE?.trim() || '';
    expect(ciSessionPath, 'reuse the workflow-provisioned production CI account session').toBeTruthy();
    await verifyCiBootstrapInput(ciSessionPath);
    const fixtureDir = await mkdtemp(path.join(tmpdir(), 'fabushi-miniapp-fixture-'));
    const inputFile = path.join(fixtureDir, 'mu07-packet-fixture.txt');
    const fixtureBytes = Buffer.from(('FABUSHI MU-07 offline packet acceptance\n0123456789abcdef\n').repeat(256), 'ascii');
    expect(fixtureBytes.length).toBeGreaterThan(0);
    expect(fixtureBytes.length).toBeLessThanOrEqual(maxInputBytes);
    await writeFile(inputFile, fixtureBytes, { flag: 'wx' });
    const fixtureStat = await stat(inputFile);
    expect(fixtureStat.isFile()).toBe(true);
    expect(fixtureStat.size).toBe(fixtureBytes.length);
    const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-miniapp-app-data-'));
    await mkdir(evidenceRoot, { recursive: true });
    const evidenceDir = await mkdtemp(path.join(evidenceRoot, 'run-'));
    const evidence = {
      sourceSha,
      binarySource: sourceSha,
      harnessSource,
      appId,
      version: packageVersion,
      packageDigest,
      inputBytes: fixtureStat.size,
      launchModes: [] as string[],
      wasmLoaded: [] as boolean[],
      reportObserved: [] as boolean[],
      reportVerified: [] as boolean[],
      bundledNativeVerified: [] as boolean[],
      installedAfterColdLaunch: false,
      codeRemoved: false,
      reinstalledOnline: false,
      settingEntriesRetained: 0,
    };
    let app: Awaited<ReturnType<typeof launchSignedApp>> | null = null;
    let networkServices: NetworkService[] = [];
    try {
      app = await launchSignedApp(appDataDir);
      await waitForProductionLogin(app.page);
      let catalog = await refreshUntilCatalogItem(app.page);
      const installButton = catalog.row.getByRole('button', { name: '安装并验证', exact: true });
      await expect(installButton).toBeEnabled();
      await installButton.click();
      await expect(catalog.row).toContainText('已安装', { timeout: 90_000 });
      await catalog.row.getByRole('button', { name: '打开', exact: true }).click();
      let mini = await findMiniAppPage(app.browser);
      evidence.bundledNativeVerified.push(await verifyBundledNativeHost(mini));
      await verifyWasmFileReport(mini, inputFile, fixtureStat.size);
      await exportAndVerifyReport(mini, app.process.pid!, evidenceDir, inputFile, fixtureStat.size);
      evidence.launchModes.push('online-install');
      evidence.wasmLoaded.push(true);
      evidence.reportObserved.push(true);
      evidence.reportVerified.push(true);
      await setAndVerifyDisplayPreference(mini, true);
      evidence.settingEntriesRetained = 1;
      await writeFile(path.join(evidenceDir, 'online-installed.png'), await mini.screenshot());
      await quitSignedApp(app.process, app.browser);
      app = null;

      networkServices = disableRunnerNetwork();
      await expectInternetUnavailable();
      app = await launchSignedApp(appDataDir);
      await waitForProductionLogin(app.page);
      catalog = await refreshUntilCatalogItem(app.page);
      await expect(catalog.row).toContainText('已安装', { timeout: 60_000 });
      evidence.installedAfterColdLaunch = true;
      await catalog.row.getByRole('button', { name: '打开', exact: true }).click();
      mini = await findMiniAppPage(app.browser);
      evidence.bundledNativeVerified.push(await verifyBundledNativeHost(mini));
      await verifyWasmFileReport(mini, inputFile, fixtureStat.size);
      await exportAndVerifyReport(mini, app.process.pid!, evidenceDir, inputFile, fixtureStat.size);
      evidence.launchModes.push('os-network-disabled-cold-launch');
      evidence.wasmLoaded.push(true);
      evidence.reportObserved.push(true);
      evidence.reportVerified.push(true);
      await setAndVerifyDisplayPreference(mini, false);
      await app.page.bringToFront();
      await catalog.row.getByRole('button', { name: '移除代码，保留数据', exact: true }).click();
      await expect(catalog.section.locator('li').filter({ hasText: packageDigest }))
        .toHaveCount(0, { timeout: 90_000 });
      await expect.poll(() => Promise.resolve(mini.isClosed()), {
        timeout: 90_000,
        message: 'removing code must close the still-open Mini App window',
      }).toBe(true);
      evidence.codeRemoved = true;
      await expect.poll(async () => app!.browser.contexts().flatMap((context) => context.pages())
        .filter((page) => page.url().startsWith('fabushi-miniapp://')).length,
      { timeout: 10_000, message: 'remove-code must leave no live Mini App window' }).toBe(0);

      await quitSignedApp(app.process, app.browser);
      app = null;
      restoreRunnerNetwork(networkServices);
      networkServices = [];
      await expect.poll(async () => {
        try { await fetch('https://api.ombhrum.com/v1/extensions', { signal: AbortSignal.timeout(5_000), cache: 'no-store' }); return true; }
        catch { return false; }
      }, { timeout: 60_000, message: 'runner network must be restored before the online reinstall phase' }).toBe(true);

      app = await launchSignedApp(appDataDir);
      await waitForProductionLogin(app.page);
      catalog = await refreshUntilCatalogItem(app.page);
      await expect(catalog.row).toContainText('未安装');
      await catalog.row.getByRole('button', { name: '安装并验证', exact: true }).click();
      await expect(catalog.row).toContainText('已安装', { timeout: 90_000 });
      evidence.reinstalledOnline = true;
      await catalog.row.getByRole('button', { name: '打开', exact: true }).click();
      mini = await findMiniAppPage(app.browser);
      await setAndVerifyDisplayPreference(mini, false);
      evidence.launchModes.push('online-reinstall-retained-app-data');
      await writeFile(path.join(evidenceDir, 'reinstalled.png'), await mini.screenshot());
      await writeFile(path.join(evidenceDir, 'acceptance.json'), JSON.stringify(evidence, null, 2));
    } catch (error) {
      // Disposable Actions desktop only; never collect the native credential store.
      execFileSync('/usr/sbin/screencapture', ['-x', path.join(evidenceDir, 'failure-desktop.png')], { timeout: 10_000 });
      await writeFile(path.join(evidenceDir, 'report-directories.json'), JSON.stringify(
        await Promise.all((await readdir(evidenceDir)).filter((name) => name.startsWith('report-'))
          .map(async (name) => ({ directory: name, members: await readdir(path.join(evidenceDir, name)) }))), null, 2));
      throw error;
    } finally {
      try {
        if (app) await quitSignedApp(app.process, app.browser);
      } finally {
        try {
          if (networkServices.length > 0) restoreRunnerNetwork(networkServices);
        } finally {
          await rm(appDataDir, { recursive: true, force: true });
          await rm(fixtureDir, { recursive: true, force: true });
        }
      }
    }
  });

  test('real account departure and two-account installed code and browser data isolation', async () => {
    test.skip(process.env.MINIAPP_ACCOUNT_SWITCH_ACCEPTANCE !== '1', 'Fixed-binary diagnostics do not claim the shipping real-account-switch gate.');
    test.setTimeout(12 * 60_000);
    expect(process.platform).toBe('darwin');
    expect(process.env.GITHUB_ACTIONS).toBe('true');
    expect(process.env.FABUSHI_ACCOUNT_ENVIRONMENT, 'factory identities must remain in the explicit isolated issuer namespace').toBe('ci-isolated');
    expect(process.env.FABUSHI_API_BASE_URL).toBe('https://fabushi-ci-isolated-auth.bhrumom.workers.dev');
    expect(process.env.FABUSHI_CI_SESSION_DELIVERY).toBe('private-fifo');
    const sessionA = process.env.FABUSHI_CI_ACCOUNT_SESSION_FILE?.trim() || '';
    const sessionB = process.env.FABUSHI_CI_SECOND_ACCOUNT_SESSION_FILE?.trim() || '';
    for (const sessionPath of [sessionA, sessionB]) {
      expect(sessionPath, 'two real protected CI sessions are required; no publisher-session fallback').toBeTruthy();
      await verifyCiBootstrapInput(sessionPath);
    }
    const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-miniapp-account-isolation-'));
    await mkdir(evidenceRoot, { recursive: true });
    const evidenceDir = await mkdtemp(path.join(evidenceRoot, 'account-switch-'));
    let app: Awaited<ReturnType<typeof launchSignedApp>> | null = null;
    const accountReference = async (page: Page): Promise<string> => {
      const id = await page.evaluate(async () => {
        const bridge = (window as typeof window & { desktop?: { cursorAccount?: { getStatus?: () => Promise<{ kind?: string; authId?: string }> } } }).desktop;
        const status = await bridge?.cursorAccount?.getStatus?.();
        if (status?.kind !== 'logged-in' || !status.authId) throw new Error('Real native account identity is unavailable.');
        return status.authId;
      });
      return createHash('sha256').update(id).digest('hex');
    };
    try {
      app = await launchSignedApp(appDataDir, sessionA);
      await waitForProductionLogin(app.page);
      const accountA = await accountReference(app.page);
      let catalog = await refreshUntilCatalogItem(app.page);
      await catalog.row.getByRole('button', { name: '安装并验证', exact: true }).click();
      await expect(catalog.row).toContainText('已安装', { timeout: 90_000 });
      await catalog.row.getByRole('button', { name: '打开', exact: true }).click();
      let mini = await findMiniAppPage(app.browser);
      const originA = new URL(mini.url()).host;
      await setAndVerifyDisplayPreference(mini, true);
      await app.page.bringToFront();
      await catalog.dialog.getByRole('button', { name: 'Close', exact: true }).click();
      await app.page.getByRole('button', { name: 'Account', exact: true }).click();
      await app.page.getByText('Settings', { exact: true }).click();
      await app.page.getByRole('dialog', { name: 'Fabushi settings' }).getByRole('button', { name: 'Sign Out', exact: true }).click();
      await expect.poll(() => mini.isClosed(), { timeout: 30_000, message: 'real native account departure must destroy the live mini-app window' }).toBe(true);
      await expect.poll(async () => app!.page.evaluate(async () => {
        const bridge = (window as typeof window & { desktop?: { cursorAccount?: { getStatus?: () => Promise<{ kind?: string }> } } }).desktop;
        return (await bridge?.cursorAccount?.getStatus?.())?.kind;
      }), { timeout: 30_000, message: 'Native logout must not replay the cached CI bootstrap session' }).toBe('logged-out');
      await quitSignedApp(app.process, app.browser);
      app = await launchSignedApp(appDataDir, sessionB);
      await waitForProductionLogin(app.page);
      const accountB = await accountReference(app.page);
      expect(accountB, 'the second session must belong to another real account').not.toBe(accountA);
      catalog = await refreshUntilCatalogItem(app.page);
      await expect(catalog.row).toContainText('未安装');
      await catalog.row.getByRole('button', { name: '安装并验证', exact: true }).click();
      await expect(catalog.row).toContainText('已安装', { timeout: 90_000 });
      await catalog.row.getByRole('button', { name: '打开', exact: true }).click();
      mini = await findMiniAppPage(app.browser);
      expect(new URL(mini.url()).host).not.toBe(originA);
      expect(await mini.evaluate((key) => localStorage.getItem(key), preferenceKey), 'B must not see A browser data').toBeNull();
      await app.page.bringToFront();
      await catalog.row.getByRole('button', { name: '移除代码，保留数据', exact: true }).click();
      await expect.poll(() => mini.isClosed(), { timeout: 90_000 }).toBe(true);
      await quitSignedApp(app.process, app.browser);
      app = await launchSignedApp(appDataDir, sessionA);
      await waitForProductionLogin(app.page);
      expect(await accountReference(app.page)).toBe(accountA);
      catalog = await refreshUntilCatalogItem(app.page);
      await expect(catalog.row).toContainText('已安装');
      await catalog.row.getByRole('button', { name: '打开', exact: true }).click();
      mini = await findMiniAppPage(app.browser);
      expect(new URL(mini.url()).host).toBe(originA);
      await setAndVerifyDisplayPreference(mini, false);
      await writeFile(path.join(evidenceDir, 'returned-account-A.png'), await mini.screenshot());
      await writeFile(path.join(evidenceDir, 'account-isolation.json'), JSON.stringify({ sourceSha, harnessSource, issuerEnvironment: 'ci-isolated', sessionDelivery: 'private-fifo', accountA, accountB, realDepartureClosedWindow: true, distinctInstallations: true, distinctOrigins: true, retainedAccountASetting: true }, null, 2));
    } finally {
      if (app) await quitSignedApp(app.process, app.browser);
      await rm(appDataDir, { recursive: true, force: true });
    }
  });
});
