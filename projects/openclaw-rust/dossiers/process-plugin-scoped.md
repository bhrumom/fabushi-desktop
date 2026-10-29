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
   - `Consumer`: No production consumer is currently wired to these extension owners. The only current consumer is the contract fixture `source/host/tests/openclaw_process_plugin_contract.rs`; the existing Box exec-daemon path has separate child-lifecycle and framing code and must not be treated as an implicit caller.

## 3. Verification & Oracle Evidence
- **Contract Tests**: `source/host/tests/openclaw_process_plugin_contract.rs`:
  - `test_ocrm_process_bounded_line_framer` (PASSED)
  - `test_ocrm_process_custody_and_tree_reaper` (PASSED)
  - `test_ocrm_plugins_runtime_final` (PASSED)

## 4. AppContainer negative-oracle scope correction (2026-10-29)

The locked `src/process/appcontainer-spawn.test.ts` source reference was read
directly at blob `ea084ac6af2d706fb898cbdf1edd456d67e2bca8` from the pinned
OpenClaw commit/tree; its actual length is 76 lines. The test is a Windows
AppContainer pre-launch boundary oracle: pipe-producing direct spawn,
synchronous Windows probes, custom supervisor transports, and Execa options
must throw `AppContainerSpawnUnsupportedError` before the child or launch
callback exists.

This source reference does not itself establish bounded line framing, process
group reaping, command-capacity groups, or deadline enforcement. Those are
separate source responsibilities in the broader ledger row. The current Rust
`source/host/src/extensions/process/command_custody.rs` owns capacity,
cancellation, and deadline bookkeeping but has no AppContainer token/stdio
preflight or Windows production caller; the existing process dossier therefore
retains `read=partial`, `wiring=not-wired`, and `verification=not-run` for the
full responsibility. The direct source read strengthens the negative oracle
only and is not a Windows acceptance receipt.

## 5. Production wiring boundary (2026-10-29)

Static caller inspection confirms that `CommandProcessCustody`,
`ChildProcessTreeTracker`, and `extensions/process::BoundedLineFramer` have no
production call site in `source/host/src` or `source/host/app/src`. Their
current Rust use is limited to the contract test fixture. The separate
`source/host/src/runner/bounded_line_framer.rs` and
`source/host/src/box/exec_daemon_process.rs` owners are not interchangeable
without a reviewed process-execution adapter, because the custody contract
also owns admission, cancellation, deadline, and process-tree semantics.

Accordingly this responsibility remains `wiring=not-wired`, and no production
acceptance claim may cite the contract test as runtime evidence. A future
wiring change must identify the real command-dispatch owner, acquire custody
before spawn, register/unregister the child with the same lifetime, propagate
cancellation/deadline outcomes, and add the corresponding exact-head
verification before changing this status.
