# MSU-v1 shared COMMIT oracle worker handoff

## Terminal state

**IMPLEMENTED; source self-review complete; exact-H executable verification pending.** The shared contract target now composes an independently bounded media command/consumer proof with the unchanged channel command/consumer proof and the unchanged remaining-source bare-COMMIT prohibition. No parser, test, test-list, build, lint, schema, capture, generator, or htch validation was executed. This is not a test-pass or acceptance verdict.

Configured model and actually inherited **xhigh** were retained. The assigned scope involves transaction uncertainty, source-owner recognition, macro/alias boundaries, and preservation of existing security/correctness gates.

Only the additionally authorized production-repository path was edited during this phase:

`/workspace/scratch/7d27ea239237/media-settlement-work/source/host/tests/openclaw_sessions_and_config_contract.rs`

The earlier DB source/patch/report were not changed. No App, Cargo, composition, spec, schema, ledger, workflow, capture, or other repository path was changed by this phase. No commit/push occurred.

## 1. Exact source and artifact identities

The oracle base was copied from the authorized `media-settlement-current` path after checking its exact Git blob. It is identical at fdb and the independently fetched a2 object:

- fdb: `fdb0f6acb06165b6f2796961186e06a2de98c229`, tree `abeb6ffdf09b676279462fea3b9e1f6d72abc1b6`.
- a2: `a2a0c9ef326c1ddeb09b4f9180267fdaeb21db74`, tree `a0dceefeae75fccd7ab5aec143f6d651f7cf2fd3`.
- Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.

| Artifact | Git blob | SHA-256 | Size |
|---|---|---|---|
| Original oracle | `2049875451202ba05aecb7e5de9a6841dc378d63` | `4bc59d27b1b9c3f5fbd8d709e34633ba0f00a54301c18d640dc4c2d03da53f0a` | 133668 bytes; 2845 lines |
| Changed oracle | `ba7dde9c12eb14cb0a0615a49a013a2e126b860d` | `e61629a709e4154dda1297d3f5fa9fd13efc38c00d67c08edbc93fb70a27d32e` | 162305 bytes; 3386 lines |
| Oracle worker patch | `9e5bec68ba23fd3b625fbdebb4f266205830fa08` | `c152e248a240922ab319ab47eb23bea1c3fe7f7a7b7c1516f4aa647af4ab1149` | 32181 bytes; 590 lines |

Patch: `/workspace/scratch/7d27ea239237/media-settlement-oracle-worker.patch`. It is a full-index Git diff with repository-relative `a/source/host/tests/openclaw_sessions_and_config_contract.rs` and `b/...` paths, **+546 / -5**, one path. The change has one added media-proof/test region and four small integration hunks. No old assertion or negative-control body was removed.

Preserved earlier DB identities were rehashed, without reading or rewriting the earlier report:

| Existing DB handoff artifact | Preserved identity |
|---|---|
| AgentDb candidate | blob `6c27e9bb9f665b1160457b9acc29dd5d64fba289`; SHA-256 `9a8ad4aa952f11a3d7595eebf636f7d20c450381120416c00667b93ef0149c7b` |
| DB worker patch | SHA-256 `cc691a4377d744270632d4487195a970dafe7559b34c9132dd30fe16ca222906` |
| DB worker report | SHA-256 `76888e616bd40ddcd19f66e2930e631e719ab9a30b8791f8ec931c71cfda4c98` |

## 2. Authority and actual reading scope

Read the complete current MSU-v1 durable spec, including the new MSU-06 decision authorizing a separate media proof. Root AGENTS and main OCRM spec §17 were read from immutable 370 Git objects; current SOT authority/constraint sections were checked without treating its historical discovery log as current runtime evidence. No other worker review or root arbitration/publication narrative was used.

This was a focused oracle integration, not a claim to have deeply reviewed all 2845 preexisting target lines or all 10214 AgentDb candidate lines during this phase. The load-bearing reads comprise:

- Full newly added/edited oracle regions and their diff.
- Existing admission oracle 715–970 and its owner/cardinality contract.
- Existing commit-related checks and channel oracle/negative controls 1345–2178; the actual-source callers whose scope setup changed.
- Source-declared test names in the rest of the target, without executing discovery.
- AgentDb media command factory, entire media append owner, exact retirement helper, complete media command/authorizer methods, and adjacent ownership boundaries. Its unchanged inner admission/replay/receipt/CAS transaction had already been read during the DB phase.
- The bounded rusqlite/SQLite/Cargo source contracts recorded separately in `media-settlement-dependency-oracle-notes.md`; that report retains the archive/source and reading-depth distinctions.

## 3. What changed and why

The previous oracle assumed the channel adapter was the only argument-taking `.commit(...)` call and exempted only its exact method from the global literal prohibition. Adding the real media command adapter legitimately breaks both assumptions. This patch does not raise the global call count or remove the global prohibition.

### Independent media command proof

`media_commit_checked_method_source` binds the complete expected method to one actual, unconditional, nongeneric inherent `MediaSettlementCommands` implementation. It checks a unique named method, private visibility and no method-level attributes, removes only the exact method text, reparses, and requires that precisely that method disappeared from precisely that implementation. A copy in a comment, string, foreign owner or nested location cannot earn the exception.

The entire COMMIT method is pinned, including real COMMIT before the gated acknowledgement-loss observation. The entire ROLLBACK method is also proven, including the actual SQLite command, scoped authorizer drop and returned Result. The rollback proof is checked but its code is retained in the remaining source.

### Independent media consumer proof

`media_commit_owner_is_handled` binds one public, correctly typed `SandAgentDb::append_transcript_entries_and_release_media` method to a real AST owner. Conditional/unsupported file or method attributes, conditional/foreign/generic/duplicate inherent owners, missing methods and changed signatures fail closed.

The real method body must consist of:

1. The exact entry prefix: malformed input refusal, poisoned-lock retirement/unknown, closed-owner refusal, preexisting-transaction retirement/unknown, command claim and clean-versus-unsettled BEGIN handling.
2. One directly invoked synchronous typed write closure, retaining exactly three admission-identity guard calls through the existing admission recognizer.
3. The exact final returned match: every COMMIT error retains unknown classification after any cleanup; live-after-COMMIT and unconfirmed rollback retire before returning unknown; confirmed success drops the connection guard before notifications; ordinary precommit error returns only after the pinned rollback/autocommit check.

The returned tail must actually be the final expression. A matching comment, nested fixture, discarded match, added terminal expression, conditional tail, altered closure entry, omitted rollback/retirement or changed error variant does not pass.

This checks the control boundary without copying/removing the long admission/replay/receipt/CAS closure. Its source remains visible to the global COMMIT check.

### Composition preserves the original channel proof

After the media proof, `source_without_owned_media_commit_adapter` removes only the exact media COMMIT method and replaces only the single proven `commands.commit(db)` expression with a non-call marker in an oracle-local syntax image. This image is never compiled product source. Every other method, owner, call, macro input and raw COMMIT remains.

`source_without_owned_transaction_commit_adapters` then invokes the unchanged channel checker on that image. Consequently its original **Some(1)** rule still binds the one remaining argument-taking call to its exact channel owner. An extra media caller, alias, method reference, hidden/macro call or extra channel caller remains visible and invalidates the composed result.

The main actual-source gate uses the composed proof, followed by the unchanged assertion that the remaining source contains no `db.execute_batch("COMMIT")?;` statement. The three old actual-source channel tests first obtain the separately proven media-normalized image and then run their old assertions and mutation cases unchanged. In particular, the original actual-source direct-string **count == 1** is retained on the appropriate checked scope.

### Byte-preserved original mechanisms

Static source-slice identity comparisons, not parser/test execution:

| Preserved block | Bytes | SHA-256 |
|---|---:|---|
| Admission owner/cardinality/AST recognizer block | 13409 | `5c25ecf71de084ae99b260b3f4ab37a1ef21d12388e43f65de65e9370624d395` |
| Complete channel constants, implementation recognizer, macro/token counter and transaction-owner proof | 10661 | `daece98360159768b70e9089a009fac1d93b0e5aefbc786a0477ba3fed2ad9db` |
| `source_without_owned_channel_commit_adapter` | 1910 | `691ae771badec43d0a2445f620dea42f1a0765813cd9fd12c262449314d17d3e` |

The old complete channel method/tail pins, all old macro/alias/caller/hidden-owner negative cases, all old ordinary-RAII positives and all old raw-COMMIT assertions remain. No synonym rename in production evades the oracle.

## 4. New causal source-oracle tests

All eight names deliberately contain `media_commit_oracle_` for root's existing-target native selection. These are source mutation oracles and are separate from the real SQLite fault tests already added to AgentDb.

| New declared test | Required boundary / counterfactuals |
|---|---|
| `media_commit_oracle_accepts_actual_owner_and_preserves_admission_guards` | Actual source passes each bounded proof; all eight existing admission-owner requirements still apply; media guard count remains three; channel method and rollback method remain in the intermediate scope; original zero-argument/unary-negation RAII positives remain supported |
| `media_commit_oracle_rejects_missing_duplicate_or_conditional_helper` | Missing/foreign/qualified/duplicate helper; duplicate method; exact method only in comment; cfg-hidden impl/commit/rollback/file; malformed source |
| `media_commit_oracle_rejects_missing_duplicate_or_conditional_owner` | Wrong owner name; duplicate owner impl or method; cfg-hidden impl/method; wrong visibility/typed result; qualified foreign owner; matching tail only in a comment |
| `media_commit_oracle_rejects_changed_commit_or_rollback_commands` | Fake or ignored COMMIT; altered acknowledgement result; extra transaction; fake rollback; leaked authorizer guard; discarded real rollback error |
| `media_commit_oracle_rejects_extra_aliased_or_macro_callers` | Additional direct/renamed/raw-identifier receiver; associated function and alias; unresolved method-substituting macro definition; imported/qualified/nested/raw-identifier macro invocation; opaque zero-argument-looking forwarding; sibling helper caller |
| `media_commit_oracle_rejects_discarded_or_changed_settlement_handling` | Discarded returned match; cfg-hidden tail; wrong result arm; unknown demotion; missing rollback or retirement; weakened rollback/autocommit test; missing guard release; fake success; altered direct write-closure entry |
| `media_commit_oracle_rejects_changed_unknown_entry_handling` | Poisoned/prior/unsettled-BEGIN demotion; omitted retirement/rollback; bypassed autocommit distinction; cfg-hidden lock statement |
| `media_commit_oracle_preserves_raw_commit_prohibition_outside_proven_method` | Extra raw COMMIT before/after owner, as helper sibling, and inside the media write closure remains in the returned source and is rejected by the unchanged global prohibition |

Source inspection found that the same write-closure declaration also occurs in the older channel owner. The new closure mutations and inner-write raw-COMMIT control are therefore anchored to the **complete media entry prefix plus declaration**, not a whole-file first occurrence. This makes their claimed owner falsifiable rather than accidentally mutating the adjacent transaction.

No new dependency, runner or standalone parser infrastructure was added. Existing syn parsing/full features and existing helpers are reused.

## 5. Exact source-declared commit filter set

The requested command is for execution only in the authorized existing Actions capture:

```text
cargo test --locked --manifest-path source/host/Cargo.toml --test openclaw_sessions_and_config_contract commit -- --test-threads=1
```

**27 source-declared `#[test]` functions contain `commit`: 19 retained + 8 new.** This enumeration came from source text inspection, not Cargo/Rust test discovery or execution:

```text
config_set_racing_one_owner_hash_has_one_commit_and_one_conflict
test_cron_owner_writes_rollback_when_commit_fails
test_archive_owner_rolls_back_when_commit_fails
test_background_result_owner_rolls_back_when_commit_fails
test_session_transcript_append_rolls_back_when_commit_fails
test_session_transcript_update_and_identity_settlement_roll_back_when_commit_fails
test_session_transcript_delete_and_conversation_clear_roll_back_when_commit_fails
test_session_checkpoint_and_root_metadata_roll_back_when_commit_fails
test_session_metadata_owners_roll_back_when_commit_fails
test_transcript_owners_roll_back_when_commit_fails
media_commit_oracle_accepts_actual_owner_and_preserves_admission_guards
media_commit_oracle_rejects_missing_duplicate_or_conditional_helper
media_commit_oracle_rejects_missing_duplicate_or_conditional_owner
media_commit_oracle_rejects_changed_commit_or_rollback_commands
media_commit_oracle_rejects_extra_aliased_or_macro_callers
media_commit_oracle_rejects_discarded_or_changed_settlement_handling
media_commit_oracle_rejects_changed_unknown_entry_handling
media_commit_oracle_preserves_raw_commit_prohibition_outside_proven_method
test_agent_db_has_no_direct_commit_error_propagation
test_commit_error_oracle_rejects_unguarded_propagation_and_missing_cleanup
channel_commit_oracle_retains_unsafe_owners_before_after_and_inside_helper_impl
channel_commit_oracle_rejects_extra_unhandled_callers_and_method_references
channel_commit_oracle_rejects_changed_missing_or_discarded_error_handling
channel_commit_oracle_rejects_missing_ambiguous_or_malformed_helper_boundaries
channel_commit_oracle_rejects_hidden_owner_and_aliased_calls_in_actual_source
test_conversation_blob_cleanup_rolls_back_when_commit_fails
test_compaction_owner_rolls_back_when_commit_fails
```

Root can strictly enumerate/count this set while retaining the existing commit/error/rollback capture controls. A parser panic, fixture syntax failure, missing declared test, compile failure, wrong count or any negative case accepted remains non-passing evidence.

## 6. Requirement mapping and remaining verification

| Requirement | Implementation evidence | Execution state |
|---|---|---|
| MSU-06 independently bounded media adapter/consumer | Exact methods; strict AST owners/signature; prefix/closure/tail proof; eight new tests | Pending exact-H Actions |
| Preserve existing channel invariants and negative controls | Unchanged core/helper hashes; old test assertions retained with media-proof scope setup | Pending exact-H Actions |
| Preserve three media admission guards | Existing algorithm/cardinality block unchanged; new actual-source positive calls it; direct closure proof also requires three | Pending exact-H Actions |
| MSU-01 unknown/rollback/retirement contract | Handler prefix and final returned tail tied to actual method; negative demotion/cleanup controls | Source proof only; real DB tests pending |
| MSU-08 real command ordering/scoped cleanup | Complete command methods pinned; fake command/ignored error/leaked guard controls | Source proof only; real SQLite tests pending |
| Global raw COMMIT prohibition | Original final assertion retained; extra raw owners preserved across normalization | Pending exact-H Actions |
| Shipping default-off feature support | Root owns manifests and capture; root chose existing cargo check/test-build compiler-artifact.features observations with exact Host manifest/src/kind and profile.test=false | No worker command added or run |

Root's selected compiler-artifact feature observation supersedes the optional cargo-tree suggestion in the earlier dependency note for this integration. It uses the existing shipping check and App test build, rather than introducing an additional build or graph command. The worker made no manifest/capture changes.

This recognizer intentionally accepts only the declared syntax boundary and fails closed outside it. It does not expand macros, evaluate cfg, resolve Rust types, prove arbitrary control-flow equivalence, establish native SQLite behavior by itself, or claim full migration acceptance. Root's independent review and exact published-H native plus Windows/Linux App evidence are still required.

