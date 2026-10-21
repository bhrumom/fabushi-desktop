# Perform-task skill complete read — strict recursive indices 45-47

Status: parent exact-head verified at 8ee1201ef957348bc1f91255211221a7d77efaad; strict index 47 source-grounded and existing-owner mapped; child exact-head verification pending. Earlier index46 validator repairs remain retained and index45/index46 traceability is preserved.

Authority: telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844 tree 1fc479a4b448aed851181bf4a4796d3015368348.
Index 45 source: `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`.
Index 46 source: `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`.
Index 47 source: `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259` (14,178 bytes).
Next strict entry: index 48 `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef`.

## Parent exact-head evidence

TDRP Revision 9 run 39523164637 is terminal success on 8ee1201ef957348bc1f91255211221a7d77efaad with 61/61 jobs successful and every job bound to that exact head. Source authority artifact 12312202495 digest sha256:a374e7d53b3b71260bff8fd14addb5b7259bf135d66d27fe9b8dc443e1040bcd binds target 8ee1201ef957348bc1f91255211221a7d77efaad, upstream 0eca52223cceed229932add1cc80792763120844 and tree 1fc479a4b448aed851181bf4a4796d3015368348. Boost artifact 12311802679 digest sha256:6aa8176849937d0c2cb9327d37eaff86742056ac303b25dbdefe6a200f9f945c is 582/582/open=0. Root aggregate 12312222100 digest sha256:f6812550ef3eadc67c98abb1d87d7e28c280bc2874ec98c02cbfa6f0f37dfb47 covers 0-11; child aggregate 12311438106 digest sha256:b3cab5670ff66b7dab4112ca0393910c0bb3a1d70ee32c83094f954c3eed77ce covers 0-23; next aggregate 12311558479 digest sha256:77312c491c87d4a39e78b3ec26b700c159e278b4da64df2cb11e940c9ce5eca0 covers 0-15; root-source archive 12312412067 digest sha256:1341d3639a438d2aaaf3cff456af7266700ac77251f9e643b4962706e529f13d. Fixed-point artifact 12311308763 digest sha256:660c85192a30217ec493bc748e510791b5870d3c2b58e9dadaec94da81a6fe29 reaches final_scanned_authorities=203 and unresolved_gitlink_authorities=0 while source_closure_ready=false. Reachability artifact 12311603620 digest sha256:a56a6676732008b3cf416bbd3fd61ad57b3e189f73a8c41a4a36e70d1774cc62 remains partial-audited-open with remaining_open_totals authorities=95, gitlinks=133, acquisition_candidates=3865, generated_inputs=4674, resource_inputs=4602, tool_inputs=4255; baseline_identity_accepted=false and source_closure_ready=false.

Rust desktop runtime run 39523164652 is terminal success on 8ee1201ef957348bc1f91255211221a7d77efaad: rust-host, renderer, platform-worker and Ubuntu/Windows/macOS pressure-profiler are 6/6 success. Artifact 12311798004 `rust-messaging-responsibilities-8ee1201ef957348bc1f91255211221a7d77efaad` digest sha256:c422ca9a391c4d3ea3eb53e4e2a50ec4f43fb609a259464050f5bf31b167bfab binds the exact parent.

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

## Verification disposition

This closes only recursive strict index 47. Counters become source_entries_total=16,305, unknown=16,255, unread=16,230, omitted=0, responsibilities_total=66, unmapped=0. baseline_identity_accepted=false, baseline_ready=false, accepted=false and source_closure_ready=false because source coverage and recursive reachability/disposition remain open.

Parent 8ee1201ef957348bc1f91255211221a7d77efaad evidence is supporting evidence only after this write. Exact-head acceptance of index47 requires fresh TDRP Revision 9 61-job and Rust desktop runtime 6-job workflows on the immutable child. Only after every child job/step/log/artifact/digest closes without deterministic failure may the audit enter strict index48.
