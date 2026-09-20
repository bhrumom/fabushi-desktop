import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, '..');
const shellPath = path.join(desktopRoot, 'src', 'messaging-shell-v2.tsx');
const shell = fs.readFileSync(shellPath, 'utf8');
const networkControllerPath = path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-network-controller.ts');
const networkController = fs.readFileSync(networkControllerPath, 'utf8');
const workflowControllerPath = path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-workflow-controller.ts');
const workflowController = fs.readFileSync(workflowControllerPath, 'utf8');
const storeSyncControllerPath = path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-store-sync-controller.ts');
const storeSyncController = fs.readFileSync(storeSyncControllerPath, 'utf8');
const directoryControllerPath = path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-directory-controller.ts');
const directoryController = fs.readFileSync(directoryControllerPath, 'utf8');
const agentComposerPath = path.join(desktopRoot, 'src', 'agent-workspace', 'agent-composer.tsx');
const agentComposer = fs.readFileSync(agentComposerPath, 'utf8');
const agentRichEditorPath = path.join(desktopRoot, 'src', 'agent-workspace', 'agent-rich-text-editor.tsx');
const agentRichEditor = fs.readFileSync(agentRichEditorPath, 'utf8');

const forbidden = [
  ['renderer-global Agent operation pointer', /\bagentOperationId\b/],
  ['renderer-global pending Agent send pointer', /\bpendingSend\b/],
  ['renderer-owned queued Agent transcript', /\bqueuedAgentPrompts\b/],
  ['renderer-owned Agent delta buffer', /pendingAgentDeltaRef|agentDeltaFrameRef/],
  ['renderer-owned Agent operation claiming', /\bclaimAgentOperation\s*\(/],
  ['renderer-owned Agent operation clearing', /\bclearAgentOperation\s*\(/],
  ['renderer-owned AssistantTurn event reducer', /\bappendAssistantTurnEvent\s*\(/],
  ['renderer-owned Agent submission queue', /\bagentSubmissionQueue\b|submissionQueue\s*:/],
  ['renderer-owned Agent transport dispatch', /\bdispatchAgentPromptNow\b/],
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
if (/from\s+['"]\.\.\/grok-shell\//.test(agentComposer)) {
  violations.push('primary Agent Composer implementation still depends on the Grok compatibility shell');
}
if (/composerValue=\{composer\}/.test(shell) || /onComposerChange=\{updateComposer\}/.test(shell)) {
  violations.push('primary Agent Composer leaked back into renderer-global Messenger composer state');
}
if (/onDraftRestored\s*:\s*\([^)]*\)\s*=>\s*\{[^}]*setComposer/s.test(shell)) {
  violations.push('failed Agent submissions copy restored drafts back into Messenger composer state');
}
if (!/composerValue=\{agentWorkspaceController\.draftForPeer\(activePeer\.key\)\}/.test(shell)
  || !/onComposerSubmit=\{\(event\) => sendAgentMessage\(event, activePeer\)\}/.test(shell)) {
  violations.push('primary Agent Composer is not bound directly to AgentWorkspaceController');
}
if (!/composerRichText=\{agentWorkspaceController\.richTextForPeer\(activePeer\.key\)\}/.test(shell)) {
  violations.push('primary Agent Composer is not bound to Agent rich-text draft state');
}
if (!/from\s+['"]@tiptap\/react['"]/.test(agentRichEditor)
  || !/useEditor\s*\(/.test(agentRichEditor)
  || !/commands\.setContent\(expected,\s*\{\s*emitUpdate:\s*false\s*\}\)/.test(agentRichEditor)) {
  violations.push('Agent rich editor no longer uses the Fabu-compatible TipTap document boundary');
}
if (/contentEditable=/.test(agentComposer) || /innerText\s*=/.test(agentComposer)) {
  violations.push('primary Agent Composer regressed to a hand-managed contentEditable surface');
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
if (!/submit:\s*submitAgentWorkspace/.test(shell) || !/submitAgentWorkspace\s*\(\s*\{/.test(shell)) {
  violations.push('primary Agent send path is not routed through Agent workspace runtime submit');
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

if (!/useAgentCommandPaletteController\s*\(/.test(shell)) {
  violations.push('command palette lifecycle escaped the Agent-owned controller hook');
}
if (/setGrokPalette|agentPaletteOpen|agentPaletteQuery/.test(shell)) {
  violations.push('primary shell recreated command palette runtime state');
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

if (!/useAgentWorkflowController\s*\(/.test(shell)) {
  violations.push('Agent workflow discovery escaped the Agent workspace controller');
}
if (/agentWorkflowsById|setAgentWorkflowsById|type:\s*['"]workflow\.list['"]/.test(shell)) {
  violations.push('primary shell recreated Agent workflow cache or raw workflow.list ownership');
}
if (!/client\.listWorkflows\s*\(/.test(workflowController)
  || !/event\.type === ['"]workflow\.listed['"]/.test(workflowController)
  || !/event\.type === ['"]workflow\.changed['"]/.test(workflowController)) {
  violations.push('Agent workflow controller no longer owns workflow list/cache refresh');
}

if (!/useAgentStoreSyncController\s*\(/.test(shell)) {
  violations.push('Agent memory/automation CAS sync escaped the Agent workspace controller');
}
if (/type:\s*['"]memory\.list['"]|case\s+['"]memory\.(?:changed|listed)['"]|case\s+['"]automation\.(?:changed|listed)['"]/.test(shell)) {
  violations.push('primary shell recreated Agent memory/automation runtime sync ownership');
}
if (!/client\.listMemory\s*\(/.test(storeSyncController)
  || !/event\.type === ['"]memory\.changed['"]/.test(storeSyncController)
  || !/event\.type === ['"]memory\.listed['"]/.test(storeSyncController)
  || !/event\.type === ['"]automation\.changed['"]/.test(storeSyncController)
  || !/event\.type === ['"]automation\.listed['"]/.test(storeSyncController)) {
  violations.push('Agent store sync controller no longer owns memory/automation synchronization');
}

if (!/useAgentDirectoryController\s*\(/.test(shell)) {
  violations.push('Agent directory cache/commands escaped the Agent workspace controller');
}
if (/type:\s*['"]bot\.(?:list|create|update|clone|delete|setHidden)['"]|case\s+['"]bot\.(?:listed|changed)['"]|setBots\s*\(/.test(shell)) {
  violations.push('primary shell recreated raw Bot/Agent directory ownership');
}
if (/agentCoordinatorClient\.(?:listAgents|createAgent|updateAgent|duplicateAgent|deleteAgent|setAgentHidden)\s*\(/.test(shell)) {
  violations.push('primary shell bypassed AgentDirectoryController');
}
for (const method of ['listAgents', 'createAgent', 'updateAgent', 'duplicateAgent', 'deleteAgent', 'setAgentHidden']) {
  if (!new RegExp(`client\\.${method}\\s*\\(`).test(directoryController)) {
    violations.push(`Agent directory controller no longer routes ${method} through AgentCoordinatorClient`);
  }
}
if (!/event\.type === ['"]bot\.listed['"]/.test(directoryController)
  || !/event\.type === ['"]bot\.changed['"]/.test(directoryController)) {
  violations.push('Agent directory controller no longer owns Host directory event projection');
}

if (violations.length) {
  console.error('Agent workspace boundary regression detected:');
  for (const violation of violations) console.error(`- ${violation}`);
  process.exitCode = 1;
} else {
  console.log('Agent workspace boundary check passed.');
}
