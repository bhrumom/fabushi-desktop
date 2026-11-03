# PS-LOCK-RECONCILE-v2 — recovery R4 STRUCTURE / REUSE source review

Verdict: APPROVED (scoped immutable-source review only).
MUST_FIX: none.
NON_BLOCKING: no new finding.

Task: 647e0932-bea4-4113-9a61-65ae3db58955; recovery R4, independent worker round 1.
Worker invocation recorded by the parent: gpt-6.1-sol / xhigh. This worker did not perform backend introspection.
Intended report path: projects/openclaw-rust/evidence/plugin-skills-admission-lock-647e0932-20261103/review-r4/structure.md

## Authority and immutable subject

Independently obtained GitHub repository metadata: bhrumom/fabushi-desktop is public, repository ID 1375312893, organization owner ID 281146136. Authenticated login bhrum has admin permission. Independently read the subject Git commit and its recursive tree: d29be0b93225943851c6054016686d571f8d9e2a / 16a9b18517874fa7d82e562729bbe4782bd99d79. The recursive tree was not truncated; all sixteen scope path/blob identities match it. No applicable nested AGENTS.md exists under this scope.

Fixed upstream identity was independently obtained from openclaw/openclaw: 1440bb1771372334bea23d7642bb52deb7db1845 / 90c8aab39972da11cfad220361781d49e1bc6700. No mutable branch head was substituted.

Original base: 9864d1c02e7e67881d644800963abc807b3abaab.
Previous source candidate used only as the fix-delta boundary: b8a648abd90b6ae5c4f9feae8a1e159f126fd69b.
Scope manifest: 0569f7bf2fb1c3767b4813d46edcb66a811d3923.
Original-base diff: fec62e54326254def9301405dc869ba7ea77f551.
Focused delta: 6cf3ab16fc6f2576e4395f0ad86e5745279ebace.

Read root AGENTS.md, the applicable Plugin Skills specification (77a99b596b43333e2f9ef5bff5b1395025032247), the source contract, and relevant OCRM source-of-truth/module-map/main-Spec sections. LR2-01 / PSL08 and AR-LOCK-R3 were treated as requirements. Historical verdict/disposition strings embedded in canonical documents or metadata were not used as acceptance evidence. No prior review report, other reviewer output, runner decision file, failure assessment, or Actions log was opened.

## Focused findings and reasoning

1. The recovered desktop helper is a complete image/status oracle.

source/host/tests/mcp_coordinator_relay_contract.rs::RealService::assert_uninstall_custody_with_refresh (lines 719–759) requires the v3 key to be present, accepts only explicit null or a canonical UUID string, and requires a non-null identity whenever these fixtures establish actual confirmed/pending uninstall custody. It still compares the entire decoded object, complete status, and exact initialization marker bytes (ASCII 1 followed by LF). Unknown fields or other image-field changes cannot be hidden by a subset assertion. UninstallCustodySnapshot retains status, raw image bytes and marker, and now also exposes the observation for independent lifecycle comparison.

The actual field is included in the expected JSON only after shape/canonicality checks. Its behavior is independently constrained, rather than accepted solely by copying it: seed requires null; held refresh-only reservation requires null; real uninstall reservation establishes a value; exact Removed settlement must rotate it; stale-refresh settlement and fresh-refresh reservation preserve it; fresh post-fact materialization/release must retain a non-null identity and rotate it.

2. The existing three actual desktop consumers retain their negative controls.

desktop_observation_rejects_a_pre_fact_success_from_the_actual_loader (lines 940–1011) still holds the real relay loader, enters actual uninstall orchestration, retains display-projection failure, demands the stale-sync error, preserves cache/skill bytes, verifies current admission remains closed, and then proves fresh exact-ID restoration. Added comparisons distinguish reserve, terminal settlement, refresh-only bookkeeping and positive release.

late_desktop_uninstall_reply_cannot_settle_the_successor_service_request (lines 1014–1094) still has two real relay/service instances and distinct UUID-qualified request identities. Late success/error replies cannot settle the current waiter or modify either complete custody snapshot, cache or skill bytes. Independent real reservations have distinct observations; only the matching current reply rotates the current settlement observation.

closed_desktop_service_never_replays_an_admitted_uninstall_or_dispatches_new_work (lines 1097–1146) inherits the strict helper and continues comparing the entire raw snapshot after close, each late reply, and each rejected new desktop method. Its unchanged-byte and zero-new-dispatch assertions remain.

3. The same-root handoff remains a real consumer and lifecycle test.

source/host/src/extensions/mcp/coordinator_relay.rs::lifecycle_tests::desktop_confirmation_keeps_the_actual_reader_across_same_root_handoff (lines 879–1042) is entirely inside the existing cfg(test) module. Production relay code is unchanged by the focused delta. The test now checks a complete confirmed v3 image and a canonical observation, then requires a distinct real reservation for 43 to rotate it. It retains the complete pending v3 image and exact marker.

The test still binds actual ProductionSessionWorkers stores, proves Arc::ptr_eq across successor construction, denies confirmed 41 and pending 43 through retained/new consumers, and keeps 42's body readable. A late receipt reaching the retired predecessor cannot settle its ticket; complete journal bytes/status remain unchanged. A new call through the retired Skills owner stops before mutation, and cache/all original skill-file bytes remain unchanged. The added reads do not introduce a new policy owner, reconciliation loop or lifecycle fixture.

4. Production ownership and serialization support these oracles.

Deep-read the changed admission/lifecycle sections of plugin_skills.rs, including is_denied/acquire, constructor, begin/settle uninstall, durable observation import, refresh custody/reservation/settlement, one-shot retirement/disposal, aggregate import, publication/release, cleanup and reporting. begin_uninstall and exact settle_uninstall use the existing publication and admission EX interval and route through persist_uninstall_transition_locked, which rotates a fresh canonical observation. Auth/refresh transitions use persist_auth_transition_locked and preserve it. Confirmed release follows cache publication/readback and selects the uninstall transition only for a real confirmed-set change. Cross-process identity is checked at import, publication/release and cleanup; in-process Arc fences remain.

Deep-read the private lock/DTO/decoder/encoder/read/publication primitives and the added v3 migration/integrity/lifecycle tests in plugin_skills_uninstall_state.rs. New writes are strict v3 with required nullable observation; v1/v2 remain strict reads with no invented observation. Exact image/marker bytes, owned staging, barriers, expected-versus-candidate failure cuts and non-null identity-loss refusal remain.

Read the actual McpHostService desktop/uninstall orchestration and production PluginSkillsPort delegation. The status helper invokes the shipping status projection; uninstall uses actual reserve-before-backend and exact settlement-before-display projection. Read actual Host binding preparation/start/stop: same-root construction inherits the existing reader after one-shot retirement, preparation I/O stays outside the binding lock, and latest-owner retention precedes a later fallible bind.

Read the actual Session forwarding and WorkflowStore admission/current-record mutation paths, plus the publisher lease and cleanup callback path. These continue to extend existing owners: the forwarding slot is released before delegate access; Workflow update re-reads the record/current publisher under the actual lease; resync/unpublish retain their lease through protected work and release it before refresh/diagnostics.

5. Shared evidence wiring is preserved.

Read every changed hunk of capture_portable_contracts.py, parity-ledger.json and requirements-traceability-matrix.json, including independent SC identity and LXO/LXR preservation. Existing evidence references, selected tests and bounded acceptance language are retained; the LXR unresolved-custody and SC invocation-identity additions are not accepted as runtime proof by this Plugin verdict. No global completion flag is promoted.

The capture already includes both edited Rust files in its source-input inventory (coordinator_relay.rs at line 25, mcp_coordinator_relay_contract.rs at line 39). Its existing plugin-uninstall-service requires the three affected desktop names, and plugin-uninstall-relay-handoff requires the exact affected library name. The existing command/capture exit, required-name uniqueness, nonempty summaries and failed-group rejection remain. No duplicate runner or workflow is needed.

## Read depth

The complete six-path focused delta was read. The sixteen-path original-base scope was retained and inspected through its change hunks, with deep reading of the complete edited helper/tests, edited production functions and the adjacent owner/serializer/consumer boundaries described above. Original-base auth recovery, owned-process/service, Workflow mutation and publisher cleanup tests were read for their real-owner gates, failure/retention controls and test ownership. Large unchanged Session/Host/ledger files were read in the relevant sections; this is not a claim to full-file or whole-repository semantic coverage.

The sixteen paths are the exact scope.json set: the Plugin spec and source contract; parity ledger, RTM and shared capture; plugin_skills.rs, plugin_skills_uninstall_state.rs, plugin_skills_auth_recovery_tests.rs, plugin_skills_sync_tests.rs, skill_publish.rs, Session production.rs, workflow_store.rs, mcp_service_contract.rs, workflow_store_contract.rs, mcp_coordinator_relay_contract.rs and coordinator_relay.rs.

Rust source readiness was assessed by direct source review, including Value/String borrowing in assert macros, Option<String> snapshot comparisons, cfg(test) placement, existing uuid v4 and serde_json dependencies, and actual module/test selection. No concrete compile defect was found. Compilation was not executed or proved by this review.

## Execution and acceptance limits

No local executor, filesystem checkout, compiler, test, lint, parser, generator, benchmark, htch runtime, Actions operation, commit/ref mutation or PR action was used. All source acquisition was through read-only GitHub tools. This report may be saved only as a new GitHub blob under the task's optional report authorization; no commit/ref publication is performed by this worker.

APPROVED means no structural/reuse or causal-oracle defect requiring a source repair was found in this frozen recovery candidate. Corrected exact-H shipping compilation and the affected Actions groups remain required and unexecuted by this worker. This is not real ecosystem/account/backend, supported-platform/package/power-loss, performance/soak, full-source/dependency-closure or final OCRM acceptance.
