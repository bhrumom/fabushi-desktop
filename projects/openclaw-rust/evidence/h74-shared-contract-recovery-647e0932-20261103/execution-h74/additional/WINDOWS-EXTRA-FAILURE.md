# Windows-only transaction-owner fixture failure

## Observed original evidence

H74 `74f11d3d31c6245ac5886dc0181460b2d4349744`, runtime run `41580539491`, attempt 1, Windows job `125734079542`.

The original `transaction-commit-owners.log` reports **20 passed, 7 failed**. Relative to the six Linux failures supplied by the parent, the Windows-only additional failure is:

```text
---- media_commit_oracle_rejects_missing_duplicate_or_conditional_owner stdout ----

thread 'media_commit_oracle_rejects_missing_duplicate_or_conditional_owner' (2548) panicked at tests\openclaw_sessions_and_config_contract.rs:1945:59:
actual media signature
```

Original log lines 855–858; all seven unmodified panic blocks are retained in `windows-seven-failures-original.log`. The original full log SHA-256 is `2e25b24f4252140b8d4c9f063b62b9d596b62d2ed833fc3b2afef896fa922bf3`.

## Causal source and byte evidence

Exact H74 test blob `ba7dde9c12eb14cb0a0615a49a013a2e126b860d`:

- Lines 1597–1604 define `MEDIA_COMMIT_OWNER_SIGNATURE` as a multiline raw string.
- Line 1944 obtains the production source with `include_str!("../src/extensions/session/agent_db.rs")`.
- Line 1945 calls `source.find(MEDIA_COMMIT_OWNER_SIGNATURE).expect("actual media signature")`, before exercising the counterfactual matrix.
- The adjacent boundary split at 1946 and complete-method suffix at 1948 also assume LF line endings.

Production owner blob is `6c27e9bb9f665b1160457b9acc29dd5d64fba289`. The original Windows receipt records working-tree SHA-256 `276b6c58a5260c7c0c3fd1561d65249eba307827767c894e149ba0e4bcb9fdb9`, while its normalized blob identity agrees with H74. Independently converting exact Git LF bytes to CRLF reproduces that working-tree hash exactly.

Literal byte comparisons, recorded in `windows-extra-failure-byte-evidence.json`, find the LF signature once in the Git source, zero times in the receipt-proven Windows CRLF source, and its CRLF spelling once in those same Windows bytes.

The [official Rust Reference input-format rules](https://doc.rust-lang.org/stable/reference/input-format.html) state that source CRLF is normalized before tokenization, whereas `include_str!` and `include_bytes!` do not apply that transformation. Thus the compiled raw-string constant has LF, while the included source retains CRLF. This explains the observed panic at the literal search without executing the repository parser or test locally.

## Assessment

**TEST_FLAW for this Windows-only additional failure:** the counterfactual fixture cannot reach its intended negative cases because it assumes a line-ending representation that differs from the actual Windows checkout. This diagnosis does not clear the six common transaction-owner failures or the two media-settlement failures.

A focused repair should make this source fixture's signature/boundary/suffix operations agree on line-ending representation, or extract the method through an already qualified source mechanism, while retaining every negative case and assertion. Fixing only the initial `find` leaves the adjacent LF-only operations exposed. The parent owns implementation and any future exact-subject execution; no mutation or rerun was performed here.

