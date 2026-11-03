# Device bearer admission — independent structure/reuse/performance R2 review

Verdict: **REQUEST CHANGES — one P2 performance finding.** This is a source review of the frozen candidate, not executable validation or acceptance. The shared-owner design and incoming composition are otherwise suitable for the declared DFA slice.

## Review identity, settings and constraints

- Task: `647e0932-bea4-4113-9a61-65ae3db58955`, round 1, independent review R2.
- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Frozen original base / starting HEAD: `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6`.
- Previous immutable tree: `9cef1a8dc09c684ad09594fcd85b82f5f8514d40`.
- Reviewed integrated candidate tree: `b415b8a429b3e0ffe7a2f2ba0e86b2c67f62eab0`.
- Publication parent: `74f11d3d31c6245ac5886dc0181460b2d4349744`.
- Adjacent incoming App main: blob `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df`, independently confirmed in the candidate tree.
- Fixed upstream identity declared by the applicable contracts: U `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Assigned lens: combined structure/reuse/performance. The parent launch states actual `gpt-6.1-sol` / `xhigh`, selected for paired-credential currentness, shared workflow/ledger integration and the real HTTP/Settings fixture lifecycle. This worker has no independent runtime model/effort introspection surface. It did not change global/session settings, start another CLI, or add workers.

I used Git object reads/diffs, source-text searches and read-only hashes. I did not run a build, test, formatter, lint, generator, schema checker, harness or benchmark locally or on htch; I did not dispatch Actions or write to GitHub. I did not inspect peer reviews, previous reviewer reports, runner decisions or implementation-assessment reports. The only write is this report.

The manifest's twelve candidate blob identities match `git ls-tree`. The supplied patch files also match their declared SHA-256 values:

| Artifact | SHA-256 |
| --- | --- |
| `complete.patch` | `94945a6f24bb751dcb748be1c1688a65a65ddfdb285ba678b4366c535301053d` |
| `integration-delta.patch` | `aeb6909eefc21fd6c6cbbca1d58a2f2e87cee68acc68570c6b2e0078967c498b` |
| `publication.patch` | `54ecab7dc36f6a5c48f3c88ed1d008c0e38f68e1b54de0f8e85b23e3ca492b5f` |

## Finding STR-PERF-01 — P2: ordinary fresh authentication adds quadratic scope-membership work under the shared pairing mutex

**Affected source:** `source/host/src/device_pairing_owner.rs`, newly added call in `authenticate_device_token` around lines 672–678; shared `validate_current_device_authorization_in` around lines 1570–1595; `role_scope_allows` around lines 94–99.

**Trigger and failure:** A legitimately approved credential can contain N distinct literal scopes of its own role. For example, a `node` pairing with scopes `node.s00000` through `node.s19999`, whose approved baseline and issued token grant contain those same 20,000 entries, is valid under the current native contract. `handle_pair_request` stores the submitted scope array without a scope-count bound, and `handle_pair_approve` copies that baseline into approval and calls `scopes_for_role` to issue the selected token. The Gateway's current body bound is much larger than this sub-megabyte input. No direct database corruption or forged proof is needed; initial trusted pairing approval is a stated precondition.

The new fresh-authentication call holds `self.conn`'s mutex while invoking the shared validator. That validator checks every live scope against the approved vector, then every admitted scope against both the live and approved vectors. Every `role_scope_allows` membership lookup is `granted.iter().any(...)`. With matching first-seen order and N distinct nonoperator scopes, the three successful membership passes perform `3 * N * (N + 1) / 2` string equality attempts. For N = 20,000 that is 600,030,000 equality attempts for a single fresh lookup, in addition to parsing/canonicalization. This is an algorithmic source count, not an observed duration or benchmark.

Before the patch, fresh lookup performed token matching and canonicalization but did not perform those quadratic full-grant membership passes. After the patch, every small ordinary request carrying that bearer pays the cost before method-level admission. Concurrent bearer requests queue behind the same pairing mutex, delaying other authentication and pairing-owner operations. The retained validator already had the vector-scan shape; the regression here is placing that shape on every ordinary fresh request. This does not dispute the required new authorization check or propose bypassing it.

**Specific fix:** Keep one shared connection-taking validator and the existing policy semantics. Construct local membership sets for the canonical live and approved grants once, and pass their membership operations to the existing `role_scope_policy_allows` helper. Preserve the vectors used for public/sealed projections, exact scope spelling, role prefixes, operator implications, complete-live-grant checks, retained-admission intersection and error precedence. Do not hide the work by weakening the grant check or adding an incompatible request/storage scope cap. Sets can remain local to validation; no persistent cache, new owner or extra connection is needed.

**Falsifiable recovery evidence:** Source inspection should show that full-grant validation no longer performs a linear scan of the complete grant for each scope. In exact-H Actions, use a valid large literal same-role grant through fresh authentication/actual HTTP, with a healthy ordinary credential and complete unchanged-state controls. Preserve all existing implication, Unicode, malformed-storage, retained-proof and management controls. A machine-dependent timing threshold is unnecessary to prove the algorithmic fix; a successful large-grant runtime control establishes that the optimized path retains admission semantics.

## Complete scope and integration observations

I read every original-base-to-candidate changed hunk in the declared twelve paths, the complete DFA spec, the full integration delta for those paths, the edited functions and necessary callers/consumers. The owner and five affected owner/HTTP/App test files are unchanged relative to the previous immutable candidate, confirmed by an empty Git diff over those six paths. This review does not borrow an earlier review verdict.

| Scope path | Source disposition |
| --- | --- |
| `projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md` | Full DFA requirements, boundaries, oracles and prerequisite text read. |
| `source/host/src/device_pairing_owner.rs` | All changed hunks and adjacent fresh lookup, sealed/current-row/grant policy, canonicalization, request/approval and relevant mutation callers read. |
| `source/host/tests/device_pairing_rotation_authority_contract.rs` | All changed hunks; proof acquisition, assertion helper, empty-baseline and reopen controls read. |
| `source/host/tests/device_pairing_revocation_authority_contract.rs` | All changed hunks; existing proof/storage/snapshot helpers and new fresh admission cases read. |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | All changed hunks and affected proof acquisition/withdrawal/rollback contexts read. |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | All changed hunks and successor/withdrawal context read. |
| `source/host/app/src/tests/config_secret_boundary_tests.rs` | All changes and existing Settings fixture/interface read; actual Gateway lifecycle and Settings dispatch consumers traced. |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | Exact import and floating-point fixture prerequisite changes read; publication-parent diff is empty. |
| `.github/workflows/rust-desktop-runtime.yml` | All complete-scope/integration changes and relevant input, execution selection, receipt and strict result-capture consumers read. |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | All changes; selected owner targets, positive-name requirements and strict complete-summary/exit acceptance read. |
| `projects/openclaw-rust/parity-ledger.json` | All changes read; DFA additions and incoming cron reference union inspected. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | All changes read; DFA additions and incoming cron evidence references inspected. |

The incoming cron integration is additive in the shared workflow, ledger and RTM. It retains the five exact cron test names and its three explicit source inputs alongside the two DFA App cases. Existing test names are not removed in the complete diff. The Linux App capture selects each required name with `--exact --list`, requires that exact single listing, then requires every captured summary to be one passed / zero failed / zero ignored and requires all cases to be recorded. Its counts follow the resulting `required` list rather than a stale hardcoded total. The portable owner capture adds the three fresh cases to the existing revocation target and retains unique-positive-name, exact complete-summary and zero command/capture-exit requirements.

The App test inputs bind the current main, pairing owner, Gateway server, actual Settings owner/projection, manifests/locks, DFA spec and test source. Repeated input names are deduplicated by the existing `sorted(set(inputs) | ...)` mechanism. No new duplicate import was introduced by the remote local-media repair: the candidate contains one explicit `serde_json::{json, Value}` import and the timestamp literal is `1_000.0`. That file exactly matches the publication parent and is not a new owned publication delta.

The actual incoming main still delegates `UnifiedGatewayApi::authenticate_device_token` to `self.device_pairing.authenticate_device_token(token)`. Its ordinary Gateway dispatch reaches `call_openclaw_config_set`, then `call_public_settings_command`, then `dispatch_public_settings_command`, then `SettingsService::set_public_host_settings_if_current`. The concrete composition still supplies the same pairing owner and the shared Settings `Arc`, and `GatewayServerDeps` still receives `gateway_api.clone()`. I checked the exact source-binding delimiters and mutation literals against this incoming main; the specific twelve-space pairing/Settings composition literals have their intended occurrence. The older blob identity in the test comment is historical context; actual test input comes from `include_str!("../main.rs")` and the current receipt input hash.

The fresh-authentication product change does not recursively acquire the mutex, open another connection, write state, add a schema field or create another policy/store owner. Only typed `GatewayCommandError::Unauthorized` becomes absent fresh authentication; corruption still propagates as an internal error, and the public retained validator keeps its original error behavior. The new App HTTP adapter executes the real pairing owner and Settings helper, while separately identifying shipping text bindings as source evidence. It does not claim to construct the full `UnifiedGatewayApi`.

The fixture's teardown handles the production Gateway server's detached handler ownership: server close stops/joins the accept thread, then the fixture drops its own API reference and awaits the API Drop acknowledgement after the pairing owner is dropped. Reopen occurs only after that acknowledgement. A teardown timeout preserves the Settings fixture instead of recursively removing a still-owned database directory. Refusal counters distinguish zero-dispatch authentication refusal from an admitted reader's later 403; successful controls require the same actual Settings file to change and a fresh Settings owner to observe the result.

The DPA empty-baseline compatibility helper allows only the exact named unselected invalid credential and counts that exception. It does not weaken the global all-current helper. Changed retained/management controls acquire genuine proofs before invalidating authority; fresh-after-reopen controls use direct refusal rather than allowing old-owner issuer mismatch to stand in for grant withdrawal. No acceptance/status/global flag is promoted by the ledger/RTM changes.

## Read and verdict limits

I read root `AGENTS.md` and confirmed there is no additional AGENTS file on the reviewed paths' ancestor chain. I read main Spec §17 including OCRM-EXEC-02, full DFA, DPA, DVR, DRT and DTR contracts, and the necessary Settings/LFA contracts. I read current SOT authority/ownership sections and relevant module-map responsibility text; the long historical discovery appendix was not a fresh full read. Adjacent production reads covered the actual App auth/configuration/composition seams, Gateway authentication/guard/dispatch and accept/close lifetimes, scope and approval producers, and relevant Cargo declarations. I searched additional retained/App/role-token consumers where needed; I did not deep-read every line of the large preexisting test files or unrelated incoming cron product implementation.

I did not re-read the entire pinned upstream/source-dependency universe or certify historical source-pack read claims, global G01/G02 closure, JSON/YAML execution, compilation or authored tests. No timing was measured. Source scope identity/hash checks and semantic reasoning above cannot establish runtime success, scoped authority/coverage, full App construction, real providers/devices, platform packaging or global acceptance. The one P2 finding needs adjudication and, if accepted, a focused immutable source re-review plus the affected exact-H Actions evidence.
