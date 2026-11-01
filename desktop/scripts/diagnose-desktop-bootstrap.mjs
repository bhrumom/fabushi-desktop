import { _electron as electron } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const desktopRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const appData = await mkdtemp(path.join(tmpdir(), 'fabushi-bootstrap-'));
const output = { console: [], pageErrors: [], snapshots: [] };
let app;

const limited = (value, length = 2000) => String(value ?? '').slice(0, length);
const settle = (promise, timeoutMs = 2500) => Promise.race([
  Promise.resolve(promise).then((value) => ({ status: 'resolved', value })).catch((error) => ({ status: 'rejected', error: limited(error?.stack || error) })),
  new Promise((resolve) => setTimeout(() => resolve({ status: 'timeout' }), timeoutMs)),
]);

try {
  app = await electron.launch({
    args: [desktopRoot],
    env: {
      ...process.env,
      FABUSHI_APP_DATA: appData,
      FABUSHI_FEATURE_HOST_MODE: process.env.FABUSHI_FEATURE_HOST_MODE || 'test',
      FABUSHI_E2E: '1',
      MAHAYANA_APP_HOST_BIN: process.env.MAHAYANA_APP_HOST_BIN || '',
    },
  });
  const page = await app.firstWindow({ timeout: 15_000 });
  page.on('console', (message) => output.console.push(`${message.type()}: ${limited(message.text(), 1000)}`));
  page.on('pageerror', (error) => output.pageErrors.push(limited(error?.stack || error)));

  for (const waitMs of [0, 750, 2250, 5000]) {
    if (waitMs) await page.waitForTimeout(waitMs);
    const snapshot = await page.evaluate(async () => {
      const testids = Array.from(document.querySelectorAll('[data-testid]'))
        .map((node) => node.getAttribute('data-testid'))
        .filter(Boolean)
        .slice(0, 100);
      const shell = document.querySelector('[data-testid="desktop-shell"]');
      const bootstrap = document.querySelector('[data-testid="desktop-fast-start-bootstrap"]');
      return {
        href: location.href,
        readyState: document.readyState,
        title: document.title,
        bodyText: document.body?.innerText?.slice(0, 1200) || '',
        testids,
        rootLength: document.querySelector('#root')?.innerHTML?.length ?? -1,
        shellAttributes: shell ? Object.fromEntries(Array.from(shell.attributes).map((item) => [item.name, item.value])) : null,
        bootstrapVisible: Boolean(bootstrap),
        bridges: {
          mahayana: typeof window.mahayana?.invoke === 'function',
          native: typeof window.fabushiNative?.invoke === 'function',
        },
      };
    });
    output.snapshots.push({ afterMs: waitMs, ...snapshot });
  }

  output.nativePersistence = await page.evaluate(async () => {
    const settle = (promise, timeoutMs = 2500) => Promise.race([
      Promise.resolve(promise).then((value) => ({ status: 'resolved', value })).catch((error) => ({ status: 'rejected', error: String(error?.stack || error).slice(0, 1500) })),
      new Promise((resolve) => setTimeout(() => resolve({ status: 'timeout' }), timeoutMs)),
    ]);
    return typeof window.fabushiNative?.invoke === 'function'
      ? settle(window.fabushiNative.invoke('readClientPersistence', { key: 'fabushi.desktop.messenger.projection.v1' }))
      : { status: 'bridge-missing' };
  });
  output.authStatus = await page.evaluate(async () => {
    const settle = (promise, timeoutMs = 2500) => Promise.race([
      Promise.resolve(promise).then((value) => ({ status: 'resolved', value })).catch((error) => ({ status: 'rejected', error: String(error?.stack || error).slice(0, 1500) })),
      new Promise((resolve) => setTimeout(() => resolve({ status: 'timeout' }), timeoutMs)),
    ]);
    return typeof window.mahayana?.invoke === 'function'
      ? settle(window.mahayana.invoke('feature.auth.status', {}))
      : { status: 'bridge-missing' };
  });

  const phase = async () => page.evaluate(() => {
    if (document.querySelector('[data-testid="onboarding-gate"]')) return 'onboarding';
    if (document.querySelector('[data-testid="browser-login-start"]')) return 'login';
    const workspace = document.querySelector('[data-testid="messenger-workspace"]');
    if (workspace?.getAttribute('data-initial-host-hydrated') === 'true') return 'ready';
    if (workspace) return 'workspace-waiting';
    if (document.querySelector('[data-testid="login-gate"]')) return 'auth-resolving';
    return 'waiting';
  });

  output.loginPhases = [];
  for (let step = 0; step < 16; step += 1) {
    const current = await phase();
    output.loginPhases.push({ step, phase: current });
    if (current === 'onboarding') {
      await page.getByTestId('onboarding-next').click();
      await page.waitForTimeout(100);
      continue;
    }
    if (current === 'auth-resolving' || current === 'waiting') {
      await page.waitForTimeout(250);
      continue;
    }
    if (current === 'login') {
      await page.getByTestId('browser-login-start').click();
      await page.waitForTimeout(250);
      continue;
    }
    if (current === 'ready') break;
    if (current === 'workspace-waiting') {
      await page.waitForTimeout(250);
      continue;
    }
  }

  output.finalPhase = await phase();
  output.authStatusAfterLogin = await page.evaluate(async () => {
    const settle = (promise, timeoutMs = 2500) => Promise.race([
      Promise.resolve(promise).then((value) => ({ status: 'resolved', value })).catch((error) => ({ status: 'rejected', error: String(error?.stack || error).slice(0, 1500) })),
      new Promise((resolve) => setTimeout(() => resolve({ status: 'timeout' }), timeoutMs)),
    ]);
    return settle(window.mahayana.invoke('feature.auth.status', {}));
  });

  if (output.finalPhase !== 'ready') {
    output.runtimeProbe = await page.evaluate(async () => {
      const events = [];
      const unsubscribe = window.mahayana.subscribe?.((event) => events.push(event?.type || 'unknown')) || (() => {});
      const commands = ['conversation.list', 'bot.list', 'group.list'];
      const results = [];
      for (const type of commands) {
        try {
          results.push({ type, value: await window.mahayana.invoke('feature.execute', { command: { type, requestId: `bootstrap-diag-${type}-${Date.now()}` } }) });
        } catch (error) {
          results.push({ type, error: String(error?.stack || error).slice(0, 1000) });
        }
      }
      await new Promise((resolve) => setTimeout(resolve, 1500));
      unsubscribe();
      return { results, events };
    });
    await page.waitForTimeout(250);
    output.phaseAfterRuntimeProbe = await phase();
  }

  output.resolvedSurface = output.finalPhase === 'ready' || output.phaseAfterRuntimeProbe === 'ready';
  console.log(`FABUSHI_BOOTSTRAP_DIAGNOSTIC=${JSON.stringify(output)}`);
  if (!output.resolvedSurface) process.exitCode = 1;
} catch (error) {
  output.launchError = limited(error?.stack || error, 4000);
  console.log(`FABUSHI_BOOTSTRAP_DIAGNOSTIC=${JSON.stringify(output)}`);
  process.exitCode = 1;
} finally {
  if (app) await app.close().catch(() => {});
  await rm(appData, { recursive: true, force: true }).catch(() => {});
}
