# Plugin Skills committed snapshot admission

Contract: `OCRM-PLUGIN-SNAPSHOT-ADMISSION-v1`  
Status: implementation in progress; exact-H evidence and ecosystem acceptance pending  
Discovery base: `3aa679586d76e446f76a1c94700ff7abab623d42`  
Current integration base: `febef8ba077fd667ce2c06f4b0cdd3edd1570cde`  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`

## Problem, flow and ownership

The existing Electron SDK edge can return an unavailable marketplace listing
as successful empty data. The Rust Plugin Skills owner then replaces its cache
and prunes materialized plugins. Successful listings also lose enabled IDs
when the SDK cannot materialize a version or inline content. Cached skills can
survive a permission denial because fallback and workflow consumers ignore
the denial list, and a later transient failure clears that list. Dispose can
race the final cache rename after the present early disposed check.

Keep the existing effective-listing relay, `SandPluginSkillsService`, cache
file and `FileWorkflowStore`. Electron materializes SDK results and preserves
source identity; Rust owns snapshot reconciliation, publication, eligibility,
lifecycle and cleanup. No second permission truth, registry, scheduler or
runtime is introduced. Existing install/startup/refresh callers keep using
the service; workflow list/get/update and their prompt/run consumers share
the same committed-snapshot admission helper.

## Requirements

- PS-01: a failed or malformed authoritative effective listing returns the
  existing loader error. It must not publish an empty cache or prune files.
  A successful, valid empty listing is authoritative and retires prior skills.
  Capture enabled canonical database IDs from the raw effective response
  before SDK filtering/version resolution. Enabled entries require valid
  IDs; unknown listing shape or malformed enabled identity is an error.
  IDs are exact nonempty opaque strings without surrounding whitespace,
  positive safe-integer numbers, or positive bigints converted exactly to
  strings (bigints are not limited by Number's safe-integer range). Never
  substitute a display name or marketplace slug. Raw entries require boolean
  `isEnabled`; disabled entries need no plugin ID.
- PS-02: every SDK per-entry failure preserves its canonical `pluginDbId`
  when supplied. Host auth blocks use only that ID. Rust refuses malformed
  canonical loaded snapshots, including empty listed/block IDs and blocks
  outside the authoritative listed set; it does not infer missing identities.
  Duplicate listed/materialized identities are ambiguous and rejected. Before
  writing, validate the constructed cache with the same strict parser used by
  consumers. A generated unsafe skill ID or relative file path is an error
  preserving the old snapshot, not successful publication of an unreadable
  cache. This batch does not silently change existing skill-ID encoding.
  An SDK auth failure without a valid canonical ID makes the Host loader fail
  as a whole; the legacy standalone Rust projection cannot guess an identity
  through its unchanged Vec-returning interface and excludes such failures.
- PS-03: reconcile authoritative listed IDs L, fresh successfully materialized
  IDs S (including a successful plugin with zero skills), current denials A,
  and prior canonical denials B. Current denial wins over conflicting success.
  Retain each B while still in L and not freshly successful; a fresh successful
  load without A clears B. Absent/disabled IDs and their blocks retire only
  after a valid authoritative listing. Preserve old eligible records only
  while listed, not freshly successful and not denied in the new snapshot.
  Fresh success replaces all prior records for that plugin, including with an
  empty set. Do not create an executable record for an unlisted plugin.
- PS-04: one internal parsed snapshot supplies eligibility to service current/
  sync results and workflow list/get/plugin update. Current-format denials
  exclude exactly the matching canonical ID. Preserve public cache structs
  and RPC shapes; raw cache access remains available for diagnostics and
  existing publication/reference metadata consumers, not execution admission.
- PS-05: add private `cacheFormatVersion: 1` on corrected cache writes. An
  unversioned cache with missing/empty `authBlocked` keeps legacy unblocked
  behavior. An unversioned cache with a valid nonempty denial list is ambiguous:
  preserve raw diagnostics but admit no old plugin record or fallback until a
  canonical refresh. Numeric-looking legacy slugs cannot prove database ID
  identity. Present malformed denial data, malformed/unknown version, or a
  versioned cache missing `authBlocked` is invalid and admits no records or
  fallback; preserve its bytes until an authoritative replacement. Never
  silently convert invalid denials into an empty list.
- PS-06: stage serialization and temporary-file writes outside service state
  locks. Final rename and dispose serialize through a service publication
  mutex, followed by a brief disposed check under the existing state mutex.
  If dispose wins, discard only the owned staged file, do not rename, prune or
  report successful change, and return retained eligible current records.
  If publication wins, the committed pass may finish reporting after dispose.
  Reporting and external callbacks remain outside state/publication locks to
  allow reentry. Pure filesystem pruning reacquires the old service publication
  lock, briefly checks its one-shot replacement claim under state, drops state,
  and retains publication through the prune. If replacement claimed first,
  skip the old prune; if prune entered first, successor claim/construction waits.
  Plain dispose without a successor still permits this already-committed prune.
  This narrow ordering supports same-owner handoff without allowing stale
  cleanup to delete a successor's materialization. No state/admission lock or
  external callback spans cleanup I/O. Staging/rename error preserves the prior
  cache and reports failure through the existing Result; no success event or
  pruning. Failed strict readback after rename retains live refusals, reports
  failure and does not prune; it does not promise restoration of old bytes.
- PS-07: Host materialization disables SDK pruning before Rust commits. Honor
  explicit `pruneOldVersions: false` at both SDK pruning sites while preserving
  other callers' default behavior. Rust prunes only after a committed valid
  listing with no unresolved listed materialization. A partially failed pass
  preserves old files even when it publishes valid denial/fallback state.
  An authoritative empty successful pass may retire files. No new version GC.
- PS-08: changed reporting compares admitted skill identities and canonical
  denial state, so a permission change cannot be reported unchanged merely
  because skill ID/version did not change. Preserve sync coalescing, panic
  recovery, recursive-sync refusal and valid existing permission controls.
- PS-IDENTITY-01: after a successful authoritative refresh, publisher edit
  eligibility comes only from that pass's current identity. The Electron SDK
  edge must not memoize a successful `getMe` identity across refreshes. Capture
  the existing account owner's credential for each pass and bind effective
  listing, identity lookup and SDK Git authentication to that immutable
  credential. Before returning the materialized result, compare the existing
  account scope and backend context with the current account; changed, missing
  or unreadable context rejects the pass. Same-account token renewal may pass
  that scope check only with nonempty canonical account subjects in both
  credentials; otherwise only an identical credential proves continuity.
  This is a credential/currentness bridge, not a new account
  identity or permission owner, and tokens are never logged or persisted here.
  A failed or invalid `getMe` leaves the identity explicitly unknown while a
  valid authoritative listing may still commit its ordinary eligible skills.
  Rust writes that pass's identity, including unknown, without filling it from
  an older snapshot. Unknown identity grants no publisher edit permission.
  The existing sync `changed` report includes a changed current-user identity,
  even when admitted skill IDs, versions and denials are otherwise identical.
  Preserve the existing `FileWorkflowStore` publisher checks and successful
  current-publisher editing, listing/denial reconciliation and file retention.
- PS-IDENTITY-02: the account owner's non-refreshing credential read must
  reject a read superseded by logout/login even if the later account is active
  again. The retained default `SandCursorAuthService.peekAccessToken` captures
  its existing auth-operation epoch and credential-mutation tail, awaits already
  scheduled mutations before reading secrets, and returns unknown if either
  identity changes before those reads complete. A new login can start before a
  read while leaving A active until B is published, so the operation epoch alone
  is insufficient. Reuse the existing account owner; add no public
  port or second generation store. The current Fabushi product binding's
  `FabushiAuthService.peekAccessToken` retains its existing epoch and issuer
  checks, with the pending-login rule below. A causal held-secret-read test uses the real Sand service,
  public logout/login and Host loader: the stale materialized A pass is refused,
  then a fresh B pass succeeds. Also hold the read after B login starts but
  before B credentials publish; completing that login cannot authorize old A.
  This correction does not change `exportTokens`,
  credential refresh policy or the post-response lifecycle limits below.
- PS-IDENTITY-03: the actual Fabushi product account owner returns unknown from
  `peekAccessToken` while its existing login controller is active, checking both
  before and after the asynchronous `currentSession` read. Login's operation
  epoch begins before new credentials publish, so a peek started during that
  login must not treat the previously stored account as current. Keep the
  existing epoch/issuer checks, ordinary stable-account reads and successful
  fresh-account reads after login settles. A held browser-poll control uses the
  real service and public login path through the Host loader: a materialized A
  pass is refused during B login and a fresh B pass succeeds afterward. Inject
  only network/secret-storage I/O for this local oracle, not the identity or
  authorization decision. Other `currentSession` consumers, Host credential-file
  publication races and full account revocation remain separate open work.
- PS-CACHE-RETENTION-01: a complete successful refresh must retain the actual
  install directory of every freshly materialized, canonically listed plugin,
  including a plugin with zero skills. The SDK's existing cache owner sanitizes
  marketplace and plugin directory names; raw listing keys such as `review.tools`
  do not identify the resulting `review_tools` directory. Reuse each successful
  plugin's returned `installPath` alongside committed skill paths when the
  existing Rust owner prunes uninstalled directories. Do not duplicate SDK name
  normalization, add a cache index, or treat a directory name as a permission ID.
  Only the existing canonical successful-materialization set may supply these
  extra retention paths. Keep the complete-pass/no-denial/publication gates;
  partial, denied, failed and disposed passes retain their current safeguards.
  Unlisted sibling directories must still be removed after a complete commit,
  and a subsequent authoritative empty commit must retire the formerly listed
  zero-skill plugin. This is the separate production follow-up to deferred
  `PS-N1`, not a change to publisher identity or install/update authorization.

### PS-IDENTITY-01 scope and existing account bridge

The successful-refresh publisher correction starts from
`f2f2ba00d66b3db79f9cd46690b3ce3737fe296b`. The production MCP adapter retains
one Host loader closure across `resetMcpManager`; account transition cleanup
only resets the manager. The SDK edge therefore obtains identity on every
pass, using the existing `getValidAccessToken`/`peekAccessToken` and
`accountCacheScope` contract. The Rust service remains the cache/publisher
admission owner and Session workflow commands remain its real edit consumers.
No cache schema, RPC field, account lifecycle or second permission registry
is added. The returned identity/currentness check cannot fence an account
transition after the response has left the Electron edge. Immediate account
switch revocation, failed-publication revocation, old active passes after that
boundary, and original-process restart admission remain separate open work.
Reopening the newly committed cache proves only this successful-refresh rule.

## Shared internal interface

The integrator owns `plugin_skills_cache.rs` and workflow composition serially.
Expose internally `read_plugin_skills_snapshot(cache_dir) -> Option<PluginSkillsSnapshot>`
with `index: PluginSkillsCache`, `eligible_skills()` and
`canonical_auth_blocked()`. `eligible_skills` is also the only source for prior
fallback. `stage_plugin_skills_cache(cache_dir, index, now)` returns an owned
staged file with `publish(self) -> Result<(), String>`; existing public write
wraps these operations. A private unique temporary filename avoids deleting
another writer's temporary file. This does not claim cross-process CAS.

The Rust lifecycle worker owns only `plugin_skills.rs` and its sync tests.
The SDK worker owns only Host loader/SDK loader and their focused contract
tests. The integrator owns workflow/cache oracles, build declarations, shared
workflow, task and source records. Each works from this versioned contract,
returns its exact diff, and performs no local executable validation.

## Verification and acceptance

Use GitHub Actions only: shipping Host compilation, the existing plugin/cache/
workflow contract targets, focused library sync tests and Node SDK/Host-loader
contracts compiled with the existing shipping Electron esbuild options and
dependency resolution. This is not a whole-graph strict TypeScript typecheck.
Tests must call
the actual loader/service/workflow owners, not duplicate reconciliation.

Required controls: unavailable versus valid empty; raw listed-but-unmaterialized
inline IDs; canonical ID/slug collision; denial then transient failure then
fresh success (including zero skills); denial wins over success; absent ID
retirement; malformed cache variants and ambiguous legacy cache; denial blocks
workflow list/get/update without changing unrelated user/managed workflows;
stage/write failure preserves bytes/files; disposal between staging and rename
with deterministic channels; success callbacks outside locks; partial load
does not prune old materialization, complete empty may prune; SDK explicit
false disables both pruning sites while normal callers retain behavior.

PS-IDENTITY-01 controls use the same loader instance for consecutive A/B
identities, a failed/invalid current-user lookup after prior success, immutable
per-pass SDK credentials, account departure/change during a pass and
same-account token renewal. PS-IDENTITY-02 also holds the actual default account
service's asynchronous secret read across a completed logout/login transition;
the loader must refuse that pass without relying on a mocked account-generation
decision. These are local owner oracles, not real account-service acceptance.
A real Rust service writes fresh snapshots consumed
by `FileWorkflowStore`: an old publisher's edit must be refused with the actual
skill bytes unchanged, both before and after reopening the committed store;
unknown identity has the same refusal, and the current publisher can still
edit. Preserve the existing own/foreign and snapshot-denial controls. Exact-H
Actions and independent permission/currentness review remain required.

PS-CACHE-RETENTION-01 uses the actual Host/SDK loader to demonstrate that a
dotted marketplace/plugin name returns a sanitized install path and zero skills.
A real Rust service refresh must commit that zero-skill snapshot, retain the
materialized file bytes, remove an unlisted sibling, and retire the same install
directory after a later authoritative empty refresh. Keep the existing empty,
denial, partial-load and failed-publication controls. Reuse their existing Node
and Rust Actions selectors; no new workflow, persistence format or migration.
Rollback of this bounded pruning repair reintroduces the deletion risk and is
not lifecycle acceptance. Exact-H execution remains required before approval.

Freeze the complete candidate and perform independent general and permission/
lifecycle/oracle review. Record any finding decision before repairs. Acceptance
requires exact-H Actions and scoped source authority; real marketplace/accounts,
all MCP permissions, platform packages and full OCRM acceptance remain pending.

## Migration, rollback and explicit remaining work

The marker distinguishes corrected canonical block identities from legacy
ambiguous identifiers without rewriting existing bytes on read. A successful
authoritative refresh migrates the cache. Old binaries ignore the marker and
may ignore denial eligibility; rollback is not a permission-acceptance proof.
Preserve raw metadata and materialized files on failed refresh for recovery.

This batch governs **committed snapshot** admission. A newly observed remote
denial/uninstall whose cache publication fails does not yet synchronously
revoke an older committed eligible snapshot. Closing that live owner-to-
consumer revocation boundary, wiring unhandled manual status/sync RPC paths,
account-switch invalidation, UI lifecycle, MCP grant/revoke and process-restart
behavior remain follow-up production work. These are not scope exemptions or
accepted final behavior. No real service/permission compatibility claim is made.

## References

OCRM main Spec §17, existing module map M16/M17, FPM-001
`docs/specs/fabushi-plugins-marketplace-open-ecosystem.md`, existing MCP owner
inventory and plugin dossier. Source review at fixed U covers plugin-skill
enablement/loading, root contributions, configuration activation and uninstall
cleanup; exact blob identities and read limits stay in the existing dossier.
No historical reading fact, inventory identity or global flag is promoted.

## Live admission amendment — OCRM-PLUGIN-LIVE-ADMISSION-v1

Status: implementation in progress under the approved v2 contract and exact-path
authorization. The prior shared gate has passed; this batch still requires its
own independent review and exact-H Actions evidence. Implementation base:
`febef8ba077fd667ce2c06f4b0cdd3edd1570cde`; fixed U:
`1440bb1771372334bea23d7642bb52deb7db1845`. Design authoring base was
`f2f2ba00d66b3db79f9cd46690b3ce3737fe296b`; that old source is not the
implementation input. The incoming PS-IDENTITY-01..03 contract, its controls
and its currentUserId reader/changed-report behavior are preserved in full.
The live refusal reader adds a negative admission condition; it does not
replace publisher identity or infer a refusal from unknown publisher identity.
Parent-H results do not verify this live-admission implementation.

### Problem and user behavior

At this implementation base, a valid newly observed canonical refusal is local to
`SandPluginSkillsService::run_pass`. A stage or rename error preserves the
old disk snapshot, but also loses that refusal. `current()` and existing or
new workflow stores can then serve the old eligible plugin skill. A later
partial materialization can republish the same old fallback. This is a live
admission implementation gap, not a reason to weaken PS-06's preservation of
old cache bytes and materialized files.

There is also a concrete production root mismatch: shipping `start_mcp`
receives `default_unified_app_data_dir()`, whereas the SDK materializer and
Session workflow stores use the configured SAND root. Desktop bootstrap
intentionally permits different `FABUSHI_APP_DATA` and `SAND_DATA_ROOT` paths.
This batch must bind MCP publication and its actual consumers to the existing
Session SAND root; it must not merge the two global data directories.

After the existing Plugin Skills owner accepts a fully validated canonical
refusal, subsequent skill-admission reads exclude that plugin even if cache
publication fails. A fresh successful materialization may lift the refusal
only after its corresponding valid snapshot commits. Existing data and
unrelated user, managed and allowed plugin workflows remain available under
their existing rules.

### Requirements

- **PS-LIVE-01 — One owner and observation boundary.** Keep canonical refusal
  state private to `SandPluginSkillsService`. Import the validated current
  `auth_blocked` facts before any fallible cache serialization, staging or
  rename. Reject malformed `LoadedPlugins` as before; do not manufacture a
  canonical refusal from a display name, slug, directory, unclassified error,
  unavailable listing, or merely absent plugin. The import has one ordering
  point against the existing publication/dispose boundary. A pass already
  retired at that boundary cannot add a new fact.
- **PS-LIVE-02 — Failure persistence.** A successfully imported refusal
  survives load, stage and publication failures, unresolved partial passes,
  coalesced returns and disposal. Readers retained after disposal still read
  the same refusal state. Publication failures still return the existing
  error, preserve old cache/file bytes, emit no publication-success event and
  perform no prune. Raw cache/status/reference metadata remains diagnostic
  data with its existing shape, not live admission authority.
- **PS-LIVE-03 — Positive release.** A current refusal wins over simultaneous
  fresh success. Otherwise only a fresh successful materialization of the
  exact canonical plugin ID, including a success with zero skills, may lift
  its live refusal, and only after the corresponding staged snapshot is
  renamed and read back as valid under the publication boundary. A failed
  fresh-success publication cannot reopen the old snapshot. Retained live
  refusals participate in later unresolved reconciliation; they are not lost
  merely because the old disk cache lacks them. PS-03's committed absent-ID
  retirement remains unchanged. This amendment does not infer a new live
  uninstall/revoke fact from absence alone.
- **PS-LIVE-04 — Read-only sharing.** Expose a read-only admission reader from
  the same owner. Every actual admission combines committed snapshot
  eligibility with that reader. Consumers hold the reader, not a copied
  denial table. `FileWorkflowStore` checks it on every plugin list/get/update
  admission, including parse-cache hits, before reading or modifying skill
  content. `current()` and every successful `sync` return, including a
  completed coalesced pass, apply the same check. A poisoned admission or
  forwarding-reader lock fails closed for plugin admission; it does not
  silently become an empty denial set. Unrelated workflow families retain
  their current behavior.
- **PS-LIVE-05 — Session and Runner binding.** `ProductionSessionWorkers`
  owns only a shared forwarding reader slot; it stores no refusal policy.
  Every store it opens, including an instance created before the first MCP
  binding, uses that same slot. Session list/get/update, run-now, reference
  expansion and the next provider skill prompt therefore consume the same
  admission. `HostRunnerComposition::compose_turn_state_surfaces` passes the
  actual Session-opened workflow store directly to
  `SandAgentState::new_with_workflow_store`; it must not first construct and
  discard a default store, whose constructor performs legacy migration.
  Runner workflow/reference/review reads cannot fall back to an independent
  store. Existing `new` and agent-ID validation ordering remain; mutations
  remain routed through Session.
- **PS-LIVE-06 — Actual root identity.** Shipping startup passes the parent
  of the actual `session_workers.agents_root()` to `start_mcp`. Missing parent
  is an explicit Host startup error through its existing fail/log/return
  path, without a fallback to app-data or the current directory. MCP cache,
  prune, publication/reference and legacy-sweep paths use this SAND root,
  matching SDK materialization and Session plugin reads. When a Session is
  registered, `start_mcp` checks its supplied root against that actual
  Session's root before retiring or constructing a service, starting polling
  or changing a binding. Mismatch is a diagnostic error preserving the old
  owner. Use the existing configured `Path` component semantics only: no
  new `canonicalize`, filesystem creation, symlink traversal, case folding,
  or guessed alias equivalence. This repairs an inconsistent producer/
  consumer configuration; it does not establish arbitrary Session root
  switching or filesystem alias equivalence.
- **PS-LIVE-07 — Same-root stop/start.** Preserve legitimate MCP stop/start
  within the same Host and Session lifetime. Host composition retains the
  latest service `Arc` after `stop_mcp` takes its runtime. Before a replacement
  can start, a crate-internal one-shot successor operation permanently fences
  the predecessor through publication then state, and inherits its same
  private admission `Arc`. A predecessor cannot fork two successor writers.
  Reader binding to that same `Arc` is idempotent. Old loader/stage work and
  delayed old-runtime stop cannot add refusals, rename a snapshot, clear the
  successor's refusals, or dispose its local lifecycle state. Old committed
  cleanup orders against handoff under old publication: briefly read the
  one-shot replacement claim, drop state, then either skip superseded prune
  or retain publication until the pure filesystem prune ends. Ordinary
  dispose without a replacement claim keeps valid committed cleanup;
  reporting remains outside publication and may finish after handoff. The
  original PS-06 clause is narrowed consistently above.
- **PS-LIVE-08 — Unbound startup and different roots.** Preserve MCP startup
  before Session registration. Once a Session is registered it must bind to
  the already-current service reader before that registration succeeds; both
  startup orders must be serialized so neither can miss the other. If there
  has never been a registered Session consumer in this Host, a stopped MCP
  runtime may still restart with a different configured root: permanently
  fence the old service and create a separate reader for the new root.
  Never carry a denial-release operation from one root into another root's
  retained reader. A later Session must match the current root. This is one
  current Host binding, not a map of roots or a new permission registry.
  A registered Session cannot be silently unbound or rebound to an unrelated
  service by this operation; broader Session/account/root migration remains
  open.
- **PS-LIVE-09 — Failure-safe handoff.** Check root and known lifecycle
  preconditions before retiring the old service. Once a successor has been
  constructed, retain its `Arc` in Host before any later fallible Session
  binding or startup action; a failed startup cannot leave Host pointing at
  an already-consumed predecessor and make the next attempt fork or lose the
  refusal state. Do not start polling after a binding error. Failed successor
  retirement/one-shot claim does not create another writer. Pending old
  callers wake under the existing dispose rules and return live-filtered
  retained records; old runtime stop remains idempotent. Preserve existing
  active-pass completion, panic recovery, recursive-sync refusal and real
  polling shutdown behavior.
- **PS-LIVE-10 — Lock and callback boundaries.** Consumer reads never acquire
  `ServiceState` or the publication lock. A forwarding slot clones its
  delegate and releases its lock before invoking the query. Service writers
  take publication, briefly inspect state, release state, then update
  admission; no state/admission lock covers I/O or a callback. Positive
  release is inside the final publication transaction, after strict valid
  readback; staging remains outside those locks. Correspondence to this pass
  relies on the existing single production writer, that P interval, and a
  successor being constructed only after predecessor P is claimed. It is not
  a cross-process CAS or protection from arbitrary external file writers.
  Missing/malformed readback does not release a refusal. The old pure prune
  may hold P as PS-06 specifies; it holds neither state nor admission.
  State-to-pass-result ordering stays unchanged. Preserve the existing explicit `drop(state)` before disposed
  `current()` returns; those releases already exist and are not a current
  deadlock defect. Host binding synchronization must have no reverse edge
  into Session/MCP lifecycle slots and must release before callbacks, poll
  startup, joins, reporting or cleanup. Do not introduce a public setter,
  writer generation service, second scheduler or permission truth.

### Minimum shared interface

The frozen read surface is `PluginSkillsAdmissionQuery::is_denied(&self,
plugin_id: &str) -> bool`, shared as `Arc<dyn PluginSkillsAdmissionQuery>`.
`SandPluginSkillsService::admission_reader()` returns its stable read-only
handle. Its mutable implementation remains private. Session binds this handle
through an idempotent `bind_plugin_skills_admission` operation and gives its
forwarding reader to `FileWorkflowStore::with_plugin_skills_admission`.
`SandAgentState::new_with_workflow_store(sand_root: impl Into<PathBuf>,
agent_id: impl Into<String>, workflows: FileWorkflowStore) -> Result<Self,
String>` directly accepts the Session-opened store used by production
composition. Existing `new` keeps its original default store behavior; both
constructors retain the original agent-ID validation before owned path/store
construction or default-store migration. These are admission/routing
interfaces, not grants or a new disk format.

The restricted service handoff is one-shot and crate-internal. Same-root
successor construction preserves the exact reader. Unbound different-root
replacement only fences the old service and creates a new owner-local reader;
it never shares writable refusal state across the two roots. Host keeps only
the current service and the actual registered Session reference needed to
compose them. The integrator owns the exact Host binding lock implementation
and the minimal shipping-main hunk; no worker modifies a shared main file.

### Required verification and acceptance

Run only in GitHub Actions after freezing the integrated exact H. The tests
must call real service/store/Session/composition entrypoints and use channels
or existing production-used control callbacks for ordering; no sleep oracle
or duplicate test-only reconciliation loop is acceptable.

1. Seed allowed canonical plugin 9 and unrelated plugin 10. Introduce a
   validated refusal for 9, force a real stage error and separately a real
   rename error, and prove old cache/files stay byte-identical while service,
   old/warm and new workflow stores refuse 9 and retain the controls.
2. Retain the existing no-new-refusal publication-failure control unchanged:
   the old allowed snapshot remains available. Invalid loader data must not
   fabricate a new refusal. Preserve strict/legacy cache and canonical ID/
   slug controls, user/managed workflows and plugin edit permissions.
3. Exercise refusal plus publication failure, listing failure, unresolved
   partial success, fresh success whose publication fails, then fresh success
   whose valid snapshot commits. Only the final step lifts the live refusal;
   zero-skill success and current-denial-wins remain independently covered.
4. Prove a genuine stable parse-cache hit with controlled old file metadata
   or an actual cache clock, then verify the live reader still denies it.
   Merely reading a newly written file twice is insufficient because of the
   existing racy-mtime non-cache window.
5. Use actual `ProductionSessionWorkers`, `SandAgentSessionStore`, run-now,
   reference expansion and `HostRunnerComposition` state surfaces. Verify
   refused content/path is absent after failure, and unaffected user/managed/
   allowed plugin content remains. Feed the actual Session store projection
   into the production `append_budgeted_workflows_system_prompt` and inspect
   its real `ProviderMessage` output: the refused skill path/description must
   be absent and the unaffected controls present. Run-now/reference tests
   separately inspect recipe bodies. Add a bounded shipping-source assertion
   that the real turn calls this builder with that Session store; it is a
   wiring supplement, not a substitute for the prompt-output assertion.
6. Construct distinct app-data and SAND roots as existing desktop bootstrap
   does. Prove the selected MCP owner writes/reads/prunes/publishes/sweeps
   against the Session/SDK root and never treats the other root as that
   producer's cache. A mismatched registered Session root fails before old
   service retirement, new polling or reader binding. Cover the missing
   parent error without guessing a fallback or physical alias.
7. Cover same-root stop/start after a failed-refusal publication, an old
   loader returning after handoff, an old staged success attempting late
   rename/positive release, delayed old-runtime stop, one-shot fork refusal,
   retry after binding/start failure, and both Session/MCP startup orders.
   Readers opened before binding or before replacement remain effective.
   Pause an old real pass after valid commit and before cleanup, let its
   successor materialize and publish a new directory, then release old cleanup:
   the successor directory and actual consumer content must survive while
   the old report still completes. An isolated direct invocation of the real
   old pruner with the old keys/paths must delete an equivalent new directory,
   proving the fixture detects the defect. In a separate plain-dispose control
   with no successor, old committed cleanup must still delete an obsolete
   sentinel. Also order a real prune before handoff and prove the successor
   is constructed only after that protected prune ends. Use private bounded
   barriers around the production-used cleanup operation, not a copied loop;
   prove state/admission are not held during its filesystem work.
8. Without any registered Session consumer, replace root A with root B and
   prove a success on B cannot clear a refusal held by an old A reader. A
   later Session for B binds; a Session for A is rejected without changing
   current B. No root registry or union of denials is introduced.
9. Keep dispose/coalescing, late-return, panic, recursive callback and real
   poll-join controls. Prove any newly added Host binding lock is released
   before external startup/cleanup callbacks, and that consumer reads cannot
   reenter `ServiceState` through the reader path.

Acceptance requires complete candidate review, the affected shipping Host
compile and selected actual owner contracts on the same H, plus the required
scoped source authority/coverage. Existing parent-H green results, this spec,
source-only assertions or mocks do not establish this batch as verified.

### Migration, observability and remaining scope

There is no cache schema change, deletion of old cache bytes, or persisted
permission grant. A corrected successful sync writes the existing canonical
snapshot in the actual Session/SDK root. Any cache under the previous wrong
app-data path is left intact as historical data; it must not be merged or
promoted into the SAND root's authority on this batch's behalf. Current sync
errors and Host startup diagnostics remain the observable failure surfaces.
Rollback to a binary without live admission/root wiring loses this protection
and is not permission acceptance.

This batch does not durably preserve a refusal through process loss when disk
publication failed, complete post-response account principal invalidation, add live uninstall
facts, revoke already captured prompts/records or in-flight tool effects,
complete MCP OAuth/per-tool grants, repair manual sync/status dispatch, or
prove UI invalidation, direct file-access sandboxing or real ecosystem
compatibility. Legitimate in-process same-root stop/start is included above;
it must not be dismissed as an excluded process-restart case. Full OCRM
source/implementation/platform/ecosystem gates remain unchanged.

### Source provenance

Reuse the unchanged fixed-U review in `dossiers/snapshot-skills-scoped.md`
and its existing responsibility mapping; do not relabel that work as new
full-source reading. This amendment is motivated by exact target-source
owner/consumer/lifecycle reads at the authoring base, including the real
startup root producer and existing desktop bootstrap root tests. Main,
large Session/Host files and SDK consumers were read in the cited relevant
scope, not promoted to complete-file reading. No ledger/RTM/global completion
flag is changed by this scoped implementation.

### Retained Session mutation oracle repair — 2026-11-02

Decision **PS-LIVE-ORACLE-01 — FIX / TEST_FLAW**, before editing the test.
At H `f80b4baca00a999659a8ebfe8ebe92e7d27b6018`, Runtime `41434802065`,
job `125241848734`, the retained Host Runner state-writer test fails after
archiving a Session and expecting a global workflow-definition edit to be
refused. The actual callback still delegates to `SandAgentSessionStore` and
the existing per-agent filesystem mutation owner. That owner checks canonical
identity and deletion custody; the definition-edit contract at
`session-lifecycle-delete-fence.md` explicitly specifies deletion-race
serialization. Workflow definitions use the existing global shared library.
The fixture constructs retained state surfaces, not an active admitted Runner
turn with a captured lifecycle generation. PS-LIVE-05 does not add archive
policy for shared workflow management.

Use the actual `begin_agent_delete`/`end_agent_delete` owner to exercise the
promised fence. Observe its durable deletion bit through a retained DB handle,
then clear only the existing process-local fast marker so the negative control
must still be enforced by durable owner state. Do not remove the agent directory.
Retain the exact failed-result and unchanged-file assertions while fenced, and
prove the same retained writer can update the actual workflow after custody is
cleared. Preserve all plugin denial, failed-publication, publisher, managed
workflow and live-reader assertions. Existing archive/restore and archived
turn/transcript controls remain unchanged in their own Session contracts.

Disposition **REJECT**: adding an archive rejection to the shared filesystem
guard or all workflow-definition APIs solely to satisfy this fixture would
introduce an unselected management policy and could interfere with lifecycle
CAS/restore. The earlier broad implementation-defect hypothesis was based on
the generic new-work requirement; the specific definition-owner contract and
actual shared-library consumer falsify that inference for this test. This
oracle repair does not decide a new archive-management policy or close stale
Runner-generation and full permission/restart responsibilities. Independent
frozen review and the corrected exact-H Actions gate remain required.

### Live refusal restoration reporting — PS-LIVE-REPORT-01

Decision `PS-LIVE-REPORT-01 — FIX / IMPL_BUG`, revalidated against the adopted
existing owner at `e70a72574c108f8b7704eb45e5109875a24eddb9` before this fix.
After an allowed snapshot is retained across a failed denial publication, a
fresh successful materialization can release the live refusal without changing
skill IDs, versions, publisher identity or the old disk denial list. The current
report compares only those disk facts and incorrectly sends `changed: false`
to the actual Host telemetry consumer, even though admitted skills return.

Preserve PS-08 by carrying the effective admission transition out of the existing
owner's successful publication interval. Releasing an existing live refusal
must make that pass's successful report changed, including same-version/current-
user restoration and valid zero-skill restoration. The next identical successful
refresh is unchanged. Do not infer a release before rename and strict readback,
create another permission set, mutate failed-pass reporting, or run callbacks
under publication/state/admission locks. All prior identity, denial, fallback,
cleanup, replacement and coalescing assertions remain mandatory.

Use the actual service, retained cache, stable reader and reporter with a real
failed staging/rename boundary; assert refusal remains live and no success is
reported on failure, the fresh equal-identity release reports one change, and a
following equal refresh reports no change. Include the zero-skill positive and
keep the prior unchanged-refresh and new-denial controls. Existing
`extensions::mcp::plugin_skills::sync_tests` is the Actions entrypoint. This is
truthful reporting of the existing permission transition; no UI refresh, durable
revocation, external effect or full lifecycle acceptance is claimed.

### Confirmed uninstall live admission — OCRM-PLUGIN-UNINSTALL-LIVE-v1

Decision `PS-UNINSTALL-LIVE-01 — FIX / IMPL_BUG`, before implementation,
at exact base `736b862a7646351e1e25558d9ba81879a5d606e4`; fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`. The actual Runner management
sink calls `McpHostService::uninstall_plugin`. After the existing backend
confirms `removed: true`, that service currently removes only legacy live
references and ignores a subsequent Skills sync failure. The existing
canonical cache and materialized skill bytes can therefore remain eligible
for later Session/Runner reads. Existing strict cache and live-reader controls
are correct consumers; the confirmed uninstall never reaches their owner.

This batch closes that in-process observation-to-new-read boundary. It does
not label an uninstall as an authentication failure and does not change the
cache schema or introduce a second permission registry.

- **PS-UNINSTALL-01:** add the required internal
  `PluginSkillsPort::observe_confirmed_uninstall(&self, plugin_id: &str)
  -> Result<(), String>` operation. The production adapter delegates to the
  same method on `SandPluginSkillsService`. After a confirmed `removed=true`,
  call it before any legacy cleanup or best-effort refresh. Invalid canonical
  identity or an unavailable owner is an explicit error, never silently
  accepted as a complete local revocation. `removed=false` and backend errors
  do not manufacture a confirmed fact. Preserve the backend's ordinary
  removed/reason result and all existing install/uninstall controls.
- **PS-UNINSTALL-02:** keep confirmed-uninstall state private to the existing
  admission owner, distinct from canonical authentication refusals. Its stable
  read-only reader denies the exact canonical ID, including warm cache reads.
  Keep unrelated plugins, user/managed workflows and publisher checks intact.
  Do not write a fabricated `authBlocked` entry, delete preserved materialized
  files, or infer identities from names, slugs, source URLs or directories.
- **PS-UNINSTALL-03:** capture the current owner-local uninstall observation
  identity before each actual loader invocation. Serialize a new confirmed
  fact with existing publication ordering. A pass that began before that fact
  cannot import/release it through an old success, publish its staged image,
  prune against that image or report successful publication. Reject it with an
  explicit stale-pass error and retain the old files plus the new live refusal.
  No callback or loader call runs under publication/state/admission locks.
  Reuse the existing coalescing and one-writer loop; add no scheduler.
  The ordering point is final publication: if that commit happened before
  the fact, its truthful publication report may finish later outside locks,
  as PS-06 already permits. Returned records still use current live admission.
  A prune not yet entered skips if the observation identity has changed; a
  prune already holding publication finishes before the fact can linearize.
  This `PS-UNINSTALL-ORDER-01` clarification preserves PS-LIVE-10 and does not
  authorize an old staged image to commit after a newer fact.
- **PS-UNINSTALL-04:** a later pass must obtain a fully valid authoritative
  listing and freshly materialize this exact ID before releasing its live
  uninstall refusal, only after its corresponding valid snapshot commits and
  strict readback succeeds. This permits a legitimate reinstall, including by
  another client and a successful plugin with zero skills. A listed unresolved
  fallback, load/stage/rename error, current authentication denial or stale pass
  cannot release it. Fresh success on another ID cannot release this one.
  Existing authentication-denial reconciliation is unchanged. The successful
  report includes this real admission transition even if disk identities match.
- **PS-UNINSTALL-05:** same-root successor construction keeps the exact stable
  reader and observation state. Predecessor load/stage work cannot overwrite or
  release it after retirement; a disposed/replaced service refuses a new local
  observation explicitly. Do not claim that a late remote action on a retired
  MCP instance is settled by this local operation. Preserve existing handoff,
  cleanup ordering, startup roots, close/poll and coalescing behavior.

The focused oracle must use the actual MCP service and Skills owner, actual
cache/files and Session-opened workflow reads; fake only backend/loader I/O.
After confirmed uninstall and failed refresh, retained and newly opened reads
must exclude only the target while its old cache/file bytes remain intact.
Exercise list/get/update and the existing real workflow prompt builder; retain
unrelated plugin/user controls. Use bounded channels at actual load and stage
boundaries to prove that pre-fact success cannot republish/release. Cover the
opposite commit-before-fact order, fresh post-fact success and zero skills,
publication failures, false/error backend results, malformed IDs, same-root
replacement and truthful changed reporting. Keep every prior valid assertion.
Use the existing `mcp_service_contract`, plugin/workflow/Session targets and
`extensions::mcp::plugin_skills::sync_tests` selection in GitHub Actions only;
freeze exact H, independently review the complete delta, then read actual logs.

The in-memory fact survives same-Host stop/start through the existing shared
reader. If the later authoritative cache publication fails, process loss still
loses this observation; a fresh process can read the old committed snapshot.
That durable restart boundary remains explicitly **blocked**, with no restart
acceptance claim. Closing it needs an existing-owner durable intent/publication
contract before external mutation, strict migration and unknown-outcome
reconciliation; disguising uninstall as `authBlocked` is rejected. Successful
ordinary authoritative cache retirement retains its existing restart behavior.
Already captured prompts, in-flight effects, stale remote-command completion,
all MCP grants/OAuth and real ecosystem/UI/platform acceptance also remain open.
Rollback loses this live protection. No OCRM source authority or global flag
is promoted by this target-source correction.

### Desktop MCP intents — OCRM-MCP-DESKTOP-INTENT-v1

At base `485c388b082f71d9e6fb87535e87240551d19dde`, explicit desktop plugin
removal calls the Electron manager directly. Settings synchronization can fail
before its empty best-effort `refreshMcp` call. The latter and the existing
renderer sync/status methods have no shipping Host handlers. Thus desktop
removal bypasses the Rust service's PS-UNINSTALL observation. Fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`. Implement this bounded wiring
after the shared compile/contract gate is repaired; this specification is not
execution evidence. Retain the preceding PS, PS-LIVE and PS-UNINSTALL contracts.

| Method | Input | Existing owner and result |
| --- | --- | --- |
| `uninstallMcpPlugin` (new native intent) | `{pluginId: string}` | MCP service uninstall; original `{state, removed, reason?}` desktop result |
| `refreshMcp` | `{}` | Await existing Skills sync; null acknowledgment or error |
| `syncPluginSkills` | `{}` | Existing service's synchronized records array |
| `getPluginSyncStatus` | `{}` | Existing service's status object |

- **PS-DESKTOP-01 — one mutation path.** Desktop IPC sends one uninstall intent
  through the existing Coordinator MainData leg. `UnifiedGatewayApi` delegates
  before its Host-lane fallback to its existing `McpHostService` Arc. The service
  uses the existing concrete backend/lifecycle relay and reverse
  `uninstallHostMcpPlugin` callback. That callback continues calling the existing
  manager, never the desktop intent. Preserve account/team checks; do not use
  the separate `feature.plugin.uninstall` package installer.
- **PS-DESKTOP-02 — exact identity.** The shared Rust service uninstall core,
  used by Runner and Desktop, validates the ID before catalog or mutation calls:
  a canonical, nonempty unpadded string with no control characters; numeric IDs
  cannot contain leading-zero aliases. Padding includes downstream ECMAScript
  trim whitespace, especially leading/trailing U+FEFF, which Rust's whitespace
  property does not include. Reject these aliases before catalog, mutation or
  observation; do not normalize them. Preserve exact opaque official IDs.
  The Desktop intent separately requires exactly `{pluginId: string}` and rejects
  caller-supplied outcome fields. Existing Runner tool-level catalog checks do
  not replace this public service boundary: the legacy manager passes numeric
  strings through `BigInt`, whereas Skills observations use the exact ID key.
  Reject aliases without remote I/O or local observation; do not normalize a
  different identity, duplicate the official catalog, or infer confirmation
  from names, URLs or remaining rows.
- **PS-DESKTOP-03 — confirmation before projection.** Preserve the backend's
  confirmation and original display state separately. Only its correlated
  boolean `removed=true` reaches the existing Skills observation, before any
  display-state error is returned, legacy cleanup, refresh or desktop settings
  synchronization. False, error, malformed confirmation and unknown outcome
  create no uninstall fact. Preserve state from the same response, including
  warnings and row policy fields; no extra list request or lossy reconstruction.
  Share uninstall orchestration with the Runner entrypoint and retain its
  existing removed/reason result. Keep the explicit unavailable/retired Skills
  owner error after a known confirmation. A Desktop display projection error
  returns after the observation and before cleanup/sync; the admission fence
  remains. Runner's existing removed/reason projection does not depend on the
  Desktop display shape and retains its current cleanup/sync follow-ups.
- **PS-DESKTOP-04 — bridge and refresh.** The bridge awaits the Rust result
  before its existing settings synchronization. A settings failure cannot skip
  the confirmed observation. Do not directly fall back to a manager, resend the
  intent to a replacement Host, or duplicate the service-owned refresh. The
  three existing refresh/sync/status routes consume the actual Skills owner;
  malformed arguments or a missing owner are errors. Completion-bearing
  `refreshMcp` is explicitly unsupported here, not silently treated as empty.
- **PS-DESKTOP-05 — existing ingress authority.** Coordinator accepts the new
  uninstall intent only on MainData and refuses Data/Control before forwarding.
  Renderer sync/status retain their Data route. Host HTTP retains its existing
  local/admin and device-token authority; its context does not prove MainData
  origin. Add uninstall/refresh/sync to the existing mutator scope predicate and
  all four methods to the operator-only predicate. Read-only operators may read
  status, cannot mutate; nodes cannot invoke these operator methods. Preserve
  default local operation and existing bearer behavior, with no new credential,
  native-origin header or permission registry. Register the new uninstall
  intent in its own native management group, preserving the frozen public
  inventory and existing Miniapp, Cloud Worker and Box groups. Do not put it
  in the Box-only group, whose paired-device denial remains unchanged.
- **PS-DESKTOP-06 — retained custody.** Reuse the existing one-attempt request,
  separate HTTP/SSE/Control lanes, relay UUID and close/registration boundary.
  Caller timeout does not cancel admitted remote work or authorize replay.
  Late reverse resolution may contact a successor but cannot settle its UUID-
  qualified waiter. No new retry loop, scheduler or disposal mechanism is added.
- **PS-DESKTOP-07 — internal completion authority.** Exact `29669c8b` source
  shows that paired read/write/node principals can obtain a pending lifecycle
  request ID through SSE and forge its completion. Restrict
  `mcp.resolveLifecycleRequest` and `host-mcp-lifecycle-request` events to the
  existing native control-plane authority calculation used by Box methods:
  configured shared secret or trusted loopback with no configured auth token,
  using the same role, scope, host/origin and bearer checks. Preserve the current
  Box behavior for an unknown bearer when no shared secret is configured;
  this does not grant recognized paired tokens native authority. All paired credentials, including
  `operator.write`, are denied for the resolver before delegation and cannot
  receive that event. Event filtering applies before channel/slimming/gzip
  projection; ordinary events remain available. UUIDs remain correlation,
  not authorization. Do not change the relay's one-settlement, timeout, close
  or successor rules or invent an extra credential/permission owner. This
  internal authority is separate from PS-DESKTOP-05 management intent access.
  The same boundary also covers the actual routed MCP execution pair
  `runner.resolveRoutedToolRequest` / `runner-tool-request`: audited source
  `812bf3a47b7ab16e49545446bd25f5e5ce8da27d` exposes the same known-UUID forgery path through `CoordinatorToolRelay`, whose
  definitive-result check validates shape, not the caller's authority. Both
  resolver/event pairs use the one existing native authority calculation;
  preserve their own correlation, execution-unknown, close and successor rules.
  The external Coordinator Data/Control ingress must likewise refuse both
  settlement methods before forwarding with its native Host credential. Reuse
  its existing MainData carrier gate; native reverse replies keep their current
  route. The adopted `dda1467e2480ad730a684402f616802d06b8b277` already guards
  lifecycle receipts at HTTP/SSE and Coordinator ingress. Preserve that work,
  share its authority with the Box predicate, and extend the same boundary to
  routed tool settlement without introducing a second permission owner.

The shared Rust dispatch surface is `McpHostService::dispatch_desktop_command
(&self, method: &str, args: &Value) -> Option<Result<Value, String>>`; unrelated
methods return `None`. The internal backend result may retain display-state
validation separately from confirmation so its error is propagated only after
the existing observation boundary. The cache and admission algorithms remain
with their current owners.

The Actions oracle must exercise the real service, concrete lifecycle backend/
relay, Skills cache and retained/new Session stores, with only remote/loader I/O
controlled. Confirmed uninstall followed by failed refresh or malformed display
state still excludes only the target from list/get/update, run/reference and the
actual prompt builder while preserving cache/files and unrelated workflows.
Retain false/error/unknown, canonical-ID, close/no-new-dispatch and successor
controls. Invoke the actual registered desktop IPC handler and production port
delegate: one intent, no manager fallback/retry, and settings failure only after
the Host result. Actual Gateway HTTP with DevicePairingOwner-issued tokens must
prove read/node negatives dispatch zero calls, status-read and local/admin
positives work, and headers cannot upgrade token authority. Select
`gateway_server_contract` alongside the existing MCP/Session/Coordinator and
Electron targets; freeze H and review complete wiring plus negative controls.
For PS-DESKTOP-07 use a real pending relay/backend request and its known real
ID: each denied credential must reach zero resolver delegates, leave the waiter
pending and deliver no result; a later authorized completion resolves it once
with its original value. Apply the same real-waiter test to
`CoordinatorToolRelay::request_tool_execution`, using a valid definitive MCP
result shape so an unknown ID or malformed result cannot explain rejection. For SSE subscribe before publishing and read through
an ordinary later marker, proving stream progress past both actual internal
request events; paired clients receive neither request ID and native clients
receive their unchanged payloads.
An empty capture or timeout does not establish event filtering. Keep native
secret/loopback positives and every existing Box authority assertion.

Allowed production paths are existing `mcp_service.rs`, `coordinator_relay.rs`,
`gateway_server.rs`, `host_gateway_api.rs`, shipping Host/Coordinator `main.rs`,
Coordinator `mcp_lifecycle_relay.rs`, Desktop `mcp-desktop.ts`, its `mcp-oauth.ts`
adapter and `shared/rpc/coordinator-main.ts`. Extend existing selected contract
tests, workflow and OCRM responsibility references; add no owner or cache schema.
Reuse the unchanged fixed-U review in the existing snapshot-skills and MCP
dossiers, without new full-reading or acceptance claims.

Process-loss durability after failed publication, post-response account changes,
server-row removal, OAuth completion, SSE readiness, already captured prompts,
unknown remote effects and real ecosystem/platform acceptance remain open.
Rollback reintroduces the desktop bypass; it is not revocation acceptance.
Current errors and sync status remain the observability surfaces. This bounded
delivery cannot promote OCRM global flags or replace full FPM/OCRM acceptance.
