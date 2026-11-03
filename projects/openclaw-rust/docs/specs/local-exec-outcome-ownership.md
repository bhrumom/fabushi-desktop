# LocalExec outcome ownership — LXO-v1

## Authority and scope

This batch continues OCRM at fixed OpenClaw U `1440bb1771372334bea23d7642bb52deb7db1845` (tree `90c8aab39972da11cfad220361781d49e1bc6700`). Discovery SHAs remain historical. Implementation base is Fabushi `4bc25f453108268087e9a11f91aee51680ce2822`, tree `a78ed0bdc19cad8e3f19a3b6ae29f2d440b73213`, on the existing migration branch. Incoming Official MCP and Box HTTP work is preserved.

Root AGENTS, OCRM SOURCE_OF_TRUTH, master Spec §17, and the existing process-exec-shell dossier govern this narrow production correction. Historical dossier reading facts are not reclassified. The batch does not close global source/dependency inventory, generated HTML provenance, remote device compatibility, or final G01–G12 / AC01–AC24 acceptance.

## Problem and existing chain

ExternalShell → ProductionExternalMachineExecutor → HostLocalExecExtension → permissioned GatewayLocalExecSandBox / GatewayLocalExecManager → SandLocalExecBridge → actual provider → shell producer is already wired. The manager owns operation records and a version-2 journal. Runner's existing common tool guard recognizes ProviderSessionError::ToolOutcomeUnknown and retains sticky custody; it does not inspect arbitrary successful JSON.

Current defects are demonstrated by source: outcomeUnknown results are returned as successful tool JSON; manager errors after dispatch are ordinary errors; cancel delivery or generic stream closure can be called Cancelled without Exit; a refused duplicate invocation may finish an older operation with the same ID; active/history moves and snapshot publications are unordered. Multiple facades from one Host extension each load and write the same journal independently. Node system.run consumes the same manager and currently reports unknown as success.

## Ownership and data flow

LXO-01: Keep the existing LocalExec Host extension as the unique operation-state owner. Its cloned facades share a lazily initialized operation owner. Capture the journal path at first operation-owner initialization, preserving current startup configuration timing. Sandbox clones and distinct per-turn permission gates reference that same owner. Directly constructed isolated sandboxes retain independent owners for tests and explicitly isolated usage. No process-global registry, second operation database, second Runner, or scheduler is introduced.

The owner contains the active set, bounded terminal history, journal health and publication ordering. Admission, active-to-history movement, owner-scoped cancellation checks, and durable snapshots are coherent. Serialize mutation and publication through the same owner transaction: no torn snapshot, disappearing operation, or older publication overwriting a newer snapshot. Preserve version 2, bounded UTF-8 output, atomic file replacement/fsync, fail-closed corruption handling and recovery of Accepted/Running to OutcomeUnknown.

LXO-02: An invocation acquires an internal registration identity only after it successfully registers. Use the exact bridge request identity in addition to public operationId for mutations. Refusal before registration must never finish or cancel another invocation. Failed transport admission after registration may settle only its own registration. An agent-scoped cancel must check ownership and send through that same active record without an intervening replacement race. Duplicate admission and terminal movement must be atomic.

## Outcome and wire contract

LXO-03: Only an observed shell Exit establishes an execution terminal: aborted=true is Cancelled; otherwise Completed, even if cancellation was requested earlier. Cancel delivery merely requests cancellation. No Exit plus deadline, receive failure, generic StreamClose, malformed/missing client or control message, opaque Throw, or unexpected file error is OutcomeUnknown after request delivery. Do not replay or infer non-entry from missing Start, error wording or a backend error code.

Positive Rejected / PermissionDenied shell events remain known refusal only before observed execution activity. A contradictory refusal after Start, output or background activity is unknown. A dedicated pre-entry tag has the same requirement. Preserve partial bounded stdout/stderr and stable operation/result identity on error paths. Terminal journal failure after a real Exit preserves the observed Exit facts but reports unknown durability/custody to consumers. An in-memory terminal does not prove a durable commit.

Shared error API: SandLocalExecError::new(message) is known; SandLocalExecError::outcome_unknown(message) is typed uncertain; is_outcome_unknown() exposes the distinction. Keep the existing public message and Display behavior. Shared codec API: GatewayExecControl::Throw retains error, stack_trace and optional error_code. No protobuf schema regeneration is needed: canonical ExecClientThrow already contains optional error_code.

LXO-04: The wire literal LOCAL_EXEC_PRE_ENTRY_REFUSED is emitted only at actual pre-entry refusal sites: provider permission refusal, server-message decode refusal, and the controlled manager's no-handler outcome. The existing LocalExecExecutor.throwControl and production codec createThrowControl gain an optional errorCode argument; old callers remain compatible. Generic handler exceptions, EXEC_BACKEND_UNAVAILABLE and AGENT_EXEC_STREAM_START_TIMEOUT remain uncertain because they can occur after entry. This is a thin protocol adaptation; Rust owns settlement and policy. Unsupported or older providers without the tag remain conservative. A tag after observed activity cannot erase that evidence.

## Consumers and compatibility

LXO-05: ProductionExternalMachineExecutor maps uncertain successful manager results AND typed uncertain manager errors to ProviderSessionError::ToolOutcomeUnknown. Preserve operationId, requestId, stable resultRef, bounded observed output and cause in the error detail, using the existing owner query for an error snapshot where available. Known pre-entry failures remain Tool; normal/aborted Exit retains the existing successful result fields. Status/history remain agent-scoped and cancellation remains owner-scoped. The existing registered Runner/common guard supplies sticky unknown custody; do not add a parallel retry fence.

LXO-06: Actual App node.invoke system.run exposes unknown as a failure with code EXECUTION_OUTCOME_UNKNOWN and operation/result reference plus available output details. Known errors retain EXECUTION_ERROR; confirmed Exit retains success. Complete the existing node invocation response lifecycle without claiming process exit or inventing a new node state store. The LocalExec operation journal remains the execution-result owner. No automatic replay is added.

## Verification and oracle

All executable verification runs only in GitHub Actions after freezing H. Reuse current workflows and build caches; no local or htch tests/builds/lints/parsers/generators/test enumeration. Source inspection, authoring and immutable evidence hashing are not execution claims.

LXO-T01: Through actual bridge delivery and unique fixture request IDs, prove duplicate or pre-entry-refused calls cannot retire/cancel an older live operation, cross-agent cancel cannot reach it, independent Host facades observe one state, concurrent finish/admit/publication remains coherent, and fresh-owner restart retains unknown without replay. A constant request-ID fixture is not valid for these assertions.

LXO-T02: Exercise no-Exit close, cancellation followed by close, silence/deadline, malformed client/control, opaque Throw, post-entry refusal and persistence failure. Assert truthful status, typed uncertainty, bounded partial output and operation identity. Preserve the real-child kill plus aborted Exit positive. Replace the old cancel-plus-close Cancelled expectation because it contradicts the producer contract, not to remove a valid negative.

LXO-T03: Use the actual ProductionExternalMachineExecutor, HostLocalExecExtension/permission owner and registered generated production Runner. Receive the real bridge request before injecting transport frames; do not inject ToolOutcomeUnknown from a mock executor. A counted external effect enters once and remains independently alive when Host reports uncertainty. Verify zero pending effects with sticky DownstreamOutcomeUnknown, no uncertain checkpoint, subsequent tool admission or replay, and terminal retention. Pair with known permission/pre-entry refusal and confirmed Exit controls.

LXO-T04: Actual provider/controlled-manager/production-codec tests distinguish pre-entry tag from opaque or entered failures. Rust generated codec preserves the optional field and rejects malformed oneofs. Existing consumer/wiring contracts remain supplementary rather than the behavioral oracle.

LXO-T05: Actual App system.run tests expose EXECUTION_OUTCOME_UNKNOWN and retain stable operation identity for unknown; ordinary/aborted Exit and local permission denial remain truthful. Existing node lifecycle, LocalExec, Runner custody and current Fabushi controls remain selected.

Capture exact H/U, dependency identities, selected commands, positive test names/summary, exit codes, full logs/artifact pointers and before/after causal inputs. No claim of real external account/device/platform acceptance follows from controlled transport. Review full frozen scope independently with general and configured structure reviewers; repair findings before dependent acceptance.

## Delivery, recovery and completion

Implementation, consumer wiring and tests ship together. Existing journal schema version remains 2; no migration rewrites historical false Cancelled records into asserted truth without evidence. On fresh startup only currently recoverable Accepted/Running records become unknown as before. Old binaries that ignore the narrow tag may report a generic failure; rollback restores code but does not authorize replay of uncertain journal records.

Failures preserve cause and resultRef; journal health blocks new shell admission after persistence failure. Preserve current permission, telemetry, file limits, process capacity, transfer/read contracts and per-turn direction gates. Neither cancellation, a stream ending, nor a successful evidence publication counts as process settlement.

Done for this batch means reviewed source and complete production wiring are pushed/read back, scoped source authority/coverage passes, exact-H affected Actions have direct positives for the stated oracles, and unresolved gaps are recorded honestly. All eight OCRM global flags and existing unaccepted units stay unchanged until their actual authority and acceptance requirements are met.

## Source anchors

- U src/process/exec-termination.ts: blob 8b799aea4cccb13dd5b76b4b25c49b4c025cf5d1, full source read at fixed U; signal delivery is not exit.
- Host manager fd3f23c69ef5841980df39e6c486f311b97c33d1; bridge f19d738cec812920eb28b377015ddc8e240310ca; extension 79024f929209994077b1b6c57fcdaf97b4a36724.
- Runner adapter f4a36930d8b233a5c6a4f9d8d1bf277366ea4a60; common guard aed2d181ba495ad529f4eb083a42a76b6c460439.
- Provider 56a4905d142e12d1be8f642697861f401d57cfc3; controlled manager cf79ca962b78e73d15828181a6e66c6e05e60e4b; production executor ab36e8824bd960750bcf6e0f7b386ed524609244.
- Actual App eafdf6a0cbae3651bb70d55c5cae719e31a89fdb; node invocation runtime 5fe97531210e9bbf1661e24f51873fe792650249.
- Independent source assessment /root/external_unknown_contract at H14e637798b7c76c8530f9a3710051461f793924f, source identities unchanged at this batch base. No execution result claimed.

## LXR-v1 — unresolved outcome retention and bounded admission

This follow-on contract is fixed before implementation against source subject
`4fb025cf9639ab9e7e558b3184d4544fbc80a100`, tree
`dd38bb0a35dab1f159c069e7457c08fc0b4483d6`. The actual existing owner is
`source/host/src/extensions/local_exec/gateway_local_exec_sand_box.rs`, blob
`cdfda2d2ca8c25a5ae1d5b15656a53176bc8d5e9`. Fixed U and all OCRM scope remain
unchanged. This is a prerequisite shared custody repair, before extending the
same owner to file publication; that file-transfer outcome work is not implemented
or accepted by this slice.

The source-established defect is ordinary history churn: finish_operation
unconditionally pops the oldest record at 128, while load_operation_history
unconditionally truncates the older prefix. Either can discard OutcomeUnknown.
The existing duplicate check then forgets that operation ID and may dispatch it
again despite no observed terminal. No executed reproduction is claimed.

- LXR-01: Keep the existing operation owner, journal, agent-scoped queries,
  invocation ticket and common Runner guard. Accepted, Running and OutcomeUnknown
  are protected custody; only Completed, Cancelled and Failed are eligible for
  bounded historical eviction. Evict oldest eligible known records while
  preserving every protected record, its identity and bounded observed output.
- LXR-02: Reserve journal capacity inside the same owner transaction before a
  provider frame is sent. The existing limit bounds history plus active
  registrations. If protected history plus active registrations already occupy
  that capacity, refuse a fresh operation as known pre-entry without provider
  dispatch or mutation of existing custody. A refusal or duplicate cannot retire
  another invocation. Do not add a queue, retry fence or second state store.
- LXR-03: Finishing an operation consumes its already reserved place; it must
  never evict unresolved custody. Permission-scoped facades share this capacity
  and the existing ordered durable publication boundary.
- LXR-04: Upgrade the private journal writer to version 3 as the retention
  semantic boundary. Accept the existing version-2 record shape for migration,
  preserve all recoverable records, convert Accepted/Running to OutcomeUnknown
  as before, and durably publish version 3 before allowing new effects. Version-3
  records use the same fields; reject unsupported versions, malformed input and
  duplicate identities. A failed upgrade/publication fails closed without clearing custody. A failure
  after atomic replacement does not claim that the prior byte image survived
  or that the new image is durably settled. Do not infer truth for old false Cancelled records or
  reconstruct records already lost by an earlier reader.
- LXR-05: A valid legacy image containing more protected records than the current
  capacity is retained intact and remains queryable; new admission is refused.
  Only eligible known history can be pruned. Persist any recovery, migration or
  pruning before using that loaded state. Older version-2-only readers must
  refuse the version-3 image; rollback retains that image and requires a reader
  supporting its custody semantics. Never downgrade or clear it for rollback.
- LXR-06: Extend the existing owner and real-bridge contracts. Require unknown
  retention through known-result churn, duplicate refusal before another frame,
  fresh-owner recovery with identical unknown identity/output, full-capacity
  refusal with no new provider frame, released capacity after a known terminal,
  shared-facade/concurrent admission ordering, protected legacy over-capacity,
  exact version-2 to version-3 migration, failed migration and future-version
  negatives. Keep existing assertions and known refusal/Exit controls.
- LXR-07: Reuse the existing three-platform native LocalExec capture and cached
  workflow commands; bind the new authority, owner/test blobs and required names
  to the same exact H. Existing Runner/App consumers remain required at their
  selected actual H. No local or htch execution, per-file full campaigns, or
  transfer of historical green evidence.

LXR supersedes only the initial LXO-v1 version-2 preservation and indiscriminate
history-retention behavior. The earlier authority remains recoverable at its
original commit. No public wire/protobuf or package dependency changes are
authorized. Global source closure, physical devices, full process-tree shutdown,
file-publication outcome propagation, package/upgrade/rollback acceptance and
G01–G12/AC01–AC24 remain open; all eight global flags stay false.
