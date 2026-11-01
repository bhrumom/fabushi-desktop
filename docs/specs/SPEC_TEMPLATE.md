# <Task / Feature Name> — Specification

Status: draft | active | superseded | completed  
Owner: <team/person/agent role>  
Last updated: YYYY-MM-DD  
Related project: <projects/... or N/A>  
Related task / issue / PR: <IDs or N/A>

## 1. Context / problem

Describe the user/business/engineering problem and the current observable failure or limitation.

## 2. Goal

State the exact outcome this work must achieve.

## 3. Non-goals / out of scope

State what this work intentionally does not change.

## 4. Requirements

Use stable IDs for non-trivial work.

- R1:
- R2:
- R3:

Include both functional requirements and externally observable behavior.

## 5. Current state

Document the verified current implementation, architecture, behavior, and known constraints.

## 6. Target state

Describe the desired end state after this specification is implemented.

## 7. Architecture and ownership boundaries

Define:
- component/module ownership;
- state ownership;
- runtime/process boundaries;
- allowed dependencies;
- prohibited dependencies;
- source-of-truth boundaries.

## 8. Interfaces / contracts / schemas / data flow

Document affected APIs, IPC, events, storage schemas, wire formats, state transitions, or control flow.

Use N/A with a reason when not applicable.

## 9. Constraints and non-functional requirements

Consider only what is relevant:
- performance / latency;
- power / CPU / memory;
- security / privacy;
- compatibility;
- reliability;
- accessibility;
- offline behavior;
- migration compatibility;
- maintainability / architecture constraints.

## 10. Failure modes and edge cases

List expected errors, retries, interruptions, stale state, race conditions, partial failures, backward-compatibility cases, and other relevant edge behavior.

## 11. Implementation strategy

Describe the intended implementation approach, important sequencing, migrations, and architectural decisions.

Do not reduce this section to a list of filenames; explain the behavior and ownership changes.

## 12. Verification / test strategy

For each requirement, define how it will be verified.

Examples:
- static / architecture checks;
- unit / contract tests;
- integration tests;
- E2E / packaged-app checks;
- performance / power measurements;
- migration / rollback checks;
- exact GitHub Actions workflow/run evidence.

Do not invent a test gate that conflicts with a newer explicit user instruction.

## 13. Acceptance criteria / Definition of Done

Use stable IDs when useful.

- AC-1:
- AC-2:
- AC-3:

Each acceptance criterion must be objective and evidence-backed.

## 14. Release / migration / rollback

Document rollout, migration, versioning, rollback triggers, and rollback steps when applicable.

Use N/A with a reason when not applicable.

## 15. Observability / evidence

Define the logs, metrics, traces, screenshots, videos, CI results, or other evidence needed to verify this work.

Use N/A with a reason when not applicable.

## 16. References / provenance

Record:
- latest explicit user requirement;
- owning project source of truth;
- relevant architecture docs / ADRs;
- reference repositories or upstream docs;
- issues / PRs / commits / workflow runs used as evidence.

## 17. Spec compliance record

Complete this before declaring the implementation finished.

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1 | pending | |
| AC-1 | pending | |

Allowed statuses: `passed`, `blocked`, `not-applicable`.

If implementation intentionally diverges from this spec, update the spec or decision record before marking the work complete.
