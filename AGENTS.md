# Fabushi Desktop — Agent Instructions

These instructions apply repository-wide to all AI-assisted development in `bhrumom/fabushi-desktop`, unless a more specific nested `AGENTS.md` applies to a subtree.

## CRITICAL: mandatory standard project lifecycle

Every AI agent performing product-affecting work must follow:

**Discover → Duplicate-work check → Classify → Spec → Architecture/ADR → Migration → Plan → Implement → Verify → Compliance Review → PR/Merge → Release → Post-release verification**

The depth of a stage may scale with the task, but required gates do not disappear because a change looks small.

Read-only investigation is allowed before a Spec is complete when needed to understand current state or repair/create the Spec. Product-affecting implementation is not.

## 1. Discover before implementation

At the start of every development task:

1. Read this root `AGENTS.md`.
2. Locate the matching project under `projects/`, if one exists.
3. Read the project's applicable source of truth and normalized documents, especially when present:
   - `SOURCE_OF_TRUTH.md`
   - `docs/01-范围与非目标.md`
   - `docs/02-需求与成功指标.md`
   - `docs/03-架构与实现策略.md`
   - `docs/04-质量与测试策略.md`
   - `docs/19-完成定义与验收.md`
   - relevant `management/tasks/*.md`, decisions, contracts, source records, and evidence.
4. Read the applicable task/feature Spec under `docs/specs/`.
5. Read relevant current architecture under `docs/architecture/`.
6. Read relevant ADRs under `docs/adr/`.
7. Inspect the current code and live GitHub branch/PR/CI/release facts needed for the task.
8. Reconcile durable docs with the latest explicit user requirement and exact repository state.

### Mandatory duplicate-work check — before writing code or creating a new implementation branch

Before any product-affecting implementation, the agent must verify that the same or materially overlapping work is not already being implemented.

Check, at minimum:

1. the current checked-out branch and canonical base/main SHA;
2. active project tasks and applicable Specs for the same requirement IDs, feature, bug, architecture change, or migration;
3. relevant remote branches, including branch names and their actual diffs/commits rather than names alone;
4. **all open PRs** whose scope, Spec, issue, touched subsystem/files, or stated goal overlaps the task;
5. relevant recently merged PRs/commits that may already satisfy or supersede the request;
6. active CI/release work associated with the overlapping branch/PR when delivery status matters.

The check must be semantic, not keyword-only: different branch/PR names can still implement the same requirement.

If materially overlapping work exists:

- **do not create a second parallel implementation by default**;
- inspect the existing branch/PR exact HEAD, diff, Spec/ADR/migration links, review state, CI state, blockers, and remaining acceptance criteria;
- continue, repair, review, rebase, or complete the existing work when it is the authoritative/latest implementation;
- reuse existing code and commits where possible instead of re-writing the same behavior;
- if the existing work is stale, invalid, abandoned, or contradicts the latest authoritative requirement, record that finding in the owning Spec/PR and explicitly supersede/replace it rather than silently creating duplicate code;
- if multiple overlapping implementations already exist, consolidate around one canonical path before adding more implementation;
- create an independent replacement only when the latest explicit user requirement or governing Spec actually requires one.

Before creating a new implementation branch, the agent should be able to state either:

- `duplicate-work check: no materially overlapping active branch/PR found`; or
- `existing work found: <branch/PR/SHA>; continuing/superseding it for <documented reason>`.

Do not implement from chat memory alone. Do not assume a new chat means a new code path is needed.

## 2. Classify the change

Before implementation, classify the work. A task may have multiple classifications:

- bug fix;
- feature;
- architecture change;
- contract/schema/protocol change;
- migration;
- security/privacy change;
- dependency/build/tooling change;
- release/operations change;
- documentation-only change.

Treat a task as an **architecture change** when it changes any durable property such as:

- module/process ownership;
- authoritative state ownership;
- dependency direction;
- cross-module/runtime protocol;
- persistence/storage model;
- failure/recovery semantics;
- security/capability boundary;
- deployment topology;
- replaceable runtime boundary;
- durable framework/runtime choice.

## 3. Spec-first development — No Spec, No Code

Every AI agent **must read the applicable durable specification before starting product-affecting implementation**.

Product-affecting development includes changes to application/runtime code, tests, schemas, contracts, dependencies, build/release configuration, migrations, security controls, user-visible assets, or other behavior-affecting files.

A file named `*.spec.ts` is a test specification, not automatically the product/development Spec required by this rule.

### Spec discovery order

1. This root `AGENTS.md`.
2. Matching project under `projects/`.
3. Task/feature-specific Spec under `docs/specs/`.
4. Architecture/ADR/contracts referenced by the owning project/Spec.
5. Latest explicit user requirement and live GitHub facts used to validate or amend the durable record.

### If no usable Spec exists

Stop before implementation and create or repair one first.

Default location:

`docs/specs/<short-kebab-case-task-name>.md`

Use:

`docs/specs/SPEC_TEMPLATE.md`

If the task already belongs to a governed project under `projects/`, prefer updating that project's source-of-truth/spec/task records instead of creating a conflicting parallel Spec.

### Minimum Spec contents

At the appropriate depth, define:

- context/problem;
- goal;
- non-goals/out of scope;
- functional requirements;
- current state and target state;
- architecture/ownership boundaries;
- interfaces/contracts/schemas/data/control flow when applicable;
- constraints and non-functional requirements;
- failure modes and edge cases;
- implementation strategy;
- verification/test strategy;
- acceptance criteria / Definition of Done;
- release, migration, rollback, and observability when applicable;
- references/provenance.

For non-trivial work, requirements and acceptance criteria should use stable IDs so implementation and evidence can be traced back to the Spec.

A small fix may use a short Spec, but it still needs a durable statement of required behavior and completion criteria.

The canonical detailed policy remains `docs/specs/spec-first-ai-development.md`.

## 4. Architecture / ADR gate

For architecture-affecting work, before implementation:

1. Read `docs/architecture/README.md`, `system-overview.md`, and `invariants.md`.
2. Update the applicable Spec with the proposed boundary/state/protocol change.
3. Create or supersede an ADR in `docs/adr/` for the durable decision.
4. Update canonical architecture docs to describe the accepted target/current architecture.
5. If implementation will temporarily differ from the canonical target, create/update a migration record that explicitly describes the transitional state.
6. Add or update architecture checks where the invariant can be enforced automatically.

Do not use a one-off Spec as the only permanent record of a durable architecture decision.

Accepted ADRs are historical records. Do not rewrite history to make a later decision appear original. Supersede an ADR with a new ADR.

## 5. Migration gate

Create/update `docs/migrations/` when a change requires a controlled transition involving any of:

- persistent data/schema changes;
- protocol or IPC compatibility changes;
- runtime/process ownership transfer;
- authoritative state ownership transfer;
- replacement/removal of legacy architecture;
- phased rollout, dual-read/write, compatibility adapters, backfill, or rollback sequencing.

A migration must define:

- old state;
- transitional state;
- target state;
- sequencing/preconditions;
- compatibility behavior;
- failure/restart behavior;
- rollback triggers and rollback path;
- verification/evidence;
- legacy-removal criteria;
- completion criteria.

A new path existing does not mean a migration is complete. Close the migration only after the transitional/legacy path is removed or explicitly adopted as permanent by a durable decision.

## 6. Plan from the Spec

Before implementation, derive the technical plan from the durable records:

- affected ownership boundaries;
- interfaces/contracts;
- files/components;
- risks/failure modes;
- migration steps;
- requirement-to-test mapping;
- release/rollback impact.

For non-trivial work, keep implementation tasks traceable to requirement IDs.

## 7. Implement

- Implement against the durable Spec, not chat memory.
- Stay within declared scope.
- Preserve `docs/architecture/invariants.md`.
- Keep privileged behavior behind explicit capability/security boundaries.
- Do not allow a temporary compatibility adapter to silently become permanent architecture.
- Do not weaken requirements, tests, architecture checks, security controls, or acceptance criteria merely to make work pass.
- If implementation must intentionally differ from the durable design, update the Spec/ADR/architecture/migration record first.

## 8. Verify

Use the verification layers required by the affected system and governing Spec. See `docs/testing/strategy.md`.

Verification may include:

- static/type/format checks;
- architecture/invariant checks;
- unit tests;
- contract/protocol tests;
- integration tests;
- Electron/renderer E2E;
- failure/recovery tests;
- migration/rollback tests;
- security/privacy checks;
- performance/power checks;
- real packaged-app acceptance;
- release/updater/signing/notarization checks.

When a Spec/repository workflow requires GitHub Actions or exact-source evidence, local results are supplemental only.

**Never use a green run from an older SHA to accept a newer SHA.**

Do not claim packaged behavior from source tests when packaging/signing/process wiring/updater behavior is part of the risk.

## 9. Spec compliance review

Before declaring implementation complete, compare the final implementation against **every applicable requirement and acceptance criterion**.

Record each as:

- `passed`, with evidence;
- `blocked`, with reason;
- `not-applicable`, with reason.

Also verify:

- intentional architecture changes are reflected in canonical architecture docs;
- durable decisions are reflected in ADRs;
- transitional work is reflected in migrations;
- relevant regression/failure paths are tested;
- documentation does not claim a state newer than actual GitHub/CI/release evidence.

If implementation intentionally differs from the Spec, update the durable record before completion.

## 10. PR / merge

For non-trivial work, use a reviewable branch/PR when repository permissions/workflow allow it.

Use `.github/PULL_REQUEST_TEMPLATE.md`.

The PR must identify:

- applicable Spec;
- owning project/source of truth;
- related ADR(s);
- architecture/migration impact;
- exact head SHA;
- requirement-to-evidence summary;
- tests/workflows and artifact evidence;
- risks and rollback;
- deferred/blocked items.

Do not claim **merged** until canonical `main` actually contains the accepted change.

## 11. Release / delivery

If the task requires release/publication, merge is not completion.

Follow `docs/operations/release-and-evidence.md` and the governing Spec. When applicable verify:

- canonical main merge SHA;
- intended version;
- release workflow bound to exact source SHA;
- required build/test gates;
- signing/notarization/stapling;
- updater metadata;
- checksums;
- installable artifacts;
- packaged acceptance;
- publish step actually succeeded rather than being skipped;
- published release assets.

Do not call a candidate package a published release.

## 12. Post-release verification and closure

When release is in scope:

- validate the real published/installed artifact;
- verify reported version and key user journeys;
- verify updater behavior and no stale/same-version update loop where applicable;
- collect Spec-required logs/screenshots/video/traces;
- update the Spec compliance record;
- update `CHANGELOG.md` only with verified release information;
- close migration records only when transitional/legacy paths are actually closed.

## 13. Canonical documentation model

Use each document class for one purpose:

- `docs/architecture/` — **what the architecture is now / what boundaries are canonical**;
- `docs/adr/` — **why a durable architecture decision was made and what it supersedes**;
- `docs/specs/` — **what this specific change must achieve and how it will be verified**;
- `docs/migrations/` — **how the repository safely moves from old state to target state**;
- `docs/testing/` — **how correctness/evidence is established**;
- `docs/operations/` — **how changes are released, rolled back, and operationally verified**;
- Git commits/PRs/Actions/releases — **what actually changed and what evidence exists**.

Do not turn one giant Spec into a substitute for all these layers.

## 14. Completion-state vocabulary

Use precise status language:

- **specified** — durable requirements exist;
- **implemented** — source changes exist;
- **source-verified** — source-level checks passed;
- **CI-verified** — required exact-head CI passed;
- **merged** — canonical main contains the accepted change;
- **release-candidate verified** — candidate package passed required gates;
- **released** — publication actually succeeded;
- **post-release verified** — published/installed artifact passed required real-world verification;
- **blocked** — a named requirement/gate is unsatisfied.

Never collapse these into one unsupported “done”.

## 15. Fail-closed rules

The following are prohibited:

- starting product-affecting implementation without first reading an applicable Spec;
- skipping Spec creation because a requested change appears obvious;
- using the chat prompt as the only persistent specification for substantive work;
- inventing requirements from memory when an authoritative source exists;
- silently expanding or reducing scope;
- creating a new parallel implementation without first checking active/relevant branches, Specs/tasks, open PRs, and recent merged work for material overlap;
- duplicating an existing active implementation instead of continuing/consolidating it, unless an independent replacement is explicitly required;
- making an architecture change without Architecture/ADR review;
- making an incompatible transition without a migration record;
- rewriting accepted ADR history instead of superseding it;
- deleting, bypassing, or weakening acceptance criteria/tests/security controls to make work appear complete;
- accepting a newer SHA with evidence from an older SHA;
- claiming merge/release/publication/post-release success without corresponding evidence;
- leaving Specs, architecture docs, ADRs, migrations, or changelog intentionally stale after a behavior/design change;
- inventing legal/ownership metadata such as LICENSE terms or CODEOWNERS identities.

When a required durable record is missing, unclear, contradictory, or stale, **repair the record or gather evidence instead of guessing and coding**.

## 16. Repository governance entry points

- Product definition: `PRODUCT.md`
- Roadmap: `ROADMAP.md`
- Documentation map: `docs/README.md`
- Architecture: `docs/architecture/README.md`
- Architecture invariants: `docs/architecture/invariants.md`
- ADR process: `docs/adr/README.md`
- Specs: `docs/specs/`
- Migration process: `docs/migrations/README.md`
- Testing strategy: `docs/testing/strategy.md`
- Release/evidence: `docs/operations/release-and-evidence.md`
- Incident response: `docs/operations/incident-response.md`
- Contributing: `CONTRIBUTING.md`
- Security: `SECURITY.md`
