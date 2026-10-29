# Plugin SDK Browser Subsystem, Archive Inspection, & Utilities Dossier

## 1. Scope & Source Attribution
- **Target Files (19 files in `src/plugin-sdk/`)**:
  - `src/plugin-sdk/archive.ts` (11 lines)
  - `src/plugin-sdk/async-lock-runtime.ts` (3 lines)
  - `src/plugin-sdk/blob-runtime.ts` (8 lines)
  - `src/plugin-sdk/boolean-param.test.ts` (13 lines)
  - `src/plugin-sdk/boolean-param.ts` (10 lines)
  - `src/plugin-sdk/browser-bridge.test.ts` (127 lines)
  - `src/plugin-sdk/browser-bridge.ts` (45 lines)
  - `src/plugin-sdk/browser-cdp.ts` (84 lines)
  - `src/plugin-sdk/browser-config.test.ts` (21 lines)
  - `src/plugin-sdk/browser-config.ts` (9 lines)
  - `src/plugin-sdk/browser-control-auth.ts` (60 lines)
  - `src/plugin-sdk/browser-facades.test.ts` (77 lines)
  - `src/plugin-sdk/browser-maintenance.test.ts` (310 lines)
  - `src/plugin-sdk/browser-maintenance.ts` (59 lines)
  - `src/plugin-sdk/browser-profiles.browser-import.test.ts` (28 lines)
  - `src/plugin-sdk/browser-profiles.ts` (62 lines)
  - `src/plugin-sdk/browser-subpaths.test.ts` (34 lines)
  - `src/plugin-sdk/browser-trash.ts` (4 lines)
  - `src/plugin-sdk/browser-types.ts` (67 lines)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugin-sdk).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGIN-SDK-BROWSER-AND-ARCHIVE-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 Bounded Archive Extraction & Security Guards
- **Extraction Limits**: Enforces `ArchiveExtractLimits` (`max_files`, `max_total_bytes`, `max_file_size`) during tar unpacking to defend against zip/tar bombs.
- **Directory Traversal Defense**: Sanitizes all entry paths; rejects `ParentDir` (`..`) components before writing to destination disk.
- **Single-Entry Extraction**: `read_archive_entry` locates and decompresses a specific target file entry directly from tar without writing other contents to disk.

### 2.2 Process-Local Locks & Blob Handling
- **Async Locks**: Key-based runtime file locking mechanism (`AsyncLockRuntime`) for protecting concurrent writes to session and config files.
- **Blob Part Snapshotting**: Exact byte range snapshotting (`buffer_to_blob_part`) with slice bounds verification.
- **Base64 Canonicalization**: Trims whitespace and newlines, pads with `=` to a multiple of 4, and validates decoding.

### 2.3 Boolean Parameter Parsing & Coercion
- **Key Precedence**: Exact camelCase key takes priority over snake_case key (e.g. `dryRun` over `dry_run`).
- **Coercion Matrix**: Coerces strings `"true"`, `"1"`, `"yes"`, `"on"` to `true`; `"false"`, `"0"`, `"no"`, `"off"` to `false`; numbers `1` and `0` to `true` and `false` respectively; all other values to `None`.

### 2.4 Browser Endpoint Parsing & WHATWG Normalization
- **Scheme Validation**: Strictly restricts protocols to `http`, `https`, `ws`, and `wss`.
- **Port Preservation**: Detects whether the user explicitly supplied a default port (e.g., `:80` for HTTP or `:443` for HTTPS) before standard URL normalization drops it, restoring it in `normalized_with_port`.
- **Port 0 Rejection**: Explicitly rejects port 0 as invalid.
- **Credential & Token Redaction**: Strips username/password from CDP URLs and scrubs `token=***` from query parameters for diagnostics and logs.

### 2.5 Browser Configuration, Profiles, and Authentication
- **Default Resolution**: Exposes standard OpenClaw defaults (`DEFAULT_OPENCLAW_BROWSER_ENABLED = true`, `DEFAULT_OPENCLAW_BROWSER_COLOR = "#FF4500"`, `DEFAULT_OPENCLAW_BROWSER_PROFILE_NAME = "openclaw"`).
- **Authentication**: Precedence resolves token/password from environment variables (`OPENCLAW_BROWSER_TOKEN`) before falling back to gateway config, auto-generating securely when permitted.
- **Profile Resolution**: Resolves default `openclaw` profile or custom profiles with port, URL, loopback status, and driver parameters.

### 2.6 Safe Trash & Tracked Tab Maintenance
- **Platform Trash Movement**: Safe directory reservation (`.Trash/<uuid>/<file>`) with unix mode `0o700`.
- **Tab Cleanup Safety**: Enforces caller contract requiring `sessionEntryCurrent` to pair with `prepareCurrent`; aborts cleanup when caller ownership checks fail (`isCurrent() === false`).

## 3. Rust Equivalents Implemented
- [`source/host/src/extensions/plugin_sdk/archive_blob.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/archive_blob.rs)
- [`source/host/src/extensions/plugin_sdk/boolean_param.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/boolean_param.rs)
- [`source/host/src/extensions/plugin_sdk/browser.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/browser.rs)

## 4. Contract Test Verification
- [`source/host/tests/openclaw_plugin_sdk_contract.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/tests/openclaw_plugin_sdk_contract.rs):
  - `test_ocrm_plugin_sdk_boolean_param`
  - `test_ocrm_plugin_sdk_archive_and_blob`
  - `test_ocrm_plugin_sdk_async_lock`
  - `test_ocrm_plugin_sdk_browser_url_parsing_and_redaction`
  - `test_ocrm_plugin_sdk_browser_config_and_profiles`

## 5. Audit Status
- Mapped Files: 19 added (Total mapped: 764 / 58,750).
- Unclassified Remaining: 57,986.
- Global completion flags: all 8 remaining `false` (fail-closed).
