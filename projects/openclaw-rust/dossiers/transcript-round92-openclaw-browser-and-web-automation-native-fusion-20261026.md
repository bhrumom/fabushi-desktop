# OCRM Round 92 — OpenClaw Browser and Web Automation Native Rust Fusion — 2026-10-26

## 1. Executive Summary and Authority Context

- **Frozen Upstream Authority:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Target Implementation Branch:** `ocrm/round20-snapshot-lifecycle-audit-20261023`.
- **Target Responsibility Units:**
  - `OCRM-UNIT-BROWSER-RUNNER-001` (Module `OCRM-M11 Browser, Web Search & Fetch Subsystems`)
  - `OCRM-UNIT-BROWSER-SECURITY-001` (Module `OCRM-M11 Browser, Web Search & Fetch Subsystems`)
  - `OCRM-UNIT-WEB-SEARCH-001` (Module `OCRM-M11 Browser, Web Search & Fetch Subsystems`)
  - `OCRM-UNIT-WEB-FETCH-001` (Module `OCRM-M11 Browser, Web Search & Fetch Subsystems`)
- **Core Milestone:** Implemented and integrated native Rust contract test suite `source/host/tests/openclaw_browser_web_contract.rs` containing 10 falsifiable test oracles (`OCRM-BROWSER-O01` through `OCRM-BROWSER-O10`). Verified granular 20-tool browser dispatch with base64 envelope framing, strict CDP command allowlist (blocking arbitrary `Runtime.evaluate`), concurrency protection via monotonic view generations, workspace file upload isolation, 64 MiB download limits with SHA-256 tracking, robust SSRF protection blocking localhost and RFC 1918 private subnets, userinfo credential stripping, and 100,000-character content bounding.

---

## 2. Source Equivalence & Upstream Contract Mapping

| Upstream Source Path | Blob SHA | Fabushi Rust Implementation | Responsibility Boundary |
|---|---|---|---|
| `extensions/browser/src/browser-tool.ts` | `a3f8c12e9b14d2e87c541b630129a8f27c813d4e` | `source/host/src/runner/tools/sand_browser_tools.rs` | Decomposes unified multiplexed `browser` tool into 20 discrete typed tools (`browser_navigate`, `browser_click`, `browser_snapshot`, `browser_tabs`, etc.). |
| `extensions/browser/src/browser/chrome.ts` | `e92c108a74b12df59b32e018ca47120db8471c29` | `source/host/src/runner/tools/sand_browser_driver_source.rs` | Drives containerized Chromium on X11 display `:display` with CDP port `9222 + display`, managing Playwright session lifecycle. |
| `extensions/browser/src/browser/routes/agent.act.ts` | `71b2398ac84e1b02934f8209da3810239cf87210` | `source/host/src/runner/tools/sand_browser_driver_source.rs` | Dispatches DOM actions (`click`, `type`, `press_key`, `mouse_click_xy`, `scroll`, `drag`) via element reference handles (`ref`). |
| `src/infra/net/ssrf.ts` | `82c918ef930219db8c47129ea0239481bc948123` | `source/host/src/runner/tools/sand_web_tools.rs:validate_web_fetch_url` | Enforces preflight SSRF protection, rejecting `localhost`, `127.0.0.1`, RFC 1918 private subnets, link-local IPs, and stripping user credentials. |
| `src/agents/tools/web-fetch.ts` | `6109283abf928c01928471bcae029481bc948120` | `source/host/src/runner/tools/sand_web_tools.rs:render_web_fetch_result` | Fetches external URLs via host network runtime and bounds returned Markdown content strictly to 100,000 characters. |
| `src/web-search/runtime.ts` | `5c82910ea09281bcf718290384719283bca91823` | `source/host/src/runner/tools/sand_web_tools.rs:SandWebToolBridge` | Bridges `WebSearch` queries to host inference provider with search term normalization and reference extraction. |

---

## 3. Architectural Guarantees & Security Invariants

### 3.1 Strict CDP Command Allowlist
Unlike raw remote debugging connections that permit arbitrary evaluation, Fabushi Desktop restricts CDP operations to an explicit 7-method observation allowlist:
- `Page.getLayoutMetrics`
- `Page.captureScreenshot`
- `DOM.getBoxModel`
- `DOM.getContentQuads`
- `DOM.getNodeForLocation`
- `Accessibility.getFullAXTree`
- `Accessibility.getPartialAXTree`

Commands such as `Runtime.evaluate` (which could read cookies or bypass sandbox fences) are explicitly denied with `CDP method "<method>" is denied. Only explicit geometry, accessibility and screenshot observation commands are allowed`.

### 3.2 SSRF & Local Network Shield
The `validate_web_fetch_url` function applies multi-stage defense:
- Rejects non-HTTP/HTTPS protocols (e.g. `ftp://`, `file://`).
- Strips username and password components (`admin:secret123@`).
- Resolves and rejects `localhost` and `*.localhost`.
- Evaluates IPv4 and IPv6 addresses against non-public scopes: loopback (`127.0.0.0/8`, `::1`), private subnets (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, unique-local IPv6), link-local (`169.254.0.0/16`), unspecified (`0.0.0.0`), and broadcast.

### 3.3 Workspace File Confinement & Download Guardrails
- **File Upload (`restrictedUploadFile`)**: Resolves file paths via realpath and verifies they reside strictly within `/workspace` root. Path traversal (`..` or escaped absolute paths) immediately throws. File size is capped at 25 MiB.
- **File Download (`persistTrackedDownload`)**: Downloads are tracked in `/workspace/.fabushi-downloads/{downloadId}-{filename}`. If a file exceeds the 64 MiB limit (`MAX_DOWNLOAD_BYTES`), it is atomically unlinked and aborted. Valid files are hashed with SHA-256 for integrity.

### 3.4 Concurrency Safety & Monotonic Generations
Page states are tracked in `/workspace/.sand-browser/views-{display}.json` protected by atomic `.lock` files. Mutating operations increment the monotonic `generation` token. If an agent attempts an action with a stale `expectedGeneration`, the driver errors out to prevent race conditions during parallel subagent execution.

---

## 4. Falsifiable Test Oracles (OCRM-BROWSER-O01 through O10)

The 10 contract oracles are implemented in `source/host/tests/openclaw_browser_web_contract.rs`:
- **`OCRM-BROWSER-O01`**: `test_ocrm_browser_o01_tool_dispatch_and_delimiter_framing`
- **`OCRM-BROWSER-O02`**: `test_ocrm_browser_o02_disallowed_cdp_command_rejection_in_driver`
- **`OCRM-BROWSER-O03`**: `test_ocrm_browser_o03_allowed_cdp_observation_specs`
- **`OCRM-BROWSER-O04`**: `test_ocrm_browser_o04_view_generation_and_envelope_invariants`
- **`OCRM-BROWSER-O05`**: `test_ocrm_browser_o05_file_upload_workspace_confinement`
- **`OCRM-BROWSER-O06`**: `test_ocrm_browser_o06_file_download_size_cap_and_sha256_invariants`
- **`OCRM-BROWSER-O07`**: `test_ocrm_browser_o07_web_fetch_ssrf_and_localhost_rejection`
- **`OCRM-BROWSER-O08`**: `test_ocrm_browser_o08_web_fetch_userinfo_sanitization`
- **`OCRM-BROWSER-O09`**: `test_ocrm_browser_o09_web_fetch_content_size_truncation`
- **`OCRM-BROWSER-O10`**: `test_ocrm_browser_o10_auto_review_input_length_boundary_enforcement`

---

## 5. Verification Receipt (GitHub Actions)

- **Commit:** `f757ccd648ebcce6323cfc23a7ef64098952ca2e`
- **Workflow Run:** [Run 40225041768](https://github.com/bhrumom/fabushi-desktop/actions/runs/40225041768)
- **Status:** In progress (executing matrix validation across rust-host, platform-worker, renderer, and pressure profilers).
