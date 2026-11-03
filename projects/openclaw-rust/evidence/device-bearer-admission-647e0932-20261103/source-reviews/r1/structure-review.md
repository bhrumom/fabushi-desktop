# DFA-v1 round 1 — independent structure, reuse and performance review

**Verdict: APPROVE for the reviewed source scope. No actionable findings.** This is not a build result, executed behavior verdict, unit-verification promotion, shipping verdict or global acceptance.

## Reviewer settings and isolation

- Assigned lenses: structure, reuse and performance.
- Parent-applied worker settings: `gpt-6.1-sol`, `xhigh`, fresh `fork_turns=none`, as supplied in this worker's launch instruction. I did not change a model/effort setting or launch another worker. The high-risk reason is the interaction between permission currentness, one shared SQLite mutex, causal fixture lifetime and exact-subject CI closure.
- I read immutable Git objects through `git show`/`git grep`/`git diff`/`git ls-tree`, plus the supplied frozen manifest and required-case list. I did not read peer reports, root arbitration/implementation reports or mutable product-file contents. A later integration-status message from the parent did not change this review subject or verdict.
- No source edit, Git mutation, dispatch, build, test, formatter, linter, generator, schema check, harness, benchmark, local validation or htch execution was performed. The only write is this report.

## Exact subject and complete scope

- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Exact base and starting head: `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6`.
- Immutable candidate tree: `9cef1a8dc09c684ad09594fcd85b82f5f8514d40`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`; upstream tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Frozen patch: `/workspace/scratch/29e7135d8bfe/reviews/device-bearer-admission-r1/complete.patch`, directly observed as **82,558 bytes**, SHA-256 **`4fc6eb682286d13b96f478b028f1d529637cca1b1cf68aba383c1526bad799cd`**.
- The exact base-to-tree name/status diff has the same 12 paths as the manifest: one added spec and 11 modified files. I read every changed hunk in that scope. The preceding plugin CI repair is part of the immutable base, not a new DFA finding or newly approved implementation.

The candidate blob identities below were read back with `git ls-tree` and agree with the frozen manifest.

| Path | Candidate blob |
| --- | --- |
| `projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md` | `f0d8a608d1556ff2dd71bc9829df478af3c01116` |
| `source/host/src/device_pairing_owner.rs` | `6645d073450ce3fab5b726a4579cccc41cabaad5` |
| `source/host/tests/device_pairing_rotation_authority_contract.rs` | `a06e0593cf3be3673e91706e1636399dab90d0a8` |
| `source/host/tests/device_pairing_revocation_authority_contract.rs` | `65f57e3c458d0ea74099c20d8b284af6bbd40c0d` |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | `7c37e8093b8b7f2975eb031d76dae1b1419ed13c` |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | `e7c42f88067d8d4695ff7ee356987e493b1f4224` |
| `source/host/app/src/tests/config_secret_boundary_tests.rs` | `04f7b0da6600c60b6540008fbd4f542007bdcf34` |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `aefc325fa76365dc79c3f7d997d37b79f337da5a` |
| `.github/workflows/rust-desktop-runtime.yml` | `d46e99280dd33cb33510d5c79be9c3b3b943904d` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `f5f3d5624568cb553441b8ca0c0f63aff76ebeae` |
| `projects/openclaw-rust/parity-ledger.json` | `228f10fa834ce1a3b25e17fd38843261be880c3e` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `7b6688bbe0f0054c69396f519dd83cde0649b677` |

## Review basis and read limits

I read root `AGENTS.md`, OCRM main-spec §17 including OCRM-EXEC-02, the complete frozen DFA-v1 spec, and the DPA/DVR/DRT/DTR contracts. I inspected the applicable Settings public-projection/commit and LFA contracts, OCRM source-of-truth authority/ownership sections and the relevant module-map entries. These are scoped contract reads, not a claim of rereading the entire OCRM universe or all fixed-U/dependency implementations.

The product deep read covers `authenticate_device_token`, its proof construction and marker/canonicalization dependencies; the extracted private validator and unchanged current-row/role/baseline/token checks; adjacent management admission and token-rotation boundaries; owner construction and the relevant SQLite schema. Integration reads cover the actual Gateway fresh-auth/error/ordinary-command dispatch path and report ordering, server shutdown ownership, actual `UnifiedGatewayApi` authentication/currentness/request hooks, configuration dispatch/helper/commit consumer and production shared-owner composition. I also inspected the App Cargo declarations and the actual floating-point `set_window_focused` signature.

For tests I read all added DFA code, the complete edited DPA/DVR/DTR functions and their proof/observation helpers, and relevant retained-stream successor/reopen/withdrawal consumers. For the large workflow, capture script, ledger and RTM, I read every changed hunk and the adjacent selection, strict acceptance, source-input binding and existing-owner entries needed to judge those hunks. I did not deep-read unrelated unchanged workflow jobs, all unrelated tests, the entire 1.45 MB ledger, or every production owner reachable from the full App.

## Source assessment

### One owner, one currentness policy, same connection

`source/host/src/device_pairing_owner.rs:660–678` constructs the existing sealed candidate and passes it to `validate_current_device_authorization_in` before returning it. Both the fresh path and the public retained validator use that one private implementation (`:1558–1589`). Fresh authentication keeps its existing `conn` guard and supplies `&Connection`; the helper does not acquire the mutex, open another connection, issue a token, write activity/revision state or create a cache/store. The public retained entry still acquires the owner mutex once and returns the validator's typed error.

The exact-credential check still uses private issuer allocation identity, sealed device/role/fingerprint, role-array approval and the active marker. The complete canonical live grant and sealed admitted grant remain checked against the decisive current approval. The SQL-NULL fallback and strict decisive parsing remain in the existing owner. Public projection fields, primary labels and opaque issuer/lifecycle metadata gain no authority.

Fresh error conversion matches only `GatewayCommandError::Unauthorized(_)` to `Ok(None)`. It does not compare numeric status or swallow storage errors. The actual Gateway forwards owner errors before command admission (`gateway_server.rs:1769–1777`), so the fixed decisive-corruption error remains an ingress 500 rather than an absent credential or command report. The retained public method retains `Err(Unauthorized)`.

The extra work is the existing matched-device currentness read and canonical policy on the same connection. It does not add a connection per request, recursive locking, a retained database guard across Settings/network work, repeated retries, polling or a quadratic replacement for the existing set-based canonicalization. No concrete new performance defect was found. No throughput/latency measurement is claimed.

### Causal regression fixtures remain meaningful

The edited late-admission controls preserve their decisive boundary. Invalid live-grant cases now obtain a genuine empty-grant proof before injecting the invalid complete live grant; admitted implication cases obtain the intended valid live grant. DPA's empty-baseline case singles out exactly the known invalid unselected node credential with a counted explicit exception while the ordinary all-current helper remains strict. Its node proof is acquired before approval withdrawal, and post-rotation stale validation preserves the committed image.

The DTR owner cases acquire their proofs before role-array withdrawal. Reopen cases that cannot use a proof from the old issuer assert the precise fresh refusal instead. The changed HTTP successor case makes fresh refusal explicit after reopen. Existing rollback, retained/currentness, untouched-state and unaffected-role assertions in those edited functions remain present; no raw public proof construction or copied policy was introduced.

### Actual HTTP and Settings oracle, bounded composition claim

`config_secret_boundary_tests.rs:481–770` uses a real file-backed pairing owner, real TCP Gateway and the shipping `dispatch_public_settings_command`/`SettingsService` mutation consumer. The wrapper supplies no grant predicate. Seven separate counters distinguish lookups, accepted proofs, the request hook, dispatch, commit callbacks, completion and error reports. Refusals require the exact expected counters, unchanged raw Settings bytes and directory entries, and independently observed full paired/pending/machine rows, including raw SQL values and revision types. Positives require actual persisted timezone changes read through a new Settings service and unchanged pairing state. The cases retain both configured-secret and credential-absent loopback native positives, a current reader's 403, an independent current writer, malformed matched authority, overgrant, partial overgrant, role withdrawal, repair/fallback and owner reopen.

The fixture owns a UUID directory and listener. TCP reads/writes and API-drop acknowledgment are bounded. Request EOF occurs after the command callback/report path. `stop` closes the listener, drops its fixture API reference and waits until the final API drop explicitly closes the pairing connection. Failed teardown retains the Settings fixture rather than deleting live owned files. This addresses the existing server contract that `GatewayServer::close` joins only the accept thread. No unbounded scheduling sleep or global process cleanup is introduced.

The source-binding predicate (`:773–814`) inspects actual production authentication, config dispatch, Settings commit and composition anchors; each of its ten destructive string controls targets an anchor present once in the production portion of the frozen `main.rs`. This is explicitly bounded source evidence. The TCP fixture does **not** construct the complete `UnifiedGatewayApi`, run full App startup, execute its complete post-commit roster side effects or establish packaged/remote-device acceptance. The source, spec, workflow receipts and ledger consistently preserve that distinction.

### Existing CI and evidence owners retained

The existing portable capture adds the three exact new owner names to the complete DVR target; the native workflow mirrors those names. Their acceptance paths still require unique named positives, zero exits, exact count, zero failed/ignored/measured/filtered cases for the complete DVR/DRT/DTR/retained targets. Existing DPA, ordinary Gateway, retained and management selections remain in place. The App selections add both required names to the existing native exact-filter loop and Linux exact executable/list/run path, retaining the prior Settings, media and App cases. Linux still separately observes ordinary shipping feature mode and App test-support mode.

The new spec is included in the applicable native/portable/Linux App input lists. Owner/main/Gateway inputs remain bound. The Linux App input list explicitly includes both changed App test modules, their manifests/locks and the relevant Settings/owner sources. The native recovery input map does not individually enumerate those two App test modules; their identity is bound by its exact full source tree, while the Linux App manifest additionally records their individual hashes. This review does not claim a complete per-file native compiler-dependency hash graph.

The LFA prerequisite is exactly the explicit `serde_json::{Value, json}` import and `1_000.0` fixture timestamp required by the real f64 API. No existing assertion, incoming MSU adapter, production media behavior, dependency or lock is changed by those hunks. This is a source-judged compilation repair; actual compiler acceptance remains pending.

Ledger and RTM changes extend the existing pairing unit and evidence references. They introduce no second completion ledger, alter no global completion flag, and do not claim a unit or global verification result. All required exact-H Actions, scoped authority/coverage and final acceptance obligations remain open.

## Actionable findings

None. No severity/path/scenario/fix entry is warranted by the inspected source. Source approval applies only to BASE `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6` → TREE `9cef1a8dc09c684ad09594fcd85b82f5f8514d40` and does not automatically transfer to a later integrated tree.

No candidate executable check was run or credited. Build/compiler validity, actual owner/HTTP/Settings behavior, teardown behavior on the selected runners, scoped G01/G02 and all runtime/global acceptance require their own evidence.
