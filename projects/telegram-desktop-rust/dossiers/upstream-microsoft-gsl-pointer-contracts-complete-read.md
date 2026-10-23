# Microsoft/GSL include/gsl/pointers complete read — strict154

## Authority
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41
- include/gsl/pointers@63e193e92478632520eca7e872ac83c643ffd2c0, fully read: 348 line slots / 11854 bytes
- tests/notnull_tests.cpp@df8e5a9acf529a9da9e578da413e955900e91e7d, fully read: 649 line slots / 18407 bytes
- tests/strict_notnull_tests.cpp@c1b1f54c97abe1c2cf825e81e713bee2fb8a888f, fully read: 305 line slots / 7437 bytes
- tests/owner_tests.cpp@87c7cec89a5fab4e1ad9ddda56a938766967316d, fully read: 44 line slots / 1285 bytes
- Parent exact head 017def57c0bf13e02bb14a609edeeb3b313d9bab, tree 9dc493e35bad08468319e461454da43abd183f05
- Parent TDRP 40040846965 61/61 success; authority 12538848834 sha256:d2cabc8af56a49db5a2ab4831fd49dd46adb140e98dec476280d630b59ffe17a; closure 12538949061 sha256:7312bc2ea22a4960f7c61c31561493cbad01d921cab06592e1f4698f594ac0cf; reachability 12539790509 sha256:7679cfaf262f793bf0aa1234a1076abec3eae8a10f4fde60a9fc7ad993486128; Rust 40040846998 6/6 success; artifact 12539735416 sha256:b05f4beb2bcb70f8b6c30f121f2ad9052de4e4a4f5437353d4de68a78d64b578.
- Accepted exact-head tdesktop not_null usage is widespread across Window, Settings, Calls, API, media, UI and other domains.
- Next: Microsoft/GSL include/gsl/span@d2ef9f7d85ca6e25bf69b001e9308b3d069c5d16.

## Exact behavior
`owner<T*>` is an ownership-intent annotation over a raw pointer, not an allocator or lifetime manager. `not_null<T>` requires comparability to null, validates non-null construction, rejects nullptr construction/assignment, exposes get/deref/arrow/conversion, and deliberately deletes pointer arithmetic/indexing so the wrapper denotes one object. Its comparison/hash/stream adapters forward to the underlying handle. `strict_not_null<T>` makes raw construction explicit while preserving the invariant; moves preserve the contained pointer rather than creating a moved-from null representation. The focused tests exercise raw/smart/custom pointer construction, casting, assignment, comparison, dereference, deduction, hashing, strict construction and owner pointer constraints.

## Existing-owner-first
Use Rust ownership/borrowing, Box/Arc/Rc/owned values, references and Option where nullability is a real domain state. Use NonNull/raw pointers only at unavoidable platform/FFI seams with a separately proven lifetime. Do not create a GSL PointerRuntime, NotNull service, cross-domain lifetime owner or second state/identity root.

## Consumer boundary
tdesktop uses not_null pervasively, but that is only a low-level precondition/typing primitive. Window, Settings, Calls, API, media, UI and every other consumer retain their own behavior, state, lifecycle, persistence, concurrency, security and product effects. This row cannot close them.

## Fail-closed
Required-reference boundaries cannot silently become nullable; nullable domain state cannot be hidden behind a required handle; raw interop cannot manufacture lifetimes. Process pointer values are never durable state. Source/test/caller drift invalidates this mapping.

## Traceability
Requirement TDRP-R9-GSL-NONNULL-POINTER-CONTRACT-DEPENDENCY-001.
Oracles: ORA-TDRP-GSL-POINTERS-EXACT-SOURCE-001, ORA-TDRP-GSL-POINTERS-NONNULL-001, ORA-TDRP-GSL-POINTERS-OWNERSHIP-ANNOTATION-001, ORA-TDRP-GSL-POINTERS-NATIVE-OWNERSHIP-001.
Invariants: INV-TDRP-GSL-POINTERS-NO-NULL-REQUIRED-001, INV-TDRP-GSL-POINTERS-NO-ARITHMETIC-001, INV-TDRP-GSL-POINTERS-NO-PARALLEL-LIFETIME-OWNER-001, INV-TDRP-GSL-POINTERS-NO-CONSUMER-ABSORPTION-001, INV-TDRP-GSL-POINTERS-EXACT-PROVENANCE-001.
Counters after strict154: unknown=16,222 unread=16,197 omitted=0 responsibilities=172.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
