import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, '..');
const repoRoot = path.resolve(desktopRoot, '..');
const violations = [];

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

const desktopApp = read(desktopRoot, 'src', 'app', 'DesktopApp.tsx');
const rootShell = read(desktopRoot, 'src', 'agent-workspace', 'agent-root-shell.tsx');
const rootCss = read(desktopRoot, 'src', 'agent-workspace', 'agent-root-shell.module.css');
const productControllers = read(desktopRoot, 'src', 'agent-workspace', 'use-agent-product-controllers.ts');
const runtimeFacade = read(desktopRoot, 'src', 'agent-workspace', 'use-agent-workspace-runtime.ts');
const directoryController = read(desktopRoot, 'src', 'agent-workspace', 'use-agent-directory-controller.ts');
const networkController = read(desktopRoot, 'src', 'agent-workspace', 'use-agent-network-controller.ts');
const computerController = read(desktopRoot, 'src', 'agent-workspace', 'use-agent-computer-controller.ts');
const agentTranscript = read(desktopRoot, 'src', 'agent-workspace', 'agent-transcript.tsx');
const agentHeader = read(desktopRoot, 'src', 'grok-shell', 'grok-agent-header.tsx');
const agentSidebar = read(desktopRoot, 'src', 'grok-shell', 'grok-agent-sidebar.tsx');
const agentOverlays = read(desktopRoot, 'src', 'agent-workspace', 'agent-overlays.tsx');
const fabAvatar = read(desktopRoot, 'src', 'ui', 'avatar', 'fab-avatar.tsx');
const primitives = read(desktopRoot, 'src', 'ui', 'primitives', 'fab-primitives.tsx');
const tokens = read(desktopRoot, 'src', 'ui', 'tokens.css');
const electronMain = read(desktopRoot, 'electron', 'main.cjs');
const remoteSupervisor = read(desktopRoot, 'electron', 'remote-device-agent-supervisor.cjs');

const runtimeRoot = path.join(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-runtime', 'src');
const runtimeLib = read(runtimeRoot, 'lib.rs');
const actor = read(runtimeRoot, 'conversation_actor.rs');
const broker = read(runtimeRoot, 'capability_broker.rs');
const runtimeStore = read(runtimeRoot, 'runtime_store.rs');
const featureHost = read(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-feature-host', 'src', 'implementation.rs');
const computerRuntime = read(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-computer', 'src', 'lib.rs');

const legacyShell = path.join(desktopRoot, 'src', 'adapters', 'legacy-messaging', 'legacy-messaging-shell.tsx');
const obsoleteShell = path.join(desktopRoot, 'src', 'messaging-shell-v2.tsx');
if (fs.existsSync(legacyShell)) violations.push('legacy-messaging-shell.tsx still exists; the cutover must delete it instead of renaming it');
if (fs.existsSync(obsoleteShell)) violations.push('messaging-shell-v2.tsx returned');

requirePattern(
  'DesktopApp must boot through DesktopAuthBoundary directly into AgentRootShell',
  desktopApp,
  /<DesktopAuthBoundary>[\s\S]*<AgentRootShell\s+onLogout=\{onLogout\}\s*\/>[\s\S]*<\/DesktopAuthBoundary>/,
);
forbidPattern('DesktopApp must not import any legacy messaging shell', desktopApp, /legacy-messaging|messaging-shell-v2/);

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
]) requirePattern(`Fabushi UI primitive missing: ${primitive}`, primitives, new RegExp(`(?:function|\\{)\\s*${primitive}\\b|export\\s+\\{[^}]*\\b${primitive}\\b`));
for (const token of ['--fab-bg-primary', '--fab-bg-raised', '--fab-border-subtle', '--fab-text-primary', '--fab-text-muted', '--fab-radius-sm', '--fab-radius-md', '--fab-space-1']) {
  if (!tokens.includes(token)) violations.push(`Fabushi design token missing: ${token}`);
}
requirePattern('Agent root layout no longer exposes the three-column Agent grid', rootCss, /grid-template-columns:[\s\S]*minmax\(420px,\s*1fr\)/);

if (/backgroundThrottling:\s*false/.test(electronMain)
  || /HOST_EVENT_LONG_POLL_MS/.test(electronMain)
  || /feature\.receive/.test(electronMain)) {
  violations.push('desktop power architecture regressed to unthrottled renderer or Host polling');
}
requirePattern('Electron Main no longer receives pushed Host runtime events', electronMain, /host\.onRuntimeEvent\s*\(/);
forbidPattern('Renderer Computer controller reintroduced WebRTC/polling ownership', computerController, /RemoteComputerDesktopController|RTCPeerConnection|SIGNAL_POLL_MS|SESSION_POLL_MS|HEARTBEAT_MS/);
requirePattern('Remote-device Main supervisor lost adaptive refresh scheduling', remoteSupervisor, /sessionRefreshDelay\s*\(/);

requirePattern('ConversationActor registry is missing', actor, /pub struct ConversationActorRegistry/);
requirePattern('ConversationActor per-conversation gate is missing', actor, /AsyncMutex/);
requirePattern('Runtime no longer resolves a ConversationActor before execution', runtimeLib, /\.actors[\s\S]{0,120}\.actor\(&conversation_id\)/);
requirePattern('Runtime no longer locks the ConversationActor execution gate', runtimeLib, /actor\.gate\.lock\(\)\.await/);
requirePattern('Runtime no longer persists LogicalTurn/ExecutionRun state transitions', runtimeLib, /transition_turn_state/);
requirePattern('CapabilityBroker is missing', broker, /pub struct CapabilityBroker/);
requirePattern('CapabilityBroker no longer owns authorization decisions', broker, /pub fn authorize_request\s*\(/);
requirePattern('Runtime InvokeCapability bypasses CapabilityBroker', runtimeLib, /RuntimeCommand::InvokeCapability[\s\S]{0,2400}capability_broker[\s\S]{0,240}authorize_request/);
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
