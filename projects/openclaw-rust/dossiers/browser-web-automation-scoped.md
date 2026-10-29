# OCRM Technical Dossier — Unit 7: Browser & Web Automation (`OCRM-UNIT-BROWSER-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M11 (Browser, Web Search & Fetch Subsystems)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The complete 9-line `src/shared/browser-image-mime-types.ts` source (blob
`26f9a040cab5391c86383453a8b457e0a654bbea`) is now recorded as a locked
source read and mapped to `OCRM-RESP-BROWSER-IMAGE-TYPES-001`; no production
acceptance status is changed by this source audit.

The following 6 core source files (totaling 2,568 lines) were independently read and mapped into canonical shards:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0040` | `src/infra/net/ssrf.ts` | `664540bba2ec156f9ef5ef57bfd1fb7bd7f93d0d` | 736 | 1..736 (full-file) | `OCRM-RESP-BROWSER-SSRF-GUARD-001` |
| `part-0026` | `src/agents/tools/web-fetch.ts` | `e9dd6374f40b7917a0a8b63398b047ad7e84e47c` | 950 | 1..950 (full-file) | `OCRM-RESP-BROWSER-WEB-FETCH-001` |
| `part-0048` | `src/web-search/runtime.ts` | `5cf9e9d4a8f394e5399e2658015137a1735b1a61` | 616 | 1..616 (full-file) | `OCRM-RESP-BROWSER-WEB-SEARCH-001` |
| `part-0048` | `src/web-fetch/runtime.ts` | `c818a88aa10cc7c72d2d9d254c60b2487ad65c57` | 166 | 1..166 (full-file) | `OCRM-RESP-BROWSER-FETCH-RUNTIME-001` |
| `part-0046` | `src/shared/browser-image-mime-types.ts` | `26f9a040cab5391c86383453a8b457e0a654bbea` | 9 | 1..9 (full-file) | `OCRM-RESP-BROWSER-IMAGE-TYPES-001` |
| `part-0048` | `src/worker/browser-runtime.ts` | `765032c984a44c6337373c9b1007b68764dc0463` | 91 | 1..91 (full-file) | `OCRM-RESP-BROWSER-WORKER-LIFECYCLE-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-BROWSER-SSRF-GUARD-001`**:
   - **Behavior**: SSRF defense preflight inspection, verifying target URLs against loopback (`127.0.0.0/8`, `::1`), RFC 1918 private subnets (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), link-local IPs, cloud metadata endpoints (`169.254.169.254`, `metadata.google.internal`), and stripping userinfo credentials.
   - **Rust Owner**: `source/host/src/runner/tools/sand_web_tools.rs:validate_web_fetch_url`.
   - **Oracles**: `OCRM-BROWSER-O07`, `OCRM-BROWSER-O08`.

2. **`OCRM-RESP-BROWSER-WEB-FETCH-001`**:
   - **Behavior**: Fetch result markdown extraction and rendering, strictly capping returned external page content to 100,000 characters and appending standard truncation footers.
   - **Rust Owner**: `source/host/src/runner/tools/sand_web_tools.rs:render_web_fetch_result`.
   - **Oracle**: `OCRM-BROWSER-O09`.

3. **`OCRM-RESP-BROWSER-WEB-SEARCH-001`**:
   - **Behavior**: Web search query normalization, stripping ASCII control characters, provider endpoint query routing, and reference citation synthesis.
   - **Rust Owner**: `source/host/src/runner/tools/sand_web_tools.rs:SandWebToolBridge`.
   - **Oracles**: `OCRM-BROWSER-O01`, `OCRM-BROWSER-O10`.

4. **`OCRM-RESP-BROWSER-FETCH-RUNTIME-001`**:
   - **Behavior**: Typed host `WebToolExecutor` trait and HTTP execution bridge.
   - **Rust Owner**: `source/host/src/runner/tools/sand_web_tools.rs:WebToolExecutor`.
   - **Oracle**: `OCRM-BROWSER-O09`.

5. **`OCRM-RESP-BROWSER-IMAGE-TYPES-001`**:
   - **Behavior**: Viewport screenshot raster MIME type validation and base64 envelope framing with monotonic generation tokens.
   - **Rust Owner**: `source/host/src/runner/tools/sand_browser_tools.rs:BrowserEnvelope`.
   - **Oracles**: `OCRM-BROWSER-O01`, `OCRM-BROWSER-O04`.

6. **`OCRM-RESP-BROWSER-WORKER-LIFECYCLE-001`**:
   - **Behavior**: Containerized browser worker lifecycle and strict 7-method CDP observation allowlist, actively denying arbitrary `Runtime.evaluate` to prevent sandbox escape.
   - **Rust Owner**: `source/host/src/runner/tools/sand_browser_tools.rs:BOX_CDP_PORT_BASE`.
   - **Oracles**: `OCRM-BROWSER-O02`, `OCRM-BROWSER-O03`.

---

## 3. Falsifiable Test Oracles (`source/host/tests/openclaw_browser_web_contract.rs`)

1. **`OCRM-BROWSER-O01`**: `test_ocrm_browser_o01_tool_dispatch_and_delimiter_framing`
2. **`OCRM-BROWSER-O02`**: `test_ocrm_browser_o02_disallowed_cdp_command_rejection_in_driver`
3. **`OCRM-BROWSER-O03`**: `test_ocrm_browser_o03_allowed_cdp_observation_specs`
4. **`OCRM-BROWSER-O04`**: `test_ocrm_browser_o04_view_generation_and_envelope_invariants`
5. **`OCRM-BROWSER-O05`**: `test_ocrm_browser_o05_file_upload_workspace_confinement`
6. **`OCRM-BROWSER-O06`**: `test_ocrm_browser_o06_file_download_size_cap_and_sha256_invariants`
7. **`OCRM-BROWSER-O07`**: `test_ocrm_browser_o07_web_fetch_ssrf_and_localhost_rejection`
8. **`OCRM-BROWSER-O08`**: `test_ocrm_browser_o08_web_fetch_userinfo_sanitization`
9. **`OCRM-BROWSER-O09`**: `test_ocrm_browser_o09_web_fetch_content_size_truncation`
10. **`OCRM-BROWSER-O10`**: `test_ocrm_browser_o10_auto_review_input_length_boundary_enforcement`

---

## 4. Conservation & Invariant Metrics

- `tracked_entries_total`: **58,750**
- `mapped_paths`: **100** (94 prior + 6 Unit 7)
- `unread`: **58,650**
- `unclassified`: **58,650**
- `registered_responsibilities`: **42**
- All 58 inventory shards checksums verified in `manifest.json`.
- All global completion flags strictly maintained as **false**.
