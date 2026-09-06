import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;

async function launchDesktopApp(appDataDir: string) {
  return electron.launch({
    ...(packagedExecutable
      ? { executablePath: packagedExecutable, args: [] }
      : { args: [appRoot] }),
    env: {
      ...process.env,
      FABUSHI_APP_DATA: appDataDir,
      FABUSHI_FEATURE_HOST_MODE: process.env.FABUSHI_FEATURE_HOST_MODE || 'test',
      MAHAYANA_APP_HOST_BIN: process.env.MAHAYANA_APP_HOST_BIN || '',
    },
  });
}

async function completeBrowserLogin(page: Page): Promise<void> {
  const onboardingGate = page.getByTestId('onboarding-gate');
  const loginGate = page.getByTestId('login-gate');
  const workspace = page.getByTestId('messenger-workspace');
  type LoginPhase = 'onboarding' | 'login' | 'ready' | 'waiting';

  // Read the auth surface in one renderer evaluation. During the HostClient ->
  // Messenger transition individual locator probes can straddle a destroyed
  // execution context and wait on navigation even though auth already finished.
  const readPhase = async (): Promise<LoginPhase> => {
    try {
      return await page.evaluate(() => {
        if (document.querySelector('[data-testid="onboarding-gate"]')) return 'onboarding';
        if (document.querySelector('[data-testid="login-gate"]')) return 'login';
        const messenger = document.querySelector('[data-testid="messenger-workspace"]');
        if (messenger?.getAttribute('data-initial-host-hydrated') === 'true') return 'ready';
        return 'waiting';
      }) as LoginPhase;
    } catch {
      return 'waiting';
    }
  };

  for (let phase = 0; phase < 12; phase += 1) {
    await expect.poll(readPhase, { timeout: 15_000 }).not.toBe('waiting');
    const currentPhase = await readPhase();

    if (currentPhase === 'onboarding') {
      await page.getByTestId('onboarding-next').click();
      continue;
    }
    if (currentPhase === 'login') {
      await page.getByTestId('browser-login-start').click();
      await expect(loginGate).toBeHidden();
      continue;
    }
    if (currentPhase === 'ready') break;
  }

  await expect(workspace).toHaveAttribute('data-initial-host-hydrated', 'true', { timeout: 15_000 });
  await expect(workspace).toBeVisible();
}

async function openMessenger(page: Page): Promise<void> {
  const workspace = page.getByTestId('messenger-workspace');
  await expect(workspace).toBeVisible();
  await expect(page.getByTestId('profile-navigation-trigger')).toBeVisible();
  await expect(page.getByTestId('open-messenger')).toHaveCount(0);
}

async function getMessagingIdentity(page: Page): Promise<{ actorId: string; deviceId: string; sessionId: string }> {
  return page.evaluate(async () => {
    const bridge = (window as unknown as {
      fabushiNative: {
        invoke<T>(method: string, params?: Record<string, unknown>): Promise<T>;
      };
    }).fabushiNative;
    return bridge.invoke<{ actorId: string; deviceId: string; sessionId: string }>('getMessagingIdentity', {
      deviceId: 'desktop:e2e-unread',
      sessionId: `messenger-e2e:${Date.now()}`,
    });
  });
}

async function executeMessagingCommand(
  page: Page,
  actorId: string,
  command: Record<string, unknown>,
  suffix: string,
): Promise<void> {
  await page.evaluate(
    async ({ actorId: envelopeActorId, command: envelopeCommand, suffix: requestSuffix }) => {
      const bridge = (window as unknown as {
        mahayana: {
          invoke<T>(method: string, params?: Record<string, unknown>): Promise<T>;
        };
      }).mahayana;
      const requestId = `messaging-e2e-${requestSuffix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
      await bridge.invoke('feature.execute', {
        command: {
          type: 'messaging.execute',
          requestId,
          envelope: {
            protocolVersion: 2,
            context: {
              requestId,
              deviceId: `e2e:${envelopeActorId}`,
              actorId: envelopeActorId,
              sessionId: `e2e:${envelopeActorId}`,
              sentAtMs: Date.now(),
            },
            command: envelopeCommand,
          },
        },
      });
    },
    { actorId, command, suffix },
  );
}

test('desktop Messenger unifies Telegram-class navigation with Fabushi agent identity', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMessenger(page);

    const profileNavigation = page.getByTestId('profile-navigation-trigger');
    const profileMark = profileNavigation.locator('[data-engine="fabushi-motion-v3"]').first();
    await expect(profileMark).toBeVisible();
    await expect(profileMark).toHaveAttribute('data-motion-tier', 'ambient');
    await profileNavigation.click();
    await expect(page.getByTestId('profile-navigation-menu')).toBeVisible();
    for (const label of [
      '聊天',
      '联系人',
      'Bots',
      '群组',
      '频道',
      '通话',
      '收藏',
      '归档',
      '文件夹',
      'Mini Apps',
      '支付',
      '设置',
    ]) {
      await expect(page.getByTitle(label, { exact: true })).toBeVisible();
    }
    await page.getByTestId('profile-navigation-trigger').click();

    const sidebar = page.getByTestId('messenger-sidebar');
    const expandedBox = await sidebar.boundingBox();
    expect(expandedBox?.width ?? 0).toBeGreaterThan(200);
    await page.getByTestId('sidebar-resizer').dblclick();
    await expect(sidebar).toHaveAttribute('data-collapsed', 'true');
    const collapsedBox = await sidebar.boundingBox();
    expect(collapsedBox?.width ?? 999).toBeLessThanOrEqual(112);
    await page.getByTestId('sidebar-resizer').dblclick();
    await expect(sidebar).not.toHaveAttribute('data-collapsed', 'true');

    await page.getByTestId('global-search-trigger').click();
    const searchSurface = page.getByTestId('global-search-surface');
    await expect(searchSurface).toBeVisible();
    const searchSurfaceBox = await searchSurface.boundingBox();
    const searchSidebarBox = await sidebar.boundingBox();
    expect(searchSurfaceBox?.x ?? 9999).toBeGreaterThanOrEqual(searchSidebarBox?.x ?? 0);
    expect((searchSurfaceBox?.x ?? 0) + (searchSurfaceBox?.width ?? 9999)).toBeLessThanOrEqual((searchSidebarBox?.x ?? 0) + (searchSidebarBox?.width ?? 0) + 1);
    for (const category of ['chats', 'channels', 'apps', 'posts', 'images', 'videos', 'downloads', 'links', 'files', 'music', 'audio']) {
      await expect(page.getByTestId(`global-search-tab-${category}`)).toBeVisible();
    }
    await page.getByRole('button', { name: '关闭搜索' }).click();

    const assistant = page.getByTestId('peer-legacy:conversation:mahayana-ai:agent:assistant');
    await expect(assistant).toBeVisible();
    await expect(assistant.locator('[data-engine="fabushi-motion-v3"]').first()).toBeVisible();
    await assistant.click();
    await expect(page.getByTestId('messenger-input')).toBeVisible();

    await page.getByTestId('messenger-input').fill('统一消息链路验收');
    await page.getByTestId('messenger-send').click();
    await expect(page.getByTestId('message-list').locator(':scope > article').getByText('统一消息链路验收', { exact: true })).toBeVisible({ timeout: 1_500 });
    await expect(page.getByTestId('message-list').locator(':scope > article').getByText('收到：统一消息链路验收', { exact: true })).toBeVisible();
    await expect(page.locator('[data-testid="agent-step"]:visible')).toHaveCount(0);

    await page.getByTitle('置顶').click();
    await page.getByTitle('静音').click();
    await expect(page.getByTitle('开启通知')).toBeVisible();

    await page.getByTitle('搜索当前会话').click();
    await expect(page.getByTestId('conversation-search-scope')).toContainText('此聊天');
    await expect(page.getByTestId('global-search-input')).toBeFocused();
    await expect(page.getByTestId('global-search-surface')).toHaveAttribute('data-scoped', 'true');
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('Router settings modal binds providers, usage, sandbox, preferences and fast-start projection', async ({}, testInfo) => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-settings-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMessenger(page);

    const cachedLegacyPeer = page.getByTestId('peer-legacy:conversation:mahayana-ai:agent:assistant');
    await expect(cachedLegacyPeer).toBeVisible();
    await expect.poll(async () => page.evaluate(() => {
      const projection = JSON.parse(localStorage.getItem('fabushi.desktop.messenger-projection.v1') || 'null');
      return Boolean(projection?.legacyConversations?.some((item: { id?: string }) => item.id === 'mahayana-ai:agent:assistant'));
    }), { timeout: 5_000 }).toBe(true);

    await page.getByTestId('profile-navigation-trigger').click();
    await page.getByTitle('设置', { exact: true }).click();
    await expect(page.getByTestId('settings-modal-backdrop')).toBeVisible();
    await expect(page.getByTestId('telegram-settings-navigation')).toBeVisible();
    await expect(page.getByTestId('telegram-settings-workspace')).toBeVisible();

    for (const category of ['account', 'router', 'usage', 'updates']) {
      await expect(page.getByTestId(`settings-category-${category}`)).toBeVisible();
    }

    await page.getByTestId('settings-category-router').click();
    await expect(page.getByTestId('router-provider-settings')).toBeVisible();
    await expect(page.getByTestId('router-provider-select')).toHaveValue('fabushi');
    // Native <option> disabled-state accessibility differs across Electron's
    // platform builds. The DOM attribute is the cross-platform product contract.
    await expect(page.getByTestId('router-provider-claude-code')).toHaveAttribute('disabled', '');
    await expect(page.getByTestId('router-provider-openrouter')).toHaveAttribute('disabled', '');
    await expect(page.getByTestId('router-usage-settings')).toContainText('tokens');
    await expect(page.getByTestId('router-sandbox-host')).toHaveAttribute('data-selected', 'true');
    const localDockerSandbox = page.getByTestId('router-sandbox-local-docker');
    const localDockerAvailable = (await localDockerSandbox.textContent())?.includes('可用') ?? false;
    expect(await localDockerSandbox.getAttribute('disabled')).toBe(localDockerAvailable ? null : '');
    await testInfo.attach('router-settings-modal', { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });

    await page.getByTestId('settings-category-usage').click();
    await expect(page.getByTestId('usage-billing-settings')).toContainText('最近 7 天');
    await page.getByTestId('settings-category-updates').click();
    await expect(page.getByTestId('updates-settings')).toContainText('electron-updater');
    await expect(page.getByTestId('settings-update-track')).toHaveValue('stable');

    await page.getByTestId('settings-category-account').click();
    await expect(page.getByTestId('settings-theme')).toBeVisible();
    await expect(page.getByTestId('settings-local-tool-permission')).toBeVisible();
    await expect(page.getByTestId('settings-time-zone')).toBeVisible();
    await expect(page.getByText('Enter 发送消息')).toBeVisible();
    await expect(page.getByText('显示资料侧栏')).toBeVisible();
    await expect(page.getByText('减少动态效果')).toBeVisible();
    await page.getByTestId('settings-toggle-reduced-motion').check();
    await expect(page.getByTestId('messenger-workspace')).toHaveAttribute('data-reduce-motion', 'true');

    const preferences = await page.evaluate(() => JSON.parse(localStorage.getItem('fabushi.desktop.telegram-settings.v1') || '{}'));
    expect(preferences.reducedMotion).toBe(true);

    await page.getByTestId('settings-close').click();
    await expect(page.getByTestId('settings-modal-backdrop')).toHaveCount(0);
    await page.getByTestId('profile-navigation-trigger').click();
    await page.getByTitle('聊天', { exact: true }).click();
    await page.getByRole('button', { name: '新建', exact: true }).click();
    await page.getByRole('button', { name: '新建频道' }).click();
    await page.getByPlaceholder('频道名称').fill('本地优先投影验收');
    await page.getByPlaceholder('频道简介').fill('Telegram-style local-first');
    await page.getByRole('button', { name: '创建频道' }).click();
    const channelPeer = page.locator('[data-testid^="peer-selfhosted:channel:"]').filter({ hasText: '本地优先投影验收' }).first();
    await expect(channelPeer).toBeVisible();
    await channelPeer.click();

    await expect.poll(async () => page.evaluate(() => {
      const projection = JSON.parse(localStorage.getItem('fabushi.desktop.messenger-projection.v1') || 'null');
      return Boolean(projection?.activePeerKey?.startsWith('selfhosted:') && projection?.selfConversations?.some((item: { title?: string }) => item.title === '本地优先投影验收'));
    }), { timeout: 5_000 }).toBe(true);

    await page.reload({ waitUntil: 'domcontentloaded' });
    await expect(page.getByTestId('messenger-workspace')).toBeVisible({ timeout: 5_000 });
    await expect(page.getByTestId('messenger-workspace')).toHaveAttribute('data-testid-ready-projection', 'true');
    await expect(page.getByText('本地优先投影验收').first()).toBeVisible();
    await expect(page.getByTestId('peer-legacy:conversation:mahayana-ai:agent:assistant')).toBeVisible();
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('account settings logs out and clears account-scoped fast-start caches', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-logout-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMessenger(page);

    const assistant = page.getByTestId('peer-legacy:conversation:mahayana-ai:agent:assistant');
    await expect(assistant).toBeVisible();
    await assistant.click();
    await page.getByTestId('messenger-input').fill('退出登录缓存清理验收');
    await page.getByTestId('messenger-send').click();
    await expect(page.getByTestId('message-list').locator(':scope > article').getByText('收到：退出登录缓存清理验收', { exact: true })).toBeVisible();
    await expect.poll(async () => page.evaluate(() => {
      const journal = JSON.parse(localStorage.getItem('fabushi.desktop.mahayana-conversation-journal.v1') || 'null');
      return Object.keys(journal?.conversations ?? {}).length;
    })).toBeGreaterThan(0);

    await page.evaluate(() => {
      localStorage.setItem('fabushi.desktop.messenger-projection.v1', JSON.stringify({ version: 1, selfActors: [], selfConversations: [], selfMessages: {}, savedAtMs: Date.now() }));
      localStorage.setItem('fabushi.desktop.messenger-drafts.v2', JSON.stringify({ sample: 'private draft' }));
    });

    await page.getByTestId('profile-navigation-trigger').click();
    await page.getByTitle('设置', { exact: true }).click();
    await page.getByTestId('settings-category-account').click();
    const logout = page.getByTestId('settings-logout');
    await expect(logout).toHaveAttribute('data-agent-id', 'settings-logout');
    await expect(logout).toBeVisible();
    await expect(logout).toHaveText('退出登录');
    await logout.click();

    await expect(page.getByTestId('login-gate')).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId('messenger-workspace')).toHaveCount(0);
    const accountCaches = await page.evaluate(() => [
      localStorage.getItem('fabushi.desktop.messenger-projection.v1'),
      localStorage.getItem('fabushi.desktop.messenger-drafts.v2'),
      localStorage.getItem('fabushi.desktop.mahayana-conversation-journal.v1'),
    ]);
    expect(accountCaches).toEqual([null, null, null]);

    await page.getByTestId('browser-login-start').click();
    await expect(page.getByTestId('messenger-workspace')).toBeVisible({ timeout: 15_000 });
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('returning-user local-first conversation list is interactive within the one-second target', async ({}, testInfo) => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-startup-perf-e2e-'));
  let app = await launchDesktopApp(appDataDir);

  try {
    let page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMessenger(page);

    await page.getByRole('button', { name: '新建', exact: true }).click();
    await page.getByRole('button', { name: '新建频道' }).click();
    await page.getByPlaceholder('频道名称').fill('首屏性能验收');
    await page.getByPlaceholder('频道简介').fill('local-first startup timing');
    await page.getByRole('button', { name: '创建频道' }).click();
    const seededPeer = page.locator('[data-testid^="peer-selfhosted:channel:"]').filter({ hasText: '首屏性能验收' }).first();
    await expect(seededPeer).toBeVisible();
    await seededPeer.click();

    const identity = await getMessagingIdentity(page);
    const seededConversationId = await page.evaluate(() => {
      const projection = JSON.parse(localStorage.getItem('fabushi.desktop.messenger-projection.v1') || 'null');
      const activePeerKey = typeof projection?.activePeerKey === 'string' ? projection.activePeerKey : '';
      return activePeerKey.startsWith('selfhosted:') ? activePeerKey.slice('selfhosted:'.length) : '';
    });
    expect(seededConversationId).not.toBe('');
    const historySeedCount = 32;
    for (let index = 0; index < historySeedCount; index += 1) {
      await executeMessagingCommand(page, identity.actorId, {
        type: 'sendMessage',
        conversationId: seededConversationId,
        clientMessageId: `desktop:e2e-startup-history:${index}`,
        content: { type: 'text', data: { text: { text: `startup-history-${String(index).padStart(2, '0')}`, entities: [] } } },
        replyToMessageId: null,
        threadRootMessageId: null,
        scheduledAtMs: null,
        silent: false,
        protectedContent: false,
      }, `startup-history-${index}`);
    }

    await expect.poll(async () => page.evaluate((conversationId) => {
      const projection = JSON.parse(localStorage.getItem('fabushi.desktop.messenger-projection.v1') || 'null');
      return projection?.selfMessages?.[conversationId]?.length ?? 0;
    }, seededConversationId), { timeout: 10_000 }).toBeGreaterThanOrEqual(historySeedCount);
    await expect.poll(async () => page.evaluate(async ({ conversationId, minimumMessages }) => {
      const bridge = (window as unknown as {
        fabushiNative?: { invoke<T>(method: string, params?: Record<string, unknown>): Promise<T> };
      }).fabushiNative;
      if (!bridge) return false;
      const projection = await bridge.invoke<{
        activePeerKey?: string;
        selfConversations?: Array<{ title?: string }>;
        selfMessages?: Record<string, unknown[]>;
      } | null>('readClientPersistence', { key: 'fabushi.desktop.messenger-projection.v1' });
      return Boolean(projection?.activePeerKey === `selfhosted:${conversationId}`
        && projection?.selfConversations?.some((item) => item.title === '首屏性能验收')
        && (projection?.selfMessages?.[conversationId]?.length ?? 0) >= minimumMessages);
    }, { conversationId: seededConversationId, minimumMessages: historySeedCount }), { timeout: 10_000 }).toBe(true);

    await app.close();

    const launchStartedAtMs = Date.now();
    app = await launchDesktopApp(appDataDir);
    page = await app.firstWindow();
    const workspace = page.getByTestId('messenger-workspace');
    const projectedPeer = page.locator('[data-testid^="peer-selfhosted:channel:"]').filter({ hasText: '首屏性能验收' }).first();

    await expect(workspace).toBeVisible({ timeout: 5_000 });
    await expect(workspace).toHaveAttribute('data-testid-ready-projection', 'true');
    await expect.poll(async () => page.evaluate(() => {
      const projection = JSON.parse(localStorage.getItem('fabushi.desktop.messenger-projection.v1') || 'null');
      return Boolean(projection?.selfConversations?.some((item: { title?: string }) => item.title === '首屏性能验收'));
    }), { timeout: 2_000 }).toBe(true);
    await expect(projectedPeer).toBeVisible({ timeout: 5_000 });

    const rendererToConversationListMs = await page.evaluate(() => performance.now());
    const launchToConversationListMs = Date.now() - launchStartedAtMs;
    await projectedPeer.click();
    await expect(page.getByTestId('messenger-input')).toBeVisible({ timeout: 2_000 });
    await expect(page.getByTestId('message-list').getByText('startup-history-31', { exact: true })).toBeVisible({ timeout: 5_000 });
    const rendererToComposerInteractiveMs = await page.evaluate(() => performance.now());

    // The async account-status poll must not replace the locally restored Messenger
    // with the login shell after first paint. Waiting longer than the retry cadence
    // turns the previous transient failure into an explicit returning-session gate.
    await page.waitForTimeout(1_100);
    await expect(page.getByTestId('login-gate')).toHaveCount(0);
    await expect(workspace).toBeVisible();
    await expect(projectedPeer).toBeVisible();

    const requiredPhases = ['P0', 'P1', 'P2', 'P3', 'P4', 'P5', 'P6', 'P7', 'P8', 'P9'];
    await expect.poll(async () => page.evaluate(() => {
      const trace = (window as unknown as {
        __fabushiStartupCriticalPath?: { entries?: Array<{ phase?: string }> };
      }).__fabushiStartupCriticalPath;
      return [...new Set((trace?.entries ?? []).map((entry) => entry.phase).filter(Boolean))];
    }), { timeout: 10_000 }).toEqual(expect.arrayContaining(requiredPhases));

    const startupCriticalPath = await page.evaluate(() => {
      const trace = (window as unknown as {
        __fabushiStartupCriticalPath?: Record<string, unknown>;
      }).__fabushiStartupCriticalPath;
      return trace ? JSON.parse(JSON.stringify(trace)) as Record<string, unknown> : null;
    });
    expect(startupCriticalPath).not.toBeNull();

    const evidence = {
      targetMs: 1_000,
      metric: 'renderer-navigation-to-cached-conversation-list-interactive',
      rendererToConversationListMs: Math.round(rendererToConversationListMs * 100) / 100,
      rendererToComposerInteractiveMs: Math.round(rendererToComposerInteractiveMs * 100) / 100,
      launchToConversationListMs,
      packaged: Boolean(packagedExecutable),
      platform: process.platform,
      passed: rendererToConversationListMs < 1_000,
    };
    const evidenceJson = `${JSON.stringify(evidence, null, 2)}\n`;
    console.log(`[startup-performance] ${JSON.stringify(evidence)}`);
    await writeFile(testInfo.outputPath('startup-performance.json'), evidenceJson);
    await testInfo.attach('startup-performance', { body: Buffer.from(evidenceJson), contentType: 'application/json' });

    const criticalPathEvidence = {
      taskId: 'M3-DESKTOP-003',
      exactHead: process.env.GITHUB_SHA?.trim() || null,
      diagnosticOnly: true,
      rootCauseClaim: null,
      historySeedCount,
      initialSyncLimitBoundary: 20,
      packaged: Boolean(packagedExecutable),
      platform: process.platform,
      trace: startupCriticalPath,
    };
    const criticalPathJson = `${JSON.stringify(criticalPathEvidence, null, 2)}\n`;
    console.log(`[startup-critical-path] ${JSON.stringify(criticalPathEvidence)}`);
    await writeFile(testInfo.outputPath('startup-critical-path.json'), criticalPathJson);
    await testInfo.attach('startup-critical-path', { body: Buffer.from(criticalPathJson), contentType: 'application/json' });
    await testInfo.attach('startup-critical-path-screen', { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });

    expect(
      rendererToConversationListMs,
      `cached conversation list must become interactive within 1000ms; measured ${rendererToConversationListMs.toFixed(2)}ms`,
    ).toBeLessThan(1_000);
  } finally {
    await app.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('desktop Messenger creates a self-hosted channel and executes message mutation commands', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-channel-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMessenger(page);

    await page.getByRole('button', { name: '新建', exact: true }).click();
    await page.getByRole('button', { name: '新建频道' }).click();
    await expect(page.getByText('Fabushi 自建广播会话')).toBeVisible();
    await page.getByPlaceholder('频道名称').fill('自建频道验收');
    await page.getByPlaceholder('频道简介').fill('不依赖 Telegram API');
    await page.getByRole('button', { name: '创建频道' }).click();

    const channelPeer = page.locator('[data-testid^="peer-selfhosted:channel:"]').filter({ hasText: '自建频道验收' }).first();
    await expect(channelPeer).toBeVisible();
    await expect(page.getByText('不依赖 Telegram API').first()).toBeVisible();

    await page.getByTestId('messenger-input').fill('自建频道消息');
    await page.getByTestId('messenger-send').click();
    const message = page.locator('article').filter({ hasText: '自建频道消息' }).last();
    await expect(message).toBeVisible();

    await message.click({ button: 'right' });
    await page.getByRole('button', { name: /反应/ }).click();
    await expect(message.getByText('👍 1')).toBeVisible();

    await message.click({ button: 'right' });
    await page.getByRole('button', { name: '编辑' }).click();
    await page.getByTestId('edit-message-input').fill('编辑后的频道消息');
    await page.getByRole('button', { name: '保存' }).click();
    await expect(page.getByText('编辑后的频道消息')).toBeVisible();

    const edited = page.locator('article').filter({ hasText: '编辑后的频道消息' }).last();
    await edited.click({ button: 'right' });
    await page.getByRole('button', { name: /^置顶$/ }).last().click();
    await expect(edited.locator('svg')).toHaveCount(2);

    await page.getByTitle('发送账单').click();
    await page.getByTestId('invoice-title-input').fill('E2E 账单');
    await page.getByTestId('invoice-amount-input').fill('1.99');
    await page.getByRole('button', { name: '创建账单' }).click();
    await expect(page.getByText('🧾 账单')).toBeVisible();

    await edited.click({ button: 'right' });
    await page.getByRole('button', { name: '删除' }).click();
    await expect(page.getByText('编辑后的频道消息')).toBeHidden();
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('desktop Messenger creates a real Bot collaboration group and sends into it', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-group-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMessenger(page);

    await expect(page.getByText('Research Bot', { exact: true })).toHaveCount(0);
    await expect(page.getByText('Incident Bot', { exact: true })).toHaveCount(0);

    await page.getByRole('button', { name: '新建', exact: true }).click();
    await expect(page.getByTestId('create-bot')).toBeVisible();
    await page.getByTestId('create-bot').click();
    await page.getByTestId('new-bot-name').fill('协作验收 Bot');
    await page.getByTestId('new-bot-description').fill('由用户创建，用于群组协作验收。');
    await page.getByTestId('create-bot-submit').click();
    await expect(page.locator('[data-testid^="peer-legacy:bot:"]').filter({ hasText: '协作验收 Bot' }).first()).toBeVisible({ timeout: 10_000 });

    await page.getByRole('button', { name: '新建', exact: true }).click();
    await page.getByRole('button', { name: '新建群组' }).click();
    await expect(page.getByText('现有 AI 群组 Host 会执行 Bot 多轮协作')).toBeVisible();
    await page.getByPlaceholder('群组名称').fill('人机协作验收群');

    const researchBot = page.locator('[data-testid^="group-bot-"]').filter({ hasText: '协作验收 Bot' }).first();
    await expect(researchBot).toBeVisible();
    await researchBot.click();
    await expect(researchBot).toHaveAttribute('data-selected', 'true');
    await expect(page.getByRole('button', { name: '创建群组' })).toBeEnabled();
    await page.getByRole('button', { name: '创建群组' }).click();

    const groupPeer = page.locator('[data-testid^="peer-legacy:group:"]').filter({ hasText: '人机协作验收群' }).first();
    await expect(groupPeer).toBeVisible();
    await groupPeer.click();
    await page.getByTestId('messenger-input').fill('群组消息链路验收');
    await page.getByTestId('messenger-send').click();
    await expect(page.getByText('群组消息链路验收')).toBeVisible();
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('online Mini App installs and opens from global Application search', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-miniapp-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);

    await openMessenger(page);
    await page.getByTestId('global-search-trigger').click();
    await page.getByTestId('global-search-tab-apps').click();
    await page.getByTestId('global-search-input').fill('全球法布施');
    const appResult = page.getByTestId('global-search-app-global-dharma');
    await expect(appResult).toBeVisible();
    const install = appResult.getByRole('button', { name: '安装' });
    if (await install.isVisible().catch(() => false)) {
      await install.click();
    }
    const open = appResult.getByRole('button', { name: '打开' });
    await expect(open).toBeVisible();
    await open.click();
    await expect(page.getByText('Mini App · 已安装线上包 · 账号云同步')).toBeVisible();
    await expect(page.locator('iframe[title="global-dharma"]')).toBeVisible();
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('desktop Messenger persists per-peer drafts and performs real in-conversation search', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-draft-search-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMessenger(page);

    const assistant = page.getByTestId('peer-legacy:conversation:mahayana-ai:agent:assistant');
    await assistant.click();
    const input = page.getByTestId('messenger-input');
    const draft = '每个会话独立保存的草稿';
    await input.fill(draft);

    await page.reload();
    await completeBrowserLogin(page);
    await openMessenger(page);
    await page.getByTestId('peer-legacy:conversation:mahayana-ai:agent:assistant').click();
    await expect(page.getByTestId('messenger-input')).toHaveValue(draft);

    const marker = `会话搜索唯一标记-${Date.now()}`;
    await page.getByTestId('messenger-input').fill(marker);
    await page.getByTestId('messenger-send').click();
    await expect(page.getByTestId('message-list').locator(':scope > article').getByText(marker, { exact: true })).toBeVisible();

    await page.getByTitle('搜索当前会话').click();
    await expect(page.getByTestId('conversation-search-scope')).toContainText('此聊天');
    const search = page.getByTestId('global-search-input');
    await search.fill(marker);
    const scopedSurface = page.getByTestId('global-search-surface');
    await expect(scopedSurface).toContainText(marker);
    await expect(page.getByTestId('global-search-tab-posts')).toHaveText('消息');
    await expect(page.getByTestId('global-search-tab-chats')).toHaveCount(0);

    await search.fill('绝对不存在的会话内搜索结果-20260822');
    await expect(scopedSurface.getByText('当前已加载内容中没有匹配结果')).toBeVisible();
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('desktop Messenger rejects self-hosted actor impersonation at the real Host boundary', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-auth-boundary-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMessenger(page);

    const identity = await getMessagingIdentity(page);
    const now = Date.now();
    const peerActorId = `human:e2e:forged:${now}`;

    await executeMessagingCommand(page, identity.actorId, {
      type: 'upsertProfile',
      actor: {
        id: identity.actorId,
        kind: 'human',
        displayName: 'Authenticated E2E User',
        capabilities: ['messages'],
        presence: { status: 'online', lastSeenAtMs: now },
        verified: false,
      },
    }, 'current-profile');

    let rejection = '';
    try {
      await executeMessagingCommand(page, peerActorId, {
        type: 'upsertProfile',
        actor: {
          id: peerActorId,
          kind: 'human',
          displayName: 'Forged Peer',
          capabilities: ['messages'],
          presence: { status: 'online', lastSeenAtMs: now },
          verified: false,
        },
      }, 'forged-peer-profile');
    } catch (cause) {
      rejection = cause instanceof Error ? cause.message : String(cause);
    }
    expect(rejection).toContain('Messaging envelope actor does not match authenticated account');
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
