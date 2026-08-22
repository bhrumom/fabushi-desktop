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
