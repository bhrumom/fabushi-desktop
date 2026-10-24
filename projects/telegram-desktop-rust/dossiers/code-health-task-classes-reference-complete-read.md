# Complete read — .agents/skills/code-health/references/classes.md

Status: independently understood and mapped; exact-head verification pending  
Project: TDRP-001 Revision 9  
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`  
Source: `.agents/skills/code-health/references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e`

## Inventory identity

The write lease for this closure is Fabushi `main@af61df71b5cd7a32c5fe057f1473df7f826b9165`, upstream `dev@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, tree `5151db79560ed40bf3802482fcfb74b950be6c40`. The immediately preceding exact-head source-authority campaign produced fresh authority artifact `12229539580` with digest `sha256:4b5f7206e9c86c41d225c82fdc4610ee5ecbfd47e2fb3261797cfca406638bae`; its recursive authority has 16,202 non-directory entries and identifies actual 1-based recursive array entry 21 as this exact path/blob. Before this write, the merged parity ledger had zero rows for this exact path/blob and coverage was unknown=16,182, unread=16,155, omitted=0.

This closure accounts only this independent entry. Files read to understand its callers and proof machinery remain separately inventoried unless they already have their own row.

## Complete source responsibility

`classes.md` defines a risk/proof taxonomy for behavior-preserving maintainability work. The class is not a label for scanner output; it decides the proof, routing lane, prerequisites, placement constraints and trust boundary of the resulting ordinary task.

The common rules are load-bearing:

- one concern per revertible commit or short series; feature work and defect fixes do not hide inside cleanup;
- observable behavior remains unchanged, including UI, network traffic, persisted formats and logs consumed by tests;
- maintenance must not silently add build work or binary weight; narrow mechanical work cannot introduce new semantic dispatch/templates/header-defined logic;
- source should shrink except where tests are the product of the task; a new abstraction must remove real duplication or enforce an approved invariant;
- placement follows the nearest approved area map and its leave-alone decisions;
- every task states the metric it moves and the before/after evidence;
- defects discovered while reading are ordinary defects and leave the code-health task.

The six classes bind increasingly strong proof:

### Q0 — delete

Delete only dead/unreferenced code, stale compatibility/format paths and source/resources used solely by that dead code. Proof combines the candidate identity with complete reference negatives, affected build/link coverage and the change-time health ratchet. Platform-specific source may be deleted only when its platform build is exercised or every source/generated input proves the symbol unreferenced. A textual scan exists specifically to catch platform paths or unused-function compiler settings that a local Debug build may miss.

### Q1 — move

Move/split source or reduce header reach without changing logic. The move proof must distinguish relocation scaffolding from substantive new/lost/reordered statements, then run the required build and health ratchet. Header work must not increase build-work cost. Scheduling is part of correctness: do not move files currently named by an in-progress task or unreconciled recent authority changes, and record the new placement in the area map so later work follows the move.

### Q2 — merge duplicates

Collapse copies only when they are equivalent apart from names and literal values. Shared code must live where all consumers already link; each pre-existing difference becomes an explicit input rather than being erased. Proof is the affected build, health ratchet and finding verification showing the clone family resolved. Copies whose logic differs are not Q2.

### Q3 — pin behavior with tests

Add headless characterization tests for pure behavior such as parsers, serializers, converters, model operations, state machines or merge rules. The suite must pass unchanged code and prove sensitivity through at least five small deliberate mutations, each observed failing and reverted. Enum-dispatched code prefers exhaustive kind coverage. This class enters the full pipeline.

### Q4 — reshape

Perform semantic, behavior-preserving refactors such as extracting collaborators, replacing string errors with types, tables for switches, splitting god objects behind the same façade or reconciling divergent copies. Characterization coverage must exist before the task and pass before and after; runtime checks are added where behavior lives only in the running client. The first Q4 task in an area requires an approved area map. This class uses the full pipeline.

### Q5 — redesign

Change seams, layering, ownership or interfaces only after the intended end state is expressed as owner-approved, checkable invariants in the area map. At least two independent designs are compared by a separate reviewer, preferring deletion, fewer concepts and an existing repository mechanism when it fits. Execution is staged through independently proved move/reshape tasks and finishes by turning every approved invariant into an enforced check or test.

## Trust and routing responsibility

The source reference also owns the routing trust contract. Q0–Q2 may use the narrow mechanical lane only while their semantic bounds remain true. Q3/Q4 require the owner routing decision, Q4 additionally requires an approved map, and Q5 requires approved invariants. A failed audit or a later regression attributed to a landed code-health task suspends that class in that area until the owner explicitly re-enables it. Suspension preserves the underlying finding rather than pretending it is resolved.

## Implementation and caller relationships read

The following exact upstream relationships were read to distinguish this reference's semantic responsibility from the implementation instruments around it:

- `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` makes code-health measurement/proposal-only, sends all improvements through the ordinary task queue, chooses the mechanical lane for Q0–Q2 and the full pipeline for Q3–Q5, applies proposal/trust limits, and audits landed work.
- `.agents/skills/code-health/scripts/report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` maps scanner finding kinds to the classes: unused→Q0; large/heavy file/header/unit→Q1; clone→Q2; untested hot area→Q3; large class/long function→Q4; layering→Q5. That mapping proposes risk classes; it does not prove a task fits one.
- `.agents/skills/code-health/scripts/health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` provides the source-specific instruments used by the contract: exact-source scan state plus change `check`, move-only `moves` and finding `verify`.
- `.agents/skills/code-health/references/atlas.md@7f43dd75fcfb670c327bc3e71c1de0589e7506f0` supplies stable area identity; `references/maps.md@89dd78f48283b15d6ad959b2d450895258a0980f` supplies Modules/Placement/Invariants/Leave alone/Direction and the owner-approval precondition for redesign; `references/survey.md@d900e7a42b768679c69cd4c584559c783b59d47a` produces a dependency-ordered Q0–Q5 plan with tests before reshapes and sends defects to ordinary work.
- `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` recognizes the code-health class in the ordinary task and dispatches Q0–Q2 to its bounded mechanical exception.
- `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` makes Q0–Q2 a narrow proof/review lane. It fails closed: when an item needs semantic edits, expanded scope or stronger proof, the item is dropped/reclassified rather than being forced through the mechanical lane; discovered defects become separate ordinary tasks.
- `.agents/skills/perform-task/references/pipeline.md` and `phase-prompts.md` retain the full task implementation/review/evidence pipeline for semantic work and define the surrounding task ownership/evidence mechanics.
- the scheduled `code-health scan route` command and host registration are invocation adapters only: they can cause eligible proposals to enter the ordinary task lifecycle but do not own class semantics or create a second task queue.
- `process-inbox` and `continue` callers consume the preserved task metadata, dependency and ownership state; they do not reinterpret a health finding as completed evidence.

These caller reads show that the source entry's applicable responsibility is the class/proof/trust/placement contract, not Telegram's Python scripts, host prompts or ai-tdesktop queue implementation.

## Existing-owner-first resolution

Current Fabushi already has one source-neutral owner for the applicable responsibility:

1. root `AGENTS.md` owns repository development, exact-authority verification, quality-health ratchets, responsibility maps and ordinary task/review lifecycle;
2. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` owns proportional verification and already distinguishes mechanical proof, characterization, semantic reshape and redesign;
3. `projects/fabushi-communication-platform/quality/evidence-contract.md` owns exact-base, finding, proof, mutation, placement, routing and audit evidence;
4. the ordinary Fabushi task/review/evidence lifecycle owns implementation, conflict management, independent review and acceptance.

The missing part was explicit class qualification: semantic-risk boundaries, per-class causal proof, placement/precondition rules, recent-authority/owned-path scheduling, drop/reclassification and trust suspension. This closure extends only those existing owners.

Rejected alternatives:

- shipping Agent/Coordinator/Host/Runner are product execution owners, not repository maintainability-task classification;
- product stores and release packaging consume outcomes but do not own cleanup proof;
- copying Telegram `health.py`, `report.py`, perform-task mechanical-lane code or host prompts would reproduce a source-specific workflow runtime rather than preserve the source-neutral responsibility;
- a second quality-task queue or second repository ownership graph would conflict with the canonical ordinary Fabushi task/review lifecycle.

No new shipping capability, owner or ADR is required.

## Fabushi-equivalent contract

`fresh exact finding/source -> classify the actual semantic change -> validate area/map/invariant/test prerequisites and active ownership -> ordinary task receives class+area+finding+metric+proof -> mechanical delete/move/equivalent-merge gets narrow causal proof and fresh review OR characterization/reshape/redesign enters full pipeline -> fail/drop/reclassify if class bounds do not hold -> approve only with before/after evidence -> later audit/regression can suspend class+area routing -> explicit re-enable only`

The mapping deliberately preserves the meaning without preserving Telegram's Q-class runtime implementation. The class names remain provenance in this dossier/ledger; Fabushi's canonical owner is semantic risk plus ordinary task evidence.

## Fail-closed branches

- no scanner classification can downgrade a change whose diff is semantically stronger;
- a mechanical item that needs logic edits, new behavior or wider scope is dropped/reclassified;
- deletion cannot pass with unresolved platform/generated callers or orphaned last users;
- a move cannot pass with substantive new/lost/reordered logic or disallowed placement;
- divergent duplicates cannot pass as equivalent-copy consolidation;
- characterization cannot pass without observed mutation failures;
- reshape cannot begin without existing behavior-pinning coverage;
- redesign cannot begin without owner-approved checkable invariants and architecture choice evidence;
- active path ownership, unreconciled fresh-authority drift or a map leave-alone rule blocks placement-sensitive cleanup;
- failed audit or later regression suspends routing for the affected class/area without deleting the finding;
- class metadata cannot weaken product/security/release gates or become a second task/runtime owner.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-TASK-CLASSES-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-CLASSIFICATION-001`, `ORA-TDRP-CODE-HEALTH-PROPORTIONAL-PROOF-001`, `ORA-TDRP-CODE-HEALTH-ROUTING-TRUST-001`
- Invariants: `INV-TDRP-CODE-HEALTH-NO-SEMANTIC-MECHANICAL-001`, `INV-TDRP-CODE-HEALTH-PLACEMENT-001`, `INV-TDRP-CODE-HEALTH-REDESIGN-APPROVAL-001`, `INV-TDRP-CODE-HEALTH-NO-SECOND-OWNER-001`
- Contract: `CONTRACT-TDRP-CODE-HEALTH-TASK-CLASSES-001`
- Integration: `INT-TDRP-CODE-HEALTH-CLASS-ROUTING-001`
- Fault controls: `FAULT-TDRP-CODE-HEALTH-MISCLASSIFICATION-001`, `FAULT-TDRP-CODE-HEALTH-ACTIVE-PATH-CONFLICT-001`, `FAULT-TDRP-CODE-HEALTH-SUSPENDED-ROUTING-001`
- Security control: `SEC-TDRP-CODE-HEALTH-PROOF-DOWNGRADE-001`

## Verification disposition

This source entry is repository quality/task governance, not shipping runtime. The closure changes only the existing source-neutral governance/evidence owners plus this independent dossier/ledger/inventory/lock row. Exact-head verification therefore remains the project-wide required GitHub Actions pair on one final PR HEAD:

- TDRP Revision 9 source authority must keep accepted mode fail-closed while `upstream.accepted=false` / source closure is open, emit fresh authority bound to this PR HEAD with recursive=16,202, unknown=16,181, unread=16,154, omitted=0, and retain the complete recursive authority chain: 12 root shards, 24 child shards, 16 next shards, 202→203 fixed point with zero unresolved gitlinks, reachability/root-candidate disposition, root archives, and Boost 582/582/open=0.
- Rust desktop runtime must remain terminal-success on the same PR HEAD and preserve renderer → Electron → Coordinator → Host → Runner composition, platform-worker checks, rust-host tests, and the `native/mahayana-messaging/Cargo.toml` manifest evidence.

Only after both campaigns are terminal-success on the same immutable PR head, the PR/main/upstream/entry21 lease is reread and unchanged, and GitHub reports the exact head mergeable may this one entry be merged. Full TDRP source closure remains open after the merge at unknown=16,181, unread=16,154, omitted=0.
