# DFA-v1 R1 independent general, security and lifetime review

Verdict: **APPROVE — source review only**. No blocking or non-blocking defect is established in the complete frozen delta. This is not a compilation result, runtime result, scoped G01/G02 approval, unit `verified` decision, or global/packaged acceptance.

The parent supplied the applied worker setting as `gpt-6-astra` / `xhigh`, selected for fresh bearer authority, sealed proof/current-state semantics, ordinary HTTP mutation and fixture lifetime risk. This worker made no further model/effort override. Review was fresh and blind: no peer review or runner decision report was read. No source edit, Git mutation, build, test, lint, formatting, generator, schema, harness, benchmark, dispatch or local/htch executable validation was performed. The only write is this report.

## Frozen authority and identity

- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Base / starting HEAD: `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6`.
- Reviewed immutable tree: `9cef1a8dc09c684ad09594fcd85b82f5f8514d40`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`; upstream tree: `90c8aab39972da11cfad220361781d49e1bc6700`.
- Scope manifest: `/workspace/scratch/29e7135d8bfe/reviews/device-bearer-admission-r1/manifest.json`.
- Required-case declaration: the adjacent `required-cases.json` (three fresh-owner cases and two distinct App HTTP/source-binding cases, retaining prior selected management/retained/Gateway/App assertions).
- Complete patch independently read by path through `git diff BASE TREE`; patch carrier independently observed as 82,558 bytes with SHA-256 `4fc6eb682286d13b96f478b028f1d529637cca1b1cf68aba383c1526bad799cd`.
- All twelve tree member blob IDs below were independently read with `git ls-tree` and agree with the supplied manifest. The report does not claim independent recomputation of each member's supplied SHA-256.

| Frozen path | Exact blob | Review disposition |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md` | `f0d8a608d1556ff2dd71bc9829df478af3c01116` | Complete specification, DFA-01–08 and App prerequisite amendment read |
| `source/host/src/device_pairing_owner.rs` | `6645d073450ce3fab5b726a4579cccc41cabaad5` | Complete delta; deep affected authentication/currentness/proof/storage/scope/mutation boundaries |
| `source/host/tests/device_pairing_rotation_authority_contract.rs` | `a06e0593cf3be3673e91706e1636399dab90d0a8` | Complete delta, exact unselected-invalid exception and affected fixture/oracle contexts |
| `source/host/tests/device_pairing_revocation_authority_contract.rs` | `65f57e3c458d0ea74099c20d8b284af6bbd40c0d` | Complete delta, new cases and actual fixture/observation/error/proof helpers |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | `7c37e8093b8b7f2975eb031d76dae1b1419ed13c` | Complete delta, proof acquisition/rollback/reopen consumers and selected-successor helper |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | `e7c42f88067d8d4695ff7ee356987e493b1f4224` | Complete delta and successor/reopen/withdrawal context |
| `source/host/app/src/tests/config_secret_boundary_tests.rs` | `04f7b0da6600c60b6540008fbd4f542007bdcf34` | Complete delta, HTTP harness, Settings fixture, state and counter oracles, cleanup and source mutation controls |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `aefc325fa76365dc79c3f7d997d37b79f337da5a` | Both prerequisite hunks and affected imports/fixture construction read |
| `.github/workflows/rust-desktop-runtime.yml` | `d46e99280dd33cb33510d5c79be9c3b3b943904d` | Complete delta; affected subject-input, command selection, exact-name/count/exit and App-executable capture consumers |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `f5f3d5624568cb553441b8ca0c0f63aff76ebeae` | Complete delta; selected owner target and fail-closed receipt consumer read |
| `projects/openclaw-rust/parity-ledger.json` | `228f10fa834ce1a3b25e17fd38843261be880c3e` | Complete delta; existing pairing responsibility extended without status promotion |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `7b6688bbe0f0054c69396f519dd83cde0649b677` | Complete delta; references extended within the existing matrix |

The later remote `74f11d3d31c6245ac5886dc0181460b2d4349744` integration mentioned by the parent was not read or approved in this round. A future integrated snapshot needs its own focused source review.

## Product authority and error analysis

The product delta is confined to the existing pairing owner. At `device_pairing_owner.rs:660–678`, fresh lookup still constructs its proof from the matched token under the existing connection mutex, then calls the private connection-taking validator before returning it. Only the concrete `GatewayCommandError::Unauthorized` variant becomes `Ok(None)`. Other errors propagate. There is no numeric-status catch-all, alternate issuer, independent grant store, new database connection, recursive mutex acquisition or authentication write.

The retained entry at lines 1558–1563 continues acquiring its existing lock and delegates to the same implementation. Its result remains `Result<(), GatewayCommandError>`, preserving retained `Unauthorized` errors rather than converting them into absence.

I did not assume that reuse establishes correctness. I read the validator and its decisive dependencies:

- `current_pairing_record` checks `Arc::ptr_eq` issuer identity before database facts, reads the sealed device identity, requires membership in the normalized approved role array, selects the sealed role's token entry, checks its literal role, the shared active-marker predicate and constant-time fingerprint equality. It does not consult mutable public projection fields or substitute another generation.
- The decisive SQL selection uses `COALESCE(approved_scopes_json, scopes_json)` before decoding. A present valid approval may ignore an unused malformed fallback; SQL NULL uses the fallback. Wrong SQL type or malformed required JSON fails through the fixed `stored pairing authority is invalid` boundary.
- The complete live grant is strictly decoded as a string array. The permissive initial projection at lookup cannot escape as a successful empty/partial proof because the strict validator runs before return. Every canonical live scope must fit approval, and every sealed admitted scope must fit both the live grant and approval. A partially overbroad token is refused as a whole.
- Scope canonicalization uses the shared ECMAScript spelling policy, including BOM trimming and significant NEL; operator implication rules remain enumerated and nonoperator scopes require their own prefix and exact membership. Empty valid arrays remain admissible.
- Existing selected-token identity classifications, integer-zero/absent/null active-marker semantics, metadata opacity and owner-instance boundaries are preserved. The introduced code neither parses lifecycle metadata as permission nor writes last-use, revision, pending rows, surfaces or other credentials.

The bound management admission and retained-stream consumers retain their independent late checks. This is fresh admission at the existing owner's read boundary. It does not hold the pairing lock through a later Settings operation or promise cancellation of an effect already admitted by another owner.

## Shipping chain and actual HTTP oracle

Adjacent exact source identities read for the chain include App `main.rs` blob `7ec9bd8a22fd595f63cb3f4b632673e298294dfa`, Gateway server blob `2cc83cf387ae37c3b1c28663f72d9d0492e84ccd`, Settings service blob `7c17f9a08af73365b062e2f212e0b444bfa546f3`, and Gateway error declaration identity `799a47b3f741f9e9bb224d481cb08a241113529d`.

The source trace is concrete: `gateway_server.rs:1752–1806` distinguishes absent native loopback authority, configured native secret, and explicit paired bearer lookup. An explicit failed lookup remains unauthenticated; a propagated owner error returns before command dispatch. The existing generic mutating operator check runs before the per-request hook. `UnifiedGatewayApi::authenticate_device_token` at `main.rs:13428` delegates to the same durable pairing owner; its ordinary request hook at line 13472 is node-address binding, so it does not independently repair the fresh operator approval gap. Accepted `config.set` follows `call` → `call_openclaw_gateway_command` → `call_openclaw_config_set` → `call_public_settings_command` → `dispatch_public_settings_command` → `SettingsService::set_public_host_settings_if_current`. The latter enters its existing locked compare/merge/persist owner. Composition at lines 15943–15952, 17493–17548 and 17789–17795 supplies that durable pairing owner and the shared Settings Arc to the shipping Gateway.

The new App test executes the actual `DevicePairingOwner`, actual TCP Gateway ingress and actual App `dispatch_public_settings_command` / Settings consumer through a bounded test `GatewayApi`. It does **not** instantiate `UnifiedGatewayApi`, launch the complete App or establish packaged execution. Its separate source-binding test checks the shipping delegation/composition text and deliberately mutates each selected binding to make the source predicate fail. I inspected the actual matched shipping locations, including the unique first matches used by `replacen`; this evidence category remains distinct from the HTTP helper execution.

The HTTP case keeps known bearer bytes while independently changing stored approval, an additional live scope, the approved role array or decisive malformed approval. It tests both configured and unconfigured listeners, HTTP 401 versus fixed 500, zero bound-dispatch/commit/report counters, byte-identical actual Settings data, no new Settings-side entry, and all paired/pending/machine-state rows. The valid legacy fallback writer, independent writer, admitted read-only 403 and genuine native controls prevent a deny-all implementation from satisfying the test. Reopen proves persistent overgrant refusal; explicit repair is followed by a successful real Settings commit and a fresh Settings owner reading the changed timezone. Error checks exclude bearer, native-secret, synthetic Settings secrets and the injected malformed authority value.

Fixture lifetime is compatible with the real server's detached connection handlers: `GatewayServer::close` alone joins only its accept thread, while `DfaHttpFixture::stop` additionally drops its own API Arc and waits for the final API destructor's acknowledgment after the pairing owner has been dropped. Reopen occurs only after that acknowledgment. A timeout keeps the Settings fixture directory rather than removing storage while its connection may remain owned. HTTP reads use an absolute deadline and bounded response size; timeout/parse failure cannot count as a successful refusal.

## Compatibility and evidence review

DFA-08 changes preserve causality. DPA captures the node proof while it is still valid, narrows approval afterward, then explicitly identifies the one unchanged but now-invalid unselected `(TARGET, node)` credential. Its general all-current helper is not globally weakened: the exception is passed only by that case and the helper verifies exactly one match. The existing complete-state/token/revision assertions remain. The later retained node proof is still required to fail without state change.

DVR's invalid implication cases obtain a genuine empty-grant proof before changing the live grant; valid cases still obtain their intended valid grant. DPA and DTR reopen cases now test fresh refusal directly where an old-owner proof would prove only issuer mismatch. DTR primary-only rollback and concurrent-manager cases acquire proofs before role withdrawal and retain the later refusal/rollback assertions. The HTTP primary-only successor case requires `None` on withdrawn reconnect instead of conditionally accepting either branch.

I searched the repository-wide fresh/retained proof consumers and inspected the necessary fixture and invalidation contexts in the existing DRT owner, DPA/DVR HTTP and retained-stream targets, plus the affected DTR contexts. Existing retained event/bridge tests authenticate before their controlled withdrawal/registration/admission barriers; their late validation remains reachable. In the unmodified DRT target, corrupt target issuance baselines are not used to mint caller proofs, and selected retired-token lookup remains an absence check. The DTR success helper already requires the new successor to pass currentness; issuance materializes the selected primary-only role before that lookup. No further concrete incompatibility was established by these source checks.

The new owner cases use real file-backed storage, independent read-only complete-row snapshots, fixed typed/redacted failures, unaffected credential controls, explicit repair and reopen/foreign-issuer checks. They cover strict malformed scopes/roles/baselines and SQL types, valid NULL fallback, invalid missing fallback, unused malformed fallback, full canonical grants, role/map mismatch and retirement. These are authored causal oracles, not observed test passes.

Actions and portable capture retain their existing targets and names while adding the three fresh-owner positives and two distinct App positives. The owner capture still requires complete-target exact counts, zero failed/ignored/measured/filtered cases, unique required positive names and zero command/capture outcomes. The Linux App path still checks shipping compilation, binds the actual produced test executable, lists each exact requested name and demands one passed/zero failed/zero ignored completion. No old result is upgraded by the added scope prose. The single ledger/RTM change adds behavior/contracts/test references and explicitly retains exact-H execution and scoped-authority requirements; it does not mark the unit verified or change global acceptance flags.

The LFA prerequisite is limited to importing `serde_json::{Value, json}` and making the existing focused-window fixture argument floating point (`1_000.0`). Existing App dependency declarations already provide serde_json and rusqlite. The media fixture's imported custody adaptations and assertions are retained. This source assessment neither re-runs nor grants execution credit to the original failed App build, the base plugin recovery or the candidate.

## Read and verdict limits

Read authority was the immutable Git tree and its complete base delta. I read root `AGENTS.md`, the complete frozen DFA specification, OCRM execution §17, the relevant DPA/DVR/DRT/DTR contracts and Settings/LFA contracts. Large unchanged source/test/workflow files were read at the affected functions, necessary callees/callers, fixture producers, invalidation sites and capture consumers; this report does not claim every unchanged line in those files was deeply reviewed. The large unchanged ledger/RTM bodies and unrelated product domains were not re-audited. Discovery searches selected those reads and are not represented as semantic proof by themselves.

The frozen source-identity pack was consulted for upstream provenance/read limits. The direct fixed-U policy blob `36b7e7da9ee177b31fad17e379743418c558a18f` is not present in this repository's Git object database; no substitute live upstream was fetched. This round therefore does not claim a new full upstream source read, scoped source-authority closure, dependency qualification or exact upstream parity. Historical read facts and their limits remain unchanged.

There are no findings requiring repair from this review. Required remaining gates are the independently reviewed integrated snapshot, exact published-H Actions compilation and selected runtime/oracle capture, scoped G01/G02 authority/responsibility evidence before unit verification, and the original wider OCRM product/device/platform/package acceptance. Source approval is confined to tree `9cef1a8dc09c684ad09594fcd85b82f5f8514d40` against base `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6`.
