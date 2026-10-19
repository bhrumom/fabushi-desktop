# Fabushi Test Strategy

Status: active / normative  
Strategy ID: FQT-001  
Revision: 1  
Parent: FBCP-001 Revision 7  
Updated: 2026-10-07

## Quality model

验证 functional suitability、reliability、performance efficiency、usability/accessibility、security/privacy、compatibility、maintainability/observability 与 release/recovery。UI 好看、build 绿色或单一路径可用均不是完整质量。

## Risk classification

- P0 critical: 数据/权限/账号隔离、消息丢失/串线、terminal result 错误、安全/隐私、不可恢复 corruption。
- P1 high: ordering/reconciliation、Agent/tool settlement、reconnect/restart、发送/附件、关键 UI route/controls、升级/回滚。
- P2 medium: 次要交互、非关键 settings、低频 presentation states。
- P3 low: cosmetic/minor。

P0/P1 默认需要多层测试，不能只靠 E2E。

## Required test levels

1. Unit/table-driven：纯函数、validation、projection、formatting、state transitions。
2. Property/state-machine：ordering、dedupe、idempotency、replay、merge、cancel/restart、组合 event sequences。
3. Contract：Rust/Host/bridge/renderer/service schemas、IDs、terminal/error/cancel semantics。
4. Integration：real owners/stores/reducers/projections 注入复杂时序。
5. Functional E2E：完整 Electron shipping composition 真实 user journey。
6. Temporal/recovery E2E：关键状态点、quiet settlement、switch/reconnect/reload/restart。
7. UI functional + visual + a11y：control/routes/state + layout/style/accessibility/responsive/locale。
8. Fault/security/performance/soak：适用风险域。
9. Independent exploratory acceptance：独立 reviewer 审阅 packaged timeline/video/evidence。

## Test design techniques

使用 equivalence partitions、boundary values、decision tables、state-transition tests、pairwise/combinatorial、property/model-based、error guessing（补充）和 risk-based exploratory。关键状态机至少有合法/非法 transition cases。

## Standard flow

Discover test basis -> risk -> oracle/invariants -> RTM/test design -> fixture/environment -> implement tests -> GitHub Actions -> evidence review -> defect/regression -> independent acceptance -> release summary。

## Pass/fail semantics

Pass = expected behavior + required evidence。Fail = oracle violation。Blocked/skipped/not-run = no pass evidence。Flaky = unresolved nondeterminism；critical gate 视为 block/fail。禁止 rerun-until-green。

## Cross-review semantics

Independent cross-review covers the complete declared diff, not a convenient subset. General and specialist reviewers remain read-only and report concrete correctness, integration, lifetime, reuse, structure, performance, security, data-safety or maintainability failures rather than preference-only suggestions. Specialist NOT_APPLICABLE requires changed-path/hunk-specific proof; uncertainty remains applicable.

Synthesis rechecks findings against current code, merges duplicates, drops unsupported findings, and rejects unsupported NOT_APPLICABLE decisions. A NEEDS_CHANGES-style gate requires at least one supported blocking failure. After fixes, focused re-review confirms every accepted fix and checks regressions in edited functions plus relevant callers/consumers without reopening unrelated unchanged code.

### Arbitration semantics

Cross-review scope is frozen once per session and snapshotted per round so reviewers judge the same declared change, including explicitly in-scope pre-existing uncommitted work. Round 1 uses fresh blind independent general + applicable specialist reviewers in parallel; later rounds use a fresh focused general reviewer and only the specialist lenses that originated accepted findings. Reviewer reports remain mutually blind; persistent state belongs to the source-neutral runner/ledger, not to a provider process.

Every synthesized finding is arbitrated before fixes as `FIX`, `REJECT`, `DEFER`, or `COMPROMISE`; `NON_BLOCKING` never starts a fix round alone. Reversals must reference the earlier decision and add a previously unweighed concrete failure before reconsideration. One bounded supersession is allowed; a second attempted reversal is a human-decision stop. After any FIX, focused re-review must confirm the previous finding and check the touched functions plus adjacent callers/consumers against the complete frozen scope.

### Focused re-review and stopping

The loop terminates only on convergence (no FIX after valid reversal handling), configured round cap (default 10), non-convergence/scope expansion into another subsystem, or a required human product/arbitration choice. Open real defects in changed code remain blocking regardless of round pressure.

## AI review phase execution policy

Repository AI review/test phases use the source-neutral worker-effort contract in root `AGENTS.md`: routine already-specified execution may use `medium`; planning, implementation, judgment, evidence design/assessment, diagnosis, synthesis and convergence default to `xhigh`; `high` is allowed only with a concrete inspected-scope reason that no complexity is present. Failures needing diagnosis/repair escalate while preserving prior valid evidence.

On Codex review providers, reuse, structure and performance lenses run on `gpt-6.1-sol` at the selected effort. If unavailable they fall back to the configured model and record the fallback. General review/synthesis, lifetime/security lenses and other phases retain the configured model unless another source-neutral host policy explicitly overrides it. Host adapters that cannot set effort/model inherit the actual session configuration and record that limitation rather than approximating it through prompt text.

### AI worker lifecycle qualification

Any repository adapter that launches an external AI worker/reviewer is qualified as a durable lifecycle state machine rather than by observing only the launcher/runner process. A terminal success requires a completed worker turn plus a fresh non-empty final response bound to the same durable run identity; unreadable transcripts, missing completion, stale/empty finals, unresolved ownership, or ambiguous state remain non-passing. Transient pre-completion events and partial trailing records may be tolerated only when the later authoritative completion/final evidence is intact.

State publication and replacement are fail-closed. State/terminal records are atomically replaced under an owned lock or equivalent primitive, bounded sharing-violation retries do not overwrite the prior valid state, unchanged updates do not manufacture new evidence, and terminal states are not rewritten by later polling. A lost launcher/runner is not itself worker failure: while the worker, its owned process group/job, or its transcript writer is directly observed alive, the lifecycle remains running; probe errors remain unresolved. Once the owned worker is gone, outcome recovery is reconstructed from the durable transcript/final response rather than inferred from the runner exit.

Stop, timeout, recovery and replacement act only on the exact owned process group/job/session. Reused PIDs, unrelated groups/jobs, and plain readers are negative controls and must survive. An ownership/probe failure never authorizes a kill or replacement. Resume requires a completed source with a stable worker/thread identity and is refused while that source remains active. Replacement is allowed only after the prior leaf is proven inactive; its state, transcript, final response, stderr and runner diagnostics are archived as one coherent generation, with an all-or-none rollback contract on archive failure.

Qualification must cover normal completion; worker/turn failure; missing, stale and empty finals; startup failure; time limit; stop; runner loss before and after worker claim; resume; concurrent waits; state-lock/state-write failures; replacement/archive failure; untracked live writers; reused process identities; and applicable POSIX process-group plus Windows job-object/file-sharing behavior. These are source-neutral repository workflow responsibilities: the qualification does not require copying an upstream Codex helper and does not create a shipping Agent/Coordinator/Host/Runner owner.

## Independence and environment

Implementer 可写 tests，但 release reviewer 必须独立复核。FBCP/TDRP 所有 executable verification 只在 GitHub Actions；fixture/locale/theme/viewport/network/fault/account identity 必须进入 manifest。

## Artifact identity and copy provenance

Packaging/release tests treat artifact identity as a first-class contract. A byte-exact staging or installation copy must verify independent source/copy whole-file digests and size, re-check source stability across the copy window, and preserve the machine-readable result with exact-head evidence. Intentional transforms require platform-specific semantic equivalence oracles with paired positive/negative controls and bounded resource behavior; absence of such an oracle is a fail-closed blocker, not permission to infer equivalence from filenames, signatures, normalized hashes, or successful launch. Executable verification remains GitHub-Actions-only.

### Format-transform positive/forbidden controls and report

Any platform-specific executable transformation oracle must be tested with independent accepted controls and paired forbidden mutations that prove protected loader/content/metadata semantics can actually fail. When the real toolchain performs the transform, at least one real-toolchain control is required in addition to synthetic fixtures. Tests must exercise declared parser/resource boundaries and stable-acquisition failures. Passing evidence is a machine-readable exact-head report containing both raw artifact identities, the declared profile/limits, acquisition/stability observations, semantic allowances/comparisons, and the final verdict; optimized/runtime modes that can disable assertions are separately controlled where applicable.



### Transform-oracle independent qualification suite

A format-specific executable transform oracle is qualified only by controls that are independent of the implementation layout. Literal fixtures, field locations, mutation offsets and expected semantics must not be derived from the production parser under test. The full qualification suite must contain accepted cases and paired forbidden mutations covering loader/program/header semantics, allocated/NOBITS/TLS state, retained and unknown section payloads, qualified debug/compression allowances, retained static-symbol meaning/order, exact and one-over parser/I/O/resource limits, nonregular/symlink/FIFO acquisition, deterministic mutation/replacement races, and final stability checks. Refusing every input is not qualification.

When a real shipping toolchain performs the transform, qualification also runs a genuine compiler/transform pair and independently reads the resulting artifact identities. Runtime modes that can alter assertion behavior, including optimized interpreter modes where applicable, are exercised separately. Focused authoring subsets may shorten iteration but never replace the full qualification suite. FBCP/TDRP executable qualification remains GitHub-Actions-only.


### Configured-build graph qualification

A configured-build provenance oracle is qualified with independent parser fixtures plus at least one real CMake/Ninja fixture when the production build uses that toolchain. Qualification covers Ninja and Ninja Multi-Config manifest/config binding, minimum supported tool capability, safe tool/operand construction, query/compilation-database parsing, stable manifest acquisition, bounded shell/compiler/link grammar, object/link graph reconciliation, compile producer/source/output identity, archive/shared-library classification, command-line definition ordering, PCH producer/consumer coherence, exact path mapping, and machine-readable PASS/error behavior. Positive fixtures must also demonstrate that the oracle reads the graph without executing graph commands.

Negative controls include malformed or ambiguous graph/query/compdb records, unsupported shell operators/drivers/definition channels, unexpanded response files, duplicate/substituted/missing link objects, order-only substitutions, producer/source/output mismatches, incoherent PCHs, stale/changed manifests, wrong configuration/phony aliases, ambiguous source selectors, and unsupported path identities. A real fixture may use compiled objects to independently cross-check selection, but that cross-check does not widen the oracle's own configured-graph-only claim.

This qualification remains source-neutral. Do not create a Telegram- or Ninja-specific Fabushi test owner when the existing quality/evidence owners can express the contract. Archive-member reachability, executable/runtime reachability, symbol evidence, and release acceptance remain separate gates and cannot be inferred from configured-build provenance.


## Adaptive evidence loop and bounded recovery

The source-neutral verification lifecycle is `EVIDENCE_AUTHOR -> RUN_SELECTED_CHECKS -> ASSESS`. Evidence design is reconciled against the final diff (or a directly proved already-satisfied proposition) before execution. Each check binds one claim to a direct instrument, oracle, control/negative, durable evidence, and falsifier. One run groups only setup-compatible related checks; unrelated commands are not packed together merely to reduce run count.

Assessment defaults to not approved. `APPROVED` requires direct positive evidence for every applicable selected check. `TEST_FLAW` means the evidence mechanism cannot decide the claim and repairs the command/environment/fixture/probe/oracle/capture without changing retained implementation. `IMPL_BUG` means a sound check exposed a defect; the implementation is repaired, required focused review is performed, still-valid approvals are carried forward, and only invalidated checks rerun. `BLOCKED/UNRECOVERABLE` requires an independent directness assessment proving that a required subject/capability cannot safely be reached; unsupported hardware/service/account configuration is recorded as unverified/blocked, never simulated into a pass.

Passing evidence is cumulative and explicitly invalidated rather than discarded. A rerun executes only failed or invalidated checks. A repeated failure signature must change the evidence strategy or advance the directness of the observation; repeating the same flaky technique is forbidden. The default managed campaign budget is two campaigns total: one normal campaign and at most one focused recovery campaign. The focused campaign contains only unmet checks and their controls. A third campaign is not started automatically; at the cap, an independent exhaustion assessment either proves all applicable strategies unsafe/unavailable/non-causal or records the still-plausible strategy and required human/environment decision. Caps, timeouts, elapsed time, blank captures, and repeated failures never become approval or a blocker by themselves.

Flake is unresolved nondeterminism and remains non-passing. Prefer condition/state settlement over fixed sleeps, exact subject identity over descendant/order guesses, and observed output/runtime values over summaries. Timeout or cancellation is classified by causality: an owned implementation deadlock/wedge is an implementation failure; a harness that failed to reach or observe the subject is a test flaw unless direct evidence proves otherwise.

### Build-lock and recovery safety

When a verification/build job owns a mutable build tree, lock recovery is exact-subject and bounded. Resolve the exact build root, executable/command owner, and named locked artifact before acting. Preserve the complete failing output. Stop only a verified owned process/session or holder tied to that build root; delete only exact named artifacts inside the owned build tree. Broad process-name kills and unverified cleanup are prohibited.

Retry only after the recovery record proves the named artifacts are deleted/absent and no verified holder remains. A cleanup/query error never authorizes killing an unverified process. Where the lock-recovery mechanism applies, use at most three recovery-and-build rounds for one build phase; recovery bookkeeping itself does not consume an implementation attempt or evidence run. Exhaustion remains a hard stop, not a pass. For FBCP/TDRP, all executable recovery/build verification still runs only in GitHub Actions.
