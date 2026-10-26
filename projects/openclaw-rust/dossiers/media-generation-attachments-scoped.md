# OCRM Technical Dossier — Unit 13: Media Understanding & Generation (`OCRM-UNIT-MEDIA-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M23/M24 (Media Understanding, Attachments, Image Generation & Assets Subsystems)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 core media caps, generated assets, resolution normalization, local access containment, default model resolution, and base64 sniffing files (totaling 904 lines) were audited line-by-line and mapped into canonical shards:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/configured-max-bytes.ts` | `0a0c31d1017e765bc1bd399fc7179e1aca4e97f8` | 96 | 1..96 (full-file) | `OCRM-RESP-MEDIA-BYTE-CAPS-001` |
| `part-0039` | `src/image-generation/image-assets.ts` | `ec73319b6e70662f7b537ae720ce3f4f0b3512a1` | 275 | 1..275 (full-file) | `OCRM-RESP-MEDIA-IMAGE-ASSETS-001` |
| `part-0039` | `src/image-generation/resolution.ts` | `d5d6f12047a9fce50aecb340f30fff3d99c10fe7` | 15 | 1..15 (full-file) | `OCRM-RESP-MEDIA-RESOLUTION-001` |
| `part-0042` | `src/media/local-media-access.ts` | `960de1cd63cb989efba972c7ef8bc9c78e6c1cdc` | 290 | 1..290 (full-file) | `OCRM-RESP-MEDIA-LOCAL-ACCESS-001` |
| `part-0042` | `src/media-understanding/defaults.ts` | `d3ea82eeff26e5f8ca2cb529d1d3b5240b55239f` | 193 | 1..193 (full-file) | `OCRM-RESP-MEDIA-DEFAULT-MODELS-001` |
| `part-0042` | `src/media/sniff-mime-from-base64.ts` | `109a3e78d1fbadb2c3bec859b9cf8d4264693b4f` | 29 | 1..29 (full-file) | `OCRM-RESP-MEDIA-SNIFF-PREVIEW-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-MEDIA-BYTE-CAPS-001`**:
   - **Behavior**: Enforces `ATTACHMENT_BYTE_LIMIT` (25 MiB), `VIDEO_BYTE_LIMIT` (200 MiB), `resolveGeneratedMediaMaxBytes`, and channel/account media caps.
   - **Rust Owner**: `source/host/src/media_mime.rs:ATTACHMENT_BYTE_LIMIT`.
   - **Oracles**: `OCRM-ATTACH-O01`, `OCRM-ATTACH-O02`.

2. **`OCRM-RESP-MEDIA-IMAGE-ASSETS-001`**:
   - **Behavior**: Generated image assets management, magic-byte sniffing (PNG, JPEG, WEBP), data URL generation, and standard Base64 roundtrip verification.
   - **Rust Owner**: `source/host/src/extensions/attachments/generate_image_service.rs:SandGenerateImageService`.
   - **Oracles**: `OCRM-ATTACH-O05`, `OCRM-ATTACH-O06`, `OCRM-ATTACH-O07`.

3. **`OCRM-RESP-MEDIA-RESOLUTION-001`**:
   - **Behavior**: Image generation resolution normalization (1K, 2K, 4K), invalid resolution rejection (`ToolInputError`), tool bridge argument validation, and propagation of `GenerateImageToolOptions` with resolution to `GenerateImageToolExecutor`.
   - **Rust Owner**: `source/host/src/runner/tools/sand_generate_image_tool.rs:normalize_image_generation_resolution, SandGenerateImageToolBridge`.
   - **Production Call Chain**: Producer `SandGenerateImageToolBridge::call_tool` → Owner `normalize_image_generation_resolution` → Options `GenerateImageToolOptions` → Consumer `ProductionGenerateImageToolExecutor::generate_with_options` → Storage `/workspace/assets/{filename}`.
   - **Oracles**: `OCRM-ATTACH-O08` (positive/negative normalization), `OCRM-ATTACH-O09` (invalid rejection + executor options delivery verification).
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

4. **`OCRM-RESP-MEDIA-LOCAL-ACCESS-001`**:
   - **Behavior**: Local media filesystem access boundary, canonical boundary path resolution (`resolveCanonicalBoundaryPath`), directory traversal prevention, and assets isolation.
   - **Rust Owner**: `source/host/src/extensions/attachments/generate_image_resource_accessor.rs:SandGenerateImageResourceAccessor`.
   - **Oracle**: `OCRM-ATTACH-O03`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

5. **`OCRM-RESP-MEDIA-DEFAULT-MODELS-001`**:
   - **Behavior**: Media understanding default provider and model resolution (`resolve_default_media_model`), vision/audio capability registration, configured model retrieval, and auto-discovery provider priority sorting (`resolve_auto_media_key_providers`).
   - **Rust Owner**: `source/host/src/media_understanding_defaults.rs:resolve_default_media_model, resolve_auto_media_key_providers`.
   - **Wiring Status**: Honest calibration: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (dynamic plugin provider metadata resolution implemented, pending live production model router dispatch integration).
   - **Oracle**: `OCRM-ATTACH-O04`.

6. **`OCRM-RESP-MEDIA-SNIFF-PREVIEW-001`**:
   - **Behavior**: Full-payload canonical Base64 inspection (canonicalPadBits & whitespace tolerance), prefix-bounded MIME detection with hint precedence (MimeSniffHints), attachment text preview classification with filesystem truncation (>64 KiB) vs binary detection, and production wiring into attachment upload and inline image materialization.
   - **Rust Owner**: `source/host/src/media_mime.rs:sniff_mime_from_base64_with_hints, source/host/src/media_mime.rs:inspect_canonical_base64` and `source/host/src/extensions/attachments/attachments_service.rs:AttachmentTextPreview, read_attachment_text_with_root`.
   - **Production Call Chain**: Producer `attachments_service.rs::upload, inline_image_materialization.rs::materialize_one` → Owner `media_mime.rs::inspect_canonical_base64, sniff_mime_from_base64_with_hints` → Storage `ingest_attachment_bytes` → Consumer `attachments_service.rs::read_text`.
   - **Oracles**: `OCRM-ATTACH-O10` (tail corruption, non-canonical pad bits, hints fallback & conflict), `OCRM-ATTACH-O11` (real filesystem text truncation & binary discrimination), `OCRM-ATTACH-O12` (extension dependency graph).
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

---

## 3. Falsifiable Test Oracles (`source/host/tests/openclaw_media_generation_contract.rs`)

1. **`OCRM-ATTACH-O01`**: `test_ocrm_attach_o01_byte_limits`
2. **`OCRM-ATTACH-O02`**: `test_ocrm_attach_o02_directory_names`
3. **`OCRM-ATTACH-O03`**: `test_ocrm_attach_o03_resource_accessor_containment`
4. **`OCRM-ATTACH-O04`**: `test_ocrm_attach_o04_media_understanding_default_models`
5. **`OCRM-ATTACH-O05`**: `test_ocrm_attach_o05_generation_and_persistence`
6. **`OCRM-ATTACH-O06`**: `test_ocrm_attach_o06_persistence_failure`
7. **`OCRM-ATTACH-O07`**: `test_ocrm_attach_o07_invalid_base64`
8. **`OCRM-ATTACH-O08`**: `test_ocrm_attach_o08_resolution_normalization`
9. **`OCRM-ATTACH-O09`**: `test_ocrm_attach_o09_tool_resolution_validation_and_rejection`
10. **`OCRM-ATTACH-O10`**: `test_ocrm_attach_o10_sniff_mime_from_base64`
11. **`OCRM-ATTACH-O11`**: `test_ocrm_attach_o11_attachment_text_preview_truncation_and_binary`
12. **`OCRM-ATTACH-O12`**: `test_ocrm_attach_o12_extension_id_and_dependencies`

---

## 4. Invariant Verification & Global Conservation

- **Exact Mapped Paths**: 136 (cumulative across Units 1–13)
- **Omitted Known**: 0
- **Total Tracked Paths**: 58,750
- **Unread / Unclassified Paths**: 58,614
- **Strict Conservation**: $136 + 58614 = 58750$.
- **All Global Completion Flags**: `false` strictly preserved.
