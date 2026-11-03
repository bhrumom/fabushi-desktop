import { expect, test, type Page, type TestInfo } from '@playwright/test';
import { createLogger, createServer, type ViteDevServer } from 'vite';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import type {} from './fixtures/native-message-author-recovery';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const historyIds = ['human-server-message:101', 'human-server-message:102', 'human-server-message:103', 'human-server-message:104', 'human-server-message:105'];
const firstSentId = 'human-server-message:601';
const secondSentId = 'human-server-message:602';
const viteLog: string[] = [];
const viteRequests: Array<Record<string, unknown>> = [];
const browserDiagnostics = new WeakMap<Page, {
  events: Array<Record<string, unknown>>;
  bodies: Array<Promise<void>>;
  errors: string[];
}>();
let server: ViteDevServer | undefined;
let origin = '';

// Twenty-three actual Chromium cases. No skip, retry or mock renderer path.
test.describe.configure({ retries: 0, timeout: 90_000 });
test.use({ viewport: { width: 1280, height: 900 }, reducedMotion: 'reduce' });

test.beforeAll(async () => {
  const logger = createLogger('info', { allowClearScreen: false });
  for (const level of ['info', 'warn', 'warnOnce', 'error'] as const) {
    const original = logger[level].bind(logger);
    logger[level] = (message, options) => {
      viteLog.push('[' + level + '] ' + message);
      if (options?.error != null) viteLog.push(String(options.error.stack ?? options.error));
      original(message, options);
    };
  }
  server = await createServer({
    configFile: path.join(appRoot, 'vite.config.ts'),
    root: appRoot,
    customLogger: logger,
    server: { host: '127.0.0.1', port: 0, strictPort: false },
    plugins: [{
      name: 'native-message-author-recovery-root-fixture',
      configureServer(vite) {
        vite.middlewares.use((request, response, next) => {
          const started = Date.now();
          response.once('finish', () => viteRequests.push({
            method: request.method, url: request.url, status: response.statusCode, durationMs: Date.now() - started,
          }));
          if (request.url?.split('?')[0] !== '/native-message-author-recovery.html') return next();
          // The module is inside the actual desktop Vite root. Do not replace
          // this with an untransformed external /@fs HTML script entry.
          const html = '<!doctype html><html><head><meta charset="UTF-8">' +
            '<link rel="icon" href="data:,"><style>html,body,#root{height:100%;margin:0}</style>' +
            '</head><body><div id="root"></div>' +
            '<script type="module" src="/e2e/fixtures/native-message-author-recovery.tsx"></script></body></html>';
          void vite.transformIndexHtml(request.url, html).then((transformed) => {
            response.setHeader('content-type', 'text/html; charset=utf-8');
            response.end(transformed);
          }, next);
        });
      },
    }],
  });
  await server.listen();
  const address = server.httpServer?.address();
  if (address == null || typeof address === 'string') throw new Error('Fixture did not bind a loopback HTTP port');
  origin = 'http://127.0.0.1:' + address.port;
});

test.afterAll(async () => {
  try { await server?.close(); }
  finally {
    const directory = path.join(appRoot, 'author-recovery-results');
    await mkdir(directory, { recursive: true });
    await writeFile(path.join(directory, 'native-message-author-recovery-vite.log'), viteLog.join('\n') + '\n');
    await writeFile(path.join(directory, 'native-message-author-recovery-vite-requests.json'), JSON.stringify(viteRequests, null, 2) + '\n');
  }
});

test.beforeEach(async ({ page }) => {
  const diagnostic = { events: [] as Array<Record<string, unknown>>, bodies: [] as Array<Promise<void>>, errors: [] as string[] };
  browserDiagnostics.set(page, diagnostic);
  page.on('pageerror', (error) => {
    diagnostic.errors.push(error.stack ?? error.message);
    diagnostic.events.push({ kind: 'pageerror', message: error.message, stack: error.stack });
  });
  page.on('console', (message) => diagnostic.events.push({
    kind: 'console', level: message.type(), message: message.text(), location: message.location(),
  }));
  page.on('request', (request) => diagnostic.events.push({
    kind: 'request', method: request.method(), url: request.url(), resourceType: request.resourceType(),
  }));
  page.on('requestfailed', (request) => diagnostic.events.push({
    kind: 'requestfailed', method: request.method(), url: request.url(), error: request.failure()?.errorText,
  }));
  page.on('response', (response) => {
    diagnostic.events.push({ kind: 'response', url: response.url(), status: response.status() });
    if (response.status() >= 400) {
      diagnostic.bodies.push(response.text().then((body) => {
        diagnostic.events.push({ kind: 'http-error-body', url: response.url(), status: response.status(), body });
      }, (error: unknown) => {
        diagnostic.events.push({ kind: 'http-error-body-unavailable', url: response.url(), error: String(error) });
      }));
    }
  });
});

const transcript = (page: Page) => page.getByRole('log', { name: 'Conversation transcript' });
const messageRow = (page: Page, id: string) => transcript(page).locator('[data-entry-id="' + id + '"]');
const editor = (page: Page) => page.locator('.sand-prompt-form [contenteditable]');
const domIds = (page: Page) => transcript(page).locator('[data-entry-id]').evaluateAll(
  (entries) => entries.map((entry) => entry.getAttribute('data-entry-id')),
);
const canonicalServiceRow = (page: Page, id: string) => page.evaluate((entryId) => {
  const entries = window.nativeMessageAuthorRecoveryFixture.diagnostics().canonicalServiceRows.find(([conversation]) => conversation === 'human:a')?.[1];
  return entries?.find((entry) => entry.id === entryId);
}, id);

async function domSnapshot(page: Page) {
  return page.evaluate(() => ({
    url: location.href,
    theme: document.documentElement.getAttribute('data-theme'),
    rows: [...document.querySelectorAll('[role="log"][aria-label="Conversation transcript"] [data-entry-id]')].map((entry) => ({
      id: entry.getAttribute('data-entry-id'),
      text: entry.querySelector('.sand-message')?.textContent,
      role: entry.getAttribute('data-role'),
      pending: entry.getAttribute('data-pending'),
      failed: entry.getAttribute('data-failed'),
      busy: entry.getAttribute('aria-busy'),
      timestamp: entry.querySelector('time')?.getAttribute('datetime'),
      attachments: [...entry.querySelectorAll('[data-attachment-label]')].map((item) => item.getAttribute('data-attachment-label')),
      reactions: [...entry.querySelectorAll('.sand-reaction-pill')].map((reaction) => ({
        label: reaction.getAttribute('aria-label'), pressed: reaction.getAttribute('aria-pressed'), text: reaction.textContent,
      })),
    })),
    tools: [...document.querySelectorAll('[role="log"] [data-kind="tool-call"]')].map((entry) => ({
      text: entry.textContent, status: entry.getAttribute('data-status'),
      expanded: entry.querySelector('button')?.getAttribute('aria-expanded'),
    })),
    composer: document.querySelector('.sand-prompt-form [contenteditable]')?.textContent ?? null,
    notice: document.querySelector('.sand-prompt-attachment-notice')?.textContent ?? null,
  }));
}

// A browser event-loop/render barrier after transporting a controlled reply.
// The negative oracles below also retain positive DOM and current-request controls.
async function drainUi(page: Page) {
  await page.evaluate(async () => {
    await Promise.resolve();
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
  });
}

async function clientDraft(page: Page, accountId = '1', conversationId = 'human:a') {
  return page.evaluate(({ id, conversationId }) => {
    const key = 'sand.client.slice.account.' + encodeURIComponent(id).replaceAll('.', '%2E') + '.composer-drafts';
    const value = window.nativeMessageAuthorRecoveryFixture.diagnostics().clientPersistence.find(([name]) => name === key)?.[1];
    return value == null ? null : JSON.parse(value).value?.agents?.[conversationId] ?? null;
  }, { id: accountId, conversationId });
}

async function milestone(page: Page, info: TestInfo, name: string) {
  await info.attach(name + '-dom', { body: JSON.stringify(await domSnapshot(page), null, 2), contentType: 'application/json' });
  const screenshot = info.outputPath(name + '.png');
  await page.screenshot({ path: screenshot, fullPage: true });
  await info.attach(name, { path: screenshot, contentType: 'image/png' });
}

async function openFixture(page: Page, info: TestInfo, options: Record<string, string> = {}) {
  const params = new URLSearchParams({ theme: 'dark', ...options });
  try {
    const response = await page.goto(origin + '/native-message-author-recovery.html?' + params, { waitUntil: 'domcontentloaded' });
    expect(response?.ok(), 'Vite must serve the fixture HTML successfully').toBe(true);
    await page.waitForFunction(() => window.nativeMessageAuthorRecoveryFixture != null && document.getElementById('root')!.childElementCount > 0, undefined, { timeout: 30_000 });
    if (options.holdAgentRoster === '1') {
      // Plugins is a shell capability while the real roster RPC is pending.
      await expect(page.getByRole('region', { name: 'Agent list', exact: true })).toBeVisible();
      await expect(page.getByRole('button', { name: 'Plugins', exact: true })).toBeVisible();
    } else {
      await expect(page.getByRole('button', { name: 'Alice', exact: true })).toBeVisible();
    }
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'cursor-' + params.get('theme'));
  } catch (error) {
    await milestone(page, info, 'bootstrap-failure');
    await info.attach('bootstrap-html', { body: await page.content(), contentType: 'text/html' });
    throw new Error('ProductionRenderer fixture did not mount; inspect Vite, page and network attachments.', { cause: error });
  }
}

async function openAlice(page: Page, info: TestInfo, theme = 'dark') {
  await openFixture(page, info, { theme });
  // The production roster selects its first Human conversation. Clicking again
  // while its initial read is held would create an unrelated second generation.
  await expect(messageRow(page, 'human-server-message:103')).toBeVisible();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().older)).toBeGreaterThan(0);
  await expectOpaqueOlderReads(page);
}

async function expectOpaqueOlderReads(page: Page) {
  const reads = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().humanHistoryReads);
  const continuations = reads.filter(({ request }) => request.args.conversationId === 'human:a'
    && (request.args.beforeSeq != null || request.args.beforeCursor != null));
  expect(continuations.length, 'The actual renderer must request an older Human page').toBeGreaterThan(0);
  for (const continuation of continuations) {
    const { request } = continuation;
    expect(request.args.beforeCursor, 'The client must send the opaque cursor, not open a new legacy walk').toMatch(/^htp1:[0-9a-f]{32}:[0-9]+:[1-9][0-9]*$/);
    const source = reads.find((read) => read.response.nextBeforeCursor === request.args.beforeCursor
      && read.request.sequence < request.sequence);
    expect(source, 'Every continuation must reuse a token returned by an earlier real RPC response').toBeDefined();
    expect(request.args.beforeSeq).toBe(source!.response.nextBeforeSeq);
    expect(continuation.snapshotId).toBe(source!.snapshotId);
    expect(request.accountId).toBe(source!.request.accountId);
    expect(request.accountEpoch).toBe(source!.request.accountEpoch);
    expect(request.args.conversationId).toBe(source!.request.args.conversationId);
    expect(continuation.response.entries.map((entry) => entry.id)).toEqual(historyIds.slice(0, 2));
    expect(continuation.selectedPhysicalSeqs).toEqual([1, 2]);
  }
  return continuations.length;
}

async function loadOlder(page: Page) {
  await expectOpaqueOlderReads(page);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.releaseOlder());
  await expect.poll(() => domIds(page)).toEqual(historyIds);
}

async function sendMessage(page: Page, text: string, nonce?: string) {
  const index = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sendRequests);
  await editor(page).fill(text);
  if (nonce != null) await page.evaluate((value) => window.nativeMessageAuthorRecoveryFixture.reserveSendNonce(value), nonce);
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sendRequests)).toBe(index + 1);
  const request = await page.evaluate((position) => window.nativeMessageAuthorRecoveryFixture.requests.filter((entry) => entry.method === 'sendHumanMessage')[position], index);
  expect(request.args.text).toBe(text);
  if (nonce != null) expect(request.args.clientNonce).toBe(nonce);
  await expect(messageRow(page, 'pending-' + String(request.args.clientNonce))).toHaveAttribute('data-pending', 'true');
  return { index, nonce: String(request.args.clientNonce), id: 'pending-' + String(request.args.clientNonce) };
}

async function sendAgentMessage(page: Page, text: string) {
  const index = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentSendRequests);
  await editor(page).fill(text);
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentSendRequests)).toBe(index + 1);
  const request = await page.evaluate((position) => window.nativeMessageAuthorRecoveryFixture.requests.filter((entry) => entry.method === 'sendPrompt')[position], index);
  expect(request.args.agentId).toBe('agent:helper');
  expect(request.args.prompt).toBe(text);
  await expect(messageRow(page, 'pending-' + String(request.args.clientNonce))).toHaveAttribute('data-pending', 'true');
  return { index, nonce: String(request.args.clientNonce), id: 'pending-' + String(request.args.clientNonce) };
}

async function selectReply(page: Page, entryId: string) {
  const row = messageRow(page, entryId);
  const anchor = row.locator('.sand-message-action-anchor');
  await anchor.hover();
  await expect(row.getByRole('toolbar')).toHaveCSS('pointer-events', 'auto');
  const button = row.getByRole('button', { name: /^Reply to / });
  await expect(button).toHaveCount(1);
  const anchorBounds = await anchor.boundingBox();
  const buttonBounds = await button.boundingBox();
  expect(anchorBounds).not.toBeNull();
  expect(buttonBounds).not.toBeNull();
  await page.mouse.move(buttonBounds!.x + buttonBounds!.width / 2, anchorBounds!.y + anchorBounds!.height + 1, { steps: 10 });
  await button.click();
  await expect(page.getByRole('button', { name: 'Cancel reply', exact: true })).toBeVisible();
}

async function reactTo(page: Page, entryId: string, emoji: string) {
  const index = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().reactionRequests);
  const row = messageRow(page, entryId);
  const reactionAnchor = row.locator('.sand-message-action-anchor');
  await reactionAnchor.hover();
  await expect(row.getByRole('toolbar')).toHaveCSS('opacity', '1');
  await expect(row.getByRole('toolbar')).toHaveCSS('pointer-events', 'auto');
  const anchorBounds = await reactionAnchor.boundingBox();
  const reactionButton = row.getByRole('button', { name: 'Add reaction', exact: true });
  const buttonBounds = await reactionButton.boundingBox();
  expect(anchorBounds).not.toBeNull();
  expect(buttonBounds).not.toBeNull();
  // Exercise the real pointer path into the toolbar, including its padding.
  await page.mouse.move(buttonBounds!.x + buttonBounds!.width / 2, anchorBounds!.y + anchorBounds!.height + 1, { steps: 10 });
  await expect(row.getByRole('toolbar')).toHaveCSS('pointer-events', 'auto');
  await reactionButton.click();
  await page.getByRole('button', { name: 'React with ' + emoji, exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().reactionRequests)).toBe(index + 1);
  return index;
}

async function expectSent(page: Page, id: string) {
  await expect(messageRow(page, id)).toHaveCount(1);
  await expect(messageRow(page, id)).not.toHaveAttribute('data-pending', 'true');
  await expect(messageRow(page, id)).not.toHaveAttribute('data-failed', 'true');
  await expect(messageRow(page, id).getByRole('group', { name: 'Failed message actions' })).toHaveCount(0);
}

async function expectEmptyMessage(page: Page, id: string) {
  await expect(messageRow(page, id).locator('.sand-message')).toHaveText('');
  await expect(messageRow(page, id).locator('[data-attachment-label]')).toHaveCount(0);
  await expect(messageRow(page, id).locator('.sand-reaction-pill')).toHaveCount(0);
}

test.afterEach(async ({ page }, info) => {
  const diagnostic = browserDiagnostics.get(page);
  await Promise.allSettled(diagnostic?.bodies ?? []);
  const fixture = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture?.diagnostics() ?? null).catch((error: unknown) => {
    diagnostic?.events.push({ kind: 'fixture-diagnostics-unavailable', error: String(error) });
    return null;
  });
  await info.attach('browser-console-network-errors', { body: JSON.stringify(diagnostic?.events ?? [], null, 2), contentType: 'application/json' });
  await info.attach('coordinator-desktop-and-client-draft-boundaries', { body: JSON.stringify(fixture, null, 2), contentType: 'application/json' });
  await info.attach('vite-log', { body: viteLog.join('\n'), contentType: 'text/plain' });
  await info.attach('vite-http-requests', { body: JSON.stringify(viteRequests, null, 2), contentType: 'application/json' });
  if (!page.isClosed()) {
    await milestone(page, info, info.status === 'passed' ? 'terminal' : 'failure');
    if (info.status !== 'passed') await info.attach('failure-html', { body: await page.content(), contentType: 'text/html' });
  }
  expect(diagnostic?.errors ?? [], 'No uncaught browser error may qualify a renderer case').toEqual([]);
  expect(fixture?.unhandledRequests ?? [], 'Unimplemented fixture RPCs must not silently pass').toEqual([]);
  expect(fixture?.humanHistoryRejections ?? [], 'A malformed, expired or foreign cursor cannot silently restart history').toEqual([]);
});

for (const theme of ['light', 'dark']) {
  test('NMR-UI-01 paged Human edits and empty fields survive tail refresh and conversation switching (' + theme + ')', async ({ page }, info) => {
    await openAlice(page, info, theme);
    await loadOlder(page);
    const olderReads = await expectOpaqueOlderReads(page);
    await milestone(page, info, 'loaded-history-' + theme);
    await page.evaluate(() => {
      window.nativeMessageAuthorRecoveryFixture.mutate('human:a', 'human-server-message:101', { content: 'Edited old first message', reactions: [{ emoji: '👍', by: 'me' }] });
      window.nativeMessageAuthorRecoveryFixture.mutate('human:a', 'human-server-message:102', { content: '', attachments: [], reactions: [] });
      window.nativeMessageAuthorRecoveryFixture.baseline('human:a', ['human-server-message:104', 'human-server-message:105']);
    });
    await expect.poll(() => domIds(page)).toEqual(historyIds);
    await expect(messageRow(page, 'human-server-message:101')).toContainText('Edited old first message');
    await expect(messageRow(page, 'human-server-message:101').locator('.sand-reaction-pill')).toHaveAttribute('aria-pressed', 'true');
    await expectEmptyMessage(page, 'human-server-message:102');
    await milestone(page, info, 'old-row-update-' + theme);
    await page.getByRole('button', { name: 'Bob', exact: true }).click();
    await expect(messageRow(page, 'human-server-message:201')).toBeVisible();
    await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.mutate('human:a', 'human-server-message:101', { content: 'Updated while Alice is inactive' }));
    await page.getByRole('button', { name: 'Alice', exact: true }).click();
    await expect(messageRow(page, 'human-server-message:101')).toContainText('Updated while Alice is inactive');
    await expectEmptyMessage(page, 'human-server-message:102');
    await expect.poll(() => domIds(page)).toEqual(historyIds);
    expect(await expectOpaqueOlderReads(page), 'Returning to cached history must not issue another older-page read').toBe(olderReads);
    await milestone(page, info, 'returned-history-' + theme);
  });
}

test('NMR-UI-01 an initial tail cannot overwrite mutations received before any row was cached', async ({ page }, info) => {
  await openFixture(page, info, { holdInitial: '1' });
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().initials)).toBeGreaterThan(0);
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.mutate('human:a', 'human-server-message:103', { content: 'Mutation before the initial tail', reactions: [{ emoji: '🎉', by: 'me' }] });
    window.nativeMessageAuthorRecoveryFixture.mutate('human:a', 'human-server-message:104', { content: '', attachments: [], reactions: [] });
    window.nativeMessageAuthorRecoveryFixture.releaseInitial();
  });
  await expect(messageRow(page, 'human-server-message:103')).toContainText('Mutation before the initial tail');
  await expect(messageRow(page, 'human-server-message:103').locator('.sand-reaction-pill')).toContainText('🎉');
  await expectEmptyMessage(page, 'human-server-message:104');
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().older)).toBeGreaterThan(0);
  await loadOlder(page);
  await expect(messageRow(page, 'human-server-message:103')).toContainText('Mutation before the initial tail');
  await expectEmptyMessage(page, 'human-server-message:104');
  await milestone(page, info, 'initial-tail-after-mutations');
});

for (const order of ['page-first', 'refresh-first']) {
  test('NMR-UI-01 overlapping older and full reads preserve newer feed changes (' + order + ')', async ({ page }, info) => {
    await openAlice(page, info);
    await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.reconnect());
    await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().refreshes)).toBeGreaterThan(0);
    await page.evaluate(() => {
      window.nativeMessageAuthorRecoveryFixture.mutate('human:a', 'human-server-message:101', { content: 'Newer than both outstanding reads', reactions: [{ emoji: '❤️', by: '2' }] });
      window.nativeMessageAuthorRecoveryFixture.mutate('human:a', 'human-server-message:102', { content: '', attachments: [], reactions: [] });
    });
    if (order === 'page-first') {
      await loadOlder(page);
      await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.releaseRefreshes());
    } else {
      await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.releaseRefreshes());
      await expect(messageRow(page, 'human-server-message:101')).toContainText('Newer than both outstanding reads');
      await loadOlder(page);
    }
    await drainUi(page);
    await expect(messageRow(page, 'human-server-message:101')).toContainText('Newer than both outstanding reads');
    await expect(messageRow(page, 'human-server-message:101').locator('.sand-reaction-pill')).toContainText('❤️');
    await expectEmptyMessage(page, 'human-server-message:102');
    await expect.poll(() => domIds(page)).toEqual(historyIds);
    await milestone(page, info, 'overlapping-reads-settled-' + order);
  });
}

test('NMR-UI-01 peer nonce and late successful receipt preserve the originating conversation canonical row', async ({ page }, info) => {
  await openAlice(page, info);
  await loadOlder(page);
  const sent = await sendMessage(page, 'Original outgoing content');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.peerCollision());
  await expect(messageRow(page, sent.id)).toHaveAttribute('data-pending', 'true');
  await expect(messageRow(page, 'human-server-message:701')).toContainText('Independent peer message');
  expect(await canonicalServiceRow(page, 'human-server-message:701')).toMatchObject({
    role: 'user', authorKind: 'human', authorId: '2', remoteMessageId: '701', clientNonce: sent.nonce, delivery: 'sent',
  });
  await page.evaluate(() => {
    const id = window.nativeMessageAuthorRecoveryFixture.publishSend();
    window.nativeMessageAuthorRecoveryFixture.mutate('human:a', id, { content: '', attachments: [], reactions: [{ emoji: '❤️', by: 'me' }] });
  });
  await expect(messageRow(page, sent.id)).toHaveCount(0);
  await expect(messageRow(page, firstSentId).locator('.sand-message')).not.toContainText('Original outgoing content');
  expect(await canonicalServiceRow(page, firstSentId)).toMatchObject({
    role: 'user', authorKind: 'human', authorId: '1', remoteMessageId: '601', clientNonce: sent.nonce, delivery: 'sent',
  });
  await expect(editor(page)).toHaveText('');
  await page.getByRole('button', { name: 'Bob', exact: true }).click();
  await expect(messageRow(page, 'human-server-message:201')).toBeVisible();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sends)).toBe(1);
  await expect(editor(page)).toBeEditable();
  await editor(page).fill('Bob draft while Alice awaits its receipt');
  await expect(page.getByRole('button', { name: 'Send message', exact: true })).toBeEnabled();
  await milestone(page, info, 'bob-draft-before-origin-receipt');
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.mutate('human:a', 'human-server-message:601', { content: 'Edited after canonical feed acknowledgement' });
    window.nativeMessageAuthorRecoveryFixture.settleSend();
  });
  await drainUi(page);
  await expect.poll(() => domIds(page)).toEqual(['human-server-message:201']);
  await expect(editor(page)).toHaveText('Bob draft while Alice awaits its receipt');
  await expect(editor(page)).toBeEditable();
  await expect(page.getByRole('button', { name: 'Send message', exact: true })).toBeEnabled();
  await milestone(page, info, 'bob-draft-after-origin-receipt');
  await page.getByRole('button', { name: 'Alice', exact: true }).click();
  await expect(messageRow(page, firstSentId)).toContainText('Edited after canonical feed acknowledgement');
  await expect(messageRow(page, firstSentId).locator('.sand-reaction-pill')).toHaveAttribute('aria-pressed', 'true');
  await expectSent(page, firstSentId);
  await expect.poll(() => domIds(page)).toEqual([...historyIds, firstSentId, 'human-server-message:701']);
  await milestone(page, info, 'late-receipt-after-conversation-switch');
});

test('NMR-UI-01 logout and account replacement fence old send and page replies', async ({ page }, info) => {
  await openAlice(page, info);
  await sendMessage(page, 'Previous account private message');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.reconnect());
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().refreshes)).toBeGreaterThan(0);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.setAccount(null));
  await expect(transcript(page)).toHaveCount(0);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.setAccount('9'));
  await expect(messageRow(page, 'human-server-message:901')).toBeVisible();
  const rosterReads = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter((request) => request.method === 'listHumanConversations').length);
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.settleSend();
    window.nativeMessageAuthorRecoveryFixture.releaseOlder();
    window.nativeMessageAuthorRecoveryFixture.releaseRefreshes();
  });
  await drainUi(page);
  await expect.poll(() => domIds(page)).toEqual(['human-server-message:901']);
  await expect(transcript(page)).not.toContainText('Previous account private message');
  await expect(page.locator('[data-entry-id^="pending-"]')).toHaveCount(0);
  await expect(editor(page)).toHaveText('');
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter((request) => request.method === 'listHumanConversations').length)).toBe(rosterReads);
  await milestone(page, info, 'account-fence-after-late-replies');
});

test('NMR-UI-01 old-page reaction settlement cannot overwrite a later empty Human message', async ({ page }, info) => {
  await openAlice(page, info);
  await loadOlder(page);
  await reactTo(page, 'human-server-message:101', '👍');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.settleReaction());
  await expect(messageRow(page, 'human-server-message:101').locator('.sand-reaction-pill')).toContainText('👍');
  await reactTo(page, 'human-server-message:102', '❤️');
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.mutate('human:a', 'human-server-message:102', { content: '', attachments: [], reactions: [] });
    window.nativeMessageAuthorRecoveryFixture.settleReaction(1);
  });
  await drainUi(page);
  await expectEmptyMessage(page, 'human-server-message:102');
  await expect.poll(() => domIds(page)).toEqual(historyIds);
  await milestone(page, info, 'old-row-reaction-receipt');
});

test('NMR-UI-01 retained peer nonce, assistant and unscoped Human rows never consume a local send or its recoverable failure', async ({ page }, info) => {
  await openAlice(page, info);
  await loadOlder(page);
  const nonce = '10000000-0000-4000-8000-000000000048';
  await page.evaluate((value) => window.nativeMessageAuthorRecoveryFixture.legacyPeer(value), nonce);
  await expect(messageRow(page, 'human-server-message:702')).toContainText('Legacy peer content must survive');
  await expectSent(page, 'human-server-message:702');
  const sent = await sendMessage(page, 'Recoverable local draft', nonce);
  await expectSent(page, 'human-server-message:702');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.nonceNegatives());
  await expect(messageRow(page, 'human-server-message:703')).toContainText('Independent assistant content');
  await expect(messageRow(page, 'human-server-message:704')).toContainText('Unscoped Human content');
  expect(await canonicalServiceRow(page, 'human-server-message:702')).toMatchObject({
    role: 'user', authorKind: 'human', authorId: '2', remoteMessageId: '702', clientNonce: nonce,
  });
  expect(await canonicalServiceRow(page, 'human-server-message:703')).toMatchObject({
    role: 'assistant', authorKind: 'human', authorId: '1', remoteMessageId: '703', clientNonce: nonce,
  });
  const unscoped = await canonicalServiceRow(page, 'human-server-message:704');
  expect(unscoped).toMatchObject({ role: 'user', authorKind: 'human', remoteMessageId: '704', clientNonce: nonce });
  expect(unscoped).not.toHaveProperty('authorId');
  await expect(messageRow(page, sent.id)).toHaveAttribute('data-pending', 'true');
  await expect(editor(page)).toHaveText('Recoverable local draft');
  await milestone(page, info, 'foreign-nonce-before-local-settlement');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.rejectSend(0, 'Unsettled send remains recoverable'));
  await expect(messageRow(page, sent.id)).toHaveAttribute('data-failed', 'true');
  await expect(messageRow(page, sent.id).getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await expect(editor(page)).toHaveText('Recoverable local draft');
  await expect(page.locator('.sand-prompt-attachment-notice')).toContainText('Unsettled send remains recoverable');
  await expect.poll(async () => (await clientDraft(page))?.recovery?.prompt).toBe('Recoverable local draft');
  for (const id of ['human-server-message:702', 'human-server-message:703', 'human-server-message:704']) await expectSent(page, id);
  await expect(messageRow(page, 'human-server-message:702')).toContainText('Legacy peer content must survive');
  await expect.poll(() => domIds(page)).toEqual([...historyIds, 'human-server-message:702', sent.id, 'human-server-message:703', 'human-server-message:704']);
  await milestone(page, info, 'only-local-send-failed');
});

test('NMR-UI-01 late semantic refusal remains an error without demoting sent content or restoring an unsent draft', async ({ page }, info) => {
  await openAlice(page, info);
  await loadOlder(page);
  const sent = await sendMessage(page, 'Already accepted dispatch');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.publishSend(0, 'updated'));
  await expect(messageRow(page, sent.id)).toHaveCount(0);
  await expectSent(page, firstSentId);
  await expect(editor(page)).toHaveText('');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.mutate('human:a', 'human-server-message:601', {
    content: 'Newer canonical body', attachments: [], reactions: [{ emoji: '🎉', by: 'me' }],
  }));
  await expect(messageRow(page, firstSentId)).toContainText('Newer canonical body');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.rejectSend(0, 'Conflicting native acknowledgement'));
  await expect(page.locator('.sand-prompt-attachment-notice')).toContainText('Conflicting native acknowledgement');
  await drainUi(page);
  await expectSent(page, firstSentId);
  await expect(messageRow(page, firstSentId)).toContainText('Newer canonical body');
  await expect(messageRow(page, firstSentId).locator('.sand-reaction-pill')).toContainText('🎉');
  await expect(editor(page)).toHaveText('');
  await expect.poll(() => clientDraft(page)).toBeNull();
  await expect.poll(() => domIds(page)).toEqual([...historyIds, firstSentId]);
  await milestone(page, info, 'canonical-sent-after-semantic-refusal');
});

test('NMR-UI-01 late Agent retry approval completion cannot erase a newer failed send recovery', async ({ page }, info) => {
  await openFixture(page, info, { agent: '1' });
  await page.getByRole('button', { name: 'Fixture Agent', exact: true }).click();
  await expect(messageRow(page, 'agent-answer')).toContainText('The Agent answer is intact.');
  const agentDraft = () => clientDraft(page, '1', 'agent:helper');
  await selectReply(page, 'agent-request');
  const original = await sendAgentMessage(page, 'Retry A without touching a later recovery');
  const originalRequest = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.find((request) => request.method === 'sendPrompt'));
  expect(originalRequest?.args.replyToId).toBe('agent-request');
  expect(originalRequest?.args.richText).toEqual(expect.any(String));
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.rejectAgentSend(0, 'Initial A dispatch failed'));
  await expect(messageRow(page, original.id)).toHaveAttribute('data-failed', 'true');
  await expect.poll(async () => (await agentDraft())?.recovery?.prompt).toBe('Retry A without touching a later recovery');

  const previousClears = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().approvalClearRequests);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.holdApprovalClears());
  await messageRow(page, original.id).getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentSendRequests)).toBe(2);
  const retry = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter((request) => request.method === 'sendPrompt')[1]);
  expect(retry.args.agentId).toBe('agent:helper');
  expect(retry.args.prompt).toBe('Retry A without touching a later recovery');
  expect(retry.args.richText).toBe(originalRequest?.args.richText);
  expect(retry.args.replyToId).toBe(originalRequest?.args.replyToId);
  expect(retry.args.clientNonce).not.toBe(original.nonce);
  const retryId = 'pending-' + String(retry.args.clientNonce);
  await expect(messageRow(page, original.id)).toHaveCount(0);
  await expect(messageRow(page, retryId)).toHaveAttribute('data-pending', 'true');
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.publishAgentSend(1);
    window.nativeMessageAuthorRecoveryFixture.settleAgentSend(1);
  });
  await expectSent(page, 'agent-accepted:802');
  await expect(messageRow(page, retryId)).toHaveCount(0);
  await expect(editor(page)).toHaveText('');
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().approvalClears)).toBe(1);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().approvalClearRequests)).toBe(previousClears + 1);

  // A's real queue slot is available while its external approval callback waits.
  // B must pass through the same renderer queue and durable client draft bridge.
  await expect(editor(page)).toBeEditable();
  await selectReply(page, 'agent-answer');
  const newer = await sendAgentMessage(page, 'B recovery belongs to the newer failure');
  expect(newer.index).toBe(2);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.rejectAgentSend(2, 'B remains recoverable'));
  await expect(messageRow(page, newer.id)).toHaveAttribute('data-failed', 'true');
  await expect.poll(async () => (await agentDraft())?.recovery?.prompt).toBe('B recovery belongs to the newer failure');
  const draftBefore = await agentDraft();
  expect(draftBefore?.recovery?.replyToId).toBe('agent-answer');
  expect(draftBefore?.recovery?.richText).toEqual(expect.any(String));
  await expect(editor(page)).toHaveText('B recovery belongs to the newer failure');
  await expect(page.locator('.sand-prompt-attachment-notice')).toContainText('B remains recoverable');
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().approvalClears)).toBe(1);
  await milestone(page, info, 'newer-failure-before-old-approval');

  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.releaseApprovalClears());
  await drainUi(page);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().approvalClears)).toBe(0);
  await expectSent(page, 'agent-accepted:802');
  await expect(messageRow(page, newer.id)).toHaveAttribute('data-failed', 'true');
  await expect(messageRow(page, newer.id).getByRole('button', { name: 'Resend', exact: true })).toBeVisible();
  await expect(editor(page)).toHaveText('B recovery belongs to the newer failure');
  await expect(page.locator('.sand-prompt-attachment-notice')).toContainText('B remains recoverable');
  expect(await agentDraft()).toEqual(draftBefore);
  await expect.poll(() => domIds(page)).toEqual(['agent-request', 'agent-answer', 'agent-accepted:802', newer.id]);
  await expect(transcript(page).locator('[data-kind="tool-call"]')).toHaveCount(1);
  await expect(transcript(page).locator('[data-kind="tool-call"]')).toHaveAttribute('data-status', 'done');
  const humanRequests = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter((request) => request.method === 'sendHumanMessage' || request.method === 'retryHumanMessage'));
  expect(humanRequests, 'The held approval must belong to the actual Agent owner, never a fake Human retry').toEqual([]);
  await milestone(page, info, 'newer-recovery-after-old-approval');
});

for (const order of ['feed-first', 'receipt-first'] as const) {
test('NMR-UI-01 same-nonce Human retry preserves a newer draft with identical text (' + order + ')', async ({ page }, info) => {
  await openAlice(page, info);
  await loadOlder(page);
  const text = 'An identical later draft still has its own identity';
  const original = await sendMessage(page, text);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.rejectSend(0, 'Original same-text dispatch failed'));
  await expect(messageRow(page, original.id)).toHaveAttribute('data-failed', 'true');
  await expect.poll(async () => (await clientDraft(page))?.recovery?.prompt).toBe(text);
  const originalDraft = await clientDraft(page);
  expect(originalDraft?.draftId).toEqual(expect.any(String));

  // Two actual editor changes produce a new persisted identity even though
  // the final content/attachments match A's old failed submission exactly.
  await editor(page).fill('An intervening independent edit');
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe('An intervening independent edit');
  const intermediateId = (await clientDraft(page))?.draftId;
  expect(intermediateId).not.toBe(originalDraft.draftId);
  await editor(page).fill(text);
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe(text);
  const newerDraft = await clientDraft(page);
  expect(newerDraft?.draftId).toEqual(expect.any(String));
  expect(newerDraft.draftId).not.toBe(originalDraft.draftId);
  expect(newerDraft.draftId).not.toBe(intermediateId);
  expect(newerDraft.draft).toEqual(originalDraft.draft);
  const expectedDraft = { draftId: newerDraft.draftId, draft: newerDraft.draft };
  const expectNewerDraft = async () => {
    await expect(editor(page)).toHaveText(text);
    const persisted = await clientDraft(page);
    expect({ draftId: persisted?.draftId, draft: persisted?.draft }).toEqual(expectedDraft);
  };

  const previousClears = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().approvalClearRequests);
  await messageRow(page, original.id).getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().retryRequests)).toBe(1);
  const retry = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.find((request) => request.method === 'retryHumanMessage'));
  expect(retry?.args).toEqual({ conversationId: 'human:a', clientNonce: original.nonce });
  expect(retry?.accountId).toBe('1');
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sendRequests)).toBe(1);
  await expect(messageRow(page, original.id)).toHaveAttribute('data-pending', 'true');
  await expect(editor(page)).toBeEditable();
  await expectNewerDraft();

  // Each order has a positive first settlement. The receipt-first branch has
  // no feed event available to hide a missing authoritative retry callback.
  if (order === 'feed-first') await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.publishRetry());
  else await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.settleRetry());
  await expectSent(page, firstSentId);
  await expect(messageRow(page, original.id)).toHaveCount(0);
  await expectNewerDraft();
  await milestone(page, info, 'same-text-new-identity-after-first-' + order);
  if (order === 'feed-first') await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.settleRetry());
  else await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.publishRetry());
  await drainUi(page);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().approvalClears)).toBe(0);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().approvalClearRequests)).toBe(previousClears);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().retryRequests)).toBe(1);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sendRequests)).toBe(1);
  await expectNewerDraft();
  await expect(editor(page)).toBeEditable();
  await expect(page.getByRole('button', { name: 'Send message', exact: true })).toBeEnabled();
  await expectSent(page, firstSentId);
  await expect(messageRow(page, original.id)).toHaveCount(0);
  await expect.poll(() => domIds(page)).toEqual([...historyIds, firstSentId]);
  const canonical = await canonicalServiceRow(page, firstSentId);
  expect(canonical).toMatchObject({ clientNonce: original.nonce, authorKind: 'human', authorId: '1', remoteMessageId: '601', content: text });
  await milestone(page, info, 'same-text-new-identity-after-all-settlements');
});
}

test('NMR-UI-01 pre-frame Human retry resumes the original input and partial attachment commits', async ({ page }, info) => {
  await openAlice(page, info);
  await loadOlder(page);
  const text = 'Original Human payload survives partial attachment commit';
  await page.locator('.sand-prompt-form input[type="file"]').setInputFiles([
    { name: 'first.txt', mimeType: 'text/plain', buffer: Buffer.from('first-file') },
    { name: 'second.txt', mimeType: 'text/plain', buffer: Buffer.from('second-file') },
  ]);
  await expect(page.getByRole('list', { name: 'Attachments', exact: true })).toContainText('first.txt');
  await expect(page.getByRole('list', { name: 'Attachments', exact: true })).toContainText('second.txt');
  await editor(page).fill(text);
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe(text);
  const originalDraft = await clientDraft(page);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.failAttachmentCommit(1, 'Second attachment commit failed before request admission'));
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  const failed = transcript(page).locator('[data-entry-id^="pending-"][data-failed="true"]');
  await expect(failed).toHaveCount(1);
  const failedId = await failed.getAttribute('data-entry-id');
  expect(failedId).toMatch(/^pending-[0-9a-f-]{36}$/);
  const nonce = failedId!.slice('pending-'.length);
  await expect(page.locator('.sand-prompt-attachment-notice')).toContainText('Second attachment commit failed before request admission');
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sendRequests)).toBe(0);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().retryRequests)).toBe(0);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().attachmentCommits)).toEqual([
    { index: 0, paths: ['memory:staged/first.txt'], names: ['first.txt'], committedPaths: ['memory:committed/first.txt'], failure: null },
    { index: 1, paths: ['memory:staged/second.txt'], names: ['second.txt'], committedPaths: [], failure: 'Second attachment commit failed before request admission' },
  ]);

  await editor(page).fill('A newer draft does not replace the retained retry input');
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe('A newer draft does not replace the retained retry input');
  const newer = await clientDraft(page);
  expect(newer.draftId).not.toBe(originalDraft.draftId);
  const expectedDraft = { draftId: newer.draftId, draft: newer.draft };
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sendRequests)).toBe(1);
  const request = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.find((entry) => entry.method === 'sendHumanMessage'));
  expect(request?.args).toMatchObject({
    conversationId: 'human:a', clientNonce: nonce, text,
    attachments: [{ path: 'memory:committed/first.txt', name: 'first.txt' }, { path: 'memory:committed/second.txt', name: 'second.txt' }],
  });
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().retryRequests)).toBe(0);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().attachmentCommits)).toEqual([
    { index: 0, paths: ['memory:staged/first.txt'], names: ['first.txt'], committedPaths: ['memory:committed/first.txt'], failure: null },
    { index: 1, paths: ['memory:staged/second.txt'], names: ['second.txt'], committedPaths: [], failure: 'Second attachment commit failed before request admission' },
    { index: 2, paths: ['memory:staged/second.txt'], names: ['second.txt'], committedPaths: ['memory:committed/second.txt'], failure: null },
  ]);
  await expect(messageRow(page, failedId!)).toHaveAttribute('data-pending', 'true');
  await expect(editor(page)).toBeEditable();
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.publishSend());
  await expectSent(page, firstSentId);
  await expect(messageRow(page, firstSentId)).toContainText(text);
  await expect(messageRow(page, firstSentId)).toContainText('first.txt');
  await expect(messageRow(page, firstSentId)).toContainText('second.txt');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.settleSend());
  await drainUi(page);
  const persisted = await clientDraft(page);
  expect({ draftId: persisted?.draftId, draft: persisted?.draft }).toEqual(expectedDraft);
  await expect(editor(page)).toHaveText('A newer draft does not replace the retained retry input');
  await expect(messageRow(page, failedId!)).toHaveCount(0);
  await expect.poll(() => domIds(page)).toEqual([...historyIds, firstSentId]);
  await milestone(page, info, 'pre-frame-original-input-and-commit-progress');
});

test('NMR-DRAFT-01 a failed submission binding write admits zero message RPC and retains the original retry input', async ({ page }, info) => {
  await openAlice(page, info);
  await loadOlder(page);
  const text = 'The original draft must be durable before message admission';
  await editor(page).fill(text);
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe(text);
  const original = await clientDraft(page);
  expect(original?.draftId).toEqual(expect.any(String));
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.holdNextSubmissionBindingWrite());
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().submissionBindingWrites)).toBe(1);
  const write = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().draftPersistenceWrites.find((entry) => entry.state === 'held'));
  expect(write?.containsSubmissionBinding).toBe(true);
  expect(write?.key).toBe('sand.client.slice.account.1.composer-drafts');
  const written = JSON.parse(write!.value).value.agents['human:a'];
  expect(written.submissions).toHaveLength(1);
  expect(written.submissions[0]).toMatchObject({ authorId: '1', draftId: original.draftId });
  const nonce = written.submissions[0].nonce as string;
  expect(nonce).toMatch(/^[0-9a-f-]{36}$/);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter((entry) => entry.method === 'sendHumanMessage' || entry.method === 'retryHumanMessage'))).toEqual([]);
  // A failed ordinary draft autosave would be a different scenario. The
  // controlled rejection is this first envelope carrying the real binding.
  const persistedWhileHeld = await clientDraft(page);
  expect(persistedWhileHeld.draftId).toBe(original.draftId);
  expect(persistedWhileHeld.draft).toEqual(original.draft);
  expect(persistedWhileHeld.submissions ?? []).toEqual([]);
  await milestone(page, info, 'binding-write-held-before-any-message-rpc');

  const failure = 'Submission identity persistence denied at the external bridge';
  await page.evaluate((message) => window.nativeMessageAuthorRecoveryFixture.rejectSubmissionBindingWrite(message), failure);
  await expect(messageRow(page, 'pending-' + nonce)).toHaveAttribute('data-failed', 'true');
  await expect(page.locator('.sand-prompt-attachment-notice')).toContainText(failure);
  await expect(editor(page)).toHaveText(text);
  await drainUi(page);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter((entry) => entry.method === 'sendHumanMessage' || entry.method === 'retryHumanMessage'))).toEqual([]);
  const afterFailure = await clientDraft(page);
  expect(afterFailure.draftId).toBe(original.draftId);
  expect(afterFailure.draft).toEqual(original.draft);
  const failures = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().draftPersistenceWrites.filter((entry) => entry.state === 'failed'));
  expect(failures).toHaveLength(1);
  expect(failures[0]).toMatchObject({ index: write!.index, failure, containsSubmissionBinding: true });

  // The existing pre-frame retry now succeeds through the same input and
  // nonce. This positive control rules out an always-disabled send oracle.
  await messageRow(page, 'pending-' + nonce).getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sendRequests)).toBe(1);
  const request = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.find((entry) => entry.method === 'sendHumanMessage'));
  expect(request?.args).toMatchObject({ conversationId: 'human:a', clientNonce: nonce, text });
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().retryRequests)).toBe(0);
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.publishSend();
    window.nativeMessageAuthorRecoveryFixture.settleSend();
  });
  await expectSent(page, firstSentId);
  await expect(editor(page)).toHaveText('');
  await expect.poll(() => clientDraft(page)).toBeNull();
  await milestone(page, info, 'binding-failure-recovered-by-original-input');
});

test('NMR-DRAFT-01 canonical history before delayed draft restore retires only the recorded original identity', async ({ page }, info) => {
  await openAlice(page, info);
  await loadOlder(page);
  const text = 'Canonical history and delayed draft restore share only the original identity';
  const original = await sendMessage(page, text);
  const persisted = await clientDraft(page);
  expect(persisted?.draftId).toEqual(expect.any(String));
  expect(persisted.submissions).toEqual([
    { nonce: original.nonce, authorId: '1', draftId: persisted.draftId, recoveryId: null },
  ]);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sends)).toBe(1);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.unmount());
  await expect(page.locator('#root')).toBeEmpty();
  await drainUi(page);
  const oldEndpoints = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().endpoints);
  expect(oldEndpoints.length).toBeGreaterThan(0);
  expect(oldEndpoints.every((endpoint) => endpoint.closed)).toBe(true);
  // The service commits while no renderer is mounted. No internal draft
  // store or envelope is seeded: all persisted data came from the real UI.
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.publishSend();
    window.nativeMessageAuthorRecoveryFixture.settleSend();
    window.nativeMessageAuthorRecoveryFixture.holdComposerDraftReads();
    window.nativeMessageAuthorRecoveryFixture.mount();
  });
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().draftReads)).toBeGreaterThan(0);
  await expectSent(page, firstSentId);
  await expect(messageRow(page, firstSentId)).toContainText(text);
  const reads = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().heldDraftReads.filter((entry) => !entry.settled));
  expect(reads.length).toBeGreaterThan(0);
  for (const read of reads) {
    expect(read.key).toBe('sand.client.slice.account.1.composer-drafts');
    expect(JSON.parse(read.value!).value.agents['human:a']).toEqual(persisted);
  }
  expect((await clientDraft(page))?.draftId).toBe(persisted.draftId);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sendRequests)).toBe(1);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().retryRequests)).toBe(0);
  await milestone(page, info, 'canonical-visible-while-draft-read-is-held');

  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.releaseComposerDraftReads());
  await expect.poll(() => clientDraft(page)).toBeNull();
  await expect(editor(page)).toHaveText('');
  await expectSent(page, firstSentId);
  await expect(messageRow(page, original.id)).toHaveCount(0);
  await milestone(page, info, 'original-identity-retired-after-delayed-restore');

  // A new UI draft can deliberately have the same text. Re-mounting against
  // the same canonical history must preserve its independently persisted ID.
  await selectReply(page, historyIds[0]);
  await editor(page).fill('An intervening independent post-recovery draft');
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe('An intervening independent post-recovery draft');
  const intermediateId = (await clientDraft(page))?.draftId;
  expect(intermediateId).not.toBe(persisted.draftId);
  await editor(page).fill(text);
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe(text);
  const newer = await clientDraft(page);
  expect(newer.draft.replyToId).toBe(historyIds[0]);
  expect(newer.draft.richText).toEqual(expect.any(String));
  expect(newer.draftId).not.toBe(persisted.draftId);
  expect(newer.draftId).not.toBe(intermediateId);
  expect(newer.submissions ?? []).toEqual([]);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.unmount());
  await expect(page.locator('#root')).toBeEmpty();
  await drainUi(page);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.mount());
  await expectSent(page, firstSentId);
  await expect(editor(page)).toHaveText(text);
  await expect.poll(() => clientDraft(page)).toEqual(newer);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sendRequests)).toBe(1);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().retryRequests)).toBe(0);
  await expect.poll(() => domIds(page)).toEqual([...historyIds, firstSentId]);
  await expect(page.getByRole('button', { name: 'Cancel reply', exact: true })).toBeVisible();
  await milestone(page, info, 'new-same-text-identity-survives-canonical-remount');

  // No new reply selection is made after mount: actual Send must consume the
  // account/conversation-scoped persisted target that the pill displays.
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().sendRequests)).toBe(2);
  const secondRequest = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter((request) => request.method === 'sendHumanMessage')[1]);
  expect(secondRequest.args).toMatchObject({ text, conversationId: 'human:a', replyToId: historyIds[0] });
  expect(secondRequest.args.clientNonce).not.toBe(original.nonce);
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.publishSend(1);
    window.nativeMessageAuthorRecoveryFixture.settleSend(1);
  });
  await expectSent(page, secondSentId);
  await expect(editor(page)).toHaveText('');
  await expect.poll(() => clientDraft(page)).toBeNull();
  await milestone(page, info, 'restored-human-reply-consumed-by-actual-send');
});


test('NMR-DRAFT-01 queued Agent cancellation preserves restored reply and partial attachment progress', async ({ page }, info) => {
  await openFixture(page, info, { agent: '1' });
  await page.getByRole('button', { name: 'Fixture Agent', exact: true }).click();
  await expect(messageRow(page, 'agent-answer')).toContainText('The Agent answer is intact.');
  const agentDraft = () => clientDraft(page, '1', 'agent:helper');
  const reopenAgent = async () => {
    await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.unmount());
    await expect(page.locator('#root')).toBeEmpty();
    await drainUi(page);
    await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.mount());
    await page.getByRole('button', { name: 'Fixture Agent', exact: true }).click();
    await expect(messageRow(page, 'agent-answer')).toBeVisible();
  };
  const text = 'Retain the complete Agent input through queue cancellation and recovery.';
  await selectReply(page, 'agent-request');
  await editor(page).fill(text);
  await expect.poll(async () => (await agentDraft())?.draft?.replyToId).toBe('agent-request');
  const original = await agentDraft();
  expect(original.draft.richText).toEqual(expect.any(String));
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.disconnect());
  await drainUi(page);
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  const queued = transcript(page).locator('[data-entry-id^="pending-"]');
  await expect(queued).toHaveCount(1);
  await expect(queued).toContainText('Will send when reconnected');
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentSendRequests)).toBe(0);
  await editor(page).fill('A different visible draft while the original send is queued');
  await expect.poll(async () => (await agentDraft())?.draft?.prompt).toBe('A different visible draft while the original send is queued');
  await queued.getByRole('button', { name: 'Cancel', exact: true }).click();
  await expect(queued).toHaveCount(0);
  await expect.poll(async () => (await agentDraft())?.recovery).toEqual(original.draft);
  // Clearing the independent visible draft exposes the complete cancelled
  // recovery; it does not seed or call the internal draft store.
  await editor(page).fill('');
  await expect.poll(async () => (await agentDraft())?.draft).toBeNull();
  await expect(editor(page)).toHaveText(text);
  const recoveryOnly = await agentDraft();
  expect(recoveryOnly.recovery).toEqual(original.draft);
  expect(recoveryOnly.recoveryId).toEqual(expect.any(String));
  await reopenAgent();
  await expect(editor(page)).toHaveText(text);
  await expect(page.getByRole('button', { name: 'Cancel reply', exact: true })).toBeVisible();
  await expect.poll(() => agentDraft()).toEqual(recoveryOnly);
  await page.getByRole('button', { name: 'Cancel reply', exact: true }).click();
  await expect.poll(async () => (await agentDraft())?.recovery).toBeNull();
  const cancelledReply = await agentDraft();
  const { replyToId: _discardedReply, ...withoutReply } = original.draft;
  expect(cancelledReply.draft).toEqual(withoutReply);
  expect(cancelledReply.draftId).not.toBe(original.draftId);
  expect(cancelledReply.recoveryId ?? null).toBeNull();
  await reopenAgent();
  await expect(editor(page)).toHaveText(text);
  await expect(page.getByRole('button', { name: 'Cancel reply', exact: true })).toHaveCount(0);
  await expect.poll(() => agentDraft()).toEqual(cancelledReply);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentSendRequests)).toBe(0);
  await milestone(page, info, 'cancelled-reply-does-not-return-after-recovery-reopen');

  // Selecting a replacement target is a positive control after explicit
  // cancellation. Per-file commit failure then exercises the real Agent queue.
  await selectReply(page, 'agent-answer');
  await page.locator('.sand-prompt-form input[type="file"]').setInputFiles([
    { name: 'agent-first.txt', mimeType: 'text/plain', buffer: Buffer.from('first-agent-file') },
    { name: 'agent-second.txt', mimeType: 'text/plain', buffer: Buffer.from('second-agent-file') },
  ]);
  await expect.poll(async () => (await agentDraft())?.draft?.attachments?.length).toBe(2);
  const complete = (await agentDraft()).draft;
  expect(complete.replyToId).toBe('agent-answer');
  expect(complete.richText).toBe(original.draft.richText);
  expect(complete.attachments).toEqual([
    { path: 'memory:staged/agent-first.txt', name: 'agent-first.txt', size: Buffer.byteLength('first-agent-file') },
    { path: 'memory:staged/agent-second.txt', name: 'agent-second.txt', size: Buffer.byteLength('second-agent-file') },
  ]);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.failAttachmentCommit(1, 'Second Agent attachment commit failed'));
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  const failed = transcript(page).locator('[data-entry-id^="pending-"][data-failed="true"]');
  await expect(failed).toHaveCount(1);
  const firstFailedId = await failed.getAttribute('data-entry-id');
  await expect.poll(async () => (await agentDraft())?.recovery).toEqual(complete);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentSendRequests)).toBe(0);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().attachmentCommits)).toEqual([
    { index: 0, paths: ['memory:staged/agent-first.txt'], names: ['agent-first.txt'], committedPaths: ['memory:committed/agent-first.txt'], failure: null },
    { index: 1, paths: ['memory:staged/agent-second.txt'], names: ['agent-second.txt'], committedPaths: [], failure: 'Second Agent attachment commit failed' },
  ]);
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentSendRequests)).toBe(1);
  const firstRequest = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter((request) => request.method === 'sendPrompt')[0]);
  expect(firstRequest.args).toMatchObject({
    agentId: 'agent:helper', prompt: text, richText: complete.richText, replyToId: 'agent-answer',
    attachmentPaths: ['memory:committed/agent-first.txt', 'memory:committed/agent-second.txt'],
    attachmentNames: ['agent-first.txt', 'agent-second.txt'],
  });
  expect('pending-' + String(firstRequest.args.clientNonce)).not.toBe(firstFailedId);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().attachmentCommits)).toEqual([
    { index: 0, paths: ['memory:staged/agent-first.txt'], names: ['agent-first.txt'], committedPaths: ['memory:committed/agent-first.txt'], failure: null },
    { index: 1, paths: ['memory:staged/agent-second.txt'], names: ['agent-second.txt'], committedPaths: [], failure: 'Second Agent attachment commit failed' },
    { index: 2, paths: ['memory:staged/agent-second.txt'], names: ['agent-second.txt'], committedPaths: ['memory:committed/agent-second.txt'], failure: null },
  ]);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.rejectAgentSend(0, 'Agent request failed after attachments committed'));
  await expect(failed).toHaveCount(1);
  await expect.poll(async () => (await agentDraft())?.recovery).toEqual(complete);
  await failed.getByRole('button', { name: 'Resend', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentSendRequests)).toBe(2);
  const secondRequest = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter((request) => request.method === 'sendPrompt')[1]);
  expect(secondRequest.args.clientNonce).not.toBe(firstRequest.args.clientNonce);
  expect(secondRequest.args).toMatchObject({
    prompt: text, richText: complete.richText, replyToId: 'agent-answer',
    attachmentPaths: firstRequest.args.attachmentPaths, attachmentNames: firstRequest.args.attachmentNames,
  });
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().attachmentCommits)).toHaveLength(3);
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.publishAgentSend(1);
    window.nativeMessageAuthorRecoveryFixture.settleAgentSend(1);
  });
  await expectSent(page, 'agent-accepted:802');
  await expect(editor(page)).toHaveText('');
  await expect.poll(() => agentDraft()).toBeNull();
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter((request) =>
    request.method === 'sendHumanMessage' || request.method === 'retryHumanMessage'))).toEqual([]);
  await milestone(page, info, 'Agent-complete-input-retry-without-repeated-commits');
});

test('NMR-UI-01 clear invalidates old pages, send failure and reaction receipt before the same row ID is reused', async ({ page }, info) => {
  await openAlice(page, info);
  await reactTo(page, 'human-server-message:103', '❤️');
  await sendMessage(page, 'Before clear private dispatch');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.reconnect());
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().refreshes)).toBeGreaterThan(0);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.clear('human:a'));
  await expect.poll(() => domIds(page)).toEqual([]);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.appendCurrent('human:a', 'human-server-message:103', 'A fresh row after clear'));
  await expect(messageRow(page, 'human-server-message:103')).toContainText('A fresh row after clear');
  await editor(page).fill('New draft after clear');
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe('New draft after clear');
  const draftBefore = await clientDraft(page);
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.releaseOlder();
    window.nativeMessageAuthorRecoveryFixture.releaseRefreshes();
    window.nativeMessageAuthorRecoveryFixture.settleReaction();
    window.nativeMessageAuthorRecoveryFixture.rejectSend(0, 'Old clear-scope semantic failure');
  });
  await drainUi(page);
  await expect.poll(() => domIds(page)).toEqual(['human-server-message:103']);
  await expect(messageRow(page, 'human-server-message:103')).toContainText('A fresh row after clear');
  await expect(messageRow(page, 'human-server-message:103').locator('.sand-reaction-pill')).toHaveCount(0);
  await expectSent(page, 'human-server-message:103');
  await expect(editor(page)).toHaveText('New draft after clear');
  expect(await clientDraft(page)).toEqual(draftBefore);
  await expect(page.getByText('Old clear-scope semantic failure', { exact: false })).toHaveCount(0);
  await milestone(page, info, 'clear-generation-rejects-old-results');
});

test('NMR-UI-01 A to B to A cannot reuse the old callback lifetime even when conversation and row IDs match', async ({ page }, info) => {
  await openAlice(page, info);
  await reactTo(page, 'human-server-message:103', '❤️');
  await sendMessage(page, 'Old A dispatch');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.reconnect());
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().refreshes)).toBeGreaterThan(0);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.setAccount('9', 'human-server-message:903', 'B account row'));
  await expect(messageRow(page, 'human-server-message:903')).toContainText('B account row');
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.setAccount('1', 'human-server-message:103', 'A returned in a new lifetime'));
  await expect(messageRow(page, 'human-server-message:103')).toContainText('A returned in a new lifetime');
  await editor(page).fill('New A draft');
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe('New A draft');
  const draftBefore = await clientDraft(page);
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.releaseOlder();
    window.nativeMessageAuthorRecoveryFixture.releaseRefreshes();
    window.nativeMessageAuthorRecoveryFixture.settleReaction();
    window.nativeMessageAuthorRecoveryFixture.rejectSend(0, 'Old A lifetime refusal');
  });
  await drainUi(page);
  await expect.poll(() => domIds(page)).toEqual(['human-server-message:103']);
  await expect(messageRow(page, 'human-server-message:103')).toContainText('A returned in a new lifetime');
  await expect(messageRow(page, 'human-server-message:103').locator('.sand-reaction-pill')).toHaveCount(0);
  await expect(editor(page)).toHaveText('New A draft');
  expect(await clientDraft(page)).toEqual(draftBefore);
  await expect(page.getByText('Old A lifetime refusal', { exact: false })).toHaveCount(0);
  const current = await sendMessage(page, 'A current lifetime positive control');
  expect(current.index).toBe(1);
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.publishSend(1);
    window.nativeMessageAuthorRecoveryFixture.settleSend(1);
  });
  await expectSent(page, secondSentId);
  await expect(editor(page)).toHaveText('');
  await expect.poll(() => domIds(page)).toEqual(['human-server-message:103', secondSentId]);
  await milestone(page, info, 'aba-generation-and-current-send-control');
});

test('NMR-UI-01 root disposal releases its port and old work cannot enter a new renderer instance', async ({ page }, info) => {
  await openAlice(page, info);
  await reactTo(page, 'human-server-message:103', '❤️');
  await sendMessage(page, 'Disposed renderer dispatch');
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe('Disposed renderer dispatch');
  const draftBeforeUnmount = await clientDraft(page);
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.unmount();
    // Deliberately settle in the same task as unmount, before deferred cleanup.
    window.nativeMessageAuthorRecoveryFixture.rejectSend(0, 'Disposed renderer refusal');
  });
  await expect(page.locator('#root')).toBeEmpty();
  await drainUi(page);
  expect(await clientDraft(page)).toEqual(draftBeforeUnmount);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().endpoints.every((endpoint) => endpoint.closed))).toBe(true);
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.replaceConversation('human:a', 'human-server-message:103', 'New renderer service row');
    window.nativeMessageAuthorRecoveryFixture.mount();
  });
  await expect(messageRow(page, 'human-server-message:103')).toContainText('New renderer service row');
  await editor(page).fill('New renderer draft');
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe('New renderer draft');
  const draftBefore = await clientDraft(page);
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.releaseOlder();
    window.nativeMessageAuthorRecoveryFixture.releaseRefreshes();
    window.nativeMessageAuthorRecoveryFixture.settleReaction();
  });
  await drainUi(page);
  await expect.poll(() => domIds(page)).toEqual(['human-server-message:103']);
  await expect(messageRow(page, 'human-server-message:103')).toContainText('New renderer service row');
  await expect(messageRow(page, 'human-server-message:103').locator('.sand-reaction-pill')).toHaveCount(0);
  await expect(editor(page)).toHaveText('New renderer draft');
  expect(await clientDraft(page)).toEqual(draftBefore);
  await expect(page.getByText('Disposed renderer refusal', { exact: false })).toHaveCount(0);
  const endpoints = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().endpoints);
  expect(endpoints.filter((endpoint) => !endpoint.closed)).toHaveLength(1);
  await milestone(page, info, 'disposed-port-and-new-root');
});

test('NMR-UI-01 the same renderer keeps Agent tool expansion and real attachment staging controls', async ({ page }, info) => {
  await openFixture(page, info, { agent: '1' });
  await page.getByRole('button', { name: 'Fixture Agent', exact: true }).click();
  await expect(messageRow(page, 'agent-answer')).toContainText('The Agent answer is intact.');
  await expect(messageRow(page, 'agent-request')).toContainText('agent.txt');
  const tool = transcript(page).locator('[data-kind="tool-call"]');
  await expect(tool).toHaveAttribute('data-status', 'done');
  await tool.getByRole('button').click();
  await expect(tool.getByRole('button')).toHaveAttribute('aria-expanded', 'true');
  await expect(tool.locator('.sand-outline-item__detail')).toContainText('Tool result retained');
  await expect(page.getByRole('button', { name: 'Attach file', exact: true })).toBeEnabled();
  await page.locator('.sand-prompt-form input[type="file"]').setInputFiles({
    name: 'staged.txt', mimeType: 'text/plain', buffer: Buffer.from('attachment-control'),
  });
  await expect(page.getByRole('list', { name: 'Attachments', exact: true })).toContainText('staged.txt');
  await page.getByRole('button', { name: 'Remove staged.txt', exact: true }).click();
  await expect(page.getByRole('list', { name: 'Attachments', exact: true })).toHaveCount(0);
  const calls = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().desktopCalls);
  expect(calls.filter((call) => call.method === 'stageAttachmentBytes')).toEqual([
    { method: 'stageAttachmentBytes', name: 'staged.txt', path: 'memory:staged/staged.txt', bytes: [...Buffer.from('attachment-control')] },
  ]);
  expect(calls.filter((call) => call.method === 'discardStagedAttachment')).toEqual([
    { method: 'discardStagedAttachment', path: 'memory:staged/staged.txt' },
  ]);
  await milestone(page, info, 'agent-tools-and-attachment-controls');
});

test('NMR-UI-01 background transcript recovery preserves Plugins until explicit navigation', async ({ page }, info) => {
  await openFixture(page, info, { agent: '1', holdAgentRoster: '1' });
  const plugins = page.getByRole('button', { name: 'Plugins', exact: true });
  const dialog = page.getByRole('dialog', { name: 'Plugins', exact: true });
  const close = dialog.getByRole('button', { name: 'Close', exact: true });
  const agentTailRequests = () => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter(
    (request) => request.method === 'openAgentTail' && request.args.id === 'agent:helper',
  ).length);

  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentRosters)).toBeGreaterThan(0);
  expect(await agentTailRequests()).toBe(0);
  await plugins.click();
  await expect(dialog).toBeVisible();
  await expect(close).toHaveCount(1);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.releaseAgentRosters());
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentRosters)).toBe(0);
  await expect.poll(agentTailRequests).toBeGreaterThan(0);
  await expect(messageRow(page, 'agent-answer')).toContainText('The Agent answer is intact.');
  await drainUi(page);
  await expect(dialog).toBeVisible();
  await expect(close).toBeVisible();
  await milestone(page, info, 'Plugins-survives-late-roster-and-actual-tail');
  await close.click();
  await expect(dialog).toHaveCount(0);

  // Clearing the selected transcript invokes the existing background reload.
  // Hold only that external tail response so user actions occur while the same
  // Agent is still loading; no component state or callback is injected.
  await plugins.click();
  await expect(dialog).toBeVisible();
  await page.evaluate(() => {
    window.nativeMessageAuthorRecoveryFixture.holdAgentTranscriptReads();
    window.nativeMessageAuthorRecoveryFixture.clear('agent:helper');
  });
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentTails)).toBe(1);
  const heldTailCount = await agentTailRequests();
  await drainUi(page);
  await expect(dialog).toBeVisible();
  await expect(close).toHaveCount(1);
  await close.click();
  await expect(dialog).toHaveCount(0);
  await milestone(page, info, 'Plugins-Close-remains-actionable-during-background-clear-reload');

  // The normal Mod+1 shortcut explicitly reselects the only Agent. It must
  // dismiss Plugins even though the background first-page request is pending.
  await plugins.click();
  await expect(dialog).toBeVisible();
  await page.keyboard.press('Control+1');
  await expect(dialog).toHaveCount(0);
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentTails)).toBe(1);
  expect(await agentTailRequests()).toBe(heldTailCount);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.releaseAgentTranscriptReads());
  await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().agentTails)).toBe(0);
  await expect.poll(() => domIds(page)).toEqual([]);
  await expect(editor(page)).toBeEditable();
  expect(await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.requests.filter((request) =>
    request.method === 'sendPrompt' || request.method === 'sendHumanMessage' || request.method === 'retryHumanMessage'))).toEqual([]);
  await milestone(page, info, 'explicit-navigation-closes-Plugins-without-duplicating-pending-tail');
});

test('NMR-TRANSPORT-RECOVERY-01 runtime Host and SSE recovery preserve the active Human draft after a held Retry', async ({ page }, info) => {
  await openAlice(page, info);
  await loadOlder(page);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.releaseRefreshes());
  await drainUi(page);

  const counts = () => page.evaluate(() => {
    const requests = window.nativeMessageAuthorRecoveryFixture.requests;
    return {
      roster: requests.filter((request) => request.method === 'listAgents').length,
      humanSync: requests.filter((request) => request.method === 'syncHumanConversations').length,
      transcript: requests.filter((request) => request.method === 'getHumanConversationTranscript'
        && request.args.conversationId === 'human:a').length,
    };
  });
  const heldRosters = () => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics().heldAgentRosters);
  const runtime = (event: Parameters<Window['nativeMessageAuthorRecoveryFixture']['runtimeLifecycle']>[0]) =>
    page.evaluate((value) => window.nativeMessageAuthorRecoveryFixture.runtimeLifecycle(value), event);
  const reconnectNotice = page.locator('.sand-agents-reconnect-notice');
  const initialUrl = page.url();
  const initialDiagnostics = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics());
  const currentPorts = initialDiagnostics.endpoints.filter((endpoint) => !endpoint.closed);
  expect(currentPorts).toHaveLength(1);
  expect(initialDiagnostics.mountCount).toBe(1);
  const portId = currentPorts[0].id;

  // Establish the actual native runtime epoch on the already serving port.
  // A Host process being alive is not yet a connected Gateway event stream.
  const beforeEpoch = await counts();
  expect(await runtime({ type: 'host.lifecycle', lifecycle: 'running', generation: 1, sequence: 1 })).toBe(portId);
  expect(await runtime({ type: 'gateway.lifecycle', lifecycle: 'transport-connected', generation: 1, sequence: 2 })).toBe(portId);
  await expect.poll(counts).toEqual({
    roster: beforeEpoch.roster + 1, humanSync: beforeEpoch.humanSync + 1, transcript: beforeEpoch.transcript + 1,
  });
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.releaseRefreshes());
  await expect.poll(() => domIds(page)).toEqual(historyIds);
  await drainUi(page);

  const text = 'Keep this complete Human draft while the native transport recovers.';
  await selectReply(page, 'human-server-message:104');
  await editor(page).fill(text);
  await expect.poll(async () => (await clientDraft(page))?.draft?.prompt).toBe(text);
  const draftBefore = await clientDraft(page);
  expect(draftBefore.draft.replyToId).toBe('human-server-message:104');
  expect(draftBefore.draft.richText).toEqual(expect.any(String));
  expect(draftBefore.draftId).toEqual(expect.any(String));
  const expectedIds = [...historyIds];
  const expectRetained = async () => {
    await expect(page.getByRole('button', { name: 'Alice', exact: true })).toHaveAttribute('aria-current', 'page');
    await expect.poll(() => domIds(page)).toEqual(expectedIds);
    await expect(editor(page)).toHaveText(text);
    await expect(page.getByRole('button', { name: 'Cancel reply', exact: true })).toBeVisible();
    expect(await clientDraft(page)).toEqual(draftBefore);
    expect(page.url()).toBe(initialUrl);
    const diagnostic = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics());
    expect(diagnostic.mountCount).toBe(1);
    expect(diagnostic.endpoints).toHaveLength(1);
    expect(diagnostic.endpoints[0]).toMatchObject({ id: portId, closed: false });
  };
  const finishCurrentRoster = async (index: number, before: Awaited<ReturnType<typeof counts>>, entryId: string, content: string) => {
    // The response goes through the actual Coordinator client and Root scope
    // fence. Only that accepted current attempt may start Human convergence.
    await page.evaluate((position) => window.nativeMessageAuthorRecoveryFixture.releaseAgentRoster(position), index);
    await expect.poll(counts).toEqual({
      roster: before.roster + 1, humanSync: before.humanSync + 1, transcript: before.transcript + 1,
    });
    await expect.poll(() => page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.pending().refreshes)).toBe(1);
    await expect(messageRow(page, entryId)).toHaveCount(0);
    await expectRetained();
    await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.releaseRefreshes());
    expectedIds.push(entryId);
    await expect(messageRow(page, entryId)).toContainText(content);
    await expectRetained();
  };

  // Replace the Host without navigating or handing the renderer another port.
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.holdAgentRosterReads());
  const beforeHost = await counts();
  await runtime({ type: 'host.lifecycle', lifecycle: 'stopped', generation: 1, sequence: 3 });
  await expect(reconnectNotice).toBeVisible();
  await expectRetained();
  await runtime({ type: 'host.lifecycle', lifecycle: 'running', generation: 2, sequence: 4 });
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.appendServiceOnly(
    'human:a', 'human-server-message:106', 'History recovered from the replacement Host',
  ));
  await drainUi(page);
  expect(await counts(), 'Host running alone cannot start the recovery read').toEqual(beforeHost);
  await expect(messageRow(page, 'human-server-message:106')).toHaveCount(0);
  await runtime({ type: 'gateway.lifecycle', lifecycle: 'transport-connected', generation: 2, sequence: 5 });
  await expect.poll(async () => (await heldRosters()).length).toBe(1);
  expect(await counts()).toEqual({ ...beforeHost, roster: beforeHost.roster + 1 });
  await expectRetained();
  await finishCurrentRoster(0, beforeHost, 'human-server-message:106', 'History recovered from the replacement Host');
  await milestone(page, info, 'native-Host-replacement-refreshes-Human-without-navigation');

  // A reconnect of the same Gateway generation needs the same production
  // refresh path; a process-generation-only notification would miss this row.
  const beforeSse = await counts();
  await runtime({ type: 'gateway.lifecycle', lifecycle: 'transport-down', generation: 2, sequence: 6 });
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.appendServiceOnly(
    'human:a', 'human-server-message:107', 'History recovered after the same Host SSE reconnect',
  ));
  await expectRetained();
  await runtime({ type: 'gateway.lifecycle', lifecycle: 'transport-connected', generation: 2, sequence: 7 });
  await expect.poll(async () => (await heldRosters()).length).toBe(2);
  expect(await counts()).toEqual({ ...beforeSse, roster: beforeSse.roster + 1 });
  await finishCurrentRoster(1, beforeSse, 'human-server-message:107', 'History recovered after the same Host SSE reconnect');
  await milestone(page, info, 'same-generation-SSE-restores-the-active-Human-cache');

  // Make the existing Retry UI available by failing a real roster RPC. Never
  // inject controller state or call its retry callback from the fixture.
  const beforeFailure = await counts();
  await runtime({ type: 'gateway.lifecycle', lifecycle: 'transport-down', generation: 2, sequence: 8 });
  await runtime({ type: 'gateway.lifecycle', lifecycle: 'transport-connected', generation: 2, sequence: 9 });
  await expect.poll(async () => (await heldRosters()).length).toBe(3);
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.rejectAgentRoster(
    2, 'The native roster is temporarily unavailable before the user Retry',
  ));
  await expect(reconnectNotice.getByRole('button', { name: 'Retry', exact: true })).toBeEnabled();
  expect(await counts()).toEqual({ ...beforeFailure, roster: beforeFailure.roster + 1 });
  await expectRetained();
  await reconnectNotice.getByRole('button', { name: 'Retry', exact: true }).click();
  await expect.poll(async () => (await heldRosters()).length).toBe(4);
  await expect(reconnectNotice.getByRole('button', { name: 'Retrying…', exact: true })).toBeDisabled();
  const oldRetry = (await heldRosters())[3];
  expect(oldRetry.settled).toBe(false);
  expect(oldRetry.request).toMatchObject({ method: 'listAgents', args: {}, accountId: '1', portId });

  const beforeHandoff = await counts();
  await runtime({ type: 'gateway.lifecycle', lifecycle: 'transport-down', generation: 2, sequence: 10 });
  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.appendServiceOnly(
    'human:a', 'human-server-message:108', 'The new Retry owns the post-reconnect Human refresh',
  ));
  await runtime({ type: 'gateway.lifecycle', lifecycle: 'transport-connected', generation: 2, sequence: 11 });
  // Controller subscribes before Root. The replacement must begin after the
  // complete transport fanout, otherwise Root captures an obsolete generation.
  await expect.poll(async () => (await heldRosters()).length).toBe(5);
  const newRetry = (await heldRosters())[4];
  expect(newRetry.settled).toBe(false);
  expect(newRetry.request).toMatchObject({ method: 'listAgents', args: {}, accountId: '1', portId });
  expect(newRetry.request.sequence).toBeGreaterThan(oldRetry.request.sequence);
  expect(newRetry.request.requestId).not.toBe(oldRetry.request.requestId);
  expect(await counts()).toEqual({ ...beforeHandoff, roster: beforeHandoff.roster + 1 });
  await expect(reconnectNotice.getByRole('button', { name: 'Retrying…', exact: true })).toBeDisabled();
  await expectRetained();

  await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.rejectAgentRoster(
    3, 'Stale pre-reconnect roster reply must stay inert',
  ));
  await drainUi(page);
  expect((await heldRosters())[3].settled).toBe(true);
  expect((await heldRosters())[4].settled).toBe(false);
  expect(await counts(), 'The superseded Retry cannot start Human sync or consume the current attempt')
    .toEqual({ ...beforeHandoff, roster: beforeHandoff.roster + 1 });
  await expect(reconnectNotice.getByRole('button', { name: 'Retrying…', exact: true })).toBeDisabled();
  await expect(page.getByText('Stale pre-reconnect roster reply must stay inert', { exact: false })).toHaveCount(0);
  await expectRetained();
  await milestone(page, info, 'late-old-Retry-cannot-clear-the-current-recovery');

  await finishCurrentRoster(4, beforeHandoff, 'human-server-message:108', 'The new Retry owns the post-reconnect Human refresh');
  await expect(reconnectNotice).toHaveCount(0);
  await drainUi(page);
  await expectRetained();
  const final = await page.evaluate(() => window.nativeMessageAuthorRecoveryFixture.diagnostics());
  expect(final.heldAgentRosters).toHaveLength(5);
  expect(final.heldAgentRosters.every((pending) => pending.settled)).toBe(true);
  expect(final.requests.filter((request) => request.method === 'sendPrompt'
    || request.method === 'sendHumanMessage' || request.method === 'retryHumanMessage')).toEqual([]);
  const runtimeFrames = final.frames.filter((entry) =>
    (entry.frame as { family?: string }).family === 'runtime');
  expect(runtimeFrames).toHaveLength(11);
  expect(runtimeFrames.every((entry) => entry.portId === portId && entry.dropped === false)).toBe(true);
  expect(runtimeFrames.map((entry) => (entry.frame as { payload: { sequence: number } }).payload.sequence))
    .toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
  await milestone(page, info, 'current-Retry-completes-Human-sync-and-Transcript-with-retained-draft');
});
