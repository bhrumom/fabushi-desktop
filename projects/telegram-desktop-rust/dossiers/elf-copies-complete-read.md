# Complete read — .agents/shared/evidence/elf-copies.md

Status: understood; responsibility decomposed; source-specific ELF mechanism not currently applicable; generic evidence mapping prepared; parent entries exact-head verification pending  
Upstream: `telegramdesktop/tdesktop@0a11537c220df35fe59a96aebe413b1e52bc6e47`  
Path: `.agents/shared/evidence/elf-copies.md`  
Blob: `49fc28d671276e1787753267ae92397266d6bacd`

## Complete responsibility decomposition

This file is build/evidence tooling documentation, not Telegram product runtime. It defines a deliberately narrow Linux x86-64 ELF execution-copy comparator and its qualification process.

Its responsibilities include:
- accept only a bounded, explicitly supported ELF64 little-endian x86-64 structural profile and refuse unsupported header/program/section/reference encodings rather than guess;
- compare loader-visible bytes exactly while allowing only explicitly documented section-table/header bookkeeping changes and qualified source-debug/static-symbol removal;
- distinguish normalized structural/load equivalence from exact whole-file identity, and retain unmasked SHA-256 identities for both source and copy;
- acquire both files once with read-only/no-follow/nonblocking/close-on-exec semantics, verify regular-file/stability properties, and refuse observed path/stat mutation, short reads, unsupported objects, or ambiguous acquisition;
- bound file size, table counts, name sizes, read requests, buffers and aggregate symbol-name work before allocation/I/O;
- stream payload/hash work instead of loading whole ELF files and preserve measured time/RSS/read evidence;
- create a fresh task-local execution copy instead of mutating the source or reusing historical evidence;
- independently re-check whole-file digests, stat stability and distinct artifact identity around comparison;
- qualify the comparator with positive/negative controls, high-offset sparse fixtures, mutation/refusal tests, optimized/interpreted modes, and owned child-process cleanup that does not kill unrelated processes;
- document the evidence limits precisely: comparator success is not authenticity, safe execution, successful startup, dynamic-library identity or a general linker-equivalence theorem.

## Fabushi applicability and owner audit

Current Fabushi shipping release authority is the existing exact-source macOS packaging workflow `.github/workflows/release-macos-main.yml`, which binds source identity, signs/notarizes/staples the packaged app, verifies codesign, records SHA-256 for packaged worker/archive artifacts, and uploads immutable Actions artifacts. Current Rust desktop runtime includes Linux test/profiling jobs but does not define a shipping `original ELF -> stripped execution copy` relationship. Repository searches found no current ELF identity comparator, strip-debug execution-copy helper, AppImage/deb/rpm Linux release owner, or equivalent source-specific mechanism.

Therefore the upstream ELF64 structural comparator, GNU-strip copy recipe, Linux syscall profile, and ELF-specific fixture corpus are not copied into Fabushi merely to match source shape. Their generally applicable evidence guarantees already belong to:
- `projects/fabushi-communication-platform/quality/evidence-contract.md`: exact target/run/job/environment/artifact name/ID/digest provenance;
- `.github/workflows/release-macos-main.yml`: exact-source packaged candidate identity, signing/notarization verification, independent SHA-256 checks and immutable artifact publication;
- `AGENTS.md`: FBCP/TDRP executable verification only in GitHub Actions and missing/unsupported environments remain blocked rather than pass.

If a future Fabushi Linux shipping pipeline creates a transformed ELF execution copy, that future capability must define its own source-neutral, format-appropriate identity/equivalence contract and tests rather than silently inheriting this Telegram-specific profile.

Shipping Host/Coordinator/Runner and messaging/storage owners are rejected: this is release/evidence tooling, not product lifecycle or data state.

## Equivalent contract

For current Fabushi release artifacts:

`exact source SHA -> allowed GitHub Actions build -> immutable packaged candidate -> platform signing/notarization verification -> independent artifact SHA-256 -> packaged acceptance -> artifact/evidence publication`

Any transformed binary relationship must be explicitly defined and independently evidenced. A normalized/semantic digest cannot replace exact artifact identity. Unsupported format/profile/environment is fail-closed, not proof of equivalence.

## Traceability

Requirement: `TDRP-R9-ELF-EXECUTION-COPY-EVIDENCE-001`  
Oracle: `ORA-TDRP-ARTIFACT-IDENTITY-EVIDENCE-001`  
Invariant: `INV-TDRP-EXACT-ARTIFACT-IDENTITY-001`

Oracle: current Fabushi packaging/evidence records exact source and artifact identity with independent digest/provenance and fail-closed verification; no unsupported ELF/strip equivalence is claimed or introduced. A future Linux transformed-executable release must add an explicit source-neutral equivalence contract before such a relation can be accepted.

Current parent basis: this dossier is staged on a descendant preparation branch after entries 3–5. It does not advance recursive coverage, ledger target commit, baseline readiness or acceptance. Those fields may move only after each parent entry is merged and this entry receives its own exact-head GitHub Actions evidence.
