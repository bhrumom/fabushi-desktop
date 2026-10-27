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

