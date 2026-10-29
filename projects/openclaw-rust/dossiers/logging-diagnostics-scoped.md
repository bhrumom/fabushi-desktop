# OpenClaw Logging & Diagnostics Parity Dossier

## 1. Overview & Responsibility Mapping
- **Module ID**: `OCRM-RESP-LOGGING-CORE-CONSOLE-001`, `OCRM-RESP-LOGGING-SECRET-REDACTION-001`, `OCRM-RESP-LOGGING-DIAGNOSTIC-TELEMETRY-001`
- **Scope**: 155 upstream files covering 100% of `src/logging/` (including `src/logging/test-helpers/`), `src/logger.ts`, `src/logger.test.ts`, and `src/logging.ts`.
- **Owner**: `mahayana_host_runtime::extensions::telemetry` (`openclaw_logging_core.rs`, `openclaw_secret_redactor.rs`, `openclaw_diagnostic_telemetry.rs`).

## 2. Audited Files & Upstream Anchors
- **Core Logging, Formatting & Filtering** (51 files):
  - `src/logging/config.ts`, `src/logging/config.test.ts`
  - `src/logging/console.ts`, `src/logging/console-capture.test.ts`, `src/logging/console-timestamp.test.ts`
  - `src/logging/env-log-level.ts`
  - `src/logging/json-console-line.ts`
  - `src/logging/levels.ts`, `src/logging/levels.test.ts`, `src/logging/level-filter.test.ts`
  - `src/logging/log-file-path.test.ts`, `src/logging/log-format.test.ts`, `src/logging/log-line.test.ts`
  - `src/logging/logger-env.test.ts`, `src/logging/logger-file-message.ts`, `src/logging/logger-file-rotation.test.ts`
  - `src/logging/logger.ts`, `src/logging/logger.browser-import.test.ts`, `src/logging/logger.test-support.ts`
  - `src/logging/message-lifecycle.test.ts`, `src/logging/parse-log-line.test.ts`
  - `src/logging/state.ts`, `src/logging/state.test.ts`
  - `src/logging/structured-authorization-code.ts`
  - `src/logging/subsystem.ts`, `src/logging/subsystem.test.ts`
  - `src/logging/timestamps.ts`, `src/logging/timestamps.test.ts`
  - `src/logging/types.ts`
  - `src/logger.ts`, `src/logger.test.ts`, `src/logging.ts`
  - And all related test fixtures and file rotation suites.
- **Secret Redaction & Token Masking** (26 files):
  - `src/logging/redact.ts`, `src/logging/redact-bounded.ts`, `src/logging/redact-edit-composition.ts`
  - `src/logging/redact-json.ts`, `src/logging/redact-linear-matchers.ts`, `src/logging/redact-mask.ts`
  - `src/logging/redacting-stream.test.ts`
  - `src/logging/secret-redaction-registry.test-support.ts`
  - And related secret masking and structured authorization pattern matchers.
- **Diagnostic Telemetry, Heap/CPU Profilers & GC** (78 files):
  - `src/logging/diagnostic-argument-churn-activity.ts`
  - `src/logging/diagnostic-backend-liveness.test.ts`
  - `src/logging/diagnostic-controller-projection.ts`
  - `src/logging/diagnostic-cpu-profile.ts`, `src/logging/diagnostic-cpu-profile.test.ts`
  - `src/logging/diagnostic-embedded-run-index.ts`
  - `src/logging/diagnostic-gc.ts`, `src/logging/diagnostic-gc.test.ts`
  - `src/logging/diagnostic-heap-profile.ts`, `src/logging/diagnostic-heap-profile.test.ts`, `src/logging/diagnostic-heap-profile.fixture.json`
  - `src/logging/diagnostic-heap-snapshot.ts`
  - `src/logging/diagnostic-heartbeat-context.ts`, `src/logging/diagnostic-heartbeat-context.test.ts`
  - `src/logging/diagnostic-run-activity-snapshot.ts`
  - `src/logging/code-mode-diagnostic.ts`
  - And all remaining 67 diagnostic runner and profiler files.

## 3. Rust Native Equivalence Architecture
1. **Core Logging & Console Formatting** (`openclaw_logging_core.rs`):
   - Structured `LogLevel` hierarchy (`Silent`, `Fatal`, `Error`, `Warn`, `Info`, `Debug`, `Trace`).
   - Env-driven resolution (`OPENCLAW_LOG_LEVEL`, `DEBUG`) and verbose flag overrides.
   - Dual console formatting: typed JSON lines with ISO8601 timestamps and subsystem metadata, plus human-readable colorized text.
2. **Secret Redaction & Masking** (`openclaw_secret_redactor.rs`):
   - Token masking preserving 4-character boundaries for auditability without exposing entropy.
   - Comprehensive URL query parameter sanitizer stripping `api_key`, `token`, `secret`, `password`, `auth`, `credential`.
   - Multiline PEM private key block redaction into `[REDACTED_PRIVATE_KEY]`.
   - Authorization header sanitizer for `Bearer` and `Basic` credentials.
   - **Current evidence boundary**: `error_diagnostics::sanitize_error_diagnostic` now delegates query and PEM redaction to this owner, but repository search found no production caller of `DiagnosticError::with_diagnostic` or `format_display`; the available evidence is helper/local-contract only. `OCRM-RESP-LOGGING-SECRET-REDACTION-001` therefore remains `wiring=not-wired` with `BL-LOGGING-SECRET-WIRING-001`.
3. **Diagnostic Telemetry & Health Monitoring** (`openclaw_diagnostic_telemetry.rs`):
   - Multi-tier memory pressure evaluator (`Normal`, `Moderate`, `Critical`) triggering GC and memory cache shedding.
   - Session heartbeat registry tracking active controller operations with sweep liveness expiration.

## 4. Contract Tests & Verification
- Test harness: `source/host/tests/openclaw_logging_contract.rs`
  - `test_ocrm_logging_levels_and_formatting`
  - `test_ocrm_logging_secret_redaction`
  - `test_ocrm_logging_diagnostics_and_memory_pressure`
- Evidence: 3/3 passing contract tests with full positive and negative assertions.
- Additional local-contract evidence: `source/host/tests/openclaw_crash_guard_media_contract.rs::test_ocrm_crash_o03_error_diagnostics_redaction` covers URL-query and PEM sanitization through a directly constructed `DiagnosticError`.
