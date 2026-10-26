# OCRM Technical Dossier — Unit 16: Media Understanding Runtime, Transports & STT Artifacts (`OCRM-UNIT-MEDIA-003`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Media Understanding Runtime, Shared Transports, STT Artifact Filters & Decision Types)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 core media understanding apply, cache, runtime, shared transport, artifact filter, and types files (totaling 2,111 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media-understanding/apply.ts` | `bd8c0fd0d1efa78cb0d0a0f30ca41ede8f3fbb04` | 344 | 1..344 (full-file) | `OCRM-RESP-MEDIA-APPLY-UNDERSTANDING-001` |
| `part-0042` | `src/media-understanding/attachments.cache.ts` | `987e8a1bc3fb8e8276bf8e99b0d481b67acd9f45` | 550 | 1..550 (full-file) | `OCRM-RESP-MEDIA-ATTACHMENTS-CACHE-001` |
| `part-0042` | `src/media-understanding/runtime.ts` | `d9e792decf4b56f24fbc16fa6b8ceb7a5d4e8869` | 387 | 1..387 (full-file) | `OCRM-RESP-MEDIA-UNDERSTANDING-RUNTIME-001` |
| `part-0042` | `src/media-understanding/shared.ts` | `c95cdc7df6b3084f8d5635fc018c8998d09b4ff7` | 587 | 1..587 (full-file) | `OCRM-RESP-MEDIA-SHARED-TRANSPORT-001` |
| `part-0042` | `src/media-understanding/transcription-text.ts` | `dab43b64e0a188cc8562f910c9ce819cc8266750` | 11 | 1..11 (full-file) | `OCRM-RESP-MEDIA-TRANSCRIPTION-ARTIFACTS-001` |
| `part-0042` | `src/media-understanding/types.ts` | `ec438ecdf26ee1574499cdf00243c7ba06a3b63f` | 232 | 1..232 (full-file) | `OCRM-RESP-MEDIA-UNDERSTANDING-TYPES-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-MEDIA-APPLY-UNDERSTANDING-001`**:
   - **Behavior**: Application of media understanding across attachments in capability order (image, audio, video), voice note placeholder handling (`[Voice note could not be transcribed because the audio attachment was too small]`), and body block concatenation.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_understanding_service.rs::append_file_blocks`.
   - **Production Call Chain**: Producer `append_file_blocks` → Consumer `source/host/src/extensions/transcript/production_runtime.rs`.
   - **Oracles**: `OCRM-MEDIA-UND-O04`, `OCRM-MEDIA-UND-O05`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

2. **`OCRM-RESP-MEDIA-ATTACHMENTS-CACHE-001`**:
   - **Behavior**: Inbound media reference source classification (`media://` local store URLs vs remote fetch), maxBytes bounds checking, and fetch retry policies.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_understanding_service.rs::is_media_store_url`.
   - **Production Call Chain**: Producer `attachments_service.rs::upload` → Owner `is_media_store_url` → Consumer `attachments_service.rs::read_attachment_text_with_root`.
   - **Oracle**: `OCRM-MEDIA-UND-O07`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

3. **`OCRM-RESP-MEDIA-UNDERSTANDING-RUNTIME-001`**:
   - **Behavior**: Runtime dispatch for image description, structured extraction, video description, and audio input budgeting.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_understanding_service.rs::AudioInputBudget`.
   - **Production Call Chain**: Producer `attachments_service.rs::upload` → Owner `AudioInputBudget` → Consumer `attachments_service.rs`.
   - **Oracle**: `OCRM-MEDIA-UND-O06`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

4. **`OCRM-RESP-MEDIA-SHARED-TRANSPORT-001`**:
   - **Behavior**: Shared media transport utilities: OpenAI-compatible auth header generation, audio transcription upload filename mapping (AAC to `.m4a`), and multipart form data building.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_understanding_service.rs::resolve_audio_transcription_upload_file_name`.
   - **Production Call Chain**: Producer `attachments_service.rs::upload` → Owner `resolve_audio_transcription_upload_file_name` → Consumer `attachments_service.rs`.
   - **Oracles**: `OCRM-MEDIA-UND-O02`, `OCRM-MEDIA-UND-O03`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

5. **`OCRM-RESP-MEDIA-TRANSCRIPTION-ARTIFACTS-001`**:
   - **Behavior**: Detection and suppression of empty STT prompt artifact echoes (`context:`, `###`, `transcribe the audio`) from transcript entries and preview pipelines.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_understanding_service.rs::is_transcript_artifact_text`.
   - **Production Call Chain**: Producer `attachments_service.rs::read_attachment_text_with_root` → Owner `is_transcript_artifact_text` → Consumer `read_attachment_text_with_root`.
   - **Oracles**: `OCRM-MEDIA-UND-O01`, `OCRM-MEDIA-UND-O08`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

6. **`OCRM-RESP-MEDIA-UNDERSTANDING-TYPES-001`**:
   - **Behavior**: Typed data contracts for media understanding decisions, attachment dispositions, processing states, and model request structures.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_understanding_service.rs`.
   - **Production Call Chain**: Producer `media_understanding_service.rs` → Consumer `attachments_service.rs`.
   - **Oracle**: `OCRM-MEDIA-UND-O04`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

---

## 3. Falsifiable Test Oracles (`source/host/tests/openclaw_media_understanding_contract.rs`)

1. **`OCRM-MEDIA-UND-O01`**: `test_ocrm_media_und_o01_stt_transcript_artifact_rejection` (verifies detection of STT echoes like `###`, `context:`, `transcribe the audio`).
2. **`OCRM-MEDIA-UND-O02`**: `test_ocrm_media_und_o02_audio_transcription_upload_filename` (verifies `.aac` mapping to `.m4a`, audio/aac mime mapping, and original filename retention).
3. **`OCRM-MEDIA-UND-O03`**: `test_ocrm_media_und_o03_openai_auth_headers` (verifies `Bearer <key>` construction and empty key suppression).
4. **`OCRM-MEDIA-UND-O04`**: `test_ocrm_media_und_o04_capability_order_and_voice_note_placeholder` (verifies capability order `["image", "audio", "video"]` and voice note placeholder string).
5. **`OCRM-MEDIA-UND-O05`**: `test_ocrm_media_und_o05_append_file_blocks` (verifies body and suffix block concatenation with double newlines).
6. **`OCRM-MEDIA-UND-O06`**: `test_ocrm_media_und_o06_audio_input_budget` (verifies payload bounds validation against 25 MiB cap).
7. **`OCRM-MEDIA-UND-O07`**: `test_ocrm_media_und_o07_media_store_url_classification` (verifies `media://` classification as local store URL).
8. **`OCRM-MEDIA-UND-O08`**: `test_ocrm_media_und_o08_attachment_service_artifact_text_suppression` (verifies `read_attachment_text_with_root` suppresses STT artifact echo files).

---

## 4. Invariant and Metric Ledger

- **Inventory Tracking**:
  - `mapped_paths`: 154
  - `unclassified_paths`: 58,596
  - `tracked_paths`: 58,750 (strictly conserved: $154 + 58,596 = 58,750$)
  - `omitted_known`: 0
- **Responsibilities**:
  - Total: 96
  - Active blockers: `BL-CURSOR-BACKEND-IMAGE-RESOLUTION-001`
- **Global Completion Flags**:
  - All 8 flags remain `false`.
