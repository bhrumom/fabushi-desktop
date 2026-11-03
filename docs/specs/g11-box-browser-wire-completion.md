# G11 Box browser wire completion

Status: active; owner: existing BoxExecRuntime and Host browser driver;2026-10-15.
Parent: fdn-g11-host-browser-actions.md HB02/06; FDN-001 R12/13/19/20.

## Context, goal and non-goals
Actual Actions38697503722 starts/authenticates the real canonical daemon but the
first production accessor shell RPC fails. Source proves Host wraps Exec requests
in a Connect envelope while daemon decodes the whole body as bare protobuf. The
daemon also omits frozen Write tag3 and denies browser files under/tmp because
they escape workspace. Complete these existing owners, not a parallel browser or
RPC service. No arbitrary filesystem roots, credential export, installed personal
browser, external Box provider or packaged/extension claim.

## Requirements
- BW01: Exec accepts exactly one bounded uncompressed Connect data envelope when
  Content-Type is application/connect+proto; reject flags, length mismatch,
  trailing/multiple frames. Preserve existing bare application/proto compatibility.
  Body limit16MiB; authenticate existing bearer before dispatch. Preserve response
  Connect stream, IDs, errors and lifecycle. Do not unwrap Control unary messages.
- BW02: Add existing frozen Write tag3 request/result. Args path1/toolCallId3/bytes5;
  result success1/error5/rejected6. Reuse workspace identity/path authority.
  Writes at most8MiB, require existing parent, reject symlink/escape/nonregular
  targets, use mode0600 staging, sync and atomic replacement, remove own staging on
  failure. On Unix anchor traversal/rename to workspace directory descriptors and
  reject symlink components so concurrent symlink replacement cannot redirect.
  Other platforms explicitly reject unsupported secure writes for this slice.
- BW03: Browser runtime driver/results/views/snapshots live under the existing
  authorized /workspace/.sand-browser root, not a new daemon filesystem exception.
  Keep directory private0700. Old ephemeral claims are not migrated: require fresh
  list/claim; no credentials are moved or broad filesystem access granted.
- BW04: Actions unit/protocol tests cover framed success and malformed envelopes,
  frozen Write tags, actual file bytes, overwrite, symlink/escape/size rejection,
  no orphan staging. Full existing daemon/Host tests plus actual12 browser cases
  through production accessor/daemon are mandatory on exact PR and merged main.
- BW05: Actual framed RPC now executes navigation, but run38698699437 shows no
  screenshot bytes: daemon Read always lossily decodes binary as UTF8. When no
  encoding hint is supplied, return frozen Data field5 for invalid UTF8 or NUL
  bytes; retain Content for normal text and explicit text encoding behavior.
  Reject line ranges for binary data. Bound files to8MiB before allocation and
  return a structured invalid-file response on excess. Verify exact binary bytes
  and range/size rejection, plus real PNG retrieval through the shipping client.
- BW06: Run38699218529 passes11/12 actual Rust/Box cases, but history loses its
  RPC response at the fixed15s transport idle deadline while the shipping driver
  permits bounded25s navigation stages. Keep Control/unary deadline15s and give
  Exec stream reads a bounded120s idle deadline, accommodating the driver's total
  connect/navigation/load/screenshot/cleanup budget. This does not extend account
  grants, execution leases or caller cancellation, and never retries a lost call.
  The test watchdog must exceed that transport bound. Require the real history
  and existing mutation/recovery/daemon cancellation contracts before accepting.

## Architecture, strategy and interfaces
Host generated_production.rs remains the client wire authority. BoxExecRuntime
server.rs remains request dispatch/filesystem owner. Existing driver constants and
STATE_DIR select the confined browser runtime path; Host browser executor still
uploads/executes/reads via normal production resource APIs and safety callback.
Spec -> strict envelope decode + existing Write case + confined browser paths ->
focused Actions protocol regressions -> real browser/Rust/Box matrix -> integrate.
Test-only Actions adapter follows production paths and must not translate around
or hide unsupported RPCs. No new schema, auth system or external credential.

## Failure, acceptance and delivery
AC1: BW01–06 focused/whole-owner CI pass; actual direct and Rust/Box12-case suites
pass with actual screenshots and hashes. AC2: exact-main evidence recorded.
External hosted Box, signed extension/native messaging and packaged release remain
independent. Any missing/failed case blocks; do not weaken workspace guard.
Rollback pauses browser dispatch and retains old ephemeral state; never restore
claims by URL or import personal profiles. Record source/run/count/exit, reject
categories and synthetic fixture observations only, no bearer token/profile data.

## Provenance
Actual failure38697503722, Host generated_production.rs encode_exec_server_message /
send_connect_server_stream and daemonserver.rs frozen enum/route/resolve_path read
from current canonical main. User requests completing real G11 environment and all
remaining conditions; this repair is necessary to run the existing production path.

## BW07–BW12: Box response completion and outcome custody (BXR-v1.3)

Status: implementation contract, 2026-11-03; execution and acceptance pending.
Task: OCRM 647e0932-bea4-4113-9a61-65ae3db58955, round 1.
Implementation base: 72b6ed8b6e91a0ac7e0588fd9492a7d30c535079;
fixed OpenClaw U: 1440bb1771372334bea23d7642bb52deb7db1845.
This is a necessary existing-owner public contract under OCRM-M10/M11 and a
dependency of OCRM-RESP-MEDIA-GENERATED-HTML-PROVENANCE-001, not completion of
that media responsibility or a new runtime owner.

### Problem and source boundary

The real daemon echoes request id and exec_id and emits StreamClose(request.id).
The Host currently discards data identity, ignores close/heartbeat identity,
takes the first length-delimited field as a payload (including metadata tag15),
and accepts a Connect body without an end envelope. A transport EOF can
therefore expose unrelated or incomplete Shell/Write/Read output as a result.
ForeverBox also converts the resulting transport errors to ordinary Tool errors,
losing the distinction between refusal and an execution whose outcome is unknown.

The reviewed frozen schema uses request/response id1 and exec_id15; response
metadata also includes local_execution_time_ms39 and hook_additional_contexts45.
ShellResult metadata101–104 is separate from its result oneof. The Host currently
sends empty exec_id, and the daemon's proto3 encoder legitimately omits id0 and
empty strings. These defaults remain compatible. Existing test peers that omit
nonzero identity or StreamClose are incomplete fixtures, not an external
compatibility exemption. Uninspected external peer compatibility stays unaccepted.

The original Shell .proto/reserved-tag history is not available in this tree.
local_exec.proto supplies the separate local-exec generator and uses Struct
placeholders for ordinary Shell; it is not a Box ShellSuccess schema authority.
BXR-v1 allocates no tags, changes no wire schema or generated TS, and does not
produce raw stdout bytes, a generated-HTML grant, a private capture receipt,
a second execution lease, or a new persistence record.

### Requirements and shared interface version BXR-v1.3

- BW07 / BXR01 — Decode the entire existing ExecClientMessage and distinguish its
  declared payload oneof from metadata, regardless of field order. Preserve data
  identity in the existing generic envelope:
  `ExecClientMessage { id: u64, exec_id: String, message: ClientMessage }`.
  Parse omitted scalar/string identity as 0/empty. Known identity fields with a
  wrong wire type, contradictory repeated identity, ambiguous multiple payloads,
  malformed fields or an absent supported envelope are errors. Ignore well-formed
  unknown metadata according to the frozen schema boundary; do not treat arbitrary
  length-delimited metadata as a RawResource. Decode ShellResult's declared
  result separately from its metadata as the adjacent affected boundary.

- BW08 / BXR02 — BoxRemoteExecManager remains the sole per-accessor request-ID
  allocator and response-correlation owner. Before serialization/transport, refuse
  IDs outside the existing uint32 wire range instead of truncating, saturating or
  reusing a value. Preserve the existing create_exec_instance(ctx, serialize(id))
  call shape. BXR-v1 still sends empty exec_id; every received data exec_id must
  equal that actual empty request value. No new nonce or permissive nonempty/empty
  fallback is introduced. Every data, heartbeat and close must match the allocated
  request ID. Throw's omitted optional ID is the same proto3 default0 and must
  match the request; a foreign Throw cannot become this request's remote error.
  Retain matching Throw's error/stack/code fields as an error, never success.

- BW09 / BXR03 — A successful manager return requires exactly one matching
  StreamClose after the response data. Reject data/control after close, duplicate
  close, missing close, empty/unsupported control and identity mismatch. Multiple
  ordered data messages remain legal for the existing streaming resource path.
  One-shot Shell/Write/Read/ComputerUse/RawResource accessors require exactly one
  expected operation result; mixed or duplicate results cannot be silently
  filtered with find_map. Preserve the operation owner's existing tag mapping;
  this batch does not guess mappings for every unsupported raw resource.
  Matching Throw remains a failure and does not fabricate successful completion.

- BW10 / BXR04 — The existing Exec HTTP/Connect reader must prove complete
  framing. Require exactly one legal Connect end envelope, reject missing/end
  duplicates, unsupported flag bits, malformed end objects, trailing data and
  truncated frames. Retain the remote Connect error detail as a failure.
  For chunked HTTP require the terminating zero chunk, complete trailer terminator
  and no trailing unconsumed bytes; preserve valid chunk extensions and trailers.
  Validate a declared Content-Length against the exact received body; reject
  conflicting/ambiguous framing. A close-delimited response may still be consumed,
  but TCP EOF alone never replaces StreamClose plus Connect-end. Use checked
  length arithmetic. Keep existing authentication, request limits, unary15s and
  Exec120s idle deadlines and never retry a dispatched call.

- BW11 / BXR05 — Preserve uncertainty through the existing production error path.
  Add `BoxRemoteExecError::IdExhausted` (before dispatch) and
  `BoxRemoteExecError::ProtocolViolation(&'static str)` (after transport).
  Add `ProductionBoxTransportError::ExecutionOutcomeUnknown(
  Box<ProductionBoxTransportError>)` at the first attempted request write:
  every write/flush/read/framing/status/decode failure after that boundary is
  wrapped once, retaining its cause. URL/header/connection/setup refusal before
  that boundary is not promoted to unknown.
  Expose `ProductionBoxExecError::outcome_unknown() -> bool`: true for wrapped
  transport uncertainty, protocol violations, RemoteThrow, InvalidResult and
  MissingResult; false for IdExhausted, pre-dispatch transport failures,
  ProtectedPath and NoMonitor. ForeverBox's actual Shell, timed/background Shell,
  Read, Write and ComputerUse consumers preserve this as the existing
  ProviderSessionError::ToolOutcomeUnknown. A stream with no terminal operation
  result or an invalid post-execution background identifier is also unknown.
  BXR-v1.3 requires the timed-Shell consumer to inspect the complete received
  event sequence before publishing a terminal result or registering a watcher.
  Exactly one decisive terminal is required; repeated or conflicting terminals
  remain unknown. Preserve stdout/stderr received after Exit through RPC close,
  and treat field8 hook-context as nonterminal metadata, matching the existing
  Shell consumer contract. No early terminal may hide later data or outcomes.
  Preserve the known sandboxUnsupported refusal on background spawn field5
  and timed Shell field9: their shared frozen ShellSandboxUnsupported message
  has STRING sandbox_policy_type at field3, with empty-string proto3 default.
  Retain every legal string occurrence's UTF-8 validation and last-value
  semantics; malformed/wrong-wire values remain unknown after dispatch.
  A complete sole valid refusal projects its command, workingDirectory,
  sandboxPolicyType, reason and isReadonly through the existing consumer,
  creates no watcher and is not missing-terminal uncertainty. Ordinary
  ShellResult has no sandboxUnsupported variant; do not invent one.
  The real Runner bridge must retain that class through its effect-custody
  consumer. No retry, new admission, successful settlement, cleanup authority or
  generated origin may be inferred from an unknown result.
  BXR-v1.3 includes the real Runner's optional MCP output spill: the existing
  large_output_spill helper and RunnerPromptGlue forwarding methods return
  Result<Value, ProviderSessionError>. A known write refusal still returns the
  original inline result, but ToolOutcomeUnknown propagates unchanged through
  RunnerBoxToolBridge; it cannot be swallowed into Ok and receipt.complete().
  Do not issue another spill write, publish an outputLocation or delete a path
  on that unknown outcome. Verify one upstream invocation and one attempted
  spill, with known-refusal and unknown controls.
  The pre-existing navigation finally probe remains an independent observation
  under its existing captured resource owner; it must retain the original
  delegate error and cannot retry that delegate or settle it as success.

- BW12 / BXR06 — Keep real daemon and current Fabushi consumers connected.
  Preserve bare request compatibility, valid default-zero/default-empty peers,
  successful text/binary Read, Write, Shell, multi-data ShellStream, background
  handoff, matching Throw and the existing protected-path/no-monitor controls.
  At least one exact-H Actions positive must use the production accessor against
  the actual daemon, including a nonzero request after a first call; mirrored
  fake codecs alone are insufficient. Real external providers/platforms not
  exercised remain pending, not mock-accepted.

### Ownership, implementation and recovery

The existing Host box_remote_accessor.rs owns correlation and the per-call
Open/Closed response state. generated_production.rs owns wire/framing and phase
classification. The daemon remains the producer; ForeverBox runner_tools.rs
continues to project its results into the existing Runner port and Tool outcome.
There is no new scheduler, Agent loop, Session store, protocol service, persistent
lease or permission truth. React/Electron business logic is unchanged.

The root integrator owns the shared envelope/error contract, spec, composition,
workflow/capture and canonical ledger/RTM amendments. Independent changes use
disjoint paths and the exact shared BXR-v1 contract. Implement and review one
coherent source/consumer/test batch; freeze its final H before the affected
Actions set. No local or htch compilation, parser, test, generator or validation.

On disconnect, malformed completion or post-write error, return unknown and
leave existing operation/resource reconciliation to its owner. This batch
does not add cancellation wiring to a socket, promise child stdout EOF, reclaim
a shared daemon, or grant late work new attempt authority. Restart creates a
fresh accessor/connection and does not replay an ambiguous prior mutation.
No database migration is needed. Rollback is an atomic source batch revert;
it must not silently reinstate automatic retries or weaken preserved media
custody. Diagnostics identify protocol phase/category without adding credentials,
whole command lines or response byte dumps.

### Direct test oracles and acceptance

Extend existing Box manager and generated-production contract targets. Required
positive/negative families are: default0/empty and nonzero sequential IDs;
metadata before/after payload and ShellResult metadata; correct multi-data stream;
wrong data/heartbeat/close/Throw identity; uint32 exhaustion before any call;
missing/duplicate close; data after close; missing/duplicate/end-after-data and
unknown Connect flags; truncated headers/frames/chunks/trailers; valid trailers;
declared-length mismatch; contradictory framing; matching Throw detail; wrong,
missing or multiple one-shot result kinds; pre-write refusal versus post-write
loss; exactly one request despite a lost completion; ForeverBox and the actual
Runner's ToolOutcomeUnknown preservation; actual production accessor-to-daemon
Shell/Write/Read success with a nonzero request and binary bytes.
Known sandbox refusals additionally require actual background/timed consumer
controls using the frozen string field, empty default, valid repeated values,
wrong-wire and invalid UTF-8 occurrence negatives, with one request and no
watcher. Retain every existing repeated/conflicting timed-terminal control.

Keep all existing valid assertions and repair simplified test peers to echo the
decoded request identity and close it. A cancellation test observed to use
`echo $ > abort-pid.txt` is not an accepted PID/death oracle: if brought into
this batch's claimed coverage, repair it to a real parsed positive PID and
prove it was alive before disconnect. Do not claim cancellation from that
historical test name alone.

Reuse rust-desktop-runtime.yml and its existing Linux Host/daemon cache and
portable capture. The selected execution set must compile the affected ordinary
Host, run all affected manager/accessor/ForeverBox/Runner targets, and run the
real daemon contract. Bind the exact command, required names, count, exit,
complete log/artifact, source inputs and H/U. No empty missing-name list, zero-test
success or unchanged historical receipt substitutes for a direct positive.
Keep source G01/G02 scoped authority and independent fresh general/structure
source review separate from runtime evidence. Campaign failures use the existing
classification/recovery policy; no rerun-until-green.

BXR-v1.3 execution input clarification: the existing box-exec-daemon crate has
no tracked Cargo.lock at the selected baseline, and its established workflow
builds it without --locked. The new same-H real-producer control must not invent
a locked build or reuse an unbound cached executable. Build that existing crate
once, preserve the generated Cargo.lock as an output with full bytes and hash,
and bind the resulting executable before and after the control. The Host test
graph remains --locked. Review and persist the actual generated daemon lock
before claiming reproducible producer dependency closure; this output is not
a source input fixed by H and does not close OCRM source accounting. A failed
build prevents the live-producer case from running a stale binary.

BXR acceptance requires every family above and the affected existing regressions
at the final H, plus independent source-review convergence. Source-ready or
native-only success does not accept actual App, packages, external providers,
generated HTML, all platforms, OCRM AC01–AC24/G01–G12 or global completion flags.

### Exact source provenance and retained unknowns

At H862, the independently read protocol inputs were server.rs
a40dc3b5f78e414c3ac302137668f1d5be8dc73f; generated_production.rs
81f5a74dbf0ebc3700f177639bdd875f8c00eb56; box_remote_accessor.rs
fd74139f8e75e9635c85faf43de2637bbcbe4686; runner_tools.rs
c38b3dda53e5d9eccb1581ed3bd6ec38599f7ff6; exec_pb.ts
b3d569d1ad5f923444d472a08efb8bad58c5941f; shell_exec_pb.ts
43079a2f5df12a5f570589ce312fd7fcfb71bd1f; exec_service_pb.ts
da743f25c6a21ec38d1fdeb83dd5fc7f96c12b6f. The four Rust owner blob identities remain unchanged
at base72b6; the manager, codec and ForeverBox source bodies were re-read there.
Runner box_tool_access.rs changed
with the incoming MCP work and is separately re-read at blob
7d856c9f39bf50642d3d757451f0cc954bc47b1b. Incoming changes are preserved.

The protocol assessor returned its complete report in the task conversation;
its final local report write could not be confirmed after workspace loss.
This appendix records the implementation contract and exact source anchors; it
does not invent a saved full-report hash. Raw stdout fidelity, reader EOF/error,
output caps, attempt-to-socket cancellation, private generated-origin grants and
Session persistence remain explicit later work. Generated field contracts are
not claimed as original generator-input or full upstream closure evidence.

### BXR-HTTP-v1 — HTTP octet and streaming-status correction

Exact starting subject: 14e637798b7c76c8530f9a3710051461f793924f.
This corrects BW10/BXR04 in the existing reader and retains BXR-v1.3's complete
identity, typed-field, sandbox, terminal, spill and custody contracts.

Parse HTTP headers as octets separated by CRLF. Header names remain ASCII tokens;
opaque values and the optional reason phrase may include valid obs-text bytes.
Keep HTAB legal in values/reasons, require the status-code/reason separating SP
even for an empty reason, and refuse bare CR/LF, NUL, DEL and other invalid controls.
Interpret only Content-Length and Transfer-Encoding as ASCII framing values.
An HTTP/1.0 response with Transfer-Encoding is faulty framing. The existing
HTTP/1.0 Content-Length and close-delimited compatibility remains available.
An Exec streaming response succeeds only with HTTP status 200 and the existing
complete Connect/RPC termination; other status codes retain a typed HttpStatus
cause inside the single post-write unknown-outcome wrapper. Never replay.

Use complete valid Connect data/close/end bodies when testing header admission,
so missing payload cannot mask the header decision. Preserve all 41 existing
codec/consumer cases. Replace the old incomplete non-UTF-8-header negative with
an actual malformed framing-field negative, and add legal opaque-header/reason,
empty reason, invalid status separator/control, HTTP-version/transfer and non-200
streaming-status controls. Require typed outcomes and one actual request.
Reuse the existing portable capture and its full required-name/exit oracle.
Changes require fresh focused source review and exact-H Actions; no local runs.

Primary protocol contracts: RFC 9112 sections 2.2, 4, 5 and 6.1
(https://www.rfc-editor.org/rfc/rfc9112.html), and Connect Streaming-Response
(https://connectrpc.com/docs/protocol/#streaming-response).

### BXR-LOCK-v1 — freeze the observed producer dependency resolution

The actual daemon build at subject 14e637798b7c76c8530f9a3710051461f793924f,
Actions run 41597864483 attempt 1 job 125789361548, produced the complete lock
already retained in execution-r1/box-response-daemon-resolved.Cargo.lock.
Use those exact 15,520 bytes (SHA-256
9c26a883d53db1c5de9d69c0b6247d87be982eb86c0074ad3cc13526dc491431) as
source/box-exec-daemon/Cargo.lock. Do not resolve a new version set, substitute
the Host lock, or modify the existing dependency declarations. Its 68 package
records include the local daemon and 67 registry packages with exact checksums.

The existing portable capture already detects this path in exact H, includes
its bytes in source bindings, builds the daemon with --locked, and checks lock
and binary identity again after the actual producer control. Preserve that
single path and its build-failure refusal. Require the next exact-H campaign
to observe tracked-lock admission, a successful locked build, actual daemon
operation results and unchanged source/lock/binary through final capture.

This source freeze is generated-output/consumer-contract work and preserves
the original build-output record. It does not assert full registry-archive,
license, security or platform closure, nor change any global completion flag.
Keep the full BXR scope and source approvals; review this necessary lock-input
extension with its exact provenance and existing capture consumer before
publishing the combined production candidate.

### BXR-SHELL-CAPTURE-v1 — foreground execution and capture custody

Contract fixed before implementation at source subject
ac615165744da1b8462a35b07221800c06aad1fb, tree
61ef27820a6ba5dc696a276cd58b30e153a25b57. The producer is
source/box-exec-daemon/src/server.rs blob
a40dc3b5f78e414c3ac302137668f1d5be8dc73f. Fixed upstream U remains
openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845.

The real foreground owner currently ignores read_to_end errors, replaces a
capture-worker panic with empty output, and maps all run_process errors to
ShellSpawnError even after spawn succeeded. These paths can publish a known
result for an execution whose effects or output custody were not observed.
This is a necessary existing Box/Runner contract correction, not a new runtime.

- SC01: Separate refusal before a child is created from failure after successful
  spawn. Preserve known ShellSpawnError for the former. Missing pipes, capture
  setup/worker creation, read, wait and join failures after spawn must remain
  entered-execution uncertainty. Never publish partial or substituted empty
  capture as complete output. Retry Interrupted reads under the Read contract.
- SC02: BoxExecRuntime remains the only child, process-group, pipe and capture
  worker owner. Retain ownership through terminal cleanup. Early failure must
  terminate/reap the owned child where still live, observe both capture workers
  and remove the foreground registration. Do not detach the second worker after
  an error from the first. Cleanup must not signal a stale PID after reaping,
  deadlock on an inherited pipe, or erase the original unknown classification.
  A cleanup error remains unknown; it is never evidence that effects did not occur.
- SC03: Both ordinary ShellArgs and MiniSweAgentBashArgs project post-spawn
  failure through the existing matching Throw control frame and StreamClose.
  Keep existing request/exec identity and transport completion rules. A broken
  connection remains transport uncertainty. Use bounded phase/category
  diagnostics without command payload or captured stdout/stderr. Allocate no
  protobuf field, result variant, public fault option or retry policy.
- SC04: Preserve normal success/nonzero exit, timeout and abort result shapes
  when their execution/capture has been observed, and preserve existing lossy
  UTF-8 text display. This contract does not grant raw-byte or HTML provenance.
  Adapt every actual Rust helper caller to retain the typed failure distinction.
- SC05: The actual Host accessor, ForeverBox resource port, Runner bridge and
  existing tool guard retain RemoteThrow as ToolOutcomeUnknown and mark
  DownstreamOutcomeUnknown before local receipt completion. Further tool admission
  and checkpoint/replay remain refused. Reuse these owners and their custody
  state; do not add a second retry, process registry or permission authority.
- SC06: Use private per-runtime fault injection at actual operations only in
  cfg(test). Negative controls must run a real child and observe a real prefix
  after a counted side effect before read/worker/wait loss; require one effect,
  Throw with no definitive result, bounded cleanup and the real downstream
  unknown-custody behavior. Include stdout/stderr, both foreground dispatch cases,
  pre-spawn refusal, complete text/nonzero/invalid-UTF8 and timeout/abort controls.
  A test-only ignored child entrypoint in the actual daemon test executable may
  serve this real producer over authenticated loopback to Host tests. Bind that
  test executable separately from the normal shipping daemon. No production CLI,
  environment fault control, default feature or dependency change is authorized.
- SC07: Extend the existing portable capture, source bindings and required-name
  checks. Build the actual daemon test executable from the tracked lock using
  --locked and compiler artifact metadata, verify its bytes before/after tests,
  and fail if it is absent/stale/ambiguous. Reuse existing Box 77 direct names and
  downstream custody cases; add the actual producer fault cases and actual Host
  consumer chain without replacing prior assertions. All execution is exact-H
  GitHub Actions only. Fresh complete-scope general/structure review is required.

Foreground Rust state is ephemeral; no database migration is introduced.
Restart must not replay an unresolved request. Existing Runner custody/recovery
remains authoritative. This slice does not claim descendant-process, arbitrary
platform/device/service, raw capture cap, generated-HTML provenance or full
OCRM acceptance. Those responsibilities and the existing eight global flags
remain open. Record actual source review depth, failed results and execution
subject separately from evidence carriers; no mock is real compatibility proof.
