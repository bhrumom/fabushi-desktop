# Config Gateway source projection — Specification

Status: active  
Owner: Fabushi Desktop Host / Gateway owner  
Last updated: 2026-10-30  
Related project: `projects/openclaw-rust`  
Related task / issue / PR: N/A

## 1. Context / problem

The native `config.get` and `config.set` Gateway methods already use the
existing `SettingsService`, but their response only exposes the adapted
settings value and a projection hash. Consumers cannot distinguish an absent,
malformed, or valid settings file from a successfully projected in-memory
settings value.

## 2. Goal

Expose non-secret source metadata for the existing settings owner through the
Gateway config projection, without introducing another configuration store or
claiming full OpenClaw schema parity.

## 3. Non-goals / out of scope

- No raw configuration body or secret values may be returned.
- No replacement of `SettingsService` or its persistence policy.
- No OpenClaw-wide schema migration, restart orchestration, or provider config
  implementation.
- No change to the existing `settings`/`config` compatibility payloads.

## 4. Requirements

- R1: `config.get` and successful `config.set` return `configSource` with the
  settings path, file existence, parse/version validity, and SHA-256 of the
  raw file bytes when readable.
- R2: Missing or unreadable files are represented as metadata state, not as a
  successful raw-config claim; no raw bytes or parse errors are disclosed.
- R3: Metadata is read from the existing `SettingsService` path and remains a
  read-only projection; all writes remain owned by `set_host_settings`.
- R4: Existing base-hash checks and settings/config aliases remain unchanged.

## 5. Current state

`UnifiedGatewayApi` reads and writes `SettingsService`, computes a hash over
the adapted settings value, and returns compatibility aliases. The settings
owner already exposes its canonical path but does not publish source status.

## 6. Target state

Both config responses include a bounded, secret-free `configSource` metadata
object. The projection hash continues to describe the adapted response value;
`configSource.rawHash` describes the persisted raw file bytes and is not used
as a replacement for the existing `baseHash` contract.

## 7. Architecture and ownership boundaries

`SettingsService` owns source-path inspection and persistence. The Gateway
owns response adaptation only. No caller, UI, or new store may become a source
of configuration truth.

## 8. Interfaces / contracts / schemas / data flow

`configSource` fields: `path` (string), `exists` (boolean), `valid` (boolean),
and `rawHash` (string or null). `valid` is true only for a readable JSON
object with the supported settings version (`1`).

## 9. Constraints and non-functional requirements

Metadata must not include raw settings, credentials, or parser diagnostics.
Inspection is bounded to the existing settings file and performs no mutation.

## 10. Failure modes and edge cases

Missing files return `exists=false`, `valid=false`, `rawHash=null`.
Unreadable files return `exists=true`, `valid=false`, `rawHash=null`.
Malformed JSON or an unsupported version returns `exists=true`, `valid=false`
with the raw-byte hash retained when the file was readable.

## 11. Implementation strategy

Add a source-metadata method to `SettingsService` using the canonical path,
then attach its JSON-safe result to the existing Gateway get/set responses.
Keep the current settings projection, write path, and concurrency guard intact.

## 12. Verification / test strategy

Perform source inspection, focused contract assertions for response fields and
failure-state mapping, and `git diff --check`. Runtime tests/builds/CI remain
separate acceptance evidence.

## 13. Acceptance criteria / Definition of Done

- AC-1: Both successful config methods publish `configSource` from the existing
  settings owner.
- AC-2: The projection never returns raw settings bytes or secret values.
- AC-3: Existing aliases and `baseHash` behavior remain present in source.
- AC-4: The change passes static review and `git diff --check`.

## 14. Release / migration / rollback

No migration. Reverting the implementation removes the additive metadata
without changing persisted settings.

## 15. Observability / evidence

The metadata is the observable source-state evidence. No additional logging is
required because file paths and hashes must not be emitted into diagnostics by
this projection.

## 16. References / provenance

- `projects/openclaw-rust/dossiers/config-gateway-projection-frontier-20261030.md`
- `source/host/src/extensions/settings/settings_service.rs`
- `source/host/app/src/main.rs` (`config.get` / `config.set`)

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1–R4 | passed | `SettingsService::get_config_source_metadata` and Gateway responses |
| AC-1–AC-3 | passed | source inspection of get/set responses and unchanged base-hash path |
| AC-4 | passed | `git diff --check` |
