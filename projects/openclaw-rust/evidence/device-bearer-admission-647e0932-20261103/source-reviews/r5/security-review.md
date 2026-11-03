# DFA round 5 — independent integration, security and lifetime review

**Verdict: APPROVE the bounded source integration. No actionable finding.** This is a source-only review of the immutable original 12-path DFA scope, focused on the incoming Automations gate integration. It is not a test result, unit-verification verdict, source-closure verdict, or global acceptance.

Reviewer launch metadata supplied by the root host: accepted model `gpt-6-astra`, effort `xhigh`, `fork_turns=none`. This report records that supplied host metadata; it does not infer configuration from the prompt or independently inspect the launch adapter. Xhigh is appropriate because the assignment combines shared-gate integration, authorization and custody/lifetime boundaries, and source-authority completeness. It is not routine bookkeeping.

The review was fresh and blind. I read no prior or peer review report, arbitration decision file, or implementation report. Repository/source/spec files, the supplied manifests, and the two scope patches were the inputs. No repository source was edited. No local/htch/Actions parser, generator, lint, build, test, benchmark, or harness was executed. Git object inspection, literal source reading/search, byte counting and SHA-256 identity checks were the only investigative commands.

## Immutable scope and input facts

Repository root was verified as `/workspace/scratch/29e7135d8bfe/fabushi-desktop`. The reviewed object resolves to a Git tree. Original base and starting HEAD both remain `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6`; this review did not substitute the publication parent for the original review base.

| Identity | Value |
| --- | --- |
| Candidate tree | `63385ed61d0ef7074dab86baea7fab548224416d` |
| Previous-round tree | `f3a5f945195bd82f32537a6740817b5bdbf45857` |
| Publication parent | `f41a6d3c00150ba59453d14887b5b4914801d4f4` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Complete original-scope patch | 104,106 bytes; SHA-256 `246be0aeb1cdc86153e0a2922f8bc1fca16a1b66b08eef38738f9c2016a3cc85` |
| Focused integration patch | 3,647 bytes; SHA-256 `c06e34f77857158443dd639494a7c909851ac412649cc2b941d9936b89a3afde` |

Inputs are `/workspace/scratch/29e7135d8bfe/reviews/device-bearer-admission-r5/{manifest.json,complete.patch,focused-delta.patch,required-cases.json,media-scope-preservation.json}`. Both on-disk patch sizes and digests match the manifest. Reproducing the binary/full-index Git diff against the original base with the exact 12 member paths yields the complete-patch digest above. Reproducing the workflow diff from the previous-round tree yields the focused-patch digest above. These are identity checks, not executable project validation.

All 12 candidate members were checked directly with `git ls-tree -rl`; all sizes and blob identities match. Each candidate blob was independently read through Git and hashed, with the following matching results:

| Original scope member | Git blob | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| `projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md` | `f99fe2e81da2822ef7dd871bceba1215d3b0229e` | 12414 | `b1a4ad977cedcf99535994b4173e5a21e40ec51bb124c6570ccf10f2ecd04437` |
| `source/host/src/device_pairing_owner.rs` | `9825003cf1f7047027aa1b8e496c40f0c1693a23` | 103155 | `719a84fe502691e1a14008167f3652174b9a7fe7bd3975ed34067b5351885073` |
| `source/host/tests/device_pairing_rotation_authority_contract.rs` | `a06e0593cf3be3673e91706e1636399dab90d0a8` | 53748 | `c274e92f0cdc112115b30a61c6c51ce07ea994f1b3f71c0dc112e6c481a66bf0` |
| `source/host/tests/device_pairing_revocation_authority_contract.rs` | `212b1067c4ffed3523aec5ebb2c3104eafe4c86f` | 77142 | `3b47cfc16cf3d26cd0c34443c5d3d356bddf06ef7568c10bfbc6895014ecc7e3` |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | `7c37e8093b8b7f2975eb031d76dae1b1419ed13c` | 100799 | `5b8777a7959a283c682287def308f92d41fe3e9f03023da9a3aa87195d0f7396` |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | `e7c42f88067d8d4695ff7ee356987e493b1f4224` | 73706 | `913542fe4597457c96500843f057c1b5952d27ec22956ad3b32a77bd2dcdb736` |
| `source/host/app/src/tests/config_secret_boundary_tests.rs` | `8f3405e6335dac35e6df87674241ad4d2df2dc07` | 47284 | `1ce6049924c9f1bcfbe305a1a94994232bffe6c29bc8848b688623c1c83f52e0` |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `57a0a3393f1bb9560eb036687688656b1f3fe6c8` | 36330 | `a310f803114b6e3274ea6b8256a09ee2b5f590ef614508f22f8b8947e4126944` |
| `.github/workflows/rust-desktop-runtime.yml` | `25e80da72d20d4dbfed8ab6f2a50a3eff62b53af` | 456079 | `68f0f49cc3d6d5f1c96156a4a96e862555accdcc6fe015a36ed5195f2e524803` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `f5f3d5624568cb553441b8ca0c0f63aff76ebeae` | 105974 | `e101cf22f5dbda30f2a024010bff64e13d0ed4bee7b6dbac89afee4c629d8b2e` |
| `projects/openclaw-rust/parity-ledger.json` | `285f9ca672947d9ad054832557c0b4abf38a0d68` | 1457556 | `d8b197d9e9bf18b5a7f1515116ef524bcf20d518382618cab1f9446c3ae70c8b` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `1bdbd1ca17b2ff334f365183c51bcd15c7ca0871` | 161514 | `e724440db1cb0391d4512c03f8fab5f7e3a08f5762e35c42c82b41f3fa271961` |

Within these 12 members, the previous-tree-to-candidate diff names only `.github/workflows/rust-desktop-runtime.yml`. Thus the complete DFA specification, product, tests, App prerequisite, portable capture, ledger and RTM content is preserved byte for byte from that tree. The complete 104,106-byte scope patch was retained and read as context; the review was not silently reduced to a new three-test scope.

The whole candidate tree also carries incoming work outside these 12 paths. Whole-tree differences are not being represented as the DFA scope. The publication-parent workflow diff contains the DFA additions and no deletion or alteration of the incoming poll-custody gate. The parent-to-candidate whole-tree path list also includes two separately scoped MSU members; those are outside this DFA verdict and are accounted for by their preserved identities below.

## Authority and reading boundary

Read the frozen root `AGENTS.md`, including the independent/frozen/focused review contract and model/effort policy, and checked that no applicable nested AGENTS file exists under the reviewed paths. Read the current authority sections of `projects/openclaw-rust/SOURCE_OF_TRUTH.md`, the fixed-source lock, and relevant owner-map context. Read OCRM-001's risk-proportionate source and execution amendment §17, including identity/disposition, existing-owner integration, scoped G01/G02 before verified, Actions-only execution and unchanged final gates. Read the complete DFA-v1 specification, including DFA-01 through DFA-10 and its precise App/HTTP/source-binding limits.

The adjacent incoming contract read was `docs/specs/sand-completion-ack-safety.md`'s checked-poll-custody section. The actual imported test bodies and their producer/owner paths were read directly: `automations_backend_contract.rs` lines 232–565; `sand_automation_fire_consumer.rs`'s parsing, state, complete, poll, retry and failure paths; `backend_transport.rs`; `production_lifecycle.rs` construction and error sink; and the actual `production.rs` construction/poll/dispatch/completion call sites. This is a bounded integration read, not a new comprehensive Automations or global audit.

## Focused integration assessment

The workflow command at lines 3834–3849 still runs all nine declared targets with `--locked --no-fail-fast`, including the actual `automations_backend_contract` target, and records both Cargo and tee capture exit statuses. The incoming additions at lines 3898–3906 require precisely these three names, each counted from an anchored `test NAME ... ok` result and accepted only when its count equals one:

- `fire_runtime_invalid_poll_preserves_all_custody_until_checked_recovery`
- `fire_runtime_invalid_poll_does_not_consume_notify_or_unschedulable_drain`
- `fire_runtime_checked_poll_preserves_opaque_ids_duplicates_and_optional_fields`

These names exactly match the test methods at test-source lines 242, 407 and 476. The target has 21 `#[test]` declarations in the candidate versus 18 in the previous-round tree. This is a source count; no claim is made that 21 tests executed. The gate adds three exact required positives and does not replace the retained invocation, memory, legacy-slot, nonempty-summary, zero-failure/ignored, process/capture or exact-HEAD predicates. An absent, ignored, failed or duplicate new positive cannot satisfy `poll_custody_passed`. The aggregate `accepted` expression includes that predicate, and line 3968 returns nonzero for refusal.

The new receipt fields are data derived from the gate's actual names and occurrences. The input set at lines 3908–3938 adds the precise safety spec, backend transport and production lifecycle files while preserving the fire-consumer owner, `production.rs`, the actual test target, Host Cargo inputs and shipping App source. The receipt records subject commit/tree, upstream, run identity, command, both exits, input hashes and original-log hash. Its scope text expressly limits the claim to local in-memory response acceptance and excludes persisted restart, Sand-principal binding and deployed ACK/idempotency acceptance. This matches the adjacent source and spec.

The source read establishes substantive test binding. The first new case seeds Running, Completed-with-unconfirmed-report, and Reported entries using real `tick`/`complete` methods. Invalid envelopes and malformed members, including a valid prefix, must retain all custody, preserve the exact deferred completion payload, avoid dispatch/retry, repeat the same ACK body, observe error backoff, then recover through valid polls and per-ID retention/release. The second case exercises stop/readiness and both initial/notify-drain paths, showing that malformed responses cannot consume successful-drain eligibility. The third checks exact opaque/case/Unicode identities, valid duplicate suppression, optional/extra-field compatibility, completion payloads and final valid-empty release. Only the final transport is controlled.

In the actual owner, the complete response is collected as `Result<Vec<_>, _>` before successful-drain state, Reported retention, dispatch or completion retry is changed (`sand_automation_fire_consumer.rs` lines 262–345). Decode errors reach existing backoff; production construction routes the same consumer through its existing transport and lifecycle. These observations support the narrow receipt wording and reveal no new lifetime or authorization owner introduced by the integration.

## Retained DFA assessment

All ten DFA requirements remain represented in the unchanged source and tests; none is weakened by the workflow union.

| Requirement group | Source assessment |
| --- | --- |
| DFA-01–03: proof admission, shared lock/validator, typed error mapping | `DevicePairingOwner::authenticate_device_token` builds a candidate sealed proof and calls the connection-taking private validator under its existing lock before returning it (owner lines 611–683). Only typed `GatewayCommandError::Unauthorized` becomes absence. The public retained validator still locks once and delegates without mapping its error (1559–1601). No recursive lock acquisition or new connection/store/write is introduced. |
| DFA-04–05: semantics and state | `current_pairing_record` checks owner-local issuer identity, approved roles, selected map entry/role, active marker and exact token fingerprint (1605–1641). Strict decisive baseline and scope parsing remain in the shared path. Currentness uses sealed fields; public projections do not become authority. Authentication adds no write, revision, activity or downstream transaction claim. |
| DFA-06–07: actual owner, HTTP and Settings consumer | The App case uses the real file-backed pairing owner, TCP Gateway ingress and actual `dispatch_public_settings_command`/Settings file. It checks 401 refusal, fixed/redacted 500, zero bound/dispatch/commit/report effects and full paired/pending/machine-state equality, then valid native/read/writer/fallback/repair/reopen controls. The separate mutation-sensitive source-binding case remains distinct from runtime evidence. Actual `main.rs` delegation/configuration/Settings/composition sites and Gateway fresh authentication/error handling were inspected; full App construction is not claimed. |
| DFA-08: causal retained/management fixtures | Full patch edits acquire genuine valid proofs before withdrawing roles/grants, retain subsequent rejection/state controls, use fresh refusal after reopen where issuer differences would otherwise mask the intended reason, and pin exactly the known unselected node credential in the empty-baseline DPA case. The global default helper still checks all other credentials. |
| DFA-09–10: bounded membership work and strict parsing | Live/approved membership sets are built once and passed through the existing implication policy (1579–1594). Role deduplication uses a local set while preserving first-seen order, Rust trim, empty filtering and strict validation of every element (74–85). Large owner/HTTP controls retain the whole literal grant, reject a single unapproved tail and malformed trailing role value, and repair/reopen. No timing execution or performance measurement is claimed. |

The fixture lifetime controls are compatible with the source-only verdict. `DfaHttpFixture` bounds response waits and size, closes the server and drops its local API reference, waits for the actual API owner's drop acknowledgement before reopening, and retains the Settings fixture if teardown cannot be acknowledged. It does not remove files while a surviving handler may retain the pairing connection. Existing downstream effects remain outside the admission lock, consistent with the spec's refusal to promise cross-owner cancellation.

The three owner case names in `required-cases.json` remain required by both the workflow and portable capture. The portable `device-pairing-revocation-authority` target still requires each exact positive once and the exact complete zero-failed/ignored/measured/filtered summary before its aggregate receipt may pass (`capture_portable_contracts.py` lines 1094–1131). Workflow rotation aggregation retains original DPA/DVR/DRT/DTR and retained-stream selections and exact command/capture/subject binding (2602–2683). The two fully qualified App names remain selected in both affected workflow paths. The Linux App lane still compiles shipping and test modes, observes the intended Host feature mode, requires exact test listing, checks each one-passed/zero-failed/zero-ignored result, preserves logs, and exits nonzero unless the complete receipt passes (4135–4412). None of these source predicates was executed in this review.

The local-media prerequisite is still only the explicit serde imports and floating-point timestamp fixture argument. Ledger and RTM changes retain their pending scope, and all eight top-level completion flags remain false. No read, mapping, implementation, execution or acceptance state is promoted by this source review.

## Separate MSU identity carry and final limits

Direct tree listings prove these three members have identical blobs and sizes in both `f3a5f945195bd82f32537a6740817b5bdbf45857` and the candidate:

| Separate MSU member | Unchanged blob | Bytes |
| --- | --- | ---: |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `8aee21c69bab72824025e4304eec38388ba1e9f8` | 15620 |
| `source/host/src/extensions/session/agent_db.rs` | `87186bc4b246d88a5ce51ebda0623b76f4425d3d` | 426814 |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `88a80214aa08a1d61b3bd290063069d5aa4d417e` | 168267 |

This is identity preservation only. I did not read their prior reviews, assign a fresh MSU implementation verdict, or create an execution verdict for those members.

There are no findings requiring a fix, no preference-only requests and no deferred source defect in this bounded review. Approval applies to the exact immutable original 12-path scope and its focused shared-gate integration. Exact published-H Actions compilation/execution, named positives and negative oracles, original artifact custody, scoped G01/G02, and all applicable G01–G12/AC01–AC24 release/ecosystem/platform gates remain separate obligations. `unitVerified=false` and `globalAcceptance=false` are preserved.
