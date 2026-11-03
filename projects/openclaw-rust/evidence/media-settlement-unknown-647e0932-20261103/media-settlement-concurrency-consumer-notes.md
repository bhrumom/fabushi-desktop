# MSU: same-ID concurrency and generated consumer decision notes

Task 647e0932-bea4-4113-9a61-65ae3db58955; 2026-11-03. Read-only dependency confirmation; configured model retained, inherited actual xhigh. No implementation or execution claim.

Source subject: `bhrumom/fabushi-desktop@370c6680b6b8c072615875d72be67ca0e45ebe45`, tree `b31903209a22e8fbd77e5eb4a9a865723a98f54c`. Reads used immutable Git objects only. Parent identifies the fdb0 continuation as having the same production chain; the source assertions below are independently bound to the full 370 commit, not to an independently resolved fdb0 abbreviation. This note supplements the preceding MSU reports and does not reopen their scope or replace root's durable spec.

## 1. Decision: the recorder gives a real, narrower guarantee; do not infer sink-wide exclusivity

**Within one generated invocation, repeated first-party SendMessage IDs are rejected before delegation.** `runner/turn_agent_composition/tool_call_recorder.rs:248–270` holds its state mutex, requires the recording phase, refuses nested synchronous registration on the same thread, validates the first-party definition, and inserts the ID into `send_message_ids`. A duplicate returns a recorder error. The set remains for the invocation, including after completion. `CheckpointRecordingRoutedToolBridge::call_tool` registers before calling its delegate (`turn_agent_composition.rs:131–161`), and the wrapper is installed around the production toolset at 1060–1068. This is stronger than an assumption that a provider usually returns unique IDs, and must be preserved.

It is **not a sink-wide resolve/send/abort ownership guarantee**:

- `SendMessageSink` is `Send + Sync` and its methods take `&self` (`runner/tools/send_message_tool.rs:61–100`). `SendMessageToolBridge` merely holds an `Arc<dyn SendMessageSink>` and has no same-ID lock (`207–220,388–451`). The same accepted call invokes resolution, encoding and send as separate synchronous operations.
- The actual App creates a shared Counting/Production sink (`app/src/main.rs:11563–11585`), then clones it for MCP callbacks. The connector callback directly calls `send_message` with `mcp-management:{server_id}:{variant}` (`11587–11615`). The routine callback directly calls it with `{tool_call_id}:listener:{platform}` (`11623–11666`). These sends are not registered as a first-party SendMessage invocation by that recorder. There is no sink-level restriction reserving those string namespaces against a provider's supplied call ID.
- A new generated invocation replaces its recorder while retaining cloned composition dependencies (`turn_agent_composition.rs:510–518,1028`). The comment explicitly acknowledges that a prior worker may still retain its old composition. `production_turn_agent_owner.rs:530–532,564–580` clones composition for a subsequent invocation. Per-invocation dedup therefore does not establish a permanent same-sink ID fence.
- The actual tool execution guard spawns a worker (`runner/tools/mcp_meta_tools.rs:187–211`). Effect admission counts revisions, does not receive a tool-call ID, and permits more than one pending effect while no unknown result exists (`runner/stream_attempt.rs:407–432`). Its timeout does not stop the worker. It is a custody mechanism, not a same-ID mutex.

Consequently, retain the existing recorder guarantee, but do not make MSU cleanup correctness depend on it covering every direct sink access, old invocation or re-entrant callback. This is a source-derived boundary assessment; no concurrent failure was executed.

## 2. Per-entry locking is an appropriate local change, with three necessary qualifications

The proposed single index is appropriate:

```rust
pending_media: Mutex<HashMap<String, Arc<Mutex<PendingOutgoingMedia>>>>
```

It remains the existing App media owner. It does not require a second state store, new Session owner or global lock. Recommended operational contract:

1. Under the map mutex, find/create the exact entry and clone its Arc; immediately release the map mutex. Keep that Arc alive while using its guard. Helpers should return the Arc, not a self-referential guard borrowing a temporary Arc.
2. Resolve, send and abort use `try_lock` on that entry. Hold the entry guard through the relevant materialization/admission/settlement or known-abort operation, so another same-ID operation cannot cross its state transition. Never retain the map guard across filesystem work, Session/DB calls, listeners or App events.
3. On `WouldBlock`, resolution/send refuse with `ToolOutcomeUnknown` and do not materialize, recapture, append or release. Abort is void, so it must simply retain the entry and do no cleanup. A re-entrant call fails promptly instead of waiting on its own non-reentrant lock.
4. On poison, refuse normal operation and retain the entry. Do not remove it or resume SQL/filesystem work through the poisoned guard. Custody must be sealed before owner append so an unwind after a possible write cannot leave Drop eligible to delete files. Any poison-recovery used only to disarm cleanup is not permission to continue the operation.
5. Before owner append, transition the exact entry from collecting/prepared to owner-settlement-pending. Hold its guard through result classification. A typed unknown leaves it sealed; only confirmed precommit refusal permits rollback, and only confirmed success permits committed disposition. Release the entry guard before publishing success callbacks that do not need to mutate it.

### Qualification A — busy must not cause a later abort of the winner

`SendMessageToolBridge::call_tool` currently invokes `abort_attachment_custody` for **every** build error (`send_message_tool.rs:409–414`). Merely making a busy resolver return unknown is insufficient:

1. Resolver A owns the entry and is materializing/capturing media.
2. Duplicate resolver B gets busy and returns unknown.
3. A releases its entry guard with prepared media.
4. B's outer build-error arm now calls abort; it can acquire the entry and delete A's files.

**Minimal fix at the existing Tool owner:** the build-error arm must preserve `ToolOutcomeUnknown` without invoking abort. Known pre-send failures retain their cleanup behavior. This also protects already-unknown attempts from the existing bridge's unconditional cleanup. The send-error arm already returns the original error without another abort (`470–481`). Do not weaken this to a string check or assume abort will still see the entry busy.

Abort itself must still inspect the entry disposition and retain settling/unknown/committed entries, because other callers can invoke it directly. Once send holds the entry lock through settlement, a concurrent abort cannot acquire it; after release the terminal disposition governs cleanup.

### Qualification B — do not recreate the same media identity after finalization

Removing a map entry while another operation holds a cloned Arc allows an old detached entry and a newly created entry to coexist for the same key. Removing success or known-abort entries also lets a later resolve materialize/recapture under an already-used media work ID.

The smallest solution within the **same index** is to retain compact terminal entries for media-tracked IDs until the sink is destroyed: committed, aborted/refused and unknown. Clear unnecessary cached path/result payload after safe finalization, but retain enough disposition/identity to reject re-entry. Unknown retains the recovery-relevant custody. No second completed-ID set is needed.

This does not require creating tombstones for all non-media sends: leave the existing no-pending-entry path for unrelated text/connector sends unchanged. An operation that already cloned an Arc observes that same entry's terminal phase after obtaining its lock. It must never reset that phase or call `or_insert` to replace an unknown/terminal identity.

If root instead removes known terminal entries, it must separately solve both stale-Arc re-entry and subsequent same-key recreation; `Arc::ptr_eq` alone solves only part of that problem. Retaining the terminal media entry is the simpler bounded-lifetime choice.

### Qualification C — multiple resolutions with one ID are legitimate

The text-with-images producer resolves each image using the same tool-call ID (`send_message_tool.rs:264–280`). A blanket “one resolve per ID” rule would break that existing behavior.

Before settlement, an entry may collect distinct source URLs for that one message. To prevent a repeated resolution of the same source from creating/capturing another file, retain a small source-to-resolved-result cache **inside that PendingOutgoingMedia entry**. An exact repeated source returns the existing resolved URL/name without persistence or Session capture. This is local materialization provenance, not another custody index. A distinct source remains allowed while collecting; after the entry is sealed or terminal, every resolution refuses before materialization. Preserve the existing image-batch behavior and the R6 resolver's ordinary error propagation.

The entry lock makes these method operations mutually exclusive. **It does not by itself authenticate that two non-overlapping method calls using the same string belong to one original invocation.** That stronger claim requires the existing recorder's invocation guarantee or an additional operation identity contract. Do not advertise arbitrary same-ID workflow isolation solely from `try_lock`. For this batch, enforce the concrete invariants above: no overlapping mutation, no busy-triggered destructive abort, no materialization after seal, no recapture of the same source, and no second settlement of a terminal media entry. The current generated first-party route provides the stronger invocation dedup within its declared boundary.

## 3. Actual generated Runner observation points after an App ToolOutcomeUnknown

### Sticky effect custody is the decisive terminal observation

The App error travels through the Counting sink and SendMessage Tool bridge unchanged. The Counting sink increments only on `Ok` (`send_message_tool.rs:166–176`); the bridge emits a completed **error** observation then returns the original error (`450–481`).

The actual guard worker checks for `ToolOutcomeUnknown`, marks `DownstreamOutcomeUnknown`, and only then completes its local receipt and sends the result (`mcp_meta_tools.rs:198–209`). The runtime's unknown result is sticky; checkpoint acceptance requires both no pending effects and no unknown result (`stream_attempt.rs:435–481`). Later tool admission refuses once unknown is recorded (`421–423`).

The provider attempt's cancellation predicate includes unknown custody (`production_turn_run_shell_adapter.rs:440–446`). After the provider returns, the adapter checks custody again (`470–490`). Its retry path checks `custody.retry_block_reason()` **before** retry policy and returns an error at **552–573**, with a `GaveUpIneligible` report. It does not call the normal retry callback for that rejected retry.

The exact composition's snapshot is frozen only after generated lifecycle persistence returns (`generated_agent_turn_stream.rs:651–665`). The actual production owner exposes it as:

```text
runner.last_tool_effect_custody()
  -> Known { run_generation, snapshot }
snapshot.unknown_result == Some(DownstreamOutcomeUnknown)
```

Source: `production_turn_agent_owner.rs:571–585`; public Runner forwarding is `sand_agent_runner.rs:432–433`. In a single synchronous fault case with no competing timeout/cancellation, expect one started effect, no accepted checkpoint covering it, zero pending effects after worker completion, and the downstream-unknown reason. If another loss occurred first, the sticky first reason can differ; isolate the intended case rather than forcing an ambiguous observation green.

### Do not assert the final outer error is still ToolOutcomeUnknown

The Fabushi/Responses adapter maps tool errors to `CodexDirectError::Tool(error.to_string())` (`extensions/inference/provider_session.rs:1673–1681`). The shell can replace the provider result with an unresolved-custody Tool error (`production_turn_run_shell_adapter.rs:485–488`), and the retry refusal itself returns `ProviderSessionError::Tool("Runner refused automatic retry: ...")` at 571–573.

Therefore test the typed unknown at the **actual App/sink boundary**, and test sticky custody plus non-success/no retry at the **generated Runner boundary**. Require target `SendMessageDeliveryCounter` count zero and retry-event count zero in a one-tool case. A retry-report event explaining ineligibility is allowed and useful; it is not an executed retry.

### Unknown does not publish failed SendMessage canonical history

This point is definite in current source. On a composition error, `generated_agent_turn_stream.rs:380–417` invalidates completion authority, drains checked tool observations, then **skips staging/persistence** if any recorded call is an error or custody is unsettled (`391–396`). Thus a completed-error live tool event can be recorded in memory without becoming a newly persisted canonical Agent step or Journal tool-error record.

The recorder itself can qualify an observed completed error faithfully (`tool_call_recorder.rs:156–164,289–325`), but the generated history owner deliberately refuses to publish this unknown invocation. Its earlier success-only interrupted-history route is not available here. Final result errors also invalidate original-turn completion at `generated_agent_turn_stream.rs:651–656` and `production_turn_agent_owner.rs:500–505`; successful lifecycle hooks run only on `Ok` (`593–595`).

Consequences for MSU oracles:

- **Lost ACK:** actual SQL send/receipt may exist and media bytes must survive. That does not authorize a successful tool result, a canonical SendMessage success step, a new final Assistant step, a completion receipt or a second send to “repair” history.
- **Rollback refusal/retirement:** the target SQL send is absent from an independent fresh connection; durable pending-media input remains until existing ordinary recovery. This invocation still contributes no new canonical tool/error history or completion authority.
- Compare canonical checkpoint/blob/Journal state with the pre-invocation baseline that actually existed. Do not apply the existing SGF success-history oracle, which requires two successful sends and their generated bytes.

## 4. Shortest existing fixture extension and required controls

Use `app/src/tests/channel_host_lane_tests.rs` without introducing a parallel fixture owner:

- Add direct private-sink concurrency controls beside `ChannelHostLaneFixture` (`397–472`). Existing `sink: Option<ProductionSendMessageSink>` permits access to the per-entry Arc. A test can deliberately hold the exact entry mutex, invoke another same-ID operation on a scoped worker, and require prompt refusal; this is a lock-boundary control, not a fabricated DB outcome. It does not need another global fault hook.
- Extend the existing `generated_send_message_fidelity` module (`1538–2320`) for one actual attachment tool call through the real Counting sink and `TurnAgentComposition` (`2035–2077`). Arm the previously designed cached-owner transaction fault before dispatch. Retain old successful cases unchanged.
- Its existing response fixture needs an explicit one-tool unknown case; do not leave it waiting for the successful two-request/two-send continuation. Observe request count, retry callback count, send counter, live completed-error event, `last_finished`, sticky custody and invalidated completion authority. Use an unknown-specific history comparison.
- Reuse its owner disposal/reopen path (`2181–2217`). Drop every extra concrete sink Arc held by a test before claiming destructor evidence. Check SQL/media separately from canonical history; ordinary fresh Session recovery is allowed to clean only the uncommitted case.

Minimum concurrency controls, all proposed for Actions only:

1. Same entry busy: resolve/send refuse promptly, abort retains; unrelated entry remains usable. No map mutex crosses the held operation.
2. Busy resolver through the actual SendMessage Tool bridge: arrange the winner's release before the loser's error handling finishes; verify the loser never aborts the winner. This checks the necessary unknown-specific build-error change.
3. Same source resolved twice before settlement: identical returned normalized URL, one owned file and one recorded durable path; distinct images still work.
4. Unknown and committed media entry: later resolve/send/abort cannot recapture, append or delete. Retained old Arcs observe terminal disposition; no second entry is created for that media ID.
5. Sealed entry poisoned/unwound: further operations refuse, and dropping the last sink/entry handles retains bytes needed for owner recovery.

No executable validation was run. Only this agreed notes file was written; production source, manifests, public contracts and schema remain untouched. The bounded conclusion is that the existing recorder's guarantee should be credited but is insufficient as a universal sink contract; the per-entry approach is appropriate when its abort, terminal-entry and repeated-resolution edges are explicitly implemented and tested.

### Immutable source identities newly relevant to this confirmation

| Source | Blob |
| --- | --- |
| `runner/turn_agent_composition.rs` | `c988b9a02db95ed5106a34d67bdd5c9ff30447d6` |
| `runner/turn_agent_composition/tool_call_recorder.rs` | `8078fbc13ccc66ae70a479d9d18fb6380db2767e` |
| `runner/generated_agent_turn_stream.rs` | `4617051c700665287008c8cec225ae333af02387` |
| `runner/production_turn_agent_owner.rs` | `6e52b116611257ff3a4bcd18eb81cce66bf2097f` |
| `runner/production_turn_run_shell_adapter.rs` | `d0fe2714923338dd717284f08f05eef438db69d9` |
| `runner/stream_attempt.rs` | `6eb8a3eae5556eb6d6a6c03d00059d7c1180190f` |
| `runner/tools/send_message_tool.rs` | `5a2afd591da79f24652b59eef2be6c05ecb1134c` |

All listed paths are beneath `source/host/src/`. Primary source links: [invocation ID refusal](https://github.com/bhrumom/fabushi-desktop/blob/370c6680b6b8c072615875d72be67ca0e45ebe45/source/host/src/runner/turn_agent_composition/tool_call_recorder.rs#L248-L287), [shared App sink callbacks](https://github.com/bhrumom/fabushi-desktop/blob/370c6680b6b8c072615875d72be67ca0e45ebe45/source/host/app/src/main.rs#L11563-L11666), [unknown-history refusal](https://github.com/bhrumom/fabushi-desktop/blob/370c6680b6b8c072615875d72be67ca0e45ebe45/source/host/src/runner/generated_agent_turn_stream.rs#L380-L417), [retry custody fence](https://github.com/bhrumom/fabushi-desktop/blob/370c6680b6b8c072615875d72be67ca0e45ebe45/source/host/src/runner/production_turn_run_shell_adapter.rs#L552-L573).
