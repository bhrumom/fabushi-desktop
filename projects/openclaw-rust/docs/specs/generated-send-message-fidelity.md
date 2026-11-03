# Generated SendMessage fidelity

Contract: `OCRM-SEND-MESSAGE-FIDELITY-v1` (SGF-v1).  
Status: implementation and consumer controls authored; independent review, execution and scoped acceptance pending.  
Fixed OpenClaw U: `1440bb1771372334bea23d7642bb52deb7db1845`.  
Integration base: `073b98b4826c8125d0a42d15f8ccdbfebb5eb488`, preserving
the incoming media HHP/IPB and file-URL admission implementations and evidence,
and including
published CRA `4fdf18a4906f5ea924e125aed6082de634d410e1`, its result-oracle
correction `ff0d456f322094ca82f063942f548f8d7fe90f00` and the existing media owner
correction. Exact-H gates remain separate. Initial assembly used immutable
CRA candidate `0602eb45750bcd418fa62aec331ae8618b0b11f3`; preserve that history. The SendMessage producer,
recorder and Session consumer were inspected at `cb62ae7c22a8264c49e90443e41cb6ad3c9b9a5d`.

## Problem, goal and existing owners

The actual SendMessage producer already emits frozen initial/completed
`AgentToolCall` observations after argument normalization and sink execution.
Its public JSON arguments/result have a different shape from the generated
protobuf descriptors. The later generic checkpoint JSON conversion loses the
generated arguments/result and actual SQL message identity. Capture the actual
producer observation and retain its bytes through the existing canonical
checkpoint and Journal path, independently of UI observation wiring.

The current recorder is shared by cloned compositions and is cleared at run
entry. An old timed-out worker can retain that recorder across a later run.
The existing Session decoder also promotes argument-only or failed SendMessage
records into sent-message outline items and recovery rows. Correct those two
boundaries in their existing owners. This creates no message store, runtime,
permission authority, replay engine or scheduler.

Entry/producer: existing provider tool dispatch -> `SendMessageToolBridge` ->
the existing `SendMessageSink` (the App binds `ProductionSendMessageSink`).
The App sink owns the SQL append; channel delivery keeps its existing owner
and does not become acknowledged merely because the local append succeeded.
`TurnAgentComposition` owns invocation/call observation custody; generated
stream and `ProductionAgentStateCheckpointSink` own canonical staging and
publication. Session owns durable protobuf decoding; Runner outline semantics
and existing Session maintenance consume its projection.

The existing OCRM loop/recovery responsibility and its GTH evidence context
remain the migration anchor. Existing FBCP `MSG-SEND-SETTLEMENT`,
`MSG-HISTORY-ORDER-GAP` and `STORAGE-MIGRATION-RECOVERY` rows receive a narrow
current-owner update. Historical Grok parity `implemented` rows do not prove
SGF. No source-reading or verification status is automatically upgraded.

## Required behavior

| ID | Requirement |
| --- | --- |
| SGF-01 | Persist the actual completed SendMessage ToolCall bytes, including normalized text/attachment projection, exact tool-call identity, producer timestamps and success/error; a successful real App send preserves its SQL message ID. |
| SGF-02 | Capture independently of optional live observation; fan out the unchanged event after releasing the recorder lock. Preserve public tool JSON, original errors, cancellation and existing side effects. |
| SGF-03 | Each generated invocation owns a fresh private recorder; each delegated call owns an opaque receipt and monotonic sequence. A late worker, duplicate/mismatched callback or reused provider ID cannot attach an event to another invocation/call. |
| SGF-04 | Checked begin, finish and seal/drain refuse poison, integrity faults, active/unfinished calls, duplicate phases, initial-only outcomes and successful sends lacking completed provenance. Both successful staging and GTH use the checked drain. |
| SGF-05 | A known error before any producer observation may become an explicit generated rejection with `Some(empty args)` and no message oneof. Never manufacture a successful send or guess normalized arguments/SQL identity from raw JSON. Initial-only or inconsistent observations refuse publication. |
| SGF-06 | Session emits a sent-message outline/recovery item only from a fully decodable effective SendMessage success with supported decoded arguments. Initial, error, absent/empty result, unsupported args and malformed bytes create no send row. Preserve canonical tool history. |
| SGF-07 | Preserve GTH all-success/count/custody guards, original typed provider errors, permanent completion revocation, checked canonical root binding and no replay/marker acknowledgement after failure. |
| SGF-08 | Actual App-sink, Session/backfill, ordering/isolation and fresh-owner controls must be selected explicitly by the existing exact-H Actions workflow; no local/htch validation or real-provider/package claim follows from controlled Responses. |

## Recorder contract and lifetime

Replace the existing completed-vector and sequence Arcs with one private
recorder under `TurnAgentComposition`. A private child module may organize
that implementation/tests; it does not introduce a product owner. Allocate a
fresh recorder in `with_stream_attempt_runtime`, which the generated stream
calls once per invocation. Internal provider retries retain that invocation's
recorder. A checked one-use begin must refuse a second run on the same derived
recorder before dispatch or clearing state. Reusing the production owner/facade
continues to derive a fresh recorder for each explicit invocation.

Keep `latest_provider_checkpoint` shared: the existing separate completion
probe consumes it. Do not isolate or replace that observation as part of SGF.

Record phase, sequence, active calls, completed calls and sticky integrity
fault under one mutex. A private receipt identifies the exact call; the recorder
Arc identifies its run. Current worker `ThreadId` may locate the synchronous
active slot, but is not a durable identity and must not authorize matching by
provider call ID alone. Nested/foreign-thread callbacks refuse rather than
searching for another matching ID. Do not hold the mutex while delegating or
calling the live observer. An abandoned receipt faults the recorder.

The capture/fanout validates the exact first-party tool identity, frozen tool
variant, exact call ID, initial -> completed phase order, unchanged generated
args/start time, terminal result and completion time. Terminal success must
match the actual flat `sent`, `timestampMs` and `messageId` result; error must
match the producer's existing error formatting. A pre-effect registration
failure can refuse dispatch; a post-effect observation failure must retain
the original public tool result and refuse canonical publication at drain.
Duplicate SendMessage IDs within one run are refused before delegation or
make the run ineligible; they are never joined by taking a latest match.
Producer timestamps remain the actual emitted values. Their presence and stable
initial/start binding are checked; they need not equal the wrapper clock and
wall-clock monotonicity is not assumed. A pre-observation `ToolOutcomeUnknown`
is not a known rejection: it refuses publication while retaining the original
public error and effect custody.

Checked drain seals once, sorts by recorder sequence and returns only qualified
records. No recovery from a poisoned mutex may silently restore publication.
Late old-worker activity cannot reopen a sealed recorder or mutate the next
invocation. Existing unknown/pending effect custody remains authoritative.

Expose captured bytes through a wrapper whose constructor remains private to
the recorder. The generated stream wraps those bytes unchanged in the existing
ConversationStep field 2, reusing the codec's existing envelope operation.
Do not alter frozen protobuf declarations, generic MCP normalization, provider
schemas, persisted schema or CRA checkpoint-sink APIs.

## Consumer compatibility and failure semantics

Use the existing frozen prost types to decode the complete selected SendMessage
payload, including its effective result and arguments. For SendMessage, also
validate the complete AgentToolCall envelope after the existing full tool-oneof
selection. The partial frozen binding must not replace selection across other
supported tool variants. Do not use tolerant first-field/valid-prefix helpers
to recover a success from malformed data. Retain frozen protobuf merge/oneof
semantics, including a later effective error superseding success. Because the
Rust envelope binding covers only a subset of frozen tool variants, the existing
complete tool-selector remains authoritative: repeated SendMessage payloads
merge only while that oneof remains selected. Any intervening recognized other
tool resets the accumulated SendMessage value, including variants absent from
the partial Rust binding. Validate the entire envelope, then decode/merge the
effective selected payloads; an earlier success must not leak across a different
tool into a later argument-only SendMessage.

The existing `build.rs` descriptor generator must include the complete frozen
`agent.v1.ToolCall` root and its reachable declarations, including known
metadata and tool wrappers absent from the partial prost binding. The existing
`generated_occurrence_codec` owner validates every declared field occurrence
before selection, including overwritten oneofs, nested strings, maps and packed
scalars. Use a borrowed wire traversal without constructing a JSON projection;
limit descent to 100 nested messages/map entries/groups from the root, and
refuse invalid tags, overlong/overflowing varints, truncated values and declared
wire-type mismatches. Unknown extensions retain framed-wire compatibility and
their length-delimited bytes remain opaque. The effective SendMessage continues
to use the frozen prost merge behavior, independently of this validation pass.
No frozen protocol declaration or persisted format changes.

Session's outer tool selector and effective SendMessage merge must use the same
borrowed codec framing rules as this validator. A valid unknown group is one
skipped field, including when it occurs before or after the selected send at
the ToolCall root. Tool-looking fields inside that group must not participate
in outer oneof selection. Reuse the codec's strict tag/value primitives and
bounded group skip for the existing Session field reader; remove its redundant
private framing/varint implementation. Existing valid scalar, byte and fixed
fields retain their meaning. Root-group positive controls cover prefix, suffix,
nested tool-looking contents and the 100-descent boundary; unmatched, unclosed,
end-only, over-depth and declared-tool wrong-wire groups remain refusals.

Actions input evidence must include `build.rs` and all frozen generated `.ts`
inputs traversed by that generator. Durable controls include malformed known
field-54 metadata (`b2 03 01 80` and invalid UTF-8), known unbound tool payloads
and wrong wire types before a later SendMessage, and bounded traversal/overflow
refusal. Valid known metadata, ordinary unknown extensions and existing
repeated-message success controls must remain accepted. Each refusal leaves
canonical bytes unchanged and creates no sent-message outline or SQL row.

The generic sink permits `Ok(None)`; its generated success has an empty message
ID. Keep that compatibility rather than requiring a nonempty ID universally.
The real production SQL-sink oracle separately requires its actual nonempty ID.
Existing initial/args-only records lose no canonical bytes, but are no longer
presented/backfilled as proof of a sent message. Never infer missing success
or match old empty generated records to SQL by text, timestamps or position.

## Verification and acceptance for this bounded batch

- Private recorder controls use barriers/channels for reversed completion,
  old-worker/new-invocation overlap with identical call IDs, duplicate/mismatched
  IDs/args/times/phases, foreign-thread callbacks, initial-only outcomes, poison,
  abandoned receipts, checked drains and explicit fresh-invocation reuse.
- The actual App fixture routes controlled Responses through the real generated
  Runner and `ProductionSendMessageSink`, not its direct `fixture.send()` helper.
  Compare public provider continuation, SQL ID, completed event wire and canonical
  inner bytes. Cover observation absent, successful completion and subsequent
  provider failure/cancellation/custody refusal as applicable; fresh reads must
  not resend, redispatch or mint original-turn completion.
- The existing Session positive fixture gains a real success result while
  retaining its outline/backfill assertions. Its old args-only shape becomes
  a negative. Exercise error, empty/missing result, unsupported args, truncated
  outer/nested bytes, invalid known-field UTF-8 and effective later-error cases
  through real blobs, outline, preparation and owner reopen. Successful text
  and attachment controls remain stable; negative cases create no send row.
- Preserve existing SendMessage producer, generated custody/facade-reuse,
  GTH9, CRA8, original-turn, Session outline/recovery and channel App controls.
  Because complete-envelope validation explicitly roots the shared descriptor
  closure and tightens its varint reader, select all 13 existing generated-codec regressions
  with the 15 producer and 14 Session controls in the portable command and
  require their exact positive names on both selected platforms.
  Freeze reviewed H, push/read back exact content, and assess actual command,
  capture, required names/counts, raw hashes and receipts only from Actions.

Acceptance here is limited to the emitted generated contract's faithful
capture/publication and success-aware Session consumer. Rich variants already
projected by the producer, exact reconstruction of lost SQL identity during
backfill, ordinary provider-history replay, external delivery acknowledgement,
MCP fidelity, original interrupted-turn restart, real ecosystem/platform/UI
and all global G01-G12/AC01-AC24 gates remain open unless separately proved.

## Release, rollback and provenance

No persisted schema is changed. Corrected entries use the existing frozen wire
format. A rollback must not replay sends, rewrite prior canonical records or
turn missing/error outcomes into success. Preserve existing explicit recovery,
pending effects and channel delivery state. Diagnostics report fixed phases
and identities without message/credential payloads. No new dependency is added.

Authority: OCRM Spec §17; FBCP Revision 8; FMEP Revision 2 §12; current owner
inventory/matrix; GTH contract in `provider-bound-model-identity.md`. Reuse the
already-assessed fixed-U GTH source chain (`c90c99156ecf10812708c221b014df9900a8ffe0`,
`e5735df007a1a469257c8797e6b2a2ce12e3ff0f`, `9fa2d2954c1d3c9421a94ef5cdc22ad75b3cc3cb`,
`bc558648d2dbd88b153630edbea1b3772c58d993`) without new full-read credit.
Frozen generated contracts: `agent_pb.ts` blob
`3cd3d020e69b54d1a6df27c0f967afe7e9fc942c` and `send_message_tool_pb.ts` blob
`fff9a91c8bf835d33b9d38b855c8785a9ebab03e`; Rust bindings remain in
`runner/agent_v1_wire.rs`. Actual producer, wrapper chain, generated codec,
Session decoder/outline/maintenance and private App fixture were traced before
this spec. Full recursive source/dependency/legal closure is not claimed.

## Actions admission recovery — SGF-CI-01

At published H `497c2c7a7bc5201aead83d3d36e44f52ba164d04`, runtime run
`41569288081` failed before creating any job. GitHub's annotation states that
the 512,095-byte workflow exceeds its 500 KB admission limit. This is
`TEST_FLAW`: no product compile or behavior result exists for that run.

Move the existing portable-contract Python body into
`projects/openclaw-rust/scripts/capture_portable_contracts.py`, alongside the
existing OCRM capture instruments. The same workflow step invokes that file
from the same repository working directory. Preserve its checkout, condition,
step identity, environment, cache, timeout, command order, stdout/capture,
required names and counts, strict acceptance predicates, 14 oracle controls,
receipt fields, palette export and downstream/artifact behavior. The only
Python-body change adds this script to its own source-input hash list. No
command, assertion or gate is removed or weakened, and no product owner changes.

The source move must reproduce the original dedented body exactly after
removing that single declared input-list addition. A fresh focused review
checks this identity and the unchanged invocation context. Only admission and
execution on the new published H can resolve the failure; a smaller file or a
source review alone is not runtime acceptance. Preserve the original failure,
do not rerun unchanged H, and keep all broader acceptance limitations above.

Publication reconciliation preserves incoming Memory subject
`b55657cf87950ca014c2bfe827f5b043adbcab7e`: its complete 945-line portable
body, including the added MSI commands, required names, predicates and input
identities, becomes the extraction preimage. Its surrounding workflow additions
remain unchanged. The earlier 497 extraction identity is historical evidence;
fresh focused review compares the final move against this incoming preimage.

The latest device role-token checkpoint `0e845ce1679e0cdf95346adc54f5d58bf06464e8`
also extends the shared instrument. Preserve its full current body and surrounding
workflow, including DRT selected-role controls and all MSI/DVR predicates. The
same mechanical move and sole self-input addition apply. Earlier preimage
reviews remain historical; the new frozen snapshot requires fresh focused
review and actual combined-H Actions before any acceptance claim.
