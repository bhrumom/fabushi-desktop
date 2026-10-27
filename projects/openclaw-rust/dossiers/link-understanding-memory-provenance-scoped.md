# OCRM Dossier: Link Understanding & Memory Root Provenance

- **Unit ID**: `OCRM-UNIT-039`
- **Module ID**: `OCRM-M26` (`link-understanding` & `memory`)
- **Responsibility IDs**:
  - `OCRM-RESP-LINK-UNDERSTANDING-001`
  - `OCRM-RESP-MEMORY-ROOT-PROVENANCE-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 9 upstream files (1,390 lines total) are audited, mapped, and natively implemented:

### `src/link-understanding/` (5 files, 798 lines) -> `OCRM-RESP-LINK-UNDERSTANDING-001`
1. `src/link-understanding/apply.runtime.ts` (3 lines)
2. `src/link-understanding/apply.ts` (235 lines)
3. `src/link-understanding/defaults.ts` (5 lines)
4. `src/link-understanding/runner.test.ts` (338 lines)
5. `src/link-understanding/runner.transport.test.ts` (217 lines)

### `src/memory/` (4 files, 592 lines) -> `OCRM-RESP-MEMORY-ROOT-PROVENANCE-001`
1. `src/memory/root-memory-files.ts` (75 lines)
2. `src/memory/root-memory-files.test.ts` (35 lines)
3. `src/memory/memory-artifact-provenance.ts` (255 lines)
4. `src/memory/memory-artifact-provenance.test.ts` (227 lines)

## 2. Invariants & Implementation Details

- **Link Extraction & SSRF Filtering**:
  - Markdown citation link spans (`[...](...)`) are identified and masked out, ensuring that displayed citation URLs never trigger automatic link fetches.
  - Bare URLs (`http://` / `https://`) are extracted, validated against internal/loopback blocked host rules (`localhost`, `127.0.0.1`, `::1`, `*.local`, `*.internal`), deduplicated, and clamped to `max_links` (default 3).
  - CLI command templates containing `LinkUrl` or `LinkFinalUrl` bypass secondary stdin pipe execution for curl/wget to prevent redundant network downloads.
  - Inbound message body is enriched with link extraction text separated by double newlines.
- **Root Memory Files & Auxiliary Skipping**:
  - Canonical memory file name is fixed to `MEMORY.md`; legacy memory file is `memory.md`.
  - Migration repair path is `.openclaw-repair/root-memory`.
  - Auxiliary scan skips legacy `memory.md`, repair directories, and repair contents, but preserves canonical `MEMORY.md`.
- **Memory Artifact Provenance**:
  - Path normalization validates relative paths, rejects absolute paths (`/`), prevents traversal (`..`), allows `MEMORY.md`, `memory.md`, `USER.md`, `users/{user}/USER.md`, and `memory/*.md`, while explicitly forbidding dream paths (`memory/dreaming/`, `memory/.dreams/`).
  - SHA-256 content hashing guarantees artifact integrity.
  - Write provenance origin resolution: writes by agents remain `Agent` only if the previous record was also `Agent` and the before-content hash matches; any mismatch degrades the artifact to `Untrusted`.

## 3. Rust Native Equivalence

Implemented in:
- `source/host/src/extensions/attachments/link_understanding_service.rs`:
  - `DEFAULT_LINK_TIMEOUT_SECONDS`, `DEFAULT_MAX_LINKS`, `LinkUnderstandingConfig`, `LinkUnderstandingModelConfig`.
  - `find_markdown_link_spans`, `extract_bare_links_from_message`, `is_blocked_link_host`, `is_link_url_template`, `enrich_message_body_with_links`.
- `source/host/src/extensions/memory/root_memory_provenance.rs`:
  - `CANONICAL_ROOT_MEMORY_FILENAME`, `LEGACY_ROOT_MEMORY_FILENAME`, `ROOT_MEMORY_REPAIR_RELATIVE_DIR`.
  - `resolve_canonical_root_memory_path`, `resolve_legacy_root_memory_path`, `resolve_root_memory_repair_dir`, `should_skip_root_memory_auxiliary_path`.
  - `normalize_memory_artifact_relative_path`, `compute_file_content_hash`, `MemoryArtifactOriginClass`, `MemoryArtifactProvenance`, `resolve_memory_artifact_write_origin`.

## 4. Verification Evidence

- Dedicated contract test suite: `source/host/tests/openclaw_link_memory_contract.rs`
  - `test_ocrm_link_und_o01_markdown_span_exclusion_and_bare_link_extraction`: tests markdown span exclusion, SSRF blocked host filtering, link deduplication, and max link clamping.
  - `test_ocrm_link_und_o02_template_detection_and_body_enrichment`: tests `LinkUrl` template detection and message body enrichment.
  - `test_ocrm_memory_root_o01_paths_and_auxiliary_filtering`: tests canonical/legacy root paths, repair dir, and auxiliary skipping.
  - `test_ocrm_memory_provenance_o02_relative_paths_and_origin_resolution`: tests relative path normalization, forbidden dream directories, SHA-256 calculation, and agent/untrusted write origin resolution.
- All 4 tests passed with 0 failures; media understanding (31 tests) and media generation (5 tests) suites continue to pass cleanly.
