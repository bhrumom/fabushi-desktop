# Complete read — .agents/shared/scripts/codex_leaf_test.py

Status: independently understood and mapped; exact-head verification pending  
Project: TDRP-001 Revision 9  
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`  
Source: `.agents/shared/scripts/codex_leaf_test.py@2831841e6b131ca2c42688fe24e00b739f825ccb`

## Inventory identity

The current recursive authority artifact is from GitHub Actions run `39330010837`, artifact `12223269894`, digest `sha256:fb1d8c1ccb13090b97725f40d6c32394f1c01180872763d4f3c66918c98217d0`. Its `upstream-recursive-inventory.json` contains 16,202 non-directory entries. Reading the artifact's `entries` array directly, the actual 1-based entry 15 is `.agents/shared/scripts/codex_leaf_test.py`, object/blob `2831841e6b131ca2c42688fe24e00b739f825ccb`, size 94,503 bytes.

Earlier durable notes used a migration-progression ordinal that called `.agents/shared/test-loop.md` “Entry 14”; in this artifact that path is array entry 17. This dossier uses the actual artifact array position requested by the current task and does not infer contiguous closure from those older ordinal notes.

## Complete source read

The full 2,277-line source was read. It contains 79 `test_*` cases around `codex_leaf.py@313c4941c61a4ec0e3b3b65cdaa59cc213916fdd`, with a fake Codex executable, lifecycle hooks, process/lock fixtures, state/transcript/final-output files, and POSIX/Windows-specific controls.

Independent responsibilities covered by the suite:

- terminal adjudication: a completed worker turn plus fresh non-empty final output is success; failed turns, nonzero exits, unreadable/incomplete transcripts, missing/empty/stale finals fail closed; the first durable `thread.started` identity is authoritative;
- atomic state publication: recognized sharing violations receive bounded retry, unrelated errors do not; exhausted retry is a named state-write failure; unchanged state does not rewrite evidence;
- locking: nested acquisition is busy and lock ownership is released after holder death;
- replacement/archive: the complete prior evidence generation moves under one collision-safe suffix or none moves; rollback failure identifies any stranded file;
- activity identity: writer tokens, POSIX process groups and Windows job membership distinguish alive/gone/unresolved; PID reuse, foreign groups, plain readers and failed probes do not establish ownership;
- monotonic lifecycle state: terminal states are never rewritten, startup grace is bounded, busy locks write nothing, and absent state does not invent a run;
- launch contract: sandbox, effort, model, schema, project-doc and time-limit options plus cwd/environment are preserved in durable command state;
- runner loss: losing the runner before/after worker spawn does not equal losing a still-live owned worker; waiting continues and later derives terminal outcome from durable transcript/final state;
- resume: a completed source resumes from its durable thread identity, a lost-runner transcript may recover it, and resume is refused while the source remains active;
- startup/argument failures: missing binaries and invalid inputs fail without fabricating or partially archiving a new lifecycle;
- timeout/stop: only the exact owned process set is ended, including after runner loss; stopping an already-finished worker does not rewrite the terminal result;
- concurrency/races: spawn-gap writers remain discoverable, unresolved identity blocks destructive replacement, state-write sharing errors do not start duplicate work, and concurrent waiters cannot race a terminal state backward;
- diagnostics: worker stderr remains distinct from runner lifecycle/exception diagnostics;
- incident sequence: resume + runner loss + blocked replacement + eventual completion + whole-generation archive + resumed replacement preserves lineage;
- Windows controls: Job Object membership after runner loss, reader-vs-writer distinction, file-sharing violations, breakaway/no-breakaway caller jobs, and caller-job closure are explicitly exercised.

These are repository workflow qualification responsibilities, not Telegram messaging/product runtime responsibilities.

## Real callers and dependencies

The test directly imports and exercises `.agents/shared/scripts/codex_leaf.py@313c4941c61a4ec0e3b3b65cdaa59cc213916fdd`. Repository search found no production caller of the test filename; the test is a direct qualification suite for the helper.

The helper is consumed by current upstream workflow surfaces:

- `.agents/shared/codex-cli-leaves.md@659da44a8083c433c4e9b59ca28f64ba525bfbf8` — availability, start/wait, runner-lost, resume, replacement/archive, stop and recovery semantics;
- `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361` — availability check, concurrent reviewer starts, foreground wait, bounded replacement retry, synthesis resume;
- `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` — Codex review/assessment provider and blocking wait as completion signal;
- `.agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba` — review/assessment phases, synthesis resume and completion checks;
- `.agents/shared/phase-effort.md@dd07a4c611922a29a6892332db98afe76cd58952` — effort/model arguments;
- `.agents/shared/cross-review/protocol.md@51dabe142d5c3c7247e4050790c4e74b40f35b27` and `prompts.md@4e155d2bd7e647c12df7b008c279969bfd65e44d` — the source-neutral review lifecycle that host adapters drive through the helper.

No shipping Telegram product/runtime caller owns this test or helper.

## Existing-owner-first resolution

Current Fabushi has no copied `codex_leaf.py` or Codex-specific workflow runtime. That is not a novel product-capability gap. The applicable source-neutral responsibilities map to existing owners:

1. root `AGENTS.md` — repository AI development/review lifecycle, worker effort, bounded review lifecycle and exact cleanup ownership;
2. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` — cross-review/AI phase policy and lifecycle qualification matrix;
3. `projects/fabushi-communication-platform/quality/evidence-contract.md` — evidence-command ownership, AI worker evidence and durable lifecycle evidence.

Rejected candidates:

- shipping Coordinator/Host/Runner own product execution, not repository review-worker orchestration;
- release packaging consumes evidence but does not own reviewer-worker process/state semantics;
- copying the upstream helper or creating a Telegram/Codex workflow runtime would duplicate source-neutral repository quality ownership.

No new product owner or ADR is required.

## Fabushi-equivalent contract

The absorbed lifecycle is:

`declared AI worker -> immutable run identity -> atomic starting/running state -> exact owned session/process-group/job + durable transcript -> sanctioned wait -> completed turn + fresh final => terminal success OR causal failure`

with fail-closed branches:

- `runner lost + worker directly proved alive -> remain running/wait`;
- `ownership probe uncertain -> unresolved; no replacement and no collateral kill`;
- `deadline/stop -> exact owned cleanup; survivor/probe failure => non-pass`;
- `replace -> prove inactive -> whole-generation archive or no move -> new run`;
- `resume -> prior source terminal + durable first thread/session identity`;
- `concurrent waiter -> observe but never rewrite terminal state`.

The upstream Python, fake Codex fixture, POSIX calls, Windows Job Object calls and sharing-violation hooks are instruments. Fabushi absorbs their source-neutral invariants into existing repository quality/evidence owners rather than creating another executable worker subsystem.

## Traceability IDs

- Requirement: `TDRP-R9-AI-WORKER-LIFECYCLE-QUALIFICATION-001`
- Oracles: `ORA-TDRP-AI-WORKER-TERMINAL-001`, `ORA-TDRP-AI-WORKER-OWNERSHIP-001`
- Invariants: `INV-TDRP-WORKER-TERMINAL-MONOTONIC-001`, `INV-TDRP-WORKER-RUNNER-LOSS-NOT-FAILURE-001`, `INV-TDRP-WORKER-EXACT-OWNERSHIP-001`, `INV-TDRP-WORKER-REPLACE-ATOMIC-001`, `INV-TDRP-WORKER-RESUME-LINEAGE-001`

## Verification disposition

This closure changes repository governance/evidence contracts only; it adds no shipping UI, network or service owner. Exact-head executable verification remains the existing TDRP Revision 9 source-authority workflow plus the Rust desktop runtime regression, both on the final PR head.

After this one independent source-entry closure the counters are recursive=16,202, unknown=16,187, unread=16,160, omitted=0. Baseline acceptance remains fail-closed because full recursive source closure is still open. These counters do not assert that artifact array entries 1–15 are all independently closed.
