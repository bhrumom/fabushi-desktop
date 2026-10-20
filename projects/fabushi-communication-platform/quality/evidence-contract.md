# Test Evidence Contract

Status: active / normative  
Evidence ID: FQT-EVID-001  
Updated: 2026-10-07

每份 passing evidence 至少包含 target SHA、upstream/baseline SHA（若相关）、workflow/run/attempt/job/event、actual checkout SHA、suite/case IDs、fixture/environment IDs、start/end、result、artifact name/ID/digest。

Unit/property/contract 保存 machine-readable result 与 failing seed/counterexample；E2E/temporal 保存 runtime trace、semantic/DOM manifest、screenshots，关键 journey 加 video；visual 保存 baseline/actual/diff；a11y 保存 scanner + keyboard/focus；performance 保存 raw samples/time series；fault 保存 injected profile/time/recovery trace。

相关 source/spec/oracle/baseline 改变后旧证据降 historical。截图/视频存在但未关联 case/verdict 不算 pass。独立验收记录 reviewer、reviewed evidence IDs、异常与 verdict。

## Independent review evidence

独立 review evidence 还必须记录 declared change scope、reviewer role/lens、实际 checked paths/surfaces、每个 finding 的 location/severity/concrete failure/fix，以及 reviewer verdict。NOT_APPLICABLE 必须保存与 changed paths/hunks 绑定的 proof；synthesis 必须记录 merged/dropped findings 及理由。Focused re-review 必须逐项记录 previous finding 的 CONFIRMED FIXED / NOT FIXED、检查过的 adjacent callers/consumers、new findings 与最终 verdict。

### Frozen review-scope snapshot evidence

Each cross-review session records the verified repository root, scope mode, exact base commit, base subject, starting HEAD, bounded commit count/list when applicable, whether the worktree was already dirty, session identity, and creation time. Each round records an immutable tree/object identity and durable pointers/digests for the complete rename-aware binary-safe scope diff and changed-path/status manifest, plus file count and scope statistic. From round two onward also record the nearest prior snapshot identity and the fix-delta artifact/stat against that snapshot.

Scope capture must prove staging isolation from the developer's real index or an equivalent mechanism, inclusion of untracked state, and successful tree/diff acquisition. Repository/base/session parse failures, impossible commit ranges, isolated-staging failures, or missing snapshot artifacts are acquisition failures. A genuinely empty diff is recorded distinctly as an empty-scope stop and does not count as review acceptance. Reviewer and synthesis evidence must reference the immutable round snapshot it actually consumed rather than an ambient mutable worktree.

### Durable review ledger evidence

Cross-review session evidence 还必须持久化 frozen scope identity（base revision、scope mode、explicit in-scope uncommitted state）、每轮 immutable snapshot identity、fresh/blind reviewer set、每份原始 reviewer report 与 synthesis。Runner arbitration ledger 对每个 synthesized finding 记录 `FIX|REJECT|DEFER|COMPROMISE`、rationale、change/no-change、round/synthesis provenance、以及 supersedes 链；`NON_BLOCKING` 也记录但不能单独触发 fix round。

若 finding 构成 reversal，evidence 必须引用先前 decision，记录新增的 concrete failure scenario（若有），并保存新旧 rationale。一次 supersession 后再次试图翻转同一 decision，必须记录 human-decision-needed stop，而不是继续自动循环。每个 round 还要记录 finding/blocking 数量、fix/reject/defer/compromise/reversal-dismissed 统计、focused re-review confirmation，以及最终 stop reason：converged / round-cap / non-converging / human-decision-needed。Round cap 默认值与任何人工 override 也必须入证据。

## Repository quality-health evidence

A durable quality-health observation records repository/source authority, exact scanned revision, fetch/resolve outcome, observation time, scanner/contract version, and every unavailable or skipped pinned source. Canonical state may be advanced only by an exact-revision observation; working-tree measurements stay task-local. The findings ledger preserves each stable identity with kind/class, area, affected paths, metric payload, score/rationale, first-seen time, current status, and resolved/ignored provenance. A disappeared or renamed identity is not called resolved unless the comparison proves the original finding existed and the relevant condition is now absent.

Atlas and area-map evidence records the scanned revision, patterns and ownership coverage, unmapped/ambiguous/stale-pattern checks, the source facts used for split/merge/placement decisions, verifier disposition, and owner approval for committed maps or invariants. Survey evidence binds every proposal to the exact scan/map/open-ledger state and separately records inferred versus independently rechecked claims; defects found in passing are emitted as ordinary defect evidence rather than being hidden in maintainability work.

## Repository responsibility-atlas evidence

An atlas draft records the exact source revision, stable area name, declared layer and responsibility, path patterns, deterministic classification result, and the retained source facts used for each material split/merge/placement judgment. Preserve the generated candidate/skeleton identity separately from the human-authored draft, the independent source-reading/consolidation reports, the verifier disposition, and explicit unmapped/ambiguous/unused-pattern results. Candidate thresholds and maintainability metrics are supporting observations only; they are not owner approval and do not become semantic ownership.

The canonical atlas evidence identifies the exact text approved by the declared owner and the commit that made it repository instruction. Amendments preserve the prior identity/history, state why source drift requires the narrow edit, and record any intentional rename as an identity reset. Area-map evidence must identify the atlas area it refines and prove it does not contradict the atlas boundary. Unapproved drafts, stale exact-source bindings, coverage gaps, contradictory maps, or a generator attempting to publish its own ownership decisions remain non-passing.

## Repository responsibility-atlas instrument evidence

Passing atlas-instrument evidence binds the exact source revision, approved atlas text identity when one exists, declared mapped source scope and the classifier configuration that produced the result. Preserve parsed area/pattern identities, path normalization, specificity disposition and any equal-best claims required to reproduce classification. Coverage evidence records the complete unmapped, ambiguous and unused-pattern sets; a check with any non-empty set is explicitly non-passing. If no approved atlas can be loaded, evidence records that absence and any fallback measurement identity separately so the fallback cannot be mistaken for repository ownership.

Skeleton evidence preserves the candidate thresholds, selected candidate folders, per-file line inputs and the derived per-area facts used for drafting: file/line totals, largest files, own-name groups when applicable, cross-area dependency counts, recent non-merge co-change/solo counts where repository history permits it, churn, task count/median duration and tested-line totals. The evidence must distinguish raw measurements from human responsibility judgments and must label histories that cannot be compared across repositories instead of inventing cross-repository co-change.

## Repository maintenance-history measurement evidence

Passing maintenance-history evidence records the exact scanned source revision, explicit lookback window, repository identity for each history source, pinned submodule revision where applicable, Git-history acquisition outcome, and the raw per-path commit counts before aggregation. Rename-disabled counting is recorded as a measurement convention, not a claim of stable semantic file identity. If a pinned submodule history cannot be acquired, preserve that source as unavailable; do not emit zero churn or merge it silently with a different revision.

Task-effort evidence records each accepted `Task:` trailer, touched source paths, earliest Start event, latest Approve event, positive elapsed wall time, approval date, and highest recognized review-round marker. Missing queue state, absent task trailers, incomplete Start/Approve endpoints, non-positive intervals, or unrecognized review artifacts are retained as absent/unknown measurement rather than zero effort. Count/median/total summaries identify the exact retained observation set from which they were derived.

Churn, hotspot, elapsed hours and review rounds are repository-quality prioritization evidence only. They may rank investigation or survey candidates but cannot independently assign responsibility, approve an atlas/map, authorize task routing, prove product behavior, configured-build success, packaging, security, or release acceptance. The evidence remains an input to the existing repository-quality report owner and creates no second task store, review owner, queue, product runtime, or release verdict.


Command evidence records whether the operation was draft-check or skeleton-generation, the exact source revision, state/temporary output location, machine-readable and human-readable output identities, result code and failure class. Coverage violations and execution failures remain distinct: coverage failures preserve their negative sets; Git/process failures preserve the underlying error and cannot be converted to a clean check. Successful skeleton generation is evidence only; it provides no atlas approval, product/source mutation, queue receipt, shipping runtime, security authorization or release acceptance.

## Repository duplicate/clone evidence

Passing clone-detection evidence binds the exact source revision, declared source-kind scope, normalization contract and detector thresholds. Preserve each reported clone's stable normalized-content identity, normalized line count, both source paths and original start/end line coordinates, plus enough detector configuration to reproduce window, minimum-line and bucket-limit behavior. Evidence for excluded declaration-only blocks, same-file overlap clipping and common-window suppression must remain distinguishable from a claim that no duplication exists; these are detector-scope decisions, not clean-code acceptance.

Review-copy evidence additionally records the exact base/current source identity, added hunk coordinates, normalization result sufficient to reproduce the match, copied-line count and the existing source path/start that supplied the candidate. The command boundary records whether the result was advisory or strict-failing. A clone warning or finding is not self-executing: any Q2 merge needs the existing code-health class proof, mechanical review and task evidence, while behavior-divergent copies require a non-mechanical disposition.

Scan/report evidence may aggregate clone pairs and clone-line totals and carry stable findings through the canonical quality ledger, but clone metrics remain investigation selectors rather than ownership or acceptance authority. Detector/process failure, incomplete exact-source input, stale source identity, or an unverifiable normalization/match cannot be rewritten as zero clones or a passing review. Successful clone detection provides no product mutation, task-routing, security authorization, ownership change, or release acceptance.

## Repository source-text normalization evidence

Passing source-text evidence records the exact source revision, source path/kind, scanner profile or recognizer contract, input identity and the deterministic transformation used for the claim. For line-oriented evidence it preserves physical input-line cardinality and original 1-based line provenance after comment blanking/removal; for normalized comparison it records that whitespace collapse and logic-free-line filtering are lexical preparation only. Include, identifier, definition, function and class observations retain the bounded pattern that recognized them so a reader can distinguish recognized evidence from unmodeled language semantics.

Consumer evidence separately records the caller and the result semantics it owns: metric threshold/finding identity, include resolution/layering/build-cost interpretation, clone window/maximal-run identity, review warning/error/strictness, atlas candidate facts, or health command publication/error/routing outcome. A source-text helper creates no durable finding history, ownership fact, source edit, queue record, approval or release verdict on its own.

Stale/partial source, scanner failure, malformed input, lost line provenance or an unsupported/ambiguous lexical case is non-passing for any dependent claim and cannot be rewritten as a clean negative. Lexical evidence cannot prove full C++ parse correctness, type resolution, overload binding, macro expansion, configured target membership, successful compilation, runtime behavior or release acceptance; those require their existing stronger evidence owners.

## Repository source-derived build-model evidence

Passing source-derived build-model evidence records the exact source revision, build-file scope, recognizer version/grammar and deterministic traversal/normalization rules, then preserves the recognized target/source/link/PCH/executable/library/test facts consumed by the quality check. It must be possible to distinguish a recognized negative from a relationship the recognizer did not model; unsupported variables, generator expressions, quoting, conditionals, helper commands or malformed/incomplete input remain explicit limitations rather than evidence of absence.

Consumer evidence records the purpose for which each derived fact was used: include-cost/build-work estimation, cross-target layering investigation, coarse test-source attribution, or atlas drafting. It also records the caller-owned threshold/result semantics and any fallback/unknown state. The build-model library itself creates no finding history, owner approval, source edit, queue record or release verdict.

When a claim crosses from source approximation into configured build truth, evidence must reference the existing Configured-build graph provenance evidence for the exact build directory/generator/configuration/manifest subject. Source-derived CMake text alone cannot prove actual target membership, producer/output identity, PCH consumption, evaluated link operands, compiler definitions, generated inputs, build execution, artifact identity, runtime reachability or release acceptance. Stale source, parser failure or undecidable syntax is non-passing for any claim that depends on that fact and may not be rewritten as a clean result.

## Repository source include-graph evidence

Passing include-graph evidence records the exact scanned source revision, declared source roots, source-text include recognizer identity, source-derived build-model identity, resolution order, unresolved include counts and the complete resolved path/line edge set used by the consumer. Style-import evidence records the bounded `using "<path>";` grammar, originating line and resolved declared-root target. Cyclic input must preserve a finite closure containing every reachable recognized source node.

Build-work evidence preserves each translation unit's recognized target, PCH identity/closure when present, own include closure, PCH-excluded source-line weight, header reach/rebuild contribution and the exact source-line counts from which totals were derived. These are source-level proxies and are labeled separately from wall-clock timings, configured compiler inputs or artifact reachability. A missing/unrecognized PCH or unresolved include remains unavailable/unknown evidence and cannot be silently treated as an empty dependency.

Layering evidence preserves source path/line, resolved include target, recognized source target, recognized included target and the source-derived dependency closure that made the edge pass or fail the bounded check. Caller evidence separately owns the report aggregate or review warning/error/strictness result. The graph itself creates no durable finding, source edit, owner decision, task route, configured-build proof or release verdict.

When a claim concerns evaluated target/link/PCH membership, generated include inputs, actual compiler preprocessing, build execution, linked artifacts, runtime reachability or release acceptance, evidence must reference the existing stronger configured-build/runtime/release owners. Source include-graph evidence alone is non-passing for those claims.

## Repository responsibility-area-map evidence

Passing area-map evidence records the exact source revision, parent atlas area and exact approved map text, then preserves the map's five semantic parts: Modules file/file-group ownership facts; Placement rules and forbidden homes; checkable Invariants with approval provenance; Leave-alone rationale; and Direction. It records the reviewer/source facts that motivated each material placement or boundary statement, stale-file-reference results, atlas-consistency checks, and the declared owner's approval identity/date or equivalent durable provenance. Unapproved invariant text is explicitly labeled proposal-only.

Map-amendment evidence identifies the prior approved map, exact source/task change that made the amendment necessary, the narrow before/after map diff, and whether the edit changes only Modules facts or also semantic Placement/Invariants/Leave-alone/Direction. A mechanical deletion/move/consolidation may claim the narrow path only when semantic map policy is unchanged. A semantic policy change requires its ordinary owner approval and stronger task proof. Evidence also records re-survey/revalidation when repeated placement findings show the map no longer explains the area reliably.

A map is non-passing when it is stale, contradicts its atlas area, contains ambiguous/uncheckable invariants presented as approved, widens ownership to normalize a known break, lacks required owner approval, or becomes a parallel repository/product owner. Area-map evidence can prove placement governance and task preconditions; it cannot by itself prove product behavior, safe execution, security, successful build/runtime, packaging or release acceptance.


## Repository quality-health integrated regression evidence

Passing integrated repository-quality regression evidence identifies the exact test subject revision, fixture repository/build/map inputs, exercised existing owners, oracle or invariant for each case, and the positive or negative observation that decided the assertion. Coverage spans the composed source-text, source-build, include/dependency, clone, atlas, report, review and health-command contracts without treating the regression harness as an independent semantic owner.

State-lifecycle evidence demonstrates exact-ref durable publication and scheduling-window idempotence, forced rerun history append, mutable-worktree nonpublication, exact published-revision cache rebuild, and exact-commit owned worktree create/reuse/remove behavior. Review/verification evidence preserves warning/error and present/resolved/changed/unknown distinctions; move evidence distinguishes relocation/reordering that retains substantive lines from changed values, statements or logic. A clean result is passing only when the relevant negative controls would fail under the prohibited behavior.

Fixture helpers, subprocess wrappers, temporary Git repositories and sample C/C++ text are test mechanics. They do not become source authority, production code, a second scanner/findings store, ownership graph, task queue, configured-build proof, or release acceptance. Evidence remains attributable to the existing repository-quality owner and must be produced by the exact-head automated workflow selected for the task.

## Repository quality-health command orchestration evidence

Passing command-orchestration evidence records the exact source revision selected for the run, acquisition/ref mode, whether the run is durable or explicitly temporary, the canonical state location when applicable, prior ledger identity, run directory, totals, stable finding/status merge result, atomic publication outcome, append-only history receipt, and any cache identity used or rebuilt. A scheduled idempotent skip records the already-published scheduling window and exact revision that justified the skip; it cannot be inferred from a local cache or wall-clock assumption. A working-tree scan records that it was noncanonical and proves canonical ledger/history were not mutated.

For area, atlas, check, moves, verify, and detached-source inspection, evidence preserves the command operands, exact base/ref/staged subject, delegated instrument outputs, declared exit/result semantics, and the negative observation required to distinguish clean, warning, error, present, changed, resolved, unknown, or non-move-only states. Detached-worktree evidence records requested commit, resulting HEAD, dedicated temporary path identity, narrowly initialized source dependencies, reuse/removal decision, and mismatch/failure outcome. Git/fetch/process failure, stale or partial source, malformed state, coverage negatives, recognition limits, or missing proof are explicit non-passing/unknown observations and cannot be collapsed into zero findings or a successful review.

The command layer's evidence must identify the existing owners it composed rather than claiming their semantics: source authority, metric/report findings, responsibility atlas, build/text/include/clone instruments, review policy, and ordinary proposal/approval/task routing. It creates no second scanner, findings ledger, ownership graph, task queue, product runtime, or release owner. Temporary reports, caches, generated atlas skeletons, and clean repository-quality checks are supporting evidence only; they cannot substitute for exact-head build/test, runtime, packaging, security, or independent release acceptance evidence.

## Repository responsibility-area survey evidence

Passing survey evidence records the exact scanned revision and detached read snapshot, area identity, canonical area-report identity, prior survey/current map identity when present, and the open finding identities supplied as inputs. For every independent lens it preserves reviewer identity, the lens scope and direct path/line observations without merging another lens's conclusions into the evidence. Scanner measurements retained as inputs are distinguished from semantic claims produced by source reading.

The verification record is separate from the lens records. Every claim used by a map or proposal carries a verifier disposition and the reproduced observation: repeated count/search, source lines re-read, caller/reference negative, or equivalent falsifiable check. Claims not rechecked are durably marked inferred; verified status cannot be inferred from agreement between reviewers. The evidence must make it possible to trace each material map/proposal statement back to both the originating lens evidence and, when required, its independent verification.

Survey outputs preserve distinct identities for the narrative/plan, draft map, maintainability proposals and incidental defects. Draft-map evidence points to the existing area-map approval lifecycle and is non-authoritative until owner approval is recorded separately. Proposal evidence records class/area/finding/metric/proof intent but is not a queue receipt; defect evidence is ordinary product-defect evidence rather than maintainability success. A survey evidence bundle records that the survey changed no source and created no queue record. Any later publication, approval or routing has its own ordinary evidence/receipt, so survey completion can never be used to bypass source ownership, task trust, security, runtime, packaging or release gates.

Change-ratchet evidence binds exact base/head (or staged snapshot), changed-path scope, errors/warnings, and the direct proof selected for the task class. Preserve move-only novel/lost-line results, finding verification status, mutation cases for characterization tests, before/after metrics, required build/test results, and any explicit dropped-item reason. A clean scanner report is supporting evidence only; it cannot prove behavior preservation, successful shipping, or release acceptance by itself.

## Repository quality-task class evidence

Every maintainability task records one semantic class, one responsible area, stable finding identities, the exact base used for proof, before/after metric, changed/owned paths, nearest approved placement map when one exists, and whether the narrow proof remained valid through the final diff. Deletion evidence includes complete reference searches across platform/generated inputs plus affected build/link results and orphan/last-use negatives. Move evidence preserves a line-level account of substantive additions, removals and reorderings, identifies relocation-only scaffolding, and proves the destination is allowed by the current map. Duplicate-consolidation evidence records the compared copies, every pre-existing difference and its parameterization, all migrated call sites, and the resolved finding. Any item dropped from a mechanical task records the precise reason instead of being counted as resolved.

Characterization evidence lists each deliberate mutation, the expected oracle and the observed failure before reverting it. Reshape evidence binds the same characterization/runtime checks before and after the semantic refactor. Redesign evidence identifies the owner-approved invariants and approval provenance, the independent alternatives considered, why the chosen existing mechanism or new seam fits the repository, the staged constituent tasks and the final executable enforcement of each invariant. Class evidence also records scheduling/routing negatives: active path ownership, recent authority drift that invalidates placement assumptions, audit/regression suspensions, missing map/characterization/invariant preconditions and any explicit re-enable decision. A class label can never downgrade product-risk verification that is otherwise required by the ordinary task/spec.

Routed proposal evidence includes finding identities, class, area, metric before/target, proportional proof, trust/approval state, conflicts with active ownership, and the ordinary task identity that received the work. Scheduled-run evidence also records route limits and whether the run was skipped idempotently. Audit evidence records the sampled landed task, exact revision, reviewer/checks, verdict, and any class+area routing suspension/re-enable decision. Histories are append-only enough to explain why a finding was routed, ignored, resolved, dropped, or suspended without recreating a source-specific health runtime in Fabushi.

## Repository quality-health native invocation evidence

Native skill-registration evidence records the host-visible display/description metadata, default prompt or equivalent invocation selector, the canonical source-neutral workflow owner it resolves to, and the exact skill/command/scheduled routing contract that remains authoritative. Passing evidence must show the registration adds no independent scanner/state/task owner and cannot weaken fresh exact-revision publication, reporting order, approval/trust/conflict gates, or ordinary task routing. Provider-specific `$skill`, slash-command, or adapter spelling is provenance only.

## Artifact copy and transformation identity evidence

Any release, packaging, archive, staging, installation, or acceptance step that creates a second executable artifact must state whether the relationship is a byte-exact copy or an intentional transformation. Evidence for a byte-exact copy records the source and copy paths, sizes, separate raw whole-file SHA-256 identities, and proves equality; it also re-reads the source identity after the copy window and fails closed if source size or digest changed. A normalized/semantic digest may support a format-specific transformation proof, but it never replaces either artifact's raw identity and cannot by itself prove authenticity, safe execution, successful startup, signature/notarization validity, or dependency identity.

Format-specific equivalence exceptions are allowed only when that platform's shipping transform actually requires them and must have a bounded parser/profile, explicit accepted/refused cases, stable acquisition, resource limits, independent positive and forbidden controls, and a machine-readable report. Otherwise the stronger byte-exact rule applies. Historical copies, mutable task-local outputs, a copy produced before the current evidence window, or a report not bound to the exact candidate SHA are not acceptable release evidence.

### Format-transform stable acquisition and resource bounds

A format-transform oracle must acquire each source/copy artifact as one stable inspection subject, reject unsupported or mutable acquisition states, validate every offset/count/size before the corresponding I/O, and publish finite per-input parser/read/resource limits. The final verdict must re-check source/copy stability after comparison. A parser that can follow replacement paths, consume unbounded declared metadata, or silently continue after partial/short reads cannot provide passing release evidence.

### Format-transform loader-visible identity

A transform exception must protect the complete loader-visible executable identity for its declared platform/profile: ordered load/program mapping semantics, executable entry reachability, permissions, zero-fill boundaries, mapped padding and any independently loader-consumed records. Any normalized allowance must be narrow, enumerated and proven unable to mask loader-visible bytes. Raw whole-file source/copy identities remain mandatory and unmasked.

### Format-transform section/debug metadata

A transform may remove or repack only metadata classes explicitly allowed by its profile after both inputs independently validate. Unknown or retained sections/records, their semantic references, and retained non-loader payloads remain protected; malformed references, unsupported compression/encoding, invented identities, or unqualified removals fail closed. A byte-exact shipping edge does not need or gain such an exception.

### Format-transform symbol semantics

Where the binary format exposes symbol/index metadata affected by the shipping transform, the oracle must preserve retained symbol meaning and ordering, resolve index-based references to stable semantic identities, and enumerate any narrowly removable bookkeeping records. Symbol or normalized semantic digests are supporting evidence only and never replace raw artifact identities.



### Evidence-command ownership and cleanup

When a format-transform qualification harness launches a compiler, strip tool, comparator, or other external evidence command, a passing record must establish command ownership rather than relying on process names or best-effort teardown. The harness records an attempted state before launch and a final state after cleanup, including argv, working directory, environment additions, stdout/stderr destinations, start/end/deadline, timeout, exit/outcome, exception, owned PID/PGID, cleanup signals/errors, and any remaining owned processes.

Each launched command uses a dedicated owned process group/session or an equivalent source-neutral ownership primitive. Timeout, cancellation, wait error, and finalization interruption clean only that owned group with bounded graceful-to-forced escalation while a valid group identity anchor is retained; the direct child is reaped and surviving owned descendants are checked before pass. Cleanup/probe failure, unverifiable remaining ownership, or incomplete final evidence is a failed result. Unrelated process groups must survive. Signal deferral is limited to bounded launch/cleanup/evidence-finalization critical sections; the prior handler/mask is restored and cancellation is propagated after cleanup and the final record. This contract covers trusted descendants that remain within the declared ownership boundary and does not claim containment of adversarial or daemonized processes that deliberately escape it.


### Configured-build graph provenance evidence

Any evidence claim that a configured build target consumes a particular object, source, PCH, archive path, shared-library path, or command-line definition must be bound to one exact configured build graph rather than inferred from basenames, directory scans, generic compilation databases, or source-name coincidence. The evidence records the CMake generator, configuration, selected manifest, configured build-directory identity, actual host build-directory identity when different, target output, graph rule, tool versions, acquisition start/end, and a stable digest of the CMake cache plus every relevant Ninja manifest before and after acquisition. A changed manifest/cache snapshot invalidates the result.

Configured-graph acquisition is read-only and fail-closed. It may inspect only evaluated graph/tool metadata with a fixed tool allowlist and explicit operands; it must not execute build graph commands. Shell/compiler/link command parsing accepts only a bounded declared grammar, rejects unexpanded response files or unsupported macro/include channels, and reconciles link operands against explicit/implicit graph inputs and reverse consumer edges. Compile producer/source/output identity, PCH producer/consumer dialect/header agreement, and requested command-line macro state must agree across the selected cohort. Multi-config evidence must prove the selected configuration rather than relying on a phony alias or basename.

The machine-readable report distinguishes configured graph facts from stronger evidence. It may list archive and shared-library paths, but archive-member reachability remains a separate archive/reachability responsibility. It does not prove that a command executed, an object exists on disk, a symbol is present, runtime reachability, package identity, or release acceptance. Those claims require their own source-neutral evidence owners and exact-head GitHub Actions evidence.


## AI phase/worker effort evidence

When repository workflow evidence includes AI phase or worker execution, record the classified whole-scope effort (`medium`, `high`, or `xhigh`), the scope reason, and the actually applied host setting or inheritance fallback. A `high` choice requires a concrete inspected-scope justification. If the host cannot apply a requested effort/model override, evidence records that limitation rather than claiming prompt wording changed execution.

If a routine `medium` phase encounters a failure requiring diagnosis or repair, preserve the completed output and evidence, then record the escalated follow-up effort; unexpected complexity in `high` similarly records escalation to `xhigh`. Existing valid artifacts and checks remain attributable to their original exact execution rather than being discarded or silently rewritten.

For Codex review providers, reuse/structure/performance lens evidence also records whether the required `gpt-6.1-sol` pin was applied. If unavailable, the lens runs on the configured model and the evidence explicitly records that fallback. General review/synthesis, lifetime/security lenses, and unrelated phases retain the configured model unless another source-neutral policy explicitly overrides it. Same-session or adapter-limited work records inherited settings and must not add processes/workers or mutate global settings solely to tune effort/model.

### AI worker lifecycle evidence

When a repository review/assessment phase is executed by an out-of-process AI worker, preserve a durable per-worker record sufficient to distinguish parent/runner loss from worker loss and to adjudicate the result without process-name inference. At minimum record the immutable run identity, worker name, command/adapter, cwd, prompt or instruction identity, sandbox/effort/model settings, start/deadline/finish, launcher/runner identity, owned worker/process-group/job identity when exposed, transcript/event location, stderr and runner-log locations, final-result location/timestamp, thread/session identity, resume lineage, terminal status/reason, and the ownership/probe observations used by wait or stop.

A passing terminal state requires a completed worker turn/event and a fresh non-empty final result. A nonzero exit or failed turn, unreadable/incomplete transcript, missing/empty/stale final result, startup failure, or expired deadline is failure. Runner loss is recorded separately and does not by itself change a still-owned live worker to failed; the sanctioned waiter may later derive the terminal outcome from the durable transcript/final result. Once terminal, a worker state is immutable to later waiters.

State mutation is serialized by a per-worker ownership/lock boundary and atomically published. Bounded retry may address a specifically recognized sharing violation, but other write errors fail closed and an exhausted write retry is named in evidence. A no-op update leaves the evidence object unchanged. Replacement first proves that no owned worker/writer remains, then archives the complete prior evidence set under one collision-safe identity; partial archival, an unresolved ownership probe, live writer, active process, or failed rollback prevents replacement and starts nothing new.

Worker activity/cleanup evidence uses a verified owned session/process group, Windows job membership, durable writer token, or an equivalent source-neutral host identity. A recycled PID, foreign group/job, plain reader, broad process name, or query error is insufficient. Timeout/stop cleanup names only the verified owned processes, preserves unrelated processes, and records unresolved survivors as failure. Resume evidence binds the new execution to the source worker's first durable thread/session identity and proves the source was terminal before resume. Platform-specific Job Object, POSIX, file-sharing, and helper details remain adapter evidence rather than a new Fabushi product owner.

### AI worker adapter execution evidence

For an external repository AI worker adapter, evidence records the admission result and fallback decision plus the exact effective launch contract bound to the immutable run identity: worker name, cwd, prompt/instruction identity, sandbox, effort, model override or inheritance, output schema, project-document policy, deadline, executable/adapter identity, effective command arguments, and intentional child-only environment additions. Availability or authentication failure is an adapter-availability result, not permission to install, update, log in, weaken the sandbox, or alter global configuration.

Detached launch and terminal completion are separate observations. Preserve each bounded foreground-wait outcome, including a still-running return that requires another wait on the same run, and preserve the eventual terminal adjudication from durable transcript/event and final-result evidence. Do not replace this evidence with sleep/mtime polling or infer completion from the runner process alone. After terminal success, record the independent validation of the phase's required artifacts or source effects; a worker terminal status by itself does not prove those outputs are complete or correct.

## Context provenance and handoff evidence

When repository workflow evidence depends on project/task context, preserve enough provenance to reconstruct why each context source was selected without making the read set grow with unrelated history. Record the current request/task identity, exact live source/spec revision, applicable repository or nested instruction scope, selected project/reference paths, and the exact source/reference locations carried into the task handoff. A link or archived artifact is discovery input, not evidence that its contents govern the current behavior.

If older context, plans, proposals, READMEs, docs, receipts, or snapshots materially inform a decision, evidence records that they were checked against the current request and source before use. Conflicts that affect correctness remain fail-closed until resolved by current authority; optional durable-context amendments may be omitted when they cannot be safely reconciled without changing task correctness.

Task-local handoff evidence stays compact: present request, relevant implementation facts, load-bearing decisions, remaining assumptions, and exact locators. It must not claim a navigation index, historical proposal, host pointer, or stale project summary owns live task status or overrides canonical repository/spec/source authority. A durable shared-context amendment is passing only after the latest canonical target is reread, concurrent edits are reconciled, the published edit is narrow and still valid, and blocked/split-required work leaves proposed shared facts task-local.



## Adaptive test-loop evidence and recovery history

Every adaptive verification attempt preserves an append-only attempt/run history. Before execution, each check records its requirement/risk, changed surface, direct instrument, expected observable, oracle source, control or negative, durable evidence destination, and falsifier. After execution, record the exact command/action, working directory, relevant environment additions, start/end/deadline, exit/outcome, complete log or durable artifact pointers, literal observation, result, and final causal classification. Do not overwrite a failed run after recovery.

Carried-forward passes remain bound to the exact implementation, fixture, oracle, environment and dependency state that produced them. A later change records which checks were invalidated and why; only invalidated or previously failed checks rerun. Evidence from a source/spec/oracle/baseline change is historical until revalidated. Rerunning the same command until it happens to pass is not acceptable evidence.

For `TEST_FLAW` recovery, preserve: prior positive proof, exact failed assumption, failure signature, forbidden repeated technique, next directness strategy, why the next instrument executes/observes the changed surface more causally, and which checks remain unmet. At a normal campaign cap, preserve the completed passes and start at most one focused recovery campaign under the repository strategy. At the focused cap, preserve an independent recovery-exhaustion assessment; a cap, timeout, repeated signature, flake, blank/missing capture, or ambiguous reading is never transformed into pass evidence.

For `IMPL_BUG`, evidence identifies the sound failing check, root cause when known, implementation fix provenance, required focused review, checks invalidated by the fix, and the still-valid approvals carried forward. The implementation-fix handoff names the failing check, evidence, failure signature/root cause and fix hint so another phase can reconstruct the boundary without chat history.

Timeout/cancellation/wait-error evidence records owned process/session identity, cleanup actions, remaining owned processes, and whether the observation proves an implementation wedge or only an evidence-path failure. Build-lock recovery additionally records exact build root, named locked artifacts, verified holders/ownership basis, failing build log, recovery round, cleanup result, safe-to-retry decision, and final exhaustion reason if any. Cleanup must never infer ownership from a broad process name.

These source-neutral records are repository workflow evidence. Telegram portable-account, overlay, UI-driver, and Debug-harness mechanics remain upstream-specific instrumentation details unless an independent Fabushi requirement selects an equivalent instrument; they do not create a Telegram test owner in Fabushi.
