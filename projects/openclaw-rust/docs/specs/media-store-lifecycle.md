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
