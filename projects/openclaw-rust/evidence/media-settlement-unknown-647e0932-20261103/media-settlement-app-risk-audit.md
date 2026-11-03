# MSU App supplemental implementation-risk audit

## Result and limits

**No actionable new destructive, same-entry race, typed-outcome erasure, or source-level compilation defect was demonstrated in the reviewed immutable snapshot.** This is a bounded source assessment of the App media owner and its immediate dependencies. It is not the required independent final-candidate approval, an executable verification result, or approval of the complete MSU implementation.

The observed implementation closes the previously identified busy-resolver/outer-abort counterexample and retains custody across the admitted append's unknown outcome or unwind. The remaining uncertainty is executable behavior and complete-candidate composition, including the actual App tests being authored separately. Those active work-tree files were neither read as authority nor edited.

Authority: `media-settlement-app-audit-manifest.json`, based on H `a2a0c9ef326c1ddeb09b4f9180267fdaeb21db74` and fixed U `1440bb1771372334bea23d7642bb52deb7db1845`. The snapshot includes the frozen MSU-v1 contract; this review uses its local-state, typed-chain, and recovery requirements.

| Reviewed snapshot file | Manifest Git blob |
| --- | --- |
| `source/host/app/src/main.rs` | `7ec9bd8a22fd595f63cb3f4b632673e298294dfa` |
| `source/host/src/attachment_paths.rs` | `6e1a2b8eb5752e005bbce30ed0b6eaaacfda0cda` |
| `source/host/src/extensions/session/production.rs` | `c5912cab4f95d3a75fc13cb264a21b1deef9a733` |
| `source/host/src/extensions/transcript/production_runtime.rs` | `9f15ddbdebe0ec7fe612975e28e9a7d5d0c3beb6` |
| `source/host/src/extensions/session/agent_db.rs` | `6c27e9bb9f665b1160457b9acc29dd5d64fba289` |
| `source/host/src/runner/tools/send_message_tool.rs` | `86a9e0e394b0145173bc033ceb757b7fb10835e5` |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `356b0071742903dfcc2f52ebad012f37ad1a221c` |

All abbreviated locations below refer to these files under `/workspace/scratch/7d27ea239237/media-settlement-app-audit-snapshot/`. These are the manifest's identities, not an assertion that executable checks were performed against them.

## Concrete failure sequences examined

### 1. A second resolver observes busy, then aborts another operation's prepared files

The previous counterexample was: operation A owns the entry; B receives an unknown refusal while A is busy; A becomes prepared and releases the entry; B's outer Tool error branch then aborts the now-available custody.

The snapshot prevents that sequence at both relevant boundaries. `main.rs` L817 uses the entry's `try_lock`; L819 returns `ToolOutcomeUnknown` for busy. `send_message_tool.rs` L409 excludes that exact typed error from construction-error cleanup and returns the original error at L412. `abort_attachment_custody` at App L1088 also acquires an existing entry through `try_lock`; it cannot wait for or remove a busy entry. This is the narrow error-class behavior the earlier Tool correction intended.

Known construction errors still clean the same collecting attempt. App L1080 limits resolver cleanup to known errors, and the Tool's encoding-error cleanup remains intact at Tool L418. An already Aborted entry makes the resulting repeated abort inert through the rollback phase match at App L862.

### 2. A stale cloned Arc or repeated resolution creates a second custody instance

`pending_media_entry` at App L795 is the sole lookup/insertion method. Its map guard is limited to L800/L807/L810 and is gone when the caller takes the entry guard at L1012 or L1117. There is no removal/replacement branch in the reviewed production owner. Terminal entries therefore retain the same Arc identity.

At App L1013, every resolution checks the collecting phase before source materialization. Pending, committed, or custody-sealed states refuse; Aborted returns the known terminal refusal. The same guard remains held through generation checking, resolution, path tracking, capture, and cache insertion. App L1018 reuses an exact source's cached resolved value before another call to the materializer or capture owner; distinct sources still use the normal collection path. Successful and refused terminal transitions clear the cache at L774 and L870.

This prevents the specific remove/recreate and duplicate exact-source materialization sequences for an existing entry. It does not establish authentication or exclusive ownership across arbitrary whole workflows that happen to reuse a string.

### 3. Real COMMIT succeeds but its acknowledgment is lost

App L1136 seals custody before the Runtime method begins its history read at `production_runtime.rs` L1809. The local cleanup disposition becomes `OwnerSettlementPending` in `attachment_paths.rs` L62, and App phase changes at L766 before the potentially committing owner call at App L1154.

The DB media method's COMMIT-error branch at `agent_db.rs` L5066 retires the exact connection at L5072 and returns `TranscriptCommitUnknown` at L5073 regardless of whether SQLite is already in autocommit. No successful DB publication occurs along that return path. Session L4076 retains the inner typed `Result` inside the existing filesystem boundary; its outer error mapping at L4083 applies only to the enclosing before-operation error. Runtime L1829 propagates that typed result.

App L1166 maps the variant to `ToolOutcomeUnknown`, and L1203 excludes it from rollback. The entry, captured session/revision/work identity, resolved-source cache, and owned paths remain. Subsequent resolve/send calls refuse before history allocation or another append. Ordinary abort reaches the non-cleaning phase arm at L867. Custody Drop calls `rollback` at `attachment_paths.rs` L103, whose disposition guard at L85 leaves pending bytes intact.

No source path in this reviewed chain converted this unknown result into an ordinary cleanup error or a successful send.

### 4. Receipt insertion fails after an inserted prefix, and rollback is refused

The DB media result-error branch invokes rollback at `agent_db.rs` L5094. A rollback error or remaining live transaction retires the owner at L5096 and returns typed unknown at L5097. App therefore takes the same retention branch described above.

For a confirmed ordinary precommit refusal, the DB returns the original ordinary error only after the rollback/autocommit checks. App's current entry guard remains held when L1205 calls `rollback_pending_media(..., true)`. Only that branch restores pending cleanup authority through `attachment_paths.rs` L72. The ordinary public abort passes `false` at App L1092 and cannot perform that restoration.

The local boolean is an internal control-flow proof rather than a new public durable identity. In the reviewed owner, its `true` use is restricted to the same guarded send's known error branch. No competing operation can substitute the entry between sealing and that decision.

### 5. The append succeeds but returns an unusable entry shape

The Runtime validates an ID before entering Session at `production_runtime.rs` L1822, and DB validates entry identities at `agent_db.rs` L4994. App nevertheless treats a missing/empty returned ID as unknown at L1216. That branch leaves sealed custody in place and returns before local commit or App publication. It does not call `commit` merely to suppress deletion.

On the ordinary success path, App commits custody at L1220, releases the entry guard at L1224, and only then enters the channel-delivery branch at L1225, publishes the App transcript event at L1259, fulfills the ACK obligation at L1267, and returns success at L1273. The existing counting wrapper consequently cannot count an unknown return as a successful delivery.

### 6. Owned media uses the legacy no-fence append API

App captures `has_owned_media` before entering the append at L1132. The legacy error mapper at L1138 returns typed unknown when that flag is true, preserving sealed bytes because the String API cannot establish a clean refusal. The untracked-text branch retains its existing ordinary API behavior.

This matches the frozen contract's conservative retention fallback. It is not a new durable receipt or proof that no-fence compatibility durability has been completed.

### 7. An owner call unwinds or an entry mutex is poisoned

For an unwind after send begins the owner call, both local phase and cleanup disposition were sealed first. Dropping the guard, sink, or custody cannot delete those bytes. A later poisoned-entry access at App L822 does not clear poison or resume the operation. If the interrupted entry was still Collecting, L826/L827 can only disarm cleanup and mark it pending; it then returns unknown.

A collecting operation that unwinds before any append has no transcript commit from this attempt to protect. This review does not promote arbitrary materialization crash windows or the compatibility path into the stronger admitted-transaction guarantee.

## Cleanup and recovery dependencies

The ordinary cleanup helper at App L856 changes an eligible attempt to Aborted and removes its cache. If file cleanup reports errors, L881 returns without releasing the durable media fence. A Session release error is logged while leaving the work identity present. `release_session_media_admission` in Session L2773 checks both captured revision and work ID; mismatch returns `false` before the durable record is mutated. App's `Ok(_)` branch at L889 clears the local attempted work token even for that no-op result, but does not clear a foreign durable fence. No unknown-settlement path reaches this branch, and this read did not establish a destructive counterexample from that bookkeeping choice.

First-open recovery remains the existing Session owner. Session L1687 returns an existing cached owner instead of silently replacing a retired one. The first-open branch calls pending-media recovery at L1715. Recovery inspects the durable receipt/fence at L1775; only after its existing checks does it create fresh rollback-eligible custody at L1802 and track verified paths at L1808. The changed `track` return is handled. Recovery's successful fence clear is separately required at L1818.

This is consistent with retaining a live unknown tombstone, retiring the transaction owner, and letting an explicit fresh-owner recovery decide the durable outcome. The audit did not execute lost-ACK, denied-rollback, or reopen cases.

## Important boundaries for the root's description and tests

**“Callbacks run after terminal state” is accurate for the App's outbound delivery, Gateway event, and ACK sequence, not for every lower-level notification.** The DB still publishes entries and notifies metadata listeners at `agent_db.rs` L5085 and L5087 before returning through Session. Its listener dispatcher invokes the existing callbacks synchronously at L820 and catches their unwinds at L821. At that moment, the App entry guard is still held and phase is pending, although the DB commit is confirmed. The map guard is not held. A same-ID sink reentry refuses through `try_lock`; it cannot reacquire the entry and mutate custody. No new global callback-order guarantee should be claimed, and no unrelated listener redesign is proposed by this audit.

**The local owner is not a universal tool-ID registry.** `send_message` at App L1115 looks up an existing entry without creating one for untracked text. The per-invocation recorder remains the existing narrower duplicate-ID boundary. Its immutable `370c6680b6b8c072615875d72be67ca0e45ebe45` source registers and rejects reused first-party SendMessage IDs at `source/host/src/runner/turn_agent_composition/tool_call_recorder.rs` L255/L269 before delegation. This does not authenticate unrelated non-overlapping calls, and the reviewed implementation does not claim that it does.

**Source consistency is narrower than compilation.** The App imports the typed DB error at L76, uses the existing `ResolvedAttachmentSource` type at L473, and constructs the new map with inferred entry type at L11712. The Session nested-result signature and Runtime typed return line up with the sole reviewed App media call. The existing recovery caller now handles `track`'s Result. I found no concrete signature/coercion/borrow defect by source inspection, but no compiler, formatter, parser, feature-gating build, or test listing was run. The active App test worker's changed fixture construction and assertions are outside this frozen authority and remain an integration dependency.

## Minimal fix recommendation and outstanding evidence

No production fix is proposed without a demonstrated defect. Preserve the current typed error-class decision, sealing order, map/entry separation, and terminal-entry retention while integrating the tests. If documentation describes all callbacks as outside custody locks, qualify that wording to the actual App publication sequence above.

The actual Actions evidence still needs to establish the root-owned cases: real lost acknowledgment and real refused rollback with applied fault observations; known-refusal cleanup control; same-ID busy/retry/abort behavior; sink Drop and two fresh-owner observations; generated Runner downstream-unknown custody and no replay/success; and normal feature-off shipping compilation. This audit supplies no runtime verdict for those cases.

## Read and write scope

Read: the immutable manifest and frozen MSU spec; the App's media materializer, pending-entry type/helpers, resolver, abort, send, relevant imports and production constructor; local custody methods and adjacent private tests; typed Runtime and Session media facades; Session admission/capture/release, filesystem boundary, owner-open and media recovery; the DB media transaction, outcome/retirement branches, immediate notification/command helpers, and the compaction pending-media fence check used to bound a generation-change hypothesis. Immutable 370 Git reads were limited to custody caller discovery and the recorder registration excerpt. The unrelated bulk of the approximately 1 MB App source was not reviewed.

Written: only `/workspace/scratch/7d27ea239237/media-settlement-app-risk-audit.md`.

No production/shared/test edit, no local or htch executable validation, no build/test/parser/test-listing/capture, no commit/push, and no Actions dispatch/rerun occurred. Prior A2 evidence was untouched during this audit. No failure was claimed to have been reproduced.
