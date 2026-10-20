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

##### Repository responsibility-atlas governance

A repository responsibility atlas is source-neutral quality metadata, not a product runtime or a second ownership graph. It divides an exact source authority into stable named areas with one clear product/engineering responsibility each; path classification must be deterministic and complete, with exact-file rules more specific than globs and globs more specific than directory prefixes. Every in-scope source file must resolve to exactly one approved area, layer ordering must express intended dependency direction, and a known boundary violation is recorded explicitly rather than widening an area's responsibility to normalize the violation.

Atlas generation and amendment are evidence-bearing review work. Build a draft only from a freshly resolved exact source snapshot and retain the source facts used to suggest boundaries, such as file/line size, dependency edges, co-change, churn, task cost and test reach. Those measurements are heuristics for investigation, not authority: independent source readers describe responsibilities, consolidation decides split/merge/placement judgments against the source, and an independent verifier checks the claims plus unmapped, ambiguous and stale-pattern negatives. Drafts do not become repository instructions until the declared owner approves the text; later structural drift produces the narrowest amendment needed, area maps must remain consistent with the atlas, and stable area names are preserved because renaming changes historical identity.

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

Read-only investigation, spec/test-plan editing, evidence review and Git/API operations may occur outside Actions. Missing runner/account/service/signing configuration is blocked, never pass.
