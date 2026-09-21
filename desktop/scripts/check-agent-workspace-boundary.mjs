import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, '..');
const repoRoot = path.resolve(desktopRoot, '..');
const srcRoot = path.join(desktopRoot, 'src');
const read = (relative) => fs.readFileSync(path.join(repoRoot, relative), 'utf8');
const exists = (relative) => fs.existsSync(path.join(repoRoot, relative));
const violations = [];

function walk(root, extensions = new Set(['.ts', '.tsx', '.js', '.mjs', '.cjs'])) {
  const entries = [];
  for (const item of fs.readdirSync(root, { withFileTypes: true })) {
    const full = path.join(root, item.name);
    if (item.isDirectory()) entries.push(...walk(full, extensions));
    else if (extensions.has(path.extname(item.name))) entries.push(full);
  }
  return entries;
}

function requireMatch(label, text, pattern) {
  if (!pattern.test(text)) violations.push(label);
}

function forbidMatch(label, text, pattern) {
  if (pattern.test(text)) violations.push(label);
}

const desktopApp = read('desktop/src/app/DesktopApp.tsx');
const rootShell = read('desktop/src/agent-workspace/agent-root-shell.tsx');
const mainEntry = read('desktop/src/main.tsx');
const compatibilityProjection = read('desktop/src/adapters/compatibility/messenger-compatibility-adapter.tsx');
const primitives = read('desktop/src/ui/primitives/fab-primitives.tsx');
const tokens = read('desktop/src/ui/tokens.css');
const avatar = read('desktop/src/ui/avatar/fab-avatar.tsx');
const avatarCss = read('desktop/src/ui/avatar/fab-avatar.module.css');
const electronMain = read('desktop/electron/main.cjs');
const computerController = read('desktop/src/agent-workspace/use-agent-computer-controller.ts');
const sidebarController = read('desktop/src/agent-workspace/use-agent-sidebar-controller.ts');
const actor = read('third_party/mahayana/mahayana-rs/mahayana-runtime/src/conversation_actor.rs');
const broker = read('third_party/mahayana/mahayana-rs/mahayana-runtime/src/capability_broker.rs');
const runtime = read('third_party/mahayana/mahayana-rs/mahayana-runtime/src/lib.rs');
const runtimeStore = read('third_party/mahayana/mahayana-rs/mahayana-runtime/src/runtime_store.rs');

for (const removed of [
  'desktop/src/messaging-shell-v2.tsx',
  'desktop/src/adapters/legacy-messaging/legacy-messaging-shell.tsx',
  'desktop/src/adapters/legacy-messaging/legacy-messaging-shell.module.css',
  'desktop/src/agent-workspace/messenger-compatibility-adapter.tsx',
]) {
  if (exists(removed)) violations.push(`removed legacy product shell/adapter returned: ${removed}`);
}

requireMatch('DesktopApp must directly render AgentRootShell', desktopApp, /return\s+<AgentRootShell\s*\/>/);
forbidMatch('DesktopApp must not mount a legacy messaging product root', desktopApp, /LegacyMessagingAdapter|legacy-messaging-shell|messaging-shell-v2/);
requireMatch('main.tsx must mount DesktopApp', mainEntry, /<DesktopApp\s*\/>/);

for (const [label, pattern] of [
  ['AgentRootShell must own AgentSidebar', /<AgentSidebar\b/],
  ['AgentRootShell must own AgentWorkspace conversation/composer/transcript surface', /<AgentWorkspace\b/],
  ['AgentRootShell must own AgentNetwork', /<AgentNetwork\b/],
  ['AgentRootShell must own Agent runtime facade', /useAgentWorkspaceRuntime\s*\(/],
  ['AgentRootShell must own Agent product controllers', /useAgentProductControllers\s*\(/],
  ['AgentRootShell must own Computer controller', /useAgentComputerController\s*\(/],
  ['AgentRootShell must own Agent settings controller', /useAgentSettingsController\s*\(/],
  ['AgentRootShell must identify itself as product shell', /data-agent-root-shell=['"]?true|data-agent-root-shell="true"/],
]) requireMatch(label, rootShell, pattern);

for (const requiredAdapter of [
  '../adapters/contacts/contacts-compatibility-adapter',
  '../adapters/telegram/telegram-compatibility-adapter',
  '../adapters/miniapps/miniapps-compatibility-adapter',
  '../adapters/payments/payments-compatibility-adapter',
  '../adapters/calls/calls-compatibility-adapter',
  '../adapters/settings/settings-compatibility-adapter',
  '../adapters/compatibility/messenger-compatibility-adapter',
]) {
  if (!rootShell.includes(requiredAdapter)) violations.push(`AgentRootShell does not compose required compatibility adapter: ${requiredAdapter}`);
}

forbidMatch('AgentRootShell re-embedded extracted Call dialog implementation', rootShell, /function\s+CallDialog\s*\(/);
forbidMatch('AgentRootShell re-embedded extracted MiniApp dialog implementation', rootShell, /function\s+MiniAppDialog\s*\(/);
forbidMatch('AgentRootShell re-embedded extracted Settings workspace implementation', rootShell, /function\s+SettingsWorkspace\s*\(/);
forbidMatch('AgentRootShell re-embedded extracted payment overview implementation', rootShell, /function\s+PaymentOverview\s*\(/);
forbidMatch('AgentRootShell re-embedded compatibility overlay implementation', rootShell, /function\s+(?:CommunityAdminDialog|StoryViewer|ForwardMessageDialog)\s*\(/);

const adapterRoot = path.join(srcRoot, 'adapters');
for (const file of walk(adapterRoot, new Set(['.ts', '.tsx']))) {
  const source = fs.readFileSync(file, 'utf8');
  const relative = path.relative(repoRoot, file);
  const forbiddenAgentOwnership = /(?:from\s+['"][^'"]*agent-workspace\/(?:agent-sidebar|agent-header|agent-network|agent-workspace|use-agent-workspace-runtime|use-agent-product-controllers|use-agent-computer-controller|use-agent-settings-controller)['"]|<Agent(?:Sidebar|Header|Network|Workspace)\b|useAgent(?:WorkspaceRuntime|ProductControllers|ComputerController|SettingsController)\s*\()/;
  if (forbiddenAgentOwnership.test(source)) {
    violations.push(`compatibility adapter owns Agent product/runtime boundary: ${relative}`);
  }
}

requireMatch('compatibility projection must own peer construction', compatibilityProjection, /export function buildCompatibilityPeers\s*\(/);
requireMatch('compatibility projection must delegate Contacts classification', compatibilityProjection, /contactKindForLegacyConversation\s*\(/);
requireMatch('compatibility projection must delegate Telegram classification', compatibilityProjection, /telegramKindForLegacyConversation\s*\(/);
requireMatch('compatibility projection must own secondary surface routing', compatibilityProjection, /export function CompatibilitySurface\s*\(/);

const desktopSourceFiles = walk(srcRoot, new Set(['.ts', '.tsx']));
for (const file of desktopSourceFiles) {
  const source = fs.readFileSync(file, 'utf8');
  const relative = path.relative(repoRoot, file);
  if (/\bBotMark\b|fabushi-avatar-runtime|FabushiAvatarRuntime/.test(source)) {
    violations.push(`desktop bundle references legacy avatar runtime: ${relative}`);
  }
}
forbidMatch('FabAvatar must not observe parent DOM', avatar, /MutationObserver|\.closest\s*\(/);
forbidMatch('FabAvatar must not own a per-instance animation frame', avatar + avatarCss, /requestAnimationFrame|cancelAnimationFrame|setInterval\s*\(/);
requireMatch('FabAvatar must accept explicit state input', avatar, /state\??:\s*FabAvatarInputState/);
for (const surface of [
  'desktop/src/grok-shell/grok-agent-sidebar.tsx',
  'desktop/src/grok-shell/grok-agent-header.tsx',
  'desktop/src/agent-workspace/agent-transcript.tsx',
  'desktop/src/agent-workspace/agent-overlays.tsx',
]) {
  const source = read(surface);
  requireMatch(`${surface} must use FabAvatar`, source, /FabAvatar/);
  forbidMatch(`${surface} regressed to BotMark`, source, /\bBotMark\b/);
}

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
  if (!new RegExp(`(?:export\\s+(?:function|\\{)[^\\n]*\\b${primitive}\\b|export\\s+\\{\\s*${primitive}\\s*\\})`).test(primitives)) {
    violations.push(`missing Fabushi UI primitive: ${primitive}`);
  }
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
]) {
  if (!tokens.includes(token)) violations.push(`missing Fabushi design token: ${token}`);
}

forbidMatch('AgentRootShell reintroduced fixed interval polling', rootShell, /setInterval\s*\(/);
forbidMatch('Agent sidebar reintroduced fixed interval polling', sidebarController, /setInterval\s*\(/);
forbidMatch('Renderer owns WebRTC/remote-computer polling loop', computerController, /RTCPeerConnection|SIGNAL_POLL_MS|SESSION_POLL_MS|HEARTBEAT_MS|setInterval\s*\(/);
forbidMatch('Electron disabled renderer background throttling', electronMain, /backgroundThrottling:\s*false/);
forbidMatch('Electron reintroduced Host feature.receive polling', electronMain, /HOST_EVENT_LONG_POLL_MS|feature\.receive/);
requireMatch('Electron runtime events must be push driven', electronMain, /host\.onRuntimeEvent\s*\(/);

requireMatch('ConversationActorRegistry missing', actor, /pub struct ConversationActorRegistry/);
requireMatch('ConversationActor gate missing', actor, /pub\(crate\) gate:\s*AsyncMutex/);
requireMatch('ConversationActor independent-gate test missing', actor, /registry_reuses_actor_and_gates_only_the_same_conversation/);
requireMatch('ConversationActor lifecycle-state test missing', actor, /actor_owns_turn_state_sequence_and_active_run_lifecycle/);
requireMatch('runtime must resolve a ConversationActor', runtime, /\.actors[\s\S]{0,180}\.actor\(&conversation_id\)/);
requireMatch('runtime must lock the per-conversation actor gate', runtime, /actor\.gate\.lock\(\)\.await/);
requireMatch('runtime must persist turn lifecycle transitions', runtime, /transition_turn_state\s*\(/);

requireMatch('CapabilityBroker authorize_request missing', broker, /pub fn authorize_request\s*\(/);
requireMatch('CapabilityBroker policy test missing', broker, /broker_maps_availability_to_policy_before_execution/);
requireMatch('Runtime InvokeCapability bypassed CapabilityBroker', runtime, /RuntimeCommand::InvokeCapability[\s\S]{0,2200}capability_broker[\s\S]{0,240}authorize\s*\(/);
requireMatch('Runtime local MiniApp tool bypassed CapabilityBroker', runtime, /RuntimeCommand::CallLocalPluginTool[\s\S]{0,2600}capability_broker[\s\S]{0,260}authorize_request\s*\(/);
requireMatch('Runtime direct MCP tool bypassed CapabilityBroker', runtime, /RuntimeCommand::McpToolCall[\s\S]{0,1800}capability_broker[\s\S]{0,260}authorize_request\s*\(/);
forbidMatch('Runtime direct MCP tool executes before broker authorization', runtime,
  /RuntimeCommand::McpToolCall\s*\{[\s\S]{0,900}call_mcp_tool[\s\S]{0,900}capability_broker/);

for (const storeBoundary of [
  'PRAGMA journal_mode=WAL',
  'CREATE TABLE IF NOT EXISTS turns',
  'CREATE TABLE IF NOT EXISTS runs',
  'CREATE TABLE IF NOT EXISTS pending_intents',
  'CREATE TABLE IF NOT EXISTS capability_audit',
  'CREATE TABLE IF NOT EXISTS computer_leases',
]) {
  if (!runtimeStore.includes(storeBoundary)) violations.push(`RuntimeStore lost required durable boundary: ${storeBoundary}`);
}

if (violations.length) {
  console.error('Agent architecture boundary regression detected:');
  for (const violation of violations) console.error(`- ${violation}`);
  process.exitCode = 1;
} else {
  console.log('Agent architecture boundary check passed.');
}
