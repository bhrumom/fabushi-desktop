# Channel credential write admission — CCWA-v1

Status: implementation contract; no execution or acceptance claim.
Discovery subject: `f71bb36a8757b4acf2356fa34de9d73721ccfbda`.
Current integration base: `7eb81e20b7dc2eac5a71516a9a0c78795d252e95`.
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`.
Relevant production and test inputs are unchanged between these two subjects.

## Problem and decisions before implementation

`SandConnectorSecretStore::set_secret` currently merges the public permissive
`read` projection. That projection converts malformed JSON, a non-object root,
invalid UTF-8 and non-NotFound read failures to an empty object. A legitimate
credential submission can consequently overwrite existing fields whose state
was never read. Both native `connectChannel` and secure-input widgets use this
same existing writer through Session.

Decision **CCWA-01 — FIX / IMPL_BUG**: introduce a private checked read at this
writer's existing admission boundary. Keep the current store, paths, Session
per-Agent mutation lock and lifecycle admission. No new credential owner,
runtime, permission source, queue, schema or dependency is warranted.

Decision **CCWA-ORACLE-01 — SUPERSEDE / INTENTIONAL SAFETY CHANGE**: the test
`connector_secret_store_matches_frozen_validation_merge_and_remove_contract`
accurately describes the legacy overwrite of `"not-json"`; that successful
overwrite is the behavior being repaired, not a flaky test. Replace only that
expectation with rejection and exact byte preservation. Keep its permissive
inspection assertion, identifier/field validation, field merge, empty-secret
filtering, newline and removal assertions. Explicit removal followed by a new
write remains a successful recovery control. Do not delete valid assertions
or implement a raw-database/alternate-store fallback to obtain passing tests.

## Required behavior

| ID | Contract |
| --- | --- |
| CCWA-01 | `set_secret` validates identifiers and a nonempty field exactly as today, returning `Ok(false)` for those invalid inputs before any filesystem work. |
| CCWA-02 | Ordinary NotFound permits first creation. A valid JSON object, including an empty object, preserves every unrelated field and value while updating the requested field. Preserve input normalization, pretty JSON with a trailing newline, and the existing replacement mechanism. |
| CCWA-03 | Malformed/truncated/empty JSON, a non-object root, invalid UTF-8, and every non-NotFound read error return an error before directory creation, temporary write or rename. Never turn an uncertain read into absence with `exists()` or another permissive probe. Rejected bytes remain exactly unchanged, including through a reopened owner. |
| CCWA-04 | New read-admission error Display/Debug and diagnostics contain only fixed text or bounded error classes. Never include stored bytes, submitted values, parsed objects, or type-error messages that can echo a JSON scalar. Existing diagnostic fields may retain Agent identity and bounded error class. |
| CCWA-05 | Public `read` and `get_secret` retain their compatibility projections; list callers and storage identities do not change. Explicit removal still uses the existing owner and may be followed by legitimate creation. A failed normal submission never silently repairs or deletes unknown content. |
| CCWA-06 | Actual Session `store_connector_credential` propagates rejection before metadata mutation. Existing metadata remains byte-identical, and absent metadata remains absent. Another Agent/platform is unaffected. Same-owner and reopened admission remain consistent. |
| CCWA-07 | Actual Session Gateway `connectChannel` returns its existing internal error, with no stored/submitted secret text and no success response. The shipping timeline publication remains after error propagation. Native/paired HTTP permission admission remains the separate CGWA-v1 boundary. |
| CCWA-08 | Actual `WidgetResponses::submit_secret` returns its store error before channel-config callback, `secretProvided` update or acknowledgement. After explicit repair of the same path, the same persisted request may succeed once; repeated already-completed submission retains existing behavior. |
| CCWA-09 | For this store error, actual `TranscriptManager::submit_secret_with` returns `Ok(false)`, invokes no resume and, when production services are bound, emits the existing fixed redacted tray. Preserve the successful same-request repair path. Its earlier `switch_agent` lifecycle/open failures can still return `Err`; do not collapse them into this store-error result. |

## Production ownership, effects and recovery

The existing native HTTP path is Gateway authentication/authorization →
UnifiedGatewayApi → TranscriptManager Session dispatch →
`ProductionSessionWorkers::store_connector_credential` →
`SandConnectorSecretStore::set_secret` → channel metadata. The secure-input
path is TranscriptManager → WidgetResponses → the same Session/store writer →
channel-config callback → transcript flag → acknowledgement/resume.

The checked read must occur inside `set_secret`, so both consumers and any
other legitimate writer receive the same admission rule. Session already
holds its per-Agent filesystem boundary and checks lifecycle admission; reuse
those owners without a second lock or copy of the credential map. Reader-only
inspection compatibility is separate from permission to replace durable data.

Refusal creates no asynchronous operation, automatic retry or recovery work.
The user-visible secret request stays incomplete; a later explicit repair or
removal can unblock it. Do not invent a success acknowledgement from a failed
write. No persistent format conversion is needed and no stored file is rewritten
on startup. Rollback must not knowingly restore destructive unreadable-input
overwrite; there is no data migration to undo.

This repair does not make secret and metadata a two-file transaction. It does
not make the later callback/transcript update atomic with the secret write.
Cross-process exclusion, path replacement/symlink confinement, partial later
writes, crash durability, lost-response replay, real provider registration and
complete channel lifecycle remain open. These limitations cannot excuse a
failure of the checked read contract above.

## Implementation and causal verification

Production changes belong only in
`source/host/src/extensions/session/connector_secret_store.rs`. Extend the
existing `connector_secret_store_contract`, `session_gateway_contract`,
`widget_responses_full_contract` and `transcript_manager_contract` targets.
Root serializes this spec and existing workflow selectors/receipts. Keep the
existing real Session, Transcript, Memory, trays and filesystem fixtures; a
fake store callback is not proof of either production consumer.

Store controls cover malformed/truncated/empty input, JSON scalar/array/null,
invalid UTF-8, and a real non-NotFound I/O error. Include a readable-after-repair
positive and preserve unrelated nested/non-string values. A directory-only
negative is insufficient to expose the old destructive overwrite: the old
rename could already fail there. Use a fixture-owned Unix read-denied regular
file with a writable parent in the Linux/macOS lanes, restore its permissions
before comparing bytes, and never alter process-global or unrelated paths.
Retain a separate directory fixture where useful for consumer error behavior.

Actual Session/Gateway controls assert raw secret and metadata preservation,
absent metadata preservation, redacted errors and subsequent legitimate
success. Widget controls assert unchanged persisted entry, no config callback
or acknowledgement, and same-request success after valid repair. Manager
controls bind real production services/trays, stabilize active-Agent state
before measuring, assert zero resume plus the fixed tray, then exercise repair
through the same manager. New error and diagnostic assertions use synthetic
sentinels and generic assertion messages, never credential-bearing debug dumps.

Freeze complete changed scope and independently review production, callers,
tests and workflow. Only GitHub Actions may execute build, tests or evidence
controls. Reuse the existing locked Linux/macOS scoped runtime campaign and
caches; require each selected named positive with actual zero command/capture
outcomes, H/tree/U/input/log identity and shipping compilation. Preserve the
CGWA and other inherited regressions. Missing platform/service/device evidence
is not a pass. Unit verification requires the applicable scoped source
authority as well as behavior evidence; final OCRM acceptance remains separate.

## Source and evidence boundaries

This is a native Fabushi credential integrity repair required by the existing
full-scope migration, not a new OpenClaw runtime or a claim of upstream channel
API equivalence. Discovery deeply read the complete current store (blob
`d304c7cb153a03d0c07a9f310aa01a81e533edba`), its complete current contract,
the actual Session writer and lifecycle boundary, both actual consumers and
their immediate error/notification paths. The widget/manager readiness record
retains its exact full versus scoped read extents. Fixed-U source references
and upstream lifecycle limitations remain those of
`channel-gateway-credential-authorization.md` and
`session-lifecycle-delete-fence.md`; no historical read fact is promoted.
Use only the canonical OCRM inventory, parity ledger and RTM for later evidence
links. No global flag, source closure or full-channel acceptance follows from
this spec, a source review or isolated controlled tests.
