# Microsoft/GSL .clang-format complete read — strict136

## Authority
- Accepted tdesktop upstream: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Root gitlink: `Telegram/ThirdParty/GSL` is mode `160000` and resolves exactly to `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37`.
- GSL child tree: `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`.
- Atomic child source: `.clang-format@c12d3bf2994fd5a083c04025d355a5ab4b3f6802`, 776 bytes, fully read.
- Parent Fabushi exact head: `ac152432d84855f74346b5cc9246f76276dbeb47`, tree `01627ad569447a3afb2ed2993c4eb84395af6e31`.
- Parent exact-head evidence: TDRP 39998164484 61/61; authority 12520278378 sha256:a97c604982bdba074fa1b59e6b6316a7613d060a5ecd4ac00e296101e86aa18e; closure 12520953207 sha256:fe369c5f8ba9e5c14a1ef5f559af958daa69d35c2ca6fab47af08b778c51d1ea; reachability 12520633420 sha256:e4ba64901933aba0ea3a95cfd5c4694843e67ee1dec07e877db59ed6c6d1c9aa; Rust 39998164311 6/6; artifact 12520862526 sha256:c924010d92cb6c328a85717a8163eddeb3e7ed4d21b8b2f2d7506aab746b4cb8.
- Submission lease basis: live main `7394f6dc7687f9b986ee5589d40aa680ce23b90a`; live upstream dev remains `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Next GSL root non-directory entry is `.gitattributes@3455dc9da5064e1a54deba6e2a5869aae2442d42`.

## Formatting-policy provenance
The file is declarative C++ authoring-format policy. It fixes `ColumnLimit: 100`, `UseTab: Never`, `IndentWidth: 4`, `AccessModifierOffset: -4`, and `NamespaceIndentation: Inner`.

Brace handling is custom: namespaces, enums, structs, classes, control statements, functions and unions use the declared wrapped form; `BeforeElse: true`; empty functions are not forcibly split. Template declarations always break. Constructor initializer layout uses the declared break-before-comma / one-line-or-one-per-line policy. Short blocks, functions, if statements and loops may remain on one line.

Pointer alignment is left. Consecutive assignments are not aligned, trailing comments are aligned, C-style casts receive a following space, and `CommentPragmas: '^ NO-FORMAT:'` defines the formatting pragma recognizer. These are formatting semantics only; the file does not declare build success, runtime behavior, security guarantees, source closure or release acceptance, and it does not identify a formatter version whose execution can be presumed.

## Existing-owner-first / applicability
The shipping Fabushi product does not retain this Microsoft/GSL C++ formatter configuration as a product/runtime owner. The applicable responsibility is source-neutral: language-specific authoring/toolchain policy must remain attributable to its exact source, map into the existing language/toolchain quality owner, and stay weaker than exact-head executable evidence.

`projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@4d2c88e118ab489d3d0339d2a61371a7ea74e72e` already states that source-specific policy maps to Fabushi's `existing language/toolchain quality owner`, and that `Fabushi executable static-analysis/build verification remains exact-head GitHub Actions evidence`. Those existing quality/evidence owners are therefore reused. No Microsoft/GSL formatter owner, Telegram formatter owner, C++ runtime, second quality root or product surface is introduced.

Because the source language/runtime is not retained as Fabushi product architecture, C++-specific style keys remain provenance. Any independently adopted Rust/TypeScript/platform formatting policy belongs to the native existing toolchain owner and must be qualified on its own terms; this upstream file does not silently define that policy.

## Fail-closed
The gitlink, child commit/tree and source blob are part of one immutable provenance chain. Drift at any layer invalidates dependent evidence. A clean local formatting result, an editor applying these options, or the presence of the file itself cannot be promoted to successful compilation, tests, security, package identity, runtime reachability, migration source closure or release acceptance.

Unsupported formatter keys/versions, partial formatting, concurrent source mutation, cancellation, or a `NO-FORMAT` pragma remain bounded formatting conditions; they cannot be interpreted as semantic correctness or a clean negative. Exact-head GitHub Actions remains required after this ledger publication. Recursive source closure and independent release acceptance remain open.

## Ledger source identity
The current Revision 9 validator binds every ledger row to the accepted tdesktop root tree. Accordingly, the ledger source identity for strict136 is the exact root gitlink `Telegram/ThirdParty/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37`; the atomic child file `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37:.clang-format@c12d3bf2994fd5a083c04025d355a5ab4b3f6802` is locked in source-symbol, dependency and dossier provenance. No validator weakening is performed.

## Traceability
Requirement `TDRP-R9-GSL-CLANG-FORMAT-POLICY-001`.

Oracles: `ORA-TDRP-GSL-CLANG-FORMAT-SOURCE-001`, `ORA-TDRP-GSL-CLANG-FORMAT-GITLINK-001`, `ORA-TDRP-GSL-CLANG-FORMAT-OWNER-001`, `ORA-TDRP-GSL-CLANG-FORMAT-EVIDENCE-BOUNDARY-001`.

Invariants: `INV-TDRP-GSL-CLANG-FORMAT-NO-CXX-RUNTIME-001`, `INV-TDRP-GSL-CLANG-FORMAT-STYLE-NOT-ACCEPTANCE-001`, `INV-TDRP-GSL-CLANG-FORMAT-EXACT-PROVENANCE-001`.

Closing only strict136 changes counters to unknown=16,240, unread=16,215, omitted=0, responsibilities=154. `baseline_identity_accepted=false`, `source_closure_ready=false` and independent release acceptance remain mandatory.

Next strict137: `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37 .gitattributes@3455dc9da5064e1a54deba6e2a5869aae2442d42`.
