# OCRM Technical Dossier — Unit 21: Media Persistence, Workspace Scoping, Read Scope, and Storage Lifecycle (`OCRM-UNIT-MEDIA-008`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Media Storage Persistence, Read Scope Authority, Workspace Traversal Protection, Inbound/Outbound Lifecycle & Transcode Cache Pruning)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 11 core media store persistence, read scope authority, remote ingestion, test support, and test suite files (totaling 2,681 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/store.ts` | `d7006ace851e80e4c34bda6164a0fc0425237ba4` | 700 | 1..700 (full-file) | `OCRM-RESP-MEDIA-STORE-PERSIST-001`, `OCRM-RESP-MEDIA-STORE-LIFECYCLE-001` |
| `part-0042` | `src/media/store.read-scope.ts` | `d2989e690ec6c530f1355ba6b4f6c03ed59afb8d` | 281 | 1..281 (full-file) | `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001` |
| `part-0042` | `src/media/store.remote.runtime.ts` | `24cf3e17c222fc1befa6576334f80a19bbb7f5b7` | 50 | 1..50 (full-file) | `OCRM-RESP-MEDIA-STORE-REMOTE-001` |
| `part-0042` | `src/media/store-filename.test-support.ts` | `083a226c24dc7f91544314326da519fb8a65a259` | 27 | 1..27 (full-file) | `OCRM-RESP-MEDIA-STORE-TEST-SUPPORT-001` |
| `part-0042` | `src/media/store-network.test-support.ts` | `adc55afd5964323cfd2642a53dc15a002f307ba3` | 58 | 1..58 (full-file) | `OCRM-RESP-MEDIA-STORE-TEST-SUPPORT-001` |
| `part-0042` | `src/media/store.test.ts` | `9daf7bee5fc044bf23fb4e7ad4d29140269d1ee1` | 618 | 1..618 (full-file) | `OCRM-RESP-MEDIA-STORE-PERSIST-001`, `OCRM-RESP-MEDIA-STORE-LIFECYCLE-001` |
| `part-0042` | `src/media/store.read-scope.test.ts` | `e5ccd8d1b93526b6ae3981e6643660b0808a6481` | 438 | 1..438 (full-file) | `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001` |
| `part-0042` | `src/media/store.cleanup.test.ts` | `e872a1de80427849f9d65679844ee75dede10929` | 161 | 1..161 (full-file) | `OCRM-RESP-MEDIA-STORE-LIFECYCLE-001` |
| `part-0042` | `src/media/store.outside-workspace.test.ts` | `7eb52831b51db40242548f71ba4078f0fd03d22f` | 70 | 1..70 (full-file) | `OCRM-RESP-MEDIA-STORE-PERSIST-001` |
| `part-0042` | `src/media/store.redirect.test.ts` | `c16fc79aef29d3f51b0fbdb568144806530fc568` | 175 | 1..175 (full-file) | `OCRM-RESP-MEDIA-STORE-REMOTE-001` |
| `part-0042` | `src/media/store-network.test-support.test.ts` | `566ef2ada5d8ecaeddff7efbf4f94d475720e1e1` | 103 | 1..103 (full-file) | `OCRM-RESP-MEDIA-STORE-TEST-SUPPORT-001` |

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

The audited source code was decomposed into 5 discrete Rust runtime responsibilities registered in `parity-ledger.json`. Following rigorous scrutiny of actual production callers in `source/host`, all runtime responsibilities lacking production consumer pipelines are strictly marked `wiring=not-wired`, `verification=not-run`, `acceptance=pending` with explicit blocker IDs:

1. **`OCRM-RESP-MEDIA-STORE-PERSIST-001`**:
   - **Behavior**: Core media persistence engine providing atomic buffer, stream, and local file storage with UUID identifiers, original filename preservation (`extract_original_filename` / `---[uuid]` convention), NFC-normalized cross-platform filename sanitization, traversal protection, fractional size limits (`MEDIA_MAX_BYTES = 5MB` default), and atomic temp file creation with `MEDIA_FILE_MODE` (0o644).
   - **Rust Owner**: `source/host/src/extensions/attachments/media_store.rs::save_media_buffer, save_media_file, save_media_source, read_media_buffer, delete_media_buffer, resolve_media_buffer_path, extract_original_filename, sanitize_filename`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `media_store::save_media_buffer` / `media_store::read_media_buffer` → Consumer: *Pending inbound/outbound channel adapters and chat attachment ingestion pipelines*.
   - **Oracles**: `OCRM-MEDIA-STORE-O01`, `OCRM-MEDIA-STORE-O02`, `OCRM-MEDIA-STORE-O03`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-STORE-PERSIST-CONSUMERS-001"]` (inbound/outbound channel adapters and attachment ingestion pipelines do not yet route attachment storage through `media_store::save_media_buffer` / `media_store::save_media_source`).

2. **`OCRM-RESP-MEDIA-STORE-READ-SCOPE-001`**:
   - **Behavior**: Scoped media read authority and publication lifecycle, asserting directory identity and file handle integrity before commit and retaining descriptor lifecycle until host consumer acceptance.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_store.rs::resolve_media_scoped_dir, save_media_buffer, save_media_source`.
   - **Production Call Chain**: Producer `save_media_buffer, save_media_source` → Owner `media_store::resolve_media_scoped_dir` → Consumer: *Pending channel read authority scopes and session media receivers binding directory guard tokens*.
   - **Oracles**: `OCRM-MEDIA-STORE-O04`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-STORE-READ-SCOPE-CONSUMERS-001"]` (channel read authority scopes and session receivers do not yet bind directory guard tokens into `write_read_scope_media`).

3. **`OCRM-RESP-MEDIA-STORE-LIFECYCLE-001`**:
   - **Behavior**: Media storage lifecycle retention policies: playback transcode cache capped at 512MB disk budget with oldest-first LRU eviction and 7-day TTL; outbound staging with 24-hour TTL; cleanOldMedia routine with explicit exclusion of playback-transcode and SQLite-managed outgoing chat history.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_store.rs::write_playback_transcode_cache, prune_playback_transcode_cache, prune_playback_transcode_cache_to_size, prune_outbound_media, clean_old_media`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway("cleanOldMedia")` → Owner `media_store::clean_old_media` → Consumer: *Pending host periodic cleanup loop daemon and video playback rendition pipeline*.
   - **Oracles**: `OCRM-MEDIA-STORE-O05`, `OCRM-MEDIA-STORE-O06`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-STORE-LIFECYCLE-DAEMON-001"]` (host periodic cleanup daemon and video playback transcode pipeline do not yet invoke `clean_old_media` and `prune_playback_transcode_cache`).

4. **`OCRM-RESP-MEDIA-STORE-REMOTE-001`**:
   - **Behavior**: Remote media ingestion boundary for media store with 30-second bounded timeouts, 5 redirect limits, URL extension fallback without basename leakage, and SavedMedia result mapping.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_store.rs::save_remote_media_for_store`.
   - **Production Call Chain**: Producer `save_media_source` (when URL starts with `http://` or `https://`) → Owner `media_store::save_remote_media_for_store` → Consumer: *Pending external URL attachment ingestion in channel/web media pipelines*.
   - **Oracles**: `OCRM-MEDIA-STORE-O07`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-STORE-REMOTE-CONSUMERS-001"]` (channel/web media pipelines do not yet invoke remote URL media ingestion).

5. **`OCRM-RESP-MEDIA-STORE-TEST-SUPPORT-001`**:
   - **Behavior**: Scoped test fixtures, filename test assertion utilities, and network SSRF routing scoping for media store testing.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_store.rs::StoreRemoteFixtureScope`.
   - **Production Call Chain**: Producer `openclaw_media_store_contract` → Owner `media_store::StoreRemoteFixtureScope` → Consumer: `openclaw_media_store_contract` (*test-only harness boundary, no production runtime claim*).
   - **Oracles**: `OCRM-MEDIA-STORE-O08`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`, `blocker_ids=[]` (*support scope only*).

---

## 3. Unit 20 Blocker Status Re-verification & Honest Boundary Definition

- **`BL-MEDIA-STORE-SHARED-PIPELINE-001`**: `OCRM-RESP-MEDIA-STORE-SHARED-001` (`MEDIA_FILE_MODE`, `SaveMediaSourceError`, `format_media_size`) is implemented and consumed by `media_store.rs`, but because `media_store.rs` itself is not yet connected to external production channel consumers, `OCRM-RESP-MEDIA-STORE-SHARED-001` conservatively remains `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-STORE-SHARED-PIPELINE-001"]`.
- **`BL-MEDIA-QR-IMAGE-CALLER-001` & `BL-MEDIA-QR-TERMINAL-CALLER-001`**: `DevicePairingOwner::handle_pair_setup_code` generates `qrDataUrl` and `qrTerminal` in its RPC reply payload, but neither the CLI terminal command nor the desktop frontend React pairing dialog currently consumes and displays this output. Therefore, `OCRM-RESP-MEDIA-QR-IMAGE-001` and `OCRM-RESP-MEDIA-QR-TERMINAL-001` remain conservatively `wiring=not-wired`, `verification=not-run`, preserving their respective blocker IDs.

---

## 4. Strict Conservation Invariant Verification

- Previous mapped paths: **182**
- Newly mapped in Unit 21: **11**
- Updated mapped paths: **193**
- Previous unclassified paths: **58,568**
- Newly classified in Unit 21: **11**
- Updated unclassified paths: **58,557**
- Total tracked paths: **193 + 58,557 = 58,750** (strictly conserved, zero omitted, zero orphans)
- Total responsibilities: **119 + 5 = 124**
- Global completion flags: All 8 flags remain `false` pending final independent acceptance.

---

## 5. Verification & Contract Test Coverage

All contracts verified in `source/host/tests/openclaw_media_store_contract.rs` (9 passed, 0 failed):
- `test_ocrm_media_store_save_and_read_buffer`: Inbound media save, MIME detection, original filename prefixing (`---[uuid]`), buffer readback, and filename extraction.
- `test_ocrm_media_store_size_limit_rejection`: Enforces exact byte budget and reports fractional unit limits.
- `test_ocrm_media_store_filename_sanitization_and_truncation`: NFC normalization, cross-platform illegal character filtering, underscore collapsing, and 60 UTF-16 code unit truncation.
- `test_ocrm_media_store_subdir_traversal_protection`: Rejection of `..`, `\0`, absolute paths, slash/backslash escapes.
- `test_ocrm_media_store_local_file_source_boundaries`: Rejection of directories (`not-file`), missing files (`not-found`), symlinks (`invalid-path`), and paths outside workspace root.
- `test_ocrm_media_store_delete_and_lifecycle`: Media buffer deletion and file removal.
- `test_ocrm_media_store_playback_cache_and_clean_old_media`: Playback transcode cache LRU oldest-first eviction and exclusion of protected `playback-transcode` and `outgoing` subtrees from mtime sweep.
- `test_ocrm_media_store_gateway_service_dispatch`: End-to-end AttachmentsService dispatch of `saveMediaBuffer`, `readMediaBuffer`, `extractOriginalFilename`, and `deleteMediaBuffer`.
- `test_ocrm_device_pairing_qr_wiring`: Verifies that `DevicePairingOwner::handle_pair_setup_code` produces `qrDataUrl` and `qrTerminal`.
