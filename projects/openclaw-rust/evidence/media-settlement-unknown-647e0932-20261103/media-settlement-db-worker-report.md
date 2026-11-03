# MSU-v1 AgentDb owner implementation handoff

Task 647e0932-bea4-4113-9a61-65ae3db58955 · round 1 · 2026-11-03

## Status

**Implemented in the sole authorized production file; ready for root integration and focused review, with executable verification still pending.** No test, build, parser, lint, schema, capture, generator, test-list or acceptance command was run locally or on htch-runtime. No commit or push was made.

This work retains the configured model and the actually inherited xhigh setting specified by root. The scope needs that effort because it changes transaction outcome classification, connection retirement and real SQLite failure instrumentation.

Authorized production file:

- /workspace/scratch/7d27ea239237/media-settlement-work/source/host/src/extensions/session/agent_db.rs

Deliverables:

- /workspace/scratch/7d27ea239237/media-settlement-db-worker.patch
- /workspace/scratch/7d27ea239237/media-settlement-db-worker-report.md

No other production, Cargo, composition, schema, ledger, capture or specification file was edited by this worker.

## Authority and exact identities

The complete durable MSU-v1 specification was read before implementation:

/workspace/scratch/7d27ea239237/media-settlement-work/projects/openclaw-rust/docs/specs/media-settlement-unknown.md

Target base is **fdb0f6acb06165b6f2796961186e06a2de98c229**, tree **abeb6ffdf09b676279462fea3b9e1f6d72abc1b6**. The supplied AgentDb file matched the declared base blob and was byte-identical to the immutable 370c6680b6b8c072615875d72be67ca0e45ebe45 Git object. The immutable Git object was read from /workspace/scratch/29e7135d8bfe/fabushi-desktop; that checkout's mutable working bytes were not used as authority.

Fixed source U remains **1440bb1771372334bea23d7642bb52deb7db1845**, tree **90c8aab39972da11cfad220361781d49e1bc6700**. The prior bounded source discovery supplies the keep-on-unknown/native-evidence distinction; no new upstream refresh was performed.

| Artifact | Identity |
| --- | --- |
| Base agent_db.rs Git blob | 78740768f7f1b0307dfd6756c4d50c368a91b721 |
| Base source SHA-256 | 3257b66faf35ddcce6b4cba650390053002457a047b58023063e6eeab918eb82 |
| Base source size | 387,819 bytes / 9,374 lines |
| Changed agent_db.rs Git blob | 6c27e9bb9f665b1160457b9acc29dd5d64fba289 |
| Changed source SHA-256 | 9a8ad4aa952f11a3d7595eebf636f7d20c450381120416c00667b93ef0149c7b |
| Changed source size | 426,548 bytes / 10,214 lines |
| Patch SHA-256 | cc691a4377d744270632d4487195a970dafe7559b34c9132dd30fe16ca222906 |
| Patch size / delta | 42,881 bytes; 850 added lines, 10 removed lines |

The patch is a single-path unified Git patch against the declared base blob. The fact that its old bytes were acquired from 370 does not change its base content identity.

Additional immutable source read for the narrow implementation boundary:

| Source at 370 | Git blob | Use / read depth |
| --- | --- | --- |
| source/host/src/extensions/session/agent_db_serde.rs | d1885e2d31313c1f525fffd85bafc5031cf8d127 | Lines 110–195: actual transcript and hidden-control-row validity rules; the new fixture uses a valid send-message entry. |
| source/host/Cargo.toml | 9b477d438dd0a9325cbe52b670c2ada774f352e8 | Dependency declaration lookup only: rusqlite 0.39, bundled. No Cargo edit or feature implementation by this worker. |

Root AGENTS, applicable SOT governing text and main specification §17 were consulted through immutable Git. Historical SOT narratives were not treated as current acceptance evidence. This is not a full-file semantic review of the 10,214-line AgentDb owner or its transitive dependencies.

## Production behavior

### One existing transaction and owner

append_transcript_entries_and_release_media continues to use the same SQL transaction containing the existing:

- canonical session admission and metadata binding checks;
- lifecycle, archive and deletion guards;
- receipt replay identity/content checks;
- exact transcript validation and insertion;
- hidden outgoing-media-settlement receipt construction;
- compare-and-set update clearing the captured durable media fence.

The entire inner transaction closure is byte-identical to the baseline. There is no format, schema, receipt or fence migration, new owner, generic retry helper or reopen path.

### Outcome classification and retirement

The changed outer transaction handling distinguishes:

| Boundary | Returned outcome | Connection disposition |
| --- | --- | --- |
| Ordinary invalid input, closed owner, or clean BEGIN refusal | Existing ordinary error variant | No new transaction is treated as successful. Clean BEGIN refusal leaves the live owner usable. |
| A pre-existing transaction | TranscriptCommitUnknown | Retires this exact connection before returning. |
| Poisoned connection lock | TranscriptCommitUnknown | Takes the poisoned guard, retires the exact connection, then returns unknown. |
| BEGIN fails while leaving non-autocommit state | TranscriptCommitUnknown | Attempts cleanup, then retires regardless of cleanup success; no unsettled BEGIN state is reused. |
| Pre-COMMIT work fails and ROLLBACK succeeds with autocommit confirmed | Original ordinary error | Keeps the owner usable. |
| Pre-COMMIT work fails and ROLLBACK is not confirmed or autocommit is false | TranscriptCommitUnknown | Retires before returning. |
| Any COMMIT error, including lost acknowledgement after real successful COMMIT | TranscriptCommitUnknown | Retires even if autocommit is already true or subsequent rollback succeeds. |
| COMMIT reports success but leaves a live transaction | TranscriptCommitUnknown | Attempts cleanup and retires. |
| Confirmed COMMIT and autocommit | Original success/count | Publishes the existing success notifications. |

All unknown branches reuse retire_channel_inbound_connection. The helper itself is unchanged. There is no reopen, retry, fabricated receipt, successful transcript notification, admission notification or generation publication in an unknown branch. Normal success publication now occurs after releasing the DB connection mutex; the transaction's successful result and existing notification shape/count are preserved.

The existing AgentDbProjectionError enum and all its display strings are unchanged, including the existing TranscriptCommitUnknown display. No error-string matching classifies unknown.

### Test support interface

All public fault types, observations, the arm method, the one owner state field and its constructor initialization use cfg(any(test, feature = "test-support")). Root remains responsible for the default-off Cargo feature and App dev-dependency activation.

The interface is:

~~~rust
pub enum MediaSettlementTestFault {
    LoseCommitAcknowledgement,
    RefuseRollback,
}

pub fn arm_media_settlement_fault_for_test(
    &self,
    session_key: &str,
    lifecycle_revision: &str,
    work_id: &str,
    fault: MediaSettlementTestFault,
) -> Result<MediaSettlementFaultLease<'_>, AgentDbProjectionError>;
~~~

The borrowed lease has was_claimed() and was_applied(). Root's App tests should bind the Arc returned by the existing Session owner-open for the lease's lifetime; this port exposes no connection or arbitrary SQL executor.

There is one cfg-gated Mutex<Option<Arc<MediaSettlementFaultState>>> in SandAgentDb and one initialization in its existing sole constructor. The Arc allocation is the unique token: an old lease retaining its Arc cannot collide with a later allocation, and Drop compares pointer identity before removing an unconsumed slot.

Arm validates nonempty exact identifiers without surrounding whitespace or control characters, rejects occupied/closed/live-transaction owners, and takes the connection before the slot. Claim compares the supplied session/revision/work identities exactly, removes the matching slot under the connection boundary before BEGIN, and releases the slot before SQL. A mismatch does not consume it. A claimed fault cannot be reused by a later invocation. Lease Drop takes only the slot and never subsequently acquires the connection, so it creates no reversed lock order and cannot change durable/product custody.

The always-callable media_settlement_commands and MediaSettlementCommands helpers keep cfg statements outside append_transcript_entries_and_release_media. That method's existing three admission guard expressions and full inner body remain unchanged for the established AST oracle.

### Real command faults

LoseCommitAcknowledgement runs the real COMMIT first. Only after SQLite returns success does it mark the applied observation and return a rusqlite error in place of that acknowledgement. A real COMMIT error exits before the observation or injected acknowledgement error. The helper never returns TranscriptCommitUnknown; the production owner's normal transaction classification does.

RefuseRollback installs a scoped SQLite authorizer only around the actual ROLLBACK invocation. Its callback returns SQLITE_DENY only for SQLITE_TRANSACTION / ROLLBACK and marks applied only when SQLite actually requests authorization for that command. The guard retains its Arc user data and borrows the live connection. It removes the callback synchronously before the helper returns to the retirement decision. Other SQL, including BEGIN, SAVEPOINT, reads and COMMIT, is allowed.

ChannelInboundTransactionControl, its production caller and the channel transaction/outcome implementation were left untouched. No general fault framework, environment flag, new factory or mock unknown result was added.

## Requirement and test-design map

**All entries below describe implemented source and authored tests, not passing execution.** The new private module is media_settlement_tests. It uses file-backed databases, actual owner methods, separate SQLite readers, real send-message entries, persisted receipt/fence checks, original file bytes and observers for both transcript and admission notifications.

| Requirement | Implementation / authored oracle | Execution state |
| --- | --- | --- |
| MSU-01 | Existing inner transaction unchanged; explicit BEGIN, rollback and COMMIT classification; unconditional exact-connection retirement for every unknown branch | Pending exact-H Actions |
| MSU-06 | cfg-gated test interface/state; always-callable command helper; no cfg in the media method; original channel adapter preserved | Cargo integration, feature-off compile and AST oracle pending root / Actions |
| MSU-07 | One owner slot, exact three identities, Arc lease token, one claim before BEGIN, strict arm validation and token-scoped Drop | Covered by authored identity/lease tests; execution pending |
| MSU-08 | Real successful COMMIT before lost ACK; real authorizer-denied ROLLBACK with scoped removal; observations do not classify product outcomes | Both fault-application assertions authored; execution pending |
| MSU-09 | Thirteen private tests listed below; independent readers observe rows, hidden receipt and fence; owner usability/retirement and notifications checked | Pending exact-H Actions |
| MSU-02 / 03 / 04 / 05 / 10 | Typed Session/Runtime chain, local custody, App concurrency/tool behavior and real App recovery remain root/other assigned owners | Outside this file; not claimed implemented here |

### Named tests and causal purpose

Names below are declarations read from source, not the output of a test-list command.

| Test under media_settlement_tests | Causal design / independent oracle |
| --- | --- |
| media_settlement_success_and_exact_replay_preserve_durable_contract | Real successful append; manually specified exact receipt and cleared fence; exact replay produces zero new rows; conflicting content and stale revision remain refused; existing notification counts preserved. |
| media_settlement_known_refusal_rolls_back_a_real_receipt_failure_prefix | A SQLite trigger refuses receipt insertion only after observing the inserted send prefix in the same transaction. Normal rollback preserves the original fence, leaves no rows, emits nothing and permits a later explicit successful request. |
| media_settlement_lost_commit_acknowledgement_retires_without_publication | Real COMMIT precedes lost ACK. Both was_claimed and was_applied must be true. Independent reader finds committed send/receipt and cleared fence; owner is retired; zero notifications/generation changes; two fresh owner observations preserve durable state and bytes. |
| media_settlement_receipt_failure_with_real_rollback_denial_is_unknown | Same proven send prefix and receipt trigger failure; SQLite must actually deny ROLLBACK. Both fault observations asserted. Retirement removes the uncommitted prefix, preserves durable fence/bytes, emits nothing, and two fresh readers/owners observe the same durable state. |
| media_settlement_deferred_commit_failure_is_unknown_with_clean_or_denied_rollback | Deferred foreign-key violation makes real COMMIT fail after insert preparation. Runs clean rollback and authorizer-denied rollback cases; both remain unknown/retired. Checks original FK cause, zero child rows, no receipt/transcript, retained fence, zero publication. |
| media_settlement_real_commit_error_is_not_replaced_by_lost_ack_fault | Arms lost ACK but induces a real FK COMMIT failure. Claimed is true, applied must be false, and the real FK cause survives. |
| media_settlement_prior_transaction_retires_before_claiming_or_appending | Creates a real pre-existing transaction and inserted prefix on the owning connection. Media append returns unknown and retires without claiming/applying the armed fault or publishing. |
| media_settlement_poisoned_transaction_is_unknown_and_retired | Catches an actual unwind while the owner lock holds a real transaction; next media append retires the poisoned connection and returns unknown. |
| media_settlement_clean_begin_refusal_is_known_and_fault_is_one_shot | Independent writer holds BEGIN IMMEDIATE. Owner gets real SQLITE_BUSY with clean autocommit and ordinary error. The fault is claimed but not applied; after the external lock releases, a new explicit call succeeds without reusing it. |
| media_settlement_fault_slot_rejects_invalid_occupied_closed_and_unsettled_owners | Invalid/canonical-identity validation across all three fields, occupied slot refusal, unconsumed cancellation, live transaction and closed-owner refusal. Arming must not settle the existing transaction. |
| media_settlement_fault_matches_three_identities_and_only_its_exact_owner | Each identity mismatch leaves the slot unclaimed. A second real owner of the same file can commit the exact identities without consuming the first owner's slot. The first owner's later lost-ACK result retires only its connection; the second remains usable. |
| media_settlement_old_lease_drop_cannot_clear_a_new_arm | Drops an unconsumed lease, claims another on known refusal, arms a replacement, drops the old consumed token, then requires actual application of the replacement fault. |
| media_settlement_rollback_authorizer_denies_only_rollback_and_is_scoped | With the real authorizer installed, BEGIN/SAVEPOINT/read/COMMIT are allowed and actual ROLLBACK returns SQLITE_AUTH. After scope exit, BEGIN/ROLLBACK succeeds. This private helper control supplements the real owner-path denial tests. |

The DB tests' fresh owner observations are storage observations. They do not claim to exercise Session's first-open media recovery or App file-deletion policy; MSU-10 owns those actual consumer proofs.

## Source preservation checks

Only source reads, byte comparisons, content hashes and a unified diff were computed locally. These facts are not substitutions for the required AST/admission tests.

| Preserved source block | Byte-identical to baseline | SHA-256 of preserved bytes |
| --- | --- | --- |
| Complete media inner transaction closure | Yes | 81048517fa91eaedc1699d7d52707f6570a1bdc153dc4d40d76b11b9cc027811 |
| ChannelInboundTransactionControl | Yes | b7ae6bab92ebca0589f2a9dd2bb8bb4b2f054fc1dee0af0966aa81969b2b3cb0 |
| Channel transaction/failure outcome functions | Yes | 11aee146934fb3d47193b5960895c49e0949a7a27128b27dfb3357352bc81a69 |
| Channel deferred method/caller block | Yes | 050853c4bc0901bf89b2b62b5b8d452cbe7a261708069e0d07ffa8c3b9661b17 |
| Existing exact-connection retirement helper | Yes | 56c073551c9d69530ba3933fff115dc41915511b370951e1d4c9454d866db547 |
| Media receipt builder | Yes | 670737f0de9d7b93b36f21bffaad4623fdaf5c8233ca839445f0db3cac17ac0c |
| AgentDbProjectionError enum/displays | Yes | c12752c3088546b715c2cfb9d5406142d93fda2462b544c98697302e07ec9e59 |
| Existing original-terminal and channel tests | Yes | e662137f93426bf4a8e1f6a2d50db1ac40716970edeca013a689c30630982baa |
| Existing fresh-append test module | Yes | 719da174f726cdcd8ccc830e25e3b3bc46c164c4917d0990b906980f707b4560 |

The new source and complete production diff were manually read. Existing constructor/connection lifecycle, media method, retirement helper, channel precedent, fresh-append failure tests, receipt builder, admission helpers and notification consumers were inspected. Reading names or hashing the rest of the large file is not claimed as a full semantic review.

## Integration and remaining verification

Root must integrate the default-off Host test-support feature and App dev-dependency, Session/Runtime typed returns and App/custody changes as one MSU-v1 batch. The borrowed lease interface and exact named tests above are available for that integration. No shared contract change was needed in this worker.

After freezing the resulting H, required Actions work remains:

1. Compile the affected Host and App paths, including ordinary shipping App without test-support.
2. Execute the new media_settlement_tests module and the applicable existing admission AST, original terminal, channel and fresh-append regressions.
3. Execute root's real Session/Runtime/App/Runner controls with explicit was_claimed and was_applied assertions for both fault kinds.
4. Preserve failures and classify implementation/oracle/environment causes, then perform focused review and only invalidated rechecks.
5. Bind evidence to the exact newly frozen H, platforms, inputs, command exits, counts and logs.

No executable result, test count, platform success, independent review approval, source-closure state or migration acceptance is asserted by this implementation handoff.
