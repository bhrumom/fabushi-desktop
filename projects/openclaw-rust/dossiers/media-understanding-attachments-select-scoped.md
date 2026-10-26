# OCRM Technical Dossier — Unit 31: Media Understanding Attachment Normalization, Path Guards & Capability Selection (`OCRM-UNIT-MEDIA-UND-002`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Media Understanding Attachment Normalization, UNC Path Traversal Guards, Capability-Based Selection, Ordering Policies & Index Accounting)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 7 media understanding attachment files (totaling 1,000 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media-understanding/attachments.ts` | `8c06a0b3fedb50767c51c83643a0c9cb1a4afca2` | 8 | 1..8 (full-file) | `OCRM-RESP-MEDIA-UND-ATTACHMENTS-SELECT-001` |
| `part-0042` | `src/media-understanding/attachments.normalize.ts` | `ee275a078707defe401671e8fe648f6c64ced6b9` | 74 | 1..74 (full-file) | `OCRM-RESP-MEDIA-UND-ATTACHMENTS-SELECT-001` |
| `part-0042` | `src/media-understanding/attachments.normalize.test.ts` | `ac3e65bfa28be9b6b298aeb948f2fb7cb4320323` | 72 | 1..72 (full-file) | `OCRM-RESP-MEDIA-UND-ATTACHMENTS-SELECT-001` |
| `part-0042` | `src/media-understanding/attachments.select.ts` | `dce24589afd191f8a1775b9874fa18c7e8b6fea9` | 81 | 1..81 (full-file) | `OCRM-RESP-MEDIA-UND-ATTACHMENTS-SELECT-001` |
| `part-0042` | `src/media-understanding/attachments.guards.test.ts` | `88ac641ad0ecd1ff3d20bc27d444a9d46542cb6c` | 85 | 1..85 (full-file) | `OCRM-RESP-MEDIA-UND-ATTACHMENTS-SELECT-001` |
| `part-0042` | `src/media-understanding/attachments.file-url.windows.test.ts` | `bfa3b593e050f8ff35188d6808ec2aa0bea95213` | 40 | 1..40 (full-file) | `OCRM-RESP-MEDIA-UND-ATTACHMENTS-SELECT-001` |
| `part-0042` | `src/media-understanding/attachments.cache.test.ts` | `a7c310e4ab4e38d27df972a45a5a6161a90ccb94` | 635 | 1..635 (full-file) | `OCRM-RESP-MEDIA-ATTACHMENTS-CACHE-001` |

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

1. **`OCRM-RESP-MEDIA-UND-ATTACHMENTS-SELECT-001` (NEW)**:
   - **Behavior**: Attachment path normalization (`normalize_attachment_path`) rejecting remote/UNC paths (`\\server\share`, `//server/share`), media kind resolution (`resolve_attachment_media_kind`, mapping stickers to image and documents to unknown), capability attachment filtering, and preference-ordered selection (`select_attachments`). Supports preference modes (`first`, `last`, `path`, `url`), max limit clamping (`mode=all` vs `mode=first`), dropped index tracking, and audio deduplication (skipping already transcribed items and previously decided audio indexes).
   - **Rust Owner**: `source/host/src/extensions/attachments/media_understanding_service.rs`.
   - **Production Call Chain**: Producer `MediaUnderstandingService::run_capability` → Owner `media_understanding_service` → Consumer: *Per-capability attachment runner and decision synthesizer*.
   - **Oracle**: `OCRM-MEDIA-UND-O13`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-UND-ATTACHMENTS-SELECT-CHAT-WIRING-001"]`.

2. **`OCRM-RESP-MEDIA-ATTACHMENTS-CACHE-001` (EXPANDED TEST COVERAGE)**:
   - **Behavior**: Inbound media reference source classification and local store caching.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_understanding_service.rs::is_media_store_url`.
   - **Source Additions**: Associated comprehensive unit test suite `src/media-understanding/attachments.cache.test.ts` (635 lines).
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-UNDERSTANDING-UPSTREAM-CALLER-001"]`.

---

## 3. Strict Conservation Invariant Verification

- **Total tracked repository paths**: **58,750** (strictly conserved, zero omitted/untracked paths).
- **Cumulative mapped paths**: **277** (increased by +7 from 270; unclassified reduced from 58,480 to 58,473).
- **Active tracked responsibilities**: **141** (increased by +1 from 140).
- **8 Global Completion Flags**: All remain `false` pending full migration and verified evidence gates.

---

## 4. Contract Test Verification

Contract tests in `source/host/tests/openclaw_media_understanding_contract.rs` verify all core normalization and selection invariants:
- `test_ocrm_media_und_o11_attachment_path_normalization_and_guards`: Validates rejection of Windows UNC network paths (`\\server\share`, `//server/share`), safe decoding of `file://` URLs, and whitespace rejection.
- `test_ocrm_media_und_o12_attachment_kind_resolution`: Validates precedence of authoritative kind over MIME and extension (including sticker-to-image coercion), MIME fallback, and extension fallback.
- `test_ocrm_media_und_o13_attachment_selection_policy_ordering_and_limits`: Validates `First` vs `All` modes, `Last` preference reversal, `Path` vs `Url` prioritization, dropped index accounting, and audio preflight deduplication.
- **Result**: 13/13 tests passing in `openclaw_media_understanding_contract.rs`.
