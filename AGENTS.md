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

#### Stage 1 — Spec
- Read the applicable spec completely.
- Reconcile it with the latest explicit user requirement.
- Create or update the spec before implementation if needed.
- Make scope, constraints, edge cases, and Definition of Done explicit.

#### Stage 2 — Architecture / Plan
- Derive the technical design from the spec.
- Identify ownership boundaries, affected interfaces, migration needs, risks, and verification.
- Break implementation into traceable tasks for non-trivial work.

#### Stage 3 — Implement
- Implement against the durable spec, not chat memory.
- Stay within declared scope.
- Preserve architectural boundaries.
- Do not weaken requirements or acceptance criteria merely to make checks pass.

#### Stage 4 — Verify
- Execute or inspect the verification required by the spec and repository workflow.
- Verify the exact source revision being accepted.
- Check relevant regression, architecture, security, performance, power, migration, and compatibility requirements when the spec makes them applicable.

#### Stage 5 — Spec Compliance Review
Before declaring completion, compare the final implementation against **every applicable requirement and acceptance criterion**.

Record each criterion as:
- passed, with evidence;
- blocked, with reason;
- not applicable, with reason.

If the implementation intentionally changed the design or behavior, update the spec/decision record before completion. Never leave intentional behavior undocumented.

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
