# Plugin Computer Use Contract & Configuration Engine Dossier

## 1. Scope & Source Attribution
- **Target Files (16 files in `src/plugins/`)**:
  - `src/plugins/compiled-sdk-host.test-support.ts` (88 lines)
  - `src/plugins/compiled-sdk-host.test.ts` (100 lines)
  - `src/plugins/computer-use-contract.test.ts` (371 lines)
  - `src/plugins/computer-use-contract.ts` (488 lines)
  - `src/plugins/computer-use-registration.ts` (224 lines)
  - `src/plugins/config-activation-shared.ts` (202 lines)
  - `src/plugins/config-contract-matches.ts` (116 lines)
  - `src/plugins/config-contracts.test.ts` (244 lines)
  - `src/plugins/config-contracts.ts` (160 lines)
  - `src/plugins/config-normalization-shared.ts` (212 lines)
  - `src/plugins/config-policy.test.ts` (65 lines)
  - `src/plugins/config-policy.ts` (1 lines)
  - `src/plugins/config-schema.test.ts` (108 lines)
  - `src/plugins/config-schema.ts` (168 lines)
  - `src/plugins/config-state.test.ts` (450 lines)
  - `src/plugins/config-state.ts` (254 lines)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugins).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGINS-COMPUTER-USE-CONFIG-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 Computer Use Protocol & Execution Management
- **Actions Matrix**:
  - 40 V2 Actions: `screenshot`, mouse (`left_click`, `right_click`, `middle_click`, `double_click`, `triple_click`, `mouse_move`, `left_click_drag`, `left_mouse_down`, `left_mouse_up`), keyboard/timing (`scroll`, `type`, `key`, `hold_key`, `wait`), apps/windows (`list_apps`, `list_windows`, `get_accessibility_tree`, `get_cursor_position`, `get_window_state`, `launch_app`, `kill_app`, `bring_to_front`, `set_value`, `zoom`), browser actions (`get_browser_state`, `browser_prepare`, `browser_navigate`, `browser_click`, `browser_type`, `browser_dialog`, `browser_set_input_files`, `browser_download`, `browser_pointer`), scope/recording (`escalate_scope`, `get_recording_state`, `start_recording`, `stop_recording`, `replay_trajectory`, `invoke_menu`).
  - V1 Action subsets: 15 actions for V1, 13 act actions for V1.
  - Scroll directions: `up`, `down`, `left`, `right`.
  - Escalation reasons: `ax_tree_pixel_mismatch`, `background_delivery_failed`, `foreground_ineffective`, `no_window_target`, `other`.
- **Single-Flight Concurrency & Admission**:
  - `COMPUTER_EXECUTION_ID_PATTERN`: UUID v4 pattern.
  - Concurrency enforcement: Only one active provider execution owns the computer session. Non-matching execution IDs fail fast with `COMPUTER_HOST_BUSY: another provider execution owns this computer`.
  - Clean close barrier drains pending actions before allowing another execution to acquire the computer lock.

### 2.2 Plugin Configuration Engine
- **Contract Matching**:
  - Path traversal across wildcards (`*`), canonical array indexing (`0`, `1`, etc.), and nested structures.
  - `hasPluginConfigMigrationSource`: Evaluates touched config paths against declared contract migration patterns without widening unrelated config edits.
- **Normalization & Enablement State**:
  - Plugin ID canonicalization (trimmed lowercase).
  - Explicit enable/disable markers in config entries (`enabled: true/false`, string `"disabled"` / `"off"`).
  - Precedence order: explicit entries > slot selection (`memory`, `contextEngine`) > allowlist (for non-bundled plugins) > default enablement > auto.
- **Memory Slot Decisions**:
  - Handles exclusive single-provider constraints for memory management.
  - Retired plugin detection fails closed to prevent deprecated store corruption.
  - Automatically selects single available memory provider when not explicitly configured.

## 3. Native Rust Implementation & Equivalence
- **Implementation Modules**:
  - [`source/host/src/extensions/plugins/computer_use.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugins/computer_use.rs).
  - [`source/host/src/extensions/plugins/config.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugins/config.rs).

## 3.1 Current production reachability boundary (2026-10-31)

The production Worker path constructs
`ProductionComputerToolExecutor` in `source/host/app/src/main.rs` and routes
actions through the existing Runner/box Computer owner. The adjacent
`ComputerUseCoordination` lease and audit surfaces are therefore real runtime
evidence. The plugin-owned `ComputerExecutionManager` and the plugin
configuration functions in `extensions/plugins/config.rs` are not directly
consumed by that path: no provider execution ID is handed to the manager and
no production plugin-config admission call was found. The responsibility
therefore remains `wiring=not-wired` and acceptance remains pending until the
single-flight/config contract reaches its intended producer and consumer.

## 3.2 Production recheck — 2026-11-01

The current checkout was re-read at the two candidate boundaries. Worker
construction still creates `ProductionComputerToolExecutor` and delegates
through the existing Box/Computer owner, while the production runner already
uses `ComputerUseCoordination` for its scoped lease and audit lifecycle. No
production caller passes a provider execution ID to
`extensions::plugins::computer_use::ComputerExecutionManager`, and no
production caller invokes the plugin configuration activation/migration
resolvers before Computer execution. Wiring the plugin manager directly into
the executor would introduce a second concurrency/configuration authority and
would contradict the existing owner boundary. The responsibility therefore
remains `implementation=implemented`, `wiring=not-wired`,
`verification=not-run`, `acceptance=pending`.

The native execution-id validator was also compared with the fixed upstream
`COMPUTER_EXECUTION_ID_PATTERN`. It now rejects uppercase hex, unsupported
version nibbles and unsupported variant nibbles instead of checking only
length and generic hexadecimal characters. Focused negative contract cases
were added; they remain unexecuted during the development freeze.

## 3.3 Test Oracles

- `source/host/tests/openclaw_plugins_contract.rs`:
  - `test_computer_use_contract_and_execution_manager`
  - `test_plugin_config_contract_matching_and_activation`

## 3.4 Fixed-subject equivalence gap — 2026-11-01

The fixed upstream `src/plugins/computer-use-registration.ts` is a
node-host registration owner, not merely a UUID validator or mutex. Its
observable execution lifecycle includes a close barrier that blocks a new
acquisition until the previous physical close settles, joins concurrent close
requests, preserves a failed physical close for an explicit retry, and treats
failed open/failed close ownership differently. The snapshot command also has
a short-lived execution path when no `executionId` is supplied.

The native `ComputerExecutionManager` currently provides only the synchronous
single-active-ID portion of that contract. `closed_executions` is write-only,
there is no pending/closing barrier or close-join state, and the manager is not
called by the production `ProductionComputerToolExecutor`/Runner/Box path.
The existing `ComputerUseCoordination` lease is the production owner for the
current Rust computer-use path, but it does not establish equivalence to the
upstream node-host provider lifecycle.

This is an explicit unresolved equivalence gap, not production acceptance
evidence. It does not authorize wiring the plugin manager into the executor or
introducing a second concurrency owner. Status remains
`implementation=implemented` for the present native helper slice,
`wiring=not-wired`, `verification=not-run`, and `acceptance=pending` until a
spec-backed node-host producer/consumer boundary exists and the lifecycle
semantics can be verified there.
