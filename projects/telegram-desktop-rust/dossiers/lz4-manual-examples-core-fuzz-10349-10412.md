# LZ4 manuals, examples, core and first OSS-Fuzz — deterministic source read 10,349–10,412

Authority: `lz4/lz4@5ff839680134437dbf4678f3d0c7b371d84f4964`, mounted by accepted `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941` at `Telegram/ThirdParty/lz4`.

Identity/materialization evidence: exact-head Fabushi `99e027714e9ee707e544073c6e9240fa0c77c289`, Rust desktop runtime `38127755807`, Source authority / recursive inventory job `114435611847`, artifact `11697459714`, digest `sha256:c53bc4284dd3e358aa891df11d89083625fa56c2092d4f4b8ab1a509513a4953`. Revision 9 validation and the exact review-window materialization succeeded before this accounting descendant.

## Responsibility decomposition

- Manual/build/package inputs define deterministic API documentation generation, Meson optional components, static/shared/DLL/pkg-config installation, Snap release/update metadata and BSD/GPL license boundaries.
- Block format/API requires explicit compressed size and decompressed capacity metadata, rejects malformed/zero-offset streams, bounds output, and documents ecosystem block-size/termination constraints.
- Frame format/API adds versioned headers, linked/independent block history, block/content checksums, content size, dictionary identity, concatenation ordering, typed unsupported/error behavior and reusable compression/decompression contexts.
- Streaming and dictionary examples make caller-owned history, ring/double-buffer lifetime, random-access jump tables, safe continued decompression and byte-for-byte round-trip validation explicit.
- File wrappers add open/read-write/close state, frame-derived buffer budgets, allocation cleanup and I/O error propagation.
- HC keeps bounded dictionary/match-distance and overflow protections.
- Bundled xxHash is a non-cryptographic checksum/integrity primitive; it must never be treated as authentication or security hashing.
- OSS-Fuzz deliberately varies frame preferences and output capacities, including undersized destinations, and requires successful compression to round-trip exactly.

## Existing-owner-first audit

Current `source/host/Cargo.toml` already declares `flate2` and `tar`; the shipping Box/resource restore path owns archive/resource state and enforces path containment, symlink/root safety, sha/size matching, bounded concurrency/disk-space budgets and deterministic cleanup. Attachment/resource paths also enforce byte caps and bounded parsing. No current LZ4 or Zstd production dependency is present.

The accepted tdesktop source observed so far proves the pinned LZ4 gitlink and build/package inputs, but this tranche still does not prove a shipping LZ4-specific consumer. Therefore no LZ4 runtime, Telegram compression subsystem, second archive owner or custom checksum/security primitive is added. The applicable source-neutral safety/state/failure semantics remain mapped to current Resource/Storage/Build/Quality/Packaging/Updater/License owners.

## Closure state

All 64 rows are `read_complete=true`, `responsibility_decomposition_complete=true`, `disposition=mapped-open`, `unknown_closed=false`, `omitted=false`. This advances deterministic read-through only; it does not close product/release responsibility or claim migration completion.

Accounting after this tranche: **10,412/16,125 read; 5,713 unread; 15,845 unknown; 0 omitted**. First unread is order 10,413 `Telegram/ThirdParty/lz4::ossfuzz/compress_fuzzer.c@edc8aad9a929dc7d384de191a812e83b632f22d8`.

Fresh exact-head GitHub Actions are required for this descendant.
