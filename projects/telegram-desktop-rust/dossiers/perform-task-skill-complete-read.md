# Perform-task skill complete read — strict recursive indices 45-46

Status: parent exact-head verified at b34f18004a58db1c1341505fc4d0bf3ccb9b1ce6; strict index 46 source-grounded and existing-owner mapped. First child f83be332053e55181913c4a134b9142a806c2f63 exposed a deterministic dossier-only validator regression because the index 46 dossier rewrite omitted the retained index 45 traceability literals; this revision restores those literals without changing coverage or ownership. Fresh child exact-head verification pending.

Authority: telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844 tree 1fc479a4b448aed851181bf4a4796d3015368348.
Index 45 source: `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`.
Index 46 source: `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a` (4 lines).
Next strict entry: index 47 `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259`.

## Parent exact-head evidence

TDRP Revision 9 run 39520394783 is terminal success on b34f18004a58db1c1341505fc4d0bf3ccb9b1ce6. Source authority artifact 12310588122 digest sha256:55b710b2f6cbfbe5ca63634d9fe8ba13ab8d44915754ae05395c1d38bd2b68bd binds exact target b34f18004a58db1c1341505fc4d0bf3ccb9b1ce6, upstream 0eca52223cceed229932add1cc80792763120844 and tree 1fc479a4b448aed851181bf4a4796d3015368348. Boost artifact 12310877766 digest sha256:dfa75b7258ba4312d757fa9133b7f21677aba2358271e9d93febf74279a03638 remains 582/582/open=0. Root aggregate 12309843951 digest sha256:a4dcb74e68aa544f25f72a690a1dd11694719a22aede6c27e41452fa3d3b1b54 covers shards 0-11; child aggregate 12310528420 digest sha256:688b6882a552ae23129da57c19aa2c62dad55990b41723ea20c903c829784d5c covers 0-23; next aggregate 12310004502 digest sha256:157ec5d6649751b022c3d3f44aac4720fd356f4cb13f150458e3fc16c0b6c67e covers 0-15. Fixed-point artifact 12310818800 digest sha256:713214558c41b8f39ed1ac7a8923414314c82f914de19832543192ba0b730401 reaches final_scanned_authorities=203 and unresolved_gitlink_authorities=0. Reachability artifact 12309949569 digest sha256:1897b118f890815eaa1f10dffb72f3b4c977899e18047a22db9543e9c20b1a10 remains partial-audited-open with remaining_open_totals authorities=95, gitlinks=133, acquisition_candidates=3865, generated_inputs=4674, resource_inputs=4602, tool_inputs=4255; baseline accepted=false and source_closure_ready=false.

Rust desktop runtime run 39520394839 is terminal success on b34f18004a58db1c1341505fc4d0bf3ccb9b1ce6: Windows/macOS/Ubuntu pressure profiler, renderer, platform-worker and rust-host are 6/6 success. Artifact 12310398263 `rust-messaging-responsibilities-b34f18004a58db1c1341505fc4d0bf3ccb9b1ce6` digest sha256:412b57aa056eff120f32fd1ccb3c33223a396ac82e23b6adaf31b84fe9b6b45f binds exact b34f18004a58db1c1341505fc4d0bf3ccb9b1ce6.

## Strict index 46 complete source read

The entire YAML is interface metadata only:

- `interface.display_name = "Perform AI Task"`
- `interface.short_description = "Implement or resume one Telegram AI task"`
- `interface.default_prompt = "Use $perform-task to implement, adaptively review and test, and publish one named ai-tdesktop task."`

There is no executable task state machine, persistence, queue, scheduler, network client, product UI, token, runtime adapter or product Task/Agent owner in this file.

## Real discovery and invocation path

The exact upstream source establishes a two-layer boundary:

1. `.agents/skills/perform-task/SKILL.md@590e...` declares frontmatter `name: perform-task` and owns the repository-development performer semantics.
2. The sibling `agents/openai.yaml@f238...` supplies OpenAI-host interface metadata; its default prompt invokes `$perform-task` and therefore delegates execution to the shared skill rather than implementing it.
3. The same convention is present in sibling skills: continue `agents/openai.yaml@e981...`, process-inbox `@b664...`, code-health `@59bb...`, and dependency-watch `@a61c...`; each contains display/description/default-prompt metadata that names its existing skill.
4. Provider command wrappers are separate adapters, not semantic owners: `.grok/commands/perform-task.md@313e...` and `.claude/commands/perform-task.md@98ec...` explicitly read the shared perform-task SKILL.md and apply their host adapters.
5. The continue scheduler `.agents/skills/continue/SKILL.md@aeb81...` delegates selected one-task execution to perform-task; that scheduler/performer relationship is owned by the shared skill contracts, not by this OpenAI metadata file.

Therefore the real responsibility of index46 is discoverability/invocation presentation for the existing repository-development skill. It cannot establish a second Task, Agent, scheduler, queue, Host, Runner or product runtime owner.

## Current Fabushi existing owner

Fabushi exact b34f18004a58db1c1341505fc4d0bf3ccb9b1ce6 already has the applicable source-neutral owner:

- `AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b` owns the ordinary repository task/review/evidence lifecycle, source-neutral adaptive verification and repository quality command boundary.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb` owns repository quality-task qualification, AI review execution policy and adaptive evidence recovery.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` owns independent/durable review and task-local handoff evidence.

Exact-head GitHub Contents checks show Fabushi has no copied `.agents/skills/perform-task/SKILL.md` or `.agents/skills/perform-task/agents/openai.yaml` at b34f18004a58db1c1341505fc4d0bf3ccb9b1ce6. That absence is intentional for this mapping: a provider interface file from the source authority is not a novel shipping capability and does not justify duplicating the upstream repository AI workflow.

Shipping Fabushi Task/Agent/Host/Runner owners remain product runtime owners only and are explicitly rejected for this responsibility. No new owner or ADR is required; `novel_capability=false`.

## Fabushi-equivalent contract

`host skill interface metadata -> invoke existing named repository-development performer -> existing repository task/review/evidence lifecycle`

Invariants:

- interface metadata never becomes semantic task state;
- `$perform-task` resolves to the existing performer contract rather than a second runtime;
- host/provider adapters do not become product ownership;
- shipping Fabushi Task/Agent/Host/Runner state remains separate;
- exact-head GitHub Actions evidence is required again after this governance-only mapping commit;
- open recursive coverage keeps baseline_identity_accepted=false and source_closure_ready=false.

## Index 45 retained traceability

The index 46 dossier extension retains the traceability identifiers of the already-mapped index 45 shared perform-task semantic engine so the combined dossier remains a valid oracle for both ledger rows.

- Requirement: TDRP-R9-PERFORM-TASK-WORKFLOW-001
- Oracles: ORA-TDRP-PERFORM-TASK-SINGLE-OWNERSHIP-001, ORA-TDRP-PERFORM-TASK-REVIEW-EVIDENCE-001, ORA-TDRP-PERFORM-TASK-PUBLICATION-001
- Invariants: INV-TDRP-PERFORM-TASK-ONE-TASK-ONLY-001, INV-TDRP-PERFORM-TASK-FAIL-CLOSED-EVIDENCE-001, INV-TDRP-PERFORM-TASK-RESUMABLE-STATE-001, INV-TDRP-PERFORM-TASK-NO-SHIPPING-TASK-COLLISION-001
- Security: SEC-TDRP-PERFORM-TASK-EVIDENCE-DOWNGRADE-001
- Fault recovery: FAULT-TDRP-PERFORM-TASK-SOURCE-DRIFT-001, FAULT-TDRP-PERFORM-TASK-TEST-FLAW-001, FAULT-TDRP-PERFORM-TASK-PUBLICATION-CONFLICT-001

## Traceability

- Requirement: TDRP-R9-PERFORM-TASK-OPENAI-INTERFACE-001
- Oracle: ORA-TDRP-PERFORM-TASK-OPENAI-INTERFACE-001
- Invariants: INV-TDRP-PERFORM-TASK-OPENAI-METADATA-ONLY-001, INV-TDRP-PERFORM-TASK-NO-SHIPPING-TASK-COLLISION-001
- Contract: CONTRACT-TDRP-PERFORM-TASK-OPENAI-INTERFACE-001
- Integration: INT-TDRP-PERFORM-TASK-OPENAI-TO-SHARED-SKILL-001
- Security: SEC-TDRP-PERFORM-TASK-HOST-METADATA-NO-AUTHORITY-001

## Verification disposition

This closes only recursive strict index 46. Counters become source_entries_total=16,305, unknown=16,256, unread=16,231, omitted=0, responsibilities_total=65, unmapped=0. baseline accepted=false and source_closure_ready=false because source coverage and recursive reachability/disposition remain open.

Parent b34f18004a58db1c1341505fc4d0bf3ccb9b1ce6 evidence is supporting evidence only after this write. Exact-head acceptance of index 46 requires fresh TDRP Revision 9 and Rust desktop runtime workflows on the new immutable child commit. Only after those workflows close may audit enter strict index 47.
