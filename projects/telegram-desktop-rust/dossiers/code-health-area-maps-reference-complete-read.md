# Complete read — .agents/skills/code-health/references/maps.md

Status: independently understood and mapped; exact-head verification pending  
Project: TDRP-001 Revision 9  
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`  
Source: `.agents/skills/code-health/references/maps.md@89dd78f48283b15d6ad959b2d450895258a0980f`

## Inventory identity

The write lease for this closure is Fabushi `main@1ac669188f453f2765728aaee83a20c64866ff87`, upstream `dev@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, tree `5151db79560ed40bf3802482fcfb74b950be6c40`. The immediately preceding exact-head source-authority campaign produced fresh authority artifact `12231188389` with digest `sha256:4d23c9166f690be40254810e757ee5fb981dd8772a68d84d1f958dc618ef8f3b`, bound to validation head `f7672e0c6f5e81ad8804aa4bd216dae80d00641e`. Its `upstream-recursive-inventory.json` contains 16,202 non-directory entries and identifies actual 1-based recursive array entry 22 as this exact path/object; entry 21 is `references/classes.md` and entry 23 is `references/survey.md`. Before this write, the merged parity ledger has zero rows for this exact path/blob and coverage is unknown=16,181, unread=16,154, omitted=0.

This closure accounts only entry 22. The skill, scripts, survey, classes, perform-task pipeline and routing surfaces read to understand its consumers remain separately inventoried unless already represented by their own independent row.

## Complete source responsibility

`maps.md` defines an area-local source responsibility and placement contract whose purpose is to stop each task or reviewer from re-deriving structure from whatever code happens to surround the change. Reviews judge placement against the approved map instead of treating current drift as precedent.

One map refines one atlas area. Upstream stores the map in the nearest area `AGENTS.md` named by the area's first path; areas that share a directory may share that instruction file, while `lib_*` submodules keep only their atlas entry. The location rule is an upstream instruction-layout detail. The portable responsibility is that the local map refines, but never replaces, the parent atlas area and remains compact enough for an owner to review as plain language rather than by rereading implementation.

### Modules

Modules maps each file or tight file group to the responsibility it owns. A file that needs paragraph-scale explanation is itself evidence that the area's decomposition may be too broad and belongs in Direction as a split candidate. Source file names are explicit so automated review can detect stale references after deletion or movement.

### Placement

Placement answers the recurring task question "I am adding this kind of code; which owner/module receives it and which location is forbidden?" This is the part ordinary feature/refactor tasks consume directly. Placement is prescriptive review authority only after approval; surrounding drift does not override it.

### Invariants

Invariants are limited to checkable rules: dependency direction, which module may touch secrets or durable formats, purity requirements, or comparable mechanically/reviewably falsifiable claims. Each approved invariant preserves approval provenance/date. An unapproved invariant remains a proposal. Q5 redesign work depends on these approved invariants rather than inventing architectural success criteria during implementation.

### Leave alone

Leave alone records choices that may look like cleanup targets but are deliberate and explains why. It is a negative scope boundary: maintainability work does not churn the named choice merely because a local metric or generic style preference suggests a rewrite.

### Direction

Direction describes the target shape for the area. It is empty when the area is stable and does not silently make future structure current ownership. For semantic redesign it provides intended shape together with approved checkable invariants.

## Lifecycle and approval contract

A survey drafts or amends the map from exact scanned source. The declared owner approves it in words or adjusts it before it becomes source instruction. Because the map is itself source governance, a task that truly needs a different structure edits the map narrowly in the same diff so reviewers judge the structural change and its code together.

The update boundary is intentionally asymmetric. Invariants and Direction change only with explicit owner approval. Q0-Q2 mechanical work touches only Modules lines when files move or disappear; it cannot smuggle a semantic placement or architecture decision through the mechanical lane. Placement/Leave-alone changes are likewise semantic review facts and remain under ordinary owner/task review rather than scanner self-publication. An area that keeps generating placement findings is re-surveyed instead of accumulating one-off amendments until the map merely documents drift.

## Implementation and caller relationships read

The following exact upstream relationships were read to distinguish the portable map responsibility from Telegram-specific execution machinery:

- `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` makes code-health measurement/proposal-only, makes a survey draft/amend the map, requires owner approval for drafted maps/invariants, and routes Q0-Q2 to the mechanical lane while stronger classes use the ordinary full pipeline.
- `.agents/skills/code-health/references/atlas.md@7f43dd75fcfb670c327bc3e71c1de0589e7506f0` is the parent whole-tree responsibility authority. Its stable areas and path classification are the identity a local map refines; local maps must remain consistent with that boundary.
- `.agents/skills/code-health/references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e` consumes the nearest approved map for placement, excludes Leave-alone items from cleanup scope, requires an approved map before the first Q4 reshape in an area, and requires owner-approved checkable map invariants for Q5 redesign.
- `.agents/skills/code-health/references/survey.md@d900e7a42b768679c69cd4c584559c783b59d47a` reads the previous survey/current map and exact scanned source, independently verifies claims, then emits `map.md` only as a draft/amendment for approval; surveys change no source and create no queue record themselves.
- `.agents/skills/code-health/scripts/report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` treats stale-map, unmapped and stale-atlas conditions as map-class findings and scans explicit map file references so deleted/moved names can be detected.
- `.agents/skills/code-health/scripts/health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` owns exact-source scan/worktree/atlas instrumentation around the contract; it is not the semantic map owner to copy.
- `.agents/skills/code-health/scripts/atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43` supports whole-tree atlas data, not a replacement for owner-approved local map semantics.
- `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` dispatches bounded Q0-Q2 tasks to the mechanical lane while retaining the ordinary task lifecycle for stronger work.
- `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` reads the nearest map at preflight/review, allows mechanical edits to the affected Modules lines, and requires moved/shared homes to follow the map or else reclassify/drop rather than expand semantics inside the narrow lane.
- `.agents/skills/perform-task/references/pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7` keeps all non-mechanical work in the ordinary adaptive implementation/review/evidence lifecycle; its structure lens owns containment and cross-module placement.
- `.agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba` instructs general/structure reviewers to judge placement against the nearest directory area map rather than surrounding code when a map exists.
- `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a` exposes scan/survey/check and permits proposal routing only when arguments say `route`; the scheduled mode is `scan route`. That schedule does not self-approve map semantics.
- `.agents/skills/code-health/agents/openai.yaml@59bb6304445f21e2a3ee1a97a69a8e231f934d30` is host registration/provenance only and delegates to the same shared code-health workflow.
- root `AGENTS.md@d8d59480277b7defbca1cdbccf7ed164ab2db086` tells ordinary source changes to read the atlas and nearest area map and amend the map in the same task when a different shape is genuinely required.
- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5` owns ordinary scheduler/task routing boundaries; map approval does not create a parallel queue.

## Existing-owner-first Fabushi audit

Current Fabushi already has the applicable source-neutral owner:

- root `AGENTS.md` owns repository-wide exact-authority development, quality-health, responsibility-atlas, quality-task classification, ordinary task/review/evidence, and fail-closed verification rules;
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` already qualifies atlas lifecycle, placement-sensitive task classes and the ordinary full verification pipeline;
- `projects/fabushi-communication-platform/quality/evidence-contract.md` already owns exact-source atlas/map evidence, owner approval, task evidence and release-evidence boundaries;
- current shipping ProductShell, Conversation, Agent/Coordinator/Host/Runner, product stores, Marketplace/MCP/Computer/Automations, packaging and updater owners do not and should not own repository-source placement governance.

No second map store, ownership graph, task queue, code-health runtime or Telegram-specific executor is required. The narrow change is to make the existing governance owner explicitly preserve the five map sections, lifecycle/update boundaries, approval provenance, stale-reference checks and re-survey rule.

## Fabushi disposition

Disposition: **workflow-equivalent through existing owner**.

The applicable responsibility is preserved by:

- `AGENTS.md -> Repository responsibility-area-map governance`;
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md -> Repository responsibility-area-map qualification`;
- `projects/fabushi-communication-platform/quality/evidence-contract.md -> Repository responsibility-area-map evidence`.

Telegram-specific paths, Python scanner implementation, Claude/Codex command spelling, health-state worktrees and perform-task lane implementation are provenance/instrumentation only. They are not copied into Fabushi shipping/runtime architecture.

## Verification and remaining scope

This entry is complete only when the mapping commit passes exact-head TDRP Revision 9 source-authority validation and Rust desktop runtime verification on GitHub Actions, including the fail-closed Accepted gate and the existing recursive/source/runtime proof set. Until then its repository row remains implementation-complete but exact-head-evidence-pending.

Closing this one independent entry changes only the source-completeness counters to unknown=16,180, unread=16,153, omitted=0. Overall TDRP/FBCP migration remains open and source_closure_ready remains false.
