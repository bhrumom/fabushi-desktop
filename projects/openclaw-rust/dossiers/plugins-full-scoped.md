# Plugins Full Lifecycle, Installed Index Store, Manifest Doctor & Hook Runner Dossier

## 1. Scope and Responsibilities
- **Scope**: Native Rust host runtime extensions and contract tests for plugin installed index store, manifest doctor, and lifecycle hook runners. Bulk shard coverage claims for `src/plugins/` have been revoked and downgraded to unread/unclassified pending individual per-file reading per Spec §6.2-6.3.
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Mapped Responsibilities**:
  1. `OCRM-RESP-PLUGINS-INSTALL-MANAGEMENT-001`: Installed Plugin Index Store, Schema V1, Package Management & Enablement.
  2. `OCRM-RESP-PLUGINS-MANIFEST-DOCTOR-001`: Plugin Doctor Diagnostics, Manifest Health, Deprecated Config Linting.
  3. `OCRM-RESP-PLUGINS-HOOKS-PIPELINE-001`: Lifecycle Hook Runner with Priority Ordering, Timeout Boundaries & Fail-Soft Recovery.
  4. `OCRM-RESP-PLUGINS-PROVIDER-EXTENSIONS-001`: Provider Extension Bridge for Dynamic Speech, LLM, and Image Adapters.

## 2. Producer -> Owner -> Storage -> Consumer Traces
1. **Installed Plugin Index Pipeline**:
   - `Producer`: Plugin installation (ClawHub, npm, local) and registry discovery commands.
   - `Owner`: `source/host/src/extensions/plugins/installed_index_store.rs` (`InstalledPluginIndexStore`).
   - `Storage`: State database index record cache with signature and warning header.
   - `Consumer`: Plugin discovery runner and desktop UI installed plugins view.
2. **Manifest Doctor Diagnostics Pipeline**:
   - `Producer`: Plugin loading errors, developer diagnostic CLI commands, and host startup checks.
   - `Owner`: `source/host/src/extensions/plugins/manifest_doctor.rs` (`PluginDoctorReport`, `check_plugin_manifest_health`).
   - `Storage`: Ephemeral diagnostic error list.
   - `Consumer`: Plugin doctor report viewer and issue remediator.
3. **Hook Runner & Provider Extension Pipeline**:
   - `Producer`: Agent turn execution lifecycle events (beforeReply, afterInit, beforeCommand).
   - `Owner`: `source/host/src/extensions/plugins/hook_runner.rs` (`PluginHookRunner`, `ProviderExtensionBridge`).
   - `Storage`: Process-local sorted hook priority list and provider map.
   - `Consumer`: Agent turn orchestrator and chat output pipeline.

## 3. Verification & Oracle Evidence
- **Contract Tests**: `source/host/tests/openclaw_plugins_full_contract.rs`:
  - `test_ocrm_plugins_installed_index_store` (PASSED)
  - `test_ocrm_plugins_manifest_doctor` (PASSED)
  - `test_ocrm_plugins_hook_runner_and_provider_bridge` (PASSED)
  - `test_ocrm_plugins_boundary_invariants` (PASSED)
  - `test_ocrm_plugins_agent_event_payload_and_active_run_steering` (PASSED)
  - `test_ocrm_plugins_architectural_boundary_guardrails` (PASSED)
    - Verifies bundled extensions (`telegram`, `discord`, `slack`, etc.) use owned `channel-config-schema` or `../config-api.js` and never `bundled-channel-config-schema`.
    - Verifies core plugin-sdk contains no Ollama facade modules and generic core fixtures contain no vendor provider terms (`Ollama`, `Moonshot`, `xAI`).
    - Verifies production code contains no deprecated config load/write internal seams.
    - Verifies model catalog imports use `@openclaw/model-catalog-core/...` package subpaths instead of legacy internal bridge modules.
  - `test_ocrm_plugins_host_hook_fixture_and_ui_projection` (PASSED)
    - Verifies host hook fixture double registration (session extension, tools, control UI, lifecycle, events, scheduler).
    - Verifies UI projection structure, generation bump, and required operator scopes on admin panel.
  - `test_ocrm_plugins_runtime_import_side_effects_and_seams` (PASSED)
    - Verifies hot runtime and config modules keep channel registry cold during import.
    - Verifies on-demand markdown table mode resolution.
    - Verifies bundled plugin public surface access requires enabled state in config.
    - Verifies SSRF guard blocks requests to private IPv4 networks.
  - `test_ocrm_plugins_config_boundary_and_test_api_consumption` (PASSED)
    - Verifies channel action runtime helpers do not invoke `loadConfig()` directly and instead receive configuration context.
    - Verifies extension `test-api.ts` can only be imported from test files (`.test.ts`, `.spec.ts`), strictly prohibiting runtime consumption.
  - `test_ocrm_plugins_host_hook_identity_and_gateway_methods` (PASSED)
    - Verifies next-turn prompt injection queuing, draining, and empty queue handling per session identity.
    - Verifies plugin session extension store patching, retrieval, and owned state clearing.
    - Verifies gateway UI descriptors surface operator.admin required scopes and session action echoes.
  - `test_ocrm_plugins_run_context_and_scheduled_turns` (PASSED)
    - Verifies plugin run context lifecycle scoped to active run IDs and cleared on termination.
    - Verifies session turn scheduling within MAX_DATE_TIMESTAMP_MS bounds and tag-based unscheduling.
  - `test_ocrm_plugins_session_actions_attachments_and_projection` (PASSED)
    - Verifies plugin session actions gateway dispatch and scope authorization.
    - Verifies session attachment delivery, MIME sniffing bounds (FILE_TYPE_SNIFF_MAX_BYTES), and temporary resource cleanup.
    - Verifies plugin session extension isolation within entry.plugins[pluginId][namespace] and non-destructive retirement stripping.
  - `test_ocrm_plugins_agent_executor_tool_middleware_and_before_reply` (PASSED)
    - Verifies agent prompt surface kind normalization (`pi_main` -> `openclaw_main`).
    - Verifies agent executor controller deprecation error guard (`api.registerAgentExecutorController`).
    - Verifies tool result middleware runtime normalization, tool matching, and execution pipeline.
    - Verifies before-agent-reply payload packaging with `SILENT_REPLY_TOKEN` (`NO_REPLY`).
  - `test_ocrm_plugins_guarded_api_lifecycle_auth_refs_and_widget_kinds` (PASSED)
    - Verifies late-callable plugin API method access after activation (`emitAgentEvent`, `clearRunContext`, etc.).
    - Verifies closed registration error on registration methods called after activation completion.
    - Verifies cross-workspace generation mismatch rejection.
    - Verifies recursive rewriting of legacy `authProfileId` and `defaultAuthProfileId` credentials.
    - Verifies board widget content kind naming constraints, reserved kind rejection (`html`, `mcp-app`, `plugin`), surface pattern, and gateway local resource path boundaries.

## 4. Audited Agent Executor, Tool Result Middleware & Before Reply Files (11 files)
- **Responsibility**: `OCRM-RESP-PLUGINS-HOOKS-PIPELINE-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Native Implementation**: `source/host/src/extensions/plugins/hook_runner.rs` (`normalize_agent_prompt_surface_kind`, `resolve_agent_executor_controller`, `AgentToolResultMiddlewarePipeline`, `build_handled_before_agent_reply_payloads`)

| Path | Lines | Size (bytes) | Git Blob SHA |
| --- | --- | --- | --- |
| `src/plugins/AGENTS.md` | 46 | 3943 | `91f06de2d130a0f80f747d096f76e439bd84d755` |
| `src/plugins/agent-executor-controller.test.ts` | 12 | 522 | `5a7b1531a9888e6156ef247ab1e945f203a74cad` |
| `src/plugins/agent-executor-controller.ts` | 10 | 633 | `c9b67fb2f946caa522cff3662ee3e368235566c1` |
| `src/plugins/agent-executor-controller.types.ts` | 25 | 1015 | `a375b971b16a9c49f88cfbbacd5a3543c7543e0f` |
| `src/plugins/agent-prompt-surface-kind.ts` | 13 | 544 | `be6e9d9cdad8c194d4bdfac4d3463bc3e10344d9` |
| `src/plugins/agent-tool-result-middleware-loader.ts` | 80 | 3284 | `1a316586757985de8db082c87557f2e50dd20966` |
| `src/plugins/agent-tool-result-middleware-types.ts` | 45 | 1377 | `05338bcd56e3a5a41e9596696e6fefa0d8ec2921` |
| `src/plugins/agent-tool-result-middleware.test.ts` | 68 | 2554 | `2c65970425202a83ce6fddd209d69334f2a1a650` |
| `src/plugins/agent-tool-result-middleware.ts` | 120 | 4245 | `712b12d08afc67ab53e3ae7ec484f324e834f1dc` |
| `src/plugins/before-agent-reply.test.ts` | 176 | 6311 | `092e8ddfde3c48eed3fddbf652e553186dfadd8d` |
| `src/plugins/before-agent-reply.ts` | 111 | 4316 | `5c0469af1e491fc9e896083590222d34aa8241f9` |

## 5. Audited Plugin API Lifecycle, Facades, Auth Profile & Board Widget Files (10 files)
- **Responsibility**: `OCRM-RESP-PLUGINS-RUNTIME-FINAL-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Native Implementation**: `source/host/src/extensions/plugins/runtime_final.rs` (`GuardedPluginApiLifecycle`, `PluginAuthProfileRewriter`, `BoardWidgetContentKindValidator`)

| Path | Lines | Size (bytes) | Git Blob SHA |
| --- | --- | --- | --- |
| `src/plugins/api-builder.ts` | 146 | 5527 | `26b38c35bb5f2b23a9d7bb36277259166f24d142` |
| `src/plugins/api-facades.background.test.ts` | 149 | 6246 | `6e5e8e8ae382a893cb62e0867d7a126781cb9f2a` |
| `src/plugins/api-facades.cli.test.ts` | 451 | 20195 | `2f9d54e4df5e236cbca8ebec886d5fffa8d10332` |
| `src/plugins/api-facades.ts` | 157 | 5594 | `a492c30310243be4c5e3f53835f8d672ea351b6a` |
| `src/plugins/api-lifecycle.test.ts` | 160 | 6643 | `49635e985834927f884a44b581c817ea8941fcbb` |
| `src/plugins/api-lifecycle.ts` | 79 | 2604 | `9dd4251f50e82c5f956c38b2518e381665e77695` |
| `src/plugins/auth-profile-config-refs.ts` | 43 | 1349 | `f735d4faee2ec4a8ecfcff91ea11fa241aeaf68a` |
| `src/plugins/board-widget-content-kind.types.ts` | 39 | 1244 | `11d512a83e098fa31a545086d1bfd7a31bdf72fa` |
| `src/plugins/board-widget-content-kinds.ts` | 153 | 5780 | `60677cb6378e9f2df1e51b3fbc06296181774d75` |
| `src/plugins/build-smoke-entry.ts` | 10 | 586 | `b783f60f64be7bc15f0eb5e8d5ae399c5be4e526` |







