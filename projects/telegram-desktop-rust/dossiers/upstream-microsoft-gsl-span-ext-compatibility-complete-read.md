# Microsoft/GSL include/gsl/span_ext complete read — strict156

## Authority
- Live Fabushi main: 7394f6dc7687f9b986ee5589d40aa680ce23b90a, tree fbeb0bb32d944d0c1053880c471fe5c52279f052.
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- include/gsl/span_ext@2a7664c70550477e9ff228d6370ff635b5217c81, fully read: 215 line slots / 6756 bytes.
- tests/span_ext_tests.cpp@dfd2725db3612de97c4a3173f5f168131f7341f3, fully read: 381 line slots / 11004 bytes; 12 focused TEST cases.
- Parent exact head f2e6007709825b16f351f3d4d85f0d202766c4e7, tree 6830164b1f5ed469553f9825c4a8197e6da7ee5c.
- Parent TDRP 40043916450 61/61 success; authority 12539664651 sha256:1a13f69d30e25b94e4329f3960abc1b025f66f39dbb9df3b1adc553722e596c4; closure 12540263985 sha256:a6ca1c4942db3e911dc5eaad88fad8235bfbe19ae2ce35ebacfd3fcede213636; reachability 12539669845 sha256:61939fcd5c268bccf6ccf2d1f2b346140a9d84a56d67bbfdd2cc21bd83618c07.
- Parent Rust desktop runtime 40043916458 6/6 success; artifact 12539884045 sha256:2f89760a2ca2d2899123ac66415608443c81bb241263b34724c0bb98a8f2ed69.
- Accepted exact-head tdesktop code search found 16 direct `gsl::make_span` consumer files spanning MTProto, Passport, Calls, media/audio/streaming, send-files and UI/grouped/poll paths. No direct named `gsl::ssize` or `gsl::at` search hit was found.
- Next: Microsoft/GSL include/gsl/string_span@65790ab8bc9ba9e7fc45cf31f96f32ab27b850f7.

## Exact behavior
`span_ext` preserves GSL span features removed while aligning `gsl::span` with `std::span`. Outside `GSL_KERNEL_MODE`, equality uses element equality and ordering uses lexicographical comparison; the relational operators derive the complementary relations. `make_span` adapts pointer/count, pointer/pointer, C arrays and mutable/const containers into non-owning spans, while legacy smart-pointer overloads are explicitly deprecated. `gsl::at(span,index)` rejects negative signed indices before delegating to the span bounds contract, `ssize` projects the size as `ptrdiff_t`, and the free begin/end/rbegin/rend/cbegin/cend/crbegin/crend helpers forward to the span iterators. Focused tests cover all make_span forms, null pointer with nonzero length death, signed-index negative/end death paths, iterator helper identity, signed size and comparison semantics.

## Existing-owner-first
The Fabushi equivalent remains Rust native `&[T]`, `&mut [T]`, array references, iterator adapters, checked indexing and ordinary `PartialEq`/ordering over the actual domain values. Each current product/domain storage owner keeps its own data and lifecycle; an unavoidable raw pointer+length conversion belongs only to the narrow platform/FFI adapter that proves pointer validity, length, lifetime, alignment, aliasing and mutability. Do not add a GSL SpanExtRuntime, compatibility buffer owner, second storage root or source-shaped helper subsystem.

## Consumer boundary
The 16 direct accepted tdesktop `gsl::make_span` consumers include protocol/auth buffers, passport encryption/form flows, calls/group calls, audio/streaming, send-files and UI rendering/layout. `span_ext` only supplies a borrowed-view construction/helper contract. Those callers retain independent state machines, security, persistence, network, UI and product responsibilities and remain separately open.

## Fail-closed
A null pointer with a nonzero count, a negative signed index, an out-of-range element request, an invalid raw range or a borrow that escapes its storage lifetime cannot be normalized into a valid view. Deprecated smart-pointer adapters do not justify reproducing pointer-centric ownership in Rust. Kernel-mode comparison absence is a source/platform constraint, not authority to invent a parallel runtime. Source, gitlink, focused-test or consumer drift invalidates this mapping.

## Traceability
Requirement TDRP-R9-GSL-SPAN-EXT-COMPATIBILITY-DEPENDENCY-001.
Oracles: ORA-TDRP-GSL-SPAN-EXT-EXACT-SOURCE-001, ORA-TDRP-GSL-SPAN-EXT-MAKE-SPAN-001, ORA-TDRP-GSL-SPAN-EXT-CHECKED-AT-001, ORA-TDRP-GSL-SPAN-EXT-ITERATOR-SIZE-001, ORA-TDRP-GSL-SPAN-EXT-COMPARISON-001.
Invariants: INV-TDRP-GSL-SPAN-EXT-NO-OWNERSHIP-001, INV-TDRP-GSL-SPAN-EXT-BOUNDS-001, INV-TDRP-GSL-SPAN-EXT-NO-COMPAT-RUNTIME-001, INV-TDRP-GSL-SPAN-EXT-NO-CONSUMER-ABSORPTION-001, INV-TDRP-GSL-SPAN-EXT-EXACT-PROVENANCE-001.
Counters after strict156: unknown=16,220 unread=16,195 omitted=0 responsibilities=174.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
