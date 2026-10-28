# Microsoft/GSL GSL.natvis complete read — strict142

## Authority
- Accepted tdesktop upstream: telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Root gitlink: Telegram/ThirdParty/GSL resolves exactly to Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37.
- GSL child tree: 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- Atomic child source: GSL.natvis@965888cd9c70d7a19401aca4252c8a4a01c456c2, fully read: 32 line slots / 1159 bytes.
- Parent Fabushi exact head: 078d466c07ffcd90b4bd94fed59e1fa27086191d, tree 2ffdd3442f734d49a8a7e432f4608d26ce26a9af.
- Parent exact-head evidence: TDRP 40019138159 61/61 success; authority 12530131399 sha256:d27801a4cb0543097628906622a9c717b5a14f31bf4eb6dc04eb506331061508; closure 12530067154 sha256:fcc8f1d0d4fea641db080e780fa57c7061fa78a9117cf38d080d22bfbcf30260; reachability 12530376354 sha256:708ae333dafc8037a2ea256925f2389d0728d958d46fe0788e0e29e4387d3a4d; Rust 40019138234 6/6 success; artifact 12529752451 sha256:654c404c4b1757cb5ed7054c7e0d9f31874a4f6de36377a5d481fd6a5a6a3e39.
- Submission lease: live main 7394f6dc7687f9b986ee5589d40aa680ce23b90a/tree fbeb0bb32d944d0c1053880c471fe5c52279f052; live upstream a7f453612f4896ac04d0007a36ab0d377e5bcbcf/tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Previously closed CMakeLists.txt@b1d24e5facfe56b1ae28468a5959797e9f6bacda establishes GSL.natvis as a BUILD_INTERFACE source and install asset.
- Next GSL root non-directory entry: LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02.

## Exact source
The NatVis XML defines Visual Studio debugger projections for three GSL type families. final_action displays invoke_ and f_ and expands them as invoke/callback; span displays storage_.size_ as extent and exposes ArrayItems using storage_.size_ plus storage_.data_; not_null displays the dereferenced ptr_ value under the source comment that its non-null invariant permits that debugger dereference. These expressions depend on GSL private C++ field layout and are debugger presentation metadata, not runtime protocol, storage schema or ABI authority.

The already-closed CMake composition makes this file available in the GSL build interface and installation payload. That distribution relationship remains developer/build-package provenance: presence of the file does not prove a configured build, executed test, package identity, runtime behavior, memory safety or release acceptance. The separate LICENSE entry remains independently open.

## Existing-owner-first / applicability
Fabushi reuses AGENTS.md Repository development-environment adapter governance for debugger/editor conveniences, desktop/package.json canonical build/package composition for any deliberately adopted developer asset distribution, and FQT-EVID-001 source-derived/configured-build plus exact-head evidence boundaries. The applicable source-neutral responsibility is optional, source-reconstructible developer debugging support without creating shipping product state. No current Fabushi requirement justifies copying GSL private C++ field layout, retaining a Visual Studio NatVis runtime, or creating a Microsoft/GSL/Telegram debugger or package owner.

## Fail-closed
GSL.natvis cannot prove final_action callbacks execute, span size/data are valid, not_null is safe at runtime, Rust/TypeScript types share the C++ layout, a configured target consumed the asset, packaging succeeded, or release acceptance passed. The debugger dereference is display logic, not a memory-safety control. Missing or disabled IDE visualization is not a passing build, and branch/gitlink/tree/blob drift, partial read, unsupported layout inference or cancelled publication is non-passing.

## Traceability
Requirement: TDRP-R9-GSL-NATVIS-DEBUGGER-VISUALIZATION-PROVENANCE-001.
Oracles: ORA-TDRP-GSL-NATVIS-SOURCE-001, ORA-TDRP-GSL-NATVIS-OWNER-BOUNDARY-001, ORA-TDRP-GSL-NATVIS-RUNTIME-BOUNDARY-001, ORA-TDRP-GSL-NATVIS-LAYOUT-BOUNDARY-001, ORA-TDRP-GSL-NATVIS-EVIDENCE-BOUNDARY-001.
Invariants: INV-TDRP-GSL-NATVIS-NO-SECOND-DEBUGGER-OWNER-001, INV-TDRP-GSL-NATVIS-CXX-LAYOUT-NOT-PRODUCT-CONTRACT-001, INV-TDRP-GSL-NATVIS-NOT-EXECUTION-EVIDENCE-001, INV-TDRP-GSL-NATVIS-NOT-SAFETY-PROOF-001, INV-TDRP-GSL-NATVIS-EXACT-PROVENANCE-001.

Closing only strict142 changes counters to unknown=16,234, unread=16,209, omitted=0, responsibilities=160. baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain mandatory.

Next strict143: Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37 LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02.
