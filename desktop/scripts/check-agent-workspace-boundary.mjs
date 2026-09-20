import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, '..');
const shellPath = path.join(desktopRoot, 'src', 'messaging-shell-v2.tsx');
const shell = fs.readFileSync(shellPath, 'utf8');

const forbidden = [
  ['renderer-global Agent operation pointer', /\bagentOperationId\b/],
  ['renderer-global pending Agent send pointer', /\bpendingSend\b/],
  ['renderer-owned Agent delta buffer', /pendingAgentDeltaRef|agentDeltaFrameRef/],
  ['renderer-owned Agent operation claiming', /\bclaimAgentOperation\s*\(/],
  ['renderer-owned Agent operation clearing', /\bclearAgentOperation\s*\(/],
  ['renderer-owned AssistantTurn event reducer', /\bappendAssistantTurnEvent\s*\(/],
  ['Agent transcript copy-back through renderer messages', /setMessages\(toDisplayAgentMessages/],
];

const violations = forbidden
  .filter(([, pattern]) => pattern.test(shell))
  .map(([label]) => label);

if (!/new AgentRuntimeCoordinator\s*\(/.test(shell)) {
  violations.push('AgentRuntimeCoordinator is not mounted by the desktop Agent shell');
}
if (!/agentTranscriptStoreRef\.current\.thread\(activePeer\.key\)/.test(shell)) {
  violations.push('normal Agent rendering no longer reads directly from AgentTranscriptStore');
}
if (!/agentCoordinatorClient\.connect\s*\(/.test(shell)) {
  violations.push('Host transport lifecycle escaped AgentCoordinatorClient');
}

if (violations.length) {
  console.error('Agent workspace boundary regression detected:');
  for (const violation of violations) console.error(`- ${violation}`);
  process.exitCode = 1;
} else {
  console.log('Agent workspace boundary check passed.');
}
