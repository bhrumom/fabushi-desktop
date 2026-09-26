import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(desktopRoot, "..");
const violations = [];

const read = (...parts) => fs.readFileSync(path.join(...parts), "utf8");
const exists = (...parts) => fs.existsSync(path.join(...parts));

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

for (const legacyRoot of [
  ["desktop", "src"],
  ["desktop", "electron"],
  ["frontend", "apps", "web"],
  ["third_party", "mahayana"],
]) {
  if (exists(repoRoot, ...legacyRoot)) {
    violations.push(`forbidden legacy runtime root still exists: ${legacyRoot.join("/")}`);
  }
}

const indexHtml = read(desktopRoot, "index.html");
const rendererEntry = read(repoRoot, "frontend", "src", "main.tsx");
const productionRenderer = read(repoRoot, "frontend", "src", "production", "ProductionRenderer.tsx");
const electronBuild = read(desktopRoot, "scripts", "build-electron-runtime.mjs");
const electronMain = read(repoRoot, "source", "electron-main", "entry.ts");
const preloadPrimary = read(repoRoot, "source", "electron-preload", "runtime", "primary.ts");
const coordinatorCarrier = read(
  repoRoot,
  "source",
  "electron-main",
  "coordinator",
  "rust-coordinator-carrier.ts",
);
const coordinator = read(repoRoot, "source", "node-agent-coordinator", "src", "main.rs");
const host = read(repoRoot, "source", "host", "app", "src", "main.rs");
const runtimeRoot = path.join(
  repoRoot,
  "source",
  "mahayana",
  "mahayana-rs",
  "mahayana-runtime",
  "src",
);
const runtimeLib = read(runtimeRoot, "lib.rs");
const actor = read(runtimeRoot, "conversation_actor.rs");
const broker = read(runtimeRoot, "capability_broker.rs");
const runtimeStore = read(runtimeRoot, "runtime_store.rs");
const computerRuntime = read(
  repoRoot,
  "source",
  "mahayana",
  "mahayana-rs",
  "mahayana-computer",
  "src",
  "lib.rs",
);

requirePattern(
  "Desktop renderer entry must be frontend/src/main.tsx",
  indexHtml,
  /<script\s+type=["']module["']\s+src=["']\.\.\/frontend\/src\/main\.tsx["']/,
);
requirePattern(
  "Renderer entry must mount ProductionRenderer",
  rendererEntry,
  /import\s+\{\s*ProductionRenderer\s*\}[\s\S]{0,500}<ProductionRenderer\b/,
);
requirePattern(
  "Production Renderer must consume the Grok conversation composer",
  productionRenderer,
  /\bConversationComposer\b/,
);
requirePattern(
  "Production Renderer must consume the Grok conversation transcript",
  productionRenderer,
  /\bConversationTranscript\b/,
);
forbidPattern(
  "Production Renderer must not depend on deleted legacy runtime roots",
  productionRenderer,
  /desktop\/src|desktop\/electron|frontend\/apps\/web|third_party\/mahayana/,
);

requirePattern(
  "Electron build must compile source/electron-main/entry.ts",
  electronBuild,
  /source["'],\s*["']electron-main["'],\s*["']entry\.ts["']/,
);
requirePattern(
  "Electron build must compile source/electron-preload entries",
  electronBuild,
  /source["'],\s*["']electron-preload["']/,
);
requirePattern(
  "Electron build must compile the Rust Coordinator carrier",
  electronBuild,
  /rust-coordinator-carrier\.ts/,
);
forbidPattern(
  "Electron build must not compile a deleted legacy runtime",
  electronBuild,
  /desktop\/electron|desktop\/src|frontend\/apps\/web|third_party\/mahayana/,
);

requirePattern(
  "Electron Main must start the production main composition",
  electronMain,
  /startElectronMainProduction\s*\(/,
);
requirePattern(
  "Electron Main must bind the independent Coordinator",
  electronMain,
  /createElectronProductionCoordinatorBinding\s*\(/,
);
requirePattern(
  "Electron Main must compose Coordinator IPC without absorbing Coordinator ownership",
  electronMain,
  /composeElectronProductionCoordinatorBindings\s*\(/,
);
forbidPattern(
  "Electron Main must not own Rust Host or Runner implementation",
  electronMain,
  /struct\s+CoordinatorState|HostRunnerComposition|ConversationActorRegistry|CapabilityBroker/,
);

requirePattern(
  "Primary preload must remain a thin production bridge",
  preloadPrimary,
  /installPrimaryPreloadEntrypoint\s*\(loadPrimaryPreloadElectron\(require\(["']electron["']\)\)\)/,
);
forbidPattern(
  "Primary preload must not own Coordinator, Host, or Runner logic",
  preloadPrimary,
  /GatewayHostSupervisor|HostRunnerComposition|ConversationActorRegistry|CapabilityBroker|spawn\s*\(/,
);

for (const channel of [
  "coordinator-control",
  "coordinator-data",
  "coordinator-main-data",
]) {
  if (!coordinatorCarrier.includes(`"${channel}"`)) {
    violations.push(`Rust Coordinator carrier is missing channel ${channel}`);
  }
}
requirePattern(
  "Rust Coordinator carrier must use Electron utilityProcess process.parentPort",
  coordinatorCarrier,
  /process\.parentPort/,
);
requirePattern(
  "Rust Coordinator carrier must spawn the independent Coordinator binary",
  coordinatorCarrier,
  /spawn\s*\([\s\S]{0,500}mahayana-node-agent-coordinator/,
);
forbidPattern(
  "Rust Coordinator carrier must not import Host/Runner implementation",
  coordinatorCarrier,
  /HostRunnerComposition|ConversationActorRegistry|CapabilityBroker/,
);

for (const [label, pattern] of [
  ["Coordinator must own an independent CoordinatorState", /struct\s+CoordinatorState/],
  ["Coordinator must own Gateway host supervision", /GatewayHostSupervisor/],
  ["Coordinator must own renderer-port request\/reply\/event serving", /RendererPortServer/],
  ["Coordinator must own MCP\/OAuth forwarding state", /McpOAuthForwarderState/],
  ["Coordinator must own local-exec supervision", /LocalExecDaemonRuntime/],
  ["Coordinator must track Host generations", /host_generation:\s*AtomicU64/],
  ["Coordinator must track crash settlement", /consecutive_crashes:\s*AtomicU64/],
  ["Coordinator must own active inference stream routing", /active_inference_streams:\s*ActiveInferenceStreamRegistry/],
]) {
  requirePattern(label, coordinator, pattern);
}
forbidPattern(
  "Coordinator must not collapse Host Runner composition into the Coordinator process",
  coordinator,
  /HostRunnerComposition::production/,
);

requirePattern(
  "Shipping Host must compose the production Runner",
  host,
  /HostRunnerComposition::production\s*\(/,
);
requirePattern(
  "Shipping Host must own the authenticated Gateway server",
  host,
  /start_gateway_server\s*\(/,
);
requirePattern(
  "Shipping Host must own Local Exec extension composition",
  host,
  /start_local_exec_extension\s*\(/,
);
forbidPattern(
  "Shipping Host must not absorb the Coordinator renderer-port server",
  host,
  /RendererPortServer|CoordinatorState/,
);
if (!exists(repoRoot, "source", "host", "src", "runner")) {
  violations.push("source/host/src/runner is missing; Runner must remain an independent Host responsibility");
}
if (!exists(repoRoot, "source", "node-agent-coordinator")) {
  violations.push("source/node-agent-coordinator is missing");
}
if (!exists(repoRoot, "source", "mahayana", "mahayana-rs")) {
  violations.push("source/mahayana/mahayana-rs is missing");
}

requirePattern(
  "ConversationActor registry is missing from the production Mahayana runtime",
  actor,
  /pub\s+struct\s+ConversationActorRegistry/,
);
requirePattern(
  "ConversationActor must serialize per-conversation execution",
  actor,
  /AsyncMutex/,
);
requirePattern(
  "CapabilityBroker is missing from the production Mahayana runtime",
  broker,
  /pub\s+struct\s+CapabilityBroker/,
);
requirePattern(
  "CapabilityBroker must own authorization decisions",
  broker,
  /pub\s+fn\s+authorize_request\s*\(/,
);
requirePattern(
  "CapabilityBroker decisions must be durably audited",
  broker,
  /append_audit\s*\(/,
);

for (const table of [
  "turns",
  "runs",
  "turn_requests",
  "handoff_dispatch",
  "capability_audit",
  "computer_leases",
]) {
  if (!runtimeStore.includes(`CREATE TABLE IF NOT EXISTS ${table}`)) {
    violations.push(`RuntimeStore is missing durable table: ${table}`);
  }
}
requirePattern(
  "RuntimeStore must use SQLite WAL durability",
  runtimeStore,
  /PRAGMA\s+journal_mode=WAL/,
);
requirePattern(
  "Runtime must resolve ConversationActor before execution",
  runtimeLib,
  /\.actors[\s\S]{0,200}\.actor\(&conversation_id\)/,
);
requirePattern(
  "Runtime must lock the ConversationActor execution gate",
  runtimeLib,
  /actor\.gate\.lock\(\)\.await/,
);
requirePattern(
  "Runtime must persist logical turn state transitions",
  runtimeLib,
  /transition_turn_state/,
);
requirePattern(
  "Runtime startup must recover interrupted user turns",
  runtimeLib,
  /recover_interrupted_turns\(\)/,
);
requirePattern(
  "Runtime startup must recover durable handoffs",
  runtimeLib,
  /recover_pending_handoffs\(\)/,
);
requireOrdered(
  "Runtime startup must recover interrupted turns before durable handoffs",
  runtimeLib,
  ["recover_interrupted_turns()?", "recover_pending_handoffs()?"],
);
requirePattern(
  "Waiting-user restart recovery must fail closed",
  runtimeLib,
  /TurnState::WaitingUser[\s\S]{0,1800}explicit retry is required/,
);
requirePattern(
  "Capability execution must fail closed for NeedsUser",
  runtimeLib,
  /CapabilityPolicyDecision::NeedsUser\s*=>\s*Err\(/,
);
requirePattern(
  "Capability execution must fail closed for Deny",
  runtimeLib,
  /CapabilityPolicyDecision::Deny\s*=>\s*Err\(/,
);
requirePattern(
  "ComputerControlLease execution owner is missing",
  computerRuntime,
  /pub\s+fn\s+execute_with_lease\s*\(/,
);
requirePattern(
  "Computer control must maintain one physical lease owner",
  computerRuntime,
  /CONTROL_LEASE:\s*LazyLock<Mutex<Option<ComputerControlLeaseSnapshot>>>/,
);

if (violations.length > 0) {
  console.error("Grok 0.18 production architecture boundary regression detected:");
  for (const violation of [...new Set(violations)]) console.error(`- ${violation}`);
  process.exitCode = 1;
} else {
  console.log(
    "Grok 0.18 production architecture boundary passed: Renderer -> Electron -> Coordinator -> Host -> Runner with durable Mahayana ownership.",
  );
}
