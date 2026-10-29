# OCRM Technical Dossier: Round 100 — OpenClaw Storage, Migration, Diagnostics & Observability Native Fusion

## 1. Executive Summary

This dossier records the full-source native Rust equivalence and fusion for:
1. **`OCRM-M29`**: Storage, Migration, Data Recovery & Compatibility (`source/host/src/extensions/session/session_recovery.rs`, `source/host/src/agents/agent_profile.rs`, `source/host/src/agents/settings_file.rs`)
2. **`OCRM-M30`**: Logging, Diagnostics, Health Checks & Observability (`source/host/src/host_diagnostics.rs`, `source/host/src/extensions/telemetry/`)

All upstream responsibilities from `openclaw/openclaw` across `src/state`, `src/storage`, `src/compat`, `src/logging`, and `src/status` are mapped directly to canonical Fabushi Rust architecture without dual writers or secondary state stores.

---

## 2. Upstream Source to Canonical Fabushi Rust Mapping

### 2.1 Storage, Migration & Data Recovery (`OCRM-M29`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/storage/recovery.ts` | Transcript entry deduplication & match recovery | `transcript_entry_matches_recovered` | Host Session Extension |
| `src/storage/blob-cache.ts` | Coalesced async blob reading & write-through storage | `CachedBlobReadStore`, `cache_blob_reads` | Host Session Extension |
| `src/storage/profile.ts` | Agent profile file idempotent initialization & preservation | `ensure_profile_file`, `write_sand_profile_file` | Host Agents Engine |
| `src/storage/settings.ts` | Agent settings file idempotent initialization & preservation | `ensure_settings_file`, `write_sand_settings_file` | Host Agents Engine |

### 2.2 Diagnostics, Health & Observability (`OCRM-M30`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/logging/diagnostic.ts` | Structured diagnostic event representation | `HostDiagnostic` | Host Diagnostics Engine |
| `src/logging/reporter.ts` | Thread-safe diagnostic reporter registration & unregistration | `pin_host_diagnostics_reporter` | Host Diagnostics Engine |
| `src/status/report.ts` | Synchronous diagnostic dispatch to active sink | `report_host_diagnostic` | Host Diagnostics Engine |
| `src/status/event.ts` | Convenience JSON event dispatcher | `report_diagnostic_event` | Host Diagnostics Engine |

---

## 3. Falsifiable Test Oracles

1. **`OCRM-STORAGE-O01` (Message Recovery Equality)**: Validates transcript message recovery equality based on role and content, ignoring transient metadata.
2. **`OCRM-STORAGE-O02` (SendMessage Equality)**: Validates outbound message equality based on structured message payload.
3. **`OCRM-STORAGE-O03` (ToolCall Equality)**: Validates tool call recovery matching on name, status, and summary.
4. **`OCRM-STORAGE-O04` (Kind Mismatch & Rejections)**: Validates rejection on differing entry kinds or notice entries.
5. **`OCRM-STORAGE-O05` (Async Blob Read Coalescing)**: Validates single-flight async read execution across multiple concurrent getters.
6. **`OCRM-STORAGE-O06` (Blob Write-Through)**: Validates immediate cache invalidation and update upon write-through.
7. **`OCRM-STORAGE-O07` (Profile File Idempotency)**: Validates creation of default profile on first access and preservation on subsequent calls.
8. **`OCRM-STORAGE-O08` (Settings File Idempotency)**: Validates creation of default settings on first access and preservation on subsequent calls.
9. **`OCRM-DIAG-O01` (Diagnostic Representation)**: Validates kind and fields serialization and type safety in `HostDiagnostic`.
10. **`OCRM-DIAG-O02` (Reporter Lifecycle)**: Validates registration and clearing of thread-safe diagnostic reporters.
11. **`OCRM-DIAG-O03` (Synchronous Diagnostic Dispatch)**: Validates synchronous delivery of diagnostic events to active reporter.
12. **`OCRM-DIAG-O04` (JSON Event Dispatcher)**: Validates `report_diagnostic_event` parsing and forwarding of structured event payloads.
