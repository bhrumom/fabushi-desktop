# Media store lifecycle cleanup

Status: implementation in progress; verification not run  
Project: OCRM-001  
Responsibility: `OCRM-RESP-MEDIA-STORE-LIFECYCLE-001`  
Owner boundary: existing Attachments extension → `media_store` owner

## Context and goal

The Rust media store already implements the upstream retention rules, but its
cleanup is only reachable through a manual Gateway `cleanOldMedia` call. A
production Host therefore does not periodically prune expired outbound staging
files, expired playback transcodes, or old unprotected media. The lifecycle
owner must be reached by one bounded worker in the existing Attachments
extension.

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
| Periodic producer | None | One Attachments-owned stoppable worker |
| Playback/outbound cleanup | Helpers exist, no daemon consumer | Invoked in every worker pass |
| Shutdown | No lifecycle handle | Stop and join before Box/Attachments teardown |

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

1. Add the stoppable worker beside the existing media-store owner.
2. Start and stop it through `HostAttachmentsExtension` only.
3. Keep the manual Gateway dispatch unchanged.
4. Update the scoped dossier and parity row to record the new source wiring;
   retain verification and acceptance as pending until their required evidence
   exists.

## References

- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/media-store-read-scope-lifecycle-scoped.md`
- `source/host/src/extensions/attachments/media_store.rs`
- `source/host/src/extensions/attachments/extension.rs`
- `source/host/app/src/main.rs`
