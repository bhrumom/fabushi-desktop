# Windows H74 supplemental diagnosis

This supplement preserves the completed Linux diagnosis without rewriting its original evidence.

## Extra failure and exact bytes

The original Windows `transaction-commit-owners.log` retained by the evidence collector at `../h74-additional-evidence/windows-artifact/` has SHA-256 `2e25b24f4252140b8d4c9f063b62b9d596b62d2ed833fc3b2afef896fa922bf3`, and reports **20 passed / 7 failed**. Comparing the exact failed-name sets to the original Linux log identifies exactly one extra failure:

`media_commit_oracle_rejects_missing_duplicate_or_conditional_owner`, H74 `source/host/tests/openclaw_sessions_and_config_contract.rs:1945:59`, panic `actual media signature`.

The failing operation is `source.find(MEDIA_COMMIT_OWNER_SIGNATURE).expect(...)`, before any owner mutation. It searches a multiline string constant against `include_str!` of the actual checkout.

This is an additional **TEST_FLAW** caused by line-ending-sensitive fixture construction. The Windows capture records the same Git blob identities but CRLF working-tree source. Byte reading independently establishes that converting the retained exact LF blobs to CRLF reproduces both Windows input byte counts and SHA-256 values exactly:

| Input | Windows bytes | Windows SHA-256 |
| --- | ---: | --- |
| `agent_db.rs` | 436762 | `276b6c58a5260c7c0c3fd1561d65249eba307827767c894e149ba0e4bcb9fdb9` |
| `openclaw_sessions_and_config_contract.rs` | 165691 | `119cf6ef914b4342719f63cfc7a032c754152b0c6c04a0f9d031d665ef83d946` |

The matching proof and exact Linux/Windows failed-name sets are retained in `windows-supplement-byte-proof.json`. No product/parser/test execution was performed. Original artifact bytes, receipts and logs remain unchanged.

## Narrow repair proposal sent to the source-neutral parent

Normalize only CRLF to LF in the test oracle's in-memory source image. Normalize actual-source fixture input **before** multiline `find`/`replacen` construction; boundary normalization alone would leave Windows mutations as no-ops. Also normalize at the composed media-oracle boundary so explicit CRLF counterfactuals can qualify the same logic.

Retain all tokens apart from CRLF transport spelling, all original raw input hashes, the exact command/owner checks, and the entire trait bodies in global scope. Pair LF and CRLF positive and negative controls. Extend existing named tests so the strict capture retains its 27-name command contract and no workflow edit is needed.

The repository already uses `include_str!(...).replace("\\r\\n", "\\n")` in `box_store_sync_production_wiring_contract.rs` for source-image comparisons. No production reader, SQLite transaction, fixture receipt contract, evidence metadata, or other source format should be changed for this failure.

This proposal requires a supplemental durable FIX decision by the parent before its implementation. The previously authorized trait/fixture changes are separate from this newly established failure cause.

## Subsequent authorized implementation

The parent subsequently recorded and authorized `H74-WINDOWS-CRLF-ORACLE-INPUT` in `../h74-shared-contract-recovery/decision-windows-crlf-before-fix.json`, which was read before normalization edits. The isolated implementation and its unchanged verification boundary are described in `../h74-shared-contract-recovery/media-repair-report.md`. This does not change any original failure or passing-evidence claim above.
