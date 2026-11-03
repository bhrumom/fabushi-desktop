# MSU Tool-owner worker handoff

## Result and authority

Implemented the narrow Tool-owner correction in the two assigned work files. The production change preserves `ProviderSessionError::ToolOutcomeUnknown` from `build_sand_send_message` without calling `abort_attachment_custody`; all other construction errors retain the existing abort-and-return behavior. The original error value is returned without conversion. Encoding, awaiting-user, final-send, and delivery-counter behavior are unchanged.

Authority is the root's implementation assignment and the reviewed `MSU-v1` contract at `/workspace/scratch/7d27ea239237/media-settlement-work/projects/openclaw-rust/docs/specs/media-settlement-unknown.md`. Assigned production base: `fdb0f6acb06165b6f2796961186e06a2de98c229`, tree `abeb6ffdf09b676279462fea3b9e1f6d72abc1b6`; fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`. The immutable `media-admission-r7` snapshot is the assigned published-LFA comparison source.

## Modified files

Both files were absent in the assigned work tree and were copied from the corresponding R7 files before the edits:

- `/workspace/scratch/7d27ea239237/media-settlement-work/source/host/src/runner/tools/send_message_tool.rs`
- `/workspace/scratch/7d27ea239237/media-settlement-work/source/host/tests/send_message_tool_contract.rs`

No other production, test, support, Cargo, schema, specification, composition, or ledger file was edited by this worker. No commit or push was performed.

The unified patch against R7 is `/workspace/scratch/7d27ea239237/media-settlement-tool-worker.patch`. Its paths are relative to the repository root. The production hunk adds the typed-error condition and its rationale; the contract-test hunk adds one existing-type import and an additive fixture with two tests. Existing test bodies, including both LFA file-URL facade tests and every assertion within them, are preserved verbatim.

## Causal tests added

| Test | Authored oracle |
| --- | --- |
| `send_message_bridge_preserves_prepared_custody_on_unknown_resolution` | Resolver sees the correct call ID and source, reads the already-prepared file bytes, then returns the exact typed unknown error. The bridge must return that variant and payload. The call trace must contain only `resolve`; the file bytes remain; no send, abort, delivery count, or success interaction occurs. |
| `send_message_bridge_aborts_prepared_custody_on_known_resolution_error` | The same prepared-file witness is established before an ordinary Tool error. The exact call trace must be `resolve`, `abort`; the real `OutgoingMediaCustody::rollback` removes the prepared file. The ordinary variant and payload survive; no send, delivery count, or success interaction occurs. |

The fixture uses the existing `persist_agent_media_bytes` and `OutgoingMediaCustody` APIs to create and track an actual file before bridge entry. The resolver's file read occurs before it consumes and returns its single supplied error. The abort method invokes real custody rollback, so the ordinary-error control checks an effective cleanup operation rather than an inert abort counter. Both tests call the actual `SendMessageToolBridge` through the existing `CountingSendMessageSink` and interaction recorder.

This is a Tool error-classification test. Its resolver deliberately supplies the error after proving the prepared witness. It does not execute a SQLite transaction, exercise a Session owner, simulate concurrent scheduling, establish sticky Runner custody, or prove unknown-state Drop/reopen behavior. Fixture teardown happens only after the asserted observations. Actual COMMIT-lost-ACK, ROLLBACK-refusal, App locking, unknown disposition, generated consumer, and Session recovery tests remain with their assigned root/owner work.

The regression is causal by source: reinstating the old unconditional build-error abort would add `abort` to the unknown case's trace and delete its witness. Removing cleanup for all errors would fail the ordinary control's trace and file-removal assertions. These statements describe the authored assertions and reviewed source, not an executed mutation test.

## Source-read scope

The following current-task reads informed the change:

- Full shared MSU-v1 contract at `/workspace/scratch/7d27ea239237/media-settlement-work/projects/openclaw-rust/docs/specs/media-settlement-unknown.md`.
- Actual bridge, resolver, interaction, and counter implementations in `/workspace/scratch/7d27ea239237/media-admission-r7/source/host/src/runner/tools/send_message_tool.rs`.
- Existing adjacent fixtures, assertions, media persistence conventions, and LFA tests in `/workspace/scratch/7d27ea239237/media-admission-r7/source/host/tests/send_message_tool_contract.rs`.
- Immutable exact-base Git reads of `source/host/src/attachment_paths.rs`, specifically the existing `OutgoingMediaCustody` tracking/rollback/Drop interface, and `source/host/src/extensions/inference/provider_session.rs`, specifically the `ProviderSessionError` definition. Those unchanged files are not materialized in the sparse R7 snapshot; no mutable repository working-tree source was used.
- A narrow R7 App-source search for `pending_media`, plus the resolution-observer and `assert_custody_idle` excerpts in `/workspace/scratch/7d27ea239237/media-admission-r7/source/host/app/src/tests/local_media_file_url_tests.rs`, solely to answer the root's outstanding terminal-tombstone/idle-oracle dependency question. No App code was edited.
- Literal unified source diffs of the two assigned work files against R7, used to prepare the requested patch.

The previously read root AGENTS and Session/transcript ownership specifications remain applicable. No other review report or root arbitration was read for this implementation task.

## Outstanding App integration note already sent to root

The sole pending map can retain terminal entries with at least `Prepared`, `OwnerSettlementPending` (including unknown), `Committed`, and `Aborted` distinctions. Committed/aborted entries keep their original Arc and reject repeated resolution/send before materialization; abort is inert for terminal entries. Unknown preserves its recovery evidence. A second completed-ID index or successful-result cache is unnecessary for this safety requirement.

R7 `LocalMediaAppFixture::assert_custody_idle` currently requires durable work ID absent, durable paths empty, and the literal map empty. Retained terminal tombstones make the last container-level assertion incompatible with the intended representation. Root should preserve the two durable assertions and give the map assertion the precise observable meaning “every remaining entry is terminal committed/aborted and owns no active custody/admission.” Busy, poisoned, prepared, and pending/unknown entries must not count as idle. A path that rejects before creating any entry can additionally retain a literal empty-map assertion. This worker did not change any of those App assertions.

## Execution and blockers

**No local or htch validation, tests, builds, parser, formatting, lint, test listing, or capture execution was performed.** Work consisted of bounded source reads, the two authorized copies/edits, and generation/review of the requested textual diff. No test has been claimed to pass, and no SQL or concurrency failure has been claimed to reproduce.

No source-level blocker was found for this narrow change. Compilation and behavioral verification remain unperformed and require the root's Actions run. That run should include both new tests and the unchanged `send_message_tool_contract` target/LFA assertions, together with the separately owned MSU transaction/App/Runner tests. The complete repository and the root-owned custody changes must be composed before that verification.
