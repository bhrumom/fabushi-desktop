# Mature Project Governance Baseline — Specification

Status: active  
Owner: Fabushi desktop / repository governance  
Last updated: 2026-09-22  
Related project: repository-wide  
Related task / issue / PR: user request on 2026-09-22; GitHub PR #22

## 1. Context / problem

The repository already has a strong Spec-first AI-development gate, active feature/refactor specs, exact-source GitHub Actions, and project-specific source-of-truth documents. However, repository-wide governance is incomplete: there is no canonical architecture documentation tree, ADR process, migration-record process, contribution guide, security-reporting guide, changelog baseline, PR template, issue templates, or unified testing/release evidence guide.

Without these durable layers, architecture decisions can be buried inside one-off specs, current-state architecture can drift from implementation, migration history can be lost, and AI agents can follow Spec-first development without following the rest of a mature software lifecycle.

## 2. Goal

Create a minimal but complete repository governance baseline that:

- separates current architecture, durable decisions, task specs, and migrations;
- defines canonical architecture invariants and state/ownership boundaries;
- documents standard testing, exact-source verification, release, rollback, and evidence rules;
- gives contributors standard PR/issue entry points;
- upgrades root `AGENTS.md` so every AI agent follows the full standard project lifecycle, not only Spec-first development;
- avoids inventing legal, ownership, product, or release facts that are not established in the repository.

## 3. Non-goals / out of scope

- No application/runtime behavior changes.
- No new product features.
- No architecture implementation refactor in this task.
- No arbitrary open-source license selection. Licensing is a legal/owner decision.
- No `CODEOWNERS` entry until a valid GitHub user/team ownership mapping is explicitly designated.
- No fabricated roadmap or historical changelog entries.
- No weakening or replacing existing project-specific source-of-truth documents or active specs.

## 4. Requirements

- R1: Add a canonical documentation map for architecture, ADRs, specs, migrations, testing, operations, and governance.
- R2: Add architecture current-state/boundary documentation plus stable architecture invariants.
- R3: Add an ADR policy, template, and initial ADR adopting the architecture/spec/ADR/migration documentation model.
- R4: Add a migration policy and template.
- R5: Add repository-wide contribution, security-reporting, changelog, PR-template, and issue-template baselines.
- R6: Add testing and release/evidence guides grounded in existing exact-HEAD GitHub Actions practices.
- R7: Update `AGENTS.md` to require the lifecycle: Discover → Classify → Spec → Architecture/ADR → Plan → Implement → Verify → Compliance Review → PR/Merge → Release → Post-release verification.
- R8: Require architecture-affecting changes to update both the applicable spec and durable architecture/ADR records before implementation.
- R9: Require migration plans for incompatible state/schema/process/protocol/ownership changes.
- R10: Require completion claims to be evidence-backed and distinguish source-complete, CI-complete, merged, released, and post-release-verified states.
- R11: Update `README.md` with links to the governance and architecture entry points.

## 5. Current state

Verified at canonical main `4277fe1009327a23e8a171281d6172213a2b5142`:

- root `AGENTS.md` exists and enforces Spec-first / No Spec, No Code;
- `docs/specs/` exists with a template and active specs;
- project-specific governance exists under `projects/`;
- exact-source Actions workflows exist under `.github/workflows/`;
- `docs/architecture/`, `docs/adr/`, `docs/migrations/`, `docs/testing/`, and `docs/operations/` do not exist;
- root `CONTRIBUTING.md`, `SECURITY.md`, and `CHANGELOG.md` do not exist;
- no PR or issue templates are present outside `.github/workflows/`.

## 6. Target state

The repository has one discoverable governance system:

- `README.md` points to the canonical documentation entry points.
- `AGENTS.md` defines mandatory AI execution policy.
- `docs/architecture/` describes current/canonical architectural boundaries and invariants.
- `docs/adr/` preserves durable architecture decisions without rewriting history.
- `docs/specs/` remains the per-change executable specification layer.
- `docs/migrations/` records transitions from old state to new state.
- `docs/testing/` defines verification layers and evidence rules.
- `docs/operations/` defines release/rollback/evidence closure.
- GitHub templates require traceability from requirement to evidence.

## 7. Architecture and ownership boundaries

This governance task does not change runtime ownership. It documents the current canonical design direction already established by active specs:

- React/TypeScript renderer owns UI projection and interaction, not authoritative Agent runtime state.
- Electron main owns desktop shell/OS integration, not Agent lifecycle truth.
- preload remains a thin typed bridge.
- Mahayana Coordinator is the authoritative Agent/runtime coordination boundary.
- Host/Runner remain independent runtime boundaries and must not be collapsed into renderer or Electron main.
- privileged local execution, computer control, connectors/MCP, and OAuth remain behind explicit capability/security boundaries.

If implementation and architecture documentation disagree, an active migration spec must make that transitional state explicit.

## 8. Interfaces / contracts / schemas / data flow

No runtime interfaces change. New repository-governance contracts are Markdown conventions and GitHub templates.

## 9. Constraints and non-functional requirements

- Keep documentation concise enough to stay maintainable.
- Do not duplicate project-specific specs verbatim.
- Prefer links and ownership rules over stale copied details.
- Do not invent test commands or release facts; reference existing scripts/workflows.
- Do not weaken existing exact-source CI or packaged acceptance requirements.
- Documentation must distinguish normative rules from examples.

## 10. Failure modes and edge cases

- Architecture doc becomes aspirational while code differs: require active migration spec and explicit transitional status.
- ADR is edited to hide history: ADR policy forbids rewriting accepted decisions except typo/clarity corrections; supersede with a new ADR.
- AI agent skips architecture review for an architecture-affecting task: root `AGENTS.md` must fail closed.
- Release candidate passes source tests but real package fails: release guide requires packaged acceptance when applicable and distinguishes merged from released.
- Legal/ownership metadata is missing: do not fabricate LICENSE/CODEOWNERS.

## 11. Implementation strategy

1. Create this spec first.
2. Add governance/documentation directories and templates.
3. Add canonical architecture overview/invariants.
4. Add contribution/security/changelog and GitHub templates.
5. Expand `AGENTS.md` while preserving existing Spec-first rules.
6. Update `README.md` with documentation navigation.
7. Read back changed files from the branch.
8. Open a PR against `main` and use CI/status evidence for integration.

## 12. Verification / test strategy

- Structural verification: every required file exists on the branch.
- Content verification: `AGENTS.md` explicitly contains the mandatory lifecycle and fail-closed architecture/ADR/migration rules.
- Traceability verification: PR template links Spec/ADR/architecture/evidence fields.
- Regression safety: no runtime/application files are changed.
- CI: inspect the exact PR HEAD checks; documentation-only changes may not trigger path-filtered runtime workflows, which must not be misrepresented as runtime validation.

## 13. Acceptance criteria / Definition of Done

- AC-1: R1–R11 are present on one branch and reviewed against this spec.
- AC-2: no runtime source file is changed.
- AC-3: root `AGENTS.md` preserves Spec-first policy and adds full standard lifecycle governance.
- AC-4: README exposes the new governance entry points.
- AC-5: branch changes are opened as a PR against `main`.
- AC-6: final report identifies any intentionally uncreated governance files and why.
- AC-7: no completion claim says “merged” or “released” unless GitHub evidence actually proves it.

## 14. Release / migration / rollback

Documentation/governance only. Merge through a normal PR. Rollback is a revert of the governance commit(s). No application release is required solely for this change.

## 15. Observability / evidence

Required evidence:

- exact base SHA;
- exact branch HEAD SHA;
- changed-file list;
- PR URL/number;
- CI/check state if available;
- read-back of root `AGENTS.md` and key governance files from the branch.

## 16. References / provenance

- Root `AGENTS.md` at main.
- `docs/specs/spec-first-ai-development.md`.
- `docs/specs/SPEC_TEMPLATE.md`.
- Active Grok/Fabushi architecture refactor specs.
- Existing `.github/workflows/` exact-HEAD verification patterns.
- User requirement: complete the repository’s missing mature-project structure and require AI agents to follow the standard project process.

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1 | passed | `docs/README.md` establishes the canonical documentation map. |
| R2 | passed | `docs/architecture/system-overview.md` and `docs/architecture/invariants.md` added. |
| R3 | passed | ADR policy/template plus `ADR-0001` added under `docs/adr/`. |
| R4 | passed | migration policy/template added under `docs/migrations/`. |
| R5 | passed | `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md`, PR template and issue forms added. |
| R6 | passed | `docs/testing/strategy.md` and `docs/operations/release-and-evidence.md` added and grounded in exact-source evidence rules. |
| R7 | passed | root `AGENTS.md` now requires the full standard lifecycle, including Migration and post-release verification. |
| R8 | passed | root `AGENTS.md` fail-closes architecture-affecting work on Spec + ADR + architecture-doc updates. |
| R9 | passed | root `AGENTS.md` and migration policy require controlled migration records for incompatible ownership/state/protocol transitions. |
| R10 | passed | `AGENTS.md` defines specified/implemented/source-verified/CI-verified/merged/release-candidate/released/post-release-verified/blocked states. |
| R11 | passed | root `README.md` links product, roadmap and governance entry points. |
| AC-1 | passed | branch compare against `main@4277fe1009327a23e8a171281d6172213a2b5142` shows the complete governance set on one branch. |
| AC-2 | passed | GitHub compare shows no runtime/application source changes; only docs/governance/templates and `.editorconfig`. |
| AC-3 | passed | `AGENTS.md` was read back from the branch with Spec-first preserved and the full lifecycle added. |
| AC-4 | passed | `README.md` exposes product/governance/architecture/testing/release entry points. |
| AC-5 | passed | GitHub PR #22 opened against `main`. |
| AC-6 | passed | LICENSE and CODEOWNERS are intentionally not fabricated; final report must state this. |
| AC-7 | passed | no merged/released claim is made by this Spec; those statuses require later GitHub evidence. |

Allowed statuses: `passed`, `blocked`, `not-applicable`.
