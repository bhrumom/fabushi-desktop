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

async function installReactionKeyboardDiagnostics(page: Page) {
  await page.evaluate(() => {
    const trace: Record<string, unknown>[] = [];
    const describe = (element: EventTarget | null) => {
      if (!(element instanceof Element)) return null;
      const rect = element.getBoundingClientRect();
      return { tag: element.tagName, id: element.id, label: element.getAttribute('aria-label'),
        role: element.getAttribute('role'), className: element.getAttribute('class'),
        entry: element.closest('[data-entry-id]')?.getAttribute('data-entry-id'),
        connected: element.isConnected, hidden: getComputedStyle(element).visibility,
        display: getComputedStyle(element).display,
        bounds: { x: rect.x, y: rect.y, width: rect.width, height: rect.height } };
    };
    const record = (phase: string, event?: Event, extra: Record<string, unknown> = {}) => {
      if (trace.length === 400) trace.shift();
      trace.push({ phase, time: performance.now(), key: event instanceof KeyboardEvent ? event.key : undefined,
        shift: event instanceof KeyboardEvent ? event.shiftKey : undefined,
        defaultPrevented: event?.defaultPrevented, target: describe(event?.target ?? null),
        active: describe(document.activeElement), ...extra });
    };
    const capture = (event: KeyboardEvent) => {
      if (!['Tab', 'Enter', 'Escape'].includes(event.key)) return;
      record('keydown-capture', event);
      queueMicrotask(() => record('keydown-microtask', event));
      requestAnimationFrame(() => record('keydown-frame', event));
    };
    const bubble = (event: KeyboardEvent) => {
      if (['Tab', 'Enter', 'Escape'].includes(event.key)) record('keydown-bubble', event);
    };
    const focusEvent = (event: FocusEvent) => record(event.type, event, { related: describe(event.relatedTarget) });
    const originalFocus = HTMLElement.prototype.focus;
    function observedFocus(this: HTMLElement, options?: FocusOptions) {
      const relevant = this.matches('.sand-emoji-cell, .sand-message-hover-actions__button');
      if (relevant) record('focus-call-before', undefined, { candidate: describe(this), stack: new Error().stack });
      originalFocus.call(this, options);
      if (relevant) record('focus-call-after', undefined, { candidate: describe(this) });
    }
    HTMLElement.prototype.focus = observedFocus;
    window.addEventListener('keydown', capture, true);
    document.addEventListener('keydown', bubble);
    document.addEventListener('focusin', focusEvent, true);
    document.addEventListener('focusout', focusEvent, true);
    (window as unknown as { reactionKeyboardDiagnostics: { trace: Record<string, unknown>[]; dispose(): void } }).reactionKeyboardDiagnostics = {
      trace,
      dispose() {
        window.removeEventListener('keydown', capture, true);
        document.removeEventListener('keydown', bubble);
        document.removeEventListener('focusin', focusEvent, true);
        document.removeEventListener('focusout', focusEvent, true);
        if (HTMLElement.prototype.focus === observedFocus) HTMLElement.prototype.focus = originalFocus;
      },
    };
  });
}

async function openReactionPicker(page: Page, id: string) {
  const row = messageRow(page, id);
  // Rows span the transcript width; only the actual message anchor reveals
  // its hover controls. The row center can be empty space beside the bubble.
  await row.locator('.sand-message').hover();
  const toolbar = row.getByRole('toolbar');
  await expect(toolbar).toHaveCSS('opacity', '1');
  await expect(toolbar).toHaveCSS('pointer-events', 'auto');
  const trigger = row.getByRole('button', { name: 'Add reaction', exact: true });
  await trigger.click();
  await expect(trigger).toHaveAttribute('aria-expanded', 'true');
  const popover = page.getByRole('dialog', { name: 'Choose reaction' });
  await expect(popover).toBeVisible();
  await expect(popover).toBeInViewport({ ratio: 0.95 });
  await expect(popover.getByRole('button', { name: 'React with 👍', exact: true })).toBeInViewport();
}

async function openAlice(page: Page, theme = 'dark') {
  await page.goto(`${origin}/native-message-projection.html?theme=${theme}`);
  await expect(page.getByRole('button', { name: 'Alice', exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await expect(messageRow(page, 'm3')).toBeVisible();
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
  const keyboardTrace = await page.evaluate(() => {
    const diagnostics = (window as unknown as { reactionKeyboardDiagnostics?: { trace: Record<string, unknown>[]; dispose(): void } }).reactionKeyboardDiagnostics;
    diagnostics?.dispose();
    return diagnostics?.trace ?? [];
  }).catch(() => []);
  if (keyboardTrace.length > 0) await info.attach('reaction-keyboard-focus-diagnostic', {
    body: JSON.stringify(keyboardTrace, null, 2), contentType: 'application/json',
  });
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
  await installReactionKeyboardDiagnostics(page);
  await openReactionPicker(page, 'm1');
  await page.getByRole('dialog', { name: 'Choose reaction' }).getByRole('button', { name: 'React with 👍', exact: true }).click();
  await expect(page.getByRole('dialog', { name: 'Choose reaction' })).toHaveCount(0);
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().reactions)).toBe(1);
  await page.evaluate(() => window.nativeMessageFixture.settleReaction());
  await expect(messageRow(page, 'm1').locator('.sand-reaction-pill')).toContainText('👍');

  await openReactionPicker(page, 'm2');
  await page.getByRole('dialog', { name: 'Choose reaction' }).getByRole('button', { name: 'React with ❤️', exact: true }).click();
  await expect(page.getByRole('dialog', { name: 'Choose reaction' })).toHaveCount(0);
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().reactions)).toBe(2);
  await page.evaluate(() => {
    window.nativeMessageFixture.mutate('human:a', 'm2', { content: '', attachments: [], reactions: [] });
    window.nativeMessageFixture.settleReaction(1);
  });
  await expect(messageRow(page, 'm2').locator('.sand-reaction-pill')).toHaveCount(0);
  await expect(messageRow(page, 'm2').locator('.sand-message')).not.toContainText('Old second message');
  await expect(messageRow(page, 'm2')).not.toContainText('note.txt');
  await expect.poll(() => domIds(page)).toEqual(['m1', 'm2', 'm3', 'm4', 'm5']);
  await openReactionPicker(page, 'm5');
  const popover = page.getByRole('dialog', { name: 'Choose reaction' });
  const bounds = await popover.evaluate((element) => {
    const box = element.getBoundingClientRect();
    const viewport = document.querySelector('.sand-virtual-transcript')?.getBoundingClientRect();
    if (viewport == null) throw new Error('The real transcript viewport is missing');
    return { left: box.left, right: box.right, top: box.top, bottom: box.bottom,
      minLeft: viewport.left + 8, maxRight: viewport.right - 8,
      minTop: viewport.top + 8, maxBottom: viewport.bottom - 8 };
  });
  expect(bounds.left).toBeGreaterThanOrEqual(bounds.minLeft - 1);
  expect(bounds.right).toBeLessThanOrEqual(bounds.maxRight + 1);
  expect(bounds.top).toBeGreaterThanOrEqual(bounds.minTop - 1);
  expect(bounds.bottom).toBeLessThanOrEqual(bounds.maxBottom + 1);
  const firstEmoji = popover.getByRole('button', { name: 'React with 👍', exact: true });
  const trigger = messageRow(page, 'm5').getByRole('button', { name: 'Add reaction', exact: true });
  const nextAction = messageRow(page, 'm5').getByRole('button', { name: 'Reply to your message' });
  await expect(trigger).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(firstEmoji).toBeFocused();
  // Non-modal portal traversal: backward Tab returns to the real trigger,
  // forward Tab re-enters the first emoji, never cycles around the picker.
  await page.keyboard.press('Shift+Tab');
  await expect(trigger).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(firstEmoji).toBeFocused();
  for (let index = 0; index < 6; index += 1) await page.keyboard.press('Tab');
  await expect(popover.getByRole('button', { name: 'More emoji' })).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(nextAction).toBeFocused();
  await expect(popover).toHaveCount(0);
  // Reopen with the actual keyboard command and verify Escape restoration.
  await trigger.focus();
  await page.keyboard.press('Enter');
  await expect(popover).toBeVisible();
  await expect(trigger).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(firstEmoji).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(popover).toHaveCount(0);
  await expect(messageRow(page, 'm5').getByRole('button', { name: 'Add reaction', exact: true })).toBeFocused();
  await milestone(page, info, 'old-row-reaction-receipt');
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

test('HRR-05 persisted Human Resend uses one original Host nonce across refusal and recovery without changing the composer', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  const nonce = 'durable-human-retry-1';
  await page.evaluate((original) => window.nativeMessageFixture.stageFailedHuman(original), nonce);
  const pendingId = `human-message:${nonce}`;
  const failed = messageRow(page, pendingId);
  await expect(failed).toContainText('Original pending Human text');
  await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await page.locator('[contenteditable="true"]').fill('This is a different, still unsent draft');

  await failed.getByRole('button', { name: 'Resend', exact: true }).dblclick();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(1);
  const first = await page.evaluate(() => window.nativeMessageFixture.requests.filter((r) => r.method === 'retryHumanMessage'));
  expect(first).toHaveLength(1);
  expect(first[0].args).toEqual({ conversationId: 'human:a', clientNonce: nonce });
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((r) => r.method === 'sendHumanMessage'))).toEqual([]);
  await expect(page.locator('[contenteditable="true"]')).toHaveText('This is a different, still unsent draft');
  await page.evaluate(() => window.nativeMessageFixture.rejectHumanRetry(0));
  await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(2);
  const attempts = await page.evaluate(() => window.nativeMessageFixture.requests.filter((r) => r.method === 'retryHumanMessage'));
  expect(attempts).toHaveLength(2);
  expect(attempts[0].args).toEqual(attempts[1].args);
  await page.evaluate(() => window.nativeMessageFixture.settleHumanRetry(1));
  const canonical = messageRow(page, 'human-server-message:901');
  await expect(canonical).toContainText('Original pending Human text');
  await expect(canonical).toContainText('original.txt');
  await expect(failed).toHaveCount(0);
  await expect(canonical.getByRole('button', { name: 'Resend', exact: true })).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText('This is a different, still unsent draft');
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((r) => r.method === 'sendHumanMessage'))).toEqual([]);
  await milestone(page, info, 'human-durable-retry-settled');
});

async function flushRenderer(page: Page) {
  // Replies first settle the actual client promise; two layout frames then
  // expose any deferred React update before asserting absence of stale output.
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))));
}

async function beginHumanRetry(page: Page, nonce: string) {
  await openAlice(page);
  await loadOlder(page);
  await page.evaluate((original) => window.nativeMessageFixture.stageFailedHuman(original), nonce);
  const failed = messageRow(page, `human-message:${nonce}`);
  await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(1);
  return failed;
}

test('HRR-05 malformed correlated Coordinator failure releases Human Resend without replacing its durable intent or newer draft', async ({ page }, info) => {
  const nonce = 'retry-malformed-coordinator-envelope';
  const failed = await beginHumanRetry(page, nonce);
  const draft = 'Newer draft while the original Host retry is pending';
  await page.locator('[contenteditable="true"]').fill(draft);

  // This enters the real Coordinator client's transferred-port listener. The
  // correlated failed envelope deliberately omits its required failure object.
  await page.evaluate(() => window.nativeMessageFixture.replyHumanRetryOutcome(0, { status: 'failed' }));
  await expect(page.getByText('Human retry unavailable: protocol-error: retryHumanMessage returned a malformed Coordinator reply outcome', { exact: true })).toBeVisible();
  await expect(failed).toHaveAttribute('data-failed', 'true');
  await expect(failed).toContainText('Original pending Human text');
  await expect(failed).toContainText('original.txt');
  await expect(failed.locator('[data-reply-target-id="human-server-message:7"]')).toBeVisible();
  await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await expect(page.locator('[contenteditable="true"]')).toHaveText(draft);
  await expect(messageRow(page, 'human-server-message:901')).toHaveCount(0);
  await milestone(page, info, 'malformed-coordinator-retry-visible-failure');

  await failed.getByRole('button', { name: 'Resend', exact: true }).dblclick();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(2);
  const attempts = await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'retryHumanMessage'));
  expect(attempts).toHaveLength(2);
  expect(attempts.map((request) => request.args)).toEqual([
    { conversationId: 'human:a', clientNonce: nonce },
    { conversationId: 'human:a', clientNonce: nonce },
  ]);
  expect(attempts[1].requestId).not.toBe(attempts[0].requestId);

  // A late success for the consumed malformed reply must not consume the
  // replacement attempt, settle its row or replace the newer composer draft.
  await page.evaluate(() => window.nativeMessageFixture.replyHumanRetry(0, { content: 'Late original reply must stay ignored' }));
  await flushRenderer(page);
  await expect(messageRow(page, 'human-server-message:901')).toHaveCount(0);
  await expect(failed).toContainText('Original pending Human text');
  await expect(page.locator('[contenteditable="true"]')).toHaveText(draft);

  await page.evaluate(() => window.nativeMessageFixture.settleHumanRetry(1));
  const canonical = messageRow(page, 'human-server-message:901');
  await expect(canonical).toHaveCount(1);
  await expect(canonical).toContainText('Original pending Human text');
  await expect(canonical).toContainText('original.txt');
  await expect(canonical.locator('[data-reply-target-id="human-server-message:7"]')).toBeVisible();
  await expect(failed).toHaveCount(0);
  await expect(canonical.getByRole('button', { name: 'Resend', exact: true })).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText(draft);

  await page.evaluate(() => {
    window.nativeMessageFixture.replyHumanRetry(0, { content: 'Duplicate original reply must not overwrite settlement' });
    window.nativeMessageFixture.replyHumanRetryOutcome(0, { status: 'failed' });
  });
  await flushRenderer(page);
  await expect.poll(() => domIds(page)).toEqual(['m1', 'm2', 'm3', 'm4', 'm5', 'human-server-message:901']);
  await expect(canonical).toContainText('Original pending Human text');
  await expect(page.getByText(/Late original reply|Duplicate original reply/)).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText(draft);
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'sendHumanMessage'))).toEqual([]);
  expect(await page.evaluate(() => window.nativeMessageFixture.attachmentCommits)).toEqual([]);
  const persisted = await page.evaluate((originalNonce) => {
    const stored = JSON.parse(sessionStorage.getItem('native-message-fixture-canonical-rows') ?? '[]') as Array<[string, Array<Record<string, unknown>>]>;
    return stored.find(([id]) => id === 'human:a')?.[1].filter((entry) => entry.clientNonce === originalNonce);
  }, nonce);
  expect(persisted).toEqual([expect.objectContaining({
    id: 'human-server-message:901', remoteMessageId: '901', delivery: 'sent',
    clientNonce: nonce, content: 'Original pending Human text', replyToId: 'human-server-message:7',
    attachments: [{ path: '/committed/original.txt', name: 'original.txt', resourceId: 'resource-1' }],
  })]);
  await milestone(page, info, 'malformed-coordinator-retry-same-intent-recovered');
});

for (const outcome of ['success', 'failure'] as const) {
  test(`HRR-06 conversation A-B-A ignores a previous retry ${outcome} and preserves the current draft`, async ({ page }, info) => {
    const nonce = `retry-selection-${outcome}`;
    const failed = await beginHumanRetry(page, nonce);
    await page.locator('[contenteditable="true"]').fill('Current Alice draft');
    await page.getByRole('button', { name: 'Bob', exact: true }).click();
    await expect(messageRow(page, 'b1')).toBeVisible();
    await page.getByRole('button', { name: 'Alice', exact: true }).click();
    await expect(failed).toBeVisible();
    await expect(page.locator('[contenteditable="true"]')).toHaveText('Current Alice draft');
    await page.evaluate((result) => {
      if (result === 'success') window.nativeMessageFixture.settleHumanRetry(0);
      else window.nativeMessageFixture.rejectHumanRetry(0);
    }, outcome);
    await flushRenderer(page);
    await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
    await expect(messageRow(page, 'human-server-message:901')).toHaveCount(0);
    await expect(page.getByText(/^Human retry unavailable:/)).toHaveCount(0);
    await expect(page.locator('[contenteditable="true"]')).toHaveText('Current Alice draft');
    await milestone(page, info, `retry-selection-aba-${outcome}`);
    if (outcome === 'success') {
      // Ignoring the expired UI receipt does not cancel the Host-owned send.
      // A fresh root recovers the durable canonical row through normal history.
      await page.reload();
      await expect(messageRow(page, 'human-server-message:901')).toContainText('Original pending Human text');
      await expect(failed).toHaveCount(0);
    } else {
      await failed.getByRole('button', { name: 'Resend', exact: true }).click();
      await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(2);
      await page.evaluate(() => window.nativeMessageFixture.settleHumanRetry(1));
      await expect(messageRow(page, 'human-server-message:901')).toBeVisible();
    }
  });
}

test('HRR-06 a retry waiting for Coordinator readiness cannot post after leaving and returning to its conversation', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  await page.getByRole('button', { name: 'Bob', exact: true }).click();
  await expect(messageRow(page, 'b1')).toBeVisible();
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  const nonce = 'retry-before-frame';
  await page.evaluate((original) => {
    window.nativeMessageFixture.stageFailedHuman(original);
    window.nativeMessageFixture.pauseReadiness();
  }, nonce);
  const failed = messageRow(page, `human-message:${nonce}`);
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await page.getByRole('button', { name: 'Bob', exact: true }).click();
  await expect(messageRow(page, 'b1')).toBeVisible();
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await expect(failed).toBeVisible();
  await page.locator('[contenteditable="true"]').fill('Draft while the port reconnects');
  await page.evaluate(() => window.nativeMessageFixture.resumeReadiness());
  await flushRenderer(page);
  expect(await page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(0);
  await expect(page.getByText(/^Human retry unavailable:/)).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText('Draft while the port reconnects');
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(1);
  await page.evaluate(() => window.nativeMessageFixture.settleHumanRetry(0));
  await expect(messageRow(page, 'human-server-message:901')).toBeVisible();
  await expect(page.locator('[contenteditable="true"]')).toHaveText('Draft while the port reconnects');
  await milestone(page, info, 'retry-before-frame-selection-fence');
});

for (const outcome of ['success', 'failure'] as const) {
  test(`HRR-06 account A-B-A rejects late retry ${outcome} while its replacement attempt stays single-flight`, async ({ page }, info) => {
    const nonce = `retry-account-${outcome}`;
    const failed = await beginHumanRetry(page, nonce);
    await page.evaluate(() => window.nativeMessageFixture.setAccount('next-account'));
    await expect(messageRow(page, 'new-a')).toBeVisible();
    await page.evaluate(() => window.nativeMessageFixture.setAccount('local'));
    await flushRenderer(page);
    await expect(messageRow(page, 'new-a')).toBeVisible();
    await page.evaluate((original) => {
      window.nativeMessageFixture.stageFailedHuman(original);
      window.nativeMessageFixture.mutate('human:a', `human-message:${original}`, { content: 'Replacement account lifetime row' });
    }, nonce);
    await expect(failed).toContainText('Replacement account lifetime row');
    await page.locator('[contenteditable="true"]').fill('New account lifetime draft');
    await failed.getByRole('button', { name: 'Resend', exact: true }).click();
    await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(2);
    await page.evaluate((result) => {
      if (result === 'success') window.nativeMessageFixture.replyHumanRetry(0);
      else window.nativeMessageFixture.rejectHumanRetry(0);
    }, outcome);
    await flushRenderer(page);
    await expect(failed).toContainText('Replacement account lifetime row');
    await expect(messageRow(page, 'human-server-message:901')).toHaveCount(0);
    await expect(page.getByText(/^Human retry unavailable:/)).toHaveCount(0);
    await failed.getByRole('button', { name: 'Resend', exact: true }).dblclick();
    expect(await page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(2);
    await page.evaluate(() => window.nativeMessageFixture.settleHumanRetry(1));
    await expect(messageRow(page, 'human-server-message:901')).toContainText('Replacement account lifetime row');
    await expect(page.locator('[contenteditable="true"]')).toHaveText('New account lifetime draft');
    await milestone(page, info, `retry-account-aba-${outcome}`);
  });
}

test('HRR-05 a same-nonce retry receipt from another author cannot consume the local failed row or draft', async ({ page }, info) => {
  const nonce = 'retry-author-collision';
  const failed = await beginHumanRetry(page, nonce);
  await page.locator('[contenteditable="true"]').fill('Keep this local draft');
  await page.evaluate(() => window.nativeMessageFixture.replyHumanRetry(0, { authorId: 'peer-a' }));
  await expect(page.getByText('Human retry unavailable: retryHumanMessage did not return a matching canonical settlement', { exact: true })).toBeVisible();
  await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await expect(messageRow(page, 'human-server-message:901')).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText('Keep this local draft');
  expect(await page.evaluate(() => window.nativeMessageFixture.attachmentCommits)).toEqual([]);
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(2);
  await page.evaluate(() => window.nativeMessageFixture.settleHumanRetry(1));
  await expect(messageRow(page, 'human-server-message:901')).toContainText('Original pending Human text');
  await expect(failed).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText('Keep this local draft');
  await milestone(page, info, 'retry-receipt-author-fence');
});

test('HRR-06 a local pre-admission retry retains its nonce and committed attachment after selection expiry', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  await page.evaluate(() => window.nativeMessageFixture.failNextAttachmentCommit());
  await page.locator('input[type="file"]').setInputFiles({ name: 'retry.txt', mimeType: 'text/plain', buffer: Buffer.from('original file') });
  await expect(page.locator('.sand-prompt-form')).toContainText('retry.txt');
  await page.locator('[contenteditable="true"]').fill('Original failed composer send');
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  const failed = page.getByRole('log', { name: 'Conversation transcript' }).locator('[data-entry-id^="pending-"]');
  await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  const pendingId = await failed.getAttribute('data-entry-id');
  expect(pendingId).not.toBeNull();
  const originalNonce = pendingId!.slice('pending-'.length);
  expect(await page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(0);
  await page.locator('[contenteditable="true"]').fill('Newer composer draft remains untouched');
  await page.evaluate(() => window.nativeMessageFixture.holdNextAttachmentCommit());
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.heldAttachmentCommitCount())).toBe(1);
  await page.getByRole('button', { name: 'Bob', exact: true }).click();
  await expect(messageRow(page, 'b1')).toBeVisible();
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await page.evaluate(() => window.nativeMessageFixture.releaseAttachmentCommits());
  await flushRenderer(page);
  expect(await page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(0);
  await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await expect(page.locator('[contenteditable="true"]')).toHaveText('Newer composer draft remains untouched');
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(1);
  const requests = await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'sendHumanMessage'));
  expect(requests).toHaveLength(1);
  expect(requests[0].args.clientNonce).toBe(originalNonce);
  expect(requests[0].args.text).toBe('Original failed composer send');
  expect(requests[0].args.attachments).toEqual([{ path: '/committed/retry.txt', name: 'retry.txt' }]);
  expect(await page.evaluate(() => window.nativeMessageFixture.attachmentCommits)).toHaveLength(2);
  await page.evaluate(() => { window.nativeMessageFixture.publishSend(0); window.nativeMessageFixture.settleSend(0); });
  await expect(messageRow(page, 'human-server-message:100')).toContainText('Original failed composer send');
  await expect(failed).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText('Newer composer draft remains untouched');
  expect(await page.evaluate(() => window.nativeMessageFixture.approvalClears)).toEqual([]);
  await milestone(page, info, 'local-retry-selection-expiry');
});

test('HRR-06 unmount retires a pending retry presentation without another dispatch', async ({ page }) => {
  await beginHumanRetry(page, 'retry-unmount');
  await page.evaluate(() => {
    window.nativeMessageFixture.unmount();
    window.nativeMessageFixture.replyHumanRetry(0);
  });
  await flushRenderer(page);
  await expect(page.getByRole('log', { name: 'Conversation transcript' })).toHaveCount(0);
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'retryHumanMessage'))).toHaveLength(1);
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'sendHumanMessage'))).toEqual([]);
  expect(await page.evaluate(() => window.nativeMessageFixture.approvalClears)).toEqual([]);
});

for (const outcome of ['failure', 'selection-expired-success'] as const) {
  test(`HRR-06 local retry ${outcome} advances its queued successor once without reconnecting`, async ({ page }, info) => {
    await openAlice(page);
    await loadOlder(page);
    await page.evaluate(() => window.nativeMessageFixture.failNextAttachmentCommit());
    await page.locator('input[type="file"]').setInputFiles({ name: 'queued-retry.txt', mimeType: 'text/plain', buffer: Buffer.from('retained original file') });
    await expect(page.locator('.sand-prompt-form')).toContainText('queued-retry.txt');
    await page.locator('[contenteditable="true"]').fill('Original retry before successor');
    await page.getByRole('button', { name: 'Send message', exact: true }).click();
    const initial = page.getByRole('log', { name: 'Conversation transcript' }).locator('[data-entry-id^="pending-"]');
    await expect(initial.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
    const originalId = await initial.getAttribute('data-entry-id');
    expect(originalId).not.toBeNull();
    const original = messageRow(page, originalId!);
    await original.getByRole('button', { name: 'Resend', exact: true }).click();
    // The original retry has committed its file and entered Host. Its reply
    // remains held while a distinct text-only successor enters the same lane.
    await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(1);
    expect(await page.evaluate(() => window.nativeMessageFixture.attachmentCommits)).toHaveLength(2);
    await page.getByRole('button', { name: 'Remove queued-retry.txt', exact: true }).click();
    await expect(page.locator('.sand-prompt-form')).not.toContainText('queued-retry.txt');
    await page.locator('[contenteditable="true"]').fill('Queued successor must advance');
    await page.getByRole('button', { name: 'Send message', exact: true }).click();
    const successor = page.getByRole('log', { name: 'Conversation transcript' }).locator('[data-entry-id^="pending-"]').filter({ hasText: 'Queued successor must advance' });
    // Online queued work waits behind the active retry; offline copy is distinct.
    await expect(successor).toContainText('Waiting to send…');
    expect(await page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(1);
    await page.locator('[contenteditable="true"]').fill('New unsent draft after successor');

    if (outcome === 'selection-expired-success') {
      await page.getByRole('button', { name: 'Bob', exact: true }).click();
      await expect(messageRow(page, 'b1')).toBeVisible();
      await page.getByRole('button', { name: 'Alice', exact: true }).click();
      await expect(original).toBeVisible();
      await page.evaluate(() => window.nativeMessageFixture.settleSend(0));
    } else {
      await page.evaluate(() => window.nativeMessageFixture.rejectSend(0));
    }

    await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(2);
    await expect(original).toContainText('Original retry before successor');
    await expect(original.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
    // The successor now owns the active send lane; the editor becomes
    // read-only but its unsent draft must remain visible in Prompt.
    await expect(page.getByRole('textbox', { name: 'Prompt' })).toHaveText('New unsent draft after successor');
    await expect(page.getByRole('button', { name: 'Send message', exact: true })).toBeDisabled();
    const requests = await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'sendHumanMessage'));
    expect(requests).toHaveLength(2);
    expect(requests.map((request) => request.args.text)).toEqual(['Original retry before successor', 'Queued successor must advance']);
    expect(requests[0].args.clientNonce).toBe(originalId!.slice('pending-'.length));
    expect(requests[1].args.clientNonce).not.toBe(requests[0].args.clientNonce);
    expect(requests[0].args.attachments).toEqual([{ path: '/committed/queued-retry.txt', name: 'queued-retry.txt' }]);
    expect(requests[1].args.attachments).toEqual([]);
    expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'retryHumanMessage'))).toEqual([]);
    await page.evaluate(() => { window.nativeMessageFixture.publishSend(1); window.nativeMessageFixture.settleSend(1); });
    await expect(messageRow(page, 'human-server-message:101')).toContainText('Queued successor must advance');
    await expect(successor).toHaveCount(0);
    await expect(original).toContainText('Original retry before successor');
    await expect(page.getByRole('textbox', { name: 'Prompt' })).toHaveText('New unsent draft after successor');
    await expect(page.locator('[contenteditable="true"]')).toHaveText('New unsent draft after successor');
    await flushRenderer(page);
    expect(await page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(2);
    await milestone(page, info, `local-retry-successor-${outcome}`);
  });
}

for (const missingField of ['id', 'delivery'] as const) {
  test(`HRR-05 local retry rejects a raw receipt missing ${missingField} before display defaults can settle it`, async ({ page }, info) => {
    await openAlice(page);
    await loadOlder(page);
    await page.evaluate(() => window.nativeMessageFixture.failNextAttachmentCommit());
    await page.locator('input[type="file"]').setInputFiles({ name: 'raw-receipt.txt', mimeType: 'text/plain', buffer: Buffer.from('original attachment') });
    await expect(page.locator('.sand-prompt-form')).toContainText('raw-receipt.txt');
    await page.locator('[contenteditable="true"]').fill('Original raw-receipt retry message');
    await page.getByRole('button', { name: 'Send message', exact: true }).click();
    const failed = page.getByRole('log', { name: 'Conversation transcript' }).locator('[data-entry-id^="pending-"]');
    await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
    const originalId = await failed.getAttribute('data-entry-id');
    expect(originalId).not.toBeNull();
    await failed.getByRole('button', { name: 'Resend', exact: true }).click();
    await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(1);
    await page.locator('[contenteditable="true"]').fill('New draft while retry receipt is pending');
    await page.evaluate((field) => window.nativeMessageFixture.replySend(0, {}, [field]), missingField);
    await expect(page.getByText('Human message retry did not settle; original draft and nonce are retained', { exact: true })).toBeVisible();
    await expect(failed).toHaveAttribute('data-entry-id', originalId!);
    await expect(failed).toContainText('Original raw-receipt retry message');
    await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
    await expect(messageRow(page, 'entry-0')).toHaveCount(0);
    await expect(messageRow(page, 'human-server-message:100')).toHaveCount(0);
    await expect(page.locator('[contenteditable="true"]')).toHaveText('New draft while retry receipt is pending');
    expect(await page.evaluate(() => window.nativeMessageFixture.approvalClears)).toEqual([]);
    expect(await page.evaluate(() => window.nativeMessageFixture.attachmentCommits)).toHaveLength(2);
    const requests = await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'sendHumanMessage'));
    expect(requests).toHaveLength(1);
    expect(requests[0].args.clientNonce).toBe(originalId!.slice('pending-'.length));
    await milestone(page, info, `raw-retry-missing-${missingField}-rejected`);

    // Malformed UI acknowledgement does not erase Host-admitted work. The
    // unchanged valid ordinary transcript event still settles the same nonce.
    await page.evaluate(() => window.nativeMessageFixture.publishSend(0));
    await expect(messageRow(page, 'human-server-message:100')).toContainText('Original raw-receipt retry message');
    await expect(failed).toHaveCount(0);
    await expect(messageRow(page, 'entry-0')).toHaveCount(0);
    await expect(page.locator('[contenteditable="true"]')).toHaveText('New draft while retry receipt is pending');
    expect(await page.evaluate(() => window.nativeMessageFixture.approvalClears)).toEqual([]);
    expect(await page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(1);
    expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((request) => request.method === 'retryHumanMessage'))).toEqual([]);
    await milestone(page, info, `raw-retry-missing-${missingField}-event-recovery`);
  });
}

test('HRR-10 a failed transferred request frame preserves one local Human retry and its committed attachment', async ({ page }, info) => {
  await openAlice(page);
  await loadOlder(page);
  await page.locator('input[type="file"]').setInputFiles({
    name: 'once.txt', mimeType: 'text/plain', buffer: Buffer.from('committed once'),
  });
  await expect(page.locator('.sand-prompt-form')).toContainText('once.txt');
  await page.locator('[contenteditable="true"]').fill('Original failed port send');
  await page.evaluate(() => window.nativeMessageFixture.failNextSendPost());
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  const failed = page.getByRole('log', { name: 'Conversation transcript' }).locator('[data-entry-id^="pending-"]');
  await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  const pendingId = await failed.getAttribute('data-entry-id');
  expect(pendingId).not.toBeNull();
  const originalNonce = pendingId!.slice('pending-'.length);
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((r) => r.method === 'sendHumanMessage'))).toEqual([]);
  expect(await page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(0);
  expect(await page.evaluate(() => window.nativeMessageFixture.attachmentCommits)).toHaveLength(1);
  await page.locator('[contenteditable="true"]').fill('Independent unsent draft after port failure');

  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().sends)).toBe(1);
  const calls = await page.evaluate(() => window.nativeMessageFixture.requests.filter((r) => r.method === 'sendHumanMessage'));
  expect(calls).toHaveLength(1);
  expect(calls[0].args.clientNonce).toBe(originalNonce);
  expect(calls[0].args.text).toBe('Original failed port send');
  expect(calls[0].args.attachments).toEqual([{ path: '/committed/once.txt', name: 'once.txt' }]);
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((r) => r.method === 'retryHumanMessage'))).toEqual([]);
  expect(await page.evaluate(() => window.nativeMessageFixture.attachmentCommits)).toHaveLength(1);
  await page.evaluate(() => {
    window.nativeMessageFixture.publishSend(0);
    window.nativeMessageFixture.settleSend(0);
  });
  await expect(messageRow(page, 'human-server-message:100')).toContainText('Original failed port send');
  await expect(messageRow(page, 'human-server-message:100')).toContainText('once.txt');
  await expect(failed).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText('Independent unsent draft after port failure');
  await milestone(page, info, 'human-port-post-failure-same-nonce-recovered');
});

test('HRR-12 a displayed Human retry failure retires after conversation A-B-A without altering drafts or durable messages', async ({ page }, info) => {
  const nonce = 'notice-selection-scope';
  const failed = await beginHumanRetry(page, nonce);
  await page.locator('[contenteditable="true"]').fill('Draft remains independent of retry notice');
  await page.evaluate(() => window.nativeMessageFixture.rejectHumanRetry(0));
  await expect(page.getByText(/^Human retry unavailable:/)).toBeVisible();
  await page.getByRole('button', { name: 'Bob', exact: true }).click();
  await expect(messageRow(page, 'b1')).toBeVisible();
  await expect(page.getByText(/^Human retry unavailable:/)).toHaveCount(0);
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await expect(page.getByText(/^Human retry unavailable:/)).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText('Draft remains independent of retry notice');
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(2);
  await page.evaluate(() => window.nativeMessageFixture.settleHumanRetry(1));
  await expect(messageRow(page, 'human-server-message:901')).toContainText('Original pending Human text');
  await expect(page.getByText(/^Human retry unavailable:/)).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText('Draft remains independent of retry notice');
  expect(await page.evaluate(() => window.nativeMessageFixture.attachmentCommits)).toEqual([]);
  await milestone(page, info, 'human-retry-notice-selection-lifetime');
});

test('HRR-12 a second same-intent retry clears the visible failure before its new Host receipt', async ({ page }, info) => {
  const failed = await beginHumanRetry(page, 'notice-retry-replacement');
  await page.locator('[contenteditable="true"]').fill('New draft before second attempt');
  await page.evaluate(() => window.nativeMessageFixture.rejectHumanRetry(0));
  await expect(page.getByText(/^Human retry unavailable:/)).toBeVisible();
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageFixture.pending().humanRetries)).toBe(2);
  await expect(page.getByText(/^Human retry unavailable:/)).toHaveCount(0);
  await expect(failed.getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await page.evaluate(() => window.nativeMessageFixture.settleHumanRetry(1));
  await expect(messageRow(page, 'human-server-message:901')).toContainText('Original pending Human text');
  await expect(page.getByText(/^Human retry unavailable:/)).toHaveCount(0);
  await expect(page.locator('[contenteditable="true"]')).toHaveText('New draft before second attempt');
  expect(await page.evaluate(() => window.nativeMessageFixture.requests.filter((r) => r.method === 'retryHumanMessage').map((r) => r.args.clientNonce))).toEqual([
    'notice-retry-replacement', 'notice-retry-replacement',
  ]);
  await milestone(page, info, 'human-retry-notice-replaced');
});
