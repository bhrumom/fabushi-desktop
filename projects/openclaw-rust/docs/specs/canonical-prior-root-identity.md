# Canonical prior-root identity — PRID-v1

Contract: `OCRM-CANONICAL-PRIOR-ROOT-IDENTITY-v1`.
Status: specified before implementation; runtime and acceptance pending.
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`.
Starting base: `3438eb5bac755c399c9ec686414bbb4c0816888d`.
Historical producer base: `e19a0a265e400a4c9b1d96520c84fb612a3843e8`,
tree `5be2d7ca128dae5fefc8f7551e7088934daf19ac`.

## Problem and existing owner

CRA already captures the complete optional `ProductionAgentCheckpointSnapshot`
and binds actual root identity through guarded publication. Its generated
UserMessage field 10 still hashes nonempty prior bytes and omits present-empty
roots. For a valid legacy root L holding bytes B with no SHA256(B) alias, the
generated reference does not resolve to the root actually used. Existing ORT
records retain L and its reader reconstructs this old writer image. This is a
wire-provenance defect; ordinary Session/Transcript recovery does not dereference
field 10, so an ordinary restore failure is not established by this finding.

Extend the existing Runner checkpoint writer and completion verifier in
`production_agent_checkpoint.rs`. Session/AgentStore remain the only durable
owners; the current generated Runner, Journal, terminal receipt and shipping
App marker-recovery path remain consumers. No new runtime, store, permission
authority, schema/version, alias, replay engine or garbage-collection root.

## Required behavior

| ID | Requirement and oracle |
| --- | --- |
| PRID-01 | All three checkpoint builders accept the existing complete optional snapshot instead of prior bytes alone. No root means no field 10; every present snapshot emits its exact nonempty root ID, including empty bytes and legacy fixed IDs. Prior root wire bytes remain unchanged before appending the new turn. |
| PRID-02 | Production stage uses the same captured snapshot already compared against retained and checked-live authority. Preserve CRA compare-and-swap, failure invalidation, GTH/SGF bytes and the unchanged artifact record shape. Do not re-read and adopt a successor or create a guessed SHA alias. |
| PRID-03 | Fresh completed-turn publication requires the exact current writer image using the recorded actual prior ID. A historical hash or empty-root omission must not qualify a new proof when it differs from that image. Preserve all preceding user, turn, root, result, custody, lifecycle and Session raw-image checks. |
| PRID-04 | Reconciliation of existing durable ORT version-1 receipts accepts only the exact current writer image or the exact historical writer image: SHA256(nonempty prior bytes), omission for empty bytes. Match the complete user image, not merely its decoded last field. Retain strict normalized optional rich text and reject duplicate, wrong, unsupported or malformed fields. |
| PRID-05 | Valid legacy roots remain valid without their content-hash alias. Do not rewrite historical roots, user blobs, terminal records, mirrors, provider checkpoints, markers or credentials. Missing required recorded roots remain refusal, not empty state. |
| PRID-06 | Genuine historical positives are produced by exact BASE production code in Actions with only an identified test-support overlay. Use actual accepted input, generated Runner, counted effect, sealed observations, proof issuance and persisted terminal record; never manufacture a positive receipt or rewrite new-writer bytes into historical syntax. |
| PRID-07 | Export present-empty and legacy nonempty/no-alias histories with two generated invocations. Retain closed stores and original process/request/effect evidence. Require two fresh old-reader checks, then new-H fresh readers and the existing actual App marker acknowledgement/no-op with unavailable provider credentials and no extra effect. |
| PRID-08 | Adversarial semantic controls may coherently rehash copied blobs and recorded references to reach the user-image check; label them as mutations, require the intended diagnostic, and preserve unrelated proof/custody/fence state. An earlier hash error is not field-10 semantic coverage. |
| PRID-09 | Retain existing canonical, CRA8, GTH9, ORT, checkpoint/store/recovery, SGF and App assertions. Select new controls and historical export/import explicitly in the existing exact-H Actions workflow with command/capture exits, unique positives, source/binary/log hashes and full original artifacts. No local or htch execution. |

## Interfaces and compatibility

The first parameter of `build_text_turn_checkpoint`, its rich-text variant and
`build_generated_turn_checkpoint_with_rich_text` becomes
`Option<&ProductionAgentCheckpointSnapshot>`. Production passes the already
captured `expected_base.as_ref()`. Existing source callers are limited to the
checkpoint owner and three integration-test targets; update all together.
`TextTurnCheckpointArtifacts` and the checked sink/store interfaces do not change.

Fresh terminal publication and historical reconciliation must choose explicit
validation modes inside the same existing verifier. A shared exact-image helper
may avoid duplicated prefix/rich-text/suffix validation. Historical compatibility
does not weaken new proof admission or accept arbitrary root IDs. Duplicate or
extra fields cannot be hidden by protobuf merge semantics.

## Actions evidence and delivery

Use the existing process harness in `production_checkpoint_restart_contract.rs`
and its existing shipping App child. A test-only support module may export and
consume fixtures; its two explicit ignored utility entries fail without their
required absolute directories. Normal filtered tests must not accidentally count
those utilities as positives. Build the exact historical BASE Host target with
only that support module and its root module declaration overlaid; verify every
production/build/schema/lock input is BASE and preserve the complete overlay.
Resolve test binaries from the unique Cargo compiler-artifact record and retain
their hashes. Reuse the existing new-H Host/App build and capture owners.

The existing portable capture explicitly selects four private writer-image
controls and three current-writer/fresh-process parents; it skips only the two
ignored export/import utility entries in that group. The historical capture is
`projects/openclaw-rust/scripts/capture_prior_root_contracts.py`, executed after
the existing actual-App step succeeds. It verifies every tracked source blob
before and after, permits only the BASE test support/declaration overlay,
retains H binaries before rebuilding BASE in the shared Cargo target, and
resolves each Host artifact from one exact Cargo JSON compiler-artifact record.
The two utilities require exact single passing results and preserve complete
command stdout/stderr; their successes are evidence operations, not additional
product-test counts. Current source, old producer source, binary identities,
fixture inventory and archive digests remain separately bound.

Historical fixtures retain complete closed state, producer/reader receipts and
logs, request/model accounting, one actual effect, prior-ID/alias observations,
file inventories and digests. Hash the archive outside its own payload. No
credential value is exported. If copied state cannot relocate without rewriting
stored bytes, keep the same owned location within the job and report portability
as blocked; do not fabricate a portable artifact. New source review and real
Actions results are required before this contract is verified.

## Release, rollback and remaining scope

No persisted schema migration or old-history rewrite occurs. Withdrawing the
new writer must retain the compatible reader for already-written legacy/empty
identities; an older binary may reject those new completed receipts. Refusal
must preserve custody and markers and never replay tools. This bounded contract
does not promise permanent retention of prior roots, repair missing history,
ordinary typed replay, unfinished original-turn restart, real ecosystem/device
compatibility or final G01–G12/AC01–AC24 acceptance.

Authority: OCRM §17, the existing provider-bound model/CRA/GTH/ORT contracts and
the actual frozen UserMessage `conversation_state_blob_id` field 10 in
`source/packages/proto/generated/agent/v1/agent_pb.ts` (blob
`3cd3d020e69b54d1a6df27c0f967afe7e9fc942c`). Target checkpoint source at BASE is
blob `a966e88220c8314a67273d0ff29c960e365b77a5`. Reuse the existing fixed-U loop
source authority and literal read facts; no new complete-source or full-read
credit is created by this target repair.

## Original artifact completeness correction — PRID-CAPTURE-01

Independent inspection of H `0c7da50fcf457184f38d6a61b73f8e32810cff49`,
run `41574595343`, original final artifact `13204320664` found that the scoped
upload's default hidden-file exclusion omits four inventoried
`agents/.call-sessions.db` copies from directory payloads. Both historical
copies remain in the original `historical-fixtures.tar.gz`; the two current
`state-after` copies have no standalone ZIP member or consumer archive at that
H. Their complete original bytes are nevertheless retained in the uploaded
`persistent-image-after.json` files at `/other_files/agents~1.call-sessions.db`:
each is a 32,768-element u8 array, SHA-256
`678a56ce2376b5996556e3f692199423d285feaa36a55aa04076b25370e63974`, matching
the original consumer inventory and byte-for-byte equal to its BASE tar member.
Thus the standalone directory payload is incomplete, while the full original
content comparison remains possible using these original representations.
Preserve both facts and the original command, source/binary and runtime
positives. The archive requirement below closes the standalone packaging gap;
it does not rewrite the old artifact or replace any retained assertion.

Before declaring the capture complete, the existing capture script must also
archive the already closed, inventoried `current-consumer` subtree as
`current-consumer.tar.gz`. Bind its directory, unchanged inventory digest,
archive digest and byte size outside the archive in `consumer_payload`; retain
the existing `consumer_inventory_sha256` field for existing readers. Archive
creation or inventory drift must leave the receipt non-passing. Preserve the
historical archive and all existing source, binary, command, credential-exclusion
and no-replay checks. Do not widen the whole upload's hidden-file policy or
archive unrelated temporary roots.

The next exact-H Actions capture must retain this ordinary archive file in the
existing artifact. Independently compare its exact member set, file sizes and
hashes to the original consumer inventory, including both hidden databases.
Original H evidence remains immutable; the correction grants no retroactive
completeness, extra product-test count, real-provider or global acceptance.
