# Media settlement: App fault-port and implementation dependencies

Task: 647e0932-bea4-4113-9a61-65ae3db58955 — bounded read-only continuation of MSU-01  
Date: 2026-11-03  
Reviewer: media_settlement_unknown_audit; configured model retained; inherited actual xhigh, appropriate for transaction outcome, connection lifetime, filesystem custody and recovery.  
Status: **design recommendation only; no implementation, execution or acceptance claim**. Root owns the final durable specification, public contracts, Cargo/composition changes and ledger.

## 1. Exact subject and decision

Repository: `bhrumom/fabushi-desktop`; assigned migration branch: `codex/openclaw-rust-fusion-chat-20261101-writer`. Inspected immutable target **370c6680b6b8c072615875d72be67ca0e45ebe45**, tree **b31903209a22e8fbd77e5eb4a9a865723a98f54c**. Git object inspection confirmed these identities. Fixed upstream U remains **1440bb1771372334bea23d7642bb52deb7db1845**; no upstream rebaseline or additional source-reading credit is claimed.

Repository reads used immutable Git objects in `/workspace/scratch/29e7135d8bfe/fabushi-desktop`, never its worktree. The supplied frozen R6 App file was read only around the sink and resolver, with declared blob **4831ae8cf002ce64e6409f2d695b48aaa1914c45**. That overlay changes resolver error propagation; it does not change the pending-custody fields or the settlement classification examined here. Its identity is the integrator-provided snapshot identity, not a new independently computed file-hash claim. No mutable R5, other review report or root arbitration was read.

Root AGENTS and current SOT/main §17 were checked. Their blobs, and the applicable final-media, session-work, fresh-append and generated-send fidelity specifications, are unchanged from the preceding audit. Their prior substantive readings remain applicable under §17.1. This continuation does not replace the durable spec required before implementation.

**Recommended dependency decision:** expose a default-off Host `test-support` feature through the App's dev-dependency; place one exact-owner, exact-session/revision/work, single-use fault slot on the existing `SandAgentDb`; access it through the existing Session `open_agent_db_owner` result. Intercept only the real media transaction's command result or real ROLLBACK authorization. Keep production outcome classification in the existing DB owner, propagate its existing typed unknown through the two existing facades, and retain local custody whenever settlement is unresolved.

Do not export the connection, an arbitrary SQL/closure executor, a replacement Session/Transcript implementation, a general fault registry, an environment-controlled effect flag or a second persistence owner.

### Source notation

All ranges below refer to this target unless marked R6:

| Alias | Path |
| --- | --- |
| DB | `source/host/src/extensions/session/agent_db.rs` |
| Session | `source/host/src/extensions/session/production.rs` |
| Runtime | `source/host/src/extensions/transcript/production_runtime.rs` |
| App | `source/host/app/src/main.rs` |
| Paths | `source/host/src/attachment_paths.rs` |
| App tests | `source/host/app/src/tests/channel_host_lane_tests.rs` |
| Oracle | `source/host/tests/openclaw_sessions_and_config_contract.rs` |
| Tool | `source/host/src/runner/tools/send_message_tool.rs` |

## 2. Existing precedents and compile reach

The Host manifest has no `[features]`; App has a normal path dependency on `mahayana-host-runtime` and only `rusqlite` in its dev-dependencies. Both packages declare edition 2024. No ancestor repository Cargo workspace manifest was found on their directory chain.

A matching repository precedent already exists:

- `source/mahayana/mahayana-rs/mahayana-host/Cargo.toml:12–22` declares `test-support = []` outside its defaults.
- `source/mahayana/mahayana-rs/mahayana-feature-host/Cargo.toml:31,40–43` uses the same package normally and enables `test-support` only on the dev-dependency.
- `mahayana-host/src/lib.rs:208–213` exposes a `#[doc(hidden)]`, feature-gated test entry point.
- Inside the Host runtime, `extensions/mcp/plugin_skills.rs:453–464,489–505` already places test-only state on an individual owner and initializes it in that owner's constructor. It is a field-placement precedent, not a suitable transaction injector.

Proposed manifest fragments, **not applied**:

```toml
# source/host/Cargo.toml
[features]
default = []
test-support = []
```

```toml
# source/host/app/Cargo.toml
# Keep the existing normal dependency unchanged.
[dev-dependencies]
mahayana-host-runtime = { path = "..", features = ["test-support"] }
rusqlite = { version = "0.39", features = ["bundled"] }
```

Gate the Host's test enum, armed slot, lease, diagnostics and setter with `#[cfg(any(test, feature = "test-support"))]`. Host library unit tests get `cfg(test)`; App tests need the dependency feature because the Host library is a dependency, not the App's test crate. The App test module already has test reach through `App:17949`; it should not use `cfg(feature = "test-support")` as though the dependency's feature were an App feature.

Cargo's documented resolver behavior supports this arrangement: dev-dependency features participate when building targets that need dev-dependencies, and are unified for test/all-target builds. A normal bin-only build does not activate that dev feature merely because it is listed. Edition 2024 defaults to resolver 3; the workspace root controls resolver selection. Sources: [Cargo feature resolver](https://doc.rust-lang.org/cargo/reference/features.html#feature-resolver-version-2) and [Cargo resolver versions/unification](https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions). These are compile expectations, not executed proof.

The existing workflow already has a normal App check at `.github/workflows/rust-desktop-runtime.yml:150`:

```text
cargo check --locked --profile ci --bin mahayana-app-host --manifest-path source/host/app/Cargo.toml
```

Retain a separate normal-target Actions check without the feature, plus App test compilation/execution with the dev feature. A test build, `--all-targets`, or a release profile alone does not prove absence of the test surface. Do not move the feature into normal dependencies or default features to repair a test compile. No manifest resolution, Cargo metadata, build or test was executed here.

## 3. Exact owner-local test access

### Location and interface

`SandAgentDb` holds the real connection at **DB:883–891**. Its public `open`, existing-agent and options constructors converge on **`open_with_options_and_seed`, DB:952–985**, whose sole `Self` initializer is at 970–978. Add the test-only field there, initialized empty; no production constructor argument changes are needed.

Illustrative interface shape, **design text, not compiled code**:

```rust
#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
pub enum MediaSettlementTestFault {
    LoseCommitAcknowledgement,
    RefuseRollback,
}

// Additional private SandAgentDb field, under the same gate:
media_settlement_test_fault: Mutex<Option<ArmedMediaSettlementFault>>,

// On SandAgentDb, under the same gate and #[doc(hidden)]:
pub fn arm_media_settlement_fault_for_test(
    &self,
    expected_session_key: &str,
    expected_lifecycle_revision: &str,
    media_work_id: &str,
    fault: MediaSettlementTestFault,
) -> Result<MediaSettlementFaultLease<'_>, AgentDbProjectionError>;
```

`ArmedMediaSettlementFault` needs only the three captured identifiers, the selected fault, a unique lease token, and per-attempt observation state. The lease borrows this exact owner and owns its token. Its diagnostics distinguish *armed*, *claimed*, and *fault actually applied*; observations must never feed the production outcome decision. A small per-lease state/counter is sufficient. This is not a process-wide switch or an outcome mock.

The App fixture already has the required path:

```rust
let owner = sessions.open_agent_db_owner(&agent_id)?;
let lease = owner.arm_media_settlement_fault_for_test(
    &session_key, &lifecycle_revision,
    &format!("send-media:{tool_call_id}"),
    MediaSettlementTestFault::LoseCommitAcknowledgement,
)?;
// Drive the existing App sink / generated Runner, retaining owner and lease.
```

`Session:1640–1652` returns `Arc<SandAgentDb>` under the existing filesystem boundary. The cached path at **1687–1694** returns the same owner. Do not create a second `SandAgentDb` connection just to install the fault, and do not call the public owner-open API while already holding the non-reentrant Session filesystem mutex. No new Session or Runtime test setter/factory is necessary.

### Arming, consumption and cleanup contract

1. Arming rejects empty/noncanonical identifiers, a closed/unavailable or already-unsettled connection, and an occupied slot. It does not replace an existing arm. It grants no production admission; the existing media transaction still performs all its normal identity and generation checks.
2. A fixture may arm the expected work before attachment resolution, so the actual producer still materializes and captures custody. It obtains session/revision from the existing Session snapshot. Requiring the work already be captured would obstruct the actual generated producer or tempt a duplicate manual resolution.
3. At entry to the actual media transaction, under the connection mutex, claim the slot only if **session key, lifecycle revision and media work ID all exactly match**. A mismatch leaves the arm untouched and uses ordinary commands. Claim before attempting BEGIN; any later refusal consumes this attempt without rearming it.
4. Move the claimed fault into the invocation's private command context. There is no retained next-call flag after consumption, no wildcard and no retargeting by tool name. A subsequent call with the same work ID cannot consume the same fault again.
5. A lease dropped before consumption clears only its own token. An old consumed lease must not clear a newer arm, even if a later fixture reuses the same work string. A unique token is essential for this case.
6. Lease cleanup touches only the test slot. It does not release media admission, delete files, reopen a connection or attempt another SQL transaction. After consumption, the local command context owns any temporary authorizer; dropping the lease cannot undo an active command.
7. All access follows one lock order: connection mutex before fault-slot mutex when both are needed. Lease Drop needs only the slot. Never retain the slot mutex across SQL, callbacks, publication or external fixture code.
8. A retired owner cannot be armed or execute another attempt. A newly opened owner starts with an empty slot. Unused arms cannot survive fixture-owner destruction or leak into another owner with the same path.

### Two real command-edge faults

**Lost COMMIT acknowledgment.** Keep `db.execute_batch("COMMIT")` in the media owner's explicit settlement flow. Pass its actual `Result` to a narrow helper that ordinarily returns it unchanged. Only a claimed `LoseCommitAcknowledgement` may replace **an actual `Ok(())`** with an error, matching the existing channel mechanism at **DB:6094–6100,8753–8774**. Preserve a real COMMIT error unchanged. Mark the test observation applied only after actual success. The normal media owner must decide typed unknown and publication suppression; the test helper must not return `TranscriptCommitUnknown` itself or fabricate durable rows.

**Real ROLLBACK refusal.** Reuse the static SQLite authorizer mechanism from **DB:8908–8938**: deny only `SQLITE_TRANSACTION` with operation `ROLLBACK`, returning `SQLITE_OK` for everything else. Install it on the owning connection only around the selected invocation's actual ROLLBACK execution. The callback retains no SQLite pointers, has no borrowed user data, and never re-enters SQLite. A scoped guard removes it before the connection can be retired or the helper returns. The actual SQL command must return the authorization failure; production checks `is_autocommit()` and performs existing exact-connection retirement.

The channel control's `fail_rollback` at **DB:6103–6108** returns a synthetic error before issuing SQL. It is not evidence for this requested real-refusal case and should not be the App injector. A failed authorizer installation or unexpected command result must fail the fixture's applied-fault assertion, not be reported as a successful fault exercise. Ensure authorizer teardown occurs before `connection.take()` so no guard holds a raw handle to a dropped connection.

No generic same-owner SQL executor is needed for the two App cases. A fixture can create an ordinary persisted SQL trigger through its existing fixture inspection connection before driving the App. The authorizer itself must be on the cached owner. A separate connection's `PRAGMA foreign_keys` would not configure that owner and must not be used as proof of a same-owner deferred-COMMIT failure.

## 4. Existing source contracts constrain helper placement

Two current oracles materially affect the design:

- **Oracle:1568–1598,1745–1818,1833–1869** pins the complete `ChannelInboundTransactionControl::commit` implementation and exactly one nonempty `.commit(...)` invocation returning to its checked channel transaction owner. Widening its `cfg(test)`, adding a second caller, or renaming it would require unrelated channel-oracle changes. Keep that adapter intact for this batch.
- **Oracle:723–760,1125–1144,1149–1194** checks three admission-identity guards owned by `append_transcript_entries_and_release_media`. Its AST walk accepts only doc attributes inside that owner body and rejects conditional local/expression attributes. Direct `#[cfg(test)]` or `#[cfg(feature = "test-support")]` statements inside the media method would invalidate the current oracle even if the guards were textually retained.

Use an always-callable, narrowly scoped media command context whose feature-dependent implementation/state lives outside the media owner body. In a normal build it has no fault state; after-COMMIT handling is identity and rollback issues real SQL. The media method calls it without conditional statements, retains its three admission/replay guards and explicitly handles every command outcome.

An illustrative command result boundary is:

```rust
let commit_result = media_commands.after_commit(db.execute_batch("COMMIT"));
// Existing owner, with repaired semantics, handles this Result explicitly.
```

This keeps real COMMIT execution and outcome ownership visible and adds no new raw `COMMIT?` escape. The helper does not need to borrow the channel adapter, generalize transactions across unrelated owners, or relocate media admission into another function. Any deliberate oracle amendment remains root-owned and must retain its negative fixtures. **No parser or oracle was run**; these constraints are source observations.

## 5. The two typed facade changes and all direct callers

Immutable source searches found the following complete direct-call graph for these two methods; there are no additional code/test callers in the target tree:

| API changing return type | Current signature/range | Sole direct caller | Required adaptation |
| --- | --- | --- | --- |
| `ProductionSessionWorkers::append_agent_transcript_entries_and_release_media` | `Result<usize, String>`, Session 4062–4080 | `ProductionTranscriptRuntime::append_generated_send_message_entry_with_media_custody`, Runtime 1826–1832 | Preserve `AgentDbProjectionError`, including `TranscriptCommitUnknown`, across the filesystem closure. |
| `ProductionTranscriptRuntime::append_generated_send_message_entry_with_media_custody` | `Result<Value, String>`, Runtime 1797–1833 | `ProductionSendMessageSink::send_message`, App 1032–1042; frozen R6 call starts at 1036 | Match typed unknown before any ordinary cleanup or string formatting. |

Recommended signatures are respectively `Result<usize, AgentDbProjectionError>` and `Result<Value, AgentDbProjectionError>`. The DB method already returns this error type; no new serialized error/schema is needed.

Session should use the existing nested-result pattern at **Session:3930–3945**: the filesystem closure returns `Ok(owner_operation_result)`, its own pre-operation admission/open failures become `WorkAdmissionInvalid`, and the inner DB error remains typed. Do not change the generic filesystem mutation helper or stringify the inner result.

Runtime's history read at 1809 and generated-ID validation at 1821–1824 occur before this append attempt. Map those failures explicitly to appropriate existing admission/append-invalid errors; propagate the typed Session result with `?`. They are not evidence of an attempted commit. A catch-all `From<String>` conversion or string search for “unknown” is unnecessary.

App currently unifies media, bound non-media and compatibility branches into `Result<Value, String>`. Avoid forcing changes to the non-media APIs: classify the media branch's typed result into `ProviderSessionError`, map the existing non-media strings at their own branches, then apply cleanup only for a known precommit/refused result. Preserve existing `ProviderSessionError::ToolOutcomeUnknown` through the Tool bridge. Its guard at `runner/tools/mcp_meta_tools.rs:198–207` marks `DownstreamOutcomeUnknown` before completing the receipt or downstream serialization.

`TranscriptCommitUnknown` currently displays “fresh transcript commit…” (**DB:65–66**). Reusing the variant is correct; a narrow wording adjustment can remove the misleading “fresh” qualifier. That is an error-text compatibility decision for root, not a new error family. The fresh-append planner itself is not a drop-in replacement: media admission, hidden receipt and fence CAS must remain atomic in this existing media owner.

## 6. Minimal explicit custody disposition and retry/reopen contract

### Current facts

`PendingOutgoingMedia` has only custody plus optional session/revision/work fields (**App:721–726; R6:724 onward**). It does not retain an unknown disposition. `abort_attachment_custody` removes it, rolls back files, then releases the fence (**App:965–991; R6:969 onward**). `OutgoingMediaCustody` uses a `committed` boolean and Drop calls rollback (**Paths:21–63**).

There are three Tool pre-send abort sites: failed message construction, failed wire encoding and awaiting-user refusal (**Tool:409–447**). Counting delegates abort unchanged (**162–164**). The sink adds admission, append-error and post-append missing-ID abort paths (**App:1014–1016,1068–1084**). The Tool bridge's final send-error arm returns the original error (**470–481**); it does not itself abort after that call.

Session capture permits more paths for the same work ID (**Session:2722–2746**). After actual COMMIT, the durable media fence is cleared. Therefore simply removing the local pending entry after unknown, or merely skipping one explicit abort, would permit local re-entry or later Drop deletion.

### Recommended local disposition

Replace the implicit boolean meaning with an explicit local cleanup disposition. A compact design has:

| Disposition | Ordinary abort / Drop | Transition authority |
| --- | --- | --- |
| `RollbackEligible` (default) | May remove tracked files. | Materialization before owner settlement; known pre-attempt failure. |
| `OwnerSettlementPending` | Must retain files and must not release the durable fence. | Enter immediately before handing an admitted media attempt to the Session/Runtime append. Retain on typed unknown or unwinding before a definitive result. |
| `Committed` | Must retain files; local path list may be cleared. | Only a confirmed successful owner return. |

`OwnerSettlementPending` deliberately covers both the synchronous attempt in progress and an unknown returned outcome. This adds one cleanup disposition, avoids using `commit()` to mean uncertainty, and protects Drop before the App can process an uncertain result. A definite precommit refusal may explicitly restore rollback eligibility and perform the existing cleanup/release. A COMMIT error may never make that transition merely because a later ROLLBACK succeeded.

Suggested local methods are a begin/retain-for-owner-settlement operation, a disposition query, and an explicit cleanup-after-confirmed-refusal operation. These are API design names, not an implemented contract. Existing `track`/`absorb` users must only add paths while rollback-eligible. The Session recovery code creates a fresh default handle at **Session:1802–1810**, so its verified cleanup remains eligible and does not inherit the old App handle's retained state.

App obligations:

1. Keep the exact pending entry as a per-tool-call tombstone after unknown. Do not remove it, clear its identifiers, call local rollback or call `release_session_media_admission`.
2. Check this disposition before **both** attachment resolution/materialization and send admission/history allocation. Repeated calls using the same tool ID return `ToolOutcomeUnknown`; the attachment path must not create another file or append another durable path first. This includes retrying with text/no attachment.
3. `abort_attachment_custody` must inspect before removing. For owner-settlement-pending/unknown it is a no-op, including when called indirectly by the existing Tool build/encoding/refusal arms. Recover a poisoned local mutex for safe disposition handling rather than dropping a cleanup-eligible unknown handle.
4. Seal the custody before entering the synchronous owner append and release the local map mutex before doing DB/Session work. This prevents an abort or unwound sink teardown from deleting files while the result is unresolved. Only the same invocation's known refusal restores cleanup eligibility.
5. On success, commit and remove the pending handle before normal external delivery/event/ACK success paths. Never abort after a confirmed append. The current missing-ID fallback is precluded by Runtime's validation, but must not remain a destructive post-success escape if the contract changes; retain conservatively and return unknown on an impossible result-shape violation.
6. `track`/`absorb` must not reopen a sealed entry. A pre-resolution check alone is not atomic with a later absorb. The final implementation must either enforce existing single-producer-per-tool-ID use or recheck/reserve that same entry so a concurrent duplicate cannot attach new paths to a settling work item. Do not claim concurrent same-ID serialization from the proposed sequential retry test alone.

The sink's ordinary destruction then drops retained handles without deleting bytes. This local tombstone lives only as long as the sink; it is not a durable replay registry. Across process restart, the existing durable Session receipt/fence and existing Runner unknown-effect custody remain authoritative. A fresh producer must not infer permission to resend from the absence of the old in-memory map.

### Reopen behavior to preserve

- Actual committed/lost-ACK settlement: send and receipt exist, pending fence is clear; retain bytes across sink drop and fresh owner reads. No duplicate send or reconstructed file is required.
- Rolled-back/retired uncommitted settlement: durable work/path input remains; after safe paired retirement and ordinary first open, existing recovery verifies paths and removes only owned, provably uncommitted files (**Session:1757–1820**).
- Unknown/malformed paths or identity remain fenced. Strict first-open refuses pending media without caching it (**1703–1712**).
- Cached owner reads do not run media recovery (**1687–1694**). The exact DB retirement helper (**DB:4516–4524**) and Session paired owner retirement (**Session:2025–2051**) are different operations. No automatic replacement owner or automatic tool replay is introduced by this fix.

## 7. Causal App tests and existing fixture placement

Use **App tests:397–472** `ChannelHostLaneFixture`, which owns actual Session, transcript runtime and private App sink. Its `dispose_owners` drops the sink first (**858–862**), making it useful for the destructor oracle; `stop_for_reopen` and bounded lane shutdown already exist (**865–885**). Do not use the fixture's convenience `send()` for fault cases: it assumes success and waits for external channel dispatch (**564–590**).

Place narrow sink/abort/re-entry controls beside this fixture, and use the existing `generated_send_message_fidelity` module (**1538–2320**) for actual generated Runner closure. That module already transfers the concrete App sink into `CountingSendMessageSink` and `TurnAgentComposition` (**2035–2077**) and reopens twice (**2181–2217**). Arm the cached owner after its real admission snapshot and before producer dispatch. If retaining a concrete `Arc<ProductionSendMessageSink>` for explicit abort/retry assertions, drop that extra Arc before claiming sink-Drop evidence.

For new generated fault cases, prefer one real attachment tool call with a known call ID. The existing fixture currently assumes two successful sends in its SQL/wire/history/count checks. Add explicit fault-case expectations; do not weaken the three existing SGF positive/provider-refusal cases or call their success-only history oracle unchanged. Controlled provider input remains an external fixture input; Session, App sink, transaction, effect guard and recorder remain production code.

| Proposed case | Real cause and decisive oracle |
| --- | --- |
| Actual COMMIT, acknowledgment lost | Arm only the target work. Drive real attachment materialization and App send. Prove fault applied after actual success; independent SQL shows send plus hidden receipt and cleared fence; App sink returns typed unknown, target delivery count is zero, no App success/ACK/channel dispatch occurs; owned bytes survive explicit abort, same-ID retry and all sink handles dropping. Two ordinary fresh owners preserve the rows and bytes without replay. |
| Actual inserted send, receipt refusal, ROLLBACK denied | Before the call, create a fixture-local `BEFORE INSERT` trigger matching the exact hidden receipt ID. Its expected failure branch must require that the receipt's referenced send row already exists in this transaction, proving a real inserted prefix. Arm `RefuseRollback`. Observe actual SQLite authorization failure on ROLLBACK, typed unknown and retirement of the exact cached DB. No same-connection prefix remains consumable; a separate reader sees no committed target send/receipt. Bytes and durable pending work/path remain until ordinary owner recovery. |
| Same receipt refusal, normal ROLLBACK | Use the identical trigger without the rollback fault. Ordinary refusal, clean connection, no target send/receipt, local cleanup and matching fence release remain allowed. This is the control distinguishing confirmed refusal from unknown. |
| Production guard/terminal consumer | Drive the actual App fault through the existing generated Runner composition. Observe downstream-unknown custody, no automatic retry or successful completion authority, and no fabricated SendMessage success result. Provider serialization can change the outer error representation; assert the sticky owner observation and actual terminal semantics, not an unverified requirement that every provider preserves the same enum variant. |
| Fault isolation and lease cleanup | Different owner, different work or revision cannot consume the arm. A consumed arm cannot fire twice. An unconsumed lease drop clears only itself, and a consumed old lease cannot clear a newer token. None of these operations release media work or delete files. |
| Abort, Drop, retry and recovery | A later abort and repeated resolution/send cannot delete retained bytes, allocate another entry, recapture paths or report success. Dropping all concrete/wrapper/Runner sink references is required. Reopen cleans only the uncommitted case and preserves source files, unrelated siblings and the committed case. |

A trigger can use the incoming receipt's `transcriptEntryIds[0]` to check the actual preceding send row, rather than hard-coding a guessed generated message ID. A fixture that never reaches receipt insertion or never actually denies ROLLBACK is non-passing even if the returned error happens to be unknown.

Existing fresh-append tests already cover real same-owner deferred COMMIT failure and rollback refusal (**DB:9091–9185**). Extend the media owner unit tests using the same real SQLite mechanisms for COMMIT-failure-plus-clean-rollback and COMMIT-failure-plus-refused-rollback. The two App ports above do not require a third public arbitrary-SQL facility. Unit and App results must remain separately identified; do not label composed evidence as a different fault executed end to end.

All execution belongs in Actions. Current workflow exact App selectors at **1366–1371**, required-name mapping at **1925–1927**, and subsequent expected-name/command sets around **4234–4236** need root-owned additions for new tests. Existing selectors do not discover a new exact test name automatically. Preserve the normal App check and the affected owner/admission/oracle checks. This report supplies proposed causes and oracles, not test results or an executable campaign receipt.

## 8. Remaining decisions and bounded handoff

The inspection resolves the cross-crate test-access question: **a narrow default-off feature and owner-bound single-use command seam are feasible without new Session factories or persistence schema**. It also resolves the direct caller set and identifies both existing AST constraints before implementation.

The final durable spec still needs root decisions on:

- Exact public test-support names and lease observation type; preserve the owner/session/revision/work/token constraints above.
- The explicit local custody disposition and its seal/known-refusal transition, including whether concurrent reuse of a tool ID is prevented by producer contract or by local reservation. Sequential unknown retry refusal alone does not settle that concurrency claim.
- The final App generated fault-case expectations: existing SGF history helpers assume successful sends and cannot simply be reused for an unknown/retired-owner outcome. Determine retained error history and terminal checks from the actual production consumer behavior.
- Publication plumbing if the DB settlement repair changes it. Current media notifications occur under the DB and Session mutation boundaries; fresh append defers publication. Moving publication must preserve both transcript and admission observations and their existing consumers. The fault port itself does not require inventing another transaction owner or changing persisted receipt format.
- Exact freeze H after LFA publication, allowed paths, source-contract amendments if any, and Actions selection/provenance. This assessment is bound to the stated immutable target plus frozen R6 excerpt, not future merged source.

No production file, Cargo manifest, shared contract, ledger or public schema was edited. No tests, parser, lint, build, capture, generator or fault injection was executed locally, on htch-runtime, or in Actions. Only this report was authored. The original MSU-01 remains a source-confirmed outcome-classification finding; the proposed faults have not been reproduced in this assessment.

### Exact blob index for changed-boundary decisions

| Source | Blob at target |
| --- | --- |
| Root AGENTS | `e4682748087759b2d4bb2b3a47916b6148e8a6f2` |
| Main OCRM spec | `69c9667af2328e20a1fba09ebf80c5aab4d5c336` |
| OCRM SOT | `3748d734b10c9ed5b2d948467fbaa9483f1dd306` |
| Final-media spec | `ec472b0ebc5b3a7aa86056c5d9d05eb4b6a94826` |
| Session-work spec | `4292a12fa7340ebc5157c7253b58f4945e50dd0d` |
| Fresh-append/channel handoff spec | `bb27d60a7f85645afd6360ca1cc7aa54c4f9fd7a` |
| Generated-send fidelity spec | `aed7099be9f381f62381e10c1af7fa8965ad29af` |
| Host Cargo | `9b477d438dd0a9325cbe52b670c2ada774f352e8` |
| App Cargo | `4cd51b5e35192f3f8ad1e6dff3699ec9a63e8928` |
| DB | `78740768f7f1b0307dfd6756c4d50c368a91b721` |
| Session | `f585fae13720e08f34323bed82759253590a3753` |
| Runtime | `1f336135cb1c521d16dc0d8ad9e42ef0cf6bccba` |
| Paths | `c66a2d1ce8dfb70532a16857a0fe02d31788ca62` |
| Target App | `ca038b1aa9f3a78a1951551e0a80d543142c08dd` |
| Frozen R6 App overlay, supplied identity | `4831ae8cf002ce64e6409f2d695b48aaa1914c45` |
| App fixture | `ff4ebba71eae35f4be98a695654a0d33b84f1818` |
| Source-contract Oracle | `2049875451202ba05aecb7e5de9a6841dc378d63` |

Primary immutable source links: [DB owner and constructor](https://github.com/bhrumom/fabushi-desktop/blob/370c6680b6b8c072615875d72be67ca0e45ebe45/source/host/src/extensions/session/agent_db.rs#L883-L985), [media settlement](https://github.com/bhrumom/fabushi-desktop/blob/370c6680b6b8c072615875d72be67ca0e45ebe45/source/host/src/extensions/session/agent_db.rs#L4660-L4919), [source-contract oracles](https://github.com/bhrumom/fabushi-desktop/blob/370c6680b6b8c072615875d72be67ca0e45ebe45/source/host/tests/openclaw_sessions_and_config_contract.rs#L723-L760), [App fixture](https://github.com/bhrumom/fabushi-desktop/blob/370c6680b6b8c072615875d72be67ca0e45ebe45/source/host/app/src/tests/channel_host_lane_tests.rs#L397-L472).
