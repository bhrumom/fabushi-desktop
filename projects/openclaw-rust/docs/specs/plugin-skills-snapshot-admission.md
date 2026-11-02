# Plugin Skills committed snapshot admission

Contract: `OCRM-PLUGIN-SNAPSHOT-ADMISSION-v1`  
Status: implementation in progress; exact-H evidence and ecosystem acceptance pending  
Discovery base: `3aa679586d76e446f76a1c94700ff7abab623d42`  
Current integration base: `ecb6f6bd6f618975feca1a7b7b992255fb38fdb3`  
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
  If publication wins, the committed pass may finish reporting after dispose;
  callbacks and pruning remain outside state/publication locks to allow
  reentrant callbacks. Staging/publication error preserves the prior cache and
  reports failure through the existing Result; no success event or pruning.
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
