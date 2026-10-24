# Upstream root changelog.txt release-note/history provenance — strict index 109

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `changelog.txt@41b7466f8a11a1866a66ff583809dfe74ad9a3c3`, 5400 parsed lines / 157860 parsed characters / 157914-byte upstream blob, fully read.
- Structural accounting: 945 version/date headers (522 unlabeled, 199 beta, 97 alpha, 54 stable, 73 dev) + 2544 `- ` release-note records + 7 legacy `— ` records + 1891 blank lines + 13 wrapped continuation / section-label / preserved legacy-fragment lines = 5400 total lines.
- History bounds: newest `7.3.3 (21.10.26)`; oldest retained header `0.1.16 (29.11.13)`.
- Parent exact HEAD: `51b5bb86ed636a451b541519a219b0aee5898df7`, tree `943ebdd12b35c8528cd8cd6194c4b9b3000f72fe`.
- Parent TDRP run `39920230784`: 61/61 success; authority `12487610158` sha256:9f2162ea7d87a2aa2e7ad4909f97ba48e90b6f17186d84b6e912a770892e788f; closure `12487655772` sha256:1c362e040d124f650d745d1a4f5a92c187541c52eafd7a6957d798fa3bf69b84; reachability `12487401431` sha256:9fab76e0c8bea69ebe2dece88c5655fc3be15ff99c4763c48f6cd0316affc7cf.
- Parent Rust run `39920230822`: 6/6 success; artifact `12485894801` sha256:d22eea53c70a02768a1e23d84e8fac687dbf7df902fc41bccdd22fc00d667ddd.

## Responsibility decomposition

The file is an exact historical release-note ledger: ordered version/date/channel headers plus user-facing change records. It preserves historical product claims, bug-fix descriptions, platform notes, packaging/update notes and release voice, but it is not executable implementation or a current capability oracle. Historical bullets remain discovery/provenance inputs whose actual product responsibilities must still be closed by their authoritative source files, owners, tests and exact-head evidence.

All 13 non-header/non-bullet/nonblank lines were also read and retained: wrapped Linux icon-renaming continuations, section labels, one quoted historical UI line, and four legacy `),` fragments. They are preserved as source provenance rather than silently corrected.

## Existing-owner / validator uniqueness

Fresh parent-head preflight found zero prior `changelog.txt` path rows, zero `41b7466f8a11a1866a66ff583809dfe74ad9a3c3` blob rows, and zero `TDRP-R9-UPSTREAM-ROOT-CHANGELOG-RELEASE-HISTORY-PROVENANCE-001` rows. The selected canonical target is the existing `AGENTS.md` release-note authority:

> Release notes come from the exact commit interval selected by repository release policy and are user-facing summaries, not a commit dump. Exclude CI/build-only/refactor/WIP noise unless user-visible, collapse related changes, preserve historical entries, and keep the repository's established release-note voice and format. When a stable release must subsume prior prereleases, summarize from the last stable baseline rather than only the newest prerelease. Telegram `changelog.txt` paths, fixed bullet rules, and provider syntax are provenance only; use Fabushi's canonical release-note authority when it exists.

That paragraph occurs exactly once at the parent head, and the exact `AGENTS.md` path + selected symbol has zero ledger owners. Existing supporting owners are already mapped: `TDRP-R9-RELEASE-CHANGELOG-RENDERING-001` owns derived changelog presentation, `TDRP-R9-RELEASE-CHANGELOG-PAGES-PUBLICATION-001` owns replaceable Pages publication, while `FQT-REL-001`, `FQT-EVID-001` and the exact-source release workflow retain version/package/signing/evidence/acceptance authority. No second release-note truth, version owner, release owner, publisher authority or acceptance validator is introduced.

## Fail-closed boundaries

- Historical Telegram version/channel syntax and fixed bullet conventions are provenance only; they do not select a Fabushi version.
- Historical feature/fix bullets do not prove current Fabushi implementation, UI behavior, compatibility or service availability.
- A derived changelog renderer/publisher cannot tag, sign, notarize, deploy, promote or create release ACCEPT.
- The changelog does not prove artifact identity, supply-chain provenance, security posture, packaged acceptance or independent release acceptance.
- Historical malformed/wrapped lines remain exact provenance; normalization must not rewrite source history.
- Any changelog identity/content drift, selected release-owner drift, PR/ref drift or upstream lease drift requires fresh reconciliation.

## Traceability

- Requirement: `TDRP-R9-UPSTREAM-ROOT-CHANGELOG-RELEASE-HISTORY-PROVENANCE-001`.
- Oracles: `ORA-TDRP-CHANGELOG-EXACT-SOURCE-001`, `ORA-TDRP-CHANGELOG-STRUCTURAL-ACCOUNTING-001`, `ORA-TDRP-CHANGELOG-CANONICAL-RELEASE-AUTHORITY-001`.
- Invariants: `INV-TDRP-CHANGELOG-NO-SECOND-RELEASE-OWNER-001`, `INV-TDRP-CHANGELOG-HISTORY-NOT-CURRENT-PRODUCT-PROOF-001`, `INV-TDRP-CHANGELOG-NO-RELEASE-ACCEPT-001`.

Closing strict109 yields unknown=16,267, unread=16,242, omitted=0, responsibilities=127. Next strict root index 110 is `cmake@7ceca3019b4750d7a5acb2de0faffd523ee14095` (gitlink to `desktop-app/cmake_helpers`, tree `f5c386cb34208309aae521187611fac4a2045577`). Fresh child exact-head TDRP/Rust evidence is required.
