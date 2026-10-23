# Microsoft/GSL ThirdPartyNotices.txt complete read — strict146

## Authority
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41
- ThirdPartyNotices.txt@552b254c63a2f9909aa46ef21c256ef9a79a0035, fully read: 42 line slots / 2022 bytes
- Parent exact head a6343d125980a6b98b8149bb368e2a1897262ed0, tree 5d9f69e7997607274880e5ea41ceb44155750881
- Parent TDRP 40027474728 61/61 success; authority 12533327369 sha256:61a152569fbf509dd0802cfb67d0dd1fe2a23cd324afce7726c596797ce85dd5; closure 12533826417 sha256:9c55fd1a56b7e47ff18e8e29c5de516bcb1d2fbad59d8815615c412b240629af; reachability 12532699819 sha256:2622a369d496bc270de02259e8bb2cbd611c0e4bd1e8b80f6fbda2c9483956e0; Rust 40027474784 6/6 success; artifact 12533346672 sha256:2bf97e87b48eb520cf069d28f743dda111780af80ab07839f086eaa73be820aa
- Next: Microsoft/GSL docs/headers.md@11dd0cdfd5840cbcb7d8538e49efa2ccd5ac6a68

## Exact source and disposition
The locked notice states that GSL incorporates third-party material and names Google Test (Google Inc., source github.com/google/googletest) under BSD 3-Clause. It records source-redistribution notice retention, binary-redistribution notice reproduction, non-endorsement, and AS-IS warranty/liability disclaimer provenance. This is legal/source provenance for the accepted recursive dependency. It does not by itself prove that Fabushi ships, links, packages, executes, or redistributes Google Test, nor does it prove packaged-license/SBOM/release acceptance.

## Existing-owner-first
Reuse TDRP-SRC-03 documentation_or_license, the module-map dependency/license provenance mapping, and existing Fabushi CI/packaging/SBOM/release-license evidence lifecycle. No Google Test runtime, second dependency/package graph, source-specific legal store, or second release owner is introduced.

## Fail-closed
The notice is retained exactly as provenance, but any Fabushi redistribution obligation remains contingent on independently evidenced shipping/package composition. The notice cannot transitively close docs/headers.md, include/, tests/, runtime equivalence, SBOM/provenance, or independent release acceptance.

## Traceability
Requirement TDRP-R9-GSL-THIRD-PARTY-NOTICES-GOOGLETEST-BSD-PROVENANCE-001.
Oracles: ORA-TDRP-GSL-NOTICE-EXACT-SOURCE-001, ORA-TDRP-GSL-NOTICE-GOOGLETEST-BSD-PROVENANCE-001, ORA-TDRP-GSL-NOTICE-SHIPPING-SEPARATION-001, ORA-TDRP-GSL-NOTICE-NO-TRANSITIVE-CLOSURE-001.
Invariants: INV-TDRP-GSL-NOTICE-NO-OMISSION-001, INV-TDRP-GSL-NOTICE-NO-SHIPPING-INFERENCE-001, INV-TDRP-GSL-NOTICE-LEGAL-PROVENANCE-001, INV-TDRP-GSL-NOTICE-EXACT-PROVENANCE-001.
Counters after strict146: unknown=16,230 unread=16,205 omitted=0 responsibilities=164.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
