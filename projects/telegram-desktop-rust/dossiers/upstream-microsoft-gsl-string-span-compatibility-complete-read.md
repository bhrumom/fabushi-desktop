# Microsoft/GSL include/gsl/string_span complete read — strict157

## Authority
- Live Fabushi main: ea0689be9ec66e92591c33d57801eb149db27749, tree 9bba99a29e5eb28ac12fa872be3eee3a15f42d35.
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- include/gsl/string_span@65790ab8bc9ba9e7fc45cf31f96f32ab27b850f7, fully read: 5 line slots / 225 bytes.
- Redirect target include/gsl/zstring@2cc13d83840afec61e5bd709c77900026ed225f4 remains independently open.
- Parent exact head 82f642b99966cadebbaa5176eff9d12e22b65992, tree 5581ea73843acf7517798e1833210aee1bf298f8.
- Parent TDRP 40045459039 completed 61/61 success; authority 12541290947 sha256:0b6b374a47a0a0e951917b50641afe18d6d3000acfb71d6a22c0f3b981cedf7d; closure 12541162056 sha256:f19f6743c9bef2ded89345e011eb4cecc4bccfd47f10a48fa1379d5f7711198e; reachability 12541611679 sha256:0943c19c2dab31f07bc0e3c03d091afdafd25b2411948a387a43d06da2ba90e0.
- Parent Rust desktop runtime 40045458992 completed 6/6 success; artifact 12540947614 sha256:e8dbc86308f73f52a15371e05380a303c45cce450dd2b39e1d74f379e731c15e.
- Accepted exact-head tdesktop code search found no direct `string_span`, `gsl::zstring`, or `basic_zstring` hits.
- Next: Microsoft/GSL include/gsl/util@26b2f5fb9d0d8c28c3096f50d575eb42a4b89bad.

## Exact behavior
`string_span` has no string-span implementation. It is a deprecated compatibility include: `#pragma once`, a compiler message announcing removal and directing callers to `<gsl/zstring>`, then `#include "zstring"`. The file owns no runtime state, storage, validation, encoding, null-termination behavior, persistence, concurrency, network, UI or service contract.

## Existing-owner-first
Fabushi does not need a source-shaped compatibility runtime. Exact source/dependency provenance stays in the TDRP inventory/ledger. Real string/byte/null-termination semantics remain with native Rust types and the actual domain owners after the separately locked `zstring` source is independently read and mapped. Do not add a `StringSpanCompat` owner, alias layer, second string type root or product subsystem.

## Boundary and fail-closed rules
This row cannot be used to close `zstring`, any caller, or any product capability. The redirect itself says callers should leave `string_span`; that is migration evidence, not authority to infer `zstring` safety semantics. A future accepted tdesktop direct consumer, changed redirect, changed GSL gitlink/child tree or changed `zstring` blob invalidates this mapping and requires fresh reconciliation.

## Traceability
Requirement TDRP-R9-GSL-STRING-SPAN-DEPRECATED-COMPATIBILITY-DEPENDENCY-001.
Oracles: ORA-TDRP-GSL-STRING-SPAN-EXACT-SOURCE-001, ORA-TDRP-GSL-STRING-SPAN-DEPRECATION-REDIRECT-001, ORA-TDRP-GSL-STRING-SPAN-NO-ZSTRING-CLOSURE-001.
Invariants: INV-TDRP-GSL-STRING-SPAN-NO-RUNTIME-001, INV-TDRP-GSL-STRING-SPAN-NO-DOWNSTREAM-CLOSURE-001, INV-TDRP-GSL-STRING-SPAN-EXACT-PROVENANCE-001.
Counters after strict157: unknown=16,219 unread=16,194 omitted=0 responsibilities=175.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
