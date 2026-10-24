# Microsoft/GSL .gitignore complete read — strict138

## Authority
- Accepted tdesktop upstream: telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Root gitlink: Telegram/ThirdParty/GSL resolves exactly to Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37.
- GSL child tree: 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- Atomic child source: .gitignore@326971f0331d1f8c3226fb3b341687f3198cd8a2, fully read: 16 lines.
- Parent Fabushi exact head: d4a4ed2b9015996faf0da7160e762c7d49cf8aa0.
- Parent exact-head evidence: TDRP run 40007031379 61/61 success; authority 12525400642 sha256:9e70fff0a3745ca742eda4a20f4bd66ff1aa3cb4f6c57dfdd7d4abe7a3ba64af; closure 12524942439 sha256:8803e478e54a2f7b6d47ea8cb485faa0742770df2f79794ec47d939ae6d84141; reachability 12524133934 sha256:4de9b391641df25df9cdff3f7443636c8308927de560f7d2dfb5b8cdd5cb20e5; Rust run 40007031384 6/6 success; artifact 12524826812 sha256:72fdc8945dbcb7ae91c0a235aadbc78f4162d3850258336dfef9a9c633f30755.
- Submission lease basis: live main 7394f6dc7687f9b986ee5589d40aa680ce23b90a; live upstream remains a7f453612f4896ac04d0007a36ab0d377e5bcbcf/c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Next GSL root non-directory entry: CMakeLists.txt@b1d24e5facfe56b1ae28468a5959797e9f6bacda.

## Exact source
The exact file contains 16 ordered ignore lines:
1. CMakeFiles
2. build
3. tests/CMakeFiles
4. tests/Debug
5. *.opensdf
6. *.sdf
7. tests/*tests.dir
8. *.vcxproj
9. *.vcxproj.filters
10. *.sln
11. *.tlog
12. Testing/Temporary/*.*
13. CMakeCache.txt
14. *.suo
15. .vs/
16. .vscode/

## Responsibility decomposition
The applicable responsibility is repository-local/generated-state exclusion provenance:
- CMake/build local outputs: CMakeFiles, build, CMakeCache.txt.
- test local/generated/temporary outputs: tests/CMakeFiles, tests/Debug, tests/*tests.dir, Testing/Temporary/*.*.
- Visual Studio local/project/cache outputs: *.opensdf, *.sdf, *.vcxproj, *.vcxproj.filters, *.sln, *.tlog, *.suo, .vs/.
- VS Code local editor state: .vscode/.

These rules affect ordinary discovery/addition of matching untracked files. They do not delete bytes, untrack tracked source, authorize source-inventory omission, prove a compiler/test runner executed, establish product persistence, or supply release acceptance.

## Existing-owner-first / applicability
Fabushi reuses the already accepted TDRP-R9-REPOSITORY-LOCAL-STATE-EXCLUSION-PROVENANCE-001 owner: AGENTS.md source-neutral repository development-environment and source-integrity governance, with exact-head source-authority validators as the verification boundary. No Microsoft/GSL/Telegram ignore runtime, second build graph, product-persistence owner, IDE owner or C++ runtime is introduced.

## Fail-closed
The accepted tdesktop gitlink, exact GSL child commit/tree and exact .gitignore blob form one immutable provenance chain. Drift at any layer invalidates dependent evidence. An ignore match cannot turn tracked or recursively required source into omitted source, and ignored build/test-temporary state cannot become build/test execution evidence. Partial reads, stale gitlink resolution, unsupported ignore inference or cancelled publication remain non-passing.

## Traceability
Requirement: TDRP-R9-GSL-GITIGNORE-LOCAL-STATE-EXCLUSION-PROVENANCE-001.

Oracles: ORA-TDRP-GSL-GITIGNORE-SOURCE-001, ORA-TDRP-GSL-GITIGNORE-OWNER-001, ORA-TDRP-GSL-GITIGNORE-TRACKED-SOURCE-001, ORA-TDRP-GSL-GITIGNORE-NON-ACCEPTANCE-001.

Invariants: INV-TDRP-GSL-GITIGNORE-NO-SOURCE-OMISSION-001, INV-TDRP-GSL-GITIGNORE-NO-BUILD-TEST-PROOF-001, INV-TDRP-GSL-GITIGNORE-NO-PRODUCT-PERSISTENCE-001, INV-TDRP-GSL-GITIGNORE-EXACT-PROVENANCE-001.

Closing only strict138 changes counters to unknown=16,238, unread=16,213, omitted=0, responsibilities=156. baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain mandatory.

Next strict139: Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37 CMakeLists.txt@b1d24e5facfe56b1ae28468a5959797e9f6bacda.

## Current exact-head revalidation
- Revalidated Fabushi head: f3c367e3be93e432806ad47d8efa0ca8c97bd13a.
- TDRP Revision 9 source authority run 40009202091 completed success with all 30 jobs successful.
- Authority artifact 12525666806 digest sha256:2cf23f9f6f0563b99db9468b1c38a518a3a3a30dacac00ab706178225a2440e5.
- Recursive closure artifact 12525537974 digest sha256:ae748f8007453733bd2253c0e28c82d998ecc261463081411265cc6d25e0f38a.
- Recursive reachability artifact 12525673116 digest sha256:8aebb6db8661f59bf455868129781b9c07db5e9ae644533a598a06debb199ba7.
- Rust desktop runtime run 40009201979 completed success with all 6 jobs successful; rust messaging artifact 12525965803 digest sha256:69a2025effc64e9d76137e6223935093b104878c34203526e8e3e297734bf31b.
- This revalidation confirms the strict138 owner-anchor repair at f3c367e3be93e432806ad47d8efa0ca8c97bd13a. It does not change source-completeness counters or acceptance state: unknown=16,238, unread=16,213, omitted=0, baseline_identity_accepted=false, source_closure_ready=false, independent release acceptance still open.
