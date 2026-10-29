# OCRM Dossier: Media Understanding Provider Capability Registry & Execution

- **Unit ID**: `OCRM-UNIT-036`
- **Module ID**: `OCRM-M24` (`media-understanding`)
- **Responsibility IDs**:
  - `OCRM-RESP-MEDIA-UND-PROVIDER-REGISTRY-001`
  - `OCRM-RESP-MEDIA-UND-PROVIDER-EXEC-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 12 upstream files in `src/media-understanding/` (1,441 lines total) are audited and mapped:
1. `src/media-understanding/provider-registry-metadata.ts` (39 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-REGISTRY-001`
2. `src/media-understanding/provider-registry.ts` (80 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-REGISTRY-001`
3. `src/media-understanding/provider-registry.test.ts` (142 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-REGISTRY-001`
4. `src/media-understanding/provider-registry.clawrouter.test.ts` (85 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-REGISTRY-001`
5. `src/media-understanding/provider-capability-registry.ts` (51 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-REGISTRY-001`
6. `src/media-understanding/provider-capability-registry.test.ts` (50 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-REGISTRY-001`
7. `src/media-understanding/provider-capability-registry.loading.test.ts` (126 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-REGISTRY-001`
8. `src/media-understanding/provider-execution.ts` (271 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-EXEC-001`
9. `src/media-understanding/provider-json-post.ts` (35 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-EXEC-001`
10. `src/media-understanding/shared.test.ts` (451 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-EXEC-001`
11. `src/media-understanding/media-config.test-support.ts` (26 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-EXEC-001`
12. `src/media-understanding/media-understanding-url-fallback.test.ts` (85 lines) -> `OCRM-RESP-MEDIA-UND-PROVIDER-EXEC-001`

## 2. Invariants & Implementation Details

- **Provider Registry & Metadata Resolution**:
  - Provider IDs are case-insensitively trimmed and normalized.
  - `resolve_default_media_model_from_registry`: looks up registered provider, returning configured default model for the specified media capability.
  - `resolve_auto_media_key_providers_from_registry`: selects providers implementing the required capability, sorting by ascending `autoPriority`, and breaking ties lexicographically by provider ID.
  - `build_media_understanding_registry`: merges declared plugin providers, dynamically auto-registers configured providers with image capabilities, and applies configuration overrides.
- **Provider Request Context Formulation**:
  - `resolve_provider_request_context`: merges URL and header configuration along the hierarchical precedence chain: `entry` (model) overrides `tools` (capability config), which overrides global `provider` defaults.
  - Header merges respect override values per-key while preserving inherited headers.

## 3. Rust Native Equivalence

Implemented in `source/host/src/extensions/attachments/media_understanding_service.rs`:
- `MediaUnderstandingProvider`, `normalize_media_provider_id`
- `resolve_default_media_model_from_registry`, `resolve_auto_media_key_providers_from_registry`
- `build_media_understanding_registry`, `resolve_provider_request_context`

## 4. Verification Evidence

- Contract test suite: `source/host/tests/openclaw_media_understanding_contract.rs`
  - `test_ocrm_media_und_o26_provider_registry_and_auto_priority_sorting`: verifies default model retrieval and ascending priority provider sorting with tie-breaking.
  - `test_ocrm_media_und_o27_provider_registry_build_and_request_context`: verifies registry assembly, image-capable auto-registration, override merging, and hierarchical request context resolution.
- All 27 tests in `openclaw_media_understanding_contract.rs` pass cleanly with 0 failures.
