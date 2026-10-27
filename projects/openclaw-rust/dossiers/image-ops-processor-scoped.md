# OCRM Technical Dossier — Unit 22: Image Operations, Pixel Caps, Display Metadata & Attachment Preparation (`OCRM-UNIT-MEDIA-009`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Image Operations, Display Dimension Normalization, EXIF Orientation Swaps, VP8X/APNG Animation Detection, BMP Normalization & Attachment Preprocessing)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 10 core image operations, processor configuration, worker execution, attachment preparation, and image test suite files (totaling 961 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/image-ops.ts` | `a3570b0fdbb410f26fc58596910152d45e6832b8` | 236 | 1..236 (full-file) | `OCRM-RESP-MEDIA-IMAGE-OPS-001` |
| `part-0042` | `src/media/image-processor.ts` | `dc10fca84534218baa58fd9614ec4656bc64f843` | 119 | 1..119 (full-file) | `OCRM-RESP-MEDIA-IMAGE-PROCESSOR-SPEC-001` |
| `part-0042` | `src/media/image-processor.worker.ts` | `ee115ba30b9152be338bb278055214a445b09f92` | 51 | 1..51 (full-file) | `OCRM-RESP-MEDIA-IMAGE-PROCESSOR-SPEC-001` |
| `part-0042` | `src/media/attachment-processor.runtime.ts` | `0fcf4ed44b34069328b3bac5bef2a6b0c4a0f4b4` | 71 | 1..71 (full-file) | `OCRM-RESP-MEDIA-ATTACHMENT-PROCESSOR-001` |
| `part-0042` | `src/media/attachment-processor.worker.ts` | `f17ac5981ec76e822bdda51eae5591bf762098b2` | 12 | 1..12 (full-file) | `OCRM-RESP-MEDIA-ATTACHMENT-PROCESSOR-001` |
| `part-0042` | `src/media/image-ops.helpers.test.ts` | `339087517caf0f8325b9116262a1498b37002d78` | 20 | 1..20 (full-file) | `OCRM-RESP-MEDIA-IMAGE-OPS-001` |
| `part-0042` | `src/media/image-ops.rastermill-adapter.test.ts` | `8d636e43f72edb6ea0a9456eba89a5214437fb41` | 297 | 1..297 (full-file) | `OCRM-RESP-MEDIA-IMAGE-OPS-001` |
| `part-0042` | `src/media/image-ops.worker.test.ts` | `e8f86c0e02c3a858a5d3a9fcc5608fe1e46f03fb` | 97 | 1..97 (full-file) | `OCRM-RESP-MEDIA-IMAGE-OPS-001` |
| `part-0042` | `src/media/image-processor.pixel-limits.test.ts` | `f4941c67fbdf8f0de17c19c2cb94c8a0b14d3a59` | 30 | 1..30 (full-file) | `OCRM-RESP-MEDIA-IMAGE-PROCESSOR-SPEC-001` |
| `part-0042` | `src/media/photon.runtime.test.ts` | `3a32779f6217527b39be304c146927b93d6fc397` | 28 | 1..28 (full-file) | `OCRM-RESP-MEDIA-PHOTON-RUNTIME-001` |

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

The audited sources were mapped to 4 discrete Rust runtime responsibilities registered in `parity-ledger.json`. In adherence to the strict honest wiring policy, because full desktop chat rendering pipelines and UI transform hooks have not yet connected these helpers into complete end-to-end paths, they are conservatively marked `wiring=not-wired`, `verification=not-run`, `acceptance=pending` with explicit blocker IDs:

1. **`OCRM-RESP-MEDIA-IMAGE-OPS-001` (NEW)**:
   - **Behavior**: Descending de-duplicated image resize side grid generation (`[sideStart, 1800, 1600, 1400, 1200, 1000, 800]`), ladder quality reduction steps (`[85, 75, 65, 55, 45, 35]`), byte-budget quality search steps (`[92, 85, 78, 70, 62, 52, 42]`), animated WebP detection via RFC 9649 VP8X bit 1 (0x02), APNG / still PNG distinction scanning up to 512 chunks (rejecting `acTL` before `IDAT`), EXIF orientation parsing (swapping width and height for display when orientation is 5..8), and Anthropic inline image normalization converting unsupported BMP to PNG.
   - **Rust Owner**: `source/host/src/extensions/attachments/image_ops.rs::build_image_resize_side_grid, is_animated_webp_buffer, is_still_png_buffer, read_image_probe_from_header, read_image_metadata_from_header, convert_bmp_to_png, prepare_attachment, normalize_anthropic_inline_image`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `image_ops` → Consumer: *Pending chat image resize and preview UI consumers*.
   - **Oracles**: `OCRM-IMAGE-OPS-O01`, `OCRM-IMAGE-OPS-O02`, `OCRM-IMAGE-OPS-O03`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-IMAGE-OPS-CONSUMERS-001"]`.

2. **`OCRM-RESP-MEDIA-IMAGE-PROCESSOR-SPEC-001` (UPDATED)**:
   - **Behavior**: Image processor pixel admission gates (`MAX_IMAGE_INPUT_PIXELS = 25_000_000`, `DOWNSCALE_IMAGE_INPUT_PIXELS = 40_000_000`), local probe vs worker delegation, and pixel limit rejection.
   - **Rust Owner**: `source/host/src/extensions/attachments/image_ops.rs` and `media_file_context.rs`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-IMAGE-PROCESSOR-RASTERMILL-001"]`.

3. **`OCRM-RESP-MEDIA-ATTACHMENT-PROCESSOR-001` (UPDATED & CALIBRATED)**:
   - **Behavior**: Attachment preprocessing with `ATTACHMENT_OFFLOAD_THRESHOLD_BYTES = 2_000_000`, base64 inspection, MIME sniffing, container MIME refinement (`application/zip` / `application/octet-stream`), and non-decoding image dimension extraction with EXIF display orientation correction.
   - **Rust Owner**: `source/host/src/extensions/attachments/image_ops.rs::prepare_attachment, is_generic_container_mime`.
   - **Status**: Conservatively calibrated from premature `wired` to `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-ATTACHMENT-PROCESSOR-CONSUMERS-001"]` (chat message draft and inbound channel ingestion pipelines do not yet route through `prepare_attachment`).

4. **`OCRM-RESP-MEDIA-PHOTON-RUNTIME-001` (UPDATED)**:
   - **Behavior**: Pure Rust BMP-to-PNG fallback decoder and encoder preserving exact RGB/RGBA byte buffers.
   - **Rust Owner**: `source/host/src/extensions/attachments/image_ops.rs::convert_bmp_to_png` and `media_qr_png.rs::convert_bmp_to_png`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-PHOTON-BMP-PIPELINE-001"]`.

---

## 3. Strict Conservation Invariant Verification

- **Total tracked repository paths**: **58,750** (strictly conserved, zero omitted/untracked paths).
- **Mapped paths**: **203** (increased by 10 from 193).
- **Unclassified paths**: **58,547** (decreased by 10 from 58,557).
- **Conservation Check**: `203 + 58,547 = 58,750` (100% exact parity).
- **Total Ledger Responsibilities**: **125** (124 existing + 1 newly registered `OCRM-RESP-MEDIA-IMAGE-OPS-001`).
- **Global Completion Flags**: All 8 flags remain strictly `false`.

---

## 4. Contract Verification

All 9 contract tests in `source/host/tests/openclaw_image_ops_contract.rs` pass cleanly:
1. `test_ocrm_image_ops_build_resize_side_grid` — ok
2. `test_ocrm_image_ops_quality_steps_ladder` — ok
3. `test_ocrm_image_ops_animated_webp_detection` — ok
4. `test_ocrm_image_ops_still_png_and_apng_detection` — ok
5. `test_ocrm_image_ops_display_metadata_orientation_swap` — ok
6. `test_ocrm_image_ops_bmp_to_png_conversion_and_pixel_limit` — ok
7. `test_ocrm_image_ops_prepare_attachment` — ok
8. `test_ocrm_image_ops_normalize_anthropic_inline_image` — ok
9. `test_ocrm_image_ops_gateway_dispatch` — ok
