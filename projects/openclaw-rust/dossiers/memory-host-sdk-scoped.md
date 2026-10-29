# OCRM Dossier: Memory Host SDK, Dreaming Promotion & Event Store

- **Unit ID**: `OCRM-UNIT-044`
- **Module ID**: `OCRM-M26` (`memory-host-sdk`)
- **Responsibility IDs**:
  - `OCRM-RESP-MEMORY-DREAMING-001`
  - `OCRM-RESP-MEMORY-EVENT-STORE-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 13 upstream files in `src/memory-host-sdk/` (1,348 lines total, 100% of the directory) are audited, mapped, and natively implemented:

### `OCRM-RESP-MEMORY-DREAMING-001` (2 files, 823 lines)
1. `src/memory-host-sdk/dreaming.ts` (567 lines)
2. `src/memory-host-sdk/dreaming.test.ts` (256 lines)

### `OCRM-RESP-MEMORY-EVENT-STORE-001` (11 files, 525 lines)
1. `src/memory-host-sdk/event-store.ts` (324 lines)
2. `src/memory-host-sdk/event-types.ts` (80 lines)
3. `src/memory-host-sdk/events.ts` (66 lines)
4. `src/memory-host-sdk/event-export.ts` (20 lines)
5. `src/memory-host-sdk/secret.ts` (8 lines)
6. `src/memory-host-sdk/status.ts` (5 lines)
7. `src/memory-host-sdk/multimodal.ts` (5 lines)
8. `src/memory-host-sdk/query.ts` (5 lines)
9. `src/memory-host-sdk/engine-storage.ts` (2 lines)
10. `src/memory-host-sdk/host/config-utils.ts` (5 lines)
11. `src/memory-host-sdk/host/types.ts` (5 lines)

## 2. Invariants & Implementation Details

- **Memory Dreaming Promotion & Calibration**:
  - Cron frequency defaults to `0 3 * * *` (3 AM daily) tagged with `[managed-by=memory-core.short-term-promotion]`.
  - Promotion calibration thresholds: `min_score = 0.75`, `min_recall_count = 3`, `min_unique_queries = 3`.
  - Recency half-life: 14 days, decaying score by $2^{-\Delta t / 14}$. Snippets older than several half-lives fail promotion unless repeatedly reinforced across distinct queries.
- **Memory Host Event Store Boundaries**:
  - Namespace: `memory-host.events` with a 10,000-event retention ceiling.
  - Strict payload bounds: arrays clamped to `MAX_MEMORY_HOST_EVENT_ITEMS = 10`, paths to 256 bytes, queries/texts to 2048 bytes.
  - **Storage Truncation Invariant**: When diagnostic items are truncated, `storage_truncated = true` is flagged, but aggregate counts (e.g. `result_count`, `eligible_result_count`) strictly remain exact.
  - Legacy discovery: `resolve_memory_host_event_log_path` returns `<workspace_dir>/memory/.dreams/events.jsonl` for doctor migration checks.

## 3. Rust Native Owner & Verification Evidence

- **Rust Implementation**:
  - `source/host/src/extensions/memory/dreaming_service.rs`
  - `source/host/src/extensions/memory/event_store_service.rs`
  - Registered in `source/host/src/extensions/memory/mod.rs`
- **Contract Tests**:
  - `source/host/tests/openclaw_memory_host_sdk_contract.rs`
    - `test_ocrm_memory_dreaming_promotion_and_decay`
    - `test_ocrm_memory_event_store_normalization_and_truncation`
    - `test_ocrm_memory_event_store_path_truncation_and_slice`
- **Verification Result**: 3 passed; 0 failed.
