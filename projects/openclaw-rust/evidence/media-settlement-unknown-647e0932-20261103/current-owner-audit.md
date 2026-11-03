# Media settlement unknown-outcome audit

Task: 647e0932-bea4-4113-9a61-65ae3db58955 — independent owner audit, round 1  
Assessment: bounded source review; **one P1 correctness finding; no implementation or reproduced-failure claim**  
Reviewer: media_settlement_unknown_audit; configured model retained; inherited actual xhigh (lifetime, transactional outcome, filesystem custody and recovery scope)  
Date: 2026-11-03

## 1. Frozen authority and limits

Reviewed target: bhrumom/fabushi-desktop, migration branch codex/openclaw-rust-fusion-chat-20261101-writer, exact commit **78aa967cc9400bb5c207441b2a78dbf7021d596c**, tree **f62fc666ade7ad6f8dbf40e889e5ec100a74e974**. Git object inspection confirmed the commit and tree. The branch name is the assigned migration line, not an independently refreshed live-ref claim.

Fixed upstream U remains **1440bb1771372334bea23d7642bb52deb7db1845**; no rebaseline or new upstream source-reading credit is claimed.

All repository reads used immutable Git objects in /workspace/scratch/29e7135d8bfe/fabushi-desktop. No mutable worktree source, mutable R5, other review report or root arbitration was read. The permitted R4 App overlay was unnecessary for this settlement audit and was not used. The inspected current-base App blob is ca038b1aa9f3a78a1951551e0a80d543142c08dd. Local-file admission/LFA is not reviewed here.

No build, test, lint, generator, fault-injection, capture or other executable validation was run locally, on htch-runtime, or in Actions. The only authored artifact is this report. Existing test bodies and workflow selectors were read as source, not promoted into execution evidence.

### Governing requirements read

- Root AGENTS.md, including the independent immutable review contract, actual effort/model policy, existing-owner requirement and Actions-only OCRM execution rule (lines 237–270, 371–395).
- OCRM SOURCE_OF_TRUTH current authority/constraints (lines 1–84), main spec §7, §9–10 and current **§17 OCRM-EXEC-02** (lines 671–720). In particular, §9 distinguishes unknown mutation outcomes from refusal, and §17.3 requires the producer-to-consumer and cancellation/restart chain.
- final-assistant-media-custody.md, completely: MAC-01 existing owners; MAC-03 atomic settlement; MAC-04 cleanup only before commit; MAC-06 generation; MAC-08 durable materialized paths and bounded recovery; explicit no-delete-after-commit non-goal (lines 35–104).
- Applicable session-work-admission.md requirements and current media/recovery boundaries: SWA-03/06/07, owner-local identity, unknown-result no-retry, serialized recovery and media generation (lines 24–164, 196–324, 686–747); strict-first-open recovery controls (1763–1791).
- channel-inbound-worker-handoff.md **SESSION-FRESH-APPEND-v1**, especially FC-03/04/07 and “Fresh failure settlement” (331–457).
- generated-send-message-fidelity.md producer/consumer/unknown-effect contract and current test scope (1–216). The old frozen-validation wording in older media documents does not override current §17.
- Current relevant FBCP source-of-truth, architecture map and FBCP spec owner/settlement/quality requirements; OCRM module-map M07/M23/M29. These retain the same Session/Transcript/resource owners and distinguish source observation from acceptance. No unrelated historical dossiers were needed.

## 2. Finding MSU-01 — P1: an unknown media settlement authorizes destructive abort

**The shipping media path has no safe distinction between “this append was rolled back” and “the commit outcome is unknown.”** After an error it removes the newly materialized files, even when the database may already contain the committed message and media receipt, or may still contain an uncommitted prefix on its live connection.

The exact outcome-erasure chain is:

1. **DB** `append_transcript_entries_and_release_media`, lines **4899–4917**: COMMIT failure attempts ROLLBACK, discards its result, and returns an ordinary `AgentDbProjectionError::Sqlite`; a pre-COMMIT error likewise discards rollback outcome and returns the original error. It neither checks autocommit nor retires an unsettled connection.
2. **Session**, lines **4062–4080**: `append_agent_transcript_entries_and_release_media` converts every DB error to `String`.
3. **Runtime**, lines **1797–1833**: `append_generated_send_message_entry_with_media_custody` returns `Result<Value, String>` and propagates that undifferentiated error.
4. **App**, lines **1068–1074**: every append error calls `abort_attachment_custody` and becomes `ProviderSessionError::Tool`.
5. **App**, lines **965–991**, removes the pending local handle, invokes its filesystem rollback and then attempts to release the durable fence. **Paths**, lines **43–62**, physically removes every tracked file; Drop invokes the same rollback when the handle is not committed.

Primary source: [DB settlement](https://github.com/bhrumom/fabushi-desktop/blob/78aa967cc9400bb5c207441b2a78dbf7021d596c/source/host/src/extensions/session/agent_db.rs#L4660-L4919), [App settlement/abort](https://github.com/bhrumom/fabushi-desktop/blob/78aa967cc9400bb5c207441b2a78dbf7021d596c/source/host/app/src/main.rs#L965-L1091), [custody destructor](https://github.com/bhrumom/fabushi-desktop/blob/78aa967cc9400bb5c207441b2a78dbf7021d596c/source/host/src/attachment_paths.rs#L21-L63).

This contradicts the already applicable distinction in SWA-06 and OCRM-C02/C10, and the final-media contract that pre-commit cleanup must never delete media referenced by a committed transcript. The old MAC source-wiring “passed” statements are not evidence that these unknown-outcome cases are safe.

### A. Successful COMMIT, lost acknowledgment

Concrete causal scenario, **not executed in this audit**:

- The normal producer records materialized path P under work W and generation G.
- The media transaction inserts the send entry, inserts `outgoing-media-settlement:W` retaining P and the transcript IDs, and clears the pending media work/path fields.
- SQLite has committed those changes, but the command boundary reports a failed acknowledgment.
- The current DB error arm returns an ordinary error; its best-effort ROLLBACK cannot undo a transaction that is already committed. Session/Runtime flatten the result and the App deletes P.
- The App does not reach its success event, delivery dispatch or ACK fulfillment. The persisted send still points to P, but P no longer exists. `release_session_media_admission` sees that the committed transaction already cleared W and can return `Ok(false)`; App ignores that false result.
- On first reopen, media recovery sees no pending work and returns immediately. The receipt retains a path, not its bytes; recovery cannot reconstruct the deleted media.

The existing channel test seam already models the decisive condition correctly: perform real COMMIT, then return an acknowledgment error. It is **not connected to the media path today**. This audit does not assert that a native SQLite call at this target has actually produced this failure, or that any user data has been lost.

### B. Real inserted prefix, later refusal, rollback refused

A single App send is sufficient: arrange a real SQL trigger failure on the hidden media-receipt insert, after the send entry has been inserted; refuse the ROLLBACK command on that same owning connection.

DB lines 4881–4917 leave the transaction unsettled and return the original ordinary error. The App deletes P. Its later media-release CAS starts another BEGIN IMMEDIATE on the same open transaction (DB line 1497); the release may fail and be logged, but deletion already happened.

The connection is still cached and open. `get_transcript_entries` reads that connection without an autocommit check (DB 5659–5672); the public Session reader delegates to it (Session 1290–1299). Consequently, the uncommitted send prefix can be consumed as transcript state in this process. A later close/reopen may roll it back, but that does not justify the earlier ordinary-error classification or same-connection reuse.

The existing fresh-append test installs a real SQLite authorizer that refuses ROLLBACK; it also performs actual inserts and separately exercises insert and COMMIT failures. This provides a concrete reusable fault mechanism, not a reason to assume the media behavior is already tested.

## 3. Actual producer → owner → settlement → consumer chain

| Boundary | Exact implementation at the frozen target | Consequence |
| --- | --- | --- |
| Shipping composition | App 11563–11585 creates the real `ProductionSendMessageSink`, then wraps it in `CountingSendMessageSink`. | This is production reachability, not an isolated helper. |
| Tool producer | Tool 388–447 parses/validates, resolves attachments, encodes arguments and handles pre-send failures; 450–481 invokes the sink and returns its original error. | Build/encoding/awaiting-selection failures legitimately abort before settlement. |
| Materialization | App 618–701 uses existing attachment persistence; Paths 236–254 writes a UUID-named file through temporary-file rename. App 877–960 tracks newly returned owned paths and captures W/G in Session. | Local cleanup authority is attached to the files created for this call. Source-file bytes are not deleted by this custody handle. |
| Durable fence | Session 2681–2766 records canonical session key, work ID, lifecycle revision and materialized paths through the existing owner. Other pending work is refused. | The recovery input already exists; no second journal or media store is needed. |
| Precheck and handoff | App 744–768 and 1014–1042 recheck the captured generation, select the media transaction and pass work/revision/session identity. | A facade check does not replace transaction-edge checks. |
| Generated entry | Runtime 1809–1825 reads current visible history, uses the existing send pipeline, validates an ID and stamps session provenance. Pipeline 276–300 allocates the existing SendMessage ID and reply/batch metadata. | This is not the pure fresh-append planner; blindly replacing it with that API would expand semantics. |
| Single writer | Session 4062–4080 holds the existing per-agent filesystem/deletion boundary; DB 4677–4897 owns one BEGIN IMMEDIATE transaction. | Transcript row, receipt and media-fence release already share one SQLite owner. |
| Identity/replay | DB 4688–4823 checks admission; 4709–4812 checks receipt, current generation, absence of pending custody, exact incoming IDs and persisted content. | Identical committed replay returns zero inserts. Conflicting/rebound replay refuses. |
| Receipt | DB 4873–4895 and 6384–6405 stores session/work/generation, media paths and transcript-entry IDs, then clears the captured fence by CAS. | Preserve this durable format and identity binding. |
| Successful publication | DB 4905–4913 publishes mutation/admission observations; App 1089–1140 commits local custody, schedules optional channel delivery, publishes the App transcript event, fulfills ACK and records activity. Tool 172–175 increments the count only for Ok. | Local SQL success and external channel delivery remain separate outcomes. |
| Consumer/reopen | Session 1290–1299 hides control receipts from visible history; 1757–1889 uses them only for media recovery. Existing SGF App controls inspect actual SQL/media bytes, canonical wire and fresh-owner history. | A send error must not manufacture success, completion authority or a second delivery. |

The media function does not itself use the compatibility `run_write_value` retry/default helper. The problem is the transaction outcome and its consumers, not an observed automatic retry inside this function.

## 4. Recovery behavior and its practical limit

The ordinary first owner open validates checkpoint and session identity, then invokes media recovery before caching the owner (Session **1696–1719**). A cached owner path invokes profile/memory recovery but **does not run media recovery again** (1687–1694). Calling a read facade after an unknown settlement therefore is not equivalent to a fresh media reconciliation.

Recovery behavior is bounded:

- No media work and no recorded paths: return without cleanup (1768–1772). This is the expected state after an actual committed media transaction.
- Pending work plus a matching receipt: retain files, clear the matching fence (1774–1796).
- Pending work without a receipt: require durable paths, verify existing files against canonical agent media roots, reject symlink/outside-root uncertainty, remove only verified owned files, then clear the matching fence (1798–1889).
- Work without paths, paths without work, malformed identity or cleanup/CAS failure: fail and retain the durable fence.
- Strict terminal first-open intentionally refuses pending media without caching an unrecovered owner (1703–1712).

The existing paired owner retirement API is Session **2025–2051**; the existing recovery test uses it before reopen. DB connection retirement and Session cache eviction are distinct actions. Closing an unsettled connection must not silently imply a second writer, an automatic producer replay, or an arbitrary replacement of a live AgentStore.

After a future fix, a clean-rollback unknown result may remain fenced until an explicitly safe fresh owner open. A committed-but-unacknowledged result can be read after reopen while its media bytes remain intact. Neither state authorizes the tool/provider to resend or reallocate an entry automatically.

## 5. Minimal reuse contract and affected interfaces

### Existing mechanisms to reuse

Fresh append already provides the needed failure semantics:

- `AgentDbProjectionError::TranscriptCommitUnknown(String)`: DB **63–66**.
- Explicit pre-COMMIT rollback plus autocommit check: DB **4288–4302**. Ordinary refusal returns only after rollback acknowledgment and a clean connection; otherwise typed unknown.
- Every failed COMMIT acknowledgment remains unknown, even if cleanup subsequently succeeds: DB **4304–4314**.
- Existing exact-connection retirement: DB **4516–4524**. It marks closed, drops the connection and unregisters precisely that live handle.
- Session preserves the typed error using a nested Result under the existing String-based filesystem lock wrapper: Session **3930–3945**.
- A2A consumer retains its reply claim on the typed unknown and only releases on ordinary refusal: A2A **626–637**. This is the appropriate disposition pattern.
- Existing `ProviderSessionError::ToolOutcomeUnknown`: Provider **303–304**. The real guarded tool worker marks sticky `DownstreamOutcomeUnknown` before completing its local receipt (Guard **198–207**), even before later provider adapters serialize errors. The retry classifier preserves explicit uncertainty (Shell **778–789**).

The fresh append function is not a drop-in media transaction. Its pure planner contract forbids owner mutations and it does not atomically release the media fence or write the existing receipt. Reuse the **transaction settlement/retirement semantics within SandAgentDb**, with a small owner-local extraction if justified; keep the media-specific receipt/CAS validation in that same owner. Do not move this responsibility into App, a new store, a background registry or a new runtime.

### Required type/control changes if implementation is authorized

| Existing surface | Narrow required change |
| --- | --- |
| DB `append_transcript_entries_and_release_media(...)->Result<usize, AgentDbProjectionError>` | The public error type already suffices. Apply the existing typed-unknown decisions to failed COMMIT and unconfirmed rollback; retire an unsettled connection before returning it to any consumer. Pre-existing live transaction and BEGIN failure must not be mislabeled as clean NotWritten. Do not auto-replay. |
| Session `append_agent_transcript_entries_and_release_media(...)->Result<usize, String>` | Preserve `AgentDbProjectionError` through the filesystem/admission composition. Reuse the nested Result technique in `commit_agent_transcript_entries`; do not globally widen the unrelated lock helper. |
| Runtime `append_generated_send_message_entry_with_media_custody(...)->Result<Value, String>` | Carry the same typed error to the sink. Classify entry construction/history/admission errors before the settlement attempt explicitly; never parse error strings to recover outcome. |
| App media branch of `send_message` | Separate confirmed success, confirmed pre-commit refusal and unknown. Unknown retains media/fence, emits no success/ACK/channel dispatch, and returns existing `ToolOutcomeUnknown`; ordinary pre-commit refusal continues cleanup. Keep non-media compatibility APIs separately typed/adapted rather than treating their String as proof of rollback. |
| `PendingOutgoingMedia` / `OutgoingMediaCustody` | Add an explicit disposition that relinquishes local deletion authority to the existing Session recovery owner on uncertainty. It must survive sink/handle Drop and subsequent abort calls, retain enough identity to refuse accidental same-attempt replay, and must not claim “committed.” Simply omitting the immediate abort, removing the map entry, or calling `commit()` to disguise unknown is insufficient or semantically wrong. |
| Post-commit handling | Establish the confirmed custody disposition before any later fallible action could route through pre-commit cleanup. The current Runtime already checks the ID before writing; preserve that invariant. |
| Notifications | If settlement/publication is refactored, reuse the existing deferred publication pattern. Media currently calls synchronous entry/admission listeners while its DB and Session guards remain held (DB 4905–4912); fresh append explicitly publishes after both locks. Carry both kinds of observations outside those locks rather than copying the locked callback pattern. This is an adjacent integration constraint, not a separately reproduced deadlock finding. |

The existing media contract needs a narrow amendment spelling out these three outcomes and the Drop/recovery handoff before implementation. No new persisted schema is intrinsically required for this bounded no-delete/no-replay repair: the pending fence and committed receipt already distinguish the two durable outcomes after reopen. Full detached-result/restart delivery remains a separate contract.

Do not implement “retry the same Runtime method to find out”: Runtime regenerates an entry from current history, whereas receipt replay requires the original exact IDs and content. An unknown append does not authorize a fresh ID, a second tool execution or channel dispatch.

## 6. Existing fault ports and test coverage inspected

| Existing source/test | What it actually controls or asserts | Limit for this task |
| --- | --- | --- |
| DB 6083–6109, `ChannelInboundTransactionControl` | Private per-call control: real COMMIT followed by a synthetic lost ACK; optional failed rollback command. Production default executes real commands. | It is currently connected only to channel append. Its cfg(test) fields are not a public media/App fault port. |
| DB 8688–8713, `real_commit_failure_stays_unknown_after_successful_rollback` | Deferred FK makes real inserts succeed and real COMMIT fail; unknown survives confirmed clean rollback. | Channel owner semantics only. |
| DB 8717–8749, `unconfirmed_rollback_retires_the_exact_connection_before_reuse` | Real inserted prefix, later trigger refusal, rollback-control failure; no publication, no live connection/handle, clean independent inspection. | Channel path, not media. |
| DB 8753–8774, `real_commit_with_lost_acknowledgement_is_never_not_written` | Real durable channel rows exist despite returned unknown; no success publication. | This is the correct lost-ACK mechanism to adapt, not existing media evidence. |
| DB 8908–8938, `refuse_rollback_commands`; 9132–9185 | Installs an authorizer on the actual owned SQLite connection; separately tests real pre-COMMIT insert refusal and COMMIT refusal, exact retirement and unchanged fresh-owner rows. | Private fresh-append unit scope; no App access today. |
| DB 9091–9128 | Fresh append real deferred-COMMIT failure, one planner call, typed unknown, unchanged rows/generation, no publication/recovery. | Test explicitly says it does not simulate successful-COMMIT lost ACK. |
| agent_db_owner_contract.rs 1570–1825 | Real media atomic append/release, payload/provenance/identity refusal, partial-batch clean rollback, receipt persistence/reopen, exact replay/conflict, malformed receipt and deletion fence. | No lost-ACK or refused-ROLLBACK control in this media test. |
| Paths 272–298 | Explicit rollback removes two files; explicit commit prevents deletion. | No unknown/Drop disposition case exists. |
| agent_session_contract.rs 1492–1614 | Durable media blocks archive; matching release; paired retirement/reopen cleans actual pending media. | Does not begin with an unknown media transaction. |
| Session 6576–6680 | Strict first-open retains media bytes/rows; ordinary open removes only owned file and preserves sibling; work-only fence remains unknown across repeated opens. | Reusable recovery invariants, not commit-error coverage. |
| App channel_host_lane_tests.rs 397–472 and SGF module 1538–2319 | Real App sink/Session/Runner fixture; generated attachment normalization and exact bytes; SQL/canonical-history agreement; two fresh owner reads without resend. | Its existing SGF failure is subsequent provider failure, not a media settlement SQL failure. |
| background_wakes_production_contract.rs 2911–2935 | Static source ordering/presence checks around persisted send and cleanup. | Does not decide transaction outcome or file survival. |

**Fault reach is a real implementation dependency.** The private library cfg(test) command control and raw connection authorizer are not available when the Host runtime is compiled as the App's ordinary dependency. This target declares no general fault-injection feature (Host Cargo.toml 1–59), and configured SQLite setup enables WAL/NORMAL but does not explicitly enable foreign keys (agent_db_recovery.rs 381–405). Setting PRAGMA foreign_keys on an unrelated inspection connection would not establish a causal COMMIT fault on the owning connection.

For owner tests, reuse/adapt the existing private command control and actual authorizer. For an actual App lost-ACK test, first choose the narrowest **per-owner/per-attempt test access** to that same command boundary, or qualify another real same-owner fault mechanism. Do not add a global environment fail flag, duplicate settlement implementation, mocked “unknown” result, or a second fault framework. A composed unit-plus-App disposition test can be recorded as composed evidence, but must not be labeled an end-to-end lost-ACK run.

## 7. Proposed causal Actions checks

These are proposed oracles, not executed results. Run only after the integrated code/spec/test subject is frozen and the relevant shared Windows dependency is resolved.

| ID | Causal action | Required observation |
| --- | --- | --- |
| MSU-T01 | Real App/tool materialization followed by successful media settlement. Reuse SGF positive fixture. | One send and one receipt; exact file bytes survive sink drop and reopen; pending fields clear; success event/count and ACK only after owner success. Optional channel dispatch follows the same committed ID. |
| MSU-T02 | Fail the hidden receipt insert after a real send insert; allow ROLLBACK. | Ordinary known refusal; clean reusable connection; no send/receipt; App removes only its new files and releases matching fence; original source/sibling unchanged; no success side effects. |
| MSU-T03 | Real deferred-constraint COMMIT refusal on the actual owner, with cleanup allowed. | Typed unknown despite clean rollback; no fresh publication; actual App boundary retains files and pending fence, survives Drop/abort, returns ToolOutcomeUnknown, and records unknown effect custody. No retry/ACK/channel/completion. |
| MSU-T04 | Execute real media COMMIT, then lose only its acknowledgment through the command-control seam. | Fresh independent DB inspection sees exact send/receipt and cleared fence; caller gets unknown; all media bytes survive local teardown and two fresh owner opens; no automatic resend, new ID or external delivery. |
| MSU-T05 | Real prefix insert/receipt refusal and real COMMIT refusal as separate cases, each with same-connection authorizer denying ROLLBACK. | Unknown, exact connection retired/unregistered, no same-owner prefix exposure or reuse; no files deleted by App/Drop; no success observation. Preserve original SQL cause in diagnostics. |
| MSU-T06 | Fresh owner open after T03/T05, after safe owner retirement and quiescence. | The existing recovery owner alone clears a proven uncommitted fence and removes only recorded, verified files. Preserve original source/sibling; repeat reopen is idempotent. Unknown/malformed/path cases remain fenced. |
| MSU-T07 | Invoke abort and then drop after uncertainty; attempt same tool/work identity again before reconciliation. | No local deletion and no implicit second append/materialization/dispatch. Confirmed pre-commit failures still clean normally, so retention does not disable all cleanup. |
| MSU-T08 | Preserve existing stale generation/key, archived/deleting owner, mismatched receipt, conflicting replay, hidden projection and listener-reentry controls. | No widening of admission/replay; no callbacks under owner locks if publication is changed; no weakening of existing assertions. |

Select the existing `.github/workflows/rust-desktop-runtime.yml` command/capture machinery. It already selects agent_db_owner_contract/agent_session_contract (1738–1747), private fresh/channel/recovery filters (1758–1777), and real App SGF tests (1366–1371). The portable capture script already requires exact fresh-failure names (362–368, 790) and SendMessage controls (582–650). **New test functions are not automatically selected by the App's exact-name list**; update the same selectors, required-positive names and input identities in the eventual batch.

Record actual integrated H/tree, command, executed names/counts, raw logs, artifacts, runner identity and exit outcome. Existing green runs or source-read test assertions cannot be relabeled as this repair's evidence.

## 8. Remaining dependencies and bounded conclusion

1. The requested pending shared Windows publication/Actions work is outside this audit. This report neither reads its mutable candidate nor authorizes dependent persistence edits while that shared contract is unresolved.
2. The minimal type chain and existing owner are established. The narrow spec amendment, local custody terminal state and causal App test seam still need an implementation decision and independent review before coding.
3. Unknown outcome must remain explicit through the real guarded tool boundary. A mere formatted error string misses the existing sticky effect-custody mechanism. Real-provider acceptance is not established by a controlled App fixture.
4. First-open recovery can preserve/clean files but is not a detached-result replay engine. Cached rereads, receipt absence on an uncertain connection, elapsed time, or an already-closed owner do not authorize replay.
5. The no-fence legacy generated-send/compatibility paths still use different String-returning APIs. This assessment and proposed minimal fix concern the durable-fence media settlement path; they do not confer unknown-outcome safety on every generic append.
6. Entry allocation concurrency, capture-before-settlement failures, complete descriptor-bound media path identity, cross-session placement and full detached recovery remain outside this bounded change. MAC-05's broader limitations remain open. Neither source-accounting flags nor product acceptance should be raised.

**Recommended next step:** record MSU-01 in the existing task/spec owner with this exact source identity, then implement the narrowly typed outcome/custody handoff in the existing owners after the shared dependency boundary is safe. The source supports a repair; it does not yet support a verified or accepted verdict.

## 9. Immutable source identity index

All ranges above refer to the following paths at commit 78aa967cc9400bb5c207441b2a78dbf7021d596c. Aliases are report shorthand only.

| Alias / path | Git blob |
| --- | --- |
| AGENTS.md | e4682748087759b2d4bb2b3a47916b6148e8a6f2 |
| docs/specs/openclaw-rust-equivalence-migration.md | 69c9667af2328e20a1fba09ebf80c5aab4d5c336 |
| projects/openclaw-rust/SOURCE_OF_TRUTH.md | 3748d734b10c9ed5b2d948467fbaa9483f1dd306 |
| projects/openclaw-rust/docs/specs/final-assistant-media-custody.md | ec472b0ebc5b3a7aa86056c5d9d05eb4b6a94826 |
| projects/openclaw-rust/docs/specs/session-work-admission.md | 4292a12fa7340ebc5157c7253b58f4945e50dd0d |
| projects/openclaw-rust/docs/specs/channel-inbound-worker-handoff.md | bb27d60a7f85645afd6360ca1cc7aa54c4f9fd7a |
| projects/openclaw-rust/docs/specs/generated-send-message-fidelity.md | aed7099be9f381f62381e10c1af7fa8965ad29af |
| App — source/host/app/src/main.rs | ca038b1aa9f3a78a1951551e0a80d543142c08dd |
| Paths — source/host/src/attachment_paths.rs | c66a2d1ce8dfb70532a16857a0fe02d31788ca62 |
| DB — source/host/src/extensions/session/agent_db.rs | 78740768f7f1b0307dfd6756c4d50c368a91b721 |
| Session — source/host/src/extensions/session/production.rs | f585fae13720e08f34323bed82759253590a3753 |
| Runtime — source/host/src/extensions/transcript/production_runtime.rs | 1f336135cb1c521d16dc0d8ad9e42ef0cf6bccba |
| Pipeline — source/host/src/extensions/transcript/send_pipeline.rs | 12f58e2923cd909b9564b99e0a6393955a0945e0 |
| A2A — source/host/src/extensions/transcript/agent_to_agent_messaging.rs | e1b3769993c5086f3fbce654d431a42b9bfa1783 |
| Tool — source/host/src/runner/tools/send_message_tool.rs | 5a2afd591da79f24652b59eef2be6c05ecb1134c |
| Provider — source/host/src/extensions/inference/provider_session.rs | b2513672ca563e72efe0f4b56440b9aaf8f98c60 |
| Guard — source/host/src/runner/tools/mcp_meta_tools.rs | 923288198b898083cbfe5da9a1f76e9f309ee516 |
| Shell — source/host/src/runner/production_turn_run_shell_adapter.rs | d0fe2714923338dd717284f08f05eef438db69d9 |
| source/host/tests/agent_db_owner_contract.rs | 5d2e46e7c2bf507bb3822d1d75fdca83e8626a2a |
| source/host/tests/agent_session_contract.rs | fd10353b72f9a75970e2db1705e1da1f6180c646 |
| source/host/app/src/tests/channel_host_lane_tests.rs | ff4ebba71eae35f4be98a695654a0d33b84f1818 |
| .github/workflows/rust-desktop-runtime.yml | e705a065dc34f066af6a095bcc7475d36da1f4e2 |
| projects/openclaw-rust/scripts/capture_portable_contracts.py | e683bb4494c3d28ca8bc466472eb3d198578198e |

