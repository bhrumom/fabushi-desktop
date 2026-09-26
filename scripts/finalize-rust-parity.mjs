#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifestPath = path.join(root, "projects/grok-fabu-parity/architecture-manifest.json");
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));

const runnerContract = "source/host/tests/runner_contract.rs";
const utilityContract = "source/host/tests/runner_small_modules_contract.rs";
const specializedAutoReviewContract = "source/host/tests/sand_auto_review_specialized_contract.rs";
const extensionContract = "source/host/tests/extensions_turn_execution_contract.rs";
const localExecContract = "source/host/tests/extensions_local_exec_contract.rs";
const hostFoundationContract = "source/host/tests/host_foundation_contract.rs";
const hostStorageContract = "source/host/tests/host_storage_contract.rs";
const grokSmallFoundationContract = "source/host/tests/grok_small_foundation_parity_contract.rs";
const manualFinalizationRequired = new Set([
  "source/host/runner/stream-attempt.ts",
  "source/host/runner/turn-settle.ts",
]);

const completed = new Map([
  ["source/host/runner/sand-auto-review-tool-escalations.ts", ["source/host/src/runner/sand_auto_review_tool_escalations.rs", specializedAutoReviewContract]],
  ["source/host/runner/sand-automation-auto-review.ts", ["source/host/src/runner/sand_automation_auto_review.rs", specializedAutoReviewContract]],
  ["source/host/runner/sand-browser-auto-review.ts", ["source/host/src/runner/sand_browser_auto_review.rs", specializedAutoReviewContract]],
  ["source/host/runner/sand-cloud-agent-auto-review.ts", ["source/host/src/runner/sand_cloud_agent_auto_review.rs", specializedAutoReviewContract]],
  ["source/host/runner/sand-computer-auto-review.ts", ["source/host/src/runner/sand_computer_auto_review.rs", specializedAutoReviewContract]],
  ["source/host/runner/sand-shell-auto-review-enrichment.ts", ["source/host/src/runner/sand_shell_auto_review_enrichment.rs", specializedAutoReviewContract]],
  ["source/host/runner/sand-subagent-auto-review.ts", ["source/host/src/runner/sand_subagent_auto_review.rs", specializedAutoReviewContract]],
  ["source/host/ports/product-analytics.ts", ["source/host/src/ports/product_analytics.rs", grokSmallFoundationContract]],
  ["source/host/ports/sand-analytics-types.ts", ["source/host/src/ports/sand_analytics_types.rs", grokSmallFoundationContract]],
  ["source/host/transcript-mutation-events.ts", ["source/host/src/transcript_mutation_events.rs", grokSmallFoundationContract]],
  ["source/host/selected-image-inputs.ts", ["source/host/src/selected_image_inputs.rs", grokSmallFoundationContract]],
  ["source/host/extensions/webauthn-proxy/webauthn-proxy-marker.ts", ["source/host/src/extensions/webauthn_proxy/webauthn_proxy_marker.rs", grokSmallFoundationContract]],
  ["source/host/notify-drain-gate.ts", ["source/host/src/notify_drain_gate.rs", hostStorageContract]],
  ["source/host/storage/agent-paths.ts", ["source/host/src/storage/agent_paths.rs", hostStorageContract]],
  ["source/host/agents/settings-file.ts", ["source/host/src/agents/settings_file.rs", hostStorageContract]],
  ["source/host/extensions/session/conversation-blobs-path.ts", ["source/host/src/extensions/session/conversation_blobs_path.rs", hostStorageContract]],
  ["source/host/extensions/box-store-sync/box-store-sync-error.ts", ["source/host/src/extensions/box_store_sync/box_store_sync_error.rs", hostStorageContract]],
  ["source/host/extensions/box-store-sync/box-store-diagnostics.ts", ["source/host/src/extensions/box_store_sync/box_store_diagnostics.rs", hostStorageContract]],
  ["source/host/extensions/cloud-agents/cloud-agent-launch-error.ts", ["source/host/src/extensions/cloud_agents/cloud_agent_launch_error.rs", hostStorageContract]],
  ["source/host/extensions/transcript/channel-delivery-unregistered-error.ts", ["source/host/src/extensions/transcript/channel_delivery_unregistered_error.rs", hostStorageContract]],
  ["source/host/extensions/transcript/send-not-persisted-error.ts", ["source/host/src/extensions/transcript/send_not_persisted_error.rs", hostStorageContract]],
  ["source/host/sand-quiet-work-origin.ts", ["source/host/src/sand_quiet_work_origin.rs", hostFoundationContract]],
  ["source/host/sha256.ts", ["source/host/src/sha256.rs", hostFoundationContract]],
  ["source/host/storage/folder-id.ts", ["source/host/src/storage/folder_id.rs", hostFoundationContract]],
  ["source/host/host-diagnostics.ts", ["source/host/src/host_diagnostics.rs", hostFoundationContract]],
  ["source/host/box/box-monitor-layout.ts", ["source/host/src/box/box_monitor_layout.rs", hostFoundationContract]],
  ["source/host/automations/automation-id.ts", ["source/host/src/automations/automation_id.rs", hostFoundationContract]],
  ["source/host/attachment-paths.ts", ["source/host/src/attachment_paths.rs", hostFoundationContract]],
  ["source/host/ports/user-computer.ts", ["source/host/src/ports/user_computer.rs", hostFoundationContract]],
  ["source/host/ports/transport.ts", ["source/host/src/ports/transport.rs", hostFoundationContract]],
  ["source/host/durable-file-policy.ts", ["source/host/src/durable_file_policy.rs", hostFoundationContract]],
  ["source/host/sand-user-identity.ts", ["source/host/src/sand_user_identity.rs", hostFoundationContract]],
  ["source/host/extensions/local-exec/local-exec-error.ts", ["source/host/src/extensions/local_exec/local_exec_error.rs", localExecContract]],
  ["source/host/extensions/local-exec/local-exec-failure-classifier.ts", ["source/host/src/extensions/local_exec/local_exec_failure_classifier.rs", localExecContract]],
  ["source/host/extensions/extension-ids.generated.ts", ["source/host/src/extensions/extension_ids.generated.rs", extensionContract]],
  ["source/host/extensions/registry.ts", ["source/host/src/extensions/registry.rs", extensionContract]],
  ["source/host/extensions/turn-execution/extension.ts", ["source/host/src/extensions/turn_execution/extension.rs", extensionContract]],
  ["source/host/extensions/turn-execution/turn-execution-service.ts", ["source/host/src/extensions/turn_execution/turn_execution_service.rs", extensionContract]],
  ["source/host/runner/conversation-state.ts", ["source/host/src/runner/conversation_state.rs", runnerContract]],
  ["source/host/runner/tool-call-identity.ts", ["source/host/src/runner/tool_call_identity.rs", runnerContract]],
  ["source/host/runner/transient-stream-error.ts", ["source/host/src/runner/transient_stream_error.rs", runnerContract]],
  ["source/host/runner/turn-run-shell.ts", ["source/host/src/runner/turn_run_shell.rs", runnerContract]],
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

for (const referencePath of manualFinalizationRequired) {
  if (completed.has(referencePath)) {
    throw new Error(
      `manual-finalization row cannot be auto-promoted by cargo coverage alone: ${referencePath}`
    );
  }
}

for (const row of manifest.modules) {
  if (
    row.status === "existing-needs-parity"
    && completed.has(row.referencePath)
  ) {
    throw new Error(
      `auto-finalizer cannot promote an architecture row that is explicitly existing-needs-parity: ${row.referencePath}`
    );
  }
}

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
const remaining = manifest.modules
  .filter((row) => row.status === "planned" || row.status === "existing-needs-parity")
  .map((row) => ({
    referencePath: row.referencePath,
    targetPath: row.targetPath,
    targetLanguage: row.targetLanguage,
    processOrPackage: row.processOrPackage,
    status: row.status,
  }));
console.log(JSON.stringify({ finalized: touched.length, counts, touched, remaining }, null, 2));
