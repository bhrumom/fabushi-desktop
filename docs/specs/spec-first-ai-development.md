# Spec-first AI development governance

Status: active  
Date: 2026-09-21  
Scope: repository-wide AI-assisted development in `bhrumom/fabushi-desktop`

## 1. Problem

AI agents can begin implementation from a chat prompt or partial task description without first establishing a durable, testable specification. That creates scope drift, architecture drift, unverifiable completion claims, and repeated rework.

This repository therefore uses a mandatory **Spec-first / No Spec, No Code** development gate.

## 2. Goal

Before any AI agent changes implementation code, tests, runtime configuration, build/release configuration, schemas, contracts, or other product-affecting files, it must:

1. find and read the applicable durable specification;
2. reconcile it with the latest explicit user requirement and current repository facts;
3. create or update the specification first when it is missing, incomplete, or stale;
4. derive implementation and verification work from that specification.

Read-only investigation needed to understand the system or write the specification is allowed before the spec is complete. Product-affecting implementation is not.

## 3. Non-goals

- This rule does not require a large document for every tiny change. A small change may use a short spec, but it must still be durable and testable.
- This rule does not replace source inspection, issue/PR context, CI evidence, or user requirements.
- A test file named `*.spec.ts` is not automatically a product specification. “Spec” in this policy means the durable requirements/design/acceptance source for the work.

## 4. Canonical spec discovery order

Before implementation, the agent must search in this order:

1. Root `AGENTS.md`.
2. A matching project under `projects/<project>/`:
   - `SOURCE_OF_TRUTH.md`
   - `docs/01-范围与非目标.md`
   - `docs/02-需求与成功指标.md`
   - `docs/03-架构与实现策略.md`
   - `docs/04-质量与测试策略.md`
   - `docs/19-完成定义与验收.md`
   - relevant `management/tasks/*.md`, decisions, contracts, and source records.
3. Task/feature-specific specifications under `docs/specs/`.
4. Other authoritative architecture/contracts/docs named by the applicable source of truth.
5. Latest explicit user requirement and live GitHub code/PR/CI/release facts, used to validate or amend the durable spec.

Chat memory alone is not a durable specification.

## 5. Requirement: create the spec when none exists

If no applicable durable spec exists, the AI agent must create one under:

`docs/specs/<short-kebab-case-task-name>.md`

before implementation.

If the work already belongs to a governed project under `projects/`, prefer creating/updating the project’s normalized spec and task records instead of creating a parallel standalone spec.

Use `docs/specs/SPEC_TEMPLATE.md` as the minimum structure.

## 6. Minimum spec contents

Every implementation spec must contain enough information to make the work executable and objectively verifiable. At minimum:

1. **Context / problem**
2. **Goal**
3. **Non-goals / out of scope**
4. **Requirements** with stable IDs when the work is non-trivial
5. **Current state and target state**
6. **Architecture / ownership boundaries**
7. **Interfaces, contracts, schemas, data/control flow** when applicable
8. **Constraints and non-functional requirements**, including performance, power, security, compatibility, migration, or operational constraints when relevant
9. **Failure modes and edge cases**
10. **Implementation strategy**
11. **Verification / test strategy**
12. **Acceptance criteria / Definition of Done**
13. **Release, migration, rollback, and observability plan** when applicable
14. **References / provenance**, including the user requirement and relevant source/decision links

A spec is insufficient if it only says what file to edit without defining the required behavior and completion criteria.

## 7. Standard AI development lifecycle

Every AI development task follows this order:

### Stage 0 — Discover
- Read root instructions.
- Locate the matching project/spec/source of truth.
- Inspect current code and live GitHub state needed to understand the task.

### Stage 1 — Spec
- Read the existing spec completely.
- Compare it with the latest explicit user requirement.
- Create or update the spec before implementation if required.
- Resolve contradictions in favor of the latest authoritative requirement, while preserving history when the project requires it.

### Stage 2 — Architecture and plan
- Derive the technical design from the spec.
- Identify ownership boundaries, interfaces, migrations, risks, and affected tests.
- Break the work into traceable implementation tasks.

### Stage 3 — Implement
- Implement against the spec, not against chat memory.
- Keep changes within the declared scope.
- Do not weaken acceptance criteria merely to make tests pass.

### Stage 4 — Verify
- Run or inspect the verification required by the spec and repository workflow.
- Use exact-source CI/build/test evidence where required.
- Check regressions, architecture boundaries, and non-functional requirements applicable to the change.

### Stage 5 — Spec compliance review
Before declaring completion, compare the final diff and behavior against every relevant requirement and acceptance criterion.

For each criterion, record one of:
- passed with evidence;
- blocked with reason;
- not applicable with reason.

If implementation intentionally differs from the spec, update the spec/decision record first so the repository does not end with undocumented behavior.

### Stage 6 — Integrate and deliver
- Follow the repository’s branch/PR/merge/release rules.
- Do not claim the task is complete merely because code exists.
- Completion requires the spec’s Definition of Done and the applicable integration/delivery gates to be satisfied.

## 8. Fail-closed rules

The following are prohibited:

- starting product-affecting implementation when no applicable spec has been read;
- inventing requirements from memory when a spec exists;
- treating a chat prompt as the only persistent spec for substantive work;
- silently changing scope during implementation;
- deleting/weakening acceptance criteria to make a change appear complete;
- claiming completion without spec-to-evidence verification;
- leaving the durable spec stale after an intentional design or behavior change.

When the spec is missing, unclear, contradictory, or stale, the agent’s next development action is to repair the spec, not to guess and code.

## 9. Acceptance criteria for this governance change

- AC-1: repository root contains an `AGENTS.md` with an explicit mandatory Spec-first / No Spec, No Code gate.
- AC-2: `AGENTS.md` defines spec discovery, missing-spec creation, standard lifecycle, and completion review.
- AC-3: `docs/specs/SPEC_TEMPLATE.md` exists and contains the minimum required structure.
- AC-4: the policy is merged to canonical `main` and read back from `main`.
