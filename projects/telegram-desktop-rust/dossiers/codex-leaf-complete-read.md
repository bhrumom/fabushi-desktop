# Complete read — .agents/shared/scripts/codex_leaf.py

Status: independently understood and mapped; exact-head verification pending  
Project: TDRP-001 Revision 9  
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`  
Source: `.agents/shared/scripts/codex_leaf.py@313c4941c61a4ec0e3b3b65cdaa59cc213916fdd`

## Inventory identity

The current recursive authority artifact is from GitHub Actions run `39332739034`, artifact `12225800249`, digest `sha256:cf0c3eca3fd586674f048a90182ab60a000030a48d261370f3d093c654c2dc56`. Direct reading of `upstream-recursive-inventory.json` shows 16,202 non-directory entries. The actual 1-based array entry 14 is `.agents/shared/scripts/codex_leaf.py`, blob `313c4941c61a4ec0e3b3b65cdaa59cc213916fdd`, size 44,543 bytes. Entry 15 is its qualification oracle `.agents/shared/scripts/codex_leaf_test.py@2831841e6b131ca2c42688fe24e00b739f825ccb`; entry 16 is `.agents/shared/scripts/cross_review_scope.py@d9fafdf9fa0857cc83f0ccb49d15d4fad8782768`.

Earlier durable notes used a migration-progression ordinal that called `.agents/shared/test-loop.md` entry 14. In the current authority artifact that path is array entry 17. This dossier uses the actual artifact array position and does not infer contiguous completion from older progression ordinals.

## Complete source read

The full 1,287-line production helper was read. It is repository workflow tooling for executing out-of-process Codex review/assessment leaves; it is not a Telegram messaging or Fabushi shipping runtime.

The helper owns these source responsibilities:

- **Availability/admission** — `command_check` locates the CLI (including platform-specific app lookup), executes version and login-status checks, emits a machine-readable availability result and fails closed when the adapter is unavailable or unauthenticated.
- **Durable run identity and files** — `paths`, `command_start`, `claim_leaf` and state helpers bind one immutable `run_id` to state, event transcript, worker stderr, final response, runner diagnostics and a per-leaf lock.
- **Serialized atomic state** — `leaf_lock`, `read_state`, `replace_file`, `write_state`, `update_state` serialize state mutation, atomically publish replacement state and bound recognized sharing-violation retry.
- **Terminal adjudication** — `transcript_events`, `thread_id`, `turn_failure`, `judge` accept only a completed turn plus a fresh non-empty final response, while nonzero exit, failed/incomplete/unreadable transcript, or missing/empty/stale final fails.
- **Exact activity ownership** — `transcript_writer`, POSIX group probes, Windows Job Object/file-sharing probes and `leaf_activity` classify the owned worker as alive/gone/unresolved without treating a recycled PID, foreign group/job, read-only handle or failed probe as ownership.
- **Exact cleanup** — `terminate_leaf`, `settle`, `end_own_leaf` terminate only the verified owned process group/job and preserve unrelated processes; unresolved ownership does not authorize destructive cleanup.
- **Start contract** — `command_start` validates cwd/prompt/binary, proves a resume source if requested, refuses duplicate names or partial files without explicit replacement, records sandbox/effort/model/schema/project-doc/deadline inputs, and only then launches a detached runner.
- **Execution fidelity** — `codex_command` constructs initial versus resume commands from the durable launch state. `launch_codex` feeds the exact prompt by stdin and applies only child-scoped environment compatibility changes rather than global configuration.
- **Runner ownership** — `launch_runner`, `claim_leaf`, `supervise`, `command_run` distinguish detached-runner acceptance from worker execution, persist runner/worker ownership identity, enforce the deadline and keep runner diagnostics separate from worker stderr.
- **Runner-loss recovery** — `lost_outcome` and `check_leaf` do not turn runner loss into worker failure while the worker/writer remains directly observable; once the worker is gone they reconstruct terminal truth from the durable transcript/final response.
- **Bounded sanctioned wait** — `command_wait` repeatedly checks the same durable run, returns a still-running result when its bounded wait expires, and returns success only when all selected leaves are terminal.
- **Resume lineage** — `resumable_thread` requires a terminal/inactive source and stable durable thread identity, recovering that identity from the transcript when needed and refusing active/no-thread/self resume.
- **Replacement/archive** — `refuse_if_active` prevents replacement while activity is alive or unresolved; `archive` moves the complete prior evidence generation under one collision-safe suffix or attempts rollback and fails closed.
- **Stop** — `command_stop` monotonically marks only a running leaf stopped, then cleans only its exact owned process set and reports surviving/unresolved activity as failure.
- **Diagnostics/projection** — `describe` and related helpers expose terminal reason, runner loss, owned PIDs, worker stderr and runner log without conflating those channels.

## Real callers and dependencies

Repository search plus full-file reads identified the real workflow consumers:

- `.agents/shared/codex-cli-leaves.md@659da44a8083c433c4e9b59ca28f64ba525bfbf8` — canonical availability, start/wait, runner-loss, resume, replacement/archive, stop and recovery contract; explicitly treats bounded foreground wait as the completion signal and requires phase-output validation after worker terminal success.
- `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361` — checks adapter availability, starts independent reviewers back-to-back, waits in the foreground, permits one bounded replacement retry and resumes the general reviewer for synthesis.
- `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` — selects Codex leaves for review/assessment phases, records native fallback on availability failure, and treats start plus foreground wait as the provider boundary.
- `.agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba` — binds review/assessment leaf inputs, synthesis resume and phase completion checks.
- `.agents/shared/phase-effort.md@dd07a4c611922a29a6892332db98afe76cd58952` — supplies selected effort and optional model pins.
- `.agents/shared/cross-review/protocol.md@51dabe142d5c3c7247e4050790c4e74b40f35b27` and `prompts.md@4e155d2bd7e647c12df7b008c279969bfd65e44d` — source-neutral frozen-scope, blind-review, arbitration, focused re-review and convergence semantics driven through the helper.

The helper is directly qualified by `.agents/shared/scripts/codex_leaf_test.py@2831841e6b131ca2c42688fe24e00b739f825ccb`, already closed independently as actual array entry 15. That suite exercises the helper's terminal oracle, atomic state, per-worker locking, archive rollback, activity ownership, runner loss, resume, stop/time limits, concurrent waits, diagnostics, POSIX process groups and Windows Job Objects/file sharing. The production helper and its qualification oracle therefore remain separate source entries with separate traceability rows.

No shipping Telegram/Fabushi product caller owns this helper.

## Existing-owner-first resolution

Current Fabushi has no copied `codex_leaf.py` or Codex-specific runtime, and none is needed. The applicable source-neutral responsibilities map to existing owners:

1. root `AGENTS.md` — repository AI development/review lifecycle and out-of-process worker safety/execution;
2. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` — independent review and AI worker lifecycle/execution qualification;
3. `projects/fabushi-communication-platform/quality/evidence-contract.md` — durable worker lifecycle and adapter-execution evidence.

Rejected candidates:

- shipping Coordinator/Host/Runner own product execution, not repository review-worker orchestration;
- release packaging consumes evidence but does not own external reviewer adapter admission/process/state semantics;
- copying the upstream helper or adding a Telegram/Codex runtime would create a source/provider-specific duplicate of existing source-neutral quality ownership.

No new product owner or ADR is required.

## Fabushi-equivalent contract

The source-neutral absorbed contract is:

`availability/auth gate -> declared fallback OR immutable run identity + effective launch inputs -> detached runner accepted -> exact owned worker/process state + durable transcript/final/diagnostics -> sanctioned bounded wait on the same run -> terminal adjudication -> independent phase-output validation`

with fail-closed branches:

- `adapter unavailable/unauthenticated -> declared fallback + recorded reason; no install/login/global mutation`;
- `detached runner launched -> not completion`;
- `bounded wait says running -> wait same run again; no sleep/poll/duplicate/replacement`;
- `runner lost + worker directly proved alive -> remain running`;
- `worker gone -> derive outcome from durable transcript/final`;
- `ownership unresolved -> no kill/replacement`;
- `deadline/stop -> exact-owned cleanup; survivors/probe errors remain non-pass`;
- `replace -> prove prior inactive -> whole-generation archive or fail closed -> new run`;
- `resume -> source terminal/inactive + durable first thread identity`;
- `worker terminal success -> separately validate required phase artifacts/effects`.

The upstream Python implementation, Codex command spelling, POSIX process calls and Windows Job Object calls are source-specific instruments. Fabushi retains the source-neutral adapter execution/lifecycle/evidence contract in existing repository governance owners rather than copying a Codex helper or adding a product runtime.

## Traceability IDs

- Requirement: `TDRP-R9-AI-WORKER-ADAPTER-EXECUTION-001`
- Oracles: `ORA-TDRP-AI-WORKER-ADAPTER-EXECUTION-001`, `ORA-TDRP-AI-WORKER-TERMINAL-001`, `ORA-TDRP-AI-WORKER-OWNERSHIP-001`
- Invariants: `INV-TDRP-WORKER-LAUNCH-BOUND-001`, `INV-TDRP-WORKER-WAIT-SAME-RUN-001`, `INV-TDRP-WORKER-TERMINAL-MONOTONIC-001`, `INV-TDRP-WORKER-RUNNER-LOSS-NOT-FAILURE-001`, `INV-TDRP-WORKER-EXACT-OWNERSHIP-001`, `INV-TDRP-WORKER-REPLACE-ATOMIC-001`, `INV-TDRP-WORKER-RESUME-LINEAGE-001`

## Verification disposition

This closure changes repository governance/evidence contracts only. It adds no shipping UI, network, Coordinator, Host, Runner, Telegram or Codex runtime. Exact-head executable verification remains the existing TDRP Revision 9 source-authority workflow plus the Rust desktop runtime regression on the final PR head.

After this one independent source-entry closure the counters are recursive=16,202, unknown=16,186, unread=16,159, omitted=0. Baseline acceptance remains fail-closed because full recursive source closure is still open. This counter change does not assert that array entries 1–15 are all independently closed.
