# OCRM Dossier: Model Catalog Manifest Planning, Remote Overlays & Wire Bundle Projections

- **Unit ID**: `OCRM-UNIT-041`
- **Module ID**: `OCRM-M06` (`model-catalog`)
- **Responsibility IDs**:
  - `OCRM-RESP-MODEL-CATALOG-PLANNER-001`
  - `OCRM-RESP-MODEL-CATALOG-REMOTE-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 17 upstream files in `src/model-catalog/` (3,687 lines total) are audited, mapped, and natively implemented (bringing `src/model-catalog/` to 100% completion alongside existing `pricing.ts`):

### `OCRM-RESP-MODEL-CATALOG-PLANNER-001` (4 files, 1,104 lines)
1. `src/model-catalog/index.ts` (29 lines)
2. `src/model-catalog/manifest-planner.ts` (380 lines)
3. `src/model-catalog/manifest-planner.test.ts` (582 lines)
4. `src/model-catalog/endpoint-eligibility.test.ts` (113 lines)

### `OCRM-RESP-MODEL-CATALOG-REMOTE-001` (13 files, 2,583 lines)
1. `src/model-catalog/bundled-catalog-stamp.ts` (27 lines)
2. `src/model-catalog/pricing.test.ts` (817 lines)
3. `src/model-catalog/remote-bundle.ts` (110 lines)
4. `src/model-catalog/remote-config.ts` (25 lines)
5. `src/model-catalog/remote-overlay.ts` (218 lines)
6. `src/model-catalog/remote-overlay.test.ts` (266 lines)
7. `src/model-catalog/remote-overlay.test-support.ts` (35 lines)
8. `src/model-catalog/remote-overlay.worker.test-support.ts` (13 lines)
9. `src/model-catalog/remote-refresh.ts` (201 lines)
10. `src/model-catalog/remote-refresh.test.ts` (394 lines)
11. `src/model-catalog/remote-store.ts` (152 lines)
12. `src/model-catalog/remote-store.test.ts` (223 lines)
13. `src/model-catalog/remote-store.worker.ts` (102 lines)

## 2. Invariants & Implementation Details

- **Remote Catalog Configuration & Active Source Checking**:
  - Default URL: `https://catalog.openclaw.ai/models/v2/catalog.json`.
  - Retired Default URL: `https://catalog.openclaw.ai/models/v1/catalog.json`.
  - `is_remote_catalog_source_active`: Accepts the exact configured URL or gracefully matches the retired V1 URL against the default V2 source to preserve offline access across upgrades.
- **Cache Freshness & Storage Boundaries**:
  - `REMOTE_MODEL_CATALOG_TTL_MS`: Fixed at 6 hours (21,600,000 ms).
  - `REMOTE_MODEL_CATALOG_TIMEOUT_MS`: 15,000 ms.
  - `REMOTE_MODEL_CATALOG_MAX_BYTES`: 4 MB body limit.
  - Snapshot freshness evaluates `current_time - checked_at < TTL`.
- **Wire Bundle Versions & Projections**:
  - Supports V1, V2, and V3 wire bundle structures.
  - V1 maps embedded provider models with implicit pricing.
  - V2/V3 project separate models table into provider catalogs with explicit pricing (`explicit: true`).
  - Hosted recommendations mapped per provider.
- **Manifest Planning & Trusted Transport Overrides**:
  - Merges remote provider models with local trusted configuration.
  - Local trusted configuration strictly overrides remote `base_url`, `headers`, and `api` endpoints, preventing untrusted remote catalogs from redirecting traffic to unauthorized hosts.
  - Evaluates suppression rules: models matching `merge_key` are filtered out.
  - Detects duplicate `(provider, model_id)` combinations and surfaces them as conflicts without throwing or aborting.

## 3. Rust Native Owner & Verification Evidence

- **Rust Implementation**:
  - `source/host/src/extensions/inference/model_catalog_service.rs`
  - Registered in `source/host/src/extensions/inference/mod.rs`
- **Contract Tests**:
  - `source/host/tests/openclaw_model_catalog_contract.rs`
    - `test_ocrm_model_catalog_o01_config_and_freshness`
    - `test_ocrm_model_catalog_o02_bundle_projection_v1_and_v3`
    - `test_ocrm_model_catalog_o03_manifest_planner_and_trusted_transport`
    - `test_ocrm_model_catalog_o04_conflict_detection_and_filtering`
- **Verification Result**: 4 passed; 0 failed.
