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
