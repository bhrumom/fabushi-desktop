# Plugin Skills committed snapshot admission

Contract: `OCRM-PLUGIN-SNAPSHOT-ADMISSION-v1`  
Status: implementation in progress; exact-H evidence and ecosystem acceptance pending  
Discovery base: `3aa679586d76e446f76a1c94700ff7abab623d42`  
Current integration base: `7382badafe5c3a793835b7760bf3ab54e7fb54c9`  
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
