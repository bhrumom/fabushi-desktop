# Plugin materialization attempt custody

Contract: `OCRM-PLUGIN-MATERIALIZE-CUSTODY-v1`  
Finding: `PS-MATERIALIZE-CUSTODY-01`  
Status: implementation authored; independent review and exact-H Actions pending  
Implementation base: `0e845ce1679e0cdf95346adc54f5d58bf06464e8`  
Discovery base: `071aeb6af1b223cceee9d81b5f7df3bad484d5a4`  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`

## Problem and actual production flow

The shipping Host composes `CoordinatorPluginSkillsLoader` and the existing
`SandPluginSkillsService`. Its lifecycle request reaches the existing Electron
`createHostPluginSkillsLoader`, then `loadFromMarketplaceSource` and
`DefaultPluginCacheManager`. At the discovery/implementation bases the SDK
creates the shared final cache directory before installation, calls
`installPlugin` in that directory, and
recursively removes it when installation or completion-marker writing fails.

Coordinator `control_command` waits 15 seconds. `call_control_command` retires
only its local waiter on timeout; it does not cancel Electron work. Each
lifecycle event has its own Coordinator worker, and the Electron control server
dispatches asynchronous executors independently. Thus an old materialization A
can remain active after its Host sync fails. A subsequent legitimate refresh B
can finish the same plugin/version and commit its snapshot before A fails. A's
unconditional removal then deletes B's actual files and completion marker.
The real workflow reader subsequently loses the committed skill. A plugin with
zero skills can likewise lose its command and manifest files.

This is distinct from `PS-N1`, already repaired by
`PS-CACHE-RETENTION-01`: Rust retains actual successful install paths when it
prunes after a valid complete commit. That repair and its controls are retained.
The Rust publication and successor locks do not order a late SDK failure's
filesystem deletion after its remote waiter has retired.

## Ownership and scope

`SandPluginSkillsService` remains the only product lifecycle/admission owner:
canonical listing, authentication denials, uninstall custody, publisher facts,
fallback, snapshot publication, positive release and committed pruning remain
in Rust. Session and `FileWorkflowStore` remain the actual skill consumers.

The existing SDK cache manager owns only materialization I/O: allocate the
attempt's temporary directory, install its files, publish complete files and
clean only the attempt's own temporary output. A temporary directory is not
an installed-state index, grant, permission registry or lifecycle generation.
No new service, scheduler, cancellation protocol, dependency or cache schema is
introduced. The loader continues returning the actual final `installPath`.

Allowed paths are exactly:

- This specification.
- `source/packages/cursor-plugins/cursor-marketplace.ts`.
- `source/packages/cursor-plugins/loader.ts`.
- `source/packages/cursor-plugins/loader.contract.test.ts`.
- `source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts`.

The integrator owns shared workflow/receipt selection, task/ledger/RTM changes,
independent review, publication and exact-H Actions. No local or htch-runtime
executable validation is allowed.

## Requirements

- **PM-01 — owned attempt.** A cache miss installs into a unique directory
  allocated with `mkdtemp` by the existing cache manager. The real backend gets
  this directory rather than the final cache directory. Staging is a sibling
  of the configured cache root, outside the subtree traversed by Rust's
  two-level installed-plugin pruner. It is created on the parent filesystem;
  publication requires an atomic same-filesystem rename. An unsupported
  cross-device/symlink mount topology fails explicitly through the existing
  per-entry error path; never fall back to copying partially visible files.
- **PM-02 — complete publication.** After installation succeeds, write the
  reserved `.cache-complete` marker in the owned staging directory, then
  publish the complete nonempty directory by rename to the existing deterministic
  cache path. No successful result or final marker may describe partially
  installed files. A pre-existing marker in staged plugin content is a conflict,
  not authority to follow a symlink or overwrite an unknown marker.
- **PM-03 — collision behavior.** Inspect an existing final target without
  following a target-directory or completion-marker symlink. A real directory
  with a regular completion marker is a completed cache entry under the
  unchanged cache-key contract and may be reused. A non-directory, incomplete
  directory, nonregular marker or unreadable target fails without removing or
  changing it. Check before installation and again immediately before rename.
  If a competing SDK publisher wins the rename race, reuse its target only
  after observing the same complete-directory condition; otherwise report
  failure. No existing final directory is recursively removed or merged by this
  operation.
- **PM-04 — cleanup ownership.** On installation, marker or publication failure,
  and when reusing another completed publisher's target, remove only this
  invocation's owned staging directory. A successful rename transfers that
  directory to the final path and ends staging ownership; do not attempt a
  later removal using the old staging name. Cleanup failure remains an error,
  not evidence that the temporary output disappeared. If the primary operation
  already failed, preserve its original message and cause while also reporting
  cleanup failure and unknown residual state. Expose only a controlled I/O code
  in neutral cleanup text; retain the full cleanup error in the aggregate, not
  in the message parsed by existing auth classifiers. A failed cleanup must
  neither mask a canonical authorization denial nor introduce one through its
  message or path. Preserve completed final
  bytes, old versions and unrelated temporary/file paths.
- **PM-05 — loader and lifecycle compatibility.** The SDK loader calls the
  new cache-manager materialization operation and then reads the returned final
  path with its existing parser. Retain canonical `pluginDbId`, per-entry error
  reporting, unavailable-versus-empty behavior and ordinary cache hits. Explicit
  `pruneOldVersions: false` still suppresses both SDK prune sites; omission and
  true keep their current behavior. Host still supplies false. Rust retains
  its complete-pass/denial/publication gates, zero-skill retention, successor
  protection, pending-uninstall preservation and authoritative-empty retirement.
  Existing standalone extraction, cache-completion, version enumeration and
  explicit-removal interfaces keep their purposes. Exact-source caller search
  found no production `extractToCache` caller; changing that separate public
  operation merely to reuse the new helper is outside this repair.
- **PM-06 — cache I/O is not an authorization signal.** Errors owned by the
  new materialization operation expose only fixed phase/state text and a
  controlled I/O code. This includes initial/prepublication/winner inspection,
  target conflicts, staging-parent creation/allocation, exclusive completion
  marker creation, final-parent creation, directory publication and cleanup.
  Paths and native diagnostic text remain in the error's cause or structured
  details, never in the message consumed by existing authorization classifiers.
  A supported data root containing `permission denied` must not create an
  authorization block or revoke an otherwise eligible prior-version fallback.
  The real `install(stagingDir)` callback's error remains unchanged; genuine
  remote authorization denials must still reach Host/Rust, including when
  owned-staging cleanup also fails. No public Host/Rust error protocol or
  classifier changes are part of this correction (`PM-R1-G1`).
- **PM-07 — primary failure classification.** The existing SDK classifier uses
  the primary `cause` carried by a cleanup `AggregateError`, not the combined
  diagnostic message. Follow nested aggregate primary causes without looping
  on a cyclic cause; ordinary neutral cache-I/O wrappers stay opaque. Explicit
  `undefined` or non-Error primary failures remain primary values. Classify
  native `EACCES`, `EPERM` and `EROFS` code fields, and the existing textual
  forms, as install failures. A cleanup code such as `ETIMEDOUT` or `EACCES`
  cannot change a genuine clone denial into timeout/install or an ordinary
  failure into a clone denial. Keep the complete public diagnostic, original
  primary object/cause and raw cleanup error for observation. Preserve the
  existing Host clone-category gate and Rust typed admission policy.

The integrated base `54b32b96ff827f4fee56fed494c866ca8c8b53ec` also carries
the Host's clone-category guard from `cd3430db43ca22b66ab0ee8b401c0aa3f9f2da44`.
Preserve that guard and its local-permission negative. The canonical-ID and
cleanup-failure denial controls supply an explicit Git clone transport error,
which the SDK categorizes as the primary `clone` failure; a bare permission
phrase is not that producer contract. Reuse `MarketplaceLoadFailure` at the
Host consumer so its `errorType` access follows the actual SDK type. Preserve
the incoming late-failure cleanup test from `370c6680b6b8c072615875d72be67ca0e45ebe45`.
The inline staging introduced at `94edb3025eb2a41381a97ed0833c7544217d15ca`
is consolidated into this cache adapter, retaining its attempt-isolation
behavior and adding the collision, pruning-boundary and cleanup requirements
above; there is only one materialization implementation.

The later incoming SDK filesystem-permission and Host remote-clone controls
remain part of the same load path. Preserve each named incoming test once,
including the restored EACCES/EPERM/EROFS test wrappers. The failed-publication
control creates its conflicting final directory during the real installer callback,
so it reaches owned-staging cleanup; its oracle checks the controlled
incomplete-target refusal and actual version child, preserved target bytes,
and disappearance of the exact stage.

### Atomicity limit

The cooperating SDK publishers create final entries only by renaming complete,
nonempty directories. Directory rename makes their completion visible as one
filesystem operation; a later publisher cannot merge into or overwrite the
nonempty completed winner. The target inspections also reject incomplete and
non-directory collisions already present or created while installation runs.

The standard Node directory-rename interface does not provide a portable
no-replace compare-and-swap primitive. This contract therefore does not claim
protection from an arbitrary non-cooperating filesystem writer inserting an
empty directory after the final target check but before rename, or from an
external actor replacing a parent/symlink or deleting completed files. Adding
a persistent lock/index or moving product policy into Electron is not an
authorized workaround for that broader problem.

## Interface and implementation

Add to `DefaultPluginCacheManager`:

```ts
materializeToCache(
  args: PluginCacheKey,
  install: (stagingDir: string) => Promise<void>,
): Promise<string>
```

It returns the existing final path after its own complete publication or a
verified complete winner. A private inspection helper distinguishes missing,
complete, incomplete and non-directory cache entries. The existing `isCached`
uses that helper while preserving its boolean fail-soft surface; the
materialization operation propagates conflicts and I/O failures.

Cache-owned I/O failures use a private neutral-message wrapper with the native
error retained as cause; conflict diagnostics retain their target as structured
details. The wrapper does not enclose the backend installation callback. Reuse
the same narrow I/O-code extraction for cleanup diagnostics, so no native error
message or data-root path can acquire authorization meaning.

Create the staging directory under the lexical absolute cache-root parent;
create only the required final parent directories after staging is complete.
The renamed directory already contains the completion marker. Track whether
the attempt still owns staging so a successful rename never schedules deletion
of a newly reused temporary name. The SDK loader replaces its direct mkdir/install/
marker/rm block with the single materialization call.

## Verification and acceptance

Author tests against actual `loadFromMarketplaceSource`,
`DefaultPluginCacheManager` and `createHostPluginSkillsLoader`; control only
remote listing/install I/O and explicitly scoped cleanup I/O faults. Use
explicit bounded barriers, not sleeps or a copied installation/reconciliation
algorithm. The primary defect must be detectable in actual bytes and cache
behavior, not just call counters.

Required oracles:

1. **PM-O01 late failed attempt:** A enters installation and waits; B for the
   same key/version actually completes; inspect B's returned final file bytes
   and sentinel; release A with a real per-entry install failure; B remains
   complete and byte-identical. A fresh third load hits those bytes without
   reinstalling. Cover both the actual SDK loader and the actual Host loader,
   including a zero-skill Host plugin with real command/manifest files.
2. **PM-O02 completed winner:** hold A, complete B, then let A finish
   successfully with distinguishable attempt bytes. Both calls resolve to the
   completed B directory; A neither replaces nor mixes B's files. Its temporary
   directory disappears. A subsequent cache hit preserves the winner.
3. **PM-O03 collisions:** a pre-existing incomplete directory, file target and
   invalid marker remain byte-identical and trigger no installation. Actual
   target-directory and marker symlinks are rejected without changing either
   the link or the referenced directory/marker bytes. Also
   create an incomplete target during a held/controlled install; publication
   fails, preserves that target and cleans only this attempt's staging.
4. **PM-O04 isolated failure:** a single failed install removes only its own
   partial staging, leaves no newly complete final cache entry, and preserves
   a seeded prior version and unrelated sentinel output. Preserve canonical
   failure ID and existing `pruneOldVersions: false` assertions.
5. **PM-O05 publication and marker errors:** make the final parent unusable or
   provide a staged reserved-marker collision. The pass reports per-entry
   failure with no claimed successful plugin or fresh final cache entry;
   owned staging is cleaned and foreign bytes remain unchanged.
6. **PM-O06 cleanup failure:** cause installation to fail with a real canonical
   authorization denial, then make owned-staging cleanup fail through a
   controlled I/O fault inside the fixture's exclusive parent. The actual Host
   must still return the same canonical `authBlocked` identity, preserve the
   primary denial in `loadError`, and explicitly expose unsuccessful cleanup
   with unknown residue. A paired ordinary-install-error case deliberately uses
   an auth-like fixture directory name in the real cleanup error's path; it
   must retain `authBlocked: []`. Assert that protected residue actually remains
   and no successful install path or final completion is reported.
7. **PM-O07 cache-owned failure classification:** under an auth-like data-root
   path, run the actual Host loader against an initial incomplete target, an
   incomplete target appearing during installation and an exclusively-created
   staged marker conflict. All three must report a per-entry failure with the
   exact canonical ID, an empty install path and `authBlocked: []`. Preserve
   protected collision bytes and the seeded old version; publish no new final
   completion. Retain PM-O06's genuine install-denial/cleanup-failure control.
8. **PM-O08 primary category during cleanup faults:** inject `ETIMEDOUT` and
   `EACCES` only for removal of the exact stage allocated by the real cache
   materializer. All other filesystem calls remain native. The actual SDK
   failure must retain the primary clone/ordinary category and original
   primary and cleanup objects; the actual Host must retain canonical denial
   for the clone case and no denial for the ordinary case. Inspect surviving
   staged bytes, no completion marker/final success, and unchanged old-version
   bytes. Restore the I/O binding before cleanup and a fresh successful load.
   These are controlled fault oracles, not claims of live filesystem-timeout
   or marketplace compatibility. Retain the actual ENOTDIR residue oracle.
9. Preserve all existing default/false/true SDK pruning tests and the real
   Host loader's listing, identity/currentness, canonical-denial and zero-skill
   path controls. Retain the existing Rust lifecycle/cache/workflow tests and
   Coordinator `timeout_retires_only_owned_waiter_while_remote_effect_continues`
   contract; they establish adjacent boundaries but do not replace PM-O01.

The existing Node contract selectors compile these two test files with shipping
Electron options and dependency resolution. The integrator selects/records the
new named positives on the final H and obtains independent general and
permission/lifetime/oracle review. No parent-H result or test-source addition
is execution evidence. Approval requires the new actual oracles and applicable
existing controls in GitHub Actions on the same candidate H.

## Failure, recovery, rollout and remaining work

No permission cache migration is required. Existing complete version directories
are reused; ambiguous/incomplete destinations fail and preserve their bytes.
Interrupted or crashed attempts can leave uniquely named staging siblings.
This batch does not discover, delete or claim recovery of those residues, and
does not automatically remove incomplete legacy final directories. The ordinary
load failure metadata remains the observability surface. Rollback reintroduces
the old failed-attempt deletion risk and is not lifecycle acceptance.

This repair does not revoke an already-admitted old success, cancel remote
materialization, fence all post-response account changes, repair lossy cache-key
identity collisions, add artifact signatures/digests, alter version GC or
complete filesystem-alias/cross-process/packaged-platform acceptance. A future
authoritative pass/ticket or revocation contract belongs to the existing Rust
publication/uninstall-observation owner and its current relay, not a new
Electron lifecycle or permission truth.

## Provenance

Authority: root `AGENTS.md`, OCRM source of truth and main §17;
`plugin-skills-snapshot-admission.md`; the current MCP/runtime parity contract;
the existing Plugins/Marketplace owner inventory; and exact target production
reads at the bases above. The four existing code/test paths are unchanged
between the discovery and implementation bases and had no uncommitted content
when this worker acquired them.

Reuse fixed-U `dossiers/snapshot-skills-scoped.md` with its recorded read limits.
This target I/O repair adds no upstream product requirement, relabels no source
reading, and changes no inventory/ledger/RTM or global completion flags. Tests
are authored only until the integrator records exact-H execution.
