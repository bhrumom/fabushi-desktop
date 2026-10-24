# Strict171 — Microsoft/GSL span extension tests source provenance

## Exact authority

- PR #154 branch `tdrp-r9-entry31-includes-r27-20261020`, parent exact HEAD `9b83aaf942ec850ad13fd91d2ca28245b4128e05`, tree `25182cc1139b228551ae41bc682733f4b65a7968`.
- Live Fabushi main: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Locked `telegramdesktop/tdesktop@dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- `Telegram/ThirdParty/GSL` gitlink: `87f9d768866548b5b86e72be66c60c5abd4d9b37`; child tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`.
- Exact source `Microsoft/GSL:tests/span_ext_tests.cpp@dfd2725db3612de97c4a3173f5f168131f7341f3`: 381 lines / 11,004 bytes, fully re-read.
- Related pinned blobs re-read: `include/gsl/span_ext@2a7664c70550477e9ff228d6370ff635b5217c81`, `include/gsl/span@d2ef9f7d85ca6e25bf69b001e9308b3d069c5d16`, `include/gsl/util@26b2f5fb9d0d8c28c3096f50d575eb42a4b89bad`, `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a`, root `CMakeLists.txt@b1d24e5facfe56b1ae28468a5959797e9f6bacda`, MIT `LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02`.

## Source behavior and negative oracle

This is dependency-test provenance only. The source contains 12 GoogleTest cases covering `make_span` pointer/count, pointer/pointer, C array, dynamic array, std::array/const std::array and container construction; `gsl::at` interoperability; free iterator helpers; signed size; and comparison operators.

- Five active `EXPECT_DEATH` expressions were verified: one rejects `make_span(nullptr, 2)`; four reject negative and end-boundary `gsl::at` indices.
- There are no `ASSERT_DEATH`, `EXPECT_THROW`, `ASSERT_THROW`, or `static_assert` expressions in this source.
- `tests/CMakeLists.txt` registers `span_ext_tests.cpp` in `gsl_tests` at line 209 and CTest `gsl_tests` at line 221.
- `comparison_operators` is excluded under `GSL_KERNEL_MODE`; the std::array assignment block also preserves a compiler-specific GCC 6.4/6.5 optimized-build regression expectation. These are dependency-test variants, not Fabushi runtime owners.
- `deathTestCommon.h` supplies the terminate/death matcher. The pinned license is MIT.

## Existing-owner-first boundary

The test file is not a shipping owner and closes no product caller. The previously identified 15 exact-ref tdesktop `gsl::span` production consumers remain independently open across attachment/resource, media/audio, networking/protocol, editor, profile, settings/session and Conversation/poll owners.

Selected owner: existing TDRP source/dependency inventory and test-provenance governance. Actual view/lifetime semantics remain with native Rust slices/references/arrays and the real domain owners. Rejected: `GslSpanExtRuntime`, `TelegramSpanRuntime`, a cross-domain buffer/view root, or any source-named runtime. No UI, service, persistence or shipping runtime is added.

## Parent-head exact-head evidence

Parent HEAD `9b83aaf942ec850ad13fd91d2ca28245b4128e05` is terminal green:

- TDRP Revision 9 run `40077360069`: 61/61 jobs success; Source authority job `120823500154`.
- Authority artifact `12553573731`, `sha256:8727b23a22f27a9d2dd1fe9f9dcf48ae2f2b6ac9a75472b4c3556e559ba4a52a`.
- Recursive closure `12553947186`, `sha256:cb90dbf3920cc82081c822852261be45c9d4d9005c6e1743930123dcacff762f`.
- Recursive reachability `12553544238`, `sha256:e7fbe758124e58ec01e1735541d0ca18e1a76889690bd6498bee52e978f247bb`.
- Boost reachability `12553832502`, `sha256:bce380578a2f394498f2f5a32a0e03fd6b7cee54d3fbc5e11e169f989fe0fa43`.
- Rust desktop runtime `40077360156`: 6/6 jobs success; artifact `12553349416`, `sha256:ccce53d6c36040cc984035e55c29acb5eed9b608f0dd2bef038f867e4ec29103`.
- Official MCP marketplace `40077360113`: success.

These are parent-head evidence only. They do not claim Microsoft/GSL C++ tests ran in Fabushi CI, prove production implementation, or close downstream callers. The receipt commit requires fresh exact-head Actions.

## Coverage after strict171

- Recursive source entries: 16,379
- Unknown: 16,205
- Unread: 16,180
- Omitted: 0
- Responsibility records: 189
- Baseline identity accepted: false
- Source closure ready: false
- Independent release acceptance: not accepted

Requirement: `TDRP-R9-GSL-SPAN-EXT-TEST-PROVENANCE-001`. Target anchor: `TDRP_GSL_SPAN_EXT_TEST_PROVENANCE_001`.

## Traceability anchors

- Requirement: `TDRP-R9-GSL-SPAN-EXT-TEST-PROVENANCE-001`
- Oracles: `ORA-TDRP-GSL-SPAN-EXT-TEST-EXACT-SOURCE-001`, `ORA-TDRP-GSL-SPAN-EXT-TEST-DEATH-ORACLES-001`, `ORA-TDRP-GSL-SPAN-EXT-TEST-CMAKE-REGISTRATION-001`
- Invariants: `INV-TDRP-GSL-SPAN-EXT-TESTS-ONLY-001`, `INV-TDRP-GSL-SPAN-EXT-TEST-NO-PARALLEL-RUNTIME-001`, `INV-TDRP-GSL-SPAN-EXT-TEST-NO-CALLER-CLOSURE-001`

Source-only provenance; no production-completion or release-acceptance claim.
