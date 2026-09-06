import { _electron as electron, expect, test, type Page, type TestInfo } from '@playwright/test';
import { access, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;
const executionKey = 'fabushi.desktop.miniapp-execution.v1:global-dharma';

async function launchDesktopApp(appDataDir: string) {
  return electron.launch({
    ...(packagedExecutable ? { executablePath: packagedExecutable, args: [] } : { args: [appRoot] }),
    env: { ...process.env, FABUSHI_APP_DATA: appDataDir, FABUSHI_FEATURE_HOST_MODE: process.env.FABUSHI_FEATURE_HOST_MODE || 'test', MAHAYANA_APP_HOST_BIN: process.env.MAHAYANA_APP_HOST_BIN || '' },
  });
}

async function shot(page: Page, testInfo: TestInfo, name: string) {
  await page.screenshot({ path: testInfo.outputPath(name), fullPage: true });
}

async function completeBrowserLogin(page: Page): Promise<void> {
  const loginGate = page.getByTestId('login-gate');
  const workspace = page.getByTestId('messenger-workspace');
  type LoginPhase = 'onboarding' | 'login' | 'ready' | 'waiting';
  const readPhase = async (): Promise<LoginPhase> => {
    try {
      return await page.evaluate(() => {
        if (document.querySelector('[data-testid="onboarding-gate"]')) return 'onboarding';
        if (document.querySelector('[data-testid="login-gate"]')) return 'login';
        const messenger = document.querySelector('[data-testid="messenger-workspace"]');
        return messenger?.getAttribute('data-initial-host-hydrated') === 'true' ? 'ready' : 'waiting';
      }) as LoginPhase;
    } catch { return 'waiting'; }
  };
  for (let phase = 0; phase < 12; phase += 1) {
    await expect.poll(readPhase, { timeout: 15_000 }).not.toBe('waiting');
    const current = await readPhase();
    if (current === 'onboarding') { await page.getByTestId('onboarding-next').click(); continue; }
    if (current === 'login') { await page.getByTestId('browser-login-start').click(); await expect(loginGate).toBeHidden(); continue; }
    if (current === 'ready') break;
  }
  await expect(workspace).toHaveAttribute('data-initial-host-hydrated', 'true', { timeout: 15_000 });
}

async function navigate(page: Page, title: string): Promise<void> {
  await page.getByTestId('profile-navigation-trigger').click();
  await expect(page.getByTestId('profile-navigation-menu')).toBeVisible();
  await page.getByTitle(title, { exact: true }).click();
}

function bot(page: Page) { return page.getByRole('button', { name: /@global_dharma_bot\b/ }).first(); }
async function waitBot(page: Page) {
  await navigate(page, 'Bots');
  const peer = bot(page);
  await expect(peer).toBeVisible({ timeout: 15_000 });
  await expect(peer).toContainText('全球法布施');
  return peer;
}
function frame(page: Page) { return page.frameLocator('iframe[title="global-dharma"]'); }
async function tools(page: Page) { return frame(page).locator('body').evaluate(() => (window as any).__fabushiWebMcp?.list?.().map((t: any) => String(t.name)) ?? []); }
async function execution(page: Page) { return frame(page).locator('body').evaluate(async () => (window as any).__fabushiMiniAppHost.execution()); }
async function session(page: Page) { return frame(page).locator('body').evaluate(async () => (window as any).__fabushiMiniAppHost.session()); }
async function entitlement(page: Page) { return frame(page).locator('body').evaluate(async () => (window as any).__fabushiMiniAppHost.entitlement()); }
async function parentExecution(page: Page) { return page.evaluate((key) => JSON.parse(localStorage.getItem(key) || 'null'), executionKey); }
async function writeCloudProbe(page: Page, value: string) {
  const stored = await frame(page).locator('body').evaluate(async (_body, probe) => {
    const api = (window as any).FabushiMiniApp?.CloudStorage;
    await api.setItem('m2_sync_packaged_probe', probe);
    return api.getItem('m2_sync_packaged_probe');
  }, value);
  expect(stored).toBe(value);
}
async function readCloudProbe(page: Page) { return frame(page).locator('body').evaluate(async () => (window as any).FabushiMiniApp?.CloudStorage.getItem('m2_sync_packaged_probe')); }

test('Global Dharma packaged journey keeps Bot WebMCP, UI revision, account and CNY 1080 entitlement in one host boundary', async ({}, testInfo) => {
  test.setTimeout(150_000);
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-global-dharma-journey-'));
  const probeValue = `packaged-sync-${Date.now()}`;
  const statusText = 'please show status now';
  const prayerText = '启动转经轮';
  const firstApp = await launchDesktopApp(appDataDir);
  let restartedApp: Awaited<ReturnType<typeof launchDesktopApp>> | null = null;
  let videoPage: Page | null = null;
  let videoPath: string | null = null;
  let screencastStarted = false;
  let screencastAttached = false;
  let restartVideoPage: Page | null = null;
  let restartVideoPath: string | null = null;
  let restartScreencastStarted = false;
  let restartScreencastAttached = false;
  try {
    const page = await firstApp.firstWindow();
    videoPage = page;
    videoPath = testInfo.outputPath('global-dharma-user-journey.webm');
    await page.screencast.start({ path: videoPath, size: { width: 1280, height: 800 } });
    screencastStarted = true;
    page.on('dialog', (dialog) => void dialog.accept());
    await completeBrowserLogin(page);
    await shot(page, testInfo, '01-authenticated-messenger.png');

    await page.getByTestId('global-search-trigger').click();
    await page.getByTestId('global-search-tab-apps').click();
    await page.getByTestId('global-search-input').fill('全球法布施');
    const appResult = page.getByTestId('global-search-app-global-dharma');
    await expect(appResult).toBeVisible();
    await shot(page, testInfo, '02-marketplace-search-global-dharma.png');
    const install = appResult.getByRole('button', { name: '安装' });
    if (await install.isVisible().catch(() => false)) await install.click();
    await expect(appResult.getByRole('button', { name: '打开' })).toBeVisible();
    await shot(page, testInfo, '03-marketplace-installed.png');

    await navigate(page, '联系人');
    await expect(bot(page)).toBeVisible({ timeout: 15_000 });
    await shot(page, testInfo, '04-contact-bot-projection.png');

    const peer = await waitBot(page);
    await peer.click();
    await expect(page.getByTestId('miniapp-bot-open')).toBeVisible();
    const input = page.getByTestId('messenger-input');
    await input.fill('/');
    await expect(page.getByTestId('miniapp-bot-commands')).toContainText('/status');
    await input.fill(statusText);
    await page.getByTestId('messenger-send').click();
    await expect(page.getByTestId('message-list').locator(':scope > article').last()).toContainText('已读取全球法布施状态');
    await expect.poll(() => parentExecution(page), { timeout: 15_000 }).toMatchObject({ protocol: 'fabushi.miniapp.execution.v1', source: 'bot', phase: 'completed', tool: 'status' });
    const botState = await parentExecution(page);
    expect(botState.revision).toBeGreaterThan(0);
    await shot(page, testInfo, '05-bot-natural-language-webmcp-complete.png');

    await page.getByTestId('miniapp-bot-open').click();
    await expect(page.locator('iframe[title="global-dharma"]')).toBeVisible();
    await expect.poll(() => tools(page), { timeout: 15_000 }).toEqual(expect.arrayContaining(['status', 'start', 'stop', 'send']));
    const opened = await execution(page);
    expect(opened.revision).toBe(botState.revision);
    expect(opened).toMatchObject({ source: 'bot', phase: 'completed', tool: 'status' });
    const stateBar = frame(page).getByTestId('fabushi-miniapp-host-state');
    await expect(stateBar).toHaveAttribute('data-revision', String(botState.revision));
    const account = await session(page);
    expect(account).toMatchObject({ protocol: 'fabushi.miniapp.session.v1', pluginId: 'global-dharma', loggedIn: true, tokenExposed: false });
    expect(JSON.stringify(account)).not.toMatch(/accessToken|refreshToken|bearer/i);
    const before = await entitlement(page);
    expect(before.access.allowed).toBe(false);
    const lifetime = before.purchaseOptions.find((option: any) => option.sku === 'local-prayer-wheel.lifetime');
    expect(lifetime).toMatchObject({ productId: 'prod.global-dharma.local-prayer-wheel.lifetime', productKind: 'digital_durable', currency: 'CNY', amount: 108000 });
    await shot(page, testInfo, '06-open-app-same-revision-account-and-paywall.png');

    const purchaseButton = frame(page).getByTestId('fabushi-miniapp-purchase-lifetime');
    await expect(purchaseButton).toBeVisible();
    await expect(purchaseButton).toContainText('¥1080');
    await purchaseButton.click();
    await expect.poll(() => execution(page), { timeout: 15_000 }).toMatchObject({ phase: 'completed', source: 'web-ui', tool: 'purchaseLifetime', entitlementAllowed: true });
    const paidState = await execution(page);
    const paid = paidState.result;
    expect(paid).toMatchObject({ status: 'entitled', paymentId: expect.any(String) });
    expect(paid.product).toMatchObject({ sku: 'local-prayer-wheel.lifetime', currency: 'CNY', amount: 108000 });
    expect(paid.checkout.callback).toMatchObject({ duplicate: false });
    expect(paid.entitlement.access.allowed).toBe(true);
    await shot(page, testInfo, '07-cny1080-lifetime-entitlement-purchased.png');

    const restoreButton = frame(page).getByTestId('fabushi-miniapp-restore-purchases');
    await expect(restoreButton).toBeVisible();
    await restoreButton.click();
    await expect.poll(() => execution(page), { timeout: 15_000 }).toMatchObject({ phase: 'completed', source: 'web-ui', tool: 'restorePurchases', entitlementAllowed: true });
    const restoreState = await execution(page);
    const restored = restoreState.result;
    expect(restored.restored.restored).toBe(true);
    expect(restored.entitlement.access.allowed).toBe(true);
    await writeCloudProbe(page, probeValue);
    await shot(page, testInfo, '08-entitlement-restored.png');

    await page.getByTestId('miniapp-close').click();
    await input.fill(prayerText);
    await page.getByTestId('messenger-send').click();
    await expect(page.getByTestId('message-list').locator(':scope > article').last()).toContainText('本地转经轮已通过宿主权限校验并启动');
    await expect.poll(() => parentExecution(page), { timeout: 15_000 }).toMatchObject({ source: 'bot', phase: 'completed', tool: 'start', surface: 'local-prayer-wheel', entitlementAllowed: true });
    const prayerState = await parentExecution(page);
    expect(prayerState.revision).toBeGreaterThan(restoreState.revision);
    await shot(page, testInfo, '09-bot-starts-entitled-local-prayer-wheel.png');

    await page.getByTestId('miniapp-bot-open').click();
    const reopened = await execution(page);
    expect(reopened.revision).toBe(prayerState.revision);
    expect(reopened).toMatchObject({ tool: 'start', surface: 'local-prayer-wheel', entitlementAllowed: true });
    await expect(frame(page).getByTestId('fabushi-miniapp-host-state')).toHaveAttribute('data-revision', String(prayerState.revision));
    await shot(page, testInfo, '10-open-app-follows-bot-prayer-wheel-revision.png');

    await page.screencast.stop();
    screencastStarted = false;
    if (!videoPath) throw new Error('Global Dharma user-journey video path was not initialized.');
    await access(videoPath);
    await testInfo.attach('global-dharma-user-journey-video', { path: videoPath, contentType: 'video/webm' });
    screencastAttached = true;

    await firstApp.close();
    restartedApp = await launchDesktopApp(appDataDir);
    const restartedPage = await restartedApp.firstWindow();
    restartVideoPage = restartedPage;
    restartVideoPath = testInfo.outputPath('global-dharma-user-journey-restart-logout.webm');
    await restartedPage.screencast.start({ path: restartVideoPath, size: { width: 1280, height: 800 } });
    restartScreencastStarted = true;
    restartedPage.on('dialog', (dialog) => void dialog.accept());
    await completeBrowserLogin(restartedPage);
    const recovered = await waitBot(restartedPage);
    await recovered.click();
    await expect(restartedPage.getByText(statusText, { exact: true })).toBeVisible({ timeout: 15_000 });
    await expect(restartedPage.getByText(prayerText, { exact: true })).toBeVisible();
    await restartedPage.getByTestId('miniapp-bot-open').click();
    const recoveredState = await execution(restartedPage);
    expect(recoveredState.revision).toBe(prayerState.revision);
    expect(recoveredState).toMatchObject({ phase: 'completed', tool: 'start', surface: 'local-prayer-wheel', entitlementAllowed: true });
    await expect.poll(() => readCloudProbe(restartedPage), { timeout: 15_000 }).toBe(probeValue);
    expect((await entitlement(restartedPage)).access.allowed).toBe(true);
    await shot(restartedPage, testInfo, '11-restart-recovers-history-state-entitlement-cloud.png');

    await restartedPage.getByTestId('miniapp-close').click();
    await restartedPage.getByTestId('profile-navigation-trigger').click();
    await restartedPage.getByTitle('设置', { exact: true }).click();
    await restartedPage.getByTestId('settings-category-account').click();
    await restartedPage.getByTestId('settings-logout').click();
    await expect(restartedPage.getByTestId('login-gate')).toBeVisible({ timeout: 10_000 });
    await expect.poll(() => restartedPage.evaluate(async (key) => {
      const session = await (window as any).fabushiNative?.invoke?.('getMiniAppSessionProjection', { pluginId: 'global-dharma' });
      const durable = await (window as any).fabushiNative?.invoke?.('readClientPersistence', { key });
      return {
        session,
        localExecution: localStorage.getItem(key),
        durableExecution: durable ?? null,
      };
    }, executionKey), { timeout: 10_000 }).toMatchObject({
      session: { protocol: 'fabushi.miniapp.session.v1', pluginId: 'global-dharma', loggedIn: false, tokenExposed: false },
      localExecution: null,
      durableExecution: null,
    });
    await shot(restartedPage, testInfo, '12-logout-clears-miniapp-session-and-execution.png');
    await restartedPage.screencast.stop();
    restartScreencastStarted = false;
    if (!restartVideoPath) throw new Error('Global Dharma restart/logout video path was not initialized.');
    await access(restartVideoPath);
    await testInfo.attach('global-dharma-user-journey-restart-logout-video', { path: restartVideoPath, contentType: 'video/webm' });
    restartScreencastAttached = true;
  } finally {
    if (videoPage && screencastStarted) await videoPage.screencast.stop().catch(() => {});
    if (restartVideoPage && restartScreencastStarted) await restartVideoPage.screencast.stop().catch(() => {});
    if (!screencastAttached && videoPath) {
      try {
        await access(videoPath);
        await testInfo.attach('global-dharma-user-journey-video-partial', { path: videoPath, contentType: 'video/webm' });
      } catch {}
    }
    if (!restartScreencastAttached && restartVideoPath) {
      try {
        await access(restartVideoPath);
        await testInfo.attach('global-dharma-user-journey-restart-logout-video-partial', { path: restartVideoPath, contentType: 'video/webm' });
      } catch {}
    }
    await restartedApp?.close().catch(() => {});
    await firstApp.close().catch(() => {});
    await rm(appDataDir, { recursive: true, force: true });
  }
});
