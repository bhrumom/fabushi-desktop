# OCRM Technical Dossier — Unit 11: System Commands, Shell & Local Exec (`OCRM-UNIT-EXEC-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M16/M17 (Subprocess Execution, Shell Runner, Process Tree Termination & Line Framing Subsystems)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 core process execution, timeout handling, error sanitization, process termination, tree killing, and bounded line framing files (totaling 1,454 lines) were audited line-by-line and mapped into canonical shards:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0045` | `src/process/exec.ts` | `b46a1511556b7da193b88c6d29fdd9630f4b67d3` | 383 | 1..383 (full-file) | `OCRM-RESP-EXEC-PROCESS-SPAWN-001` |
| `part-0045` | `src/process/exec-runner.ts` | `e9a9c349eddab54fc6f697f75463bfca858f9708` | 721 | 1..721 (full-file) | `OCRM-RESP-EXEC-RUNNER-TIMEOUT-001` |
| `part-0045` | `src/process/command-error.ts` | `6a49f9c79278aca9381b11a1323d30e6d351995b` | 87 | 1..87 (full-file) | `OCRM-RESP-EXEC-COMMAND-ERROR-001` |
| `part-0045` | `src/process/exec-termination.ts` | `8b799aea4cccb13dd5b76b4b25c49b4c025cf5d1` | 211 | 1..211 (full-file) | `OCRM-RESP-EXEC-TERMINATION-CONTROLLER-001` |
| `part-0045` | `src/process/kill-tree.ts` | `1a0ccc2f9732f307de6f87604fab3fec24d79514` | 11 | 1..11 (full-file) | `OCRM-RESP-EXEC-KILL-TREE-001` |
| `part-0045` | `src/process/bounded-line-framer.ts` | `e2395772f795cd0597b854c478f9f1cfb69bb5ea` | 36 | 1..36 (full-file) | `OCRM-RESP-EXEC-LINE-FRAMER-001` |

---

## 2. Decomposed Runtime Responsibilities

### Locked-source file audit (2026-10-28)

The following files were read in full from the locked authority checkout
`openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Their canonical inventory rows
record the complete line ranges, blob identities, module and responsibility
mapping; this is file-level evidence, not a directory or batch claim.

| File | Blob SHA | Lines | Read/mapping result |
|---|---|---:|---|
| `src/process/exec-termination.ts` | `8b799aea4cccb13dd5b76b4b25c49b4c025cf5d1` | 211 | Full controller semantics read; command-exec owner remains pending because Fabushi has no production caller implementing pending-spawn, PID-start identity, grace/force settlement, and Windows taskkill joining. |
| `src/process/exec-termination.test.ts` | `9338c4fc21acc3c2872776fd01fc82e9066c68f5` | 311 | Full upstream controller oracle read; retained as source behavior evidence only. |
| `src/process/exec-termination.liveness.test.ts` | `74dabf0ca05c5e26420ced9198acf666fadf873f` | 108 | Full liveness/settlement oracle read; no production wiring inferred. |
| `src/process/kill-tree.ts` | `1a0ccc2f9732f307de6f87604fab3fec24d79514` | 11 | Full compatibility barrel read; local Rust supervisor cleanup owner is separate from the command controller. |
| `src/process/kill-tree.test.ts` | `5fba870d269d7b00a0a666183f3dc68509900e17` | 636 | Full recursive-tree/force/Windows test source read; current Rust evidence is explicitly scoped to supervisor cleanup and Unix descendant exit. |

The source rows are `read-complete` and mapped to `OCRM-M25`. The two
termination-controller rows remain `not-wired` with blocker
`BL-EXEC-TERMINATION-CONTROLLER-CALLER-001`; the kill-tree row remains scoped
to the real supervisor callers and does not upgrade command-exec parity.

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-EXEC-PROCESS-SPAWN-001`**:
   - **Behavior**: Sandboxed process spawning, execution timeout enforcement (`timeoutMs`), cwd isolation, environment variable allowlisting/sanitization, and standard descriptor configuration.
   - **Rust Owner**: `source/host/src/extensions/local_exec/gateway_local_exec_sand_box.rs:GatewayLocalExecSandBox`.
   - **Oracles**: `OCRM-EXEC-O01`, `OCRM-EXEC-O02`.

2. **`OCRM-RESP-EXEC-RUNNER-TIMEOUT-001`**:
   - **Behavior**: Bounded command execution runner, activity timers, `no-output-timeout` deadline detection, and buffered output accumulation with memory bounds (`DEFAULT_EXEC_MAX_BUFFER_BYTES`).
   - **Rust Owner**: `source/host/src/runner/production_turn_run_shell_adapter.rs:run_production_turn_shell_lifecycle`.
   - **Oracles**: `OCRM-EXEC-O03`, `OCRM-EXEC-O04`.

3. **`OCRM-RESP-EXEC-COMMAND-ERROR-001`**:
   - **Behavior**: Command execution error formatting, output tail truncation, ANSI escape sequence stripping, and Grok spawn failure classification (`SpawnEnoent`, `SpawnPermissions`, `SpawnOther`).
   - **Rust Owner**: `source/host/src/extensions/local_exec/local_exec_failure_classifier.rs:classify_local_exec_failure`.
   - **Oracle**: `OCRM-EXEC-O05`.

4. **`OCRM-RESP-EXEC-TERMINATION-CONTROLLER-001`**:
   - **Behavior**: Process termination controller, graceful kill grace window (`killGraceMs`), fallback to SIGKILL / taskkill force kill, and process group reap settlement.
   - **Rust Owner**: pending command-exec termination controller owner; `GatewayLocalExecManager::cancel_operation` currently only sends a bridge cancellation request.
   - **Production Call Chain**: not wired. The shared `runner::process_tree::terminate_process_tree` is used by managed network-worker and box exec-daemon supervisors, but that is the separate `OCRM-RESP-EXEC-KILL-TREE-001` cleanup behavior and does not prove command-exec controller parity.
   - **Blocker**: `BL-EXEC-TERMINATION-CONTROLLER-CALLER-001` — command-exec cancellation/timeout lacks a production owner implementing pending-spawn handling, PID-start identity validation, grace/force settlement, and Windows taskkill joining.
   - **Oracles**: `OCRM-EXEC-O06`, `OCRM-EXEC-O07`.

5. **`OCRM-RESP-EXEC-KILL-TREE-001`**:
   - **Behavior**: Recursive process-tree discovery and termination, Unix process group signaling, Windows taskkill PID enumeration, and orphan process cleanup.
   - **Rust Owner**: `source/host/src/runner/process_tree.rs:terminate_process_tree`.
   - **Production Call Chain**: `NetworkWorkerProcess::terminate_child` and `BoxExecDaemonProcess::terminate_child` call the shared owner during managed-service shutdown; the owner captures descendants, signals the Unix process group or Windows taskkill tree, waits the grace interval, then force-signals the captured targets and waits for root settlement.
   - **Evidence Boundary**: the current Unix contract proves the root and recorded descendant are gone on graceful and force paths. PID-reuse identity protection, Windows taskkill behavior, and command-exec pending-spawn settlement belong to separate contracts and remain unverified here.
   - **Oracle**: `OCRM-EXEC-O08`.

6. **`OCRM-RESP-EXEC-LINE-FRAMER-001`**:
   - **Behavior**: Bounded LF-delimited byte stream framing, zero-copy chunk search, buffer memory cap enforcement with overflow error generation, and safe line yielding.
   - **Rust Owner**: `source/host/src/runner/mod.rs:BoundedLineFramer`.
   - **Oracles**: `OCRM-EXEC-O09`, `OCRM-EXEC-O10`.

---

## 3. Falsifiable Test Oracles (`source/host/tests/gateway_local_exec_sand_box_contract.rs`, `production_turn_run_shell_adapter_contract.rs`, `extensions_local_exec_contract.rs`)

1. **`OCRM-EXEC-O01`**: `sandbox_projects_provider_state_and_user_computers`
2. **`OCRM-EXEC-O02`**: `upload_and_download_enforce_gate_and_use_frozen_frames`
3. **`OCRM-EXEC-O03`**: `first_output_timeout_for_attempt`
4. **`OCRM-EXEC-O04`**: `run_production_turn_shell_lifecycle`
5. **`OCRM-EXEC-O05`**: `failure_classifier_matches_grok_spawn_errno_contract`
6. **`OCRM-EXEC-O06`**: `GatewayLocalExecSandBox::terminate`
7. **`OCRM-EXEC-O07`**: `local_exec_file_too_large_message`
8. **`OCRM-EXEC-O08`**: `box_exec_daemon_process_contract`
9. **`OCRM-EXEC-O09`**: `BoundedLineFramer::push`
10. **`OCRM-EXEC-O10`**: `bounded_line_framer_overflow_rejection`

---

## 4. Invariant Verification & Global Conservation

- **Exact Mapped Paths**: 124 (cumulative across Units 1–11)
- **Omitted Known**: 0
- **Total Tracked Paths**: 58,750
- **Unread / Unclassified Paths**: 58,626
- **Strict Conservation**: $124 + 58626 = 58750$.
- **All Global Completion Flags**: `false` strictly preserved.
