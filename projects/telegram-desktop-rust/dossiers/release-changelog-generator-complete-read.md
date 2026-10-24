# Release changelog generator complete read — strict index 76

## Authority
- Upstream: `telegramdesktop/tdesktop@e14f1566d277d2204f4c8e553bf7cc157047e88b`, tree `241f8ab614292cc90e395ef0c966e289fdb3e985`.
- Source: `.github/scripts/generate_changelog.py@6cdd12652d31b2a47ed9a8ef906308af59cca3d0`, 329 lines / 9036 bytes; complete read and SHA revalidated after the live dev relock.
- Applicable responsibilities: deterministic release-note parsing, supported version/date normalization, stable ordering/provenance projection, HTML escaping/static rendering, bounded output writes and bounded presentation-asset copying.
- Provenance-only details: Telegram changelog path, Telegram branding, repository/license links, favicon names, GitHub Pages markup/client-side search/anchor-copy behavior.
- Existing-owner evidence: `AGENTS.md@ff07a5abee9e124f4d4f104da0dbedb51f0ddc1b` owns release preparation and exact-head verification; `.github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643` owns exact-source version/package/sign/notarize/publish mechanics; `FQT-REL-001@c5eb9d0897234bb2fdad4f505549b2350c44aa72` owns release entry/exit and artifact identity; `FQT-EVID-001@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` owns evidence provenance.
- Parent exact head/tree: `44bedab87bff3e7ac1f872215c0f67312b4aa942` / `a4a7e6f1db8ff7ed5cac613b5f562db8682f88c8`.
- Parent TDRP run `39749565989` completed success, 61/61; authority `12412044470` / `sha256:936249af78f1cf50661545f621291e591a4a1d8fea41ac60a8c357e4bd0c71be`; closure `12412189536` / `sha256:de237dfeb817b931e2c6072ba12224c49635fe9be08025f9609619ac94262ff9`; reachability `12413068729` / `sha256:5f2b0b71daff60cb6ea98d33e2de0e87063ff88c535ff4883f42ea3d642b16c4`.
- Parent Rust run `39749565996` completed success, 6/6; artifact `12412164987` / `sha256:c804472b6e1e7233dda567c081820853e699a5a7ee5d34f7a1e65a48e521ba7f`.

## Existing-owner-first disposition
The generator is a derived documentation adapter, not a canonical release/version owner. Fabushi keeps canonical release-note/version and publication/acceptance truth in the existing repository/release lifecycle. Deterministic parse/order behavior, supported date normalization, HTML escaping, bounded output and asset-copy behavior are retained as source-neutral governance; Telegram page layout, branding, links, icons and client-side search remain provenance. No Telegram changelog runtime, second documentation store, release root, packaging owner or acceptance authority is introduced; `novel_capability=false`.

## Exact traceability IDs
- Requirement: `TDRP-R9-RELEASE-CHANGELOG-RENDERING-001`
- Oracles: `ORA-TDRP-CHANGELOG-DETERMINISTIC-PARSE-001`, `ORA-TDRP-CHANGELOG-HTML-ESCAPE-001`, `ORA-TDRP-CHANGELOG-RELEASE-AUTHORITY-SEPARATION-001`
- Invariants: `INV-TDRP-CHANGELOG-NO-GENERATED-AS-TRUTH-001`, `INV-TDRP-CHANGELOG-NO-PUBLISH-AUTHORITY-001`, `INV-TDRP-CHANGELOG-EXACT-SOURCE-BOUND-001`
- Contracts: `CONTRACT-TDRP-CHANGELOG-EXACT-INPUT-001`, `CONTRACT-TDRP-CHANGELOG-ESCAPED-OUTPUT-001`, `CONTRACT-TDRP-CHANGELOG-BOUNDED-WRITE-001`
- Fault/recovery: `FAULT-TDRP-CHANGELOG-MALFORMED-INPUT-001`, `FAULT-TDRP-CHANGELOG-STALE-INPUT-001`, `FAULT-TDRP-CHANGELOG-WRITE-FAILURE-001`
- Security: `SEC-TDRP-CHANGELOG-HTML-OUTPUT-ESCAPE-001`
- Regression: `REG-TDRP-CHANGELOG-NO-SECOND-RELEASE-OWNER-001`, `REG-TDRP-CHANGELOG-NO-TELEGRAM-PRESENTATION-AUTHORITY-001`

Closing strict index 76 changes counters to unknown=16,227, unread=16,202, omitted=0, responsibilities=94. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 77 `.github/telegram-bot-api/Dockerfile@a361602441040028a3332c3732c4902d9c880122`.
