import { _electron as electron, expect, test, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
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
  while (await page.getByTestId('onboarding-gate').isVisible().catch(() => false)) {
    await page.getByTestId('onboarding-next').click();
  }
  const loginGate = page.getByTestId('login-gate');
  if (await loginGate.isVisible().catch(() => false)) {
    await page.getByTestId('browser-login-start').click();
    await expect(loginGate).toBeHidden();
  }
  await expect(page.getByTestId('host-status')).toHaveAttribute('data-state', 'ready');
}

async function openMessenger(page: Page): Promise<void> {
  await page.getByTestId('open-messenger').click();
  await expect(page.getByTestId('messenger-workspace')).toBeVisible();
  await expect(page.getByTitle('聊天')).toBeVisible();
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

test('desktop Messenger exposes Telegram-class navigation and preserves the real AI Host', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMessenger(page);

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

    const assistant = page.getByTestId('peer-legacy:conversation:codex:agent:assistant');
    await expect(assistant).toBeVisible();
    await assistant.click();
    await expect(page.getByTestId('messenger-input')).toBeVisible();

    await page.getByTestId('messenger-input').fill('统一消息链路验收');
    await page.getByTestId('messenger-send').click();
    await expect(page.getByText('收到：统一消息链路验收')).toBeVisible();

    await page.getByTitle('置顶').click();
    await page.getByTitle('静音').click();
    await expect(page.getByTitle('开启通知')).toBeVisible();

    await page.getByTitle('搜索当前会话').click();
    await expect(page.getByPlaceholder('在当前会话中搜索')).toBeVisible();
  } finally {
    await app.close();
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

    await page.getByRole('button', { name: '新建频道' }).first().click();
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

    await page.getByRole('button', { name: '新建群组' }).first().click();
    await expect(page.getByText('现有 AI 群组 Host 会执行 Bot 多轮协作')).toBeVisible();
    await page.getByPlaceholder('群组名称').fill('人机协作验收群');

    const researchBot = page.getByTestId('group-bot-research-bot');
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

test('installed Mini App opens from the unified Messenger surface', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-miniapp-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);

    await page.getByTestId('open-marketplace').click();
    const install = page.getByTestId('install-miniapp');
    if (await install.isEnabled()) await install.click();
    await expect(install).toBeDisabled();
    await page.getByRole('button', { name: '关闭插件市场' }).click();

    await openMessenger(page);
    await page.getByTitle('Mini Apps').click();
    await page.getByRole('button', { name: /全球法布施/ }).last().click();
    await expect(page.getByText('Mini App · 受控宿主容器')).toBeVisible();
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

    const assistant = page.getByTestId('peer-legacy:conversation:codex:agent:assistant');
    await assistant.click();
    const input = page.getByTestId('messenger-input');
    const draft = '每个会话独立保存的草稿';
    await input.fill(draft);

    await page.reload();
    await completeBrowserLogin(page);
    await openMessenger(page);
    await page.getByTestId('peer-legacy:conversation:codex:agent:assistant').click();
    await expect(page.getByTestId('messenger-input')).toHaveValue(draft);

    const marker = `会话搜索唯一标记-${Date.now()}`;
    await page.getByTestId('messenger-input').fill(marker);
    await page.getByTestId('messenger-send').click();
    await expect(page.getByText(marker, { exact: true })).toBeVisible();

    await page.getByTitle('搜索当前会话').click();
    const search = page.getByTestId('conversation-search-input');
    await search.fill(marker);
    await expect(page.locator('article').filter({ hasText: marker })).toHaveCount(1);

    await search.fill('绝对不存在的会话内搜索结果-20260822');
    await expect(page.getByTestId('message-search-empty')).toBeVisible();
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('desktop Messenger projects unread from another self-hosted actor and consumes it on open', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-messenger-unread-e2e-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMessenger(page);

    const identity = await getMessagingIdentity(page);
    const now = Date.now();
    const peerActorId = `human:e2e:unread:${now}`;
    const conversationId = `direct:e2e-unread-${now}`;
    const permissions = {
      canSendMessages: true,
      canSendMedia: true,
      canSendPolls: true,
      canAddMembers: true,
      canPinMessages: true,
      canManageTopics: true,
      canManageCalls: true,
    };

    await executeMessagingCommand(page, peerActorId, {
      type: 'upsertProfile',
      actor: {
        id: peerActorId,
        kind: 'human',
        displayName: 'Unread E2E Peer',
        capabilities: ['messages'],
        presence: { status: 'online', lastSeenAtMs: now },
        verified: false,
      },
    }, 'peer-profile');

    await executeMessagingCommand(page, identity.actorId, {
      type: 'createConversation',
      conversation: {
        id: conversationId,
        kind: 'direct',
        title: 'Unread E2E Conversation',
        participants: [
          { actorId: identity.actorId, role: 'owner', joinedAtMs: now },
          { actorId: peerActorId, role: 'member', joinedAtMs: now },
        ],
        ownerId: identity.actorId,
        unreadCount: 0,
        mentionCount: 0,
        pinnedMessageIds: [],
        notificationSettings: { showPreview: true, notifyMentions: true },
        permissions,
        historyVisibility: 'allMembers',
        topics: [],
        folderIds: [],
        archived: false,
        pinned: false,
        markedUnread: false,
        createdAtMs: now,
        updatedAtMs: now,
      },
    }, 'conversation');

    const incomingText = `unread-e2e-${now}`;
    await executeMessagingCommand(page, peerActorId, {
      type: 'sendMessage',
      conversationId,
      clientMessageId: `e2e:${now}`,
      content: { type: 'text', data: { text: { text: incomingText, entities: [] } } },
      replyToMessageId: null,
      threadRootMessageId: null,
      scheduledAtMs: null,
      silent: false,
      protectedContent: false,
    }, 'incoming-message');

    const peer = page.getByTestId(`peer-selfhosted:${conversationId}`);
    await expect(peer).toBeVisible({ timeout: 15_000 });
    await expect(peer.locator('b')).toHaveText('1');

    await peer.click();
    await expect(page.getByText(incomingText, { exact: true })).toBeVisible();
    await expect(peer.locator('b')).toHaveCount(0, { timeout: 15_000 });
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
