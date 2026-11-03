# MSU-v1 dependency and source-oracle notes

## Result and limits

**The pinned-source contracts support the implemented real-COMMIT/lost-acknowledgement and real-ROLLBACK/authorizer-denial design. The unchanged pre-MSU transaction oracle has a concrete integration failure that must be adapted without weakening its channel proof or its remaining-source COMMIT prohibition.** This report records source reasoning, not a build, parser, test, feature-resolution, or acceptance result. No such executable validation was run locally or through htch. Exact published-H Actions evidence remains pending.

The delivered DB candidate is unchanged by this task: `source/host/src/extensions/session/agent_db.rs` Git blob `6c27e9bb9f665b1160457b9acc29dd5d64fba289`, SHA-256 `9a8ad4aa952f11a3d7595eebf636f7d20c450381120416c00667b93ef0149c7b`. Its earlier patch/report remain immutable handoff history. This report preceded the separately authorized oracle implementation; that implementation and its tests must be assessed from its own patch/report.

## 1. Target and dependency identities

Target base is `fdb0f6acb06165b6f2796961186e06a2de98c229`, tree `abeb6ffdf09b676279462fea3b9e1f6d72abc1b6`. Root's current frozen H is `a2a0c9ef326c1ddeb09b4f9180267fdaeb21db74`, tree `a0dceefeae75fccd7ab5aec143f6d651f7cf2fd3`. I independently read fdb immutable Git objects and fetched the a2 locks/oracle through the exact-ref GitHub connector. Those three a2 blobs match fdb. I did not inspect concurrent evidence/review files.

| Target source | Exact Git blob | Read scope |
|---|---|---|
| Host Cargo.lock | `9895ce7752768b1c8d3fe107ef95e513457ca518` | rusqlite 2915–2927; libsqlite3-sys 1716–1724; relevant-name navigation |
| App Cargo.lock | `5f929af729ac7b537e054283c5977cc269be38ad` | rusqlite 4282–4294; libsqlite3-sys 2312–2320; relevant-name navigation |
| Host Cargo.toml at fdb | `9b477d438dd0a9325cbe52b670c2ada774f352e8` | complete manifest |
| App Cargo.toml at fdb | `4cd51b5e35192f3f8ad1e6dff3699ec9a63e8928` | complete manifest |
| openclaw_sessions_and_config_contract.rs | `2049875451202ba05aecb7e5de9a6841dc378d63` | 715–970; 1365–1635; 1660–1875; targeted navigation elsewhere |

Both locks specify:

| Dependency | Version | Registry archive checksum in both locks | Declared license |
|---|---|---|---|
| rusqlite | 0.39.0 | `a0d2b0146dd9661bf67bb107c0bb2a55064d556eeb3fc314151b957f313bcd4e` | MIT |
| libsqlite3-sys | 0.37.0 | `b1f111c8c41e7c61a49cd34e44c7619462967221a6443b0ec299e0ac30cfb9b1` | MIT |

Host requests rusqlite `version = "0.39", features = ["bundled"]`. Rusqlite's manifest selects libsqlite3-sys 0.37.0 for the native targets at issue, and its bundled feature enables libsqlite3-sys/bundled plus modern_sqlite. This is a Windows/Linux native-source check, not a wasm backend check.

The official rusqlite repository tag v0.39.0 resolves to immutable commit **`2a1790a69107cd03dae85d501dcbdb11c5b32ef3`**. Its manifests state both versions above. The bundled header identifies SQLite **3.51.3**, version number 3051003, source ID **`2026-03-13 10:38:09 737ae4a34738ffa0c3ff7f9bb18df914dd1cad163f28fd6b6e114a344fe6d618`**. The official SQLite mirror tag version-3.51.3 resolves to commit **`a5333afb9ad1aa473f8963b92caeaa955f47dc74`**; its `manifest.uuid` equals that exact Fossil source ID. This gives an explicit bridge from the bundled release header to the narrowly read SQLite C implementation.

**Archive distinction:** no matching local rusqlite/libsqlite3-sys crate archives were available in the narrowly searched caches. The two checksum values above are verified lock-file values, not independently recomputed archive hashes. docs.rs exact-version source requests were unavailable. I acquired the official tagged Git source and verified every ordinary captured file against its returned Git blob; I do not claim a downloaded crate archive was compared byte-for-byte to that tag.

### Acquired source identity and reading depth

Source captures and their complete SHA-256 inventory are under `/workspace/scratch/7d27ea239237/media-settlement-dependencies`, in `source-fetch-manifest.json`. Full-file acquisition does not imply full-file reading.

| Official immutable source, relative to repository root | Git blob | Actual semantic read |
|---|---|---|
| rusqlite Cargo.toml | `a1af9ff97a869dee75ff23f48ab42f792e963b9a` | full |
| rusqlite src/lib.rs | `ef599d338bbc516a86c2210ebec5fd1d45ca8fdf` | 349–421, 531–603, 795–867, 1042–1114 |
| rusqlite src/inner_connection.rs | `0751cfc0375b0c57c343f6eb81a21e1fe6358e40` | 9–81, 98–221, 293–365, 376–391, 396–447 |
| rusqlite src/raw_statement.rs | `73ee67e4e3304a18bd8eaca1080c4e5600851d7a` | 136–167, 219–249, 271–283 |
| rusqlite LICENSE | `08c660f5be514958a1179ccadf63a1f488807aa1` | full |
| libsqlite3-sys Cargo.toml | `0c5c19c5985edfd4f5a1b8ac29b6f64dc7d8ef5d` | full |
| libsqlite3-sys LICENSE symlink | `ea5b60640b01f74e295037aa8a6b7d4ea278a739` | target text `../LICENSE`; connector resolves it to the full MIT license above |
| libsqlite3-sys build.rs | `845fd3d3c23becb1ae586c69616412b1c14495b1` | 1–90, 96–146, 249–321; relevant-symbol navigation |
| libsqlite3-sys sqlite3/sqlite3.h | `302f0f5b1698d1624216abbb7c2870c5981a581a` | 1–160, 298–360, 3342–3456, 3470–3508, 6824–6856 |
| SQLite manifest.uuid | `bae34afb92144a2b9d46b89971a914dad5a4d229` | full one-line identity |
| SQLite src/auth.c | `9ec2e7d046220dfc1ca2ae428395e308181d5076` | 1–100 |
| SQLite src/main.c | `6efe538d4d9621818e41e996b622d131d3e216f1` | 1220–1411, 3900–3922 |

The sys LICENSE capture contains the resolved MIT file, whose byte blob is `08c660...`; it is deliberately not mislabeled as the symlink blob. SQLite's header and auth.c carry its copyright disclaimer; rusqlite and its sys wrapper carry the MIT notice. No new runtime dependency or copying of their implementation into production is proposed by this report. This is the relevant attribution/license binding, not a complete dependency-license audit.

Primary exact-source links: [rusqlite commit](https://github.com/rusqlite/rusqlite/tree/2a1790a69107cd03dae85d501dcbdb11c5b32ef3), [SQLite commit](https://github.com/sqlite/sqlite/tree/a5333afb9ad1aa473f8963b92caeaa955f47dc74), [SQLite source identity](https://github.com/sqlite/sqlite/blob/a5333afb9ad1aa473f8963b92caeaa955f47dc74/manifest.uuid).

## 2. Contracts on which the DB implementation depends

### execute_batch and actual COMMIT/ROLLBACK

Rusqlite `Connection::execute_batch` (lib.rs 539–558) prepares each input statement, calls its step operation, and propagates prepare/step errors. It does not wrap a failed COMMIT in an automatic success, add a compensating COMMIT, or convert it into an application outcome. RawStatement's ordinary step calls `sqlite3_step`; its Drop finalizes the statement.

Thus `MediaSettlementCommands::commit` executes an actual SQLite COMMIT first. Only after its success can the gated lost-acknowledgement branch return its injected error. A real COMMIT failure exits at the preceding `?`; the fault observation cannot cause a real failure to be interpreted as successfully applied lost ACK. The production media owner classifies every returned commit error as unknown and retires, even if later cleanup succeeds. This is stronger conservative application policy than merely checking the current autocommit bit.

SQLite's official transaction documentation says some statement failures preserve a transaction and some can end it; its deferred-FK example explicitly shows COMMIT failure with the transaction still open. These facts support a real file-backed FK failure before COMMIT completion and distinct clean/refused rollback cases. They do not authorize an automatic retry in this product. [SQLite transaction errors](https://sqlite.org/lang_transaction.html), [deferred foreign-key constraints](https://sqlite.org/foreignkeys.html#fk_deferred).

### Authorizer registration, actual denial, and userdata lifetime

The pinned header 3344–3436 specifies connection-local, compilation-time authorization; SQLITE_DENY rejects the statement, NULL removes the authorizer, and a new registration replaces any previous one. The pinned C implementation auth.c 70–84 performs the callback/userdata assignment under SQLite's connection mutex and returns SQLITE_OK for a valid handle. Under the bundled API-armor option an invalid handle can return SQLITE_MISUSE before assignment.

The fault helper retains an Arc allocation and borrows the live connection for the whole `execute_batch("ROLLBACK")` call. It checks the operation string pointer for NULL, changes only its atomic observation, and denies only the transaction/ROLLBACK action. All other operations return SQLITE_OK. It does not reenter SQLite from the callback. A prepare denial therefore becomes an actual SQLite error; the test observation is not the production outcome enum.

The explicit guard drop occurs before the caller can retire its connection. For this valid, still-borrowed handle, pinned auth.c shows NULL registration clears callback and userdata synchronously before the Arc field can drop. The uninstall return value is not a general guarantee for invalid handles, but the ownership/lock boundary here supplies the valid-handle precondition. The new hook is not composable with an independent preexisting authorizer: SQLite has one slot and this guard clears it rather than restoring a prior callback. The private DB owner and scoped test-only installation are therefore a necessary precondition; no parallel authorizer owner is introduced. Existing other authorizer references in this AgentDb file are private test code.

### Autocommit is an observation, not a durable receipt

Connection::is_autocommit delegates through InnerConnection to `sqlite3_get_autocommit(ptr) != 0`. The pinned C implementation returns `db->autoCommit`; the header states BEGIN disables it and COMMIT/ROLLBACK reenable it, and directs callers to inspect it after errors that can automatically roll back. The header explicitly excludes racing changes by another thread. The AgentDb connection mutex supplies the exclusive observation window.

A true bit after an error cannot distinguish “COMMIT succeeded but its acknowledgement was lost” from “rollback completed.” Accordingly, the implemented owner never downgrades a COMMIT error. It permits ordinary precommit rejection only on the known-clean branch; its present code conservatively escalates a failed explicit rollback even if another mechanism had already restored autocommit. This conservatism preserves custody.

### Connection retirement, close, and Drop

The existing unchanged retirement helper sets the owner's closed flag, takes/drops its exact connection Option, and releases that owner's live-handle registration. It does not open a replacement connection. Rusqlite Connection::drop flushes the statement cache; InnerConnection::drop calls close and ignores its Result. That close uses **sqlite3_close**, not sqlite3_close_v2.

Pinned SQLite main.c 1246–1302 and header 319–357 distinguish this sharply: sqlite3_close returns SQLITE_BUSY and leaves a connection open if statements/backups remain. When it can destroy the connection, main.c 1355–1381 calls rollback-all before freeing connection resources. The application's closure-local statements have dropped before its outer error handling; execute_batch statements also have lexical Drop/finalization. This supports the designed ordinary file-backed tests, including denied explicit rollback followed by retirement.

However, **owner retirement is the unconditional product guarantee; successful native deallocation/rollback is not an unconditional consequence of ignoring Drop errors.** A leaked/foreign raw statement would defeat such a stronger claim. The implementation appropriately returns unknown and retains media regardless; it does not use Drop as a success publication or known-refusal certificate. This is a bounded dependency caveat, not evidence that such a leaked handle exists in the reviewed path.

### Build/source preconditions

The bundled build path compiles sqlite3.c with API armor, foreign keys, and threading enabled; no SQLITE_OMIT_AUTHORIZATION flag appears in the examined ordinary configuration. The build script also permits `LIBSQLITE3_SYS_USE_PKG_CONFIG` to select an external library and `LIBSQLITE3_FLAGS` to add compile options. Therefore the lock/version alone does not prove a runner ignored such overrides. Exact-H Actions must preserve the intended bundled native configuration; I did not execute a build or inspect the complete native toolchain/cc dependency closure.

## 3. Existing AST oracle: concrete compatibility gap

The unchanged oracle is bound to blob `2049875451202ba05aecb7e5de9a6841dc378d63`.

1. `channel_commit_call_count` recursively counts any nonempty-argument `.commit(...)`, independent of receiver. `agent_db_transaction_owner_source` requires its result to be exactly Some(1). The new `commands.commit(db)` is a second such call, alongside `control.commit(db)`. Consequently the source extraction returns None before the old actual-source test can pass.
2. The subsequent whole-remaining-source prohibition rejects the literal `db.execute_batch("COMMIT")?;`. It currently removes only the pinned `ChannelInboundTransactionControl::commit` method. The new `MediaSettlementCommands::commit` contains another occurrence and is not covered by that exception.

These are deterministic source-level incompatibilities, not observed test failures. The original channel method and its unique consumer remain byte unchanged in the delivered DB candidate.

The acceptable adaptation is to independently bind the exact media adapter method and one correctly handled media-owner consumer to real AST ownership, reject duplicate/hidden/aliased/macro/unhandled calls and altered rollback/unknown/retirement behavior, and normalize only those proven regions before the unchanged channel proof and remaining-source prohibition. Increasing Some(1) to Some(2), deleting the raw prohibition, or renaming a call to evade the counter would lose the original invariant.

The media admission-guard oracle is separate. It supports the match/if/let-else/try/directly invoked closure syntax used by the new owner, and method calls traverse their receiver/arguments without requiring helper bodies to have the same cfg rules. The new owner has no cfg inside its body; its preexisting inner transaction closure and three `session_admission_identity_is_well_formed(&admission)` checks are unchanged. Source inspection therefore identifies no new admission-guard compatibility gap. This is not a claimed parser run.

## 4. Default-off test support and the shipping feature observation

The exact fdb manifests above use edition 2024. They predate the new test-support declaration/activation. Root owns those new manifest changes. Required candidate shape: Host has a declared `test-support = []` feature that is absent from defaults; App's normal Host dependency does not request it; App's dev-dependency on that same Host path requests it for integration tests. The DB hook state/API is gated by `any(test, feature = "test-support")`.

Cargo's official resolver documentation separates dev-dependency feature activation from ordinary builds under the modern resolver, and states edition 2024's default is resolver 3. The controlling resolver is that of the top-level package/workspace, so any future workspace override must also be considered. [Cargo resolver](https://doc.rust-lang.org/cargo/reference/resolver.html#feature-resolver-version-2).

Keep the existing ordinary shipping command already present in the workflow:

```text
cargo check --locked --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host
```

Do not add a duplicate check. A supported, nonbuilding observation of its ordinary dependency feature graph is:

```text
cargo tree --locked --manifest-path source/host/app/Cargo.toml --edges normal,build,features --invert mahayana-host-runtime --format "{p} features=[{f}]" --prefix none
```

The normal/build selection excludes dev edges; specifying features alongside those edges avoids the features-only default that includes other kinds. The selected Host's feature list must exclude `test-support`. Preserve the exact command, exit status, and complete relevant output in the existing exact-H Actions evidence. An optional dev-enabled observation uses `--edges normal,build,dev,features` with the same remaining arguments and should show the dev activation. Neither command was run here.

Cargo documents that tree approximates the feature-unified build graph and does not guarantee exact compilation equivalence. Therefore pair this observation with the existing ordinary shipping check and source-bound default/dev declarations; do not call a tree alone a compiled shipping-feature proof. Do not use `--no-default-features`, `--all-features`, or a shipping `--features test-support` command, which would answer a different question. [Cargo tree command and feature formatting](https://doc.rust-lang.org/cargo/commands/cargo-tree.html).

## 5. Remaining scope and evidence

This is the bounded direct dependency/ownership contract check needed by the MSU owner and its fault mechanism. It is not a full rusqlite/SQLite source review, archive verification, dependency closure, licensing audit, current whole-product review, or migration acceptance. Full raw source captures exceed the explicitly read ranges. The previous worker patch/report, other reviewers' reports, incoming decisions, and unrelated publication/evidence content were not edited or used as authority.

Remaining concrete evidence is exact published-H Windows/Linux Actions compilation, existing shipping check plus feature observation, preserved channel/admission source oracles with the new media proof and negative controls, real file-backed DB fault tests, and the separately owned App/custody integration tests. Their execution and resulting acceptance remain with root and the authorized Actions workflow.

