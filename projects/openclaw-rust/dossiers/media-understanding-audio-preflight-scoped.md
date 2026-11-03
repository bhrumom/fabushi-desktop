# OCRM Technical Dossier — Unit 30: Media Understanding Audio Transcription Preflight & Echo Formatting (`OCRM-UNIT-MEDIA-UND-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Media Understanding Audio Transcription Preflight, STT Prompt Echo Rejection, Upload Filename Normalization & Deliverable Channel Echoes)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 media understanding files (totaling 778 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media-understanding/echo-transcript.ts` | `20891acad1523385cc8d5208ea02d09ab7a84cc4` | 75 | 1..75 (full-file) | `OCRM-RESP-MEDIA-UND-AUDIO-PREFLIGHT-001` |
| `part-0042` | `src/media-understanding/echo-transcript.test.ts` | `0b87e6a7c50790fa6b25547539e11c57e2e605b2` | 96 | 1..96 (full-file) | `OCRM-RESP-MEDIA-UND-AUDIO-PREFLIGHT-001` |
| `part-0042` | `src/media-understanding/audio-preflight.ts` | `e47c017541f391f392ba424cc0d8f283bce2d86b` | 138 | 1..138 (full-file) | `OCRM-RESP-MEDIA-UND-AUDIO-PREFLIGHT-001` |
| `part-0042` | `src/media-understanding/audio-preflight.test.ts` | `65954401ce2cb6a06a92a71e2a1fcaeca9678b49` | 99 | 1..99 (full-file) | `OCRM-RESP-MEDIA-UND-AUDIO-PREFLIGHT-001` |
| `part-0042` | `src/media-understanding/audio-preflight.attachments.test.ts` | `b1c32ffc83097ec0f1388fbbbc45ba23dc49bd36` | 287 | 1..287 (full-file) | `OCRM-RESP-MEDIA-UND-AUDIO-PREFLIGHT-001` |
| `part-0042` | `src/media-understanding/audio.test-helpers.ts` | `86aa24e3039714c05ae3be0d1540821a29272122` | 83 | 1..83 (full-file) | `OCRM-RESP-MEDIA-UND-AUDIO-PREFLIGHT-001` |

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

1. **`OCRM-RESP-MEDIA-UND-AUDIO-PREFLIGHT-001` (NEW)**:
   - **Behavior**: Voice note preflight transcription before mention checking (enabling voice messages in group chats requiring mentions), STT prompt artifact echo suppression, upload filename mapping (AAC to `.m4a`), and transcript echo formatting (`DEFAULT_ECHO_TRANSCRIPT_FORMAT = '📝 "{transcript}"'`). Preserves literal `$` sequences in transcripts without template corruption. Evaluates deliverable message channels (`is_deliverable_message_channel`) and safely skips echo delivery when target channel or recipient is missing or non-deliverable.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_understanding_service.rs`.
   - **Production Call Chain**: Producer `AutoReplyRouter` / inbound message dispatcher → Owner `media_understanding_service` → Consumer: *Voice note preflight transcript injector and chat echo sender*.
   - **Oracle**: `OCRM-MEDIA-UND-O01`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-UND-AUDIO-PREFLIGHT-CHAT-WIRING-001"]`.

---

## 3. Strict Conservation Invariant Verification

- **Total tracked repository paths**: **58,750** (strictly conserved, zero omitted/untracked paths).
- **Cumulative mapped paths**: **270** (increased by +6 from 264; unclassified reduced from 58,486 to 58,480).
- **Active tracked responsibilities**: **140** (increased by +1 from 139).
- **8 Global Completion Flags**: All remain `false` pending full migration and verified evidence gates.

---

## 4. Contract Test Verification

Contract tests in `source/host/tests/openclaw_media_understanding_contract.rs` verify all core preflight invariants:
- `test_ocrm_media_und_o01_stt_transcript_artifact_rejection`: Validates prompt echo artifacts (`"context:"`, `"###"`, `"transcribe the audio"`) are rejected while genuine speech is accepted.
- `test_ocrm_media_und_o02_audio_transcription_upload_filename`: Validates `.aac` and extensionless files with `audio/aac` MIME are correctly mapped to `.m4a` for provider compatibility.
- `test_ocrm_media_und_o09_echo_transcript_formatting`: Validates default and custom template formatting, guaranteeing literal `$` preservation.
- `test_ocrm_media_und_o10_echo_transcript_channel_deliverability`: Validates channel deliverability rules and skip conditions.
- **Result**: 10/10 tests passing in `openclaw_media_understanding_contract.rs`.
