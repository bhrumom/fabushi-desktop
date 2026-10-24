# Project context complete read

Status: mapped / exact-upstream read complete / exact-head verification pending  
Project: TDRP-001 Revision 9  
Fabushi write lease: bhrumom/fabushi-desktop@2b394d7551f3dbbe1320bba7f771a4694134e382  
Upstream: telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a  
Tree: 5151db79560ed40bf3802482fcfb74b950be6c40  
Path: .agents/shared/project-context.md  
Blob: 53c8657eb7949a5a2fba7e7adc8d1073f3079ead  
Primary workflow callers: .agents/skills/process-inbox/SKILL.md@0ce70794ac743d99e6d5c80894cde970b52842db; .agents/skills/perform-task/references/pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7; .agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba; .agents/skills/continue/references/consolidate-pending-tasks.md@bd301d93f8f670ad041b3db72330e17cbb50eb5c  
Repository/host instruction anchors: AGENTS.md@d8d59480277b7defbca1cdbccf7ed164ab2db086; CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155; GROK.md@b1ae452d94cf48971c6d7ea17cac6b45cabee08d; REVIEW.md@e65be8231a8e0f50ee86b20ef16120d58ea40376

## Live authority reconciliation

Immediately before writing, Fabushi main was re-read at `2b394d7551f3dbbe1320bba7f771a4694134e382` (tree `96c80d118016cbe962018fbadcd83b41b04c5158`); upstream `dev` was re-read at `351204f3695b4a39bc3f4d2f230c061b9a01fe2a` (tree `5151db79560ed40bf3802482fcfb74b950be6c40`), and the target entry still resolves to blob `53c8657eb7949a5a2fba7e7adc8d1073f3079ead`. This is the thirteenth ordered recursive source entry in the live Revision 9 rebaseline.

Closing only this independently owned source entry changes recursive counters from unknown=16,190/unread=16,163 to unknown=16,189/unread=16,162; omitted remains 0. Baseline readiness, full source closure and final release acceptance remain false.

## Independent responsibility

This file is repository workflow/context governance, not Telegram product runtime. It independently defines how project-local durable context is discovered, layered, bounded, handed to the next phase, and optionally amended without allowing accumulated history to become a second source of truth.

The applicable Fabushi responsibility is source-neutral repository workflow/context/evidence governance. Existing root `AGENTS.md` already owns repository lifecycle and authority ordering, while the Test Evidence Contract owns durable provenance. No Telegram owner, communication runtime, scheduler, or product state owner is justified.

## Context layers and precedence

The contract separates roles instead of treating every document as equal authority:

- the current explicit request defines the requested outcome;
- current exact source establishes what is actually implemented;
- repository and nested `AGENTS.md` define applicable engineering instructions for their scopes;
- `CLAUDE.md` and `GROK.md` are host pointers/adapters and defer to canonical repository instructions rather than becoming competing policy roots;
- task specification and referenced inputs define the concrete task boundary;
- a project's `project.md`, relevant docs/specs/README/reference material and prior decisions are selective supporting context, not an instruction to recursively load their targets;
- task-local `work/context.md` is the compact present-tense handoff for downstream phases;
- `tasks.md` is navigation while the live task status owner remains `state.yaml` in the upstream workflow model;
- historical context, plans, proposals, snapshots and receipts remain discoverable but do not override the current request or source.

When authorities conflict, the workflow revalidates the older/supporting material against the current request/source instead of inheriting it. A load-bearing ambiguity or correctness conflict fails closed rather than being resolved by "latest approved task" heuristics or accumulated amendments.

## Selective discovery and bounded reading

Discovery starts from explicit lineage, task metadata, the current request and relevant source. Project names/titles and targeted search identify candidate small overviews. A dependency, input, reference or earlier decision is followed only when a concrete requirement or ambiguity makes it relevant.

Large legacy documents are handled by headings/search plus bounded relevant sections. The read set of an unchanged request must not grow merely because unrelated tasks were completed. Links are locators, not recursive-loading commands. This keeps context cost proportional to the current task while retaining exact source locators for downstream reconstruction.

The process-inbox caller applies this before routing and project creation, reads the repository instruction roots plus inbox inputs, searches live/archived projects selectively, and keeps routing/decision prose in task artifacts or receipts rather than bloating indexes. The consolidation caller independently preserves its mandatory unfinished-state inventory while applying the same selective project read policy.

## Worker handoff contract

The perform-task pipeline and Phase 1 prompt require the context/planning phase to read repository instructions, the task specification, referenced inputs and only relevant project context. The leaf writes `work/context.md` with enough present request, exact files/sections, implementation facts, patterns and remaining assumptions for another phase to proceed without conversation memory.

A compact worker reply is only a notification; durable artifacts and repository state are proof. On resume, validated completed boundaries are reused. An old full project proposal is not a completion requirement or replacement to promote. This keeps handoff state task-local and auditable instead of depending on chat/session continuity.

## Durable amendment lifecycle

A shared project amendment is optional and narrow. It is created only when a task changes a useful durable shared fact, names the exact target section, gives the proposed change/rationale and supporting source/task reference, and leaves implementation detail in the task.

The proposal remains task-local until implementation, review and evidence approve the outcome. Immediately before publication the workflow rereads the latest canonical target section, reconciles concurrent edits, applies only the still-valid narrow change, and preserves unrelated text. Redundant or obsolete amendments are skipped. Legacy whole-document proposals are historical only and must not be promoted wholesale or chained forward.

If only the optional amendment cannot be reconciled, the task may finish while leaving it task-local. If the conflict invalidates task correctness or actual publication fails, the boundary fails closed. Blocked or split-required work never publishes proposed shared facts.

## Caller-chain verification

- `.agents/skills/process-inbox/SKILL.md` explicitly invokes the shared project-context policy for project discovery, selective expansion and compact indexes before routing.
- `.agents/skills/perform-task/references/pipeline.md` invokes it in preflight, resume and final project-amendment publication.
- `.agents/skills/perform-task/references/phase-prompts.md` invokes it for the Phase 1 context/plan worker and requires exact task context instead of accumulated history.
- `.agents/skills/continue/references/consolidate-pending-tasks.md` invokes it while preserving an independent fail-closed unfinished-state inventory and pre-write race recheck.
- upstream `CLAUDE.md` points to `AGENTS.md` as canonical repository-wide instructions; `GROK.md` does the same and adds only workflow-adapter reading for supported commands. Neither creates a parallel instruction truth.

No shipping Telegram product caller is implied by this source entry.

## Existing-owner-first disposition

Fabushi already has the correct source-neutral owners:

- root `AGENTS.md` owns repository authority discovery, spec/source precedence, development lifecycle and fail-closed behavior;
- `projects/fabushi-communication-platform/quality/evidence-contract.md` owns durable evidence provenance and reconstruction requirements.

The mapped change extends those owners with selective context discovery, instruction/document precedence, compact handoff, revalidation of historical context, and narrow amendment publication. No Telegram-specific owner or second repository workflow/context system is introduced.

## Traceability anchors

Requirement: `TDRP-R9-PROJECT-CONTEXT-GOVERNANCE-001`  
Oracle: `ORA-TDRP-PROJECT-CONTEXT-GOVERNANCE-001`  
Invariants: `INV-TDRP-CONTEXT-CURRENT-AUTHORITY-001`, `INV-TDRP-CONTEXT-SELECTIVE-READ-001`, `INV-TDRP-CONTEXT-HANDOFF-RECONSTRUCTABLE-001`, `INV-TDRP-CONTEXT-AMENDMENT-RECONCILED-001`

These anchors bind the ledger row to this exact source-read dossier. Exact-head execution evidence remains pending until the Actions runs for the final PR head succeed.

## Verification boundary

This closure is documentation/governance mapping only. It does not claim that the whole upstream inventory is closed or that Fabushi product/release acceptance is complete. The exact mapping commit must pass the existing TDRP Revision 9 source-authority workflow and Rust desktop runtime workflow on the same PR head before the row's exact-head blocker can be cleared. Even after those runs pass, unknown=16,189 and unread=16,162 keep baseline/source closure fail-closed.
