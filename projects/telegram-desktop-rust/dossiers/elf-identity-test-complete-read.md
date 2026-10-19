# Complete read — .agents/shared/evidence/elf_identity_test.py

Status: understood; responsibility decomposed; existing-owner production/evidence wiring audited; exact-head verification pending  
Upstream: `telegramdesktop/tdesktop@fab79561cf8030eade9d0098bc438346b8c39261`  
Tree: `76afabc085867a9c2e086489270d0ca0d5c9bcac`  
Path: `.agents/shared/evidence/elf_identity_test.py`  
Blob: `a75a29225b6eb70f255848e30026d02523d808f3`  
Comparator companion: `.agents/shared/evidence/elf_identity.py@70dc54620651e8c6fd9cce6966c9964b1a6c90d7`

## Live-baseline reconciliation

The original write lease was re-read before modification and then re-read again while GitHub Actions were queued. Live `telegramdesktop/tdesktop@dev` first advanced from `1e18da644e37856c9b1d87434bfcd6a2e3d11c57` to `fab79561cf8030eade9d0098bc438346b8c39261`, then advanced two additional commits to `34a86ed1a547ca6d12e25688c6289b60e6b6216c` (tree `41c7133ce1efab1e68bad4bcace2fea7cc10f24f`). The target test blob remains exactly `a75a29225b6eb70f255848e30026d02523d808f3`.

The first delta advanced `Telegram/lib_base` and `Telegram/lib_ui`; `lib_base` added two tracked recursive files while `lib_ui` added none. The second delta does not change direct gitlinks. It removes three legacy premium OBJ files, adds nine diamond binary resources and two shader files, for a net +8 root non-directory entries. GitHub's untruncated recursive root tree at `41c7133ce1efab1e68bad4bcace2fea7cc10f24f` reports 7,014 total entries, 6,681 blobs, 36 direct gitlinks, 3,271 SourceFiles blobs, 3,129 Resources blobs and 6,717 non-directory root entries. Because the nested gitlink pins are unchanged from the immediately preceding baseline, recursive non-directory authority is 16,202.

The latest delta's only path under the acquisition scanner's directories is `Telegram/cmake/qrhi_shaders.cmake`; its current content contains no acquisition-like git/curl/wget/URL/GIT_REPOSITORY/GIT_TAG tokens, so the already dispositioned 331/19/0 build-time acquisition accounting remains structurally unchanged pending exact-head Actions revalidation.

Closing this eighth ordered source entry at the current live baseline yields unknown=16,194, unread=16,167, omitted=0. Baseline readiness and final acceptance remain false.

## Source role and caller closure

This file is the independent qualification/control suite for the ELF execution-copy comparator. It is build/release evidence tooling, not Telegram product runtime and not a Telegram messaging, network, UI, search, identity, or storage owner. It imports the comparator directly as `subject` and drives the helper CLI through the sibling `elf_identity.py`.

The shared test loop routes Linux Debug execution-copy claims to `.agents/shared/evidence/elf-copies.md`, which explicitly invokes this full suite and requires exact digests, strip-debug equivalence, independent positive/negative controls, real sparse/high-offset checks, and both normal/optimized Python controls. Focused subsets are authoring aids only and cannot replace the complete suite.

Ninja configured-graph evidence remains independently owned by `.agents/shared/evidence/ninja_target.py` / `ninja-target-evidence.md`: it proves exact consuming graph membership but explicitly does not prove command execution, object existence, archive-member selection, or runtime reachability. TDRP recursive root/child/next/fixed-point authority, recursive build reachability, root source archives, Boost reachability, and dependency/source acquisition likewise remain separate source/build-input owners. This test suite does not absorb or duplicate them.

Fabushi therefore maps the applicable responsibilities into the existing source-neutral quality/release evidence owners: `TEST_STRATEGY.md`, `evidence-contract.md`, `release-entry-exit-criteria.md`, and the existing release workflow. No Telegram-specific test system, ELF-specific release owner, or second packaging truth is introduced.

## Independent literal fixtures and accepted controls

`fixture()` and `page_fixture()` construct ELF64 little-endian x86-64 inputs from literal ABI constants and offsets rather than asking the production comparator where fields live. They cover allocated text/data, TLS initialized/uninitialized state, BSS/NOBITS, zero-sized sections, overlapping conceptual metadata, dynamic/static symbol tables, section references, non-LOAD payloads, debug sections, section-name/string tables, prefix/tail page exposure, final writable BSS and independent interpreter records. Repacking and reordering controls independently reconstruct section/string/symbol bookkeeping.

Accepted controls include exact raw identity; qualified debug stripping with repacked/reordered bookkeeping; canonical local FILE/SECTION bookkeeping removal; qualified compressed-debug envelopes; valid section-index remapping; large NOBITS memory without proportional file reads; zero-filled unaccounted gaps; final writable BSS semantics; nonaliased page mappings; aligned nonfinal BSS/shared-page union; normal and optimized Python CLI modes; and a genuine GNU toolchain pair.

The genuine toolchain case requires `g++`, `strip`, `readelf` and `sha256sum`. It compiles a small C++ program with initialized data, BSS, TLS and TLS-BSS using `g++ -g -O0`, then runs `strip --strip-debug --no-merge-notes`. Independent `readelf` observations prove debug removal while data/BSS/TLS remain, independent `sha256sum` observations bind raw identities, the comparator passes in both normal and optimized Python, and a real stripped `.text` mutation is refused.

A separate sparse real-CLI control places the section table above 4 GiB, verifies sparse allocation, hashes complete logical files, compares them through the CLI, mutates high-offset metadata, and proves the one-MiB read ceiling without converting the sparse file into an in-memory fixture.

## Paired forbidden mutations

The suite proves the oracle can reject—not merely accept—protected semantics. Forbidden mutations cover loader bytes and exposed page padding; every program-header field and order; ELF header protected fields; allocated section fields; NOBITS/TLS mapping metadata; dynamic-symbol section retargeting; numeric/reference semantics; retained/unknown nonloaded payloads; nonzero uncovered trailer/gaps; retained static-symbol name/info/visibility/target/value/size/order; insertion/removal/reordering; forged debug/static roles; invalid static dependencies; compressed-debug kind/reserved/size/alignment; unqualified debug names/types/flags; exact raw section-name identity; malformed ELF/header/count/size/range/section/program/symbol grammar; load overlap and rounding overflow; partial EOF pages; independent INTERP/record aliasing; and real executable text mutation.

Paired positive controls remain beside these refusals so an implementation that rejects every transform cannot qualify.

## Stable acquisition, descriptor safety, and resource limits

Acquisition controls reject missing paths, directories, symlinks, FIFOs and other nonregular inputs without blocking. Descriptor tests verify read-only acquisition, close-on-exec, closure on success and each failure path, short-read refusal, and final stability failure. Deterministic late-change controls force same-size mutation, truncate, grow, pathname replacement and symlink replacement immediately before final verification and require a stability refusal; an atime-only read change is accepted because it does not alter artifact content identity.

Bounds are exercised at exact and one-over values with real `os.pread` interception. Per-file limits are: 16 GiB input, 1 MiB maximum read request, 1,024 program headers, 4,096 sections, 16 MiB section-name table, 4,096-byte section names, 4,194,304 symbols per table, 1 GiB private static-name table, 65,536-byte symbol names, 4 GiB aggregate resolved symbol-name bytes including NUL, 20 GiB aggregate symbol-name read bytes, 5,242,880 name-read calls, 4 KiB name-read chunks, zero name-cache bytes, 4 KiB interpreter bytes, 4 KiB load pages and 16 GiB declared uncompressed debug bytes. Over-limit declarations are rejected before they can drive large I/O; repeated names remain streamed rather than cached.

These Linux/ELF mechanics are not copied into current Fabushi macOS packaging. Their applicable source-neutral responsibility is that any future intentional executable-transform oracle must have independent exact/one-over resource controls and stable acquisition. Current macOS installation remains a stronger byte-exact executable-copy edge.

## Command harness, ownership, cancellation, and evidence

`FixtureCase.command` persists an attempted JSON record before launch and a final record for success, nonzero exit, launch failure, timeout, cancellation, wait error, or cleanup failure. The record binds argv and shell display, working directory, environment additions, stdout/stderr file paths, Python optimization mode, start/end/deadline, timeout, exit/outcome/exception, owned PID/PGID, cleanup signals/errors/remaining ownership, and direct-child reaping. Stdout/stderr go directly to files rather than pipes that descendants could keep open.

Each command starts a dedicated session/process group. The group leader remains unreaped as an identity anchor until signaling is complete. Cleanup sends TERM with a bounded grace period, escalates to KILL when needed, checks live members, then reaps the direct child. Cleanup/probe failure is explicitly `cleanup_failed` and cannot pass. SIGINT is deferred only across launch/cleanup/final-evidence critical sections; pending cancellation is recorded and propagated after cleanup, and the previous signal handler and mask are restored.

The isolated `_command_owner_probe` enables child subreaping only inside its own test subprocess. A compiler-like descendant ignores TERM and inherits output files; timeout, cancellation, injected wait error, cleanup-entry SIGINT races and successful parent exit all prove owned descendants are eventually reaped. A separate-session sentinel proves unrelated processes survive. Finalization tests inject SIGINT before/after final JSON writes, handler restoration, pending-signal checks, repeated delivery, nonzero exit, timeout, launch failure and a custom prior handler; cancellation remains durable while owned children are gone. The suite explicitly does not claim containment of daemonized/adversarial descendants that deliberately escape the process-group boundary.

Fabushi now owns this as a source-neutral evidence-command contract, not as an ELF-specific command runner. It applies when a future format-transform qualification harness actually spawns external tools. Incomplete command evidence, unverifiable cleanup, or collateral signaling cannot be accepted as passing evidence.

## Current shipping disposition

Current Fabushi macOS shipping is a byte-exact relation, not an intentional ELF transform:

`signed/notarized source executable -> source SHA-256/size -> fresh installed copy -> installed SHA-256/size -> source SHA-256/size recheck -> exact equality -> machine-readable exact-head evidence -> packaged acceptance`

The existing release workflow additionally verifies signing/ticket/Gatekeeper on the installed app. Equal raw bytes are stronger for this edge than a format-specific semantic transform exception, so this closure does not add an ELF parser or execute upstream qualification Python in production. If a future Linux/release edge introduces stripping or any intentional binary transform, the existing source-neutral contracts require a bounded format-specific oracle and its full independent qualification suite before that edge can pass.

## Traceability

Responsibilities:
- `TDRP-R9-TRANSFORM-QUALIFICATION-SUITE-001` — independent literal accepted/forbidden controls, real-toolchain qualification, acquisition races, exact/over resource bounds, and normal/optimized execution.
- `TDRP-R9-EVIDENCE-COMMAND-OWNERSHIP-001` — durable argv/env/stdout/stderr/timing/PID/PGID evidence plus bounded timeout/cancel/error descendant cleanup and unrelated-process survival.

Capabilities: `CAP-RELEASE-ARTIFACT-IDENTITY`, `CAP-RELEASE-TRANSFORM-EQUIVALENCE`.  
Oracles: `ORA-TDRP-TRANSFORM-EQUIVALENCE-001`, `ORA-TDRP-ARTIFACT-COPY-IDENTITY-001`.  
Invariants: `INV-TDRP-ARTIFACT-RAW-IDENTITY-001`, `INV-TDRP-ARTIFACT-SOURCE-STABLE-001`, `INV-TDRP-ARTIFACT-NORMALIZED-NOT-IDENTITY-001`, `INV-TDRP-EVIDENCE-COMMAND-OWNERSHIP-001`.

Existing production/evidence owner: Fabushi source-neutral test strategy, test evidence contract, release entry/exit gate and release packaging.  
New owner: none.  
Current platform disposition: exact-byte macOS copy; format-transform qualification dormant until an actual transform exists.

This closes exactly one recursive source entry at the live baseline. Reconstructed recursive inventory is 16,202; counters after closure are unknown=16,194, unread=16,167, omitted=0. Source closure, baseline readiness and final release acceptance remain false.
