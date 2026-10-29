# OCRM Dossier: Media Generation & Music Generation Core

- **Unit ID**: `OCRM-UNIT-038`
- **Module ID**: `OCRM-M25` (`media-generation` & `music-generation`)
- **Responsibility IDs**:
  - `OCRM-RESP-MEDIA-GEN-CORE-001`
  - `OCRM-RESP-MUSIC-GEN-CORE-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 21 upstream files (11 in `src/media-generation/` and 10 in `src/music-generation/`, 1,931 lines total) are audited, mapped, and natively implemented:

### `src/media-generation/` (11 files, 1,385 lines) -> `OCRM-RESP-MEDIA-GEN-CORE-001`
1. `src/media-generation/geometry-normalization.ts` (134 lines)
2. `src/media-generation/live-test-helpers.test.ts` (39 lines)
3. `src/media-generation/live-test-helpers.ts` (72 lines)
4. `src/media-generation/model-ref.ts` (3 lines)
5. `src/media-generation/provider-assets.test.ts` (79 lines)
6. `src/media-generation/provider-assets.ts` (124 lines)
7. `src/media-generation/provider-registry.ts` (74 lines)
8. `src/media-generation/registry.test.ts` (99 lines)
9. `src/media-generation/registry.ts` (48 lines)
10. `src/media-generation/runtime-shared.test.ts` (321 lines)
11. `src/media-generation/runtime-shared.ts` (733 lines)

### `src/music-generation/` (10 files, 546 lines) -> `OCRM-RESP-MUSIC-GEN-CORE-001`
1. `src/music-generation/capabilities.test.ts` (20 lines)
2. `src/music-generation/capabilities.ts` (26 lines)
3. `src/music-generation/live-test-helpers.ts` (26 lines)
4. `src/music-generation/provider-assets.test.ts` (55 lines)
5. `src/music-generation/provider-assets.ts` (152 lines)
6. `src/music-generation/runtime-types.ts` (21 lines)
7. `src/music-generation/runtime.resources.test.ts` (89 lines)
8. `src/music-generation/runtime.test.ts` (46 lines)
9. `src/music-generation/runtime.ts` (8 lines)
10. `src/music-generation/types.ts` (103 lines)

## 2. Invariants & Implementation Details

- **Geometry & Dimension Normalization**:
  - `derive_aspect_ratio_from_size`: reduces `WIDTHxHEIGHT` to minimal integer ratio `W:H` via Euclidean GCD.
  - `resolve_closest_aspect_ratio`: evaluates candidates via primary log-distance ratio `|ln(cand_val / req_val)|` and secondary dimension difference `|cand_w * req_h - req_w * cand_h|`.
  - `resolve_closest_size`: ranks supported dimensions by aspect ratio preservation, then area closeness.
  - `resolve_closest_resolution`: parses resolution rank (`1K`..`4K`, `720P`..`1080P`); breaks ties in favor of higher resolution (secondary score `0 < 1`).
- **Duration & Timeout Clamping**:
  - `normalize_duration_to_closest_max`: rounds float durations and caps them between `1` and `max_duration_seconds`.
  - `resolve_media_provider_request_timeout_ms`: clamps request or provider default timeout to `[1_000, 2_147_483_647]` milliseconds.
- **Music Generation Capabilities & Image Conditioning**:
  - `list_supported_music_generation_modes`: returns `["generate"]` by default and appends `"edit"` when edit capabilities are enabled.
  - `resolve_reference_image_capability_error`: verifies that reference images are rejected when edit is disabled, and enforces maximum input image limits.
- **Generated Video Asset Handling**:
  - `read_generated_video_asset`: inspects MIME type, generates numbered filenames, validates byte caps, and falls back cleanly to `overflow_url` when limits are exceeded.

## 3. Rust Native Equivalence

Implemented in `source/host/src/extensions/attachments/media_generation_service.rs`:
- Types: `MusicGenerationOutputFormat`, `GeneratedMusicAsset`, `GeneratedVideoAsset`, `MusicGenerationModeCapabilities`, `MusicGenerationEditCapabilities`, `MusicGenerationProviderCapabilities`.
- Geometry: `derive_aspect_ratio_from_size`, `parse_aspect_ratio_value`, `parse_size_dimensions`, `resolve_closest_aspect_ratio`, `resolve_closest_size`, `parse_resolution_rank`, `resolve_closest_resolution`.
- Limits & Timeouts: `normalize_duration_to_closest_max`, `resolve_media_provider_request_timeout_ms`, `greatest_common_divisor`.
- Capabilities & Video: `list_supported_music_generation_modes`, `resolve_music_generation_mode_capabilities`, `resolve_reference_image_capability_error`, `read_generated_video_asset`.

## 4. Verification Evidence

- Dedicated contract test suite: `source/host/tests/openclaw_media_generation_contract.rs`
  - `test_ocrm_media_gen_o01_aspect_ratio_normalization_and_derivation`: GCD reduction, exact ratio matching, size derivation, log distance ranking.
  - `test_ocrm_media_gen_o02_size_and_resolution_normalization`: exact size lookup, aspect ratio derivation, resolution rank parsing and higher resolution tie-breaking.
  - `test_ocrm_media_gen_o03_duration_and_timeout_clamping`: float duration rounding and ceiling clamping, timeout bounds clamping.
  - `test_ocrm_media_gen_o04_music_generation_capabilities_and_reference_images`: mode enumeration, input image overflow and disabled rejection.
  - `test_ocrm_media_gen_o05_video_asset_reading_and_overflow`: buffer reading, overflow URL fallback, strict overflow error.
- All 5 tests passed with 0 failures; 31 media understanding tests continue to pass with 0 failures.
