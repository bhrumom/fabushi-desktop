# Complete read — .agents/shared/evidence/elf_identity.py

Status: understood; responsibility decomposed; existing-owner production wiring audited; exact-head verification pending  
Upstream: `telegramdesktop/tdesktop@1e18da644e37856c9b1d87434bfcd6a2e3d11c57`  
Tree: `30c1d5cce4c1bde962882c1f2ca4b2e51bb03ae6`  
Path: `.agents/shared/evidence/elf_identity.py`  
Blob: `70dc54620651e8c6fd9cce6966c9964b1a6c90d7`  
Companion control suite: `.agents/shared/evidence/elf_identity_test.py@a75a29225b6eb70f255848e30026d02523d808f3`

## Source role and caller closure

This file is build/release evidence tooling, not Telegram product runtime and not a Telegram network or message owner. Direct source callers at the accepted upstream baseline are the ELF execution-copy guide and its companion unit/control suite. The shared test loop points Linux Debug execution-copy evidence to that guide. Ninja target provenance is deliberately separate: `.agents/shared/evidence/ninja_target.py` proves configured CMake/Ninja graph membership for one exact consuming output, while explicitly refusing to claim command execution or archive-member reachability. The ELF comparator therefore does not own Ninja graph, archive-member, or recursive-source reachability.

Fabushi keeps the same ownership split. TDRP root/child/next/fixed-point, recursive build reachability, root source archives and dependency reachability remain source/build-input authority owners. Executable copy/transformation identity remains in the existing source-neutral release/evidence owners: `.github/workflows/release-macos-main.yml`, `projects/fabushi-communication-platform/quality/evidence-contract.md`, `TEST_STRATEGY.md`, and `release-entry-exit-criteria.md`. No Telegram-specific or duplicate ELF packaging owner is introduced.

## Stable acquisition and bounded I/O

`ElfFile` performs one descriptor-owned acquisition per subject on Linux/WSL. It requires `pread`, `O_NOFOLLOW`, `O_CLOEXEC`, `O_NONBLOCK` and `fcntl`; opens read-only; accepts regular files only; and records device, inode, size, mtime and ctime. Path identity is checked against the opened descriptor before parsing and again at final verification. A same-size mutation, truncate/grow, pathname replacement or symlink swap observed by those identities is a refusal. Atime-only change is not treated as a content mutation. This is observed stability, not a lock.

All parser offsets and sizes are checked before I/O. One `pread` request is capped at 1 MiB and whole payloads are streamed. The profile caps each file at 16 GiB, program headers at 1,024, sections at 4,096, the section-name table at 16 MiB, section names at 4,096 bytes, a static symbol table at 4,194,304 records, the private static string table at 1 GiB, symbol names at 64 KiB, resolved symbol-name bytes at 4 GiB, name-read bytes/count at 20 GiB/5,242,880 with 4 KiB chunks, interpreter bytes at 4 KiB, and keeps no symbol-name cache. Sparse/high-offset inputs are supported without whole-table reads.

These Linux descriptor/parser mechanics are format-specific. The applicable Fabushi invariant is stable bounded acquisition for any intentional transform comparator. The current macOS shipping path is not a semantic transform: its signed/notarized executable is copied with `ditto`, and the release workflow independently records source-before, installed-copy, and source-after SHA-256/size and fails on any drift. That byte-exact requirement is stronger than invoking a format parser for this current edge.

## ELF profile and loader-visible identity

The comparator is intentionally narrow: ELF64, little-endian, x86-64, ordinary section/program numbering, ET_EXEC or ET_DYN, supported System V/GNU ABI values, fixed ELF/program/section header sizes, and a restricted known program/section/flag grammar. Unsupported encodings fail closed rather than being normalized.

It validates every program mapping and all loader-visible bytes. PT_LOAD mappings use a 4,096-byte page profile with fully backed rounded file pages, disjoint rounded virtual ranges, writable BSS requirements, zero-fill boundaries, executable entrypoint reachability, and compatible mappings for PHDR/INTERP/DYNAMIC/TLS/RELRO and related records. Non-NULL/non-LOAD record payloads may not alias the small mutable ELF-header bookkeeping fields. Program-header order and every program field remain exact, and merged loader file coverage is compared byte-for-byte, including exposed padding.

Only the ELF header bytes for `e_shoff`, `e_shnum`, and `e_shstrndx` may differ, and only after both complete inputs independently validate. Those fields are reported as explicit changed-byte allowances; they are never masked from the raw whole-file artifact SHA-256 identities.

For current Fabushi macOS packaging this ELF load profile is not applicable because the executable copy is byte-identical. If a future Fabushi Linux release actually introduces strip/debug-copy or another binary transformation, the existing evidence/release contract requires a source-neutral format-specific comparator before equivalence can pass; it must protect loader-visible bytes at least as strictly as this source responsibility.

## Sections, debug data and symbol equivalence

The source validates section-name-table grammar, unique raw names, ranges, alignment, supported section types/flags, reference semantics, TLS/allocated mapping, dynamic-table correspondence, and compression envelopes before comparison. Section bookkeeping may not overlap loader-visible bytes.

The copy may not invent a section. The only removable source sections are a fixed, qualified set of nonallocated debug PROGBITS sections with constrained flags and no forbidden dependencies. Compression metadata accepts only the bounded zlib/zstd envelope and validates declared uncompressed size/alignment without decompression. Unknown or retained nonloaded sections keep their payload and semantic metadata; uncovered nonloaded bytes must remain zero.

Private static symbol/name bookkeeping is allowed to repack only after strict validation. Dynamic symbol section indexes must resolve to the same raw section identity. Retained static symbols preserve ordered semantic identity: resolved name, info, visibility, resolved target, value, and size. Only canonical source-local FILE and unnamed SECTION bookkeeping records may disappear; retained symbols may not be removed, reordered, inserted or retargeted. Semantic digests describe those symbol sets but never replace raw artifact identities.

This exception model is applicable only to a real intentional ELF transformation. It is not copied into the current byte-exact Mach-O path and does not create a second Fabushi packaging truth.

## Exact identities and machine-readable report

After semantic comparison, the helper streams raw SHA-256 over each complete file, then rechecks final descriptor/path identity. `build_report` emits schema version, `load_equivalent`, whether exact raw SHA-256 values are equal, source/copy absolute paths and sizes, raw SHA-256 identities, acquisition timestamps/stats, read accounting, exact profile and all numerical limits, allowed header-field changes, program/load coverage, section comparison/removals, static-symbol semantics, dynamic-section-index identities, and zero-gap findings.

The report explicitly scopes itself to the artifact relationship. It does not prove authenticity, safe execution, successful startup, dynamic-library identity, or general linker equivalence. Fabushi's evidence contract preserves the same separation: raw artifact identities remain mandatory, while a semantic oracle is only an additional transform proof.

## Companion positive/forbidden controls

`elf_identity_test.py@a75a29225b6eb70f255848e30026d02523d808f3` was read together with this source because it defines the oracle controls. It uses independent literal fixtures plus real GNU tooling. Positive cases cover exact identity, debug stripping with bookkeeping repack/reorder, qualified debug deletion, compressed-debug envelopes, canonical local FILE/SECTION removal, bounded BSS/TLS/load mapping, parser limits, sparse metadata above 4 GiB, normal and optimized Python, and a real `g++ -g -O0` plus `strip --strip-debug --no-merge-notes` pair.

Forbidden controls mutate loader bytes/padding, all program/header fields, allocated/NOBITS/TLS metadata, retained/unknown nonloaded content, dynamic section targets, retained static-symbol semantics/order, debug/static roles, compression envelopes, malformed counts/ranges/references, uncovered nonzero bytes, oversized declarations, nonregular/symlink/FIFO inputs, and real executable text. Acquisition controls force same-size mutation, truncate/grow, replacement and symlink swap. Descriptor tests prove close-on-success/failure. The command harness records argv/environment/stdout/stderr/timing/PID/PGID and owns descendants through timeout/cancel/error cleanup; cleanup failure cannot pass and unrelated process groups survive.

This test file is a separate recursive source entry. Reading it here supplies the required control context for `elf_identity.py`; this closure advances only the single `elf_identity.py` inventory entry and does not pre-close the companion test entry.

## Existing-owner production disposition

Current Fabushi shipping relationship is:

`signed/notarized source executable -> source SHA-256/size -> fresh installed byte copy -> installed SHA-256/size -> source SHA-256/size recheck -> exact equality -> machine-readable exact-head artifact -> packaged acceptance`

The existing release workflow also verifies codesign, Apple ticket and Gatekeeper on the installed app. The executable-copy report records the exact source SHA, source/copy paths, separate raw SHA-256 values and sizes, source stability, exact equality and `copyRelationship="byte-exact"`, and is uploaded with packaged acceptance evidence.

Therefore no ELF parser is needed in the current shipping composition. The generic quality contract already fail-closes any future intentional transform unless its actual platform adds a bounded format-specific equivalence oracle with stable acquisition, explicit accepted/refused cases, positive/forbidden controls, resource limits and a machine-readable report. This is existing-owner-first closure, not an N/A dismissal of the helper's engineering responsibilities.

## Traceability

Responsibilities:
- `TDRP-R9-ELF-ACQUISITION-BOUNDS-001` — stable single acquisition and bounded parser/I/O.
- `TDRP-R9-ELF-LOADER-IDENTITY-001` — protected header/program/load semantics and loader-visible bytes.
- `TDRP-R9-ELF-SECTION-EQUIVALENCE-001` — section/debug/compression and nonloaded equivalence constraints.
- `TDRP-R9-ELF-SYMBOL-EQUIVALENCE-001` — dynamic/static symbol identity and narrowly removable bookkeeping.
- `TDRP-R9-ELF-CONTROLS-REPORT-001` — independent positive/forbidden controls plus bounded machine-readable evidence.

Capabilities: `CAP-RELEASE-ARTIFACT-IDENTITY`, `CAP-RELEASE-TRANSFORM-EQUIVALENCE`.  
Existing production owner: Fabushi release packaging + source-neutral quality/evidence contracts.  
New owner: none.  
Current platform disposition: exact-byte macOS copy; ELF transform mechanics dormant/not applicable until a Linux ELF transform exists.

This closes exactly one recursive source entry. Recursive inventory remains 16,192; counters advance from unknown=16,186 / unread=16,159 / omitted=0 to unknown=16,185 / unread=16,158 / omitted=0. Baseline readiness, source closure and final acceptance remain false.
