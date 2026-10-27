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
- **Test Oracles**:
  - `source/host/tests/openclaw_plugins_contract.rs`:
    - `test_computer_use_contract_and_execution_manager`
    - `test_plugin_config_contract_matching_and_activation`
