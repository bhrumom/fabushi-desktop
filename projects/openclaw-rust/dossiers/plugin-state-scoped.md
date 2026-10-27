# OpenClaw Plugin State, Blob Store & Runtime Health Parity Dossier

## 1. Overview & Responsibility Mapping
- **Module ID**: `OCRM-RESP-PLUGIN-STATE-BLOB-STORE-001`, `OCRM-RESP-PLUGIN-STATE-KEYED-STORE-001`, `OCRM-RESP-PLUGIN-STATE-RUNTIME-HEALTH-001`
- **Scope**: 60 upstream files in `src/plugin-state/` (100% of directory) in `part-0043` inventory shard.
- **Owner**: `mahayana_host_runtime::extensions::plugin_state` (`blob_store.rs`, `state_store.rs`, `runtime_health.rs`).

## 2. Audited Files & Upstream Anchors
- **Plugin Blob Store & Quotas** (10 files):
  - `src/plugin-state/plugin-blob-store.ts`, `src/plugin-state/plugin-blob-store.sqlite.ts`
  - `src/plugin-state/plugin-blob-store.types.ts`, `src/plugin-state/plugin-blob-worker-client.ts`
  - `src/plugin-state/plugin-blob-worker-contract.ts`, `src/plugin-state/plugin-blob-store.worker.ts`
  - `src/plugin-state/plugin-blob-store.admission.test.ts`, `src/plugin-state/plugin-blob-store.eviction.test.ts`
  - `src/plugin-state/plugin-blob-store.test.ts`, `src/plugin-state/plugin-blob-store.test-helpers.ts`
- **Plugin State Keyed Store & Journal** (33 files):
  - `src/plugin-state/plugin-state-store.ts`, `src/plugin-state/plugin-state-store.sqlite.ts`
  - `src/plugin-state/plugin-state-store.kernel.ts`, `src/plugin-state/plugin-state-store.journal.ts`
  - `src/plugin-state/plugin-state-store.validation.ts`, `src/plugin-state/plugin-store-validation.ts`
  - `src/plugin-state/plugin-state-store.reads.ts`, `src/plugin-state/plugin-state-store.mutations.ts`
  - `src/plugin-state/plugin-state-store.retention.ts`, `src/plugin-state/plugin-state-store.migration.ts`
  - And all related SQLite database, moves worker, and retained state test suites.
- **Runtime Health Store & Process Liveness** (17 files):
  - `src/plugin-state/runtime-health-store.ts`, `src/plugin-state/runtime-health-records.ts`
  - `src/plugin-state/runtime-health-store.kernel.ts`
  - `src/plugin-state/plugin-state-operation.ts`, `src/plugin-state/plugin-state-operation.kernel.ts`
  - `src/plugin-state/plugin-state-operation-epochs.ts`, `src/plugin-state/plugin-state-publication.ts`
  - `src/plugin-state/plugin-state-observation.ts`, `src/plugin-state/plugin-state-observation-cache.ts`
  - And all related state error causes and operation contract suites.

## 3. Rust Native Equivalence Architecture
1. **Blob Store & Storage Limits** (`blob_store.rs`):
   - Strict size and entry limits matching upstream SQLite backing:
     - `MAX_PLUGIN_BLOB_BYTES_PER_ENTRY = 100 MiB`
     - `MAX_PLUGIN_BLOB_BYTES_PER_PLUGIN = 512 MiB`
     - `MAX_PLUGIN_BLOB_ENTRIES_PER_PLUGIN = 50,000`
   - Deterministic TTL expiration and selective eviction.
2. **Keyed State Store & Sequenced Journal** (`state_store.rs`):
   - 1 MiB value size limit (`MAX_PLUGIN_STATE_VALUE_BYTES`).
   - Bulk deletion bounded by 512 entries (`MAX_PLUGIN_STATE_BULK_DELETE_ENTRIES`).
   - Monotonic sequence counter and journal audit log.
3. **Runtime Health & Process Tokens** (`runtime_health.rs`):
   - Process token scoping preventing restarted processes from adopting predecessor's records.
   - Display group deduplication supporting `Earliest` (root cause) and `Latest` (most recent symptom) selection.

## 4. Contract Tests & Verification
- Test harness: `source/host/tests/openclaw_plugin_state_contract.rs`
  - `test_ocrm_plugin_state_blob_store_bounds`
  - `test_ocrm_plugin_state_keyed_store_and_journal`
  - `test_ocrm_plugin_state_runtime_health`
- Evidence: 3/3 passing contract tests with full positive and negative assertions.
