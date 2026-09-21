import { _electron as electron, expect, test, type ElectronApplication, type Locator, type Page, type TestInfo } from '@playwright/test';
import { createHash } from 'node:crypto';
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

const referenceCropSha256 = '0a94bcf48630f4d872bab2e765df8df396026ac08d2b6c3b5fe64b112c1d268d';
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || '';
const referenceScreenshot = process.env.OBF_REFERENCE_SCREENSHOT?.trim() || '';
const realAcceptance = process.env.OBF_REAL_ACCEPTANCE === '1';
const sourceSha = (process.env.OBF_SOURCE_SHA || process.env.GITHUB_SHA || '').trim().toLowerCase();
const canonicalMainSha = (process.env.OBF_CANONICAL_MAIN_SHA || '').trim().toLowerCase();
const visualThreshold = Number(process.env.OBF_MAX_DIFF_PIXEL_RATIO || '0');
const pixelThreshold = Number(process.env.OBF_PIXEL_COLOR_THRESHOLD || '0');
const coworkers = [
  ['Chief', 'Chief of staff'],
  ['Research', 'Research and evidence'],
  ['Builder', 'Product engineering'],
  ['Launch', 'Go-to-market'],
] as const;

test.skip(!realAcceptance, 'OBF packaged acceptance runs only with OBF_REAL_ACCEPTANCE=1 against a real packaged Fabushi runtime.');

type LifecycleSample = { at: number; status: string; text: string };
type RuntimeLog = { at: number; source: 'page-console' | 'page-error' | 'app-stdout' | 'app-stderr'; text: string };
type Box = { x: number; y: number; width: number; height: number };
type GeometryReport = {
  viewport: { width: number; height: number; dark: boolean };
  navigation: Box | null;
  roster: Box | null;
  header: Box | null;
  transcript: Box | null;
  finalCard: Box | null;
  table: Box | null;
  sourceFiles: Box | null;
  attachmentCard: Box | null;
  hoverActions: Box | null;
  composer: Box | null;
  composerInput: Box | null;
  peerRows: Record<string, Box | null>;
  rosterAvatars: Record<string, Box | null>;
  peerRowGaps: number[];
};

type RegionDiff = { differingPixels: number; totalPixels: number; differingPixelRatio: number };
type VisualDiffReport = {
  width: number;
  height: number;
  pixelThreshold: number;
  maxDiffPixelRatio: number;
  global: RegionDiff & { zeroDiff: boolean };
  regions: Record<string, RegionDiff>;
  residualRegions: Array<{ name: string; differingPixelRatio: number; differingPixels: number; totalPixels: number }>;
};

function assertProductionEvidenceEnvironment(): void {
  if (!packagedExecutable) throw new Error('FABUSHI_ELECTRON_EXECUTABLE is required for packaged acceptance');
  if (!referenceScreenshot) throw new Error('OBF_REFERENCE_SCREENSHOT is required; static or synthetic replacement is forbidden');
  if (!/^[0-9a-f]{40}$/.test(sourceSha)) throw new Error('OBF_SOURCE_SHA or GITHUB_SHA must provide the exact 40-character source SHA');
  if (!/^[0-9a-f]{40}$/.test(canonicalMainSha)) throw new Error('OBF_CANONICAL_MAIN_SHA must provide the exact post-merge canonical main SHA');
  if (sourceSha !== canonicalMainSha) throw new Error(`Packaged source SHA ${sourceSha} does not equal canonical main ${canonicalMainSha}`);
  if (visualThreshold !== 0) throw new Error('OBF_MAX_DIFF_PIXEL_RATIO must be exactly 0 for literal 1:1 acceptance');
  if (pixelThreshold !== 0) throw new Error('OBF_PIXEL_COLOR_THRESHOLD must be exactly 0 for literal 1:1 acceptance');
  const inheritedMode = (process.env.FABUSHI_FEATURE_HOST_MODE || '').trim().toLowerCase();
  if (['test', 'mock', 'stub'].includes(inheritedMode)) {
    throw new Error(`Real packaged acceptance refuses FABUSHI_FEATURE_HOST_MODE=${inheritedMode}; mock/test host evidence is inadmissible`);
  }
  if (process.env.FABUSHI_E2E === '1') {
    throw new Error('Real packaged acceptance refuses FABUSHI_E2E=1; the packaged runtime must use production defaults');
  }
}

async function launchPackaged(appDataDir: string, videoDir: string): Promise<ElectronApplication> {
  const launchEnv: Record<string, string> = Object.fromEntries(
    Object.entries(process.env).filter((entry): entry is [string, string] => typeof entry[1] === 'string'),
  );
  launchEnv.FABUSHI_APP_DATA = appDataDir;
  delete launchEnv.FABUSHI_FEATURE_HOST_MODE;
  delete launchEnv.FABUSHI_E2E;
  delete launchEnv.MAHAYANA_APP_HOST_BIN;
  return electron.launch({
    executablePath: packagedExecutable,
    args: [],
    env: launchEnv,
    recordVideo: { dir: videoDir, size: { width: 1671, height: 937 } },
  });
}

async function completeLogin(page: Page): Promise<void> {
  type LoginPhase = 'onboarding' | 'login' | 'ready' | 'waiting';
  const readPhase = async (): Promise<LoginPhase> => {
    try {
      return await page.evaluate(() => {
        if (document.querySelector('[data-testid="onboarding-gate"]')) return 'onboarding';
        if (document.querySelector('[data-testid="login-gate"]')) return 'login';
        const messenger = document.querySelector('[data-testid="messenger-workspace"]');
        return messenger?.getAttribute('data-initial-host-hydrated') === 'true' ? 'ready' : 'waiting';
      }) as LoginPhase;
    } catch {
      return 'waiting';
    }
  };
  for (let attempt = 0; attempt < 12; attempt += 1) {
    await expect.poll(readPhase, { timeout: 15_000 }).not.toBe('waiting');
    const phase = await readPhase();
    if (phase === 'onboarding') {
      await page.getByTestId('onboarding-next').click();
      continue;
    }
    if (phase === 'login') {
      await page.getByTestId('browser-login-start').click();
      continue;
    }
    if (phase === 'ready') return;
  }
  throw new Error('Packaged Fabushi did not reach Messenger ready state');
}

async function setReferenceWindow(app: ElectronApplication, page: Page): Promise<void> {
  await app.evaluate(({ BrowserWindow }) => {
    const win = BrowserWindow.getAllWindows()[0];
    if (!win) throw new Error('Fabushi BrowserWindow missing');
    win.setContentSize(1671, 937, false);
    win.center();
  });
  await page.emulateMedia({ colorScheme: 'dark', reducedMotion: 'reduce' });
  await expect(page.getByTestId('messenger-workspace')).toBeVisible();
  const viewport = await page.evaluate(() => ({
    width: window.innerWidth,
    height: window.innerHeight,
    dark: window.matchMedia('(prefers-color-scheme: dark)').matches,
  }));
  expect(viewport).toEqual({ width: 1671, height: 937, dark: true });
}

function installRuntimeLogCapture(app: ElectronApplication, page: Page, logs: RuntimeLog[]): void {
  page.on('console', (message) => logs.push({ at: Date.now(), source: 'page-console', text: `${message.type()}: ${message.text()}` }));
  page.on('pageerror', (error) => logs.push({ at: Date.now(), source: 'page-error', text: error.stack || error.message }));
  const child = app.process();
  child.stdout?.on('data', (chunk) => logs.push({ at: Date.now(), source: 'app-stdout', text: String(chunk) }));
  child.stderr?.on('data', (chunk) => logs.push({ at: Date.now(), source: 'app-stderr', text: String(chunk) }));
}

async function createCoworker(page: Page, name: string, description: string): Promise<void> {
  await page.evaluate(async ({ botName, botDescription }) => {
    if (!window.mahayana?.invoke) throw new Error('Mahayana bridge unavailable');
    const now = Date.now();
    await window.mahayana.invoke('feature.execute', {
      command: {
        type: 'bot.create',
        requestId: `obf-real-bot-create-${botName}-${now}`,
        name: botName,
        description: botDescription,
      },
    });
    await window.mahayana.invoke('feature.execute', {
      command: { type: 'bot.list', requestId: `obf-real-bot-list-${botName}-${now}` },
    });
  }, { botName: name, botDescription: description });
  await expect(peerByName(page, name)).toBeVisible({ timeout: 20_000 });
}

function peerByName(page: Page, name: string): Locator {
  return page.locator('[data-testid^="peer-legacy:bot:"]').filter({ hasText: name }).first();
}

async function botShape(locator: Locator): Promise<string> {
  const mark = locator.locator('[data-engine="fab-avatar"]').first();
  await expect(mark).toBeVisible();
  const shape = await mark.getAttribute('data-shape');
  expect(shape).toBeTruthy();
  return shape!;
}

async function directBotShape(mark: Locator): Promise<string> {
  await expect(mark).toBeVisible();
  const shape = await mark.getAttribute('data-shape');
  expect(shape).toBeTruthy();
  return shape!;
}


async function runtimeAgentId(page: Page, name: string): Promise<string> {
  const peer = peerByName(page, name);
  const explicit = await peer.getAttribute('data-agent-id');
  if (explicit) return explicit;
  const testId = await peer.getAttribute('data-testid');
  expect(testId).toBeTruthy();
  return testId!.replace(/^peer-legacy:bot:/, '');
}

async function verifyDirectHandoffIsolation(
  page: Page,
  fromAgentId: string,
  targetAgentId: string,
  isolatedAgentId: string,
): Promise<void> {
  const marker = `obf-direct-handoff-${Date.now()}`;
  const result = await page.evaluate(async ({ fromAgentId: fromId, targetAgentId: targetId, isolatedAgentId: isolatedId, marker: text }) => {
    if (!window.mahayana?.invoke) throw new Error('Mahayana bridge unavailable');
    const waitFor = <T,>(predicate: (detail: any) => T | undefined, timeoutMs = 20_000) => new Promise<T>((resolve, reject) => {
      const timer = window.setTimeout(() => {
        window.removeEventListener('fabushi:mahayana-runtime-event', handler as EventListener);
        reject(new Error('Timed out waiting for Mahayana collaboration event'));
      }, timeoutMs);
      const handler = (event: Event) => {
        const value = predicate((event as CustomEvent).detail);
        if (value === undefined) return;
        window.clearTimeout(timer);
        window.removeEventListener('fabushi:mahayana-runtime-event', handler as EventListener);
        resolve(value);
      };
      window.addEventListener('fabushi:mahayana-runtime-event', handler as EventListener);
    });
    const directEvent = waitFor((detail) =>
      detail?.type === 'agent.peerMessage'
        && detail.message?.fromAgentId === fromId
        && detail.message?.targetId === targetId
        && detail.message?.text === text
        ? detail.message
        : undefined);
    await window.mahayana.invoke('feature.execute', {
      command: {
        type: 'agent.send',
        requestId: `obf-direct-${Date.now()}`,
        fromAgentId: fromId,
        targetId,
        text,
        priority: true,
      },
    });
    await directEvent;

    const loadHistory = async (agentId: string) => {
      const historyEvent = waitFor((detail) =>
        detail?.type === 'agent.peerHistory' && detail.agentId === agentId
          ? detail.messages
          : undefined);
      await window.mahayana!.invoke('feature.execute', {
        command: {
          type: 'agent.peerHistory',
          requestId: `obf-history-${agentId}-${Date.now()}`,
          agentId,
          limit: 100,
        },
      });
      return await historyEvent as Array<{ text?: string }>;
    };
    const targetHistory = await loadHistory(targetId);
    const isolatedHistory = await loadHistory(isolatedId);
    return {
      targetHasMarker: targetHistory.some((message) => message.text === text),
      isolatedHasMarker: isolatedHistory.some((message) => message.text === text),
    };
  }, { fromAgentId, targetAgentId, isolatedAgentId, marker });
  expect(result.targetHasMarker).toBeTruthy();
  expect(result.isolatedHasMarker).toBeFalsy();
}

async function verifyTargetedBroadcast(page: Page, targetIds: string[]): Promise<void> {
  const result = await page.evaluate(async ({ targetIds }) => {
    if (!window.mahayana?.invoke) throw new Error('Mahayana bridge unavailable');
    const event = new Promise<{ total: number; scheduled: number }>((resolve, reject) => {
      const timer = window.setTimeout(() => {
        window.removeEventListener('fabushi:mahayana-runtime-event', handler as EventListener);
        reject(new Error('Timed out waiting for agent.broadcasted'));
      }, 20_000);
      const handler = (runtimeEvent: Event) => {
        const detail = (runtimeEvent as CustomEvent).detail;
        if (detail?.type !== 'agent.broadcasted') return;
        window.clearTimeout(timer);
        window.removeEventListener('fabushi:mahayana-runtime-event', handler as EventListener);
        resolve(detail.result);
      };
      window.addEventListener('fabushi:mahayana-runtime-event', handler as EventListener);
    });
    await window.mahayana.invoke('feature.execute', {
      command: {
        type: 'agent.broadcast',
        requestId: `obf-broadcast-${Date.now()}`,
        targetIds,
        message: `OBF signed-candidate broadcast ${Date.now()}`,
      },
    });
    return await event;
  }, { targetIds });
  expect(result.total).toBe(targetIds.length);
  expect(result.scheduled).toBe(result.total);
}

async function installLifecycleJournal(page: Page): Promise<void> {
  await page.evaluate(() => {
    const scope = window as typeof window & { __obfLifecycle?: LifecycleSample[] };
    scope.__obfLifecycle = [];
    const sample = () => {
      for (const node of document.querySelectorAll<HTMLElement>('[class*="agentThinkingRow"]')) {
        const text = (node.innerText || '').trim();
        const seen = scope.__obfLifecycle?.some((entry) => entry.status === 'thinking' && entry.text === text);
        if (!seen) scope.__obfLifecycle?.push({ at: Date.now(), status: 'thinking', text });
      }
      for (const node of document.querySelectorAll<HTMLElement>('[data-testid="agent-step"]')) {
        const status = node.dataset.status || '';
        const text = (node.innerText || '').trim();
        const last = scope.__obfLifecycle?.at(-1);
        if (!last || last.status !== status || last.text !== text) scope.__obfLifecycle?.push({ at: Date.now(), status, text });
      }
    };
    sample();
    const observer = new MutationObserver(sample);
    observer.observe(document.documentElement, { subtree: true, attributes: true, childList: true, characterData: true });
    (scope as typeof scope & { __obfObserver?: MutationObserver }).__obfObserver = observer;
  });
}

async function sendRealTurn(page: Page, prompt: string): Promise<string> {
  const before = await page.locator('article[class*="messagePeer"]').count();
  await page.getByTestId('messenger-input').fill(prompt);
  await page.getByTestId('messenger-send').click();
  await expect(page.getByRole('article').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 5_000 });
  const workbench = page.getByTestId('agent-workbench');
  await expect(workbench).toBeVisible({ timeout: 30_000 });
  const run = page.getByTestId('agent-run').last();
  await expect(run).toHaveAttribute('data-status', 'completed', { timeout: 180_000 });
  await expect.poll(async () => run.getByTestId('agent-step').count(), { timeout: 30_000 }).toBeGreaterThan(0);
  await expect.poll(async () => page.locator('article[class*="messagePeer"]').count(), { timeout: 30_000 }).toBeGreaterThan(before);
  const finalMessage = page.locator('article[class*="messagePeer"]').last();
  await expect(finalMessage).toBeVisible();
  return (await finalMessage.innerText()).trim();
}

async function attachFile(page: Page, filePath: string): Promise<void> {
  await page.getByTitle('附件').click();
  await page.getByRole('button', { name: '文件' }).click();
  const fileInput = page.locator('form input[type="file"]:not([accept])');
  await fileInput.setInputFiles(filePath);
  await expect(page.getByRole('article').filter({ hasText: path.basename(filePath) }).last()).toBeVisible({ timeout: 20_000 });
}

async function readBox(locator: Locator): Promise<Box | null> {
  const box = await locator.boundingBox();
  if (!box) return null;
  return { x: box.x, y: box.y, width: box.width, height: box.height };
}

async function captureGeometry(page: Page, finalArticle: Locator, structured: Locator, table: Locator): Promise<GeometryReport> {
  const peerRows: Record<string, Box | null> = {};
  const rosterAvatars: Record<string, Box | null> = {};
  for (const [name] of coworkers) {
    const peer = peerByName(page, name);
    peerRows[name] = await readBox(peer);
    rosterAvatars[name] = await readBox(peer.locator('[data-engine="fab-avatar"]').first());
  }
  const orderedRows = coworkers.map(([name]) => peerRows[name]).filter((box): box is Box => Boolean(box));
  const peerRowGaps = orderedRows.slice(1).map((box, index) => box.y - (orderedRows[index].y + orderedRows[index].height));
  const report: GeometryReport = {
    viewport: await page.evaluate(() => ({ width: window.innerWidth, height: window.innerHeight, dark: window.matchMedia('(prefers-color-scheme: dark)').matches })),
    navigation: await readBox(page.locator('[class*="navRail"]').first()),
    roster: await readBox(page.locator('[class*="chatList"]').first()),
    header: await readBox(page.locator('[class*="chatHeader"]').first()),
    transcript: await readBox(page.locator('[class*="messageArea"]').first()),
    finalCard: await readBox(finalArticle),
    table: await readBox(table),
    sourceFiles: await readBox(structured.getByTestId('assistant-source-files')),
    attachmentCard: await readBox(page.getByRole('article').filter({ hasText: 'launch-brief.md' }).last()),
    hoverActions: await readBox(finalArticle.getByTestId('message-hover-actions')),
    composer: await readBox(page.locator('[class*="composer"]').last()),
    composerInput: await readBox(page.getByTestId('messenger-input')),
    peerRows,
    rosterAvatars,
    peerRowGaps,
  };
  expect(report.viewport).toEqual({ width: 1671, height: 937, dark: true });
  for (const [name, box] of Object.entries(report.rosterAvatars)) {
    expect(box, `${name} roster avatar geometry missing`).not.toBeNull();
    expect(box!.width).toBeGreaterThanOrEqual(32);
    expect(box!.width).toBeLessThanOrEqual(36);
    expect(box!.height).toBeGreaterThanOrEqual(32);
    expect(box!.height).toBeLessThanOrEqual(36);
  }
  for (const [name, box] of Object.entries({
    navigation: report.navigation,
    roster: report.roster,
    header: report.header,
    transcript: report.transcript,
    finalCard: report.finalCard,
    table: report.table,
    sourceFiles: report.sourceFiles,
    attachmentCard: report.attachmentCard,
    hoverActions: report.hoverActions,
    composer: report.composer,
    composerInput: report.composerInput,
  })) expect(box, `${name} geometry missing`).not.toBeNull();
  return report;
}

function geometryRegions(geometry: GeometryReport): Record<string, Box | null> {
  const regions: Record<string, Box | null> = {
    'column-navigation': geometry.navigation,
    'column-roster': geometry.roster,
    header: geometry.header,
    transcript: geometry.transcript,
    'final-message-card': geometry.finalCard,
    table: geometry.table,
    'source-files': geometry.sourceFiles,
    'attachment-card': geometry.attachmentCard,
    'hover-actions': geometry.hoverActions,
    composer: geometry.composer,
  };
  for (const [name, box] of Object.entries(geometry.rosterAvatars)) regions[`avatar-${name.toLowerCase()}`] = box;
  return regions;
}

async function measureVisualDiff(page: Page, referenceBytes: Buffer, actualBytes: Buffer, regions: Record<string, Box | null>): Promise<VisualDiffReport> {
  const report = await page.evaluate(async ({ referenceBase64, actualBase64, threshold, maxDiffPixelRatio, inputRegions }) => {
    const load = async (base64: string): Promise<ImageBitmap> => {
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
      return createImageBitmap(new Blob([bytes], { type: 'image/png' }));
    };
    const [reference, actual] = await Promise.all([load(referenceBase64), load(actualBase64)]);
    if (reference.width !== actual.width || reference.height !== actual.height) {
      throw new Error(`Reference/actual dimensions differ: ${reference.width}x${reference.height} vs ${actual.width}x${actual.height}`);
    }
    const readPixels = (image: ImageBitmap): Uint8ClampedArray => {
      const canvas = document.createElement('canvas');
      canvas.width = image.width;
      canvas.height = image.height;
      const context = canvas.getContext('2d', { willReadFrequently: true });
      if (!context) throw new Error('2D canvas unavailable for visual diff');
      context.drawImage(image, 0, 0);
      return context.getImageData(0, 0, image.width, image.height).data;
    };
    const referencePixels = readPixels(reference);
    const actualPixels = readPixels(actual);
    const channelThreshold = Math.round(threshold * 255);
    const score = (box?: Box | null): RegionDiff => {
      const x0 = Math.max(0, Math.floor(box?.x ?? 0));
      const y0 = Math.max(0, Math.floor(box?.y ?? 0));
      const x1 = Math.min(reference.width, Math.ceil((box?.x ?? 0) + (box?.width ?? reference.width)));
      const y1 = Math.min(reference.height, Math.ceil((box?.y ?? 0) + (box?.height ?? reference.height)));
      let differingPixels = 0;
      let totalPixels = 0;
      for (let y = y0; y < y1; y += 1) {
        for (let x = x0; x < x1; x += 1) {
          const offset = (y * reference.width + x) * 4;
          const delta = Math.max(
            Math.abs(referencePixels[offset] - actualPixels[offset]),
            Math.abs(referencePixels[offset + 1] - actualPixels[offset + 1]),
            Math.abs(referencePixels[offset + 2] - actualPixels[offset + 2]),
            Math.abs(referencePixels[offset + 3] - actualPixels[offset + 3]),
          );
          if (delta > channelThreshold) differingPixels += 1;
          totalPixels += 1;
        }
      }
      return { differingPixels, totalPixels, differingPixelRatio: totalPixels ? differingPixels / totalPixels : 0 };
    };
    const globalScore = score();
    const regionScores: Record<string, RegionDiff> = {};
    for (const [name, box] of Object.entries(inputRegions)) if (box) regionScores[name] = score(box);
    const residualRegions = Object.entries(regionScores)
      .filter(([, value]) => value.differingPixels > 0)
      .map(([name, value]) => ({ name, ...value }))
      .sort((left, right) => right.differingPixelRatio - left.differingPixelRatio);
    return {
      width: reference.width,
      height: reference.height,
      pixelThreshold: threshold,
      maxDiffPixelRatio,
      global: { ...globalScore, zeroDiff: globalScore.differingPixels === 0 },
      regions: regionScores,
      residualRegions,
    };
  }, {
    referenceBase64: referenceBytes.toString('base64'),
    actualBase64: actualBytes.toString('base64'),
    threshold: pixelThreshold,
    maxDiffPixelRatio: visualThreshold,
    inputRegions: regions,
  });
  return report as VisualDiffReport;
}

async function saveRuntimeEvidence(
  page: Page,
  testInfo: TestInfo,
  logs: RuntimeLog[],
  referenceBytes?: Buffer,
  geometry?: GeometryReport,
  visualDiff?: VisualDiffReport,
  identity?: Record<string, unknown>,
  appVersion?: string,
): Promise<void> {
  const evidenceDir = testInfo.outputPath('obf-runtime');
  await mkdir(evidenceDir, { recursive: true });
  const lifecycle = await page.evaluate(() => (window as typeof window & { __obfLifecycle?: LifecycleSample[] }).__obfLifecycle || []);
  await writeFile(path.join(evidenceDir, 'lifecycle.json'), JSON.stringify(lifecycle, null, 2));
  const agentDrafts = await page.evaluate(() => window.localStorage.getItem('fabushi.agent-workspace.drafts.v1'));
  await writeFile(path.join(evidenceDir, 'agent-workspace-drafts.json'), agentDrafts || '{}');
  await writeFile(path.join(evidenceDir, 'runtime.log'), logs.map((entry) => `${new Date(entry.at).toISOString()} [${entry.source}] ${entry.text}`).join('\n'));
  if (referenceBytes) await writeFile(path.join(evidenceDir, 'openbot-reference.png'), referenceBytes);
  if (geometry) await writeFile(path.join(evidenceDir, 'geometry.json'), JSON.stringify(geometry, null, 2));
  if (visualDiff) await writeFile(path.join(evidenceDir, 'visual-diff-report.json'), JSON.stringify(visualDiff, null, 2));
  if (identity) await writeFile(path.join(evidenceDir, 'identity.json'), JSON.stringify(identity, null, 2));
  await writeFile(path.join(evidenceDir, 'evidence-manifest.json'), JSON.stringify({
    canonicalMainSha,
    sourceSha,
    appVersion: appVersion || null,
    referenceCropSha256,
    referencePathBasename: referenceScreenshot ? path.basename(referenceScreenshot) : null,
    packagedExecutableBasename: packagedExecutable ? path.basename(packagedExecutable) : null,
    visualThreshold,
    pixelThreshold,
    generatedAt: new Date().toISOString(),
  }, null, 2));
}

test('OBF exact-main packaged reference journey is pixel-identical and uses real Mahayana events', async ({}, testInfo) => {
  test.setTimeout(12 * 60_000);
  assertProductionEvidenceEnvironment();
  const referenceBytes = await readFile(referenceScreenshot);
  if (referenceBytes.length < 10_000) throw new Error('OBF reference screenshot is unexpectedly small');
  const referenceHash = createHash('sha256').update(referenceBytes).digest('hex');
  expect(referenceHash).toBe(referenceCropSha256);

  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-obf-real-'));
  const fixtureDir = await mkdtemp(path.join(tmpdir(), 'fabushi-obf-fixture-'));
  const csvPath = path.join(fixtureDir, 'launch-metrics.csv');
  const briefPath = path.join(fixtureDir, 'launch-brief.md');
  await writeFile(csvPath, 'metric,value\nclaims_verified,7/8\nrollback_ready,true\n');
  let app: ElectronApplication | null = null;
  let page: Page | null = null;
  let tracingActive = false;
  let appVersion = '';
  let chiefRosterShape = '';
  let chiefHeaderShape = '';
  let chiefTranscriptShape = '';
  const runtimeLogs: RuntimeLog[] = [];
  const evidenceDir = testInfo.outputPath('obf-runtime');
  const tracePath = path.join(evidenceDir, 'trace.zip');
  const videoDir = path.join(evidenceDir, 'video');
  await mkdir(videoDir, { recursive: true });
  try {
    app = await launchPackaged(appDataDir, videoDir);
    page = await app.firstWindow();
    installRuntimeLogCapture(app, page, runtimeLogs);
    appVersion = await app.evaluate(({ app: electronApp }) => electronApp.getVersion());
    await page.context().tracing.start({ screenshots: true, snapshots: true, sources: true });
    tracingActive = true;
    await completeLogin(page);
    await setReferenceWindow(app, page);
    await installLifecycleJournal(page);

    for (const [name, description] of coworkers) await createCoworker(page, name, description);
    for (const [name] of coworkers) await expect(peerByName(page, name)).toBeVisible();

    const researchAgentId = await runtimeAgentId(page, 'Research');
    const builderAgentId = await runtimeAgentId(page, 'Builder');
    const launchAgentId = await runtimeAgentId(page, 'Launch');
    await verifyDirectHandoffIsolation(page, researchAgentId, builderAgentId, launchAgentId);
    await verifyTargetedBroadcast(page, [researchAgentId, launchAgentId]);

    chiefRosterShape = await botShape(peerByName(page, 'Chief'));

    await peerByName(page, 'Research').click();
    const research = await sendRealTurn(page,
      'Use at least one available read-only browser or research tool to verify a harmless public fact. Return a concise evidence note and clearly state which tool was used.');

    await peerByName(page, 'Builder').click();
    const builder = await sendRealTurn(page,
      "Use an available local shell or file tool to perform a harmless readiness check (for example printf 'rollback-ready'). Return a concise rollout note and the observed result.");

    await peerByName(page, 'Launch').click();
    const launch = await sendRealTurn(page,
      "Use the file-read tool (not a shell command) to read the exact missing path /tmp/fabushi-obf-intentionally-missing so the tool itself returns an error; then recover and return a concise launch note. Do not create, modify, or delete anything.");

    await writeFile(briefPath, ['# Coworker launch notes', '', `Research: ${research}`, '', `Builder: ${builder}`, '', `Launch: ${launch}`].join('\n'));

    await peerByName(page, 'Chief').click();
    chiefHeaderShape = await directBotShape(page.locator('[class*="chatIdentity"] [data-engine="fab-avatar"]').first());
    expect(chiefHeaderShape).toBe(chiefRosterShape);
    await attachFile(page, briefPath);
    await attachFile(page, csvPath);

    const chiefPrompt = [
      'Synthesize these real coworker outputs into the final launch brief. Do not invent a new runtime or tool result.',
      `Research note: ${research}`,
      `Builder note: ${builder}`,
      `Launch note: ${launch}`,
      'Your final response MUST contain a heading exactly "Final launch brief", then a Markdown table with exactly the columns Workstream | Owner | Status and exactly three rows for Evidence/Research, Rollout/Builder, Release/Launch.',
      'After the table include a line exactly beginning "Source files:" and include `launch-brief.md` and `launch-metrics.csv` so Fabushi renders source-file chips.',
    ].join('\n\n');
    await sendRealTurn(page, chiefPrompt);

    const structured = page.getByTestId('structured-message-body').last();
    await expect(structured).toContainText('Final launch brief');
    const table = structured.getByTestId('assistant-result-table');
    await expect(table).toBeVisible();
    await expect(table.locator('th')).toHaveCount(3);
    await expect(table.locator('th').nth(0)).toHaveText('Workstream');
    await expect(table.locator('th').nth(1)).toHaveText('Owner');
    await expect(table.locator('th').nth(2)).toHaveText('Status');
    await expect(table.locator('tbody tr')).toHaveCount(3);
    await expect(table.getByTestId('assistant-owner-chip')).toHaveCount(3);
    await expect(structured.getByTestId('assistant-source-files')).toContainText('launch-brief.md');
    await expect(structured.getByTestId('assistant-source-files')).toContainText('launch-metrics.csv');

    const finalArticle = structured.locator('xpath=ancestor::article[1]');
    chiefTranscriptShape = await directBotShape(finalArticle.locator('[data-engine="fab-avatar"]').first());
    expect(chiefTranscriptShape).toBe(chiefRosterShape);
    await finalArticle.hover();
    await expect(finalArticle.getByTestId('message-hover-actions')).toBeVisible();
    await expect(page.getByTestId('messenger-input')).toBeVisible();

    const lifecycle = await page.evaluate(() => (window as typeof window & { __obfLifecycle?: LifecycleSample[] }).__obfLifecycle || []);
    expect(lifecycle.some((sample) => sample.status === 'thinking')).toBeTruthy();
    expect(lifecycle.some((sample) => sample.status === 'running')).toBeTruthy();
    expect(lifecycle.some((sample) => sample.status === 'completed')).toBeTruthy();
    expect(lifecycle.some((sample) => sample.status === 'failed')).toBeTruthy();

    const geometry = await captureGeometry(page, finalArticle, structured, table);
    const workspace = page.getByTestId('messenger-workspace');
    const actualBytes = await workspace.screenshot({ animations: 'disabled', caret: 'hide' });
    await writeFile(path.join(evidenceDir, 'fabushi-openbot-comparison.png'), actualBytes);
    const visualDiff = await measureVisualDiff(page, referenceBytes, actualBytes, geometryRegions(geometry));
    expect(visualDiff.global.differingPixels).toBe(0);
    expect(visualDiff.global.differingPixelRatio).toBe(0);
    expect(visualDiff.global.zeroDiff).toBeTruthy();
    expect(visualDiff.residualRegions).toEqual([]);

    const identityBeforeRestart = {
      Chief: {
        roster: chiefRosterShape,
        header: chiefHeaderShape,
        transcript: chiefTranscriptShape,
      },
    };
    await saveRuntimeEvidence(page, testInfo, runtimeLogs, referenceBytes, geometry, visualDiff, identityBeforeRestart, appVersion);
    await page.context().tracing.stop({ path: tracePath });
    tracingActive = false;

    const expectedPath = testInfo.snapshotPath('openbot-reference-app.png');
    await mkdir(path.dirname(expectedPath), { recursive: true });
    await copyFile(referenceScreenshot, expectedPath);
    await expect(workspace).toHaveScreenshot('openbot-reference-app.png', {
      animations: 'disabled',
      caret: 'hide',
      threshold: pixelThreshold,
      maxDiffPixelRatio: visualThreshold,
    });

    await app.close();
    app = null;

    app = await launchPackaged(appDataDir, videoDir);
    page = await app.firstWindow();
    installRuntimeLogCapture(app, page, runtimeLogs);
    await completeLogin(page);
    await setReferenceWindow(app, page);
    const restoredChief = peerByName(page, 'Chief');
    await expect(restoredChief).toBeVisible({ timeout: 30_000 });
    const restartShape = await botShape(restoredChief);
    expect(restartShape).toBe(chiefRosterShape);
    const identity = {
      Chief: {
        roster: chiefRosterShape,
        header: chiefHeaderShape,
        transcript: chiefTranscriptShape,
        restart: restartShape,
        stable: new Set([chiefRosterShape, chiefHeaderShape, chiefTranscriptShape, restartShape]).size === 1,
      },
    };
    expect(identity.Chief.stable).toBeTruthy();
    await writeFile(path.join(evidenceDir, 'identity.json'), JSON.stringify(identity, null, 2));
    await writeFile(path.join(evidenceDir, 'runtime.log'), runtimeLogs.map((entry) => `${new Date(entry.at).toISOString()} [${entry.source}] ${entry.text}`).join('\n'));
  } catch (error) {
    if (page && tracingActive) await saveRuntimeEvidence(page, testInfo, runtimeLogs, referenceBytes, undefined, undefined, {
      Chief: { roster: chiefRosterShape || null, header: chiefHeaderShape || null, transcript: chiefTranscriptShape || null },
    }, appVersion).catch(() => undefined);
    throw error;
  } finally {
    if (tracingActive && page) {
      await page.context().tracing.stop({ path: tracePath }).catch(() => undefined);
    }
    await app?.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
    await rm(fixtureDir, { recursive: true, force: true });
  }
});
