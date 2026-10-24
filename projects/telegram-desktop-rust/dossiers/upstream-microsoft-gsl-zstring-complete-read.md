# Microsoft/GSL include/gsl/zstring complete read — strict159

## Authority
- Live Fabushi main: fb1189efd9d85c650864d6987ea8fb68c913558d, tree 1a1ef09be4c9fd801354439a5b82cc460f0b4b6a.
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- include/gsl/zstring@2cc13d83840afec61e5bd709c77900026ed225f4, fully read: 59 line slots / 1976 bytes.
- LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02 confirms Microsoft/GSL MIT licensing.
- Original strict158 child 1b84c2c3f38207676643f28af3595699e0586664: TDRP 40048841440 completed failure with 58 success / 1 failure / 2 skipped; failed Source authority / recursive inventory job 120736679845 identified a duplicate target symbol. Rust 40048842264 completed cancelled with 5 success / 1 cancelled.
- Minimal repair 92d9f63f98a141a3c4e9032e9f992ce919b3b954 changed only projects/telegram-desktop-rust/parity-ledger.json, deleting the duplicate `### TDRP-MAP-01 — 文件全覆盖` target symbol from strict158.
- Repaired parent TDRP 40049224913 completed 61/61 success; authority 12542722367 sha256:0d6b4713f89e1d9c75e39fbb144e4621c9e768137e3e7f549297472c2c5698ce; closure 12542234806 sha256:c33765474010635265d940cf70ecbd2aa8e474ec14edcb9aef143f956b78de3d; reachability 12542748005 sha256:525811a6f89796201200cee94b75deb675190d2c513f48ca8b90d39bd92f4cbd.
- Repaired parent Rust desktop runtime 40049224929 completed 6/6 success; artifact 12542527912 sha256:cbdc2d2e41a7b3e96e49a7ef3354fbc5ddfeee668bf7e5bfb54ab42f734ee005.
- Accepted tdesktop current dev search returned zero direct matches for `gsl::basic_zstring`, `gsl::zstring`, `gsl::czstring`, `gsl::wzstring`, `gsl::cwzstring`, `gsl::u16zstring`, `gsl::cu16zstring`, `gsl::u32zstring`, and `gsl::cu32zstring`.
- Next: Microsoft/GSL tests/algorithm_tests.cpp@e369be0e34dcf786a63631807ebe3cc06fb59544.

## Exact behavior
`zstring` is a header-only family of tag aliases for C-style zero-terminated character arrays. `basic_zstring<CharT, Extent = dynamic_extent>` is exactly `CharT*`; the `Extent` template parameter is carried in the alias signature but the resolved type remains a raw pointer. Const and mutable aliases exist for `char`, `wchar_t`, `char16_t`, and `char32_t`.

The header explicitly states that these aliases are intended to help static analysis and that no additional type-system feature/semantic is added because doing so would either impose significant runtime cost or break existing programs. There is no construction, scan, length check, null check, bounds check, encoding check, allocation, ownership transfer, lifetime management, persistence, synchronization, or runtime validation in this file.

## Existing-owner-first
Fabushi does not need a `GSLZStringRuntime`, second canonical string model, global C-string owner, or compatibility typedef layer. Canonical product text remains in existing native text/domain owners. Rust `&str`/`String` and byte slices/vectors remain the default representations; `std::ffi::CStr`/`CString` (or a narrower platform adapter equivalent) is appropriate only when an independently migrated real C ABI/platform boundary requires zero termination.

Because accepted tdesktop has no direct `zstring`/ `basic_zstring` consumer, strict159 does not invent a caller, FFI boundary, UI surface, persistence model, service, or production entrypoint. Any future indirect or newly discovered consumer remains an independent responsibility in its actual owner.

## Boundary and fail-closed rules
- A zstring alias is not proof that a pointer is non-null, valid, bounded, correctly encoded, live, owned, secret-safe, or actually terminated.
- Null-terminated representation must not leak into ordinary Fabushi product text or storage merely to mimic the C++ helper.
- Const/mutable intent and character-width obligations must be preserved at a real independently audited native boundary.
- Raw-pointer/C ABI lifetime across asynchronous work belongs to the actual caller and must be fenced there; the helper owns no concurrency.
- Any accepted tdesktop/GSL gitlink, child tree, zstring/license blob, or direct-consumer-set drift requires fresh reconciliation.
- No downstream caller, product capability, security property, or release acceptance is closed transitively by this dependency row.

## Traceability
Requirement TDRP-R9-GSL-ZSTRING-NULL-TERMINATED-STRING-TAG-DEPENDENCY-001.
Oracles: ORA-TDRP-GSL-ZSTRING-EXACT-SOURCE-001, ORA-TDRP-GSL-ZSTRING-NO-RUNTIME-SEMANTICS-001, ORA-TDRP-GSL-ZSTRING-NO-DIRECT-CONSUMER-001, ORA-TDRP-GSL-ZSTRING-NO-PARALLEL-STRING-OWNER-001.
Invariants: INV-TDRP-GSL-ZSTRING-BOUNDARY-ONLY-001, INV-TDRP-GSL-ZSTRING-NO-GSL-RUNTIME-001, INV-TDRP-GSL-ZSTRING-NO-IMPLIED-POINTER-SAFETY-001.
Counters after strict159: unknown=16,217 unread=16,192 omitted=0 responsibilities=177.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
