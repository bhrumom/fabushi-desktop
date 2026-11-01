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

### Review-scope snapshot qualification

The frozen-scope owner is qualified independently from reviewer judgment. Cover uncommitted mode and bounded last-N-commit mode; exact base/starting-HEAD capture; too-deep commit-range rejection; repository-root normalization; pre-existing dirty-state capture; inclusion of untracked files; isolated staging/index behavior that leaves the real index untouched; stable tree identity; rename-aware and binary-safe complete scope diff; changed-path/status and short-stat projection; round-to-round fix-delta selection from the nearest prior snapshot; missing/corrupt session failure; and explicit empty-scope termination.

Positive qualification proves that two reviewers consuming the same round observe the same immutable scope identity even while fixes create a later round. Negative controls must prove that scope capture cannot silently omit untracked/binary/renamed changes, widen the base, mutate the real index, invent a previous round, or classify failed acquisition as an empty/pass result. These are source-neutral review-evidence responsibilities and do not require preserving an upstream Python helper.

### Repository quality-health qualification

Qualify repository health observation separately from product behavior. Positive cases bind a scan to a freshly resolved exact source revision, prove that only an exact-ref scan can publish canonical findings, preserve stable finding identities and first-seen/status history across successive observations, and distinguish a genuinely resolved finding from an identity that is unknown at the comparison base. When a pinned dependency is unavailable, qualification requires an explicit skipped/unobserved record rather than silently treating that source as healthy.

Atlas qualification covers exact-file/glob/prefix specificity, one-area ownership, zero ambiguous/unmapped source files for an approved map, stale pattern detection, and responsibility-oriented area boundaries. A hot-area survey consumes the exact scanned checkout plus current map and open findings, uses independent source-reading lenses, rechecks every claim that drives the map or proposal, and emits proposals/defects separately. Survey output does not itself change source or create product behavior.

### Repository quality-health integrated regression qualification

Qualify the repository-quality subsystem with source-neutral integration fixtures that exercise the declared text, build-model, include/dependency, clone, atlas, report, review and command-orchestration contracts together without creating a second implementation of those owners. The harness may construct temporary repositories and representative source/build/map inputs, but every assertion must be tied to an already-declared oracle: lexical span/provenance, dependency/layer direction, clone maximality, stable findings, atlas specificity/coverage, change-ratchet warnings/errors, move-only accounting, finding verification, or canonical quality-state lifecycle.

Stateful positives prove durable exact-ref scans publish once per scheduling window unless forced, append history on a deliberate rerun, keep mutable working-tree scans noncanonical, rebuild a missing local snapshot from the exact published revision, and create/reuse/remove only the owned exact-commit inspection worktree. Command-level positives and negatives preserve exit/result distinctions such as clean versus warning/error, present/resolved/changed/unknown finding state, and move-only versus substantive change. Fixtures must include the relevant negative controls so a test cannot pass merely because the instrument returns no observation.

This integration suite is qualification evidence for the existing repository-quality owners, not a production runtime, canonical scanner, alternate findings ledger, ownership map, or task queue. Upstream-specific Python unittest plumbing, temporary Git fixture helpers and literal source samples are test mechanics rather than responsibilities to port. Fabushi may use different test technology so long as the same source-neutral oracles and negative controls are represented by exact-head automated evidence.

### Repository quality-health command orchestration qualification

Qualify the quality-health command boundary independently from its metric libraries. Durable-scan positives bind acquisition, snapshot, findings merge, report and publication to one immutable source revision; preserve stable finding identities/status and prior first-seen history; record the exact run/totals; publish replacement state atomically; append trend history; and prove an already-recorded scheduling window is skipped without creating a second durable run. A working-tree scan is explicitly temporary and must leave canonical ledger/history untouched. Missing cached raw snapshots may be rebuilt only from the exact revision named by canonical state and then remain cache evidence rather than a new authority.

Command-surface qualification covers exact-ref scan, temporary scan, area report, atlas draft/check, base-relative review check, move-only proof, finding verification, and detached source inspection. Positive cases prove each command delegates to the qualified repository-quality instruments and preserves their declared result/exit semantics. Detached inspection binds the worktree HEAD to the requested/published commit, initializes only the declared source dependencies needed for inspection, reuses only a matching exact worktree, and removes only a path carrying the dedicated temporary-worktree identity. Negative controls cover missing/failing fetch, commit mismatch, stale/partial source, malformed canonical state, execution exceptions, atlas coverage negatives, present/changed/unknown findings, non-move-only changes, and review errors; none may be rewritten as a clean scan or release proof.

Integration qualification proves orchestration owns no parallel parser, metrics implementation, findings semantics, responsibility atlas, product owner, task queue, or release verdict. Scanner/report/review/atlas facts remain with their existing owners, while proposal routing and owner approval remain in the ordinary quality-task/process-inbox/perform-task lifecycle. Temporary output directories and caches cannot substitute for durable publication receipts, and any passing command evidence remains repository-quality evidence rather than proof of product behavior, configured build success, packaging, runtime correctness, or release acceptance.

### Repository responsibility-atlas lifecycle qualification

Qualify atlas creation and upkeep as a source-review lifecycle, not as scanner output becoming truth. Positive cases bind the draft to an exact source revision, preserve stable unique area identities and layer ordering, retain the skeleton/source facts that motivated candidate boundaries, and prove that independent readers plus a separate verifier can falsify responsibility claims before owner approval. The coverage check must fail on any unmapped source file, equal-specificity ownership ambiguity, or path pattern that matches nothing. Candidate split thresholds, dependency/co-change statistics, churn, task cost, and test reach are investigation signals only and cannot approve an area boundary by themselves.

### Repository maintenance-history measurement qualification

Qualify repository-history measurements as bounded evidence inputs, not semantic ownership or acceptance. Churn must be computed against one exact scanned revision and explicit lookback window; root and pinned submodule histories remain revision-bound, unavailable submodule history is recorded as unavailable rather than fabricated as zero churn, and rename-disabled counting is treated as a declared measurement convention rather than stable file identity. Task-effort qualification binds source changes to explicit `Task:` trailers, the earliest recorded Start and latest recorded Approve lifecycle events, a positive elapsed interval, the touched source paths, approval date, and the highest observed review-round marker. Missing queue state, missing trailers, incomplete lifecycle endpoints, non-positive durations, or unrecognized review artifacts produce absent/unknown measurement rather than a healthy or zero-cost result.

Aggregation qualification preserves raw per-path/per-task observations and proves count/median/total summaries are derived only from the retained evidence set. Hotspot, churn, elapsed hours and review rounds may prioritize investigation or survey selection, but cannot establish responsibility ownership, authorize routing, approve atlas/map changes, prove product behavior, or satisfy build/release acceptance. Integration checks prove the history instrument remains a delegated input to the existing repository-quality report owner and does not become a second task store, review owner, queue, product runtime, or release verdict.


Committed atlas text requires explicit owner approval. Subsequent source reshaping produces narrow amendments rather than whole-map rewrites; area maps must continue to agree with the approved atlas, and an area rename is treated as an identity/history change rather than silently preserving old metrics. Negative controls reject auto-committing a generated skeleton, using mutable working-tree observations as canonical atlas evidence, broadening an area to hide a known boundary break, or creating a second repository/product ownership runtime from atlas metadata.

### Repository responsibility-atlas instrument qualification

Qualify the atlas instrument independently from the human atlas lifecycle. Parser/classifier positives prove source-root normalization, exact-file > glob > directory-prefix specificity, longer-pattern precedence within one pattern class, stable cached results, and explicit reporting of equal-best claims instead of silently selecting one. Empty/missing approved atlas text must not fabricate authority; any coarse fallback used by measurement remains visibly non-authoritative. Coverage qualification must enumerate the declared C/C++/style source scope and reject any unmapped file, equal-specificity ambiguity, or configured pattern with no matching in-scope file.

Candidate-skeleton qualification proves thresholds are drafting heuristics rather than ownership decisions. A large candidate may split only into children meeting the configured minimum, residual descendants remain owned by the nearest selected candidate ancestor, and the skeleton retains line/file totals plus applicable largest-file/name-group, cross-area include/dependency, recent co-change, churn, task-cost and test-reach evidence. Repository-local co-change must not be synthesized across independent submodule histories. Negative controls reject using the thresholds, dependency counts, churn or task cost as owner approval, treating a generated skeleton as a committed atlas, or creating a second atlas/ownership store.

CLI/integration qualification proves the library does not own the command surface: an explicit draft check returns a non-passing result when coverage negatives exist; generation writes both machine-readable and human-readable skeleton evidence under an exact-revision state directory or an explicit temporary directory; ordinary scan/report consumers use an approved atlas for stable area identity when present and otherwise retain only a coarse measurement fallback. Underlying Git/process failures must surface as execution failure rather than being rewritten as coverage success. A passing generation/check cannot approve atlas text, modify product source, route tasks, or substitute for owner approval and release evidence.

### Repository duplicate/clone evidence qualification

Qualify clone evidence independently from any proposed refactor. Detector positives prove that only the declared translation-unit/header source kinds are scanned; comment removal, logic-free-line filtering and whitespace collapse retain deterministic normalized sequences plus original source line provenance; candidate windows are grouped without turning very common low-signal buckets into quadratic evidence; matches extend to maximal runs; overlapping same-file matches are clipped; mirrored declaration/interface blocks are rejected; output ordering is deterministic; and stable clone IDs derive from the normalized duplicated block rather than mutable path coordinates.

Review-copy qualification proves that added hunks are compared against the exact current source snapshot after the same normalization, the identical current location is excluded, only runs meeting the review threshold are emitted, and every warning/error identifies both the added source range and existing source location. Negative controls cover comments-only or logic-free similarity, declaration mirroring, below-threshold runs, same-location self-matches, overlapping same-file sequences, high-frequency common windows, and code that only appears similar after a behavior-changing normalization. Strictness belongs to the existing review/check command owner: non-strict may warn, strict may reject, and detector output never becomes a source edit or task by itself.

Integration qualification proves scan/report consumers may count clone pairs/lines, attribute both sides to existing responsibility areas, create stable clone findings, and let surveys display clone evidence; Q2 proposals and `health.py verify` remain owned by the existing class/task lifecycle. A detected clone is investigation evidence, not proof that a mechanical merge is correct. Passing clone evidence cannot authorize a merge, create a second findings ledger, mutate product source, bypass ordinary review, or substitute for exact-head GitHub Actions evidence.

### Repository source-text normalization evidence qualification

Qualify source-text normalization against one fixed exact source snapshot and the lexical grammar the instrument actually implements. Positive fixtures cover declared translation-unit/header suffix classification; quoted `#include`/`#import` extraction with original 1-based line numbers; identifier extraction; bounded definition-name recognition including qualified names/operators; multiline column-zero function spans and constructor initializer lists; class/struct boundaries and one-level member counting; block-comment removal across lines without changing line cardinality; inline comment trimming; logic-free-line filtering; and deterministic whitespace collapse.

Negative controls cover forward declarations and aggregate/object initializers that must not become functions; unsupported include forms; nested or macro-generated definitions outside the recognizer; malformed/unclosed comment or body boundaries; comment-like text in literals/raw strings; preprocessor/namespace/using/trivial lines excluded from normalization; and source kinds outside the declared translation-unit/header set. The result must state the recognizer limitation instead of treating a miss as proof that the corresponding C++ semantic fact is absent. Original path and line provenance must survive every reportable lexical observation.

Integration qualification proves only caller-owned repository-quality effects: metrics may derive function/class/unused candidates, includes may build a resolvable project include graph, clones may compare normalized substantive lines while retaining source positions, report may aggregate those observations into existing findings, review may derive its own warnings/errors/strictness, atlas may scope candidate source facts, and the health command may publish or check them under its existing exact-ref/error/routing rules. The text primitive itself must not mutate source, create findings/tasks, approve placement, infer compiler semantics, replace a build/test result, or become a second parser/ownership runtime.

### Repository source-derived build-model evidence qualification

Qualify a source-derived build model against a fixed exact source snapshot and the recognizer grammar it actually implements. Positive cases prove deterministic build-file ordering, bounded balanced-call extraction, declared comment handling, namespace/scope normalization, target/source assignment, direct and transitive link closure, recognized PCH mapping, executable/library classification and the deliberately coarse test-source rule. Where a source can appear in multiple recognized lists, qualification must prove the documented deterministic ownership choice rather than silently depending on map iteration order.

Negative controls cover empty calls, aliases that are not concrete libraries, namespace-qualified targets, filtered scopes/options/variables/quoted tokens, nonmatching source roots, unsupported or unevaluated build-language constructs, cyclic link graphs, PCH tokens outside the recognized form, non-test executables and toolkit/support libraries that must not become tested production sources by inference. Missing recognition is never evidence that the corresponding real build relationship is absent.

Integration qualification proves only the source-level consumers selected by repository quality governance: include-graph/build-cost estimation, cross-target layering checks, coarse tested-line attribution and responsibility-atlas skeleton facts. The same cases prove those consumers keep their own warning/error, findings, approval and task-routing boundaries. Any claim about an evaluated configured target, object/source producer, selected PCH, link input, generator/configuration or command-line definition must switch to configured-build graph provenance qualification. Passing source-derived model evidence cannot replace a real build, release acceptance, configured graph evidence, product mutation or exact-head GitHub Actions evidence.

### Repository responsibility-area-map qualification

Qualify an area map as an owner-approved refinement of one approved atlas area. Positive coverage proves the map is bound to the exact reviewed source authority and parent area; Modules account for current files or tight file groups without inventing a second ownership model; Placement gives deterministic answers for the kinds of additions the area actually receives; Invariants are objectively checkable and carry approval provenance; Leave alone protects deliberate choices from cleanup churn; Direction states the intended target shape without becoming current authority by implication. File references named by the map are checked for staleness.

Lifecycle qualification distinguishes draft, approved instruction and narrow amendment. A survey or other source review may propose a map but cannot self-approve it. Owner approval is required before committed Placement/Invariants/Leave-alone/Direction become governing facts. A structural task that changes placement updates the map in the same reviewable change; Q0-Q2 mechanical work may adjust only Modules facts that moved or disappeared unless separately approved; semantic reshape/redesign work must satisfy its map/invariant preconditions before implementation. Repeated placement findings require re-survey/revalidation rather than an ever-growing exception list.

Negative controls reject a map that contradicts the atlas boundary, broadens an area to hide an ownership break, treats surrounding code as authority when an approved map says otherwise, accepts an unapproved invariant or Direction edit, lets a mechanical task rewrite semantic map policy, retains stale file names as passing evidence, or creates a second repository/product ownership runtime. Map qualification supports placement and maintainability proof only; it never replaces the task's product, security, build, runtime or release verification.


### Repository responsibility-area survey qualification

Qualify a deep area survey against the exact scanned source authority, never the caller's mutable working tree. Positive cases prove the survey consumes the canonical area report plus any prior survey/current map and open area findings; avoids treating already-measured scanner facts as new semantic evidence; and keeps responsibility/state-machine, duplication, coupling/build, testability and incidental-defect lenses independent, each grounded in direct path/line observations. Selection by hotspot/task cost or an explicitly named area is prioritization only and cannot pre-decide the survey outcome.

Verification is a separate qualification phase. Every claim that influences a map or proposal is independently reproduced by re-reading the cited lines, repeating the relevant count/search, or confirming caller absence. Rechecked claims may be labeled verified; unrechecked claims remain explicitly inferred. Negative controls reject cross-contaminating lens conclusions, promoting inferred material to verified evidence, using the same unreviewed conclusion as both proposal and verification, or treating a model consensus as owner approval.

Output qualification keeps four effects distinct: survey narrative/plan, draft or amended map, bounded maintainability proposals, and defects found in passing. A map stays draft until the ordinary map owner approves it; proposals remain requests until ordinary class/trust/routing gates accept them; defects enter the ordinary defect lifecycle instead of code-health cleanup. The survey itself must produce zero source-code changes and zero queue records. Any source mutation, self-approved map, direct task insertion, hidden defect, second survey/map/task store, or claim that survey output alone proves product/build/security/release acceptance is non-passing.

The change-time health ratchet is proportional to the diff and exact base. Qualify new cross-owner dependency rejection, oversized-new-file and over-limit growth controls, class/function growth where applicable, added clone detection, header/build-reach growth, removed-definition references across platform files, and orphaned last users including styles/keys. Move-only evidence must reject substantive new/lost logic and reordered statements while permitting relocation scaffolding. Finding verification distinguishes present, resolved, changed-successor, and unknown identities instead of treating absence from the current scan as automatically resolved.

### Repository quality-task class qualification

Qualify the class before accepting its proof. For deletion, prove every relevant build target, platform/source variant and generated input has no remaining reference, then prove the affected targets still build and the ratchet finds no stranded last user. For move-only work, prove substantive lines are neither created, lost nor reordered, placement follows the nearest approved map, direct consumers/build declarations remain correct, and build-work reach does not regress. For identical-copy consolidation, prove the copies differ only by explicit parameters, all named call sites use the shared implementation, and the clone finding is resolved. If any of these changes requires semantic edits, new behavior, broadened scope or an unproven equivalence, reject the mechanical classification and require the stronger ordinary pipeline.

Characterization work must pass on unchanged product code and fail under multiple deliberate, independently reverted mutations of the behavior under test; enumerated conversions prefer kind-complete coverage. Semantic reshapes require the affected characterization suite to exist and pass before implementation and again after it, with targeted runtime/integration proof whenever behavior lives outside pure logic. Redesign requires owner-approved checkable invariants before coding, at least two independent structural alternatives when a new seam/layer/owner is being chosen, a preference for an existing repository mechanism over a new one when equivalent, and staged implementation whose intermediate tasks retain their own proof. The final redesign proof turns the approved invariants into executable checks or tests.

Task proof follows the selected health class. Q0–Q2 use narrow mechanical proof plus an independent review; characterization work must demonstrate mutation sensitivity; semantic reshapes require already-passing characterization coverage; redesign requires approved checkable invariants and independent alternatives. Routing limits, in-progress path conflicts, recent-authority drift, owner approvals, placement-map leave-alone rules and audit suspension are negative controls. Health metrics, scanner output, or a clean ratchet alone never replace the task's required build/test/runtime/release evidence.

### Repository quality-health native invocation qualification

Native skill-surface qualification is intentionally smaller than scanner qualification. For a host registration/default prompt, prove discovery resolves to the existing repository quality-health owner, the default invocation selects only a supported mode, fresh exact-source authority remains mandatory before durable publication, regressions/decisions are reported before proposals, and scheduled/ad-hoc routing still obeys the same trust, conflict, approval, and ordinary task lifecycle gates. Negative controls reject a registration or prompt that embeds a second health implementation, advances canonical state from a mutable/stale observation, routes despite suspension/conflict, or treats provider-specific command syntax as semantic ownership.

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

### AI worker adapter execution qualification

Repository adapters that implement an external AI worker lifecycle must be qualified for both lifecycle safety and execution fidelity. In addition to the lifecycle matrix above, cover: availability/authentication probe and declared fallback; immutable start-state capture; sandbox, effort, model, output-schema, project-document and deadline argument preservation; new-run versus resume command construction; detached-runner launch acceptance versus actual worker completion; bounded foreground wait returning still-running versus all-terminal; child-scoped environment compatibility without global mutation; diagnostics separation; and phase-artifact validation after the worker itself reaches a valid terminal result.

The production adapter contract is qualified by an independent oracle rather than by copying its implementation. Positive and negative controls must exercise unavailable binaries/authentication, startup/argument failures, normal completion, nonzero/failed turns, missing/stale/empty finals, resume lineage, replacement/archive, runner loss, stop/time limit, state/lock failures, ownership ambiguity, unrelated processes, and applicable POSIX/Windows ownership behavior. The adapter and its qualification suite remain repository tooling responsibilities, not a shipping Agent/Coordinator/Host/Runner runtime.

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

### Repository helper-inventory instrument qualification

Qualify reusable-helper inventory as exact-revision repository evidence, separate from the retention decision policy that consumes it. The instrument must discover helper modules and documentation from the scanned source authority, derive current-path birth/age only from reachable history, classify helper kind, identify direct source callers and transitive internal dependents, retain declarative workflow and open-task references, and classify archived task observations into independent product, uncertain/unclassified, harness-only, self-test-only and creation-task evidence. Distinct tasks are consumers; duplicate calls or saved patches within one task do not inflate reuse.

Archive coverage is passing only when the canonical task source is readable and stable enough to support the claimed observations. Missing task storage, no usable approved patches, read errors, dirty/changing task state, shallow source history, unknown task kind, missing approval date or missing saved patch remain explicit limitations. A failed or incomplete acquisition cannot become observed zero use. Final saved patches are a lower bound, not execution telemetry; direct source references and internal dependency closure are static evidence only and do not prove configured-build/runtime reachability.

Positive contracts cover exact-source document reads, current-path birth provenance, helper-kind separation, unique-symbol ownership, direct/internal caller separation, transitive helper dependency closure, archived-consumer de-duplication, uncertain/harness/self-test/creation classification, open-task holds, workflow references, documentation/index facts and JSON/Markdown evidence identity. Negative controls cover shallow history, absent queue, missing patches, dirty queue, unreadable state, ambiguous symbol ownership and comments/string literals that must not manufacture consumers. Stage A authors but does not execute these contracts.

### Repository helper-retention lifecycle qualification

Qualify reusable repository-test/helper retention as a source-neutral quality decision lifecycle, not as automatic dead-code deletion. Every durable helper inventory is bound to the same freshly resolved exact source revision as the quality scan. A helper may become a review candidate only after its current-path age is at least 90 days and the retained observations show either no independent use in the prior 90 days or at most two observed lifetime product consumers. Unknown or unclassified consumers count conservatively against an absence-of-use claim, and any known open consumer holds routing. Shallow or unavailable source history, incomplete archive coverage, missing approval dates, read failures, or a changing/dirty canonical task queue make the retention signal unverified rather than healthy or unused.

Documentation qualification is part of the same owner: the mandatory helper guide reaching 8 KiB, being missing, or an indexed helper lacking its on-demand reference is a decision signal, not product/build failure by itself. Stable retention/documentation identities survive daily scans; an unchanged candidate is not a new finding and age increasing by another day does not create another routing request.

Review newly eligible or materially changed retention candidates at most once per UTC calendar week. Before any retirement proposal, qualification must inspect the helper header/implementation role, direct callers, internal dependents, registration/runner entry points, declarative build inputs, workflow references, archived task consumers, known open work, and any already-pending local replacement. The archive is explicitly a lower bound: a missing saved patch or sparse archived use cannot prove that no consumer exists. Pure self-test/internal helpers are inventoried separately and cannot be retired merely because direct product calls are absent.

A qualifying decision records one of retain, defer, propose a smaller reference/API, or propose retirement through the ordinary quality-task lifecycle. Q0 or another task class is selected only after its own proof is satisfied; age, source/documentation weight, or consumer count alone never authorizes deletion. Positive contract cases cover stable identity, 90-day/lifetime thresholds, conservative unknown handling, open-consumer holds, weekly idempotence and exact-revision continuity. Negative controls prove stale/incomplete evidence cannot resolve a prior finding, sparse use cannot become automatic retirement, and a second helper findings/task owner cannot be created. Stage A may author these contracts but does not execute them.



### Repository helper-inventory regression contract-suite qualification

Qualify the helper-inventory instrument with an explicit source-neutral regression suite owned by the existing Fabushi repository quality/test-evidence lifecycle. The suite preserves the fixed-U `harness_test.py` contract surface without copying its Python `unittest` plumbing or creating a second scanner, findings ledger, task queue, fixture runtime, or product owner.

The authored contract matrix covers: the exact 90-day eligibility boundary and low-lifetime-consumer thresholds; helpers younger than the boundary; absent archive and no-saved-patch coverage remaining unavailable rather than zero use; creation-task, harness-only, self-test-only and independent product-use separation; surviving final-patch context counting while deleted lines and helper-implementation edits do not; de-duplication by task across repeated calls and patches; comments, string/raw-string literals and commented includes not manufacturing consumers; self-test/internal-helper classification, transitive internal consumption and open-work routing holds; unclassified consumers remaining uncertain rather than known-zero; exact-ref documentation reads versus mutable worktree reads; dirty archive state invalidating retirement evidence; namespace-qualified symbol ownership suppressing unrelated-name collisions; and stable finding identity/status when current evidence becomes unavailable.

The contract IDs are `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-BOUNDARIES-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-COVERAGE-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-CONSUMERS-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-PATCH-FILTERING-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-LEXICAL-OWNERSHIP-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-EXACT-REF-001`, and `CONTRACT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-FINDING-LIFECYCLE-001`; the suite-level integration contract is `INT-TDRP-CODE-HEALTH-HARNESS-REGRESSION-001`.

Stage A authors these contracts only. Their execution belongs to the Stage-B exact-head GitHub Actions candidate campaign; no source review, fixture design, or upstream test result is a passing Fabushi execution claim.


### Repository include-graph and build-cost qualification

Qualify the repository include/dependency graph as source-neutral maintainability evidence, not configured-build truth or product runtime behavior. Resolve only project-owned quoted C/C++ include/import edges plus declared style `using "...";` imports against the exact scanned source tree, in local-path, own-root, then other-root order; generated `styles/style_<name>.h` references may map to the matching `<name>.style` source. Unresolved names remain explicit observations.

Closure qualification is cycle-safe. Translation-unit `weight` is the source-line count of reachable project headers beyond the target PCH closure; header `reach` counts dependent translation units; header `rebuild` accumulates each dependent unit's own source lines plus non-PCH header weight; total build-work proxy adds all unit work plus each distinct declared PCH closure once. These are comparative source-line proxies, not wall-clock compile timings.

Layering qualification uses the existing target/dependency model: an include is a violation only when both file owners are known and the included target is outside the including target's transitive link dependency closure. Review/report/atlas consumers may project the same exact graph into warnings, findings or area edges, but none of these observations proves configured-build reachability, ownership, product behavior, acceptance or release.

Stage-A contracts authored, not run: `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-RESOLUTION-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-CLOSURE-PCH-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-BUILD-COST-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-LAYERING-001`, `CONTRACT-TDRP-CODE-HEALTH-STYLE-IMPORT-001`, and `INT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-001`.


### Repository source-metric, unused-candidate, and hotspot qualification

Qualify source-size/shape metrics as exact-revision repository maintainability evidence, not product runtime performance, configured-build reachability, or deletion authority. The instrument may group a path into a source-neutral fallback area, count source lines, parse translation-unit functions, record the maximum parsed function length, preselect functions at or above 150 lines, and preselect classes whose rough top-level method-plus-field count is at least 40. These are collection/pre-filter facts only. Downstream findings keep their own thresholds: the current source contract raises long-function findings at 300 lines and large-class findings at 120 methods or 60 fields; a pre-filter threshold must never be presented as the finding or release threshold.

Unused-function evidence is explicitly candidate-only. Count identifier mentions over the exact scanned source set while excluding the language resource from the identifier universe; inspect only non-test `Telegram/SourceFiles/*.cpp|*.cc|*.cxx` definitions; preserve the declared exemption family; require an alphabetic short name with no more than two total mentions; exclude constructor-shaped names; and accept a candidate only when remaining local mentions are declaration-shaped and the paired header plus local source account for the global count. Initializer-list calls, ordinary calls, assignments, indented uses, unrelated same-name mentions, exempt callbacks/operators, unknown external/generated consumers, or incomplete source acquisition must prevent an absence-of-use claim. Even a qualifying candidate is not deletion proof: removal requires the ordinary exact-head build/reference/test evidence in Stage B.

Hotspot is a downstream composition, not a primitive owned by the metric scanner. The current source contract combines exact-revision file lines with Git churn as `round(churn * lines / 1000, 1)`, aggregates the value by area, and may surface only the hottest bounded area set; an untested-hot-area signal additionally requires zero headless-tested lines and positive hotspot. Git churn is commit-count history for scanned paths, not execution frequency, latency, CPU, memory, user impact, or defect probability. Atlas classification may override the fallback directory grouping; fallback grouping itself is not canonical product ownership.

The same lexical declaration/exemption helpers may be reused by diff-time orphan-reference review, but report/review/health projections remain consumers of one source-neutral quality contract. Stable findings are routing signals, not automatic code deletion, architecture ownership, acceptance, or release decisions. No second scanner, findings ledger, task queue, product performance telemetry owner, or shipping runtime is introduced.

Stage-A contracts authored, not run: `CONTRACT-TDRP-CODE-HEALTH-METRICS-AREA-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-FILE-SHAPE-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-UNUSED-CANDIDATE-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-UNUSED-NEGATIVE-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-HOTSPOT-COMPOSITION-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-FINDING-THRESHOLDS-001`, and `INT-TDRP-CODE-HEALTH-METRICS-REPORT-001`.


### Repository report snapshot, finding lifecycle, and projection qualification

Qualify the repository-quality report layer as the source-neutral aggregation and finding-lifecycle boundary that consumes already-qualified repository evidence; it is not a second scanner, command orchestrator, findings store, task queue, product runtime, or release verdict. A report snapshot binds all contributing observations to one exact scanned tree/ref/commit and preserves the provenance and limitations of its inputs: approved-atlas classification when available with fallback area grouping otherwise; source metrics; source-derived build model/include cost; history churn/task cost; clone evidence; layering; unused candidates; tested-file projection; stale map/atlas observations; and helper inventory coverage. Missing or limited inputs stay limited in the snapshot and cannot be normalized into healthy zeroes.

Finding qualification preserves deterministic semantic identity by `kind:key`, the declared class mapping, source paths/area, title, raw supporting data, and the explicit impact × churn heat score used for prioritization. File-size, function/class, build-cost, clone, layering, unused, stale-map/atlas, untested-hot-area and helper-retention/documentation findings keep their own declared thresholds and evidence limitations. Priority class and score are routing/investigation signals only; they do not authorize deletion, refactoring, ownership changes, product acceptance, or release.

Lifecycle qualification covers historical merge semantics independently from publication. A currently observed finding is open and retains prior `first_seen`; a vanished prior open/ignored/unverified finding becomes resolved with `resolved_at`; a finding whose evidence kind is explicitly unavailable remains ignored when already ignored or becomes unverified rather than falsely resolving; already-resolved findings stay resolved; and resolved history is retained only inside the declared 180-day window. Stable identity must survive repeated scans and temporary evidence loss.

Report-projection qualification proves deterministic Markdown output from the snapshot plus merged ledger: totals and prior-run deltas, new/resolved summaries, bounded hotspot areas, top open findings, open counts by kind, build-cost projections, and helper-lifecycle coverage language. The renderer must preserve exact ref/commit identity and source notes and must state that retention findings are review signals rather than deletion proof. `health.py` remains the caller that owns exact-ref acquisition, daily idempotence, ignored overrides, atomic ledger publication/history append and temporary-vs-durable run placement.

Stage-A contracts authored, not run: `CONTRACT-TDRP-CODE-HEALTH-REPORT-SNAPSHOT-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-FINDINGS-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-MERGE-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-MAP-REFERENCES-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-MARKDOWN-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-UNAVAILABLE-EVIDENCE-001`, and `INT-TDRP-CODE-HEALTH-REPORT-HEALTH-COMPOSITION-001`.


### Repository diff-time review ratchet and move-only qualification

Qualify the repository review layer as a source-neutral, change-scoped quality ratchet over one explicit base and one staged/working-tree/head subject. It is a read-only review instrument, not a product runtime, second scanner, findings ledger, task queue, source editor, configured-build substitute, or acceptance authority. Scope is the declared tracked C/C++ translation-unit/header set plus style/language names needed for reference-liveness checks; untracked in-scope files count as added for working-tree review, while staged review reconstructs the staged subject exactly.

Severity qualification is part of the contract. A new in-scope file that starts above the shared file-size limit is an error. A newly introduced include crossing between CMake targets is an error when no prior crossing exists; another crossing across an already-violated target pair is a warning. Growth of an already-oversized file beyond the 20-line tolerance and added code copied from existing source are warnings normally and errors under strict review. Growth of an already-large class, material header rebuild-cost increase, removed-definition residual references, and last-use orphan candidates remain review warnings. Strictness must not silently escalate unrelated warning classes or downgrade hard errors.

Reference-liveness qualification distinguishes two conservative cases. A removed definition is stranded only when no definition remains and surviving mentions are relevant to that symbol; platform files are included even when the current host would not compile them, and method-name collisions outside files naming the owning class are not treated as uses. A removed last use is orphaned only when remaining mentions are declarations/definitions; C++ declarations, constructors/initializers, styles and language keys use their declared lexical rules, numeric literal suffixes are not identifiers, exemptions remain honored, and a complete deletion or a moved definition is clean rather than a warning.

Clone/include/build-cost qualification composes existing evidence owners rather than replacing them. Added hunks are checked through the existing clone copied-from detector with stable source provenance. Fresh includes are resolved through the existing include graph and source-derived CMake target model. Header cost warnings use only source-derived closure/reach and the declared `BUILD_SHARE=0.002` threshold over the include-cost proxy; they are not wall-clock compiler or configured-build proof. Shared file/class limits come from the existing report policy.

Move-only qualification is a separate proof surface. Every substantive removed line must reappear, every substantive added line must be a removed line or declared scaffolding/declaration, and statements moved within a function must preserve source order. Whole functions/groups may be reordered only at declared boundaries. Default arguments and selected declaration prefixes may normalize for declaration/definition moves, but changed logic, changed initializer values, lost lines, novel substantive lines, or out-of-order moved statements produce `NOT_MOVE_ONLY`. Staged, worktree/untracked and explicit base/head subjects retain distinct provenance.

`health.py` remains the command owner: `check` materializes the requested subject, renders Errors and Warnings, and returns non-zero only when the error set is non-empty; `moves` renders the move proof and returns success only for `MOVE_ONLY`. Review findings are change evidence for ordinary repository review/task governance and do not mutate the durable quality ledger by themselves.

Stage-A contracts authored, not run: `CONTRACT-TDRP-CODE-HEALTH-REVIEW-SCOPE-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-SEVERITY-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-ORPHAN-REFERENCE-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-CLONE-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-INCLUDE-LAYERING-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-BUILD-COST-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-CLASS-GROWTH-001`, `CONTRACT-TDRP-CODE-HEALTH-MOVE-ONLY-001`, and `INT-TDRP-CODE-HEALTH-REVIEW-HEALTH-COMPOSITION-001`.

### Repository source authority acquisition and filtering qualification

Qualify source acquisition as a source-neutral repository-authority primitive used by the existing quality-health command owner. It is not a product runtime, source editor, configured-build authority, second scanner, findings store, ownership graph, or task queue. Every result is either an immutable Git-object snapshot resolved from an explicit ref or a mutable developer working-tree snapshot with that distinction preserved in provenance.

Scope qualification is deterministic. The canonical Telegram quality scan admits the language catalog at `Telegram/Resources/langs/lang.strings`; source/build text under `Telegram/SourceFiles/**`, `Telegram/lib_*/**`, `Telegram/cmake/**`, and `Telegram/CMakeLists.txt`; declared C/C++/Objective-C++ header/translation-unit/style/CMake extensions; nested `CMakeLists.txt`; and `AGENTS.md` only when it is under `Telegram/SourceFiles/**`. Paths containing `ThirdParty` or `.git` are excluded. Working-tree traversal starts only from `SourceFiles`, `cmake`, `CMakeLists.txt`, and top-level `lib_*` entries, so a broader checkout walk must not silently enlarge the scan contract.

Exact-ref qualification first resolves `ref^{commit}` to an immutable commit, recursively enumerates the tracked `Telegram` tree, retains eligible parent-repository blobs by exact object ID, records every top-level `Telegram/lib_*` gitlink and its pinned commit, and reads a pinned submodule tree only from that exact object when the object is locally available. A missing local submodule object is explicit incomplete coverage with the path and pinned OID retained; it is never proof that the subtree has no source and cannot support a complete-scan or acceptance claim. Git/subprocess failure propagates as non-passing evidence.

Working-tree qualification records the current `HEAD` plus whether `git status --porcelain -- Telegram` reports local changes, reads the current eligible checkout files, records current `lib_*` submodule status, and labels the subject `WORKTREE`. It is developer evidence only: it may support review/check commands but cannot mutate canonical durable findings merely because `HEAD` is known. A concurrent checkout/index/submodule change makes previously gathered working-tree evidence stale.

Blob/text qualification preserves Git authority. Exact-ref text comes from `cat-file --batch` for the enumerated object IDs. NUL-bearing content is not treated as text; UTF-8 is preferred with one-byte Latin-1 fallback, while working-tree CRLF is normalized to LF to match Git diff/storage semantics. Enumeration and decoded-file coverage remain separate facts: an eligible tracked blob that cannot be represented as text must remain an explicit coverage limitation for any completeness-dependent claim rather than disappearing into a clean result.

The `SourceTree` carrier owns only source provenance and decoded text: repository, file map, resolved commit, requested ref/subject kind, notes, scan roots, gitlink pins, line-count lookup, and source-root classification. It owns no mutation, durable finding, approval, task insertion, release verdict, or product state. The existing quality-health command composes this carrier; report/metrics/include/clone/review/harness/atlas/history consumers retain their already-qualified semantics and must not reinterpret a mutable or incomplete source subject as exact repository authority.

Stage-A contracts authored, not run: `CONTRACT-TDRP-CODE-HEALTH-SOURCE-SCOPE-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-EXACT-REF-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-WORKTREE-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-GITLINK-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-DECODE-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-GIT-FAILURE-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-PROVENANCE-001`, and `INT-TDRP-CODE-HEALTH-SOURCE-CONSUMER-COMPOSITION-001`.

### Repository task/process authority qualification

Qualify checkout-level development scheduling as repository process governance rather than a shipping runtime. The authority surface has one canonical task state, one ownership truth and one explicitly frozen invocation scope. Host adapters may invoke the scheduler but may not redefine task state, ownership, dependency, split, consolidation or acceptance semantics.

Batch qualification begins from a fresh canonical queue/repository snapshot and an explicit specification. Existing checkout-owned active or split-required state is exclusive and must be resolved first. Ordinary later queue additions do not join the frozen batch. Only ordered split replacements and explicitly routed follow-ups caused by an in-scope result may rewrite/grow the invocation-local batch, with the startup selection retained as history. A missing specification is read-only/report-only and must not mutate shared state.

Task-state qualification preserves a small authoritative lifecycle and single ownership boundary. Starting shared work atomically assigns and activates exactly one task; receipts and batch files are never reservations; another owner's unfinished task is not stealable; interrupted local phase state is resumable rather than a reason to publish a block. A performer owns one task only and returns at an approved, genuinely blocked, split-required, or explicit hard-stop boundary. Executable validation and acceptance remain separate from source-level implementation state.

Source-lineage qualification runs before a new start/retry/pre-Phase-1 resume and treats approved prerequisites as shipped code lineage, not merely queue readiness. A lineage mismatch cannot be hidden by creating branch-sync/integration queue work. Before task execution, source preparation must preserve local changes, honor exact recorded gitlink targets, and refuse unsafe worktree overlap; mutable or unavailable source state remains non-passing authority evidence.

Split qualification is an atomic routing transaction. The split source must already be canonically split-required; replacements are independently shippable/testable; all original acceptance/dependencies/links are accounted; retained implementation has at most one first carrier; live dependencies/project links are rewritten; and the source retires to durable lineage without duplicate runnable state. Unsafe retained-work transfer or semantic publication conflict preserves the source state and stops.

Discovery routing and consolidation are separate from task execution. Routing may create only independently actionable follow-ups attributable to the current result and records durable source lineage. Consolidation may merge only compatible unclaimed `todo` work within the same project and frozen-batch-membership class, never approved/claimed/active history, and must preserve every criterion, dependency, input and old-id alias. A pre-commit race produces no partial mutation; a published no-merge marker is still a durable no-repeat boundary. Publication uses compare-and-swap semantics and never force-pushes semantic conflicts.

Stage-A contracts authored, not run: `CONTRACT-TDRP-CONTINUE-BATCH-FREEZE-001`, `CONTRACT-TDRP-CONTINUE-TASK-STATE-AUTHORITY-001`, `CONTRACT-TDRP-CONTINUE-OWNERSHIP-001`, `CONTRACT-TDRP-CONTINUE-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-CONTINUE-SPLIT-TRANSACTION-001`, `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-001`, `CONTRACT-TDRP-CONTINUE-RECOVERY-001`, and `INT-TDRP-CONTINUE-HOST-ADAPTER-COMPOSITION-001`.

OpenAI host-adapter qualification treats `.agents/skills/continue/agents/openai.yaml` as invocation metadata only. The interface may expose discoverability copy and a default prompt, but it cannot redefine canonical task state, ownership, split/consolidation, dependency or acceptance semantics. A passing default-prompt contract invokes `$continue`, places already-active work first, freezes exactly the caller's batch specification behind it, and drains one task at a time. It must not silently imply all-pending work, inbox processing, ownership, verification or acceptance.

Additional Stage-A contracts authored, not run: `CONTRACT-TDRP-CONTINUE-OPENAI-HOST-METADATA-001` and `INT-TDRP-CONTINUE-OPENAI-HOST-INVOCATION-001`.


### Process Inbox acquisition/routing qualification

Acquisition qualification binds one intake to the source checkout, checkout tag, canonical AI repository, isolated inbox publication worktree and digest of the immutable transaction snapshot. A new snapshot requires clean canonical/inbox worktrees and no unpublished inbox commit; a single matching active transaction resumes instead of duplicating work. Empty input, invalid identity, unsafe worktree state or ambiguous active transactions are non-mutation outcomes. The task-execution slot must remain outside the intake transaction.

Routing qualification reads only the snapshot and targeted source/project/task context. It resolves durable superseded/split history to live successors, treats retired dated paths as occupied, preserves source-project continuity by default, and requires concrete checkout-without-origin evidence before routing derived work to another project or standalone. Branch-integration-only requests are receipt-only. A proposed task owns one independently shippable/testable boundary with a direct acceptance oracle; inseparable changes remain one task and independently useful boundaries split. Q0/Q1/Q2 pre-sized code-health requests remain one standalone task.

Task-state/source-lineage qualification requires dated collision-safe task ids; self-contained `task.md`; copied durable inputs; and exact `state.yaml` semantics: `todo`, `implement`, no claim/lease/phase ownership, no completion `model`, and dependencies naming only live canonical tasks. Dependencies preserve code lineage and readiness; no backport/cherry-pick/rebase/merge companion task may be manufactured to satisfy lineage. Consolidation aliases and split records resolve cycle-safely for deduplication/dependencies, while later performer `split-required` remains a scheduler-owned replacement transaction rather than inbox-owned queue mutation.

Publication/recovery qualification requires a digest-bearing receipt accounting for every request/input, an explicit changed-path allowlist, and exclusion of raw inbox paths, videos/full extraction, portable accounts, credentials, browser profiles, complete run/build logs and persisted commit hashes. Publication may retry an ordinary non-fast-forward race only after refresh/rebase; semantic conflicts and remote failures preserve the commit/transaction and never force-push. Finalize succeeds only after canonical master contains the exact receipt/digest and clears live input only when its digest is unchanged; otherwise the human edit remains. Abort is legal only before a tracked inbox commit exists.

Stage-A contracts authored, not run: `CONTRACT-TDRP-PROCESS-INBOX-ACQUISITION-SNAPSHOT-001`, `CONTRACT-TDRP-PROCESS-INBOX-ROUTING-LINEAGE-001`, `CONTRACT-TDRP-PROCESS-INBOX-TASK-STATE-SCHEMA-001`, `CONTRACT-TDRP-PROCESS-INBOX-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-PROCESS-INBOX-SPLIT-CONSOLIDATION-INTEROP-001`, `INT-TDRP-PROCESS-INBOX-PUBLICATION-FINALIZE-001`, and `FAULT-TDRP-PROCESS-INBOX-RECOVERY-001`.

### Continue consolidation transaction qualification

The fixed-U consolidation reference is qualified as a source-neutral repository-process transaction, not as a second queue or product runtime. Eligibility must fail closed: only pristine unclaimed `todo` work may be considered; project lineage and frozen-batch membership are hard partitions; approved, claimed, active, blocked/non-mergeable, already-started, or class-isolated work cannot be retired. Reverse dependencies are part of the decision: an active/claimed/non-mergeable unfinished dependent vetoes a source candidate, while eligible `todo` dependents may be rewritten only atomically. Replacement simulation must reject direct/transitive self-dependency and every dependency cycle.

Merge qualification requires a material shared context/implementation/measurement setup and one coherent normal implementation pass. It is not satisfied by same-project membership or by concatenating unrelated tasks. Every source acceptance criterion, prerequisite, dependency, input, fixture/safety fact, and source-specific scope boundary remains traceable. `NO_MERGE` is a successful optimization outcome only when it publishes the durable completion/no-repeat boundary and changes no unrelated queue state.

Retirement/alias qualification preserves source identity. The old task retains its specification and inputs, loses runnable state, gains an exact old-to-new alias, receipt, type/project identity, and retained-content digest; alias resolution must be cycle-safe and end at live canonical work. A late edit that changes the retained digest is non-passing. Replacement work must be pristine unclaimed `todo`, preserve type/project, name the consolidation receipt, and have at least two retired sources per merged target.

Publication qualification requires a last-moment reread, explicit changed-path scope, exact mapping/receipt coverage, no stale live dependency or project link to retired ids, full dependency/alias/split graph validation, and a single compare-and-swap boundary. Pre-commit drift is `RACED`: no tracked mutation and the pending marker remains. Once a consolidation commit exists, semantic/remote publication failure is `BLOCKED`: preserve the commit for generic publish/recovery, revalidate after fetch/rebase, and never force-push. The generic publish path must recognize consolidation commits and invoke the same validator.

Additional Stage-A contracts authored, not run: `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-ELIGIBILITY-001`, `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-MERGE-NO-MERGE-001`, `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-ALIAS-FINGERPRINT-001`, `INT-TDRP-CONTINUE-CONSOLIDATION-PUBLISH-001`, and `FAULT-TDRP-CONTINUE-CONSOLIDATION-RECOVERY-001`.

### Continue split-required transaction qualification

The fixed-U split reference is qualified as a scheduler-owned source-neutral repository routing transaction. Eligibility is fail-closed: the source must already be canonical `split-required` work owned by the current checkout, with its complete task context, split proposal/result and carried-work seal readable. The routing worker may inspect retained source state but may not edit, reset, stash, commit, build or test the product source. Replacements are created only by this transaction and there must be at least two unique `type: implement` successors.

Replacement qualification requires each successor to own one independently shippable and independently testable outcome with a direct oracle. Source project lineage is preserved unless independence from that project is proved; source dependencies stay attached wherever their shipped code is still required; successor ordering follows code/behavior prerequisites rather than prose order. Every original acceptance criterion and retained source path must map to a successor, and an integration successor is allowed only for behavior not already proved by component successors.

Retained-work qualification is exclusive. `implementation: retained` requires exactly one first implementation carrier whose boundary coherently owns the complete retained diff. Before split publication every successor is pristine routed `todo`; publication transfers the carrier's checkout ownership metadata, owned-path scope and `carried_from` lineage without marking it in-progress or approved. Successors that consume retained code depend on the carrier. The later canonical Start revalidates the worktree seal and then transfers the source refs; no-retained-work splits have no carrier and leave all successors unclaimed. Any seal mismatch or inability to name one coherent carrier is non-passing and must preserve the source implementation.

Dependency/publication qualification is atomic. Every live dependency and project link to the retired source is rewritten to the exact successor set in the same transaction; the source keeps its retained content and loses only runnable state in favor of a multi-target split record with ordered successors, receipt, optional carrier and retained-content digest. Passing publication proves explicit changed-path scope, receipt coverage, pristine successor state, no stale source dependency/project link, complete dependency/split/superseded graph integrity, retained digest/worktree-seal integrity and compare-and-swap publication. Ordinary publication races may retry only after refresh and revalidation. A semantic conflict, unavailable remote, changed seal or incoherent carrier preserves recoverable state and is a hard stop; force publication is forbidden.

Additional Stage-A contracts authored, not run: `CONTRACT-TDRP-CONTINUE-SPLIT-ELIGIBILITY-001`, `CONTRACT-TDRP-CONTINUE-SPLIT-REPLACEMENT-001`, `CONTRACT-TDRP-CONTINUE-SPLIT-CARRIER-HANDOFF-001`, `INT-TDRP-CONTINUE-SPLIT-PUBLISH-001`, and `FAULT-TDRP-CONTINUE-SPLIT-RECOVERY-001`.



### Repository dependency advisory qualification

Qualify dependency monitoring independently from product update/install behavior. Positive cases bind a dependency snapshot to one freshly resolved exact source commit, enumerate the complete declared dependency-input union including recursive gitlinks/submodules and configuration-specific build/package/runtime sources, and preserve per-dependency consumed series/platform/configuration plus immutable source/package identity where available. Candidate-navigation output must not substitute for the full inventory. A failed fetch, missing required manifest, unread recursive dependency, unresolved distro/package revision, stale advisory source or other incomplete range yields explicit partial/unavailable coverage and cannot produce a clean “all current” result.

Release/advisory qualification separates urgency from adoption readiness. Cases cover ordinary stable patch/minor/major classification without forcing SemVer onto unsupported schemes; release-trust assessment with publication date, tag/peeled commit or package/artifact identity, source form, trust/regression/unknown checks and next-review condition; and a default observation hold for routine fresh releases without converting elapsed time into proof of safety. Negative controls reject treating absence from GHSA/OSV/CVE as safety, a signature/checksum as benign-code proof, build success as supply-chain proof, or a mutable/latest tag as immutable evidence.

Security/fork qualification covers applicability before urgency, patched-fork/copy provenance, and backport evidence. A reported vulnerability must distinguish confirmed applicable-and-missing, already present/backported, not applicable, and needs-investigation states. Missing ancestry, a matching subject, a patch that applies, or a version-only match cannot prove a missing fix. For copied/patched dependencies, qualification preserves consumed fork pin, fork head, original upstream/import mapping, reviewed unimported range, fix prerequisites, local patch conflicts and platform scope. Confirmed missing fixes carry a reviewed update/backport route; hold/skip/track decisions do not silently close an unresolved security finding.

Lifecycle/report qualification preserves stable finding identity, first/last seen, pending state across failures, prior run history, per-dependency check times, coverage state, decision/reason, source links and fork evidence. Publication writes run-local report/findings first and atomically replaces only the current latest projection. Both “Most pressing updates pending” and “Review someday” sections remain present, urgent unresolved items survive later partial scans, and resolved findings require new evidence rather than a revision change alone. Stage-A contracts are authored only and remain `not-run`.

Authored contracts for fixed-U dependency-watch closure: `CONTRACT-TDRP-DEPENDENCY-SNAPSHOT-COVERAGE-001`, `CONTRACT-TDRP-DEPENDENCY-RELEASE-TRUST-001`, `CONTRACT-TDRP-DEPENDENCY-SECURITY-FORK-001`, `CONTRACT-TDRP-DEPENDENCY-FINDING-LIFECYCLE-001`, `FAULT-TDRP-DEPENDENCY-PARTIAL-RECOVERY-001`, and `INT-TDRP-DEPENDENCY-ADVISORY-OWNER-001`.


OpenAI Dependency Watch host-adapter qualification treats `.agents/skills/dependency-watch/agents/openai.yaml` as invocation/discovery metadata only. The interface may expose `display_name`, `short_description`, and a default prompt, but it cannot become dependency snapshot, finding, release-trust, backport, mutation, build/test, updater, release, deployment, or acceptance authority. A passing default-prompt contract invokes `$dependency-watch`, requires an audit over freshly fetched `origin/dev`, and preserves report priority by presenting pressing pending updates before the review-someday list. It must not imply complete coverage, safety, adoption readiness, or execution of an update/backport.

Additional Stage-A contracts authored, not run: `CONTRACT-TDRP-DEPENDENCY-OPENAI-HOST-METADATA-001` and `INT-TDRP-DEPENDENCY-OPENAI-HOST-INVOCATION-001`.


Patched-fork/copied-source qualification additionally separates three identities that must never be collapsed: the exact fork revision consumed by the inspected Telegram snapshot, the fork's current upstream branch, and the original project's imported upstream revision. For `tg_owt` and `tg_angle`, a passing case rebuilds the baseline from the exact consumed pins and import provenance rather than treating repository names or remembered Chromium milestones as fixed baselines. Import subjects such as WebRTC M123 or ANGLE M115 are mapping clues only until tied to an exact original revision by provenance files, commit-position/Change-Id trailers, DEPS/version metadata, nested gitlinks, or source equivalence. When copied history has no shared ancestry, use a verified import mapping; a failed merge-base, raw commit-count subtraction, or approximate milestone must not be used as proof of divergence.

Fixed-U fixtures preserve the current consumed facts without converting them into future baselines: `Telegram/build/prepare/prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b`, `Telegram/build/docker/centos_env/Dockerfile@3fea9e6866e0ea6351f8a13ebb825ebf49cadfc2`, and `snap/snapcraft.yaml@a181d828f25d901303f7221824ac30b0ff5e773a` consume `desktop-app/tg_owt@07fb35bc33b3e79125a1beab7690e830f183f5d8`; fixed-U prepare consumes `desktop-app/tg_angle@9fd174a91e6155d1006c5f427366b23006f5d899` on Windows. Fork history contains a WebRTC import anchor `5b6a5c00017c115f6a54b90b109354abc1444a01` ("M123+patches") and an ANGLE import anchor `3e2cbd93cb60a4f677af68bac4e3849176f688fa` ("M115"); the consumed pins are descendants of those anchors, but those milestone labels do not establish the exact original upstream commits. Platform applicability is derived from the consuming build selection: the fixed-U prepare path builds `tg_angle` only on Windows and explicitly wires the static ANGLE library into the Qt<6 branch, so stage presence alone must not be treated as proof of exposure in every Windows/Qt configuration.

Missing-fix/backport qualification requires the actual fixing diff and prerequisites plus evidence for introduced/affected versions, source presence, enabled backend/feature, build flags and reachable use. Ancestry is used when meaningful; rewritten/cherry-picked fixes require equivalent-code or patch-ID evidence. Missing ancestry does not prove a missing fix, and matching subjects or a patch that applies do not prove that a fix is present or correct. Every candidate is classified as confirmed applicable-and-missing, already present/backported, not applicable, or needs-investigation. A confirmed missing fix records the original fix and prerequisites, affected fork paths, intended destination (fork or shared patches repository), likely conflicts with local changes, platform scope, and focused Stage-B regression/Debug validation. The advisory owner may propose that route but does not apply the backport.

Additional Stage-A contracts authored, not run: `CONTRACT-TDRP-DEPENDENCY-FORK-IMPORT-HISTORY-001` and `CONTRACT-TDRP-DEPENDENCY-FORK-FIX-APPLICABILITY-001`.



Release-trust-specific qualification binds each actionable candidate to publication date, canonical source, exact tag object plus peeled commit (or exact package revision/artifact digest), and the source form actually consumed by the recipe. Mutable tags/assets are compared with the previous observation; a changed tag or asset requires an explained provenance transition rather than silent acceptance. Trust evidence must distinguish primary release metadata from genuinely independent validation: reposts are not independent, and a checksum published on the same compromised release surface cannot establish independent integrity.

Positive trust cases inspect relevant source and packaging/build deltas, including install hooks, generated scripts, binary/test blobs, downloads, unexpected dependency changes, publisher/signer/repository ownership changes, and release-process changes. Signature/attestation/reproducibility evidence is recorded only when verification actually succeeds and is compared with established signer history or separately published keys where available. A verified signature establishes attribution/integrity only within its trust assumptions; it does not prove benign code, while an unsigned release is not automatically compromised. Archive-only generated files must be assessed when the recipe consumes archives, and known malicious releases must not be downloaded or executed by the advisory workflow.

Decision qualification covers the full `update | backport | hold | skip | track` state machine. Routine bugfix releases use seven days after publication as the default observation window unless configured otherwise, but elapsed time never clears a trust or compatibility concern. A confirmed applicable security fix, especially under active exploitation, may justify immediate reviewed adoption or minimal backport with the tradeoff recorded; an unverified CVE/version match does not establish that urgency. Holds retain the missing evidence or release condition and a concrete review date/condition; skips name the exact withdrawn/compromised/incompatible/superseded candidate and alternative rather than blacklisting the project; backports receive the same trust review as releases. Persisted decision evidence includes `decision`, `decision_reason`, `release_published_at`, `source_identity`, `trust_checks`, `regression_checks`, `unknown_checks`, and `review_after` or `review_condition`.

The XZ incident is a standing malicious-release regression case, not a permanent project ban. The reference names compromised 5.6.0/5.6.1 release tarballs and requires later releases and old-branch backports to be reassessed from current evidence. Its historical “initial Telegram snapshot” example (prepare 5.4.5 / Docker 5.8.1) must not be inherited as current fixed-U state: current fixed-U prepare clones the XZ v5.4 branch and checks out immutable commit `05af863c3166cddeb4ab935829b14e995c8346b4`, while the current CentOS Docker recipe clones tag `v5.8.4`. This fixture specifically tests re-establishing identity from each inspected snapshot rather than reusing an old clearance.

Additional Stage-A contracts authored, not run: `CONTRACT-TDRP-DEPENDENCY-RELEASE-PROVENANCE-001` and `CONTRACT-TDRP-DEPENDENCY-RELEASE-DECISION-001`.


### Dependency Watch snapshot/compare helper qualification

The fixed-U helper `.agents/skills/dependency-watch/scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38` is an implementation support surface of the existing repository dependency-advisory owner, not a second monitor or product/runtime owner. Qualification binds one helper invocation to one repository top level, one `origin/dev` snapshot identity, one Git common-directory report root, and one run-local immutable source copy. The helper may update only the `refs/remotes/origin/dev` remote-tracking ref during a fresh snapshot; it must not switch HEAD, edit the working tree, recurse/fetch live submodules, or treat an offline remote-tracking ref as fresh.

Snapshot/fetch cases require canonicalizing the supplied path with `rev-parse --show-toplevel`; online mode performs exactly the no-tags, no-submodule-recursion fetch of `origin` head `dev` into `refs/remotes/origin/dev`, then timestamps the successful fetch and resolves `refs/remotes/origin/dev^{commit}` once. All selected inputs and gitlinks are enumerated from `git ls-tree -rz` at that exact commit and file bytes come from `git cat-file blob <oid>`, not from checkout contents. Required manifests are `.gitmodules`, `Telegram/build/prepare/prepare.py`, `Telegram/build/qt_version.py`, `Telegram/build/docker/centos_env/Dockerfile`, and `snap/snapcraft.yaml`; each must exist as mode `100644` or `100755` and remain readable UTF-8 non-NUL text. Selection also includes all files under `Telegram/build/`, `snap/`, and `.github/`, plus any basename in the helper manifest-name set. Gitlinks are captured separately and remain recursive-review inputs rather than proof of closed dependency coverage.

Storage qualification resolves `rev-parse --git-common-dir`; a relative common dir is resolved from the canonical top level. The only helper-owned root is `<common-git-dir>/dependency-watch`, with preserved `runs/` history and a timestamp-prefixed run directory. Exact selected regular source blobs are copied beneath `run/sources/<repository path>`; non-regular blobs, non-UTF-8 text and NUL-bearing content are recorded as skipped. A linked worktree therefore converges on the primary repository's common dependency-watch storage rather than creating a worktree-local findings owner.

Candidate extraction is navigation-only. For each selected readable source line the helper records path, one-based line and stripped text when the line matches its source/download/package/toolchain heuristic (URLs; git clone/checkout/fetch/submodule; source tag/branch/commit/checksum keys; Snap package/snap keys; FROM/QT/rustToolchain; pip install; dnf/apt/apt-get/pacman; or workflow `uses:`). A passing contract never upgrades that candidate list into complete dependency inventory or safety evidence.

Version comparison applies only when both strings fully match an optional `v`, `n`, or `openssl-` prefix followed by exactly three numeric components. Equal tuples classify `same`; numerically lower candidates classify `older`; higher candidates classify `major`, `minor`, or `patch` by the first changed component. Prereleases, hashes, milestones, legacy-letter OpenSSL versions, four-component versions and every unsupported scheme classify `unknown` and defer to upstream-native policy rather than forced SemVer.

Freshness/failure cases are fail-closed. `--no-fetch` is test/offline mode only and must emit `fresh_fetch=false` while retaining `audit_status=snapshot-only`; it cannot support a current-clean claim. A Git command failure, 180-second helper timeout, OS error, missing required manifest, or unread required manifest makes the CLI return failure and print that no fresh audit is available. Fetch failure occurs before helper report-root creation; later decode/readability failure may leave a run-local partial source directory but cannot write a successful `snapshot.json`/`candidates.json` or publish freshness. Successful snapshot JSON records schema version, repository/ref/exact commit, UTC check time, freshness, snapshot-only audit status, storage/run identities, selected files, gitlinks, skipped inputs and an explicit note that dependency/security review remains pending.

Direct fixed-U workflow caller is `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`, which invokes the helper's `snapshot` and `compare` CLI. Direct test consumer is `.agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea`, whose `VersionTests` and `SnapshotTests` cover stable ordering/prefixes, remote fetch without working-tree/HEAD mutation, stale-ref refusal after fetch failure, missing required manifests, offline freshness marking and common-dir storage for linked worktrees.

Additional Stage-A contracts authored, not run: `CONTRACT-TDRP-DEPENDENCY-WATCH-SNAPSHOT-001`, `CONTRACT-TDRP-DEPENDENCY-WATCH-STORAGE-001`, `CONTRACT-TDRP-DEPENDENCY-WATCH-CANDIDATES-001`, `CONTRACT-TDRP-DEPENDENCY-WATCH-VERSION-COMPARE-001`, and `FAULT-TDRP-DEPENDENCY-WATCH-FRESHNESS-001`.


### Dependency Watch upstream regression-fixture qualification

The fixed-U regression source `.agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea` (4,347 bytes / 99 logical lines, recursive-tree index 64) is test-only evidence for the existing repository dependency-advisory owner. It imports the exact helper module and is executable through Python `unittest` discovery / its own `unittest.main()` entrypoint; the Dependency Watch skill does not invoke this test file as production workflow logic, and the test owns no durable advisory state.

`VersionTests.test_stable_ordering_and_prefixes` freezes the helper's stable-version grammar and numeric ordering. Positive controls require `v1.23.5 -> v1.23.10 = patch`, `n6.1.6 -> n6.2.0 = minor`, `openssl-3.2.1 -> openssl-4.0.0 = major`, `1.2.9 -> 1.2.8 = older`, and `v1.2.3 -> 1.2.3 = same`. Negative controls require prerelease `v1.2.4-rc1`, commit hashes, Chromium-style milestones, four-component versions, and legacy-letter OpenSSL versions to classify `unknown` rather than being coerced into stable SemVer.

`SnapshotTests` builds a disposable `dev` remote containing every required manifest, commits it, and clones a checkout. Its fresh-fetch regression then changes the remote manifest, stages a synthetic `160000` gitlink, makes an unfinished local checkout edit, records checkout status and HEAD, and calls `watch.snapshot`. The oracle requires the saved source copy to equal the remote pinned bytes, `fresh_fetch=true`, the resolved snapshot commit to equal the remote HEAD, the gitlink list to contain the exact path/object pair, and both checkout status and HEAD to remain byte-for-byte unchanged. It also requires no helper-owned `latest.md` projection at the storage root.

Freshness failure is fail-closed: after origin is pointed at a missing path, `watch.snapshot` must raise the Git nonzero-exit error and must not create checkout `.git/dependency-watch` storage, so an older remote-tracking ref cannot be presented as a fresh result. Required-manifest regression deletes and commits `snap/snapcraft.yaml` and requires `ValueError`. Offline regression calls `snapshot(fetch=False)` and requires `fresh_fetch=false` plus `audit_status=snapshot-only`. Linked-worktree regression creates a detached worktree, snapshots offline, and requires its `storage_root` to resolve to the primary checkout's shared `.git/dependency-watch` common storage.

These upstream regressions are evidence-bearing oracles, not Stage-A execution permission. Fabushi keeps them under the existing Test Strategy/Evidence owner; no Telegram-specific Python test harness, second monitor, product runtime owner, updater owner, or platform-core owner is introduced.

Additional Stage-A contracts authored, not run: `CONTRACT-TDRP-DEPENDENCY-WATCH-UPSTREAM-VERSION-REGRESSION-001` and `CONTRACT-TDRP-DEPENDENCY-WATCH-UPSTREAM-SNAPSHOT-REGRESSION-001`.

### Repository single-task performer qualification

Qualify the fixed-U performer as the source-neutral executor for one already selected repository-development task, composed under the existing repository task/process authority. The performer must not become a second queue, inbox router, split scheduler, consolidation engine, product Tasks/Automations runtime, or Agent/Coordinator/Host/Runner owner. Host entrypoints are invocation adapters only.

Task-state qualification starts from one canonical task id and fresh canonical status/ownership. Positive cases cover eligible todo -> canonical Start, owned blocked -> Retry while retaining prior artifacts, and owned in-progress -> resume without a duplicate state commit. Negative cases cover already-approved and split-required terminal reports, unfinished dependencies, another checkout's ownership, and source-lineage mismatch. Soft batch marks, receipts, local phase files and local recovery refs cannot establish ownership. Source prerequisites are qualified both as task dependencies and as source-lineage facts before the first implementation phase.

Phase/resume qualification requires context and an assessed implementation/evidence plan before source edits, explicit owned-path sets for each implementation unit, durable task-local phase artifacts, and restart from the first incomplete validated boundary. The assessment may reject an undersized/over-engineered plan or emit `split-required` before edits; later review convergence may reach the same boundary while preserving retained implementation. The performer publishes that task result but leaves replacement creation, dependency rewrite and retained-work carrier transfer to the scheduler.

Review qualification preserves independence and carry-forward. The mandatory general reviewer covers the complete retained change and evidence design; the lifetime, reuse, structure, performance and security lenses independently prove non-applicability or review their owned mechanisms, with harness-specific narrowing only where the current policy permits it. A fix invalidates only the exact findings, invariants, specialists, validations and evidence checks it changes. Focused re-review must enumerate carried and invalidated approvals. Unresolved findings cannot be approved because a round cap was reached; convergence decisions are explicit `CONTINUE | REPLAN_CURRENT | RESCOPE_REQUIRED | HARD_STOP` dispositions and never themselves approve code.

Mechanical-lane qualification is limited to code-health Q0 deletion, Q1 move and Q2 merge-identical-copies tasks whose behavior-preserving class remains provable. Q0 removes no-longer-referenced code and its task-owned remnants; Q1 moves code without semantic edits beyond necessary declarations/includes/build registration; Q2 replaces genuinely identical copies with one shared implementation while preserving every difference as data. A candidate leaving the allowed paths or behavior bounds is dropped/routed rather than promoted through the mechanical proof. The lane still preserves source-lineage, owned-path, review, commit and final-publication authority.

Evidence qualification uses the existing task evidence state machine: selected checks must bind claim, changed surface, direct instrument, oracle, window, control, falsifier and durable evidence before execution. `APPROVED` requires positive evidence for every selected check; `TEST_FLAW` repairs only the evidence path and moves along the directness ladder; `IMPL_BUG` returns to implementation plus focused review and invalidated checks; `UNRECOVERABLE` requires an independently established unsafe/unreachable subject rather than a run cap. Prior passing evidence carries forward unless changed code, fixture state or oracle invalidates it. An already-satisfied outcome requires direct proof without manufacturing a source edit.

Publication qualification distinguishes retained implementation from task acceptance. The final result identifies task status/outcome, retained implementation, split proposal when applicable, attempts, touched paths, evidence, exact unverified behavior and checkout disposition. Only canonical approved, genuine blocked or split-required publication closes the performer boundary. Interruption/tool loss leaves resumable in-progress state. Publication is non-force compare-and-swap; semantic conflicts preserve recoverable state.

FMEP-001 remains the execution-timing authority for this migration: Stage A authors these contracts and source wiring only. No build, test, validator, evidence campaign, runtime check or acceptance is executed here, so every new performer contract remains `validation_status=not-run` and `accepted=false`.

Stage-A contracts authored, not run: `CONTRACT-TDRP-PERFORM-TASK-STATE-AUTHORITY-001`, `CONTRACT-TDRP-PERFORM-TASK-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-PERFORM-TASK-PHASE-RESUME-001`, `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-LANE-001`, `CONTRACT-TDRP-PERFORM-TASK-REVIEW-CONVERGENCE-001`, `CONTRACT-TDRP-PERFORM-TASK-EVIDENCE-RECOVERY-001`, `CONTRACT-TDRP-PERFORM-TASK-PUBLICATION-001`, and `INT-TDRP-PERFORM-TASK-HOST-COMPOSITION-001`.


#### OpenAI performer host-adapter qualification

Qualify `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a` as a declarative host adapter over the existing single-task performer. The exact metadata must expose `display_name = "Perform AI Task"`, describe implementation/resumption of one Telegram AI task, and use a default prompt that invokes `$perform-task` to implement, adaptively review and test, and publish one named ai-tdesktop task.

The adapter passes only when invocation preserves the canonical perform-task boundary: one already selected/named task, no queue or inbox ownership, no replacement-task creation, no split/consolidation routing, and no independent task/review/evidence/publication state. Claude and Grok command wrappers remain peer host adapters for the same shared skill; provider-specific syntax cannot change semantic ownership. FMEP-001 keeps executable qualification deferred.

Stage-A contracts authored, not run: `CONTRACT-TDRP-PERFORM-TASK-OPENAI-HOST-METADATA-001` and `INT-TDRP-PERFORM-TASK-OPENAI-HOST-INVOCATION-001`.


#### Repository performer UI-driver qualification

Qualify fixed-U `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259` as a UI-driver adapter inside the existing single-task performer evidence loop. It must not become a second performer, product Computer runtime, Telegram-specific quality owner, or acceptance store.

Selection cases cover deterministic/overlay-only operation and hybrid selection only when physical input/native window mechanics are themselves part of the acceptance claim. Negative controls reject hybrid selection justified only by broader coverage, internal state/text/geometry already observable deterministically, or a driver narrative without planned semantic assertions.

Capability qualification proves the active host exposes the approved driver mechanism, permissions are already satisfied, and the exact test app is distinguished from any release/user instance before the first state/action call. Hybrid operation revalidates exact app/process/start identity around every state/action boundary, refreshes accessibility/visual state before selecting ephemeral targets, and treats wrong/replaced subject, stale target, focus steal, local input interference, or user-stopped sessions as contaminated/test-evidence failures rather than product success.

Operation qualification requires one bounded predeclared action per ready step, deterministic task-owned setup, a stable semantic target when available, immediate post-action identity/state refresh, application-owned observation of the result, watchdog/terminal handling, and durable semantic plus accessibility/visual artifacts at decisive checkpoints. Exploration alone cannot assign a product verdict.

Safety qualification forbids unattended credential/secret entry, login/permission/security approval, account/privacy/network/OS changes, server-visible content mutation solely for evidence, external confirmation/link flows, payments, CAPTCHA/security-dialog approval, and unrelated-app control. If safe local fixtures cannot preserve the acceptance claim, record the precise unavailable/test-blocked condition through the ordinary recovery contract.

Fallback qualification preserves the oracle: automatic mode may use a prewritten equivalent deterministic fallback; explicitly required physical interaction cannot be silently downgraded. Locked graphical sessions are never driven at the lock screen and use complete in-process instrumentation when that host path is sufficient. Missing capability/permission is a test/environment fact, not an implementation defect.

Durable-evidence qualification preserves driver policy, capability result, host/driver identity, exact app/process identity, scenario/step, safety envelope, ordered action outcomes, decisive before/after accessibility state and tightly scoped visual captures, plus the application assertion/log paths that decide the claim. Evidence precedence is application assertion/crash state > semantic/in-process state and measured geometry > durable accessibility state > tight visual capture > narration. Stale target, wrong-window control, capability/permission failure, missing durable evidence, or ambiguous capture is `TEST_FLAW` or the precise test blocker according to recoverability.

Stage-A contracts authored, not run: `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-SELECTION-001`, `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-SAFETY-001`, `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-EVIDENCE-001`, and `FAULT-TDRP-PERFORM-TASK-UI-DRIVER-RECOVERY-001`.

### Repository performer — Q0/Q1/Q2 mechanical-lane qualification

Fixed-U source: `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef`, composed by perform-task `SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, `references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493`, code-health `SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f`, and `references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e`.

Authored/not-run contracts: `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-ELIGIBILITY-001` (Q0/Q1/Q2-only eligibility, exact-base recheck, stale/class-drift drop); `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-OWNED-EDIT-001` (owned-path allowlist and Q0/Q1/Q2 edit boundaries); `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-PROOF-DROP-001` (exact-revision build/health/class proof, invalidation after edit/drop, restore/drop on failure); `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-REVIEW-PUBLICATION-001` (one fresh full-diff review, one focused fix/resume maximum, distinct retained-change versus all-dropped publication). The existing `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-LANE-001` remains the umbrella contract. Stage A executes none of these.



### Repository performer phase-prompt qualification

Qualify fixed-U `.agents/skills/perform-task/references/phase-prompts.md@f62400b00933fe3757d90b809e456b200389c65c` as the exact phase-worker protocol inside the existing single-task performer, not as a second performer or scheduler. Direct fixed-U consumers are `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, `references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493`, `.grok/ai-workflow-adapter.md@db447f204316d3991f6d20db3d8f097fcc021154`, and `.agents/shared/cross-review/prompts.md@4e155d2bd7e647c12df7b008c279969bfd65e44d`.

Prompt/artifact qualification requires the exact pre-execution prompt log and post-execution result record, the compact leaf reply fields, unique non-delegating/non-committing workers, and source-defined artifact completion checks. Context/plan must remain read/plan-only; assessment either produces an assessed bounded plan plus `test-design.md`, rejects fast-path/approach sizing for a fresh Phase 1 rerun, or emits `split-required` before source edits. Implementation touches only the declared phase write set; a phase is complete only when its status/touched-path contract is reconciled. Pre-review validation remains the assessed direct check rather than a ceremonial broad build.

Worker/retry-resume qualification proves delegation becomes authoritative only after the first real spawn succeeds. An immediate host depth/capacity/policy rejection selects the declared same-session fallback. Once delegated, Phase 1/3/4/6 may retry one incomplete disposable leaf once after the original worker has stopped; a second failure returns a recoverable hard stop and must not silently rerun locally or publish a canonical Block. Host continuation may resume the saved general-review worker for synthesis, but provider syntax cannot transfer task ownership or publication authority to a leaf.

Review qualification covers independent initial general/lens reads, harness-specific narrowing only where declared, focused re-review after a fix, explicit carried versus invalidated approvals, and convergence dispositions `CONTINUE | REPLAN_CURRENT | RESCOPE_REQUIRED | HARD_STOP`. A convergence assessor cannot approve code. Evidence-recovery qualification preserves prior passing checks, records failed assumption/forbidden technique/new direct instrument/reachability/oracle independence, edits only the evidence path until a sound check proves `IMPL_BUG`, and treats campaign caps as assessment boundaries rather than success.

Stage-A contracts authored, not run: `CONTRACT-TDRP-PERFORM-TASK-PHASE-PROMPT-ARTIFACT-001`, `CONTRACT-TDRP-PERFORM-TASK-PHASE-ASSESSMENT-001`, `CONTRACT-TDRP-PERFORM-TASK-PHASE-REVIEW-FOCUS-001`, `FAULT-TDRP-PERFORM-TASK-PHASE-WORKER-RECOVERY-001`, and `FAULT-TDRP-PERFORM-TASK-EVIDENCE-CAMPAIGN-001`. Existing umbrella phase-resume, review-convergence and evidence-recovery contracts remain authoritative. No executable qualification is run in Stage A.
### Repository performer end-to-end pipeline qualification

Qualify fixed-U `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493` as the host-neutral end-to-end composition of the existing single-task performer. The source composes the already-qualified task-state/source-lineage, phase/resume, mechanical-lane, phase-prompt, review/convergence, evidence-recovery, UI-driver and publication owners; it must not introduce a second repository task truth or a shipping runtime owner.

Preflight qualification binds one invocation to one canonical task, exact source/spec/project context, one consistent host/tool inventory, the source-lineage gate, safe source preparation, read-only preflight facts and task-local recovery refs before Phase 1. Positive resume cases accept dirty source only when every path is already in the task-owned set and completed artifacts establish the boundary. Negative controls cover foreign ownership, unfinished dependencies, lineage mismatch, unsafe submodule/worktree overlap, dirty paths outside ownership, missing selected secret/funded fixtures, and attempts to stash/reset/clean unexpected work. These cases extend the existing state-authority/source-lineage contracts rather than replacing them.

Resumption qualification preserves the declared work artifacts and local checkpoints, reuses completed validated boundaries, and resumes at the first incomplete boundary. The performer alone owns commit/final-publication boundaries; leaf workers remain non-owning. Implementation follows assessed owned phases, selected direct pre-review validation, independent initial review, focused invalidation-based re-review and explicit convergence. An already-satisfied path has no manufactured source diff and spends its budget on direct measurement.

Evidence-adapter qualification requires every selected check to name claim, changed surface, instrument, oracle, window, control, falsifier and durable output before execution. Each run yields exactly `APPROVED | TEST_FLAW | IMPL_BUG | UNRECOVERABLE`; recovery carries unaffected passes, does not repeat an incapable technique, and does not turn a campaign cap into success or Block. Final-publication qualification verifies the complete result schema, retained/no-change/split disposition, exact `Unverified` scope, clean or intentionally retained checkout boundary and non-force canonical publication. Failure cases cover pre/post-Phase-1 lineage mismatch, split-required routing, disposable-worker exhaustion, test-recovery exhaustion, unsafe file-lock ownership, locked-host UI fallback, semantic publication conflict and remote outage without converting interruption into approval.

Stage-A contracts authored, not run: `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-PREFLIGHT-001`, `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-RESUME-001`, `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-EVIDENCE-ADAPTER-001`, `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-FINAL-PUBLICATION-001`, `FAULT-TDRP-PERFORM-TASK-PIPELINE-RECOVERY-001`, and `INT-TDRP-PERFORM-TASK-PIPELINE-END-TO-END-001`. Existing performer umbrella contracts remain authoritative. No executable qualification is run in Stage A.


### Repository process-inbox OpenAI host-adapter qualification

Fixed-U source `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546` is declarative OpenAI host invocation/discovery metadata for the already-qualified `$process-inbox` repository process. It may expose `interface.display_name`, `interface.short_description`, and `interface.default_prompt`, but it does not own inbox transaction state, project/task lineage, routed task state, split/consolidation, publication, validation, or acceptance.

A passing metadata contract preserves the exact display name `Process AI Inbox`, short description `Route inbox notes into project-aware AI tasks`, and a default prompt that invokes `$process-inbox` to route and plan the local AI inbox while preserving existing project lineage. Invocation qualification requires the adapter to delegate to `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d` instead of embedding a second planner/router. The host adapter must not broaden the route-and-plan boundary into implementation, task claiming, build/test execution, direct task-slot mutation, branch-history work items, a second queue, or acceptance authority.

Stage-A contracts authored, not run: `CONTRACT-TDRP-PROCESS-INBOX-OPENAI-HOST-METADATA-001` and `INT-TDRP-PROCESS-INBOX-OPENAI-HOST-INVOCATION-001`. No OpenAI adapter invocation, process-inbox helper, inbox transaction, build, test, validator, reviewer, runtime evidence or acceptance is executed in Stage A.

### Process Inbox supplied-video media-input qualification

Fixed-U source `.agents/skills/process-inbox/scripts/video.py@977d75aca7aea0b6c74471d533a5ee0e2a156ac3` is a repository-planning evidence helper under the existing process-inbox/task-process owner. It is not the shipping attachments/resource/media owner and does not create a second media store. Its only fixed-U direct invocation is the supplied-video section of `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`; the later performer consumes the committed task specification and referenced input files.

Probe/timing qualification requires `ffprobe` on PATH, the first video stream, per-frame `best_effort_timestamp_time`, normalization to the first decoded frame, and failure when no frame exists. Probe output binds the local reference to SHA-256, byte count, codec, dimensions, nominal frame rate, decoded-frame count, last-frame time and container duration, plus min/max intervals and a constant-rate decision when intervals exist. Cited timing may never be recomputed as frame-index / fps.

Extraction qualification accepts `all`, individual indices and inclusive ranges, deduplicates/sorts them, rejects any index outside the decoded frame count, and accepts crop only as `WxH+X+Y`. The ffmpeg select expression may compact contiguous runs, optional crop precedes scaling, enlargement uses nearest-neighbour and shrink uses area filtering, and decode uses passthrough frame pacing. The helper must reject a decoded-output count different from the requested set. Durable frame names bind decoded index and normalized milliseconds, with an optional label. Contact-sheet qualification samples after frame selection, rejects more than 200 selected cells, labels each cell with frame/time, and requires Pillow.

Durable-reference qualification keeps the original video and full extraction untracked: the original remains in ignored transaction/backup state, the bulk extraction is disposable scratch, and only phase-complete crops/sheets, key poses and context frames enter tracked task `input/`. The receipt binds digest/probe facts and the task links exact frame/time evidence while stating the video is absent from the repository. Failure-cleanup qualification proves helper-owned temporary raw decode files are removed on success/failure; missing tools/dependencies, malformed input, subprocess failure, empty decode or count mismatch remain non-passing. A post-decode target filesystem/write failure is not represented as transactional rollback by the upstream helper, so any partial target output must remain non-authoritative and be regenerated or excluded before publication.

Stage-A contracts authored, not run: `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-PROBE-PTS-001`, `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-FRAME-EXTRACT-001`, `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-DURABLE-REFERENCE-001`, `INT-TDRP-PROCESS-INBOX-VIDEO-PERFORMER-HANDOFF-001`, and `FAULT-TDRP-PROCESS-INBOX-VIDEO-CLEANUP-001`. No helper invocation, ffmpeg/ffprobe/Pillow execution, inbox transaction, build, validator, test, reviewer, runtime evidence or acceptance is executed in Stage A.



### Process Inbox workspace-helper qualification

Fixed-U source `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d` is the concrete repository-process helper beneath the already-qualified process-inbox/continue/perform-task governance. The exact blob is 185,107 bytes/decoded characters, 6,122 newline characters with a trailing newline, 6,123 split lines, and 221 class/function declarations at fixed-U recursive-tree index 80 zero-based. It is not shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner runtime, product Storage/Recovery, or a Telegram-specific product scheduler.

Identity/worktree qualification binds source-root recognition, stable lowercase machine/checkout identity, canonical AI repository root/branch, and slot/inbox/routing/code-health linked worktrees to one common Git directory and exact expected branch. Positive cases prove clean fast-forward synchronization and isolation of mutation domains. Negative controls cover invalid/missing tag, wrong checkout/root/branch, foreign or moved worktree, dirty canonical/worktree state, unpublished workspace commits during sync, unsafe nested-worktree/submodule overlap, and attempts to use the task slot as inbox/routing authority.

Transaction qualification computes one SHA-256 over sorted relative payload paths and exact bytes with explicit separators, excludes the backup directory, rejects symlinks, copies an immutable snapshot, and records checkout/repository identity plus digest. Resume accepts exactly one matching active transaction; multiple, foreign-checkout, foreign-repository, empty-new-transaction, or path-escaped transaction state fails closed. Finalize requires a canonical-master receipt containing the exact digest, always archives the snapshot, clears live input only when its digest is unchanged, and otherwise preserves the edited live inbox. Abort archives a failed pre-publication snapshot without clearing live input.

Task-state/source-lineage qualification preserves status/type/dependency/claim/phase/model schema; Start is the atomic ownership transition for eligible todo work, Retry is limited to owned blocked work, competing active ownership is rejected, and dependency readiness is distinct from source ancestry. Approved source-producing prerequisites must resolve through durable history to task commit tips reachable from current source HEAD; missing history/ancestry is non-passing and compatible branches are diagnostic only. Local task `base`/`green`/`run` refs and carried-work seals are resumable recovery metadata rather than canonical task truth.

Publication qualification stages only explicit normalized task/project/receipt paths, refreshes canonical state, rebases the isolated worktree, reruns the applicable semantic validator after rebase, and retries only ordinary non-fast-forward/fetch-first races. Rebase conflict aborts the rebase while preserving the unpublished commit; semantic conflict, remote outage, lost ownership, invalid retained-work seal, or unsafe source leaves recoverable state. The source helper's optional force push to a dedicated private `ai-<checkout-tag>` mirror is qualified only as non-authoritative recovery backup and cannot authorize force publication of canonical/shared refs. Approval-ref cleanup is permitted only after canonical approved state and source invariants are re-established.

The same exact source also houses already-mapped repository-quality mechanics for source/submodule preparation, owned-path commits, overlays/fences, build-lock recovery, portable test-account cleanup, host-load/console-gated test running, code-health worktrees, split/consolidation validation, and stale-project archiving. Those remain subordinate to the existing performer/quality/process contracts and are not executed or promoted to product/runtime ownership by this closure.

Stage-A contracts authored, not run: `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-IDENTITY-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKTREE-ISOLATION-001`, `CONTRACT-TDRP-PROCESS-INBOX-TRANSACTION-DIGEST-001`, `CONTRACT-TDRP-PROCESS-INBOX-TASK-STATE-TRANSITION-001`, `CONTRACT-TDRP-PROCESS-INBOX-SOURCE-LINEAGE-REFS-001`, `INT-TDRP-PROCESS-INBOX-WORKSPACE-PUBLICATION-001`, and `FAULT-TDRP-PROCESS-INBOX-WORKSPACE-RECOVERY-001`. No helper command, workspace test, source-lineage operation, Git publication simulation, build, test, validator, reviewer, runtime evidence, package, deployment, installation, or acceptance is executed in Stage A.


### Process Inbox workspace regression-suite qualification

Fixed-U source `.agents/skills/process-inbox/scripts/workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf` is the executable regression consumer of the already-mapped repository workspace/process helper. The exact blob is 214,649 Git bytes / decoded characters with 6,447 newline characters, a trailing newline, 6,448 split lines, 254 class/function declarations, 10 class declarations and 140 `test_*` methods at fixed-U recursive-tree index 81 zero-based. It directly imports `workspace` and terminates in the standard `unittest.main()` entrypoint; it is repository quality/test evidence, not shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, product Storage/Recovery, or platform-core runtime.

Identity/worktree regression preserves separation between source checkout identity, canonical AI-state repository and isolated slot/inbox/source-preparation/code-health worktrees. Direct cases prove that inbox setup does not create or mutate the task slot; a dirty active task slot survives inbox prepare/publish/finalize unchanged; registered linked worktrees are ignored only by source-specific cleanliness checks; moved/unregistered repositories, symlink aliases, submodule-target overlap and unsafe local/staged changes fail closed; recursive recorded submodule pins may advance only when the source is otherwise safe.

Transaction/publication/finalize regression uses the digest returned by `prepare` as the receipt binding, rejects a transaction owned by another checkout, requires narrow publication paths, preserves deletions including non-ASCII paths, and allows retry only for ordinary fetch-first/non-fast-forward races. Finalize rejects path escape and any receipt not present at canonical master HEAD, and the lifecycle case proves that successful publication can archive the transaction, clear unchanged live input and retain its backup while leaving an unrelated dirty task slot untouched.

Task-state/source-lineage regression proves owned blocked Retry resumes without republishing task truth, Start atomically claims an unclaimed todo, Start refuses missing source ancestry before mutation, compatible local branches are diagnostic, checkpoint remains local, normal lifecycle publishes only Start and Approve, competing active work blocks Retry, retained task commits may precede later independent work, and source-begin initializes/reconciles local base/green/run recovery refs. Batch cases prove batch selection is metadata rather than pre-claiming, unknown ids fail before mutation, a lost concurrent Start leaves no batch publication, and queue views preserve actual owner/status.

Publication/recovery regression additionally constrains task-owned source commits, non-authoritative private recovery mirroring, carried-work/split publication, approval-source-ref cleanup and interruption recovery. Approval refs survive fetch failure, unsafe/dirty source, foreign/non-approved canonical state and unrelated ownership; they are deleted only after the canonical approved commit and source invariants are re-established. A private `ai-<checkout-tag>` force mirror remains best-effort recovery only and never changes canonical publication authority.

Coverage is recorded without inflation. In the exact test blob, `workspace.command_prepare` appears in six direct calls, `command_finalize` in three, `command_start` in eleven, `command_retry` in two, `command_inbox_publish` in three, `command_publish` in one and `source_lineage_report` in three; `workspace.command_abort` has zero direct calls. The suite consumes prepare-produced digests in receipts but does not directly invoke a transaction/payload/inbox digest helper. Therefore this source is positive regression evidence for the exercised boundaries, but it does not independently prove the helper's digest algorithm or abort command. The previously authored helper contracts remain normative, and the Fabushi regression contract explicitly retains digest and abort cases instead of inferring them from absent upstream tests.

Stage-A contracts authored, not run: `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-IDENTITY-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-WORKTREE-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-TRANSACTION-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-TASK-STATE-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-FINALIZE-ABORT-001`, `INT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-PUBLICATION-001`, and `FAULT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-RECOVERY-001`. No workspace helper/test invocation, Git/source publication simulation, build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, packaging, deployment, installation, runtime evidence or acceptance is executed in Stage A.


### Intent-aware repository rebase qualification

Fixed-U source `.agents/skills/rebase/SKILL.md@0660dc78b26614e40d7151d1da59241b1bad5430` defines a repository branch-integration process rather than an ai-tdesktop task or shipping product capability. The exact blob is 15,114 Git bytes / 15,036 decoded characters, 321 newline characters with a trailing newline, 322 split lines, and 12 Markdown headings at recursive-tree index 83 zero-based. Direct fixed-U host callers are Claude `.claude/commands/rebase.md@df14e750067aeac1258a92068886cd2b33f83120` and Grok `.grok/commands/rebase.md@dc095e3c837aba1b30ff35e1bb92952cef5f1780`; both read the shared skill completely, keep decision-making in the current context, and forbid their normal AI workflow/task publication adapters. Fixed-U `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32` and `REVIEW.md@e65be8231a8e0f50ee86b20ef16120d58ea40376` are style/host dependencies, while the performer pipeline remains a boundary consumer that forbids branch integration inside a task.

Scope/source-lineage qualification resolves `ONTO`, `REPLAY_BASE` and `OLD_TIP`, proves the exact replay list and overlap surface, rejects sibling-worktree ownership, dirty-tree mutation, ambiguous prose/command mismatch and implicit remote refresh, and treats an existing rebase as resume rather than a nested operation. Merge-containing ranges require an explicit flatten versus `--rebase-merges` decision. Non-interactive host qualification requires editor-free continue/todo mechanics and preserves the checkout's line-ending convention.

Conflict qualification distinguishes mechanical adjacency from overlapping logic. Overlapping cases require stage-1/base, stage-2/ONTO and stage-3/replayed views plus relevant histories; the oracle is the joint intent in the current upstream shape, not textual side selection. Negative cases reject blanket ours/theirs, hard-conflict skip, deleted-hunk escape, residual markers, resurrecting removed upstream structure merely to host the old patch, and silently guessing a genuine semantic contradiction. `rerere` is re-reviewed as pre-filled conflict evidence, and an empty commit is skipped only when end-state equivalence is positively established.

Recovery qualification binds `OLD_TIP`/reflog as the undo path, reconstructs resume state from current Git metadata, preserves an unknown state rather than auto-aborting, and limits abort to an explicit requested recovery. Qualification must also show that no backup refs, unrelated branch mutations, autonomous fetches, pushes or force-pushes are performed by the rebase owner.

Whole-operation qualification requires conflict-region review plus one-to-one whole-series range-diff reasoning, final replay log and clean/understood status. Changed or missing mappings are investigated rather than accepted by command exit code. Publication remains a separate owner boundary: a rewritten published branch may be reported as needing force publication, but the rebase process itself never performs it. FMEP Stage-B may pair these semantic oracles with applicable build/compile checks; Stage A records them only.

Stage-A contracts authored, not run: `CONTRACT-TDRP-REBASE-SCOPE-LINEAGE-001`, `CONTRACT-TDRP-REBASE-CONFLICT-INTENT-001`, `CONTRACT-TDRP-REBASE-RERERE-EMPTY-001`, `CONTRACT-TDRP-REBASE-RESUME-ABORT-RECOVERY-001`, `CONTRACT-TDRP-REBASE-NONINTERACTIVE-HOST-001`, `INT-TDRP-REBASE-HOST-COMPOSITION-001`, `INT-TDRP-REBASE-PUBLICATION-BOUNDARY-001`, `FAULT-TDRP-REBASE-UNKNOWN-CONTRADICTION-001`, and `SEC-TDRP-REBASE-NO-FORCE-PUBLISH-001`. No rebase/fetch/range-diff command, build, compile, test, validator, reviewer, CI, publication simulation, package, deploy, installation, runtime evidence or acceptance is executed in Stage A.


### Claude rebase command host-adapter qualification

Qualify fixed-U `.claude/commands/rebase.md@df14e750067aeac1258a92068886cd2b33f83120` strictly as the Claude Code host projection of the already-qualified shared rebase skill. Source/host authority positives prove the wrapper completely loads `.agents/skills/rebase/SKILL.md@0660dc78b26614e40d7151d1da59241b1bad5430`, passes raw `$ARGUMENTS`, and explicitly excludes `.claude/ai-workflow-adapter.md`; negative controls reject task records, phase leaves, continue/perform-task/process-inbox routing, AI publication or any copied Claude-specific rebase state machine.

Invocation/source-lineage qualification reuses the shared ONTO/REPLAY_BASE/OLD_TIP and replay-list oracles while checking the host-specific decision gate. When two readings replay different commits or a genuine semantic contradiction remains, `AskUserQuestion` must name both intents and offer two to four concrete directions, and no `rebase --continue` may occur before an answer. For long replay series, `TodoWrite` is at most a one-item-per-commit progress mirror; qualification fails if Todo state substitutes for Git lineage, rebase metadata, reflog recovery or evidence.

Non-interactive qualification proves every continuation uses `git -c core.editor=true rebase --continue` and todo-list edits use a non-interactive `GIT_SEQUENCE_EDITOR`. An editor timeout or unavailable host mechanism preserves the rebase for recovery rather than skipping a decision or resetting state. Delegation qualification proves `Agent` is absent, no conflict/history decision is delegated, and the current context reads both sides directly. Scratch qualification proves stage dumps and running reports remain session-only and never become tracked repository files.

Recovery/publication qualification composes the wrapper with the shared skill's resume/abort/reflog rules and no-push boundary. Unknown in-progress state remains intact; explicit abort restores the shared undo path; a published rewritten branch may be reported as needing force publication, but the adapter itself performs no autonomous fetch, push, force-push, AI publication or acceptance. Grok `.grok/commands/rebase.md@dc095e3c837aba1b30ff35e1bb92952cef5f1780` is a peer host adapter, not a caller or alternate authority.

Stage-A contracts authored, not run: `CONTRACT-TDRP-REBASE-CLAUDE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-REBASE-CLAUDE-INVOCATION-LINEAGE-001`, `CONTRACT-TDRP-REBASE-CLAUDE-DECISION-GATE-001`, `CONTRACT-TDRP-REBASE-CLAUDE-NONINTERACTIVE-001`, `CONTRACT-TDRP-REBASE-CLAUDE-DELEGATION-001`, `CONTRACT-TDRP-REBASE-CLAUDE-SCRATCH-001`, `CONTRACT-TDRP-REBASE-CLAUDE-RECOVERY-PUBLICATION-001`, `INT-TDRP-REBASE-CLAUDE-COMPOSITION-001`, `FAULT-TDRP-REBASE-CLAUDE-UNANSWERED-CONFLICT-001`, `FAULT-TDRP-REBASE-CLAUDE-EDITOR-RECOVERY-001`, and `SEC-TDRP-REBASE-CLAUDE-NO-AI-PUBLICATION-001`. No rebase/fetch/range-diff command, Todo/Agent execution, build, compile, test, validator, reviewer, CI, publication simulation, package, deploy, installation, runtime evidence or acceptance is executed in Stage A.

### Repository static-analysis policy qualification

Qualify the fixed-U `.clang-tidy` policy under the existing source-neutral repository quality/test-evidence lifecycle. Configuration-scope positives prove that only `bugprone-unsafe-functions` is enabled, that the same check is promoted through `WarningsAsErrors`, that `HeaderFilterRegex` remains empty, and that no source-specific scanner or second findings authority is created. Custom-function positives preserve the complete helper-to-standard replacement set, including the nontrivial `qBound -> std::clamp(low, value, high)` ordering note and `qRound -> int(base::SafeRound(...))` policy.

Composition qualification proves that the repository hook reads the canonical root `.clang-tidy` rather than maintaining a duplicate ban list, while the clang-tidy runner consumes the same root configuration through its normal discovery path and the generated compilation database. The toolchain gate records clang-tidy 20+ as a semantic prerequisite because an older binary can silently ignore `CustomFunctions`. Negative controls reject missing/extra checks, warning/error drift, replacement drift, a hook-local copied list, invocation against a stale/non-root configuration, or an old clang-tidy result being treated as proof. Header repetition and submodule findings are not silently converted into repository-owned pass/fail evidence.

Stage-A contracts authored, not run: `CONTRACT-TDRP-STATIC-ANALYSIS-CONFIG-SCOPE-001`, `CONTRACT-TDRP-STATIC-ANALYSIS-CUSTOM-FUNCTIONS-001`, `CONTRACT-TDRP-STATIC-ANALYSIS-HOOK-PARITY-001`, `CONTRACT-TDRP-STATIC-ANALYSIS-TOOLCHAIN-VERSION-001`, `INT-TDRP-STATIC-ANALYSIS-QUALITY-COMPOSITION-001`, and `FAULT-TDRP-STATIC-ANALYSIS-CONFIG-DRIFT-001`. No clang-tidy, hook, lint, build/configure, unit/contract/integration test, validator, reviewer, runtime evidence, CI, or acceptance is executed in this closure.


### Repository Claude AI workflow host-adapter qualification

Qualify the fixed-U Claude adapter as a host-specific projection of existing repository process authority, never as a second scheduler, performer, task store, review store or publication owner. Positive cases prove that `.claude/commands/continue.md`, `perform-task.md`, `process-inbox.md` and `code-health.md` explicitly load the adapter while `rebase.md` explicitly excludes it because branch integration is outside the AI queue pipeline.

Delegation qualification proves one stateful performer, no leaf commits, foreground completion for ordinary phase leaves, bounded retry/resume only after writer termination, no watcher/poll loop, and parallel Agent calls only for genuinely independent leaves in one step. Negative controls reject duplicate performers, two writers targeting one artifact, background phase leaves mistaken for completion, SendMessage-style secondary reporting, progress-only task termination, nested Claude subprocesses and stale cross-task claims injected into worker prompts without checking the current branch.

Provider qualification preserves the shared phase/verdict contract across Claude-native and Codex review/assessment leaves, including explicit native fallback when Codex is unavailable or opted out. Effort/model selection remains host mechanics, not task-state authority. Source-lineage qualification keeps startup/pre-Phase-1 routing decisions in the parent scheduler, post-Phase-1 mismatch inside the performer, and forbids silent cherry-pick/rebase/merge/backport/branch-sync queue work. Text-handling qualification allows the Claude adapter to omit dedicated line-ending/BOM normalization while preserving all content, ownership, review, evidence and publication gates.

Stage-A contracts authored, not run: `CONTRACT-TDRP-CLAUDE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-CLAUDE-DELEGATION-SINGLE-WRITER-001`, `CONTRACT-TDRP-CLAUDE-LEAF-COMPLETION-RECOVERY-001`, `CONTRACT-TDRP-CLAUDE-REVIEW-PROVIDER-001`, `CONTRACT-TDRP-CLAUDE-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-CLAUDE-TEXT-HANDLING-001`, `INT-TDRP-CLAUDE-HOST-PROCESS-COMPOSITION-001`, `INT-TDRP-CLAUDE-PUBLICATION-BOUNDARY-001`, and `FAULT-TDRP-CLAUDE-DUPLICATE-WRITER-RECOVERY-001`. No host Agent/Codex leaf/helper/source-lineage invocation, build, lint, validator, unit/contract/integration/E2E, reviewer, UI-driver, package, deployment or acceptance is executed in this closure.


#### Claude code-health command-adapter qualification

Qualify fixed-U `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a` as a host adapter over the existing repository quality-health command owner and Claude AI workflow adapter. Positive qualification proves that the command reads the generic Claude adapter and shared code-health skill completely before applying host mechanics, preserves the shared skill as the semantic/state authority, and passes the invocation's raw `$ARGUMENTS` into exactly the declared mode surface: no argument or `scan` for scan, `survey [area]` for one area survey, and `check [base]` for one change check.

Routing qualification is independent from mode selection. A proposal reaches the existing inbox only when the invocation explicitly includes `route`; the scheduled form is `scan route`. Negative controls reject implicit routing from a plain scan, survey, or check; host-side invention of task state, findings state, approval, or publication; and any interpretation of the front-matter tool allowlist as permission to execute Stage-A validation or source mutation. Delegation qualification proves that Claude-specific Agent mechanics come only from the already-qualified host adapter while scan/survey/check semantics, state, owner decisions, and publication remain those of the shared code-health skill.

Composition evidence must identify the direct Claude command-host consumer, the shared skill's documented `/code-health scan route` and `/code-health survey <area>` invocations, and the absence of a fixed-U repository file-level import of the command wrapper. The OpenAI `$code-health` registration is a peer adapter and cannot become a second behavior or state authority.

Stage-A contracts authored but not run: `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-INVOCATION-001`, `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-DELEGATION-001`, `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-ROUTING-001`, `INT-TDRP-CODE-HEALTH-CLAUDE-COMPOSITION-001`, and `FAULT-TDRP-CODE-HEALTH-CLAUDE-ROUTE-FENCE-001`.


#### Claude continue command-adapter qualification

Qualify fixed-U `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8` as a thin Claude host adapter over the existing continue task/process scheduler plus generic Claude workflow adapter. Positive qualification proves complete loading of both dependencies before scheduling, read-only queue behavior when no batch specification is present, exact freeze of one selected batch with active-task precedence, and strict exclusion of unrelated later queue additions.

Batch qualification allows only canonical split replacements and follow-ups routed from results produced by the same invocation to extend the effective batch. Source-lineage qualification keeps startup and pre-Phase-1 safe-switch decisions in the parent command, keeps post-Phase-1 mismatch task-local, and forbids cherry-pick/rebase/merge/backport/branch-sync queue work. Delegation qualification proves one stateful performer, fresh split worker, bounded routing/consolidation lanes, complete host-adapter loading by publication-owning workers, and no duplicate writer/scheduler authority.

Review-provider qualification proves one startup choice: explicit no-Codex arguments select `native`, otherwise `codex`; provider choice is carried into performer prompts without changing shared artifacts, verdicts, task state, routing, or publication. Negative controls reject host-side task ownership, hidden batch widening, parallel performers, in-flight routing/consolidation at invocation end, host replies as durable publication, and any Stage-A execution.

Stage-A contracts authored but not run: `CONTRACT-TDRP-CONTINUE-CLAUDE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-CONTINUE-CLAUDE-BATCH-FREEZE-001`, `CONTRACT-TDRP-CONTINUE-CLAUDE-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-CONTINUE-CLAUDE-DELEGATION-001`, `CONTRACT-TDRP-CONTINUE-CLAUDE-REVIEW-PROVIDER-001`, `CONTRACT-TDRP-CONTINUE-CLAUDE-ROUTING-LANE-001`, `INT-TDRP-CONTINUE-CLAUDE-COMPOSITION-001`, and `FAULT-TDRP-CONTINUE-CLAUDE-DUPLICATE-SCHEDULER-001`.


#### Crash-report process command qualification

Fixed-U `.claude/commands/crash-reports-process.md@5ac555406670721694e78c4e1e6b2a96b3e98fb5` is qualified as repository maintenance/quality orchestration with direct helper `.claude/scripts/crash_triage.py@2db6cc14d8e529c4e61c5dcf169aade31fd35c58`. The generic Claude workflow adapter is a negative dependency: this command does not explicitly load it.

Qualification covers optional version/branch/`--no-claim`/`--limit`/`--dry-run`, complete-triple claim and interrupted-claim recovery, incomplete-report retention, move-not-delete sweep defaults, summary-only parent context, known-group skipping, waves of at most ten, one isolated investigation per selected non-noise group, adversarial verification of every proposed fix, read-only investigation source access, the exact three verdicts, and the common-path performance fence for `fixed`.

Persistence/recovery qualification covers serial application to current target-branch source, one-crash commit identity, record-after-wave, selected-group reconciliation after every wave and before final reporting, version-independent ledger keys, atomic ledger replacement, and dry-run's no-mutation boundary. The final one-time build/report is Stage-B executable evidence only.

Authored and not run: `CONTRACT-TDRP-CRASH-REPORT-CLAUDE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-CRASH-REPORT-INVOCATION-001`, `CONTRACT-TDRP-CRASH-REPORT-CLAIM-RECOVERY-001`, `CONTRACT-TDRP-CRASH-REPORT-DELEGATION-001`, `CONTRACT-TDRP-CRASH-REPORT-VERDICT-ROUTING-001`, `CONTRACT-TDRP-CRASH-REPORT-RECORD-RECONCILE-001`, `CONTRACT-TDRP-CRASH-REPORT-BUILD-REPORT-BOUNDARY-001`, `INT-TDRP-CRASH-REPORT-PROCESS-COMPOSITION-001`, `FAULT-TDRP-CRASH-REPORT-INTERRUPTED-CLAIM-001`, and `FAULT-TDRP-CRASH-REPORT-DROPPED-VERDICT-001`.


#### Claude cross-review command-adapter qualification

Qualify fixed-U `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361` as a thin Claude host adapter over the existing cross-review protocol/snapshot/prompt and AI-worker lifecycle authorities. Qualification first proves complete loading of protocol `51dabe142d5c3c7247e4050790c4e74b40f35b27`, prompts `4e155d2bd7e647c12df7b008c279969bfd65e44d`, and Codex-leaf contract `11c59b68df43e442e596c3052296fc51a044d46a`; the generic Claude workflow adapter remains a negative dependency because the command never loads it.

Invocation cases cover uncommitted default scope, the three last-N-commit spellings, explicit round-cap override, review-only/no-fixes, resume by session/resume-last, and free-form reviewer focus. Admission proves one successful Codex availability/auth check before leaves, no install/login/global mutation, and the command-specific provider fence: Codex unavailability stops cross-review rather than silently substituting native/Claude reviewers. Scope cases reuse the existing exact-base/private-index snapshot owner, require the initial session/file-count/size disclosure, and distinguish valid empty scope from acceptance.

Round topology covers concurrent fresh general+lifetime/reuse/structure/performance/security read-only leaves, exact foreground wait, report/verdict presence, one inactive-proven `--replace` retry, twice-failed lens as a declared coverage gap, twice-failed general as a stop, and same-thread general synthesis only after applicable lens reports. Arbitration cases require runner re-read of code, ledger-before-edit `FIX|REJECT|DEFER|COMPROMISE`, bounded reversal handling, and review-only's prohibition on edits. Later-round qualification covers fresh focused general review, rerunning only lenses that originated accepted fixes, immutable next-round snapshots/fix deltas, convergence/cap/non-convergence/human-decision stops, and resume from the first incomplete durable step without widening or restarting the frozen scope.

Fix/build qualification preserves ordinary `AGENTS.md`/`REVIEW.md` ownership, source-command Debug build and isolated-index staged comment-rule checks, and the command's prohibition on commit/amend/rebase/stash. Those executable paths are Stage-B-only under FMEP-001 for the current migration.

Authored and not run: `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-INVOCATION-SCOPE-001`, `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-CODEX-PROVIDER-FENCE-001`, `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-DELEGATION-WAIT-001`, `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-REPORT-ROUTING-001`, `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-ARBITRATION-FIX-001`, `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-RESUME-RECOVERY-001`, `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-BUILD-HOOK-BOUNDARY-001`, `INT-TDRP-CROSS-REVIEW-CLAUDE-COMPOSITION-001`, `FAULT-TDRP-CROSS-REVIEW-CLAUDE-LEAF-FAILURE-001`, and `FAULT-TDRP-CROSS-REVIEW-CLAUDE-RESUME-FENCE-001`.


#### Claude icon-generation command-adapter qualification

Qualify fixed-U `.claude/commands/icon.md@479e23eceb019477fd7f2ffe4e2c13aadcfc556e` as a repository-development adapter into existing Fabushi design/asset/process/quality owners, not as a Telegram asset subsystem. Host qualification proves the only direct command consumer is Claude Code/user `/icon`, preserves the exact clipboard helpers and pinned codegen renderer lineage, and rejects Product Tasks/Automations, Agent/Coordinator/Host/Runner, product attachment/resource storage, platform-core runtime, or a second icon registry as owners.

Source-input cases cover: new request with approved raster input; attached/pasted preview whose exact captured source matches; empty/new request requiring an image; capture failure; preview/source mismatch; follow-up with no new image reusing the previous bound source; follow-up after the source has disappeared; and explicit new-source replacement. Tests prove acquisition occurs before vectorization, one exact input identity drives an iteration, ambient clipboard state cannot overwrite a bound source after capture, and provider/API credentials never enter repository content or evidence.

Generation cases cover default and non-default output dimensions/aspect/padding/alignment, background removal, unchanged intentional path geometry, transform preservation, minimal SVG assembly, distinct versus even-odd path composition, malformed provider output, and collision-safe canonical naming. Fabushi adaptation cases prove generic UI icons use semantic/current-color behavior rather than source `#FFFFFF`, externally branded colors require brand-asset authority, Telegram resource paths are not canonical targets, and scratch/provider iterations cannot become a second registry.

Validation cases distinguish structural SVG qualification from render/visual acceptance. Structural assertions cover parseability, size/viewBox/aspect, transparent-background policy, allowed attributes/elements, color policy, canonical-registry uniqueness and provenance. Visual cases cover recognizability, scale/padding, clean geometry, theme/state rendering, icon-only accessible-name usage, and absence of accidental source branding. Renderer unavailable/failure yields missing evidence rather than pass; raw provider success or human eyeballing alone is never acceptance. The fixed-U codegen renderer's invalid-SVG/open/save failure paths are negative controls, but its build/render execution is Stage-B-only under FMEP-001.

Follow-up/recovery cases preserve one source/candidate lineage, increment iteration identity without overwriting prior evidence, permit bounded text-only vector edits, require a new raster source for changes that cannot be justified from the existing vector/source, retain unparsable/raw provider output only as debug material, and route only the selected candidate through the existing Fabushi asset owner. Provider failure must stop raster generation without manual geometry invention or secret leakage.

Authored and not run: `CONTRACT-TDRP-ICON-CLAUDE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-ICON-SOURCE-INPUT-001`, `CONTRACT-TDRP-ICON-GENERATION-NORMALIZATION-001`, `CONTRACT-TDRP-ICON-FABUSHI-ASSET-ROUTING-001`, `CONTRACT-TDRP-ICON-STRUCTURAL-VISUAL-VALIDATION-001`, `CONTRACT-TDRP-ICON-FOLLOWUP-RECOVERY-001`, `CONTRACT-TDRP-ICON-PROVIDER-SECURITY-001`, `INT-TDRP-ICON-DESIGN-ASSET-COMPOSITION-001`, `FAULT-TDRP-ICON-SOURCE-MISMATCH-001`, and `FAULT-TDRP-ICON-RENDERER-UNAVAILABLE-001`.

#### Claude perform-task command-adapter qualification

Qualify fixed-U `.claude/commands/perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e` as a thin Claude host adapter over the existing single-task performer and Claude AI workflow adapter. Positive qualification proves complete loading of both exact dependencies, preserves the shared performer as the only task/source/review/evidence/publication authority, and binds one invocation to exactly one short name/full id plus an optional review-provider opt-out that is never parsed into task identity.

Invocation cases prove the command stops after the named task result and never drains the queue, routes discoveries, consolidates pending work, or creates split replacements. Split cases preserve the shared `split-required` result and retained source state for later scheduler routing. Source-lineage cases distinguish a pre-Phase-1 clean routing stop from a post-Phase-1 task-local Block after owned-state recovery; negative controls reject cherry-pick/rebase/merge/backport/branch-sync substitution at either boundary.

Delegation/provider cases prove synchronous foreground Claude Agent leaves, self-contained leaf prompts, no nested Claude subprocess, and Codex review/assessment leaves unless the explicit invocation/scheduler provider is `native`. The provider cannot alter canonical artifacts or verdict semantics. Commit-subject cases require the shared conditional `[ai] ` prefix only for exclusively AI-workflow/infrastructure commits and forbid it for product or mixed changes. Publication cases reject source commits, leaf returns, review artifacts or progress-only replies as completion and require the existing canonical performer publication boundary.

Stage-A contracts authored but not run: `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-INVOCATION-SCOPE-001`, `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-DELEGATION-001`, `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-REVIEW-PROVIDER-001`, `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-COMMIT-SUBJECT-001`, `CONTRACT-TDRP-PERFORM-TASK-CLAUDE-SPLIT-PUBLICATION-001`, `INT-TDRP-PERFORM-TASK-CLAUDE-COMPOSITION-001`, `FAULT-TDRP-PERFORM-TASK-CLAUDE-PREPHASE-LINEAGE-001`, and `FAULT-TDRP-PERFORM-TASK-CLAUDE-DUPLICATE-WRITER-001`. No helper, Agent, Codex leaf, build, test, validator, reviewer, evidence campaign, CI, publication simulation or acceptance is executed in this closure.


#### Claude process-inbox command-adapter qualification

Qualify fixed-U `.claude/commands/process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21` as a thin Claude host adapter over the existing repository inbox-routing/process authority and Claude AI workflow adapter. Positive qualification proves complete loading of both exact dependencies, keeps the shared process skill/helper as the only transaction/task/project/receipt/publication owner, and treats `$ARGUMENTS` only as routing hints. Negative controls reject implementation, build, test, task claiming, automatic task execution, a second queue/state store, or a shipping Tasks/Automations/runtime owner.

Routing cases prove project-continuity bias, the affirmative independence test, durable alias/split resolution through live successors, cohesive-versus-independent task splitting, receipt deduplication, and the receipt-only disposition for branch-movement-only requests. They prove no backport/forward-port/cherry-pick/rebase/merge/branch-sync task is synthesized and that real product work may depend directly on the existing source task. Continue-mode coverage proves the scheduler consumes the shared process-inbox skill through one inbox worker and does not call this Claude command wrapper.

Workspace cases prove processing is bound to the checkout-specific `inbox_worktree`/`inbox_branch`, ignores the task `slot_worktree` even when dirty, snapshots one immutable transaction, and forbids broad staging or unrelated-path publication. Delegation cases prove a disposable non-delegating planner reads the Claude adapter before the shared skill, proposes routing only, and leaves tracked writes to the orchestrator; no Bash-launched Claude subprocess is accepted. Host text handling may skip only the explicit line-ending/BOM validation item.

Publication/finalization cases prove explicit-path helper publication, inbox-branch commit ownership, bounded fetch/rebase/master publication, non-force behavior, local-master fast-forward, exact receipt/digest verification, ignored backup retention and clear-only-if-unchanged semantics. Recovery cases cover semantic rebase conflict, remote outage, unsafe inbox state, publication retry, an already-created inbox commit, and live inbox mutation during planning. Abort is valid only before tracked changes/commit creation; later failures retain the transaction for resumable publication.

Stage-A contracts authored but not run: `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-INVOCATION-ROUTING-001`, `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-PROJECT-CONTINUITY-001`, `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-WORKTREE-ISOLATION-001`, `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-DELEGATION-001`, `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-PUBLICATION-001`, `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-FINALIZE-DIGEST-001`, `CONTRACT-TDRP-PROCESS-INBOX-CLAUDE-RECOVERY-001`, `INT-TDRP-PROCESS-INBOX-CLAUDE-COMPOSITION-001`, `FAULT-TDRP-PROCESS-INBOX-CLAUDE-PUBLICATION-RACE-001`, and `FAULT-TDRP-PROCESS-INBOX-CLAUDE-INPUT-MUTATION-001`. No helper, planner Agent, Git publication operation, build, test, validator, reviewer, evidence campaign, CI, publication simulation or acceptance is executed in this closure.


### Claude release command host-adapter qualification

Qualify fixed-U `.claude/commands/release.md@baaa7c261db67dced2ce052c8cde016147031a7d` as a repository-process adapter over existing Fabushi release/process/quality governance, never as a second shipping release system. Host positives bind Claude Code/user `/release`, exact source identity, allowed host tools, and the absence of autonomous product-runtime or platform-core ownership.

Invocation/version tests cover explicit `major.minor`, explicit `major.minor.patch`, optional `beta`, omitted-version patch increment, and the invariant that beta/stable consume one shared numeric sequence. Negative cases reject silent reuse of an already-used number and any mutation before the chosen version receives explicit user confirmation. Clean-tree qualification requires a non-empty porcelain status to stop before changelog/version edits.

Source-lineage tests bind the latest changelog entry, applicable `v*` release baseline and commit range to HEAD. Stable-after-beta cases must include noteworthy user-visible changes since the last stable release without deleting historical beta blocks. Changelog content cases require brief imperative user-facing bullets, feature/improvement/fix grouping, build/CI/refactor/WIP exclusion, deduplication/collapse of related commits, exact date/header form and prepend-only history preservation.

Approval qualification is two-phase: writing the proposed changelog stops for human file review; no version helper or release commit may run before explicit approval. Version-helper qualification binds `set_version.bat@7e7c3f69e8706905032e58cc05dcc25802dccebe` and `set_version.py@305a2a5f37aa13c8bef29d13ea37cab7a776cde6`, including exactly-one changelog match and fail-closed handling for bad parameters, missing patterns or non-zero exit. Commit qualification checks subject format, blank-line separation, approved bullet fidelity and wrapping rules.

Recovery/publication qualification proves dirty-tree, unanswered-confirmation, unanswered-changelog-review and version-helper failure all stop without fabricating success. The adapter's terminal evidence is the newly created local release commit readback only. It must not create or move tags, push/force-push refs, publish a GitHub release, sign/package/deploy, or mark `FQT-REL-001`/migration acceptance complete.

Stage-A contracts authored, not run: `CONTRACT-TDRP-RELEASE-CLAUDE-HOST-001`, `CONTRACT-TDRP-RELEASE-INVOCATION-VERSION-001`, `CONTRACT-TDRP-RELEASE-CLEAN-TREE-001`, `CONTRACT-TDRP-RELEASE-CHANGELOG-LINEAGE-001`, `CONTRACT-TDRP-RELEASE-VERSION-SEQUENCE-001`, `CONTRACT-TDRP-RELEASE-CHANGELOG-CONTENT-001`, `CONTRACT-TDRP-RELEASE-APPROVAL-GATE-001`, `CONTRACT-TDRP-RELEASE-SET-VERSION-001`, `CONTRACT-TDRP-RELEASE-COMMIT-001`, `CONTRACT-TDRP-RELEASE-PUBLICATION-BOUNDARY-001`, `INT-TDRP-RELEASE-CLAUDE-COMPOSITION-001`, `FAULT-TDRP-RELEASE-DIRTY-TREE-001`, `FAULT-TDRP-RELEASE-SET-VERSION-001`, and `SEC-TDRP-RELEASE-NO-PUBLISH-001`. No executable release command, Git fetch/tag/log/commit, version script, build/test/validator/reviewer/CI/package/sign/deploy or acceptance is run in Stage A.


#### Windows clipboard-image acquisition host-adapter qualification

Qualify fixed-U `.claude/grab_clipboard.ps1@bece778c6038820ca4d16bbf4224548de1e385a8` as the thin Windows host-input dependency of fixed-U `.claude/commands/icon.md@479e23eceb019477fd7f2ffe4e2c13aadcfc556e`, not as a product clipboard/resource subsystem. Host-authority cases prove the direct fixed-U file-level consumer is the icon command's Windows clipboard branch, preserve `.claude/grab_clipboard.sh@80c0c6d05c99a8f1fca6c57b325f561440e7b22e` only as the macOS peer, and keep Fabushi design/asset, repository process, and quality/evidence owners authoritative.

I/O cases cover a caller-provided destination with an image on the clipboard, no-image clipboard, caller path failures, assembly/clipboard access failure, encoder/write failure, and a previously existing/transient destination disposition controlled by the caller. Positive behavior requires PNG output at the exact supplied path plus success-only exit/reporting. Negative behavior proves no image or any acquisition/save failure cannot be reported as success and cannot authorize downstream vectorization, canonical asset routing, or product Resource creation.

Composition cases bind the saved transient file back to the icon workflow's already-qualified source-match gate before generation. A successful capture whose pixels do not match the intended pasted/requested source remains non-passing. The adapter must not substitute chat pixels, arbitrary local files, network fetches, retry watchers, or hidden stores. Recovery cases require explicit re-copy/retry or destination-environment correction while preserving one caller-owned acquisition attempt/lineage; a later successful capture is a new acquisition result and does not rewrite prior failure evidence.

Security/privacy cases prove only the intended caller-selected image is retained, no unrelated clipboard content or credentials enter durable evidence, and the transient file cannot become canonical production state merely because it exists. Product clipboard, attachment/resource persistence, Computer, Agent/Coordinator/Host/Runner, platform-core, and a second icon/input registry are negative ownership controls.

Authored and not run: `CONTRACT-TDRP-WINDOWS-CLIPBOARD-IMAGE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-WINDOWS-CLIPBOARD-IMAGE-IO-001`, `CONTRACT-TDRP-WINDOWS-CLIPBOARD-IMAGE-PNG-001`, `CONTRACT-TDRP-WINDOWS-CLIPBOARD-IMAGE-FAIL-CLOSED-001`, `CONTRACT-TDRP-WINDOWS-CLIPBOARD-ICON-CONSUMER-001`, `CONTRACT-TDRP-WINDOWS-CLIPBOARD-RECOVERY-001`, `INT-TDRP-WINDOWS-CLIPBOARD-ICON-COMPOSITION-001`, `FAULT-TDRP-WINDOWS-CLIPBOARD-NO-IMAGE-001`, `FAULT-TDRP-WINDOWS-CLIPBOARD-SAVE-FAILURE-001`, and `SEC-TDRP-WINDOWS-CLIPBOARD-INPUT-BOUNDARY-001`.


#### macOS clipboard-image acquisition host-adapter qualification

Qualify fixed-U `.claude/grab_clipboard.sh@80c0c6d05c99a8f1fca6c57b325f561440e7b22e` as the thin macOS host-input dependency of fixed-U `.claude/commands/icon.md@479e23eceb019477fd7f2ffe4e2c13aadcfc556e`, not as a product clipboard/resource subsystem. Host-authority cases prove the direct fixed-U file-level consumer is the icon command's macOS clipboard branch, preserve `.claude/grab_clipboard.ps1@bece778c6038820ca4d16bbf4224548de1e385a8` only as the Windows peer, and keep Fabushi design/asset, repository process, and quality/evidence owners authoritative.

I/O cases cover missing/empty `outPath`, a caller-controlled fresh destination with PNG clipboard content, PNG-flavor conversion failure/no image, unavailable or failing `osascript`, AppleScript parse/runtime error, invalid/missing destination directory, permission/open/write/close failure, and pre-existing/partial output disposition owned by the caller. Positive behavior requires the exact controlled path to receive the intended PNG bytes, the AppleScript pipeline to produce `ok`, and the shell to report success with exit 0. Negative behavior proves any missing path or non-`ok` pipeline result cannot authorize vectorization, canonical asset routing, or product Resource creation.

Failure-oracle cases preserve the source's lossy diagnostic boundary: `No image on clipboard` plus exit 1 may represent a true no-image/PNG-conversion outcome or another suppressed AppleScript/path/write failure. Tests must therefore assert fail-closed acquisition, not infer root cause from the generic outer message. The helper does not define safe arbitrary-user path interpolation; qualification binds it to the existing icon workflow's repository-generated `.ai/icon_<random-hex>.png` path contract and treats untrusted arbitrary path text as outside the adapter boundary.

Composition cases bind the saved transient file back to the icon workflow's already-qualified reread/source-match gate before generation. A successful capture whose pixels do not match the intended pasted/requested source remains non-passing. Recovery cases require explicit re-copy/retry or destination/host correction, cleanup or disposition of any stale/partial transient file, and a new acquisition identity; no chat-pixel, arbitrary local-file, network, retry-watcher or hidden-store fallback is allowed.

Security/privacy cases prove the adapter reads only the explicitly invoked ambient image flavor and writes only the controlled caller path needed for the repository workflow; unrelated clipboard data, credentials and user imagery do not enter durable evidence. Product clipboard/history, attachment/resource persistence, Computer, Agent/Coordinator/Host/Runner, platform-core, durable `.ai/icon_*` truth and a second icon/input registry are negative ownership controls.

Authored and not run: `CONTRACT-TDRP-MACOS-CLIPBOARD-IMAGE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-MACOS-CLIPBOARD-IMAGE-INPUT-PATH-001`, `CONTRACT-TDRP-MACOS-CLIPBOARD-IMAGE-PNG-PIPELINE-001`, `CONTRACT-TDRP-MACOS-CLIPBOARD-IMAGE-FAIL-CLOSED-001`, `CONTRACT-TDRP-MACOS-CLIPBOARD-ICON-CONSUMER-001`, `CONTRACT-TDRP-MACOS-CLIPBOARD-RECOVERY-001`, `INT-TDRP-MACOS-CLIPBOARD-ICON-COMPOSITION-001`, `FAULT-TDRP-MACOS-CLIPBOARD-MISSING-PATH-001`, `FAULT-TDRP-MACOS-CLIPBOARD-NO-IMAGE-001`, `FAULT-TDRP-MACOS-CLIPBOARD-SAVE-FAILURE-001`, and `SEC-TDRP-MACOS-CLIPBOARD-CONTROLLED-PATH-001`.


#### Crash-triage mechanical helper qualification

Qualify fixed-U `.claude/scripts/crash_triage.py@2db6cc14d8e529c4e61c5dcf169aade31fd35c58` as the mechanical local-crash-corpus helper of fixed-U `.claude/commands/crash-reports-process.md@5ac555406670721694e78c4e1e6b2a96b3e98fb5`, not as a shipping telemetry/runtime subsystem. Ownership cases bind the helper to `TDRP-R9-CRASH-REPORT-PROCESS-CLAUDE-COMMAND-ADAPTER-001`, repository process governance and repository quality/evidence; product crash storage/upload, Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core, and a second durable crash ledger are negative controls.

Acquisition/recovery cases cover complete versus incomplete `log/err/dmp` triples, fresh timestamped run creation, pre-existing `.processing-*` resume, no-new-report reuse of the latest run, explicit `--new`, incomplete-triple retention, and claim/report counts. Mutation assertions prove the processing marker is renamed only after claim completion, the next inbox remains disjoint, and Stage A never executes those filesystem operations.

Version/parser cases cover explicit semantic and numeric versions, closed-alpha normalization, newest-tag selection, tagged `AppVersion` preference, numeric fallback, missing tags, git-command failure, replacement decoding, header/module/crashed-thread extraction, stack-scanning reliability metadata, unreadable log files, and exact `old-version | no-frames | unsymbolized | usable` classification.

Signature/grouping cases cover crash/allocation-noise trimming, preservation of a non-empty frame set, MSVC/GCC/Clang lambda/scope canonicalization, standard-forwarder mining, named versus generic assertions, consecutive-key collapse, line-number-independent 12-hex stable keys, hint extraction, known-ledger ordering, richest representative selection, and deterministic per-scan gid assignment. Hints are tested as metadata, not automatic product verdicts.

Repository cross-reference cases cover recursive-submodule tracked-file lookup with fallback, basename collisions, app-source preference, `<tag>..<against>` changed-path marking, absent git index, and source maps limited to distinct resolved files. Output-routing cases validate `results/`, `index.tsv`, `groups.md`, and `groups.json` schema/content, fresh/known separation, drop-id lists, accepted versions, member identities and source maps without treating those artifacts as release evidence.

Ledger/show/sweep cases cover missing and malformed ledger input, atomic temp-file replace on record, cumulative `seen`, optional note/commit and unique run identity, unknown gid failure, bounded representative rendering, default hiding of non-crashed threads/registers/Found-by lines, explicit reveal flags, default move-only sweep, explicit delete authority, dry-run non-mutation, selected kinds, and missing member-file skips. Malformed ledger qualification must expose the source's empty-ledger fallback and possible loss of prior decisions on subsequent record rather than relabel it as safe corruption recovery.

Security/privacy cases keep raw logs/dumps, username/user-tag fields, addresses, module lists and local run artifacts repository-maintenance scoped. Qualification must not copy local crash corpus into Fabushi product state or migration evidence unnecessarily, and must not infer user identity or publish raw crash content as acceptance evidence.

Authored and not run: `CONTRACT-TDRP-CRASH-TRIAGE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-CRASH-TRIAGE-VERSION-RESOLUTION-001`, `CONTRACT-TDRP-CRASH-TRIAGE-CLAIM-ROTATION-001`, `CONTRACT-TDRP-CRASH-TRIAGE-PARSE-CLASSIFY-001`, `CONTRACT-TDRP-CRASH-TRIAGE-SIGNATURE-GROUPING-001`, `CONTRACT-TDRP-CRASH-TRIAGE-REPO-CROSSREF-001`, `CONTRACT-TDRP-CRASH-TRIAGE-OUTPUT-ROUTING-001`, `CONTRACT-TDRP-CRASH-TRIAGE-LEDGER-RECORD-001`, `CONTRACT-TDRP-CRASH-TRIAGE-SHOW-SWEEP-001`, `INT-TDRP-CRASH-TRIAGE-COMMAND-COMPOSITION-001`, `FAULT-TDRP-CRASH-TRIAGE-INTERRUPTED-CLAIM-001`, `FAULT-TDRP-CRASH-TRIAGE-INCOMPLETE-TRIPLE-001`, `FAULT-TDRP-CRASH-TRIAGE-MALFORMED-LEDGER-001`, `FAULT-TDRP-CRASH-TRIAGE-NO-TAG-001`, and `SEC-TDRP-CRASH-TRIAGE-LOCAL-DATA-BOUNDARY-001`.


### Repository Claude settings / house-rule PostToolUse qualification

Responsibility `TDRP-R9-CLAUDE-REPOSITORY-SETTINGS-HOUSE-RULE-HOOK-001` owns this source-entry qualification under the existing Claude host/static-policy/process/security/quality composition.

Qualify fixed-U `.claude/settings.json@d47520dc0e3b7e8d6067456e33062140c75f7c79` as a thin Claude Code repository-host projection over existing process/security/quality owners. Positive cases bind the sole `hooks.PostToolUse` event, exact `Edit|Write|MultiEdit` matcher, exact `python3 "$CLAUDE_PROJECT_DIR/tools/rules/hook.py"` command and `Checking house rules` status text. Negative controls reject invented permission grants/denials, product authorization, or a second settings/security owner.

Composition qualification binds edit-mode dispatch to fixed-U `tools/rules/hook.py@287b36c948566659970651226c92f7b8b056b434` and canonical root `.clang-tidy@5610bd13664ca77549993def241eb3edaffeff7a`, while distinguishing sibling `--staged` consumers from settings-driven mode. Post-use qualification proves this is corrective after matched edits, not a pre-use barrier. Malformed stdin JSON and non-source paths are explicit success/no-op source behaviors; missing interpreter/script/project-dir or dispatch failure remains failed/unknown evidence, never compliance proof.

Security qualification proves the settings file adds no explicit allow/deny list, product permission state, persistence or remote service. The helper's canonical banned-function replacements remain sourced from root `.clang-tidy`; repository instructions remain process/security authority. Source-side `tools/rules/test_hook.py@dce99f3507e4ec9f71c8bbd5360e9dde9b62d45f` is retained only as a later oracle.

Stage-A contracts authored, not run: `CONTRACT-TDRP-CLAUDE-SETTINGS-HOST-AUTHORITY-001`, `CONTRACT-TDRP-CLAUDE-SETTINGS-MATCHER-SCOPE-001`, `CONTRACT-TDRP-CLAUDE-SETTINGS-HOOK-COMMAND-001`, `CONTRACT-TDRP-CLAUDE-SETTINGS-STATIC-POLICY-COMPOSITION-001`, `CONTRACT-TDRP-CLAUDE-SETTINGS-POST-USE-BOUNDARY-001`, `CONTRACT-TDRP-CLAUDE-SETTINGS-NO-PERMISSION-EXPANSION-001`, `INT-TDRP-CLAUDE-SETTINGS-HOUSE-RULE-COMPOSITION-001`, `FAULT-TDRP-CLAUDE-SETTINGS-MALFORMED-PAYLOAD-001`, `FAULT-TDRP-CLAUDE-SETTINGS-HOOK-UNAVAILABLE-001`, and `SEC-TDRP-CLAUDE-SETTINGS-TOOL-SCOPE-001`. No hook/edit tool, Python/git subprocess, clang-tidy, build, lint, validator, test, CI, packaging, deployment, installation, runtime evidence or acceptance is executed in Stage A.


### Repository development-container host/toolchain qualification

`TDRP-R9-REPOSITORY-DEVELOPMENT-CONTAINER-HOST-TOOLCHAIN-001` is qualified through authored-only Stage-A contracts and later frozen-candidate Stage-B execution. Source identity is fixed-U `.devcontainer.json@16a1bcc1192bed87c5211da3dc63b0b0ed99a93e`, with image-production lineage `Telegram/build/prepare/linux.sh@e4d2f920e4becdff6292d296efe6f21455a19fcb` -> `pyproject.toml@8f43692fb217f075ccc588c3b677177e42b5688f` / `gen_dockerfile.py@86d27c96e1f732a1faf3e85397ea3ba38e493a0d` -> `Dockerfile@3fea9e6866e0ea6351f8a13ebb825ebf49cadfc2`, and workflow documentation `docs/building-linux.md@fbdf25355734f35617849224da106f1b9fafb3c9`.

Authored, not run in Stage A:
- `CONTRACT-TDRP-DEVCONTAINER-HOST-AUTHORITY-001` — VS Code Dev Containers remains the development-host consumer; no shipping runtime owner is created.
- `CONTRACT-TDRP-DEVCONTAINER-IMAGE-LINEAGE-001` — exact `tdesktop:centos_env` name and preparation/generator/Dockerfile provenance remain bound.
- `CONTRACT-TDRP-DEVCONTAINER-CMAKE-ENVIRONMENT-001` — Ninja Multi-Config, `out`, compile commands, IntelliSense and extension settings remain exact.
- `CONTRACT-TDRP-DEVCONTAINER-WORKSPACE-MOUNT-001` — local workspace bind-mount and `/usr/src/tdesktop` workspace identity remain exact.
- `CONTRACT-TDRP-DEVCONTAINER-SECURITY-BOUNDARY-001` — `SYS_PTRACE` plus `seccomp=unconfined` are explicit development-only privilege broadening and never product authorization.
- `CONTRACT-TDRP-DEVCONTAINER-NO-SECRET-EXPANSION-001` — source declares no API credential, secret mount or production credential ownership.
- `INT-TDRP-DEVCONTAINER-TOOLCHAIN-COMPOSITION-001` — host config composes with the existing build/toolchain provenance and repository process owners without a second build system.
- `FAULT-TDRP-DEVCONTAINER-IMAGE-UNAVAILABLE-001` — missing/unprepared image remains non-passing.
- `FAULT-TDRP-DEVCONTAINER-HOST-CAPABILITY-001` — rejected mount/ptrace/seccomp/Dev Containers host capability remains non-passing.
- `FAULT-TDRP-DEVCONTAINER-WORKSPACE-MISMATCH-001` — wrong mount/workspace identity cannot qualify the environment.
- `SEC-TDRP-DEVCONTAINER-PRIVILEGE-BOUNDARY-001` — elevated development-container settings cannot leak into shipping/runtime security policy.

Stage A status is `validation_status=not-run`, `accepted=false`; no Docker/Poetry/CMake/Ninja/build/test/lint/validator/package/deploy/container execution is permitted by this authored qualification.


### Repository Git attributes / source-control normalization qualification

`TDRP-R9-REPOSITORY-GIT-ATTRIBUTES-SOURCE-CONTROL-001` is qualified through authored-only Stage-A contracts and later frozen-candidate Stage-B source-control evidence. Source identity is fixed-U `.gitattributes@d7f1c6218bd3bd6b595385b1308c86ba9923fa14`, recursive-tree index 103 zero-based, mode 100644, 419 Git bytes / 419 ASCII characters, 15 LF newline characters, no CR characters, trailing newline and 16 split lines.

Authored, not run in Stage A:
- `CONTRACT-TDRP-GITATTRIBUTES-HOST-AUTHORITY-001` — Git attribute resolution is the direct repository consumer; no product/runtime owner is created.
- `CONTRACT-TDRP-GITATTRIBUTES-TEXT-AUTO-001` — `* text=auto` preserves automatic text classification and index normalization semantics without being reinterpreted as a product encoding policy.
- `CONTRACT-TDRP-GITATTRIBUTES-DIFF-LF-001` — `*.diff text eol=lf` requires LF working-tree text when such files exist.
- `CONTRACT-TDRP-GITATTRIBUTES-BAT-CRLF-001` — `*.bat text eol=crlf` requires CRLF working-tree text for the six fixed-U Windows batch files.
- `CONTRACT-TDRP-GITATTRIBUTES-ZXCVBN-LF-001` — the two fixed-U zxcvbn word lists retain LF byte identity so digest-sensitive upstream content is not line-ending converted.
- `CONTRACT-TDRP-GITATTRIBUTES-BINARY-MACRO-001` — fourteen `.lottie` and ten `.bin` blobs are governed by Git's built-in binary macro and cannot enter text normalization/textual diff/merge treatment.
- `CONTRACT-TDRP-GITATTRIBUTES-NO-CUSTOM-DRIVERS-001` — no named diff/merge driver, filter, working-tree encoding, export-ignore or export-subst behavior may be invented from this source.
- `INT-TDRP-GITATTRIBUTES-SOURCE-CONTROL-COMPOSITION-001` — source-control normalization composes with existing repository process/provenance and quality/evidence governance, without a Telegram-specific Git owner.
- `FAULT-TDRP-GITATTRIBUTES-STALE-INDEX-001` — an attribute change with stale pre-normalized index content remains non-passing until an explicit later renormalization/re-check.
- `FAULT-TDRP-GITATTRIBUTES-PATTERN-MISCLASSIFICATION-001` — wrong text/binary or LF/CRLF classification is a qualification failure, not a tolerated platform variance.
- `FAULT-TDRP-GITATTRIBUTES-DIGEST-DRIFT-001` — zxcvbn byte/digest drift remains explicit failure and cannot be normalized to success.
- `SEC-TDRP-GITATTRIBUTES-NO-FILTER-EXECUTION-001` — the source defines no clean/smudge/process filter or custom executable merge/diff driver; Stage-B qualification must not invent or execute one as source behavior.

The current Fabushi checkpoint has no root `.gitattributes` and no tracked paths matching the upstream special patterns. This is a source-neutral governance mapping, not permission to copy Telegram resource paths or to rewrite the Fabushi index during Stage A. Stage A status is `validation_status=not-run`, `accepted=false`; no checkout/add/renormalize/diff/merge/archive/digest/build/test/lint/validator/package/deploy operation is executed by this qualification.

### Repository contribution / PR process qualification

`TDRP-R9-REPOSITORY-CONTRIBUTION-PROCESS-QUALITY-001` is qualified through authored-only Stage-A contracts and later frozen-candidate Stage-B repository-process evidence. Source identity is fixed-U `.github/CONTRIBUTING.md@4265e974d0b87622af223d5b8096b67edcbd4aab`, recursive-tree index 105 zero-based, mode 100644, 6,074 Git bytes / 6,074 ASCII characters, 118 LF newline characters, no CR characters, trailing newline and 119 split lines. Fixed-U build-link composition binds `README.md@15a6194f73ae6084a2ab25d14005dc10262cdfb3` rather than the source's floating `master` URL.

Authored, not run in Stage A:
- `CONTRACT-TDRP-CONTRIBUTING-HOST-AUTHORITY-001` — GitHub conventional contribution guidance plus contributor/reviewer consumption maps to existing repository process owners, not product runtime.
- `CONTRACT-TDRP-CONTRIBUTING-ACCEPTED-SCOPE-001` — upstream bug/optimization/docs acceptance and feature/translation/new-UI rejection remain Telegram maintainer-process provenance and do not reduce Fabushi migration scope.
- `CONTRACT-TDRP-CONTRIBUTING-BUILD-LINK-001` — build guidance resolves to the exact fixed-U README build section; floating `master` links cannot establish fixed-U identity.
- `CONTRACT-TDRP-CONTRIBUTING-UPSTREAM-LINEAGE-001` — source-side upstream remote, `master`, fetch/log/rebase and contributor-fork force-push semantics are preserved without granting force publication to Fabushi.
- `CONTRACT-TDRP-CONTRIBUTING-ATOMIC-PR-001` — one-issue/small atomic change scope remains the repository-process invariant.
- `CONTRACT-TDRP-CONTRIBUTING-SQUASH-COMMIT-001` — upstream one-commit PR guidance is recorded without rewriting unrelated Fabushi history.
- `CONTRACT-TDRP-CONTRIBUTING-WHITESPACE-ISOLATION-001` — unrelated whitespace cleanup cannot be mixed into a functional change.
- `CONTRACT-TDRP-CONTRIBUTING-CODE-CLARITY-001` — clear, straightforward naming/code guidance remains a review-quality invariant rather than a new style subsystem.
- `CONTRACT-TDRP-CONTRIBUTING-TEST-GATE-001` — source requires testing before submission; FMEP-001 defers execution to Stage B and forbids invented Stage-A pass evidence.
- `CONTRACT-TDRP-CONTRIBUTING-COMMIT-MESSAGE-001` — commit rationale and supported issue-closing linkage remain repository reporting/provenance requirements.
- `INT-TDRP-CONTRIBUTING-PROCESS-COMPOSITION-001` — contribution guidance composes with root process/source-provenance, existing rebase authority and shared quality/evidence governance without duplicate owners.
- `FAULT-TDRP-CONTRIBUTING-STALE-FORK-001` — divergent/stale upstream lineage remains non-passing until deliberate refresh/rebase recovery.
- `FAULT-TDRP-CONTRIBUTING-MIXED-SCOPE-001` — broad mixed issue/cleanup scope remains non-passing until split into reviewable responsibility.
- `FAULT-TDRP-CONTRIBUTING-UNTESTED-001` — missing required Stage-B execution evidence remains unqualified, never normalized to pass from prose review.
- `SEC-TDRP-CONTRIBUTING-NO-FORCE-PUBLISH-001` — upstream `git push --force` applies to the contributor-fork recipe only and cannot bypass Fabushi expected-SHA CAS, branch protection or publication authority.

Stage A status is `validation_status=not-run`, `accepted=false`; no fork mutation, Git rebase/reset/squash/force-push, build, test, validator, CI, PR mutation, package, deploy or runtime acceptance is executed by this qualification.


### Repository bug-report issue-form qualification

`TDRP-R9-REPOSITORY-BUG-REPORT-ISSUE-FORM-001` is qualified through authored-only Stage-A contracts and later frozen-candidate Stage-B repository-process evidence. Source identity is fixed-U `.github/ISSUE_TEMPLATE/BUG_REPORT.yml@847b56e543fc28586ad281640218dfe8954d8bee`, recursive-tree index 107 zero-based, mode 100644, 2,112 Git bytes / 2,100 Unicode code points, 74 LF newline characters, zero CR characters, trailing newline and 75 split lines.

Authored, not run in Stage A:
- `CONTRACT-TDRP-BUG-REPORT-HOST-AUTHORITY-001` — GitHub Issue Forms is the repository-host consumer; reporters and maintainers/triagers are human consumers; no product runtime or second issue tracker is created.
- `CONTRACT-TDRP-BUG-REPORT-SCHEMA-001` — the exact top-level name/description/bug-label intent and eight ordered controls are preserved as source provenance.
- `CONTRACT-TDRP-BUG-REPORT-REQUIRED-INPUTS-001` — exactly six source controls carry `required: true`; Crash ID and Logs remain optional.
- `CONTRACT-TDRP-BUG-REPORT-REPRODUCTION-001` — steps, expected behavior and actual behavior remain distinct required report dimensions.
- `CONTRACT-TDRP-BUG-REPORT-ENVIRONMENT-001` — exact OS guidance is retained while kernel-only text is recognized as source-declared poor input, not a schema regex.
- `CONTRACT-TDRP-BUG-REPORT-VERSION-001` — exact version guidance rejects `LATEST` at maintainer-policy level; no nonexistent machine validator is invented.
- `CONTRACT-TDRP-BUG-REPORT-INSTALL-SOURCE-001` — one required single-select dropdown exposes exactly seven source-declared installation choices and preserves distro-package routing as upstream policy only.
- `CONTRACT-TDRP-BUG-REPORT-CRASH-ID-001` — Crash ID stays optional and Telegram beta-enablement steps remain source-specific diagnostic provenance.
- `CONTRACT-TDRP-BUG-REPORT-LOG-RENDERING-001` — Logs stay optional; `render: shell` is formatting only and cannot imply command execution.
- `CONTRACT-TDRP-BUG-REPORT-BUG-LABEL-ROUTING-001` — source routing intent is the `bug` label; later qualification must verify current target routing rather than infer success from prose.
- `INT-TDRP-BUG-REPORT-PROCESS-COMPOSITION-001` — issue/reporting/process + privacy/security + shared quality/evidence owners compose without Telegram-specific support/runtime duplication.
- `FAULT-TDRP-BUG-REPORT-MISSING-REQUIRED-001` — a missing required field remains host-blocked/non-submittable in source semantics.
- `FAULT-TDRP-BUG-REPORT-AMBIGUOUS-ENVIRONMENT-001` — `LATEST`, kernel-only OS text, or mismatched package-source claims remain triage-quality failure even though the YAML has no custom regex.
- `FAULT-TDRP-BUG-REPORT-SENSITIVE-LOGS-001` — secrets or sensitive diagnostics in a public report are non-passing and must be redacted/omitted/routed through an approved protected process.
- `SEC-TDRP-BUG-REPORT-NO-LOG-EXECUTION-001` — shell rendering never authorizes execution of reporter-controlled text.
- `SEC-TDRP-BUG-REPORT-PRIVACY-BOUNDARY-001` — the source's lack of redaction/private-report guidance cannot weaken Fabushi repository privacy/security policy.

The current Fabushi checkpoint has no `.github/ISSUE_TEMPLATE` tree. Existing-owner-first therefore records the upstream form behavior in current repository issue/reporting/process and shared security/quality owners rather than copying Telegram branding, beta/cheat-code instructions, distro support policy, or a second issue system. Stage A status is `validation_status=not-run`, `accepted=false`; no issue submission/creation/label mutation, diagnostic acquisition, build, test, validator, CI, package, deploy or runtime acceptance is executed by this qualification.


### Repository feature-request issue-form qualification

`TDRP-R9-REPOSITORY-FEATURE-REQUEST-ISSUE-FORM-001` is qualified through authored-only Stage-A contracts and later frozen-candidate Stage-B repository-process evidence. Source identity is fixed-U `.github/ISSUE_TEMPLATE/FEATURE_REQUEST.yml@e2a8becdb9afcbd028b62378321458a2028fe348`, recursive-tree index 108 zero-based, mode 100644, 925 Git bytes / 922 Unicode code points, 29 LF newline characters, zero CR characters, trailing newline and 30 split lines.

Authored, not run in Stage A:
- `CONTRACT-TDRP-FEATURE-REQUEST-HOST-AUTHORITY-001` — GitHub Issue Forms is the repository-host consumer; reporters and maintainers/triagers/product planners are human consumers; no product runtime or second issue tracker is created.
- `CONTRACT-TDRP-FEATURE-REQUEST-SCHEMA-001` — exact top-level name/description/enhancement-label intent and four ordered textarea controls are preserved as source provenance.
- `CONTRACT-TDRP-FEATURE-REQUEST-REQUIRED-INPUTS-001` — exactly three source controls carry `required: true`; Additional context remains optional.
- `CONTRACT-TDRP-FEATURE-REQUEST-PROBLEM-SCOPE-001` — the request captures the motivating problem separately from the desired solution.
- `CONTRACT-TDRP-FEATURE-REQUEST-SOLUTION-001` — the requested outcome remains an explicit required dimension without implying acceptance or implementation.
- `CONTRACT-TDRP-FEATURE-REQUEST-ALTERNATIVES-001` — considered alternatives remain a distinct required dimension.
- `CONTRACT-TDRP-FEATURE-REQUEST-ADDITIONAL-CONTEXT-001` — optional context/screenshots remain supporting material only and do not become mandatory.
- `CONTRACT-TDRP-FEATURE-REQUEST-ENHANCEMENT-ROUTING-001` — source routing intent is the `enhancement` label; later qualification must verify current target routing rather than infer success.
- `INT-TDRP-FEATURE-REQUEST-PROCESS-COMPOSITION-001` — issue/reporting/process + privacy/security + shared quality/evidence owners compose without Telegram-specific backlog/runtime duplication.
- `FAULT-TDRP-FEATURE-REQUEST-MISSING-REQUIRED-001` — a missing required field remains host-blocked/non-submittable in source semantics.
- `FAULT-TDRP-FEATURE-REQUEST-LOW-QUALITY-SCOPE-001` — vague/duplicate/out-of-scope content can pass presence checks but remains triage-quality failure; no invented validator is claimed.
- `FAULT-TDRP-FEATURE-REQUEST-SENSITIVE-CONTEXT-001` — sensitive screenshots/context in a public report are non-passing and must be redacted/omitted/protected.
- `SEC-TDRP-FEATURE-REQUEST-PRIVACY-BOUNDARY-001` — the source's lack of redaction/private-report guidance cannot weaken Fabushi repository privacy/security policy.
- `SEC-TDRP-FEATURE-REQUEST-NO-ACCEPTANCE-ESCALATION-001` — form submission/label routing cannot mint product approval, prioritization, scope, roadmap or implementation authority.

The current Fabushi checkpoint has no `.github/ISSUE_TEMPLATE` tree. Existing-owner-first therefore records the upstream form behavior in current repository issue/reporting/process and shared security/quality owners rather than copying Telegram branding or creating a second backlog. Stage A status is `validation_status=not-run`, `accepted=false`; no issue submission/creation/label mutation, screenshot acquisition/upload, build, test, validator, CI, package, deploy or runtime acceptance is executed by this qualification.


### Repository issue-template routing configuration qualification

`TDRP-R9-REPOSITORY-ISSUE-TEMPLATE-CONFIG-001` is qualified through authored-only Stage-A contracts and later frozen-candidate Stage-B repository-process evidence. Source identity is fixed-U `.github/ISSUE_TEMPLATE/config.yml@1b864704be6a3e3aafdb1e02519bec84886942ce`, recursive-tree index 109 zero-based, mode 100644, 753 Git bytes / 727 Unicode code points, 14 LF newline characters, zero CR characters, trailing newline and 15 split lines.

Authored, not run in Stage A:
- `CONTRACT-TDRP-ISSUE-CONFIG-HOST-AUTHORITY-001` — GitHub Issues issue-template chooser/configuration is the direct host consumer; reporters and maintainers/triagers are human consumers; no product runtime or second issue tracker is created.
- `CONTRACT-TDRP-ISSUE-CONFIG-BLANK-ISSUES-DISABLED-001` — source behavior sets `blank_issues_enabled: false`; later qualification must not infer a blank upstream issue route.
- `CONTRACT-TDRP-ISSUE-CONFIG-CONTACT-LINKS-ORDER-001` — exactly four source-declared contact links remain ordered and bound to their exact names/URLs/about copy.
- `CONTRACT-TDRP-ISSUE-CONFIG-DESKTOP-SCOPE-ROUTING-001` — mobile apps, UI/UX Design or missing-feature/non-Desktop-only matters route upstream to `bugs.telegram.org`; this is provenance, not Fabushi scope authority.
- `CONTRACT-TDRP-ISSUE-CONFIG-TRANSLATION-ROUTING-001` — translation/typo reports route upstream to `translations.telegram.org`; no Fabushi localization owner is replaced.
- `CONTRACT-TDRP-ISSUE-CONFIG-ACCOUNT-SUPPORT-ROUTING-001` — banned/login support routes upstream to `telegram.org/support`; no Fabushi account/session/security authority is delegated to Telegram.
- `CONTRACT-TDRP-ISSUE-CONFIG-QUESTION-ROUTING-001` — general questions route upstream to `t.me/TelegramDesktopTalk`; no public chat becomes a Fabushi support or privacy channel.
- `CONTRACT-TDRP-ISSUE-CONFIG-EXTERNAL-ROUTE-PROVENANCE-001` — external URLs and their prose are source routing metadata only and cannot mint target approval, SLA, product scope or support ownership.
- `INT-TDRP-ISSUE-CONFIG-PROCESS-COMPOSITION-001` — repository issue/reporting/process + privacy/security + shared quality/evidence owners compose without Telegram-specific support/runtime duplication.
- `FAULT-TDRP-ISSUE-CONFIG-BLANK-ISSUE-BYPASS-001` — a blank upstream issue path contrary to `blank_issues_enabled: false` is non-passing.
- `FAULT-TDRP-ISSUE-CONFIG-WRONG-ROUTE-001` — missing/reordered/mismatched names, URLs or category routing are non-passing until corrected.
- `FAULT-TDRP-ISSUE-CONFIG-SENSITIVE-EXTERNAL-DISCLOSURE-001` — secrets, account identifiers, private content or vulnerability details must not be normalized into public/unapproved external destinations.
- `SEC-TDRP-ISSUE-CONFIG-EXTERNAL-LINK-TRUST-BOUNDARY-001` — following a contact link crosses to an external trust domain; no reporter-controlled repository payload transfer or credential disclosure is authorized by this YAML.
- `SEC-TDRP-ISSUE-CONFIG-NO-TELEGRAM-AUTHORITY-ESCALATION-001` — Telegram support/translation/community routes cannot become Fabushi product, account, privacy or security authority merely through source provenance.

The current Fabushi checkpoint has no `.github/ISSUE_TEMPLATE` tree. Existing-owner-first therefore records exact upstream routing behavior in current repository issue/reporting/process and shared security/quality owners rather than copying Telegram contact links or creating a second support system. Stage A status is `validation_status=not-run`, `accepted=false`; no issue chooser interaction, contact-link navigation, issue/support mutation, build, test, validator, CI, package, deploy or runtime acceptance is executed by this qualification.


### Repository Dependabot configuration qualification

The fixed-U source `.github/dependabot.yml@5ace4600a1f26e6892982f3e2f069ebfab108d87` is qualified as repository-host dependency-update configuration only. Positive source-contract cases preserve schema `version: 2`, exactly one update entry, `package-ecosystem: github-actions`, root `directory: "/"`, and weekly `schedule.interval`. The direct host consumer is GitHub Dependabot; the dependency surface it can propose changes for is the GitHub Actions references declared under the same fixed-U repository root, not arbitrary build/package/runtime dependencies.

Source-lineage cases bind the configuration to the exact fixed-U tree and separately enumerate the contemporaneous workflow action references, preserving whether each reference is immutable-SHA pinned, version-tag pinned, or mutable branch/tag. A weekly proposal cadence is not evidence of freshness, vulnerability applicability, release trust, compatibility, adoption readiness, or safe auto-merge. Negative controls reject treating a Dependabot PR, version bump, changelog, successful build, signed tag, or absence of an advisory as sufficient supply-chain acceptance.

Existing-owner composition must prove no second dependency updater or product updater is introduced: repository dependency-advisory governance owns candidate/release/security assessment; ordinary repository process owns review/publication; product updater/release/deployment owners remain separate. If the target repository has no approved Dependabot configuration, Stage A records provenance and contracts only and does not create or execute one. Stage-A cases are authored only and remain not-run.

Authored cases: `CONTRACT-TDRP-DEPENDABOT-SCHEMA-001`, `CONTRACT-TDRP-DEPENDABOT-ECOSYSTEM-DIRECTORY-001`, `CONTRACT-TDRP-DEPENDABOT-WEEKLY-SCHEDULE-001`, `CONTRACT-TDRP-DEPENDABOT-WORKFLOW-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-DEPENDABOT-PROPOSAL-NONAUTHORITY-001`, `INT-TDRP-DEPENDABOT-DEPENDENCY-OWNER-001`, `FAULT-TDRP-DEPENDABOT-MISSING-WORKFLOW-SCOPE-001`, `FAULT-TDRP-DEPENDABOT-MUTABLE-REF-TRUST-001`, `SEC-TDRP-DEPENDABOT-NO-AUTO-ADOPTION-001`, and `SEC-TDRP-DEPENDABOT-NO-SECURITY-CLEARANCE-001`.


### Repository changelog Pages generator qualification

Qualify fixed-U `.github/scripts/generate_changelog.py@6cdd12652d31b2a47ed9a8ef906308af59cca3d0` as repository release-documentation generation only and map `TDRP-R9-REPOSITORY-CHANGELOG-PAGES-GENERATOR-001` to existing Fabushi repository release/process governance plus shared quality/evidence and `FQT-REL-001`. Negative owner controls reject a Telegram/Fabushi second changelog runtime, shipping updater ownership, Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core service, or generated Pages output as release acceptance.

Acquisition/input-output cases bind the no-argument repository-root defaults: exact UTF-8 `changelog.txt@41b7466f8a11a1866a66ff583809dfe74ad9a3c3` -> `docs/changelog/index.html`, output-directory creation, and optional exact icon inputs `icon16.png@561369d2b2688cc3799fc2b7a7a2f918d99cc155` / `icon32.png@cff636894428e42c4fa5a063e3aa472c4bded8fc`. Separate operator cases cover argv input/output overrides without treating them as remote product input. Missing icons are an allowed degraded output; source decode, destination write/stat, and copy failures are non-success.

Parsing cases cover `major.minor`, `major.minor.patch`, optional alpha/beta/dev/stable labels, 2- versus 4-digit years, ignored prelude text, leading/trailing blank removal, the exact stray `),` filter, preserved body text, and invalid month/date behavior. Ordering cases prove the helper never sorts despite computing `sort_key`: source order is retained, index 0 becomes latest, and the last entry supplies the “since” date. Tests must therefore detect out-of-order input rather than credit the helper with correcting it. The generator is not a version allocator or monotonic-version validator.

Formatting cases require `- ` and `\u2014 ` bullets to become escaped list items, other non-empty lines to become escaped paragraphs, blanks to terminate an active list, stable labels to be hidden, prerelease labels to remain visible, and numeric-dot anchors to remain stable. HTML-injection controls place markup/script-like text in changelog bodies/tags and require escaped output with no executable content. Search and anchor/copy JavaScript are static-page behavior only.

Caller/source-lineage cases bind exact `.github/workflows/changelog.yml@445277fe4b44dc9ccef17b7b602016572c08fe7c`, its `dev` + `changelog.txt` push filter/manual-dispatch triggers, Python 3.12 setup, no-argument invocation, whole-`docs` Pages artifact upload, deploy dependency, Pages/OIDC permissions, and version-tag action references. Negative evidence rejects treating `@v7`/`@v5` tags, a successful Pages deployment, the footer's live `dev/changelog.txt` link, or generated HTML as immutable source provenance or Fabushi release acceptance.

Failure/recovery cases cover malformed month input, empty/no recognized entries, out-of-order entries, missing/moved input, unreadable UTF-8, unwritable output, icon-copy failure, stale exact blobs, and caller/action-reference drift. Recovery must restart from exact source/input/caller identities and preserve prior changelog history; it may not silently rewrite content, widen path authority, execute Stage-A workflow/deploy paths, or mark evidence verified.

Authored, not run in Stage A: `CONTRACT-TDRP-CHANGELOG-GENERATOR-ACQUISITION-001`, `CONTRACT-TDRP-CHANGELOG-GENERATOR-VERSION-PARSE-001`, `CONTRACT-TDRP-CHANGELOG-GENERATOR-FILTERING-001`, `CONTRACT-TDRP-CHANGELOG-GENERATOR-ORDERING-001`, `CONTRACT-TDRP-CHANGELOG-GENERATOR-HTML-FORMAT-001`, `CONTRACT-TDRP-CHANGELOG-GENERATOR-INPUT-OUTPUT-001`, `CONTRACT-TDRP-CHANGELOG-GENERATOR-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-CHANGELOG-GENERATOR-PAGES-CALLER-001`, `INT-TDRP-CHANGELOG-GENERATOR-RELEASE-PROCESS-001`, `FAULT-TDRP-CHANGELOG-GENERATOR-MALFORMED-DATE-001`, `FAULT-TDRP-CHANGELOG-GENERATOR-IO-RECOVERY-001`, `SEC-TDRP-CHANGELOG-GENERATOR-HTML-ESCAPE-001`, and `SEC-TDRP-CHANGELOG-GENERATOR-PATH-AUTHORITY-001`. No helper execution, workflow run, Pages upload/deploy, build, test, validator, CI, package, sign, release or acceptance is executed by this qualification.


### Repository canary Bot API container qualification

Qualify fixed-U `.github/telegram-bot-api/Dockerfile@a361602441040028a3332c3732c4902d9c880122` as source canary-publication container/build provenance and map `TDRP-R9-REPOSITORY-CANARY-BOT-API-CONTAINER-001` to existing Fabushi repository build/container/toolchain provenance, dependency-advisory / supply-chain quality, repository process/security governance, shared quality/evidence and release gates. Negative owner controls reject a Telegram Bot API sidecar/provider, product runtime service, second updater/release service, platform-core service, Agent/Coordinator/Host/Runner ownership, or a copied Telegram canary workflow as the target implementation.

Base/build cases bind both `FROM` stages to exact `ubuntu:22.04@sha256:2edbbc5dc405e9612ba3584ce95480277e3eb374407b5505fe26f17df77c7dbc`; preserve the exact build package set; prove that distro `apt-get update/install` remains mutable network acquisition despite the base digest; require non-empty `TELEGRAM_BOT_API_REF`; bind clone origin `https://github.com/tdlib/telegram-bot-api.git`, checkout of the requested object, recursive submodules, logged resolved HEAD/submodule state, Release CMake configuration, `/out` install prefix, and installed binary handoff. Caller qualification separately proves exact `.github/workflows/canary-bot-api.yml@3ef5e601948c8f3256464d0d31816eebf4ba41a9` validates a full 40-lowercase-hex commit before the Docker build and uses SHA-pinned checkout/login/build-push actions.

Runtime cases bind exact runtime packages `ca-certificates libssl3 zlib1g`, system user `botapi` UID 999, owned `/data`, copied binary, exact `entrypoint.sh@d40a7372efd8e5862e00f9b214b36a840688797e`, non-root `USER botapi`, `WORKDIR /data`, exposed 8081, and exec-form entrypoint. Entry-point cases require PID-1 exec of `telegram-bot-api` with port 8081, `/data`, `/data/temp`, conditional `--local`, and argv forwarding. Credential controls prove API ID/hash are runtime-only environment inputs and must never appear in Docker build args/layers/artifacts.

Source-lineage/consumer cases bind the manual-dispatch `canary` environment, contents-read/packages-write permissions, exact context/build arg, GHCR push, and resulting digest. They then bind fixed-U `.github/workflows/canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947`: publication must reject a non-digest `CANARY_BOT_API_IMAGE`; exactly the win64/mac/linux publish jobs consume the source-built image as a service with runtime API credentials, `TELEGRAM_LOCAL=1`, runner port 8081 and `BOT_API=http://localhost:8081`. Integration qualification binds `Telegram/build/canary_publish.sh@4eb2d790063bb8f11efc7ffa06977fe1ef79a270` readiness and upload consumer behavior without turning Telegram publication infrastructure into Fabushi release authority.

Failure/recovery cases cover empty/non-commit ref, clone/checkout/submodule failure, apt/package drift or unavailable indexes, CMake/install failure, missing or non-executable build-context entrypoint, image-push failure, malformed/unset digest reference, service startup failure, readiness timeout, missing runtime credentials, and stale source/image identities. Security negatives reject third-party image substitution, mutable-tag consumption, root/privileged fallback, secret baking, treating base digest as full dependency immutability, or treating a successful image build/push as release acceptance.

Authored, not run in Stage A: `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-BASE-IMAGE-001`, `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-BUILD-ACQUISITION-001`, `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-BUILD-INSTALL-001`, `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-RUNTIME-IDENTITY-001`, `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-ENTRYPOINT-001`, `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-NETWORK-001`, `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-INPUT-OUTPUT-001`, `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-CONSUMER-001`, `INT-TDRP-CANARY-BOT-API-CONTAINER-REPOSITORY-PROCESS-001`, `FAULT-TDRP-CANARY-BOT-API-CONTAINER-REF-UNRESOLVED-001`, `FAULT-TDRP-CANARY-BOT-API-CONTAINER-ACQUISITION-FAILURE-001`, `FAULT-TDRP-CANARY-BOT-API-CONTAINER-SERVICE-READINESS-001`, `FAULT-TDRP-CANARY-BOT-API-CONTAINER-DIGEST-DRIFT-001`, `SEC-TDRP-CANARY-BOT-API-CONTAINER-NONROOT-001`, `SEC-TDRP-CANARY-BOT-API-CONTAINER-SECRET-BOUNDARY-001`, `SEC-TDRP-CANARY-BOT-API-CONTAINER-MUTABLE-ACQUISITION-001`, and `SEC-TDRP-CANARY-BOT-API-CONTAINER-NO-THIRD-PARTY-IMAGE-001`. No Docker/BuildKit, apt, git/submodule, CMake, image registry, service container, Bot API, workflow, validator, build, test, package, release or acceptance execution is permitted by this Stage-A qualification.


### Repository canary Bot API runtime-entrypoint qualification

`TDRP-R9-REPOSITORY-CANARY-BOT-API-ENTRYPOINT-001` qualifies fixed-U `.github/telegram-bot-api/entrypoint.sh@d40a7372efd8e5862e00f9b214b36a840688797e` as repository container runtime-entrypoint provenance only. Source identity is recursive-tree index 115 zero-based, mode 100755, 322 ASCII bytes/code points, 11 LF newline characters, zero CR characters, and a trailing newline. Existing-owner-first maps it to current repository build/container/toolchain provenance plus process/security and shared quality/evidence/release governance; it must not create a Bot API sidecar/provider, product runtime, second release service, platform-core service, or a new credential/network owner.

Authored, not run in Stage A:
- `CONTRACT-TDRP-CANARY-BOT-API-ENTRYPOINT-SHELL-001` — exact interpreter, `set -e`, one absolute-path `exec`, and absence of shell-owned supervisor/retry semantics are preserved.
- `CONTRACT-TDRP-CANARY-BOT-API-ENTRYPOINT-FIXED-ARGV-001` — fixed leading argv is exactly binary + port 8081 + `/data` + `/data/temp`.
- `CONTRACT-TDRP-CANARY-BOT-API-ENTRYPOINT-LOCAL-MODE-001` — unset/empty `TELEGRAM_LOCAL` omits `--local`; every non-empty value, including `0`, adds exactly one `--local`.
- `CONTRACT-TDRP-CANARY-BOT-API-ENTRYPOINT-ENV-INHERITANCE-001` — API ID/hash are inherited by the binary and are neither validated nor printed/transformed by the shell.
- `CONTRACT-TDRP-CANARY-BOT-API-ENTRYPOINT-ARGV-FORWARDING-001` — quoted `"$@"` preserves argument boundaries and does not introduce shell re-evaluation; appended arguments remain trusted operator input.
- `CONTRACT-TDRP-CANARY-BOT-API-ENTRYPOINT-PID1-EXEC-001` — successful `exec` removes the shell supervisor and makes binary process lifecycle/exit status the container lifecycle.
- `CONTRACT-TDRP-CANARY-BOT-API-ENTRYPOINT-NETWORK-001` — only HTTP port 8081 is fixed by this script; no bind-address/TLS/firewall/auth claim may be invented.
- `CONTRACT-TDRP-CANARY-BOT-API-ENTRYPOINT-DATA-PATHS-001` — persistent working path and temp path are exactly `/data` and `/data/temp`; the shell creates neither.
- `CONTRACT-TDRP-CANARY-BOT-API-ENTRYPOINT-SOURCE-LINEAGE-001` — exact entrypoint blob is bound to exact Dockerfile COPY/ENTRYPOINT, exact build workflow, digest-pinned canary image consumption, and fixed-U tree.
- `CONTRACT-TDRP-CANARY-BOT-API-ENTRYPOINT-CONSUMER-001` — three publish jobs inject runtime API credentials/local mode, map 8081, and use the external publish-script readiness owner without moving those responsibilities into the shell.
- `INT-TDRP-CANARY-BOT-API-ENTRYPOINT-REPOSITORY-PROCESS-001` — entrypoint provenance composes with existing container/toolchain, supply-chain, process/security, evidence and release owners without duplicate runtime ownership.
- `FAULT-TDRP-CANARY-BOT-API-ENTRYPOINT-EXEC-FAILURE-001` — missing/non-executable absolute server binary or failed exec is a hard startup failure; no fallback binary/PATH lookup is allowed.
- `FAULT-TDRP-CANARY-BOT-API-ENTRYPOINT-DATA-PATH-001` — unwritable/unusable `/data` or `/data/temp` remains non-passing rather than redirecting to another path.
- `FAULT-TDRP-CANARY-BOT-API-ENTRYPOINT-BINARY-STARTUP-001` — invalid runtime API inputs or other binary startup rejection propagates as service failure; the shell does not manufacture readiness.
- `FAULT-TDRP-CANARY-BOT-API-ENTRYPOINT-READINESS-001` — process start without successful bounded downstream `getMe` readiness remains non-passing.
- `SEC-TDRP-CANARY-BOT-API-ENTRYPOINT-NO-EVAL-001` — extra argv is forwarded with `"$@"`, never concatenated into `eval`, `sh -c`, or untrusted shell source.
- `SEC-TDRP-CANARY-BOT-API-ENTRYPOINT-SECRET-BOUNDARY-001` — the shell must not log/bake API ID/hash; runtime environment inheritance is not release evidence.
- `SEC-TDRP-CANARY-BOT-API-ENTRYPOINT-ARGV-TRUST-BOUNDARY-001` — appended argv is trusted deployment/operator authority only; no untrusted product request may gain arbitrary binary-option control.
- `SEC-TDRP-CANARY-BOT-API-ENTRYPOINT-NONROOT-COMPOSITION-001` — non-root UID 999 is a Dockerfile/runtime composition requirement, not a shell privilege-drop claim.

Stage-B witnesses, when the candidate is frozen and executable validation is authorized, must distinguish shell tokenization from binary semantics: exact process argv, local-mode presence cases, signal/exit propagation, filesystem ownership/creation, network bind/port observations, secret non-disclosure, and bounded readiness. Until those witnesses exist, status remains `validation_status=not-run`, `accepted=false`. Stage A executes none of them.


### Repository canary Bot API image-build workflow qualification

`TDRP-R9-REPOSITORY-CANARY-BOT-API-BUILD-WORKFLOW-001` qualifies fixed-U `.github/workflows/canary-bot-api.yml@3ef5e601948c8f3256464d0d31816eebf4ba41a9` as repository CI/container-image provenance only. Source identity is recursive-tree index 117 zero-based, mode 100644, 2,364 ASCII bytes/code points, 68 LF characters, zero CR, trailing newline; target ownership is existing repository build/container/toolchain + supply-chain/process/security + shared quality/evidence/release governance.

Authored, not run in Stage A:
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-DISPATCH-001` — only `workflow_dispatch` exists and requires one `ref` input; no automatic trigger or workflow-call surface is invented.
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-REF-VALIDATION-001` — `REF` must match exactly 40 lowercase hex before later success-path steps; syntax validation is not commit-existence validation.
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-PERMISSIONS-001` — workflow permissions remain `contents: read` + `packages: write`; no broader token scope is inferred.
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-ENVIRONMENT-001` — build job uses `environment: canary`; actual environment branch/ref protection and approvals are external evidence, not source-code assumptions.
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-CHECKOUT-PIN-001` — checkout uses exact commit `3d3c42e5aac5ba805825da76410c181273ba90b1`.
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-GHCR-AUTH-001` — GHCR login uses `github.actor` plus scoped `GITHUB_TOKEN`, and that token is not a Docker build argument.
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-BUILD-CONTEXT-001` — build context is exact tree `050b40e726444074e504dadfa3701eedb58bc3e3` containing only exact Dockerfile and entrypoint blobs.
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-BUILD-ARGS-001` — the only declared build arg is `TELEGRAM_BOT_API_REF` bound to the validated dispatch input.
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-IMAGE-TAG-001` — push target/tag is exactly owner GHCR `telegram-bot-api:<ref>`; tag is mutable and not a release trust anchor.
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-DIGEST-HANDOFF-001` — successful push digest is printed as an owner GHCR digest reference and requires explicit operator transfer to `CANARY_BOT_API_IMAGE`; no automatic variable update is claimed.
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-SOURCE-LINEAGE-001` — exact workflow, three action commits, context tree, Dockerfile, entrypoint, consumer workflow and downstream publish script stay bound as one provenance chain.
- `CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-CONSUMER-001` — fixed-U canary publishing validates a digest-pinned GHCR variable and three publish jobs consume it with `packages: read` and runtime-only API credentials.
- `INT-TDRP-CANARY-BOT-API-WORKFLOW-REPOSITORY-PROCESS-001` — workflow provenance composes with existing Fabushi repository container/toolchain/security/evidence/release owners without importing the source service or workflow.
- `FAULT-TDRP-CANARY-BOT-API-WORKFLOW-INVALID-REF-001` — malformed/uppercase/abbreviated ref stops before checkout/build.
- `FAULT-TDRP-CANARY-BOT-API-WORKFLOW-CHECKOUT-OR-LOGIN-001` — checkout or GHCR-auth failure prevents build/push/digest handoff.
- `FAULT-TDRP-CANARY-BOT-API-WORKFLOW-UPSTREAM-REF-UNRESOLVED-001` — syntactically valid but nonexistent/unusable upstream ref fails in Dockerfile acquisition and is not silently substituted.
- `FAULT-TDRP-CANARY-BOT-API-WORKFLOW-BUILD-PUSH-001` — build or registry push failure yields no qualified digest handoff.
- `FAULT-TDRP-CANARY-BOT-API-WORKFLOW-DIGEST-HANDOFF-001` — absent/stale/manual variable transfer or non-digest consumer value remains non-passing and cannot fall back to a tag/third-party image.
- `SEC-TDRP-CANARY-BOT-API-WORKFLOW-TOKEN-SCOPE-001` — builder requires only source-read/package-write token capability; consumer later uses package-read and runtime secrets separately.
- `SEC-TDRP-CANARY-BOT-API-WORKFLOW-ACTION-PINS-001` — all three external Actions remain full-SHA pinned; floating tags are rejected as equivalent provenance.
- `SEC-TDRP-CANARY-BOT-API-WORKFLOW-INPUT-BOUNDARY-001` — dispatch input reaches shell through environment and build through an action field only after prior validation; it is not accepted as arbitrary shell command text.
- `SEC-TDRP-CANARY-BOT-API-WORKFLOW-RUNNER-TRUST-001` — custom Depot runner and canary environment controls require independent approved-infrastructure evidence before acceptance.
- `SEC-TDRP-CANARY-BOT-API-WORKFLOW-DIGEST-TRUST-001` — digest-pinned GHCR reference, not the ref-shaped tag, is the consumer trust boundary; third-party substitution is rejected.

Frozen-candidate Stage-B execution, if still applicable to the target release process, must bind exact H/U/action pins/context tree, approved runner/environment configuration, GITHUB_TOKEN scopes, pushed manifest digest, operator variable transition, consumer digest validation, and downstream service readiness. Stage A does not execute any of those gates.


### Repository canary release/publication workflow qualification

`TDRP-R9-REPOSITORY-CANARY-RELEASE-WORKFLOW-001` qualifies fixed-U `.github/workflows/canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947` as source repository build/sign/package/artifact/publication provenance. It is not copied into Fabushi. Existing target owners are `desktop/package.json`, `.github/workflows/release-macos-main.yml`, `desktop/scripts/notarize-after-sign.cjs`, existing updater/package/release owners, root repository process/security governance and this shared quality/evidence system. Stage A authors the following cases and executes none of them.

Authored contract cases, not run:
- `CONTRACT-TDRP-CANARY-WORKFLOW-TRIGGER-LANE-001` — push-only public/private canary trigger and exact repo-visibility/branch lane admission remain source-exact; sibling workflows merely exclude these branches.
- `CONTRACT-TDRP-CANARY-WORKFLOW-CONCURRENCY-001` — one repository-level canary-publish group serializes runs without cancelling the in-progress run.
- `CONTRACT-TDRP-CANARY-WORKFLOW-ENVIRONMENT-001` — all jobs name the canary environment, while actual approval/ref policy remains external evidence.
- `CONTRACT-TDRP-CANARY-WORKFLOW-PERMISSIONS-001` — global/job permissions remain least-scoped by responsibility: contents read, actions read only for version, id-token only pack/sign jobs, packages read only publishers.
- `CONTRACT-TDRP-CANARY-WORKFLOW-TIP-FENCE-001` — the triggering SHA must still be the remote branch tip before version/publication proceeds.
- `CONTRACT-TDRP-CANARY-WORKFLOW-VERSION-COUNTER-001` — three-component display version, run-number counter plus validated nonnegative base, lane suffix/folder and seven-character source commit are derived exactly once.
- `CONTRACT-TDRP-CANARY-WORKFLOW-PUBLISH-GATE-001` — absence of bot token cleanly disables publication; publication requires channel id, three metadata ids and a digest-pinned Bot API image.
- `CONTRACT-TDRP-CANARY-WORKFLOW-UNSIGNED-OVERRIDE-001` — platform signatures are mandatory for publishing unless the explicit CANARY_ALLOW_UNSIGNED=1 bring-up override is set; v2 envelope signing remains distinct.
- `CONTRACT-TDRP-CANARY-WORKFLOW-BOT-IMAGE-DIGEST-001` — consumer accepts only owner/other GHCR syntax bound to sha256 digest; no mutable-tag fallback.
- `CONTRACT-TDRP-CANARY-WORKFLOW-ACTION-PINS-001` — all external Actions listed by the workflow remain exact full-SHA identities.
- `CONTRACT-TDRP-CANARY-WORKFLOW-WINDOWS-BUILD-SIGN-001` — Release x64/LTO build, source/canary defines, cache/prepare chain, Breakpad symbols, KeyLocker signing and publish-side Authenticode witness preserve their boundaries.
- `CONTRACT-TDRP-CANARY-WORKFLOW-MACOS-BUILD-UNIVERSAL-001` — two thin architecture builds share exact toolchain identity and must assemble a universal app with executable-architecture coverage.
- `CONTRACT-TDRP-CANARY-WORKFLOW-MACOS-SIGN-NOTARY-001` — Developer ID codesign, notarization, staple and spctl build-side witnesses remain distinct from the weaker Linux publish-side _CodeSignature presence check.
- `CONTRACT-TDRP-CANARY-WORKFLOW-LINUX-BUILD-TRUST-001` — nested-submodule/mirror, generated build image, Release+LTO build and exact test_update_verify/Packer fixture source quality checks remain explicit.
- `CONTRACT-TDRP-CANARY-WORKFLOW-UPDATE-ENVELOPE-001` — Packer signing-input -> Azure Key Vault ES256 -> embedded v2 signature is required for publishing; build-only runs retain unsigned envelope output.
- `CONTRACT-TDRP-CANARY-WORKFLOW-ARTIFACT-HANDOFF-001` — named per-platform update/portable artifacts and mac thin transport have exact producer/consumer paths and 7-day/1-day retention.
- `CONTRACT-TDRP-CANARY-WORKFLOW-PUBLISH-JOB-ISOLATION-001` — each platform publishes independently after its own build and is sole writer of its metadata message; one slow/failing platform does not block another platform's publish job once prerequisites differ.
- `CONTRACT-TDRP-CANARY-WORKFLOW-BOT-API-SERVICE-001` — three publication jobs consume the separately built digest-pinned service with package-read auth, runtime API id/hash/local mode and localhost:8081.
- `CONTRACT-TDRP-CANARY-WORKFLOW-PUBLISH-SCRIPT-001` — exact canary_publish.sh remains readiness/publication owner; workflow does not duplicate its bounded getMe/message contract.
- `CONTRACT-TDRP-CANARY-WORKFLOW-SOURCE-LINEAGE-001` — fixed-U workflow, helper blobs, action commits, package artifacts, digest-pinned service producer and publisher stay one traceable chain.
- `CONTRACT-TDRP-CANARY-WORKFLOW-SECRETS-NETWORK-001` — credential scopes and mutable network acquisitions are recorded without treating runner masking or package resolution as source-proven runtime proof.
- `CONTRACT-TDRP-CANARY-WORKFLOW-SYMBOL-PROVENANCE-001` — Breakpad symbols are generated but R2 upload is only TODO/comment and must remain an explicit non-implemented source edge.

Authored integration case, not run:
- `INT-TDRP-CANARY-WORKFLOW-REPOSITORY-RELEASE-001` — source workflow guarantees compose with existing Fabushi package/release/notarize/updater/process/security/evidence owners without a Telegram-specific workflow/runtime or second release truth.

Authored fault/recovery cases, not run:
- `FAULT-TDRP-CANARY-WORKFLOW-LANE-REJECT-001` — wrong event/ref/lane/repository-visibility combination is fail-closed.
- `FAULT-TDRP-CANARY-WORKFLOW-STALE-TIP-001` — rerunning a non-tip commit cannot mint a newer canary version.
- `FAULT-TDRP-CANARY-WORKFLOW-VERSION-CONFIG-001` — malformed display version/counter/channel/message-id inputs are non-passing when their gates apply.
- `FAULT-TDRP-CANARY-WORKFLOW-BUILD-PLATFORM-001` — checkout/submodule/toolchain/cache/prepare/build/package failure blocks only the dependent path and is never reinterpreted as success.
- `FAULT-TDRP-CANARY-WORKFLOW-SIGNING-REQUIRED-001` — missing platform-signing or update-signing prerequisites fail publishing unless the explicitly modeled unsigned bring-up exception applies.
- `FAULT-TDRP-CANARY-WORKFLOW-BOT-IMAGE-DIGEST-001` — missing/stale/malformed Bot API image digest prevents publication rather than falling back to a tag or third-party image.
- `FAULT-TDRP-CANARY-WORKFLOW-ARTIFACT-HANDOFF-001` — missing/wrong/expired named artifact blocks the dependent assembly/publish job.
- `FAULT-TDRP-CANARY-WORKFLOW-SERVICE-READINESS-001` — container process existence is insufficient; bounded downstream readiness/publication failure remains non-passing.
- `FAULT-TDRP-CANARY-WORKFLOW-PREVIOUS-RUN-LOOKUP-001` — previous-run API lookup failure may degrade changelog range to empty but must not bypass any release/security gate.

Authored security cases, not run:
- `SEC-TDRP-CANARY-WORKFLOW-LEAST-PRIVILEGE-001` — job token scopes do not expand beyond their declared source/read/actions/OIDC/package needs.
- `SEC-TDRP-CANARY-WORKFLOW-ACTION-PINS-001` — full-SHA external action identity is mandatory; floating tags are not equivalent provenance.
- `SEC-TDRP-CANARY-WORKFLOW-RUNNER-ENVIRONMENT-TRUST-001` — Depot runner and GitHub environment policies require independent frozen-candidate evidence.
- `SEC-TDRP-CANARY-WORKFLOW-OIDC-SCOPE-001` — id-token write is available only to Windows/macOS-universal/Linux update-signing jobs.
- `SEC-TDRP-CANARY-WORKFLOW-SECRET-BOUNDARY-001` — API, registry, platform-signing, Apple and Azure credentials stay within their intended jobs and must not appear in artifacts/logs/config outputs.
- `SEC-TDRP-CANARY-WORKFLOW-DIGEST-TRUST-001` — Bot API service consumption remains sha256-digest anchored despite mutable producer tags.
- `SEC-TDRP-CANARY-WORKFLOW-UNSIGNED-OVERRIDE-001` — CANARY_ALLOW_UNSIGNED is a deliberate high-risk bring-up exception and must be explicit/evidenced, never silently inferred.
- `SEC-TDRP-CANARY-WORKFLOW-MUTABLE-ACQUISITION-001` — apt/Homebrew/NuGet/submodule/mirror/custom-runner inputs are not upgraded to immutable provenance by action/source SHA pins.
- `SEC-TDRP-CANARY-WORKFLOW-NO-SOURCE-PUBLISHER-IMPORT-001` — Fabushi absorbs release guarantees into existing owners and does not import Telegram Bot API/channel publication or a second updater/release credential stack.

Stage-B qualification, only after a frozen candidate exists, must bind exact source/target identities to authorized environment/ref admission, runner images/trust, action SHAs, dependency/cache identities where material, platform build/package outputs, signing/notary/Key Vault witnesses, artifact ids/digests, Bot API service digest/readiness, publication result, secret non-disclosure and the current Fabushi release/update evidence model. Source-side Linux `test_update_verify` and `canary_test_fixtures.sh` are recorded as upstream quality semantics only; they are not executed or treated as Fabushi verification during Stage A. R2 symbol upload and KeyLocker installation placeholders remain gaps rather than passing tests. `validation_status=not-run`; `accepted=false`.


### Repository cant-reproduce issue-lifecycle workflow qualification

`TDRP-R9-REPOSITORY-CANT-REPRODUCE-WORKFLOW-001` qualifies fixed-U `.github/workflows/cant-reproduce.yml@8e20e07349b5b43277f5371a4e931066bece4186` as upstream repository issue-triage/lifecycle provenance. Existing target owners are the root repository issue/reporting/development-process authority, repository privacy/security governance and this shared quality/evidence system. The workflow is not copied into Fabushi. Stage A authors the following cases and executes none of them.

Authored contract cases, not run:
- `CONTRACT-TDRP-CANT-REPRODUCE-TRIGGER-001` — only daily `0 3 * * *` schedule is admitted; no issue/issue_comment/manual/callable trigger is invented.
- `CONTRACT-TDRP-CANT-REPRODUCE-LABEL-ROUTING-001` — only open issues carrying exact label `cant reproduce` enter the source search.
- `CONTRACT-TDRP-CANT-REPRODUCE-AGE-CUTOFF-001` — omitted input uses Action default 14 days measured from the latest matching labeled-event timestamp, not from `updated_at` or comment age.
- `CONTRACT-TDRP-CANT-REPRODUCE-SEARCH-WINDOW-001` — source search is updated-descending with `per_page=30` and no search pagination; this limitation is preserved as evidence rather than silently repaired in provenance.
- `CONTRACT-TDRP-CANT-REPRODUCE-CLOSE-COMMENT-001` — configured static closure explanation remains exact source copy; Telegram support redirection never becomes Fabushi support authority.
- `CONTRACT-TDRP-CANT-REPRODUCE-STATE-MUTATION-001` — close path posts the optional comment before changing issue state to closed.
- `CONTRACT-TDRP-CANT-REPRODUCE-PERMISSIONS-001` — workflow has no explicit permissions block; effective `github.token` scope is frozen-candidate host evidence, not inferred least privilege.
- `CONTRACT-TDRP-CANT-REPRODUCE-RUNNER-001` — single ubuntu-slim job has no environment, concurrency, timeout, retry or checkout contract.
- `CONTRACT-TDRP-CANT-REPRODUCE-ACTION-LINEAGE-001` — mutable v0.5.0 tag and observed commit/action/bundle identities remain distinct; only an exact frozen candidate may qualify actual Action bytes.
- `CONTRACT-TDRP-CANT-REPRODUCE-INPUT-BOUNDARY-001` — token is required; label/comment are literal source inputs; omitted days/color use Action defaults and no hidden validation is invented.
- `CONTRACT-TDRP-CANT-REPRODUCE-ISSUE-COMMENT-ABSENCE-001` — bundled unmark/reopen behavior is not attributed to this workflow because `issue_comment` is absent.
- `CONTRACT-TDRP-CANT-REPRODUCE-CONSUMER-001` — GitHub scheduler, external Action, GitHub Issues APIs/state, maintainers and reporters are the complete direct process chain.
- `CONTRACT-TDRP-CANT-REPRODUCE-NO-TARGET-WORKFLOW-COPY-001` — target remains source-neutral existing repository governance; no Telegram automation copy or product runtime is introduced.

Authored integration case, not run:
- `INT-TDRP-CANT-REPRODUCE-REPOSITORY-ISSUE-PROCESS-001` — issue-label ownership, lifecycle mutation, privacy/security review and evidence compose through the existing repository process owner.

Authored fault/recovery cases, not run:
- `FAULT-TDRP-CANT-REPRODUCE-MISSING-PERMISSION-001` — insufficient issue/label/comment permission fails closed and is never called successful triage.
- `FAULT-TDRP-CANT-REPRODUCE-LABEL-LOOKUP-CREATE-001` — broad get-label failure and unawaited create cannot be assumed to mean the label now exists.
- `FAULT-TDRP-CANT-REPRODUCE-SEARCH-WINDOW-001` — more than 30 matching open issues exposes non-paginated/starvation behavior.
- `FAULT-TDRP-CANT-REPRODUCE-POST-LABEL-ACTIVITY-001` — a new comment/update after labeling does not by itself reset the source cutoff; qualification must not invent that safeguard.
- `FAULT-TDRP-CANT-REPRODUCE-PARTIAL-CLOSE-001` — successful close comment followed by state-update failure remains partial mutation requiring reconciliation.
- `FAULT-TDRP-CANT-REPRODUCE-ASYNC-NOT-AWAITED-001` — top-level sweep/unmark, per-issue close and label-create asynchronous completion/rejection cannot be treated as synchronously fenced.
- `FAULT-TDRP-CANT-REPRODUCE-NO-UNMARK-EVENT-001` — original-author comment cannot be credited with automatic label removal/reopen under this schedule-only workflow.

Authored security/privacy cases, not run:
- `SEC-TDRP-CANT-REPRODUCE-TOKEN-SCOPE-001` — implicit token permissions are qualified and no broader-than-needed authority is assumed.
- `SEC-TDRP-CANT-REPRODUCE-MUTABLE-ACTION-TAG-001` — version tag is never accepted as immutable source identity.
- `SEC-TDRP-CANT-REPRODUCE-THIRD-PARTY-ACTION-BOUNDARY-001` — repository token and issue mutation cross into third-party Action code only under explicit source-lineage/security qualification.
- `SEC-TDRP-CANT-REPRODUCE-ISSUE-LOG-PRIVACY-001` — issue/event debug metadata and workflow logs remain governed by repository privacy/access policy, especially for private repositories.
- `SEC-TDRP-CANT-REPRODUCE-NO-TELEGRAM-SUPPORT-AUTHORITY-001` — source close-copy/help route cannot grant Telegram support infrastructure authority inside Fabushi.

Stage-B oracles are `ORA-TDRP-CANT-REPRODUCE-EXACT-WORKFLOW-001`, `ORA-TDRP-CANT-REPRODUCE-LABEL-AGE-CLOSE-001`, `ORA-TDRP-CANT-REPRODUCE-ACTION-LINEAGE-001`, and `ORA-TDRP-CANT-REPRODUCE-FAILURE-BOUNDARY-001`. Invariants are `INV-TDRP-CANT-REPRODUCE-NO-ISSUE-COMMENT-RECOVERY-ASSUMPTION-001`, `INV-TDRP-CANT-REPRODUCE-NO-MUTABLE-TAG-TRUST-001`, `INV-TDRP-CANT-REPRODUCE-NO-TELEGRAM-SUPPORT-IMPORT-001`, and `INV-TDRP-CANT-REPRODUCE-NO-STAGE-A-EXECUTION-001`.

No source/target schedule run, issue/label/comment/state mutation, token probe, build, lint, validator, test/E2E, CI, package, deployment or acceptance is executed in Stage A.


### Repository changelog Pages workflow qualification

`TDRP-R9-REPOSITORY-CHANGELOG-PAGES-WORKFLOW-001` qualifies fixed-U `.github/workflows/changelog.yml@445277fe4b44dc9ccef17b7b602016572c08fe7c` as repository changelog-publication/Pages provenance. Existing target owners are root release/changelog/process governance, repository security/provenance governance, this shared quality/evidence owner, and final `FQT-REL-001`. No target Pages workflow, second release truth, updater owner, product runtime, or platform-core service is introduced. Stage A authors the following cases and executes none.

Authored contract cases, not run:
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-TRIGGER-001` — only dev pushes touching exact `changelog.txt` and manual dispatch are admitted.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-PATH-FILTER-001` — path filtering is exact root changelog input; unrelated dev pushes are excluded.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-PERMISSIONS-001` — workflow-level contents-read/pages-write/id-token-write applies to both jobs; source inspection does not call this least privilege.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-CONCURRENCY-001` — all runs share group `pages` and newer runs cancel in-progress predecessors.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-JOB-GRAPH-001` — deploy cannot begin before successful build/artifact production.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-BUILD-RUNNER-001` — build runs on mutable ubuntu-latest and carries no source timeout/retry declaration.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-CHECKOUT-LINEAGE-001` — checkout@v7 written ref, observed commit/action manifest and default persisted credentials remain separate evidence.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-PYTHON-LINEAGE-001` — setup-python@v7 plus `3.12` binds major/minor intent but later evidence must capture exact interpreter/runtime acquisition.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-GENERATOR-CALL-001` — exact previously-qualified helper is invoked with no argv overrides against exact changelog input.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-DOCS-SCOPE-001` — the whole `docs` tree, not only generated changelog output, enters publication scope.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-PAGES-ARTIFACT-001` — upload-pages-artifact defaults to exact name `github-pages`, one-day retention and hidden-file exclusion; fixed-U docs has no tracked symlink.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-DEPLOY-ENVIRONMENT-001` — deploy targets exact `github-pages` environment after build.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-DEPLOY-OIDC-001` — deploy-pages requires id-token and pages write; actual token/environment policy is later host evidence.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-DEPLOY-STATUS-001` — observed deploy action creates deployment, polls bounded status and treats terminal error/cancel as non-success.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-PAGE-URL-001` — environment URL is only the deploy step's returned page_url for that deployment.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-CONSUMERS-001` — Actions admission, generator, artifact service, Pages deployment/environment and Pages readers form the direct chain.
- `CONTRACT-TDRP-CHANGELOG-WORKFLOW-NO-TARGET-COPY-001` — Fabushi absorbs guarantees into existing release/process/security/quality owners rather than copying Telegram Pages automation.

Integration, not run:
- `INT-TDRP-CHANGELOG-WORKFLOW-REPOSITORY-RELEASE-001` — frozen-candidate evidence must compose exact source/action/interpreter/artifact/Pages identities with current Fabushi release evidence without treating source Pages success as target release acceptance.

Fault/recovery cases, not run:
- `FAULT-TDRP-CHANGELOG-WORKFLOW-CONCURRENT-CANCEL-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-CHECKOUT-SETUP-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-GENERATOR-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-ARCHIVE-UPLOAD-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-ARTIFACT-IDENTITY-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-DEPLOY-CREATE-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-DEPLOY-POLL-TIMEOUT-001`
- `FAULT-TDRP-CHANGELOG-WORKFLOW-PUBLICATION-SCOPE-001`

Security/provenance cases, not run:
- `SEC-TDRP-CHANGELOG-WORKFLOW-LEAST-PRIVILEGE-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-OIDC-JOB-SCOPE-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-CHECKOUT-CREDENTIALS-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-MUTABLE-ACTION-TAGS-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-RUNNER-PYTHON-MUTABILITY-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-WHOLE-DOCS-SCOPE-001`
- `SEC-TDRP-CHANGELOG-WORKFLOW-NO-SOURCE-PUBLISH-AUTHORITY-001`

Stage-B oracles must bind exact workflow/helper/input/action SHAs, exact resolved Python and runner identity, actual job token/environment policy, artifact id/name/digest/content manifest, deployment id/status/page URL and publication-scope review. A cancelled or partial deployment, mutable-tag observation, generated page, or source Pages success is not reusable target acceptance. `validation_status=not-run`; `accepted=false`.


### Repository copyright-year updater workflow qualification

`TDRP-R9-REPOSITORY-COPYRIGHT-YEAR-UPDATER-WORKFLOW-001` qualifies fixed-U `.github/workflows/copyright_year_updater.yml@7ddb3d0403a77812df954d8bb2f480672831a082` as repository source-maintenance provenance. Existing target owners are root source-maintenance/process governance, repository security/provenance governance, this shared quality/evidence owner, and final release/legal review where applicable. No target updater workflow, product runtime, second legal/version truth, or platform-core service is introduced. Stage A authors the following cases and executes none.

Authored contract cases, not run:
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-TRIGGER-001` — only exact repository_dispatch type and annual 03:00 UTC January-1 schedule are admitted.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-DISPATCH-CALLER-001` — fixed-U sibling workflows contain no direct dispatch/license-year caller; external dispatch identity remains host evidence.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-PERMISSIONS-001` — no explicit permissions block exists; effective github.token scopes must be captured later.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-RUNNER-001` — single ubuntu-slim job has no environment/concurrency/timeout/retry/checkout declaration.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-ACTION-LINEAGE-001` — mutable action master written ref is separated from observed commit/action/index blob identities.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-RUNTIME-DEPS-001` — npm init plus unversioned runtime package installs remain mutable acquisition evidence.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-TOKEN-DELIVERY-001` — github.token delivery through Node argv and token-authenticated Git URL is an explicit credential boundary.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-YEAR-DETECTION-001` — year is local-process getFullYear and previous year is current minus one.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-MATCH-REPLACE-001` — case-insensitive phrase presence gates a single unanchored first numeric-range replacement per text file.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-FILE-SCAN-001` — scan recursively covers non-.git working-tree paths and skips binary files; fixed-U has no tracked symlink.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-GITLINK-SCOPE-001` — 36 fixed-U gitlinks exist but license-year clone does not recurse submodules, so gitlink contents are outside scan payload.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-NOOP-001` — zero modified files is a successful source no-op, not target legal/release acceptance.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-BRANCH-001` — branch copyright_to_<year> is based on the runtime-cloned default branch and existing branch is rejected.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-COMMIT-PUSH-001` — all changes are staged, committed with deterministic message and pushed through token-authenticated origin.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-PR-PUBLICATION-001` — PR publication uses copyright branch head and cloned current branch base after branch push.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-PR-ERROR-BOUNDARY-001` — PR list/create errors are logged and swallowed, preserving possible branch-without-PR partial success.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-CONSUMERS-001` — scheduler/external dispatch, external Action, Git clone/files, branch/PR APIs and maintainers are the direct process chain.
- `CONTRACT-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-TARGET-COPY-001` — Fabushi absorbs guarantees into existing repository governance rather than copying Telegram automation.

Integration, not run:
- `INT-TDRP-COPYRIGHT-YEAR-WORKFLOW-REPOSITORY-SOURCE-MAINTENANCE-001` — exact source/action/runtime-acquisition/mutation/branch/PR evidence composes through existing Fabushi repository source-maintenance, provenance/security and release/legal review without creating a second automation authority.

Fault/recovery cases, not run:
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-DISPATCH-TYPE-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-PERMISSION-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-MUTABLE-MASTER-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-NPM-ACQUISITION-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-YEAR-TIMEZONE-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-WRONG-FIRST-RANGE-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-MODIFIED-FILES-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-BRANCH-EXISTS-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-COMMIT-PUSH-001`
- `FAULT-TDRP-COPYRIGHT-YEAR-WORKFLOW-PR-SWALLOWED-001`

Security/provenance cases, not run:
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-IMPLICIT-TOKEN-SCOPE-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-MUTABLE-ACTION-MASTER-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-UNPINNED-NPM-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-TOKEN-ARGV-REMOTE-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-BROAD-TEXT-SCAN-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-BRANCH-PR-WRITE-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-SOURCE-MAINTENANCE-AUTHORITY-001`
- `SEC-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-STAGE-A-EXECUTION-001`

Stage-B oracles are `ORA-TDRP-COPYRIGHT-YEAR-WORKFLOW-EXACT-SOURCE-001`, `ORA-TDRP-COPYRIGHT-YEAR-WORKFLOW-ACTION-LINEAGE-001`, `ORA-TDRP-COPYRIGHT-YEAR-WORKFLOW-MUTATION-PUBLICATION-001`, and `ORA-TDRP-COPYRIGHT-YEAR-WORKFLOW-FAILURE-BOUNDARY-001`. Invariants are `INV-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-MUTABLE-MASTER-TRUST-001`, `INV-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-HOST-PERMISSION-ASSUMPTION-001`, `INV-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-PARTIAL-PR-SUCCESS-001`, `INV-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-BROAD-SCAN-AUTHORITY-001`, and `INV-TDRP-COPYRIGHT-YEAR-WORKFLOW-NO-STAGE-A-EXECUTION-001`.

No source/target workflow run, dispatch, Action/npm execution, file mutation, branch/PR mutation, token probe, build, lint, validator, test/E2E, package, release, install or runtime acceptance is executed in Stage A.


### Repository Docker build/publish workflow qualification

`TDRP-R9-REPOSITORY-DOCKER-BUILD-WORKFLOW-001` qualifies fixed-U `.github/workflows/docker.yml@cba00688d935d50454bfb891fc37e001626cf9a0` as repository container-build/publication provenance. Existing target owners are repository container/build/toolchain governance, release/process governance, security/provenance governance, and shared quality/evidence gates. No target Telegram Docker workflow, second package/release service, product runtime, or platform-core service is introduced. Stage A authors the following cases and executes none.

Authored contract cases, not run:
- `CONTRACT-TDRP-DOCKER-WORKFLOW-TRIGGER-PATHS-001` — push admission is path-scoped to the workflow or centos_env subtree.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-DEFAULT-BRANCH-JOB-GATE-001` — admitted non-default pushes must skip the sole job; publication is default-branch-only by job condition.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-PERMISSIONS-001` — absence of explicit permissions means effective token scope is later host evidence.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-RUNNER-001` — mutable ubuntu-latest and no environment/concurrency/timeout/retry declaration remain explicit.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-CHECKOUT-LINEAGE-001` — written checkout v7 tag is separated from observed commit identity.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-RECURSIVE-SUBMODULES-001` — checkout recursively initializes source gitlinks before build.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-POETRY-BOOTSTRAP-001` — network installer is executed without version/digest pin; poetry lock applies only after bootstrap.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-DISK-CLEANUP-001` — exact free-disk-space commit plus tool-cache alias/default cleanup semantics are recorded.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-REGISTRY-AUTH-001` — GITHUB_TOKEN is piped to GHCR login; actual token/package policy is later evidence.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-BUILD-CONTEXT-001` — exact pyproject/lock/generator/template identities and existing acquisition inventory form the build-source closure.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-GENERATOR-ENV-001` — empty DEBUG and LTO environment values render both feature booleans false.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-BUILDKIT-001` — generated Dockerfile is stdin to BuildKit docker build and tagged only with IMAGE_TAG.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-IMAGE-TAG-001` — publication identity is mutable repository-scoped centos_env:latest.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-PUBLICATION-001` — source emits no digest, attestation, signature or rollback identity.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-CONSUMER-001` — fixed-U OSS scanner consumes exact mutable GHCR latest image.
- `CONTRACT-TDRP-DOCKER-WORKFLOW-NO-TARGET-COPY-001` — guarantees map into existing Fabushi repository owners instead of copied Telegram automation.

Integration, not run:
- `INT-TDRP-DOCKER-WORKFLOW-REPOSITORY-CONTAINER-BUILD-001` — frozen-candidate source/action/tool/bootstrap/template/acquisition/image evidence must compose with existing Fabushi build/release/security evidence without promoting source latest-tag publication to target acceptance.

Fault/recovery cases, not run:
- `FAULT-TDRP-DOCKER-WORKFLOW-NONDEFAULT-SKIP-001`
- `FAULT-TDRP-DOCKER-WORKFLOW-CHECKOUT-SUBMODULE-001`
- `FAULT-TDRP-DOCKER-WORKFLOW-POETRY-BOOTSTRAP-001`
- `FAULT-TDRP-DOCKER-WORKFLOW-DISK-CLEANUP-001`
- `FAULT-TDRP-DOCKER-WORKFLOW-REGISTRY-LOGIN-001`
- `FAULT-TDRP-DOCKER-WORKFLOW-GENERATOR-001`
- `FAULT-TDRP-DOCKER-WORKFLOW-BUILD-001`
- `FAULT-TDRP-DOCKER-WORKFLOW-PUSH-PARTIAL-001`
- `FAULT-TDRP-DOCKER-WORKFLOW-LATEST-OVERWRITE-001`

Security/provenance cases, not run:
- `SEC-TDRP-DOCKER-WORKFLOW-IMPLICIT-TOKEN-SCOPE-001`
- `SEC-TDRP-DOCKER-WORKFLOW-CHECKOUT-CREDENTIALS-001`
- `SEC-TDRP-DOCKER-WORKFLOW-MUTABLE-ACTION-TAG-001`
- `SEC-TDRP-DOCKER-WORKFLOW-CURL-TO-PYTHON-001`
- `SEC-TDRP-DOCKER-WORKFLOW-MUTABLE-BASE-ACQUISITIONS-001`
- `SEC-TDRP-DOCKER-WORKFLOW-REGISTRY-CREDENTIAL-001`
- `SEC-TDRP-DOCKER-WORKFLOW-MUTABLE-LATEST-001`
- `SEC-TDRP-DOCKER-WORKFLOW-NO-DIGEST-ATTESTATION-001`
- `SEC-TDRP-DOCKER-WORKFLOW-NO-STAGE-A-EXECUTION-001`

Stage-B oracles are `ORA-TDRP-DOCKER-WORKFLOW-EXACT-SOURCE-001`, `ORA-TDRP-DOCKER-WORKFLOW-BUILD-LINEAGE-001`, `ORA-TDRP-DOCKER-WORKFLOW-PUBLICATION-IDENTITY-001`, and `ORA-TDRP-DOCKER-WORKFLOW-FAILURE-BOUNDARY-001`. Invariants are `INV-TDRP-DOCKER-WORKFLOW-NO-IMPLICIT-PERMISSION-TRUST-001`, `INV-TDRP-DOCKER-WORKFLOW-NO-MUTABLE-TAG-ACCEPTANCE-001`, `INV-TDRP-DOCKER-WORKFLOW-NO-UNPINNED-BOOTSTRAP-TRUST-001`, `INV-TDRP-DOCKER-WORKFLOW-NO-SOURCE-RELEASE-AUTHORITY-001`, and `INV-TDRP-DOCKER-WORKFLOW-NO-STAGE-A-EXECUTION-001`.

No source/target workflow run, action/bootstrap execution, container build, registry operation, test, validation, package, release, install or runtime acceptance is executed in Stage A.
