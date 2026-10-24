# AGENTS repository guidance complete read

## Exact source and parent evidence
- Upstream authority: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Strict101 source: `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32`, 1,006 lines / 46,865 bytes, fully read.
- Parent Fabushi exact HEAD: `3d7acfed8ffbf17b344c2d1594dc6bb7f369175b`, tree `97a9bb40961e54a8859dcfd1e6656ba97e8f2b2e`.
- Parent TDRP run `39904761894`: 61/61 success; authority `12480098862` `sha256:1bff2a18d5e051fcae134be7c3c1fd12e6c9ae730c884344221af6311648cf1d`; closure `12481140109` `sha256:c263a0690a4ba1386a27c53af4e5fa73b1f75b0f2105f238824bb13f4e27fa9c`; reachability `12480093227` `sha256:0e94c5900188b162c9101a413f2e2b24133fbc35b0a89441fe77b59359aac4b8`.
- Parent Rust desktop runtime run `39904761917`: 6/6 success; artifact `12480029114` `sha256:1d04cfdbbadd6651b7aa5ddc53dd81546806712c318c377dba6ab60d1b5fadee`.

## Responsibility
The upstream guide is a repository-wide source of development, build, testing, serialization, platform, API-lifetime, style, input-method, localization and reactive-programming guidance. It is not itself a shipping Telegram runtime. Where it describes concrete product behavior, the document is provenance and an oracle pointer only: the underlying product source still requires independent file/symbol migration and production evidence.

Applicable source-neutral responsibilities map to current Fabushi governance:
- canonical single-target/existing-owner-first migration rules;
- GitHub-Actions-only executable validation for TDRP;
- repository quality/build/platform discipline;
- persistence/serialization, async-lifetime, input-method, localization and reactive-state guidance as requirements to be carried by their existing product/platform owners when those source files are reached.

The guide's Telegram-specific commands, Qt/CMake mechanics, MTProto request syntax, Telegram style DSL and repository task tooling are not copied as a parallel Fabushi runtime, task system, UI system, network stack or build graph.

## Existing-owner-first
Selected durable owners are existing exact-head Fabushi governance only:
- root `AGENTS.md` single-target/existing-owner-first and TDRP GitHub-Actions-only rules;
- `projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md` source-authority/existing-owner-first chain;
- `docs/specs/telegram-desktop-rust-equivalence-migration.md` non-runtime-accounting and product-responsibility proof boundary.

Preflight found zero prior `AGENTS.md` source-path rows, zero prior source-blob rows and zero `TDRP-R9-REPOSITORY-AGENT-GUIDANCE-PROVENANCE-001` rows. The four selected target statements each occur exactly once in their exact-head owner file and have zero prior target-path+symbol ownership rows. Live main advanced only with OCRM/OpenClaw-specific AGENTS additions; the selected TDRP target statements remain unique there as well. No second agent/task queue, build authority, persistence owner, UI system, network owner or acceptance root is introduced.

## Fail-closed boundaries
This mapping does not promote any product responsibility merely because the guide documents it. Serialization, request lifetime, input method, localization, RPL/reactive state, platform behavior and similar product responsibilities remain open until their actual source owners are read, mapped, implemented and verified. Repository instructions cannot substitute for source completeness, production wiring, packaged acceptance, service evidence or current-head tests. Contradictory or stale guidance is non-authoritative relative to current Fabushi source-of-truth/specs and exact-head production owners.

## Traceability
- Requirement: `TDRP-R9-REPOSITORY-AGENT-GUIDANCE-PROVENANCE-001`
- Oracles: `ORA-TDRP-AGENTS-GOVERNANCE-VS-PRODUCT-SEPARATION-001`, `ORA-TDRP-AGENTS-EXISTING-OWNER-FIRST-001`, `ORA-TDRP-AGENTS-GITHUB-ACTIONS-ONLY-001`
- Invariants: `INV-TDRP-AGENTS-NO-SECOND-RUNTIME-001`, `INV-TDRP-AGENTS-NO-DOCUMENT-AS-PRODUCTION-EVIDENCE-001`, `INV-TDRP-AGENTS-NO-SOURCE-CLOSURE-FABRICATION-001`
- Contracts: `CONTRACT-TDRP-AGENTS-GOVERNANCE-MAPPING-001`, `CONTRACT-TDRP-AGENTS-PRODUCT-RESPONSIBILITY-REMAINS-OPEN-001`, `CONTRACT-TDRP-AGENTS-EXACT-HEAD-EVIDENCE-001`
- Faults: `FAULT-TDRP-AGENTS-STALE-GUIDANCE-001`, `FAULT-TDRP-AGENTS-PRODUCT-DOC-PROMOTION-001`, `FAULT-TDRP-AGENTS-DUPLICATE-OWNER-001`
- Security: `SEC-TDRP-AGENTS-GUIDANCE-NOT-AUTHORIZATION-BOUNDARY-001`
- Regression: `REG-TDRP-AGENTS-NO-SOURCE-OMISSION-001`, `REG-TDRP-AGENTS-NO-BOT-OWNER-REGRESSION-001`

Closing strict101 yields unknown=16,275, unread=16,250, omitted=0, responsibilities=119. Next strict root is index 102 `CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155`. Fresh child exact-head TDRP/Rust evidence is required.
