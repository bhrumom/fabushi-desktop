# Media store read-scope authority

Status: specified; implementation not started; verification not run
Project: OCRM-001  
Responsibility: `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001`  
Owner boundary: existing Attachments `media_store` owner plus the existing
channel/session admission owners

## Context and goal

The locked upstream responsibility publishes media only while the caller's
read authority is valid. It verifies that the destination directory is the
directory selected by the authority, prevents a concurrent symlink or
directory replacement from changing the publication target, and retains the
opened file/descriptor lifecycle until the host consumer accepts the result.

The current Rust media store provides path-segment validation, best-effort
canonical containment, and temporary-file rename. Those checks are useful
scaffolding, but they do not implement a read-scope capability, directory
identity token, file-handle identity check, or host-consumer acceptance
boundary. This spec records the missing behavior so it is not represented as
implemented merely because `save_media_buffer` exists.

## Non-goals

- No second media store, transcript store, channel router, or session store.
- No synthetic channel/session producer or guessed provider thread identity.
- No weakening of the existing direct gateway media path while no read-scope
  consumer exists.
- No local build, test, formatter, validator, CI, package, provider, device,
  or release acceptance claim during the development freeze.

## Requirements

### MSRS-01 — Existing owner boundary

Read-scope publication remains owned by `media_store`; channel read authority
and session media receiver owners provide the caller/consumer boundary. The
implementation must not create a parallel capability or storage authority.

### MSRS-02 — Explicit authority

Every scoped publication must carry an active, non-revoked read-scope
capability or an explicitly documented direct-publication mode. An absent,
expired, revoked, or ambiguous scope fails closed.

### MSRS-03 — Directory identity

Before publication, the owner must bind the selected destination to a stable
directory identity and reject a symlink replacement, directory swap, or other
identity mismatch. `Path::canonicalize` alone is not sufficient evidence for
this invariant.

### MSRS-04 — File identity and lifecycle

The owner must publish through a verified file handle/descriptor whose identity
is checked against the selected directory and whose lifecycle remains valid
until the existing host consumer acknowledges acceptance. A returned string
path alone is not the required contract.

### MSRS-05 — Atomic and idempotent publication

Publication is atomic within the verified destination, rejects collisions or
unexpected replacement, and reports the original failure without claiming
consumer acceptance. Retry behavior must be owned by the existing caller and
must not duplicate media or scope authority.

### MSRS-06 — Honest integration boundary

The channel read-authority producer and session media receiver consumer must be
identified and wired before this responsibility can become `wiring=wired`.
Until then, the existing direct gateway save/read path remains a separate
unscoped path and this responsibility stays incomplete.

## Current and target state

| Concern | Current | Target |
| --- | --- | --- |
| Path containment | Segment checks and best-effort canonical containment | Stable scope-bound directory identity |
| Authority | No Rust read-scope token at the media-store boundary | Active/revocable typed capability |
| Publication | Temp file + rename, returns canonical path | Verified handle/descriptor + atomic publication |
| Consumer | Gateway and remote-fetch callers; no channel/session scope consumer | Channel authority → media owner → session/host acceptance |
| Status | `implementation=not-started`, `wiring=not-wired` | Implementation and consumer evidence separately proven |

## Architecture / data flow

```text
channel read authority
  → typed active scope
  → media_store destination identity + verified handle
  → atomic publication
  → session media receiver / host acceptance
  → release or revoke scope
```

The scope token is not inferred from a path, request JSON, provider name, or
session string. The existing session owner remains authoritative for session
admission; the media owner remains authoritative for media publication.

## Failure modes and edge cases

- Missing or revoked scope: reject before creating or publishing a file.
- Destination directory replaced or symlinked: reject and do not adopt the
  successor path.
- File handle identity mismatch or unexpected collision: reject closed.
- Consumer disconnect/timeout before acceptance: retain only the exact owned
  handle/custody needed by the existing owner, then release or clean up under
  that owner; do not broaden cleanup to a path prefix.
- Direct gateway publication without a scope: keep its explicitly documented
  direct mode; it must not be presented as read-scope equivalence.

## Verification and Definition of Done

- A source review identifies one real authority producer, the existing media
  owner, and one real session/host consumer.
- Focused negative contracts cover missing/revoked scope, directory swap,
  symlink replacement, file identity mismatch, collision, and consumer
  non-acceptance.
- The Rust contract preserves descriptor/handle identity through acceptance;
  a canonical path string is insufficient.
- Exact-head Actions evidence is required before `verification=verified`.
- Packaged/runtime/provider/device evidence is required before acceptance.
- Until all of the above exists: `implementation` and `wiring` remain
  incomplete, `verification=not-run`, `acceptance=pending`, and global flags
  remain false.

## References / provenance

- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/media-store-read-scope-lifecycle-scoped.md`
- `source/host/src/extensions/attachments/media_store.rs`
- `source/host/tests/openclaw_media_store_contract.rs`
- Locked upstream source records in the dossier for
  `src/media/store.read-scope.ts` and its focused tests.
