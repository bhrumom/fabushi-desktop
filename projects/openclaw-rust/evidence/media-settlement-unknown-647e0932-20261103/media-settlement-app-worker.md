# MSU-v1 App implementation handoff

## Outcome and execution boundary

Implemented eight source-level App test cases in the two allowed paths. The tests use the existing `ChannelHostLaneFixture`, real private `ProductionSendMessageSink`, real Session/TranscriptRuntime/AgentDb owners, real Counting sink and Tool bridge, and the existing SGF Responses/child fixture with `create_production_runner` for the generated case. No new production owner or production fault interface was introduced by this worker.

**Implementation only; unexecuted and unverified.** No local or htch tests, executable test listing, builds, Cargo/rustc, parser, linter, schema/capture generator, fixture server, SQL test execution, or validation was run. Operations were source/text inspection, immutable Git reads, text diffs, hashes, and the authorized edits. The Rust test code contains the runtime operations to be performed later by Actions; none were exercised here. No commit or push was performed. No review approval, ledger promotion, or current-H runtime success is claimed.

Configured model retained: GPT-6 Astra Pro. Parent confirmed the launch used `fork_turns="none"`, model omitted/inherited, and actual `reasoning_effort="xhigh"`. This is launch-applied xhigh; no independent worker effort introspection is available. The same worker continued into this implementation assignment.

## Subject, allowed writes and exact deliverables

- Fixed U supplied by parent: `1440bb1771372334bea23d7642bb52deb7db1845`.
- Implementation source base: `fdb0f6acb06165b6f2796961186e06a2de98c229`; its local immutable Git tree was read as `abeb6ffdf09b676279462fea3b9e1f6d72abc1b6`.
- Parent identified the unchanged core continuation as `a2a0c9ef326c1ddeb09b4f9180267fdaeb21db74`. That object was not available in the local immutable object store, so this report does not independently certify its tree. Parent's later remote comparison remains root-owned; the permitted edit base and paths did not change.
- Existing fixture base blob independently read: `ff4ebba71eae35f4be98a695654a0d33b84f1818`.
- Only source writes: `media-settlement-work/source/host/app/src/tests/channel_host_lane_tests.rs` and new sibling `media_settlement_unknown_tests.rs`.
- Authorized handoff artifacts: `media-settlement-app-worker.patch` and this report.

The exact unified patch uses repository-relative paths, compares the existing fixture to its immutable fdb0 base, and includes the complete new file. It was produced by text diff only; it has not been applied or checked by a validation command.

| Deliverable | Identity |
| --- | --- |
| `media-settlement-app-worker.patch` | SHA256 `fd1ce324a23f9c57d4acbb5f038c0fee32b2db320e3b15cda25244dd22a8ce4d` |
| `channel_host_lane_tests.rs` | Git blob `f6139af6bde9829f7e9147e6a2028ec77680bf42`; SHA256 `616866391e288b5bc61436f2452cf9717c65fd380ea6d0b60d6d49ad43c18c50` |
| `media_settlement_unknown_tests.rs` | Git blob `f525710de5833559de608e1cb8aaa89a8d58e2b8`; SHA256 `f1ec56ebb6b7b31bcb9809645a60221e488269bbb841780d65da01d35da5fc68` |

The existing fixture change adds one nested path module, exposes only existing SGF test helpers to that sibling, and factors `Responses::new(root, case)` through `Responses::with_events(root, closure)`. The original constructor still calls the same `response_events(root, case, ordinal)`. The SGF cases, full test names, assertions, success/failure expectations, two-send counts, and two-request counts are unchanged. The shared child cleanup, request reader, and wire-field inspection are reused. This worker did not edit LFA or any production/shared configuration source.

## Full test names

These names are read from the authored source, not an executable test listing.

```text
tests::channel_host_lane_tests::media_settlement_unknown::media_settlement_app_known_receipt_refusal_rolls_back_only_owned_media
tests::channel_host_lane_tests::media_settlement_unknown::media_settlement_app_lost_commit_ack_retains_rows_bytes_and_same_id_fence
tests::channel_host_lane_tests::media_settlement_unknown::media_settlement_app_denied_rollback_retains_until_first_open_recovery
tests::channel_host_lane_tests::media_settlement_unknown::media_settlement_app_legacy_owned_refusal_retains_without_inventing_receipt
tests::channel_host_lane_tests::media_settlement_unknown::media_settlement_app_collecting_cache_multi_image_and_terminal_identity
tests::channel_host_lane_tests::media_settlement_unknown::media_settlement_app_busy_entry_refuses_promptly_and_bridge_preserves_winner
tests::channel_host_lane_tests::media_settlement_unknown::media_settlement_app_poisoned_entry_and_sealed_unwind_retain_for_recovery
tests::channel_host_lane_tests::media_settlement_unknown::media_settlement_app_generated_runner_lost_ack_retains_unknown_custody
```

## Authored causal observations

The following are runtime assertions implemented in source. Their results are pending Actions; they are not observations from a run performed by this worker.

### Real SQL uncertainty and the ordinary rollback comparison

`direct_unknown` opens the actual cached AgentDb owner through the existing Session API. It arms a borrowed lease for the exact admitted session key, lifecycle revision, and `send-media:{tool_call_id}`. The tests assert both observations are initially false, then require `was_claimed()` and `was_applied()` after the real App call. They require the exact owner to be closed before inspecting its result.

The lost-ACK case expects exactly one new visible send row and one hidden settlement receipt from an independent file-backed read. It compares all preceding `(seq, id, entry bytes)` tuples, the send's message/provenance, every receipt field, exact media paths and linked send ID, and the complete released admission record. The unknown App entry must retain its phase, sealed custody, owned-media flag, session/revision/work identity, and resolved-source cache even though the durable fence is already clear.

The denied-rollback case uses the same real owner transaction and a file-backed receipt INSERT trigger. The trigger can raise only when the first send ID carried by that incoming receipt already exists as a `send-message` row with matching session provenance in the same transaction. Its SELECT is empty without the inserted prefix. The exact `RefuseRollback` lease must then report application at SQLite's ROLLBACK edge. After retirement, the independent reader must see no newly committed send or receipt, the complete original pending admission record, and retained owned bytes.

The known-refusal test installs the identical receipt trigger without a command fault. It requires ordinary `ProviderSessionError::Tool` with the actual prefix witness diagnostic, an open owner, no new rows, released media admission, only the captured managed original removed, and an `Aborted` terminal entry. Repeating resolve/send and abort cannot recreate that identity. This is the positive control distinguishing an acknowledged rollback from denied rollback; a mock unknown is never substituted for either SQL path.

Both actual unknown cases require zero Counting-sink success, no transport call/effect, no queued `deliverToChannel` observation, no successful App transcript event, and unchanged real ACK state. They retry resolution using a changed invalid URL, retry send, invoke abort, compare map Arc identity and media-file bytes, and attempt a cached Session read without treating that as recovery. They drop all observer/Counting/entry Arcs before asserting the final concrete sink Arc is gone. Disposal alone must preserve the durable image and media.

Two newly constructed Session/App owner sets then inspect the result. Committed lost-ACK rows and bytes remain unchanged and the hidden receipt stays hidden. Uncommitted captured media is cleaned only by the existing first-open recovery; source and unrelated sibling bytes remain unchanged, with rows unchanged and admission released. After those two observations, a separate new media ID successfully settles through another fresh owner. The rollback fixture removes only its test-owned receipt trigger before that new positive. This proves ordinary post-recovery availability without replaying the unknown original ID.

### Actual ACK obligation

`take_sink_with_ack` calls the real `AckObligations::record_send_and_mint_token`, requires a newly created obligation, and installs its token on the actual private sink. It records the actual durable ACK file bytes and full checked obligation. Unknown/refusal assertions compare those bytes and checked contents, and verify the token is bound to this agent while the original owner is live. The successful multi-image test requires that same real obligation to be cleared. An initially empty ACK store is not used as negative evidence.

### Same-ID ownership, cache and unwind controls

The collecting test resolves a real image, deliberately changes only the test source, and repeats the exact URL. It requires the identical normalized result, unchanged owned bytes and durable admission, then restores the source. A second distinct image is admitted. The actual Tool bridge receives three image arguments—first, exact repeat, second—and must commit one message with two unique owned paths, correct image dimensions, one success count, and a released fence. A retained old Arc must observe `Committed`; later resolve/send/abort cannot rematerialize, reappend, or remove either committed file. Two fresh owners retain the exact positive result.

The busy-entry test holds the actual entry mutex while a worker attempts same-ID resolution/send/abort and uses another entry. A bounded observation must arrive before the held guard is released. An observation-only wrapper pauses after the production resolver has returned busy unknown, so the test can release the winner before the actual Tool bridge handles that error. Its abort callback count must remain zero, the winner's original entry must still be Collecting, and SQL/admission/files must be unchanged. The already-prepared winner then successfully settles through the real owner. This scheduling wrapper does not manufacture a database outcome.

The poison/unwind test covers both a collecting guard poisoned by a controlled unwind and an entry explicitly sealed by the actual production transition before that unwind. Normal resolve/send refuse with typed unknown; abort retains; poison handling may only seal. Last sink and entry Arcs must disappear without deleting media or releasing the fence. Two fresh owners perform the existing safe cleanup. This is labeled as a local ownership/unwind control, not SQL fault evidence.

### No-fence legacy control is separate evidence

The eighth test selects the existing unbound compatibility sink by setting its optional session key to None. Actual resolution still materializes through production, but there is no media work identity or path fence. A real SQLite send-INSERT trigger supplies a refusal. An untracked text call through that same String API must expose the ordinary SQL diagnostic, preserving its existing error behavior; the owned-media call must conservatively return App unknown after entering the legacy append path.

The owner remains open; the raw rows and full admission record are unchanged; no settlement receipt is invented. The local entry remains sealed with owned bytes and no fabricated session/revision/work identity. Retry, abort, last Arc Drop, and two fresh owners retain the bytes and independent ACK. This exercises the specified conservative fallback. It is **not** a claim that the legacy call reached uncertain COMMIT, that a rollback was denied, or that the still-open no-fence durability responsibility is accepted.

### Actual generated Runner consumption

The child case uses the existing SGF controlled Responses server and owned child-process harness. It accepts a real original routed send, binds its original-turn model, composes the actual production checkpoint sink, and creates the actual production Runner. A real local attachment is prepared through the private App sink first so the exact admitted owner/fence can be armed; the generated Tool producer resolves that identical source through the collecting cache and performs the actual send/settlement. The parent supplies precisely one actual `SendMessage` function call with channel routing requested.

After real COMMIT/lost ACK, the test requires both fault observations, retired owner, zero success count, zero executed retry callback, zero successful lifecycle hooks, a failed generated terminal, and exact sticky custody: started revision 1, checkpoint revision 0, pending effects 0, `DownstreamOutcomeUnknown`. It reads the actual initial/completed live tool events using the existing SGF wire helper and requires an error result with no success payload.

Independent durable snapshots compare the canonical metadata root, every blob row, terminal receipt slots, Journal bytes, mirror files, and provider checkpoint files to their pre-invocation baseline. An unwritten blob store is represented as an empty logical blob image so harmless empty-store initialization is not mistaken for history. Original-turn completion observations must be invalidated; persistence of a completed terminal must refuse. The final outer error is deliberately not required to remain `ToolOutcomeUnknown`, because the existing provider/shell boundary can map it to `Tool` while sticky custody remains authoritative.

The test retries the same sink ID, aborts, disposes all generated/sink owners, and uses a bounded weak-count wait before proving the last concrete App sink Arc is gone. Two credential-free fresh owner sets must preserve committed SQL/media, unchanged canonical history, outstanding ACK, no completed-original-turn reconciliation, and no replay. The parent requires one total provider request and the completed child receipt's zero counters; it never offers a successful second Responses continuation.

## Source inputs actually read

The MSU specification and concurrency/consumer notes were read completely. Current draft input hashes below identify the bytes used for the implementation handoff; they are not an assertion that those bytes are a published H. Root owns final integration and publication identity.

| Current draft source | Git blob | SHA256 |
| --- | --- | --- |
| `source/host/app/src/main.rs` | `7ec9bd8a22fd595f63cb3f4b632673e298294dfa` | `eadde11031c9e3a92236986e42d84877be2e1b1068f7e055ad1aa2b8fa86693f` |
| `source/host/src/extensions/session/agent_db.rs` | `6c27e9bb9f665b1160457b9acc29dd5d64fba289` | `9a8ad4aa952f11a3d7595eebf636f7d20c450381120416c00667b93ef0149c7b` |
| `source/host/src/extensions/session/production.rs` | `c5912cab4f95d3a75fc13cb264a21b1deef9a733` | `b5cac3ef9d0d761ce0a0f63dabf6a930524822f0758b47714a8ee6da3900f995` |
| `source/host/src/extensions/transcript/production_runtime.rs` | `9f15ddbdebe0ec7fe612975e28e9a7d5d0c3beb6` | `4e18cb4e29e5947ee200ca69c9ff4816ca753e9319573a1695e0736d3a55c3c1` |
| `source/host/src/attachment_paths.rs` | `6e1a2b8eb5752e005bbce30ed0b6eaaacfda0cda` | `ce4ac8bc522d1b20ff97b486942dfbe2aacd1ff0f65ee70a0b657f6960fff381` |
| `source/host/src/runner/tools/send_message_tool.rs` | `86a9e0e394b0145173bc033ceb757b7fb10835e5` | `2da2145a57c5ad7d245bd36c6463a731cc511059d87d9366f68fe385d5758e87` |

Additional current inputs: full `projects/openclaw-rust/docs/specs/media-settlement-unknown.md`, SHA256 `87f33dff9bbc614e90bf5c0cacae24f31378de1dd6eddaebb484aa61d96cb501`; full `/workspace/scratch/7d27ea239237/media-settlement-concurrency-consumer-notes.md`, SHA256 `2c66c584f794efe7fdd5f74620854f6eb9119094b91e4b1a0d7b6f6db9a37034`; selected LFA fixture ranges, SHA256 `9b279aba96ba879332b45aeddcc9c8d156526917725a2c40b6e684b123915a76`.

Current-source reading focused on: App attachment materialization, pending entry helpers, full SendMessage sink implementation and relevant imports; fixture transport/owner creation/disposal and complete SGF module; AgentDb fault types/lease, full media append outcome handling, actual command adapters/authorizer removal, receipt schema, and adjacent DB fault controls; Session first-open media recovery, capture/release and typed append; Runtime original-turn setup/settlement and all three generated-send append paths; local custody dispositions; Tool resolution/Counting/build/send paths; selected LFA materialization, ACK/event and media-idle observations. These are targeted reads, not a claim to have read the entire App or Session files.

Supporting unchanged files were read with `git show` at full fdb0, never from mutable working bytes:

| Immutable source beneath `source/host/src/` | Blob | Relevant read |
| --- | --- | --- |
| `extensions/transcript/ack_obligations.rs` | `f105486efdbf0c1bef78903d9846e7da6d0e3074` | complete file; real reservation/token/fulfillment and disposal APIs |
| `extensions/transcript/sand_ack_obligation_store.rs` | `d0441ef76d1c03579c3da9330ae653514b74f62a` | 1–210; checked record and durable-file reader |
| `extensions/session/gateway.rs` | `cb535f0475a967a9bc5360869931bc0c79197745` | accepted-send persistence function entry and surrounding staging ranges |
| `extensions/session/production_agent_store.rs` | `e8afdca4bb983d6c3f7f0b062720db142739e576` | checkpoint/blob type and path declarations; targeted checkpoint symbols |
| `extensions/session/agent_db_schema.rs` | `279583c0e0ccb37a7188bfa29b5467f8a1e24b3e` | 1–120; exact table and raw-reader schema |
| `agent_isolation/conversation_blob_db.rs` | `cce3bfaee92a0f9be538ce663a19933f594de477` | 1–78; logical blob table schema |
| `runner/generated_agent_turn_stream.rs` | `4617051c700665287008c8cec225ae333af02387` | 280–420; staging timing and unknown/error history refusal |
| `runner/stream_attempt.rs` | `6eb8a3eae5556eb6d6a6c03d00059d7c1180190f` | 1–185 and 300–500; custody representation, sticky unknown and checkpoint gate |
| `runner/tools/mcp_meta_tools.rs` | `923288198b898083cbfe5da9a1f76e9f309ee516` | 170–218; actual worker records downstream unknown before receipt completion |
| `runner/production_turn_run_shell_adapter.rs` | `d0fe2714923338dd717284f08f05eef438db69d9` | 465–492 and 545–577; outer error mapping and retry refusal |
| `runner/production_agent_checkpoint.rs` | `6443bd7b8636babe665237ee1e6153652df3b924` | 1–120; real completion observation/checkpoint API |
| `runner/routed_provider_runtime.rs` | `a351081d6e8e9e9c1fb51439f4d67b48e4ace886` | checkpoint store declaration and 313–343; actual checkpoint directory |
| `runner/mod.rs` | `3b01a2a8f5c85a7c0cfc88b0e5f368f1debc421b` | relevant custody reexports |

## Integration requirements and remaining limits

Root should integrate the two source files and register all eight exact test names, including the eighth legacy control. The App tests consume the already agreed default-off Host `test-support` feature through the App dev-dependency; this worker did not edit Cargo or any capture/workflow. Existing SGF/LFA positives and the ordinary feature-off shipping check remain required.

Fresh independent source review and Actions on the newly frozen published H remain necessary. In particular, Rust compilation, module resolution, actual fault application, generated provider/guard scheduling, Windows behavior, and all asserted durable/file results remain unexecuted here. A failed new test must be investigated against its concrete causal boundary; this report supplies no permission to weaken the real applied-fault, ACK, row/receipt/fence, custody or history assertions.

The scope does not establish arbitrary crash durability, a durable same-ID replay registry, authentication of unrelated nonoverlapping calls sharing a string, full provider/channel acceptance, or acceptance of no-fence media durability. The retained terminal map entries and the legacy fallback have exactly the bounded responsibilities described in MSU-v1.

**Handoff state: implementation complete; source-only; runtime verification pending root integration, independent review and new-H Actions.**
