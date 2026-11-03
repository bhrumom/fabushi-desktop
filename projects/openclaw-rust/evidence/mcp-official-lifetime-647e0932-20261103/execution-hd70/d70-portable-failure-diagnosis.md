# Exact d70 portable failure diagnosis

Subject **d70f86f33b5e3c67545954b270373e1548809f4b**, tree **4c4c9cfd3f5067fef57dca530042d149a8e2fc5c**; original runtime **41578381556**, attempt **1**, Linux job **125727154700**, portable step **12**.

Original artifact **13204704124** contains 52 executed groups. Exactly two fail; **50 retain positive original evidence**. All **373** captured source input identities and the ordered source-derived commands match the immutable subject. Both failed commands returned **cargo 101, capture 0**, with no capture execution error. These are runtime test failures, not compiler failures or an inferred environment failure.

## Classification and repair boundary

| Group | Actual failure | Classification |
|---|---|---|
| plugin-uninstall-service | `desktop_service_contracts::desktop_refresh_and_sync_wait_for_the_actual_loader_and_status_reads_its_cache`, exact status assertion at `tests/mcp_coordinator_relay_contract.rs:889` | **TEST_FLAW** |
| plugin-uninstall-state | `extensions::mcp::plugin_skills::sync_tests::poisoned_admission_fails_closed_for_retained_and_successor_readers_without_resetting_truth`, successor unwrap at `plugin_skills_sync_tests.rs:1756` | **IMPL_BUG** |

The service test expects only `authBlocked`, while the actual status additionally contains `authRecovery: {refusals: [{pluginId: "41", pluginName: "Calendar"}], refreshOutcomeUnknown: false}`. Immutable specification **AR-08** explicitly requires that projection when refusal facts exist; `production.rs:297–305` and `plugin_skills.rs:819–836` implement it. Keep the exact-object assertion and include the required field. Retain all loader synchronization, no-extra-request, failed-loader and cache-preservation controls. The target records **15 passed / 1 failed**. Its nine other command targets completed; a separate **23 passed / 1 ignored** target must retain its ignored classification. Assertions after the failed status assertion were not reached.

The poisoned-admission test deliberately poisons the private admission Mutex. Earlier assertions establish that existing readers deny all, cached truth remains intact, and refresh/uninstall fail closed. `successor_with_loader` must then carry the same private Arc into a new lifecycle owner, under **PS-LIVE-07**, **PS-LIVE-09**, **PR-07**, and retained-control requirements. The new `retire_for_replacement` implementation sets `disposed = true` and `replacement_claimed = true`, then calls `fence_owned_refresh`; that helper returns the admission-poison error. The predecessor is consumed and no successor can be constructed. This is an implementation regression against the retained lifecycle contract, with **91 passed / 1 failed / 624 filtered** in the target.

The minimal repair boundary is the admission Mutex poison branch of refresh fencing: the shared poisoned admission already permanently denies all readers. Permit the one-shot lifecycle handoff while retaining that same poisoned Arc, with no reset, reconstruction, poison clearing, disk I/O or admission widening. Preserve healthy exact-owner refresh fencing. Handle that poison case explicitly rather than swallowing a whole fallible helper result or hypothetical future errors. Keep the original successor unwrap, pointer equality, denial and cache-byte assertions; changing the test to expect an error would remove the required behavior.

## Evidence and limits

The [machine proof](d70-portable-failure-diagnosis.json) records exact commands, exits, every raw summary, failure excerpts, source Git blobs/SHA-256 values and relevant specification excerpts. [The complete portable audit](d70-portable-audit.json) retains the 50 passing groups, required-name occurrences and all 373 source bindings. The original target log hashes are `ea16bf0c3bb6756d16189ee7639bd0f2dee423b5daf8af715f74e6b271a31a4e` (service) and `2253b044dc51ffcab26b2d2796bdc7fc25d1fdcce9fb9d513a9d5fa4cdfdc6c4` (state). Artifact SHA-256 is `37a64bad37cedad67277ceb7a71ffe40b47aa30bc38a778c708fddd7e635d005`.

Automation step 18, actual App step 19 and PRID step 20 were **skipped** after the portable failure. Neither the source-authored App 100/203 selection nor the new PRID consumer archive has execution credit. No product code was run locally, edited or rerun during this diagnosis. All global completion flags remain false. Future fixes require their own exact-H evidence; this report preserves the original d70 failures.
