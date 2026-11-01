# Fabushi Desktop — Agent Instructions

These instructions apply repository-wide to all AI-assisted development in `bhrumom/fabushi-desktop`, unless a more specific nested `AGENTS.md` applies to a subtree.

## CRITICAL: Spec-first development — No Spec, No Code

Every AI agent **must read the applicable specification before starting product-affecting development**.

Product-affecting development includes changes to application/runtime code, tests, schemas, contracts, dependencies, build/release configuration, migrations, security controls, user-visible assets, or other behavior-affecting repository files.

**Read-only investigation is allowed before the spec is complete** when it is needed to understand the current system or write/repair the spec. Implementation is not.

### 1. Before implementation, find and read the spec

At the start of every development task:

1. Read this root `AGENTS.md`.
2. Locate the matching project under `projects/`, if one exists.
3. Read the project's applicable source of truth and normalized specs, especially:
   - `SOURCE_OF_TRUTH.md`
   - `docs/01-范围与非目标.md`
   - `docs/02-需求与成功指标.md`
   - `docs/03-架构与实现策略.md`
   - `docs/04-质量与测试策略.md`
   - `docs/19-完成定义与验收.md`
   - relevant `management/tasks/*.md`
   - relevant decisions, contracts, source records, and evidence
4. Check `docs/specs/` for a task/feature-specific spec.
5. Validate the durable spec against the latest explicit user requirement and current repository/GitHub facts.

Do not start implementation from chat memory alone.

A file named `*.spec.ts` is a test specification, not automatically the product/development spec required by this rule.

### 2. If there is no usable spec, create or repair it first

If no applicable durable spec exists, **stop before implementation and create one first**.

Default location:

`docs/specs/<short-kebab-case-task-name>.md`

Use:

`docs/specs/SPEC_TEMPLATE.md`

If the task already belongs to a governed project under `projects/`, prefer updating that project's source-of-truth/spec/task documents instead of creating a parallel conflicting spec.

If an existing spec is incomplete, stale, or contradicts the latest authoritative requirement, repair the spec before coding.

### 3. Minimum spec requirements

A development spec must define enough information for another engineer or AI agent to implement and verify the task without relying on the originating chat.

At minimum it must cover:

- context / problem;
- goal;
- non-goals / out of scope;
- functional requirements;
- current state and target state;
- architecture / ownership boundaries;
- interfaces, contracts, schemas, or data/control flow when applicable;
- constraints and non-functional requirements when relevant;
- failure modes and edge cases;
- implementation strategy;
- verification / test strategy;
- acceptance criteria / Definition of Done;
- release, migration, rollback, and observability when applicable;
- references / provenance.

For non-trivial work, requirements and acceptance criteria should have stable IDs so implementation and evidence can be traced back to the spec.

A small fix may use a short spec, but it still needs a durable statement of required behavior and completion criteria.

### 4. Mandatory development lifecycle

AI development must follow this sequence:

**Discover → Spec → Architecture/Plan → Implement → Verify → Spec Compliance Review → Integrate/Deliver**

#### Stage 0 — Discover
- Inspect the relevant current code and live repository state.
- Locate the owning project/spec/source of truth.
- Do only the investigation necessary to understand the task.

##### Repository/project context discovery and handoff

Build task context selectively from the current explicit request, exact live repository state, applicable source/spec authority, task lineage, and only the project/reference material that is concretely relevant to the requested behavior or an unresolved ambiguity. Do not recursively load every linked document, project, task history, README, proposal, or archived context merely because it is discoverable.

Repository instruction precedence remains source-neutral: the root `AGENTS.md` is repository-wide unless a more specific nested `AGENTS.md` governs the edited subtree. Host pointer files such as `CLAUDE.md` or `GROK.md` may add host-adapter mechanics but do not override canonical repository or nested instructions unless that canonical policy explicitly delegates the choice. Task/spec requirements and current source establish the behavior being implemented; project overviews, docs, READMEs, older plans, receipts, snapshots, and proposals are supporting context and must be revalidated before use when they conflict or may be stale.

A task handoff must be compact and reconstructable: record the present request, relevant implementation facts, exact source/reference locations, load-bearing decisions, and unresolved assumptions needed by the next phase. Link to durable project/reference material instead of copying accumulated history. Live status belongs to the canonical task/status owner rather than a navigation index or historical context artifact.

Durable shared-context changes are narrow, optional, and fail-closed. Propose only a still-useful shared fact; after implementation/review/evidence approve the outcome, reread the latest canonical target, reconcile concurrent edits, and publish only the still-valid narrow change. Historical full-document proposals are never promoted wholesale. If a conflict affects task correctness or safe publication, stop that publication boundary rather than guessing; an optional context amendment that cannot be reconciled may remain task-local without blocking an otherwise correct task.

#### Stage 1 — Spec
- Read the applicable spec completely.
- Reconcile it with the latest explicit user requirement.
- Create or update the spec before implementation if needed.
- Make scope, constraints, edge cases, and Definition of Done explicit.

#### Stage 2 — Architecture / Plan
- Derive the technical design from the spec.
- Identify ownership boundaries, affected interfaces, migration needs, risks, and verification.
- Break the work into traceable tasks for non-trivial work.

#### Stage 3 — Implement
- Implement against the durable spec, not chat memory.
- Stay within declared scope.
- Preserve architectural boundaries.
- Do not weaken requirements or acceptance criteria merely to make checks pass.

#### Stage 4 — Verify
- Execute or inspect the verification required by the spec and repository workflow.
- Verify the exact source revision being accepted.
- Check relevant regression, architecture, security, performance, power, migration, and compatibility requirements when the spec makes them applicable.

##### Adaptive verification and evidence recovery

Verification is an evidence state machine, not a rerun-until-green loop. Design checks from the current requirement, risk, oracle/invariants, and final changed surface before execution. Select the most direct practical instrument per claim; no expensive instrument is globally mandatory merely because another check uses it.

A verification run is one related execution set with an immutable run identity. Preserve exact command or action, working directory, relevant environment additions, exit/outcome, complete log or durable artifact pointers, declared oracle/control, and the evidence needed to falsify the claim. Passing checks carry forward across recovery unless a code, fixture, oracle, environment, or dependency change invalidates them; rerun only failed or invalidated checks.

Classify outcomes causally and fail closed:

- `APPROVED`: every selected applicable check has direct positive evidence against its oracle.
- `TEST_FLAW`: the command, environment, fixture, probe, oracle, capture, or evidence path cannot decide the claim. Keep the implementation and repair the check or move to a more direct instrument.
- `IMPL_BUG`: a sound check proves the implementation is wrong. Repair the implementation, perform the required focused review, preserve still-valid approvals, and rerun only invalidated checks.
- `BLOCKED/UNRECOVERABLE`: use only after an independent directness assessment establishes that a required subject or capability cannot be reached safely with the available environment. A run cap, elapsed time, repeated signature, timeout, flake, blank capture, or missing ambiguous evidence is not approval and does not become a blocker by itself.

A repeated failure signature must trigger a fresh recovery assessment that records the prior positive proof, failed assumption, forbidden repeated technique, next direct instrument, and why it is at least as causal. The default managed campaign budget is one normal campaign plus at most one focused recovery campaign; the focused campaign runs only unmet checks and controls. At that cap, perform an independent exhaustion assessment rather than starting a third campaign or silently weakening coverage. Unresolved nondeterminism remains non-passing, and critical gates treat it as block/fail.

Timeout, cancellation, crash, wait-error, and cleanup paths must preserve ownership boundaries. Stop or clean only the exact owned process/session/build artifact set; never use broad process-name cleanup as evidence recovery. A recoverable build-output lock preserves the failing log, scopes recovery to the exact build root and named artifacts/verified holders, retries only when ownership is proven safe, and uses a bounded recovery budget (default at most three lock-recovery rounds where that mechanism applies). Recovery bookkeeping does not erase prior attempts or consume an implementation attempt merely because infrastructure cleanup was required.

The durable implementation-fix handoff must identify the failing check, classification, failure signature, root cause when known, retained positive evidence, invalidated evidence, and concrete fix/recovery hint. Historical attempts/runs are appended rather than overwritten.

##### Repository quality-health governance

Repository maintainability measurement is source-neutral quality governance, not a product runtime. A scheduled or ad-hoc quality-health scan that writes durable findings must first bind itself to a freshly resolved exact source revision; a mutable working-tree scan may inform a developer but cannot update the canonical findings ledger. If an authority, dependency, or pinned source cannot be inspected, name that gap rather than silently treating it as scanned. Metrics and hotspot rankings select where to investigate; they are never acceptance evidence by themselves.

Persistent findings use stable semantic identities and lifecycle state. Preserve first-seen history, explicit open/resolved/ignored status, and the exact scanned revision so a metric fluctuation, path move, or rerun cannot silently manufacture a new history or turn an unresolved finding into success. Area atlases/maps describe product responsibilities and placement; maps and redesign invariants require the declared owner approval, and hot-area surveys must separate verified maintainability findings from defects discovered in passing.

##### Repository helper-retention lifecycle governance

Reusable repository helpers are governed by the existing repository quality-health owner; helper retention is not a second scanner, findings store, task queue, ownership graph or product runtime. Durable helper inventory and retention decisions must bind to the same freshly resolved exact source revision as the parent quality observation. Preserve current-path age provenance, direct callers, internal dependents, registration/runner and declarative build/workflow references, known open work, source/reference weight, and distinct approved historical consumers. Archived task evidence is a lower bound: missing patches, unclassified or undated consumers, shallow history, read failures, or unavailable/dirty canonical task state remain explicit unknown coverage and cannot be converted into zero use.

A reusable helper that is at least 90 days old may become a review candidate when there is no independently observed consumer in the preceding 90 days or when at most two distinct observed lifetime product consumers remain. These are review triggers only. Unknown consumers conservatively block an absence-of-use conclusion, known open consumers hold routing, repeated calls from one task do not inflate reuse, and self-test/internal/creation-only observations stay separately classified. Sparse use, age, source size or documentation weight never proves dead code, build/runtime cost or Q0 deletion safety. A missing mandatory helper guide/indexed on-demand reference, or a mandatory guide reaching its declared 8 KiB budget, is likewise a decision signal rather than automatic failure/deletion authority.

Review newly eligible or materially changed candidates at most once per UTC calendar week and reread canonical state before publishing the decision. An unchanged candidate does not become a new request as its age increments. Before proposing retirement, inspect the actual helper role and implementation, callers/dependents, registration/runner entry points, build/workflow references, archived consumers, open work and any pending local replacement; record concrete cost/benefit and choose retain, defer, a smaller reference/API, or an ordinary retirement proposal. The ordinary task class/proof still governs any mutation. Stale, partial or conflicting review authority leaves the prior finding unverified instead of resolving or rerouting it.

##### Repository quality-health command orchestration governance

The repository quality-health command surface is the source-neutral orchestration boundary for exact-revision acquisition, transient measurement, durable findings publication, review ratchets, area evidence, and finding verification. It extends the existing repository quality owner; it is not a product runtime, a second scanner implementation, a second findings store, or a task queue. Durable scan state must be derived from one freshly resolved exact source revision. A working-tree scan may produce temporary developer evidence but must not mutate the canonical findings ledger. Exact-ref acquisition, cache reuse, detached inspection worktrees, and any source/dependency refresh remain bound to the same immutable revision; a missing commit, failed fetch, mismatched detached HEAD, incomplete dependency, or underlying process failure is explicit non-passing evidence.

Durable publication preserves one canonical history: stable finding identities and status, first-seen/resolution history, exact scanned revision, run identity, totals, and append-only trend history. Writes that replace canonical state use an atomic publication boundary; a scheduled scan may be idempotently skipped only when the canonical state proves that the same scheduling window is already recorded. Local raw snapshots are caches, not authority, and may be recomputed from the exact published revision when absent. Temporary runs, candidate atlases, area reports, change checks, move proofs, and finding verification do not become canonical state merely because their command completed.

Command orchestration delegates facts and semantics to the existing owners: source acquisition to repository authority, measurements/findings/reporting to repository quality evidence, atlas classification to the responsibility-atlas owner, source/build/text/clone/include facts to their qualified instruments, change warnings/errors to review policy, and proposal/routing/approval to the ordinary class/process-inbox/perform-task lifecycle. The command boundary may select and compose those operations and return their declared exit outcomes; it must not convert execution errors, unknown recognition, stale source, coverage negatives, unresolved findings, or failed proof into a clean scan. Detached inspection worktrees are temporary exact-source subjects only, use narrowly identified paths/dependencies, and must be safely removable without changing the canonical checkout or ownership model.

##### Repository responsibility-atlas governance

A repository responsibility atlas is source-neutral quality metadata, not a product runtime or a second ownership graph. It divides an exact source authority into stable named areas with one clear product/engineering responsibility each; path classification must be deterministic and complete, with exact-file rules more specific than globs and globs more specific than directory prefixes. Every in-scope source file must resolve to exactly one approved area, layer ordering must express intended dependency direction, and a known boundary violation is recorded explicitly rather than widening an area's responsibility to normalize the violation.

Atlas generation and amendment are evidence-bearing review work. Build a draft only from a freshly resolved exact source snapshot and retain the source facts used to suggest boundaries, such as file/line size, dependency edges, co-change, churn, task cost and test reach. Those measurements are heuristics for investigation, not authority: independent source readers describe responsibilities, consolidation decides split/merge/placement judgments against the source, and an independent verifier checks the claims plus unmapped, ambiguous and stale-pattern negatives. Drafts do not become repository instructions until the declared owner approves the text; later structural drift produces the narrowest amendment needed, area maps must remain consistent with the atlas, and stable area names are preserved because renaming changes historical identity.

##### Repository responsibility-atlas instrument governance

Responsibility-atlas instrumentation is a source-neutral evidence mechanism inside the existing repository quality owner, not a product runtime and not a second responsibility owner. An atlas parser may derive area identifiers and backtick path patterns from approved repository instruction, but classification remains deterministic evidence: paths are normalized to the source-root-relative form, exact-file patterns outrank globs, globs outrank directory prefixes, and longer patterns outrank shorter patterns of the same kind. Equal-best claims from different areas are ambiguity, not a tie to break by iteration order. Caching a classification is only an optimization and cannot change the result. If approved atlas text is missing or parses to no areas, the instrument has no canonical atlas classifier; a coarse directory grouping may remain available for non-authoritative measurement, but it must be identified as fallback and cannot prove repository ownership.

Coverage and candidate generation have separate contracts. Coverage checks enumerate the instrument's declared source scope, report every unmapped file, equal-specificity ambiguity and configured pattern that matches no in-scope file, and fail the check whenever any of those sets is non-empty. Candidate skeleton generation is only a drafting aid: it groups source roots by measured line totals, may split a directory over the configured large-area threshold into sufficiently large children, assigns remaining descendants to the nearest candidate ancestor, and combines source facts such as largest files/name groups, cross-area dependency edges, recent non-merge co-change, churn, task cost and test reach. Thresholds and metrics select review boundaries; they never approve an area or override source reading. History that cannot be compared across repositories must be labeled rather than presented as co-change evidence.

The executable command owner stays outside the atlas library. A repository health command may run an explicit draft check, or build a skeleton from one exact source tree plus its canonical metric snapshot and dependency graph, then write machine-readable and human-readable evidence under an exact-revision state directory (or an explicitly temporary directory when no durable state is requested). Coverage failure and execution failure are distinct outcomes: coverage violations return the declared check failure, while underlying Git/process failures propagate to the command boundary and are reported as execution failure. A successful skeleton generation does not approve a map, alter product/source code, route a task or mutate ownership. Ordinary scan/report consumers may use an approved atlas for stable area identity; absence of one never converts fallback measurement into repository authority.

##### Repository duplicate/clone evidence governance

Repository duplication/clone detection is source-neutral quality evidence inside the existing repository quality owner, not a product runtime, code generator, refactoring engine, findings owner, or task queue. The detector may normalize source text for comparison by removing comments and logic-free lines and collapsing whitespace, but it must preserve source path and original line-number provenance for every reported side. Only declared translation-unit/header source kinds enter clone analysis; normalization is evidence preparation and must not rewrite source semantics or establish ownership.

Clone discovery uses bounded rolling windows only to find candidates, then extends matching candidates to maximal non-overlapping normalized runs. Same-file overlapping regions are clipped rather than double-counted, very common buckets may be excluded to avoid low-signal combinatorics, and mirrored declaration/interface blocks are excluded when they contain no substantive duplicated logic. A stable clone identity is derived from the normalized duplicated block so scan-to-scan ledger history can survive path/line movement when the duplicated logic itself is unchanged. Thresholds such as minimum window or minimum reported lines select evidence for review; they do not prove that two responsibilities should be merged.

Review-time copy detection is a separate use of the same evidence boundary. Added hunks are normalized and compared against the exact current source snapshot, excluding the identical source location, to report sufficiently long copied runs together with their existing source provenance. Non-strict review may surface this as a warning while strict review may fail closed, but the detector itself never chooses a refactor, edits code, routes a task, or approves a Q2 merge. Q2 remains an ordinary task under the existing code-health class/mechanical-lane owner, and a clone that differs in behavior is not mechanically mergeable merely because normalized text overlaps.

##### Repository source-text normalization evidence governance

Repository source-text normalization is a source-neutral quality-evidence primitive inside the existing repository quality owner, not a C++ compiler or language frontend, product runtime, findings owner, ownership graph, or task queue. A lexical scanner may classify declared translation-unit/header kinds, extract bounded identifiers and quoted include/import paths, recognize deliberately narrow source-layout patterns, and normalize lines for comparison only when every result remains bound to one freshly resolved exact source snapshot and the scanner contract that produced it.

Line provenance is part of the evidence contract. Comment blanking or removal used for lexical comparison must preserve physical line cardinality and original line numbers for surviving observations; whitespace collapse and logic-free-line filtering may simplify comparison but cannot rewrite source or invent semantic equivalence. Block-comment, inline-comment, definition, class/member and token recognition are documented heuristics. Unsupported C++ syntax, literals containing comment-like text, macros, layout outside the recognizer, malformed input, or any other construct the scanner cannot decide remain limitations or unknown evidence, never proof that a declaration, use, include, function, class or behavior is absent.

Consumer semantics remain outside the primitive. Metrics/report own size and hotspot thresholds and stable findings; the include graph owns path resolution, reach/build-cost and layering interpretation; clone detection owns windows, maximal-run and stable-identity rules; review owns warnings/errors, stranded/orphaned and strictness semantics; atlas owns candidate-area facts; and the health command owns exact-ref acquisition, durable publication, process failure and scheduled/ad-hoc routing. The text primitive owns no durable state, mutation, approval, task insertion or release verdict.

Qualification and durable evidence must therefore preserve both positive observations and recognition limits. A stale/partial source authority, scanner exception, malformed/incomplete input, line-provenance loss or undecidable lexical case is non-passing for any dependent claim and cannot be rewritten as a clean metric, zero finding, safe deletion, ownership fact, successful build, or release acceptance. Copying a source-specific Telegram Python scanner or adding a second parser/findings/task owner is forbidden when the existing source-neutral quality/evidence lifecycle can carry the responsibility.

##### Repository source-derived build-model evidence governance

A source-derived build model is source-neutral repository-quality evidence inside the existing quality owner, not a configured-build authority, product runtime, build system, findings owner, ownership graph, or task queue. It may read an exact source snapshot's declarative build files with a bounded documented recognizer and derive approximate target, source, link, precompiled-header, executable/library and test-association facts without configuring or executing the build. Parser traversal and normalization must be deterministic for the same source snapshot, and every reported fact remains bound to that snapshot and recognizer contract.

The model must preserve the distinction between recognized evidence and absence of evidence. Unsupported commands, variables, generator expressions, quoting, conditionals, helper functions or other unevaluated build-language semantics cannot be silently interpreted as proving that a source, dependency, PCH or test relationship does not exist. Source-derived target/link facts may support include-cost estimates, layering investigation, coarse test-coverage attribution and responsibility-atlas drafting, but thresholds and inferred relationships remain quality evidence rather than build or ownership truth.

Configured-build claims remain owned by the stronger configured-build graph provenance contract. Any claim about the target that actually consumes an object/source/PCH, the selected configuration, evaluated link membership, compiler definitions, generated inputs, archive/shared-library inputs or other configured graph facts must use that exact configured graph evidence instead of this source-derived approximation. The source-derived model cannot replace a real build/test result, configured-graph provenance, release evidence or runtime acceptance.

The model library owns no durable findings or task state. Scan/report/check/atlas consumers decide how to aggregate or qualify its evidence; ordinary repository task classes, placement rules, mechanical/full-pipeline routing and scheduled task processing remain with their existing owners. A parser failure, stale/partial source authority or relationship that the recognizer cannot decide remains explicit incomplete/unknown evidence and cannot be rewritten as a clean quality result or used to create a second build-model/findings/ownership/task owner.

##### Repository responsibility-area-map governance

An approved area map refines exactly one approved repository responsibility area; it is a local placement and review contract, not a second ownership graph. Its **Modules** section names each file or tight file group and the responsibility it owns. **Placement** answers where recurring kinds of new work belong and where they must not go. **Invariants** contain only objectively checkable rules and preserve explicit approval provenance; an undated or otherwise unapproved invariant remains a proposal, not authority. **Leave alone** records deliberate shapes that cleanup must not churn merely because they look unusual. **Direction** states the intended target shape without pretending that an unapproved future design is current behavior.

Map lifecycle is owner-controlled and exact-authority-bound. A source survey may draft or amend a map from the exact reviewed revision, but cannot publish semantic ownership by itself. The declared owner approves the text or adjusts it before it becomes repository instruction. Structural work that genuinely changes placement updates the same approved model narrowly in the task that changes the source. Narrow mechanical delete/move/consolidation work may update only the affected Modules facts unless the owner separately approves a Placement, Invariant, Leave-alone, or Direction change. A redesign may rely only on already-approved checkable invariants, and a first semantic reshape in an area must not treat a missing map as implicit approval. Repeated placement findings trigger a fresh survey/revalidation instead of accumulating ad-hoc exceptions.

The nearest approved map and its parent atlas guide placement together with current ordinary repository ownership. A map must never contradict the atlas boundary, normalize a known ownership break by widening the area, authorize a source-specific runtime, or bypass stronger product/spec/security/release rules. Stale file references, contradictory placement, unapproved invariant/direction edits, and owner/authority drift are fail-closed evidence conditions, not reasons to guess from surrounding code.


##### Repository responsibility-area survey governance

A deep responsibility-area survey is bounded source-review work inside the existing repository quality owner, not a scanner pass becoming authority and not a new survey runtime. It starts from one freshly resolved exact scanned revision and the existing area facts: the current area report, prior survey/current approved map when present, and open findings already recorded for that area. Reviewers read the detached exact-authority source snapshot rather than a mutable working tree and do not re-measure facts the canonical area report already supplies. Hotspot, task-cost, churn and feature pressure select where deeper reading pays off; they do not prove a responsibility or defect by themselves.

The deep read is deliberately multi-lens and independent. Separate reviewers inspect model/responsibility ownership and encoded state machines, duplication against local/shared mechanisms, coupling/build direction and global state, testability and the cheapest behavior-pinning seam, plus defects noticed in passing. Each lens reads source directly and returns path/line evidence without seeing another lens's conclusions. Before any claim drives a map or task proposal, a separate verifier re-runs the relevant count/search or re-reads the cited source. A claim not independently rechecked is explicitly marked inferred and cannot be presented as verified evidence.

Survey outputs remain separated by responsibility: the survey narrative records the responsibility map, structural pressure, testability and dependency-ordered plan; any map output is only a draft/amendment subject to the existing owner-approval contract; maintainability proposals remain ordinary task requests subject to class/trust/routing gates; incidental defects remain ordinary defect requests. The survey itself changes no product/source code, approves no map, and creates no queue record. Any later publication, map approval or task routing is performed by the existing repository evidence/task lifecycle with its normal authority, conflict and proof checks; survey evidence never becomes a parallel ownership graph, map store, queue or shipping acceptance path.

Every source change is subject to a proportional quality ratchet against its exact base. The ratchet must detect newly introduced ownership/layer crossings, oversized growth, materially heavier dependency/build reach, duplicated logic, stranded definitions, and last-use/orphan leftovers where applicable. A behavior-preserving quality task proves the class of change with the narrowest causal evidence: deletion proves all relevant users/targets are gone, moves prove move-only semantics, duplicate merges prove the finding resolved, characterization work proves tests can detect deliberate mutations, and semantic redesign requires pre-existing behavior tests plus approved invariants. A scan never edits product code as part of measurement.

##### Repository quality-task classification governance

Maintainability work is classified by semantic change risk so proof stays proportional instead of treating every cleanup like feature development or letting a semantic refactor pass on a static scan. The narrow mechanical family is deletion of proven-dead material, relocation with no logic edit, and consolidation of copies proven equivalent apart from explicit parameters; these remain behavior-preserving only while user-visible behavior, stored formats, network traffic, test-observed logs, ordering and timing remain unchanged. Characterization work pins existing pure behavior with mutation-sensitive tests. Semantic reshaping may change internal structure but requires already-passing characterization coverage for the affected behavior. Redesign changes seams, layering, ownership or interfaces and therefore requires owner-approved, checkable invariants before implementation, with independent alternative designs considered when architecture rather than mechanics is being chosen.

Each quality task remains one concern and one revertible change or short series. It records the responsible area, stable finding identities, before/after metric, exact-base proof and placement authority. Deletion proof searches all relevant users, platforms and generated inputs; relocation proof must distinguish scaffolding from novel/lost/reordered logic; duplicate consolidation proves the named clone family is resolved; characterization evidence records deliberate mutations that the tests catch; reshapes prove the same behavior before and after; redesign evidence proves every approved invariant. A discovered defect leaves the cleanup task and enters the ordinary defect lifecycle. Quality work may not silently add build work, binary weight, a new abstraction that fails to remove duplication or enforce an approved invariant, or move code into a location forbidden by the nearest approved responsibility/area map.

Task placement and scheduling use the nearest approved atlas/map and ordinary repository ownership. A move does not start while another in-progress task owns the moved paths or while the authoritative branch is actively changing them without reconciliation; new homes are reflected in the same approved placement model. If a nominally mechanical change needs semantic edits, expanded files, new behavior or a stronger proof, it is reclassified rather than forced through the narrow lane. Characterization, reshape and redesign use the ordinary full verification pipeline. Audit failure, later regression attribution, stale authority, ownership conflict, missing characterization coverage, or an unmet map/invariant precondition suspends routing for the affected class/area until explicitly re-enabled; suspension never erases the underlying finding.

Quality findings enter the ordinary task/review/evidence lifecycle rather than a parallel queue owner. Automated routing is permitted only for classes whose trust policy is currently enabled; an audit failure, later regression attribution, stale authority, conflicting in-progress ownership, or unmet map/invariant precondition suspends routing without erasing the finding. Bugs discovered during a health survey are ordinary product defects, not code-health cleanup.

##### Repository quality-health native invocation

Host-native skill registration metadata, command aliases, and default prompts are invocation surfaces only. They must resolve to an existing source-neutral repository workflow owner and preserve that owner's exact-authority, evidence, trust, and routing rules; provider syntax such as `$skill` or slash commands is an adapter detail, not a second scanner, task queue, product runtime, or semantic owner. A default prompt may select an already-supported mode, but it must not weaken the canonical skill/spec, let stale or mutable observations advance durable state, bypass ordinary routing approval/trust gates, or reorder reporting so proposals hide the regressions and decisions that precede them. Scheduled and ad-hoc invocations enter the same ordinary task/review/evidence lifecycle.

#### Stage 5 — Spec Compliance Review
Before declaring completion, compare the final implementation against **every applicable requirement and acceptance criterion**.

Record each criterion as:
- passed, with evidence;
- blocked, with reason;
- not applicable, with reason.

If the implementation intentionally changed the design or behavior, update the spec/decision record before completion. Never leave intentional behavior undocumented.

##### Independent review contract
When an independent review is required, the reviewer is read-only for the reviewed source and must inspect the complete declared change scope plus the adjacent callers, consumers, build declarations, and precedents needed to judge integration. Findings must be falsifiable: identify the affected path/symbol, the concrete failure and triggering scenario, severity, and a specific fix. Preference-only wording/naming or speculative cleanup is non-blocking.

A specialized review may be marked not applicable only with scope-specific evidence tied to the changed paths/hunks; uncertainty is applicable, not proof of non-applicability. After accepted fixes, perform a focused re-review of each claimed fix and the edited functions' adjacent callers/consumers. Unsupported findings are dropped rather than promoted into blockers.

###### Frozen scope and bounded lifecycle
Freeze the declared review scope once before independent review begins, including the base revision and any pre-existing uncommitted work that is explicitly in scope. Each round must review an immutable snapshot of that scope. Reviewers are fresh and blind to one another's reports and to prior runner decisions; only the source-neutral repository review runner may carry arbitration state across rounds.

Before applying a review-driven fix, the runner must decide every synthesized finding in the durable review ledger as `FIX`, `REJECT`, `DEFER`, or `COMPROMISE`, with a concrete rationale. `NON_BLOCKING` findings are recorded but cannot start a fix round by themselves. A later finding that would reverse an earlier decision is rejected by reference to that decision unless it adds a concrete failure scenario not previously weighed; a decision may be superseded once, and any attempt to flip it again requires a human decision with both rationales preserved.

After any accepted fixes, run a fresh focused re-review of the fix delta plus the affected functions and adjacent callers/consumers, while retaining the complete frozen scope as context. The review loop stops only on convergence, a configured round cap (default ten), demonstrated non-convergence/scope explosion, or a required human product/arbitration decision. A real defect in changed code is never left open merely to save a round.

###### Immutable review-scope snapshots

The frozen review scope must be reconstructible without mutating the developer's real staging state. Bind each review session to one verified repository root, an exact base commit, the starting HEAD, scope mode, and any explicitly included uncommitted state. When the scope is the last N commits, reject ranges that extend beyond repository history instead of silently widening or changing the base.

Every review round snapshots the complete current scope, including untracked paths, through an isolated staging/index or an equivalent read-isolated mechanism; never use the developer's real index as scratch state. Preserve a stable tree identity plus a complete rename-aware, binary-safe diff and changed-path/status manifest against the frozen base. From round two onward also preserve the fix delta against the latest prior round snapshot while keeping the complete frozen-scope diff available to every focused reviewer.

Snapshot acquisition fails closed when repository identity, base, session state, staging isolation, tree creation, or diff generation is unavailable. An empty scope is an explicit terminal no-review result, not a successful review. Session and round snapshot artifacts are repository review evidence; Git implementation details and private-index paths are tooling mechanisms, not a product runtime owner.

###### Worker/phase effort and model inheritance

Before starting each AI phase or worker, classify the complete assigned scope from the current evidence and select an effort level. This governs development/review execution only; it does not create another product owner, delegation layer, or phase.

- Use `medium` only for routine scheduler bookkeeping, helper-driven setup/publication, already-specified build/test execution, artifact collection, or mechanical normalization where no design, diagnosis, source repair, or verdict is required.
- Default to `xhigh` for context/planning, visual design, independent assessment, implementation, review and synthesis, evidence design/authoring/assessment, failure diagnosis, convergence/rescoping, inbox routing/consolidation, and any stateful performer that owns an implementation/evidence campaign.
- Use `high` instead of `xhigh` only when the parent has inspected the actual scope, is sure it contains nothing complex, and records a concrete reason. Short prompts, small diffs, expected not-applicable results, or time pressure are not sufficient. Unknown scope, ambiguity, interacting subsystems, lifetime/concurrency/security concerns, or an earlier failed approach favor `xhigh`. Never select below `medium`.
- Classify the whole assignment rather than its easiest substep. A worker that plans, repairs, judges, or synthesizes does not become `medium` merely because it also runs commands.

Include the selected effort and a short scope reason in the worker/phase instruction. Record the actually applied setting, inherited setting, or adapter fallback in the durable phase/review evidence.

If a `medium` phase encounters a failure that needs diagnosis or repair, preserve its completed output/evidence and return control before expanding scope; continue the follow-up at `xhigh`, or at justified `high`. Unexpected complexity in `high` similarly escalates to `xhigh` without discarding already-valid artifacts or checks.

Keep the parent/configured model unless an applicable source-neutral host or review-lens policy explicitly overrides it. On Codex review providers, the reuse, structure, and performance lenses use `gpt-6.1-sol` in every review round at the selected effort; the general reviewer/synthesis, lifetime and security lenses, and all other phases retain the configured model. If that pinned model is unavailable, run those three lenses on the configured model and record the fallback in the durable phase/review evidence. Use an exposed effort/model field only when the current host actually supports that field. If the host cannot apply the requested override, retain the real session setting and record the limitation/fallback; prompt wording alone is not an effort override. Same-session execution follows the same rule, and agents must not start another CLI process, change global settings, add workers, or replace a stateful performer solely to tune effort/model. Re-evaluate effort when a reusable child receives a new scope; a disposable child that cannot change effort/model may finish its owned writes before a fresh correctly configured child is started.

###### Out-of-process AI worker lifecycle safety

When a host adapter executes a review, assessment, or other repository AI worker out of process, treat that worker as an owned evidence subject rather than as an ordinary best-effort subprocess. Completion requires explicit terminal adjudication from the worker's durable event/transcript state plus a fresh non-empty final result. Losing the launcher or runner alone is not failure while the owned worker can still be proved active; continue the sanctioned wait path instead of starting a duplicate, replacing its files, or inferring failure from a missing runner.

Per-worker state is monotonic and fail-closed. Terminal outcomes are never rewritten by a later waiter. State updates must be serialized and atomically published; a no-op update does not churn evidence. Replacement is allowed only after the prior worker is proved inactive and archives the complete prior evidence set as one unit, or moves nothing. Resume requires a completed source worker and durable thread/session identity; it is refused while the source is active or its identity cannot be established.

Process cleanup follows exact ownership, not a PID value or process name. POSIX session/process-group identity, Windows job membership, or an equivalent host primitive plus any durable writer token must prove the owned process set before stop/timeout cleanup. PID reuse, a foreign process group, a read-only file handle, or a failed ownership probe never authorizes termination; unresolved ownership remains non-terminal until the deadline/recovery contract resolves it. Bounded cleanup must preserve unrelated processes. Host-specific helper code, Windows Job Objects, and POSIX process mechanics are adapter details and do not create a product runtime owner.

###### Out-of-process AI worker adapter execution

A source-neutral repository adapter that starts an external AI worker must fail closed at admission and launch. Before the first detached worker of an invocation, verify that the selected adapter is actually available and authenticated through its supported status surface; an unavailable adapter follows the declared native or same-session fallback and records the reason. Do not install, update, authenticate, or mutate a user's global configuration on the worker's behalf.

Bind each launch to an immutable run identity and to the effective execution inputs before detaching it: worker name, working directory, prompt/instruction identity, sandbox, effort, model override or inheritance, output schema, project-document policy, deadline, and any adapter-specific command arguments. Environment compatibility changes must be child-scoped and recorded; they must not escape into global repository or user configuration. A detached launcher returning successfully proves only that the run was accepted for execution, not that the worker completed.

Completion uses the adapter's sanctioned foreground wait against that same durable run. A bounded wait that reports the worker still running means continue waiting on the same run; it never authorizes sleeps plus artifact polling, a duplicate start, destructive replacement, or a success verdict. After terminal adjudication, validate the phase's own required artifacts or code effects separately from the worker's terminal status.

#### Stage 6 — Integrate / Deliver
- Follow the repository's branch, PR, merge, release, migration, and deployment rules applicable to the task.
- Do not claim completion merely because code was written or pushed.
- Completion requires the spec's Definition of Done and the applicable integration/delivery gates.

### 5. Fail-closed rules

The following are prohibited:

- starting product-affecting implementation without first reading an applicable spec;
- skipping spec creation because the requested change appears obvious;
- using the chat prompt as the only persistent specification for substantive work;
- inventing requirements from memory when an authoritative spec exists;
- silently expanding or reducing scope during implementation;
- deleting, bypassing, or weakening acceptance criteria to make work appear complete;
- claiming completion without requirement-to-evidence review;
- leaving the durable spec stale after an intentional design or behavior change.

When the spec is missing, unclear, contradictory, or stale, **the next development action is to repair the spec, not to guess and code**.

## Canonical policy

The detailed repository policy for this gate is:

`docs/specs/spec-first-ai-development.md`

All AI development in this repository must comply with that policy.

## Build and test execution environment

All builds and tests must run only in **GitHub Actions** or on **`htch-runtime`**, as required by the user. Do not run builds, linters, generators, schema checks, tests, benchmarks or acceptance on a local workstation, the user's Mac/Windows desktop, or the assistant's local working container. Read-only investigation, editing and Git/API operations are allowed.

Missing runner, OS, device, account or signing configuration is `not-configured` / `blocked`, never a passing test. Record the exact source revision, actual commands, test counts, exit codes and artifacts from the allowed execution environment.

## Fabushi native communication capability absorption — FBCP-001

For messaging, rooms, channels, contacts, media, calls, identity, Telegram-source research, or Human+Agent communication work, read:

- `projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md`;
- `docs/specs/fabushi-bot-communication-platform.md`;
- `projects/fabushi-communication-platform/architecture-map.md`;
- the current FBCP task/ADR;
- `projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md` and TDRP-001 when Telegram source behavior is involved;
- the current PR #20 / canonical Grok Bot spec.

### Single target architecture

**PR #20 / canonical Fabushi is the only target product architecture.**

Telegram Desktop is a research source. Do not create a Telegram Provider, Telegram network dependency, Telegram-only product shell, or a parallel Communication Core.

For every researched capability:

1. inspect the current exact-head Fabushi architecture;
2. list plausible current owners;
3. select `existing_owner`;
4. define the absorption changes;
5. only if no owner is suitable, create a minimal `new_owner_proposal` with rejected-owner analysis and an ADR.

Prefer extending current owners such as sidebar, conversation workspace, transcript/cards, composer, Shared Room/member model, reactions, attachments/artifacts, permissions, search, settings, Computer, Plugins/MCP, Automations, Host/Coordinator/platform boundaries.

Broad parallel owners such as `TelegramCore`, `TelegramRuntime`, `TelegramProvider`, `CommunicationCore`, or `MessengerRuntime` are prohibited unless a future explicit interoperability spec authorizes them.

### Native network

Fabushi owns its communication identity, messaging, sync, presence, media, call signaling and push infrastructure. Telegram/MTProto code may be studied for mature communication behavior but is not the Fabushi runtime protocol.

### Source-informed implementation

TDRP-001 remains source-informed and is not clean-room. Do not claim Rust automatically removes GPL obligations. Required Telegram C++ product responsibilities are reimplemented in the appropriate Fabushi owner; original Telegram/desktop-app C++ must not remain the production owner.

### Execution

**FBCP-001 / TDRP-001 project exception:** all executable verification for this migration project runs **only in GitHub Actions**. The repository-wide `htch-runtime` allowance above does not apply to FBCP/TDRP. This includes build, lint, generator, schema, unit/property/state-machine, contract, integration, E2E, temporal, visual, accessibility, performance, soak, fault-injection, package and acceptance runs.

For FBCP/TDRP, follow TDRP-EXEC-01: finish the full source-responsibility and implementation scope first, then freeze one exact target commit and run applicable Actions and real acceptance together. During development do not trigger, wait for or repeat migration CI, push implementation branches/update migration PR heads, or run local executable validation. Preserve `not-run`/`pending`/false until evidence exists; this timing rule does not waive any final gate.

Read-only investigation, spec/test-plan editing, evidence review and Git/API operations may occur outside Actions. Missing runner/account/service/signing configuration is blocked, never pass.

## OpenClaw full-source Rust equivalence — OCRM-001

For OpenClaw-derived implementation, interoperability, source accounting or acceptance work, read:

- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`;
- `docs/specs/openclaw-rust-equivalence-migration.md`;
- `projects/openclaw-rust/module-map.md`;
- the current OCRM task, exact-source lock/inventory/ledger/dossiers, applicable domain specs and actual canonical Fabushi production owners.

OCRM requires the complete pinned source responsibility scope, file-by-file understanding and Rust implementation integrated existing-owner-first into current Fabushi. Preserve all current Bot capabilities and independent Coordinator/Host/Runner/Electron/renderer boundaries. React/Electron remain presentation and thin platform bridges; migrated business state, policy, protocols, scheduling, persistence and execution control belong to Rust. No parallel OpenClaw runtime, Agent loop, session store, scheduler or marketplace may become a second production owner.

Optional external-channel, Gateway/SDK and node interoperability is included by OCRM through scoped adapters under existing owners; it does not replace Fabushi native identity, messaging, network or canonical state. FBCP/TDRP requirements and evidence remain independently applicable.

Directory enumeration, source reading, mapped responsibilities, implemented code, production wiring, verified behavior and packaged acceptance are separate states. Never promote one to another without its required evidence. Missing accounts, services, devices or signing remain blocked; continue independent work without weakening the final scope.

**OCRM project execution rule:** every executable validation, including generators, schema checks, builds, tests, packaging and acceptance, runs only in GitHub Actions. The repository-wide `htch-runtime` allowance does not apply to OCRM. Read-only investigation, editing, Git/API operations and evidence review are allowed outside Actions.

## Repository task/process authority governance

Repository-development task/process state is source-neutral repository governance, not shipping product state. Canonical authority is the current published repository/project task state at an exact Git revision together with the applicable source/spec authority. Chat history, host command adapters, receipts, discovery notes, soft batch marks, local phase files, and product `Tasks` / `Automations` state are supporting context only and must not become a second task truth.

A development invocation must freeze the explicitly selected scope before claiming new shared work. Later queue refreshes may update the state of those selected items but may not silently expand the scope with unrelated work. The only scope growth permitted after freeze is an explicit replacement of an in-scope split source or an explicitly routed follow-up produced by an in-scope result, with durable lineage. Starting work remains one-task-at-a-time: an existing active task owned by the checkout/session is resumed before a new shared task, and ownership is established only by the canonical state transition that assigns it. Never infer ownership from an inbox/discovery receipt or a batch/listing artifact, and never steal an unfinished task from another owner.

A split is one canonical routing transaction, not a convenience status. It begins only from a canonically published `split-required` source owned by the current execution checkout, with the complete source task/result/split proposal and retained-work seal available for reread. The performer may request the rescope but does not create or retire queue work. The split transaction refines the proposal into at least two unique `type: implement` replacements, each independently shippable and independently testable with a direct acceptance oracle. Preserve the source project by default, preserve every dependency whose shipped code remains required, order successor dependencies by actual code/behavior prerequisites, and add an integration successor only when integration has behavior not already proved by its components.

A split must preserve every original requirement, dependency and retained implementation boundary. When the retained-work seal says implementation exists, exactly one first replacement is the implementation carrier: the whole retained diff must fit a coherent boundary that can inspect, correct, review and later validate it; the carrier inherits the checkout ownership handoff, owned-path scope and durable `carried_from` lineage, while any later successor that consumes that code depends on the carrier. The split publication itself does not reinterpret retained phase conclusions as approval. The later canonical Start of that carrier must revalidate the retained worktree seal before transferring source refs and retiring the old refs. When no implementation is retained, there is no carrier and all successors remain unclaimed. If no coherent carrier can own the retained implementation, preserve the source task, refs and working implementation and stop rather than discarding, resetting, stashing, flattening or multiplying it across unreviewed successors.

Retirement and publication are atomic repository-process state changes. Rewrite every live dependency on the source to the exact successor or successors that provide the needed behavior and replace the source link in project navigation in the same transaction; a retired split id may remain only as durable history, never as a live dependency or project link. Preserve the source task content, replace only its runnable state with a sealed multi-target split record containing ordered successors, optional carrier, receipt, original metadata and retained-content digest, and require every split/superseded chain to resolve cycle-safely to live canonical work. Publication uses an explicit changed-path set, validates the receipt, successor state, project links, complete dependency/split/superseded graphs, retained source digest and worktree seal, then publishes through compare-and-swap. An ordinary publication race may be refreshed and retried through the same validator; a changed retained-work seal, incoherent carrier, semantic conflict or remote outage preserves the recoverable source/commit state and hard-stops instead of force-pushing or partially applying the split.

Consolidation is an optimization over compatible unclaimed unfinished work only. Candidate eligibility is fail-closed: use canonical unfinished state; require pristine unclaimed todo work with no active claim/lease/phase ownership and no task-local work, evidence, or unexpected artifact showing execution has begun; preserve project lineage and frozen-batch membership as hard partition keys; and exclude any task class whose own policy sizes or isolates batching. It must never rewrite approved history, active/claimed work, cross-project or project/standalone lineage, frozen-scope membership semantics, or request meaning. An unfinished dependent that is active, claimed, blocked, or otherwise non-mergeable vetoes retirement; an eligible unclaimed todo dependent may be rewritten only in the same atomic transaction. Before selecting a cluster, simulate the replacement in the unfinished dependency graph and reject any direct/transitive self-dependency or cycle.

A merge must reuse a material common execution setup rather than merely concatenate nearby work, and it must preserve every load-bearing acceptance criterion, prerequisite, dependency, input, safety/fixture fact, and project lineage. A replacement inherits the stable union of external dependencies with internal source ids removed. Exact duplicates may collapse only with explicit criterion accounting. A no-merge decision is still durable: it replaces the pending marker with one completion/no-repeat boundary and changes no unrelated task/project/receipt state. A merge retires source ids without deleting their history: retain the original task specification and inputs, remove live state, add a durable old-to-new alias plus receipt and retained-content fingerprint, keep retired directories free of execution artifacts, and require every alias chain to terminate at live canonical work.

Immediately before any tracked consolidation write, reread the candidate sources, rewritten dependents, project/dependency state, and canonical task ownership. Any pre-commit status, owner, project, dependency, or retained-content drift is a race and writes nothing. Durable publication is one explicit-path transaction: validate aliases and fingerprints, complete dependency-graph integrity, rewritten project/dependency links, the completion marker, and exact old-to-new mappings before commit; publish through compare-and-swap; and re-run the same semantic validation after any fetch/rebase before push. A committed semantic conflict, remote outage, late source edit, new cycle, or unsafe publication state preserves the recoverable commit and is reported blocked; it is never force-pushed away. Generic resume/publication must recognize a consolidation commit and apply the same validator.

An interrupted agent/session is not by itself a product or task blocker. Preserve resumable task-local state and continue from the first incomplete validated boundary. Publish a blocked boundary only for a genuine exhausted/unresolved condition, and keep implementation state separate from executable verification/acceptance. Repository task/process governance may coordinate shipping work but does not authorize product-runtime task execution, release acceptance, credentials, deployment, or a second Telegram-specific queue/scheduler.



##### Repository inbox intake/routing governance

The repository inbox is a non-authoritative intake surface for development planning, not canonical task state and not shipping Product Tasks / Automations state. One intake transaction is bound to the exact source checkout, stable checkout tag, canonical AI repository, isolated inbox publication worktree, and a content digest of the complete ignored inbox payload. Resume only the single active transaction whose checkout tag and canonical repository match. A new transaction requires clean canonical AI state and a clean inbox worktree with no unpublished inbox commit; empty, ambiguous, invalid-tag, or unsafe publication state stops without clearing the live inbox. The active task-execution slot is outside this transaction and must not be inspected, rebased, staged, committed, or used as planning authority.

Routing reads only the immutable transaction snapshot plus the applicable source/repository instructions and targeted live project/task lineage. Durable superseded aliases and split records are history, not reusable task paths: resolve them cycle-safely to live successors for deduplication and dependency decisions, and treat the retired dated path as permanently occupied. Preserve the originating project by default when new work depends on its shipped code, behavior, branch assumptions, or ordered context. Cross-project or standalone routing requires concrete evidence that the work remains coherent, implementable, and independently testable without the originating project. Branch placement, cherry-pick, backport, rebase, merge, and equivalent history coordination are receipt-only dispositions, never autonomous product tasks.

Every routed task is a dated, collision-safe, independently testable `type: implement` record with a self-contained request, falsifiable acceptance result, copied durable inputs, canonical dependencies, and pristine unclaimed `todo` state. Inbox routing never reserves work and never writes a completion `model`; dependencies preserve code/source lineage as well as readiness. Split at product/acceptance boundaries when parts have independently useful outcomes and direct oracles, but keep inseparable API/caller changes together. The later performer may still publish `split-required` after deeper assessment; replacement creation and retirement remain scheduler-owned split routing. Consolidation aliases and split records must be honored during deduplication and new dependency creation, but inbox processing does not rewrite those completed history transactions.

Supplied-video planning input is repository evidence, not shipping attachment/resource ingress. Keep the original video in the ignored transaction and later local backup; bind the receipt to its SHA-256 plus probe facts. Timing authority is each decoded frame's own presentation timestamp normalized to the first decoded frame. Never derive a cited time as frame-index / fps, because variable-frame-rate recordings make that drift. Full extraction is disposable measurement scratch outside the transaction and task input. Commit only the durable evidence needed for each owned phase: complete tight-crop sheets for the phase, key poses and a small amount of full-frame context, with deterministic decoded-frame index and millisecond time in the filenames and task references. The task must state that the original video is not in the repository; a later performer consumes only the tracked task specification and referenced input frames, and asks the owner when those durable inputs cannot answer a later question rather than reaching for the local video.

Video-helper failure is fail-closed. Missing `ffprobe`/`ffmpeg`, a required missing Pillow dependency for sheets, malformed frame/crop selection, out-of-range indices, no decoded video frames, a non-zero tool exit, or a decoded-frame count different from the requested set is not usable evidence. Temporary raw decode files belong to the helper's temporary directory and must be cleaned on exit. If a later filesystem/image-write failure leaves target-side partial output, those files are non-authoritative and must not be published merely because they exist; rerun from the immutable local source or report the failure.

Tracked publication is explicit-path only. The receipt must bind the transaction digest and account for every request/input; no broad staging, raw inbox path, supplied video, complete frame extraction, portable account, credential, browser profile, complete run/build log, or persisted source commit hash may enter tracked planning state. Publication commits only the declared task/project/receipt paths, rebases against refreshed canonical state, retries ordinary non-fast-forward publication races, and preserves the transaction/commit on semantic conflict or remote failure instead of force-pushing or clearing input. Finalization is allowed only after the exact receipt and digest are present on canonical master; it always preserves a local backup and clears the live inbox only when the live payload digest is unchanged. Abort is only for a pre-commit planning failure; once a generated inbox commit exists, recovery resumes publication rather than discarding it.

These contracts extend the existing source-neutral repository task/process owner. They do not create a Telegram-specific scheduler, a second task/receipt truth, a shipping Agent/Coordinator/Host/Runner owner, or platform-core runtime responsibility.

##### Repository workspace-helper identity, lineage, publication, and recovery governance

Repository-process helper execution is bound to one concrete source checkout and one canonical AI-state repository. A stable lowercase machine tag plus checkout folder forms the checkout identity; the canonical AI repository must be its own Git root on the canonical branch, and slot, inbox, routing, and code-health workspaces are distinct linked worktrees whose common Git directory and expected branch must match that repository. A dirty canonical tree, a foreign or moved worktree, an unexpected branch, unpublished work in a workspace being resynchronized, or unsafe nested-worktree/submodule overlap is a fail-closed process condition. The task-execution slot, inbox publication worktree, routing worktree, and code-health worktree are separate mutation domains and cannot substitute for one another.

Canonical repository task state remains the published task record, not local Git refs or host state. State parsing preserves status, type, dependency, claim, phase, and model fields. Start atomically claims only an eligible todo task after dependency and source-lineage gates; Retry reopens only the current checkout's blocked task without erasing durable artifacts; competing active work is rejected. Approved source-producing prerequisites must have retained task commits reachable from the selected source history. Compatible branches may be reported, but missing ancestry is a routing stop. Local task base, green, and run refs, carried-work seals, overlays, fences, and recovery mirrors are recovery instrumentation only and never canonical task or acceptance authority.

Inbox transaction identity is content-addressed as well as checkout-addressed. The payload digest covers the sorted relative path and exact bytes of every non-backup payload file, rejects symlinks, and is sealed into immutable transaction metadata before routing. Publication is explicit-path only and refreshes and rebases an isolated worktree against canonical state before the semantic validator is rerun. Ordinary non-fast-forward publication races may retry, while rebase conflicts, semantic drift, remote failure, unsafe source state, or lost task ownership preserve recoverable work instead of force-publishing canonical history. An optional force-updated private ai-checkout source mirror is recovery backup only and never permission to force-update canonical or shared publication refs.

Finalize is a separate post-publication boundary: canonical master must contain the exact tracked receipt and transaction digest, the transaction snapshot is always retained as local backup, and the live inbox is cleared only when its current digest still equals the sealed snapshot. If the live inbox changed, the snapshot is archived as processed while the human-edited live input remains intact. Abort only closes a pre-publication planning failure by retaining a failed snapshot and never clearing live input; once a generated publication commit exists, recovery resumes publication. Approval/source-ref cleanup is likewise fail-closed: recovery refs are deleted only after canonical approved state, ownership/type/result, clean source, and retained-source invariants all agree; otherwise they remain available for recovery.


##### Repository dependency advisory governance

Repository dependency monitoring is source-neutral repository quality and supply-chain advisory governance, not the shipping product updater and not a dependency mutation engine. A scheduled or ad-hoc dependency audit must bind its repository snapshot to one freshly resolved exact source revision before deriving dependency identities. A failed fetch, missing manifest, unread gitlink/submodule, unavailable package revision, stale security source, or incomplete recursive dependency range is explicit partial/unavailable coverage; an older mutable ref, yesterday's inventory, or absence from one advisory database cannot be presented as a fresh clean result.

The audit discovers the complete union of dependency-bearing build, prepare, package/lock, container, Snap/runtime, workflow, Qt/toolchain and recursive gitlink inputs applicable to the inspected source. Candidate lists are navigation aids only. For each dependency, preserve consumed series/platform/configuration, immutable source/package identity when available, release scheme, current pin, candidate release/fix, source locations, release/advisory evidence, and whether it is shipped, system-provided, build-only or otherwise configuration-specific. Distro package revisions, Docker bases, mutable package installs, runtime/content snaps, copied source trees and patched forks remain distinct evidence domains.

Update urgency is separate from adoption readiness. Security applicability requires evidence for affected version/source presence, enabled backend/feature and reachable use; a CVE name, version string, missing upstream ancestry or matching commit subject is not by itself proof of vulnerability or of a missing fix. Patched forks/copies preserve three identities where applicable: consumed fork pin, fork head, and original upstream import provenance. A confirmed missing fix records the reviewed update/backport route and prerequisites; a hold/skip/track decision preserves the unresolved issue rather than silently resolving it.

Actionable release assessment records publication date, immutable tag/commit/package/artifact identity, source form consumed by the repository, release/advisory/regression/trust evidence, changed publisher/signer/release-process facts when relevant, unavailable checks, and the next review date or condition. Signatures/attestations/checksums are evidence only when actually verified under their stated trust assumptions and never prove benign code. Known malicious releases are not executed or downloaded merely for this audit. Build/test success, when later available, cannot by itself establish supply-chain safety.

Persistent dependency findings use stable identities derived from dependency + consumed series/platform + advisory or target version, preserve first/last seen and pending/resolved lifecycle, and survive partial or failed later audits until new evidence resolves them. Durable reports preserve prior runs and atomically replace only the current `latest` projection. A partial audit names gaps and never claims “all up to date”. The repository advisory owner may recommend update, backport, hold, skip or track, but it does not edit dependency locks, run builds/tests, apply backports, create product releases, approve deployment, or own `source/electron-main/update/*`; those are separate implementation/release responsibilities.

##### Repository single-task performer governance

The repository performer owns exactly one already selected development task at a time. It may resolve, claim, resume, implement, review, qualify evidence for, and canonically publish that task, but it may not process the inbox, drain a queue, select follow-up work, consolidate pending work, or perform the scheduler's split-routing transaction. Those are separate repository-process responsibilities. Host commands and model adapters may invoke the performer but cannot become a second state, ownership, review, evidence, or publication authority.

Task acquisition is fail-closed against canonical repository state. An already approved or already split-required task is reported without reopening it; another checkout's unfinished task is not stolen; unfinished dependencies stop acquisition. A todo task becomes active only through the canonical Start-like transition, an owned blocked task may be retried without erasing prior artifacts, and an owned in-progress task resumes without manufacturing a second state transition. Approved source prerequisites are also source-lineage requirements: a mismatch before implementation is a routing stop, while a mismatch discovered only after phase work preserves owned work and publishes only the genuine task-local boundary allowed by the repository process. Soft batch marks, receipts, local refs, and phase files remain recovery metadata rather than ownership truth.

Phase state is resumable but subordinate to the task authority. Planning and independent assessment precede source edits; assessment may reject an oversized approach or prove that the requested scope is intrinsically split-required. Implementation writes stay inside explicit owned-path sets, and interruption resumes from the first incomplete validated boundary rather than replaying completed work. A split-required performer result preserves its proposal and any retained implementation for the scheduler's later atomic routing transaction; the performer itself must not create replacement tasks or duplicate retained work.

Review requires one independent general correctness review and risk-specific specialist review under the repository's current review policy, followed by focused re-review of only findings and invariants invalidated by a fix. Prior approvals remain valid until their exact code or invariant changes. A round count never converts an unresolved finding into approval; repeated or cycling findings require an explicit convergence decision that continues, replans the current mechanism, requests rescoping, or records a genuine external hard stop. Behavior-preserving Q0/Q1/Q2 code-health work may use the bounded mechanical lane only while its class proof remains true; an item that leaves those bounds is dropped/rerouted rather than smuggled through a cheaper proof path.

Evidence qualification is claim-driven. Each selected check names the changed surface, direct instrument, oracle, observation window, positive control or negative control, falsifier, and durable output. Positive evidence is required; missing or ambiguous evidence is a test/evidence flaw, while a sound check exposing wrong behavior is an implementation defect. Recovery must move closer to the changed surface or remove a failed assumption, carry prior passing checks forward, and respect bounded campaign/convergence rules. A run-count cap, blank capture, tool interruption, or elapsed time is never by itself approval, recovery exhaustion, or a task Block.

The current Fabushi migration's FMEP-001 Stage A changes only the timing of executable qualification: these task/review/evidence contracts are authored and wired now but builds, tests, validators, runtime checks and acceptance remain `not-run` until the frozen Stage-B candidate. Source-level implementation therefore never implies `verified`, `accepted`, or release readiness.

Final publication is canonical repository state, not a source commit or chat report. An approved result, a genuine exhausted Block, or a split-required result must preserve its required task artifacts, retained-implementation disposition, exact unverified behavior, and clean/recoverable source boundary, then publish through the repository's non-force compare-and-swap path. Ordinary concurrent publication may refresh and retry only after revalidation; a semantic conflict or unavailable remote preserves resumable state. Agent interruption, missing narration, or an incomplete evidence campaign leaves the task in progress and cannot manufacture a Block or approval.


##### OpenAI single-task performer host adapter

The OpenAI `perform-task` adapter is discovery and invocation metadata only. Its display name and short description may identify the single-task performer, and its default prompt must delegate to `$perform-task` for one named ai-tdesktop task. It must not widen that invocation into queue draining, follow-up selection, split routing, consolidation, product Tasks/Automations execution, or a second repository task/runtime owner.

Host metadata never establishes task identity, ownership, review state, evidence state, validation, acceptance, or publication. Those remain with the canonical repository performer/process and quality-evidence authorities above. Provider-specific prompt wording may select the shared performer but cannot override its exactly-one-task, source-lineage, resume, review, evidence or non-force publication contracts.


##### Repository performer UI-driver governance

UI-driver selection belongs to the existing repository single-task performer and quality/evidence authorities; it does not create a shipping Computer owner, Telegram test runtime, or second UI-test state store. Deterministic in-process/repository-owned instrumentation is the default. External desktop driving is selected only when physical pointer, keyboard, focus, scrolling, dragging, native menus, windowing, or another host interaction is itself the acceptance subject; broader coverage or “more end-to-end” is not sufficient.

Every check declares its driver before execution. Hybrid checks retain deterministic fixture/setup, semantic assertions, watchdog/terminal markers and durable captures. The evidence author may specify the bounded action contract, but the one stateful performer is the sole desktop owner; concurrent agents or driver sessions must not control the same test app.

Hybrid capability is fail-closed. Before any action, prove the current approved driver path is callable, OS/app permissions are already satisfied without a new approval prompt, and the exact isolated test app can be distinguished from every user/release instance. Revalidate exact app/process/start identity before and after state/action calls. Ambiguous app names, stale targets, replacement/disappearance, focus theft, local input interference, or an interrupted driver session contaminate the run and cannot become implementation evidence.

The unattended safety envelope is disposable local test data. Do not enter credentials/secrets, approve permissions/security dialogs, mutate account/privacy/network/OS settings, perform payments, open confirmation-gated external flows, or change server-visible user content merely to obtain evidence. If the physical interaction reaches such a boundary, redesign around a safe local fixture or classify the exact capability as unavailable under the ordinary evidence-recovery contract.

Unavailable driving does not weaken the oracle. Automatic selection uses a predeclared deterministic fallback when behaviorally equivalent. A genuinely required physical interaction records the exact missing interaction/capability rather than silently substituting weaker evidence. Never drive a lock screen; prefer complete in-process instrumentation when the host can exercise the claim safely.

Exploratory driving is not qualification. Re-encode any useful discovery as a reproducible planned check against fresh task-owned fixture state. Passing evidence follows the existing hierarchy: application assertion/crash facts, semantic/in-process state and geometry, persisted accessibility state when applicable, then tightly scoped screenshots/visual artifacts. Narration alone is never proof. FMEP-001 Stage A authors these contracts only; no desktop driver, app launch, runtime UI evidence, build, test, validator or acceptance is executed.

## Repository performer mechanical-lane contract (fixed-U closure)

Fixed-U authority `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` specializes the existing source-neutral single-task performer; it does not create a shipping runtime owner. Q0/Q1/Q2 are eligible only while behavior is unchanged and the source-defined bounds remain true. Q0 deletes only unreferenced code plus exclusively-used residue; Q1 moves code without logic edits beyond declarations/includes/namespace/CMake scaffolding; Q2 merges copies identical apart from names/literal values into an already-linkable shared function with those differences passed as arguments. New strings/styles/settings/log behavior, reordered side effects, templates, header-defined functions or variant dispatch are outside the lane.

The performer rechecks each finding against the exact task base before edits. A present item remains actionable; an unknown/stale item is read at its named source location and dropped only when gone or no longer class-conformant. `work/owned-paths.txt` precedes edits and is limited to task-named files, the legal receive/shared file, their CMake lists, direct-user include lines, relevant area-map module lines, and style/language entries proven exclusive to removed code. Defects and adjacent cleanup discovered on the way are follow-up work, never opportunistic edits.

Any item that needs out-of-bounds edits, differs semantically, retains a platform/generated-input reference, cannot satisfy its class proof, or fails the bounded review is restored/dropped rather than promoted to the full pipeline inside the same task. Every edit or drop invalidates affected proof and requires the surviving diff to be reproved.

Proof is exact-revision and class-specific: changed-target Debug/build coverage, repository health with no change-caused error/warning, Q1 move-only proof or explicit declaration/signature-only explanation, and candidate verification/metric. One fresh independent reviewer reads the complete diff, full changed files and proof outputs, checking behavior, completeness, bounds, proofs and placement. One bounded fix plus focused resume is allowed; a second NEEDS_CHANGES drops the named items and there is no third review.

Publication preserves canonical performer state. Retained change publishes the source commit, `work/test.md`, the final mechanical review and a Code health section with Resolved/Dropped/Metric. An all-dropped result is `already-satisfied`, with no source change/review and with the recheck/proof for each drop. Under FMEP-001 Stage A authors these obligations only: `validation_status=not-run`, `accepted=false`; no health command, build, test, reviewer execution, validator, CI or acceptance is run.



## Repository performer phase-prompt contract (fixed-U closure)

Fixed-U authority `.agents/skills/perform-task/references/phase-prompts.md@f62400b00933fe3757d90b809e456b200389c65c` is the exact leaf-prompt and completion protocol for the existing source-neutral single-task performer. It does not own canonical task state, queue selection, product Tasks/Automations, Agent/Coordinator/Host/Runner execution, or a second review/evidence store. The stateful performer remains the only task owner; phase workers are disposable executors whose artifacts are accepted only after the performer validates the source-defined completion conditions.

Each delegated phase receives a written prompt before execution and writes a bounded result record afterwards. Leaf workers have unique task names, do not delegate, do not commit, and return only their phase status/artifacts/touched paths/blocker. Delegation is selected only after the first real phase spawn succeeds; an immediate depth/capacity/policy rejection chooses the source-defined same-session fallback rather than counting as a failed delegated attempt. Where delegation is selected, an incomplete returned Phase 1/3/4/6 leaf gets at most one fresh retry with more specific instructions after the original worker is known stopped; another failure becomes a recoverable hard stop to the caller, never a locally replayed phase or canonical task Block. Host-specific mechanics may alter how a leaf is spawned or resumed, but cannot alter these ownership or publication boundaries.

Phase artifacts are load-bearing contracts, not narration. Context/plan must establish the exact task, source locations, analogous repository mechanisms, bounded files/steps, expected surfaces, evidence plan and status without source edits. Independent assessment may refine the plan, reject an undersized fast path, reject an over-coupled approach for a fresh Phase 1 rerun, or return intrinsic `split-required` with a proposal and no source edit. Implementation units are restricted to their declared owned write sets; selected pre-review validation is the most direct assessed check. Initial review uses one mandatory general review plus the applicable independent lenses, while fixes invalidate only exact findings/invariants and therefore use focused re-review with unaffected approvals carried forward. Convergence is explicit `CONTINUE | REPLAN_CURRENT | RESCOPE_REQUIRED | HARD_STOP` and never approves code by round count.

Evidence recovery likewise stays inside the existing performer evidence state machine. A repeated evidence failure creates a fresh recovery plan that preserves prior positive proof, names the failed assumption and forbidden repeated technique, and chooses a more direct safe instrument. Recovery edits only the evidence path unless sound evidence proves an implementation bug. Campaign caps are independent assessment points: they may authorize one focused recovery campaign, declare actual exhaustion, or stop on a concrete human/environment boundary; elapsed runs, ambiguous screenshots, missing narration or a failed disposable worker cannot become approval or Block evidence.

Under FMEP-001 Stage A these prompt/worker/review/evidence contracts are authored and mapped only. No phase worker, build, validation command, reviewer, evidence campaign, UI driver, test, package, deployment, installation or acceptance is executed; `validation_status=not-run` and `accepted=false` remain mandatory.
## Repository performer end-to-end pipeline contract (fixed-U closure)

Fixed-U authority `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493` is the end-to-end composition contract for the existing source-neutral repository single-task performer. It does not create a second performer, queue, review/evidence store, product Tasks/Automations runtime, Agent/Coordinator/Host/Runner owner, or platform-core owner.

Preflight is fail-closed. Resolve one selected task, exact source/spec/project context, task-local work paths, recovery refs and a single consistent host/build/evidence inventory before phase work. Canonical task ownership and approved source lineage are checked before Phase 1. Fresh source preparation may initialize recorded submodules only while preserving local edits and refusing unsafe worktree overlap; an interrupted run may retain dirty source only when every path is already declared task-owned and completed artifacts prove that ownership. Unexpected paths are never stashed, reset, restored, staged, committed or deleted merely to make the performer proceed.

Task-local context, plan, visual contract when applicable, assessment, review, test design/report, result, owned-path set, prompt/result logs and compact evidence are resumable artifacts rather than a second ownership truth. Resume starts at the first incomplete validated boundary and carries completed approvals/evidence forward. The performer remains the only stateful owner; leaf workers never commit or publish. Assessment precedes source edits, may reject an undersized/over-engineered approach, or may return intrinsic split-required. Implementation stays inside assessed owned write sets, selected pre-review validation is direct rather than ceremonial, review uses independent initial reads and focused invalidation-based re-review, and evidence checks require a declared window, control and falsifier.

The evidence-loop adapter preserves the existing performer evidence state machine: `APPROVED`, `TEST_FLAW`, `IMPL_BUG` and `UNRECOVERABLE` remain distinct. Test flaws move toward a more direct instrument without discarding retained implementation or prior valid proof; implementation bugs return to retained implementation plus focused review and only invalidated checks. Campaign caps are assessment checkpoints, never success or task Block. An already-satisfied result requires direct positive proof and no manufactured source edit.

Final publication is a canonical repository-process boundary, not a source commit or chat message. The result records status/outcome, retained-implementation disposition, split proposal when applicable, attempts/test runs, UI-driver disposition, touched paths, evidence, exact unverified behavior and checkout condition. Approval requires complete positive evidence; a Block requires a genuine exhausted/unavailable condition under the recovery contract; split-required preserves owned work for the scheduler. Publication remains non-force compare-and-swap and preserves recoverable state on semantic conflict or remote outage. FMEP-001 Stage A authors these contracts only: no build, validator, review execution, test/evidence campaign, UI-driver run, package, deployment, installation or acceptance is executed.
