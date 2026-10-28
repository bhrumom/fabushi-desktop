# Microsoft/GSL tests/algorithm_tests.cpp complete read — strict160

## Authority
- Live Fabushi main: fb1189efd9d85c650864d6987ea8fb68c913558d, tree 1a1ef09be4c9fd801354439a5b82cc460f0b4b6a.
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- tests/algorithm_tests.cpp@e369be0e34dcf786a63631807ebe3cc06fb59544, fully read: 225 line slots / 6404 bytes.
- include/gsl/algorithm@584f5cd12beb15e863b1c32bc887638314261399, fully read: 64 line slots / 2339 bytes.
- Supporting tests/deathTestCommon.h@7bf242393f7bcff62c12cd985a87824f48f7bb0b, fully read: 12 line slots / 380 bytes; it only normalizes the expected death-test string around std::set_terminate behavior.
- tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a includes algorithm_tests.cpp in the GSL test source list.
- LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02 confirms Microsoft/GSL MIT licensing.
- Parent exact-head cb189b0b2eca841d817cf50bb26f50505d7cbf9f: TDRP 40051564038 completed 61/61 success; authority 12543343371 sha256:bb98d43c429c9e9c673f92f3f20ede6733c6cf33c13e49830294d6f744cbcb51; closure 12544006689 sha256:cd8d29d6b9f145b01dd169c1f93c0200834ce3c91a0726a27d30b7ac0b07ab08; reachability 12543483483 sha256:b5b02e53426a2f6ad808b8933402fd36ac663de2777ca6c9a6983be609821a63.
- Parent Rust desktop runtime 40051564033 completed 6/6 success; artifact 12542989074 sha256:5135d7ee3cb0080d9dfa1790dcbb479b0ab89326001406f208502e213d5b0037.
- Superseded fe36cedded8bc7d0cf8215059ee42fa75091fca1 TDRP 40051301759 completed failure with 58 success / 1 failure / 2 skipped; failed Source authority / recursive inventory job 120744360210 reported exactly `target symbol missing for TDRP-R9-GSL-ZSTRING-NULL-TERMINATED-STRING-TAG-DEPENDENCY-001`, then authority artifact upload had no files.
- Superseded Rust run 40051301757 completed cancelled with renderer, three pressure-profiler jobs and platform-worker successful while rust-host was cancelled.
- Accepted tdesktop current dev is the repository default branch and a fresh exact-head code search returned zero direct `gsl::copy` consumers.
- Next: Microsoft/GSL tests/assertion_tests.cpp@231befc7e415ec53daa09ced61d76505e6be5437.

## Exact behavior
`gsl::copy` is a small bounded span-copy helper. The header first requires the source element to be assignable to the destination element with a compile-time `static_assert`. If both source and destination extents are fixed, it also rejects a statically known source extent larger than the destination. For dynamic cases it enforces `dest.size() >= src.size()` through `Expects`, then executes `std::copy_n(src.data(), src.size(), dest.data())`. A successful call therefore writes exactly the source length and does not intentionally touch the remaining destination tail.

The test file exercises same-type `int -> int` copying and assignment-compatible `short -> int` copying across all four dynamic/static source-destination combinations. Each case copies once to the destination prefix and once to a destination subspan beginning after the source length, then verifies both copied regions element-by-element. Under `CONFIRM_COMPILATION_ERRORS`, `int -> int*` combinations are required not to compile. The small-destination test uses a 12-element source and 4-element destination: dynamic cases are death tests through the runtime contract, while the fully static mismatch is reserved as a compile-time error.

No ordering, persistence, retry, cancellation, network, account, UI or long-lived lifecycle responsibility exists in this test/helper pair. The source does not establish authorization, semantic validity of copied values, overlap safety beyond the underlying standard-library contract, or safe replacement by unchecked byte copying.

## Existing-owner-first
Fabushi does not need a `GSLAlgorithmRuntime`, global buffer-copy owner, second span model or compatibility test subsystem. This row belongs to the existing TDRP source/dependency inventory and test-provenance owner. If an independently migrated real caller later needs bounded copying, its actual existing Rust/domain owner keeps the product state and behavior.

For same-type Rust slices, the actual caller can use a native safe slice operation after exact length validation. Assignment-compatible cross-type behavior such as `short -> int` must remain an explicit per-element conversion/assignment in that caller; a raw memory copy would not preserve the C++ assignment contract. Because accepted tdesktop has zero direct `gsl::copy` consumers, strict160 does not invent a caller, production entrypoint, persistence model, service or UI surface.

## Boundary and fail-closed rules
- Destination capacity must be proven before a successful copy; never silently truncate.
- Incompatible element types must fail before mutation; do not reinterpret bytes to bypass type compatibility.
- The helper's static-versus-dynamic checks are source-language mechanics, not a reason to reproduce GSL spans in Fabushi.
- This source does not prove safe overlapping-range behavior; an actual caller must select an operation whose alias/overlap contract matches its real data flow.
- The helper/test pair does not close any indirect future caller, span responsibility, product capability or release gate transitively.
- Any accepted tdesktop/GSL gitlink, child tree, algorithm test/header/license blob, build registration, or direct-consumer-set drift requires fresh reconciliation.
- No source test, mock, helper contract or historical artifact substitutes for exact-head production/release acceptance.

## Traceability
Requirement TDRP-R9-GSL-ALGORITHM-COPY-CONTRACT-TEST-PROVENANCE-001.
Oracles: ORA-TDRP-GSL-ALGORITHM-EXACT-SOURCE-001, ORA-TDRP-GSL-ALGORITHM-COPY-BOUNDS-001, ORA-TDRP-GSL-ALGORITHM-TYPE-COMPATIBILITY-001, ORA-TDRP-GSL-ALGORITHM-NO-DIRECT-CONSUMER-001.
Invariants: INV-TDRP-GSL-ALGORITHM-DESTINATION-NOT-SMALLER-001, INV-TDRP-GSL-ALGORITHM-INCOMPATIBLE-TYPE-REJECTED-001, INV-TDRP-GSL-ALGORITHM-NO-PARALLEL-RUNTIME-001.
Counters after strict160: unknown=16,216 unread=16,191 omitted=0 responsibilities=178.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
