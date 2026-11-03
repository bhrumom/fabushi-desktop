# Plugin Skills denial after failed snapshot publication — source assessment

## Verdict

**No Rust change is needed for the selected immediate live-admission contract.** At the pinned source, an active `SandPluginSkillsService` accepts a fully validated canonical `auth_blocked` fact into its private shared admission state **before** cache construction, staging, rename, synchronization or strict readback can fail. Subsequent admission through the actual FileWorkflowStore, Session commands, Runner state and next provider-skill prompt uses that live state. A failed publication cannot restore the older eligible skill. Same-Host, same-root MCP stop/start also retains that state.

This conclusion is source proof, not a new executable-validation or acceptance result. It does not extend to already captured records/prompts or in-flight effects. **A true process-loss boundary remains different:** an authentication refusal whose cache publication never completed is not restored by a fresh service. That existing limitation is substantiated below, separately from the live contract that is already implemented.

## Scope, authority and method

- Exact source base: `fdb0f6acb06165b6f2796961186e06a2de98c229` in `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Immutable comparison tree: `77fa185818b171a5b3829fc54104a0beec9ad8b4`. A read-only `git diff --raw --no-renames` between the base and this tree returned no changes for the fourteen Rust production files in the owner/cache/relay, workflow, Session, transcript, memory/Runner, Host composition, prompt-builder and shipping-main chain listed in the manifest below. The current mutable checkout HEAD was not used as authority.
- Read root `AGENTS.md`; OCRM main Spec §17; the applicable snapshot, live-admission, reporting, uninstall and durable-custody sections of `plugin-skills-snapshot-admission.md`; and bounded source-of-truth/module-map context. The historical current-task search did not establish a newer behavior requirement; actual source and the later applicable amendments decide this assessment.
- Selected effort: **xhigh**, because the assignment crosses permission, failed persistence, consumer and lifecycle boundaries. Execution retained the configured session model/effort; no separate override was made or independently verified.
- All source and Git reads used `GIT_NO_LAZY_FETCH=1`. Shipping `main.rs` blob `4831ae8cf002ce64e6409f2d695b48aaa1914c45` was initially absent; the parent supplied that exact blob, after which it was read through the pinned Git revision. Other missing blobs encountered in broad navigation were unrelated to this bounded trace; no absence or full-tree coverage claim relies on those searches.
- No build, test, lint, parser, generator, validator or runtime execution occurred, locally or on htch-runtime. No product/test/spec/task/ledger file was edited. No current plugin review report was read.

## Historical facts versus current evidence

The spec's initial problem statement and its lines 248–262 describe the earlier **committed-snapshot-only** batch. The note that a failed denial publication can leave an old eligible cache describes that historical boundary, not the current implementation.

The later `OCRM-PLUGIN-LIVE-ADMISSION-v1` amendment at lines 273–554 explicitly adds the missing owner-to-consumer path: import before fallible publication, retain refusal across failed writes and disposal, share one read-only reader, wire actual Session/Runner consumers, and carry the same reader across same-root successor construction. Those responsibilities now have concrete implementations in the pinned Rust blobs. PS-LIVE-REPORT-01 at lines 591–617 also has its positive-release reporting implementation; it should not be reopened from the earlier historical note.

The later durable uninstall amendment is another distinct change. Its PR-02 at lines 881–887 expressly excludes authentication refusals from `uninstall-state.json`. It is therefore not evidence that failed-publication authentication denials survive a full process restart.

## Actual producer → owner → state/storage → consumer chain

### 1. Canonical denial reaches the real Rust owner

At this base, `source/electron-main/mcp/host-plugin-skills-loader.ts::authBlocksFromFailures` (102–135) requires the SDK clone-error branch, `classifyCloneError(...) === "user_git_access"`, and a valid exact `pluginDbId`. It rejects a qualifying auth failure without that canonical ID rather than substituting a display name or slug. `createHostPluginSkillsLoader` captures raw authoritative enabled IDs (137–185, 229–252), requests SDK materialization with precommit pruning disabled (262–280), verifies the account context before returning (286–302), and emits the result's `authBlocked` together with `listedPluginIds` (307–333).

The shipping Electron adapter constructs that loader with `getSandRootDir()` and the account-owned credentials (`adapters/mcp-oauth.ts`, 321–333), then exposes it as `loadHostPluginSkills` (371–374). `coordinator/production-provider.ts:432` binds that method to the production executors; `coordinator-executors.ts:500–503` invokes it without manufacturing a substitute result. The Coordinator lifecycle bridge permits `loadHostPluginSkills` and returns the actual executor result (`source/node-agent-coordinator/src/mcp_lifecycle_relay.rs`, 48–66, 103–118).

Rust `CoordinatorPluginSkillsLoader::load` requests precisely that method and deserializes `LoadedPlugins` (`extensions/mcp/coordinator_relay.rs`, 75–93). `ProductionHostExtensions::start_mcp` constructs this loader and the real `SandPluginSkillsService` (`host_production_extensions.rs`, 1333–1362). The service's standalone `plugin_auth_blocks_from_failures` helper is **not** the production RPC producer: the actual Host path receives `LoadedPlugins.auth_blocked` from the Electron edge.

### 2. The live refusal is ordered before the fallible snapshot write

`plugin_skills.rs::run_pass` (988–1017) first validates the complete `LoadedPlugins`. `validate_loaded_plugins` (138–175) rejects malformed/duplicate materialized/listed identities and blocks outside the authoritative listed set. Only then does `run_pass` call `import_live_refusals_if_active`.

`import_live_refusals_if_active` (752–790) takes the existing publication lock, refuses a disposed owner, checks the current uninstall-observation identity and recovery state, and inserts each block into `PluginSkillsAdmissionState.denied` (782–784). It returns a copy for reconciliation; consumers retain the stable owner reader, not that copy. The import is already complete before `stage_plugin_skills_cache` at 1081–1091.

The private reader's `is_denied` implementation (392–403) checks this map in addition to uninstall custody/fault state. A poisoned admission lock denies admission. `current()` reads committed eligibility and then filters through this live reader (688–719). `execute_pass`, completed coalesced results and disposed sync returns also filter through it (898–971). There is no error branch that clears an accepted auth refusal.

The constructed snapshot reconciles prior committed blocks **and retained live blocks** (1042–1059), excludes denied records and denied fallback (1060–1075), then stages it. A later unresolved/materialization failure therefore cannot republish the old skill merely because the old disk image lacked the denial.

### 3. Successful publication is the only positive-release boundary

`publish_snapshot_if_active` (792–855) holds the existing publication interval, checks that the owner and observation remain current, publishes the owned stage, and requires strict valid readback. Only afterward does it consider positive release. It removes `live.denied[id]` solely for a freshly materialized canonical ID absent from that pass's denial set (844–847). Current denial wins over simultaneous success. Fresh success with zero skills counts as materialization; absence or unresolved fallback does not count.

`plugin_skills_cache.rs::stage_plugin_skills_cache` (300–358) validates the generated cache through the consumer parser and performs owned unique-temp creation, writes, permissions and file synchronization. `StagedPluginSkillsCache::publish` (270–277) renames, then synchronizes the parent directory where supported. Its errors propagate before positive release. Strict readback failure also propagates before positive release (`plugin_skills.rs:820–821`).

Thus a stage or rename failure can leave the old allowed cache intact while live admission correctly refuses it. A failure after rename may leave new bytes visible; it does **not** promise restoration of old bytes. Neither case releases the live refusal or reaches this pass's success report/pruning. If the later durable-uninstall release itself fails, the same service latches a recovery fault and fails closed (583–610, 825–835), before authentication-refusal release.

### 4. Actual consumers use the same reader on every admission

| Consumer | Current production source and effect |
| --- | --- |
| FileWorkflowStore | `workflow_store.rs:149–160` attaches the stable reader. `plugin_skill_to_workflow` checks it before reading facts **and after a possible warm parse-cache hit** (339–377). Both list and get go through that function (380–463). Plugin update rechecks committed eligibility, current publisher identity and the live denial before writing (577–599). User/managed workflows follow their existing paths. |
| Session stores, including prebinding instances | `extensions/session/production.rs:235–251` holds a forwarding slot and releases the slot lock before querying its delegate. `open_workflow_store` gives every store the same forwarding Arc (1494–1500); binding is idempotent only for the identical owner reader (1503–1517). A store opened before first MCP binding sees that later binding. |
| Session list/get/update | `agent_session.rs:1037–1050` opens the production store. Upsert/update uses that same store (`1096–1128`, `1157–1164`). Surface listing is `store.list_all()` (`1374–1386`). |
| Run-now and rich-text reference expansion | `workflow_commands.rs::prepare_workflow_run_now` calls `get_agent_workflow` (149–161); `expand_workflow_references` does the same for each referenced ID (192–223). A denied record produces no recipe body. |
| Retained Runner state | `HostRunnerComposition::compose_turn_state_surfaces` obtains the Session-opened store and directly passes it to `SandAgentState::new_with_workflow_store` (`host_runner_composition.rs:130–141`). That constructor retains the supplied store (`agent_state.rs:120–145`); `workflow_records` calls its live list (`262–267`). Production workflow mutations delegate back to Session through the bound mutator (`host_runner_composition.rs:225–239`, `agent_state.rs:346–350`). |
| Shipping next provider-skill prompt | `source/host/app/src/main.rs:9774–9799` opens `session_workers.open_workflow_store`, calls `list`, builds skill path/description items and passes them to the real `append_budgeted_workflows_system_prompt`. The builder renders only those supplied items (`system_prompt_assembly.rs:475–486`). The shipping turn also calls the real Runner composition (`main.rs:10352–10359`). |

This is a guarantee for admissions after the owner has accepted the fact. A workflow record or provider message captured before that ordering point is not retroactively invalidated. File retention and diagnostic `current_index`/`current_auth_blocked`/status are not admission grants; the status projection still reads the committed denial list (`plugin_skills.rs:688–703`, `extensions/mcp/production.rs:297–302`).

### 5. Same-Host stop/start and actual root binding retain the protection

Shipping main derives the MCP root from `session_workers.agents_root().parent()` and fails explicitly if absent (`main.rs:16588–16605`). It does not select the unrelated global app-data cache.

Host binding serializes Session registration and service preparation. Session registration binds an already current service only after matching roots (`host_production_extensions.rs:479–551`); `start_session` invokes it before publishing the Session extension (`1096–1117`). A same-root replacement calls `successor_with_loader`; this permanently retires the predecessor through the publication/state boundary and clones the **same admission Arc** (`plugin_skills.rs:642–672`). Host keeps the new Arc before later fallible Session binding (`host_production_extensions.rs:544–549`). `stop_mcp` takes only the runtime, leaving the binding's latest Arc retained (`1437–1443`).

Disposed/retired work cannot import a new denial or publish/release a staged success (`plugin_skills.rs:761–768`, `806–816`). Old and prebinding Session stores continue querying the same state. The predecessor's delayed dispose changes only its own lifecycle state; it does not clear the shared denial map.

## Concrete scenario and existing unexecuted source oracles

The originally reported scenario is already handled:

1. Commit allowed plugin 9 and unrelated plugin 10; retain both materialized files and an existing/warm workflow store.
2. The real loader returns a valid authoritative aggregate with canonical denial 9.
3. The service imports denial 9. An unrelated generated record then fails strict staging, or the owned stage is removed before the real rename.
4. Sync returns an error; the old cache/skill bytes remain for the pre-rename cases. `current`, retained/new stores, Session list/get/update, run-now, reference expansion and the next provider prompt exclude 9 while controls remain.
5. Loader failure, unresolved partial loading, failed fresh-success publication or disposal does not reopen 9. A current successful materialization can reopen it only after valid commit/readback.

Existing source already expresses these causal oracles. None was executed in this assessment:

- `plugin_skills_sync_tests.rs::live_refusal_survives_invalid_stage_failed_release_and_partial_until_valid_fresh_success` (1384–1466).
- `new_refusal_is_live_before_rename_and_failed_publication_preserves_old_allowed_bytes` (1469–1503).
- `missing_or_malformed_readback_never_releases_a_live_refusal` (1529–1578), including restoration of the old allowed bytes to distinguish live refusal from malformed-cache rejection.
- `workflow_store_contract.rs::real_stage_failure_keeps_live_refusal_in_old_and_new_workflow_stores` and `real_rename_failure_keeps_live_refusal_in_old_and_new_workflow_stores` (594–675); the warm-cache control at 686 onward proves an actual stale parse-cache hit before introducing denial.
- `workflow_commands_contract.rs::failed_refusal_publication_reaches_prebound_store_commands_and_actual_provider_prompt` (476–580) exercises actual ProductionSessionWorkers, Session commands, run-now/reference recipe bodies and the real ProviderMessage builder, with allowed plugin/user/managed controls and a later valid release.

No new duplicate implementation or test is recommended merely to reopen the historical live-admission note.

## Separately bounded current limitation: authentication refusal across full process loss

**This is not the same-Host replacement case and is not evidence against the live result above.** It is a current, source-proven limitation already acknowledged by the live amendment's remaining-scope paragraph (spec 534–543).

`PluginSkillsAdmissionState.denied` is an in-memory BTreeMap (`plugin_skills.rs:382–390`). Fresh `SandPluginSkillsService::new` loads only `load_uninstall_image` and uses the admission state's default for all other fields (462–481). The durable image has only confirmed uninstall IDs and pending uninstall tickets (`plugin_skills_uninstall_state.rs:26–54`); no auth-denial field exists. A missing image plus missing marker is healthy legacy/default state (145–158). The durable-custody spec explicitly prohibits using that image to persist an authentication refusal.

**Real trigger:** seed a valid allowed canonical snapshot; accept denial 9; fail staging/rename before that denial reaches `cache.json`; terminate the process; start a fresh Host/service against the unchanged cache with no uninstall custody/fault; keep the new refresh unavailable. The fresh live map is empty, and the old committed snapshot still considers 9 eligible. The new Session forwarding binding attaches that empty reader. `current()` and the actual workflow consumers can again admit 9. This follows directly from construction/default, committed eligibility and the failed-load branch; no execution result is claimed.

**If the parent selects durable auth-denial follow-through as the next batch**, the unique existing owner is still **SandPluginSkillsService and its private admission/publication persistence**, with the existing Session/workflow reader as consumer. No new registry, lifecycle owner, copied Session denial table or uninstall receipt should be introduced. The parent must first define the auth-denial durability/release/account-scope contract; PR-02's existing journal cannot silently be broadened in this assessment.

A falsifiable follow-up oracle would run the above observation/failure in one owned process, exit without a successful replacement snapshot, then use a genuinely fresh process with the actual production binding and Session-opened workflow consumers. The expected new contract would require 9 absent from list/get/update, run-now/reference bodies and the actual provider prompt, while plugin 10/user/managed controls remain. A later fresh authorized commit would be the positive release control. Reopening a store or same-root successor in the original process is not that oracle.

No implementation, schema, workflow, ledger or task change is requested by this report. The parent's scope/spec decision is required before turning that distinct remaining limitation into an implementation batch.

## Exact source blob manifest

All identities below are from `git ls-tree -r fdb0f6acb06165b6f2796961186e06a2de98c229 -- <paths>`; cited line numbers are in those blobs. Fourteen production Rust paths from Host relay through shipping main were additionally compared to tree `77fa185818b171a5b3829fc54104a0beec9ad8b4` with an empty raw diff.

| Path | Blob |
| --- | --- |
| `AGENTS.md` | `e4682748087759b2d4bb2b3a47916b6148e8a6f2` |
| `docs/specs/openclaw-rust-equivalence-migration.md` | `69c9667af2328e20a1fba09ebf80c5aab4d5c336` |
| `projects/openclaw-rust/docs/specs/plugin-skills-snapshot-admission.md` | `cfd735257b6ba538312dbd408c1133da24d800ed` |
| `source/electron-main/mcp/host-plugin-skills-loader.ts` | `ee5773e80ccb3d3c0396566f3cc11a3669142079` |
| `source/packages/cursor-plugins/loader.ts` | `9ea7d13b479d5a8ba2f13a3ddf8c5122d97a2bfe` |
| `source/electron-main/adapters/mcp-oauth.ts` | `6653252d5f12ad3e345ee9104dd58c93734594df` |
| `source/electron-main/coordinator/production-provider.ts` | `8436339d3e2bfdf05f1001f284dfadee72ecf8ef` |
| `source/electron-main/coordinator/coordinator-executors.ts` | `0e12efd790bdfb60bb54d07048c80547c3581080` |
| `source/node-agent-coordinator/src/mcp_lifecycle_relay.rs` | `e6925d0c49a1ff9ec8a482278cd6c2c85a884a9f` |
| `source/host/src/extensions/mcp/coordinator_relay.rs` | `4190c0bab004bf6c5191cef489e77b56d81cedb9` |
| `source/host/src/extensions/mcp/plugin_skills.rs` | `ec053491e5b2ddaaa49caf24c8aa1d0345c9582c` |
| `source/host/src/extensions/mcp/plugin_skills_cache.rs` | `f395533056fd0203a68a51c51a412ff835925001` |
| `source/host/src/extensions/mcp/plugin_skills_uninstall_state.rs` | `4b158b4ad3ac1987eedfefe68c8223fcba7ef31c` |
| `source/host/src/extensions/mcp/production.rs` | `4c67df65f95a2f19e67b93ae8450e18a114ed3ea` |
| `source/host/src/workflows/workflow_store.rs` | `048f7948db3d31d1a057902f956b3f1db791dad7` |
| `source/host/src/extensions/session/production.rs` | `f585fae13720e08f34323bed82759253590a3753` |
| `source/host/src/extensions/session/agent_session.rs` | `0a54da8349bc3edc9ef21c5f1c1862e13a9e3039` |
| `source/host/src/extensions/transcript/workflow_commands.rs` | `35d5863225b4de40e6f321873f33fba3e7a2e43d` |
| `source/host/src/extensions/memory/agent_state.rs` | `5571649fa666753c3732e235523029027852cdd7` |
| `source/host/src/host_production_extensions.rs` | `08bf6b21cd9962dfee141ebee1ae61236cd5a7e7` |
| `source/host/src/host_runner_composition.rs` | `b75c24202ef15600a21e25180947a1432f44061c` |
| `source/host/src/runner/system_prompt_assembly.rs` | `6d592794b835994a81a214049b9c5907df0cb4ff` |
| `source/host/app/src/main.rs` | `4831ae8cf002ce64e6409f2d695b48aaa1914c45` |
| `source/host/src/extensions/mcp/plugin_skills_sync_tests.rs` | `29be98102d3bf4b86a841b279fc390acff4efcc3` |
| `source/host/tests/workflow_store_contract.rs` | `f10bd1d5ed45a65087e90a04a0a514d342cf0e00` |
| `source/host/tests/workflow_commands_contract.rs` | `345c73cc856f8ad925c204805afa31f6419f96ad` |

---

## Current-state addendum — `bb5a7f6098f952df6d388a672c4fb01045bc645c`

### Updated conclusion and historical boundary

**The incoming source addresses the separate authentication-refusal restart scenario identified above. Do not use the preceding fdb assessment as a current-gap claim against bb5.** The preceding analysis remains an exact-source historical assessment of `fdb0f6acb06165b6f2796961186e06a2de98c229`; its immediate live-admission conclusion remains applicable, while its auth-recovery absence is superseded by this incoming implementation.

This addendum is a bounded compatibility/source assessment, **not a new full blind review, execution result or acceptance verdict**. Independent review and exact-H Actions remain pending for purposes of this assessment. No existing owner's implementation is duplicated or edited.

The GitHub exact-SHA compare reports bb5 one commit ahead of parent/merge-base `23c2a567d874509f56353b8cae737cdf1437309d`, with fifteen changed files. The newly appended spec section is `OCRM-PLUGIN-AUTH-RECOVERY-v1`, starting at line 1052, in blob `5fd897559799de1ffde183c64cac60f2a386b985`. Its AR-01–AR-09 contract now explicitly extends the existing private image to auth refusals and refresh custody, superseding the earlier PR-02 exclusion for this new version.

The bb5 commit object was absent locally and no Git network fetch was attempted. Exact-SHA GitHub `fetch_file` reads supplied the actual source and blob identities below; local parent reads retained `GIT_NO_LAZY_FETCH=1`. The API compare metadata was inspected only to bind the change set. Incoming review-decision/report contents, capture scripts and workflow/test changes were not evaluated. No parser, test, build, lint, generator, validator or runtime was executed. This scratch addendum is the only new file change from this follow-up.

### How the actual source changes the original scenario

The relevant service implementation is `source/host/src/extensions/mcp/plugin_skills.rs`, blob `114898d12993ae7957175d82c43a54157fa4dbdc`.

1. **Reserve before entering the real loader.** `run_pass` now calls `begin_refresh` before `self.loader.load()` (1254 onward). `begin_refresh` (687–732) holds the current owner's publication interval, rejects disposed/unsafe state, clones existing durable state and retained live auth refusals, assigns a unique refresh UUID, and persists `refresh_intent` before returning the token. A failed reservation exits before loader entry. Queued callers only reserve when they become an actual worker through the existing sync loop.
2. **Record the accepted denial before content publication.** Complete `LoadedPlugins` validation still precedes `import_live_refusals_if_active` (983–1037). Import checks current owner/token and uninstall observation, adds the canonical block to live `denied`, marks the result known, merges retained refusals into the durable image, and persists it before cache construction/staging. A normal healthy pass clears its exact refresh intent in this same negative transition; inherited unknown custody remains until successful recovery. An accepted auth-write error is not relabeled as an unadmitted load error: `settle_unadmitted_refresh` explicitly preserves `KnownResult` custody.
3. **Restore the denial in a genuinely fresh service.** `new` (499–521) now copies `durable.state.auth_refusals` into `denied`, and sets `recovery_required` if a stored refresh intent exists. The shared reader's `is_denied` (426–439) checks recovered auth facts and the unknown/recovery condition in addition to existing uninstall state and integrity faults. This is the existing reader interface; no copied Session policy is introduced.
4. **Keep the old allowed snapshot from reopening unresolved work.** A fresh owner that sees an unresolved intent denies plugin admission. `settle_unadmitted_refresh` (774–817) does not clear inherited `recovery_required` on an offline error or invalid aggregate. A recovery pass suppresses all prior-cache fallback (`run_pass`, 1347), so publishing a partial fresh result cannot restore unresolved old rows. User/managed workflows remain outside plugin admission.
5. **Release only after the corresponding cache commits.** `publish_snapshot_if_active` (1039–1121) keeps cache rename/barriers and strict readback before durable positive release. Only fresh exact IDs without a current denial are removed from `auth_refusals`; pending uninstall custody remains. An unknown recovery clears only its exact intent after that cache boundary. Live auth refusals and `recovery_required` are cleared only after the necessary image write succeeds. The existing changed-report path includes the real release/restoration.

For the report's original trigger—allowed 9/10, accepted canonical auth denial 9, then cache-stage/rename failure—the normal negative-image transition now completes before that cache failure. A fresh owner loads auth refusal 9 with no inherited unknown, so the existing consumers refuse 9 while unrelated allowed 10 remains available. This is direct source ordering and restored-state proof; it is not inferred from the word “recovery” or a new test name.

### Persistence-failure boundaries remain explicit

The actual private image implementation is `plugin_skills_uninstall_state.rs`, blob `0c2a4f7fd11c6b47dc9941b659ad8eac89200c9a`.

- The image state now includes `auth_refusals` and `refresh_intent` (29–34). The v2 DTO uses strict object types for auth entries and required nullable refresh custody (55–116). Decoding still accepts valid v1 as empty auth/no intent; encoding writes version 2 (152–258). The marker remains `1\n`, and construction calls the read-only loader rather than rewriting either format (262–274).
- Auth validation reuses `is_canonical_plugin_id`, preserving exact opaque IDs and the distinction between `09` and `9`; uninstall validation retains its separate stricter predicate (198–225). The auth write does not fabricate names or marketplace metadata: stored entries originate from `PluginAuthBlock`.
- The existing image publication path retains create-new staging, file synchronization, rename, directory synchronization where supported, marker ordering and exact-byte readback (`persist_uninstall_image`, 279–311; `StagedUninstallFile`, 432 onward). It does not establish concurrent cross-process writers or hardware/platform durability acceptance.
- `persist_auth_transition` (`plugin_skills.rs:658–685`) retains adverse live facts on errors and calls `inspect_failed_auth_publication`. That helper (`plugin_skills_uninstall_state.rs:317–341`) accepts only the exact expected image or this operation's exactly encoded candidate, including its allowed marker intermediate. Foreign bytes, corruption or unreadability latch the existing full recovery fault.
- If the accepted negative write fails before its candidate replaces the earlier image, the **prior durable refresh intent** still makes fresh startup conservatively unknown. Live admission can remain precise for 9 while a fresh process blocks all plugins. This is the AR-06 availability distinction, not a silent loss of denial.
- An error after a positive image has become visible is not proof that the old denial remains on disk. The live call returns an error and retains its refusal, but a fresh owner can safely accept that completed positive image because the corresponding strict cache already crossed its publication boundary. The new contract correctly does not claim “every release error preserves refusal across restart.”

These paths support the specific restart scenario. They do not establish every fault-injection, platform-barrier or lifecycle guarantee without the required review and Actions evidence.

### Token lifetime and existing production composition

`LiveRefreshCustody` binds each in-process refresh to both `refresh_owner: Arc<()>` and an operation ID. `owns_refresh`, `fence_owned_refresh` and `finish_refresh_guard` (734–772) prevent an old guard from updating a successor's token and preserve unknown custody after an awaiting-result panic. `dispose` and one-shot retirement drop service-state before fencing their own token under P (880–900, 936–945), without doing journal I/O under Host binding. Same-root successor construction still inherits the exact shared admission Arc (868–878).

The actual Host preparation/binding path remains `prepare_plugin_skills_service` and `PluginSkillsBinding::prepare_service` in blob `36b5dbac27d346f883150df179372034e0bea223`, read at 520–581: fresh recovery is prepared outside the binding lock; the current root/owner is rechecked; a same-root predecessor supplies the existing shared reader; the current service is retained before later binding can fail. `PluginSkillsPort::status` in production blob `fb672e8c3b9473607438a0db4fc733e205d5c59b` adds optional `authRecovery` while retaining `authBlocked` and `uninstallRecovery` (297–305).

The compare contains no changes to `plugin_skills_cache.rs`, FileWorkflowStore, Session production/commands, Runner composition or shipping main. Local `git ls-tree` at parent 23c2 confirms those relevant blobs are the same ones traced above. Their existing live-reader consumer chain therefore consumes the recovered state without an additional interface or Session writer.

### SDK/Host category compatibility: no concrete mismatch found

Both actual incoming TypeScript blobs match the exact parent 23c2 blob identities:

| Boundary | Exact unchanged blob | Observed compatibility |
| --- | --- | --- |
| `source/packages/cursor-plugins/loader.ts` | `f0028c8fbecc602828fb6ed48dce9fd77855102c` | `classifyLoadError` (34–53) keeps ordinary cache-I/O wrappers opaque, handles cleanup AggregateError through the primary cause, and gives local `EACCES`/`EPERM`/`EROFS` install classification. `loadFromMarketplaceSource` retains canonical `pluginDbId` and the typed `errorType` when collecting a failure. |
| `source/electron-main/mcp/host-plugin-skills-loader.ts` | `d81e1da13c9b39f4e2943ab48e8b9073bb8dc987` | `authBlocksFromFailures` still requires `errorType === "clone"` and `classifyCloneError(errorMessage) === "user_git_access"`, then validates the canonical `pluginDbId` (102–130). No local I/O category becomes an auth fact merely because the Rust owner can now persist facts. |

The incoming Rust `LoadedPlugins` wire fields remain `plugins`, `authBlocked`, `listedPluginIds`, `listedCacheKeys`, `publisherFacts` and `currentUserId` through the same camelCase deserialization (`plugin_skills.rs:87–96`). `CoordinatorPluginSkillsLoader::load` still requests `loadHostPluginSkills` and deserializes the resulting aggregate; its actual production body is unchanged (75–93 in blob `d3eb405218a8bb2a86875b404a6719ecb95de8bd`). The new journal consumes validated `PluginAuthBlock` values rather than reinterpreting SDK error prose or changing the relay payload.

**Integration disposition:** no concrete mismatch found between the unchanged SDK/Host category path and the incoming reserve/import/publish/recovery interfaces. No duplicate implementation or product edit is called for by this assessment. The auth-recovery implementation's full state machine, source oracles, compilation and exact-H Actions must retain their independent review/verification gates.

### Addendum exact identities

| Item | Identity |
| --- | --- |
| Incoming commit | `bb5a7f6098f952df6d388a672c4fb01045bc645c` |
| Exact parent/compare base | `23c2a567d874509f56353b8cae737cdf1437309d` |
| Auth recovery spec | `5fd897559799de1ffde183c64cac60f2a386b985` |
| `plugin_skills.rs` | `114898d12993ae7957175d82c43a54157fa4dbdc` |
| `plugin_skills_uninstall_state.rs` | `0c2a4f7fd11c6b47dc9941b659ad8eac89200c9a` |
| Host `coordinator_relay.rs` | `d3eb405218a8bb2a86875b404a6719ecb95de8bd` |
| MCP `production.rs` | `fb672e8c3b9473607438a0db4fc733e205d5c59b` |
| `host_production_extensions.rs` | `36b5dbac27d346f883150df179372034e0bea223` |
| Unchanged SDK `loader.ts` | `f0028c8fbecc602828fb6ed48dce9fd77855102c` |
| Unchanged `host-plugin-skills-loader.ts` | `d81e1da13c9b39f4e2943ab48e8b9073bb8dc987` |

The earlier fdb blob manifest is preserved for historical reproducibility and is not relabeled as bb5 evidence. Selected effort remains xhigh for permission/persistence/lifetime assessment; actual configured model/effort remains inherited. The only follow-up output is this separate addendum.
