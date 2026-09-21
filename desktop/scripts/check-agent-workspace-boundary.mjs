import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, '..');
const repoRoot = path.resolve(desktopRoot, '..');
const srcRoot = path.join(desktopRoot, 'src');
const violations = [];

function read(...parts) {
  return fs.readFileSync(path.join(...parts), 'utf8');
}

function sourceFiles(root) {
  const files = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const target = path.join(root, entry.name);
    if (entry.isDirectory()) files.push(...sourceFiles(target));
    else if (/\.(?:ts|tsx)$/.test(entry.name)) files.push(target);
  }
  return files;
}

function requirePattern(label, text, pattern) {
  if (!pattern.test(text)) violations.push(label);
}

function forbidPattern(label, text, pattern) {
  if (pattern.test(text)) violations.push(label);
}

const legacyShellPath = path.join(srcRoot, 'adapters', 'legacy-messaging', 'legacy-messaging-shell.tsx');
if (fs.existsSync(legacyShellPath)) {
  violations.push('legacy-messaging-shell.tsx must be deleted; renaming or remounting the god component is forbidden');
}

const rootShell = read(srcRoot, 'agent-workspace', 'agent-root-shell.tsx');
const desktopApp = read(srcRoot, 'app', 'DesktopApp.tsx');
const compatibility = read(srcRoot, 'agent-workspace', 'messenger-compatibility-adapter.tsx');
const avatar = read(srcRoot, 'ui', 'avatar', 'fab-avatar.tsx');
const avatarCss = read(srcRoot, 'ui', 'avatar', 'fab-avatar.module.css');
const primitives = read(srcRoot, 'ui', 'primitives', 'fab-primitives.tsx');
const tokens = read(srcRoot, 'ui', 'tokens.css');
const electronMain = read(desktopRoot, 'electron', 'main.cjs');
const remoteSupervisor = read(desktopRoot, 'electron', 'remote-device-agent-supervisor.cjs');
const sidebarController = read(srcRoot, 'agent-workspace', 'use-agent-sidebar-controller.ts');

requirePattern('DesktopApp must directly mount AgentRootShell', desktopApp, /import\s+AgentRootShell[\s\S]*<AgentRootShell\s*\/>/);
forbidPattern('DesktopApp must not import a legacy messaging shell', desktopApp, /legacy-messaging-shell|LegacyMessagingAdapter/);

for (const token of [
  'AgentSidebar',
  'AgentWorkspace',
  'AgentNetwork',
  'AgentCommandPalette',
  'AgentOverlays',
  'useAgentProductControllers',
  'useAgentWorkspaceRuntime',
  'useAgentComputerController',
]) {
  if (!rootShell.includes(token)) violations.push(`AgentRootShell no longer owns required Agent product boundary: ${token}`);
}
requirePattern('AgentRootShell must remain the sole visible Agent product workspace', rootShell, /data-agent-root-shell=['"]true['"][\s\S]*data-product-shell=['"]agent['"]/);
forbidPattern('AgentRootShell must not import the deleted legacy shell', rootShell, /legacy-messaging-shell\.tsx|LegacyMessagingAdapter/);
forbidPattern('AgentRootShell must not recreate interval polling', rootShell, /setInterval\s*\(/);

const adapterPaths = [
  ['Contacts', 'adapters/contacts/contact-compatibility-adapter.ts'],
  ['Telegram', 'adapters/telegram/telegram-compatibility-adapter.ts'],
  ['MiniApp', 'adapters/miniapps/miniapp-compatibility-adapter.tsx'],
  ['Payments', 'adapters/payments/payment-compatibility-adapter.tsx'],
  ['Calls', 'adapters/calls/call-compatibility-adapter.tsx'],
  ['Settings', 'adapters/settings/settings-compatibility-adapter.tsx'],
];
const agentOwnershipPattern = /\b(?:AgentSidebar|AgentHeader|AgentNetwork|AgentWorkspace|AgentRuntimeCoordinator|useAgentProductControllers|useAgentWorkspaceRuntime|useAgentComputerController)\b/;
for (const [label, relative] of adapterPaths) {
  const target = path.join(srcRoot, relative);
  if (!fs.existsSync(target)) {
    violations.push(`${label} compatibility adapter is missing`);
    continue;
  }
  const text = fs.readFileSync(target, 'utf8');
  if (agentOwnershipPattern.test(text)) {
    violations.push(`${label} compatibility adapter illegally owns Agent UI/runtime state`);
  }
  if (/from\s+['"][^'"]*agent-workspace\/(?:agent-(?:sidebar|header|network|workspace)|use-agent-)/.test(text)) {
    violations.push(`${label} compatibility adapter imports an Agent product/controller implementation`);
  }
}
for (const fragment of [
  'contact-compatibility-adapter',
  'telegram-compatibility-adapter',
  'miniapp-compatibility-adapter',
  'payment-compatibility-adapter',
  'call-compatibility-adapter',
  'settings-compatibility-adapter',
]) {
  if (!compatibility.includes(fragment)) violations.push(`compatibility composition no longer routes through ${fragment}`);
}

const desktopSources = sourceFiles(srcRoot);
const oldAvatarPattern = /\bBotMark\b|FabushiAvatarRuntime|fabushi-avatar-runtime|fabushi-bot-mark-engine|fabushi-motion-v3/;
for (const file of desktopSources) {
  const text = fs.readFileSync(file, 'utf8');
  if (oldAvatarPattern.test(text)) {
    violations.push(`desktop bundle still references legacy avatar runtime: ${path.relative(desktopRoot, file)}`);
  }
}
forbidPattern('FabAvatar must not infer unread/business state from DOM observers', avatar, /MutationObserver|\.closest\s*\(/);
forbidPattern('FabAvatar must not create per-instance requestAnimationFrame loops', avatar, /(?:requestAnimationFrame|cancelAnimationFrame)\s*\(/);
requirePattern('FabAvatar must expose explicit normalized state input', avatar, /normalizeFabAvatarState/);
requirePattern('FabAvatar active motion must be CSS/low-frequency rather than a JS frame loop', avatarCss, /@keyframes/);

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
]) {
  if (!primitives.includes(primitive)) violations.push(`shared UI primitive is missing: ${primitive}`);
}
for (const token of [
  '--fab-bg-primary',
  '--fab-bg-raised',
  '--fab-border-subtle',
  '--fab-text-primary',
  '--fab-text-muted',
  '--fab-radius-sm',
  '--fab-radius-md',
  '--fab-space-1',
  '--fab-shadow-popover',
]) {
  if (!tokens.includes(token)) violations.push(`shared UI token is missing: ${token}`);
}

forbidPattern('desktop renderer must not disable Chromium background throttling', electronMain, /backgroundThrottling:\s*false/);
forbidPattern('desktop main must not poll feature.receive', electronMain, /HOST_EVENT_LONG_POLL_MS|feature\.receive/);
requirePattern('desktop main must consume pushed Host runtime events', electronMain, /host\.onRuntimeEvent\s*\(/);
forbidPattern('Remote Computer must not return to fixed Renderer/session polling', remoteSupervisor, /SESSION_POLL_MS|SIGNAL_POLL_MS/);
forbidPattern('Agent sidebar layout must not return to fixed polling', sidebarController, /setInterval\s*\(/);

const runtimeRoot = path.join(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-runtime', 'src');
const runtimeLib = read(runtimeRoot, 'lib.rs');
const actor = read(runtimeRoot, 'conversation_actor.rs');
const store = read(runtimeRoot, 'runtime_store.rs');
const broker = read(runtimeRoot, 'capability_broker.rs');
const featureHost = read(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-feature-host', 'src', 'implementation.rs');
const computer = read(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-computer', 'src', 'lib.rs');
const codex = read(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-agent-codex', 'src', 'implementation.rs');

for (const [label, pattern] of [
  ['ConversationActor registry', /pub struct ConversationActorRegistry/],
  ['ConversationActor async gate', /pub(?:\(crate\))?\s+gate:\s+AsyncMutex/],
  ['ConversationActor register lifecycle', /pub fn register\s*\(/],
  ['ConversationActor start lifecycle', /pub fn start\s*\(/],
  ['ConversationActor finish lifecycle', /pub fn finish\s*\(/],
  ['ConversationActor state lifecycle', /pub fn set_state\s*\(/],
]) requirePattern(`Rust runtime lost ${label}`, actor, pattern);
requirePattern('runtime execution must resolve the ConversationActor', runtimeLib, /\.actors[\s\S]{0,140}\.actor\(&conversation_id\)/);
requirePattern('runtime execution must serialize turns through the Actor gate', runtimeLib, /actor\.gate\.lock\(\)\.await/);
requirePattern('runtime execution must start the Actor before provider work', runtimeLib, /actor[\s\S]{0,100}\.start\(&turn_id,\s*run_id\.clone\(\)\)/);
requirePattern('runtime execution must finish the Actor after provider work', runtimeLib, /actor\.finish\(&run_id\)/);
requirePattern('runtime execution must persist turn/run state transitions', runtimeLib, /transition_turn_state[\s\S]*store\.set_turn_state/);
for (const schema of ['turns', 'runs', 'capability_audit', 'computer_leases', 'workspace_state']) {
  if (!new RegExp(`CREATE TABLE IF NOT EXISTS ${schema}`).test(store)) violations.push(`RuntimeStore lost ${schema} persistence`);
}
requirePattern('RuntimeStore must stay WAL-backed', store, /PRAGMA journal_mode=WAL/);

requirePattern('CapabilityBroker must expose the policy/audit boundary', broker, /pub struct CapabilityBroker[\s\S]*pub fn authorize_request\s*\(/);
requirePattern('Runtime AuthorizeCapability must route through CapabilityBroker', runtimeLib, /RuntimeCommand::AuthorizeCapability[\s\S]{0,1600}capability_broker[\s\S]{0,500}authorize_request\s*\(/);
requirePattern('FeatureHost must authorize commands before production execution', featureHost, /self\.authorize_feature_command\(&command\)\?/);
for (const capability of [
  'computer.screen.read',
  'computer.input.control',
  'computer.remote.session',
  'mcp.tool.call',
  'agent.handoff',
  'agent.handoff.broadcast',
  'filesystem.agent.write',
  'filesystem.agent.read',
  'miniapp.open',
  'connector.connect',
]) {
  if (!featureHost.includes(capability)) violations.push(`FeatureHost capability path bypass risk: missing ${capability}`);
}
requirePattern('FeatureHost capability path must invoke RuntimeCommand::AuthorizeCapability', featureHost, /RuntimeCommand::AuthorizeCapability/);
requirePattern('AI computer execution must enforce the shared control lease', codex, /mahayana_computer::execute_with_lease\s*\(/);
requirePattern('computer layer must expose a single control lease', computer, /pub struct ComputerControlLeaseRequest[\s\S]*pub fn execute_with_lease\s*\(/);

const desktopWorkflow = read(repoRoot, '.github', 'workflows', 'desktop-chat-parity-ci.yml');
const rustWorkflow = read(repoRoot, '.github', 'workflows', 'rust-desktop-runtime.yml');
for (const [label, workflow] of [['desktop parity', desktopWorkflow], ['Rust runtime', rustWorkflow]]) {
  if (!/github\.event\.pull_request\.head\.sha/.test(workflow) || !/ref:\s*\$\{\{/.test(workflow)) {
    violations.push(`${label} workflow checkout is not explicitly bound to pull_request.head.sha`);
  }
}

if (violations.length) {
  console.error('Agent architecture boundary regression detected:');
  for (const violation of violations) console.error(`- ${violation}`);
  process.exitCode = 1;
} else {
  console.log('Agent architecture boundary check passed.');
}
