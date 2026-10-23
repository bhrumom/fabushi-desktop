# Microsoft/GSL include/gsl/byte complete read — strict151

## Authority
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41
- include/gsl/byte@87d007ef94d197632872e96d91525a013c665da8, fully read: 184 line slots / 5962 bytes
- tests/byte_tests.cpp@634dcc90773e7144a127e29a2d49ce94b816e596, fully read: 135 line slots / 3518 bytes
- Parent exact head cf7ce14c5d7fe627768eac8bd19087cd2377050f, tree 64dea206821b7063a5a0fc5b65180197907111c8
- Parent TDRP 40036273453 61/61 success; authority 12537048053 sha256:fcede2a6f45aaf136156ee2c657cd2768d9b3706147894f18b1b4738e1c774f5; closure 12536939388 sha256:7a99c0159e1c41986c35b6efebde533a74bd7444352c75f88d954c86c36f1a84; reachability 12537093706 sha256:7fecba0f4319e4c40c4bbd72d8756d7fd46e64a48634307e5aa022cdb107f65f
- Parent Rust 40036273460 6/6 success; artifact 12537182837 sha256:2f2f97752df71c2c2bf07900cf6be9d7f26a9a47a3024c494581469a8382d354
- Accepted exact-head tdesktop code search observed gsl::byte in mtproto auth/RSA/DC-key paths, Passport encryption, Calls, Storage, Crash reserved memory and UI effects.
- Next: Microsoft/GSL include/gsl/gsl@53030b31747ca25e3d740bd078c0ffd5b71ebf88.

## Exact behavior
The locked header selects std::byte when the compiler/library exposes it; otherwise it defines an unsigned-char-backed, may-alias gsl::byte. The fallback supplies integral shifts, bitwise OR/AND/XOR/not, explicit to_integer, runtime to_byte restricted to unsigned char, and compile-time to_byte<I>() restricted to 0..255. The focused tests cover construction, bitwise/shift behavior, integral conversion, aliasing and compile-error boundaries.

The accepted tdesktop commit uses the primitive across unrelated domains: 256-byte MTProto auth-key storage, RSA buffers, Passport byte arithmetic, call encryption buffers, password-derived storage keys, crash reserved memory and UI-effect random inputs. Those callsites prove broad representation reuse; they do not make byte the owner of their product behavior.

## Existing-owner-first
Reuse TDRP-LANG-03 low-level dependency replacement and each actual Fabushi domain binary-data owner. Native Rust u8, fixed byte arrays, vectors or existing domain wrappers carry the representation; auth/crypto/storage/call/crash/media/UI semantics remain with those existing owners. Do not add a GSL/Byte runtime, generic binary-data root or source-branded buffer subsystem.

## Fail-closed
This row closes only the GSL byte dependency implementation. It does not close any concrete tdesktop caller, secret lifetime/zeroization, cryptographic protocol, storage format, crash lifecycle, call behavior, UI effect, later GSL header, source closure or release acceptance. Accidental implicit truncation must not replace the source's explicit type/range boundaries.

## Traceability
Requirement TDRP-R9-GSL-BYTE-REPRESENTATION-DEPENDENCY-001.
Oracles: ORA-TDRP-GSL-BYTE-EXACT-SOURCE-001, ORA-TDRP-GSL-BYTE-EXPLICIT-CONVERSION-001, ORA-TDRP-GSL-BYTE-CALLER-BOUNDARY-001, ORA-TDRP-GSL-BYTE-NATIVE-REPLACEMENT-001.
Invariants: INV-TDRP-GSL-BYTE-NO-OMISSION-001, INV-TDRP-GSL-BYTE-NO-GSL-RUNTIME-001, INV-TDRP-GSL-BYTE-NO-DOMAIN-ABSORPTION-001, INV-TDRP-GSL-BYTE-EXACT-PROVENANCE-001.
Counters after strict151: unknown=16,225 unread=16,200 omitted=0 responsibilities=169.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
