# Microsoft/GSL include/gsl/span complete read — strict155

## Authority
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41
- include/gsl/span@d2ef9f7d85ca6e25bf69b001e9308b3d069c5d16, fully read: 864 line slots / 31778 bytes
- tests/span_tests.cpp@d86458bd3bd8e5a1f1739a769436594de2a6b85a, fully read: 1384 line slots / 36875 bytes
- tests/span_compatibility_tests.cpp@c6e4b02900b22aeb0ee53262377624333a9ac577, fully read: 1024 line slots / 60486 bytes
- tests/span_ext_tests.cpp@dfd2725db3612de97c4a3173f5f168131f7341f3, fully read: 381 line slots / 11004 bytes as supplementary included-extension evidence; include/gsl/span_ext itself remains a separate open source entry
- Parent exact head 0c55e509de7caf01e4df3b183af214baa68a9151, tree 74cd54585e563abda9f388eefdcab778484866ab
- Parent TDRP 40042002309 61/61 success; authority 12540170216 sha256:51134beba021160f9a50a6176a59a7785fa6a6097084154a4b36d46403d4b1e8; closure 12539687359 sha256:c8f07f7f6bb8410517cb419928a932210dbfeeae1e8446ddfc8e8b13a0a3772f; reachability 12539738357 sha256:88275fb46bd4ce7300f46f9a1f4369ac0a2f3890a0b99e7bc3e78ef366bbcb14; Rust 40042002244 6/6 success; artifact 12539204325 sha256:ab6c324177652a0e074d1f7c3e7f44ee7be824a134bfd56fd5630e687fb575ad.
- Accepted exact-head tdesktop code search found 14 direct gsl::span consumer files: ui/chat/attach/attach_album_preview.{h,cpp}, media/audio/media_child_ffmpeg_loader.h, media/streaming/media_streaming_audio_track.{h,cpp}, mtproto/details/mtproto_dc_key_creator.{h,cpp}, mtproto/connection_abstract.{h,cpp}, iv/editor/iv_editor_commands.cpp, boxes/peers/peer_short_info_box.{h,cpp}, settings/sections/settings_websites.cpp, settings/sections/settings_active_sessions.cpp, and history/view/media/history_view_poll.cpp.
- Next: Microsoft/GSL include/gsl/span_ext@2a7664c70550477e9ff228d6370ff635b5217c81.

## Exact behavior
`gsl::span<ElementType, Extent>` is a non-owning contiguous view; it does not allocate or own the underlying elements and does not extend their lifetime. It supports fixed and `dynamic_extent` forms, pointer/count and pointer/pointer construction, arrays/std::array/compatible containers, and compatible span conversion. Fixed extents require an exact supplied length. `first`, `last`, `subspan`, `operator[]`, `front` and `back` enforce bounds/non-empty preconditions. Iterators are random-access views tied to the same underlying range; cross-range difference/order comparisons violate the contract. `as_bytes` exposes the same storage as a read-only byte view; `as_writable_bytes` is only available for non-const element views. The focused tests cover constructors/extent and CTAD, invalid extent/range death paths, slicing, iterator range identity, byte views, fixed/dynamic conversions, standard container interop and the C++20 std::span compatibility surface.

## Existing-owner-first
Use Rust `&[T]`, `&mut [T]`, `&[T; N]`, owned Vec/array storage in the actual domain owner, and safe slice operations such as checked `get/get_mut` or range slicing when the bounds are proven by the caller. Raw pointer+length conversion is permitted only at an unavoidable platform/FFI seam with explicit pointer, length, lifetime, alignment, aliasing and mutability proof. Typed-memory byte projection must use a representation-safe domain contract rather than a generic unchecked reinterpretation. Do not create a GSL SpanRuntime, cross-domain buffer/view owner, second storage root or C++-shaped compatibility subsystem.

## Consumer boundary
The 14 direct accepted tdesktop consumers span attachment preparation, audio packet streaming, protocol response parsing, editor command chains, settings/session lists, profile masks and poll percentage computation/rendering. `gsl::span` only supplies their bounded non-owning view contract. Those consumers keep their own behavior, state, lifecycle, persistence, concurrency, security, UI and product effects, so this row cannot close any of those domain responsibilities.

## Fail-closed
Fixed extent/count mismatches, invalid subviews and out-of-range element/front/back access cannot be silently truncated. Borrowed views cannot outlive their storage owner. Const/aliased storage cannot be upgraded to mutable byte access. Raw pointer+length or byte reinterpretation cannot spread beyond the narrow seam that separately proves safety. Process addresses/views are never durable state. Source, gitlink, focused-test or caller drift invalidates this mapping.

## Traceability
Requirement TDRP-R9-GSL-SPAN-VIEW-CONTRACT-DEPENDENCY-001.
Oracles: ORA-TDRP-GSL-SPAN-EXACT-SOURCE-001, ORA-TDRP-GSL-SPAN-CONTIGUOUS-VIEW-001, ORA-TDRP-GSL-SPAN-BOUNDS-001, ORA-TDRP-GSL-SPAN-BYTE-VIEW-001, ORA-TDRP-GSL-SPAN-NATIVE-SLICE-001.
Invariants: INV-TDRP-GSL-SPAN-NO-OWNERSHIP-001, INV-TDRP-GSL-SPAN-EXTENT-BOUNDS-001, INV-TDRP-GSL-SPAN-NO-UNCHECKED-SLICE-001, INV-TDRP-GSL-SPAN-NO-PARALLEL-RUNTIME-001, INV-TDRP-GSL-SPAN-NO-CONSUMER-ABSORPTION-001, INV-TDRP-GSL-SPAN-EXACT-PROVENANCE-001.
Counters after strict155: unknown=16,221 unread=16,196 omitted=0 responsibilities=173.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
