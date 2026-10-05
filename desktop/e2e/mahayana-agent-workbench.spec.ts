import { _electron as electron, expect, test, type ElectronApplication, type Locator, type Page } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;

let e2eAuthServer: ReturnType<typeof createServer> | null = null;
let e2eAuthBackendPromise: Promise<string> | null = null;

function e2eAuthToken(): string {
  const encode = (value: Record<string, unknown>) => Buffer.from(JSON.stringify(value), 'utf8').toString('base64url');
  return [
    encode({ alg: 'none', typ: 'JWT' }),
    encode({ sub: 'fabushi-e2e-account', email: 'e2e@fabushi.local', exp: 4_102_444_800 }),
    'fabushi-e2e',
  ].join('.');
}


function encodeVarint(value: number): Buffer {
  const bytes: number[] = [];
  let remaining = value >>> 0;
  do {
    let byte = remaining & 0x7f;
    remaining >>>= 7;
    if (remaining !== 0) byte |= 0x80;
    bytes.push(byte);
  } while (remaining !== 0);
  return Buffer.from(bytes);
}

function encodeLengthDelimited(fieldNumber: number, payload: Buffer): Buffer {
  return Buffer.concat([
    encodeVarint((fieldNumber << 3) | 2),
    encodeVarint(payload.length),
    payload,
  ]);
}

function encodeConnectFrame(payload: Buffer): Buffer {
  const header = Buffer.alloc(5);
  header.writeUInt8(0, 0);
  header.writeUInt32BE(payload.length, 1);
  return Buffer.concat([header, payload]);
}

function encodeCursorTextPart(text: string, isFinal: boolean): Buffer {
  const fields: Buffer[] = [];
  if (text.length > 0) fields.push(encodeLengthDelimited(1, Buffer.from(text, 'utf8')));
  if (isFinal) fields.push(Buffer.from([0x10, 0x01]));
  const textPart = Buffer.concat(fields);
  return encodeConnectFrame(encodeLengthDelimited(1, textPart));
}

async function readRequestBody(request: import('node:http').IncomingMessage): Promise<Buffer> {
  const chunks: Buffer[] = [];
  for await (const chunk of request) chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  return Buffer.concat(chunks);
}

type E2eHumanMessage = {
  id: number;
  senderUserId: string;
  senderUsername: string;
  recipientUserId: string;
  recipientUsername: string;
  text: string;
  clientRequestId: string;
  createdAt: string;
  readAt: null;
  isOutgoing: true;
  replyToMessageId: null;
  attachments: [];
  reactions: [];
};

let e2eHumanMessages: E2eHumanMessage[] = [];
let e2eHumanMessageSequence = 1;

async function ensureE2eAuthBackend(): Promise<string> {
  if (e2eAuthBackendPromise != null) return await e2eAuthBackendPromise;
  e2eAuthBackendPromise = new Promise<string>((resolve, reject) => {
    const token = e2eAuthToken();
    const server = createServer(async (request, response) => {
      const requestUrl = new URL(request.url ?? '/', 'http://127.0.0.1');
      if (requestUrl.pathname === '/auth/poll') {
        response.setHeader('content-type', 'application/json');
        response.statusCode = 200;
        response.end(JSON.stringify({ accessToken: token, refreshToken: token }));
        return;
      }
      if (requestUrl.pathname === '/oauth/token') {
        response.setHeader('content-type', 'application/json');
        response.statusCode = 200;
        response.end(JSON.stringify({ access_token: token, refresh_token: token }));
        return;
      }
      if (requestUrl.pathname === '/api/auth/browser/start' && request.method === 'POST') {
        const origin = `http://${request.headers.host ?? '127.0.0.1'}`;
        response.setHeader('content-type', 'application/json');
        response.statusCode = 200;
        response.end(JSON.stringify({
          attemptId: 'fabushi-e2e-browser-attempt',
          loginUrl: `${origin}/fabushi-e2e-browser-login`,
          pollSecret: 'fabushi-e2e-poll-secret',
          expiresAt: Date.now() + 60_000,
          pollAfterMs: 250,
        }));
        return;
      }
      if (requestUrl.pathname === '/api/auth/browser/attempts/fabushi-e2e-browser-attempt' && request.method === 'POST') {
        const body = JSON.parse((await readRequestBody(request)).toString('utf8')) as { pollSecret?: unknown };
        if (body.pollSecret !== 'fabushi-e2e-poll-secret') {
          response.setHeader('content-type', 'application/json');
          response.statusCode = 403;
          response.end(JSON.stringify({ error: { code: 'invalid-poll-secret' } }));
          return;
        }
        const expiresAt = 4_102_444_800_000;
        response.setHeader('content-type', 'application/json');
        response.statusCode = 200;
        response.end(JSON.stringify({
          status: 'completed',
          session: {
            accessToken: token,
            refreshToken: token,
            accessTokenExpiresAt: expiresAt,
            refreshTokenExpiresAt: expiresAt,
            sessionId: 'fabushi-e2e-session',
            deviceId: 'fabushi-e2e-device',
            username: 'e2e@fabushi.local',
            userId: 'fabushi-e2e-account',
            provider: 'focused-e2e',
          },
        }));
        return;
      }
      if (requestUrl.pathname === '/api/auth/user-info') {
        if (request.headers.authorization !== `Bearer ${token}`) {
          response.setHeader('content-type', 'application/json');
          response.statusCode = 401;
          response.end(JSON.stringify({ error: { code: 'invalid-access-token' } }));
          return;
        }
        response.setHeader('content-type', 'application/json');
        response.statusCode = 200;
        response.end(JSON.stringify({
          id: 'fabushi-e2e-account',
          username: 'e2e@fabushi.local',
          email: 'e2e@fabushi.local',
          displayName: 'Fabushi E2E',
        }));
        return;
      }
      if (requestUrl.pathname === '/api/auth/refresh' && request.method === 'POST') {
        const expiresAt = 4_102_444_800_000;
        response.setHeader('content-type', 'application/json');
        response.statusCode = 200;
        response.end(JSON.stringify({
          accessToken: token,
          refreshToken: token,
          accessTokenExpiresAt: expiresAt,
          refreshTokenExpiresAt: expiresAt,
          sessionId: 'fabushi-e2e-session',
          deviceId: 'fabushi-e2e-device',
          username: 'e2e@fabushi.local',
          userId: 'fabushi-e2e-account',
          provider: 'focused-e2e',
        }));
        return;
      }
      if (requestUrl.pathname === '/api/social/friends' && request.method === 'GET') {
        if (request.headers.authorization !== `Bearer ${token}`
          || request.headers['x-fabushi-device-id'] !== 'fabushi-e2e-device') {
          response.setHeader('content-type', 'application/json');
          response.statusCode = 401;
          response.end(JSON.stringify({ success: false, error: 'invalid-social-credential' }));
          return;
        }
        response.setHeader('content-type', 'application/json');
        response.statusCode = 200;
        response.end(JSON.stringify({ success: true, data: { friends: [] } }));
        return;
      }
      if (requestUrl.pathname === '/api/social/messages' && request.method === 'GET') {
        if (request.headers.authorization !== `Bearer ${token}`
          || request.headers['x-fabushi-device-id'] !== 'fabushi-e2e-device') {
          response.setHeader('content-type', 'application/json');
          response.statusCode = 401;
          response.end(JSON.stringify({ success: false, error: 'invalid-social-credential' }));
          return;
        }
        const contactId = requestUrl.searchParams.get('contactId') ?? '';
        const before = requestUrl.searchParams.get('before');
        const requestedLimit = Number.parseInt(requestUrl.searchParams.get('limit') ?? '200', 10);
        const limit = Number.isFinite(requestedLimit) ? Math.max(1, Math.min(requestedLimit, 200)) : 200;
        const rows = e2eHumanMessages
          .filter((message) => message.recipientUserId === contactId || message.senderUserId === contactId)
          .filter((message) => before == null || message.createdAt < before)
          .slice(-limit);
        response.setHeader('content-type', 'application/json');
        response.statusCode = 200;
        response.end(JSON.stringify({ success: true, data: { messages: rows } }));
        return;
      }
      if (requestUrl.pathname === '/api/social/messages' && request.method === 'POST') {
        if (request.headers.authorization !== `Bearer ${token}`
          || request.headers['x-fabushi-device-id'] !== 'fabushi-e2e-device') {
          response.setHeader('content-type', 'application/json');
          response.statusCode = 401;
          response.end(JSON.stringify({ success: false, error: 'invalid-social-credential' }));
          return;
        }
        const body = JSON.parse((await readRequestBody(request)).toString('utf8')) as {
          targetUserId?: unknown;
          text?: unknown;
          clientRequestId?: unknown;
          replyToMessageId?: unknown;
          attachments?: unknown;
        };
        const targetUserId = typeof body.targetUserId === 'string' ? body.targetUserId.trim() : '';
        const text = typeof body.text === 'string' ? body.text.trim() : '';
        const clientRequestId = typeof body.clientRequestId === 'string' ? body.clientRequestId.trim() : '';
        if (targetUserId.length === 0 || clientRequestId.length === 0
          || body.replyToMessageId != null
          || (Array.isArray(body.attachments) && body.attachments.length > 0)) {
          response.setHeader('content-type', 'application/json');
          response.statusCode = 400;
          response.end(JSON.stringify({ success: false, error: 'invalid-focused-human-message' }));
          return;
        }
        let message = e2eHumanMessages.find((candidate) => candidate.clientRequestId === clientRequestId);
        if (message == null) {
          message = {
            id: e2eHumanMessageSequence++,
            senderUserId: 'fabushi-e2e-account',
            senderUsername: 'e2e@fabushi.local',
            recipientUserId: targetUserId,
            recipientUsername: targetUserId,
            text,
            clientRequestId,
            createdAt: new Date().toISOString(),
            readAt: null,
            isOutgoing: true,
            replyToMessageId: null,
            attachments: [],
            reactions: [],
          };
          e2eHumanMessages.push(message);
        }
        response.setHeader('content-type', 'application/json');
        response.statusCode = 200;
        response.end(JSON.stringify({ success: true, message }));
        return;
      }
      if (requestUrl.pathname === '/v1/ai/responses' && request.method === 'POST') {
        const body = await readRequestBody(request);
        if (request.headers.authorization !== `Bearer ${token}`
          || !String(request.headers['content-type'] ?? '').startsWith('application/json')) {
          response.setHeader('content-type', 'application/json');
          response.statusCode = 401;
          response.end(JSON.stringify({ error: 'invalid-fabushi-responses-request' }));
          return;
        }
        const isSelfHosted = body.includes(Buffer.from('自建 Bot', 'utf8'));
        const text = isSelfHosted
          ? '收到：自建 Bot 请规划步骤'
          : '收到：请分析这个任务';
        response.setHeader('content-type', 'text/event-stream');
        response.statusCode = 200;
        response.end([
          `data: ${JSON.stringify({ type: 'response.output_text.delta', delta: text })}\n\n`,
          `data: ${JSON.stringify({
            type: 'response.completed',
            response: {
              id: 'fabushi-e2e-response',
              output: [],
              usage: {
                input_tokens: 8,
                output_tokens: 8,
                input_tokens_details: { cached_tokens: 0 },
              },
            },
          })}\n\n`,
        ].join(''));
        return;
      }
      if (requestUrl.pathname === '/aiserver.v1.InferenceService/Stream') {
        const body = await readRequestBody(request);
        const authorization = request.headers.authorization;
        const requestId = request.headers['x-request-id'];
        const contentType = request.headers['content-type'];
        if (authorization !== `Bearer ${token}`
          || typeof requestId !== 'string'
          || !requestId
          || contentType !== 'application/connect+proto'
          || body.length < 6) {
          response.setHeader('content-type', 'application/json');
          response.statusCode = 400;
          response.end(JSON.stringify({ error: 'invalid-inference-request' }));
          return;
        }
        const isSelfHosted = body.includes(Buffer.from('自建 Bot', 'utf8'));
        const text = isSelfHosted
          ? '收到：自建 Bot 请规划步骤'
          : '收到：请分析这个任务';
        response.setHeader('content-type', 'application/connect+proto');
        response.statusCode = 200;
        response.end(Buffer.concat([
          encodeCursorTextPart(text, false),
          encodeCursorTextPart('', true),
        ]));
        return;
      }
      response.setHeader('content-type', 'application/json');
      response.statusCode = 404;
      response.end(JSON.stringify({ error: 'not-found' }));
    });
    e2eAuthServer = server;
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      if (address == null || typeof address === 'string') {
        reject(new Error('Focused Electron auth backend did not bind a TCP address.'));
        return;
      }
      resolve(`http://127.0.0.1:${address.port}`);
    });
  });
  return await e2eAuthBackendPromise;
}

test.afterAll(async () => {
  const server = e2eAuthServer;
  e2eAuthServer = null;
  e2eAuthBackendPromise = null;
  if (server == null || !server.listening) return;
  await new Promise<void>((resolve, reject) => {
    server.close((error) => error == null ? resolve() : reject(error));
  });
});

async function launchDesktopApp(appDataDir: string): Promise<ElectronApplication> {
  const e2eAuthBackendUrl = await ensureE2eAuthBackend();
  // Exercise the shipping Fabushi account -> private Host credential ->
  // Responses transport chain. The browser-first login below owns the token;
  // focused acceptance must not seed a parallel Cursor inference credential.
  return electron.launch({
    ...(packagedExecutable
      ? { executablePath: packagedExecutable, args: [] }
      : { args: [appRoot] }),
    env: {
      ...process.env,
      // Focused Electron acceptance must opt out of production background
      // persistence so Playwright app.close() reaches the real before-quit
      // cleanup path instead of being converted into a hidden-window session.
      FABUSHI_E2E: '1',
      SAND_USER_DATA_DIR: appDataDir,
      SAND_BACKEND_URL: e2eAuthBackendUrl,
      CURSOR_API_BASE_URL: e2eAuthBackendUrl,
      FABUSHI_API_BASE_URL: e2eAuthBackendUrl,
      FABUSHI_RESPONSES_URL: `${e2eAuthBackendUrl}/v1/ai/responses`,
      SAND_CURSOR_WEBSITE_URL: e2eAuthBackendUrl,
      SAND_DISABLE_SENTRY: '1',
      FABUSHI_FEATURE_HOST_MODE: process.env.FABUSHI_FEATURE_HOST_MODE || 'test',
      MAHAYANA_APP_HOST_BIN: process.env.MAHAYANA_APP_HOST_BIN || '',
    },
  });
}

async function completeBrowserLogin(page: Page): Promise<void> {
  const rendererErrors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') rendererErrors.push(message.text());
  });

  await page.waitForFunction(() => {
    const candidate = window as unknown as { desktop?: { cursorAccount?: { getStatus?: unknown }; onboarding?: { setSeen?: unknown } } };
    return typeof candidate.desktop?.cursorAccount?.getStatus === 'function'
      && typeof candidate.desktop?.onboarding?.setSeen === 'function';
  }, { timeout: 15_000 });

  const accountKind = async (): Promise<string> => page.evaluate(async () => {
    const candidate = window as unknown as {
      desktop: {
        cursorAccount: { getStatus(): Promise<{ kind?: string }> };
      };
    };
    return (await candidate.desktop.cursorAccount.getStatus()).kind ?? 'unknown';
  });

  const initialKind = await accountKind();
  if (initialKind === 'logged-out') {
    // Exercise the shipping recovered-Grok sign-in surface instead of the
    // retired DesktopAuthBoundary. Its button calls cursorAccount.login(),
    // which owns the Electron/main auth contract used by the production shell.
    const signInSurface = page.getByRole('main', { name: 'Grok Bot', exact: true });
    await expect(signInSurface).toBeVisible({ timeout: 15_000 });
    await signInSurface.getByRole('button', { name: 'Sign in', exact: true }).click();
  }

  await expect.poll(accountKind, { timeout: 20_000 }).toBe('logged-in');
  // Onboarding persistence is account-scoped. Complete the shipping login
  // transition first, then persist the public preference for the authenticated
  // account and verify the mirror before recreating the renderer. This avoids
  // writing into a departing anonymous scope while still exercising the real
  // cursorAccount -> Electron main -> Coordinator/Host authentication path.
  await page.evaluate(async () => {
    const candidate = window as unknown as {
      desktop: {
        onboarding: {
          setSeen(seen: boolean): Promise<unknown>;
        };
      };
    };
    await candidate.desktop.onboarding.setSeen(true);
  });
  const onboardingSeen = async (): Promise<boolean> => page.evaluate(async () => {
    const candidate = window as unknown as {
      desktop: {
        onboarding: {
          getSeen(): Promise<boolean>;
        };
      };
    };
    return (await candidate.desktop.onboarding.getSeen()) === true;
  });
  await expect.poll(onboardingSeen, { timeout: 10_000 }).toBe(true);

  // The production renderer resolves onboarding at account/bootstrap boundaries;
  // reload only the renderer after persisting the authenticated account setting.
  // Main/Coordinator/Host and the authenticated session remain live.
  await page.reload({ waitUntil: 'domcontentloaded' });
  await page.waitForFunction(() => {
    const candidate = window as unknown as { desktop?: { cursorAccount?: { getStatus?: unknown } } };
    return typeof candidate.desktop?.cursorAccount?.getStatus === 'function';
  }, { timeout: 15_000 });
  await expect.poll(accountKind, { timeout: 10_000 }).toBe('logged-in');
  await expect(page.getByRole('main', { name: 'Grok Bot', exact: true })).toBeHidden({ timeout: 10_000 });

  const fatal = page.locator('.sand-error-boundary--app');
  if (await fatal.count()) {
    const surfaceText = await fatal.first().innerText().catch(() => 'unknown renderer failure');
    throw new Error(`renderer root fatal: ${rendererErrors.at(-1) ?? surfaceText}`);
  }

  // listAgents is now owned by the shipping Rust Session store. A fresh
  // FABUSHI_APP_DATA directory is intentionally empty, so create the focused
  // fixture through the real New -> createAgent -> listAgents path instead of
  // relying on the retired compatibility Host's synthetic roster.
  const roster = page.getByRole('region', { name: 'Agent list' });
  const primary = roster.getByRole('button', { name: 'New chat', exact: true });
  const loadedAgentRows = roster.locator('button.sand-agent-item');
  const emptyRoster = roster.getByText('No saved agents yet.', { exact: true });
  await expect.poll(async () => {
    if (await loadedAgentRows.count() > 0) return 'loaded';
    return await emptyRoster.isVisible().catch(() => false) ? 'empty' : 'pending';
  }, { timeout: 15_000 }).not.toBe('pending');
  if (await primary.count() === 0) {
    // A fresh authenticated account may already contain the shipping default
    // Grok Agent. That proves the roster is loaded, but it is not the focused
    // fixture this suite needs. Create exactly one New chat through the real
    // production New -> createAgent -> refreshRoster path.
    await page.getByRole('button', { name: 'New', exact: true }).click();
  }
  await expect(primary).toHaveCount(1, { timeout: 15_000 });
  await expect(primary).toBeVisible({ timeout: 15_000 });
}

async function openMahayanaConversation(page: Page): Promise<void> {
  const peer = page.getByRole('region', { name: 'Agent list' }).getByRole('button', { name: 'New chat', exact: true });
  await expect(peer).toBeVisible({ timeout: 15_000 });
  await peer.click();
  const prompt = page.getByRole('textbox', { name: 'Prompt' });
  await expect(prompt).toBeVisible();
  // New -> createAgent -> refreshRoster -> openAgent is a real shipping async
  // transition. The production composer intentionally remains non-editable
  // while that transition owns the busy state, so acceptance must wait for
  // the same actionable contract a user sees instead of racing visibility.
  await expect(prompt).toHaveAttribute('contenteditable', 'true', { timeout: 15_000 });
}

async function createSelfHostedBotAcceptanceChannel(page: Page): Promise<{ conversationId: string; peerTestId: string }> {
  await page.getByTestId('profile-navigation-trigger').click();
  const chats = page.getByTitle('聊天', { exact: true });
  if (await chats.isVisible().catch(() => false)) await chats.click();
  await page.getByRole('button', { name: '新建', exact: true }).click();
  await page.getByRole('button', { name: '新建频道' }).click();
  await page.getByPlaceholder('频道名称').fill('自建 Bot Mahayana 验收');
  await page.getByPlaceholder('频道简介').fill('Rust messaging → Mahayana multi-step runtime');
  await page.getByRole('button', { name: '创建频道' }).click();

  const peer = page.locator('[data-testid^="peer-selfhosted:channel:"]').filter({ hasText: '自建 Bot Mahayana 验收' }).first();
  await expect(peer).toBeVisible({ timeout: 10_000 });
  const peerTestId = await peer.getAttribute('data-testid');
  expect(peerTestId).toBeTruthy();
  await peer.click();
  await expect(page.getByTestId('messenger-input')).toBeVisible();
  return {
    peerTestId: peerTestId!,
    conversationId: peerTestId!.replace(/^peer-selfhosted:/, ''),
  };
}

async function emitBotInvocationRequested(
  page: Page,
  conversationId: string,
  text: string,
): Promise<string> {
  return page.evaluate(({ conversationId: id, text: prompt }) => {
    const invocationId = `invocation:e2e:${Date.now()}:${Math.random().toString(36).slice(2, 8)}`;
    window.dispatchEvent(new CustomEvent('fabushi:mahayana-runtime-event', {
      detail: {
        type: 'messaging.event',
        timestamp: new Date().toISOString(),
        requestId: `messaging:e2e:${Date.now()}`,
        envelope: {
          protocolVersion: 2,
          cursor: String(Date.now()),
          serverTimeMs: Date.now(),
          event: {
            type: 'botInvocationRequested',
            invocation: {
              id: invocationId,
              botId: 'bot:e2e:helper',
              senderId: 'human:e2e',
              conversationId: id,
              text: { text: prompt, entities: [] },
              metadata: { source: 'messaging-service' },
              createdAtMs: Date.now(),
            },
          },
        },
      },
    }));
    return invocationId;
  }, { conversationId, text });
}

async function expectHermesAssistantTurn(page: Page, expectedText: string): Promise<Locator> {
  const transcript = page.getByRole('log', { name: 'Conversation transcript' });
  const matchingMessages = transcript.getByRole('group', { name: 'Agent message' }).filter({ hasText: expectedText });
  await expect(matchingMessages).toHaveCount(1, { timeout: 15_000 });

  const message = matchingMessages.first();
  await expect(message).toBeVisible();
  // The accessible Agent-message group is the stable production contract.
  // Resolve its nearest semantic turn container instead of depending on a
  // renderer implementation attribute such as data-role.
  const turn = message.locator('xpath=ancestor::*[@role="article"][1]');
  await expect(turn).toBeVisible();
  await expect(message).toBeVisible();
  const body = message.locator('.sand-message-prose');
  await expect(body).toContainText(expectedText);
  await expect(body).not.toContainText('chat-response');

  // Routine success belongs in the recovered Grok transcript. The retired
  // Mahayana turn/Workbench presentation must not reappear as a parallel
  // success surface, and no operation-scoped thinking/tool row may remain
  // pending after the final assistant message has settled.
  await expect(page.getByTestId('mahayana-assistant-turn')).toHaveCount(0);
  await expect(page.getByTestId('agent-workbench')).toBeHidden();
  await expect(transcript.locator('[data-kind="thinking"]')).toHaveCount(0);
  await expect(transcript.locator('[data-kind="tool-call"][data-status="pending"]')).toHaveCount(0);

  // Token-sized runtime deltas and the late final message must reconcile into
  // one canonical assistant body rather than producing duplicate replies.
  const bodyText = (await body.allTextContents()).join('');
  expect(bodyText.split(expectedText).length - 1).toBe(1);
  return turn;
}

test('Mahayana renders one Hermes-style assistant turn instead of a completion Workbench card', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-mahayana-assistant-turn-'));
  let app: ElectronApplication | null = null;

  try {
    app = await launchDesktopApp(appDataDir);
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMahayanaConversation(page);

    const prompt = '请分析这个任务，规划步骤，调用工具并给出最终结果。';
    const promptInput = page.getByRole('textbox', { name: 'Prompt' });
    await promptInput.click();
    // The recovered TipTap editor fences scope-switch transactions until it
    // observes a real UI edit. pressSequentially exercises the same input
    // contract as a user instead of mutating contenteditable DOM via fill().
    await promptInput.pressSequentially(prompt);
    const send = page.getByRole('button', { name: 'Send message' });
    await expect(send).toBeVisible();
    await send.click();

    // The user bubble is a local-first transition and must paint before the
    // Mahayana Host finishes accepting/routing the agent turn. The Hermes
    // assistant projection is validated below with the production 15s turn
    // contract; requiring it inside this 1s local-echo window races Host
    // acceptance and prevents the stronger lifecycle assertions from running.
    await expect(page.getByRole('article').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 1_000 });
    await expect(promptInput).toBeVisible();

    const turn = await expectHermesAssistantTurn(page, '收到：请分析这个任务');
    await expect(turn).toHaveCount(1);
    await expect(promptInput).toBeVisible();
  } finally {
    await app?.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
  }
});


test('Human call surface exposes the shipping WebRTC and Electron media bridge', async () => {
  e2eHumanMessages = [];
  e2eHumanMessageSequence = 1;
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-human-call-media-'));
  let app: ElectronApplication | null = null;
  try {
    app = await launchDesktopApp(appDataDir);
    const page = await app.firstWindow();
    await completeBrowserLogin(page);
    await openMahayanaConversation(page);
    await page.getByRole('button', { name: 'New Human chat', exact: true }).click();
    const dialog = page.getByRole('dialog', { name: 'New Human chat' });
    await dialog.getByRole('textbox', { name: 'Human identity' }).fill('human-call-peer-e2e');
    await dialog.getByRole('textbox', { name: 'Conversation title' }).fill('Human Call Peer');
    await dialog.getByRole('button', { name: 'Create', exact: true }).click();

    await expect(page.getByRole('group', { name: 'Human call controls' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Start voice call' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Start video call' })).toBeVisible();
    const capability = await page.evaluate(async () => {
      const candidate = window as unknown as {
        desktop: {
          callMedia: {
            requestPermissions(input: { audio: boolean; video: boolean }): Promise<unknown>;
            listDisplaySources(): Promise<unknown>;
          };
        };
      };
      return {
        hasPeerConnection: typeof RTCPeerConnection === 'function',
        hasGetUserMedia: typeof navigator.mediaDevices?.getUserMedia === 'function',
        permissionProbe: await candidate.desktop.callMedia.requestPermissions({ audio: false, video: false }),
        hasDisplaySourceBridge: typeof candidate.desktop.callMedia.listDisplaySources === 'function',
      };
    });
    expect(capability.hasPeerConnection).toBe(true);
    expect(capability.hasGetUserMedia).toBe(true);
    expect(capability.hasDisplaySourceBridge).toBe(true);
    expect(capability.permissionProbe).toEqual({ microphone: 'not-requested', camera: 'not-requested' });
  } finally {
    await app?.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('Mahayana renders one Hermes-style assistant turn after a Human conversation survives restart and explicitly hands off', async () => {
  e2eHumanMessages = [];
  e2eHumanMessageSequence = 1;
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-human-agent-vertical-slice-'));
  let app: ElectronApplication | null = null;
  const humanMessage = 'Human durable message for the native Fabushi conversation.';

  try {
    app = await launchDesktopApp(appDataDir);
    let page = await app.firstWindow();
    await completeBrowserLogin(page);

    // Create one real Agent through the shipping sidebar so the Human handoff
    // has an existing Coordinator -> Host -> Runner target.
    await openMahayanaConversation(page);
    await expect(page.getByRole('region', { name: 'Agent list' }).getByRole('button', { name: 'New chat', exact: true })).toBeVisible();

    // Human conversations are created from the same sidebar and mounted into
    // the same recovered conversation workspace rather than a parallel shell.
    await page.getByRole('button', { name: 'New Human chat', exact: true }).click();
    const humanDialog = page.getByRole('dialog', { name: 'New Human chat' });
    await expect(humanDialog).toBeVisible();
    await humanDialog.getByRole('textbox', { name: 'Human identity' }).fill('human-peer-e2e');
    await humanDialog.getByRole('textbox', { name: 'Conversation title' }).fill('Human Alice');
    await humanDialog.getByRole('button', { name: 'Create', exact: true }).click();

    await expect(page.getByText('Human', { exact: true })).toBeVisible();
    const prompt = page.getByRole('textbox', { name: 'Prompt' });
    await expect(prompt).toHaveAttribute('contenteditable', 'true', { timeout: 15_000 });
    await prompt.pressSequentially(humanMessage);
    await page.getByRole('button', { name: 'Send message' }).click();
    const durableHumanTurn = page.getByRole('article').filter({ hasText: humanMessage }).last();
    await expect(durableHumanTurn).toBeVisible({ timeout: 10_000 });
    // The optimistic bubble is not the durability boundary. Wait for the same
    // row to settle from pending to Host-accepted before terminating the app.
    await expect(durableHumanTurn).not.toHaveAttribute('data-pending', { timeout: 15_000 });

    const roster = page.getByRole('region', { name: 'Agent list' });
    await expect(roster.getByRole('button', { name: 'Human Alice', exact: true })).toBeVisible();
    await expect(roster.getByRole('button', { name: 'New chat', exact: true })).toBeVisible();

    // Restart the packaged/runtime app data scope and prove the authenticated
    // Human identity resolves to the same durable Session/Transcript owner.
    await app.close();
    app = null;
    app = await launchDesktopApp(appDataDir);
    page = await app.firstWindow();
    await completeBrowserLogin(page);
    await page.getByRole('region', { name: 'Agent list' }).getByRole('button', { name: 'Human Alice', exact: true }).click();
    await expect(page.getByRole('article').filter({ hasText: humanMessage }).last()).toBeVisible({ timeout: 10_000 });

    // Human find-in-chat is backed by the canonical Host transcript search,
    // not a renderer-only scan of whichever tail page happens to be mounted.
    await page.keyboard.press('Control+f');
    const findInChat = page.getByRole('textbox', { name: 'Find in chat' });
    await expect(findInChat).toBeVisible();
    await findInChat.fill('durable message');
    await expect(page.locator('.sand-chat-find').getByRole('status')).toHaveText('1/1');
    await findInChat.press('Enter');
    await expect(page.getByRole('article').filter({ hasText: humanMessage }).last()).toBeVisible();
    await page.getByRole('button', { name: 'Close find' }).click();

    // Explicit Human -> Agent continuation stays on the existing
    // Coordinator -> Host -> Runner path, but the trusted Host terminal
    // projection must land back in this same Human workspace.
    const assistantReply = '收到：请分析这个任务';
    await page.getByRole('button', { name: 'Ask Agent', exact: true }).click();
    await expectHermesAssistantTurn(page, assistantReply);
    await expect(page.getByText('Human', { exact: true })).toBeVisible();
    await expect(page.getByRole('textbox', { name: 'Prompt' })).toBeVisible();

    // A second restart proves the Agent-authored result is not renderer-only
    // state: it must be replayed from the durable Human Session transcript.
    await app.close();
    app = null;
    app = await launchDesktopApp(appDataDir);
    page = await app.firstWindow();
    await completeBrowserLogin(page);
    await page.getByRole('region', { name: 'Agent list' }).getByRole('button', { name: 'Human Alice', exact: true }).click();
    await expect(page.getByRole('article').filter({ hasText: humanMessage }).last()).toBeVisible({ timeout: 10_000 });
    await expectHermesAssistantTurn(page, assistantReply);
    await expect(page.getByText('Human', { exact: true })).toBeVisible();
  } finally {
    await app?.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
  }
});

test('self-hosted Bot invocation projects into the same Hermes-style turn without actor impersonation', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-selfhosted-bot-mahayana-'));
  let app: ElectronApplication | null = null;
  const prompt = '自建 Bot 请规划步骤，调用 Mahayana 工具并完成这个任务。';

  try {
    app = await launchDesktopApp(appDataDir);
    let page = await app.firstWindow();
    await completeBrowserLogin(page);
    const { conversationId, peerTestId } = await createSelfHostedBotAcceptanceChannel(page);

    // The authenticated human turn first goes through the canonical Rust messaging store.
    await page.getByTestId('messenger-input').fill(prompt);
    await page.getByTestId('messenger-send').click();
    await expect(page.getByRole('article').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 10_000 });

    // The Rust messaging service has a separate contract test proving that a
    // human message to a Bot produces this exact BotInvocationRequested event.
    // Here we verify the Electron consumer half: invocation -> Mahayana -> one
    // ordinary assistant turn instead of a second task-card surface.
    const invocationId = await emitBotInvocationRequested(page, conversationId, prompt);
    expect(invocationId).toContain('invocation:e2e:');

    await expectHermesAssistantTurn(page, '收到：自建 Bot 请规划步骤');
    await expect(page.getByTestId('agent-run')).toBeHidden();

    // This journal is only the idempotency/claim record for consuming the Bot
    // invocation. It is not accepted as transcript or run-state authority.
    await expect.poll(async () => page.evaluate(() => {
      const journal = JSON.parse(localStorage.getItem('fabushi.desktop.selfhosted-mahayana-invocations.v1') || 'null');
      return Object.values(journal?.claims || {}).some((claim: unknown) =>
        Boolean(claim && typeof claim === 'object' && (claim as { state?: string }).state === 'accepted'));
    })).toBe(true);

    await app.close();
    app = null;

    // Restart coverage is intentionally limited to the canonical messaging
    // history here. Ordered AssistantTurn replay must come from the production
    // Rust gateway/session store and is a separate MSR-204 acceptance blocker.
    app = await launchDesktopApp(appDataDir);
    page = await app.firstWindow();
    await completeBrowserLogin(page);
    const restoredPeer = page.getByTestId(peerTestId);
    await expect(restoredPeer).toBeVisible({ timeout: 15_000 });
    await restoredPeer.click();
    await expect(page.getByRole('article').filter({ hasText: prompt }).last()).toBeVisible({ timeout: 10_000 });
  } finally {
    await app?.close().catch(() => undefined);
    await rm(appDataDir, { recursive: true, force: true });
  }
});
