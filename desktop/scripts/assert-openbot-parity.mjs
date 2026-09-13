import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join, resolve } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const desktopRoot = resolve(here, '..');
const repoRoot = resolve(desktopRoot, '..');

const read = (path) => readFileSync(path, 'utf8');

const main = read(join(desktopRoot, 'src/main.tsx'));
const css = read(join(desktopRoot, 'src/openbot-ui-parity.css'));
const shell = read(join(desktopRoot, 'src/messaging-shell-v2.tsx'));
const botMark = read(join(repoRoot, 'frontend/apps/web/src/app/host/bot-mark.tsx'));

assert.match(main, /import ['"]\.\/openbot-ui-parity\.css['"];?/u, 'desktop bootstrap must activate OpenBot parity CSS');
assert.ok(
  main.indexOf("./openbot-ui-parity.css") > main.indexOf("./grok-agent-ui-parity.css"),
  'OpenBot parity layer must load after the older Grok skin so it is authoritative',
);

for (const token of [
  "[data-testid='messenger-workspace']",
  "button[data-testid^='peer-']",
  "[class*='chatWorkspace']",
  "[class*='messageThinking']",
  "[class*='messageAction']",
  "[data-testid='messenger-input']",
  "[data-testid='messenger-send']",
]) {
  assert.ok(css.includes(token), `OpenBot CSS is missing required surface selector: ${token}`);
}

for (const shape of ['blob', 'pebble', 'squircle', 'tablet', 'wedge', 'hex', 'cloud', 'teardrop']) {
  assert.ok(botMark.includes(`"${shape}"`), `BotMark must retain OpenBot-style identity silhouette ${shape}`);
}
assert.ok(
  botMark.includes('const SHAPES: readonly BotMarkShape[]'),
  'BotMark must expose deterministic silhouette roster',
);
assert.match(botMark, /export function botMarkShape\(botId: string\)/u, 'BotMark shape must be identity-derived');
assert.doesNotMatch(
  botMark,
  /export function botMarkShape\(_botId: string\): BotMarkShape \{\s*return "blob";/u,
  'BotMark must not collapse every coworker to the same blob silhouette',
);
assert.match(botMark, /canonicalBotIdentity\(botId\)/u, 'shape selection must use canonical identity');

assert.match(shell, /kind\?: 'message' \| 'action' \| 'thinking'/u, 'messenger must retain thinking/action/result transcript model');
assert.match(shell, /operationId\?: string/u, 'reply work must correlate lifecycle rows by operation id');
assert.match(shell, /message\.kind === 'thinking'/u, 'messenger must project active thinking state');
assert.match(shell, /message\.kind === 'action'/u, 'messenger must project tool/action work');
assert.match(shell, /event\.role === 'assistant'/u, 'assistant completion must close temporary reply-work state');

console.log('OpenBot UI/reply parity contract: PASS');
