# CLAUDE host instruction pointer complete read

## Exact source and parent evidence
- Upstream authority: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Strict102 source: `CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155`, 4 lines / 100 bytes, fully read.
- Parent Fabushi exact HEAD: `a66bf8b7578538642989756feb4a5c0571f8b655`, tree `ad5977bb3b237484b4a04c76456759b217a7ffcd`.
- Parent TDRP run `39906002549` attempt 2: 61/61 success. Attempt 1 failed only in recursive root shard 3 while cloning Chromium breakpad with HTTP/2 INTERNAL_ERROR / early EOF; the failed jobs were rerun without a source change and the same exact HEAD passed.
- TDRP authority `12480139173` `sha256:b4adbde7994599cced461808f4ce0a5ca9ea8293df161d91ceb801fc4dedc90a`; closure `12480364389` `sha256:949008ba2f7192c847ddf79d6fa05560b6abea85c6bee09b4f8af431782deaf3`; reachability `12481560876` `sha256:1c10037ba6c46fc097de5b70fa5b67dee16a25637ed35f8a2733feb277d5292c`.
- Parent Rust desktop runtime run `39906002548`: 6/6 success; artifact `12480568943` `sha256:4b3835c9ceb2d0ff8e9e229c617c676c0bf798305f19c1ac8cbb7fae633362e7`.

## Responsibility
The upstream file is only a Claude-host pointer: it says to read `AGENTS.md` and treat it as canonical repository-wide instructions. It carries no independent shipping product behavior, state machine, service contract or UI. The applicable source-neutral responsibility is repository instruction delegation and precedence.

Current Fabushi already owns that responsibility in root `AGENTS.md`: repository instructions are source-neutral; root AGENTS is repository-wide unless a more specific nested AGENTS applies; host pointers such as CLAUDE.md/GROK.md may add host-adapter mechanics but do not override canonical or nested instructions unless canonical policy delegates that choice.

## Existing-owner-first and uniqueness
Fresh preflight at parent exact HEAD found zero prior source-path, source-blob, responsibility and selected target ownership rows. The selected root AGENTS instruction-precedence statement occurs exactly once. No Claude-specific instruction root, runtime, task queue, build authority, product owner, network owner or UI root is introduced.

## Fail-closed boundaries
This pointer cannot establish source closure, product implementation, security authority, packaging/release acceptance or test evidence by itself. A changed pointer requires a new exact blob and fresh reconciliation. Current task/spec/source authority and more-specific nested repository instructions remain authoritative within their declared scope.

## Traceability
- Requirement: `TDRP-R9-REPOSITORY-HOST-INSTRUCTION-POINTER-PROVENANCE-001`
- Oracles: `ORA-TDRP-CLAUDE-POINTER-DELEGATES-TO-AGENTS-001`, `ORA-TDRP-CLAUDE-POINTER-NO-OVERRIDE-001`
- Invariants: `INV-TDRP-CLAUDE-NO-SECOND-INSTRUCTION-ROOT-001`, `INV-TDRP-CLAUDE-NO-DOCUMENT-AS-PRODUCTION-EVIDENCE-001`
- Contracts: `CONTRACT-TDRP-CLAUDE-AGENTS-DELEGATION-001`, `CONTRACT-TDRP-CLAUDE-PRECEDENCE-001`, `CONTRACT-TDRP-CLAUDE-EXACT-HEAD-EVIDENCE-001`
- Faults: `FAULT-TDRP-CLAUDE-STALE-POINTER-001`, `FAULT-TDRP-CLAUDE-OVERRIDE-ATTEMPT-001`, `FAULT-TDRP-CLAUDE-DOCUMENT-PROMOTION-001`
- Security: `SEC-TDRP-CLAUDE-POINTER-NOT-AUTHORIZATION-001`
- Regression: `REG-TDRP-CLAUDE-NO-SOURCE-OMISSION-001`, `REG-TDRP-CLAUDE-NO-BOT-OWNER-REGRESSION-001`

Closing strict102 yields unknown=16,274, unread=16,249, omitted=0, responsibilities=120. Next strict root is index 103 `CMakeLists.txt@59477e251bbfea8d5efaf0bcda53798f92695449`. Fresh child exact-head TDRP/Rust evidence is required.
