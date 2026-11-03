# GENERAL — PS-LOCK-RECONCILE-v2 recovery R4

Verdict: **APPROVED — frozen source review only**

Task: `647e0932-bea4-4113-9a61-65ae3db58955`; recovery review R4, reviewer round 1.  
Reviewer configuration supplied by the actual spawn: `gpt-6-astra / xhigh`. This is the recorded applied spawn configuration, not a claim of backend introspection. Scope warrants xhigh because permission admission, durable operation custody, owner retirement, process exclusion and test-oracle causality interact.

## Identity and independence

Independently fetched the public repository API and confirmed `bhrumom/fabushi-desktop`, repository ID `1375312893`, organization ID `281146136`, and `permissions.push=true`. Independently fetched the frozen Git commit and confirmed:

- Subject: `d29be0b93225943851c6054016686d571f8d9e2a`.
- Tree: `16a9b18517874fa7d82e562729bbe4782bd99d79`.
- Original scope base: `9864d1c02e7e67881d644800963abc807b3abaab`.
- Focused-delta base: `b8a648abd90b6ae5c4f9feae8a1e159f126fd69b`.
- Fixed upstream identity remains `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`; this review does not claim a new full upstream read.

The packet scope blob is `0569f7bf2fb1c3767b4813d46edcb66a811d3923`; full original-scope diff blob is `fec62e54326254def9301405dc869ba7ea77f551`; focused delta is `6cf3ab16fc6f2576e4395f0ad86e5745279ebace`. All sixteen source paths were fetched by the fixed subject, and returned identities match the frozen manifest. The large parity ledger fetch returned its identity with empty text; its complete changed hunks were read from the immutable full and focused diffs, rather than treating that empty body as a full-file read.

Read root AGENTS and confirmed the recursive tree has no applicable nested AGENTS for these paths. Read the applicable Plugin Skills Spec, particularly LR2-01..04 and AR-LOCK-R3, plus the source-contract requirements and relevant OCRM source-of-truth/module/§17 scope rules. Prior review reports, other reviewer outputs and runner-decision documents were not fetched. Historical verdict text embedded in canonical Spec/contract metadata was not used to derive this verdict.

## MUST_FIX

None identified.

## NON_BLOCKING

No new actionable non-blocking finding. Existing limitations remain limitations; they are not additional recovery requests.

## Why the focused repair is sound

### Complete v3 observation oracle, not a copied-field equality

At `source/host/tests/mcp_coordinator_relay_contract.rs:644` and `:719`, `UninstallCustodySnapshot` and `RealService::assert_uninstall_custody_with_refresh` retain complete status, exact image bytes and marker bytes. The helper requires an explicitly present nullable `uninstallObservation`, accepts only null or a canonical UUID string, requires a non-null observation when this real fixture has confirmed/pending uninstall custody, and still compares the entire decoded image against the expected v3 object. It keeps the unaffected admission control.

Reading a dynamic UUID for the expected object is accompanied by independent lifecycle assertions:

- `RealService::seed` at line 762 pins null after the first real refresh.
- `desktop_observation_rejects_a_pre_fact_success_from_the_actual_loader` at line 940 pins null during refresh-only reservation, a present observation at the real uninstall reservation, rotation on exact settlement, unchanged observation when stale refresh custody settles, unchanged observation on a fresh refresh reservation, and a new non-null observation after confirmed-fact release.
- The same test retains the stale-success error, cache and skill bytes, denied current/read projection, pending relay counts, absence of unintended cleanup refresh, and actual later successful materialization.
- `late_desktop_uninstall_reply_cannot_settle_the_successor_service_request` at line 1014 compares independent real reservation identities, keeps both relay incarnation/correlation controls, proves stale positive and error replies leave the current waiter pending, and compares complete custody snapshots and cache/skill bytes before the matching current settlement. Only that settlement rotates its observation.
- `closed_desktop_service_never_replays_an_admitted_uninstall_or_dispatches_new_work` at line 1097 retains full snapshot equality after closure, both late reply variants and all four closed desktop commands. Its snapshot now also carries the observation. Pending custody is not converted into a confirmed fact or replay permission.

The two-fixture late-reply integration test is a distinct-owner correlation control. Same-root handoff is proved separately by the existing library lifecycle test; this review does not conflate them.

### Actual same-root reader and late settlement are preserved

At `source/host/src/extensions/mcp/coordinator_relay.rs:879`, `desktop_confirmation_keeps_the_actual_reader_across_same_root_handoff` uses real service/backend/relay, real durable files and a Session-opened workflow store. The new assertions inspect the complete confirmed v3 image, validate its canonical observation, then verify a distinct real pending reservation rotates that observation.

The retained assertions still require identical reader Arc across `successor_with_loader`, denied 41 and pending 43 through retained and reopened stores, available unrelated 42, equal recovery status, a late settlement error from the retired predecessor, exact journal-byte retention, no extra relay work, refusal of a new old-owner mutation and unchanged cache/all skill bytes. The change remains wholly inside `cfg(test)` lifecycle tests.

### Production causality matches the revised expectations

Read `plugin_skills.rs` production state machine and `plugin_skills_uninstall_state.rs` production schema/persistence implementation, rather than deriving behavior from the tests:

- `begin_uninstall` and exact `settle_uninstall` reach `persist_uninstall_transition_locked`, which generates a fresh canonical identity and advances the live opaque observation after successful durable publication.
- Auth/refresh transitions retain the existing durable uninstall identity. Confirmed-fact release uses the uninstall transition only after strict cache publication/readback; ordinary auth release uses the auth transition.
- `StoredUninstallObservation` is a required nullable custom-deserialized field. v3 emission is explicit; strict v1/v2 read compatibility, duplicate/unknown-field rejection, canonical UUID checks and accepted non-null identity loss refusal remain.
- Actual refresh captures both owner/token custody and uninstall state before the loader. Import, publication/release and cleanup compare the relevant local/durable observations. Matching final confirmed/pending sets alone cannot authorize a stale pass.
- Same-root retirement preserves the admission Arc and fences only the old owner's exact refresh custody. Late/retired uninstall operations fail before state mutation. No old guard or repeated disposal acquires authority over a successor token.
- `McpHostService::uninstall_plugin_with_projection` persists pending before backend invocation, settles the actual receipt before desktop projection, and retains unknown custody on backend errors. The malformed display fixture therefore isolates real confirmation without accidentally invoking the ordinary follow-up refresh.
- The real `PluginSkillsPort` implementation delegates to this same service and preserves the exact status projection. The relay retains one settlement, UUID-qualified correlation and close/no-replay behavior.

The revised expectations do not change production permission policy or create another owner.

## Original-scope preservation and adjacent consumers

The entire six-path focused delta was read. Complete sixteen-path original-scope changes were retained as context: the Spec; source contract; parity ledger; RTM; capture; both edited relay files; Skills owner; strict image owner; auth recovery tests; sync tests; publisher; Session forwarding; workflow store; MCP service tests; workflow tests.

Deepest reading covered the complete affected relay helpers/tests, the full Skills production owner, the strict image production owner, and relevant adjacent service/relay/production/status paths. For the larger pre-existing files, reading was of the complete changed hunks and applicable surrounding functions, not a claim that every unrelated function in Session production, the ledger, RTM, capture, existing integration tests or workflow was reread.

Preservation checks include:

- Auth recovery tests retain actual failed-negative-write → ordinary read → partial refresh denial; panic/inherited unknown → no-old-fallback recovery; active process lease through cleanup and successor guard lifetime; authoritative SH fault latching and explicit repair/fresh-owner distinction; contended-constructor unknown recovery and absent-root behavior.
- Image tests retain migration, strict invalid-shape cases, actual reservation/settlement/release rotation, auth-refresh stability, loss-to-legacy/null faults, marker and byte preservation. Source review did not substitute their assertions for execution.
- Existing owned-process test harness uses the actual current test executable, phase/nonce/PID-qualified receipts, bounded owned Child handling and actual Session/workflow/prompt consumers. The repeated-confirmation/not-removed cycles retain no intermediate owner admission read, semantic-set equality, causal observation difference, stale publication/prune/report refusal and later fresh recovery.
- Workflow mutation reacquires through the existing reader, rereads the committed record and publisher before file work, and retains the lease through replacement. The real gated tests keep changed/unknown publisher, removed/remapped record, unchanged file bytes and current-publisher success controls.
- Resync/unpublish hold the actual owner's SH lease through protected filesystem/backend work. Upload cleanup releases it before diagnostics; the actual cleanup-I/O failure test probes EX before real reentrant sync on success and failure, preserving original result and diagnostics.
- Session forwarding clones and releases its slot before invoking either owner query or lease acquisition. It does not become a second permission store.

## Capture, shared changes and source-only compile review

At `projects/openclaw-rust/scripts/capture_portable_contracts.py`, both edited files remain inputs. `plugin-uninstall-service` still requires the three full desktop test names, and `plugin-uninstall-relay-handoff` still requires the exact library name with its existing libtest selector. Command/capture exit, required positive-name uniqueness and nonempty passing summaries remain acceptance inputs. No assertion/command was deleted and no duplicate workflow is needed for this repair.

Read complete shared capture/ledger/RTM hunks. Independent SC invocation-identity controls and LXR custody references/migration limits remain preserved. No Plugin-scope verdict accepts those independent production batches or promotes their global/full-unit acceptance.

Source-only Rust review found no new borrowing, ownership, lifetime or type inconsistency in the modified helpers/tests: owned String snapshots outlive decoded temporary values where needed, borrowed JSON fields stay within their owned Value lifetime, assertion macros borrow compared fields, snapshots remain comparable, and fixture worker completions are joined before successful teardown. Existing Cargo declarations include UUID v4 and the existing serde/fs2 dependencies. This is not a compiler result.

No local/htch executable validation, build, test, lint, parser or generator was run. No workflow was dispatched and no Actions logs or runner decisions were consulted. Exact-H compilation and affected contract execution are still required before runtime verification. Real ecosystem/accounts, platform barriers/packages, complete source/dependency closure and full OCRM acceptance remain outside this source verdict.

Only this new independent report may be uploaded as a Git blob for the authorized task. No source/test/spec/shared-file mutation, commit, ref update, PR action or historical evidence replacement is part of this review.
