# Microsoft/GSL .gitattributes complete read — strict137

## Authority
- Accepted tdesktop upstream: telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Root gitlink: Telegram/ThirdParty/GSL resolves exactly to Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37.
- GSL child tree: 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- Atomic child source: .gitattributes@3455dc9da5064e1a54deba6e2a5869aae2442d42, fully read; exact content is one line: include/gsl/* linguist-language=C++.
- Parent Fabushi exact head: 29c96fa1b6545c036cf9a64aa029c225c47b2993.
- Parent exact-head evidence: TDRP 40000624296 61/61; authority 12521827806 sha256:9d78369e74e503a0d230fc299b32d5ddcc94e79dd784960d0bdc2f761b09d409; closure 12521678388 sha256:95215a88e5c9273432c2f417921e08f46fb63bda7728b8de2faf4982aec0fcdd; reachability 12523150003 sha256:55b5e261780d4f6706250d8d6ae066ad4024d174fbb470d5eb888cafe2b480c7; Rust 40000624188 6/6; artifact 12521827593 sha256:03b66a7480de76a8f5e10bfc948e9a71316a454cd1f3863f7d728c88cc6a1ea0.
- Submission lease basis: live main 7394f6dc7687f9b986ee5589d40aa680ce23b90a; live upstream remains a7f453612f4896ac04d0007a36ab0d377e5bcbcf/c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Next GSL root non-directory entry is .gitignore@326971f0331d1f8c3226fb3b341687f3198cd8a2.

## Semantics
The file declares exactly one path-scoped Git attribute: matching include/gsl/* files are assigned linguist-language=C++.

GitHub Linguist docs/overrides.md@9d56af54d3e01675fc269fa97b01cbb971adf464 states that .gitattributes Linguist overrides, including linguist-language, are used to determine language statistics and syntax-highlight files. This bounds the source responsibility to repository language-classification/display metadata.

The file does not declare text, eol, working-tree-encoding, filter, diff, merge, LFS patterns, build flags, compiler selection, runtime behavior, packaging rules, security policy or release acceptance. None of those may be inferred from this source.

## Existing-owner-first / applicability
Fabushi reuses the existing TDRP exact-head source-authority and repository quality/test-evidence governance. No Microsoft/GSL/Telegram metadata runtime, second C++ owner, Git worktree-byte owner or product surface is introduced.

The applicable responsibility is exact provenance plus evidence-strength separation: preserve the declared metadata, reject unstated Git materialization/build/runtime semantics, and keep exact-head GitHub Actions stronger than hosted language-classification output.

## Fail-closed
The accepted tdesktop gitlink, GSL child commit/tree and exact .gitattributes blob form one immutable provenance chain. Drift at any layer invalidates dependent evidence.

A language-statistics/highlighting result cannot prove compilation, tests, runtime reachability, source closure, package identity, security or release acceptance. Missing/partial reads, stale gitlink resolution, unsupported attribute inference or cancelled publication remain non-passing.

## Traceability
Requirement TDRP-R9-GSL-GITATTRIBUTES-LANGUAGE-METADATA-001.

Oracles: ORA-TDRP-GSL-GITATTRIBUTES-SOURCE-001, ORA-TDRP-GSL-GITATTRIBUTES-LINGUIST-SEMANTICS-001, ORA-TDRP-GSL-GITATTRIBUTES-OWNER-001, ORA-TDRP-GSL-GITATTRIBUTES-UNSUPPORTED-INFERENCE-001.

Invariants: INV-TDRP-GSL-GITATTRIBUTES-NO-WORKTREE-BYTE-INFERENCE-001, INV-TDRP-GSL-GITATTRIBUTES-NO-CXX-RUNTIME-001, INV-TDRP-GSL-GITATTRIBUTES-METADATA-NOT-ACCEPTANCE-001, INV-TDRP-GSL-GITATTRIBUTES-EXACT-PROVENANCE-001.

Closing only strict137 changes counters to unknown=16,239, unread=16,214, omitted=0, responsibilities=155. baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain mandatory.

Next strict138: Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37 .gitignore@326971f0331d1f8c3226fb3b341687f3198cd8a2.
