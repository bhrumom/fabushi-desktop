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

async function scrollToOlderHistory(page: Page) {
  // Cursor readiness must start the automatic page request before scrolling.
  // The mounted viewport's scroll listener must keep that request single-flight.
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().older)).toBe(1);
  await page.evaluate(async () => {
    for (let frame = 0; frame < 8; frame += 1) await new Promise(requestAnimationFrame);
  });
  await page.getByRole('log', { name: 'Conversation transcript' }).evaluate((viewport) => {
    viewport.scrollTop = 0;
    viewport.dispatchEvent(new Event('scroll'));
  });
}

async function openAlice(page: Page, theme = 'dark') {
  await page.goto(`${origin}/native-message-projection.html?theme=${theme}`);
  await expect(page.getByRole('button', { name: 'Alice', exact: true })).toBeVisible();
  // The roster opens Alice automatically. Reselecting it must retain this page.
  await expect(messageRow(page, 'm3')).toBeVisible();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().older)).toBe(1);
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await expect(messageRow(page, 'm3')).toBeVisible();
  await scrollToOlderHistory(page);
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().older)).toBe(1);
}

async function loadOlder(page: Page, expectedIds = ['m1', 'm2', 'm3', 'm4', 'm5']) {
  await page.evaluate(() => window.nativeMessageFixture.releaseOlder());
  await expect.poll(() => domIds(page)).toEqual(expectedIds);
  const olderRequests = await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) =>
    request.method === 'getHumanConversationTranscriptTail' && request.args.beforeSeq === 3));
  expect(olderRequests.length).toBeGreaterThan(0);
  expect(olderRequests.every((request) => request.args.beforeCursor === 'fixture-human-walk:3')).toBe(true);
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
    await scrollToOlderHistory(page);
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
  await scrollToOlderHistory(page);
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
    if (oldPhase === 'queued') {
      // Account replacement retains the observed transport loss. It must
      // cancel the old queue before an actual reconnect loads the new roster.
      await expect(page.locator('.sand-agents-state--unreachable')).toBeVisible();
      await expect(page.locator('[data-entry-id^="pending-"]')).toHaveCount(0);
      expect(await page.evaluate(() => window.nativeMessageFixture.requests
        .filter((request) => request.method === 'sendHumanMessage'))).toEqual([]);
      await page.evaluate(() => window.nativeMessageFixture.reconnect());
    }
    await expect(messageRow(page, 'new-a')).toBeVisible();
    if (oldPhase === 'in-flight') await page.evaluate(() => window.nativeMessageFixture.reconnect());
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
  const reactionAnchor = messageRow(page, 'm1').locator('.sand-message-action-anchor');
  await reactionAnchor.hover();
  await expect(messageRow(page, 'm1').getByRole('toolbar')).toHaveCSS('opacity', '1');
  await expect(messageRow(page, 'm1').getByRole('toolbar')).toHaveCSS('pointer-events', 'auto');
  const anchorBounds = await reactionAnchor.boundingBox();
  const reactionButton = messageRow(page, 'm1').getByRole('button', { name: 'Add reaction', exact: true });
  const buttonBounds = await reactionButton.boundingBox();
  expect(anchorBounds).not.toBeNull();
  expect(buttonBounds).not.toBeNull();
  // Cross the space below the bubble as a real pointer does. The toolbar must
  // stay active here; jumping directly to click can miss a broken hover path.
  await page.mouse.move(buttonBounds!.x + buttonBounds!.width / 2, anchorBounds!.y + anchorBounds!.height + 1, { steps: 10 });
  await expect(messageRow(page, 'm1').getByRole('toolbar')).toHaveCSS('pointer-events', 'auto');
  await reactionButton.click();
  await messageRow(page, 'm1').getByRole('button', { name: 'React with 👍', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().reactions)).toBe(1);
  await page.evaluate(() => window.nativeMessageFixture.settleReaction());
  await expect(messageRow(page, 'm1').locator('.sand-reaction-pill')).toContainText('👍');

  await messageRow(page, 'm2').locator('.sand-message-action-anchor').hover();
  await expect(messageRow(page, 'm2').getByRole('toolbar')).toHaveCSS('opacity', '1');
  await expect(messageRow(page, 'm2').getByRole('toolbar')).toHaveCSS('pointer-events', 'auto');
  await messageRow(page, 'm2').getByRole('button', { name: 'Add reaction', exact: true }).click();
  await messageRow(page, 'm2').getByRole('button', { name: 'React with ❤️', exact: true }).click();
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

test('NMR-26 a disconnected Human send queues and reconnect dispatches it exactly once', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  await page.evaluate(() => window.nativeMessageFixture.disconnect());
  await page.locator('[contenteditable="true"]').fill('Queued during disconnection');
  const beforeQueue = await page.evaluate(() => Date.now());
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  const pending = page.locator('[data-entry-id^="pending-"]');
  await expect(pending).toHaveCount(1);
  await expect(pending).toContainText('Queued during disconnection');
  await expect(pending).toContainText('Will send when reconnected');
  await expect(pending.locator('.sand-message-prose')).toHaveText('Queued during disconnection');
  await expect(pending.locator('.sand-sent-while-offline-notice')).toHaveCount(0);
  const afterQueue = await page.evaluate(() => Date.now());
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'sendHumanMessage').length)).toBe(0);
  await milestone(page, info, 'queued-with-transport-down');

  await page.evaluate(() => window.nativeMessageFixture.reconnect());
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(1);
  const request = await page.evaluate(() => window.nativeMessageFixture.requests.find((entry) => entry.method === 'sendHumanMessage'));
  expect(request?.args.queuedOffline).toBe(true);
  expect(typeof request?.args.composedAtMs).toBe('number');
  expect(request!.args.composedAtMs).toBeGreaterThanOrEqual(beforeQueue);
  expect(request!.args.composedAtMs).toBeLessThanOrEqual(afterQueue);
  const nativePendingId = await page.evaluate(() => window.nativeMessageFixture.publishPendingSend());
  const nativePending = messageRow(page, nativePendingId);
  await expect(nativePending).toHaveAttribute('data-pending', 'true');
  await expect(nativePending.locator('.sand-message-prose')).toHaveText('Queued during disconnection');
  await expect(nativePending.locator('.sand-sent-while-offline-notice')).toHaveCount(0);
  await expect(messageRow(page, 'human-server-message:100')).toHaveCount(0);
  await milestone(page, info, 'offline-origin-pending-is-not-yet-sent');
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

  const expectCanonicalOrigin = async () => {
    const record = await page.evaluate(() => window.nativeMessageFixture.canonicalRows()
      .find(([id]) => id === 'human:a')?.[1].find((entry) => entry.id === 'human-server-message:100'));
    expect(record).toMatchObject({
      id: 'human-server-message:100', authorKind: 'human', authorId: 'local',
      clientNonce: request!.args.clientNonce, queuedOffline: true, composedAtMs: request!.args.composedAtMs, delivery: 'sent',
    });
    const canonical = messageRow(page, 'human-server-message:100');
    await expect(canonical).toHaveCount(1);
    await expect(canonical.locator('.sand-message-prose')).toHaveCount(1);
    await expect(canonical.locator('.sand-message-prose')).toHaveText('Queued during disconnection');
    await expect(canonical).not.toHaveAttribute('data-pending', 'true');
    await expect(canonical).not.toHaveAttribute('data-failed', 'true');
    await expect(canonical.locator('time')).toHaveAttribute('datetime', new Date(Number(record!.timestampMs)).toISOString());
    await expect(canonical.locator('.sand-sent-while-offline-notice')).toHaveCount(1);
    await expect(canonical.locator('.sand-sent-while-offline-notice')).toContainText('Sent while offline ·');
    expect(record!.timestampMs).not.toBe(record!.composedAtMs);
  };
  await expectCanonicalOrigin();
  await page.evaluate(() => window.nativeMessageFixture.reconnect());
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().refreshes)).toBeGreaterThan(0);
  await page.evaluate(() => window.nativeMessageFixture.releaseRefreshes());
  await expect.poll(() => domIds(page)).toEqual(['m1', 'm2', 'm3', 'm4', 'm5', 'human-server-message:100']);
  await expectCanonicalOrigin();
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((entry) => entry.method === 'sendHumanMessage').length)).toBe(1);

  // A new renderer has no previous projection cache. The external fixture's
  // canonical service rows survive this reload; this is not a native Host,
  // SQLite or process-restart acceptance claim.
  await page.reload();
  await expect(page.getByRole('button', { name: 'Alice', exact: true })).toBeVisible();
  await expect(messageRow(page, 'human-server-message:100')).toBeVisible();
  await expectCanonicalOrigin();
  await page.evaluate(() => window.nativeMessageFixture.reconnect());
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().refreshes)).toBeGreaterThan(0);
  await page.evaluate(() => window.nativeMessageFixture.releaseRefreshes());
  await expect.poll(() => domIds(page)).toEqual(['m1', 'm2', 'm3', 'm4', 'm5', 'human-server-message:100']);
  await expectCanonicalOrigin();
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((entry) => entry.method === 'sendHumanMessage'))).toEqual([]);
  await milestone(page, info, 'offline-origin-after-canonical-refresh-and-renderer-reopen');
});

async function startNamedForwardToBob(page: Page) {
  const source = messageRow(page, 'm1');
  await source.locator('.sand-message-action-anchor').hover();
  await source.getByRole('button', { name: 'More message actions', exact: true }).click();
  await source.getByRole('menuitem', { name: 'Forward', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Forward message', exact: true });
  await dialog.getByRole('checkbox', { name: 'Bob', exact: true }).check();
  await dialog.getByRole('button', { name: 'Forward', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().forwards)).toBe(1);
  const request = await page.evaluate(() => window.nativeMessageFixture.requests.find((item) => item.method === 'forwardHumanMessage'));
  expect(request?.args).toMatchObject({
    sourceConversationId: 'human:a', sourceEntryId: 'm1',
    destinationConversationIds: ['human:b'], dropSenderNames: false, dropCaptions: false,
  });
  expect(typeof request?.args.clientNonce).toBe('string');
  await expect(page.getByText('Forwarded to Bob.', { exact: true })).toHaveCount(0);
}

test('UI-NMR-NAMED-FORWARD-001 real settlement survives roster failure and clears on account change', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  await startNamedForwardToBob(page);
  await page.evaluate(() => {
    window.nativeMessageFixture.holdSendRosterRefresh();
    window.nativeMessageFixture.settleForward();
  });
  await expect(page.getByRole('dialog', { name: 'Forward message', exact: true })).toHaveCount(0);
  await expect(page.getByText('Forwarded to Bob.', { exact: true })).toBeVisible();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sendRosterRefreshes)).toBe(1);
  await page.evaluate(async () => {
    window.nativeMessageFixture.failSendRosterRefresh();
    for (let frame = 0; frame < 4; frame += 1) await new Promise(requestAnimationFrame);
  });
  await expect(page.getByText('Forwarded to Bob.', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((item) => item.method === 'forwardHumanMessage').length)).toBe(1);
  await milestone(page, info, 'named-forward-after-roster-failure');

  await page.evaluate(() => window.nativeMessageFixture.setAccount('next-account'));
  await expect(page.getByText('next-account@example.test', { exact: true })).toBeVisible();
  await expect(messageRow(page, 'new-a')).toBeVisible();
  await expect(page.getByText('Forwarded to Bob.', { exact: true })).toHaveCount(0);
  await page.evaluate(() => window.nativeMessageFixture.setAccount('local'));
  await expect(page.getByText('local@example.test', { exact: true })).toBeVisible();
  await expect(page.getByText('Forwarded to Bob.', { exact: true })).toHaveCount(0);
  await milestone(page, info, 'named-forward-cleared-after-account-roundtrip');
});

test('UI-NMR-RETIRED-FORWARD-NAMES-001 old-account reply cannot publish names after an account round trip', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  await startNamedForwardToBob(page);
  await page.evaluate(() => window.nativeMessageFixture.setAccount('next-account'));
  await expect(page.getByText('next-account@example.test', { exact: true })).toBeVisible();
  await expect(messageRow(page, 'new-a')).toBeVisible();
  await expect(page.getByRole('dialog', { name: 'Forward message', exact: true })).toHaveCount(0);
  await page.evaluate(() => window.nativeMessageFixture.setAccount('local'));
  await expect(page.getByText('local@example.test', { exact: true })).toBeVisible();
  await page.evaluate(async () => {
    window.nativeMessageFixture.settleForward();
    for (let frame = 0; frame < 4; frame += 1) await new Promise(requestAnimationFrame);
  });
  await expect(page.getByText('Forwarded to Bob.', { exact: true })).toHaveCount(0);
  await expect(page.getByRole('dialog', { name: 'Forward message', exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((item) => item.method === 'forwardHumanMessage').length)).toBe(1);
  await milestone(page, info, 'retired-forward-reply-after-account-roundtrip');
});
