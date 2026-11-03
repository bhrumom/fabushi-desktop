# DFA R2 independent general / security / lifetime source review

Verdict: **APPROVE SOURCE — no blocking or nonblocking defect finding established in the declared scope.** This is an independent source review, not executable verification, unit `verified`, or release acceptance.

Task `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, review round 2. Reviewer: `/root/device_bearer_security_r2`. Selected model/effort: inherited configured Astra / `xhigh`, as declared by the parent launch, for fresh paired-credential currentness, ordinary HTTP mutation, lifetime and shared CI integration risk. No child model/effort override or replacement worker was used. The child interface exposes no independent runtime model/effort introspection, so this records the inherited launch setting rather than claiming a separate measurement.

The review was fresh and blind: no peer review, prior review report, runner decision or implementation assessment report was read. Source was read from immutable Git objects; no repository working-tree file was edited. No build, test, lint, formatter, generator, schema check, harness, local/htch execution, or GitHub write was performed. Git/text/hash reads and creation of this report were the only actions.

## Exact subject and identity checks

- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`, independently confirmed with Git.
- Frozen base / starting HEAD: `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6`.
- Previous immutable review tree: `9cef1a8dc09c684ad09594fcd85b82f5f8514d40`.
- Reviewed integrated tree: `b415b8a429b3e0ffe7a2f2ba0e86b2c67f62eab0`.
- Publication parent: `74f11d3d31c6245ac5886dc0181460b2d4349744`, tree `aca91e1cfbedeecb039d442fb3528caac89873a3`.
- Fixed source identity, used as the declared contract authority rather than newly audited upstream closure: U `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Complete 12-path patch: 87,863 bytes, SHA-256 `94945a6f24bb751dcb748be1c1688a65a65ddfdb285ba678b4366c535301053d`. Independently regenerated with Git from the frozen base and reviewed tree over the 12 paths; hash matches.
- Integration delta: 8,064 bytes, SHA-256 `aeb6909eefc21fd6c6cbbca1d58a2f2e87cee68acc68570c6b2e0078967c498b`. Independently regenerated from the previous and current trees over the same scope; hash matches.
- Publication patch SHA-256: `54ecab7dc36f6a5c48f3c88ed1d008c0e38f68e1b54de0f8e85b23e3ca492b5f`; incoming-main patch SHA-256: `be22ba5017dae1cdc86b95ff1118b15682963981ba8fbf2bd595cacd345329f5`. Supplied patch file hashes match the manifest.
- Candidate and publication-parent App main both resolve to blob `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df`. The candidate local-media test and publication-parent local-media test both resolve to `57a0a3393f1bb9560eb036687688656b1f3fe6c8`; that prerequisite has no owned publication delta. Git confirms exactly the declared 11 publication paths.

All 12 candidate identities were checked with `git ls-tree` against the manifest:

| Scoped path | Candidate blob |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `ede3b6b9586383a2d0245de933d6ff4417e6ff10` |
| `projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md` | `f0d8a608d1556ff2dd71bc9829df478af3c01116` |
| `projects/openclaw-rust/parity-ledger.json` | `2914b1a9aa1e07dac123af7fbcb10f88b1dee93c` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `58a6a94c9cb19b67199b628561d4345a26656410` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `f5f3d5624568cb553441b8ca0c0f63aff76ebeae` |
| `source/host/app/src/tests/config_secret_boundary_tests.rs` | `04f7b0da6600c60b6540008fbd4f542007bdcf34` |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `57a0a3393f1bb9560eb036687688656b1f3fe6c8` |
| `source/host/src/device_pairing_owner.rs` | `6645d073450ce3fab5b726a4579cccc41cabaad5` |
| `source/host/tests/device_pairing_revocation_authority_contract.rs` | `65f57e3c458d0ea74099c20d8b284af6bbd40c0d` |
| `source/host/tests/device_pairing_rotation_authority_contract.rs` | `a06e0593cf3be3673e91706e1636399dab90d0a8` |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | `7c37e8093b8b7f2975eb031d76dae1b1419ed13c` |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | `e7c42f88067d8d4695ff7ee356987e493b1f4224` |

## Review coverage and reasoning

**Fresh authorization and error semantics.** `device_pairing_owner.rs:610–682` constructs the same sealed candidate from an active constant-time bearer match and invokes the private connection-taking validator before returning it. The validator at `1558–1591` is also the retained validator's implementation. Fresh lookup holds its existing connection mutex and passes the connection directly; it does not recursively acquire the mutex. The exact typed `Unauthorized` arm alone becomes `Ok(None)`. Other errors propagate. No write, new connection, cache, state store, proof constructor or downstream-policy shortcut is added.

The adjacent proof and policy implementation was read: owner allocation identity, sealed device/role/fingerprint/admitted scopes, fixed operator implications, ECMAScript scope canonicalization, strict live-scope parsing and SQL-NULL-only approved-baseline fallback. `current_pairing_record` still checks the original owner and selected credential, approved role-array membership and marker before grant checks. The complete current live grant must fit approval; the original admitted grant must fit both current live and approved grants. The permissive preliminary public-scope projection cannot turn malformed decisive scopes into a successful fresh proof because the shared validator strictly rereads them. The unchanged bound management owner continues to perform its own transaction admission; the fresh check is not presented as cancellation of an already-entered Settings operation.

**Shipping HTTP/configuration chain.** The actual candidate `main.rs` was read at the Settings dispatcher (`1782`), OpenClaw/config delegates (`4436`, `4477`, `4501`), Gateway implementation (`13462` onward), durable pairing construction (`15979`), App composition (`17528`) and Gateway startup (`17824`). Authentication delegates to the same `device_pairing`; `config.set` reaches the same `settings` through `dispatch_public_settings_command`. The actual Settings public commit entry and writer path in `settings_service.rs:1360` onward were also read. The incoming cron extraction does not replace or duplicate these production anchors.

The source-binding predicate and each `replacen(..., 1)` negative control in `config_secret_boundary_tests.rs:773–816` were compared with the production source before its `#[cfg(test)]` module. The relevant occurrence for each tested binding is the shipping occurrence, including the composition's specifically indented Settings field. Incoming cron helpers add no earlier counterfeit occurrence or alternative auth/config path. These remain source evidence; the fixture does not instantiate the full `UnifiedGatewayApi`.

**Causal runtime oracles, as authored source.** `config_secret_boundary_tests.rs:481–770` uses the real pairing owner, real TCP Gateway and real App Settings dispatcher/owner. Known bearer bytes are retained across baseline/role/grant corruption. Refused requests compare lookup/admission/binding/dispatch/commit/report counters, independent complete pairing/pending/machine-state rows, actual Settings bytes and directory entries. A forged admin ceiling cannot restore refused authority. The positive uses the same consumer and verifies the setting after constructing a fresh Settings owner. Both configured-native-secret and credential-absent loopback controls remain. Malformed decisive storage has an exact fixed 500 assertion and secret-free response checks. HTTP EOF and bounded reads establish response settlement; a timeout is a failure, not a passing refusal.

**Lifetime and teardown.** The HTTP fixture's `stop`, API `Drop` and fixture `Drop` (`549`, `611`, `710`) wait for destruction of the API's final Arc and explicitly drop its pairing connection before acknowledging teardown. `GatewayServer::close` only joins the accept thread, so this additional acknowledgement is material. Reopen cannot proceed without it. A teardown timeout preserves the owned Settings fixture instead of deleting a directory still reachable by a handler. No new product cancellation owner is added. Existing event/bridge validators and wake/subscription/response boundaries were read in `gateway_server.rs:1369–1469` and `1581–1662`; they continue using the retained public validator and preserve the admitted-frame, rather than write-completion, lifetime contract.

**Compatibility tests preserve their deciding conditions.** The changed DPA/DVR/DTR functions and their adjacent fixture/proof/state helpers were read. Late-admission negatives acquire genuine valid same-owner proofs before the precise withdrawing/corrupting transition. Reopen negatives now use decisive fresh refusal rather than an old-owner proof masking the intended predicate. The DPA empty-baseline case identifies exactly the known unselected target/node credential, asserts its fresh refusal and prior-proof retirement, and requires exactly one such exception. All other callers of the ordinary rotation oracle still require unselected credentials to authenticate. Existing complete-row, revision, rollback, successor, unselected-role and post-reopen controls remain. The six owner/test production-facing paths are byte-identical to the previous review tree, confirmed by an empty Git diff over those paths.

**Shared CI integration and reporting.** The full scoped workflow/capture changes and surrounding selection/receipt consumers were read. The three new owner case names remain required in both pairing receipt paths; the two DFA App cases remain required in the macOS selection and Linux actual-App selection. The portable and paired-target receipt checks still require zero command/capture errors, each required name exactly once, exact complete target counts, zero ignored/measured/filtered results and the existing prior targets. The Linux App path checks the shipping build and actual test artifact, enumerates each exact selected name, runs it and requires one-pass/zero-failed/zero-ignored summaries.

The integration delta preserves the incoming cron spec/test/stable-ID source inputs and all five exact cron App cases, alongside DFA and local-media cases. It only adds cron references to the ledger/RTM unions. Both the cron and DFA receipt descriptions retain the distinction between actual helpers, supplementary shipping source bindings and full App/device/platform acceptance. The local-media prerequisite is the exact incoming import/f64 repair; its surrounding fixture call agrees with the existing `set_window_focused(..., now_ms: f64)` signature. No media assertion was removed. The complete diff changes no acceptance status, unit verification field, literal historical read fact or global completion flag; the RTM's eight completion flags remain false.

## Read limits and acceptance boundary

Read authority included root `AGENTS.md`, the OCRM main specification's current §17 and relevant SOT/module-map context, the complete DFA-v1 specification, and the applicable DPA/DVR/DRT/DTR currentness, scope, mutation, retained-lifetime and evidence contracts. Settings CS-01–CS-09 and LFA admission/custody/prerequisite constraints were read, along with the incoming cron identity and shared verification contract. No nested `AGENTS.md` governs these scoped paths.

Every hunk of the complete 12-path patch and integration delta was inspected. Product methods, modified test functions and deciding helpers were read in their immutable source context. Large unchanged test files, the entire 454 KB workflow, full 1.4 MB ledger, unrelated App services and all fixed-U dependency implementations were not claimed as fully reread. Adjacent retained/management/App test entrypoints were searched and the pertinent retained successor/currentness and production ingress/cleanup paths were read. Incoming cron implementation was read as adjacent composition context; this report is not a separate full acceptance of that batch or of the other incoming paths.

There is no falsifiable defect to supply a severity/scenario/fix for. This verdict permits progression to the already-required exact-published-H GitHub Actions checks. Those results, scoped G01/G02 authority/responsibility evidence, remote/provider/device/platform behavior and independent final acceptance remain separate and unproven by this source review.
