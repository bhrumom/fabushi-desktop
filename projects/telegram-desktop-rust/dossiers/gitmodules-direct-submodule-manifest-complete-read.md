# .gitmodules direct submodule manifest complete read

## Exact source and parent evidence
- Upstream authority: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Strict100 source: `.gitmodules@2a71e861fab57dfd74d66e787867f2ede203a099`, 108 lines / 4,321 bytes, fully read.
- Parent Fabushi exact HEAD: `e4c9f80158f6bcd5d3dc4772f6bd4fec1a73eea5`, tree `d48373972863f474561af24bdf0cf7792822e188`.
- Parent TDRP run `39903360678`: 61/61 success; authority `12479289689` `sha256:4e17e5bd8c15656d7b4d1c1b9b5d89085cedbded68da3de62719e965ab55c5ee`; closure `12479679023` `sha256:2d6802579c4011b5fcf1d50ac5704a64cd368cece67db6e08a4f4dc12a6addd2`; reachability `12480355228` `sha256:7156152fc061a27c2da6218aa848f07cca88a2066dcbe43b91cb6efa1e56a417`.
- Parent Rust desktop runtime run `39903360611`: 6/6 success; artifact `12480066202` `sha256:e3769ec5d3735fdb46ff1b51722159cfb987015775aa59271e7b4f944691f06d`.

## Manifest and lock reconciliation
The exact manifest has 36 submodule stanzas, 36 unique paths, 36 unique normalized repository URLs, no `branch =` overrides, no duplicate names/paths, and no missing path/URL. Its path set exactly equals the 36 `upstream.lock.json.direct_gitlinks` paths, and every normalized repository mapping agrees 36/36 with zero mismatch.

`.gitmodules` is repository-location metadata. The immutable child revision is the parent Git tree's mode-`160000` object SHA, durably recorded with path/repository/commit in `upstream.lock.json.direct_gitlinks`. A URL, redirect, default branch, remote HEAD, or provider availability is not an immutable source pin.

## Existing-owner-first
The responsibility is already implemented by the source-neutral TDRP source-authority owners:
- `upstream.lock.json#direct_gitlinks` is the durable direct path/repository/commit lock.
- `discover-tdrp-recursive-root-authority.mjs` parses the manifest, joins mode-`160000` gitlinks by path and fails when a gitlink lacks a URL.
- `discover-tdrp-recursive-fixed-point.mjs` combines locator URL with the immutable gitlink object SHA and recurses until all gitlink authorities resolve.
- `tdrp-revision9-authority.yml` requires `unresolved_gitlink_authorities === 0` while explicitly refusing to fabricate full source closure.

Preflight found zero prior `.gitmodules` source-path rows, zero prior source-blob rows, and zero `TDRP-R9-DIRECT-SUBMODULE-MANIFEST-PROVENANCE-001` rows. Each selected target symbol occurs exactly once in its exact-head owner/validator and has zero prior target-path+symbol ownership rows. No second dependency manager, source lock, Telegram submodule runtime, networking/updater owner, or acceptance root is introduced.

## Fail-closed boundaries
A missing or duplicated manifest path/URL, a manifest/lock repository mismatch, or unresolved recursive gitlink authority is incomplete evidence. Remote location drift requires a new exact upstream tree/lock reconciliation; it must never silently re-pin source. Child repositories retain independent content, license, security and provenance obligations. Gitlink fixed-point closure does not imply `unknown=0`, `unread=0`, source closure, production completeness or release acceptance.

## Traceability
- Requirement: `TDRP-R9-DIRECT-SUBMODULE-MANIFEST-PROVENANCE-001`
- Oracles: `ORA-TDRP-GITMODULES-PATH-URL-BIJECTION-001`, `ORA-TDRP-GITMODULES-GITLINK-PIN-SEPARATION-001`, `ORA-TDRP-GITMODULES-FIXED-POINT-001`
- Invariants: `INV-TDRP-GITMODULES-URL-NOT-COMMIT-PIN-001`, `INV-TDRP-GITMODULES-NO-SECOND-DEPENDENCY-OWNER-001`, `INV-TDRP-GITMODULES-NO-SOURCE-CLOSURE-FABRICATION-001`
- Contracts: `CONTRACT-TDRP-GITMODULES-DIRECT-GITLINK-JOIN-001`, `CONTRACT-TDRP-GITMODULES-MISSING-URL-FAIL-CLOSED-001`, `CONTRACT-TDRP-GITMODULES-LOCK-MANIFEST-AGREEMENT-001`
- Faults: `FAULT-TDRP-GITMODULES-MISSING-URL-001`, `FAULT-TDRP-GITMODULES-DUPLICATE-PATH-001`, `FAULT-TDRP-GITMODULES-LOCK-REPOSITORY-MISMATCH-001`, `FAULT-TDRP-GITMODULES-REMOTE-LOCATION-DRIFT-001`
- Security: `SEC-TDRP-GITMODULES-EXTERNAL-URL-NOT-TRUST-PROOF-001`
- Regression: `REG-TDRP-GITMODULES-DIRECT-GITLINK-SET-001`, `REG-TDRP-GITMODULES-NO-SOURCE-OMISSION-001`

Closing strict100 yields unknown=16,276, unread=16,251, omitted=0, responsibilities=118. Next strict root is index 101 `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32`. Fresh child exact-head TDRP/Rust evidence is required.
