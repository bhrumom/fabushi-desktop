# Isolated media test repair handoff

## Status and authority

The authorized test-only candidate is complete for parent integration and independent review. It has **not been compiled, parsed by an executable oracle, tested, linted, or otherwise executed locally**. Root owns the complete six-path review, live-parent reconciliation, publication and GitHub Actions verification.

- Phase: inherited `xhigh`; configured model unchanged.
- Exact repair base: `4f0639321c4ae03cc7dbef1d8e49dea0f042e180`, tree `cbc9418ee7572f44d59e067e450a67f6c0bc22a0`.
- Both edited base blobs are byte-identical to the H74 inputs diagnosed in the original artifacts.
- Shared MSU-v1 spec: `e42ea96e12c4647c23d98154fabd4a87749ea5c7`, read completely before editing.
- Read before their respective changes: `decision-before-fix.json` and `decision-windows-crlf-before-fix.json`.
- Only the two authorized paths inside `media-candidate/` were edited. The real checkout, production behavior, workflow, capture, schema, ledger and other owners were not changed.

## Deliverables

| Deliverable | Location |
| --- | --- |
| Complete candidate AgentDb file | `media-candidate/source/host/src/extensions/session/agent_db.rs` |
| Complete candidate contract test file | `media-candidate/source/host/tests/openclaw_sessions_and_config_contract.rs` |
| Complete binary-safe, full-index two-path Git diff | `media-candidate.diff` |
| Base/candidate byte counts, Git blob SHAs and SHA-256 values | `media-candidate-manifest.json` |
| Original Linux diagnosis and artifact evidence | `../h74-media-failure-read/diagnosis.md` |
| Additional Windows diagnosis and exact byte proof | `../h74-media-failure-read/windows-supplement.md`, `windows-supplement-byte-proof.json` |

The full diff is 14640 bytes, SHA-256 `e098cba321192b43d1716fdb620e58ccbb0cb1c0f8c0a30b31b0a8a2015f256a`.

| Candidate path | Bytes | Git blob SHA | SHA-256 |
| --- | ---: | --- | --- |
| `source/host/src/extensions/session/agent_db.rs` | 426814 | `87186bc4b246d88a5ce51ebda0623b76f4425d3d` | `1d1632094c80d6ba204f7c3a75db5eab0619628b25898449a1880e5ccca81900` |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | 165965 | `35719fc77edb1fea8f7acf0b22524c6eff7b29aa` | `a12ea0167dd7e82d8c4982d9e9ac9741466b23e6c90236a4650056fb90cfa8fc` |

## Changes and preserved requirements

### Exact owner transcript expectations

Inside the private `media_settlement_tests::Fixture`, extracted the existing exact send-plus-receipt vector into `expected_committed_entries`. The storage assertion and the two failing fresh/different-owner reader assertions now use that same expected vector.

The complete receipt identity, kind, session, work ID, lifecycle revision, original media path and send-entry IDs are preserved. So are the exact send entry, row ordering, durable fence and file-byte checks. Every typed-unknown, actual fault-applied, one-shot lease, exact connection retirement, owner liveness and no-publication assertion remains present.

All AgentDb edits are confined to the private media test module. Byte comparison establishes that the complete file prefix preceding that module is identical to the base; the full diff shows no production edits.

### Inherent-owner proof with trait coexistence

`media_commit_inherent_impl` now excludes trait implementations before testing inherent-owner uniqueness. The specialized token walk in `media_commit_owner_is_handled` also skips trait implementations before parsing inherent syntax.

These skips change only owner selection. No trait implementation text is removed from the source image. The existing global raw-COMMIT prohibition, one-caller rule, aliased/associated/macro-call detection and unchanged channel oracle still inspect that complete source.

Existing uniqueness, exact-type, attributes, signature, three admission guards, exact helper methods, entry prefix, settlement tail and mutation negatives remain intact.

### Windows CRLF source images

Only test-owned source images convert CRLF to LF:

1. Actual-source fixtures normalize before multiline signature lookup and mutation construction, so replacements cannot silently miss a Windows checkout.
2. The media adapter proof normalizes its input boundary, so LF and CRLF counterfactuals exercise the same logical source.

No bare-CR stripping or general whitespace normalization was introduced. Original artifact logs, capture receipts, input hashes and production files were not normalized or rewritten.

## Added qualification within existing test names

No top-level test was added, removed or renamed. The existing 13-case AgentDb group and 27-case selected transaction group retain their named capture contract; no workflow/capture change is required.

The existing positive media test now explicitly requires the actual `Drop` implementation and verifies its entire body survives normalization. It also accepts additional harmless traits for both `SandAgentDb` and `MediaSettlementCommands`, with their full source retained.

LF and CRLF images must yield identical remaining source for the actual baseline and the harmless-trait coexistence control. Negative controls cover ordinary and raw-identified aliased helper calls inside traits for both owner types; each is checked with LF and CRLF. Raw COMMITs in traits for both types remain visible to the global prohibition, also under both line endings. The existing settlement-tail mutation loop additionally requires CRLF to preserve each rejection, including changed unknown classification, rollback, retirement and publication handling.

These controls are written source, not newly earned passing evidence. The original Linux six green media-mutation negative tests remain unqualified until the repaired positive baseline and mutated controls execute together.

## Remaining evidence and handoff boundary

The original Linux failures are two receipt-vector expectations plus six baseline-media-oracle cascades. The original Windows command adds one distinct multiline-signature fixture failure; its exact failed-set difference and CRLF byte identity are retained in the supplemental report.

A focused successor-H Actions execution must cover both original commands on Linux and Windows:

```text
cargo test --locked --manifest-path source/host/Cargo.toml --lib extensions::session::agent_db::media_settlement_tests:: -- --test-threads=1
cargo test --locked --manifest-path source/host/Cargo.toml --test openclaw_sessions_and_config_contract commit -- --test-threads=1
```

Preserve original logs and still-valid positives. Fresh-owner assertions after the original panics have not yet executed successfully. The existing MSU actual App/Session/Runner, feature-off shipping and other applicable gates remain separately required; this test-only source repair does not establish production or migration acceptance.

No reviewer was spawned, no dispatch or push was issued, and no active implementation owner/session was overwritten.
