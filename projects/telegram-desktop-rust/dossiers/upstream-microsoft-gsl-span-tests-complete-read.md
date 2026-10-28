# Strict170 — Microsoft/GSL span tests source provenance

## Exact authority

- PR #154 branch `tdrp-r9-entry31-includes-r27-20261020`, parent exact HEAD `b504b7af993180b00a9c458326d4185d44d69c68`, tree `598098984d0ec9683362aaaf53cb7de76325faa3`. Live branch ref and PR head were re-read and matched before preparing the commit.
- Live Fabushi main: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Locked `telegramdesktop/tdesktop@dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- `Telegram/ThirdParty/GSL` gitlink commit: `87f9d768866548b5b86e72be66c60c5abd4d9b37`; child tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`, 51 entries, `truncated=false`.
- Exact source `Microsoft/GSL:tests/span_tests.cpp@d86458bd3bd8e5a1f1739a769436594de2a6b85a`: 1,384 lines / 36,875 bytes, fully re-read.
- Related pinned blobs: `include/gsl/span@d2ef9f7d85ca6e25bf69b001e9308b3d069c5d16` (864 lines); `include/gsl/span_ext@2a7664c70550477e9ff228d6370ff635b5217c81` (215 lines); `include/gsl/assert@d3e6ccdd53d5a254ed3827cf07711a6e86bb4226` (134 lines); `include/gsl/byte@87d007ef94d197632872e96d91525a013c665da8` (184 lines); `include/gsl/util@26b2f5fb9d0d8c28c3096f50d575eb42a4b89bad` (172 lines); `tests/deathTestCommon.h@7bf242393f7bcff62c12cd985a87824f48f7bb0b` (12 lines); `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a` (318 lines); root `CMakeLists.txt@b1d24e5facfe56b1ae28468a5959797e9f6bacda` (49 lines); MIT `LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02` (22 lines).

## Source behavior and negative oracle

This is dependency-test provenance only, not product code. The source contains 32 `TEST(span_test, ...)` cases spanning default/fixed/dynamic extents, pointer/count and pointer/pointer constructors, arrays and containers, constness/conversion, copy/move/assignment, first/last/subspan, iterator identity, byte views, fixed-size conversions, regex/std-library interop, CTAD and front/back.

- 29 non-comment `EXPECT_DEATH` expressions are present; 2 are within `CONFIRM_COMPILATION_ERRORS` guards, so 27 are enabled by default. Three other death expressions are comments, not active oracles.
- 17 `#ifdef CONFIRM_COMPILATION_ERRORS` sections are disabled by default; their compile-error probes must not be reported as executed default tests.
- Three `static_assert` expressions at lines 1320, 1325 and 1331 assert CTAD result types. There are no `EXPECT_THROW` / `ASSERT_THROW` assertions.
- The tests cover invalid extent/range construction, bounds on first/last/subspan/index/front/back, cross-range iterator comparisons, and mutable/const byte-view constraints. C++ standard and feature gates choose alternate iterator death expressions and conditionally enable std::span coverage.
- `tests/CMakeLists.txt` includes `span_tests.cpp` in `gsl_tests` at line 210 and registers CTest `gsl_tests` at line 221. Root `CMakeLists.txt` gates `add_subdirectory(tests)` through `GSL_TEST` at lines 12 and 22–24. `deathTestCommon.h` supplies death-test terminate-handler/message helpers. The pinned license is MIT.

## Existing-owner-first boundary

The test file has no direct product consumer. The existing `include/gsl/span` contract and 15 exact-ref tdesktop `gsl::span` consumers remain distinct. Caller blobs were re-read at locked tdesktop commit `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`:

- `Telegram/SourceFiles/ui/chat/attach/attach_album_preview.h@05084d02a2baafbadfe6ba7da96c69007e6fcf0f`
- `Telegram/SourceFiles/ui/chat/attach/attach_album_preview.cpp@653f4e137610e0ecec3758775b79e4e0e7d93d8a`
- `Telegram/SourceFiles/media/audio/media_child_ffmpeg_loader.h@65354d967ef06442759cef55236d8867fdbe3755`
- `Telegram/SourceFiles/media/streaming/media_streaming_audio_track.h@1863f3d9f85a06154ddabd9d809edd7db8b6d190`
- `Telegram/SourceFiles/media/streaming/media_streaming_audio_track.cpp@36d6b68cd4d47b1665beed2634a36f196b220cbd`
- `Telegram/SourceFiles/mtproto/details/mtproto_dc_key_creator.h@744a18ace5315806fd4d15be9e6b8d1b8283fc5f`
- `Telegram/SourceFiles/mtproto/details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03`
- `Telegram/SourceFiles/mtproto/connection_abstract.h@217b684fa222c42e968dfddeaa62f16a0054017f`
- `Telegram/SourceFiles/mtproto/connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a`
- `Telegram/SourceFiles/iv/editor/iv_editor_commands.cpp@cbeaf8cdc2300519346af9fbf42f8c872db49daa`
- `Telegram/SourceFiles/boxes/peers/peer_short_info_box.h@16fa600b9384c69c9f1057cd390c485a5b0ad235`
- `Telegram/SourceFiles/boxes/peers/peer_short_info_box.cpp@d8afaac623c4568db56c14a9cca56dfb2043873f`
- `Telegram/SourceFiles/settings/sections/settings_websites.cpp@0d7dced4529eba860b2cf75933b452cff99b981b`
- `Telegram/SourceFiles/settings/sections/settings_active_sessions.cpp@27616cbf6f76c2df1efd9e8462234839d346abd8`
- `Telegram/SourceFiles/history/view/media/history_view_poll.cpp@8e2f8e2d3bd820d9635ec0efd16b0ed91adcde9e`

These consumers belong to existing attachment/resource, media/audio, networking/protocol, editor, profile, settings/session and Conversation/poll owners. Their state, lifecycle, persistence, concurrency, security, UI and product effects remain open independent production responsibilities. This row does not close those consumers.

Selected owner: existing TDRP source/dependency inventory and test-provenance governance. Rejected: `GslSpanRuntime`, `TelegramSpanRuntime`, cross-domain buffer/view root or source-named runtime. Actual Rust view/lifetime behavior belongs in the existing domain owners via slices/references/arrays and checked operations. Raw pointer-plus-length and byte projection require a separately proven narrow seam. No UI, service, persistence or shipping runtime is added.

## Parent-head CI evidence (not the receipt commit)

Parent HEAD `b504b7af993180b00a9c458326d4185d44d69c68` has terminal-success Actions: TDRP Revision 9 run `40073333943` (authority job `120811299872` success), Rust desktop runtime `40073333925` (6/6 success), Official MCP marketplace `40073333919` (1/1 success). Artifacts: authority `12552486326`, `sha256:1f0151c6d6a91b0ba8b8fb8838cda273c5ae5e34d1e24fcccb76f0520e388eb4`; recursive closure `12552128153`, `sha256:c72a144a856ce676a136ff4957d0ad2f7d20f59a6eb0328a8312fc7196a658c7`; recursive reachability `12551399837`, `sha256:aaf061ca0ba0b56759d3435ce2969b7f25374d94143e16a666ad48a59f658770`; Boost reachability `12552312150`, `sha256:185f2d86accf37126dfdec86bbf5a44c4d99167e93917c5c75f3cb7f91f0dc1e`; Rust artifact `12551814476`, `sha256:0dd721ba998d9293e346583575e7a7a280732f9de0e5c2201b07ce0a2925a5c2`.

These artifacts are parent-head-only evidence; they do not claim Microsoft/GSL C++ tests ran in Fabushi CI, prove production implementation or close callers. The receipt commit requires fresh exact-head Actions.

## Coverage after strict170

- Recursive source entries: 16,379
- Unknown: 16,206
- Unread: 16,181
- Omitted: 0
- Responsibility records: 188
- Baseline identity accepted: false
- Source closure ready: false
- Independent release acceptance: not accepted

Requirement: `TDRP-R9-GSL-SPAN-TEST-PROVENANCE-001`. Target anchor: `TDRP_GSL_SPAN_TEST_PROVENANCE_001`. Source-only provenance; no production-completion or release-acceptance claim.

## Traceability anchors

- Requirement: `TDRP-R9-GSL-SPAN-TEST-PROVENANCE-001`
- Oracles: `ORA-TDRP-GSL-SPAN-TEST-EXACT-SOURCE-001`, `ORA-TDRP-GSL-SPAN-TEST-DEFAULT-DEATH-ORACLES-001`, `ORA-TDRP-GSL-SPAN-TEST-GUARDED-COMPILE-ERRORS-001`, `ORA-TDRP-GSL-SPAN-TEST-CMAKE-REGISTRATION-001`
- Invariants: `INV-TDRP-GSL-SPAN-TESTS-ONLY-001`, `INV-TDRP-GSL-SPAN-TEST-NO-PARALLEL-RUNTIME-001`, `INV-TDRP-GSL-SPAN-TEST-NO-CALLER-CLOSURE-001`

These anchors cover source identity, enabled death-test and guarded compile-error distinctions, CMake/CTest registration, and the prohibition on treating dependency-test provenance as a parallel runtime or production caller closure.
