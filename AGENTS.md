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

### 4. Mandatory end-to-end development lifecycle

AI development must follow this sequence unless an applicable project spec defines a stricter superset:

**Discover → Confirm Goal → Spec → Current-State Verification → Architecture → Task Decomposition → Test Design → Implement → Layered Verification → Failure/Recovery Verification → Exact-HEAD CI → Packaged Acceptance → Independent Acceptance → Spec Compliance Review → Protected Integration → Canonical-Main Verification → Release → Post-Release Smoke Test → Evidence Archive → COMPLETE**

The lifecycle is fail-closed: a later stage must not be treated as successful when a required earlier gate is incomplete, failed, stale, or unsupported by evidence.

#### Stage 0 — Discover
- Read this root `AGENTS.md` and any applicable nested agent instructions.
- Locate the owning project, source of truth, applicable specs, tasks, decisions, contracts, and previous evidence.
- Inspect only enough current code and live repository state to understand the task and repair/write the spec.
- Do not assume chat memory, an old work-session summary, an old branch, or a previous release describes the current system.

#### Stage 1 — Confirm the goal
- State the current observable problem or requested change.
- State the target user-visible or externally observable outcome.
- State scope and non-goals.
- Identify whether the work affects architecture, runtime/process boundaries, protocols, schemas, UI/UX, security, performance, migration, packaging, or release.
- Resolve contradictions in favor of the latest explicit authoritative requirement and record the resolution durably.

#### Stage 2 — Spec
- Read the applicable durable spec completely.
- Create or repair the spec before implementation when it is missing, stale, incomplete, or contradictory.
- Define requirements, edge cases, verification, acceptance criteria, and Definition of Done before coding.
- For non-trivial work, assign stable requirement and acceptance-criterion IDs.
- A spec records durable truth: required behavior, architecture, decisions, constraints, and final acceptance state. It is not a minute-by-minute development log.

#### Stage 3 — Verify the current state
Before deriving a plan, verify the live repository facts that materially affect the task, including when applicable:
- canonical/default branch and exact source SHA;
- active development branch and PR head SHA;
- actual code/module structure and ownership boundaries;
- current dependency and protocol versions;
- open PRs or known conflicting work;
- relevant CI/workflow status;
- current application/release version and published artifacts.

Do not implement against a repository shape that has not been verified.

#### Stage 4 — Architecture
- Derive the technical design from the spec.
- Define state ownership, runtime/process ownership, allowed dependency direction, interfaces, data/control flow, cancellation, timeout, retry, reconnect, resync, crash settlement, migration, and observability as applicable.
- Prefer explicit boundaries over convenience coupling.
- Record important architecture decisions durably in the spec or an applicable decision record before or together with implementation.

#### Stage 5 — Task decomposition
- Break non-trivial work into small, traceable tasks with one clear objective and an independently verifiable result.
- Map implementation tasks back to requirement/acceptance IDs.
- Avoid large unstructured batches that mix unrelated architecture, feature, migration, and release changes.

#### Stage 6 — Test design
Before implementation, define how each requirement will be proven.

Use the relevant layers:
- static/type/format/architecture checks;
- unit tests;
- contract/protocol/schema tests;
- integration tests;
- E2E/user-flow tests;
- regression tests;
- failure/recovery tests;
- migration/rollback tests;
- performance/power/security checks;
- packaged-app acceptance;
- release/update acceptance.

Tests must prove the requirement; requirements must not be weakened merely to make existing tests pass.

#### Stage 7 — Implement
- Implement against the durable spec and verified current code, not chat memory.
- Stay inside declared scope and preserve architecture boundaries.
- Keep temporary workarounds explicitly temporary and tracked.
- If implementation reveals that the intended behavior or architecture must intentionally change, update the spec/decision record before or together with that change.
- Do not leave intentional behavior undocumented.

#### Stage 8 — Layered verification
Run verification from the cheapest/narrowest useful layer toward the broadest:

**Static/Architecture → Unit → Contract → Integration → E2E → Regression → Packaged App**

- Diagnose failures at the lowest layer that can explain them instead of repeatedly rerunning a large E2E suite without evidence.
- Verify all affected core flows, not only the newly added happy path.
- A source-level test pass is not equivalent to an application-level pass.

#### Stage 9 — Failure and recovery verification
For stateful, distributed, agent, networked, or long-running features, explicitly test relevant abnormal paths, such as:
- network interruption and recovery;
- timeout and cancellation;
- process/host/runner crash and restart;
- application restart;
- reconnect and resync;
- stale or duplicate events;
- partial responses and partial failure;
- authentication/OAuth failure;
- MCP/tool failure;
- migration interruption and rollback.

The system must not be accepted only because the happy path works.

#### Stage 10 — Exact-HEAD GitHub Actions verification
- Bind CI evidence to the exact source revision being accepted.
- Record the PR/branch head SHA and the workflow run that tested that SHA.
- Required checks must be successful for that exact head; success from an earlier SHA is stale evidence.
- When repository policy requires GitHub Actions as the authoritative test/build environment, local success is supplementary only.

Minimum evidence shape when applicable:

```text
PR HEAD = <exact SHA>
workflow run = <run ID/URL>
required checks = PASS
```

#### Stage 11 — Real packaged-app acceptance
A passing source build, unit suite, or E2E suite does not prove the distributed product works.

When the deliverable is an installable application, verify the actual produced package/artifact, including as applicable:
- build and packaging;
- embedded sidecars/native binaries/resources;
- signing;
- notarization/stapling;
- updater metadata;
- checksums;
- installation/launch;
- critical user flows from the installed package.

Examples include DMG/ZIP, EXE/MSI, AppImage/deb/rpm, TestFlight builds, APK/AAB, or other shipping artifacts.

#### Stage 12 — Independent acceptance
For non-trivial, release-bound, architecture-sensitive, or high-risk work, use an acceptance pass independent from the implementation reasoning.

The acceptance pass must compare:

**Spec ↔ Final Diff/Code ↔ Exact-Source CI ↔ Packaged Behavior ↔ Evidence**

It must not treat the implementation session's completion claim as proof.

#### Stage 13 — Spec Compliance Review
Before declaring implementation complete, compare the final result against **every applicable requirement and acceptance criterion**.

Record each criterion as:
- `passed`, with evidence;
- `blocked`, with reason;
- `not-applicable`, with reason.

Any intentional design/behavior divergence must already be reflected in the durable spec or decision record.

#### Stage 14 — Protected integration
Only integrate when all required pre-merge gates are satisfied.

Typical order:

```text
development branch
→ PR
→ exact-HEAD required CI
→ packaged acceptance when applicable
→ review / independent acceptance
→ spec compliance
→ protected merge
→ canonical main
```

Do not merge merely because code was written, pushed, or partially tested.

#### Stage 15 — Canonical-main verification
- Treat the canonical-main merge SHA as a new release source identity.
- Do not assume PR-head verification automatically proves the merged canonical revision.
- Verify that any post-merge build/release workflow checks out and binds to the exact canonical-main SHA.
- Re-run the required canonical-main gates when repository/release policy requires them.

#### Stage 16 — Release
Release only from the accepted canonical source revision and satisfy all applicable release gates, including:
- version correctness and monotonic versioning where required;
- build/package success;
- signing/notarization/stapling;
- updater metadata;
- checksums and artifact publication;
- store/TestFlight/internal-track submission where applicable;
- release notes and migration/rollback readiness when required.

A PR merge is not a release, and a candidate package is not a completed release.

#### Stage 17 — Post-release smoke test
Validate the artifact obtained from the real release/distribution channel.

At minimum, exercise the critical path relevant to the change, for example:

```text
download/install
→ launch
→ authenticate
→ core user flow
→ restart/recovery
→ update check/update path when applicable
→ logout/clean exit
```

Release completion requires evidence that the actually published artifact works, not only the pre-release candidate.

#### Stage 18 — Evidence and durable-record archive
Before closing the task, preserve the evidence needed for a future engineer or AI agent to reproduce the acceptance decision.

Record as applicable:
- final canonical source SHA;
- PR/merge reference;
- workflow run IDs/URLs;
- release version/tag;
- artifact identifiers/checksums;
- test reports;
- screenshots/video;
- logs/traces;
- migration/rollback result;
- known limitations or intentionally deferred work.

Keep detailed action-by-action implementation history in commits, PRs, task/work logs, and CI. Keep the spec focused on durable requirements, architecture, decisions, and final compliance/evidence references.

#### Stage 19 — COMPLETE gate
A task may be marked `COMPLETE` only when all applicable requirements and delivery gates are satisfied and evidence-backed.

Repository-wide default hard rules:

> **No Spec, No Code.**
>
> **No Evidence, No Complete.**
>
> **No required exact-source verification, No Acceptance.**
>
> **No canonical-main packaged/release verification when applicable, No Release Complete.**

Partial implementation, a green unit suite, a successful PR build, a merged PR, or an uploaded candidate artifact is not by itself sufficient to claim end-to-end completion.


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
