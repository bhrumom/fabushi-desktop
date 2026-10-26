# OCRM Technical Dossier — Unit 17: Media References, Inbound URIs, Playback Codec Policy & Facts (`OCRM-UNIT-MEDIA-004`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Media References, Inbound URIs, Playback Codec Policies, FFprobe Metadata & Facts)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 core media reference, bounded reader, playback codec policy, probe, and facts files (totaling 1,217 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/bounded-read-file.ts` | `c6761090ffa01740cf3b4b9341f8d50062126517` | 34 | 1..34 (full-file) | `OCRM-RESP-MEDIA-BOUNDED-READ-001` |
| `part-0042` | `src/media/inbound-media-uri.ts` | `47da2a2bf0f5d6e27bcf7cee369584e064dca072` | 73 | 1..73 (full-file) | `OCRM-RESP-MEDIA-INBOUND-URI-001` |
| `part-0042` | `src/media/media-facts.ts` | `f1df5c9b2899a5e82f15aab30a7ff233e25c966a` | 513 | 1..513 (full-file) | `OCRM-RESP-MEDIA-FACTS-001` |
| `part-0042` | `src/media/media-probe.ts` | `aca3c74ebabd4c1ca6aab6f1d6c942ef2e892d5b` | 260 | 1..260 (full-file) | `OCRM-RESP-MEDIA-PROBE-METADATA-001` |
| `part-0042` | `src/media/media-reference.ts` | `a75dbbe5d16aff5b4860a2e22e2ea1b86ab60502` | 188 | 1..188 (full-file) | `OCRM-RESP-MEDIA-REFERENCE-RESOLUTION-001` |
| `part-0042` | `src/media/playback-codec-policy.ts` | `3bc97dbffc68d4f2763b7d3759abe6ff7197c5cc` | 149 | 1..149 (full-file) | `OCRM-RESP-PLAYBACK-CODEC-POLICY-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-MEDIA-BOUNDED-READ-001`**:
   - **Behavior**: Bounded outbound media file reading with max bytes enforcement and size bounds protection.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_reference.rs::read_outbound_media_file_bounded`.
   - **Production Call Chain**: Producer `read_host_attachment_video_bytes_with_root` / `read_host_attachment_image_with_root` → `AttachmentsService` → Consumer.
   - **Oracles**: `OCRM-MEDIA-REF-O04`.

2. **`OCRM-RESP-MEDIA-INBOUND-URI-001`**:
   - **Behavior**: Inbound media URI parsing, hostname restriction (`inbound`), path traversal prevention, and `MEDIA:` prefix stripping.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_reference.rs::parse_inbound_media_uri`.
   - **Production Call Chain**: Inbound URI string → `AttachmentsService::parse_inbound_media_uri` → storage resolver.
   - **Oracles**: `OCRM-MEDIA-REF-O01`.

3. **`OCRM-RESP-MEDIA-REFERENCE-RESOLUTION-001`**:
   - **Behavior**: Media reference scheme classification, sandbox path rewrite (`media/inbound/<id>`), and local store resolution.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_reference.rs::classify_media_reference_source`.
   - **Production Call Chain**: Staged attachment inputs → `resolve_media_reference_sandbox_path` → Box Staging.
   - **Oracles**: `OCRM-MEDIA-REF-O02`, `OCRM-MEDIA-REF-O03`.

4. **`OCRM-RESP-PLAYBACK-CODEC-POLICY-001`**:
   - **Behavior**: Cross-client audio/video playback transcode policy and native codec compatibility resolution.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_probe.rs::get_playback_policy`.
   - **Production Call Chain**: Producer `create_video_playback_source_with` → `resolve_native_playback_codec_compatibility` → Transcode Pipeline.
   - **Oracles**: `OCRM-MEDIA-REF-O05`, `OCRM-MEDIA-REF-O06`.

5. **`OCRM-RESP-MEDIA-PROBE-METADATA-001`**:
   - **Behavior**: FFprobe JSON output parsing, duration resolution, SAR scaling, and 90/270 rotation axis swapping.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_probe.rs::parse_ffprobe_media_metadata`.
   - **Production Call Chain**: FFprobe process stdout → `to_media_probe_result` → Display Dimensions & Codec Compatibility.
   - **Oracles**: `OCRM-MEDIA-REF-O07`.

6. **`OCRM-RESP-MEDIA-FACTS-001`**:
   - **Behavior**: Persisted and runtime media facts normalization, carrier canonicalization into `__openclaw.media`, and legacy projections.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_facts.rs::canonicalize_persisted_user_message_media`.
   - **Production Call Chain**: User transcript message JSON → `AttachmentsService::canonicalize_user_message_media` → Store.
   - **Oracles**: `OCRM-MEDIA-REF-O08`.

---

## 3. Verification & Evidence

- **Contract Test Suite**: `source/host/tests/openclaw_media_reference_contract.rs`
  - 8 passing test cases:
    - `test_ocrm_media_ref_01_inbound_uri_parsing_and_validation`: verifies `media://inbound/<id>` parsing, rejection of non-inbound hosts, rejection of traversal (`..`, `\`), rejection of search/hash/auth.
    - `test_ocrm_media_ref_02_classify_media_reference_source`: verifies Windows drive paths, HTTP URLs, data URLs, media store URLs, and unsupported schemes.
    - `test_ocrm_media_ref_03_inbound_path_resolution_and_sandbox_rewrite`: verifies managed inbound path URI generation, sandbox-relative rewriting, and local path info resolution.
    - `test_ocrm_media_ref_04_bounded_read_file`: verifies size limits enforcement before buffer allocation.
    - `test_ocrm_media_ref_05_playback_codec_policy_resolution`: verifies playback mode resolution (Native vs Transcode) and format mapping for audio and video.
    - `test_ocrm_media_ref_06_native_codec_compatibility_rules`: verifies WAV PCM requirement, H.264 profile/pix_fmt compatibility, and HEVC transcode requirement.
    - `test_ocrm_media_ref_07_probe_metadata_aspect_ratio_and_rotation`: verifies ffprobe JSON parsing, duration, SAR aspect ratio scaling, and 90-degree axis rotation swap.
    - `test_ocrm_media_ref_08_canonicalize_persisted_user_message_media`: verifies migration of legacy keys (`MediaPath`, `MediaType`, `MediaWorkspaceDir`) to `__openclaw.media` and legacy channel projections.
- **Regression Tests**: All 44 contract tests in `openclaw_crash_guard_media_contract`, `openclaw_media_generation_contract`, `openclaw_agent_roster_contract`, `openclaw_media_understanding_contract`, and `video_playback_rendition_contract` pass with 0 failures.
- **Strict Conservation Invariant**:
  - `mapped_paths`: 160
  - `unclassified_paths`: 58,590
  - `tracked_paths`: 58,750 ($160 + 58,590 = 58,750$)
  - `responsibilities`: 102
