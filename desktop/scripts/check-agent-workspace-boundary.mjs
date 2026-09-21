import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, '..');
const repoRoot = path.resolve(desktopRoot, '..');
const violations = [];
// This checker is part of the exact-PR-head gate; merge-ref success is supplemental only.
// Signed candidates are permitted only for trusted same-repository PR heads.

const read = (...parts) => fs.readFileSync(path.join(...parts), 'utf8');
const exists = (...parts) => fs.existsSync(path.join(...parts));

function walk(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === 'node_modules' || entry.name === 'dist' || entry.name === 'release') continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else out.push(full);
  }
  return out;
}

function requirePattern(label, content, pattern) {
  if (!pattern.test(content)) violations.push(label);
}

function forbidPattern(label, content, pattern) {
  if (pattern.test(content)) violations.push(label);
}

function requireOrdered(label, content, markers) {
  let cursor = 0;
  for (const marker of markers) {
    const next = content.indexOf(marker, cursor);
    if (next < 0) {
      violations.push(`${label}: missing or out of order: ${marker}`);
      return;
    }
    cursor = next + marker.length;
  }
}

const desktopApp = read(desktopRoot, 'src', 'app', 'DesktopApp.tsx');
const desktopAuthBoundary = read(desktopRoot, 'src', 'app', 'desktop-auth-boundary.tsx');
const rootShell = read(desktopRoot, 'src', 'agent-workspace', 'agent-root-shell.tsx');
const rootCss = read(desktopRoot, 'src', 'agent-workspace', 'agent-root-shell.module.css');
const productControllers = read(desktopRoot, 'src', 'agent-workspace', 'use-agent-product-controllers.ts');
const runtimeFacade = read(desktopRoot, 'src', 'agent-workspace', 'use-agent-workspace-runtime.ts');
const runtimeCoordinator = read(desktopRoot, 'src', 'agent-workspace', 'agent-runtime-coordinator.ts');
const directoryController = read(desktopRoot, 'src', 'agent-workspace', 'use-agent-directory-controller.ts');
const networkController = read(desktopRoot, 'src', 'agent-workspace', 'use-agent-network-controller.ts');
const computerController = read(desktopRoot, 'src', 'agent-workspace', 'use-agent-computer-controller.ts');
const sidebarController = read(desktopRoot, 'src', 'agent-workspace', 'use-agent-sidebar-controller.ts');
const sidebarState = read(desktopRoot, 'src', 'agent-workspace', 'agent-sidebar-state.ts');
const agentTranscript = read(desktopRoot, 'src', 'agent-workspace', 'agent-transcript.tsx');
const agentTranscriptStore = read(desktopRoot, 'src', 'agent-workspace', 'agent-transcript-store.ts');
const conversationIdentity = read(desktopRoot, 'src', 'agent-workspace', 'conversation-identity.ts');
const agentHeader = read(desktopRoot, 'src', 'grok-shell', 'grok-agent-header.tsx');
const agentSidebar = read(desktopRoot, 'src', 'grok-shell', 'grok-agent-sidebar.tsx');
const agentSidebarBoundary = read(desktopRoot, 'src', 'agent-workspace', 'agent-sidebar.tsx');
const agentOverlays = read(desktopRoot, 'src', 'agent-workspace', 'agent-overlays.tsx');
const agentComposer = read(desktopRoot, 'src', 'agent-workspace', 'agent-composer.tsx');
const agentSearch = read(desktopRoot, 'src', 'agent-workspace', 'agent-search.tsx');
const agentNetwork = read(desktopRoot, 'src', 'agent-workspace', 'agent-network.tsx');
const agentSettingsPanel = read(desktopRoot, 'src', 'agent-workspace', 'agent-settings-panel.tsx');
const fabAvatar = read(desktopRoot, 'src', 'ui', 'avatar', 'fab-avatar.tsx');
const primitives = read(desktopRoot, 'src', 'ui', 'primitives', 'fab-primitives.tsx');
const tokens = read(desktopRoot, 'src', 'ui', 'tokens.css');
const electronMain = read(desktopRoot, 'electron', 'main.cjs');
const hostProcess = read(desktopRoot, 'electron', 'host-process.cjs');
const mahayanaEdge = read(desktopRoot, 'electron', 'mahayana-edge.cjs');
const electronTransport = read(repoRoot, 'frontend', 'apps', 'web', 'src', 'lib', 'mahayana-host', 'electron-transport.ts');
const remoteSupervisor = read(desktopRoot, 'electron', 'remote-device-agent-supervisor.cjs');

const runtimeRoot = path.join(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-runtime', 'src');
const runtimeLib = read(runtimeRoot, 'lib.rs');
const actor = read(runtimeRoot, 'conversation_actor.rs');
const broker = read(runtimeRoot, 'capability_broker.rs');
const runtimeStore = read(runtimeRoot, 'runtime_store.rs');
const featureHost = read(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-feature-host', 'src', 'implementation.rs');
const computerRuntime = read(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-computer', 'src', 'lib.rs');
const codexAgentBackend = read(
  repoRoot,
  'third_party',
  'mahayana',
  'mahayana-rs',
  'mahayana-agent-codex',
  'src',
  'implementation.rs',
);
const codexSendStart = codexAgentBackend.indexOf('async fn send_message(');
const codexSendEnd = codexSendStart >= 0
  ? codexAgentBackend.indexOf('async fn interrupt(', codexSendStart)
  : -1;
const codexSendMessage = codexSendStart >= 0 && codexSendEnd > codexSendStart
  ? codexAgentBackend.slice(codexSendStart, codexSendEnd)
  : '';

requirePattern(
  'Codex Agent backend must expose the pending-turn match used for pre-response app-server events',
  codexAgentBackend,
  /fn operation_turn_matches\([\s\S]{0,500}bound_turn_id\.is_empty\(\)/,
);
requireOrdered(
  'Codex Agent backend must register operation ownership before TurnStart can emit fast completion events',
  codexSendMessage,
  [
    'let (completion, result) = oneshot::channel();',
    '.operations',
    '.insert(',
    'turn_id: String::new()',
    'request_typed(ClientRequest::TurnStart',
    'operation.turn_id = turn_id',
    'result',
    '.await',
  ],
);

const legacyShell = path.join(desktopRoot, 'src', 'adapters', 'legacy-messaging', 'legacy-messaging-shell.tsx');
const obsoleteShell = path.join(desktopRoot, 'src', 'messaging-shell-v2.tsx');
const legacyMotionCss = path.join(desktopRoot, 'public', 'grok-motion-parity.css');
if (fs.existsSync(legacyShell)) violations.push('legacy-messaging-shell.tsx still exists; the cutover must delete it instead of renaming it');
if (fs.existsSync(obsoleteShell)) violations.push('messaging-shell-v2.tsx returned');
if (fs.existsSync(legacyMotionCss)) violations.push('legacy grok-motion-parity.css returned');

if (rootShell.split('\n').length > 1200) {
  violations.push('AgentRootShell regressed into a god shell (>1200 lines)');
}
forbidPattern('AgentRootShell must not own durable state in localStorage', rootShell, /\blocalStorage\b/);
forbidPattern('AgentRootShell must not use browser-native prompt/confirm dialogs', rootShell, /window\.(?:prompt|confirm)\s*\(/);
forbidPattern('Agent Sidebar boundary must use FabDialog instead of browser-native prompt/confirm', agentSidebarBoundary, /window\.(?:prompt|confirm)\s*\(/);
requirePattern('Agent Sidebar boundary must own FabDialog management flows', agentSidebarBoundary, /\bFabDialog\b/);
forbidPattern(
  'Conversation identity must not infer Telegram from string prefixes',
  conversationIdentity,
  /startsWith\(['"]telegram:/,
);
requirePattern(
  'Conversation identity must switch on an explicit typed domain kind',
  conversationIdentity,
  /switch\s*\(peer\.kind\)[\s\S]{0,1800}case ['"]telegram['"][\s\S]{0,800}case ['"]miniapp-bot['"]/,
);
requirePattern('AgentRootShell must consume the shared FabIconButton primitive', rootShell, /\bFabIconButton\b/);
requirePattern(
  'DesktopApp must boot through DesktopAuthBoundary directly into AgentRootShell',
  desktopApp,
  /<DesktopAuthBoundary>[\s\S]*<AgentRootShell\s+transport=\{transport\}\s+onLogout=\{onLogout\}\s*\/>[\s\S]*<\/DesktopAuthBoundary>/,
);
forbidPattern('DesktopApp must not import any legacy messaging shell', desktopApp, /legacy-messaging|messaging-shell-v2/);
forbidPattern('Desktop auth boundary must not import the legacy HostClient product shell', desktopAuthBoundary, /frontend\/apps\/web\/src\/app\/host\/host-client|\bHostClient\b/);
requirePattern('Desktop auth boundary must use low-power FabAvatar', desktopAuthBoundary, /\bFabAvatar\b/);
requirePattern('Desktop auth boundary must expose a dedicated login gate', desktopAuthBoundary, /data-testid=[\"']login-gate[\"']/);

for (const required of [
  ['AgentSidebar', /<AgentSidebar\b/],
  ['AgentWorkspace', /<AgentWorkspace\b/],
  ['AgentNetwork', /<AgentNetwork\b/],
  ['AgentCommandPalette', /<AgentCommandPalette\b/],
  ['AgentOverlays', /<AgentOverlays\b/],
  ['useAgentProductControllers', /useAgentProductControllers\s*\(/],
  ['useAgentWorkspaceRuntime', /useAgentWorkspaceRuntime\s*\(/],
  ['useAgentComputerController', /useAgentComputerController\s*\(/],
  ['useAgentSettingsController', /useAgentSettingsController\s*\(/],
]) requirePattern(`AgentRootShell no longer owns ${required[0]}`, rootShell, required[1]);

forbidPattern(
  'AgentRootShell rebuilt raw bot command/event ownership instead of using AgentDirectoryController',
  rootShell,
  /type:\s*['"]bot\.(?:list|create|update|clone|delete|setHidden)['"]|case\s+['"]bot\.(?:listed|changed)['"]/,
);
forbidPattern(
  'AgentRootShell bypasses Agent runtime facades for normal Agent operations',
  rootShell,
  /coordinatorClient\.(?:send|interrupt|uploadAttachment|resolveApproval|listAgents|createAgent|updateAgent|duplicateAgent|deleteAgent|setAgentHidden)\s*\(/,
);
forbidPattern(
  'AgentRootShell re-imported Messenger compatibility peer construction',
  rootShell,
  /messenger-compatibility-adapter|buildCompatibilityPeers|compatibilityMessagingEnvelope/,
);
requirePattern('Agent product controllers no longer compose directory ownership', productControllers, /useAgentDirectoryController\s*\(/);
requirePattern('Agent product controllers no longer compose network ownership', productControllers, /useAgentNetworkController\s*\(/);
requirePattern('Agent product controllers must inject the coordinator into Sidebar persistence', productControllers, /useAgentSidebarController\s*\(options\.client,\s*options\.accountScope\)/);
requirePattern('AgentRootShell must route Sidebar RuntimeStore events', rootShell, /product\.sidebar\.handle\(event\)/);
requirePattern('Agent Sidebar local durability must read Mahayana RuntimeStore workspace state', sidebarController, /client\.readWorkspaceState\s*\(/);
requirePattern('Agent Sidebar local durability must write Mahayana RuntimeStore workspace state', sidebarController, /client\.writeWorkspaceState\s*\(/);
for (const [name, source] of [
  ['Agent Sidebar controller', sidebarController],
  ['Agent Sidebar state', sidebarState],
]) {
  forbidPattern(`${name} must not write business state to localStorage`, source, /localStorage\.(?:setItem|removeItem)\s*\(/);
  forbidPattern(`${name} must not write renderer/native client persistence`, source, /(?:writeClientPersistence|removeClientPersistence)/);
}
requirePattern('Agent runtime facade no longer owns queued submission lifecycle', runtimeFacade, /createAgentSubmissionQueue\s*\(/);
requirePattern('Agent directory no longer owns bot.listed projection', directoryController, /event\.type === ['"]bot\.listed['"]/);
requirePattern('Agent directory no longer owns bot.changed projection', directoryController, /event\.type === ['"]bot\.changed['"]/);
requirePattern('Agent network no longer routes direct handoff through coordinator', networkController, /client\.sendAgentPeer\s*\(/);
requirePattern('Agent network no longer routes broadcast through coordinator', networkController, /client\.broadcast\s*\(/);

const compatibilityAdapters = [
  ['contacts', path.join(desktopRoot, 'src', 'features', 'contacts', 'contacts-compatibility-adapter.tsx')],
  ['telegram', path.join(desktopRoot, 'src', 'features', 'telegram', 'telegram-compatibility-adapter.tsx')],
  ['miniapps', path.join(desktopRoot, 'src', 'features', 'miniapps', 'miniapp-compatibility-adapter.tsx')],
  ['payments', path.join(desktopRoot, 'src', 'features', 'payments', 'payments-compatibility-adapter.tsx')],
  ['calls', path.join(desktopRoot, 'src', 'features', 'calls', 'calls-compatibility-adapter.tsx')],
  ['settings', path.join(desktopRoot, 'src', 'features', 'settings', 'settings-compatibility-adapter.tsx')],
];
for (const [name, file] of compatibilityAdapters) {
  if (!fs.existsSync(file)) {
    violations.push(`${name} compatibility adapter is missing`);
    continue;
  }
  const body = fs.readFileSync(file, 'utf8');
  if (/Agent(?:Sidebar|Header|Network|Workspace)|useAgent(?:WorkspaceRuntime|ProductControllers|ComputerController|SidebarController|DirectoryController)|AgentRuntimeCoordinator/.test(body)) {
    violations.push(`${name} compatibility adapter illegally owns or renders Agent runtime state`);
  }
  if (name === 'settings' && /<input\b[^>]*role=['"]switch['"]/.test(body)) {
    violations.push('settings compatibility adapter bypasses FabSwitch');
  }
}

const desktopSourceFiles = walk(path.join(desktopRoot, 'src')).filter((file) => /\.(?:ts|tsx|js|mjs|cjs)$/.test(file));
for (const file of desktopSourceFiles) {
  const source = fs.readFileSync(file, 'utf8');
  const relative = path.relative(desktopRoot, file);
  if (/\bBotMark\b|FabushiAvatarRuntime|fabushi-avatar-runtime|fabushi-bot-mark-engine|fabushi-motion-v3/.test(source)) {
    violations.push(`${relative} reintroduced the old desktop avatar runtime`);
  }
}
for (const [name, source] of [
  ['Agent sidebar', agentSidebar],
  ['Agent header', agentHeader],
  ['Agent transcript', agentTranscript],
  ['Agent overlays', agentOverlays],
]) {
  requirePattern(`${name} does not use low-power FabAvatar`, source, /\bFabAvatar\b/);
}
forbidPattern('FabAvatar must not own requestAnimationFrame', fabAvatar, /requestAnimationFrame\s*\(/);
forbidPattern('FabAvatar must not infer business state through MutationObserver', fabAvatar, /MutationObserver|closest\s*\(/);
requirePattern('FabAvatar must expose stable identity marker', fabAvatar, /data-fab-avatar=['"]true['"]/);

for (const primitive of [
  'FabButton',
  'FabIconButton',
  'FabMenu',
  'FabPopover',
  'FabDialog',
  'FabTooltip',
  'FabSelect',
  'FabBadge',
  'FabAvatar',
  'FabSpinner',
  'FabInput',
  'FabSurface',
  'FabSwitch',
]) requirePattern(`Fabushi UI primitive missing: ${primitive}`, primitives, new RegExp(`(?:function|const|\\{)\\s*${primitive}\\b|export\\s+\\{[^}]*\\b${primitive}\\b`));
for (const token of ['--fab-bg-primary', '--fab-bg-raised', '--fab-border-subtle', '--fab-text-primary', '--fab-text-muted', '--fab-radius-sm', '--fab-radius-md', '--fab-space-1']) {
  if (!tokens.includes(token)) violations.push(`Fabushi design token missing: ${token}`);
}
requirePattern('Agent root layout no longer exposes the three-column Agent grid', rootCss, /grid-template-columns:[\s\S]*minmax\(420px,\s*1fr\)/);

const primaryAgentUi = [
  ['Agent sidebar', agentSidebar],
  ['Agent header', agentHeader],
  ['Agent transcript', agentTranscript],
  ['Agent composer', agentComposer],
  ['Agent search', agentSearch],
  ['Agent overlays', agentOverlays],
  ['Agent network', agentNetwork],
  ['Agent settings panel', agentSettingsPanel],
];
for (const [name, source] of primaryAgentUi) {
  forbidPattern(`${name} bypasses FabButton with a raw button element`, source, /<button\b/);
  forbidPattern(`${name} bypasses FabInput with a raw input element`, source, /<input\b/);
  forbidPattern(`${name} bypasses FabSelect with a raw select element`, source, /<select\b/);
}
for (const [name, source] of [
  ['Agent sidebar', agentSidebar],
  ['Agent header', agentHeader],
  ['Agent transcript', agentTranscript],
  ['Agent composer', agentComposer],
  ['Agent search', agentSearch],
  ['Agent overlays', agentOverlays],
  ['Agent network', agentNetwork],
  ['Agent settings panel', agentSettingsPanel],
]) requirePattern(`${name} no longer consumes FabButton`, source, /\bFabButton\b/);
for (const [name, source] of [
  ['Agent sidebar', agentSidebar],
  ['Agent composer', agentComposer],
  ['Agent search', agentSearch],
  ['Agent network', agentNetwork],
  ['Agent settings panel', agentSettingsPanel],
]) requirePattern(`${name} no longer consumes FabInput`, source, /\bFabInput\b/);
requirePattern('Agent sidebar no longer consumes FabSelect', agentSidebar, /\bFabSelect\b/);
requirePattern('Agent settings panel no longer consumes FabSelect', agentSettingsPanel, /\bFabSelect\b/);

if (/backgroundThrottling:\s*false/.test(electronMain)
  || /HOST_EVENT_LONG_POLL_MS/.test(electronMain)
  || /feature\.receive/.test(electronMain)) {
  violations.push('desktop power architecture regressed to unthrottled renderer or Host polling');
}
requirePattern('Electron Main no longer receives pushed Host runtime events', electronMain, /host\.onRuntimeEvent\s*\(/);
forbidPattern('Electron Host bridge must not retain feature.receive RPC compatibility', hostProcess, /feature\.receive/);
forbidPattern('Electron Mahayana edge must not expose feature.receive polling', mahayanaEdge, /feature\.receive/);
forbidPattern(
  'Electron renderer transport reintroduced feature.receive polling fallback',
  electronTransport,
  /feature\.receive|startEventPump|pumpEvents/,
);
forbidPattern(
  'Electron renderer transport must not persist conversation history in localStorage',
  electronTransport,
  /\blocalStorage\b|CONVERSATION_JOURNAL_KEY|persistConversationJournal/,
);
requirePattern(
  'Electron renderer transport must require pushed runtime events',
  electronTransport,
  /typeof window\.mahayana\.subscribe === ['"]function['"]/,
);
forbidPattern('Renderer Computer controller reintroduced WebRTC/polling ownership', computerController, /RemoteComputerDesktopController|RTCPeerConnection|SIGNAL_POLL_MS|SESSION_POLL_MS|HEARTBEAT_MS/);
requirePattern('Renderer Computer controller must send explicit human takeover commands', computerController, /\.takeComputerControl\s*\([\s\S]{0,1000}\.releaseComputerControl\s*\(/);
requirePattern('Renderer Computer controller must project host controlChanged state', computerController, /event\.type\s*!==\s*['"]computer\.controlChanged['"]/);
requirePattern('Agent Computer panel must expose Take Control', agentOverlays, />Take Control</);
requirePattern('Agent Computer panel must expose Release Control', agentOverlays, />Release Control</);
requirePattern('Agent root must surface waiting-user Computer takeover', rootShell, /event\.type\s*===\s*['"]turn\.state['"][\s\S]{0,500}event\.state\s*===\s*['"]waiting-user['"][\s\S]{0,900}computer\.openForAgent/);
requirePattern('FeatureHost must acquire a human ComputerControlLease', featureHost, /FeatureCommand::ComputerTakeControl[\s\S]{0,2600}acquire_control_lease\s*\(/);
requirePattern('FeatureHost must release a human ComputerControlLease', featureHost, /FeatureCommand::ComputerReleaseControl[\s\S]{0,2200}release_control_lease\s*\(/);
requirePattern('FeatureHost takeover must emit computer.controlChanged', featureHost, /FeatureCommand::ComputerTakeControl[\s\S]{0,3000}HostEvent::ComputerControlChanged/);
requirePattern('Remote-device Main supervisor lost adaptive refresh scheduling', remoteSupervisor, /sessionRefreshDelay\s*\(/);

// Execution-order checks prevent actor/broker types from existing while real calls bypass them.
const startMessageIndex = runtimeLib.indexOf('fn start_message');
const startMessageSource = startMessageIndex >= 0 ? runtimeLib.slice(startMessageIndex) : '';
requireOrdered(
  'Conversation execution must register its actor, serialize provider execution, persist the terminal state, and release the actor run',
  startMessageSource,
  [
    '.actors',
    '.actor(&conversation_id)',
    '.register(turn_id.clone())',
    'record_turn(&turn)',
    'record_run(&run)',
    'actor.gate.lock().await',
    'provider.send_message(request, sink).await',
    'transition_turn_state(&event_tx, &store, &context, terminal_state)',
    'actor.finish(&run_id)',
  ],
);

function commandArm(name, nextName) {
  const start = runtimeLib.indexOf(`RuntimeCommand::${name}`);
  const end = nextName ? runtimeLib.indexOf(`RuntimeCommand::${nextName}`, start + 1) : -1;
  if (start < 0) return '';
  return runtimeLib.slice(start, end > start ? end : undefined);
}
const invokeCapabilitySource = commandArm('InvokeCapability', 'ListPluginCommands');
requireOrdered(
  'InvokeCapability must authorize and pass the fail-closed execution gate before creating an execution run',
  invokeCapabilitySource,
  ['capability_broker', '.authorize(', 'require_capability_execution_allowed(', 'start_message('],
);
requirePattern(
  'Capability execution gate must explicitly stop NeedsUser before execution',
  runtimeLib,
  /fn require_capability_execution_allowed[\s\S]{0,1200}CapabilityPolicyDecision::NeedsUser\s*=>\s*Err\(/,
);
requirePattern(
  'Capability execution gate must explicitly stop Deny before execution',
  runtimeLib,
  /fn require_capability_execution_allowed[\s\S]{0,1200}CapabilityPolicyDecision::Deny\s*=>\s*Err\(/,
);
requireOrdered(
  'Local Mini App tools must authorize before executing the tool',
  commandArm('CallLocalPluginTool', 'McpServers'),
  ['capability_broker', 'authorize_request(', 'CapabilityPolicyDecision::Allow', '.call_tool('],
);
requireOrdered(
  'Direct MCP tools must authorize before backend execution',
  commandArm('McpToolCall', 'ConversationHistory'),
  ['authorize_human_capability(', '.call_mcp_tool('],
);
for (const [command, nextCommand] of [
  ['McpOauthLogin', 'McpOauthLogout'],
  ['McpOauthLogout', 'McpRemove'],
  ['McpRemove', 'McpSetCustomInstructions'],
  ['McpSetCustomInstructions', 'McpSetToolDisabled'],
  ['McpSetToolDisabled', 'McpRefresh'],
  ['McpRefresh', 'McpToolCall'],
]) {
  requireOrdered(
    `Runtime ${command} must authorize before backend mutation`,
    commandArm(command, nextCommand),
    ['authorize_human_capability(', 'backend'],
  );
}

requirePattern('ConversationActor registry is missing', actor, /pub struct ConversationActorRegistry/);
requirePattern('ConversationActor per-conversation gate is missing', actor, /AsyncMutex/);
requirePattern('Runtime no longer resolves a ConversationActor before execution', runtimeLib, /\.actors[\s\S]{0,120}\.actor\(&conversation_id\)/);
requirePattern('Runtime no longer locks the ConversationActor execution gate', runtimeLib, /actor\.gate\.lock\(\)\.await/);
requirePattern('Runtime no longer persists LogicalTurn/ExecutionRun state transitions', runtimeLib, /transition_turn_state/);
requirePattern(
  'Regenerate must preserve one logical user message id instead of creating a duplicate turn',
  rootShell,
  /prepareRetry\([\s\S]{0,500}messageId:\s*prompt\.id[\s\S]{0,240}retryOfMessageId:\s*prompt\.id/,
);
requirePattern(
  'Transcript retry must replace the prior run projection instead of duplicating the user prompt',
  agentTranscriptStore,
  /prepareRetry\([\s\S]{0,900}message\.operationId === operationId/,
);
requirePattern(
  'Runtime retry must resolve the next durable ExecutionRun generation',
  runtimeLib,
  /retry_of_client_message_id[\s\S]{0,1600}retry_turn_generation/,
);
requirePattern(
  'RuntimeStore must compute retry generation from durable runs',
  runtimeStore,
  /pub fn retry_turn_generation[\s\S]{0,1800}MAX\(r\.generation\)/,
);
requireOrdered(
  'RuntimeStore must persist restartable turn execution inputs',
  runtimeStore,
  ['CREATE TABLE IF NOT EXISTS turn_requests', 'pub fn record_turn_request'],
);
requirePattern(
  'Runtime must recover interrupted logical turns before durable handoffs',
  runtimeLib,
  /recover_interrupted_turns\(\)\?[\s\S]{0,120}recover_pending_handoffs\(\)\?/,
);
requirePattern(
  'Runtime restart recovery must resume one durable logical turn as a retry generation',
  runtimeLib,
  /fn recover_interrupted_turns[\s\S]{0,3200}TurnState::Recovering[\s\S]{0,1200}start_message\(/,
);
requirePattern(
  'Rust restart recovery must fail closed for waiting-user turns',
  runtimeLib,
  /pending\.state == TurnState::WaitingUser[\s\S]{0,900}TurnState::Failed[\s\S]{0,900}explicit retry is required/,
);
requireOrdered(
  'Renderer recovery must reclaim a Rust recovering run by conversation identity',
  runtimeCoordinator,
  [
    'peerByConversationId',
    'prepareOperationRecovery',
    "event.state === 'recovering'",
    'claimOperation(event.operationId, recoveryPeerKey)',
  ],
);
requirePattern(
  'Recovering turn state must synthesize the canonical assistant run projection after restart cleanup',
  agentTranscriptStore,
  /applyTurnState[\s\S]{0,2200}activeStates[\s\S]{0,700}recovering[\s\S]{0,1800}createAssistantTurn\(event\.operationId, updatedAtMs\)/,
);
requirePattern(
  'Waiting-user restart handling must remain fail-closed',
  runtimeCoordinator,
  /turnStateByOperation\.get\(operationId\) === ['"]waiting-user['"][\s\S]{0,1200}operation\.interrupted/,
);
requirePattern('CapabilityBroker is missing', broker, /pub struct CapabilityBroker/);
requirePattern('CapabilityBroker no longer owns authorization decisions', broker, /pub fn authorize_request\s*\(/);
requirePattern('Runtime InvokeCapability bypasses CapabilityBroker', runtimeLib, /RuntimeCommand::InvokeCapability[\s\S]{0,2800}capability_broker[\s\S]{0,320}\.authorize\s*\(/);
requirePattern('CapabilityBroker descriptor authorization bypasses request policy and audit', broker, /pub fn authorize[\s\S]{0,900}self\.authorize_request\s*\(/);
requirePattern(
  'Provider approval requests must enter CapabilityBroker as needs-user before execution',
  runtimeLib,
  /RuntimeEvent::ApprovalRequested[\s\S]{0,1800}CapabilityBroker::new[\s\S]{0,600}CapabilityAvailability::PermissionRequired/,
);
requireOrdered(
  'Approval resolution must record final CapabilityBroker allow/deny before provider execution',
  commandArm('ResolveApproval', null),
  ['capability_broker', 'authorize_request(', 'provider.resolve_approval('],
);
requirePattern(
  'External collaboration must be a first-class durable Runtime handoff',
  runtimeLib,
  /RuntimeCommand::ExternalHandoff[\s\S]{0,1600}dispatch_handoff\s*\(/,
);
requireOrdered(
  'Runtime handoff must authorize through CapabilityBroker before enqueue/start',
  runtimeLib.slice(runtimeLib.indexOf('fn dispatch_handoff'), runtimeLib.indexOf('fn list_conversations')),
  ['capability_broker', 'authorize_request(', 'require_capability_execution_allowed', 'reserve_handoff_slot', 'enqueue_handoff', 'start_message('],
);
requirePattern(
  'Every dynamic Computer execution must enter CapabilityBroker before provider execution',
  runtimeLib,
  /RuntimeEvent::AgentActivity[\s\S]{0,1500}kind == ["']computer["'][\s\S]{0,1800}CapabilityBroker::new[\s\S]{0,900}computer\.input\.control/,
);
requireOrdered(
  'Codex Computer pre-execution activity must propagate Runtime policy failure',
  codexAgentBackend,
  [
    'fn emit_computer_activity',
    'Result<(), AgentError>',
    'events.emit(AgentEvent::Activity',
    'if let Err(error) = self.emit_computer_activity',
  ],
);
requirePattern(
  'Durable handoff fan-out must consult RuntimeStore',
  runtimeLib,
  /reserve_handoff_slot[\s\S]{0,800}count_handoffs_for_run/,
);
requirePattern(
  'Durable handoff dispatch journal is missing',
  runtimeStore,
  /CREATE TABLE IF NOT EXISTS handoff_dispatch/,
);
requirePattern(
  'Runtime startup must recover unfinished handoffs after Ready',
  runtimeLib,
  /RuntimeEvent::Ready[\s\S]{0,320}recover_pending_handoffs\(\)/,
);
requirePattern(
  'Runtime must persist handoff running state after target dispatch',
  runtimeLib,
  /enqueue_handoff[\s\S]{0,1000}start_message[\s\S]{0,500}mark_handoff_started/,
);
requirePattern(
  'Runtime must persist handoff terminal state from target execution',
  runtimeLib,
  /mark_handoff_terminal\(intent_id, result\.is_ok\(\), now_millis\(\)\)/,
);
requirePattern(
  'Retry generations must project the canonical recovering state',
  runtimeLib,
  /retrying = retry_of_client_message_id\.is_some\(\)[\s\S]{0,2600}TurnState::Recovering/,
);
requirePattern(
  'Recovery must reuse the same logical turn through retryOfClientMessageId',
  runtimeLib,
  /fn recover_pending_handoffs[\s\S]{0,5200}retry_message_id[\s\S]{0,1200}start_message/,
);
requireOrdered(
  'Runtime startup must recover interrupted user turns before durable handoffs',
  runtimeLib.slice(runtimeLib.indexOf('RuntimeEvent::Ready'), runtimeLib.indexOf('pub fn status')),
  ['RuntimeEvent::Ready', 'recover_interrupted_turns()?', 'recover_pending_handoffs()?'],
);
requireOrdered(
  'Interrupted user-turn recovery must reuse durable request input and retry the same logical turn',
  runtimeLib.slice(runtimeLib.indexOf('fn recover_interrupted_turns'), runtimeLib.indexOf('fn recover_pending_handoffs')),
  [
    'pending.text',
    'TurnState::Recovering',
    'self.start_message(',
    'Some(pending.message_id.to_string())',
  ],
);
requirePattern(
  'Waiting-user Rust restart handling must fail closed and require explicit retry',
  runtimeLib,
  /pending\.state == TurnState::WaitingUser[\s\S]{0,1200}TurnState::Failed[\s\S]{0,1400}explicit retry is required/,
);
requirePattern(
  'Runtime must persist turn execution input before provider execution',
  runtimeLib,
  /record_turn\(&turn\)[\s\S]{0,500}record_turn_request\([\s\S]{0,700}record_run\(&run\)/,
);
requirePattern(
  'Recoverable user turns must exclude Agent handoff intents from the ordinary-turn recovery lane',
  runtimeStore,
  /pub fn recoverable_turns[\s\S]{0,5000}p\.kind = 'agent-handoff'/,
);
requirePattern(
  'FeatureHost Agent sends must use Runtime handoff rather than hidden SendMessage',
  featureHost,
  /fn schedule_agent_handoff[\s\S]{0,2600}RuntimeCommand::Handoff[\s\S]{0,2600}RuntimeCommand::ExternalHandoff/,
);
requirePattern(
  'FeatureHost broadcasts must use external durable handoff batches',
  featureHost,
  /FeatureCommand::AgentBroadcast[\s\S]{0,3000}schedule_external_agent_handoff/,
);
requirePattern(
  'FeatureHost group turns must use external durable handoff',
  featureHost,
  /fn start_next_group_turn[\s\S]{0,6000}RuntimeCommand::ExternalHandoff/,
);
requirePattern('FeatureHost no longer routes privileged commands through RuntimeCommand::AuthorizeCapability', featureHost, /RuntimeCommand::AuthorizeCapability/);
requirePattern('Capability audit persistence is missing', runtimeStore, /CREATE TABLE IF NOT EXISTS capability_audit/);
requirePattern('Turn persistence is missing', runtimeStore, /CREATE TABLE IF NOT EXISTS turns/);
requirePattern('Run persistence is missing', runtimeStore, /CREATE TABLE IF NOT EXISTS runs/);
requirePattern('Computer lease persistence is missing', runtimeStore, /CREATE TABLE IF NOT EXISTS computer_leases/);
requirePattern('Physical Computer control lease is missing', computerRuntime, /pub fn execute_with_lease\s*\(/);

if (violations.length) {
  console.error('Agent architecture boundary regression detected:');
  for (const violation of [...new Set(violations)]) console.error(`- ${violation}`);
  process.exitCode = 1;
} else {
  console.log('Agent architecture boundary check passed.');
}
