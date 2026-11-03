# Media store lifecycle cleanup

Status: implementation wired; verification not run
Project: OCRM-001  
Responsibility: `OCRM-RESP-MEDIA-STORE-LIFECYCLE-001`  
Owner boundary: existing Attachments extension → `media_store` owner

## Context and goal

The Rust media store implements the upstream retention rules, and the existing
Attachments extension now reaches that owner through one bounded periodic
worker. Manual Gateway `cleanOldMedia` dispatch remains available as an explicit
operator path; it is not the only producer. The worker must remain inside the
existing Attachments owner and must not become a second storage or scheduler
authority.

## Non-goals

- No second media store, scheduler, or cleanup owner.
- No deletion of the protected `playback-transcode` or `outgoing` subtrees by
  the general mtime sweep.
- No change to playback write-time size eviction or media retention policy.
- No local build, test, formatter, lint, package, provider, device, or release
  acceptance claim during the development freeze.

## Requirements

### MSL-01 — Single production owner

The existing Attachments extension owns exactly one stoppable lifecycle worker.
The worker may call only the existing `media_store` lifecycle functions and
must not create a parallel storage or scheduling abstraction.

### MSL-02 — Periodic producer

After the production Attachments extension starts, the worker performs one
bounded cleanup pass and repeats at a fixed interval. Shutdown signals the
worker and joins it before its extension dependencies are disposed.

### MSL-03 — Retention consumers

Each pass invokes `clean_old_media` (general TTL sweep),
`prune_playback_transcode_cache` (TTL plus 512 MiB LRU cap), and
`prune_outbound_media` (24-hour staging TTL). A failure is reported through
the existing Attachments diagnostic channel; later passes remain possible.

### MSL-04 — Safety invariants

The worker preserves the existing owner invariants: the general sweep excludes
`playback-transcode` and SQLite-managed `outgoing` media, playback eviction is
oldest-first when over budget, and all path resolution remains scoped by the
existing media-store containment checks.

## Current and target state

| Concern | Current | Target |
| --- | --- | --- |
| Manual cleanup | Gateway-only `cleanOldMedia` | Manual call remains plus worker producer |
| Periodic producer | `HostAttachmentsExtension::start_lifecycle_worker` starts `MediaStoreLifecycleWorker` | One Attachments-owned stoppable worker |
| Playback/outbound cleanup | `run_media_store_lifecycle_pass` invokes all three helpers | Invoked in every worker pass |
| Shutdown | `HostAttachmentsExtension::stop_lifecycle_worker` stops and joins the worker | Stop and join before Box/Attachments teardown |

## Architecture and failure handling

```text
Host startup
  → existing HostAttachmentsExtension
  → MediaStoreLifecycleWorker
  → media_store::{clean_old_media, prune_playback_transcode_cache,
                  prune_outbound_media}
  → scoped media files
```

The worker uses the existing media root defaults and no caller-supplied path.
Cleanup errors are diagnostic-only for that pass; they must not trigger a
second owner or an unbounded retry loop. Worker creation failure prevents the
Attachments extension from being reported as started.

## Verification and acceptance

- Static review proves one Attachments-owned producer and the complete
  producer → `media_store` owner → filesystem consumer path.
- Focused source oracles cover startup wiring, stop/join ownership, all three
  cleanup calls, error reporting, and preservation of the existing invariants.
- Exact-head Actions evidence is required before `verification=verified`.
- Packaged/runtime evidence is required before production wiring or acceptance
  is considered complete. Global completion flags remain `false`.

## Implementation strategy

1. Keep the stoppable worker beside the existing media-store owner.
2. Keep start and stop exclusively on `HostAttachmentsExtension`.
3. Keep the manual Gateway dispatch unchanged.
4. Preserve source-level wiring evidence in the scoped dossier and parity row;
   retain verification and acceptance as pending until their required evidence
   exists.

## Current implementation evidence — 2026-11-01

The source-level producer/owner/consumer chain is present:

`start_production_host_box_extensions` → `start_attachments_extension` →
`HostAttachmentsExtension::start_lifecycle_worker` →
`MediaStoreLifecycleWorker::start` → `run_media_store_lifecycle_pass` →
`media_store::{clean_old_media, prune_playback_transcode_cache,
prune_outbound_media}`.

`ProductionHostBoxExtensions::stop` calls
`HostAttachmentsExtension::stop_lifecycle_worker`; the worker unparks and joins
its owned thread before the Box extension is disposed. This proves source-level
wiring only. No local OCRM verification, packaged/runtime acceptance, or
production/device acceptance is claimed here.

## Spec compliance record

| Requirement | Status | Evidence / reason |
| --- | --- | --- |
| MSL-01 | passed (source wiring) | `HostAttachmentsExtension` owns the only worker; it delegates to existing `media_store` functions. |
| MSL-02 | passed (source wiring) | `MediaStoreLifecycleWorker` performs a bounded pass, parks for the fixed interval, and has an explicit stop/join path. |
| MSL-03 | passed (source wiring) | Each pass invokes `clean_old_media`, `prune_playback_transcode_cache`, and `prune_outbound_media`; failures are reported per operation. |
| MSL-04 | pending verification | Existing path and retention invariants remain owned by `media_store`; exact-head Actions and runtime evidence have not passed. |
| Definition of Done: static chain | passed | Producer, owner, consumer, and shutdown paths are identified above. |
| Definition of Done: focused oracles | pending | No current exact-head verification receipt is available. |
| Definition of Done: packaged/runtime acceptance | pending | No packaged/runtime or device acceptance evidence is available. |

## References

- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/media-store-read-scope-lifecycle-scoped.md`
- `source/host/src/extensions/attachments/media_store.rs`
- `source/host/src/extensions/attachments/extension.rs`
- `source/host/app/src/main.rs`

## Explicit recursive cleanup — MSL-RECURSIVE-01

### Current defect and fixed source

At target `84120837330f88d9a7b21e9a5bc75daaa728de82`, the production
`AttachmentsService::dispatch_gateway("cleanOldMedia")` passes the optional
recursive flag to `media_store::clean_old_media`. Nested saves such as
`cache/session/images` are already supported, but the cleanup owner visits
only root files and files in immediate subdirectories. An explicit recursive
call can consequently return success while expired deeper files remain on
every repeated call. Repair this existing consumer; retain its owner and API.

Source authority is fixed U `1440bb1771372334bea23d7642bb52deb7db1845`:
`src/media/store.ts` blob `d4342f8f5a4de1834346f131db70d2c6b6e80426`,
`src/media/store.test.ts` blob `490817f902a0ef9b8832cd1249fa6559c3ede492`,
and `src/media/store.cleanup.test.ts` blob
`e872a1de80427849f9d65679844ee75dede10929`. Historical dossier identities
remain historical and are not relabeled as these fixed-U reads.

The necessary mature dependency boundary is the declared
`@openclaw/fs-safe@0.27.0`: fixed-U package blob
`6989c0618742d3c2b93cc336cbe1d9c3c4f0a657` and lock blob
`83b4975eb96942ec007a2eb9847ccfe98d380bda`. Its official tag resolves to
`67dee5ef4c03655a2aa98893539475ad041cb61b`; pruning source blob
`d2c790ca760a953a9129862666b898159ea32aee` and removal sources
`c5e93f7ffe1d6b4a6b8bc653ec1d452b4029a5b6` /
`78bda7cfa3c6833f2ce677a7354dc5ae262ffbfe` establish depth, no-follow,
identity, last-observation and best-effort removal contracts. Official-tag
source review alone does not establish npm archive SRI identity or full
recursive dependency/legal closure.

Subsequent source acquisition read the actual registry archive
`https://registry.npmjs.org/@openclaw/fs-safe/-/fs-safe-0.27.0.tgz`:
1,101,331 bytes, SHA-256
`02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`.
Its computed SHA-512 SRI equals both the fixed-U lock and registry integrity:
`sha512-v9RWszhsnswr7kAg/6ZM1fUxtyY1PItj7uBarnQ2UdeTPgDdUZFS5hXIGUdLtNvSty/JEPJvjE3MWqbKbx9dng==`.
All 711 archive file identities were captured; the actual distributed prune
and removal consumers and MIT license were read. This is acquisition and
contract-review evidence, not execution, full archive reading, optional native
package closure or final legal acceptance. No archive code was executed.

The distributed dependency captures its clock per prune invocation; fixed-U
`cleanOldMedia` makes separate root/subdirectory invocations. MSL-R02's single
timestamp for the whole target pass is an explicit safety strengthening.
Default upstream removal can use guarded JavaScript fallback; native removal
is selected when native mode is required. This batch does not describe the
upstream cleanup as an atomic transaction or claim that native custody removes
every filesystem race.

Fixed-U `src/media/store.ts:298–307` also calls generated-HTML provenance
pruning. That neighboring scope already belongs to
`OCRM-RESP-MEDIA-GENERATED-HTML-PROVENANCE-001` and
`host_read_media_policy::{validate_provenance_row,filter_stale_provenance_rows}`;
its ledger records `BL-MEDIA-HTML-PROVENANCE-CHAT-WIRING-001`, not-wired,
not-run and pending. Recursive file cleanup does not close that responsibility,
prove its current error disposition, or supply its missing database consumer.

### Behavioral and safety requirements

- **MSL-R01 — Three depth modes.** Omitted `recursive` retains root plus
  immediate-subdirectory cleanup; `false` cleans root files only; `true`
  visits all eligible descendants. The periodic worker continues to pass
  default options and must not become fully recursive. Preserve API fields,
  the single Attachments worker, and the manual service projection.
- **MSL-R02 — Retention and ownership.** Capture one time per pass and remove
  only regular files whose last known mtime is strictly older than the TTL.
  Future/unknown times survive. Recheck expiration before deletion. Exclude
  the top-level `playback-transcode` and SQLite-managed `outgoing` trees before
  descending. Protected resources, playback quota logic, outbound policy,
  committed agent assets and Session fences remain owned by their existing
  consumers. Nested ordinary directories with those names are not additional
  top-level stores.
- **MSL-R03 — Confined traversal.** Keep operational names as `OsString` /
  `PathBuf`, including non-UTF-8 names. Do not follow symlink/reparse entries
  at any depth. Retain the selected root and ancestor directory identities;
  refuse an observed root/ancestor replacement and never adopt its successor.
  Deletion must operate through the retained parent or equivalent platform
  custody, using already available filesystem primitives. A canonical path
  prefix alone is insufficient. Explicitly document remaining platform or
  filesystem race limits; do not promote separate MSRS capability/publication
  acceptance from these cleanup controls.
- **MSL-R04 — Honest postorder pruning.** When `pruneEmptyDirs` is true,
  remove only actually empty descendants after processing their children,
  using nonrecursive removal. Never remove the media root. A surviving live
  file, symlink, special entry, failed unlink or uncertain directory read does
  not prove emptiness. Default and root-only modes retain their depth limits.
- **MSL-R05 — Failure and recovery.** Missing roots/entries retain the existing
  Fabushi no-op policy; upstream can create a missing store root. Keep failed
  leaf and nested-directory removals best effort; do not falsely infer that
  every requested file was deleted from an overall successful pass. For a
  requested empty-directory removal immediately below the media root, retain
  upstream's explicit refusal for errors other than missing, still-nonempty or
  already-existing entries. In particular, permission denial must reach the
  existing service/worker error path, with earlier file deletions left as
  honest partial progress. Fixed-U `store.ts:236–246` and
  `store.test.ts:408–443` establish this distinction from the dependency's
  best-effort nested removal.
  Retain MSL-R04's successful emptiness and retained-branch admission before
  attempting removal. Unlike upstream's unconditional top-level `rmdir`
  probe, a known populated or uncertain target branch is not probed for delete
  permission. This preserves the existing Fabushi empty-directory admission
  policy and is an explicit error-observation difference, not evidence that
  the complete upstream failure contract has passed compatibility acceptance.
  Root admission/enumeration or detected root identity errors remain explicit
  errors; uncertain child subtrees are retained. No global transaction,
  journal, retry queue or new scheduler is introduced. Interruption can leave
  partial progress; a later ordinary call safely processes the remainder.

### Implementation and verification contract

Extend the owner-private cleanup mechanics in `media_store.rs` and the
existing `openclaw_media_store_contract` target. Keep the service and worker
call sites unchanged. Inspect existing no-follow/native-handle precedents
before adding a primitive; do not route cleanup through the unrelated
local-exec JSON request surface or introduce another permission owner.

The primary real-filesystem oracle is a three-mode matrix using actual
`save_media_buffer` calls under root, `inbound`, `cache/session/images`,
`cache/session/docs` and a fully expiring branch. Use deterministic old and
future mtimes, without sleeps. The old implementation must fail the explicit
deep-deletion postcondition. Preserve live nested siblings, protected deep
trees and the root; prove optional empty-directory postorder cleanup.

Required negatives cover static symlink/reparse and leaf links, raw non-UTF-8
paths where supported, future and refreshed mtime, a real or controlled
failed unlink, and deterministic ancestor substitution before mutation. A
controlled seam must exercise the actual production traversal/deletion
decision; it cannot replace that owner with a mock. Extend the existing
service-dispatch case to save nested media and invoke explicit recursive
cleanup, using its existing serial fixture scope. Preserve all old tests.

Freeze the complete candidate and obtain independent general/security and
reuse/structure/performance reviews. Execute only the affected owner and
real service contracts in GitHub Actions on the published exact H, binding
actual positive names, commands, exits, input/log hashes and artifacts.
Platform-specific controls require real platform execution; skipped Windows
or macOS controls are not passes. Full MSRS read-authority/publication,
unknown filesystem effects, package, performance/soak and final OCRM
acceptance remain open. No unit/global verified flag is advanced by source
review, a local fixture or a partial green result.

Windows identity admission must not silently use a truncated identifier on a
filesystem that requires 128-bit file IDs. Compare the full volume/file identity
from the retained handle and each reobservation; if that identity cannot be
obtained, fail closed at root admission or retain the uncertain child. Existing
`same-file` handles may retain custody, but their 64-bit Windows equality alone
cannot prove the MSL-R03 successor-refusal contract. A controlled equality test
must distinguish IDs with equal low halves and different high halves; real
filesystem coverage remains a separate platform acceptance requirement.

MSL-R1 review correction: a successful Windows query does not by itself prove
a usable identity. Reject all-zero and all-ones 128-bit IDs at the production
admission helper, while admitting a valid ID whose low half is zero and high
half is nonzero. Keep the full volume/ID equality and existing failure paths.
The Windows private control must exercise that same admission helper.
Primary contract: [MS-FSCC 2.1.10][MSL-WINDOWS-FILE-ID] and the
[`FILE_ID_INFORMATION` structure][MSL-WINDOWS-FILE-ID-INFO]. These controls do
not substitute for native execution on the affected filesystem/provider.

The inherited traversal negative must construct its symlink target outside
the actual admitted base. Its old `base/outside` fixture was still contained;
repair only that fixture, preserve escape refusal and all other assertions,
and retain the complete integration target and strict positive-name selection.

MSL recovery review correction: protected root-store names must also cover
ASCII case variants on macOS, as they already do on Windows. APFS preserves
the creation spelling and is case-insensitive by default: a store created as
`Outgoing` can be the same store later addressed as `outgoing`. Refuse these
top-level variants before descent. This is deliberately conservative on a
case-sensitive macOS volume; Linux retains its existing case-sensitive name
policy. Ordinary nested lookalike directories remain eligible for cleanup.
The real-filesystem control creates the variant spelling first, saves expired
media through both variant and canonical names, and proves both retained
alongside successful deletion/pruning in an independent ordinary branch and
nested lookalike directories.

APFS accepts only valid UTF-8 names for creation. Keep the invalid-byte-name
control on non-Apple Unix, including the Linux Actions lane, and separately
execute a valid Unicode-name cleanup plus FIFO-preservation control on every
Unix lane, including macOS. This separates filesystem capabilities without a
runtime skip or a swallowed setup error. The capture helper requires the
exact applicable positive names on each platform. Primary platform contract:
[Apple APFS filename compatibility][MSL-APFS-FILENAMES].

[MSL-WINDOWS-FILE-ID]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/98860416-1caf-4c80-a9ab-8d61e1ccf5a5
[MSL-WINDOWS-FILE-ID-INFO]: https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-file_id_information
[MSL-APFS-FILENAMES]: https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/APFS_Guide/FAQ/FAQ.html

The affected Actions selection is the complete existing
`openclaw_media_store_contract` integration target and the owner-private
`extensions::attachments::media_store::cleanup::tests::` group. Reuse the
existing macOS and Linux Host caches/lanes, and add a Windows selection inside
the existing Runtime workflow using its existing Host cache convention. Bind
one strict receipt per platform to H/tree/U, exact commands, toolchain, all
required positive names, zero failed/ignored tests, complete raw logs and input
hashes before and after execution. Windows symlink controls must execute or
report a concrete non-passing capability failure; they cannot silently skip.
The shared, owner-scoped capture script must qualify its strict result oracle
with missing, duplicate, unexpected, failed, ignored, zero-test, partial-log,
nonzero-command and capture-error controls before accepting either real command.
Those synthetic oracle controls are explicitly distinct from product execution.
Record raw working-copy hashes and their Git-normalized blob identities so a
Windows checkout's line endings cannot silently break exact-H input binding.


### Windows substitution-fixture correction (MSL-WINDOWS-FIX-01)

The exact-H `8cf6143501b8c69b051d98df05f3ed523b364268` Windows
Actions owner group failed two fixtures before substitution:
`cleanup_refuses_replaced_ancestor` and `cleanup_refuses_replaced_root`
received native error 5 (`PermissionDenied`) while renaming a directory
whose descendants were already open. Preserve this failed result as
`TEST_FLAW`; the six other owner cases and all sixteen integration cases
are narrower positive evidence and do not turn this platform gate green.

The replacement oracle must actually rename the admitted original and
create the expired successor. On Windows, inject that mutation immediately
after admission of the directory being replaced, before opening any of
its descendants. The production traversal must then execute its ordinary
identity recheck: a changed root errors; a changed child retains its branch
and allows an unrelated branch to progress. Both original and successor
bytes must survive. The ancestor control still covers a directory successor
and a real directory symlink successor. Unix retains its existing mutation
at the leaf removal boundary, where ancestor rename is supported.

Use one owner-private traversal implementation with a no-op directory
admission callback in ordinary production. Keep the existing removal hook
and all its reobservation/error semantics. Invoke the extra private hook
after a root or child is admitted, before the next ordinary `current` check.
Do not release or weaken retained handles, change share flags, substitute
mock traversal, catch setup denial as success, skip a required case, or
remove any existing preservation/refusal assertion. A callback is solely a
deterministic observation/mutation seam, with no new product API or owner.

The Windows rename constraint is the documented FileRenameInformation
open-descendant rule in [MS-FSA §2.1.5.15.12][MSL-WINDOWS-RENAME] and
[§2.1.4.2][MSL-WINDOWS-OPEN-DESCENDANTS]. Attributing the observed error to
specific retained descendant handles is a source-backed inference; the
failed Actions log contains no native handle trace. Execute the unchanged
strict eight-owner/sixteen-consumer Windows selection on a new exact H.
Existing Linux and macOS selections remain applicable to the shared loop;
no prior-H green is transferred to this correction.

[MSL-WINDOWS-RENAME]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fsa/87f86c9b-6c2a-4803-84b7-131a74a434fa
[MSL-WINDOWS-OPEN-DESCENDANTS]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fsa/133840e4-778e-44ca-9b41-da2323615075
