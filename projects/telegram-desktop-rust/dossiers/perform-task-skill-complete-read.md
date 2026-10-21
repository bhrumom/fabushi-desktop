# Perform-task skill complete read — strict recursive indices 45-50

Status: parent exact-head verified at 62b48720cbdbe455bf476faef2445801e9af622e; strict index50 source-grounded and mapped; child gates pending.

Authority: telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844 tree 1fc479a4b448aed851181bf4a4796d3015368348.
Index 45 source: `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`.
Index 46 source: `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`.
Index 47 source: `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259` (14,178 bytes).
Index 48 source: `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` (9,933 bytes).
Index 49 source: `.agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba` (61,134 bytes; 1,163 lines).
Index50 pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7 was fully read (1,086 lines) and mapped to existing source-neutral repository governance. Strict index51 is intentionally deferred until fresh child exact-head TDRP/Rust gates pass; its path/blob is not guessed.

## Parent exact-head evidence

TDRP Revision 9 run 39526012643 is terminal success on a822aa26c3b3c48337b473d7a4885e470eb216ab with 61/61 jobs successful and every job bound to that exact head. Source authority artifact 12312732290 digest sha256:5165ee74c71b1bf112ae21ca9a3f8ecc387a9143e592e0fd1255779c2684ccfa binds target a822aa26c3b3c48337b473d7a4885e470eb216ab, upstream 0eca52223cceed229932add1cc80792763120844 and tree 1fc479a4b448aed851181bf4a4796d3015368348. Boost artifact 12312891818 digest sha256:f0dfe813f887deba2f8d028a0baf415a590a7942e00f86fbc4bd3cd73505410a is 582/582/open=0. Root aggregate 12313446458 digest sha256:2e523ddc50afb6fe519063d669c1cf4d27df8d78e27f54140e02af1223aa3ec5 covers 0-11; child aggregate 12313046811 digest sha256:a1fb780474ead2f2261a7f0cee96afb92e48ce6860bbee8264d0423ef89be82f covers 0-23; next aggregate 12313022389 digest sha256:2c7ba4b3fc0f227301f96fcf075a31ee71c4638dd6f7f6ad7d261273e0c00715 covers 0-15; root-source archive 12313911558 digest sha256:1e3569fe45c0a1f0fa3fd1dcaa85fb380c4e537cbd25dfd67c67d241bf1b1a88. Fixed-point artifact 12313932639 digest sha256:d65f58124493361e3a2c41b3480d7c08bd2500cb0e364cad866344a47245139a proves unresolved_gitlink_authorities=0 while source_closure_ready=false. Reachability artifact 12313827788 digest sha256:fc607272d97d85fa22a1fcca1a281ec34615cc648090844478d71beadad71c3a remains partial-audited-open with remaining_open_totals authorities=95, gitlinks=133, acquisition_candidates=3865, generated_inputs=4674, resource_inputs=4602, tool_inputs=4255; baseline_identity_accepted=false and source_closure_ready=false.

Rust desktop runtime run 39526012677 is terminal success on a822aa26c3b3c48337b473d7a4885e470eb216ab: rust-host, renderer, platform-worker and Ubuntu/Windows/macOS pressure-profiler are 6/6 success. Artifact 12313941772 `rust-messaging-responsibilities-a822aa26c3b3c48337b473d7a4885e470eb216ab` digest sha256:c6d02b9d06ba49f3fb5ef47e9f8bcc3ea24e35a663f3d5f7cb120d71ca6b641d binds the exact parent.

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

## Index 46 retained traceability

- Requirement: TDRP-R9-PERFORM-TASK-OPENAI-INTERFACE-001
- Oracle: ORA-TDRP-PERFORM-TASK-OPENAI-INTERFACE-001
- Invariants: INV-TDRP-PERFORM-TASK-OPENAI-METADATA-ONLY-001, INV-TDRP-PERFORM-TASK-NO-SHIPPING-TASK-COLLISION-001
- Contract: CONTRACT-TDRP-PERFORM-TASK-OPENAI-INTERFACE-001
- Integration: INT-TDRP-PERFORM-TASK-OPENAI-TO-SHARED-SKILL-001
- Security: SEC-TDRP-PERFORM-TASK-HOST-METADATA-NO-AUTHORITY-001

## Strict index 47 complete source read

The complete `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259` is a Codex-only repository TEST adapter over the shared test loop, not Telegram or Fabushi shipping Computer behavior.

- Driver policy: `auto` defaults to overlay; `overlay-only` forbids physical driving; `required` requires the named physical flow. Hybrid is selected only when pointer, keyboard, focus, scrolling, dragging, menus, windowing or native-UI mechanics are themselves the acceptance subject. Narration or an uninstrumented click-through is never approval evidence.
- Capability gate: a surfaced active Computer Use skill path, supported host/runtime, already-satisfied permissions and unambiguous test-app identity are all required. Installed files/configuration alone do not prove capability.
- Locked macOS override: a locked graphical session is never driven and does not block the task; the complete flow moves into the deterministic in-binary overlay.
- Interrupted sessions: Escape, local input or a stopped driver contaminates that driver session only. Other evidence work continues and a fresh bounded session requires all gates again.
- Exact-app gate: PID, start time, exact executable and full app target are revalidated around every state/action call. Display-name targeting is forbidden. Computer Use never controls terminals, ChatGPT, Git, builds or cleanup.
- Hybrid handshake: overlay emits deterministic state/observers/watchdog/`CU_READY`; runner obtains fresh state, resolves a fresh AX target or last-resort coordinate, performs exactly one bounded declared action, revalidates the app tuple, and the overlay decides/captures the semantic result.
- Safety envelope: only disposable local test state is eligible. No credentials, permission grants, real/server-visible account mutation, confirmation-gated external flow, payment or system/security prompt is crossed.
- Unavailable mapping: `auto` falls back to a prewritten equivalent overlay; `required` with no equivalent may become exact `BLOCKED(test)`; locked macOS is overlay, not blocked. One stopped session or run cap is not exhaustion.
- Durable evidence: hybrid runs persist driver metadata, fresh full AX before/after snapshots and immediate before/after screenshots. Evidence precedence is fresh `-testagent` assertion/crash, overlay semantic/logged/geometry evidence, persisted AX, tight screenshots, then narration.

## Real index 47 invocation and bounded recovery path

1. `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` requires index47 only when UI-driver selection/operation is relevant; Q0-Q2 mechanical-lane tasks skip test-loop/Computer Use references.
2. `.agents/skills/perform-task/references/pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7` says Computer Use needs the separate capability gate and reads index47 for UI-driver selection/operation while task-derived oracle/layout/watchdog/crash/account/evidence rules remain authoritative.
3. `.agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba` requires the evidence plan to name which checks need build/app/account/Computer Use/screenshot and why, using the most direct practical instrument.
4. `.agents/shared/test-loop.md@83c4732bf6139f7c6557f18dd87bb35c3d68808d` owns the directness ladder: produced artifact/runtime value -> narrow changed command/unit/callback/component -> purpose-built probe/API boundary -> real consumer/logged assertion -> established insertion seam -> exact callback/network-result injection -> physical input/visual capture only when interaction/pixels are the subject.
5. A repeated signature forbids the failed technique. `MAX_TEST_RUNS` closes the normal campaign, not the task; at most one focused recovery campaign runs unmet checks/controls. No third campaign is automatic. Cap, timeout, repeated signature, probe complexity and blank screenshots never become approval, exhaustion or task Block by themselves.

## Current Fabushi existing owner and shipping Computer boundary

Fabushi exact 8ee1201ef957348bc1f91255211221a7d77efaad already owns the applicable source-neutral responsibility:

- `AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b`: “A verification run is one related execution set with an immutable run identity” under Adaptive verification and evidence recovery; exact action/log/artifact/oracle evidence, carried-forward valid passes, `TEST_FLAW`, independent directness before `BLOCKED/UNRECOVERABLE`, exact-owned cleanup and FBCP/TDRP GitHub-Actions-only execution are already normative.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb`: “The default managed campaign budget is two campaigns total”; direct positive evidence, `TEST_FLAW` versus `IMPL_BUG`, one focused recovery campaign and independent exhaustion are already owned.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70`: “For `TEST_FLAW` recovery, preserve” prior proof, failed assumption/signature, forbidden technique, next directness strategy and unmet checks in append-only exact-run evidence.

The shipping Computer owner was independently inspected and rejected for index47 ownership:

- `frontend/src/recovered/features/computer/shell/controller.ts@52887c16f3ca4b01871bf991b460be7873f25e36` owns `useComputerExperience`, ForeverBox status/ensure, product monitors, Computer actions, open/hand-back and transcript handoff projection.
- `model.ts@b99b77f2299d83cd49e66875e7b78081ed6fec8c` owns VNC channels/partition/preferences and Computer status/monitor/cursor/handoff projection.
- `status-store.ts@e57c92249587250d310ab0c26ba187d77f47d41b` owns ForeverBox status snapshots, deadlines, retention, reconnect/focus and Computer action state.
- `vnc-webview.tsx@bdb077ea73e051afc67b80b18edbe6f86d70a885` owns shipping VNC webview/session/liveness/crash lifecycle.

Those are end-user Computer runtime owners. Repository UI-driving test policy must not be moved into them. `novel_capability=false`; no new owner or ADR is required.

## Index 47 Fabushi-equivalent contract

`task-derived acceptance/risk -> most direct GitHub Actions evidence instrument -> physical driving only when interaction itself is the subject -> exact subject/capability/safety gate -> one bounded action -> semantic/AX/capture evidence -> causal classification -> directness-advancing bounded recovery -> visible unverified/block/hard-stop rather than synthetic pass`

Invariants:
- repository UI-driving never becomes shipping Computer ownership;
- deterministic instrumented evidence is preferred unless physical interaction itself is the contract;
- ambiguous process/app/target identity prevents action rather than broadening control;
- narration and uninstrumented clicks are never proof;
- prior valid passes survive recovery until explicitly invalidated;
- repeated evidence failure advances directness; a run/campaign cap is not itself a verdict;
- unattended verification crosses no credential/confirmation/payment/system-permission boundary;
- FBCP/TDRP executable verification remains GitHub-Actions-only;
- open recursive coverage keeps baseline_identity_accepted=false and source_closure_ready=false.

## Index 47 traceability

- Requirement: TDRP-R9-PERFORM-TASK-COMPUTER-USE-TESTING-001
- Oracles: ORA-TDRP-PERFORM-TASK-CU-DRIVER-SELECTION-001, ORA-TDRP-PERFORM-TASK-CU-EVIDENCE-001
- Invariants: INV-TDRP-PERFORM-TASK-CU-EXACT-APP-001, INV-TDRP-PERFORM-TASK-CU-SAFETY-001, INV-TDRP-PERFORM-TASK-CU-NO-SHIPPING-COMPUTER-COLLISION-001
- Contract: CONTRACT-TDRP-PERFORM-TASK-CU-ADAPTER-001
- Integration: INT-TDRP-PERFORM-TASK-CU-TO-SHARED-TEST-LOOP-001
- Temporal: TEMP-TDRP-PERFORM-TASK-CU-INTERRUPTION-001
- Fault recovery: FAULT-TDRP-PERFORM-TASK-CU-UNAVAILABLE-001, FAULT-TDRP-PERFORM-TASK-CU-INTERRUPTED-001
- Security: SEC-TDRP-PERFORM-TASK-CU-SAFETY-ENVELOPE-001
- Regression: REG-TDRP-PERFORM-TASK-CU-NARRATION-NOT-PROOF-001

## Strict index 48 complete source read

The complete `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` is repository code-health workflow governance for behavior-preserving Q0/Q1/Q2 maintenance, not Telegram product behavior and not Fabushi shipping Task/Agent/Computer/Host/Runner runtime.

- Eligibility is exact: only a task whose `task.md` has `## Code health` class Q0 delete, Q1 move or Q2 merge-identical-copies enters the lane.
- Q0 removes only proven-dead material and its exclusive declarations/includes/CMake/style/language/field/function/class users after platform/generated/build references are proven absent.
- Q1 relocates code without semantic edits; only includes, declarations, namespace scaffolding and CMake source-list entries may be newly introduced.
- Q2 consolidates copies proven identical apart from names/literal values into one already-linkable shared function with caller-specific values parameterized.
- The common bound forbids new strings/styles/settings/log lines, reordered side effects, templates, header-defined functions or variant dispatch and forbids silently growing build work or binary size.
- Edit scope is bounded to task-named/receiving/shared files, direct-user includes, relevant CMake lists, area-map Module lines and only-exclusive style/language entries.
- An item that leaves those bounds is restored and recorded as Dropped with a precise reason. It never falls through to the full pipeline inside the same mechanical task; semantic/broader work must be separately reclassified/routed.
- The proof step retains complete command output and class-specific oracles: affected builds/targets, health ratchet/reference/last-use negatives, Q1 MOVE_ONLY accounting and finding verification. Proof failure is fixed only within class bounds or the item is dropped.
- Review is one fresh mechanical reviewer. One bounded repair/resume review is allowed; a second NEEDS_CHANGES drops the named items, reruns required proof, and there is no third mechanical review.
- Publication still uses the ordinary repository task lifecycle: preflight, owned paths, source commit, `work/test.md`, `work/result.md` and `finish --status approved`. An all-dropped task restores the base and finishes already-satisfied with evidence.

## Real index 48 invocation and full-pipeline boundary

1. `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` selects index48 only for a `## Code health` Q0/Q1/Q2 task; it explicitly says to read the mechanical lane plus pipeline Contract/Preflight/Telegram commits/Final AI state and to skip the ordinary test-loop and Computer Use references.
2. `.agents/skills/perform-task/references/pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7` defines Q0/Q1/Q2 as the one exception to the adaptive full implementation/review/evidence path because their class bounds what tools can prove.
3. `.agents/skills/code-health/references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e` supplies the semantic class definitions/proof contract, while `.agents/skills/code-health/scripts/health.py@f08675ef7a03c016d85f91c60b8f29e99454abe9` is the concrete health check/move/verify instrument named by the lane.
4. Directness stays class-specific: exact reference/build/platform/generated-input negatives for Q0, line-level move-only accounting and build placement for Q1, and explicit pre-existing-difference/parameterization/call-site/clone-resolution proof for Q2.
5. The fallback boundary is fail-closed: any required semantic edit, new behavior, broadened scope or unproven equivalence rejects the mechanical classification. The current task drops/restores that item; a stronger ordinary pipeline is entered only through a separate reclassified/routed task, never silently as a continuation of the mechanical lane.

## Current Fabushi existing owner and shipping-runtime boundary

Fabushi exact a822aa26c3b3c48337b473d7a4885e470eb216ab already owns this applicable source-neutral responsibility:

- `AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b` has the unique `Repository quality-task classification governance` owner: narrow mechanical work is deletion of proven-dead material, relocation with no logic edit, and consolidation of copies equivalent apart from explicit parameters; it remains mechanical only while behavior/formats/network/logs/order/timing are unchanged, and semantic/broadened work is reclassified rather than forced through the lane.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb` has `Repository quality-task class qualification`: Q0-Q2 direct class proof plus independent review, with deletion/move/consolidation oracles and a mandatory stronger ordinary pipeline whenever semantic edits, new behavior, widened scope or unproven equivalence appear.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` has `Repository quality-task class evidence`: exact base, finding identity, before/after metric, owned paths, deletion reference/build/orphan negatives, move line accounting, consolidation differences/parameterization/call sites, and explicit dropped-item reasons.

These are repository-quality owners. Shipping Task and Agent execute product work; shipping Computer owns end-user Computer/ForeverBox/VNC; shipping Host/Runner own product execution/takeover. None owns repository maintainability classification, proof or review. The generic repository-quality governance also explicitly says it is not a product runtime, second findings store or task queue. `novel_capability=false`; no new owner or ADR is required.

## Index 48 Fabushi-equivalent contract

`exact Q0|Q1|Q2 task class -> source/base/ownership preflight -> finding recheck -> bounded owned edit -> direct class proof -> fresh mechanical review -> at most one bounded repair/review -> approve remaining items or restore/drop failures -> ordinary source commit/result/finish; any semantic/broader/unproven work -> separate reclassification/full pipeline, never an in-task fallback`

Invariants:
- mechanical classification never weakens a product-risk check otherwise required by the ordinary task/spec;
- Q0/Q1/Q2 remain behavior-preserving and source-neutral rather than becoming Telegram-specific owners;
- direct proof is class-specific and a clean scanner result alone is never behavior or release proof;
- edits never expand beyond the selected class/owned-path boundary to save an item;
- dropped items are restored and explicitly recorded, not counted resolved;
- one mechanical repair/review boundary does not become an unbounded rerun/review loop;
- shipping Task/Agent/Computer/Host/Runner remain product runtime owners and never absorb repository code-health governance;
- FBCP/TDRP executable verification remains GitHub-Actions-only;
- open recursive coverage keeps baseline_identity_accepted=false and source_closure_ready=false.

## Index 48 traceability

- Requirement: TDRP-R9-PERFORM-TASK-MECHANICAL-LANE-001
- Oracles: ORA-TDRP-PERFORM-TASK-MECHANICAL-ELIGIBILITY-001, ORA-TDRP-PERFORM-TASK-MECHANICAL-PROOF-001
- Invariants: INV-TDRP-PERFORM-TASK-MECHANICAL-NO-BEHAVIOR-CHANGE-001, INV-TDRP-PERFORM-TASK-MECHANICAL-BOUNDED-FILES-001, INV-TDRP-PERFORM-TASK-MECHANICAL-NO-SHIPPING-RUNTIME-COLLISION-001
- Contract: CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-LANE-001
- Integration: INT-TDRP-PERFORM-TASK-MECHANICAL-TO-QUALITY-GOVERNANCE-001
- Temporal: TEMP-TDRP-PERFORM-TASK-MECHANICAL-ONE-FIX-BOUNDARY-001
- Fault recovery: FAULT-TDRP-PERFORM-TASK-MECHANICAL-OUT-OF-BOUNDS-DROP-001, FAULT-TDRP-PERFORM-TASK-MECHANICAL-PROOF-FAILURE-001
- Security: SEC-TDRP-PERFORM-TASK-MECHANICAL-OWNED-PATHS-001
- Regression: REG-TDRP-PERFORM-TASK-SEMANTIC-WORK-NOT-MECHANICAL-001

## Strict index 49 complete source read

The complete `.agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba` is repository perform-task orchestration and evidence governance, not Telegram product behavior and not Fabushi shipping Task/Agent/Computer/Host/Runner/Conversation runtime.

- Phase transitions are artifact-gated. Delegation is selected only after a real phase spawn succeeds; a pre-work depth/capacity/policy rejection may select same-session checklists, but a delegated timeout or missing artifact cannot silently switch execution mode.
- Phase 1 produces source-grounded context and plan artifacts. Phase 3 independently validates paths/APIs, containment, evidence design and intrinsic scope, with fail-closed Fast-Path rejected, Approach rejected, or Scope split-required outcomes before source edits.
- Phase 4 executes one assessed owned write set at a time. Phase 5 runs the assessed direct validation and limits repair to task-owned failures; build locks use exact-subject bounded recovery.
- Initial Phase 6 is one complete-diff general review plus all five mutually blind standard lenses. Blocking fixes alone proceed. Focused re-review carries unaffected approvals forward and reruns only invalidated/originating lenses and evidence.
- Review convergence is decided by an independent CONTINUE|REPLAN_CURRENT|RESCOPE_REQUIRED|HARD_STOP assessment with durable rulings. A round count never converts an unresolved blocker into approval.
- Evidence recovery distinguishes TEST_FLAW from IMPL_BUG. TEST_FLAW retains implementation and prior passes, forbids the failed technique, advances directness, and reruns only unmet/invalidated checks. IMPL_BUG alone authorizes implementation repair plus focused review.
- The default automatic evidence ceiling is one normal campaign plus at most one focused recovery campaign. A cap, timeout, flake, blank capture or repeated signature is never pass or recovery exhaustion by itself.
- Native-Windows Phase 7 is post-review and limited to task-touched textual files from result logs, preserving content/trailing-newline semantics while writing CRLF UTF-8 without BOM. WSL/Linux keep LF/no-BOM.
- Prompt/result logs make state/evidence handoff reconstructable; incomplete artifacts, ambiguous ownership and unsupported recovery fail closed.

## Real index 49 invocation and shared-reference path

1. `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` requires the complete engine and names `references/pipeline.md`, `references/phase-prompts.md`, phase effort, Codex delegation/CLI leaves, shared test loop and build-lock recovery before phase work.
2. `.agents/skills/perform-task/references/pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7` remains the authoritative stateful performer contract and delegates exact leaf phase prompts/retry rules to index49.
3. `.agents/shared/phase-effort.md@dd07a4c611922a29a6892332db98afe76cd58952` owns effort/model selection; `.agents/shared/codex-delegation.md@ed89d2a14344f50034a9363bd9ffe75f7972e652` owns native Codex completion/recovery; `.agents/shared/codex-cli-leaves.md@659da44a8083c433c4e9b59ca28f64ba525bfbf8` supplies the Claude review-leaf adapter.
4. `.agents/shared/test-loop.md@83c4732bf6139f7c6557f18dd87bb35c3d68808d` owns APPROVED/TEST_FLAW/IMPL_BUG/UNRECOVERABLE evidence state, carry-forward invalidation and the two-campaign ceiling.
5. `.agents/shared/project-context.md@53c8657eb7949a5a2fba7e7adc8d1073f3079ead` owns compact context provenance/amendments; `.agents/shared/build-lock-recovery.md@ec75573e7efe74e9c9a344f3e66f2fc43ac7680f` owns exact-subject bounded lock recovery.
6. Phase 6's `health.py` invocation is supporting repository-quality tool output only; it never becomes review or product acceptance authority.

## Current Fabushi existing owner and shipping-runtime boundary

Fabushi exact 16d95f0607a203fb16a96afa99751a39892ec7f0 already owns this applicable source-neutral responsibility:

- `AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b` owns the canonical Discover/Spec/Architecture-Plan/Implement/Verify/Review/Integrate-Deliver lifecycle, compact context handoff, worker effort, out-of-process AI-worker lifecycle/adapter safety and exact-head fail-closed delivery.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb` owns independent/focused review, arbitration/convergence, AI review phase execution, adaptive evidence recovery, build-lock safety and AI-worker qualification.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` owns frozen review scope, durable review ledger, context provenance/handoff, AI-worker execution evidence and append-only adaptive test-loop recovery history.

Shipping Task, Agent/Coordinator, Computer, Host/Runner and Conversation continue to own end-user/product task state, agent/tool execution, Computer/ForeverBox/VNC, product execution/takeover and user transcript/participant/message state. None owns repository phase prompts, reviewer fanout, worker evidence or test-recovery governance. `novel_capability=false`; no new owner or ADR is required.

## Index 49 Fabushi-equivalent contract

`exact task/source authority -> context+plan artifacts -> independent assessment with reject/replan/split gates -> bounded owned implementation phases -> selected direct validation -> blind complete-diff general+five-lens review -> blocking-only fixes with focused invalidation-aware re-review -> convergence assessment when needed -> adaptive evidence with carried passes/directness recovery and at most one focused campaign -> optional native-Windows touched-file normalization -> durable exact-result publication`

Invariants:
- the performer is the only stateful repository task owner; leaf workers do not delegate or commit;
- phase completion is established by required durable artifacts and actual effects, never narration, timeout or runner exit alone;
- implementation/fixes remain inside assessed owned paths and invalidate only actually affected review/evidence scopes;
- initial standard review remains complete and mutually blind; focused review carries unaffected approvals;
- round/run/campaign caps, flake, timeout and missing captures never become approval;
- TEST_FLAW advances evidence directness without mutating retained implementation absent sound IMPL_BUG proof;
- host-specific worker and Telegram-specific test mechanics remain adapters, not shipping product owners;
- shipping Task/Agent/Computer/Host/Runner/Conversation never absorb repository workflow governance;
- FBCP/TDRP executable verification remains GitHub-Actions-only;
- open recursive coverage keeps baseline_identity_accepted=false and source_closure_ready=false.

## Index 49 traceability

- Requirement: TDRP-R9-PERFORM-TASK-PHASE-PROMPTS-001
- Oracles: ORA-TDRP-PERFORM-TASK-PHASE-ARTIFACT-COMPLETION-001, ORA-TDRP-PERFORM-TASK-FOCUSED-REVIEW-001, ORA-TDRP-PERFORM-TASK-EVIDENCE-DIRECTNESS-001
- Invariants: INV-TDRP-PERFORM-TASK-STATEFUL-PERFORMER-001, INV-TDRP-PERFORM-TASK-LEAF-NO-DELEGATE-COMMIT-001, INV-TDRP-PERFORM-TASK-NO-SHIPPING-RUNTIME-COLLISION-001
- Contract: CONTRACT-TDRP-PERFORM-TASK-PHASE-PIPELINE-001
- Integration: INT-TDRP-PERFORM-TASK-PHASES-TO-FABUSHI-REPOSITORY-GOVERNANCE-001
- Temporal: TEMP-TDRP-PERFORM-TASK-CARRY-FORWARD-INVALIDATION-001, TEMP-TDRP-PERFORM-TASK-TWO-CAMPAIGN-CEILING-001
- Fault recovery: FAULT-TDRP-PERFORM-TASK-PHASE-ARTIFACT-INCOMPLETE-001, FAULT-TDRP-PERFORM-TASK-TEST-FLAW-DIRECTNESS-001, FAULT-TDRP-PERFORM-TASK-CONVERGENCE-001
- Security: SEC-TDRP-PERFORM-TASK-OWNED-PATH-AND-WORKER-IDENTITY-001
- Regression: REG-TDRP-PERFORM-TASK-PRODUCT-RUNTIME-SEPARATION-001

## Verification disposition

This closes only recursive strict index 49. Counters become source_entries_total=16,305, unknown=16,253, unread=16,228, omitted=0, responsibilities_total=68, unmapped=0. baseline_identity_accepted=false, baseline_ready=false, accepted=false and source_closure_ready=false because source coverage and recursive reachability/disposition remain open.

Parent 16d95f0607a203fb16a96afa99751a39892ec7f0 evidence is supporting evidence only after this write. Exact-head acceptance of index49 requires fresh TDRP Revision 9 61-job and Rust desktop runtime 6-job workflows on the immutable child. Only after every child job/step/full log/artifact/digest closes without deterministic failure may the audit enter strict index50.

## Index 50

Complete source read of pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7: preflight, validated resume/checkpoints, bounded delegation, owned implementation, independent/focused review and convergence, APPROVED/TEST_FLAW/IMPL_BUG/UNRECOVERABLE, two-campaign ceiling, exact-path publication, DONE/BLOCKED/SPLIT_REQUIRED. Existing owners: AGENTS.md, TEST_STRATEGY.md, evidence-contract.md. No shipping Task/Agent/Computer/Host/Runner/Conversation owner; novel_capability=false.

Parent evidence: TDRP 39531160961 61/61 at 62b48720cbdbe455bf476faef2445801e9af622e; Rust 39531160843 6/6; artifact 12315168809 digest sha256:2642e30438a89d5231cc1944455e8a1ddc77a016fbfdc36c5d2ca51626f0290c. Coverage now unknown=16252, unread=16227, omitted=0, responsibilities=68, unmapped=0. baseline_identity_accepted=false/source_closure_ready=false. Strict index51 only after child gates.

## Index 50 traceability

- Source responsibility: `TDRP-R9-PERFORM-TASK-PIPELINE-001`
- Requirement: `TDRP-R9-PERFORM-TASK-PIPELINE-001`
- Oracles: `ORA-TDRP-PERFORM-TASK-PIPELINE-RESUME-001`, `ORA-TDRP-PERFORM-TASK-PIPELINE-REVIEW-EVIDENCE-001`, `ORA-TDRP-PERFORM-TASK-PIPELINE-PUBLICATION-001`
- Invariants: `INV-TDRP-PERFORM-TASK-PIPELINE-STATEFUL-PERFORMER-001`, `INV-TDRP-PERFORM-TASK-PIPELINE-LEAF-NO-COMMIT-001`, `INV-TDRP-PERFORM-TASK-PIPELINE-NO-SHIPPING-RUNTIME-COLLISION-001`
- Contract: `CONTRACT-TDRP-PERFORM-TASK-PHASE-PIPELINE-001`
- Integration: `INT-TDRP-PERFORM-TASK-PHASES-TO-FABUSHI-REPOSITORY-GOVERNANCE-001`
- Temporal: `TEMP-TDRP-PERFORM-TASK-CARRY-FORWARD-INVALIDATION-001`, `TEMP-TDRP-PERFORM-TASK-TWO-CAMPAIGN-CEILING-001`
- Fault recovery: `FAULT-TDRP-PERFORM-TASK-PHASE-ARTIFACT-INCOMPLETE-001`, `FAULT-TDRP-PERFORM-TASK-TEST-FLAW-DIRECTNESS-001`, `FAULT-TDRP-PERFORM-TASK-CONVERGENCE-001`
- Security: `SEC-TDRP-PERFORM-TASK-OWNED-PATH-AND-WORKER-IDENTITY-001`
- Regression: `REG-TDRP-PERFORM-TASK-PRODUCT-RUNTIME-SEPARATION-001`
- Source: `.agents/skills/perform-task/references/pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7`
- Existing-owner boundary remains repository-only: `AGENTS.md`, `TEST_STRATEGY.md`, and `evidence-contract.md`; no shipping Task/Agent/Computer/Host/Runner/Conversation owner is introduced.


## Strict index 51 — process-inbox

Source `.agents/skills/process-inbox/SKILL.md@0ce70794ac743d99e6d5c80894cde970b52842db` was fully read at telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844 (432 lines). Dependencies `workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c` and `video.py@977d75aca7aea0b6c74471d533a5ee0e2a156ac3` were also read and SHA-verified. The workflow creates/resumes one digest-bound isolated inbox worktree transaction, resolves task lineage and duplicate digests, writes task/state/receipt artifacts, publishes explicit paths, and finalizes only after verifying the receipt in master HEAD and unchanged live inbox digest. Video inputs are probed/extracted locally and kept out of tracked artifacts.

Existing-owner-first maps this row's target only to the single existing heading `Context provenance and handoff evidence` in `projects/fabushi-communication-platform/quality/evidence-contract.md`. The broader lifecycle policy in `AGENTS.md` and review/convergence qualification in `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` remain candidates but are not target paths for this row; their responsibility-specific headings stay tracked by separate ledger rows. Shipping Task/Agent/Coordinator/Host/Runner/Conversation are rejected as unrelated product/runtime owners. This row remains `mapped` / `toolchain-equivalent` only, not implemented or verified: no equivalent local inbox helper or focused behavior test is proven. No product owner added.

Traceability: TDRP-R9-PROCESS-INBOX-001; ORA-TDRP-PROCESS-INBOX-DIGEST-TRANSACTION-001; ORA-TDRP-PROCESS-INBOX-RECEIPT-PUBLICATION-001; INV-TDRP-PROCESS-INBOX-SINGLE-ACTIVE-TRANSACTION-001; INV-TDRP-PROCESS-INBOX-RECEIPT-DIGEST-FENCE-001; CONTRACT-TDRP-PROCESS-INBOX-TRANSACTION-001; INT-TDRP-PROCESS-INBOX-TO-FABUSHI-GOVERNANCE-001; TEMP-TDRP-PROCESS-INBOX-RESUME-FINALIZE-001; FAULT-TDRP-PROCESS-INBOX-REMOTE-OUTAGE-001; SEC-TDRP-PROCESS-INBOX-WORKTREE-RECEIPT-FENCE-001; REG-TDRP-PROCESS-INBOX-NO-SHIPPING-RUNTIME-COLLISION-001.

Parent exact-head evidence: TDRP 39537102683 61/61 jobs, 506/506 steps; Rust 39537102580 6/6 jobs, 79 success steps and two expected platform-conditional skips. Source authority artifact 12318286826 sha256:1cd8c4c82a94600131d6103f9742d9553156190faf66483cc8f5308312ba8404; root archive artifact 12317737812 sha256:c0b5764046498af60ae37ca817f954fc6c8d5ccd2e08b37c391e2a4b94067a24; Rust artifact 12318237159 sha256:a93c6575eab099ca643c5ce10782689b5d16c0c6cdbe81897966b9a0236b9eb8. These are parent governance gates, not inbox helper behavior tests. Counters now unknown=16251, unread=16226, omitted=0, responsibilities=69, unmapped=0. Baseline acceptance/source closure remain false; recursive disposition has 95 authorities, 133 gitlinks, 3865 acquisition candidates, 4674 generated inputs, 4602 resource inputs and 4255 tool inputs open.

## Live dev rebaseline reread — 5c674f1f6a0f6ace55b67f8bc295138680fe588b

Strict index 50 was reread in full at `.agents/skills/perform-task/references/pipeline.md@46e158ac86ebce97f9dcd010a98b3c581ffc114c` after live `dev` advanced 49 commits from `0eca52223cceed229932add1cc80792763120844`. Existing responsibility `TDRP-R9-PERFORM-TASK-PIPELINE-001` and its requirement/oracle/invariant traceability remain applicable.

The semantic delta adds a source-host operational mirror: after `source-commit`, the helper may force-push the checkout tip to `refs/heads/ai-<checkout-tag>` on a `private` remote. The mirror is explicitly non-authoritative: nobody works on it, it is never lineage, checkout branch state is unchanged, the next commit overwrites it, missing remote skips it, and failed/timed-out push is reported but does not fail the commit/task. Manual or lease-based mirror publication is forbidden by the source contract.

This is repository-host workflow metadata, not a shipping Fabushi Task/Agent/Computer/Host/Runner/Conversation capability and not grounds for a second runtime/state owner. Applicable lifecycle/review/evidence semantics remain mapped to existing source-neutral repository governance. Rebaseline accounting: recursive total=16,306, unknown=16,250, unread=16,225, omitted=0; `baseline_identity_accepted=false` and `source_closure_ready=false` pending fresh exact-head GitHub Actions.
