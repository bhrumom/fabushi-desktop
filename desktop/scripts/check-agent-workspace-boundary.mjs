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
  ['renderer-owned queued Agent transcript', /\bqueuedAgentPrompts\b/],
  ['renderer-owned Agent delta buffer', /pendingAgentDeltaRef|agentDeltaFrameRef/],
  ['renderer-owned Agent operation claiming', /\bclaimAgentOperation\s*\(/],
  ['renderer-owned Agent operation clearing', /\bclearAgentOperation\s*\(/],
  ['renderer-owned AssistantTurn event reducer', /\bappendAssistantTurnEvent\s*\(/],
  ['Agent transcript copy-back through renderer messages', /setMessages\(toDisplayAgentMessages/],
];

const violations = forbidden
  .filter(([, pattern]) => pattern.test(shell))
  .map(([label]) => label);

if (!/useAgentWorkspaceRuntime\s*\(/.test(shell)) {
  violations.push('Agent workspace runtime facade is not mounted by the desktop Agent shell');
}
if (/new AgentRuntimeCoordinator\s*\(/.test(shell)) {
  violations.push('Messenger shell recreated AgentRuntimeCoordinator ownership');
}
if (/createAgentSubmissionQueue\s*\(/.test(shell)) {
  violations.push('Messenger shell recreated Agent submission-queue ownership');
}
if (/readAgentWorkspaceDrafts\s*\(|persistAgentWorkspaceDrafts\s*\(/.test(shell)) {
  violations.push('Messenger shell recreated Agent draft persistence ownership');
}
if (!/agentTranscriptStore\.(?:thread|entries)\(activePeer\.key\)/.test(shell)) {
  violations.push('normal Agent rendering no longer reads directly from AgentTranscriptStore');
}
if (/toDisplayAgentMessages\(agentTranscriptStore\.(?:thread|entries)/.test(shell)) {
  violations.push('normal Agent rendering reintroduced the Messenger DisplayMessage bridge');
}
const regenerateStart = shell.indexOf('function regenerateBotMessage');
const regenerateEnd = shell.indexOf('async function stopAgentOperation', regenerateStart);
const regenerateSlice = regenerateStart >= 0 && regenerateEnd > regenerateStart
  ? shell.slice(regenerateStart, regenerateEnd)
  : '';
if (!regenerateSlice.includes('agentTranscriptStore.entries(activePeer.key)')) {
  violations.push('Agent regenerate action no longer resolves its prompt from canonical Agent transcript');
}
if (/\bmessages\.(?:findIndex|slice)\b/.test(regenerateSlice)) {
  violations.push('Agent regenerate action fell back to renderer-global Messenger messages');
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
