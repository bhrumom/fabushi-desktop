#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifestPath = path.join(root, "projects/grok-fabu-parity/architecture-manifest.json");
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));

const runnerContract = "source/host/tests/runner_contract.rs";
const utilityContract = "source/host/tests/runner_small_modules_contract.rs";
const extensionContract = "source/host/tests/extensions_turn_execution_contract.rs";
const completed = new Map([
  ["source/host/extensions/extension-ids.generated.ts", ["source/host/src/extensions/extension_ids_generated.rs", extensionContract]],
  ["source/host/extensions/registry.ts", ["source/host/src/extensions/registry.rs", extensionContract]],
  ["source/host/extensions/turn-execution/extension.ts", ["source/host/src/extensions/turn_execution/extension.rs", extensionContract]],
  ["source/host/extensions/turn-execution/turn-execution-service.ts", ["source/host/src/extensions/turn_execution/turn_execution_service.rs", extensionContract]],
  ["source/host/runner/conversation-state.ts", ["source/host/src/runner/conversation_state.rs", runnerContract]],
  ["source/host/runner/stream-attempt.ts", ["source/host/src/runner/stream_attempt.rs", runnerContract]],
  ["source/host/runner/tool-call-identity.ts", ["source/host/src/runner/tool_call_identity.rs", runnerContract]],
  ["source/host/runner/transient-stream-error.ts", ["source/host/src/runner/transient_stream_error.rs", runnerContract]],
  ["source/host/runner/turn-run-shell.ts", ["source/host/src/runner/turn_run_shell.rs", runnerContract]],
  ["source/host/runner/turn-settle.ts", ["source/host/src/runner/turn_settle.rs", runnerContract]],
  ["source/host/runner/turn-usage.ts", ["source/host/src/runner/turn_usage.rs", runnerContract]],
  ["source/host/runner/agent-state.ts", ["source/host/src/runner/agent_state.rs", utilityContract]],
  ["source/host/runner/clock-skew-guard.ts", ["source/host/src/runner/clock_skew_guard.rs", utilityContract]],
  ["source/host/runner/sand-prompt-markers.ts", ["source/host/src/runner/sand_prompt_markers.rs", utilityContract]],
  ["source/host/runner/site-visit-tracking.ts", ["source/host/src/runner/site_visit_tracking.rs", utilityContract]],
  ["source/host/runner/video-container.ts", ["source/host/src/runner/video_container.rs", utilityContract]],
  ["source/host/runner/tools/mcp-server-resolution.ts", ["source/host/src/runner/tools/mcp_server_resolution.rs", utilityContract]],
  ["source/host/runner/tools/sand-permission-request.ts", ["source/host/src/runner/tools/sand_permission_request.rs", utilityContract]],
  ["source/host/runner/tools/sand-secret-request.ts", ["source/host/src/runner/tools/sand_secret_request.rs", utilityContract]],
  ["source/host/runner/tools/tool-input-error.ts", ["source/host/src/runner/tools/tool_input_error.rs", utilityContract]],
]);

const touched = [];
for (const row of manifest.modules) {
  const mapping = completed.get(row.referencePath);
  if (!mapping) continue;
  const [targetPath, testPath] = mapping;
  if (row.targetPath !== targetPath) {
    throw new Error(`target path mismatch for ${row.referencePath}: manifest=${row.targetPath}, expected=${targetPath}`);
  }
  for (const evidence of [targetPath, testPath]) {
    if (!fs.existsSync(path.join(root, evidence))) {
      throw new Error(`missing Rust parity evidence ${evidence} for ${row.referencePath}`);
    }
  }
  row.status = "implemented";
  row.parityMode = "rust-port";
  row.behavioralEvidence = [targetPath];
  row.testEvidence = [testPath, ".github/workflows/grok-rust-parity-finalize.yml"];
  row.notes = "Rust translation of the frozen Grok module with executable cargo contract coverage; finalized only after the parity workflow cargo test succeeds.";
  touched.push(row.referencePath);
}
if (touched.length !== completed.size) {
  const missing = [...completed.keys()].filter((key) => !touched.includes(key));
  throw new Error(`manifest is missing curated Rust parity rows: ${missing.join(", ")}`);
}
fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + "\n");
const counts = {};
for (const row of manifest.modules) counts[row.status] = (counts[row.status] ?? 0) + 1;
console.log(JSON.stringify({ finalized: touched.length, counts, touched }, null, 2));
