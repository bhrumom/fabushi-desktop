# Original Hfc terminal recovery assessment

Date: 2026-11-03 UTC. Scope: independent read-only diagnosis of the original Hfc Linux failure and the two proposed R6 test repairs. This assessment does not implement R6 or authorize a new workflow trigger.

## Decision

**Both reported failures are independently confirmed in the original Actions job, and both require test repairs.** The evidence-only carrier `4e80e161c6cc539f4c120d64f91421619a57def1` did not introduce these observed failures: the original `fc187ecba404e2fa95d65871301aa8d83c7f0314` job already recorded them at 05:19:58 and 05:30:22 UTC.

- **MSU-ROOT-R6-001 — TEST_FLAW.** The App test reads a nonexistent field on `PersistedSendContext`. The existing outer `RoutedSendAcceptance` owns `duplicate`. Preserve the durable-acceptance and nonduplicate predicates, changing only the latter's receiver to `accepted.duplicate`.
- **MSU-ROOT-R6-002 — TEST_FLAW.** The background-wakes test requires an obsolete whole-file rollback-loop spelling and an obsolete adjacent diagnostic spelling. The shipping sink still owns cleanup and error reporting through its phase-aware helper. Replace these two stale literal checks with bounded checks of the actual sink/helper/abort path, retaining its phase and admission-release responsibilities and every unrelated assertion.

The original portable receipt rejected its command with Cargo exit 101. The original App receipt also returned Cargo exit 101 and recorded no passing App test names. Neither source inspection nor the proposed repairs establish a passing campaign. The independent R6 implementation should be integrated and reviewed before root freezes any new execution subject.

## 1. Exact original execution and preserved primary evidence

| Identity | Original value |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| Subject Hfc | `fc187ecba404e2fa95d65871301aa8d83c7f0314` |
| Subject tree | `410414bcf3ad042cf49aeed86ce55836f4fbc1cd` |
| Workflow | `.github/workflows/rust-desktop-runtime.yml` |
| Run / attempt | `41582614116` / `1` |
| Linux job | `125740667771`, `sqlite-native-path` |
| Exact job status | `completed`, conclusion `failure` |
| Job interval | `2026-11-03T05:13:14Z` to `2026-11-03T05:30:26Z` |
| Terminal artifact | `13207615800` |
| Artifact name | `ocrm-sqlite-native-path-fc187ecba404e2fa95d65871301aa8d83c7f0314-41582614116-1` |
| Artifact size | `493564` bytes |
| Artifact SHA256 from original artifacts API | `67e6c96370531240a7d079a500b77860857f1bd144a6e62ab8d74d0ffa79b3c5` |

Original service references:

- [Original run](https://github.com/bhrumom/fabushi-desktop/actions/runs/41582614116).
- [Original Linux job](https://github.com/bhrumom/fabushi-desktop/actions/runs/41582614116/job/125740667771).
- [Exact-attempt jobs API](https://api.github.com/repos/bhrumom/fabushi-desktop/actions/runs/41582614116/attempts/1/jobs?per_page=100).
- [Original run artifacts API](https://api.github.com/repos/bhrumom/fabushi-desktop/actions/runs/41582614116/artifacts).
- [Terminal artifact](https://github.com/bhrumom/fabushi-desktop/actions/runs/41582614116/artifacts/13207615800).

I fetched the original job log and the exact-attempt jobs/artifacts metadata through the GitHub connector. They are preserved under [fc-terminal-recovery](sandbox:/workspace/scratch/29e7135d8bfe/reviews/fc-terminal-recovery/evidence-index.json). These are original-service reads, independent of the incoming carrier's classification.

| Preserved file | Bytes | Independently computed local SHA256 |
| --- | ---: | --- |
| [job-125740667771.original.log](sandbox:/workspace/scratch/29e7135d8bfe/reviews/fc-terminal-recovery/job-125740667771.original.log) | 4,989,438 | `bc2980e3a6f52dea60fe20f057ad3309d574e643e6c583b1c6ec9055266e7493` |
| [run-41582614116-attempt-1-jobs.api-response.json](sandbox:/workspace/scratch/29e7135d8bfe/reviews/fc-terminal-recovery/run-41582614116-attempt-1-jobs.api-response.json) | 46,466 | `720488e489fb1df6f474853ba35faef74be599934cbf1799777010ddd06bbd65` |
| [run-41582614116-artifacts.api-response.json](sandbox:/workspace/scratch/29e7135d8bfe/reviews/fc-terminal-recovery/run-41582614116-artifacts.api-response.json) | 7,670 | `8c3535abbbf5101639197a7a6079c7f9f7efca4beaea46329b7e572f72460ada` |
| [run-41582614116-attempt-1.api-response.json](sandbox:/workspace/scratch/29e7135d8bfe/reviews/fc-terminal-recovery/run-41582614116-attempt-1.api-response.json) | 28,803 | `f0ca5ef7a58941cc87102f2fdaa825ae1f0134cec2d1e26fb53eb6d741ff0734` |

The log file is the exact UTF-8 encoding of the fetched log content, retaining Actions timestamps. Its hash identifies these preserved text bytes, not a compressed job-log archive. The API-response files preserve complete connector response objects serialized as indented JSON plus a newline; their hashes identify that representation.

**Terminal-status caveat:** the separately fetched run-attempt summary returned stale `queued` / `conclusion: null` fields. I use it only for original run/subject identity. The exact-attempt jobs collection explicitly reports the Linux job completed/failure and supplies the terminal step results. The contradictory summary was retained rather than silently normalized.

### Original failure and receipt spans

The following line numbers refer to the preserved original job log. [failure-and-receipt-spans.txt](sandbox:/workspace/scratch/29e7135d8bfe/reviews/fc-terminal-recovery/failure-and-receipt-spans.txt) contains these exact timestamped excerpts with their source line numbers.

| Original log span | Observation |
| --- | --- |
| 24592–24609 | Named background-wakes failure, panic at test source 2930:5, failed old rollback literal, and target summary **47 passed / 1 failed / 0 ignored / 0 filtered out**. |
| 64396–64443 | Original portable `integration` command includes the complete `background_wakes_production_contract` target, `--no-fail-fast`, and `--test-threads=1`; Cargo exit 101, capture exit 0, no execution-error placeholder. |
| 64849–64852 | Portable `integration.log` receipt has `oracle_accepted: false` and its raw-member SHA256. |
| 70478–70486 | Original App `E0609` at `src/tests/media_settlement_unknown_tests.rs:805:68`; source expression reads `accepted.context.duplicate`. |
| 70832–70861 | Actual App Cargo command, exit 101, capture exit 0, empty observed passing names, all three required link-preview names missing, rejected oracle, and `app.log` raw-member SHA256. |

The original job contains one `error[E0609]` occurrence and one failed Rust target summary corresponding to these two defects. This narrow diagnostic is not a claim that all other campaign obligations completed.

### Artifact member hashes and the verification boundary

| Original evidence member | SHA256 recorded by the original job's receipt | What was independently checked |
| --- | --- | --- |
| `channel-pending-recovery/integration.log` | `b4c853603bc9676993ebca8c57e143bed45d1cc3683371cd7aba7e583156a8b7` | The original full job log directly contains the failure, Cargo exit, rejected receipt and this recorded hash. |
| `link-preview-native-gateway/app.log` | `c265d5f46491b731b3335823853bcce900fd23b655f0b07bf03ac0b0701eedbc` | The original full job log directly contains E0609, Cargo exit, absent positive names and this recorded hash. |

The portable root is established by the original Hfc capture script's `.evidence/sqlite-native-path/channel-pending-recovery` constant. The App evidence root is printed in the original job command at log line 68254. The receipt-relative names are `integration.log` and `app.log`.

**These two member digests are original-receipt attestations, not independently recomputed member hashes.** The artifacts API independently confirms the terminal ZIP digest and identity. The artifact download operation returned a file reference, but direct local transfer failed because the configured proxy endpoint was unavailable. No archive bytes were installed or rehashed. At root's instruction, I stopped transfer work; no route around the proxy and no upload was attempted. This limitation does not erase the primary original-log error evidence, but it prevents claiming independent ZIP/member-byte verification.

### Actual downstream execution limits

The exact-attempt job record marks step 12, the portable recovery/Memory/pairing/palette/Session/MCP/channel step, failed. Step 21, the native link-preview HTTP/Coordinator/App step, also failed. Typed Automation admission, the actual Automation/Settings/channel App owner step, and historical prior-root recovery were skipped (steps 18–20). The Node/palette steps 13–15 were skipped as well.

The failing App command was:

```text
cargo test --locked --target-dir source/host/target --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host tests::link_preview_gateway_tests::
```

Although its filter targets link-preview tests, the App test binary must compile the MSU test module too. The field error prevents this command's tests from executing. The receipt records zero passed names, not a partially passing App target.

The portable integration receipt's required-positive-missing list is empty, but Cargo still returned 101 and the receipt correctly rejected it. Removing the failing target, ignoring Cargo's status, or counting only those listed positives would weaken the oracle. Preserve the complete existing background-wakes target and its named cases.

## 2. Exact source identity

The following relevant files are byte-identical at Hfc, local HEAD `9059d0c4493505851339ab228ec0d58a6023c904`, and the inspected worktree. Their Git blobs were checked with read-only Git operations and `git hash-object`.

| Source | Blob at Hfc, 9059 and inspected worktree | Relevant source scope |
| --- | --- | --- |
| `source/host/app/src/tests/media_settlement_unknown_tests.rs` | `f525710de5833559de608e1cb8aaa89a8d58e2b8` | `generated_lost_ack_child` 793–966; incorrect predicate 805; parent test 969–1035. |
| `source/host/tests/background_wakes_production_contract.rs` | `2da45cf1dbca8a80ef341f4624ea877b1629d9f9` | Captured-generation/observer AST control starts at 2779; failing shipping-path test starts at 2851; obsolete cleanup assertions 2930–2935. |
| `source/host/src/extensions/transcript/production_runtime.rs` | `9f15ddbdebe0ec7fe612975e28e9a7d5d0c3beb6` | `RoutedSendAcceptance` 65–69; public acceptance API 713–729; duplicate outcomes 833–851; new dispatch outcome 1032–1035. |
| `source/host/src/extensions/transcript/send_pipeline.rs` | `12f58e2923cd909b9564b99e0a6393955a0945e0` | `PersistedSendContext` 49–65. |

The original and 9059 `source/host/app/src/main.rs` blob is `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df`. The observed dirty worktree blob was `eafdf6a0cbae3651bb70d55c5cae719e31a89fdb`. Root's concurrent MCP work changes an import and later Coordinator/MCP code; the relevant media owner spans are unchanged: the index/try-lock at 795–834, rollback helper at 856–896, abort at 1087–1092, and send settlement at 1102–1221. I preserved all concurrent edits.

The incoming R6 decision file is `projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/r6-fix-decision-before-edit.json`, blob `202f0db2137f937b8b46d450c90b8e40e12bb78a`. Its subject/run/job/artifact identity and the two classifications agree with the independently fetched original evidence and current owner source. Root supplied the evidence-only carrier identity/tree and imported it; I did not treat its narrative as proof of an additional execution.

## 3. R6-001: correct the receiver and retain the causal App test

The production API returns:

```rust
pub struct RoutedSendAcceptance {
    pub duplicate: bool,
    pub context: PersistedSendContext,
}
```

`PersistedSendContext` contains `accepted_durably`, together with the persisted echo, user-message, roster and acceptance-effect fields. It has no `duplicate` field. The acceptance owner returns `duplicate: true` for the coalesced and duplicate-noop branches and `duplicate: false` for a newly dispatched acceptance.

The complete necessary expression repair is:

```diff
-    assert!(accepted.context.accepted_durably && !accepted.context.duplicate);
+    assert!(accepted.context.accepted_durably && !accepted.duplicate);
```

This preserves both intended conditions and uses the actual owner's existing result. Adding a new field to `PersistedSendContext`, deleting the nonduplicate condition, changing acceptance production behavior, substituting a fixture-owned result, or bypassing the real `accept_routed_send` call is unnecessary.

### Assertions that must remain

The generated Runner case's source expresses substantial causal obligations. The receiver fix must retain all of them:

1. Real routed acceptance, original user-message binding, actual Session admission, registry dispatch, checkpoint composition and real materialized attachment remain the test setup.
2. The exact one-use `LoseCommitAcknowledgement` lease starts unclaimed/unapplied and must be claimed and applied by this invocation. The actual owner is closed after the lost acknowledgement.
3. The actual generated Runner returns an error, emits no text deltas, records zero sent messages, zero retries and zero successful profile hooks.
4. The actual worker terminal retains its request identity and generation; effect custody remains `DownstreamOutcomeUnknown` with the asserted revision/checkpoint/pending-effect values. A durable row is not converted into a fabricated success.
5. The real initial/completed SendMessage event pair is retained, with the actual completed error and no success arm. The canonical DB/blob/terminal/Journal/mirror/provider image remains unchanged.
6. The actual committed media/settlement rows, unknown per-ID entry, media bytes, ACK binding and retention all survive. No successful App transcript publication, channel follow-up, failure wake or transport send is allowed.
7. Re-entry with the same ID remains unknown; ordinary abort retains custody. Original-turn terminal persistence refuses, and dropping every live sink/composition/observer reference does not delete retained bytes.
8. Two fresh owners reopen the real state without replay, canonical-history change, successful checkpoint observations, lost ACK, or media deletion.
9. The isolated child process's token/PID/parent-PID proof remains checked, and the controlled Responses peer must observe exactly one request with the expected model.

All eight existing App cases must remain named and selected:

- `media_settlement_app_known_receipt_refusal_rolls_back_only_owned_media`
- `media_settlement_app_lost_commit_ack_retains_rows_bytes_and_same_id_fence`
- `media_settlement_app_denied_rollback_retains_until_first_open_recovery`
- `media_settlement_app_legacy_owned_refusal_retains_without_inventing_receipt`
- `media_settlement_app_collecting_cache_multi_image_and_terminal_identity`
- `media_settlement_app_busy_entry_refuses_promptly_and_bridge_preserves_winner`
- `media_settlement_app_poisoned_entry_and_sealed_unwind_retain_for_recovery`
- `media_settlement_app_generated_runner_lost_ack_retains_unknown_custody`

These are retained source obligations. I did not execute or simulate any of them.

## 4. R6-002: follow the actual cleanup owner

The original first failure is this whole-file string check:

```rust
SHIPPING_HOST.contains("for cleanup_error in custody.rollback()")
```

The immediately following assertion also expects the old message spelling:

```text
outgoing_media_cleanup_failed agent={} tool={} error={cleanup_error}
```

The source has moved the behavior into `ProductionSendMessageSink::rollback_pending_media`. It first chooses the cleanup operation by the entry's phase, stores the returned errors, and then iterates `for error in errors`. Its actual diagnostic identifies `tool_call` and prints `error={error}`. Therefore changing only the first string would expose the adjacent stale expectation; both need repair.

### Required bounded replacement

Keep this as a source-wiring supplement with explicit ownership and ordering. Its replacement must establish all of the following in the actual production owner:

| Boundary | Responsibility to retain |
| --- | --- |
| Actual sink and trait implementation | Select `ProductionSendMessageSink` and its actual `SendMessageSink::abort_attachment_custody` implementation. A similarly named foreign method, test-only method, detached literal, trait declaration, or comment is insufficient. |
| Abort entry lookup | `pending_media_entry(tool_call_id, false)` looks up an existing entry and does not create new custody. Missing/refused lookup returns without borrowing another entry. |
| Abort entry lock | `Self::try_pending_media(&entry)` controls the existing per-ID owner. Busy/poison outcomes return; the index is not used as a lock held across settlement. |
| Ordinary abort delegation | The actual acquired mutable entry reaches `self.rollback_pending_media(&mut pending, tool_call_id, false)`. Ordinary abort cannot claim a confirmed precommit refusal. |
| Phase guard | `Collecting` uses `pending.custody.rollback()`; `OwnerSettlementPending` permits `rollback_after_confirmed_refusal()` only when `confirmed_precommit_refusal` is true; every other phase returns. |
| Terminal cleanup bookkeeping | On an admitted rollback path, the helper records `Aborted`, clears resolved sources and owned-media status, and derives cleanup success from the returned error collection. |
| Diagnostic ownership | Every returned cleanup error is iterated and reported with the actual sink's agent ID and the same tool-call ID. Merely finding the diagnostic text elsewhere in the file is insufficient. |
| Admission-release ordering | Cleanup errors return before any release of Session media admission. Release is attempted only after successful cleanup and with the entry's recorded revision/work ID. |
| Release result | A successful Session release clears the work ID; a failed release retains it and reports the failure. The source check must not accept unconditional release/clearing. |

These properties directly preserve the existing MSU phase/custody contract: an ordinary abort must not roll back an active, unknown or already committed owner. Confirmed refusal has a distinct path. Actual send settlement already seals the owner before the potentially committing append; typed `TranscriptCommitUnknown` becomes `ToolOutcomeUnknown`, and that error branch avoids rollback. Only the non-unknown failure path passes `true` to the rollback helper. Successful append commits custody before later publication callbacks. Those source facts explain why resurrecting the old unconditional loop would be the wrong repair.

Existing adjacent `syn` helpers and mutation checks already scope source-wiring claims to actual production functions and reject detached/wrongly wired bodies. R6 should follow that bounded-owner approach for these two assertions. It should not replace them with a file-wide search for the new loop or log spelling, nor report a local parser run as runtime evidence.

### Assertions that must remain outside the replacement

The entire existing `shipping_send_message_reaches_host_owned_delivery_failure_broadcast_and_context_paths` case must remain. Preserve its checks for the actual failure worker, observed processing owner, generation provenance, checked marker completion, exact handoff and terminal drain; one shared drain per worker; no restored separate empty-take/generic successor-clearing cleanup; transcript persistence before delivery dispatch; delivery-operation identity; durable failure markers; exact Session generation; preflight pause and retention paths; roster/marker completion before queue settlement; no requeue; existing group policy; error trays; envelope images and unanswered/skipped/dismissed question context; broadcast, nudge, channel-connect/disconnect and automation-change paths.

Preserve all 48 existing named background-wakes tests and the pre-existing AST negative controls. The original 47/1 result demonstrates the failure boundary; it does not authorize dropping the one failing case or reducing the target to the required-positive subset.

## 5. Integration limits and handoff

This assessment supports the two narrow R6 test repairs, with the responsibility-preserving conditions above. Another session owns those edits. I made **no product, test, schema, workflow, ledger or lockfile changes** for this task; staged and committed nothing; triggered no workflow; uploaded no artifact or private archive.

Investigation consisted of original-service reads, source/evidence reads, Git identity comparisons, preservation of retrieved evidence, and hashing of those preserved bytes. No build, test, formatter, source parser, lint, generator, or source-only simulated execution was performed.

The original Hfc partial evidence remains bound to Hfc and its actual inputs. This diagnosis adds no MSU8, LFA6, SGF3, cron5, link-preview App, new MCP candidate, full campaign, packaging, or live-provider acceptance. Root should compare the incoming R6 source repair against the exact preimages and retained obligations here, preserve concurrent MCP changes, and freeze/review the combined candidate before any new campaign decision.

The durable original-evidence index is [evidence-index.json](sandbox:/workspace/scratch/29e7135d8bfe/reviews/fc-terminal-recovery/evidence-index.json). The original failures and receipt binding can be reviewed directly in [failure-and-receipt-spans.txt](sandbox:/workspace/scratch/29e7135d8bfe/reviews/fc-terminal-recovery/failure-and-receipt-spans.txt), with the full log and API responses retained beside it.
