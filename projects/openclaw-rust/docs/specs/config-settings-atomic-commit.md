# Atomic configuration commits through the existing Settings owner

Status: implementation and tests authored; executable verification not run  
Contract: `OCRM-CONFIG-CAS-v1`  
Date: 2026-11-02  
Implementation base: `3b797be6b65095fd16b0a978e5fcd19a8ab504b3`  
Integration input: main `5528062177e3865eef60a073a066cca144f9e3e8`  
Fixed upstream: `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`

The worker inherits its parent model and effort. The assigned effort is xhigh
because snapshot, permission, account and persistence lifetimes interact; no
separate host effort override was applied for this continuation.

## Problem and existing authority

At the implementation base, the authenticated `config.set` Gateway checks
`baseHash` before `SettingsService::set_host_settings` obtains its writer
mutex. Concurrent connections can therefore both accept the same stale
version. The notification getter and several account, timezone and MCP
mutations also persist a complete settings document without that mutex. A
stale writer can overwrite an acknowledged model choice. `get_host_settings`
assembles its projection through multiple file reads, so its hash need not
describe a single committed snapshot.

This batch repairs Fabushi's existing configuration contract. It does not
introduce new upstream model-selection semantics or promote historical source
review into current upstream acceptance. The relevant existing responsibility
records are `OCRM-RESP-CONFIG-SINGLE-WRITER-LOCK-001`,
`OCRM-RESP-CONFIG-IO-SNAPSHOT-001` and
`OCRM-RESP-CONFIG-SCHEMA-NORMALIZATION-001`; the historical source identities in
`../../dossiers/config-settings-scoped.md` remain historical. Project authority
and final acceptance remain governed by OCRM-001, especially its 2026-11-02
execution section 17.

## Goal, scope and non-goals

An accepted configuration update has one serialized comparison/commit point,
returns its own committed settings and hash, and survives reopening the same
settings file. A rejected update has no mutation or observer effects. Existing
inference consumers continue reading this same Settings owner.

There is no new configuration store, lock owner, generation ledger, session
store, permission owner, model catalog, provider-selection policy or Agent
loop. The schema and existing normalized JSON hash remain unchanged. This
batch does not implement live model switching, fix provider-specific model
dispatch, change authentication or claim cross-process filesystem locking.

The fixed upstream uses an installation-key HMAC revision token
(`src/config/config-revision-token.ts`) and carries the original snapshot hash
into the lock-owned compare/commit path (`src/config/config-write-flow.ts`).
This v1 batch preserves Fabushi's existing public SHA-256 serialization for
compatibility; it does not implement or claim equivalence to the upstream
opaque revision token. The existing public settings projection clones `agents`
and `skills`, including stored `skills.entries.*.apiKey` values. This is an
existing sensitive-data exposure, not a secret-safe boundary established by
this change. Public redaction, an HMAC revision token and installation-key
migration remain separate required work; tests here use synthetic values and
must not log real settings or secrets.

The worker may change only:

- `source/host/src/extensions/settings/settings_service.rs`;
- `source/host/tests/settings_service_contract.rs`;
- `source/host/tests/openclaw_sessions_and_config_contract.rs`;
- `source/host/tests/inference_extension_composition_contract.rs`;
- this specification.

The integrator owns the corresponding `source/host/app/src/main.rs` Gateway
wiring and all shared ledger, workflow and evidence publication changes.

## Requirements and interface

| ID | Required behavior |
| --- | --- |
| CAS-01 | One existing `SettingsService.write_lock` covers loading a single source snapshot, comparing its normalized hash, validating/merging the update and committing it. Two changes to that normalized persisted projection sharing one base hash admit at most one writer; runtime-only feature overrides retain their existing non-persistent semantics. |
| CAS-02 | `HostSettingsSnapshot { settings: Value, hash: String, config_source: Value }` binds the returned normalized JSON to its SHA-256 and captures existing path/exists/valid/rawHash provenance while the same writer lock is held. All reply fields are captured before callbacks and cannot become a later writer's reply. |
| CAS-03 | `HostSettingsUpdateError::Conflict { expected, actual }` means expected=current normalized hash and actual=supplied base hash. Gateway maps it to the existing conflict/409 outcome. `Rejected(String)` retains existing validation, readonly and persistence failures. |
| CAS-04 | `set_host_settings_if_current(base_hash: Option<&str>, update: Value)` returns the snapshot or typed error. The existing `set_host_settings(&Value) -> Result<Value, String>` delegates to this owner with no base hash. `get_host_settings_snapshot()` and `get_host_settings()` use one source snapshot. |
| CAS-05 | Conflict checks do not write even legacy notification normalization and do not notify listeners. All existing complete-document writers, including normalization reads and account/MCP/timezone/onboarding mutations, use the same writer lock. Read normalization continues persisting disabled notifications where allowed. |
| CAS-06 | Existing account transition order, include-owned roster refusal, readonly/Nix protection, model and permission normalization, unknown stored fields and notification defaults remain intact. A stale transaction cannot restore account-sensitive fields after a scope change. |
| CAS-07 | Successful notification-producing writers enqueue one notification batch while holding the existing writer mutex. A single owner-local drainer delivers complete batches in commit order outside both writer and queue locks. Callback reads and follow-up writes cannot deadlock; the original reply remains its own commit even when a callback performs another update. |
| CAS-08 | Failed IO does not emit a commit notification or report a candidate as persisted. Reopening the service reads only the last successful document. Existing inference configuration reads observe committed provider/model settings and retain existing packaged-provider restrictions. |
| CAS-09 | Only `NotFound` initializes an empty settings document. An existing unreadable, invalid-JSON, non-object or unsupported-version source rejects Result-returning mutations. Compatibility getters retain their return types and may project defaults on read failure, but must not normalize those defaults back to disk; source metadata retains its existing invalid/error indication. |
| CAS-10 | Reentrant or concurrent mutations may return their persisted receipt while another caller drains notifications. They enqueue without waiting for callback completion. Listener unwinds cannot permanently retain drainer ownership: remaining listeners and queued batches are delivered before the first panic resumes on the active drainer's caller. No callback runs with either owner lock held. |

## Production chain and transaction design

Authenticated Gateway request -> `UnifiedGatewayApi::call_openclaw_config_set`
-> `SettingsService::set_host_settings_if_current` -> existing `settings.json`
temporary-file/rename publication -> existing
`ProductionInferenceSettings` / `ProductionAgentInferenceOwner` -> the next
`runner.startRoutedProvider`.

The Settings owner projects an in-memory document through pure normalization;
projection never calls another file getter or writer. The Gateway consumes the
returned snapshot instead of calculating a check outside the owner or reading
the file again after commit. Unconditional updates retain their existing
semantics. Existing specialized writers hold the same mutex from load through
persist, and release it before observer calls. No new persistent format or
runtime dependency is needed.

Current production composition constructs Settings once in
`host_production_extensions.rs::start_production_host_extensions`, passes the
same Arc to `start_production_inference_extension`, and the app clones
`production_extensions.settings` for Gateway services. This mutex contract
covers that shared owner. Independent `SettingsService::new` instances or other
processes opening the same path do not share it and are outside this batch's
locking guarantee. Existing `configSource` is content-dependent provenance,
not immutable environment information, so Gateway must use the snapshot's
captured `config_source` rather than independently reading it after commit.

The compatibility notification getter may still normalize the stored disabled
notification field, but only after obtaining the same writer mutex and loading
the current document. The CAS comparison itself uses a pure projection, so a
conflicting request cannot modify the file as a side effect of checking it.

Independent review identified a callback-order regression in the initial
implementation: A's timezone listener can synchronously commit B, causing B's
feature overrides to run before A's remaining overrides. The production
Experiments subscriber then ends on stale A. The repair is an in-memory FIFO
inside the existing Settings owner, not a scheduler, new store or public
generation. Enqueue order follows the writer mutex. One caller drains complete
commit batches; reentrant writes append to the same queue and return their
receipt. Existing silent notification normalization remains silent, and each
specialized writer retains its previous event fields and event conditions.

Within a host-settings batch, secret audit reporting precedes timezone,
feature-override and general-change delivery, retaining the existing order.
The queue carries observer payloads and precomputed secret-free audit findings,
not complete settings documents. Audit preparation is pure and occurs before
publication; only successful commits enqueue reports.
The drainer catches individual callback unwinds so later listeners and batches
can run, then releases drainer ownership and resumes the first panic. An unwind
guard also releases ownership if dispatch itself unexpectedly unwinds. No
callback executes while a writer, queue or listener-registry mutex is held.

Persisted receipt and notification completion are distinct: while A is in a
callback, a reentrant B can be published to the settings file and return its
own receipt before A's remaining observers or B's observers run. After A finishes, B's
notifications follow. No new durability guarantee is made for pending
in-memory notifications across process termination. A callback that never
returns still blocks notification delivery; it does not retain the writer
mutex or prevent later commits. Panic-abort behavior cannot be recovered by
Rust unwind handling.

A resumed listener panic does not roll back an already-published document or
become `HostSettingsUpdateError::Rejected`. It resumes on the caller that owns
the active drain and may originate from another commit in that queue. Pending
notifications can accumulate while a callback blocks; this batch adds no
separate worker or backpressure policy.

`featureFlagOverrides` remains the existing runtime-only notification input;
neither Settings nor `HostExperimentsExtension::replace_feature_flag_overrides`
persists it. The ordering oracle applies real Experiments overrides and checks
the final runtime value against B's input, while a provider field committed in
the same transactions establishes B's persisted receipt and disk state.

## Failure, cancellation and recovery

Readonly and include-owned validation remain explicit failures. Validation and
IO failure return before notifications; temporary-file publication retains the
existing rename boundary and cleanup. Unknown stored fields are preserved in
the candidate document, while the public projection retains its existing
field set.

The internal strict read path rejects existing source read/parse/version
failures before any write. It is shared by the Result-returning settings
writers. `get_host_settings_snapshot`, `get_host_settings` and the frozen
notification getter cannot report a new error through their legacy return
types; they retain compatible default projections on unreadable input and
skip normalization persistence. A subsequent conditional update still fails
with `Rejected`, even if the caller supplies the projected default hash.

The synchronous settings API gains no invented cancellation/rollback promise.
A disconnect after publication leaves the outcome unknown to the caller;
read `config.get` and reconcile its hash before retrying. A process restart
uses the same settings path and format. Updates affect subsequent settings
reads, not the already-captured configuration of an executing turn.

## Verification and acceptance

All executable checks run only in GitHub Actions at the integrator's frozen
target H. Source inspection and test authoring are not passing evidence.

| Oracle | Requirements | Direct observation |
| --- | --- | --- |
| CONFIG-CAS-O01 | CAS-01, CAS-03, CAS-05 | Two callers race different effective changes against one real owner/hash; exactly one succeeds and one conflicts, disk matches the winner, only one update notification occurs. |
| CONFIG-CAS-O02 | CAS-02, CAS-04, CAS-07 | A synchronous notification reads and writes the owner; the first call returns its own earlier snapshot/hash/configSource, while a subsequent read sees the callback's later commit. |
| CONFIG-CAS-O03 | CAS-03, CAS-05, CAS-06, CAS-08, CAS-09 | Stale, readonly, invalid/unreadable-source and IO-failing writes leave bytes and observers unchanged; getters do not overwrite an invalid source, and a stale check on a legacy notification document performs no normalization write. |
| CONFIG-CAS-O04 | CAS-05, CAS-06 | Notification normalization and existing specialized writers cannot roll back a successful model update; unknown fields and the frozen disabled-notification format survive. |
| CONFIG-CAS-O05 | CAS-04, CAS-06, CAS-08 | Account scope changes preserve existing ordering/clearing; reopening reads the last committed model/provider and stale hashes cannot restore the previous account's values. |
| CONFIG-CAS-O06 | CAS-08 | The real `ProductionInferenceSettings` adapter consumes a successful provider setting and continues consuming it after a conflicting write and after reopening. Existing model normalization tests remain applicable. |
| CONFIG-CAS-O07 | CAS-02, CAS-07, CAS-10 | A timezone callback commits B before A's feature-override phase. Real `HostExperimentsExtension` receives A then B, ends on B=false, and B's provider/receipt match disk while A's receipt stays unchanged. |
| CONFIG-CAS-O08 | CAS-07, CAS-10 | A blocked timezone callback permits concurrent specialized commits; their notifications wait and preserve A/B/C commit order. A listener panic after a reentrant commit still allows remaining listeners/batches and a later writer to notify. |

Replace the old test-only copy of the Gateway hash-check closure with direct
calls to the production owner. Keep existing model, account, MCP, timezone,
notifications and permission contracts. The integrated Gateway must map the
typed owner conflict to 409 and use the owner snapshot for both config and
hash. A source wiring assertion may supplement these behavior tests but cannot
replace them.

The production-consumer oracle uses the actual `ProductionInferenceSettings`
adapter in the same unpackaged mode as its existing test. It does not run
Runner or inspect outbound provider requests, and does not establish model-ID
propagation in `ProductionAgentInferenceOwner` or provider payloads. Those
remaining production boundaries are not replaced with test doubles here. The
new IO rejection oracle uses a non-file source; it does not inject disk-full,
write or rename failures, or claim crash-durability beyond existing rename
behavior.

Completion requires integrated production wiring, focused Actions compile and
contract results, review of the changed owner/callers and requirement-to-oracle
evidence bound to H. Until then verification remains not-run and acceptance
pending; no project completion flag changes.

## Delivery and rollback

This is one owner-based batch with unchanged on-disk version and public
settings field meanings. There is no data migration. Revert code as one batch
if integration fails; reverting does not require rewriting settings data.
Existing Gateway conflict/error handling and diagnostics carry outcomes. No
secrets, full settings payloads or account identifiers are added to logs.

## References

- `../../../../docs/specs/openclaw-rust-equivalence-migration.md`, OCRM-R04,
  R05, R09, R13, R14, R16 and AC05, AC17, AC18, AC19.
- `../../management/tasks/P0-config-settings-authority.md`.
- `../../dossiers/config-settings-scoped.md` (historical source review only).
- Implementation base `source/host/app/src/main.rs::call_openclaw_config_set`,
  `source/host/src/gateway_server.rs::start_gateway_server`, and
  `source/host/src/extensions/settings/settings_service.rs`.
- `source/host/src/extensions/inference/production.rs` existing configuration
  consumers; no provider behavior is changed in this batch.
