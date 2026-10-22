# Adaptive test-loop complete read

Status: mapped / exact-upstream read complete / exact-head verification pending  
Project: TDRP-001 Revision 9  
Fabushi write lease: bhrumom/fabushi-desktop@ecfaa45dcbf340f94cbd12231d1896ac15109389  
Upstream: telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a  
Tree: 5151db79560ed40bf3802482fcfb74b950be6c40  
Path: .agents/shared/test-loop.md  
Blob: 83c4732bf6139f7c6557f18dd87bb35c3d68808d  
Primary workflow callers: .agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c; .agents/skills/perform-task/references/pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7; .agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba; .agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5  
Recovery/helper anchors: .agents/shared/build-lock-recovery.md@ec75573e7efe74e9c9a344f3e66f2fc43ac7680f; .agents/skills/process-inbox/scripts/workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c

## Live authority reconciliation

Immediately before writing, Fabushi main was re-read at `ecfaa45dcbf340f94cbd12231d1896ac15109389` (tree `7c8a00aa26b1a316b7eca3f975bbbe9d3a7a6d36`); upstream `dev` was re-read at `351204f3695b4a39bc3f4d2f230c061b9a01fe2a` (tree `5151db79560ed40bf3802482fcfb74b950be6c40`), and the target entry still resolves to blob `83c4732bf6139f7c6557f18dd87bb35c3d68808d`. This is the fourteenth ordered recursive source entry in the live Revision 9 rebaseline.

Closing only this independently owned source entry changes recursive counters from unknown=16,189/unread=16,162 to unknown=16,188/unread=16,161; omitted remains 0. Baseline readiness, full source closure and final release acceptance remain false.

## Independent responsibility

This file is repository test/evidence workflow governance, not Telegram product runtime. It independently owns task-derived evidence selection, deterministic execution bookkeeping, causal assessment, bounded recovery/convergence, retention of prior passing evidence, implementation-fix handoff, and source/tool safety around selected runtime instruments.

The applicable Fabushi responsibility is source-neutral repository verification/evidence governance. Existing root `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, and `projects/fabushi-communication-platform/quality/evidence-contract.md` are the correct owners. No Telegram test runtime, Telegram account owner, second test runner, or product state owner is justified.

## Deterministic evidence loop

The core state machine is `EVIDENCE_AUTHOR -> RUN_SELECTED_CHECKS -> ASSESS`. Every selected check derives from the final diff or directly proved no-change proposition and binds a claim to the most direct practical instrument, explicit oracle/control, durable evidence and falsifier.

One run is one related planned execution set under a unique identity. Unrelated checks are not packed together merely to lower run count. Commands retain exact argv/action, working directory, environment additions, exit/outcome and complete logs or artifact pointers. Passing evidence carries forward until a concrete implementation/fixture/oracle/environment/dependency change invalidates it; reruns execute only failed or invalidated checks.

Assessment defaults to not approved:
- `APPROVED`: every selected applicable check has direct positive evidence against its declared oracle.
- `TEST_FLAW`: command/environment/fixture/probe/oracle/control/capture/evidence cannot decide the claim; implementation is retained and only the invalid evidence path is repaired.
- `IMPL_BUG`: a sound check proves a defect; implementation is fixed, targeted/focused review is rerun as required, still-valid approvals carry forward, and invalidated checks rerun.
- `UNRECOVERABLE/BLOCKED`: only after independent directness assessment proves a required subject/capability cannot safely be reached.

Inference, missing evidence, a command that merely should have produced an artifact, or unsupported hardware are never passes.

## Retry, flake, timeout and cancellation

A repeated failure signature triggers a fresh recovery assessment. It records prior positive proof, the failed assumption, the forbidden repeated technique, a more direct next instrument, and why that instrument executes or observes the changed surface while retaining an independent oracle.

The normal campaign cap does not end the task. The default is one normal campaign plus at most one focused recovery campaign. Passing checks are preserved, the focused campaign runs only unmet checks and controls, and a third campaign is not started automatically. At the focused cap an independent exhaustion assessment must either prove all applicable strategies unsafe/unavailable/non-causal or name the still-plausible strategy and the human/environment decision required. A cap, elapsed time, timeout, repeated signature, blank capture, probe complexity, or unresolved flake never becomes approval or an artificial blocker.

Timeout/hang/crash classification is causal. A sound trace showing application code wedged/crashed is an implementation bug; an external deadline because the evidence harness never reached or observed the subject is a test flaw unless direct evidence proves otherwise. Missing/ambiguous evidence remains a test flaw.

## Build-lock and recovery contract

The test loop delegates Windows output-lock recovery to `.agents/shared/build-lock-recovery.md`. That contract resolves the exact source checkout, build root and executable, preserves the complete failed build output, stops only exact-path or otherwise verified build-owned holders, and deletes only exact named artifacts inside the owned build root.

A safe retry requires the named artifacts to be deleted/absent and no verified holder to remain. Query failure does not authorize killing an unverified process. Recovery is bounded to at most three recovery-and-build rounds for one build phase; a recovery round consumes neither an implementation attempt nor a test run. Exhaustion is a hard stop, never a pass.

Fabushi absorbs these source-neutral ownership/safety semantics, but not Telegram-specific Windows paths or helper implementation.

## Telegram-specific instrumentation boundary

The remaining large portions of the upstream document describe optional Telegram Debug instrumentation: portable account/golden tdata custody, test-account setup, `-testagent`, disposable overlay mechanics, Telegram widget discovery, in-process screenshots, Crashpad/Breakpad diagnostics, macOS activation/App Nap behavior, exact Telegram executable cleanup, and Telegram-specific runtime report format.

Those mechanics are not an independently applicable Fabushi product/test owner. Their source-neutral responsibilities are already captured by the selected-instrument prerequisite rule, exact subject/process ownership, durable command/runtime evidence, timeout/crash causal classification, account/data safety, and fail-closed recovery. Fabushi must use its own canonical Electron/Host/Runner/E2E/packaging test surfaces and only instantiate an equivalent specialized instrument when a Fabushi requirement actually needs it.

## Caller-chain verification

- `.agents/skills/perform-task/SKILL.md` explicitly delegates the implementation/test state machine to this file and delegates bounded build-lock recovery separately.
- `.agents/skills/perform-task/references/pipeline.md` enters the loop after a green implementation or already-satisfied review, starts from assessed `test-design.md`, writes `test.md` before execution, caps normal/focused campaigns, and uses the workspace helper for selected runtime runs.
- `.agents/skills/perform-task/references/phase-prompts.md` requires every acceptance surface to have a direct instrument/oracle/control/durable evidence and defines evidence-recovery leaves that retain implementation unless a sound check proves `IMPL_BUG`.
- `.agents/skills/continue/SKILL.md` preserves test/recovery artifacts across retries, rejects campaign-cap/`TEST_FLAW`/blank-evidence as genuine task blockers, and resumes focused test recovery without rerunning prior passes.
- `.agents/shared/build-lock-recovery.md` owns exact-checkout/build-root lock cleanup and bounded retry semantics.
- `.agents/skills/process-inbox/scripts/workspace.py` implements the upstream helper mechanics and validates that approved outcomes retain adaptive evidence; it is an implementation caller/helper, not a reason to create a Telegram runtime owner.

No shipping Telegram product caller is implied by this source entry.

## Existing-owner-first disposition

Fabushi already has the correct source-neutral owners:
- root `AGENTS.md` owns development lifecycle, verification boundaries, fail-closed policy, exact-head requirements and handoff;
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` owns test design, levels, pass/fail, flake, recovery and GitHub Actions execution policy;
- `projects/fabushi-communication-platform/quality/evidence-contract.md` owns durable run/attempt evidence, command/process ownership, exact-head provenance and append-only recovery history.

The mapped change extends only those owners with adaptive check selection, causal classification, carry-forward/invalidation, bounded focused recovery, build-lock safety and implementation-fix handoff. No Telegram-specific test owner or second repository workflow engine is introduced.

## Traceability anchors

Requirement: `TDRP-R9-ADAPTIVE-TEST-LOOP-GOVERNANCE-001`  
Oracle: `ORA-TDRP-ADAPTIVE-TEST-LOOP-001`  
Invariants: `INV-TDRP-EVIDENCE-DIRECT-001`, `INV-TDRP-PASS-CARRY-FORWARD-001`, `INV-TDRP-RECOVERY-BOUNDED-001`, `INV-TDRP-FAILURE-CAUSAL-001`, `INV-TDRP-BUILD-LOCK-OWNERSHIP-001`

These anchors bind the ledger row to this exact source-read dossier. Exact-head execution evidence remains pending until the Actions runs for the final PR head succeed.

## Verification boundary

This closure is documentation/governance mapping only. It does not claim that the whole upstream inventory is closed or that Fabushi product/release acceptance is complete. The exact mapping commit must pass the existing TDRP Revision 9 source-authority workflow and Rust desktop runtime workflow on the same PR head before the row's exact-head blocker can be cleared. Even after those runs pass, unknown=16,188 and unread=16,161 keep baseline/source closure fail-closed.

## Live dev rebaseline reread — e14f1566d277d2204f4c8e553bf7cc157047e88b
- Full source reread: `.agents/shared/test-loop.md@f3b38415883c95a6086766f965d8130333c1b09c`, 1,160 lines.
- Delta: Windows `--host-load` can boundedly wait for low non-client CPU/no build tools, trace host load during the client run, preserve process-incarnation/client-tree exclusions, and write wait/trace/JSONL evidence. Unsupported hosts and sampler/read/write failures remain diagnostic and do not alter the run outcome.
- Existing-owner-first: this is test instrumentation and evidence-quality governance, not a shipping platform/runtime capability.
- Parent exact-head 51e019e3afd13d7132d5da27bc273f0203a03eae became historical after live dev advanced: TDRP run 39743918294 completed 61/61 success with authority 12410423121/sha256:842b833317a5a1303957fc36a8152a2aad7863e0b60dab8c90e40a59a6898db2, closure 12411320551/sha256:2ed7335ec4abf02148d00427e8914da0212789bfdc136765f018c2043130a6a1, reachability 12410951384/sha256:a9337bc5ba21ff89398df9f012cbfdc3e8b31898614d3bd11354b3f905c9bd90; Rust run 39743918410 completed 6/6 success with artifact 12409889590/sha256:858be7d3c3c48657053fd8a9d576e854e04dbf2d9c4db300627376e90b7ed38d. These runs validate old 5c674f1f6a0f6ace55b67f8bc295138680fe588b only and are not reused as current evidence for e14f1566d277d2204f4c8e553bf7cc157047e88b.
