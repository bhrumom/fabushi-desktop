# Strict169 — Microsoft/GSL span compatibility test provenance

## Exact source authority

- Fabushi PR #154 parent exact HEAD: `feff59d3cc7f683d169bf96114ce9ddfdb858d1c`; parent tree: `1c861c683004e17bfc9a1e2d54f36be7c6719cc7`.
- Live Fabushi main: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Locked `telegramdesktop/tdesktop@dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- `Telegram/ThirdParty/GSL` gitlink commit: `87f9d768866548b5b86e72be66c60c5abd4d9b37`; Microsoft/GSL child tree: `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`. The exact recursive child tree has 51 entries and GitHub reports `truncated=false`.
- Source `tests/span_compatibility_tests.cpp`: blob `c6e4b02900b22aeb0ee53262377624333a9ac577`, 1,024 lines / 60,486 bytes.
- Header `include/gsl/span`: blob `d2ef9f7d85ca6e25bf69b001e9308b3d069c5d16`, 864 lines / 31,778 bytes.
- Included dependencies read at exact blobs: `include/gsl/byte@87d007ef94d197632872e96d91525a013c665da8` (184 lines / 5,962 bytes); `include/gsl/assert@d3e6ccdd53d5a254ed3827cf07711a6e86bb4226` (134 lines / 3,988 bytes).
- Build registration `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a`, 11,090 bytes: `span_compatibility_tests.cpp` is registered in `gsl_tests` at line 208 and CTest registers `gsl_tests` at line 221.
- License `LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02`: MIT license confirmed.

## Source behavior and negative oracle

This is a dependency-test source, not shipping product code. It contains one runtime GTest case, `span_compatibility_tests.assertion_tests`, with 162 `EXPECT_TRUE` assertions in the test body (source lines 72–773). It exercises default/dynamic/fixed-extent construction; pointer/count and pointer/pointer construction; raw arrays, `std::array` and container conversion; copy/assignment; iterator dereference, indexing, random-access and reverse iteration; `first`/`last`/`subspan`; `data`/`size`/`empty`; and byte views. A span is non-owning: this test does not establish production ownership or lifetime safety.

The full file has 286 `static_assert` expressions: 67 negative constructibility probes, one negative array-pointer convertibility probe, and two positive plus two negative `as_writable_bytes` availability probes. The compile-time matrix also asserts conversion and `noexcept` properties across dynamic/fixed extents and const/non-const element types. There are no `EXPECT_DEATH`/`ASSERT_DEATH` or `EXPECT_THROW`/`ASSERT_THROW` assertions in this file; its negative oracle is compile-time, not a runtime crash oracle. The neighboring `span_tests.cpp` has separately recorded bounds/death-test evidence and is not reclassified here.

The existing `include/gsl/span` implementation owns the upstream template surface: `span<ElementType, Extent>`, `span_iterator`, extent conversion, constructors, bounded `first`/`last`/`subspan`, `operator[]`/`front`/`back`, and `as_bytes`/`as_writable_bytes`. `include/gsl/assert` supplies `Expects` preconditions and `include/gsl/byte` supplies the byte representation type. This dossier does not claim those C++ contracts have been implemented in Fabushi production.

## Caller and existing-owner boundary

An exact-name search for `span_compatibility_tests.cpp` in the locked tdesktop tree returned no direct production reference. A bounded exact-ref `gsl::span` search returned 15 consumer files; each file was fetched in full at `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` and its blob SHA verified:

- `Telegram/SourceFiles/ui/chat/attach/attach_album_preview.h@05084d02a2baafbadfe6ba7da96c69007e6fcf0f`: AlbumPreview and prepareThumbs borrow PreparedFile ranges; attachment/resource owner remains open.
- `Telegram/SourceFiles/media/audio/media_child_ffmpeg_loader.h@65354d967ef06442759cef55236d8867fdbe3755`: ExternalSoundPart borrows FFmpeg packet ranges; media/audio owner remains open.
- `Telegram/SourceFiles/mtproto/details/mtproto_dc_key_creator.h@744a18ace5315806fd4d15be9e6b8d1b8283fc5f`: DC key creator parses response-word ranges; networking/protocol owner remains open.
- `Telegram/SourceFiles/iv/editor/iv_editor_commands.cpp@cbeaf8cdc2300519346af9fbf42f8c872db49daa`: RunFirst receives a command-chain span; editor command owner remains open.
- `Telegram/SourceFiles/media/streaming/media_streaming_audio_track.h@1863f3d9f85a06154ddabd9d809edd7db8b6d190`: AudioTrack exposes packet span to mixer enqueue; media streaming owner remains open.
- `Telegram/SourceFiles/mtproto/connection_abstract.h@217b684fa222c42e968dfddeaa62f16a0054017f`: AbstractConnection declares a borrowed parseNotSecureResponse result; protocol owner remains open.
- `Telegram/SourceFiles/boxes/peers/peer_short_info_box.h@16fa600b9384c69c9f1057cd390c485a5b0ad235`: PeerShortInfoCover exposes fixed-extent roundMask view; participant/profile owner remains open.
- `Telegram/SourceFiles/mtproto/connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a`: implements parseNotSecureResponse over the protocol buffer; protocol owner remains open.
- `Telegram/SourceFiles/ui/chat/attach/attach_album_preview.cpp@653f4e137610e0ecec3758775b79e4e0e7d93d8a`: AlbumPreview prepares thumbnails from a borrowed PreparedFile range; attachment/resource owner remains open.
- `Telegram/SourceFiles/settings/sections/settings_websites.cpp@0d7dced4529eba860b2cf75933b452cff99b981b`: Website settings list controller receives EntryData ranges; Settings owner remains open.
- `Telegram/SourceFiles/media/streaming/media_streaming_audio_track.cpp@36d6b68cd4d47b1665beed2634a36f196b220cbd`: AudioTrack::mixerEnqueue consumes packet ranges; media streaming owner remains open.
- `Telegram/SourceFiles/boxes/peers/peer_short_info_box.cpp@d8afaac623c4568db56c14a9cca56dfb2043873f`: PeerShortInfoCover returns a fixed four-image view; participant/profile owner remains open.
- `Telegram/SourceFiles/mtproto/details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03`: readNotSecureResponse and handleAnswer consume response-word ranges; networking/security owner remains open.
- `Telegram/SourceFiles/history/view/media/history_view_poll.cpp@8e2f8e2d3bd820d9635ec0efd16b0ed91adcde9e`: poll percentage helpers consume vote/result spans; Conversation/Transcript poll owner remains open.
- `Telegram/SourceFiles/settings/sections/settings_active_sessions.cpp@27616cbf6f76c2df1efd9e8462234839d346abd8`: Active-session settings list controller receives EntryData ranges; Settings owner remains open.

These hits establish existing product-owner candidates, not closure. Existing-owner-first mapping remains: attachment preview/resource and composer flows; media/audio streaming; networking, MTProto and credential/session handling; editor command chains; participant/profile image projections; Settings lists; and Conversation/Transcript poll rendering. Native Rust slices/references/arrays and borrowing are the target low-level view model inside the relevant canonical owner. Do not create `GslSpanRuntime`, `TelegramSpanRuntime`, a second messaging/runtime owner, or a new product owner.

## Non-duplication and completion boundary

The existing responsibility `TDRP-R9-GSL-SPAN-VIEW-CONTRACT-DEPENDENCY-001` already references this exact file as a test dependency/evidence for the span header contract. The ledger had no dedicated source-symbol responsibility or standalone source-analysis dossier for `tests/span_compatibility_tests.cpp`. This new row separately accounts for the test file's compile-time oracles, registered test-runner provenance, and source/license identity; it does not duplicate the span implementation row or mark its production responsibilities complete.

- Responsibility: `TDRP-R9-GSL-SPAN-COMPATIBILITY-TEST-PROVENANCE-001`
- Target-symbol anchor: `TDRP_GSL_SPAN_COMPATIBILITY_TEST_PROVENANCE_001`
- Source-only status: mapped; dependency-test-provenance-mapped-open.
- Production entrypoints: none. Production evidence: none.
- Microsoft/GSL C++ test execution: not claimed.
- Parent exact-head TDRP Revision 9 run `40069266397`: success, 61/61 jobs. Authority artifact `12550397537`, `sha256:a3c8c992f9b1d6903656ed09c14b08fa4e7373b2da5adccc6ab8480b4d3f0282`; recursive closure artifact `12550667890`, `sha256:07211f2c7d9159f72f6b7ce55d2d533cfd558e7e3f58261e6ae2d36811c05709`; recursive reachability artifact `12550752390`, `sha256:7b1687e2570bceca63125f493ffc636dd3b615ffdef13b7d8f284c7e093e75ef`; Boost reachability artifact `12550143228`, `sha256:e03ca2cace2b244d8f6a0972e3a9dcb638f30355a03d0e922bb7c2660107ed87`.
- Parent exact-head Rust desktop runtime run `40069266413`: success, 6/6 jobs; artifact `12549749174`, `sha256:75b856041439c54c14cc0667c42f93de2b3e1cb182feb9f36e2f69b89993aa29`.
- Parent exact-head Official MCP marketplace run `40069266392`: success.
- The `feff59d3cc7f683d169bf96114ce9ddfdb858d1c` artifacts below are parent-only. The follow-up exact-head runs on `ad606839280dfc0718b0138061d8beff78ef8624` subsequently completed successfully; the receipt commit itself still requires fresh exact-head Actions.

## Coverage after strict169

- Recursive source entries: 16,379
- Unknown: 16,207
- Unread: 16,182
- Omitted: 0
- Responsibility records: 187
- Baseline identity accepted: false
- Source closure ready: false
- Independent release acceptance: not accepted

## Traceability anchors

- Requirement: `TDRP-R9-GSL-SPAN-COMPATIBILITY-TEST-PROVENANCE-001`
- Oracles: `ORA-TDRP-GSL-SPAN-COMPATIBILITY-EXACT-SOURCE-001`, `ORA-TDRP-GSL-SPAN-COMPATIBILITY-COMPILE-TIME-NEGATIVES-001`, `ORA-TDRP-GSL-SPAN-COMPATIBILITY-CMAKE-REGISTRATION-001`, `ORA-TDRP-GSL-SPAN-COMPATIBILITY-NO-DIRECT-PRODUCTION-REFERENCE-001`
- Invariants: `INV-TDRP-GSL-SPAN-COMPATIBILITY-TESTS-ONLY-001`, `INV-TDRP-GSL-SPAN-COMPATIBILITY-NO-PARALLEL-RUNTIME-001`, `INV-TDRP-GSL-SPAN-COMPATIBILITY-NO-CALLER-CLOSURE-001`


## Round 74 — failure diagnosis, repair, and fresh parent-head evidence

The previously requested exact HEAD `4dc0596b3dfe56e7060f3cfd5f7671174a90ef77` reached terminal states: TDRP Revision 9 run `40068834594` = `failure`; Rust desktop runtime run `40068834616` = `cancelled`. The complete TDRP failed-job log for job `120797675441` identifies the concrete cause: `scripts/validate-tdrp-revision9-authority.mjs` rejected responsibility `TDRP-R9-GSL-OWNER-TEST-PROVENANCE-001` because target symbol `TDRP_GSL_OWNER_TEST_PROVENANCE_001` was absent from the dossier target text; artifact upload then failed because the validator had not produced `artifacts/tdrp-authority`. The Rust `rust-host` job `120797675895` was cancelled during step 14 (`Test Grok prompt attachment projection contract`); its log ends with `The operation was canceled`, so this run is not counted as a Rust test failure or pass.

The minimal anchor repair is present in commit `feff59d3cc7f683d169bf96114ce9ddfdb858d1c` (`Fix strict168 GSL owner dossier target-symbol anchor`). The live PR branch then advanced to `ad606839280dfc0718b0138061d8beff78ef8624`, which contains strict169 and the synchronized ledger/inventory/lock records. Fresh Actions bound to that exact parent head are terminal-success:

- TDRP Revision 9: run `40071215021`, 61/61 jobs success. Source-authority artifact `12551093555`, digest `sha256:8003c3c34e85706215ffe85f6e7244d01c53aed4741afec929177537d25bc745`; recursive-closure artifact `12551208445`, digest `sha256:ce286229e6dc06c3847390b6cb59b049ad36d864b15bdaf43dd937202ad54db3`; recursive-reachability artifact `12551700822`, digest `sha256:eb270e7969cec36209b4944e48456896df290828c4c16e2a39259732d280c0de`; Boost-reachability artifact `12551367961`, digest `sha256:a61ee3c34da5b5123bfe7bd5a78d1443eb4927921aed85b830e263a27b52c368`.
- Rust desktop runtime: run `40071215018`, 6/6 jobs success; artifact `12551053687`, digest `sha256:2cf02ec88b8882fb92198e54c1d5ec33d46963026f01e0b418f8124daaaabb07`.
- Official MCP marketplace: run `40071215073`, 1/1 job success.

The exact source was re-read at Microsoft/GSL commit `87f9d768866548b5b86e72be66c60c5abd4d9b37`: `tests/span_compatibility_tests.cpp` blob `c6e4b02900b22aeb0ee53262377624333a9ac577` (1,024 lines / 60,486 bytes), `include/gsl/span` blob `d2ef9f7d85ca6e25bf69b001e9308b3d069c5d16` (864 lines / 31,778 bytes), `tests/CMakeLists.txt` blob `9de2519b366d6bc6803329a51afe459b0a6a357a` (registration line 208; CTest line 221), and MIT `LICENSE` blob `aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02`. All 15 caller files listed above were fetched again at locked tdesktop commit `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`; every returned blob SHA matched this dossier. The locked tdesktop recursive tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5` is not truncated and pins `Telegram/ThirdParty/GSL` as gitlink `87f9d768866548b5b86e72be66c60c5abd4d9b37`; that GSL commit resolves to tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`, 51 entries, `truncated=false`.

These CI artifacts validate the repository's source-authority, runtime, and marketplace gates only. They do not claim that Microsoft/GSL's C++ test ran in Fabushi CI, that any of the 15 production caller responsibilities is closed, or that baseline identity/source closure/release acceptance is accepted. Coverage remains 16,379 recursive entries, unknown=16,207, unread=16,182, omitted=0, responsibilities=187; baseline identity accepted=false, source closure ready=false, independent release acceptance=not accepted. This section is parent-head evidence; the receipt commit below needs its own exact-head Actions.