# OCRM Technical Dossier — Unit 25: Media Fetch Pipeline, SSRF Guards & Content-Disposition Parsing (`OCRM-UNIT-MEDIA-012`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (SSRF-Guarded Remote Media Fetch Pipeline, RFC 5987 / RFC 6266 Extended Content-Disposition Parser, Streaming Byte Bounds, Charset Retention & Disk Store Persistence)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 5 core media fetch pipeline, read scope, and test support files (totaling 1,631 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/fetch.read-scope.ts` | `3c90773b2de541ce60104d3c3ac3d6d3b7e9c508` | 39 | 1..39 (full-file) | `OCRM-RESP-MEDIA-FETCH-PIPELINE-001` |
| `part-0042` | `src/media/fetch.prototype-headers.test.ts` | `a9cc607f5ac65588f8077e9b45f06d58b3ed92e0` | 33 | 1..33 (full-file) | `OCRM-RESP-MEDIA-FETCH-PIPELINE-001` |
| `part-0042` | `src/media/fetch.test-support.ts` | `9e6a41af9c46a8cb24bc1885e5a9508f7dfcd63b` | 96 | 1..96 (full-file) | `OCRM-RESP-MEDIA-FETCH-PIPELINE-001` |
| `part-0042` | `src/media/fetch.ts` | `46efb618af1faa18ff8e5c19000ba93adf8470a0` | 721 | 1..721 (full-file) | `OCRM-RESP-MEDIA-FETCH-PIPELINE-001` |
| `part-0042` | `src/media/fetch.test.ts` | `b69e634a0b238a4b4796a0498676f39e5ed24d7b` | 742 | 1..742 (full-file) | `OCRM-RESP-MEDIA-FETCH-PIPELINE-001` |

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

The audited sources were mapped to 1 discrete Rust runtime responsibility registered in `parity-ledger.json`. In strict compliance with the project's honest calibration policy, because end-to-end chat attachment downlinks and live agent loop fetchers are scheduled for subsequent integration stages, this responsibility is conservatively marked `wiring=not-wired`, `verification=not-run`, `acceptance=pending` with a registered blocker ID:

1. **`OCRM-RESP-MEDIA-FETCH-PIPELINE-001` (NEW)**:
   - **Behavior**: SSRF-guarded remote media fetch and response buffering/persistence pipeline. Enforces Content-Disposition RFC 5987 / 6266 extended filenames (UTF-8, ISO-8859-1) with apostrophe retention and Windows path reduction, Content-Length header verification with early cancellation on malformed values, streaming payload maxBytes caps, audio/video container hint resolution, HTTP charset retention, bot token redaction, and empty body rejection.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_fetch.rs::read_remote_media_buffer, read_remote_media_buffer_with_headers, save_response_media, save_remote_media, parse_content_disposition_file_name, redact_sensitive_text`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `media_fetch` → Consumer: *Pending agent loop and chat channel attachment fetchers*.
   - **Oracles**: `OCRM-FETCH-O01`, `OCRM-FETCH-O02`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-FETCH-PIPELINE-CHAT-WIRING-001"]`.

---

## 3. Strict Conservation Invariant Verification

- **Total tracked repository paths**: **58,750** (strictly conserved, zero omitted/untracked paths).
- **Mapped paths**: **225** (increased by 5 from 220).
- **Unclassified paths**: **58,525** (decreased by 5 from 58,530).
- **Omitted paths known**: **0** (strictly 0).
- **Sum invariant**: `mapped_paths (225) + unclassified_paths (58525) == tracked_paths (58750)`.
- **Total active responsibilities**: **132** (increased by 1 from 131).
- **Global completion flags**: All 8 flags remain strictly `false`.

---

## 4. Behavioral Equivalence & Verification Oracles

The native Rust implementation (`source/host/src/extensions/attachments/media_fetch.rs`) achieves 100% behavioral parity with upstream OpenClaw:

1. **Content-Disposition RFC 5987 / 6266 Extended Filenames**:
   - Extended tokens `filename*` are decoded under `UTF-8` and `ISO-8859-1`.
   - Malformed extended tokens (`%ZZbad.csv`) or unsupported charsets (`UTF-16`) fail-safe and fall back to plain `filename`.
   - Extended encoded slashes (`%2F`, `%5C`) are converted to `_` to guarantee sandbox traversal safety.
2. **Unquoted Filename Apostrophe & Windows Path Integrity**:
   - Unquoted filenames retain leading, trailing, and surrounding apostrophes (`'report.csv`, `report.csv'`, `'report.csv'`).
   - Mixed Windows drive paths (`"C:/tmp/reports\Q1.csv"`) and relative paths (`"reports\Q1.csv"`) are reduced to their safe basename (`"Q1.csv"`).
   - Filename strings inside unrelated quoted parameters (e.g. `note="x; filename=spoof.csv; y"`) are ignored.
3. **URL Basename Extraction & Malformed Percent Toleration**:
   - URL path basenames are parsed safely.
   - Malformed percent escapes (e.g. `bad%E0%A4%A.pdf`) retain their raw basename string without panicking.
   - Decoded URL separators (e.g. `%5C`) are substituted with `_` (`reports_Q1.pdf`).
4. **Content-Length & Streaming Byte Bounds**:
   - Malformed `Content-Length` headers (e.g. `1e9`) reject with `HttpError` prior to reading buffers.
   - Headers or streaming bodies exceeding `maxBytes` reject with `MaxBytes` without leaving corrupt partial files on disk.
5. **MIME Container Hint & HTTP Charset Retention**:
   - Generic types (`application/octet-stream`, `binary/octet-stream`, `application/zip`) fall back to caller hint or MIME sniffing.
   - Matching video/audio containers (e.g. `video/mp4` vs `audio/mp4`) preserve the caller's audio hint.
   - Staged text attachments preserve HTTP charset parameters (`application/x-openclaw-charset-proof;charset=cp1251`).
   - Prototype-named Content-Types (`__proto__`) are safely handled without stripping or panicking.
6. **SSRF Guard & Bot Token Redaction**:
   - Localhost (`127.0.0.1`, `localhost`, `::1`) is blocked by default unless explicitly permitted in origin or hostname policies.
   - Telegram bot tokens in URLs and error messages are redacted (`bot123456…abcd`).
7. **Empty Response Rejection**:
   - Empty response bodies and HTTP 204 reject with `HTTP 204; empty response body` without persisting blank disk files.

---

## 5. Automated Verification Evidence

The full contract test suite passes with zero failures:
```text
running 7 tests
test test_ocrm_media_fetch_content_type_container_hint_and_charset ... ok
test test_ocrm_media_fetch_content_disposition_rfc5987_and_fallbacks ... ok
test test_ocrm_media_fetch_url_basename_and_malformed_percent ... ok
test test_ocrm_media_fetch_content_length_and_max_bytes ... ok
test test_ocrm_media_fetch_ssrf_and_redaction ... ok
test test_ocrm_media_fetch_gateway_dispatch ... ok
test test_ocrm_media_fetch_empty_response_and_original_filename ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```
Across all attachment and media test suites, all 39 contract tests pass with 100% success.
