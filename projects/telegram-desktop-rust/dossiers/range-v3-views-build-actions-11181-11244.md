# Range-v3 views, build provenance and action regression dossier — orders 11,181–11,244

## Exact authority

- Upstream: `ericniebler/range-v3@a81477931a8aa2ad025c6bda0609f38e09e4d7ec`, mounted by accepted `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`.
- Exact Fabushi evidence HEAD: `af7183eb431ee41730ba0631f2306c186478890d`.
- Rust desktop runtime: `38151143307` — completed/success.
- Source-authority job: `114504686124`.
- Artifact: `11706761066`.
- Digest: `sha256:09b595119048168b7fb4f857d8ce3e642e25c46163de2ff93a2eea6a778c9d32`.
- 64/64 identities were materialized from the accepted recursive inventory and verified against exact Git blob objects before upload.

## Responsibility decomposition

Orders 11,181–11,223 are generic range-v3 lazy view/adaptor and iterator semantics: lifetime/dangling fences, category propagation, cardinality, cache and bound behavior, predicate/generator lifetime, and composition across join/split/slice/stride/transform/zip and related views.

Orders 11,224–11,225 are C++ standard iterator/associated-type compiler compatibility shims. They do not justify a C++ compatibility sidecar in Fabushi.

Orders 11,227–11,230 are development-only performance benchmarks. Orders 11,231–11,244 are action regression contracts covering mutation, reference identity, bounded container operations, remove/reverse/shuffle and quality behavior.

## Recursive build-time acquisition finding

Order 11,226 `install_libcxx.sh@dcbf38c1c3b06c3b6baf9e81b43d29d91376b51b` contains two acquisition modes:

- lines 51–53: `git clone --depth=1` of `llvm-mirror/llvm`, `llvm-mirror/libcxx`, and `llvm-mirror/libcxxabi` with no commit pin;
- lines 58–69: version-derived LLVM release URLs followed by `curl -LO` archive downloads with no in-script digest verification.

This finding is now recorded in `inventory/build-time-acquisitions.json` without changing the closed 330-line top-level candidate accounting. Existing-owner-first maps it to Build / Quality / Dependency-Provenance / License. Fabushi shipping remains fail-closed and does not import either mutable acquisition path.

## Existing-owner-first result

No source in this window establishes a new Search, Conversation, Message, Agent, Marketplace, Computer, Automations or other product-domain owner. Applicable responsibilities remain with existing Build / Quality / Dependency-Provenance / License and native Rust/TypeScript runtime/standard-library owners. No range-v3 runtime, C++ compatibility sidecar or second container abstraction is introduced.

All 64 source-disposition rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`.

## Accounting

After this window: **11,244 / 16,125 read; 4,881 unread; 15,845 unknown; 0 omitted**.

Next unread: **11,245** `Telegram/ThirdParty/range-v3::test/action/slice.cpp@ff01398e6d72622e3c105dcf3bc9a23380f8c7bc`.
