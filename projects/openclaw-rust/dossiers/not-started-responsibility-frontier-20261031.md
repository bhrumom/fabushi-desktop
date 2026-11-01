# OCRM Not-Started Responsibility Frontier — 2026-10-31

**Subject**: Fabushi Desktop working tree at the current local branch; upstream
authority remains the discovery-only candidate in `upstream.lock.json`.  
**Purpose**: Record why the remaining `not-started` ledger rows are not safely
implementable from the current production graph, and preserve the next direct
instrument required for each row. The tool-loop detector was removed from this
frontier after its production producer/consumer chain was wired; its frozen
verification remains pending in the parity ledger.

## Current frontier

| Responsibility | Current evidence | Why implementation cannot start from this tree |
| --- | --- | --- |
| `OCRM-RESP-AGENT-ACP-COMMANDS-001` | No production `AvailableCommand`/`get_available_commands` producer; current ACP manager is test-consumed only; historical source path moved to the current chat-command registry. | Requires exact upstream command-registry mapping and a real ACP producer/consumer before adding a second command owner. Blocker: `BL-PLUGIN-SDK-ACP-WIRING-001`. |
| `OCRM-RESP-AGENT-ACP-POLICY-001` | No production ACP dispatch-policy producer or consumer; current production runtime owns a separate Host turn lease/watchdog path. | Requires an exact ACP admission/settlement owner and a production caller; contract fixtures are insufficient. |
| `OCRM-RESP-AGENT-CONTEXT-COMPACTION-001` | Fixed facade delegates to agent-core compaction semantics. Current Fabushi source now has a non-test routed-provider completion producer, guarded SandAgentDb commit, and typed replay consumer; exact fixed-source semantics, live provider proof, and Actions verification remain open. Persisted compaction epochs still belong to prompt/memory snapshots. | Do not add a second owner. Keep the row `not-started/not-wired/not-run/pending` until the partial producer→state→next-turn chain is reconciled with the fixed source, unsupported provider/attachment cases are accepted, and frozen verification is available. |
| `OCRM-RESP-AGENT-MODEL-PRICING-001` | Fabushi has remote model-catalog price projection and `TurnUsage` aggregation, plus transport rate-limit/retry helpers, but no production cost calculator/consumer or provider-candidate admission path. | Requires a selected pricing authority/schema and an existing inference/agent owner that admits ordered fallback candidates and settles usage/cost; do not promote catalog projection or transport retry into a second runtime owner. |
| `OCRM-RESP-CRON-SESSION-REAPER-001` | `AutomationRunPath` creates/finishes FileAutomationStore run history and reports a run id, but no per-run session row, lifecycle revision fence, transcript archive cleanup consumer, or media/subagent preservation query is present. | Requires CRON-REAPER-D01..D05 to select the per-run session representation and existing lifecycle/archive owner; do not treat automation history, generic session deletion, or media-store TTL cleanup as a reaper. |
| `OCRM-RESP-SESSION-AUTH-PROFILE-LIFECYCLE-001` | `session_profile_files.rs` serves agent profile/avatar data; no auth-profile preservation/removal producer or lifecycle consumer is identified. | Requires a real session cleanup/auth owner and credential-preserving contract; profile files cannot be repurposed as auth storage. |
| `OCRM-RESP-SESSION-WORK-ADMISSION-001` | `ProductionSessionWorkers -> SandAgentDb` now owns bounded archive/restore admission, local background-result receipt handling, newly opened media recovery, and typed provider-review continuation admission; native continuation identity still fails closed and the full detached lifecycle remains absent. | Requires the remaining full lifecycle producers/consumers and production acceptance; owner-local state fields and bounded slices do not close the responsibility. |
| `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001` | `bundle_manifest.rs` defines parsing helpers, but no production bundle/archive producer or installer/discovery consumer calls them; Host extension registration currently publishes capability metadata only. | Requires a real plugin bundle ingress, archive custody, install transaction, and activation consumer. Blocker: `BL-PLUGINS-MANIFEST-WIRING-001`. |

## 2026-11-01 frontier-to-ledger set reconciliation

The current frontier table was compared with the live `parity-ledger.json`
using the responsibility ID set whose implementation status is
`not-started`. It contains exactly 8 IDs: 0 ledger IDs are missing from the
table and 0 table IDs are outside the current `not-started` set. Historical
recheck sections below may mention responsibilities that later advanced; they
are retained as evidence and are not part of the current frontier set.

## 2026-11-01 model-pricing and provider-fallback boundary recheck

The fixed upstream source was re-read at the locked OpenClaw revision. Pricing is not
just a token counter: `src/model-catalog/pricing.ts` builds a config/catalog-bound
`PricingContext`, resolves provider policies and authoritative/non-authoritative source
precedence, handles gateway passthrough and one-hop `priceAs`, rejects private or
loopback endpoint inheritance, and exposes `resolveModelPricing`. The model-catalog
core owns normalization of pricing feeds and the distinction between unavailable,
malformed, zero/free, and known rates. Provider fallback is a separate agent-runtime
responsibility: `src/agents/model-fallback-candidates.ts` constructs ordered candidate
chains, while the failover/error and embedded-runner paths decide when a candidate is
eligible after rate-limit/transport failure. `src/llm/provider-rate-limit.ts` is shared
rate-limit admission/backoff evidence, not the model fallback chain or billing owner.

The current Fabushi checkout has only partial adjacent shapes. `source/host/src/extensions/inference/model_catalog_service.rs`
projects remote bundle prices into `RemoteModelCatalogProjection`; it does not expose a
production per-completion pricing context or cost settlement consumer. The shipping
runner carries `TurnUsage` and `merge_turn_usage` in `source/host/src/runner/turn_usage.rs`
and reports token telemetry from `source/host/app/src/main.rs`, but no model pricing
authority, currency/rate interpretation, or accumulated monetary result is consumed by
the turn path. `source/host/src/extensions/inference/llm_transport_rate_limit.rs`,
`source/host/src/runner/transient_stream_error.rs`, and the shell adapter classify or
retry transport failures; they do not select and admit the next configured provider
candidate on HTTP 429. Existing model-catalog and turn contract tests therefore remain
static/contract evidence and cannot close the production responsibility.

Disposition remains `not-started/not-wired/not-run/pending`. The next direct instrument
is an architecture decision selecting the pricing authority/schema and the existing
inference/agent owner for ordered fallback admission, followed by a real producer →
pricing/fallback state → turn settlement consumer path. No second pricing catalog,
fallback manager, or billing store should be added while that owner decision is open.

## 2026-11-01 cron-session-reaper boundary recheck

The fixed upstream source was re-read at the locked OpenClaw revision. The reaper in
`src/cron/session-reaper.ts` is an owner-composed lifecycle operation, not an age-based
filesystem sweep: it resolves `sessionRetention` with `false` and zero disabling
pruning, invalid values falling back to the default, and per-agent/store throttling;
discovers expired isolated cron run session rows through the session accessor; skips an
unavailable/deleting agent; preserves active controller work, pending generated-media
tasks, and unsettled subagent descendants; and deletes through the guarded session
lifecycle mutation with expected session identity/update timestamp while archiving or
cleaning transcript artifacts. The scheduler invokes this operation outside its locked
section so session-store ownership cannot deadlock the cron tick.

The current Fabushi checkout has only the automation-history half of this shape.
`source/host/src/extensions/transcript/automation_run_path.rs::fire_automation_inner`
calls `FileAutomationStore::begin_run`, retains an optional run id, and settles the run
as `ok`/`error`; `AutomationRunObservation` reports that id to telemetry. The run key
used for duplicate suppression is only `agent_id:automation_id`, and the resulting run
record is part of `AutomationRecord.runs`. No production code materializes a cron run
as a session row keyed by that run id, captures a session lifecycle revision or update
timestamp, discovers expired session rows, or invokes an archive/removal mutation with
an expected-row guard. The existing media-store retention helpers and generic session
deletion route do not prove the upstream reaper invariants and must not be promoted to
this responsibility.

Disposition remains `not-started/not-wired/not-run/pending`. A source-backed
CRON-REAPER-D01..D05 architecture decision now exists and is bound to the current
Fabushi exact source subject captured before the documentation-only provenance
publication, but it remains proposed and has not been approved. It names
the durable per-run session row/schema, the existing session/transcript lifecycle owner,
the scheduled sweep producer and fence, the active media/subagent custody source, and
unknown deletion/cleanup outcomes. D01..D05 are not implemented or verified, so this
ADR does not upgrade the ledger or global flags; only after approval can a bounded Rust
slice be implemented and verified against the required negative cases.

## 2026-10-31 plugin-owner boundary recheck

The read-only source search was repeated against the current Fabushi checkout.
`source/host/src/extensions/plugins/bundle_manifest.rs` exposes the format
paths, ClawHub spec parser, archive-size limits, and SHA-256 verifier, and
`extensions/plugins/mod.rs` only exports that module. The only call sites for
`parse_clawhub_plugin_spec`, `verify_archive_limits`, and
`verify_sha256_integrity` are the `openclaw_plugins_contract.rs` tests; no
production Rust caller supplies bundle bytes or consumes a verified bundle.

Two nearby paths are deliberately not promoted to consumers:

- `extensions/mcp/installable_package/ledger.rs` owns the separately scoped
  mini-app install/activation ledger and `VerifiedPackage` transaction. It is
  not a plugin bundle ingress and must not be reused as a second interpretation
  of plugin manifests.
- `extensions/mcp/skill_publish.rs::read_manifest_name` reads only the
  `plugin.json` name from an already-installed skill directory for duplicate
  protection. It does not discover archives, enforce bundle limits, verify
  bundle digests, or activate plugin code.

This recheck strengthens the existing disposition: the bundle-manifest helper
is implementation-shaped test support, not production wiring. The next direct
instrument remains a real marketplace/ClawHub or local-bundle producer through
the existing plugin install owner, followed by one state/receipt boundary and
the activation consumer. No ledger status, completion flag, or acceptance
claim changes from this evidence.

## 2026-10-31 session model-override boundary recheck

The session override path has a real persistence owner and a real read-side
consumer. `agent_session.rs` reads and writes the `sessionModelSelection` KV
through the per-agent SQLite owner and applies the validated transition at
`apply_session_model_selection`. The shipping `runner.startRoutedProvider`
path reads the persisted selection at `source/host/app/src/main.rs:6434-6436`
and projects `model_override`/`thinking_level` into `SandSessionOptions` at
`:7296-7312`; the existing inference owner consumes that option object.

The remaining gap is the producer side: no authenticated non-test caller of
`apply_session_model_selection` was found. The row therefore remains
`implemented/not-wired/not-run/pending`, with the next direct instrument being
an existing Host session-action owner that supplies the model catalog,
visibility/operator authority, placement and generation context before calling
the policy/storage owner. A request-body policy field or a thin new Gateway
route would violate the owner boundary.

## 2026-10-31 session work-admission boundary recheck

The current source also does not contain a production equivalent of the
upstream `assertAgentSessionWriteAdmission` contract. The session runtime has
active-session switching, shutdown draining, deletion and deferred activation
paths, but none checks an archived-session state before admitting new agent
work. The `archive`/`unarchive` branches found in `main.rs` are cloud-agent
lifecycle review actions and do not consume `ProductionSessionWorkers` as a
session work-admission owner. No production caller for the contract's
background-session result or archive/restore review transition was found.

These adjacent lifecycle paths therefore cannot close
`OCRM-RESP-SESSION-WORK-ADMISSION-001`; its status remains
`not-started/not-wired/not-run/pending`. The next direct instrument is a
session-owned admission API that fences writes against archive/delete
generations and returns a durable terminal/review outcome to the existing
turn consumer. No second session store or cloud-agent archive shim is
authorized by this finding.

### 2026-11-01 bounded session-work-admission slices

The historical recheck above predates the current owner-local slices and is
retained as chronology only. At the same frozen H, the live ledger and source
now record `ProductionSessionWorkers -> SandAgentDb` as the canonical partial
owner. The implemented bounded paths are authenticated archive/restore
admission with identity/revision/foreground-stream guards, local background
result receipt consumption, recovery of fenced outgoing media when a session
owner is newly opened, and typed provider-review continuation admission.
Continuation refuses before capability issuance/dispatch when the fixed Rust
Codex Responses producer has no native continuation identity; a response ID is
not promoted into a `threadId`/`turnId` substitute.

These slices improve the producer/owner/consumer map but do not establish the
full detached lifecycle, all upstream terminal-review transitions, or real
provider/device production acceptance. The responsibility therefore correctly
remains `implementation=not-started`, `wiring=not-wired`,
`verification=not-run`, `acceptance=pending`; no global flag changes.

## 2026-10-31 auth-profile lifecycle boundary recheck

The current checkout was re-read for possible credential/profile owners. The
result separates three existing concerns that cannot be substituted for the
OpenClaw auth-profile lifecycle:

- `source/host/src/extensions/session/session_profile_files.rs` reads and
  writes `SandAgentProfile` name/description/title/avatar fields and reads
  avatar bytes from the agent directory or persisted agent snapshot. It owns
  presentation profile data, not provider auth-profile records or session
  credential references.
- `source/host/src/extensions/auth/auth_service.rs` owns the first-party
  Fabushi product credential file and token renewal/read path. Its explicit
  comment states that this credential is not a Cursor/Sand renewal source;
  it has no per-provider auth-profile preservation/removal API.
- `source/electron-main/account/cursor-auth.ts` owns the separate Cursor/Sand
  account credential lifecycle, including secret deletion, operation epochs,
  profile-cache clearing, and retained-after-failed-logout handling. It is an
  Electron account owner, not a Host session auth-profile store and does not
  receive OpenClaw session cleanup identities.

The only Host model-selection reference to `auth_profile_override` is a
session model-selection field in `model_picker_service.rs`; the current
source search found no production auth-profile record store, preservation
producer, cleanup consumer, or authenticated session action that connects it
to credentials. Reusing any of the three adjacent owners would either erase
the distinction between agent presentation and credential state or create a
second auth/session authority. `OCRM-RESP-SESSION-AUTH-PROFILE-LIFECYCLE-001`
therefore remains `not-started/not-wired/not-run/pending`; the next direct
instrument is a real provider-auth profile owner and its session cleanup
contract, not another profile-file helper.

## 2026-11-01 session work-admission architecture decision

The fixed source was re-read at the pinned H for
`session-agent-work-admission.ts`, `background-session-result.ts`,
`session-archive-restore.ts`, and provider-review terminal ownership. The
current Rust source confirms that `ProductionSessionWorkers` owns per-agent
SQLite/transcript state and lifecycle revisions. The deletion window now has a
durable owner-local `deletion_pending` fence in addition to the in-memory
`deleting_agents` fast path; no complete generation-bound background-result
producer or full authenticated archive/restore/review lifecycle exists.

The new owner contract and production-boundary spec therefore define the
minimum migration and preserve the row as not-started until that durable
producer → owner → state → consumer path exists. This is an architecture
advancement, not implementation or acceptance evidence. It explicitly rejects
using cloud-agent archive actions, filesystem age, or the deletion marker as a
second work-admission authority.

References:

- `projects/openclaw-rust/decisions/2026-11-01-session-work-admission-owner-contract.md`
- `projects/openclaw-rust/docs/specs/session-work-admission.md`

The first implementation slice now adds
`ProductionSessionWorkers::assert_agent_work_admission` to the existing
transcript append/update/delete, compaction, and conversation-clear wrappers,
with a focused contract proving rejection during `begin_agent_delete` and
admission after the fence clears. The deletion owner now also publishes and
clears a durable `deletion_pending` flag through the same admission record.
This is the bounded deletion-window sub-contract; the row remains not-started
until durable archive, background-result generation, and provider-review
ownership are implemented and wired.

The owner-local SQLite edge now also fences request-id history and pending
episode-memory record/clear writes, including the Runner's direct episode
cleanup path, plus the bulk transient-state clear. A focused owner contract
preserves both histories and metadata CAS state when the durable deletion
fence is active; checkpoint confirmation is fenced at the same SQLite edge,
as is standalone latest-root CAS, while its pre-materialized blob remains an
explicit external cleanup gap. Session maintenance now reuses the same
`SandAgentDb` root-CAS owner rather than a direct SQLite mutation. This remains
a bounded metadata correction; it does not change the row's
not-started/not-wired/not-run/pending disposition.

Runtime recovered profile-name maintenance also routes through the owner
metadata path; clone/materialization initialization remains explicitly outside
the admitted-session deletion window.
Transcript backfill likewise delegates to the owner append transaction and is
fenced during deletion custody; hidden-entry repair/delete remains a separate
cleanup contract.
Pending-card fallback updates now reuse the owner update transaction and are
fenced by the same deletion custody boundary.
Hidden-entry repair deletion likewise reuses the owner transaction and remains
fail-closed during deletion custody.
Human handoff/replay identity settlement now shares the same fenced owner
transaction.
Conversation clear also reuses the owner transaction and remains fail-closed
under deletion custody; its external blob-store ordering remains a separate
cross-storage gap.
The shared owner KV write boundary now fences the remaining owner-local KV
setters. Profile/avatar mutation and directory deletion now share a per-agent
`ProductionSessionWorkers` serialization boundary, and avatar legacy-pointer
clearing delegates to `SandAgentDb`; this closes the in-process deletion race,
while cross-process crash recovery remains a separate atomicity gap. Memory
gateway remove/clear now routes through the same filesystem owner and bounded
record-restore path; this is a local lifecycle slice and does not change the
responsibility's overall `not-started/not-wired/not-run/pending` disposition.

### 2026-11-01 background-result consumer recheck

The current Rust producer path was re-read at the interrupted continuation
boundary. `run_local_background_revival_turn_with_context` supplies
`agentId`, `agent:{id}:main`, a generated `clientNonce`, and `streamId` to the
existing routed admission, then waits for the terminal event matching that
stream. The `Completed` branch reads transcript entries before and after the
run, derives a sent-message count, and returns it to the revival telemetry
owner. The path does not capture and recheck a durable lifecycle generation at
terminal publication, nor does `SessionWorkAdmissionRecord` retain a committed
terminal receipt from which duplicate or conflicting results can be resolved.

This establishes producer identity facts but not a production
producer → owner → durable result receipt → consumer chain. No authenticated
archive/restore producer or provider-review terminal consumer was identified
in the same inspection. The row therefore remains
`not-started/not-wired/not-run/pending`; the next direct instrument is the
durable result-receipt contract at `ProductionSessionWorkers`/`SandAgentDb`,
with explicit stale, duplicate, conflicting-payload, cancellation, and
outcome-unknown results. A marker-clearing CAS is not sufficient evidence.

### 2026-11-01 context-compaction production recheck

An earlier read of this frontier recorded only the production
`ProductionTurnSummarizationPrompt` surface and no compaction caller. That
historical observation must not be reused after the later implementation
slice. The current checkout's production Runner now builds a real
`ProductionTurnSummarizationPrompt` in
`source/host/app/src/main.rs` and binds it through
`ProductionTurnAgentBuildBindings`; the routed-provider completion boundary
also estimates provider input usage, prepares a cut point, invokes the
summarizer, and calls the owner commit. The only durable compaction owner remains
`SandAgentDb::commit_compaction` plus
`SandAgentDb::get_compacted_transcript_entries`, exposed by
`ProductionSessionWorkers`; the next routed turn reads the typed replay
projection. `compaction_epoch` and `ContextWindowTracker` remain prompt/state
signals and do not independently establish this chain.

This confirms the existing context-compaction spec/ADR boundary at the same
frozen H: the owner contract and a bounded non-test producer → transactional
owner → typed next-turn consumer now exist in the checkout, but they are only
partial implementation evidence. Fixed-source transitive closure, unsupported
provider/attachment behavior, live-turn proof, and frozen verification remain
open. `OCRM-RESP-AGENT-CONTEXT-COMPACTION-001` therefore remains
`not-started/not-wired/not-run/pending`; no manual or test-only compaction route
is added.

### 2026-11-01 compaction durable deletion-fence correction

The compaction owner was rechecked after the existing production Runner path
was found to reach `ProductionSessionWorkers::commit_agent_compaction`. The
facade already rejected deletion or archive state, but the `SandAgentDb`
`BEGIN IMMEDIATE` compaction transaction did not re-read the owner-local
`sessionWorkAdmission` record. It now rejects archived or
`deletion_pending` state inside that transaction and preserves the prior
transcript. A focused contract oracle covers a deletion fence published before
commit and expects `RejectedStale` with no compaction row.

This is a durable write-edge correction only. The compaction responsibility
still remains `not-started/not-wired/not-run/pending` because fixed-source
transitive closure, unsupported provider/attachment behavior, live provider
proof, and frozen verification/acceptance remain open.

### 2026-11-01 provider-review deletion-fence correction

The same owner-local deletion race was found in the existing provider-review
record, continuation-release, and terminal-settlement methods. Their facade
checks were not sufficient at the `SandAgentDb` edge because the durable
record could enter deletion custody between the read and CAS. The owner now
rejects all three mutations when `deletion_pending` is set; focused contracts
cover refusal recording plus release and settlement rejection while fenced.
This remains a bounded write-edge correction and does not change the
responsibility's `not-started/not-wired/not-run/pending` status.

### 2026-11-01 admission-CAS deletion preservation correction

The live owner recheck found a broader race beneath the individual review
guards: the generic `compare_and_set_session_work_admission` method re-read
the current record only by lifecycle revision/session identity, so a mutation
could overwrite a newly published `deletion_pending` fence without advancing
that revision. The generic CAS now requires the deletion flag to remain equal
to the current value. Deletion cleanup uses a dedicated clear operation that
requires the observed current flag to be true before transitioning it to false.
This closes the owner-local fence-preservation edge while leaving the complete
session work-admission responsibility pending.

## Evidence boundary

These are implementation gaps, not acceptance approvals. No row is promoted by the
presence of a helper, contract test, capability metadata, or historical upstream
source path. The next direct instruments are a fresh exact-subject source/dependency
closure and, for each applicable row, a production call graph showing producer,
owner, state/storage boundary, and consumer. The rows listed above remain
`not-started/not-wired` and all global completion
flags remain unchanged. `OCRM-RESP-AGENT-LOOP-DETECTION-001` is intentionally
not listed: its current ledger state is `implemented/wired/not-run/pending`.

## References

- `projects/openclaw-rust/parity-ledger.json`
- `projects/openclaw-rust/upstream.lock.json`
- `docs/specs/openclaw-rust-equivalence-migration.md`
- `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md`

## Current-source correction — 2026-10-31

The current checkout now contains the session-model consumer edge:
`runner.startRoutedProvider` reads `sessionModelSelection` at
`source/host/app/src/main.rs:6434-6436` and projects the persisted
override/thinking level into `SandSessionOptions` at `:7296-7312`. The
remaining frontier is the missing non-test mutation ingress for
`SandAgentSessionStore::apply_session_model_selection`; the responsibility
must remain `not-wired` until an authenticated existing-owner session action
reaches the model-picker policy and SQLite state owner.

## Provider-review terminal settlement recheck — 2026-11-01

At local commit `f9c59205c`, the bounded provider-review path now carries the
captured lifecycle revision and continuation run ID from the authenticated
`sessions.providerReview.continue` Gateway owner into the existing routed
Runner. `SandAgentDb` persists that run identity in the existing
`sessionWorkAdmission` record and settles only a matching accepted review:
success clears the review, Runner cancellation/failure/outcome-unknown retain
non-actionable terminal evidence, and a pre-Runner dispatch error restores
`pending`. The Gateway projection exposes `canContinue` only for pending
reviews. This is a bounded owner-local settlement chain, not full OpenClaw
equivalence or production acceptance; exact-head Actions and real runtime
evidence remain pending, and global completion flags are unchanged.

## 2026-11-01 session-provenance producer sweep

The current Host/Runner producer sweep rechecked transcript mutation sites
that already carry a canonical session identity. Timeline events, channel
inbound materialization, acknowledgement redrive, generated sends, local-tool
permission cards, spend-guard cards, shared-room fanout, agent-to-agent sends,
box-request settlement, Human handoff, Human reaction projection, and remote
Human refresh/identity settlement all route their bound rows through the
existing admitted session owner. Calls with no producer-owned session
identity remain on the explicitly documented compatibility path; no
`agent:{id}:main` selector was synthesized.

Channel-delivery failure marker settlement remains pending for exact-marker
recovery: the worker has a stable operation ID plus partial failure provenance,
but not the complete persisted marker identity needed to safely replace
key-only clearing. This sweep therefore adds no acceptance or global-flag
claim and leaves the session-work-admission responsibility
`not-started/not-wired/not-run/pending`.

## Receipt replay admission-fence correction — 2026-11-01

The live `SandAgentDb` recheck found that an already-committed
`background-result` or `outgoing-media-settlement` receipt could be inspected
before the replay path revalidated the current owner-local admission record.
The receipt contents carried their original session identity and generation,
but that alone did not prove that the current session was still admitted: a
deletion fence could be published without rotating the lifecycle revision.

Background-result and outgoing-media replay now recheck the current canonical
session key, lifecycle revision, archive state, and `deletion_pending` fence
before returning an idempotent result. Compaction-entry replay now performs
the same owner-local archive/deletion admission check before returning
`AlreadyCommitted`. Focused owner contracts cover replay during deletion
custody for all three receipt/entry types. This closes a durable replay bypass
at the existing owner edge; the broader responsibility remains
`not-started/not-wired/not-run/pending` because detached-result/media replay,
provider-review recovery, and exact-head verification are still incomplete.

## 2026-11-01 channel-delivery failure exact-marker settlement

The production channel-delivery failure producer now retains the exact
`markedAtMs` used when writing the durable `ChannelDelivery` pending-wake
marker and carries it in the in-memory failure payload. Recovery reconstructs
the complete observed marker (`agentId`, kind, operation, timestamp, address,
reason, and optional session key) and settles it with `clear_if_matches`.
Failures without complete marker provenance are logged and left pending;
recovery no longer falls back to key-only deletion, so a newer same-key marker
cannot be removed by an older channel-failure pass. This is static source and
contract evidence only; provider, device, packaged, runtime, OCRM-AC01--AC24,
and global completion acceptance remain pending.
