import { expect, test, type Page, type TestInfo } from '@playwright/test';
import { createServer, type ViteDevServer } from 'vite';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import type {} from './fixtures/native-message-projection';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
let server: ViteDevServer;
let origin: string;
test.describe.configure({ retries: 0, timeout: 90_000 });
test.use({ viewport: { width: 1280, height: 900 }, reducedMotion: 'reduce' });

test.beforeAll(async () => {
  server = await createServer({
    configFile: path.join(appRoot, 'vite.config.ts'), root: appRoot,
    server: { host: '127.0.0.1', port: 0, strictPort: false },
  });
  await server.listen();
  const address = server.httpServer!.address();
  if (address == null || typeof address === 'string') throw new Error('Fixture did not bind a loopback port');
  origin = `http://127.0.0.1:${address.port}`;
});

test.afterAll(async () => { await server?.close(); });

const messageRow = (page: Page, id: string) => page.getByRole('log', { name: 'Conversation transcript' }).locator(`[data-entry-id="${id}"]`);
const domIds = (page: Page) => page.getByRole('log', { name: 'Conversation transcript' }).locator('[data-entry-id]').evaluateAll((entries) => entries.map((entry) => entry.getAttribute('data-entry-id')));

async function openAlice(page: Page, theme = 'dark') {
  await page.goto(`${origin}/native-message-projection.html?theme=${theme}`);
  await expect(page.getByRole('button', { name: 'Alice', exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await expect(messageRow(page, 'm3')).toBeVisible();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().older)).toBe(1);
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.requests.filter((request) =>
    request.method === 'getHumanConversationTranscriptTail'
      && request.args.conversationId === 'human:a'
      && (request.args.beforeSeq === undefined || request.args.beforeSeq === null)).length)).toBe(1);
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.requests.filter((request) =>
    request.method === 'getHumanConversationTranscriptTail'
      && request.args.conversationId === 'human:a'
      && request.args.beforeSeq === 3).length)).toBe(1);
}

async function loadOlder(page: Page, expectedIds = ['m1', 'm2', 'm3', 'm4', 'm5']) {
  await page.evaluate(() => window.nativeMessageFixture.releaseOlder());
  await expect.poll(() => domIds(page)).toEqual(expectedIds);
  const olderRequests = await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) =>
    request.method === 'getHumanConversationTranscriptTail' && request.args.beforeSeq === 3));
  expect(olderRequests.length).toBeGreaterThan(0);
  expect(olderRequests.every((request) => request.args.beforeCursor === 'fixture-human-walk:3')).toBe(true);
}

async function selectForward(page: Page, entryId: string) {
  const row = messageRow(page, entryId);
  await row.scrollIntoViewIfNeeded();
  const actionAnchor = row.locator('.sand-message-action-anchor');
  await expect(actionAnchor).toBeVisible();
  await actionAnchor.hover();
  await row.getByRole('button', { name: 'More message actions', exact: true }).click();
  await page.getByRole('menu', { name: 'More message actions' }).getByRole('menuitem', { name: 'Forward', exact: true }).click();
}

async function milestone(page: Page, info: TestInfo, name: string) {
  const manifest = await page.getByRole('log', { name: 'Conversation transcript' }).locator('[data-entry-id]').evaluateAll((entries) => entries.map((entry) => ({
    id: entry.getAttribute('data-entry-id'), text: entry.querySelector('.sand-message')?.textContent,
    pending: entry.getAttribute('data-pending'),
    reactions: [...entry.querySelectorAll('.sand-reaction-pill')].map((reaction) => ({ label: reaction.getAttribute('aria-label'), pressed: reaction.getAttribute('aria-pressed') })),
  })));
  await info.attach(`${name}-dom`, { body: JSON.stringify(manifest, null, 2), contentType: 'application/json' });
  const screenshot = info.outputPath(`${name}.png`);
  await page.screenshot({ path: screenshot, fullPage: true });
  await info.attach(name, { path: screenshot, contentType: 'image/png' });
}

test.beforeEach(async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  (page as Page & { fixtureErrors: string[] }).fixtureErrors = errors;
});

test.afterEach(async ({ page }, info) => {
  const errors = (page as Page & { fixtureErrors: string[] }).fixtureErrors;
  await info.attach('renderer-errors', { body: JSON.stringify(errors), contentType: 'application/json' });
  await info.attach('coordinator-requests', { body: JSON.stringify(await page.evaluate(() => window.nativeMessageFixture?.requests ?? []).catch(() => []), null, 2), contentType: 'application/json' });
  await info.attach('approval-clears', { body: JSON.stringify(await page.evaluate(() => window.nativeMessageFixture?.approvalClears ?? []).catch(() => [])), contentType: 'application/json' });
  expect(errors).toEqual([]);
});

for (const theme of ['light', 'dark']) {
  test(`NMR-26 production transcript preserves paged edits and empty rows through tail refresh and switching (${theme})`, async ({ page }, info) => {
    await openAlice(page, theme);
    await loadOlder(page);
    await milestone(page, info, `loaded-history-${theme}`);
    await page.evaluate(() => {
      window.nativeMessageFixture.mutate('human:a', 'm1', { id: 'm1-canonical' });
      window.nativeMessageFixture.mutate('human:a', 'm2', { content: 'Partial update keeps its attachment' }, ['attachments']);
    });
    await expect.poll(() => domIds(page)).toEqual(['m1-canonical', 'm2', 'm3', 'm4', 'm5']);
    await expect(messageRow(page, 'm1')).toHaveCount(0);
    await expect(messageRow(page, 'm2')).toContainText('Partial update keeps its attachment');
    await expect(messageRow(page, 'm2')).toContainText('note.txt');
    await page.evaluate(() => {
      window.nativeMessageFixture.mutate('human:a', 'm1-canonical', { id: 'm1', content: 'Edited old first message', reactions: [{ emoji: '👍', by: 'me' }] });
      window.nativeMessageFixture.mutate('human:a', 'm2', { content: '', attachments: [], reactions: [] });
      window.nativeMessageFixture.baseline('human:a', ['m4', 'm5']);
    });
    await expect.poll(() => domIds(page)).toEqual(['m1', 'm2', 'm3', 'm4', 'm5']);
    await expect(messageRow(page, 'm1')).toContainText('Edited old first message');
    await expect(messageRow(page, 'm1').locator('.sand-reaction-pill')).toHaveAttribute('aria-pressed', 'true');
    await expect(messageRow(page, 'm2').locator('.sand-message')).not.toContainText('Old second message');
    await expect(messageRow(page, 'm2')).not.toContainText('note.txt');
    await milestone(page, info, `old-row-update-${theme}`);

    await page.getByRole('button', { name: 'Bob', exact: true }).click();
    await expect(messageRow(page, 'b1')).toBeVisible();
    await page.evaluate(() => window.nativeMessageFixture.mutate('human:a', 'm1', { content: 'Updated while Alice is inactive' }));
    await page.getByRole('button', { name: 'Alice', exact: true }).click();
    await expect(messageRow(page, 'm1')).toContainText('Updated while Alice is inactive');
    await expect.poll(() => domIds(page)).toEqual(['m1', 'm2', 'm3', 'm4', 'm5']);
    expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'getHumanConversationTranscriptTail' && request.args.beforeSeq === 3).length)).toBe(1);
    await milestone(page, info, `returned-history-${theme}`);
    await page.reload();
    await expect(messageRow(page, 'm3')).toBeVisible();
    await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().older)).toBe(1);
    await loadOlder(page);
    await expect(messageRow(page, 'm1')).toContainText('Updated while Alice is inactive');
    await expect(messageRow(page, 'm2')).not.toContainText('Old second message');
    await expect(messageRow(page, 'm2')).not.toContainText('note.txt');
    await milestone(page, info, `reloaded-history-${theme}`);
  });
}

test('NMR-26 an initial tail cannot overwrite an identity remap and later empty-row mutation', async ({ page }, info) => {
  await page.goto(`${origin}/native-message-projection.html?holdInitial=1`);
  await expect(page.getByRole('button', { name: 'Alice', exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().initials)).toBe(1);
  await page.evaluate(() => {
    window.nativeMessageFixture.mutate('human:a', 'm3', { id: 'm3-canonical', content: 'First mutation before the initial tail' });
    window.nativeMessageFixture.mutate('human:a', 'm3-canonical', { content: '', attachments: [], reactions: [{ emoji: '🎉', by: 'me' }] });
    window.nativeMessageFixture.mutate('human:a', 'm4', { content: '', attachments: [], reactions: [] });
    window.nativeMessageFixture.mutate('human:a', 'm4', { content: 'Partial update after clearing resources' }, ['attachments']);
    window.nativeMessageFixture.releaseInitial();
  });
  await expect(messageRow(page, 'm3')).toHaveCount(0);
  await expect(messageRow(page, 'm3-canonical').locator('.sand-message')).not.toContainText('First mutation before the initial tail');
  await expect(messageRow(page, 'm3-canonical').locator('.sand-message')).not.toContainText('Tail third message');
  await expect(messageRow(page, 'm3-canonical').locator('.sand-reaction-pill')).toHaveCount(0);
  await expect(messageRow(page, 'm4').locator('.sand-message')).not.toContainText('Tail fourth message');
  await expect(messageRow(page, 'm4')).toContainText('Partial update after clearing resources');
  await expect(messageRow(page, 'm4')).not.toContainText('tail-note.txt');
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().older)).toBe(1);
  await loadOlder(page, ['m1', 'm2', 'm3-canonical', 'm4', 'm5']);
  await milestone(page, info, 'empty-row-after-late-initial');
  // Empty messages retain the existing non-actionable presentation. Restoring
  // content without a reactions field reveals the buffered canonical reaction.
  await page.evaluate(() => window.nativeMessageFixture.mutate('human:a', 'm3-canonical', { content: 'Body restored after empty snapshot' }, ['reactions']));
  await expect(messageRow(page, 'm3-canonical').locator('.sand-reaction-pill')).toContainText('🎉');
  await milestone(page, info, 'initial-tail-after-mutations');
});

test('NMR-26 a canonical clear fences an initial tail that returns after the fresh empty page', async ({ page }, info) => {
  await page.goto(`${origin}/native-message-projection.html?holdInitial=1`);
  await expect(page.getByRole('button', { name: 'Alice', exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().initials)).toBe(1);
  await page.evaluate(() => window.nativeMessageFixture.clear('human:a'));
  // The clear must permit a new read while the old request is still held.
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().initials)).toBe(2);
  await page.evaluate(() => window.nativeMessageFixture.releaseInitial(true));
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().initials)).toBe(0);
  await expect.poll(() => domIds(page)).toEqual([]);
  await expect(messageRow(page, 'm3')).toHaveCount(0);
  expect(await page.evaluate(() => window.nativeMessageFixture.pending().older)).toBe(0);
  await milestone(page, info, 'clear-after-initial-tail-started');
});

for (const order of ['page-first', 'refresh-first']) {
  test(`NMR-26 an older page and full refresh arriving after mutations cannot restore stale content (${order})`, async ({ page }, info) => {
    await openAlice(page);
    await page.evaluate(() => window.nativeMessageFixture.reconnect());
    await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().refreshes)).toBeGreaterThan(0);
    await page.evaluate(() => {
      window.nativeMessageFixture.mutate('human:a', 'm1', { id: 'm1-canonical', content: 'Transient identity remap' });
      window.nativeMessageFixture.mutate('human:a', 'm1-canonical', { content: 'Newer than both outstanding reads', reactions: [{ emoji: '❤️', by: 'peer-a' }] });
      window.nativeMessageFixture.mutate('human:a', 'm2', { content: '', attachments: [], reactions: [] });
    });
    if (order === 'page-first') {
      await loadOlder(page, ['m1-canonical', 'm2', 'm3', 'm4', 'm5']);
      await page.evaluate(() => window.nativeMessageFixture.releaseRefreshes());
    } else {
      await page.evaluate(() => window.nativeMessageFixture.releaseRefreshes());
      await expect(messageRow(page, 'm1-canonical')).toContainText('Newer than both outstanding reads');
      await loadOlder(page, ['m1-canonical', 'm2', 'm3', 'm4', 'm5']);
    }
    await expect(messageRow(page, 'm1-canonical')).toContainText('Newer than both outstanding reads');
    await expect(messageRow(page, 'm1-canonical').locator('.sand-reaction-pill')).toContainText('❤️');
    await expect(messageRow(page, 'm1')).toHaveCount(0);
    await expect(messageRow(page, 'm2').locator('.sand-message')).not.toContainText('Old second message');
    await expect(messageRow(page, 'm2')).not.toContainText('note.txt');
    await expect.poll(() => domIds(page)).toEqual(['m1-canonical', 'm2', 'm3', 'm4', 'm5']);
    await milestone(page, info, `overlapping-reads-settled-${order}`);
  });
}

test('NMR-26 a peer nonce collision, delayed receipt and roster failure preserve the successful canonical send', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  await page.locator('[contenteditable="true"]').fill('Original outgoing content');
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(1);
  await page.evaluate(() => window.nativeMessageFixture.peerCollision());
  await expect(page.locator('[data-entry-id^="pending-"]')).toHaveCount(1);
  await expect(messageRow(page, 'human-server-message:200')).toContainText('Independent peer message');
  await page.evaluate(() => {
    const id = window.nativeMessageFixture.publishSend();
    window.nativeMessageFixture.mutate('human:a', id, { content: '', attachments: [], reactions: [{ emoji: '❤️', by: 'me' }] });
  });
  await expect(page.locator('[data-entry-id^="pending-"]')).toHaveCount(0);
  await expect(messageRow(page, 'human-server-message:100').locator('.sand-message')).not.toContainText('Original outgoing content');
  await page.getByRole('button', { name: 'Bob', exact: true }).click();
  await expect(messageRow(page, 'b1')).toBeVisible();
  await page.evaluate(() => {
    window.nativeMessageFixture.mutate('human:a', 'human-server-message:100', { content: 'Edited after send acknowledgement event' });
    window.nativeMessageFixture.holdSendRosterRefresh();
    window.nativeMessageFixture.settleSend();
  });
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sendRosterRefreshes)).toBe(1);
  await expect.poll(() => domIds(page)).toEqual(['b1']);
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await expect(messageRow(page, 'human-server-message:100')).toContainText('Edited after send acknowledgement event');
  await expect(messageRow(page, 'human-server-message:100').locator('.sand-reaction-pill')).toHaveAttribute('aria-pressed', 'true');
  await expect(messageRow(page, 'human-server-message:100')).not.toContainText('Original outgoing content');
  await expect.poll(() => domIds(page)).toEqual(['m1', 'm2', 'm3', 'm4', 'm5', 'human-server-message:100', 'human-server-message:200']);
  await page.evaluate(() => window.nativeMessageFixture.failSendRosterRefresh());
  await expect(page.getByText('Human roster refresh unavailable: fixture-roster: post-send roster unavailable', { exact: true })).toBeVisible();
  await expect(page.locator('[contenteditable="true"]')).toHaveText('');
  await expect(page.locator('[data-entry-id^="pending-"]')).toHaveCount(0);
  await expect(messageRow(page, 'human-server-message:100')).toContainText('Edited after send acknowledgement event');
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'sendHumanMessage').length)).toBe(1);
  await milestone(page, info, 'late-receipt-after-switch');
});

for (const oldPhase of ['in-flight', 'queued']) {
  test(`NMR-26 logout cancels the ${oldPhase} renderer epoch before another account sends in the same conversation`, async ({ page }, info) => {
    await openAlice(page);
    if (oldPhase === 'queued') await page.evaluate(() => window.nativeMessageFixture.disconnect());
    await page.locator('[contenteditable="true"]').fill('Previous account private message');
    await page.getByRole('button', { name: 'Send message', exact: true }).click();
    const previousRequests = oldPhase === 'in-flight' ? 1 : 0;
    if (oldPhase === 'in-flight') {
      await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(1);
    } else {
      await expect(page.locator('[data-entry-id^="pending-"]')).toContainText('Will send when reconnected');
      expect(await page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(0);
    }
    await page.evaluate(() => window.nativeMessageFixture.setAccount(null));
    await expect(page.getByRole('log', { name: 'Conversation transcript' })).toHaveCount(0);
    await page.evaluate(() => window.nativeMessageFixture.setAccount('next-account'));
    await expect(messageRow(page, 'new-a')).toBeVisible();
    await page.evaluate(() => window.nativeMessageFixture.reconnect());
    await expect(page.locator('[contenteditable="true"]')).toHaveText('');
    await page.locator('[contenteditable="true"]').fill('Current account independent send');
    await page.getByRole('button', { name: 'Send message', exact: true }).click();
    // This request must start while the prior account's HTTP reply is held.
    await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(previousRequests + 1);
    const sendRequests = await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'sendHumanMessage'));
    expect(sendRequests.map((request) => request.args.text)).toEqual(oldPhase === 'in-flight'
      ? ['Previous account private message', 'Current account independent send']
      : ['Current account independent send']);
    expect(await page.evaluate(() => window.nativeMessageFixture.approvalClears)).toEqual([]);
    if (oldPhase === 'in-flight') await page.evaluate(() => window.nativeMessageFixture.settleSend(0));
    await page.evaluate(() => {
      window.nativeMessageFixture.releaseOlder();
      window.nativeMessageFixture.releaseRefreshes();
    });
    await expect(messageRow(page, 'new-a')).toBeVisible();
    await expect(page.getByRole('log', { name: 'Conversation transcript' })).not.toContainText('Previous account private message');
    await expect(page.locator('[data-entry-id^="pending-"]')).toHaveCount(1);
    expect(await page.evaluate(() => window.nativeMessageFixture.approvalClears)).toEqual([]);
    await milestone(page, info, `new-account-send-before-old-${oldPhase}-completion`);

    await page.evaluate((index) => {
      window.nativeMessageFixture.publishSend(index);
      window.nativeMessageFixture.settleSend(index);
    }, previousRequests);
    const canonicalId = `human-server-message:${100 + previousRequests}`;
    await expect(messageRow(page, canonicalId)).toContainText('Current account independent send');
    await expect.poll(() => domIds(page)).toEqual(['new-a', canonicalId]);
    await expect(page.locator('[data-entry-id^="pending-"]')).toHaveCount(0);
    await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.approvalClears)).toEqual(['next-account']);
    await expect(page.locator('[contenteditable="true"]')).toHaveText('');
    await milestone(page, info, `account-fence-after-old-${oldPhase}-replies`);
  });
}

test('NMR-26 old-row reaction commands settle the page cache and cannot overwrite a later empty-row mutation', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  const firstAddReaction = messageRow(page, 'm1').getByRole('button', { name: 'Add reaction', exact: true });
  await firstAddReaction.focus();
  await firstAddReaction.press('Enter');
  const firstReaction = messageRow(page, 'm1').getByRole('button', { name: 'React with 👍', exact: true });
  await firstReaction.focus();
  await firstReaction.press('Enter');
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().reactions)).toBe(1);
  await page.evaluate(() => window.nativeMessageFixture.settleReaction());
  await expect(messageRow(page, 'm1').locator('.sand-reaction-pill')).toContainText('👍');

  const secondAddReaction = messageRow(page, 'm2').getByRole('button', { name: 'Add reaction', exact: true });
  await secondAddReaction.focus();
  await secondAddReaction.press('Enter');
  const secondReaction = messageRow(page, 'm2').getByRole('button', { name: 'React with ❤️', exact: true });
  await secondReaction.focus();
  await secondReaction.press('Enter');
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().reactions)).toBe(2);
  await page.evaluate(() => {
    window.nativeMessageFixture.mutate('human:a', 'm2', { content: '', attachments: [], reactions: [] });
    window.nativeMessageFixture.settleReaction(1);
  });
  await expect(messageRow(page, 'm2').locator('.sand-reaction-pill')).toHaveCount(0);
  await expect(messageRow(page, 'm2').locator('.sand-message')).not.toContainText('Old second message');
  await expect(messageRow(page, 'm2')).not.toContainText('note.txt');
  await expect.poll(() => domIds(page)).toEqual(['m1', 'm2', 'm3', 'm4', 'm5']);
  await milestone(page, info, 'old-row-reaction-receipt');
});

test('NMR-50 persisted Human dispatch failure survives renderer restart; resend is single-flight and selection-ABA safe', async ({ page }, info) => {
  await page.goto(`${origin}/native-message-projection.html`);
  const failedId = await page.evaluate(() => window.nativeMessageFixture.seedFailedIntent('restart-single-flight-nonce'));
  await page.reload();
  await expect(page.getByRole('button', { name: 'Alice', exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  const failedRow = messageRow(page, failedId);
  await expect(failedRow).toBeVisible();
  const resend = failedRow.getByRole('button', { name: 'Resend', exact: true });
  await expect(resend).toBeVisible();
  await resend.evaluate((button) => { button.click(); button.click(); });
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().retries)).toBe(1);
  const request = await page.evaluate(() => window.nativeMessageFixture.requests.find((item) => item.method === 'retryHumanMessage'));
  expect(request?.args).toEqual({ conversationId: 'human:a', clientNonce: 'restart-single-flight-nonce' });

  await page.getByRole('button', { name: 'Bob', exact: true }).click();
  await expect(messageRow(page, 'b1')).toBeVisible();
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await expect(failedRow.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await page.evaluate(() => window.nativeMessageFixture.failRetry(0, 'private stale selection failure'));
  await expect(failedRow.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await expect(page.getByText('private stale selection failure', { exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((item) => item.method === 'retryHumanMessage').length)).toBe(1);
  await milestone(page, info, 'human-retry-selection-aba-after-restart');
});

test('NMR-50 retry single-flight is account-scoped and stale A-B-A replies cannot alter the restored row', async ({ page }, info) => {
  await page.goto(`${origin}/native-message-projection.html`);
  const failedId = await page.evaluate(() => window.nativeMessageFixture.seedFailedIntent('shared-account-retry-nonce'));
  await page.reload();
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  const failedRow = messageRow(page, failedId);
  await expect(failedRow.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await failedRow.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().retries)).toBe(1);

  await page.evaluate(() => window.nativeMessageFixture.setAccount('next-account'));
  await expect(page.getByRole('button', { name: 'Alice', exact: true })).toBeVisible();
  const nextFailedId = await page.evaluate(() => window.nativeMessageFixture.seedFailedIntent('shared-account-retry-nonce'));
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  const nextFailedRow = messageRow(page, nextFailedId);
  await expect(nextFailedRow.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await nextFailedRow.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().retries)).toBe(2);
  expect(await page.evaluate(() => window.nativeMessageFixture.retryAccounts())).toEqual(['local', 'next-account']);

  await page.evaluate(() => window.nativeMessageFixture.setAccount('local'));
  await expect(page.getByRole('button', { name: 'Alice', exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await expect(failedRow.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await page.evaluate(() => {
    window.nativeMessageFixture.failRetry(0, 'private stale account-A failure');
    window.nativeMessageFixture.failRetry(1, 'private stale account-B failure');
  });
  await expect(failedRow.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await expect(page.getByText('private stale account-A failure', { exact: true })).toHaveCount(0);
  await expect(page.getByText('private stale account-B failure', { exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((item) => item.method === 'retryHumanMessage').length)).toBe(2);
  await milestone(page, info, 'human-retry-account-aba-same-nonce');
});

test('NMR-26 a disconnected Human send queues and reconnect dispatches it exactly once', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  await page.evaluate(() => window.nativeMessageFixture.disconnect());
  await page.locator('[contenteditable="true"]').fill('Queued during disconnection');
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  const pending = page.locator('[data-entry-id^="pending-"]');
  await expect(pending).toHaveCount(1);
  await expect(pending).toContainText('Queued during disconnection');
  await expect(pending).toContainText('Will send when reconnected');
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'sendHumanMessage').length)).toBe(0);
  await milestone(page, info, 'queued-with-transport-down');

  await page.evaluate(() => window.nativeMessageFixture.reconnect());
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(1);
  await page.evaluate(() => {
    window.nativeMessageFixture.reconnect();
    window.nativeMessageFixture.publishSend();
    window.nativeMessageFixture.settleSend();
  });
  await expect(messageRow(page, 'human-server-message:100')).toContainText('Queued during disconnection');
  await expect(pending).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText('');
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'sendHumanMessage').length)).toBe(1);
  await expect.poll(() => domIds(page)).toEqual(['m1', 'm2', 'm3', 'm4', 'm5', 'human-server-message:100']);
  await milestone(page, info, 'queued-send-settled-after-reconnect');
});

test('NMR-FWD-RENDERER-001 multi-source forwarding intersects recipients and retries only the failed source', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  await selectForward(page, 'm1');
  const dialog = page.getByRole('dialog', { name: 'Forward message' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('option', { name: 'Bob' })).toBeVisible();
  await expect(dialog.getByRole('option', { name: 'Carol' })).toBeVisible();

  await dialog.getByRole('button', { name: 'Add another message', exact: true }).click();
  await expect(dialog).not.toBeVisible();
  await expect(page.getByRole('status', { name: 'Forward draft' })).toContainText('1 message in forwarding draft from Alice');
  await selectForward(page, 'm2');
  await expect(dialog).toBeVisible();
  await expect(dialog.getByText('Old first message', { exact: true })).toBeVisible();
  await expect(dialog.getByText('Old second message', { exact: true })).toBeVisible();
  await expect(dialog.getByRole('option', { name: 'Bob' })).toBeVisible();
  await expect(dialog.getByRole('option', { name: 'Carol' })).toHaveCount(0);
  await expect(dialog.getByRole('option', { name: 'Dave' })).toHaveCount(0);
  const searchedSources = await page.evaluate(() => window.nativeMessageFixture.requests
    .filter((request) => request.method === 'searchHumanRecipients')
    .map((request) => request.args.sourceEntryId));
  expect(searchedSources).toContain('m1');
  expect(searchedSources).toContain('m2');

  await dialog.getByRole('option', { name: 'Bob' }).click();
  await expect(dialog.getByText('1 selected', { exact: true })).toBeVisible();
  await dialog.getByRole('button', { name: 'Forward', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().forwards)).toBe(1);
  const firstRequests = await page.evaluate(() => window.nativeMessageFixture.requests
    .filter((request) => request.method === 'forwardHumanMessage'));
  expect(firstRequests).toHaveLength(1);
  expect(firstRequests[0].args.sourceConversationId).toBe('human:a');
  expect(firstRequests[0].args.sourceEntryId).toBe('m1');
  expect(firstRequests[0].args.destinationConversationIds).toEqual(['human:b']);
  expect(firstRequests[0].args.dropSenderNames).toBe(false);
  expect(firstRequests[0].args.dropCaptions).toBe(false);

  await page.evaluate(() => window.nativeMessageFixture.settleForward(0, [
    { conversationId: 'human:b', status: 'sent' },
  ]));
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().forwards)).toBe(2);
  const secondRequests = await page.evaluate(() => window.nativeMessageFixture.requests
    .filter((request) => request.method === 'forwardHumanMessage'));
  expect(secondRequests[1].args.sourceEntryId).toBe('m2');
  expect(secondRequests[1].args.destinationConversationIds).toEqual(['human:b']);
  expect(secondRequests[1].args.clientNonce).not.toBe(firstRequests[0].args.clientNonce);

  await page.evaluate(() => window.nativeMessageFixture.settleForward(1, [
    { conversationId: 'human:b', status: 'failed', error: 'fixture: second source failed' },
  ]));
  await expect(dialog.getByRole('button', { name: 'Retry', exact: true })).toBeVisible();
  await expect(dialog.getByRole('alert')).toContainText('fixture: second source failed');
  await dialog.getByRole('button', { name: 'Retry', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().forwards)).toBe(3);
  const retryRequests = await page.evaluate(() => window.nativeMessageFixture.requests
    .filter((request) => request.method === 'forwardHumanMessage'));
  expect(retryRequests).toHaveLength(3);
  expect(retryRequests[2].args.sourceEntryId).toBe('m2');
  expect(retryRequests[2].args.clientNonce).toBe(secondRequests[1].args.clientNonce);
  expect(retryRequests[2].args.destinationConversationIds).toEqual(['human:b']);

  await page.evaluate(() => window.nativeMessageFixture.settleForward(2, [
    { conversationId: 'human:b', status: 'sent' },
  ]));
  await expect(dialog).not.toBeVisible();
  await expect(page.getByRole('status', { name: 'Forward draft' })).toHaveCount(0);
  await milestone(page, info, 'multi-source-forward-partial-retry');
});

test('NMR-FWD-DRAFT-RECOVERY-001 removing a source narrows recipients and keeps its draft reviewable after navigation', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  await selectForward(page, 'm1');
  const dialog = page.getByRole('dialog', { name: 'Forward message' });
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: 'Add another message', exact: true }).click();
  await expect(dialog).not.toBeVisible();
  await selectForward(page, 'm2');
  await expect(dialog).toBeVisible();
  await expect(dialog.getByText('Old first message', { exact: true })).toBeVisible();
  await expect(dialog.getByText('Old second message', { exact: true })).toBeVisible();
  await expect(dialog.getByRole('option', { name: 'Bob' })).toBeVisible();
  await expect(dialog.getByRole('option', { name: 'Carol' })).toHaveCount(0);
  await expect(dialog.getByRole('option', { name: 'Dave' })).toHaveCount(0);

  await dialog.getByRole('button', { name: 'Remove message m1', exact: true }).click();
  await expect(dialog.getByText('Old first message', { exact: true })).toHaveCount(0);
  await expect(dialog.getByText('Old second message', { exact: true })).toBeVisible();
  await expect(dialog.getByRole('option', { name: 'Dave' })).toBeVisible();
  await expect(dialog.getByRole('option', { name: 'Carol' })).toHaveCount(0);
  const searches = await page.evaluate(() => window.nativeMessageFixture.requests
    .filter((request) => request.method === 'searchHumanRecipients'));
  expect(searches[searches.length - 1]?.args.sourceEntryId).toBe('m2');

  await dialog.getByRole('button', { name: 'Add another message', exact: true }).click();
  await expect(dialog).not.toBeVisible();
  const draft = page.getByRole('status', { name: 'Forward draft' });
  await expect(draft).toContainText('1 message in forwarding draft from Alice');
  await page.getByRole('button', { name: 'Bob', exact: true }).click();
  await expect(messageRow(page, 'b1')).toBeVisible();
  await expect(draft).toBeVisible();
  await draft.getByRole('button', { name: 'Review forward draft', exact: true }).click();
  await expect(dialog).toBeVisible();
  await expect(dialog.getByText('Old second message', { exact: true })).toBeVisible();
  await expect(dialog.getByText('Old first message', { exact: true })).toHaveCount(0);
  await dialog.getByRole('button', { name: 'Add another message', exact: true }).click();
  await expect(dialog).not.toBeVisible();
  await draft.getByRole('button', { name: 'Discard draft', exact: true }).click();
  await expect(draft).toHaveCount(0);
  expect(await page.evaluate(() => window.nativeMessageFixture.requests
    .filter((request) => request.method === 'forwardHumanMessage').length)).toBe(0);
  await milestone(page, info, 'forward-draft-removal-navigation-recovery');
});


test("NMR-UI-01d persisted reply draft restores its target and Host reply identity after conversation navigation", async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);

  const source = messageRow(page, "m1");
  await source.locator(".sand-message-action-anchor").hover();
  await source.getByRole("button", { name: "Reply to your message", exact: true }).click();

  const replyPill = page.locator(".sand-prompt-reply-pill");
  await expect(replyPill).toContainText("Old first message");
  const editor = page.locator(".sand-prompt-form [contenteditable='true']");
  await editor.fill("Reply draft survives conversation navigation");
  await expect(editor).toHaveText("Reply draft survives conversation navigation");

  await page.getByRole("button", { name: "Bob", exact: true }).click();
  await expect(messageRow(page, "b1")).toBeVisible();
  await page.getByRole("button", { name: "Alice", exact: true }).click();
  await expect(messageRow(page, "m3")).toBeVisible();

  await expect(replyPill).toContainText("Old first message");
  await expect(editor).toHaveText("Reply draft survives conversation navigation");
  await page.locator(".sand-prompt-form").evaluate((form) => (form as HTMLFormElement).requestSubmit());

  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(1);
  const request = await page.evaluate(() =>
    window.nativeMessageFixture.requests.filter((item) => item.method === "sendHumanMessage").at(-1) ?? null
  );
  expect(request?.args.replyToId).toBe("m1");
  expect(request?.args.text).toBe("Reply draft survives conversation navigation");

  await page.evaluate(() => window.nativeMessageFixture.settleSend());
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(0);
  await expect(replyPill).toHaveCount(0);
  await milestone(page, info, "reply-draft-restored-after-conversation-navigation");
});
