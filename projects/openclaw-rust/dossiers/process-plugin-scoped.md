# Process Custody, Bounded Line Framer & Plugins Final Lifecycle Dossier

## 1. Scope and Responsibilities
- **Scope**: Native Rust host runtime extensions and contract tests for process custody execution, bounded line framing, child process tree termination, and plugin runtime uninstallation receipts. Bulk shard coverage claims for `src/plugins/` and `src/process/` have been revoked and downgraded to unread/unclassified pending individual per-file reading per Spec §6.2-6.3.
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Mapped Responsibilities**:
  1. `OCRM-RESP-PLUGINS-RUNTIME-FINAL-001`: Plugin Runtime Finalization, Uninstallation Receipts, Status Reporting & Setup Wizards.
  2. `OCRM-RESP-PROCESS-CUSTODY-EXECUTION-001`: Bounded Line Framing, Child Process Tree Reaping, Command Capacity Groups & Deadline Enforcement.

## 2. Producer -> Owner -> Storage -> Consumer Traces
1. **Plugin Runtime Final Lifecycle Pipeline**:
   - `Producer`: Plugin uninstallation requests, status updates, and runtime service lifecycle triggers.
   - `Owner`: `source/host/src/extensions/plugins/runtime_final.rs` (`PluginRuntimeManager`).
   - `Storage`: Process-local status map and uninstallation receipt ledger.
   - `Consumer`: Plugin UI settings panel and host discovery sweepers.
2. **Process Custody & Bounded Framing Pipeline**:
   - `Producer`: Raw stdout/stderr byte streams from spawned child processes and command execution requests.
   - `Owner`: `source/host/src/extensions/process/bounded_framer.rs` (`BoundedLineFramer`), `source/host/src/extensions/process/command_custody.rs` (`CommandProcessCustody`), and `source/host/src/extensions/process/tree_reaper.rs` (`ChildProcessTreeTracker`).
   - `Storage`: Line framing buffer and active PID tracker set.
   - `Consumer`: PTY executors, terminal log sinks, and background command monitors.

## 3. Verification & Oracle Evidence
- **Contract Tests**: `source/host/tests/openclaw_process_plugin_contract.rs`:
  - `test_ocrm_process_bounded_line_framer` (PASSED)
  - `test_ocrm_process_custody_and_tree_reaper` (PASSED)
  - `test_ocrm_plugins_runtime_final` (PASSED)
