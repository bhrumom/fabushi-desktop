import hashlib, json, os, pathlib, re, subprocess
validation_scope = os.environ.get("OCRM_PORTABLE_SCOPE", "full")
if validation_scope not in ("full", "custody-recovery", "background-custody", "background-custody-recovery"):
    raise ValueError("Unknown portable contract scope")
root = (pathlib.Path(".evidence") / validation_scope if validation_scope != "full"
    else pathlib.Path(".evidence/sqlite-native-path/channel-pending-recovery"))
root.mkdir(parents=True, exist_ok=True)
head = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
assert head == os.environ["FABUSHI_TARGET_SHA"], "Exact CWR execution subject mismatch"
palette_fixture = pathlib.Path(os.environ["OCRM_PALETTE_FIXTURE_PATH"])
palette_fixture.parent.mkdir(parents=True, exist_ok=True)
assert not palette_fixture.exists(), "PMS export must be produced freshly by this exact-H Rust execution"
lock = json.loads(pathlib.Path("projects/openclaw-rust/upstream.lock.json").read_text())
inputs = [
    "projects/openclaw-rust/docs/specs/custody-recovery-verification.md",
    ".github/workflows/rust-desktop-runtime.yml",
    "projects/openclaw-rust/scripts/capture_portable_contracts.py",
    "projects/openclaw-rust/docs/specs/channel-outbound-host-lane.md",
    "projects/openclaw-rust/docs/specs/memory-gateway-selector-boundary.md",
    "projects/openclaw-rust/docs/specs/device-pairing-join-admission.md",
    "source/host/tests/device_pairing_join_admission_contract.rs",
    "source/host/src/gateway_command_error.rs",
    "source/host/tests/gateway_server_contract.rs",
    "projects/openclaw-rust/docs/specs/memory-append-read-integrity.md",
    "projects/openclaw-rust/docs/specs/channel-inbound-worker-handoff.md",
    "source/host/Cargo.toml", "source/host/Cargo.lock",
    "projects/openclaw-rust/docs/specs/plugin-skills-snapshot-admission.md",
    "source/host/src/extensions/mcp/mcp_service.rs",
    "source/host/src/extensions/mcp/production.rs",
    "source/host/src/extensions/mcp/coordinator_relay.rs",
    "source/host/src/extensions/mcp/plugin_skills.rs",
    "source/host/src/extensions/mcp/plugin_skills_cache.rs",
    "source/host/src/extensions/mcp/plugin_skills_uninstall_state.rs",
    "source/host/src/extensions/mcp/plugin_skills_sync_tests.rs",
    "source/host/src/extensions/mcp/plugin_skills_auth_recovery_tests.rs",
    "source/host/src/host_production_extensions.rs",
    "source/host/src/workflows/workflow_store.rs",
    "source/host/tests/mcp_service_contract.rs",
    "source/host/tests/plugin_skills_contract.rs",
    "source/host/tests/host_bundle_and_plugin_cache_contract.rs",
    "source/host/tests/workflow_store_contract.rs",
    "source/host/tests/workflow_commands_contract.rs",
    "source/host/tests/legacy_live_references_contract.rs",
    "source/host/tests/mcp_coordinator_relay_contract.rs",
    "source/host/tests/mcp_extension_contract.rs",
    "source/host/tests/mcp_production_contract.rs",
    "source/host/tests/host_production_extensions_contract.rs",
    "projects/openclaw-rust/docs/specs/roster-search-unicode-match.md",
    "source/host/src/extensions/content_search/agent_content_search.rs",
    "source/host/src/extensions/transcript/roster_search.rs",
    "source/host/src/extensions/session/gateway.rs",
    "source/host/tests/agent_content_search_contract.rs",
    "source/host/tests/roster_search_contract.rs",
    "frontend/src/production/command-palette-message-provider.ts",
    "frontend/src/production/command-palette-model.ts",
    "projects/openclaw-rust/docs/specs/palette-rust-message-presentation.md",
    "source/host/tests/palette_message_gateway_contract.rs",
    "frontend/src/production/command-palette-link-provider.ts",
    "frontend/src/production/command-palette-message-search.contract.test.ts",
    "frontend/src/production/CommandPalette.tsx",
    "frontend/src/production/ProductionRenderer.tsx",
    "desktop/package.json", "desktop/package-lock.json", "desktop/tsconfig.json",
    "source/host/src/extensions/transcript/sand_pending_wake_store.rs",
    "source/host/src/extensions/transcript/pending_wake_rearm.rs",
    "source/host/src/extensions/transcript/production_runtime.rs",
    "source/host/src/extensions/transcript/runner_registry.rs",
    "projects/openclaw-rust/docs/specs/provider-bound-model-identity.md",
    "source/host/src/extensions/transcript/transcript_manager.rs",
    "source/host/src/extensions/session/production.rs",
    "projects/openclaw-rust/docs/specs/session-owner-namespace-isolation.md",
    "source/host/src/extensions/session/session_paths.rs",
    "source/host/src/extensions/session/agent_session.rs",
    "source/host/src/extensions/session/session_materialization.rs",
    "source/host/src/extensions/session/session_roster.rs",
    "source/host/src/extensions/transcript/agent_lifecycle.rs",
    "source/host/tests/session_paths_contract.rs",
    "source/host/tests/agent_session_contract.rs",
    "source/host/tests/agent_lifecycle_contract.rs",
    "source/host/src/extensions/memory/memory_service.rs",
    "source/host/src/extensions/memory/memory_synthesis_service.rs",
    "source/host/src/extensions/memory/production.rs",
    "source/host/src/extensions/memory/production_append_read_tests.rs",
    "source/host/src/watched_directory.rs",
    "source/host/src/storage/agent_paths.rs", "source/host/src/storage/folder_id.rs",
    "source/host/src/device_pairing_owner.rs",
    "source/host/tests/device_pairing_contract.rs",
    "source/host/tests/device_pairing_rotation_authority_contract.rs",
    "source/host/tests/device_pairing_rotation_gateway_contract.rs",
    "projects/openclaw-rust/docs/specs/device-pairing-authenticated-rotation.md",
    "source/host/tests/device_pairing_revocation_authority_contract.rs",
    "source/host/tests/device_pairing_revocation_gateway_contract.rs",
    "source/host/tests/device_role_token_revocation_authority_contract.rs",
    "source/host/tests/device_role_token_revocation_gateway_contract.rs",
    "source/host/tests/device_token_rotation_wire_authority_contract.rs",
    "source/host/tests/device_token_rotation_wire_gateway_contract.rs",
    "source/host/tests/device_pairing_retained_stream_contract.rs",
    "projects/openclaw-rust/docs/specs/device-pairing-revocation-continuity.md",
    "projects/openclaw-rust/docs/specs/device-role-token-revocation.md",
    "projects/openclaw-rust/docs/specs/device-token-rotation-wire.md",
    "projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md",
    "source/host/src/gateway_config.rs",
    "source/host/src/host_event_bus.rs",
    "source/host/src/extensions/local_exec/local_exec_bridge.rs",
    "source/host/src/extensions/local_exec/production.rs",
    "source/host/src/extensions/local_exec/extension.rs",
    "source/host/src/extensions/webauthn_proxy/webauthn_proxy_bridge.rs",
    "source/host/src/extensions/webauthn_proxy/extension.rs",
    "source/host/src/extensions/node_host/runtime_state.rs",
    "source/host/src/extensions/node_host/session_registry.rs",
    "source/host/app/src/tests/device_pairing_revocation_tests.rs",
    "source/host/app/src/tests/device_token_rotation_wire_tests.rs",
    "projects/openclaw-rust/docs/specs/device-pairing-revision-admission.md",
    "projects/openclaw-rust/docs/specs/device-pairing-join-ingress.md",
    "source/host/tests/gateway_join_admission_contract.rs",
    "source/host/src/extensions/session/production.rs",
    "projects/openclaw-rust/evidence/memory-gateway-authorization-647e0932-20261102.json",
    "source/host/src/host_gateway_api.rs",
    "source/host/src/extensions/transcript/background_wakes.rs",
    "source/host/src/gateway_server.rs", "source/host/app/src/main.rs",
    "source/host/tests/durable_recovery_stores_contract.rs",
    "source/host/tests/pending_wake_rearm_contract.rs",
    "source/host/tests/transcript_manager_contract.rs",
    "source/host/tests/memory_gateway_authorization_contract.rs",
    "source/host/tests/cloud_agent_pending_wake_rearm_production_contract.rs",
    "projects/openclaw-rust/docs/specs/channel-gateway-credential-authorization.md",
    "source/host/tests/channel_gateway_authorization_contract.rs",
    "projects/openclaw-rust/docs/specs/channel-credential-write-admission.md",
    "source/host/tests/connector_secret_store_contract.rs",
    "source/host/tests/session_gateway_contract.rs",
    "source/host/tests/widget_responses_full_contract.rs",
    "source/host/src/extensions/transcript/widget_responses.rs",
    "source/host/src/extensions/session/session_diagnostics.rs",
    "source/host/src/extensions/session/connector_secret_store.rs",
    "source/host/src/extensions/session/channel_store.rs",
    "source/host/tests/memory_service_contract.rs",
    "source/host/src/runner/turn_memory.rs",
    "source/host/tests/turn_memory_contract.rs",
    "source/host/tests/fabushi_model_identity_production_contract.rs",
    "source/host/tests/memory_synthesis_store_contract.rs",
    "source/host/tests/memory_synthesis_service_contract.rs",
    "projects/openclaw-rust/docs/specs/session-communication-production-boundary.md",
    "source/host/src/extensions/session/session_conversation_state.rs",
    "source/host/src/extensions/session/agent_db.rs",
    "source/host/src/extensions/transcript/agent_to_agent_messaging.rs",
    "source/host/src/transcript_mutation_events.rs",
    "source/host/tests/openclaw_sessions_and_config_contract.rs",
    "source/host/tests/agent_to_agent_messaging_contract.rs",
    "source/host/tests/background_wakes_production_contract.rs",
    "projects/openclaw-rust/docs/specs/mcp-account-listing-retirement.md",
    "source/host/src/runner/coordinator_tool_relay.rs",
    "source/host/src/ports/mcp_state_executor.rs",
    "source/host/tests/mcp_state_executor_contract.rs",
    "source/host/src/extensions/inference/provider_session.rs",
    "projects/openclaw-rust/docs/specs/codex-model-identity.md",
    "projects/openclaw-rust/docs/specs/codex-request-workspace-binding.md",
    "source/host/src/extensions/inference/codex_request_binding.rs",
    "source/host/src/extensions/inference/codex_request_workspace_tests.rs",
    "source/host/src/extensions/inference/summary_job.rs",
    "source/host/src/extensions/inference/codex_synthesis_identity_tests.rs",
    "source/host/src/extensions/memory/production_identity_tests.rs",
    "projects/openclaw-rust/docs/specs/memory-synthesis-job-identity.md",
    "projects/openclaw-rust/evidence/memory-synthesis-identity-d56b3d1d-20261103/source-contract.json",
    "source/host/src/extensions/inference/codex_model_identity_tests.rs",
    "source/host/app/src/tests/codex_model_admission_tests.rs",
    "source/host/src/extensions/inference/production.rs",
    "source/host/src/extensions/inference/codex_direct_responses.rs",
    "source/host/src/extensions/settings/settings_service.rs",
    "source/host/src/runner_production_bridge.rs",
    "source/host/tests/inference_provider_session_contract.rs",
    "source/host/tests/inference_codex_direct_contract.rs",
    "source/host/tests/inference_extension_composition_contract.rs",
    "source/host/tests/fabushi_model_identity_production_contract.rs",
    "source/host/tests/runner_effect_custody_contract.rs",
    "source/host/src/extensions/transcript/turn_runtime.rs",
    "source/host/src/runner/production_turn_input_projection.rs",
    "source/host/src/runner/production_turn_run_shell_adapter.rs",
    "source/host/src/runner/routed_provider_runtime.rs",
    "source/host/tests/production_turn_run_shell_adapter_contract.rs",
    "source/host/src/runner/tools/turn_toolset.rs",
    "source/host/src/runner/turn_agent_composition.rs",
    "source/host/src/runner/generated_agent_turn_stream.rs",
    "source/host/src/runner/production_agent_checkpoint.rs",
    "source/host/src/runner/inactive_turn_agent_stream.rs",
    "source/host/src/runner/stream_attempt.rs",
    "source/host/tests/production_checkpoint_restart_contract.rs",
    "source/host/tests/support/prior_root_historical_fixture.rs",
    "projects/openclaw-rust/docs/specs/canonical-prior-root-identity.md",
    "source/host/src/connectors/channel_delivery.rs",
    "source/host/tests/runner_production_bridge_contract.rs",
    "source/host/tests/production_turn_input_projection_contract.rs",
    "source/host/tests/turn_toolset_contract.rs",
    "source/host/tests/runner_routed_provider_contract.rs",
    "source/host/tests/openclaw_chat_cron_node_contract.rs",
    "source/host/tests/turn_telemetry_production_wiring_contract.rs",
    "source/host/tests/group_member_runner_deletion_production_contract.rs",
    "source/host/tests/openclaw_channel_delivery_contract.rs",
    "source/host/tests/external_machine_tool_contract.rs",
    "source/host/tests/sand_generate_image_tool_contract.rs",
    "source/host/tests/sand_multitask_todo_tool_contract.rs",
    "source/host/tests/sand_web_tools_contract.rs",
    "source/host/tests/turn_toolset_await_mcp_contract.rs",
    "source/host/tests/fabushi_responses_credential_context_contract.rs",
    "source/host/tests/fabushi_responses_credential_context_contract/automatic_attempt_binding.rs",
    "source/host/src/host_runner_composition.rs",
    "source/host/src/runner/production_turn_agent_owner.rs",
    "source/host/tests/production_agent_checkpoint_contract.rs",
    "source/host/tests/canonical_checkpoint_recovery_contract.rs",
    "source/host/tests/production_agent_store_recovery_contract.rs",
    "source/host/tests/production_turn_agent_owner_contract.rs",
    "source/host/app/src/tests/original_turn_recovery_tests.rs",
    "source/host/tests/host_runner_composition_production_wiring_contract.rs",
    "source/host/tests/transcript_mirror_production_provider_contract.rs",
    "source/host/src/extensions/session/production_agent_store.rs",
    "source/host/src/runner/sand_agent_runner.rs",
    "source/host/src/transcript_mirror/production_provider.rs",
    "source/host/src/transcript_mirror/transcript_mirror.rs",
    "source/host/src/transcript_mirror/transcript_journal_codec.rs",
    "source/host/src/transcript_mirror/transcript_mirror_router.rs",
    "source/host/src/runner/tools/send_message_tool.rs",
    "source/host/src/transcript_mirror/generated_occurrence_codec.rs",
    "source/host/tests/session_conversation_state_contract.rs",
    "source/host/app/src/tests/channel_host_lane_tests.rs",
    "projects/fabushi-communication-platform/current-owner-inventory.md",
    "projects/fabushi-communication-platform/owner-resolution-matrix.md",
    "projects/openclaw-rust/docs/specs/generated-send-message-fidelity.md",
    "projects/openclaw-rust/docs/specs/media-settlement-unknown.md",
    "projects/openclaw-rust/docs/specs/final-assistant-media-custody.md",
    "source/host/src/attachment_paths.rs",
    "source/host/app/src/tests/media_settlement_unknown_tests.rs",
    "source/host/src/runner/turn_agent_composition/tool_call_recorder.rs",
    "source/host/src/runner/turn_agent_composition/tool_call_recorder_tests.rs",
    "source/host/src/runner/agent_v1_wire.rs",
    "source/host/src/runner/tools/send_message_encoding.rs",
    "source/host/tests/send_message_tool_contract.rs",
    "source/packages/proto/generated/agent/v1/agent_pb.ts",
    "source/packages/proto/generated/agent/v1/send_message_tool_pb.ts",
]
inputs = sorted(set(inputs) | {"source/host/build.rs", "source/host/tests/generated_occurrence_codec_contract.rs"} | {
    str(path) for path in pathlib.Path("source/packages/proto/generated").rglob("*.ts")
})
# BXR-v1.3 binds the actual Box owner, all changed consumers and the existing
# daemon producer separately from unrelated portable-source coverage.
box_response_inputs = [
    "projects/openclaw-rust/docs/specs/box-background-execution-custody.md",
    "projects/openclaw-rust/evidence/box-background-custody-647e0932-20261103/source-contract.json",
    "projects/openclaw-rust/evidence/box-background-custody-647e0932-20261103/h6d380-recovery-decisions.json",
    "source/host/src/runner/background_work.rs",
    "source/host/src/runner/shell_terminal_watch.rs",
    "source/host/tests/background_work_contract.rs",
    "projects/openclaw-rust/evidence/box-shell-capture-647e0932-20261103/source-contract.json",
    "source/host/src/extensions/forever_box/host_box.rs",
    "source/host/src/extensions/forever_box/forever_box_service.rs",
    "source/host/src/extensions/inference/codex_direct_responses.rs",
    "source/host/src/runner/stream_attempt.rs",
    "source/host/src/runner/production_turn_run_shell_adapter.rs",
    "source/host/src/runner/routed_provider_runtime.rs",
    ".github/workflows/rust-desktop-runtime.yml",
    "docs/specs/g11-box-browser-wire-completion.md",
    "projects/openclaw-rust/scripts/capture_portable_contracts.py",
    "projects/openclaw-rust/upstream.lock.json",
    "source/box-exec-daemon/Cargo.toml",
    "source/box-exec-daemon/src/cli.rs",
    "source/box-exec-daemon/src/lib.rs",
    "source/box-exec-daemon/src/main.rs",
    "source/box-exec-daemon/src/server.rs",
    "source/box-exec-daemon/src/workspace_write.rs",
    "source/host/Cargo.lock",
    "source/host/Cargo.toml",
    "source/host/build.rs",
    "source/host/src/box/box_file_transfer.rs",
    "source/host/src/box/box_remote_accessor.rs",
    "source/host/src/box/box_shell_command.rs",
    "source/host/src/box/exec_daemon_process.rs",
    "source/host/src/box/generated_production.rs",
    "source/host/src/box/mod.rs",
    "source/host/src/box/protected_path_guard.rs",
    "source/host/src/extensions/forever_box/runner_tools.rs",
    "source/host/src/extensions/inference/provider_session.rs",
    "source/host/src/runner/box_tool_access.rs",
    "source/host/src/runner/large_output_spill.rs",
    "source/host/src/runner/remote_box_resources.rs",
    "source/host/src/runner/runner_prompt_glue.rs",
    "source/host/src/runner/sand_action_audit.rs",
    "source/host/src/runner/tools/mcp_meta_tools.rs",
    "source/host/tests/box_exec_daemon_process_contract.rs",
    "source/host/tests/box_generated_production_contract.rs",
    "source/host/tests/box_remote_accessor_contract.rs",
    "source/host/tests/large_output_spill_contract.rs",
    "source/host/tests/runner_box_tool_composition_contract.rs",
    "source/host/tests/runner_effect_custody_contract.rs",
    "source/host/tests/runner_prompt_glue_contract.rs",
    "source/packages/agent-exec/shell-stream.ts",
    "source/packages/agent/tools/core/shell/create-shell-tool.ts",
    "source/packages/local-exec/background-shell.ts",
    "source/packages/local-exec/shell-stream.ts",
    "source/packages/proto/generated/agent/v1/background_shell_exec_pb.ts",
    "source/packages/proto/generated/agent/v1/exec_pb.ts",
    "source/packages/proto/generated/agent/v1/exec_service_pb.ts",
    "source/packages/proto/generated/agent/v1/shell_exec_pb.ts"
]
local_exec_outcome_inputs = [
    "projects/openclaw-rust/docs/specs/local-exec-outcome-ownership.md",
    "source/host/src/extensions/local_exec/gateway_local_exec_sand_box.rs",
    "source/host/src/extensions/local_exec/local_exec_error.rs",
    "source/host/src/extensions/local_exec/local_exec_bridge.rs",
    "source/host/src/extensions/local_exec/production.rs",
    "source/host/src/extensions/local_exec/extension.rs",
    "source/host/src/extensions/local_exec/agent_v1_exec_wire.rs",
    "source/host/proto/agent/v1/local_exec.proto",
    "source/host/build.rs",
    "source/host/src/extensions/local_tool_permission/extension.rs",
    "source/host/src/extensions/local_tool_permission/local_tool_permission_controller.rs",
    "source/host/src/runner/host_external_machine_dependencies.rs",
    "source/host/src/runner/tools/sand_external_machine_tools.rs",
    "source/host/src/runner/tools/mcp_meta_tools.rs",
    "source/host/src/runner/stream_attempt.rs",
    "source/host/src/runner/production_turn_run_shell_adapter.rs",
    "source/host/tests/gateway_local_exec_sand_box_contract.rs",
    "source/host/tests/extensions_local_exec_contract.rs",
    "source/host/tests/local_exec_production_codec_contract.rs",
    "source/host/tests/runner_effect_custody_contract.rs",
    "source/host/tests/openclaw_chat_cron_node_contract.rs",
    "source/host/local-exec/local-exec-provider.ts",
    "source/local-exec-daemon/production-executor.ts",
    "source/packages/agent-exec/controlled.ts",
    "source/shared/local-exec-gateway.ts",
    "source/local-exec-daemon/production-outcome.contract.test.ts",
    "source/host/app/src/tests/local_exec_outcome_tests.rs",
    "source/host/src/extensions/node_host/invocation_runtime.rs",
    "source/host/src/extensions/node_host/invoke_protocol.rs",
    "source/host/src/extensions/node_host/runtime_state.rs",
    "source/host/src/durable_file_policy.rs",
    "source/host/src/extensions/process/command_custody.rs",
    "source/host/src/host_paths.rs",
    "source/host/Cargo.toml",
    "source/host/Cargo.lock",
    "source/host/app/Cargo.toml",
    "source/host/app/Cargo.lock",
    "source/host/app/src/main.rs",
    ".github/workflows/rust-desktop-runtime.yml",
    "projects/openclaw-rust/scripts/capture_portable_contracts.py",
    "projects/openclaw-rust/scripts/capture_media_cleanup.py"
]
plugin_admission_lock_inputs = [
    ".github/workflows/rust-desktop-runtime.yml",
    "projects/openclaw-rust/scripts/capture_portable_contracts.py",
    "projects/openclaw-rust/docs/specs/plugin-skills-snapshot-admission.md",
    "projects/openclaw-rust/evidence/plugin-skills-admission-lock-647e0932-20261103/source-contract.json",
    "source/host/Cargo.toml",
    "source/host/Cargo.lock",
    "source/host/src/extensions/mcp/plugin_skills.rs",
    "source/host/src/extensions/mcp/plugin_skills_uninstall_state.rs",
    "source/host/src/extensions/mcp/plugin_skills_cache.rs",
    "source/host/src/extensions/mcp/skill_publish.rs",
    "source/host/src/extensions/mcp/plugin_skills_sync_tests.rs",
    "source/host/src/extensions/mcp/plugin_skills_auth_recovery_tests.rs",
    "source/host/src/extensions/session/production.rs",
    "source/host/src/workflows/workflow_store.rs",
    "source/host/tests/mcp_service_contract.rs",
    "source/host/tests/plugin_skills_contract.rs",
    "source/host/tests/workflow_store_contract.rs",
    "source/host/tests/workflow_commands_contract.rs",
    "source/host/tests/mcp_coordinator_relay_contract.rs",
    "source/host/tests/legacy_live_references_contract.rs",
    "source/host/tests/skill_publish_contract.rs"
]
inputs = sorted(set(inputs) | set(box_response_inputs) | set(local_exec_outcome_inputs) | set(plugin_admission_lock_inputs))
base = ["cargo", "test", "--locked", "--manifest-path", "source/host/Cargo.toml"]
integration = base + ["--no-fail-fast"]
for target in ("durable_recovery_stores_contract", "pending_wake_rearm_contract",
               "transcript_manager_contract", "memory_gateway_authorization_contract", "channel_gateway_authorization_contract", "cloud_agent_pending_wake_rearm_production_contract",
               "connector_secret_store_contract", "session_gateway_contract", "widget_responses_full_contract",
               "agent_content_search_contract", "roster_search_contract", "palette_message_gateway_contract",
               "memory_service_contract", "turn_memory_contract", "memory_synthesis_store_contract", "memory_synthesis_service_contract",
               "background_wakes_production_contract"):
    integration += ["--test", target]
required_integration = [
    "checked_pending_store_accepts_missing_versionless_and_v1_without_normalizing_markers",
    "checked_pending_store_refuses_entire_invalid_images_and_all_compatibility_mutations",
    "checked_pending_store_refuses_unreadable_images_and_preserves_the_target",
    "checked_pending_store_validates_incoming_markers_without_mutating_known_state",
    "matching_pending_wake_admission_is_read_only_and_refuses_stale_or_settled_snapshots",
    "matching_pending_wake_admission_compares_every_marker_field",
    "pending_wake_store_does_not_clear_or_overwrite_a_newer_marker",
    "production_transcript_runtime_owns_both_durable_recovery_stores_and_cleans_deleted_agent",
    "checked_rearm_refuses_invalid_whole_image_before_any_runtime_dispatch",
    "checked_rearm_accepts_known_empty_state_without_creating_recovery_work",
    "channel_rearm_success_retains_complete_marker_across_reopen",
    "channel_rearm_error_retains_complete_marker_across_reopen",
    "checked_rearm_prunes_old_shell_but_retains_deferred_channel_failure_exactly",
    "recreate_carry_still_excludes_channel_delivery_markers",
    "rearm_prunes_stale_preserves_gone_and_restores_each_runtime_kind",
    "manager_owns_pending_wake_rearm_and_replays_durable_wakes",
    "manager_refuses_partial_pending_wake_recovery_and_keeps_unknown_custody_busy",
    "memory_gateway_rejects_path_selectors_before_external_memory_read",
    "memory_gateway_keeps_required_trimmed_missing_and_unknown_read_contracts",
    "memory_gateway_preserves_projection_order_and_hundred_record_limit",
    "manager_owns_memory_gateway_and_invalidates_prompt_snapshot_after_mutation",
    "memory_gateway_rejects_paired_nodes_and_unknown_roles_before_owner",
    "memory_gateway_read_scope_and_headers_cannot_grant_mutation",
    "memory_gateway_empty_unrelated_and_session_only_grants_fail_closed",
    "memory_gateway_write_only_and_admin_only_mutations_survive_reopen",
    "memory_gateway_native_access_and_explicit_ceilings_preserve_compatibility",
    "memory_gateway_unrecognized_and_revoked_credentials_never_regain_native_access",
    "memory_gateway_preserves_projection_selectors_and_unrelated_routes",
    "channel_gateway_rejects_read_node_and_unknown_roles_before_owner",
    "channel_gateway_headers_and_unrelated_grants_cannot_authorize_credential_writes",
    "channel_gateway_write_and_admin_credentials_persist_through_reopen",
    "channel_gateway_native_credentials_and_header_ceilings_preserve_compatibility",
    "channel_gateway_explicit_bad_or_revoked_credentials_never_regain_ambient_authority",
    "channel_gateway_registry_origin_and_memory_controls_remain_intact",
    "connector_secret_store_matches_frozen_validation_merge_and_remove_contract",
    "production_session_exposes_connector_credential_ownership_at_canonical_root",
    "connector_secret_write_admission_preserves_invalid_images_and_redacts_errors",
    "connector_secret_write_admission_merges_valid_objects_and_reopens",
    "connector_secret_write_admission_rejects_read_denied_regular_file_before_replacement",
    "production_session_connector_refusal_preserves_metadata_and_reopens",
    "production_gateway_composes_channel_metadata_and_secrets",
    "production_gateway_refuses_unreadable_channel_credentials_without_metadata_changes",
    "auto_review_secret_and_reaction_delegate_to_existing_session_and_projection_owners",
    "secret_submission_refuses_unreadable_credentials_without_signalling_and_recovers_in_place",
    "manager_is_the_single_production_composition_owner",
    "manager_secret_write_refusal_is_redacted_and_does_not_resume_before_repair",
    "unicode_snippet_maps_expanding_and_contracting_lowercase_to_original_text",
    "unicode_snippet_preserves_whole_string_contextual_sigma_matching",
    "unicode_snippet_keeps_character_windows_and_whitespace_contract",
    "roster_snippet_preserves_ascii_and_utf8_byte_context_windows",
    "production_gateway_unicode_fallback_without_index_preserves_storage_and_reopens",
    "production_gateway_unicode_fallback_with_unready_index_never_queries_it",
    "production_gateway_unicode_fallback_with_unavailable_index_queries_then_scans",
    "production_gateway_fallback_preserves_unicode_match_limits_and_recency",
    "production_gateway_consumes_ready_content_search_for_indexed_messages_and_media",
    "frozen_content_match_rules_preserve_snippets_order_and_hidden_peer_filter",
    "snippet_collapses_whitespace_preserves_case_and_applies_frozen_window",
    "palette_message_gateway_exports_persisted_rust_search_results",
    "episode_append_read_failure_preserves_pending_turns_across_restart",
    "episode_append_write_failure_preserves_interleaved_turns_and_recovers",
    "episode_duplicate_append_acknowledges_existing_fact_before_clearing_pending",
    "episode_summary_failure_preserves_pending_turns_and_recovers",
    "episode_successful_no_summary_reply_keeps_existing_settlement",
    "episode_record_refusal_preserves_archived_pending_until_restore",
    "completed_memorable_turns_extract_memory_and_roll_episode_summary",
    "evidence_mode_clears_pending_episode_state_without_running_extraction",
    "frozen_turn_memory_exchange_keeps_agent_messages_before_final_assistant_text",
    "file_memory_store_append_read_errors_preserve_bytes_and_allow_repair",
    "file_memory_store_duplicate_append_checks_corrupt_destination_before_metadata",
    "file_memory_store_append_preserves_missing_readable_empty_and_duplicate_controls",
    "file_memory_store_append_directory_read_error_is_preserved",
    "bound_synthesis_gate_rejects_a_noncanonical_store_path",
    "synthesis_create_read_errors_preserve_memory_and_metadata",
    "synthesis_cross_kind_update_read_errors_preserve_source_and_metadata",
    "synthesis_batch_preflights_later_append_before_earlier_remove",
    "file_memory_store_round_trips_recall_dedupe_remove_and_clear",
    "explicit_memories_are_protected_from_synthesis_updates_and_removals",
    "synthesis_create_update_remove_round_trip_marks_origin_and_temporal_review",
    "stale_snapshot_cannot_commit_and_user_deletion_tombstones_recreation",
    "proposal_and_verification_commit_through_synthesis_target",
    "verifier_rejection_does_not_commit_and_retryable_failures_stop_at_bound",
    "synthesis_deadline_cancels_verification_and_reports_terminal_failure",
    "typed_identity_refusals_settle_without_retry_or_temporal_review",
    "one_snapshot_factory_survives_worker_timeout_and_retry",
    "stale_resnapshot_allocates_new_job_and_preserves_concurrent_evidence",
    "shipping_host_rearms_durable_cloud_agent_and_lost_subagent_wakes_after_gateway_start",
    "startup_rearm_routes_recreate_interrupted_shell_through_completion_revivals",
    "shipping_cloud_recovery_admission_transfers_only_the_observed_durable_marker",
    "shipping_generated_subagent_dispatch_persists_durable_pending_wake_before_provider_run",
    "channel_failure_readiness_on_either_side_of_pause_retries_latest_filtered_head_once",
    "channel_failure_targeted_readiness_preserves_other_tickets_paused_heads_and_raw_tails",
    "channel_failure_live_head_preserves_filtered_order_ahead_of_tail_until_definite_finish",
    "channel_failure_launch_error_and_delayed_callbacks_cannot_clear_a_successor",
    "channel_failure_entered_and_retained_heads_refuse_all_replay_and_phase_confusion",
    "channel_failure_live_take_requires_revival_and_releases_stored_empty_input",
    "channel_failure_live_enqueue_before_idle_uses_current_drain_and_claims_ordered_head_once",
    "channel_failure_live_idle_before_enqueue_admits_one_successor_without_late_release",
    "channel_failure_close_during_a_gate_revokes_all_phases_and_late_tickets_without_reopening",
    "channel_failure_retained_head_does_not_block_other_agents_or_wake_families",
    "channel_failure_wiring_oracle_rejects_cleanup_after_idle_return",
    "channel_failure_wiring_oracle_rejects_raw_take_and_bypassed_token_admission",
    "channel_failure_wiring_oracle_requires_checked_completion_before_finish",
    "channel_failure_wiring_oracle_requires_captured_generation_and_shipping_observer",
]
store_filter = "extensions::transcript::sand_pending_wake_store::checked_pending_wake_tests"
required_store = [store_filter + "::" + name for name in (
    "failed_pending_publication_preserves_prior_bytes_and_refuses_mutation_result",
    "poisoned_pending_owner_refuses_reads_admission_and_all_mutations",
    "matching_pending_admission_holds_clone_shared_lock_until_queue_admission_finishes",
)]
sse_filter = "gateway_server::tests::paired_device_event_filter_hides_host_mcp_requests_even_without_channel_selection"
claim_integration = base + ["--no-fail-fast", "--test", "openclaw_sessions_and_config_contract", "--test", "agent_to_agent_messaging_contract"]
required_claim_integration = [
    "test_ocrm_session_o01_session_key_routing",
    "test_provider_review_continuation_uses_owner_capability_and_existing_run_path",
    "test_provider_review_cleanup_oracle_rejects_missing_reordered_and_unbound_releases",
    "test_chat_abort_routing_oracle_rejects_missing_delegate_route_and_run_filters",
    "test_ocrm_session_o03_conversation_turn_correlation",
    "session_turn_claim_settlement::released_claim_cannot_settle_successor",
    "session_turn_claim_settlement::cancelled_claim_cannot_settle_reused_turn",
    "session_turn_claim_settlement::private_claim_identity_survives_payload_mutation_and_rejects_wrong_agent",
    "session_turn_claim_settlement::clones_share_single_use_authority_and_drop_retains_custody",
    "inbound_reply_commit_preserves_reentrant_replacement_claim",
    "inbound_turn_claim_is_released_on_refusal_or_duplicate_and_completed_only_after_commit",
    "inbound_turn_completion_precedes_partner_bookkeeping",
    "legacy_inbound_uses_occupied_physical_ids_and_preserves_no_admission_compatibility",
    "media_commit_oracle_accepts_actual_owner_and_preserves_admission_guards",
    "media_commit_oracle_rejects_missing_duplicate_or_conditional_helper",
    "media_commit_oracle_rejects_missing_duplicate_or_conditional_owner",
    "media_commit_oracle_rejects_changed_commit_or_rollback_commands",
    "media_commit_oracle_rejects_extra_aliased_or_macro_callers",
    "media_commit_oracle_rejects_discarded_or_changed_settlement_handling",
    "media_commit_oracle_rejects_changed_unknown_entry_handling",
    "media_commit_oracle_preserves_raw_commit_prohibition_outside_proven_method",
]
fresh_filter = "extensions::session::agent_db::fresh_append_tests"
required_fresh = [fresh_filter + "::" + name for name in (
    "fresh_commit_failure_is_typed_unknown_and_does_not_replay_or_publish",
    "fresh_insert_and_commit_failures_retire_an_unclean_connection",
    "actual_agent_inbound_commit_unknown_retains_reply_claim_without_side_effects",
    "actual_agent_inbound_rollback_unknown_retires_owner_and_retains_reply_claim",
)]
pairing_required = [
    "revision_admission::pairing_revision_refuses_corrupt_counters_without_committing_mutations",
    "revision_admission::pairing_revision_last_increment_is_durable_and_exhaustion_retains_authority",
    "revision_admission::pairing_revision_missing_and_legacy_decimal_counters_keep_monotonic_receipts",
    "test_ocrm_pair_o01_request_success",
    "test_ocrm_pair_request_reuses_compatible_pending_identity",
    "test_ocrm_pair_o02_request_requires_device_id_and_public_key",
    "test_ocrm_pair_o03_and_o04_pair_status",
    "test_ocrm_pair_o05_and_o06_pair_approve",
    "test_ocrm_pair_o07_pair_reject",
    "test_ocrm_pair_o08_pair_list",
    "test_ocrm_pair_o09_and_o10_pair_revoke",
    "test_ocrm_pair_o11_rotate_token_in_band_and_cross_device",
    "test_ocrm_pair_o12_setup_code",
    "test_ocrm_pair_o12_setup_code_is_expiring_and_single_use",
    "test_ocrm_pair_o13_join_code_validation_and_path_parsing",
    "test_ocrm_pair_o14_safe_keys_and_labels",
    "test_ocrm_pair_o15_pairing_reply_formatting",
    "test_ocrm_pair_o16_mobile_pairing_security_policy",
    "test_ocrm_pair_o17_join_code_registration_and_single_use_redeem",
    "test_ocrm_pair_o18_join_code_expiry_and_payload_bounds_fail_closed",
]
rotation_owner_required = [
    "paired_rotation_proofs_bind_owner_and_sealed_projection",
    "paired_rotation_operator_permissions_follow_fixed_scope_implications",
    "paired_rotation_header_ceiling_and_body_flags_cannot_gain_authority",
    "paired_rotation_desired_scopes_preserve_narrow_tokens_and_explicit_empty",
    "paired_rotation_live_credentials_and_approval_narrow_prior_proofs",
    "paired_rotation_stored_corruption_and_bad_requests_preserve_all_rows",
    "paired_rotation_revision_failure_rolls_back_and_recovers_durably",
    "paired_rotation_replacement_revocation_and_reopen_retire_proofs",
    "paired_rotation_admin_other_roles_require_literal_prefixed_baseline",
]
rotation_http_required = [
    "paired_http_self_rotation_derives_delivery_from_identity_and_survives_reopen",
    "paired_http_write_cannot_rotate_other_devices_roles_or_forge_authority",
    "paired_http_admin_only_rotates_operator_and_node_with_identity_bound_disclosure",
    "paired_http_read_node_unknown_and_unrelated_grants_cannot_gain_rotation_authority",
    "paired_http_explicit_failed_credentials_never_become_native_and_public_controls_remain",
    "paired_http_default_bound_hook_refuses_without_legacy_native_dispatch",
    "paired_http_rechecks_retirement_after_authentication_with_live_and_unrelated_controls",
    "paired_http_two_admitted_self_rotations_commit_exactly_one_replacement",
]
revocation_owner_required = [
    "paired_revocation_operator_self_commits_and_preserves_repairs_on_reopen",
    "paired_revocation_admin_only_grant_removes_mixed_and_corrupt_targets",
    "native_revocation_labels_keep_whole_row_semantics_and_surviving_repair",
    "paired_revocation_cross_device_denial_precedes_target_inspection",
    "paired_revocation_self_checks_primary_approved_and_every_token_role",
    "paired_revocation_decisive_role_corruption_is_internal_and_atomic",
    "paired_revocation_management_requires_current_operator_write",
    "paired_revocation_header_and_body_fields_cannot_supply_authority",
    "paired_revocation_proofs_bind_issuer_and_ignore_public_projection",
    "paired_revocation_exact_credential_retirement_precedes_role_policy",
    "paired_revocation_bad_requests_and_authorized_absence_preserve_state",
    "paired_revocation_corrupt_caller_and_baseline_fallback_fail_closed",
    "paired_revocation_live_grants_bound_sealed_management_admission",
    "revocation_revision_corruption_rolls_back_and_recovers_after_reopen",
    "paired_revocation_final_revision_increment_is_durable",
    "revocation_checks_delete_count_and_redacts_transaction_failures",
    "revocation_immediate_transaction_precedes_policy_and_absence_decisions",
    "revocation_commit_failure_rolls_back_before_reopen_and_explicit_recovery",
    "paired_revocation_concurrent_self_removal_commits_exactly_once",
    "retained_authorization_supports_current_roles_without_durable_writes",
    "retained_authorization_narrowing_retires_original_grant_without_replacement",
    "retained_authorization_expansion_does_not_promote_sealed_admission",
    "retained_authorization_uses_fixed_role_implications_and_complete_live_grant",
    "retained_authorization_replacement_leaves_other_roles_and_devices_current",
    "retained_authorization_reopen_requires_fresh_owner_issued_provenance",
    "revocation_and_retained_authorization_propagate_storage_read_failure",
    "fresh_authorization_requires_approved_role_and_entire_canonical_grant",
    "fresh_authorization_rejects_corrupt_decisive_storage_without_writes",
    "fresh_authorization_reopen_preserves_current_approval_and_issuer_boundary",
]
revocation_http_required = [
    "paired_revoke_self_operator_only_delivers_final_reply_and_reports_bound_context_once",
    "paired_revoke_write_rejects_cross_device_and_mixed_self_despite_body_or_header_forgery",
    "paired_revoke_admin_implication_ceilings_and_storage_error_reporting_use_real_owner",
    "paired_revoke_read_node_empty_and_unrelated_grants_cannot_gain_management_authority",
    "paired_revoke_failed_credentials_listener_spoofing_and_public_native_controls_are_distinct",
    "paired_revoke_default_hook_refuses_without_generic_dispatch_and_native_still_works",
    "paired_revoke_rechecks_after_authentication_with_unchanged_and_unrelated_controls",
    "paired_revoke_two_authenticated_self_requests_commit_one_removal_and_one_stale_result",
]
role_token_owner_required = [
    "role_retirement_pairing_only_self_preserves_mixed_pair_and_survives_reopen",
    "role_retirement_admin_repeats_node_retirement_and_clears_both_surfaces",
    "role_retirement_native_dispatch_and_narrow_native_ceiling_obey_target_caps",
    "role_retirement_closed_request_rejects_types_missing_fields_and_assertions",
    "role_retirement_target_nondisclosure_precedes_malformed_or_absent_target_reads",
    "role_retirement_pairing_permission_and_singular_headers_do_not_promote_proofs",
    "role_retirement_sealed_issuer_and_projection_control_identity_and_permissions",
    "role_retirement_currentness_precedes_operator_policy_for_every_sealed_role",
    "role_retirement_caller_identity_failures_keep_established_unauthorized_status",
    "role_retirement_caller_corruption_and_null_approval_fallback_are_distinct",
    "role_retirement_intersects_sealed_live_approval_and_target_grants",
    "role_retirement_target_scope_shape_is_strict_even_on_repeat",
    "role_retirement_entire_target_grant_cap_applies_to_active_and_retired_entries",
    "role_retirement_target_approval_and_nonoperator_scopes_are_not_issuance_prerequisites",
    "role_retirement_target_role_selection_unions_strict_array_with_optional_primary",
    "role_retirement_required_target_storage_corruption_is_redacted_and_atomic",
    "role_retirement_fresh_and_retained_admission_share_all_marker_variants",
    "role_retirement_rotation_and_reapproval_issue_successors_without_reviving_old_bearers",
    "role_retirement_revision_missing_corrupt_and_overflow_roll_back_on_reopen",
    "role_retirement_last_revision_and_legacy_bootstrap_behaviors_remain_distinct",
    "role_retirement_sql_and_row_count_faults_roll_back_then_recover",
    "role_retirement_immediate_write_lock_precedes_policy_and_target_read_decisions",
    "role_retirement_failed_commit_rolls_back_token_surfaces_and_counter_durably",
    "role_retirement_two_owner_self_race_commits_once_and_rejects_retired_proof",
    "role_retirement_two_owner_rotation_race_has_one_credential_generation_winner",
    "role_retirement_two_connections_preserve_both_distinct_selected_updates",
    "role_retirement_storage_read_failure_is_redacted_without_repair_or_mutation",
]
role_token_http_required = [
    "token_revoke_http_self_pairing_on_mixed_device_reports_exact_receipt_and_context",
    "token_revoke_http_admin_management_and_scope_intersection_preserve_unselected_state",
    "token_revoke_http_native_ceiling_is_bound_for_secret_and_absent_loopback",
    "token_revoke_http_closed_request_and_storage_errors_are_redacted_and_atomic",
    "token_revoke_http_pairing_permission_and_common_role_scope_guards_remain_distinct",
    "token_revoke_http_failed_credentials_and_listener_guards_never_dispatch_as_native",
    "token_revoke_http_default_hook_fails_closed_for_paired_and_native",
    "token_revoke_http_rechecks_current_authority_after_authentication_with_controls",
    "token_revoke_http_two_self_admissions_and_rotation_races_have_serial_outcomes",
    "token_revoke_http_fresh_marker_rule_denies_retired_bearers_and_preserves_live_controls",
]
wire_rotation_owner_required = [
    "wire_rotation_pairing_self_delivers_committed_successor_and_survives_reopen",
    "wire_rotation_admin_self_other_role_delivers_but_cross_device_withholds",
    "wire_rotation_native_dispatch_always_withholds_and_honors_singular_ceiling",
    "wire_rotation_closed_request_rejects_all_invalid_shapes_before_any_mutation",
    "wire_rotation_normalization_matches_fixed_u_implications_and_utf16_order",
    "wire_rotation_desired_scopes_select_explicit_existing_or_device_fallback",
    "wire_rotation_explicit_empty_repairs_excessive_or_malformed_selected_grants",
    "wire_rotation_omission_never_filters_cross_role_device_fallback",
    "wire_rotation_target_baseline_is_mandatory_even_for_empty_repair",
    "wire_rotation_only_decisive_scope_storage_is_read_and_malformed_fallback_refuses",
    "wire_rotation_role_union_allows_missing_entry_but_never_stray_approval",
    "wire_rotation_replaces_obsolete_entry_fields_and_every_retired_marker",
    "wire_rotation_preserves_opaque_metadata_and_safe_lifecycle_values",
    "wire_rotation_rejects_non_safe_lifecycle_and_nonobject_selected_entries",
    "wire_rotation_target_storage_types_are_strict_and_nonoperator_scopes_are_literal",
    "wire_rotation_scope_implications_keep_caller_and_target_ceilings_independent",
    "wire_rotation_preserves_legacy_dpa_unicode_and_response_behavior",
    "wire_rotation_two_current_admin_owners_serialize_replacements_without_lost_updates",
    "wire_rotation_and_role_revoke_race_consumes_original_self_authority_once",
    "wire_rotation_refusal_preserves_a_competing_committed_retirement_or_approval_change",
    "wire_rotation_active_marker_variants_require_a_current_caller_even_for_empty_repair",
    "wire_rotation_absent_reply_readback_does_not_reveal_or_reissue_replacement",
    "wire_rotation_two_owner_distinct_roles_preserve_both_committed_replacements",
    "wire_rotation_target_nondisclosure_precedes_malformed_or_absent_target_reads",
    "wire_rotation_pairing_permission_and_singular_headers_do_not_promote_proofs",
    "wire_rotation_sealed_issuer_and_projection_control_identity_and_permissions",
    "wire_rotation_currentness_precedes_operator_policy_for_every_sealed_role",
    "wire_rotation_caller_identity_failures_keep_established_unauthorized_status",
    "wire_rotation_caller_corruption_and_null_approval_fallback_are_distinct",
    "wire_rotation_intersects_sealed_live_approval_and_target_grants",
    "wire_rotation_revision_missing_corrupt_and_overflow_roll_back_on_reopen",
    "wire_rotation_last_revision_and_legacy_bootstrap_behaviors_remain_distinct",
    "wire_rotation_sql_and_row_count_faults_roll_back_then_recover",
    "wire_rotation_immediate_write_lock_precedes_policy_and_target_read_decisions",
    "wire_rotation_failed_commit_rolls_back_token_surfaces_and_counter_durably",
    "wire_rotation_two_owner_self_race_commits_once_and_rejects_retired_proof",
    "wire_rotation_storage_read_failure_is_redacted_without_repair_or_mutation",
    "wire_rotation_minted_nel_literal_never_becomes_admin_in_existing_consumers",
    "wire_rotation_literal_baseline_and_bom_requests_have_one_scope_meaning",
    "wire_rotation_native_and_legacy_producers_preserve_canonical_literal_grants",
    "wire_rotation_pending_scope_identity_distinguishes_nel_and_reuses_bom_subsets",
    "wire_rotation_primary_only_successors_reopen_and_later_withdrawal_retires_them",
    "wire_rotation_unselected_primary_never_revives_and_existing_role_bytes_survive",
    "wire_rotation_primary_materialization_rolls_back_with_target_revision_and_commit_faults",
]
wire_rotation_http_required = [
    "token_rotate_wire_http_self_delivers_committed_successor_with_context_and_reopen",
    "token_rotate_wire_http_native_delivery_and_singular_ceiling_bind_with_and_without_secret",
    "token_rotate_wire_http_addressing_hides_targets_and_admin_delivery_is_device_bound",
    "token_rotate_wire_http_scope_omission_empty_implied_and_legacy_repair_use_resulting_grant",
    "token_rotate_wire_http_closed_request_storage_refusal_and_explicit_repair_are_atomic",
    "token_rotate_wire_http_pairing_permission_and_existing_role_scope_guards_stay_distinct",
    "token_rotate_wire_http_large_distinct_scope_request_reaches_owner_and_preserves_read_only_authority",
    "token_rotate_wire_http_bad_credentials_and_common_ingress_never_become_native",
    "token_rotate_wire_http_default_hook_fails_closed_and_dpa_keeps_its_distinct_contract",
    "token_rotate_wire_http_authenticated_barrier_rechecks_current_sealed_authority",
    "token_rotate_wire_http_two_self_admissions_have_one_winner_and_two_manager_rotations_commit",
    "token_rotate_wire_http_socket_closed_after_commit_never_retries_or_resurrects_the_predecessor",
    "token_rotate_wire_http_issued_nel_literal_never_becomes_pairing_or_admin_at_adjacent_consumers",
    "token_rotate_wire_http_primary_only_successors_materialize_roles_atomically_and_withdraw_after_reopen",
]
retained_stream_required = [
    "retained_initial_admission_refuses_before_headers_subscription_or_provider_registration",
    "retained_default_hook_refuses_paired_streams_and_preserves_native_streams",
    "retained_bridges_recheck_after_registration_before_headers_and_drop_the_provider_once",
    "retained_bridges_recheck_initial_retry_after_the_header_admission_has_succeeded",
    "retained_events_revoke_before_queued_admission_closes_with_unchanged_and_unrelated_controls",
    "retained_event_admitted_before_commit_may_finish_but_the_next_frame_is_refused",
    "retained_quiet_events_retire_on_the_real_fifteen_second_heartbeat_with_native_control",
    "retained_role_replacement_closes_only_that_role_and_keeps_node_other_device_and_fresh_token_live",
    "retained_approval_narrowing_live_grant_narrowing_corruption_and_storage_failure_close_without_writes",
    "retained_filtered_flood_checks_currentness_before_channel_and_both_internal_mcp_filters",
    "retained_streams_preserve_native_channel_slim_gzip_and_paired_internal_mcp_exclusion",
    "retained_reconnects_refuse_removed_malformed_credentials_and_wildcard_ambient_authority",
    "retained_bridge_requests_drop_each_owned_provider_once_and_keep_native_provider_live",
    "retained_bridge_responses_admit_after_decode_before_effect_and_keep_native_and_prior_admission_controls",
    "retained_token_revocation_closes_only_selected_role_and_preserves_live_reconnect_controls",
    "retained_token_revocation_respects_queued_and_prior_frame_admission",
    "retained_token_revocation_quiet_heartbeat_keeps_other_role_and_native_live",
    "retained_token_revocation_initial_admission_and_registration_drop_are_causal",
    "retained_token_revocation_bridge_streams_drop_only_selected_role_once",
    "retained_token_revocation_bridge_responses_preserve_unaffected_and_prior_admission",
    "retained_token_rotation_closes_only_selected_role_and_preserves_live_reconnect_controls",
    "retained_token_rotation_respects_queued_and_prior_frame_admission",
    "retained_token_rotation_quiet_heartbeat_keeps_other_role_and_native_live",
    "retained_token_rotation_initial_admission_and_registration_drop_are_causal",
    "retained_token_rotation_bridge_streams_drop_only_selected_role_once",
    "retained_token_rotation_bridge_responses_preserve_unaffected_and_prior_admission",
    "retained_primary_only_rotation_successors_reconnect_after_reopen_and_close_on_role_withdrawal",
]
cases = [
    ("generated-send-message-recorder", base + ["--lib", "runner::turn_agent_composition::tool_call_recorder::tests::", "--", "--test-threads=1"], [
        "runner::turn_agent_composition::tool_call_recorder::tests::actual_producer_normalization_is_captured_without_live_observation",
        "runner::turn_agent_composition::tool_call_recorder::tests::generic_none_id_preserves_the_actual_generated_success",
        "runner::turn_agent_composition::tool_call_recorder::tests::actual_producer_error_keeps_its_original_type_and_generated_error",
        "runner::turn_agent_composition::tool_call_recorder::tests::actual_producer_pre_observation_rejection_has_no_invented_arguments",
        "runner::turn_agent_composition::tool_call_recorder::tests::fanout_preserves_actual_events_after_releasing_the_recorder_lock",
        "runner::turn_agent_composition::tool_call_recorder::tests::capture_faults_keep_live_events_and_the_public_result_unchanged",
        "runner::turn_agent_composition::tool_call_recorder::tests::reversed_worker_completion_drains_in_registration_order",
        "runner::turn_agent_composition::tool_call_recorder::tests::late_old_worker_with_identical_id_cannot_attach_to_a_fresh_invocation",
        "runner::turn_agent_composition::tool_call_recorder::tests::duplicate_send_ids_are_refused_while_active_and_after_completion",
        "runner::turn_agent_composition::tool_call_recorder::tests::nested_registration_never_replaces_the_outer_receipt",
        "runner::turn_agent_composition::tool_call_recorder::tests::send_message_registration_requires_the_exact_first_party_definition",
        "runner::turn_agent_composition::tool_call_recorder::tests::invalid_initial_callbacks_fault_the_invocation_without_latest_match_recovery",
        "runner::turn_agent_composition::tool_call_recorder::tests::completed_callbacks_require_unchanged_arguments_start_and_terminal_fields",
        "runner::turn_agent_composition::tool_call_recorder::tests::duplicate_and_out_of_order_callback_phases_refuse_publication",
        "runner::turn_agent_composition::tool_call_recorder::tests::callbacks_after_finish_cannot_replace_history_or_reopen_a_sealed_recorder",
        "runner::turn_agent_composition::tool_call_recorder::tests::observed_success_must_match_the_actual_flat_public_result",
        "runner::turn_agent_composition::tool_call_recorder::tests::terminal_disposition_and_error_text_must_match_the_original_result",
        "runner::turn_agent_composition::tool_call_recorder::tests::initial_only_and_unobserved_success_are_never_qualified",
        "runner::turn_agent_composition::tool_call_recorder::tests::pre_observation_typed_errors_encode_rejection_without_raw_argument_inference",
        "runner::turn_agent_composition::tool_call_recorder::tests::unobserved_unknown_effect_is_never_recast_as_a_known_rejection",
        "runner::turn_agent_composition::tool_call_recorder::tests::foreign_thread_callbacks_never_search_other_active_slots_by_provider_id",
        "runner::turn_agent_composition::tool_call_recorder::tests::callbacks_cannot_attach_to_a_non_send_message_call",
        "runner::turn_agent_composition::tool_call_recorder::tests::receipt_from_another_recorder_faults_both_invocations",
        "runner::turn_agent_composition::tool_call_recorder::tests::receipt_cannot_be_finished_on_a_foreign_worker_thread",
        "runner::turn_agent_composition::tool_call_recorder::tests::abandoned_receipts_and_active_drains_permanently_refuse_publication",
        "runner::turn_agent_composition::tool_call_recorder::tests::poisoned_mutex_is_never_recovered_by_capture_finish_begin_or_drain",
        "runner::turn_agent_composition::tool_call_recorder::tests::begin_and_seal_are_one_use_and_a_new_invocation_requires_a_fresh_recorder",
        "runner::turn_agent_composition::tool_call_recorder::tests::checked_sequence_exhaustion_never_wraps_or_delegates_an_unregistered_call",
        "runner::turn_agent_composition::tool_call_recorder::tests::non_send_message_calls_keep_existing_results_arguments_and_duplicate_id_behavior",
        "runner::turn_agent_composition::tool_call_recorder::tests::producer_timestamps_are_preserved_without_wrapper_clock_or_monotonicity_assumptions",
    ]),
    ("generated-send-message-contracts", base + ["--no-fail-fast", "--test", "send_message_tool_contract", "--test", "session_conversation_state_contract", "--test", "generated_occurrence_codec_contract", "--", "--test-threads=1"], [
        "send_message_guard_rejects_when_shipping_state_is_waiting_on_user",
        "schema_matches_frozen_type_fences_and_attachment_schemes",
        "widget_schema_matches_frozen_choice_contract_and_normalization",
        "generated_send_message_schema_exposes_frozen_widget_constraints",
        "encoding_preserves_frozen_text_markdown_and_summary_projection",
        "send_message_uses_frozen_agent_v1_wire_types_and_tool_call_tag",
        "send_message_bridge_is_first_party_and_delegates_other_tools",
        "cursor_agent_message_resolves_shipping_title_fail_soft",
        "send_message_interaction_emits_frozen_initial_and_completed_agent_v1_wire",
        "awaiting_user_interaction_completes_with_frozen_error_result_before_rejecting",
        "box_media_resolution_matches_frozen_root_desktop_and_image_fences",
        "send_message_bridge_resolves_text_images_and_standalone_attachments_before_persisting",
        "agent_media_persistence_is_atomic_and_separates_images_from_attachments",
        "delivery_counter_tracks_only_successful_shipping_send_side_effects",
        "shipping_send_message_composes_cloud_title_and_generated_interaction_observation",
        "lfa_send_message_file_url_facade_preserves_admitted_metadata",
        "lfa_send_message_file_url_facade_refuses_unsafe_local_sources",
        "send_message_bridge_preserves_prepared_custody_on_unknown_resolution",
        "send_message_bridge_aborts_prepared_custody_on_known_resolution_error",
        "transcript_read_owner_matches_frozen_page_window_tail_and_thread_semantics",
        "durable_outline_resolves_frozen_user_steps_send_message_task_and_shell_semantics",
        "generated_send_message_text_success_survives_session_backfill_reopen",
        "generated_send_message_attachment_success_survives_session_backfill_reopen",
        "generated_send_message_empty_success_id_remains_compatible_after_reopen",
        "generated_send_message_initial_error_and_empty_results_never_backfill",
        "generated_send_message_absent_empty_and_unsupported_arguments_never_backfill",
        "generated_send_message_malformed_outer_envelopes_never_backfill",
        "generated_send_message_malformed_nested_payloads_never_backfill",
        "generated_send_message_effective_later_error_never_backfills",
        "generated_send_message_repeated_payloads_merge_through_session_backfill_reopen",
        "generated_send_message_effective_arguments_follow_protobuf_merge_semantics",
        "generated_send_message_intervening_tools_reset_success_before_backfill",
        "generated_send_message_final_other_tool_keeps_complete_oneof_selection",
        "generated_turn_and_user_message_shapes_match_frozen_agent_v1_fields",
        "generated_assistant_thinking_and_tool_boundaries_match_frozen_cases",
        "tool_steps_fail_closed_without_canonical_generated_json_projection",
        "frozen_tool_case_field_numbers_map_to_stable_names",
        "canonical_generated_projection_matches_frozen_send_message_to_json_semantics",
        "canonical_generated_projection_ignores_unknown_tool_message_fields",
        "canonical_generated_tool_encoder_round_trips_send_message_json",
        "canonical_generated_mcp_receipt_matches_independent_protobuf_wire",
        "canonical_generated_mcp_json_values_match_independent_protobuf_wire",
        "canonical_generated_tool_encoder_preserves_original_turn_structured_receipt",
        "canonical_generated_tool_encoder_preserves_scalar_and_empty_value_wire",
        "canonical_generated_tool_encoder_preserves_nested_lists_and_envelope_named_keys",
        "canonical_generated_tool_encoder_refuses_nonobject_structured_content",
    ]),
    ("integration", integration + ["--", "--test-threads=1"], required_integration),
    ("session-owner-namespace", base + ["--no-fail-fast", "--test", "session_paths_contract", "--test", "agent_lifecycle_contract", "--test", "agent_session_contract", "--", "--test-threads=1"], [
        "sni_reserved_human_root_and_portable_aliases_are_not_agent_session_paths",
        "sni_agent_reserved_namespace_preserves_human_grammar_and_valid_agent_spelling",
        "lifecycle_preflights_reserved_deletion_batches_without_touching_session_owners",
        "gateway_preflights_reserved_deletion_batches_without_touching_session_owners",
        "reserved_agent_session_operations_refuse_before_owner_access_and_preserve_human_state",
        "agent_and_human_db_owners_are_isolated_when_human_opens_first",
        "agent_and_human_db_owners_are_isolated_when_agent_opens_first",
        "session_paths_validate_agent_ids_and_preserve_connector_secret_sibling_layout",
        "deletion_runtime_runs_owner_hooks_around_durable_session_delete",
        "deletion_runtime_rolls_back_deleting_fence_when_predelete_drain_fails",
        "deleting_active_agent_selects_existing_successor_and_returns_its_transcript",
        "guarded_delete_rejects_stale_revision_before_destructive_hooks",
        "delete_session_closes_owned_blob_store_clears_directory_and_publishes_removal",
        "deletion_fence_rejects_new_transcript_work_before_durable_write",
        "checked_backend_target_lookup_retains_durable_deletion_refusal_after_reopen",
    ]),
    ("codex-model-identity", base + ["--lib", "extensions::inference::provider_session::codex_model_identity_tests::", "--", "--test-threads=1"], [
        "extensions::inference::provider_session::codex_model_identity_tests::" + name for name in [
            "cmi_saved_model_reaches_production_retry_and_later_admission",
            "cmi_checkpoint_retry_retains_model_history_and_single_effect",
            "cmi_direct_runner_fallback_is_frozen_before_tools_and_retry",
            "cmi_model_resolution_preserves_precedence_provider_isolation_and_fallbacks",
            "cmi_model_resolution_refuses_malformed_sources_pairs_and_raw_values",
            "cmi_known_checkpoint_mismatch_and_invalid_identity_have_no_effects",
            "cmi_cancelled_dispatch_precedes_model_transport_and_tools",
            "cmi_legacy_checkpoint_preserves_history_without_replaying_tools",
            "cmi_memory_and_summary_consume_the_correct_admitted_model",
            "cmi_transport_factory_is_thread_scoped_borrow_free_and_restored",
        ]
    ]),
    ("codex-workspace-binding", base + ["--lib", "extensions::inference::provider_session::codex_request_binding::tests::", "--", "--test-threads=1"], [
        "extensions::inference::provider_session::codex_request_binding::tests::" + name for name in [
            "cwb_o1_private_file_rotation_and_workspace_refusal",
            "cwb_o1_invalid_sources_do_not_initialize_binding",
            "cwb_o1_source_path_survives_environment_and_cwd_changes",
            "cwb_o1_concurrent_first_readers_keep_one_identity",
            "cwb_o1_refresh_workspace_gate_preserves_source_and_redacts",
            "cwb_o1_jwt_shape_claim_and_workspace_validation",
            "cwb_o1_binding_debug_redacts_source_and_credentials",
        ]
    ]),
    ("codex-workspace-http", base + ["--lib", "extensions::inference::provider_session::codex_request_workspace_tests::", "--", "--test-threads=1"], [
        "extensions::inference::provider_session::codex_request_workspace_tests::" + name for name in [
            "codex_workspace_real_http_retry_refuses_replaced_account_and_new_turn_binds_current",
            "codex_workspace_real_http_retry_accepts_same_workspace_rotation",
            "codex_workspace_real_http_checkpoint_refusal_preserves_history_and_effect_custody",
            "codex_workspace_real_http_refresh_same_workspace_persists_and_resends",
            "codex_workspace_real_http_refresh_refuses_missing_or_changed_workspace_without_write",
            "codex_workspace_real_http_refusals_precede_credentials_transport_and_tools",
            "codex_workspace_real_http_memory_keeps_bound_workspace_and_summary_captures_current",
            "codex_workspace_request_hook_restores_scope_and_rejects_stale_or_unexpected_targets",
            "codex_workspace_built_request_errors_keep_cancellation_precedence",
            "codex_workspace_actual_watchdog_preserves_credential_refusal_and_user_cancellation",
            "codex_workspace_admitted_summary_keeps_identity_across_compaction_phases",
            "synthesis_identity_tests::memory_synthesis_codex_retains_snapshot_model_route_and_rotating_token",
            "synthesis_identity_tests::memory_synthesis_codex_rejects_workspace_change_before_verification_and_recovers",
            "synthesis_identity_tests::memory_synthesis_codex_retries_share_identity_and_refuse_recapture",
            "synthesis_identity_tests::memory_synthesis_codex_temporal_refusal_preserves_metadata",
            "synthesis_identity_tests::memory_synthesis_codex_dispose_during_proposal_stops_verification",
            "synthesis_identity_tests::memory_synthesis_codex_admission_deadline_retains_identity_and_sticky_refusal",
        ]
    ]),
    ("codex-model-regressions", base + ["--no-fail-fast", "--test", "settings_service_contract", "--test", "inference_extension_composition_contract", "--test", "inference_codex_direct_contract", "--test", "inference_provider_session_contract", "--test", "fabushi_model_identity_production_contract", "--test", "runner_effect_custody_contract", "--", "--test-threads=1"], [
        "bound_defaults_are_isolated_from_cursor_and_do_not_change_route_authority",
        "checkpoint_model_identity_is_durable_and_cross_model_resume_fails_before_request",
        "codex_direct_resumes_a_legacy_checkpoint_without_model_identity",
        "fabushi_checkpoint_requires_normalized_identity_and_exact_selected_model",
        "persisted_fabushi_model_is_frozen_across_tools_retry_and_summarization",
        "accepted_checkpoint_retry_refuses_a_changed_fabushi_account",
        "accepted_checkpoint_retry_accepts_same_context_token_rotation",
        "admitted_turn_summary_retains_fabushi_model_and_request_binding",
        "responses_effects_after_checkpoint_without_checkpoint_and_between_calls_do_not_replay",
        "responses_checkpoint_io_failure_expiry_and_new_effect_cannot_accept_stale_coverage",
        "responses_pre_effect_retry_and_safe_checkpoint_retry_keep_model_and_completed_tool",
        "downstream_unknown_outcomes_block_responses_checkpoint_and_replay_after_local_return",
        "definitive_relay_results_remain_checkpoint_eligible_and_preserve_returned_data",
        "prepublication_refusal_and_list_failure_add_no_unknown_effect_custody",
        "timed_out_tool_keeps_worker_receipt_and_late_result_stays_unknown_across_attempts",
        "a_lost_tool_worker_cannot_turn_error_json_into_an_accepted_checkpoint",
        "user_cancellation_between_tools_and_during_checkpoint_io_wins_without_retry",
        "real_mcp_endpoint_keeps_immutable_attempt_admission_and_counts_effects",
        "real_mcp_response_loss_refuses_more_effects_on_the_same_attempt",
        "real_mcp_effect_prevents_provider_retry_after_attempt_expiry",
        "production_fabushi_executor_refuses_replay_after_tool_expiry",
        "production_generated_runner_preserves_terminal_tool_custody",
        "production_generated_runner_retains_unknown_external_shell_effects",
        "production_generated_runner_preserves_known_external_shell_outcomes",
        "production_external_shell_adapter_types_uncertain_results_and_errors",
    ]),
    ("memory-account-binding", base + ["--test", "fabushi_model_identity_production_contract", "same_runner_nudges_and_turn_memory_", "--", "--test-threads=1"], [
        "same_runner_nudges_and_turn_memory_refuse_a_changed_fabushi_account",
        "same_runner_nudges_and_turn_memory_accept_same_context_token_rotation",
    ]),
    ("checked-store", base + ["--lib", store_filter, "--", "--test-threads=1"], required_store),
    ("paired-device-sse", base + ["--lib", sse_filter, "--", "--exact", "--test-threads=1"], [sse_filter]),
    ("device-pairing-join", base + ["--test", "device_pairing_join_admission_contract", "--", "--test-threads=1"], [
        "join_storage_read_failure_is_distinct_and_recovers_without_consuming",
        "join_storage_write_failure_preserves_row_and_redacts_errors",
        "join_storage_commit_refusal_never_publishes_payload",
        "join_malformed_rows_are_consumed_durably_before_content_validation",
        "join_expiry_and_legacy_json_preserve_single_use_contract",
        "join_expiry_is_rechecked_after_a_delayed_successful_commit",
        "join_concurrent_file_owners_publish_one_payload",
        "join_gateway_reports_storage_failure_and_preserves_public_recovery",
        "join_deferred_commit_refusal_is_qualified_and_preserves_recovery",
    ]),
    ('device-join-limit-state', base + ['--lib', 'gateway_server::join_rate_limit_tests::', '--', '--test-threads=1'], [
        'gateway_server::join_rate_limit_tests::join_limit_defaults_window_lockout_and_retry_boundaries',
        'gateway_server::join_rate_limit_tests::join_limit_success_isolation_and_storage_error_accounting',
        'gateway_server::join_rate_limit_tests::join_limit_mapped_peers_share_strict_loopback_budget',
        'gateway_server::join_rate_limit_tests::join_limit_capacity_reclaims_expired_before_idle_and_keeps_lockouts',
        'gateway_server::join_rate_limit_tests::join_limit_live_and_queued_reservations_cannot_be_evicted',
        'gateway_server::join_rate_limit_tests::join_limit_serializes_whole_attempt_without_blocking_other_peers',
        'gateway_server::join_rate_limit_tests::join_limit_clock_is_sampled_after_queue_and_owner_settlement',
        'gateway_server::join_rate_limit_tests::join_limit_stop_refuses_queued_turn_but_settles_entered_result',
        'gateway_server::join_rate_limit_tests::join_limit_poisoned_admission_state_fails_closed',
        'gateway_server::join_rate_limit_tests::join_limit_reserved_turns_are_fifo_for_mixed_outcomes',
        'gateway_server::join_rate_limit_tests::join_limit_abandoned_or_exhausted_turns_fail_closed',
    ]),
    ('device-join-ingress-http', base + ['--test', 'gateway_join_admission_contract', '--', '--test-threads=1'], [
        'join_http_default_budget_ignores_headers_and_blocks_before_owner_callbacks',
        'join_http_namespace_failures_count_without_owner_access',
        'join_http_success_resets_budget_for_legacy_payloads',
        'join_http_storage_errors_neither_spend_nor_reset_budget',
        'join_http_last_slot_concurrency_settles_before_next_admission',
        'join_http_new_generation_keeps_old_inflight_state_separate',
        'join_http_lost_response_consumes_once_without_replay',
    ]),
    ('device-join-legacy-http', base + ['--test', 'gateway_server_contract', 'pairing_join_route', '--', '--test-threads=1'], [
        'gateway_pairing_join_route_reaches_sqlite_owner_and_burns_code',
        'gateway_public_pairing_join_route_redeems_once_and_rejects_query_or_empty_code',
    ]),
    ('device-join-tls', base + ['--test', 'gateway_server_contract', 'gateway_serves_real_tls_when_cert_and_key_are_configured', '--', '--exact', '--test-threads=1'], [
        'gateway_serves_real_tls_when_cert_and_key_are_configured',
    ]),
    ("memory-synthesis-identity", base + ["--lib", "extensions::memory::production::identity_tests::", "--", "--test-threads=1"], [
        "extensions::memory::production::identity_tests::production_native_synthesis_pins_model_and_rotates_token",
        "extensions::memory::production::identity_tests::production_native_synthesis_refuses_changed_context_without_writes",
        "extensions::memory::production::identity_tests::production_native_temporal_synthesis_refuses_changed_context_without_review",
        "extensions::memory::production::identity_tests::production_native_synthesis_retry_keeps_snapshot_identity",
        "extensions::memory::production::identity_tests::production_native_synthesis_retry_refuses_changed_context",
        "extensions::memory::production::identity_tests::production_native_synthesis_cancellation_after_worker_success_prevents_apply",
    ]),
    ("production-memory-synthesis", base + ["--lib", "extensions::memory::production::append_read_tests::", "--", "--test-threads=1"], [
        "extensions::memory::production::append_read_tests::production_synthesis_read_failure_preserves_temporal_bytes_and_recovers",
        "extensions::memory::production::append_read_tests::production_synthesis_preserves_invalid_and_stale_temporal_controls",
    ]),
    ("session-claim-integration", claim_integration + ["--", "--test-threads=1"], required_claim_integration),
    ("session-claim-commit-custody", base + ["--lib", fresh_filter, "--", "--test-threads=1"], required_fresh),
    ("device-pairing", base + ["--test", "device_pairing_contract", "--", "--test-threads=1"], pairing_required),
    ("device-pairing-rotation-authority", base + ["--test", "device_pairing_rotation_authority_contract", "--", "--test-threads=1"], rotation_owner_required),
    ("device-pairing-rotation-gateway", base + ["--test", "device_pairing_rotation_gateway_contract", "--", "--test-threads=1"], rotation_http_required),
    ("device-pairing-revocation-authority", base + ["--test", "device_pairing_revocation_authority_contract", "--", "--test-threads=1"], revocation_owner_required),
    ("device-pairing-revocation-gateway", base + ["--test", "device_pairing_revocation_gateway_contract", "--", "--test-threads=1"], revocation_http_required),
    ("device-role-token-revocation-authority", base + ["--test", "device_role_token_revocation_authority_contract", "--", "--test-threads=1"], role_token_owner_required),
    ("device-role-token-revocation-gateway", base + ["--test", "device_role_token_revocation_gateway_contract", "--", "--test-threads=1"], role_token_http_required),
    ("device-token-rotation-wire-authority", base + ["--test", "device_token_rotation_wire_authority_contract", "--", "--test-threads=1"], wire_rotation_owner_required),
    ("device-token-rotation-wire-gateway", base + ["--test", "device_token_rotation_wire_gateway_contract", "--", "--test-threads=1"], wire_rotation_http_required),
    ("device-pairing-retained-stream", base + ["--test", "device_pairing_retained_stream_contract", "--", "--test-threads=1"], retained_stream_required),
    ("received-catalog-preparation", base + ["--test", "runner_production_bridge_contract", "--", "--test-threads=1"], [
        "production_runner_catalog_refusal_stops_before_execution_and_ordinary_failure_continues_preparation",
    ]),
    ("received-catalog-projection", base + ["--test", "production_turn_input_projection_contract", "--", "--test-threads=1"], [
        "production_turn_agent_projection_keeps_root_presence_and_identity_frozen",
        "production_turn_agent_projection_refuses_unsupported_snapshot_capability",
        "production_turn_agent_projection_preserves_typed_catalog_refusal",
        "composition_and_live_toolset_projection_preserve_typed_catalog_refusal",
        "live_mcp_projection_preserves_first_seen_grouping_and_complete_descriptors",
        "frozen_mcp_projection_keeps_exact_vectors_without_live_discovery",
    ]),
    ("mcp-origin-canonical-boundary", base + ["--test", "mcp_state_executor_contract", "--", "--test-threads=1"], [
        "canonical_mcp_state_preserves_origin_in_process_and_never_serializes_relay_authority",
    ]),
    ("received-catalog-toolset", base + ["--test", "turn_toolset_contract", "--", "--test-threads=1"], [
        "dynamic_turn_toolset_propagates_catalog_refusal_through_native_wrapper",
        "dynamic_turn_toolset_keeps_ordinary_discovery_failure_fallback",
        "explicit_empty_subagent_does_not_observe_catalog_discovery_failure",
    ]),
    ("received-catalog-runtime", base + ["--test", "runner_routed_provider_contract", "--", "--test-threads=1"], [
        "runner_provider_registry_preserves_supplied_cancellation_and_registration_identity",
        "runner_provider_registry_rejects_empty_supplied_token_identity",
        "transcript_runner_registry_shares_direct_and_group_cancellation_across_attachment",
        "transcript_runner_registry_scoped_cancel_visits_both_registries_and_rejects_replacement",
        "received_catalog::invalid_refresh_clears_old_tools_until_valid_or_empty_recovery",
        "received_catalog::ordinary_unavailability_preserves_last_wins_public_alias_and_real_results",
        "received_catalog::unresolved_catalog_fault_survives_unavailable_refresh",
        "received_catalog::close_observes_entered_invalid_refresh_only_after_join",
        "received_catalog::worker_join_failure_cannot_erase_recorded_refusal_or_return_clean_observation",
        "received_catalog::claude_process::production_claude_catalog_fault_survives_cli_and_retry_boundaries",
        "received_catalog::claude_process::production_claude_catalog_refusal_retains_completed_success_and_server_error_without_replay",
    ]),
    ("preentry-cancellation-wiring", base + ["--no-fail-fast", "--test", "openclaw_chat_cron_node_contract", "--test", "turn_telemetry_production_wiring_contract", "--test", "group_member_runner_deletion_production_contract", "--", "--test-threads=1"], [
        "test_ocrm_node_o06_shipping_routes_to_unique_native_owners",
        "test_node_shell_timeout_oracle_rejects_missing_owner_call_and_dropped_timeout",
        "test_ocrm_chat_o05_abort_invokes_runner_registry_cancellation",
        "shipping_host_wires_frozen_turn_telemetry_sources",
        "group_member_registry_is_independent_and_preemption_state_is_consumable",
        "shipping_group_member_turn_preemption_and_deletion_use_the_independent_owner",
        "deletion_drain_tracks_pending_exclusive_turn_dispatch_until_all_lanes_settle",
    ]),
    ("received-catalog-retry", base + ["--lib", "runner::production_turn_run_shell_adapter::tests::received_mcp_catalog_permanent_retry_policy_preserves_custody_precedence", "--", "--exact", "--test-threads=1"], [
        "runner::production_turn_run_shell_adapter::tests::received_mcp_catalog_permanent_retry_policy_preserves_custody_precedence",
    ]),
    ("received-catalog-builder-regressions", base + ["--no-fail-fast", "--test", "external_machine_tool_contract", "--test", "sand_generate_image_tool_contract", "--test", "sand_multitask_todo_tool_contract", "--test", "sand_web_tools_contract", "--test", "turn_toolset_await_mcp_contract", "--", "--test-threads=1"], []),
    ("received-catalog-credential-match", base + ["--test", "fabushi_responses_credential_context_contract", "--", "--test-threads=1"], []),
    ("channel-transport-chunks", base + ["--test", "openclaw_channel_delivery_contract", "--", "--test-threads=1"], [
        "test_channel_splitter_preserves_plain_text_and_scalar_bounds",
        "test_channel_splitter_preserves_fenced_payload_and_header_spelling",
        "test_channel_splitter_does_not_promote_inline_backticks_to_fences",
        "test_channel_splitter_uses_exact_raw_fallback_for_unrepresentable_fences",
        "test_channel_router_bounds_chunks_and_preserves_delivery_identity",
        "test_channel_router_stops_after_effect_then_error_without_replay",
        "test_channel_router_preserves_attachment_and_absent_operation_identity",
        "test_ocrm_channel_o06_message_chunking_code_fence_preservation",
    ]),
    ("plugin-uninstall-service", base + ["--no-fail-fast", "--test", "mcp_service_contract", "--test", "plugin_skills_contract", "--test", "host_bundle_and_plugin_cache_contract", "--test", "workflow_store_contract", "--test", "workflow_commands_contract", "--test", "skill_publish_contract", "--test", "legacy_live_references_contract", "--test", "mcp_coordinator_relay_contract", "--test", "mcp_extension_contract", "--test", "mcp_production_contract", "--test", "host_production_extensions_contract", "--", "--test-threads=1"], [
        "helpers_preserve_frozen_staging_and_plugin_shape",
        "publish_removes_library_copy_only_after_matching_install_is_confirmed",
        "unconfirmed_publish_keeps_library_copy_and_retries_sync_five_times",
        "list_targets_and_unpublish_restore_follow_reference_fences",
        "uninstall_reservation_failure_prevents_backend_mutation",
        "uninstall_close_after_reservation_settles_only_not_dispatched",
        "confirmed_uninstall_consumers::desktop_unknown_uninstall_retains_pending_and_denies_actual_consumers",
        "confirmed_uninstall_consumers::intent_persistence_failure_prevents_real_backend_dispatch",
        "confirmed_uninstall_consumers::pending_uninstall_cannot_be_released_by_fresh_sync",
        "confirmed_uninstall_consumers::uninstall_custody_survives_owned_process_exit_and_fresh_process_consumers",
        "confirmed_uninstall_consumers::auth_refusal_survives_failed_publication_and_fresh_process_consumers",
        "confirmed_uninstall_consumers::auth_refresh_unknown_survives_owned_process_exit_and_recovers_without_fallback",
        "confirmed_uninstall_consumers::auth_refresh_process_lease_excludes_overlap_and_releases_after_failed_worker",
        "confirmed_uninstall_consumers::skill_publish_real_owner_holds_mutation_lease_through_resync_and_unpublish",
        "confirmed_uninstall_consumers::cross_process_uninstall_cycles_fence_unobserving_refresh_before_publication",
        "live_admission::real_workflow_mutation_rechecks_committed_publisher_and_record_after_lease",
        "live_admission::real_workflow_mutation_keeps_current_publisher_after_unchanged_publication",
        "desktop_service_contracts::desktop_observation_rejects_a_pre_fact_success_from_the_actual_loader",
        "desktop_service_contracts::late_desktop_uninstall_reply_cannot_settle_the_successor_service_request",
        "desktop_service_contracts::closed_desktop_service_never_replays_an_admitted_uninstall_or_dispatches_new_work",
    ]),
    ("plugin-uninstall-state", base + ["--lib", "extensions::mcp::plugin_skills::", "--", "--test-threads=1"], [
        "extensions::mcp::plugin_skills::sync_tests::pending_uninstall_survives_fresh_sync_without_release_or_prune",
        "extensions::mcp::plugin_skills::sync_tests::pending_uninstall_reports_only_admitted_changes_at_publication",
        "extensions::mcp::plugin_skills::sync_tests::uninstall_settlement_requires_the_exact_current_ticket_without_advancing_observation_on_refusal",
        "extensions::mcp::plugin_skills::sync_tests::uninstall_non_removal_preserves_prior_confirmed_and_auth_refusals",
        "extensions::mcp::plugin_skills::sync_tests::uninstall_reservation_fences_an_old_load_or_staged_pass_but_not_its_queued_fresh_load",
        "extensions::mcp::plugin_skills::sync_tests::uninstall_terminal_settlements_fence_loads_started_before_the_reply",
        "extensions::mcp::plugin_skills::sync_tests::uninstall_recovery_loads_exact_pending_and_confirmed_facts_without_invoking_the_loader",
        "extensions::mcp::plugin_skills::sync_tests::uninstall_recovery_fault_requires_a_fresh_owner_after_repair",
        "extensions::mcp::plugin_skills::sync_tests::uninstall_recovery_preserves_a_valid_unmarked_image_until_an_explicit_transition",
        "extensions::mcp::plugin_skills::sync_tests::uninstall_release_failure_after_valid_cache_publication_keeps_denial_and_files",
        "extensions::mcp::plugin_skills::sync_tests::pending_uninstall_same_root_successor_rejects_every_retired_settlement",
        "extensions::mcp::plugin_skills::uninstall_state::tests::absent_legacy_load_has_no_filesystem_side_effects",
        "extensions::mcp::plugin_skills::uninstall_state::tests::first_transition_and_empty_terminal_state_round_trip",
        "extensions::mcp::plugin_skills::uninstall_state::tests::first_transition_creates_the_missing_directory_chain",
        "extensions::mcp::plugin_skills::uninstall_state::tests::image_only_initialization_remains_authoritative_until_explicit_transition",
        "extensions::mcp::plugin_skills::uninstall_state::tests::marker_without_image_and_invalid_markers_are_never_legacy",
        "extensions::mcp::plugin_skills::uninstall_state::tests::strict_object_format_rejects_unknown_duplicate_missing_and_mistyped_fields",
        "extensions::mcp::plugin_skills::uninstall_state::tests::semantic_duplicates_are_refused_but_confirmed_and_pending_may_overlap",
        "extensions::mcp::plugin_skills::uninstall_state::tests::canonical_plugin_and_operation_identity_is_required_on_read_and_write",
        "extensions::mcp::plugin_skills::uninstall_state::tests::writes_are_sorted_and_opaque_canonical_ids_are_preserved_exactly",
        "extensions::mcp::plugin_skills::uninstall_state::tests::exact_byte_drift_and_marker_loss_prevent_replacement",
        "extensions::mcp::plugin_skills::uninstall_state::tests::stale_image_and_corruption_cannot_overwrite_current_custody",
        "extensions::mcp::plugin_skills::uninstall_state::tests::nonregular_entries_are_preserved_as_recovery_faults",
        "extensions::mcp::plugin_skills::uninstall_state::tests::dangling_and_regular_file_symlinks_are_not_missing_or_authoritative_images",
        "extensions::mcp::plugin_skills::uninstall_state::tests::staging_cleanup_removes_only_a_successfully_created_owned_file",
        "extensions::mcp::plugin_skills::uninstall_state::tests::marker_publication_failure_does_not_erase_the_visible_image",
        "extensions::mcp::plugin_skills::uninstall_state::tests::legacy_v1_migrates_only_on_an_explicit_transition_with_all_custody_retained",
        "extensions::mcp::plugin_skills::uninstall_state::tests::auth_identity_metadata_and_uninstall_custody_remain_distinct_and_sorted",
        "extensions::mcp::plugin_skills::uninstall_state::tests::v2_requires_strict_auth_objects_and_an_explicit_nullable_refresh_field",
        "extensions::mcp::plugin_skills::uninstall_state::tests::v2_auth_and_refresh_identity_reject_malformed_or_conflicting_values",
        "extensions::mcp::plugin_skills::uninstall_state::tests::failed_auth_write_accepts_only_exact_expected_or_its_visible_candidate",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_refresh_reservation_failure_never_calls_loader_or_changes_cache",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_negative_journal_failure_distinguishes_expected_and_candidate_recovery",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_live_negative_precedes_outcome_journal_while_fresh_reader_sees_unknown",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_known_loader_errors_and_invalid_aggregates_preserve_offline_availability",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_inherited_unknown_survives_offline_and_invalid_outcomes_without_replay",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_unknown_recovery_uses_only_fresh_records_while_clean_partial_keeps_fallback",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_exact_identity_zero_skill_release_keeps_current_denial_and_changes_once",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_positive_journal_failure_follows_cache_barriers_and_preserves_actual_visibility",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_foreign_or_unreadable_outcome_image_is_not_a_known_failed_write",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_old_loader_guard_and_repeated_dispose_cannot_alter_successor_token",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_panicking_coalesced_followup_retains_unknown_and_all_waiters_settle",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_reporter_panic_after_settled_publication_does_not_recreate_unknown",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_pending_uninstall_survives_unknown_recovery_and_exact_refusal_release",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_clean_dispose_does_no_io_but_fresh_construction_cannot_assume_the_outcome",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_failed_negative_write_survives_ordinary_workflow_reads_and_partial_refresh",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_unknown_ordinary_reads_cannot_restore_partial_fallback",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_refresh_process_lock_covers_cleanup_and_releases_at_worker_terminal",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_observed_state_loss_latches_fault_across_reads_and_restoration",
        "extensions::mcp::plugin_skills::uninstall_state::tests::legacy_v2_migrates_to_v3_without_inventing_uninstall_observation",
        "extensions::mcp::plugin_skills::uninstall_state::tests::v3_requires_explicit_nullable_canonical_uninstall_observation",
        "extensions::mcp::plugin_skills::uninstall_state::tests::v3_observation_cannot_be_dropped_by_an_explicit_image_transition",
        "extensions::mcp::plugin_skills::uninstall_state::tests::uninstall_observation_rotates_for_real_transitions_and_survives_auth_refreshes",
        "extensions::mcp::plugin_skills::uninstall_state::tests::retained_v3_observation_loss_to_legacy_or_null_latches_recovery",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_authoritative_reader_faults_latch_until_fresh_owner_reconstruction",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_contended_unlocked_image_error_does_not_latch_recovery_fault",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_contended_constructor_defers_unserialized_errors_until_explicit_recovery",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_contended_constructor_recovers_after_actual_first_image_publication",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_authoritative_constructor_corruption_remains_faulted_after_restore",
        "extensions::mcp::plugin_skills::auth_recovery_tests::auth_missing_root_constructor_preserves_legacy_absence_without_filesystem_writes",
    ]),
    ("plugin-uninstall-binding", base + ["--lib", "host_production_extensions::plugin_skills_binding_tests::", "--", "--test-threads=1"], [
        "host_production_extensions::plugin_skills_binding_tests::fresh_host_binding_recovers_uninstall_custody_before_either_startup_order",
        "host_production_extensions::plugin_skills_binding_tests::prepared_disk_recovery_cannot_replace_a_newer_same_root_reader",
        "host_production_extensions::plugin_skills_binding_tests::initialized_uninstall_state_loss_blocks_the_real_host_binding",
    ]),
    ("plugin-uninstall-relay-handoff", base + ["--lib", "extensions::mcp::coordinator_relay::lifecycle_tests::desktop_confirmation_keeps_the_actual_reader_across_same_root_handoff", "--", "--exact", "--test-threads=1"], [
        "extensions::mcp::coordinator_relay::lifecycle_tests::desktop_confirmation_keeps_the_actual_reader_across_same_root_handoff",
    ]),
    ("plugin-publish-cleanup-callback", base + ["--lib", "extensions::mcp::skill_publish::cleanup_callback_tests::", "--", "--test-threads=1"], [
        "extensions::mcp::skill_publish::cleanup_callback_tests::upload_cleanup_callbacks_release_admission_before_reentrant_sync_on_success_and_error",
    ]),
    ("plugin-cache-publication", base + ["--lib", "extensions::mcp::plugin_skills_cache::tests::", "--", "--test-threads=1"], []),
    ("plugin-session-consumer", base + ["--lib", "extensions::session::production::plugin_skills_admission_tests::", "--", "--test-threads=1"], []),
    ("provider-checkpoint-publication", base + ["--lib", "runner::routed_provider_runtime::checkpoint_publication_tests::", "--", "--test-threads=1"], [
        "runner::routed_provider_runtime::checkpoint_publication_tests::production_checkpoint_round_trip_preserves_payload_and_hashed_isolation",
        "runner::routed_provider_runtime::checkpoint_publication_tests::private_checkpoint_staging_and_published_file_have_owner_only_permissions",
        "runner::routed_provider_runtime::checkpoint_publication_tests::occupied_checkpoint_temporary_preserves_original_and_unrelated_entries",
        "runner::routed_provider_runtime::checkpoint_publication_tests::occupied_checkpoint_final_preserves_original_and_cleans_only_owned_temporary",
        "runner::routed_provider_runtime::checkpoint_publication_tests::checkpoint_nested_directories_publish_in_order_and_preserve_existing_modes",
        "runner::routed_provider_runtime::checkpoint_publication_tests::checkpoint_directory_barrier_failure_is_retried_before_descendants",
        "runner::routed_provider_runtime::checkpoint_publication_tests::checkpoint_final_barrier_failure_retains_visible_payload_without_success",
    ]),
    ("provider-checkpoint-custody", base + ["--test", "production_turn_run_shell_adapter_contract", "--", "--test-threads=1"], [
        "production_turn_adapter_real_store_refusal_retains_uncheckpointed_effect_and_never_retries",
        "production_turn_adapter_persists_checkpoint_before_resuming",
        "production_turn_adapter_does_not_accept_an_unpersisted_checkpoint",
        "production_turn_adapter_tool_only_checkpoint_rearms_watchdog_and_resumes",
    ]),
    ("generated-failure-history", base + ["--test", "production_checkpoint_restart_contract", "generated_failure_history_", "--", "--test-threads=1"], [
        "generated_failure_history_provider_failure_survives_repeated_fresh_consumers",
        "generated_failure_history_cancellation_survives_repeated_fresh_consumers",
        "generated_failure_history_provider_store_refusal_preserves_unaccepted_effect",
        "generated_failure_history_unknown_downstream_outcome_preserves_canonical_bytes",
        "generated_failure_history_zero_effect_refusal_preserves_canonical_bytes",
        "generated_failure_history_root_drift_preserves_competing_checkpoint",
        "generated_failure_history_later_failure_invalidates_prior_completion",
        "generated_failure_history_final_persistence_refusal_stays_ineligible_after_repair",
        "generated_failure_history_successful_control_preserves_original_completion",
    ]),
    ("canonical-root-admission", base + ["--test", "production_checkpoint_restart_contract", "canonical_root_admission_", "--", "--test-threads=1"], [
        "canonical_root_admission_valid_roots_advance_across_explicit_invocations",
        "canonical_root_admission_pre_projection_drift_prevents_provider_execution",
        "canonical_root_admission_post_projection_drift_prevents_provider_execution",
        "canonical_root_admission_strict_read_failures_preserve_durable_state",
        "canonical_root_admission_completed_owner_retirement_prevents_provider_execution",
        "canonical_root_admission_same_sink_staging_and_persistence_keep_captured_base",
        "canonical_root_admission_early_errors_permanently_revoke_prior_completion",
        "canonical_root_admission_post_durable_journal_failure_refuses_stale_sink",
    ]),
    ("canonical-prior-root-image", base + ["--lib", "prior_root_identity_tests::", "--", "--test-threads=1"], [
        "runner::production_agent_checkpoint::prior_root_identity_tests::prior_root_fresh_publication_refuses_old_hash_and_present_empty_omission",
        "runner::production_agent_checkpoint::prior_root_identity_tests::prior_root_reconciliation_accepts_only_the_exact_two_writer_images",
        "runner::production_agent_checkpoint::prior_root_identity_tests::prior_root_validation_preserves_absence_and_content_addressed_controls",
        "runner::production_agent_checkpoint::prior_root_identity_tests::prior_root_validation_refuses_duplicate_unsupported_and_malformed_user_images",
    ]),
    ("canonical-prior-root-process", base + ["--test", "production_checkpoint_restart_contract", "prior_root_historical_fixture::", "--", "--test-threads=1", "--skip", "prior_root_historical_fixture::export_base_writer_fixtures", "--skip", "prior_root_historical_fixture::consume_base_writer_fixtures"], [
        "prior_root_historical_fixture::current_writer_present_empty_root_has_exact_identity_and_fresh_completion",
        "prior_root_historical_fixture::current_writer_legacy_without_alias_has_exact_identity_and_fresh_completion",
        "prior_root_historical_fixture::rehashed_user_image_faults_reach_the_specific_fresh_process_semantic_refusal",
    ]),
    ("canonical-root-admission-contracts", base + ["--no-fail-fast", "--test", "production_turn_agent_owner_contract", "--test", "production_agent_checkpoint_contract", "--test", "production_agent_store_recovery_contract", "--test", "canonical_checkpoint_recovery_contract", "--test", "host_runner_composition_production_wiring_contract", "--test", "transcript_mirror_production_provider_contract", "--", "--test-threads=1"], [
        "production_owner_retains_not_started_when_input_preparation_refuses",
        "production_sink_rejects_stale_snapshot_before_mirror_recovery",
        "production_sink_keeps_invocation_snapshot_binding_after_own_success",
        "original_turn_observations_are_object_safe_unavailable_by_default_and_absent_until_real_commit",
        "original_turn_journal_commit_failure_after_durable_root_and_confirmation_cannot_mint_observation",
        "original_turn_prior_snapshot_is_checked_under_the_actual_store_mutation_guard",
        "composed_sink_refuses_durable_root_corruption_before_child_blobs_or_mirror_preparation",
        "already_composed_shipping_sink_rechecks_before_staging_and_before_persisting",
        "fresh_actual_owners_distinguish_no_root_from_persisted_empty_protobuf",
        "legacy_fixed_roots_and_direct_references_remain_readable_and_same_id_drift_refuses_reentry",
        "host_runner_composition_owns_transcript_checkpoint_wiring",
        "shipping_runner_binds_generated_checkpoint_codec_into_file_transcript_mirror",
    ]),
    ("box-response-manager", base + ["--test","box_remote_accessor_contract","--","--test-threads=1"], [
        "box_remote_accessor_builds_authenticated_binary_http11_transport",
        "box_remote_accessor_classifies_ping_failures_and_latency_like_grok",
        "box_remote_exec_manager_allocates_ids_filters_control_and_surfaces_remote_throw",
        "box_remote_resource_accessor_composes_transport_client_and_manager",
        "box_remote_exec_manager_preserves_ordered_data_and_heartbeat_before_close",
        "box_remote_exec_manager_requires_matching_data_and_control_identity",
        "box_remote_exec_manager_requires_one_final_stream_close",
        "box_remote_exec_manager_default_throw_id_is_only_request_zero",
        "box_remote_exec_manager_foreign_throw_cannot_become_remote_error",
        "box_remote_exec_manager_preserves_transport_loss_without_replay_or_id_reuse"
]),
    ("box-response-codec", base + ["--test","box_generated_production_contract","--","--test-threads=1"], [
        "production_exec_service_background_shell_spawn_uses_frozen_field_16_and_real_shell_id",
        "production_exec_service_preserves_shell_stdout_for_internal_consumers",
        "production_exec_service_streams_shell_and_write_through_shipping_accessor",
        "production_loopback_factory_gates_exec_accessor_on_authenticated_readiness",
        "production_exec_service_reads_through_protected_shipping_accessor",
        "production_exec_service_computer_use_has_remote_and_no_monitor_paths",
        "production_exec_service_shell_stream_uses_frozen_field_14_and_real_backgrounded_wire",
        "forever_box_runner_resource_port_background_spawn_registers_real_shell_id_with_unique_watcher",
        "forever_box_runner_resource_port_positive_block_backgrounds_real_shell_id_with_unique_watcher",
        "forever_box_runner_resource_port_reuses_authenticated_shipping_exec_service_for_turn",
        "production_exec_service_accepts_default_identity_and_nonzero_sequential_echo",
        "production_exec_service_decodes_metadata_on_both_sides_of_exec_and_shell_payloads",
        "production_exec_service_preserves_binary_read_bytes_with_complete_identity_and_framing",
        "production_exec_service_accepts_valid_chunk_extensions_trailers_and_length_or_close_delimiting",
        "production_exec_service_retains_matching_throw_and_connect_error_details",
        "production_exec_service_rejects_malformed_identity_metadata_and_ambiguous_oneofs",
        "production_exec_service_rejects_foreign_data_exec_id_heartbeat_close_and_throw",
        "production_exec_service_does_not_accept_omitted_zero_identity_for_a_nonzero_request",
        "production_exec_service_requires_exactly_one_close_after_data",
        "production_exec_service_requires_one_legal_final_connect_end",
        "production_exec_service_rejects_incomplete_or_ambiguous_http_framing",
        "production_exec_service_all_one_shot_accessors_require_one_expected_result",
        "production_exec_service_preserves_ordered_shell_stream_and_rejects_mixed_operations",
        "production_exec_service_distinguishes_pre_write_refusal_and_local_guards_from_unknown",
        "production_exec_service_rejects_out_of_range_direct_client_id_before_connect",
        "production_exec_service_wraps_post_write_status_and_eof_once_with_original_cause",
        "production_exec_service_never_replays_a_request_after_lost_completion",
        "forever_box_runner_resource_port_preserves_unknown_shell_read_write_and_background_outcomes",
        "forever_box_runner_resource_port_preserves_unknown_missing_terminal_and_invalid_background_id",
        "forever_box_runner_resource_port_keeps_no_monitor_as_pre_dispatch_refusal",
        "production_exec_service_rejects_invalid_control_identity_and_multiple_control_cases",
        "production_exec_service_rejects_ambiguous_nested_read_write_background_and_stream_results",
        "production_exec_service_rejects_shadowed_invalid_utf8_and_nested_metadata_once",
        "production_exec_service_rejects_shadowed_uint32_overflow_once",
        "production_exec_service_preserves_valid_repeated_typed_fields_and_last_values",
        "forever_box_runner_timed_shell_rejects_repeated_or_conflicting_terminals_before_watch",
        "forever_box_runner_timed_shell_preserves_post_exit_output_and_hook_metadata",
        "production_exec_service_preserves_sandbox_refusal_string_defaults_and_last_values",
        "production_exec_service_rejects_malformed_sandbox_policy_strings_without_replay",
        "forever_box_runner_preserves_known_sandbox_refusals_without_watch_or_replay",
        "forever_box_runner_preserves_unknown_malformed_sandbox_refusals_without_watch_or_replay",
        "production_exec_service_preserves_http_status_reason_and_header_octets",
        "production_exec_service_rejects_invalid_status_and_header_framing_before_result",
        "production_exec_service_rejects_non_200_streaming_status_without_replay"
]),
    ("box-response-runner-bridge", base + ["--test","runner_box_tool_composition_contract","--","--test-threads=1"], [
        "runner_box_bridge_merges_frozen_shell_and_read_with_host_tools",
        "runner_box_bridge_preserves_explicit_background_and_positive_block_until_semantics",
        "runner_box_bridge_preserves_upstream_dispatch_and_fails_closed_on_name_collision",
        "runner_box_bridge_validates_model_arguments_before_host_resource_execution",
        "runner_box_shell_review_fences_side_effect_before_resource_execution",
        "runner_box_bridge_preserves_unknown_box_execution_outcomes",
        "runner_box_bridge_preserves_known_box_refusals",
        "runner_box_bridge_preserves_unknown_mcp_spill_write_without_replay",
        "runner_box_bridge_keeps_known_mcp_spill_refusal_inline_without_replay"
]),
    ("box-response-spill", base + ["--test","large_output_spill_contract","--","--test-threads=1"], [
        "large_mcp_text_is_materialized_into_agent_box_once",
        "small_error_and_failed_write_results_stay_inline",
        "spill_policy_matches_frozen_threshold_disable_and_cap_contracts",
        "unknown_spill_write_is_not_published_as_inline_success_or_retried"
]),
    ("box-response-prompt-glue", base + ["--test","runner_prompt_glue_contract","--","--test-threads=1"], [
        "glue_reuses_file_transfer_owner_and_prompt_collector_projection",
        "glue_applies_single_large_output_spill_policy",
        "live_owner_reads_mutable_prompt_inputs_at_phase_time",
        "shipping_host_routes_mutable_prompt_surfaces_through_runner_owner",
        "glue_preserves_unknown_spill_and_disabled_policy_without_replay"
]),
    ("box-response-manager-exhaustion", base + ["--lib","box_exec_request_identity_exhaustion_refuses_before_serialization_or_transport","--","--test-threads=1"], [
        "box::box_remote_accessor::response_identity_tests::box_exec_request_identity_exhaustion_refuses_before_serialization_or_transport"
]),
    ("box-response-forever-box", base + ["--lib","extensions::forever_box::runner_tools::tests::","--","--test-threads=1"], [
        "extensions::forever_box::runner_tools::tests::normal_shell_projection_preserves_stdout_for_navigation_probes",
        "extensions::forever_box::runner_tools::tests::normal_shell_projection_preserves_failure_diagnostics",
        "extensions::forever_box::runner_tools::tests::computer_use_preserves_unknown_outcomes_from_incomplete_wire_completion"
]),
    ("box-response-real-daemon", base + ["--test","box_exec_daemon_process_contract","box_exec_response_roundtrip_uses_actual_daemon_identity_and_completion","--","--exact","--ignored","--test-threads=1"], [
        "box_exec_response_roundtrip_uses_actual_daemon_identity_and_completion"
]),
    ("box-response-shell-capture-unit", ["cargo", "test", "--locked", "--manifest-path", "source/box-exec-daemon/Cargo.toml", "--lib", "--target-dir", "source/box-exec-daemon/target", "server::tests::foreground_capture_", "--", "--test-threads=1"], ["server::tests::foreground_capture_faults_preserve_unknown_for_both_dispatches","server::tests::foreground_capture_complete_results_preserve_text_failure_and_refusal","server::tests::foreground_capture_timeout_and_abort_escalate_owned_term_ignoring_child","server::tests::foreground_capture_stop_race_retains_owner_until_reap_and_never_resignals","server::tests::foreground_capture_inherited_pipe_refuses_incomplete_eof_without_hanging","server::tests::foreground_capture_interrupted_read_retries_without_losing_bytes","server::tests::foreground_capture_reused_pid_retains_each_invocation_until_observed","server::tests::foreground_capture_identity_exhaustion_refuses_before_capture_or_stream_spawn","server::tests::foreground_capture_stream_stop_preserves_identity_and_never_signals_alias","server::tests::foreground_capture_stream_handoff_retains_reused_pid_capture","server::tests::foreground_capture_stream_setup_error_retains_owned_child_for_cleanup","server::tests::foreground_capture_stream_unwind_retains_readers_until_later_eof","server::tests::foreground_capture_stream_late_eof_is_observed_by_normal_admission","server::tests::foreground_capture_independent_runtimes_keep_distinct_spools_and_cleanup","server::tests::foreground_capture_occupied_spool_files_survive_failure_and_recovery","server::tests::foreground_capture_occupied_spool_symlinks_survive_failure_and_recovery","server::tests::foreground_capture_occupied_terminal_files_preserve_pending_reader_ownership","server::tests::foreground_capture_occupied_terminal_symlinks_preserve_pending_reader_ownership","server::tests::foreground_capture_retired_spool_claim_cannot_remove_reused_candidate"]),
    ("box-response-shell-capture-consumer", base + ["--test", "box_exec_daemon_process_contract", "box_foreground_capture_", "--", "--ignored", "--test-threads=1"], ["foreground_capture::box_foreground_capture_faults_reach_real_runner_custody_without_replay","foreground_capture::box_foreground_capture_definitive_results_remain_usable_and_checkpointed"]),
    ("box-response-shell-capture-disconnect", ["cargo", "test", "--locked", "--manifest-path", "source/box-exec-daemon/Cargo.toml", "--lib", "--target-dir", "source/box-exec-daemon/target", "server::tests::disconnect_aborts_foreground_shell_process_group", "--", "--exact", "--test-threads=1"], ["server::tests::disconnect_aborts_foreground_shell_process_group"]),
    ("box-response-background-custody-unit", ["cargo", "test", "--locked", "--manifest-path", "source/box-exec-daemon/Cargo.toml", "--lib", "--target-dir", "source/box-exec-daemon/target", "server::tests::background_custody_", "--", "--test-threads=1"], ["server::tests::background_custody_normal_and_handoff_preserve_output_stdin_and_autonomous_completion","server::tests::background_custody_pre_spawn_refusals_preserve_files_and_nonreusable_ids","server::tests::background_custody_setup_and_controller_refusals_retain_child_and_readers","server::tests::background_custody_stop_during_setup_retains_the_entered_request","server::tests::background_custody_stop_and_hard_timeout_force_only_the_owned_unreaped_child","server::tests::background_custody_reaped_child_keeps_readers_until_explicit_eof","server::tests::background_custody_controller_loss_recovers_through_the_same_entry","server::tests::background_custody_full_pipe_input_does_not_block_registry_or_stop","server::tests::background_custody_observed_read_wait_and_worker_failures_never_fabricate_exit","server::tests::background_custody_failed_writer_and_footer_publication_remain_pending","server::tests::background_custody_handoff_id_exhaustion_retains_entered_capture","server::tests::background_custody_setup_unwind_preserves_actual_child_and_capture_cleanup"]),
    ("box-response-background-custody-consumer", base + ["--test", "box_exec_daemon_process_contract", "box_background_custody_", "--", "--ignored", "--test-threads=1"], ["foreground_capture::box_background_custody_faults_reach_real_runner_without_replay_or_watch","foreground_capture::box_background_custody_pre_spawn_refusal_is_known_without_effect_or_watch","foreground_capture::box_background_custody_ordinary_terminal_waits_for_release_and_settles_once","foreground_capture::box_background_custody_timed_handoff_terminal_waits_for_release_and_settles_once"]),
    ("box-response-background-custody-footer", base + ["--test", "background_work_contract", "shell_terminal_footer_", "--", "--test-threads=1"], ["shell_terminal_footer_and_poll_interval_follow_frozen_rules","shell_terminal_footer_handles_canonical_producer_endings_and_actual_poll","shell_terminal_footer_preserves_last_footer_and_incomplete_snapshot_refusal"]),
    ("box-response-background-custody-retained", ["cargo", "test", "--locked", "--manifest-path", "source/box-exec-daemon/Cargo.toml", "--lib", "--target-dir", "source/box-exec-daemon/target", "server::tests::shell_", "--", "--test-threads=1"], ["server::tests::shell_environment_and_background_stdin_match_daemon_contract","server::tests::shell_stream_emits_stdout_before_child_finishes","server::tests::shell_stream_timeout_backgrounds_the_same_process_into_terminal_registry"]),
]

# CRF-v1 and BGC-v1 select from the retained full plan without deleting contracts.
# A scoped success is distinct from whole-contract acceptance.
full_cases = list(cases)
custody_box_labels = (
    "box-response-manager", "box-response-codec", "box-response-runner-bridge",
    "box-response-spill", "box-response-prompt-glue", "box-response-manager-exhaustion",
    "box-response-forever-box", "box-response-real-daemon",
    "box-response-shell-capture-unit", "box-response-shell-capture-consumer",
    "box-response-shell-capture-disconnect",
)
background_custody_labels = (*custody_box_labels,
    "box-response-background-custody-unit", "box-response-background-custody-consumer",
    "box-response-background-custody-footer", "box-response-background-custody-retained")
background_custody_required = {label: tuple(required)
    for label, _, required in full_cases if label in background_custody_labels}
# The failed normal campaign remains failed. Recovery executes only the repaired
# test and real retention/admission/autonomous controls, never the entire batch.
background_recovery_cases = (
    ("box-response-background-recovery-handoff", "box-response-background-custody-unit",
        "server::tests::background_custody_handoff_id_exhaustion_retains_entered_capture"),
    ("box-response-background-recovery-retention", "box-response-shell-capture-unit",
        "server::tests::foreground_capture_stream_unwind_retains_readers_until_later_eof"),
    ("box-response-background-recovery-admission", "box-response-shell-capture-unit",
        "server::tests::foreground_capture_stream_late_eof_is_observed_by_normal_admission"),
    ("box-response-background-recovery-autonomous", "box-response-background-custody-unit",
        "server::tests::background_custody_normal_and_handoff_preserve_output_stdin_and_autonomous_completion"),
)
background_recovery_source_labels = {parent for _, parent, _ in background_recovery_cases}
background_recovery_source_commands = {label: tuple(command)
    for label, command, _ in full_cases if label in background_recovery_source_labels}
background_recovery_labels = {label for label, _, _ in background_recovery_cases}

custody_desktop_names = (
    "desktop_service_contracts::desktop_observation_rejects_a_pre_fact_success_from_the_actual_loader",
    "desktop_service_contracts::late_desktop_uninstall_reply_cannot_settle_the_successor_service_request",
    "desktop_service_contracts::closed_desktop_service_never_replays_an_admitted_uninstall_or_dispatches_new_work",
)
djc_qualified_name = "join_deferred_commit_refusal_is_qualified_and_preserves_recovery"

def select_portable_cases(scope, declared):
    if scope not in ("full", "custody-recovery", "background-custody", "background-custody-recovery"):
        raise ValueError("Unknown portable contract scope")
    indexed = {label: (label, command, required) for label, command, required in declared}
    if len(indexed) != len(declared):
        raise ValueError("Duplicate portable case label")
    if scope == "full":
        return list(declared)
    if scope == "background-custody-recovery":
        for parent in background_recovery_source_labels:
            if parent not in indexed:
                raise ValueError("Background recovery lost its source command")
            _, command, required = indexed[parent]
            if (tuple(command) != background_recovery_source_commands[parent]
                or tuple(required) != background_custody_required[parent]
                or len(required) != len(set(required))):
                raise ValueError("Background recovery source command or names drifted")
        selected = []
        for label, parent, name in background_recovery_cases:
            if indexed[parent][2].count(name) != 1:
                raise ValueError("Background recovery needs its exact original test name")
            selected.append((label, ["cargo", "test", "--locked", "--manifest-path",
                "source/box-exec-daemon/Cargo.toml", "--lib", "--target-dir",
                "source/box-exec-daemon/target", name, "--", "--exact",
                "--test-threads=1"], [name]))
        if (len(selected) != 4 or len({label for label, _, _ in selected}) != 4
            or len({name for _, _, names in selected for name in names}) != 4):
            raise ValueError("Background recovery selection is incomplete or duplicated")
        return selected
    if scope == "background-custody":
        if any(label not in indexed for label in background_custody_labels):
            raise ValueError("Background custody lost a declared command")
        selected = [indexed[label] for label in background_custody_labels]
        if len(selected) != 15 or len({case[0] for case in selected}) != 15:
            raise ValueError("Background custody plan is incomplete or duplicated")
        if any(not required or len(set(required)) != len(required)
            or tuple(required) != background_custody_required[label]
            for label, _, required in selected):
            raise ValueError("Background custody needs the complete frozen test names")
        return selected
    needed = (*custody_box_labels, "plugin-uninstall-service",
        "plugin-uninstall-relay-handoff", "device-pairing-join")
    if any(label not in indexed for label in needed):
        raise ValueError("Focused recovery lost a declared command")
    selected = [indexed[label] for label in custody_box_labels]
    service_required = indexed["plugin-uninstall-service"][2]
    for index, name in enumerate(custody_desktop_names):
        if service_required.count(name) != 1:
            raise ValueError("Focused desktop oracle is absent or ambiguous")
        selected.append((f"plugin-custody-desktop-{index + 1}",
            base + ["--test", "mcp_coordinator_relay_contract", name,
                "--", "--exact", "--test-threads=1"], [name]))
    selected.append(indexed["plugin-uninstall-relay-handoff"])
    if indexed["device-pairing-join"][2].count(djc_qualified_name) != 1:
        raise ValueError("Focused causal COMMIT oracle is absent or ambiguous")
    selected.append(("device-pairing-join-qualified",
        base + ["--test", "device_pairing_join_admission_contract", djc_qualified_name,
            "--", "--exact", "--test-threads=1"], [djc_qualified_name]))
    if len(selected) != 16 or len({case[0] for case in selected}) != len(selected):
        raise ValueError("Focused recovery plan is incomplete or duplicated")
    if any(not required or len(set(required)) != len(required)
        for _, _, required in selected):
        raise ValueError("Focused recovery needs unique nonempty test names")
    if any("join_storage_commit_refusal_never_publishes_payload" in required
        for _, _, required in selected):
        raise ValueError("Unchanged failed lock instrument cannot be retried by this scope")
    return selected

selection_controls = []
for control, scope, declared, expected in [
    ("full-preserved", "full", full_cases, True),
    ("qualified-focused", "custody-recovery", full_cases, True),
    ("qualified-background", "background-custody", full_cases, True),
    ("background-duplicate-label", "background-custody", full_cases + [full_cases[0]], False),
    ("background-missing-unit", "background-custody",
        [case for case in full_cases if case[0] != "box-response-background-custody-unit"], False),
    ("background-missing-consumer", "background-custody",
        [case for case in full_cases if case[0] != "box-response-background-custody-consumer"], False),
    ("background-missing-footer", "background-custody",
        [case for case in full_cases if case[0] != "box-response-background-custody-footer"], False),
    ("background-missing-retained-shell", "background-custody",
        [case for case in full_cases if case[0] != "box-response-background-custody-retained"], False),
    ("background-missing-retained-shell-name", "background-custody",
        [(label, command, required[1:] if label == "box-response-background-custody-retained" else required)
            for label, command, required in full_cases], False),
    ("background-missing-unit-name", "background-custody",
        [(label, command, required[1:] if label == "box-response-background-custody-unit" else required)
            for label, command, required in full_cases], False),
    ("background-missing-consumer-name", "background-custody",
        [(label, command, required[1:] if label == "box-response-background-custody-consumer" else required)
            for label, command, required in full_cases], False),
    ("background-missing-footer-name", "background-custody",
        [(label, command, required[1:] if label == "box-response-background-custody-footer" else required)
            for label, command, required in full_cases], False),
    ("background-duplicate-required-name", "background-custody",
        [(label, command, required + required if label == "box-response-background-custody-unit" else required)
            for label, command, required in full_cases], False),
    ("background-foreign-required-name", "background-custody",
        [(label, command, ["foreign::unrelated"] if label == "box-response-background-custody-consumer" else required)
            for label, command, required in full_cases], False),
    ("unknown-scope", "unqualified", full_cases, False),
    ("duplicate-label", "custody-recovery", full_cases + [full_cases[0]], False),
    ("missing-box-command", "custody-recovery",
        [case for case in full_cases if case[0] != "box-response-shell-capture-unit"], False),
    ("missing-relay-command", "custody-recovery",
        [case for case in full_cases if case[0] != "plugin-uninstall-relay-handoff"], False),
    ("missing-desktop-name", "custody-recovery",
        [(label, command, [name for name in required if name != custody_desktop_names[0]])
            for label, command, required in full_cases], False),
    ("missing-causal-commit-name", "custody-recovery",
        [(label, command, [name for name in required if name != djc_qualified_name])
            for label, command, required in full_cases], False),
    ("duplicate-required-name", "custody-recovery",
        [(label, command, required + required if label == "box-response-shell-capture-unit" else required)
            for label, command, required in full_cases], False),
]:
    try:
        selected = select_portable_cases(scope, declared)
        actual = selected == full_cases if scope == "full" else True
    except ValueError:
        actual = False
    selection_controls.append({"case": control, "expected": expected, "actual": actual})
    assert actual == expected, f"CRF selection oracle failed: {control}"
background_recovery_selection_controls = []
background_recovery_variants = [("qualified-focused-recovery", full_cases, True),
    ("duplicate-source-label", full_cases + [full_cases[0]], False)]
for parent in sorted(background_recovery_source_labels):
    background_recovery_variants.extend([
        (f"missing-source:{parent}", [case for case in full_cases if case[0] != parent], False),
        (f"changed-command:{parent}", [(label, command + ["--foreign"], required)
            if label == parent else (label, command, required)
            for label, command, required in full_cases], False),
        (f"duplicate-source-name:{parent}", [(label, command, required + required[:1])
            if label == parent else (label, command, required)
            for label, command, required in full_cases], False),
    ])
for _, parent, name in background_recovery_cases:
    background_recovery_variants.extend([
        (f"missing-positive:{name}", [(label, command, [item for item in required if item != name])
            if label == parent else (label, command, required)
            for label, command, required in full_cases], False),
        (f"foreign-positive:{name}", [(label, command,
            ["foreign::unrelated" if item == name else item for item in required])
            if label == parent else (label, command, required)
            for label, command, required in full_cases], False),
    ])
for control, declared, expected in background_recovery_variants:
    try:
        selected = select_portable_cases("background-custody-recovery", declared)
        actual = len(selected) == 4
    except ValueError:
        actual = False
    background_recovery_selection_controls.append(
        {"case": control, "expected": expected, "actual": actual})
    assert actual == expected, f"BGC recovery selection oracle failed: {control}"

cases = select_portable_cases(validation_scope, full_cases)

def successful_test_names(output):
    # libtest decorates an expected-panic success without changing its name.
    return re.findall(r"^test (\S+)(?: - should panic)? \.\.\. ok$", output, re.MULTILINE)

def successful_test_summaries(output):
    return [tuple(map(int, count)) for count in re.findall(
        r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)]

def canonical_contract_results_accepted(cargo_exit, capture_exit, required, output):
    names = successful_test_names(output)
    summaries = successful_test_summaries(output)
    return (cargo_exit == 0 and capture_exit == 0
        and all(names.count(name) == 1 for name in required)
        and len(summaries) == 6
        and all(count[0] > 0 and count[1:] == (0, 0) for count in summaries)
        and sum(count[0] for count in summaries) == len(names))

# These qualify the capture oracle; they are not product test positives.
oracle_names = ["expected_panic"] + [f"ordinary_{index}" for index in range(5)]
oracle_summary = "test result: ok. 1 passed; 0 failed; 0 ignored;\n"
oracle_text = "".join(f"test {name}{' - should panic' if index == 0 else ''} ... ok\n" + oracle_summary
    for index, name in enumerate(oracle_names))
oracle_controls = []
for label, output, cargo_exit, capture_exit, expected in [
    ("standard-expected-panic-success", oracle_text, 0, 0, True),
    ("standard-ordinary-success", oracle_text.replace(" - should panic", ""), 0, 0, True),
    ("expected-panic-did-not-occur", oracle_text.replace(" - should panic ... ok", " - should panic ... FAILED"), 0, 0, False),
    ("wrong-panic-message", oracle_text.replace(" - should panic ... ok", " - should panic ... FAILED") + "panic did not contain expected string\n", 101, 0, False),
    ("ordinary-failure", oracle_text.replace("test ordinary_0 ... ok", "test ordinary_0 ... FAILED"), 0, 0, False),
    ("ignored-case", oracle_text.replace("test expected_panic - should panic ... ok", "test expected_panic - should panic ... ignored"), 0, 0, False),
    ("ignored-summary", oracle_text.replace(oracle_summary, "test result: ok. 1 passed; 0 failed; 1 ignored;\n", 1), 0, 0, False),
    ("duplicate-name", oracle_text.replace("ordinary_0", "expected_panic"), 0, 0, False),
    ("malformed-decoration", oracle_text.replace(" - should panic", " - unexpected decoration"), 0, 0, False),
    ("trailing-content", oracle_text.replace(" - should panic ... ok", " - should panic ... ok extra"), 0, 0, False),
    ("missing-summary", oracle_text.replace(oracle_summary, "", 1), 0, 0, False),
    ("zero-tests", "", 0, 0, False),
    ("command-refusal", oracle_text, 101, 0, False),
    ("capture-refusal", oracle_text, 0, 1, False),
]:
    actual = canonical_contract_results_accepted(cargo_exit, capture_exit, oracle_names, output)
    oracle_controls.append({"case": label, "expected": expected, "actual": actual})
    assert actual == expected, f"CRA result oracle control failed: {label}"


# BXR name normalization is limited to the source spelling of the one raw
# keyword module. Both legal libtest spellings map to one exact required name;
# duplicate aliases, foreign owners and additional positives are rejected.
box_response_exhaustion_name = "box::box_remote_accessor::response_identity_tests::box_exec_request_identity_exhaustion_refuses_before_serialization_or_transport"
box_response_filtered_cases = {"box-response-manager-exhaustion", "box-response-forever-box", "box-response-real-daemon", "box-response-shell-capture-unit", "box-response-shell-capture-consumer", "box-response-shell-capture-disconnect", "box-response-background-custody-unit", "box-response-background-custody-consumer", "box-response-background-custody-footer", "box-response-background-custody-retained"}

box_response_filtered_cases.update(background_recovery_labels)

def box_response_observed_names(output):
    names = successful_test_names(output.replace("\r\n", "\n"))
    return [box_response_exhaustion_name if name == "r#" + box_response_exhaustion_name else name
        for name in names]

def exact_target_results_accepted(cargo_exit, capture_exit, required, output, allow_filtered):
    text = output.replace("\r\n", "\n")
    names = box_response_observed_names(text)
    complete = [tuple(map(int, row)) for row in re.findall(
        r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out; finished in [0-9.]+s[ \t]*$", text, re.MULTILINE)]
    return (cargo_exit == 0 and capture_exit == 0 and bool(required)
        and len(set(required)) == len(required)
        and sorted(names) == sorted(required)
        and len(complete) == 1
        and complete[0][:4] == (len(required), 0, 0, 0)
        and (allow_filtered or complete[0][4] == 0))

def box_response_results_accepted(label, cargo_exit, capture_exit, required, output):
    return exact_target_results_accepted(cargo_exit, capture_exit, required, output,
        label in box_response_filtered_cases)

box_response_positive = (
    f"test {box_response_exhaustion_name} ... ok\n"
    "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s\n"
)
box_response_oracle_controls = []
for label, case, output, cargo_exit, capture_exit, expected in [
    ("exact-owner-positive", "box-response-manager-exhaustion", box_response_positive, 0, 0, True),
    ("raw-keyword-owner-positive", "box-response-manager-exhaustion", box_response_positive.replace("test box::", "test r#box::"), 0, 0, True),
    ("crlf-positive", "box-response-manager-exhaustion", box_response_positive.replace("\n", "\r\n"), 0, 0, True),
    ("missing-positive", "box-response-manager-exhaustion", box_response_positive.split("\n", 1)[1], 0, 0, False),
    ("duplicate-positive", "box-response-manager-exhaustion", box_response_positive + f"test {box_response_exhaustion_name} ... ok\n", 0, 0, False),
    ("duplicate-raw-alias", "box-response-manager-exhaustion", box_response_positive + f"test r#{box_response_exhaustion_name} ... ok\n", 0, 0, False),
    ("foreign-owner", "box-response-manager-exhaustion", box_response_positive.replace("box::box_remote_accessor::", "other::box_remote_accessor::"), 0, 0, False),
    ("unexpected-positive", "box-response-manager-exhaustion", box_response_positive + "test unrelated ... ok\n", 0, 0, False),
    ("ignored-positive", "box-response-manager-exhaustion", box_response_positive.replace(" ... ok", " ... ignored"), 0, 0, False),
    ("ignored-summary", "box-response-manager-exhaustion", box_response_positive.replace("0 ignored", "1 ignored"), 0, 0, False),
    ("zero-tests", "box-response-manager-exhaustion", "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n", 0, 0, False),
    ("duplicate-summary", "box-response-manager-exhaustion", box_response_positive + box_response_positive.split("\n", 1)[1], 0, 0, False),
    ("command-refusal", "box-response-manager-exhaustion", box_response_positive, 101, 0, False),
    ("capture-refusal", "box-response-manager-exhaustion", box_response_positive, 0, 1, False),
    ("unexpected-measured", "box-response-manager-exhaustion", box_response_positive.replace("0 measured", "1 measured"), 0, 0, False),
    ("filtered-whole-target", "box-response-manager", box_response_positive, 0, 0, False),
]:
    actual = box_response_results_accepted(case, cargo_exit, capture_exit,
        [box_response_exhaustion_name], output)
    box_response_oracle_controls.append({"case": label, "expected": expected, "actual": actual})
    assert actual == expected, f"BXR result oracle control failed: {label}"


# Exact selected Plugin/DJC commands use the same strict single-target oracle.
selected_oracle_name = custody_desktop_names[0]
selected_oracle_text = (f"test {selected_oracle_name} ... ok\n"
    "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.01s\n")
selected_result_controls = []
for control, output, command_exit, capture_exit, expected in [
    ("exact-filtered-positive", selected_oracle_text, 0, 0, True),
    ("missing-positive", selected_oracle_text.split("\n", 1)[1], 0, 0, False),
    ("duplicate-positive", selected_oracle_text + f"test {selected_oracle_name} ... ok\n", 0, 0, False),
    ("extra-positive", selected_oracle_text + "test unrelated ... ok\n", 0, 0, False),
    ("ignored-positive", selected_oracle_text.replace(" ... ok", " ... ignored"), 0, 0, False),
    ("ignored-summary", selected_oracle_text.replace("0 ignored", "1 ignored"), 0, 0, False),
    ("measured-summary", selected_oracle_text.replace("0 measured", "1 measured"), 0, 0, False),
    ("duplicate-summary", selected_oracle_text + selected_oracle_text.split("\n", 1)[1], 0, 0, False),
    ("zero-tests", "", 0, 0, False),
    ("command-refusal", selected_oracle_text, 101, 0, False),
    ("capture-refusal", selected_oracle_text, 0, 1, False),
]:
    actual = exact_target_results_accepted(command_exit, capture_exit,
        [selected_oracle_name], output, True)
    selected_result_controls.append({"case": control, "expected": expected, "actual": actual})
    assert actual == expected, f"CRF result oracle failed: {control}"

box_daemon_lock = pathlib.Path("source/box-exec-daemon/Cargo.lock")
box_daemon_lock_tracked = subprocess.run(
    ["git", "cat-file", "-e", f"{head}:source/box-exec-daemon/Cargo.lock"],
    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False).returncode == 0
if box_daemon_lock_tracked:
    box_response_inputs.append(str(box_daemon_lock))
    inputs = sorted(set(inputs) | {str(box_daemon_lock)})
else:
    assert not box_daemon_lock.exists(), "Untracked daemon lock would be an unbound build input"

def source_bindings(paths):
    bindings = {}
    for path in paths:
        actual = pathlib.Path(path).read_bytes()
        committed = subprocess.check_output(["git", "show", f"{head}:{path}"])
        bindings[path] = {
            "git_blob": subprocess.check_output(["git", "rev-parse", f"{head}:{path}"], text=True).strip(),
            "sha256": hashlib.sha256(actual).hexdigest(), "bytes": len(actual),
            "matches_head_bytes": actual == committed,
        }
    return bindings

custody_recovery_inputs = [
    "projects/openclaw-rust/upstream.lock.json",
    "projects/openclaw-rust/docs/specs/custody-recovery-verification.md",
    "projects/openclaw-rust/docs/specs/device-pairing-join-admission.md",
    "source/host/tests/device_pairing_join_admission_contract.rs",
    "source/host/src/device_pairing_owner.rs",
    "source/host/src/extensions/mcp/coordinator_relay.rs",
    "source/host/src/extensions/mcp/mcp_service.rs",
    "source/host/app/Cargo.toml", "source/host/app/Cargo.lock",
    "source/host/app/src/main.rs",
]
custody_recovery_inputs_before = source_bindings(custody_recovery_inputs)
assert all(item["matches_head_bytes"] for item in custody_recovery_inputs_before.values()), "CRF source input differs from exact H"
plugin_admission_lock_inputs_before = source_bindings(plugin_admission_lock_inputs)
assert all(item["matches_head_bytes"] for item in plugin_admission_lock_inputs_before.values()), "Plugin admission lock source input differs from exact H"
local_exec_outcome_inputs_before = source_bindings(local_exec_outcome_inputs)
assert all(item["matches_head_bytes"] for item in local_exec_outcome_inputs_before.values()), "LXO source input differs from exact H"
box_response_inputs_before = source_bindings(box_response_inputs)
assert all(item["matches_head_bytes"] for item in box_response_inputs_before.values()), "BXR source input differs from exact H"


receipt = {
    "validation_scope": validation_scope,
    "scope_selection_controls": selection_controls,
    "selected_result_oracle_controls": selected_result_controls,
    "custody_recovery_inputs_before": custody_recovery_inputs_before,
    "full_contract_acceptance": "not-assessed-by-this-selected-receipt",
    "unexecuted_full_plan_cases": [label for label, _, _ in full_cases
        if label not in {selected[0] for selected in cases}],
    "retained_unresolved_checks": [{
        "case": "join_storage_commit_refusal_never_publishes_payload",
        "subject": "6d8c241b609948c4266e1e40c8f559795b317473",
        "run": "41618863245", "attempt": 1, "job": "125859112570",
        "state": "historical-eight-second-observation-failed; unchanged assertion retained",
        "selected_here": validation_scope == "full",
        "qualification": "DJC-07 success does not resolve this lock timing failure",
    }],
    "plugin_admission_lock_contract": "PS-LOCK-RECONCILE-v2",
    "plugin_admission_lock_inputs_before": plugin_admission_lock_inputs_before,
    "plugin_admission_lock_required_positive_names": ["extensions::mcp::plugin_skills::auth_recovery_tests::auth_failed_negative_write_survives_ordinary_workflow_reads_and_partial_refresh","extensions::mcp::plugin_skills::auth_recovery_tests::auth_unknown_ordinary_reads_cannot_restore_partial_fallback","extensions::mcp::plugin_skills::auth_recovery_tests::auth_refresh_process_lock_covers_cleanup_and_releases_at_worker_terminal","confirmed_uninstall_consumers::auth_refresh_process_lease_excludes_overlap_and_releases_after_failed_worker","extensions::mcp::plugin_skills::auth_recovery_tests::auth_observed_state_loss_latches_fault_across_reads_and_restoration","confirmed_uninstall_consumers::skill_publish_real_owner_holds_mutation_lease_through_resync_and_unpublish","extensions::mcp::plugin_skills::uninstall_state::tests::legacy_v2_migrates_to_v3_without_inventing_uninstall_observation","extensions::mcp::plugin_skills::uninstall_state::tests::v3_requires_explicit_nullable_canonical_uninstall_observation","extensions::mcp::plugin_skills::uninstall_state::tests::v3_observation_cannot_be_dropped_by_an_explicit_image_transition","extensions::mcp::plugin_skills::uninstall_state::tests::uninstall_observation_rotates_for_real_transitions_and_survives_auth_refreshes","extensions::mcp::plugin_skills::uninstall_state::tests::retained_v3_observation_loss_to_legacy_or_null_latches_recovery","extensions::mcp::plugin_skills::auth_recovery_tests::auth_authoritative_reader_faults_latch_until_fresh_owner_reconstruction","extensions::mcp::plugin_skills::auth_recovery_tests::auth_contended_unlocked_image_error_does_not_latch_recovery_fault","confirmed_uninstall_consumers::cross_process_uninstall_cycles_fence_unobserving_refresh_before_publication","live_admission::real_workflow_mutation_rechecks_committed_publisher_and_record_after_lease","live_admission::real_workflow_mutation_keeps_current_publisher_after_unchanged_publication","extensions::mcp::skill_publish::cleanup_callback_tests::upload_cleanup_callbacks_release_admission_before_reentrant_sync_on_success_and_error","extensions::mcp::plugin_skills::auth_recovery_tests::auth_contended_constructor_defers_unserialized_errors_until_explicit_recovery","extensions::mcp::plugin_skills::auth_recovery_tests::auth_contended_constructor_recovers_after_actual_first_image_publication","extensions::mcp::plugin_skills::auth_recovery_tests::auth_authoritative_constructor_corruption_remains_faulted_after_restore","extensions::mcp::plugin_skills::auth_recovery_tests::auth_missing_root_constructor_preserves_legacy_absence_without_filesystem_writes"],
    "plugin_admission_lock_scope": "Existing Rust Skills admission and durable image owner preserves live denial and unknown recovery. Strict v3 uninstallObservation fences a foreign reserve/settle ABA across loader, publication and prune; legacy v1/v2 read facts remain. Authoritative SH image faults latch retained recovery; contended reads and constructors remain observation-only, with explicit fresh recovery required. Workflow mutation re-reads canonical current record, publisher and path under the actual lease. Actual publisher cleanup releases its lease before reentrant diagnostics. All retained assertions and OS/process controls remain required. Real accounts/ecosystem, hostile directory replacement, power loss, performance and supported-platform package acceptance remain open.",
    "local_exec_outcome_contract": "LXO-v1",
    "local_exec_outcome_inputs_before": local_exec_outcome_inputs_before,
    "local_exec_outcome_required_runner_names": ["production_generated_runner_retains_unknown_external_shell_effects","production_generated_runner_preserves_known_external_shell_outcomes","production_external_shell_adapter_types_uncertain_results_and_errors"],
    "local_exec_outcome_scope": "Actual registered generated Runner, production ExternalShell adapter, shared Host LocalExec and permission owner through real bridge delivery. Counted external effects, unknown post-entry outcomes, sticky custody, no uncertain checkpoint/replay, known refusal/Exit and fresh-owner journal controls use fixture-owned peers. Separate native three-platform and actual App/protocol receipts remain required; no physical device, external account, full process-tree, package or global acceptance.",
    "box_shell_capture_contract": "BXR-SHELL-CAPTURE-v1",
    "box_shell_capture_required_names": ["server::tests::foreground_capture_faults_preserve_unknown_for_both_dispatches","server::tests::foreground_capture_complete_results_preserve_text_failure_and_refusal","server::tests::foreground_capture_timeout_and_abort_escalate_owned_term_ignoring_child","server::tests::foreground_capture_stop_race_retains_owner_until_reap_and_never_resignals","server::tests::foreground_capture_inherited_pipe_refuses_incomplete_eof_without_hanging","server::tests::foreground_capture_interrupted_read_retries_without_losing_bytes","server::tests::foreground_capture_reused_pid_retains_each_invocation_until_observed","server::tests::foreground_capture_identity_exhaustion_refuses_before_capture_or_stream_spawn","server::tests::foreground_capture_stream_stop_preserves_identity_and_never_signals_alias","server::tests::foreground_capture_stream_handoff_retains_reused_pid_capture","server::tests::foreground_capture_stream_setup_error_retains_owned_child_for_cleanup","server::tests::foreground_capture_stream_unwind_retains_readers_until_later_eof","server::tests::foreground_capture_stream_late_eof_is_observed_by_normal_admission","server::tests::foreground_capture_independent_runtimes_keep_distinct_spools_and_cleanup","server::tests::foreground_capture_occupied_spool_files_survive_failure_and_recovery","server::tests::foreground_capture_occupied_spool_symlinks_survive_failure_and_recovery","server::tests::foreground_capture_occupied_terminal_files_preserve_pending_reader_ownership","server::tests::foreground_capture_occupied_terminal_symlinks_preserve_pending_reader_ownership","server::tests::foreground_capture_retired_spool_claim_cannot_remove_reused_candidate","foreground_capture::box_foreground_capture_faults_reach_real_runner_custody_without_replay","foreground_capture::box_foreground_capture_definitive_results_remain_usable_and_checkpointed","server::tests::disconnect_aborts_foreground_shell_process_group"],
    "box_background_recovery_selection_controls": background_recovery_selection_controls,
    "box_background_recovery_selected_here": validation_scope == "background-custody-recovery",
    "box_background_recovery_required_names": {label: [name] for label, _, name in background_recovery_cases},
    "box_background_recovery_prior_failed_subject": "6d3806857b58887245ae861c15c0ecb740e5d7d6",
    "box_background_custody_contract": "BGC-v1",
    "box_background_custody_required_names": {label: list(background_custody_required[label])
        for label in background_custody_labels if label.startswith("box-response-background-custody-")},
    "box_background_custody_selected_here": validation_scope in ("full", "background-custody"),
    "box_background_custody_scope": "Existing daemon background registry retains actual Child, readers, controller and file outcomes; ordinary/timed admission, bounded stop and no stale signaling, pre-spawn refusal and entered uncertainty, exact Host/ForeverBox/Runner custody and actual terminal-watch settlement. Canonical terminal final bytes are consumed without test rewriting. Private operation-fault producer remains separately identified from the shipping daemon. Normal active streamed shutdown, arbitrary descendant trees, terminal authenticity, durable restart, real services/devices, packages and full OCRM acceptance remain open.",
    "box_shell_capture_scope": "Actual daemon foreground capture distinguishes pre-spawn known refusal from entered read/wait/worker uncertainty for ordinary Shell and MiniSwe. Owned children and incomplete captures remain in the same registry under nonreusable invocation identities through bounded observed cleanup; exhaustion refuses before spawn and streamed stop/handoff preserves exact invocation custody. Exceptional stream returns/unwinds retain the real Child and started readers; completed recovery is observed after bounded cleanup or at ordinary/stream admission without requiring shutdown, while inherited-pipe custody remains pending until actual EOF; actual Throw/Close reaches Host, ForeverBox, Runner sticky custody, no replay and checkpoint refusal. Private cfg(test) operation faults are bound to a separately identified library test artifact; normal shipping daemon controls remain independent. Descendant-process closure, hostile ownership changes, raw HTML provenance, power loss, platform packages and real services remain unaccepted.",
    "box_response_contract": "BXR-v1.3",
    "box_response_scope": "Actual existing manager/accessor identity, protobuf/HTTP/Connect completion, typed post-write uncertainty, ForeverBox Shell/Read/Write/ComputerUse projection and actual MCP spill bridge. Real owned daemon Shell/binary Write/Read uses the same production accessor and nonzero identities. Controlled peers do not prove external services, real monitor devices, cancellation, raw HTML capture, persistence, packages or all-platform compatibility. Existing downstream effect-custody tests are reused inside codex-model-regressions.",
    "box_response_inputs_before": box_response_inputs_before,
    "box_response_result_oracle_controls": box_response_oracle_controls,
    "box_response_required_downstream_custody_names": ["responses_effects_after_checkpoint_without_checkpoint_and_between_calls_do_not_replay","responses_checkpoint_io_failure_expiry_and_new_effect_cannot_accept_stale_coverage","responses_pre_effect_retry_and_safe_checkpoint_retry_keep_model_and_completed_tool","downstream_unknown_outcomes_block_responses_checkpoint_and_replay_after_local_return","definitive_relay_results_remain_checkpoint_eligible_and_preserve_returned_data","prepublication_refusal_and_list_failure_add_no_unknown_effect_custody","timed_out_tool_keeps_worker_receipt_and_late_result_stays_unknown_across_attempts","a_lost_tool_worker_cannot_turn_error_json_into_an_accepted_checkpoint","user_cancellation_between_tools_and_during_checkpoint_io_wins_without_retry","real_mcp_endpoint_keeps_immutable_attempt_admission_and_counts_effects","real_mcp_response_loss_refuses_more_effects_on_the_same_attempt","real_mcp_effect_prevents_provider_retry_after_attempt_expiry","production_fabushi_executor_refuses_replay_after_tool_expiry","production_generated_runner_preserves_terminal_tool_custody"],
    "fresh_bearer_admission_contract": "OCRM-DEVICE-FRESH-AUTHORIZATION-v1",
    "fresh_bearer_admission_scope": "Actual existing pairing owner refuses a fresh proof when the approved role or whole canonical live grant is no longer current, preserves fixed decisive-storage errors and exact durable state, and admits repaired valid state after reopen. Retained proof checks keep their existing issuer and late-admission semantics. Actual HTTP config.set through the App Settings helper and shipping source bindings are separately required in the App receipt; these owner controls do not establish full App construction, remote handshake, last-use writes, issuer-generation or physical-device acceptance.",
    "media_settlement_support_scope": "MSU-v1 support only: actual Tool construction-error classification and the independently bounded media COMMIT adapter/consumer proof, composed with all retained channel/admission/raw-COMMIT controls. Real file-backed media DB faults and local cleanup states are required by the separate native-media receipt; eight private App/Session/Runner cases and ordinary shipping feature-off compilation are required by the separate Linux App receipt. No runtime or full acceptance is inferred from source-oracle positives.",
    "contract": "CWR-v1-portable/MGS-v1/MGA-v1/RSU-v1/MWR-v1/PMS-v1-Rust-producer", "subject_commit": head,
    "additional_contracts": ["SCP-CLAIM-v1", "DPR-v1", "PRC-v1-registry", "DPA-v1", "DVR-v1", "DRT-v1", "DTR-v1", "DJC-v1", "DJL-v1", "CFL-CHECKED-v1-source-wiring", "MCP-RECEIVED-CATALOG-v1", "OCRM-CHANNEL-TRANSPORT-CHUNK-v1", "MEA-v1", "MEC-v1", "MER-v1", "CGWA-v1", "CCWA-v1", "OCRM-PLUGIN-UNINSTALL-RECOVERY-v1", "OCRM-PROVIDER-CHECKPOINT-WRITE-v1", "CMI-v1", "SNI-v1", "OCRM-GENERATED-HISTORY-v1", "OCRM-CANONICAL-ROOT-ADMISSION-v1", "CWB-v1", "OCRM-SEND-MESSAGE-FIDELITY-v1", "MSI-v1", "OCRM-CANONICAL-PRIOR-ROOT-IDENTITY-v1"],
    "memory_synthesis_identity_scope": "Per-snapshot MemorySynthesis uses one admitted model/credential identity across proposal, verification and retries. Native and Codex controls reach the production factory, actual HTTP assembly, Session apply gate and FileMemoryStore with owned credentials. Same-context token rotation is accepted; account/workspace drift and cancellation preserve memory/temporal bytes. Stale starts a new snapshot admission. Detached-worker drain, atomic cancellation/commit, original evidence account binding, durable synthesis recovery and real providers remain unaccepted.",
    "send_message_fidelity_scope": "SGF-v1 preserves actual completed SendMessage producer wire through existing canonical field 2 independently of live observation, using a fresh invocation recorder and checked receipt/callback/drain custody. Actual Session success-only decoding retains full-envelope and effective oneof semantics through backfill/reopen. Controlled actual App SQL sends compare message identity, normalized arguments, timestamps, live/canonical bytes and Journal projection; original typed failure, completion revocation and two fresh owner reads retain no replay or marker authority. Real providers/channels, external delivery ACK, lost SQL identity reconstruction, original interrupted-turn restart, rich variants and packaged-platform acceptance remain open.",
    "codex_workspace_binding_scope": "Actual private credential loading, per-turn source/workspace continuity, same-workspace rotation, recreated retry/checkpoint refusal, one-401 refresh before persistence and bound/fresh Memory consumers through real HTTP with owned redirected destinations. Real service, external-writer settlement and process-restart acceptance remain open.",
    "generated_history_scope": "Actual generated Runner retains ordered known successful tool observations and the accepted user through the existing canonical root/Journal transaction after provider failure or cancellation. Owned fresh processes observe history without provider/tool replay or completion/marker authority. Real provider-store refusal retains unaccepted effect custody; unknown/no-effect/root-drift controls, original typed error preservation, prior-completion invalidation and ordinary success remain required. Controlled Responses do not prove real provider compatibility, unknown/pending history, complete typed replay/UI, original-turn restart or packaged-platform acceptance.",
    "canonical_root_admission_scope": "Actual checked optional canonical snapshots bind Host composition, Runner projection, preflight, staging and guarded publication. Required process controls distinguish absent/present-empty and legacy-root identities, strict read/retirement refusals, same-sink successor drift, early-error permanent completion revocation, and post-durable Journal failure with fresh-owner recovery. Controlled Responses are not real provider compatibility. No lock spans provider execution; MCP catalog preparation precedes admission; recursive graph validation, prior-root wire provenance, original-turn restart, complete typed replay and package acceptance remain open.",
    "plugin_uninstall_recovery_scope": "Durable pre-dispatch tickets and exact settlements in the existing MCP/Skills owner, pending and confirmed refusal in actual Session/prompt consumers, strict image/marker recovery, cache-before-release barriers, and same-root Host handoff. Owned producer process exits and fresh consumer processes use controlled backend I/O. Pending outcomes are never replayed or cleared by ordinary sync. Account/backend incarnation, remote receipt/quiescence reconciliation, real ecosystem, hardware power-loss and packaged-platform acceptance remain open.",
    "checkpoint_publication_scope": "Actual provider checkpoint writer: private owned staging, exact payload preservation, no-clobber final publication, directory-entry barriers including retry after a failed creation barrier, and retained visible payload after a controlled final barrier failure. Actual shell adapter retains a completed file effect as uncheckpointed after real store refusal and explicit repair. No original-turn restart eligibility, claim or execution, hardware power-loss or packaged-platform acceptance.",
    "session_namespace_scope": "SNI-v1 runs the complete existing path, Agent lifecycle and Agent Session targets. Real Session/Human stores cover reserved Agent path refusal before locks, markers and caches; whole-batch lifecycle and Gateway preflight before all hooks; distinct typed Agent/Human cache owners in both opening orders; Agent close/retire/delete, unchanged Human bytes and reopen; and retained valid deletion/admission controls. Colon-bearing filesystem controls run on Unix. This is not every filesystem alias, external parent replacement, Windows or packaged acceptance, Human attachment admission, membership revocation or a new permission owner.",
    "live_custody_contract": "OCRM-CHANNEL-FAILURE-LIVE-CUSTODY-v1",
    "subject_tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], text=True).strip(),
    "upstream": {key: lock["upstream"][key] for key in ("repository", "commit", "tree")},
    "run_id": os.environ["GITHUB_RUN_ID"], "run_attempt": os.environ["GITHUB_RUN_ATTEMPT"],
    "job": os.environ["GITHUB_JOB"], "runner_os": os.environ["RUNNER_OS"],
    "rustc": subprocess.check_output(["rustc", "--version", "--verbose"], text=True),
    "inputs_sha256": {path: hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest() for path in inputs},
    "synthetic_result_oracle_controls": oracle_controls,
    "planned_commands": [{"case": label, "command": command, "required_positive_names": required} for label, command, required in cases],
    "executions": [], "result": "failed-or-incomplete",
    "scope": "MER-v1 requires an acknowledged episode record before summary work; real archived Session refusal retains pending and memory bytes across reopen, and owner restore admits ordinary recovery. Extraction remains before admission; this control is not a Host/UI archive race or a later lifecycle fence. MEC-v1 retains actual Session pending input and memory bytes on controlled summary execution errors, including cancellation results, across reopen and later ordinary-turn recovery; successful empty/whitespace/NONE replies preserve existing durable settlement. Existing bound-memory consumers require account-refusal input retention and successful same-context token rotation with fixture-owned HTTP and credentials. These controlled model results do not establish live provider or cancellation timing acceptance. Linux actual store, rearm, manager, checked-lock and BackgroundWakes live-custody controls, plus paired-device SSE permission. DJL-v1 adds strict direct-socket public join admission before optional authentication, FIFO reservation/settlement, bounded generation-owned state, real HTTP callback/error/restart/lost-response controls and retained public join/TLS regressions; trusted-proxy/configurable policy and real devices remain open. DJC-v1 executes file-backed join read/write/controlled commit refusal, durable malformed-row consumption, post-commit expiry, two-connection single-use, reopen/repair and actual Gateway HTTP refusal/recovery controls; complete setup schema/issuer UI and unknown COMMIT/crash acceptance remain open. MGS-v1 covers actual Memory dispatch, inside/outside stores, unchanged bytes/notifications, valid-after-refusal and projection/order/bound/mutation controls. RSU-v1 covers actual Session gateway persisted-transcript Unicode fallback/reopen, character/byte windows, ordering/limits and ready-index controls. MWR-v1 covers Store append read admission and actual production synthesis target/Session gate/service Failed settlement, temporal-byte preservation, Invalid/Stale controls and repair; controlled proposals do not establish real provider compatibility. MEA-v1 exercises the completed-turn consumer through actual ProductionEpisodeProgress: read/write append refusal preserves pending input across reopen and interleaved admission; repair and duplicate acknowledgements retain normal settlement. The existing Automation command requires the composed Runner Memory consumer. MGA-v1 covers actual HTTP with real issued/revoked pairing credentials, before-owner permission refusals, native/header compatibility, no ambient fallback for invalid credentials, unchanged denied state and admitted mutation/reopen. CGWA-v1 requires the same controls for native channel credential writes through real Session secret/metadata owners, preserving unrelated Agent/platform bytes and acknowledged reopen state. Real provider registration, two-file crash atomicity and complete channel lifecycle remain open. CCWA-v1 requires checked credential write admission and raw-byte preservation through real Store, Session, Gateway, Widget and Manager consumers; refusal emits a redacted real tray without acknowledgement or resume, and explicit repair reuses the same request. CMI-v1 requires the saved Codex model in actual serialized requests through production Runner retries, owned checkpoints, nudges, turn memory and summary, with raw/persisted refusal and cancellation before effects; only the final transport is controlled and real account/service and process-loss recovery remain open. PMS-v1 requires persisted/reopened actual Session search replies and exports only the asserted returned JSON for the same-H desktop consumer. Session claim evidence covers exact private settlement, real inbound post-COMMIT listener replacement, duplicate/refusal/compatibility controls and actual unknown-COMMIT/refused-rollback custody; final-write claim admission and durable correlation recovery remain open. DPR-v1 covers the existing file-backed pairing owner: all three mutation transactions refuse corrupt/exhausted revision state while retaining target rows, prior tokens and surfaces, with repeat/reopen/known-counter repair, absent/legacy decimal and final-increment controls. It retains all existing pairing contract controls without claiming HTTP/device compatibility or unknown-commit recovery. DVR-v1 retains complete owner26, HTTP8 and its original14 retained-stream cases for bound atomic whole-row removal, caller policy and redaction, exact proof retirement on every existing stream wake, real bridge provider cleanup and response admission, real quiet-heartbeat and native/MCP controls. DRT-v1 adds selected-role retirement through the same owner, explicit native/paired ceilings, fresh-marker rejection and six selected-role retained cases; whole-target counts and exact positive names remain mandatory. Original source-review/authority and actual App helper results remain separate; physical teardown, APNs and the upstream remove wire alias remain open. DTR-v1 executes the complete wire-rotation owner44 and HTTP14 targets plus all27 retained cases, preserving all previous names and assertions. It covers resulting-grant caps and explicit repair, exact identity-bound delivery, supported metadata, checked single-commit revision, current-proof races, node-only surfaces, response-loss/no-replay and successor controls. The large distinct-scope read-only HTTP request reaches the bound owner, is refused without durable mutation and preserves ordinary authorized rotation after reopen; shared canonicalization uses order-preserving set membership. Freshly issued literal scopes stay literal across adjacent management consumers; selected primary-only role materialization, decisive approval reads, rollback, reopen and later role withdrawal are required. External credential caches, issuer-generation policy, upstream WebSocket/node-surface compatibility and real devices remain open. DPA-v1 adds actual file-backed owner and real HTTP delegate controls for sealed current credential admission, target and caller scope ceilings, identity-derived token delivery, invalid-bearer refusal, stale credential races and exactly one concurrent self replacement; the shipping App override is compiled/source-reviewed, not a full-App transport or physical-device acceptance. MCP controls use actual preparation/projection/toolset, sequential HTTP refresh, owned Unix CLI attempts and retry custody; channel controls use the real splitter/Host/router with a controlled transport, including ambiguous effect refusal without replay. These do not establish real provider compatibility. Cloud-agent checks and BackgroundWakes wiring mutations remain source-wiring supplements. Controlled runtime-port outcomes do not execute the shipping app/provider/quiescence branches. Arbitrary write transactions, external filesystem races, symlink, cross-account, full HTTP/UI, macOS/Windows, indexed-backend/provider equivalence, unknown-effect and package acceptance remain open.",
}
# Bind the selected scope before the first durable partial receipt, including
# compile failures; unselected requirement metadata is not execution evidence.
if validation_scope == "custody-recovery":
    receipt["scope"] = ("CRF-v1: selected Box producer/consumer/state controls, repaired actual Plugin "
        "desktop/library custody oracles and DJC-07 qualified COMMIT refusal only. Historical DJC lock "
        "timing remains unresolved; unrelated full-plan cases, real ecosystems, packages and full OCRM "
        "acceptance are not inferred. Requirement metadata elsewhere in this receipt is not execution.")
if validation_scope == "background-custody":
    receipt["scope"] = ("BGC-v1: existing Box background resource custody, actual Host/ForeverBox/Runner "
        "uncertainty and terminal-watch completion, canonical footer parser and retained Box/foreground "
        "regressions only. Unselected full-plan cases, normal active streamed shutdown, complete upstream "
        "termination equivalence, real ecosystems, durable restart and packages remain open. "
        "Requirement metadata elsewhere in this receipt is not execution.")

if validation_scope == "background-custody-recovery":
    receipt["scope"] = ("BGC-v1 focused recovery: repaired handoff-ID-exhaustion cleanup oracle "
        "and actual retained-reader EOF/admission/autonomous completion controls only. "
        "The prior H6d380 failed subject is retained; this result does not rerun or accept "
        "the remaining BGC/full-plan groups, real ecosystems, durable restart or packages. "
        "Requirement metadata elsewhere in this receipt is not execution.")

# RoutedToolDefinition is shared by built-ins and integration fixtures. Compile
# the complete Host test graph once; this is a compile gate, not test evidence.
constructor_command = ["cargo", "check", "--locked", "--manifest-path", "source/host/Cargo.toml", "--tests"]
constructor_log = root / "mcp-origin-constructor-check.log"
constructor_check = {"command": constructor_command, "cargo_exit": None, "capture_exit": 0,
    "log": constructor_log.name, "result": "failed-or-incomplete", "execution_error": None}
receipt["mcp_origin_constructor_compile"] = constructor_check
(root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
constructor_raw = b""
try:
    with constructor_log.open("wb") as output:
        constructor_check["cargo_exit"] = subprocess.run(constructor_command,
            stdout=output, stderr=subprocess.STDOUT, check=False).returncode
    constructor_raw = constructor_log.read_bytes()
except OSError as error:
    constructor_check["capture_exit"] = 1
    constructor_check["execution_error"] = str(error)
constructor_check["log_sha256"] = hashlib.sha256(constructor_raw).hexdigest()
constructor_passed = constructor_check["cargo_exit"] == 0 and constructor_check["capture_exit"] == 0
constructor_check["result"] = "compiled" if constructor_passed else "failed-or-incomplete"
(root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(constructor_raw.decode("utf-8", errors="replace"), flush=True)
print(json.dumps({"mcp_origin_constructor_compile": constructor_check}), flush=True)
if not constructor_passed:
    raise SystemExit(constructor_check["cargo_exit"] or constructor_check["capture_exit"] or 1)

# The existing producer has no tracked lock at the initial BXR base. Preserve
# its generated resolution as an output, never describe it as an H-bound input,
# and use --locked automatically after that exact output has been committed.
daemon_command = ["cargo", "build", "--manifest-path", "source/box-exec-daemon/Cargo.toml",
    "--bin", "box-exec-daemon", "--target-dir", "source/box-exec-daemon/target"]
if box_daemon_lock_tracked:
    daemon_command.insert(2, "--locked")
daemon_log = root / "box-response-daemon-build.log"
daemon_binary = pathlib.Path("source/box-exec-daemon/target/debug/box-exec-daemon").resolve()
daemon_build = {"command": daemon_command, "cargo_exit": None, "capture_exit": 0,
    "log": daemon_log.name, "execution_error": None, "result": "failed-or-incomplete",
    "lock_was_exact_h_source_input": box_daemon_lock_tracked,
    "binary": str(daemon_binary), "binary_sha256_before_cases": None}
receipt["box_response_daemon_build"] = daemon_build
(root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
daemon_raw = b""
try:
    with daemon_log.open("wb") as output:
        daemon_build["cargo_exit"] = subprocess.run(daemon_command,
            stdout=output, stderr=subprocess.STDOUT, check=False).returncode
    daemon_raw = daemon_log.read_bytes()
except OSError as error:
    daemon_build["capture_exit"] = 1
    daemon_build["execution_error"] = str(error)
daemon_build["log_sha256"] = hashlib.sha256(daemon_raw).hexdigest()
daemon_passed = (daemon_build["cargo_exit"] == 0 and daemon_build["capture_exit"] == 0
    and box_daemon_lock.is_file() and daemon_binary.is_file())
if box_daemon_lock.is_file():
    resolved_lock = box_daemon_lock.read_bytes()
    (root / "box-response-daemon-resolved.Cargo.lock").write_bytes(resolved_lock)
    daemon_build["resolved_lock"] = {
        "path": "box-response-daemon-resolved.Cargo.lock",
        "sha256": hashlib.sha256(resolved_lock).hexdigest(), "bytes": len(resolved_lock),
        "utf8": resolved_lock.decode("utf-8"),
        "role": "exact-H source input" if box_daemon_lock_tracked else "build-generated output; dependency closure not yet source-frozen",
    }
if daemon_passed:
    daemon_bytes = daemon_binary.read_bytes()
    daemon_build["binary_sha256_before_cases"] = hashlib.sha256(daemon_bytes).hexdigest()
    daemon_build["binary_bytes"] = len(daemon_bytes)
daemon_build["result"] = "compiled-and-bound" if daemon_passed else "failed-or-incomplete"
(root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(daemon_raw.decode("utf-8", errors="replace"), flush=True)
print(json.dumps({"box_response_daemon_build": daemon_build}), flush=True)


# SC uses a separately identified cfg(test) library executable solely for the
# real operation-fault child. The ordinary shipping daemon above remains bound
# and exercised independently. A stale or unrelated test executable is refused.
capture_manifest = pathlib.Path("source/box-exec-daemon/Cargo.toml").resolve()
capture_source = pathlib.Path("source/box-exec-daemon/src/lib.rs").resolve()
capture_target = pathlib.Path("source/box-exec-daemon/target").resolve()

def select_shell_capture_test_artifact(raw):
    artifacts, finished = [], []
    for line in raw.decode("utf-8").splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(message, dict):
            continue
        if message.get("reason") == "build-finished":
            finished.append(message.get("success"))
        if message.get("reason") != "compiler-artifact":
            continue
        target, profile = message.get("target", {}), message.get("profile", {})
        executable = message.get("executable")
        if (not isinstance(target, dict) or not isinstance(profile, dict)
            or profile.get("test") is not True
            or target.get("name") != "fabushi_box_exec_daemon"
            or target.get("kind") != ["lib"]
            or not isinstance(target.get("src_path"), str)
            or pathlib.Path(target["src_path"]).resolve() != capture_source
            or not isinstance(executable, str) or not executable):
            continue
        path = pathlib.Path(executable).resolve()
        if not path.is_relative_to(capture_target):
            raise ValueError("daemon library test artifact is outside its declared target directory")
        artifacts.append((path, message))
    if finished != [True] or len(artifacts) != 1:
        raise ValueError("current Cargo build lacks one unambiguous successful daemon library test artifact")
    return artifacts[0]

capture_artifact_sample = {
    "reason": "compiler-artifact", "package_id": "owned-oracle-only",
    "target": {"name": "fabushi_box_exec_daemon", "kind": ["lib"], "src_path": str(capture_source)},
    "profile": {"test": True}, "executable": str(capture_target / "debug/deps/owned-oracle-only"),
}
capture_artifact_finish = {"reason": "build-finished", "success": True}
def capture_artifact_variant(field, value):
    changed = json.loads(json.dumps(capture_artifact_sample))
    if "." in field:
        parent, child = field.split(".", 1)
        changed[parent][child] = value
    else:
        changed[field] = value
    return changed
capture_artifact_controls = []
for control, messages, expected in [
    ("current-library-test", [capture_artifact_sample, capture_artifact_finish], True),
    ("missing-artifact", [capture_artifact_finish], False),
    ("duplicate-artifact", [capture_artifact_sample, capture_artifact_sample, capture_artifact_finish], False),
    ("foreign-source", [capture_artifact_variant("target.src_path", str(capture_source.parent / "other.rs")), capture_artifact_finish], False),
    ("foreign-target", [capture_artifact_variant("target.name", "other"), capture_artifact_finish], False),
    ("ordinary-library", [capture_artifact_variant("profile.test", False), capture_artifact_finish], False),
    ("binary-test", [capture_artifact_variant("target.kind", ["bin"]), capture_artifact_finish], False),
    ("missing-executable", [capture_artifact_variant("executable", None), capture_artifact_finish], False),
    ("foreign-executable", [capture_artifact_variant("executable", str(capture_target.parent / "foreign")), capture_artifact_finish], False),
    ("missing-build-terminal", [capture_artifact_sample], False),
    ("failed-build-terminal", [capture_artifact_sample, {"reason":"build-finished","success":False}], False),
    ("duplicate-build-terminal", [capture_artifact_sample, capture_artifact_finish, capture_artifact_finish], False),
]:
    try:
        select_shell_capture_test_artifact(("\n".join(json.dumps(item) for item in messages) + "\n").encode("utf-8"))
        actual = True
    except (ValueError, OSError):
        actual = False
    capture_artifact_controls.append({"case": control, "expected": expected, "actual": actual})
    assert actual == expected, f"SC artifact selection oracle failed: {control}"

capture_command = ["cargo", "test", "--locked", "--manifest-path", str(capture_manifest),
    "--lib", "--no-run", "--message-format=json", "--target-dir", str(capture_target)]
capture_log = root / "box-response-shell-capture-build.log"
capture_build = {
    "command": capture_command, "cargo_exit": None, "capture_exit": 0,
    "log": capture_log.name, "execution_error": None, "result": "failed-or-incomplete",
    "role": "cfg(test) operation-fault producer; separate from the ordinary daemon binary",
    "lock_was_exact_h_source_input": box_daemon_lock_tracked,
    "binary": None, "binary_sha256_before_cases": None,
    "artifact_selection_oracle_controls": capture_artifact_controls,
}
receipt["box_shell_capture_test_build"] = capture_build
(root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
capture_raw, capture_binary, capture_passed = b"", None, False
try:
    if not box_daemon_lock_tracked:
        raise ValueError("SC producer requires a tracked exact-H Cargo.lock")
    capture_build["lock_sha256_before_build"] = hashlib.sha256(box_daemon_lock.read_bytes()).hexdigest()
    with capture_log.open("wb") as output:
        capture_build["cargo_exit"] = subprocess.run(capture_command,
            stdout=output, stderr=subprocess.STDOUT, check=False).returncode
    capture_raw = capture_log.read_bytes()
    capture_build["lock_sha256_after_build"] = hashlib.sha256(box_daemon_lock.read_bytes()).hexdigest()
    if capture_build["cargo_exit"] != 0:
        raise ValueError("SC library test build failed")
    if capture_build["lock_sha256_after_build"] != capture_build["lock_sha256_before_build"]:
        raise ValueError("SC library test build changed its frozen Cargo.lock")
    capture_binary, artifact = select_shell_capture_test_artifact(capture_raw)
    if not capture_binary.is_file() or not os.access(capture_binary, os.X_OK):
        raise ValueError("SC library test artifact is missing or not executable")
    capture_bytes = capture_binary.read_bytes()
    capture_build.update({
        "binary": str(capture_binary), "binary_bytes": len(capture_bytes),
        "binary_sha256_before_cases": hashlib.sha256(capture_bytes).hexdigest(),
        "compiler_artifact": artifact, "result": "compiled-and-bound",
    })
    capture_passed = True
except (OSError, ValueError) as error:
    capture_build["capture_exit"] = 1
    capture_build["execution_error"] = str(error)
capture_build["log_sha256"] = hashlib.sha256(capture_raw).hexdigest()
(root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(capture_raw.decode("utf-8", errors="replace"), flush=True)
print(json.dumps({"box_shell_capture_test_build": capture_build}), flush=True)

for label, command, required in cases:
    receipt["active_case"] = label
    (root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    log_path = root / (label + ".log")
    cargo_exit, capture_exit, execution_error = None, 0, None
    raw = b""
    child_environment_overrides = {}
    if label.startswith("box-response-"):
        child_environment_overrides["SAND_DISABLE_LARGE_OUTPUT_SPILL"] = "0"
    if label == "box-response-real-daemon" and daemon_passed:
        child_environment_overrides["FABUSHI_BOX_EXEC_DAEMON_TEST_BIN"] = str(daemon_binary)
    if label in ("box-response-shell-capture-consumer", "box-response-background-custody-consumer") and capture_passed:
        child_environment_overrides["FABUSHI_BOX_EXEC_DAEMON_CAPTURE_TEST_BIN"] = str(capture_binary)
    child_environment = dict(os.environ, **child_environment_overrides) if child_environment_overrides else None
    try:
        if label == "box-response-real-daemon" and not daemon_passed:
            capture_exit = 1
            execution_error = "BXR real daemon control blocked: the current build did not produce a bound executable"
            raw = (execution_error + "\n").encode("utf-8")
            log_path.write_bytes(raw)
        elif (label in ("box-response-shell-capture-unit", "box-response-shell-capture-consumer", "box-response-shell-capture-disconnect", "box-response-background-custody-unit", "box-response-background-custody-consumer", "box-response-background-custody-retained") or label in background_recovery_labels) and not capture_passed:
            capture_exit = 1
            execution_error = "SC control blocked: the current library test build did not produce a bound executable"
            raw = (execution_error + "\n").encode("utf-8")
            log_path.write_bytes(raw)
        else:
            with log_path.open("wb") as output:
                completed = subprocess.run(command, stdout=output, stderr=subprocess.STDOUT,
                    check=False, env=child_environment)
            cargo_exit = completed.returncode
            raw = log_path.read_bytes()
    except OSError as error:
        capture_exit, execution_error = 1, str(error)
    text = raw.decode("utf-8", errors="replace")
    print(text, flush=True)
    raw_passed = successful_test_names(text)
    passed = box_response_observed_names(text) if label.startswith("box-response-") else raw_passed
    missing = [name for name in required if passed.count(name) != 1]
    counts = successful_test_summaries(text)
    accepted = cargo_exit == 0 and capture_exit == 0 and not missing and bool(counts) and all(count[0] > 0 and count[1] == 0 for count in counts)
    if label in ("device-pairing", "device-pairing-join", "device-pairing-rotation-authority", "device-pairing-rotation-gateway", "device-join-limit-state", "device-join-ingress-http", "device-join-legacy-http", "device-join-tls", "memory-account-binding", "codex-model-identity", "generated-failure-history", "canonical-root-admission", "codex-workspace-binding", "codex-workspace-http", "memory-synthesis-identity", "generated-send-message-recorder", "canonical-prior-root-image", "canonical-prior-root-process"):
        accepted = accepted and counts == [(len(required), 0, 0)]
    complete_counts = None
    if label in ("device-pairing-revocation-authority", "device-pairing-revocation-gateway", "device-role-token-revocation-authority", "device-role-token-revocation-gateway", "device-token-rotation-wire-authority", "device-token-rotation-wire-gateway", "device-pairing-retained-stream"):
        complete_counts = [tuple(map(int, count)) for count in re.findall(
            r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;", text)]
        accepted = accepted and counts == [(len(required), 0, 0)] and complete_counts == [(len(required), 0, 0, 0, 0)]
    if label == "generated-send-message-contracts":
        accepted = (accepted and sorted(counts) == [(13, 0, 0), (14, 0, 0), (19, 0, 0)]
            and len(passed) == len(required))
    if label == "canonical-root-admission-contracts":
        accepted = accepted and canonical_contract_results_accepted(cargo_exit, capture_exit, required, text)
    if label == "session-owner-namespace":
        accepted = (accepted and len(counts) == 3
            and all(count[2] == 0 for count in counts)
            and sum(count[0] for count in counts) == len(passed))
    if label == "mcp-origin-canonical-boundary":
        accepted = (accepted and len(counts) == 1 and counts[0][2] == 0
            and counts[0][0] == len(passed) and len(passed) >= len(required))
    if validation_scope == "custody-recovery" and not label.startswith("box-response-"):
        accepted = exact_target_results_accepted(cargo_exit, capture_exit, required, text, True)
    if label.startswith("box-response-"):
        accepted = box_response_results_accepted(label, cargo_exit, capture_exit, required, text)
        complete_counts = [tuple(map(int, count)) for count in re.findall(
            r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out; finished in [0-9.]+s[ \t]*$",
            text.replace("\r\n", "\n"), re.MULTILINE)]
    receipt["executions"].append({
        "case": label, "command": command, "cargo_exit": cargo_exit, "capture_exit": capture_exit,
        "child_environment_overrides": child_environment_overrides,
        "observed_raw_passed_test_names": raw_passed,
        "execution_error": execution_error, "required_positive_names": required,
        "missing_or_nonunique_positive_results": missing, "observed_passed_test_names": passed,
        "summaries_passed_failed_ignored": counts, "complete_summaries": complete_counts, "oracle_accepted": accepted,
        "log": log_path.name, "log_sha256": hashlib.sha256(raw).hexdigest(),
    })
    (root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
receipt.pop("active_case", None)
palette_exported = palette_fixture.is_file()
palette_required = (validation_scope == "full"
    or any("palette_message_gateway_exports_persisted_rust_search_results" in required
        for _, _, required in cases))
receipt["palette_export"] = {"path": str(palette_fixture), "required_by_selection": palette_required,
    "sha256": hashlib.sha256(palette_fixture.read_bytes()).hexdigest() if palette_exported else None,
    "produced_by": "palette_message_gateway_exports_persisted_rust_search_results" if palette_exported else None}
custody_recovery_inputs_after = source_bindings(custody_recovery_inputs)
custody_recovery_sources_unchanged = (custody_recovery_inputs_after == custody_recovery_inputs_before
    and all(item["matches_head_bytes"] for item in custody_recovery_inputs_after.values()))
receipt["custody_recovery_inputs_after"] = custody_recovery_inputs_after
receipt["custody_recovery_sources_unchanged"] = custody_recovery_sources_unchanged
plugin_admission_lock_inputs_after = source_bindings(plugin_admission_lock_inputs)
receipt["plugin_admission_lock_inputs_after"] = plugin_admission_lock_inputs_after
plugin_admission_lock_sources_unchanged = (plugin_admission_lock_inputs_after == plugin_admission_lock_inputs_before
    and all(item["matches_head_bytes"] for item in plugin_admission_lock_inputs_after.values()))
receipt["plugin_admission_lock_sources_unchanged"] = plugin_admission_lock_sources_unchanged
local_exec_outcome_inputs_after = source_bindings(local_exec_outcome_inputs)
receipt["local_exec_outcome_inputs_after"] = local_exec_outcome_inputs_after
local_exec_outcome_sources_unchanged = (local_exec_outcome_inputs_after == local_exec_outcome_inputs_before
    and all(item["matches_head_bytes"] for item in local_exec_outcome_inputs_after.values()))
receipt["local_exec_outcome_sources_unchanged"] = local_exec_outcome_sources_unchanged
box_response_inputs_after = source_bindings(box_response_inputs)
receipt["box_response_inputs_after"] = box_response_inputs_after
box_response_sources_unchanged = (box_response_inputs_after == box_response_inputs_before
    and all(item["matches_head_bytes"] for item in box_response_inputs_after.values()))
receipt["box_response_sources_unchanged"] = box_response_sources_unchanged
daemon_build["binary_sha256_after_cases"] = (
    hashlib.sha256(daemon_binary.read_bytes()).hexdigest()
    if daemon_passed and daemon_binary.is_file() else None)
daemon_build["lock_sha256_after_cases"] = (
    hashlib.sha256(box_daemon_lock.read_bytes()).hexdigest() if box_daemon_lock.is_file() else None)
daemon_unchanged = (daemon_passed
    and daemon_build["binary_sha256_after_cases"] == daemon_build["binary_sha256_before_cases"]
    and daemon_build["lock_sha256_after_cases"] == daemon_build["resolved_lock"]["sha256"])
receipt["box_response_producer_unchanged"] = daemon_unchanged
capture_build["binary_sha256_after_cases"] = (
    hashlib.sha256(capture_binary.read_bytes()).hexdigest()
    if capture_passed and capture_binary is not None and capture_binary.is_file() else None)
capture_build["lock_sha256_after_cases"] = (
    hashlib.sha256(box_daemon_lock.read_bytes()).hexdigest() if box_daemon_lock.is_file() else None)
capture_unchanged = (capture_passed
    and capture_build["binary_sha256_after_cases"] == capture_build["binary_sha256_before_cases"]
    and capture_build["lock_sha256_after_cases"] == capture_build["lock_sha256_before_build"])
receipt["box_shell_capture_test_producer_unchanged"] = capture_unchanged
accepted = ((palette_exported if palette_required else True) and len(receipt["executions"]) == len(cases)
    and all(case["oracle_accepted"] for case in receipt["executions"])
    and box_response_sources_unchanged and daemon_unchanged and capture_unchanged and local_exec_outcome_sources_unchanged
    and plugin_admission_lock_sources_unchanged and custody_recovery_sources_unchanged)
receipt["result"] = ("passed-scoped-only" if validation_scope != "full" else "passed") if accepted else "failed-or-incomplete"
(root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps(receipt, indent=2), flush=True)
raise SystemExit(0 if accepted else 1)
