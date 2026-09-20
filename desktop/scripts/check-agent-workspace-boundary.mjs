import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, '..');
const shellPath = path.join(desktopRoot, 'src', 'messaging-shell-v2.tsx');
const shell = fs.readFileSync(shellPath, 'utf8');
const networkControllerPath = path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-network-controller.ts');
const networkController = fs.readFileSync(networkControllerPath, 'utf8');

const forbidden = [
  ['renderer-global Agent operation pointer', /\bagentOperationId\b/],
  ['renderer-global pending Agent send pointer', /\bpendingSend\b/],
  ['renderer-owned queued Agent transcript', /\bqueuedAgentPrompts\b/],
  ['renderer-owned Agent delta buffer', /pendingAgentDeltaRef|agentDeltaFrameRef/],
  ['renderer-owned Agent operation claiming', /\bclaimAgentOperation\s*\(/],
  ['renderer-owned Agent operation clearing', /\bclearAgentOperation\s*\(/],
  ['renderer-owned AssistantTurn event reducer', /\bappendAssistantTurnEvent\s*\(/],
  ['Agent transcript copy-back through renderer messages', /setMessages\(toDisplayAgentMessages/],
  ['legacy Grok Agent Network mounted by primary shell', /import\s+GrokAgentNetwork\s+from\s+['"]\.\/grok-shell\/grok-agent-network['"]/],
  ['legacy Grok Command Palette mounted by primary shell', /import\s+GrokCommandPalette\s+from\s+['"]\.\/grok-shell\/grok-command-palette['"]/],
  ['primary Agent shell directly imports Grok implementation layers', /from\s+['"]\.\/grok-(?:shell|runtime)\//],
  ['Grok-named runtime state leaked back into primary Agent shell', /\bgrok(?:Palette|Network|Pinned|Sidebar|Selected|Activity|Busy|Agent)[A-Z]\w*/],
];

const violations = forbidden
  .filter(([, pattern]) => pattern.test(shell))
  .map(([label]) => label);

if (!/useAgentWorkspaceRuntime\s*\(/.test(shell)) {
  violations.push('Agent workspace runtime facade is not mounted by the desktop Agent shell');
}
if (!/useAgentSidebarController\s*\(/.test(shell)) {
  violations.push('Agent sidebar controller is not mounted by the desktop Agent shell');
}
if (/readAgentSidebarSections(?:Durable)?\s*\(|persistAgentSidebarSections\s*\(|setGrokPinnedOrder\s*\(|setGrokSidebarSections\s*\(|setGrokSelectedAgentKeys\s*\(/.test(shell)) {
  violations.push('Messenger shell recreated Agent sidebar state or persistence ownership');
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
if (!regenerateSlice.includes('agentTranscriptStore.userPromptBefore(activePeer.key, message.id)')) {
  violations.push('Agent regenerate action no longer resolves its prompt from canonical Agent transcript');
}
if (/\bmessages\.(?:findIndex|slice)\b/.test(regenerateSlice)) {
  violations.push('Agent regenerate action fell back to renderer-global Messenger messages');
}
if (!/agentCoordinatorClient\.connect\s*\(/.test(shell)) {
  violations.push('Host transport lifecycle escaped AgentCoordinatorClient');
}

if (!/import\s+AgentNetwork\s+from\s+['"]\.\/agent-workspace\/agent-network['"]/.test(shell)
  || !/<AgentNetwork\b/.test(shell)) {
  violations.push('primary shell is not mounting the Agent-owned Network surface');
}

if (!/import\s+AgentCommandPalette\s+from\s+['"]\.\/agent-workspace\/agent-command-palette['"]/.test(shell)
  || !/<AgentCommandPalette\b/.test(shell)) {
  violations.push('primary shell is not mounting the Agent-owned command palette boundary');
}

if (!/from\s+['"]\.\/agent-workspace\/agent-model['"]/.test(shell)
  || !/projectAgentSidebarItems\s*\(/.test(shell)
  || !/projectActiveAgentKey\s*\(/.test(shell)) {
  violations.push('primary shell is not consuming the Agent-owned navigation projection model');
}
if (!/useAgentNetworkController\s*\(/.test(shell)) {
  violations.push('Agent Network UI/controller state escaped the Agent workspace controller hook');
}
if (/agentCoordinatorClient\.(?:listGroups|createGroup|updateGroup|deleteGroup|sendGroup|broadcast)\s*\(/.test(shell)) {
  violations.push('primary shell directly owns Agent collaboration commands');
}
for (const method of ['listGroups', 'createGroup', 'updateGroup', 'deleteGroup', 'sendGroup', 'broadcast']) {
  if (!new RegExp(`client\\.${method}\\s*\\(`).test(networkController)) {
    violations.push(`Agent Network controller no longer routes ${method} through AgentCoordinatorClient`);
  }
}

if (violations.length) {
  console.error('Agent workspace boundary regression detected:');
  for (const violation of violations) console.error(`- ${violation}`);
  process.exitCode = 1;
} else {
  console.log('Agent workspace boundary check passed.');
}
