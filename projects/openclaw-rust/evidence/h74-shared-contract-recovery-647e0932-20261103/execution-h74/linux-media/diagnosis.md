# H74 media failure diagnosis

## Scope and verdict

Read-only, independent bounded diagnosis of Runtime run `41580539491`, attempt `1`, job `125734079585` (`sqlite-native-path`), media step 8. Phase effort: inherited `xhigh` per the parent assignment; configured model unchanged. No product/test/workflow/ledger edits, executable verification, dispatch, push, or polling loop were performed for this diagnosis.

**All eight observed failing assertions are TEST_FLAW.** The two AgentDb failures expect a presentation-only vector from the owner reader, which deliberately includes the durable settlement receipt. The six source-oracle failures share an unconditional rejection of the valid `Drop` trait implementation. The same oracle has a second trait-handling defect that would remain after fixing only candidate selection. These failures do not establish an implementation bug or an environment failure. MSU acceptance remains incomplete pending repaired checks and the separately required actual App evidence.

Read root `AGENTS.md`, the complete exact-H74 MSU-v1 specification, the actual transaction/fault/retirement/reader/receipt code, the private fixtures and failed tests, the new media oracle and its surrounding admission/channel oracles. General OCRM entry and evidence policy were consulted selectively. No source-accounting or cron-anchor work was reviewed.

## Immutable subjects and retained evidence

- Repository: `bhrumom/fabushi-desktop`.
- H74: `74f11d3d31c6245ac5886dc0181460b2d4349744`; tree `aca91e1cfbedeecb039d442fb3528caac89873a3`.
- U: `1440bb1771372334bea23d7642bb52deb7db1845`; tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Original artifact ID `13205686727`: retained unchanged as `h74-runtime-original-artifact13205686727.zip`, **142379 bytes**, SHA-256 `284a316258ebf9b60dd29d781c6aceb44ef3d1ca351a3337e0945efd7b2699b5`.
- Connector download response: `artifact-download-rawreceipt.json`, SHA-256 `ae058e3bee8594cf047a00f718ba014f6e5abb6f05dfad4d1465220cd3361965`.
- Original capture receipt: `original-artifact/receipt.json`, **120067 bytes**, SHA-256 `5fc8dfd24cf2635e253056c12d06c1f67f29e66f5cabe9e3003a1efc0bdfc6e2`.
- AgentDb failing command log: `original-artifact/media-settlement-owner.log`, **30567 bytes**, SHA-256 `26ab3027d0bd03ff450927df814349347f9fb104f0b8ec2677b4126aa838fad8`.
- Source-oracle failing command log: `original-artifact/transaction-commit-owners.log`, **32185 bytes**, SHA-256 `921b7317adf72cfad5ddd22b541f1e366a4d8beddc48214b913f642fbaaf1bc5`.

All 19 original per-command log hashes match the original receipt. The receipt reports `inputs_unchanged=true`, `capture_exit=0` and no execution error for both failed commands. Their Cargo exits are 101. The exact source blobs were read through the connector because the local tree contains their identities but not all blob bodies. Retained source bytes reproduce the exact Git blob SHA and the receipt's before/after SHA-256 values:

| Exact retained source | Git blob | SHA-256 |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `e42ea96e12c4647c23d98154fabd4a87749ea5c7` | `6b8f4eff2cb191259df998f598a3375de12522e2bc2120ce312dbb1c15e6eb68` |
| `source/host/src/extensions/session/agent_db.rs` | `6c27e9bb9f665b1160457b9acc29dd5d64fba289` | `9a8ad4aa952f11a3d7595eebf636f7d20c450381120416c00667b93ef0149c7b` |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `ba7dde9c12eb14cb0a0615a49a013a2e126b860d` | `e61629a709e4154dda1297d3f5fa9fd13efc38c00d67c08edbc93fb70a27d32e` |
| `source/host/src/extensions/session/agent_db_serde.rs` | `d1885e2d31313c1f525fffd85bafc5031cf8d127` | `aeb0c23b3c5ca02c2a96ba7d0e2063cf66e293aff00a72746457dc83ce829c5b` |

The supplemental run metadata response reported a stale `queued` status, so it was used only to recover the branch/subject identity, never to overwrite the failed command evidence. It is retained unchanged in `run-metadata-rawreceipt.json`.

## Finding 1 — owner projection mistaken for presentation projection

**Blocking TEST_FLAW**, private tests in `source/host/src/extensions/session/agent_db.rs`:

| Test | H74 failure line | Exact comparison shape |
| --- | --- | --- |
| `media_settlement_lost_commit_acknowledgement_retires_without_publication` | 9362 | `reopened.get_transcript_entries()`: actual `[send-message, outgoing-media-settlement]`; expected `[send-message]` |
| `media_settlement_fault_matches_three_identities_and_only_its_exact_owner` | 9577 | `other.get_transcript_entries()`: actual `[send-message, outgoing-media-settlement]`; expected `[send-message]` |

The actual marker has `id="outgoing-media-settlement:media-work"`, `kind="outgoing-media-settlement"`, the exact fixture session and lifecycle revision, `mediaWorkId="media-work"`, the original file in `mediaPaths`, and `transcriptEntryIds=["media-send"]`. The original logs retain the complete, unredacted fixture values and both assertion sides.

Causal chain:

1. MSU-01 preserves the existing exact receipt/fence formats; MSU-05 and MSU-09 require durable committed rows and bytes to survive unknown outcomes and fresh owner observations.
2. The real transaction `append_transcript_entries_and_release_media` inserts the send, then the receipt, then clears the fence in the same transaction (H74 lines 5035–5065).
3. `get_transcript_entries` delegates to `read_transcript_entries_from_db` (5843–5856), which uses `parse_owner_transcript_entry` (7777–7789).
4. `agent_db_serde.rs:169–188` explicitly preserves durable control records, including `outgoing-media-settlement`, for owner recovery. Presentation filtering is a different decoder.
5. The same test fixture already demands precisely the two rows in `assert_committed_storage` (9189–9204). The failed one-row expectations contradict their own exact durable-state oracle.

**Minimal repair basis:** share the fixture's independently specified exact two-row expected vector between `assert_committed_storage` and these two getter assertions. Preserve the complete receipt fields, ordering, exact send contents, file bytes, released fence, lease observations, retirement/handle counts, and publication assertions. Do not change production readers or hide/delete the receipt.

Retained partial evidence is narrower than a passing test. In the lost-ACK case, actual COMMIT fault application, typed unknown, exact owner retirement, exact committed rows/bytes, unchanged write generation, zero publication and refusal of owner reuse all passed before the first fresh-owner getter assertion failed. The first reopen succeeded; its later admission assertion and second loop iteration did not execute. In the identity case, all three mismatch controls, different-owner normal commitment without consuming the fault, subsequent exact-owner applied lost ACK, typed unknown, exact retirement with one other handle, and other-owner liveness passed. Its final `events.assert_none()` occurs after the failed assertion and did not execute.

## Finding 2 — valid trait implementation always invalidates the media proof

**Blocking TEST_FLAW**, `source/host/tests/openclaw_sessions_and_config_contract.rs`.

At 1693–1708, `media_commit_inherent_impl` first selects every implementation whose self-type's last segment matches the requested owner. For `SandAgentDb`, this selects both the inherent implementation at `agent_db.rs:951` and `impl Drop for SandAgentDb` at 6237. The uniqueness condition at 1703 fails before the later `trait_.is_none()` condition can distinguish them. Consequently `media_commit_owner_is_handled` returns `None` at 1744 for the unchanged valid source.

A second, latent instance must be repaired in the same change: the token traversal at 1785–1795 accepts any implementation with self type `SandAgentDb`, then reparses it as `impl Type { ... }`. A trait implementation reaches `for` where the parser expects the opening brace. Fixing only candidate selection would leave this parse failure.

All six observed failures are consequences of the same failed baseline media proof:

| Test | H74 line |
| --- | ---: |
| `media_commit_oracle_accepts_actual_owner_and_preserves_admission_guards` | 1889 |
| `media_commit_oracle_preserves_raw_commit_prohibition_outside_proven_method` | 2097 |
| `test_agent_db_has_no_direct_commit_error_propagation` | 2360 |
| `test_commit_error_oracle_rejects_unguarded_propagation_and_missing_cleanup` | 2411 |
| `channel_commit_oracle_retains_unsafe_owners_before_after_and_inside_helper_impl` | 2471 |
| `channel_commit_oracle_rejects_hidden_owner_and_aliased_calls_in_actual_source` | 2619 |

Their exact `expect` messages are retained at `transaction-commit-owners.log:788–815`. This is not evidence that the unchanged channel command adapter regressed. The first test had already passed its independent owner-local admission-guard check before the media exception failed.

**Minimal repair basis:** exclude trait implementations from inherent-owner candidate selection and from this specialized inherent-body token walk. Keep trait implementations intact in the returned source so the existing global raw-COMMIT prohibition and extra/aliased/macro-caller checks still inspect them. Preserve unique inherent owner/method checks, conditional/foreign/duplicate rejection, typed signature, three admission guards, exact command methods, settlement prefix/tail, and all channel checks. Add meaningful controls for actual baseline trait coexistence and for raw COMMIT/aliased helper use inside a trait body; do not increase a global caller allowance or exempt a whole implementation.

## Evidence retention and recovery boundary

The original native capture contains 19 commands: **17 fully accepted groups with 170 passing tests**, plus **11 passed / 2 failed** in AgentDb and **21 passed / 6 failed** in the transaction contract (202 observed passes, eight failures overall).

Retain all 17 accepted groups and the 11 completed AgentDb controls at their exact original H74 subject. The six passing `media_commit_oracle_rejects_*` tests are **observed green but not valid mutation-sensitivity proof**: they all ask the already-rejecting baseline to return `None`. Their qualification must be repeated after the positive baseline works. The other 15 completed transaction-contract tests retain their original bounded evidence, including the standalone channel controls and ordinary transaction owners.

The two failed commands, from cwd `/home/runner/work/fabushi-desktop/fabushi-desktop`, were:

```text
cargo test --locked --manifest-path source/host/Cargo.toml --lib extensions::session::agent_db::media_settlement_tests:: -- --test-threads=1
cargo test --locked --manifest-path source/host/Cargo.toml --test openclaw_sessions_and_config_contract commit -- --test-threads=1
```

After reviewed test-only repair, these are the focused invalidated commands, including their positive and negative controls, for execution in GitHub Actions. No local executable check was run. Source inspection is not a substitute for repaired execution or for separate MSU-10 actual App/Session/Runner acceptance.

## One bounded latest-head comparison

After binding the diagnosis to H74, the authoritative branch `codex/openclaw-rust-fusion-chat-20261101-writer` was read once at `4f0639321c4ae03cc7dbef1d8e49dea0f042e180` (root-provided tree `cbc9418ee7572f44d59e067e450a67f6c0bc22a0`). Exact-ref connector reads of both implicated files return the **same blob SHAs and identical full contents as H74**. No overlapping fix for these specific causes is present in that snapshot. `branch-head-rawreceipt.json` and `latest-comparison.json` retain the binding. No active owner/session was overwritten.
