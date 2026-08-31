import { _electron as electron, expect, test } from '@playwright/test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagedExecutable = process.env.FABUSHI_ELECTRON_EXECUTABLE?.trim() || null;

async function launchDesktopApp(appDataDir: string) {
  return electron.launch({
    ...(packagedExecutable ? { executablePath: packagedExecutable, args: [] } : { args: [appRoot] }),
    env: {
      ...process.env,
      FABUSHI_APP_DATA: appDataDir,
      FABUSHI_FEATURE_HOST_MODE: process.env.FABUSHI_FEATURE_HOST_MODE || 'test',
      MAHAYANA_APP_HOST_BIN: process.env.MAHAYANA_APP_HOST_BIN || '',
    },
  });
}

test('Grok parity motion layer exposes distinct state choreography', async () => {
  const appDataDir = await mkdtemp(path.join(tmpdir(), 'fabushi-grok-motion-'));
  const app = await launchDesktopApp(appDataDir);

  try {
    const page = await app.firstWindow();
    await expect.poll(async () => page.evaluate(() =>
      Array.from(document.styleSheets).some((candidate) =>
        String(candidate.href ?? '').includes('grok-motion-parity.css'))),
    { timeout: 10_000 }).toBe(true);
    const contract = await page.evaluate(() => {
      const sheet = Array.from(document.styleSheets).find((candidate) =>
        String(candidate.href ?? '').includes('grok-motion-parity.css'));
      const cssText = sheet ? Array.from(sheet.cssRules).map((rule) => rule.cssText).join('\n') : '';

      const wrapper = document.createElement('span');
      wrapper.dataset.engine = 'fabushi-motion-v3';
      wrapper.dataset.agentState = 'thinking';
      const aura = document.createElement('span');
      aura.className = '_botMarkAura_contract_';
      wrapper.append(aura);
      document.body.append(wrapper);

      const animationFor = (state: string) => {
        wrapper.dataset.agentState = state;
        return getComputedStyle(aura).animationName;
      };
      const animations = {
        thinking: animationFor('thinking'),
        searching: animationFor('searching'),
        working: animationFor('working'),
        toolRunning: animationFor('tool-running'),
        speaking: animationFor('speaking'),
        result: animationFor('result'),
        error: animationFor('error'),
      };
      wrapper.remove();
      return {
        loaded: Boolean(sheet),
        cssText,
        animations,
      };
    });

    expect(contract.loaded).toBe(true);
    expect(contract.cssText).toContain('gbfGrokThinkingAura');
    expect(contract.cssText).toContain('gbfGrokSearchAura');
    expect(contract.cssText).toContain('gbfGrokWorkAura');
    expect(contract.cssText).toContain('gbfGrokSpeakingAura');
    expect(contract.cssText).toContain('gbfGrokResultAura');
    expect(contract.cssText).toContain('gbfGrokAlertAura');
    expect(contract.cssText).toContain('prefers-reduced-motion');

    expect(contract.animations.thinking).toContain('gbfGrokThinkingAura');
    expect(contract.animations.searching).toContain('gbfGrokSearchAura');
    expect(contract.animations.working).toContain('gbfGrokWorkAura');
    expect(contract.animations.toolRunning).toContain('gbfGrokWorkAura');
    expect(contract.animations.speaking).toContain('gbfGrokSpeakingAura');
    expect(contract.animations.result).toContain('gbfGrokResultAura');
    expect(contract.animations.error).toContain('gbfGrokAlertAura');
  } finally {
    await app.close();
    await rm(appDataDir, { recursive: true, force: true });
  }
});
