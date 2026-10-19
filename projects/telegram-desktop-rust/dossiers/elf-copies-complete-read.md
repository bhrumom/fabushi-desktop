# Complete read — .agents/shared/evidence/elf-copies.md

Status: understood; responsibility decomposed; existing-owner production wiring implemented; exact-head verification pending  
Upstream: `telegramdesktop/tdesktop@1e18da644e37856c9b1d87434bfcd6a2e3d11c57`  
Tree: `30c1d5cce4c1bde962882c1f2ca4b2e51bb03ae6`  
Path: `.agents/shared/evidence/elf-copies.md`  
Blob: `49fc28d671276e1787753267ae92397266d6bacd`

## Complete responsibility decomposition

This source is build/release evidence tooling, not Telegram runtime product behavior. Applicable responsibilities are directional fresh-copy provenance; separate unmasked whole-file source/copy SHA-256 identities; exact-vs-semantic identity separation; stable acquisition across the evidence window; fail-closed unsupported cases; bounded parser/I/O/resource behavior when a semantic comparator is needed; machine-readable command/environment/result evidence; and paired positive/forbidden controls.

The adjacent `.agents/shared/evidence/elf_identity.py` is a deliberately narrow Linux/WSL `ELF64-LE-x86_64-ordinary-exec-dyn-v1` oracle. It validates ELF/program/section structure, loader-visible bytes, narrowly allowed section/debug/static-symbol bookkeeping changes, zero gaps, compressed envelopes, symbol/reference semantics, acquisition stability and fixed limits before `load_equivalent`. Its documentation explicitly says this does not prove authenticity, startup, dynamic-library identity or general linker equivalence; only `source.sha256` and `copy.sha256` are exact identities. `elf_identity_test.py` supplies comparison, mapping, malformed, acquisition, command-owner and real-toolchain controls, including optimized-Python and sparse/high-offset cases.

## Caller / owner audit

The upstream helper is evidence tooling paired with build-target provenance, not shipped Telegram runtime. Fabushi already has source-neutral owners: `.github/workflows/release-macos-main.yml` for exact-source packaging/signing/notarization/installed acceptance and immutable artifacts; the quality evidence contract for durable exact-head evidence; the test strategy for fail-closed identity/equivalence testing; and release entry/exit criteria for the shipping gate. TDRP recursive archive/reachability jobs remain source/build-input authority owners, not executable-copy owners. No Telegram- or ELF-specific Fabushi owner is introduced.

## Production wiring

Current macOS packaging does not strip or intentionally rewrite the executable between the signed/notarized app bundle and the installed acceptance copy. Therefore the stronger source-neutral contract is used:

`signed/notarized source executable -> source SHA-256/size -> ditto fresh installed copy -> installed SHA-256/size -> source SHA-256/size recheck -> exact equality -> machine-readable copy identity -> packaged acceptance`

The workflow emits `fabushi-packaged-executable-copy-identity.json` with exact source SHA, source/copy paths, source-before/source-after and installed SHA-256, sizes, `sourceStable=true`, `exactSha256Equal=true`, and `copyRelationship="byte-exact"`, and uploads it with packaged acceptance evidence. Any source mutation or copy drift fails before acceptance.

## Platform-specific disposition

ELF64/x86-64 grammar, Linux `pread/fcntl/O_NOFOLLOW/O_NONBLOCK/O_CLOEXEC`, ELF header masks, debug/static-symbol exceptions, 4096-byte LOAD rules, GNU section/symbol semantics and sparse multi-GiB fixtures are format-specific and not applicable to the current Mach-O byte-copy path. They are explicitly dispositioned rather than silently omitted. A future Fabushi Linux transform such as strip/debug-copy must add a source-neutral Linux equivalence implementation with equivalent bounded positive/negative controls before release acceptance may claim transformed-binary equivalence.

## Traceability

Parent: `TDRP-R9-ELF-COPY-EVIDENCE-001`  
Responsibilities: `TDRP-R9-ELF-COPY-IDENTITY-001`, `TDRP-R9-ELF-COPY-STABILITY-001`, `TDRP-R9-ELF-COPY-EVIDENCE-001`  
Oracle: `ORA-TDRP-ARTIFACT-COPY-IDENTITY-001`  
Invariants: `INV-TDRP-ARTIFACT-RAW-IDENTITY-001`, `INV-TDRP-ARTIFACT-SOURCE-STABLE-001`, `INV-TDRP-ARTIFACT-NORMALIZED-NOT-IDENTITY-001`

This closes exactly one recursive source entry. With recursive inventory fixed at 16,192, counters advance from unknown=16,187 / unread=16,160 / omitted=0 to unknown=16,186 / unread=16,159 / omitted=0. Baseline readiness, source closure and acceptance remain false.
