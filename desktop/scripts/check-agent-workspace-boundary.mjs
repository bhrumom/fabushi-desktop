import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, '..');
const obsoleteShellPath = path.join(desktopRoot, 'src', 'messaging-shell-v2.tsx');
const legacyShellPath = path.join(desktopRoot, 'src', 'adapters', 'legacy-messaging', 'legacy-messaging-shell.tsx');
const shellPath = path.join(desktopRoot, 'src', 'agent-workspace', 'agent-root-shell.tsx');
const shell = fs.readFileSync(shellPath, 'utf8');
const repoRoot = path.resolve(desktopRoot, '..');
const desktopApp = fs.readFileSync(path.join(desktopRoot, 'src', 'app', 'DesktopApp.tsx'), 'utf8');
const mainEntry = fs.readFileSync(path.join(desktopRoot, 'src', 'main.tsx'), 'utf8');
const computerController = fs.readFileSync(path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-computer-controller.ts'), 'utf8');
const electronMain = fs.readFileSync(path.join(desktopRoot, 'electron', 'main.cjs'), 'utf8');
const remoteDeviceSupervisor = fs.readFileSync(path.join(desktopRoot, 'electron', 'remote-device-agent-supervisor.cjs'), 'utf8');
const hostClient = fs.readFileSync(path.join(repoRoot, 'frontend', 'apps', 'web', 'src', 'app', 'host', 'host-client.tsx'), 'utf8');
const agentTranscript = fs.readFileSync(path.join(desktopRoot, 'src', 'agent-workspace', 'agent-transcript.tsx'), 'utf8');
const agentOverlays = fs.readFileSync(path.join(desktopRoot, 'src', 'agent-workspace', 'agent-overlays.tsx'), 'utf8');
const networkControllerPath = path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-network-controller.ts');
const networkController = fs.readFileSync(networkControllerPath, 'utf8');
const workflowControllerPath = path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-workflow-controller.ts');
const workflowController = fs.readFileSync(workflowControllerPath, 'utf8');
const storeSyncControllerPath = path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-store-sync-controller.ts');
const storeSyncController = fs.readFileSync(storeSyncControllerPath, 'utf8');
const directoryControllerPath = path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-directory-controller.ts');
const directoryController = fs.readFileSync(directoryControllerPath, 'utf8');
const mcpController = fs.readFileSync(path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-mcp-controller.ts'), 'utf8');
const productControllers = fs.readFileSync(path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-product-controllers.ts'), 'utf8');
const agentComposerPath = path.join(desktopRoot, 'src', 'agent-workspace', 'agent-composer.tsx');
const agentComposer = fs.readFileSync(agentComposerPath, 'utf8');
const agentRichEditorPath = path.join(desktopRoot, 'src', 'agent-workspace', 'agent-rich-text-editor.tsx');
const agentRichEditor = fs.readFileSync(agentRichEditorPath, 'utf8');
const compatibilityAdapterPath = path.join(desktopRoot, 'src', 'adapters', 'compatibility', 'messaging-compatibility-adapter.tsx');
const compatibilityAdapter = fs.readFileSync(compatibilityAdapterPath, 'utf8');
const accountSidebarLayout = fs.readFileSync(path.join(desktopRoot, 'src', 'agent-workspace', 'account-sidebar-layout.ts'), 'utf8');
const sidebarController = fs.readFileSync(path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-sidebar-controller.ts'), 'utf8');
const hostProtocol = fs.readFileSync(path.join(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-host-protocol', 'src', 'lib.rs'), 'utf8');
const runtimeCore = fs.readFileSync(path.join(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-core', 'src', 'lib.rs'), 'utf8');
const kernelConversation = fs.readFileSync(path.join(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-runtime', 'src', 'kernel_conversation.rs'), 'utf8');
const providerRouter = fs.readFileSync(path.join(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-host', 'src', 'provider_router.rs'), 'utf8');
const runtimeRoot = path.join(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-runtime', 'src');
const runtimeLib = fs.readFileSync(path.join(runtimeRoot, 'lib.rs'), 'utf8');
const conversationActor = fs.readFileSync(path.join(runtimeRoot, 'conversation_actor.rs'), 'utf8');
const runtimeStore = fs.readFileSync(path.join(runtimeRoot, 'runtime_store.rs'), 'utf8');
const capabilityBroker = fs.readFileSync(path.join(runtimeRoot, 'capability_broker.rs'), 'utf8');
const computerExecutor = fs.readFileSync(path.join(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-computer', 'src', 'lib.rs'), 'utf8');
const codexAgent = fs.readFileSync(path.join(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-agent-codex', 'src', 'implementation.rs'), 'utf8');
const featureHost = fs.readFileSync(path.join(repoRoot, 'third_party', 'mahayana', 'mahayana-rs', 'mahayana-feature-host', 'src', 'implementation.rs'), 'utf8');
const durableAgentState = fs.readFileSync(path.join(desktopRoot, 'src', 'durable-agent-state.ts'), 'utf8');
const agentDraftStore = fs.readFileSync(path.join(desktopRoot, 'src', 'agent-workspace', 'agent-draft-store.ts'), 'utf8');
const agentWorkspaceRuntime = fs.readFileSync(path.join(desktopRoot, 'src', 'agent-workspace', 'use-agent-workspace-runtime.ts'), 'utf8');
const removedMigrationRuntimePaths = [
  'grok-chat-parity-runtime.tsx',
  'mahayana-agent-workbench.tsx',
  'mahayana-agent-workbench.module.css',
  'mahayana-agent-inline-report.tsx',
  'mahayana-agent-inline-report.module.css',
  'mahayana-agent-inline-compat.ts',
  'mahayana-agent-transcript-semantics.ts',
  'mahayana-agent-transcript-semantics.css',
].map((name) => path.join(desktopRoot, 'src', name));

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
  ['primary Agent shell directly imports Grok implementation layers', /from\s+['"](?:\.\.\/)+grok-(?:shell|runtime)\//],
  ['Grok-named runtime state leaked back into primary Agent shell', /\bgrok(?:Palette|Network|Pinned|Sidebar|Selected|Activity|Busy|Agent)[A-Z]\w*/],
  ['renderer owns RemoteComputerDesktopController', /\bRemoteComputerDesktopController\b|\bremoteComputerControllerRef\b/],
  ['renderer owns Computer capability state', /setComputerCapabilityStatus\b|setRemoteComputerState\b/],
  ['renderer bypasses Agent Computer controller', /agentCoordinatorClient\.refreshComputerStatus\s*\(|reportOpenComputer/],
  ['renderer bypasses Agent runtime approval facade', /agentCoordinatorClient\.resolveApproval\s*\(/],
  ['renderer bypasses Agent runtime interrupt facade', /agentCoordinatorClient\.interrupt\s*\(/],
  ['renderer bypasses Agent attachment facade', /agentCoordinatorClient\.uploadAttachment\s*\(/],
];

const violations = forbidden
  .filter(([, pattern]) => pattern.test(shell))
  .map(([label]) => label);

function sourceFilesUnder(root) {
  if (!fs.existsSync(root)) return [];
  return fs.readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(root, entry.name);
    if (entry.isDirectory()) return sourceFilesUnder(full);
    return /\.(?:ts|tsx|js|mjs)$/.test(entry.name) ? [full] : [];
  });
}

const compatibilityRoots = [
  path.join(desktopRoot, 'src', 'adapters', 'compatibility'),
  path.join(desktopRoot, 'src', 'features', 'contacts'),
  path.join(desktopRoot, 'src', 'features', 'telegram'),
  path.join(desktopRoot, 'src', 'features', 'miniapps'),
  path.join(desktopRoot, 'src', 'features', 'payments'),
  path.join(desktopRoot, 'src', 'features', 'calls'),
  path.join(desktopRoot, 'src', 'features', 'settings'),
];
for (const file of compatibilityRoots.flatMap(sourceFilesUnder)) {
  const source = fs.readFileSync(file, 'utf8');
  if (/\b(?:AgentSidebar|AgentHeader|AgentNetwork|AgentWorkspace|useAgentWorkspaceRuntime|useAgentProductControllers|useAgentComputerController|AgentRuntimeCoordinator|AgentWorkspaceController)\b/.test(source)) {
    violations.push(`compatibility adapter imports or owns Agent runtime/UI: ${path.relative(desktopRoot, file)}`);
  }
}

for (const file of sourceFilesUnder(path.join(desktopRoot, 'src'))) {
  const source = fs.readFileSync(file, 'utf8');
  if (/\bBotMark\b|FabushiAvatarRuntime|fabushi-avatar-runtime|FabushiBotMarkEngine|from\s+['"][^'"]*bot-mark['"]/.test(source)) {
    violations.push(`desktop bundle still references the legacy avatar runtime: ${path.relative(desktopRoot, file)}`);
  }
}

const primitiveSource = fs.readFileSync(path.join(desktopRoot, 'src', 'ui', 'primitives', 'fab-primitives.tsx'), 'utf8');
for (const primitive of ['FabButton','FabIconButton','FabMenu','FabPopover','FabDialog','FabTooltip','FabSelect','FabBadge','FabAvatar','FabSpinner','FabInput','FabSurface']) {
  if (!new RegExp(`export (?:function|const) ${primitive}\b`).test(primitiveSource)) {
    violations.push(`unified UI primitive missing: ${primitive}`);
  }
}
if (shell.split('\n').length > 4000) {
  violations.push('AgentRootShell grew beyond the architecture budget (4000 lines)');
}

if (!/messenger-compatibility-adapter/.test(shell)
  || !/buildCompatibilityPeers\s*\(/.test(shell)
  || !/compatibilityMessagingEnvelope\s*\(/.test(shell)
  || !/CompatibilitySurface/.test(shell)) {
  violations.push('Contacts/Telegram/Mini App compatibility routing escaped the compatibility adapter');
}
if (/function\s+(?:legacyKind|selfKind)\s*\(|installedMiniAppBotProjections\s*\(/.test(shell)) {
  violations.push('Messenger shell reconstructed compatibility-only peer identity instead of using the adapter');
}
if (/type\s+PeerKind\s*=|type\s+MessengerSection\s*=/.test(shell)) {
  violations.push('Messenger shell recreated compatibility navigation/domain types');
}
if (!/export function buildCompatibilityPeers/.test(compatibilityAdapter)
  || !/export function compatibilityMessagingEnvelope/.test(compatibilityAdapter)
  || !/export function CompatibilitySurface/.test(compatibilityAdapter)) {
  violations.push('compatibility adapter no longer owns peer, event-envelope and secondary-surface routing');
}
if (fs.existsSync(obsoleteShellPath)) {
  violations.push('obsolete messaging-shell-v2.tsx still exists');
}
if (removedMigrationRuntimePaths.some((candidate) => fs.existsSync(candidate))) {
  violations.push('migration-only DOM Agent runtime or portal files still exist');
}
if (/GrokChatParityRuntime|prepareGrokChatParityRuntime|MahayanaAgentWorkbench/.test(mainEntry)) {
  violations.push('migration-only DOM runtimes returned to the desktop product entry');
}
if (/setInterval\s*\(/.test(shell)) {
  violations.push('legacy compatibility adapter reintroduced interval polling');
}
if (/\bBotMark\b/.test(agentTranscript) || /\bBotMark\b/.test(agentOverlays)) {
  violations.push('Agent-owned transcript or overlays regressed to the legacy animated BotMark engine');
}
if (!/if \(isElectronMahayanaHostAvailable\(\)\)/.test(hostClient)
  || /SESSION_POLL_MS/.test(remoteDeviceSupervisor)
  || !/sessionRefreshDelay\s*\(/.test(remoteDeviceSupervisor)
  || !/eventName === ['"]account-state-changed['"]/.test(electronMain)
  || !/remoteDeviceAgentSupervisor\?\.sync\(\)/.test(electronMain)) {
  violations.push('Electron remote computer ownership regressed to Renderer or fixed session polling');
}
if (!/readLegacyAgentWorkspaceDrafts/.test(agentDraftStore)
  || !/clearLegacyAgentWorkspaceDrafts/.test(agentDraftStore)
  || /localStorage\.setItem/.test(agentDraftStore)
  || /AGENT_WORKSPACE_DRAFT_STORAGE_KEY/.test(durableAgentState)
  || !/AGENT_WORKSPACE_DURABLE_DRAFT_KEY/.test(agentWorkspaceRuntime)
  || !/readWorkspaceState\s*\(/.test(agentWorkspaceRuntime)
  || !/writeWorkspaceState\s*\(/.test(agentWorkspaceRuntime)
  || !/normalizePersistedAgentDrafts/.test(agentWorkspaceRuntime)
  || !/read_workspace_state\s*\(/.test(runtimeStore)
  || !/write_workspace_state\s*\(/.test(runtimeStore)) {
  violations.push('Agent drafts are not Rust RuntimeStore-owned with migration-only localStorage fallback');
}
if (!/import\s+AgentRootShell\s+from\s+['"]\.\.\/agent-workspace\/agent-root-shell['"]/.test(desktopApp)
  || !/<AgentRootShell\s*\/>/.test(desktopApp)
  || /LegacyMessagingAdapter|legacy-messaging-shell/.test(desktopApp)
  || !/import\s+DesktopApp\s+from\s+['"]\.\/app\/DesktopApp['"]/.test(mainEntry)
  || !/<DesktopApp\s*\/>/.test(mainEntry)
  || /messaging-shell-v2/.test(mainEntry)
  || /<div\s+hidden\b|hidden\s+aria-hidden=['"]true['"]/.test(shell)) {
  violations.push('DesktopApp/AgentRootShell is not the sole visible product root or hidden legacy navigation returned');
}
if (fs.existsSync(legacyShellPath)) {
  violations.push('legacy-messaging-shell.tsx still exists instead of being deleted');
}
if (!/useAgentWorkspaceRuntime\s*\(/.test(shell)
  || !/useAgentProductControllers\s*\(/.test(shell)
  || !/useAgentComputerController\s*\(/.test(shell)
  || !/<AgentSidebar/.test(shell)
  || !/<AgentWorkspace/.test(shell)
  || !/<AgentNetwork/.test(shell)) {
  violations.push('AgentRootShell no longer directly owns the Agent controllers and three-column Agent product composition');
}

if (!/pub fn authorize_request\s*\(/.test(capabilityBroker)
  || !/append_audit\s*\(/.test(capabilityBroker)
  || !/RuntimeCommand::AuthorizeCapability/.test(runtimeLib)
  || !/capability_broker\s*\.authorize_request\s*\(/.test(runtimeLib)
  || !/!matches!\(decision, CapabilityPolicyDecision::Allow\)/.test(runtimeLib)
  || !/fn authorize_feature_command\s*\(/.test(featureHost)
  || !/self\.authorize_feature_command\(&command\)\?/.test(featureHost)
  || !/CapabilityAvailability::PermissionRequired/.test(featureHost)
  || !/LocalToolPermission::Ask/.test(featureHost)
  || !/computer\.input\.control/.test(featureHost)
  || !/mcp\.tool\.call/.test(featureHost)
  || !/filesystem\.agent\.(?:read|write)/.test(featureHost)
  || !/agent\.handoff/.test(featureHost)) {
  violations.push('privileged FeatureHost operations bypass the Rust CapabilityBroker or fail-open on user approval');
}
if (!/actor\s*\.register\s*\(/.test(runtimeLib)
  || !/actor\.gate\.lock\(\)\.await/.test(runtimeLib)
  || !/actor\s*\.start\s*\(/.test(runtimeLib)
  || !/context\s*\.actor\s*\.set_state\s*\(/.test(runtimeLib)
  || !/actor\s*\.finish\s*\(/.test(runtimeLib)
  || !/turn .* is already registered/.test(conversationActor)
  || !/already has an active run/.test(conversationActor)
  || !/terminal turn .* cannot transition/.test(conversationActor)
  || !/registry_returns_the_same_actor_for_one_conversation_and_isolates_others/.test(conversationActor)
  || !/broker_maps_availability_to_policy_and_audits_every_decision/.test(capabilityBroker)) {
  violations.push('ConversationActor/CapabilityBroker are present as types but their lifecycle and policy execution paths are not structurally enforced');
}

if (!/pub struct ComputerControlLeaseRequest/.test(computerExecutor)
  || !/pub fn execute_with_lease\s*\(/.test(computerExecutor)
  || !/ComputerError::LeaseRequired/.test(computerExecutor)
  || !/ComputerError::LeaseBusy/.test(computerExecutor)
  || !/USER_OVERRIDE_EPOCH\.fetch_add/.test(computerExecutor)
  || !/mahayana_computer::execute_with_lease\s*\(/.test(codexAgent)
  || !/mahayana_computer::release_control_lease\(thread_id, turn_id\)/.test(codexAgent)
  || !/ComputerControlOrigin::RemoteMobile[\s\S]{0,1800}execute_with_lease/.test(featureHost)
  || !/ComputerControlOrigin::Ai[\s\S]{0,1800}execute_with_lease/.test(featureHost)
  || /mahayana_computer::execute\([^\n]*ComputerControlOrigin::Ai/.test(codexAgent)) {
  violations.push('physical Computer control is not enforced by a single controller lease across AI/remote execution paths');
}

if (!/inference_provider:\s*Option<InferenceProvider>/.test(hostProtocol)
  || !/inference_provider:\s*Option<String>/.test(runtimeCore)
  || !/session_providers:\s*AsyncMutex/.test(kernelConversation)
  || !/inferenceProvider/.test(kernelConversation)
  || !/pub struct ProviderRoutingEngineBackend/.test(providerRouter)
  || !/PROVIDER_CODEX/.test(providerRouter)
  || !/PROVIDER_OPENROUTER/.test(providerRouter)
  || !/PROVIDER_CLAUDE_CODE/.test(providerRouter)) {
  violations.push('per-Agent inference provider UI is not backed by the Rust Agent/session/EngineBackend contract');
}
if (!/revision:\s*number/.test(accountSidebarLayout)
  || !/baseEtag:\s*current\.etag/.test(accountSidebarLayout)
  || !/expectAbsent:\s*true/.test(accountSidebarLayout)
  || !/writeAccountAgentStoreObject\s*\(/.test(accountSidebarLayout)
  || !/export function mergeAccountSidebarLayoutState\s*\(/.test(accountSidebarLayout)
  || !/AccountSidebarLayoutWriteOptions/.test(accountSidebarLayout)
  || !/mergeOrderedKeys\s*\(/.test(accountSidebarLayout)
  || !/mergeAccountSidebarLayoutState\(base, localState, current\?\.layout \?\? null\)/.test(accountSidebarLayout)
  || !/cloudSnapshotRef/.test(sidebarController)
  || !/mergeAccountSidebarLayoutState\s*\(/.test(sidebarController)
  || !/readAccountSidebarLayout\(scope\)/.test(sidebarController)
  || /setInterval\s*\(/.test(sidebarController)
  || !/subscribeNativeDesktopEvents\s*\(/.test(sidebarController)
  || !/['"]account-state-changed['"]/.test(sidebarController)
  || !/\{ base: currentBase \}/.test(sidebarController)) {
  violations.push('Agent sidebar sections/pinned order lost cross-device CAS/revision merge convergence');
}

if (!/useAgentWorkspaceRuntime\s*\(/.test(shell)) {
  violations.push('Agent workspace runtime facade is not mounted by the desktop Agent shell');
}
if (!/openConversation:\s*openAgentConversation/.test(shell)
  || !/uploadAttachment:\s*uploadAgentAttachment/.test(shell)
  || !/uploadAgentAttachment\(peer\.key/.test(shell)) {
  violations.push('normal Agent conversation/attachment IO escaped the workspace runtime facade');
}
if (!/useAgentComputerController\s*\(/.test(shell)) {
  violations.push('Agent Computer lifecycle controller is not mounted by the desktop Agent shell');
}
if (/\bnew\s+RemoteComputerDesktopController\b|\bRTCPeerConnection\b|\bSIGNAL_POLL_MS\b|\bSESSION_POLL_MS\b|\bHEARTBEAT_MS\b/.test(computerController)
  || /import\s*\{[^}]*\bRemoteComputerDesktopController\b[^}]*\}\s*from/.test(computerController)
  || !/getRemoteComputerBackgroundState/.test(computerController)
  || !/remote-computer-background-state/.test(computerController)
  || !/class RemoteDeviceAgentSupervisor/.test(remoteDeviceSupervisor)
  || !/sessionRefreshDelay\s*\(/.test(remoteDeviceSupervisor)) {
  violations.push('Agent Computer lifecycle regressed into the React renderer instead of Main');
}
if (/backgroundThrottling:\s*false/.test(electronMain)
  || /HOST_EVENT_LONG_POLL_MS/.test(electronMain)
  || /feature\.receive/.test(electronMain)
  || !/host\.onRuntimeEvent\s*\(/.test(electronMain)) {
  violations.push('desktop power/event architecture regressed to unthrottled renderer or Host polling');
}
if (!/const initialAgentWorkspaceHydrated = hostReady/.test(shell)
  || /hydrated:\s*initialLegacyHydrated/.test(shell)
  || /data-initial-host-hydrated=\{initialLegacyHydrated/.test(shell)
  || /if \(!hostReady \|\| !initialLegacyHydrated\) return;[\s\S]{0,240}agentMcpController\.list/.test(shell)) {
  violations.push('AgentRootShell/Computer/MCP readiness regressed behind legacy Messenger hydration');
}
if (!/useAgentProductControllers\s*\(/.test(shell)
  || !/agentMcpController\s*=\s*agentProductControllers\.mcp/.test(shell)
  || !/agentMcpController\.handle\(event\)/.test(shell)
  || !/useAgentMcpController\s*\(/.test(productControllers)
  || !/projectAgentMcpReferences\s*\(/.test(mcpController)
  || !/event\.type === ['"]mcp\.listed['"]/.test(mcpController)) {
  violations.push('MCP reference discovery escaped the Agent-owned Composer/controller boundary');
}
if (!/agentNetworkController\.handle\(event\)/.test(shell)
  || !/groups=\{agentNetworkController\.groups\}/.test(shell)
  || !/peerMessagesByAgentId=\{agentNetworkController\.peerMessagesByAgentId\}/.test(shell)
  || !/onSendPeer=\{agentNetworkController\.sendPeer\}/.test(shell)) {
  violations.push('Agent Network state or direct handoff escaped the Agent-owned controller boundary');
}
if (/from\s+['"](?:\.\.\/)+grok-shell\//.test(agentComposer)) {
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
if (!/AGENT_ATTACHMENT_LIMIT/.test(agentComposer)
  || !/const stageFiles\s*=/.test(agentComposer)
  || !/onPasteFiles=\{stageFiles\}/.test(agentComposer)) {
  violations.push('Agent Composer no longer caps file selection/drop/paste at the Agent attachment boundary');
}
if (!/voiceState === ['"]idle['"]/.test(agentComposer)
  || !/if \(canSend\) formRef\.current\?\.requestSubmit\(\)/.test(agentComposer)) {
  violations.push('Agent Composer can submit while voice capture/transcription is active');
}
if (!/event\.isComposing/.test(agentComposer)
  || !/editorControlsRef\.current\?\.blur\(\)/.test(agentComposer)
  || !/editorControlsRef\.current\?\.insertText\(transcript\)/.test(agentComposer)) {
  violations.push('Agent Composer keyboard/voice editor contract drifted from the frozen Fabu reference');
}
if (!/insertText\(value: string\): void/.test(agentRichEditor)
  || !/editor\.chain\(\)\.focus\(\)\.insertContent/.test(agentRichEditor)) {
  violations.push('Agent rich editor no longer exposes cursor-preserving text insertion');
}

if (!/agentSidebarController\s*=\s*agentProductControllers\.sidebar/.test(shell)
  || !/useAgentSidebarController\s*\(/.test(productControllers)) {
  violations.push('Agent sidebar controller escaped the Agent product-controller boundary');
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

if (!/import\s+AgentNetwork\s+from\s+['"](?:\.\.\/)+agent-workspace\/agent-network['"]/.test(shell)
  || !/<AgentNetwork\b/.test(shell)) {
  violations.push('primary shell is not mounting the Agent-owned Network surface');
}

if (!/import\s+AgentCommandPalette\s+from\s+['"](?:\.\.\/)+agent-workspace\/agent-command-palette['"]/.test(shell)
  || !/<AgentCommandPalette\b/.test(shell)) {
  violations.push('primary shell is not mounting the Agent-owned command palette boundary');
}

if (!/agentPaletteController\s*=\s*agentProductControllers\.palette/.test(shell)
  || !/useAgentCommandPaletteController\s*\(/.test(productControllers)) {
  violations.push('command palette lifecycle escaped the Agent product-controller boundary');
}
if (/setGrokPalette|agentPaletteOpen|agentPaletteQuery/.test(shell)) {
  violations.push('primary shell recreated command palette runtime state');
}

if (!/from\s+['"](?:\.\.\/)+agent-workspace\/agent-model['"]/.test(shell)
  || !/projectAgentSidebarItems\s*\(/.test(shell)
  || !/projectActiveAgentKey\s*\(/.test(shell)) {
  violations.push('primary shell is not consuming the Agent-owned navigation projection model');
}
if (!/agentNetworkController\s*=\s*agentProductControllers\.network/.test(shell)
  || !/useAgentNetworkController\s*\(/.test(productControllers)) {
  violations.push('Agent Network UI/controller state escaped the Agent product-controller boundary');
}
if (/from\s+['"][^'"]*use-agent-(?:command-palette|sidebar|network|workflow|mcp|store-sync|directory)-controller['"]/.test(shell)) {
  violations.push('compatibility shell directly constructs Agent product controllers');
}
if (/agentCoordinatorClient\.(?:listGroups|createGroup|updateGroup|deleteGroup|sendGroup|broadcast)\s*\(/.test(shell)) {
  violations.push('primary shell directly owns Agent collaboration commands');
}
for (const method of ['listGroups', 'createGroup', 'updateGroup', 'deleteGroup', 'sendGroup', 'broadcast']) {
  if (!new RegExp(`client\\.${method}\\s*\\(`).test(networkController)) {
    violations.push(`Agent Network controller no longer routes ${method} through AgentCoordinatorClient`);
  }
}

if (!/agentWorkflowController\s*=\s*agentProductControllers\.workflow/.test(shell)
  || !/useAgentWorkflowController\s*\(/.test(productControllers)) {
  violations.push('Agent workflow discovery escaped the Agent product-controller boundary');
}
if (/agentWorkflowsById|setAgentWorkflowsById|type:\s*['"]workflow\.list['"]/.test(shell)) {
  violations.push('primary shell recreated Agent workflow cache or raw workflow.list ownership');
}
if (!/client\.listWorkflows\s*\(/.test(workflowController)
  || !/event\.type === ['"]workflow\.listed['"]/.test(workflowController)
  || !/event\.type === ['"]workflow\.changed['"]/.test(workflowController)) {
  violations.push('Agent workflow controller no longer owns workflow list/cache refresh');
}

if (!/agentStoreSyncController\s*=\s*agentProductControllers\.storeSync/.test(shell)
  || !/useAgentStoreSyncController\s*\(/.test(productControllers)) {
  violations.push('Agent memory/automation CAS sync escaped the Agent product-controller boundary');
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

if (!/useAgentSettingsController\s*\(/.test(shell)) {
  violations.push('Agent settings mutation/generation state escaped the Agent settings controller');
}
if (/function\s+(?:updateActiveAgentProfile|setActiveAgentNotifications)\s*\(/.test(shell)) {
  violations.push('primary shell recreated Agent settings mutations');
}

if (!/agentDirectoryController\s*=\s*agentProductControllers\.directory/.test(shell)
  || !/useAgentDirectoryController\s*\(/.test(productControllers)) {
  violations.push('Agent directory cache/commands escaped the Agent product-controller boundary');
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

if (!/pub enum TurnState/.test(runtimeCore)
  || !/TurnStateChanged/.test(runtimeCore)
  || !/struct ConversationActorState/.test(conversationActor)
  || !/pub struct ConversationActorRegistry/.test(conversationActor)
  || !/AsyncMutex/.test(conversationActor)
  || !/pub fn actor\s*\(&self, conversation_id: &ConversationId\)/.test(conversationActor)
  || !/PRAGMA journal_mode=WAL/.test(runtimeStore)
  || !/CREATE TABLE IF NOT EXISTS turns/.test(runtimeStore)
  || !/CREATE TABLE IF NOT EXISTS runs/.test(runtimeStore)
  || !/CREATE TABLE IF NOT EXISTS workspace_state/.test(runtimeStore)
  || !/CREATE TABLE IF NOT EXISTS capability_audit/.test(runtimeStore)
  || !/CREATE TABLE IF NOT EXISTS computer_leases/.test(runtimeStore)
  || !/pub fn acquire_computer_lease/.test(runtimeStore)
  || !/pub struct CapabilityBroker/.test(capabilityBroker)
  || !/pub fn authorize_request\s*\(/.test(capabilityBroker)
  || !/\.capability_broker[\s\S]{0,160}\.authorize_request\s*\(/.test(runtimeLib)
  || !/\.actors[\s\S]{0,100}\.actor\(&conversation_id\)/.test(runtimeLib)
  || !/actor\.gate\.lock\(\)\.await/.test(runtimeLib)
  || !/transition_turn_state/.test(runtimeLib)) {
  violations.push('Mahayana Rust runtime lost ConversationActor/Turn/Run/CapabilityBroker/SQLite ownership');
}

if (violations.length) {
  console.error('Agent workspace boundary regression detected:');
  for (const violation of violations) console.error(`- ${violation}`);
  process.exitCode = 1;
} else {
  console.log('Agent workspace boundary check passed.');
}