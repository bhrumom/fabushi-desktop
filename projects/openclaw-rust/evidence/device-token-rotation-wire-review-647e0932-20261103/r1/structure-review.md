# DTR-v1 round 1 — independent structure, reuse and performance source review

**Verdict: APPROVE for the frozen DTR-v1 source delta.**

No concrete blocking structure, reuse, or performance defect was found in the reviewed source. This is a source-review verdict only. It does not establish exact-H compilation or execution, scoped source authority/coverage, unit verification, upstream compatibility, release readiness, or global acceptance.

## Review identity and boundaries

- Repository: `bhrumom/fabushi-desktop`.
- Committed base: `497c2c7a7bc5201aead83d3d36e44f52ba164d04`.
- Explicit preceding DRT overlay / behavior baseline: tree `f6aaaf556fe99d53b46060da46691e154bffdfac`, admitted as an immutable preceding overlay under OCRM §17.3. Its source presence is not runtime acceptance.
- Reviewed candidate: tree **`5d8dd303c69999f1f8d68d687ab74b779d675464`**.
- DTR manifest: `/workspace/scratch/29e7135d8bfe/reviews/dtr-r1/manifest.json`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Source setting supplied by the parent worker launch: **gpt-6.1-sol, xhigh**. This is the Codex structure/reuse/performance lens selection required by root AGENTS; xhigh is justified by sealed authority, transactional state, retained transport lifetime and CI evidence crossing owners. No CLI, global model/effort setting, or additional reviewer was started.
- Review was fresh and blind: no other reviewer reports, arbitration/decision ledgers, or prior conclusions were read.
- Product source was read through `git show` / `git diff` against the immutable objects above, not through the subsequently changed working tree. The root AGENTS entry instruction was read from its requested repository location.

The manifest's 15 paths define the DTR batch against the declared DRT behavior baseline. The complete diff against the committed base additionally contains preceding DRT material. I inspected the DTR change scope and the load-bearing DRT/native production neighbors; I do not claim a new complete independent review of every unchanged DRT test body.

## Findings

**Blocking findings: none. Non-blocking change requests: none.**

There is no preference-only request to merge the native and wire issuance implementations. Their request, normalization, metadata, authority, surface invalidation, revision admission and response contracts deliberately differ. A mechanical merge would need to preserve those distinctions explicitly; retaining the existing private native operation and a bounded wire operation is reasonable for this batch.

## Production ownership and reuse assessment

The complete production chain is present in the reviewed snapshot:

`POST /api/device.token.rotate` → existing method registry and Gateway ingress → `GatewayApi::rotate_device_token` → actual `UnifiedGatewayApi` override → existing `DevicePairingOwner::rotate_token_with_authority` → existing paired-token/surface/revision tables → commit → existing response/report path.

The method is registered once in the OpenClaw extension inventory and assigned to `HostGatewayOwner::DevicePairing` (`source/host/src/host_gateway_api.rs:37–81`). Gateway marks it mutating and operator-only (`source/host/src/gateway_server.rs:1018–1072`). All dispatched HTTP callers, including native callers, enter the new hook with the optional original proof, singular header ceiling and existing command context (`gateway_server.rs:2006–2023`). The default hook returns a fixed refusal (`gateway_server.rs:225–237`); there is no generic-native fallback for this method.

The actual App hook notes contact once and forwards all authority inputs directly to its existing pairing owner (`source/host/app/src/main.rs:13378–13389`). The trusted generic App branch also reaches that same owner (`main.rs:4144–4183`). The existing App node list/status projection helpers remain in use (`main.rs:2706–2830`); the batch introduces no synthetic connected remote node or replacement authority for `sand-local-node`.

`DevicePairingOwner` remains the single state and authority owner. The new operation reuses its sealed issuer/device/role/fingerprint proof, original admitted grant, existing same-row currentness admission, secure RNG, role implication policy, SQLite mutex/transaction, checked revision helper and fresh/retained credential consumers. No Cargo dependency, schema, permission store, token registry, event loop, polling task or transport owner is introduced.

The shared changes are narrow. `role_scope_policy_allows` extracts the implication predicate while the old `role_scope_allows` wrapper retains native trimming (`device_pairing_owner.rs:94–117`), and `stored_token_entries` extracts the existing strictly typed JSON-map parse (`device_pairing_owner.rs:241–251`). The wire request's ECMAScript trimming and UTF-16 ordering stay at the new wire boundary (`device_pairing_owner.rs:119–180`). The old native/DPA rotation implementation continues to use its former request, default-role, permission, scope, metadata, any-role surface and response path (`device_pairing_owner.rs:1639–1803`).

## State, transaction and lifetime assessment

`rotate_token_with_authority` parses the closed request before mutation, then holds the existing mutex and begins an IMMEDIATE transaction before decisive caller or target reads (`device_pairing_owner.rs:1392–1402`). Reused admission checks the sealed owner instance, exact token generation, approved role and entire current live grant before operator management policy (`device_pairing_owner.rs:1559–1637`). The operation's addressing check precedes target lookup (`device_pairing_owner.rs:1418–1438`), preserving inaccessible-target nondisclosure.

The issuance code preserves the contract's difference between issuance repair and retirement. Target selection uses strictly typed roles plus optional primary role; a selected object may be absent or obsolete without becoming issuance authority. Desired scopes select explicit input, otherwise selected scopes, otherwise device fallback. The approved baseline is mandatory even for an empty explicit grant, and the entire normalized result is checked against the selected role's baseline and, for operator issuance, all caller ceilings (`device_pairing_owner.rs:1439–1494`). Unknown selected metadata and supported lifecycle/issuer JSON survive; only selected generation fields and node-only surfaces change (`device_pairing_owner.rs:1495–1514`).

There is one checked target update and one checked existing revision increment, followed by one commit before any success or token is returned (`device_pairing_owner.rs:1505–1520`). Known failures leave the transaction to roll back. The new code does not start a recovery mutation, replay a lost result, use timestamp ordering as generation identity, or expose a withheld replacement through list/readback. Same-millisecond timestamps therefore do not become identity or deduplication authority.

Delivery is computed from the sealed paired device identity after the durable commit (`device_pairing_owner.rs:1521–1529`). Native callers have no fabricated self identity; a current paired cross-device admin still receives no bearer. The success path does not revalidate the now-retired self proof and suppress a committed self response.

Fresh lookup and retained checks continue to use the existing selected token/fingerprint and active-marker mechanisms. The new method retires its predecessor through the same token replacement that those consumers already observe. Existing event and bridge admission checks remain before subscription/registration/headers/initial output and after wakes, before filters, frames, heartbeats and bridge response effects (`gateway_server.rs:1368–1470`, `1581–1661`). No owner mutex or SQL transaction crosses network output. Frames admitted before commit may finish; quiet expiry remains bounded by existing receive/write behavior, without a new registry or poller.

## Performance and coupling assessment

The production delta adds no background work and no extra periodic database reads. Request normalization runs before the owner lock. While the mutation holds the lock, it performs bounded policy/parsing work on the addressed rows, the existing random-token generation and serialization, one target update and one revision update. The separate fallback/baseline reads occur within the same transaction and do not introduce a second state snapshot or owner.

Sorting normalized requested scopes is local to the mutation and uses UTF-16 iterator comparison, with deduplication after sorting. The policy extraction avoids a second implication implementation. Existing fresh authentication still enumerates stored tokens; this is an unchanged owner behavior, not a new index/scan regression introduced by DTR. I found no source-grounded performance reason to widen this task into a storage or authentication redesign. No benchmark or performance pass is claimed.

The App tests share the existing node projection fixture through test-only sibling visibility instead of copying projection policy. Retained tests extend the existing real event/bridge fixtures. The new owner/HTTP tests own their explicit fixture state and use production policy delegates; their independent SQL observations are evidence instruments, not parallel runtime owners.

## Authored verification and CI custody review

The two new complete integration-test targets were read in full: `device_token_rotation_wire_authority_contract.rs` (37 named cases, lines 1–1488) and `device_token_rotation_wire_gateway_contract.rs` (11 named cases, lines 1–1067). They use real file-backed owners, actual issued credentials, independent raw SQL snapshots and real TCP requests with thin owner delegates. They distinguish request failure, stored corruption, addressing denial, sealed/live/approved/header intersections, selected metadata preservation, known rollback, currentness races and unknown delivery. Their observations do not replace physical-device or complete App transport acceptance.

The six retained additions use existing causal admission barriers, actual event/bridge consumers, real EOF, unaffected/successor controls and provider registration/drop observations. They preserve the prior 20 names in the complete 26-case selection. The new quiet case waits for the actual production 15-second receive heartbeat; timeout is a test failure. The changed `Pending<T>` destructor releases its gate and joins its owned bounded-I/O client rather than leaving an active client behind during fixture teardown (`device_pairing_retained_stream_contract.rs:735–744`). This is compatible with the existing teardown acknowledgement boundary.

The three App additions were read in full (`source/host/app/src/tests/device_token_rotation_wire_tests.rs:1–173`). Two invoke the actual owner and shipping node projection helpers; the third is a source-binding check with deliberate mutation controls. They are correctly described as owner/helper and source-wiring coverage, not execution of a complete shipping App/device connection.

The workflow delta adds the owner and HTTP targets to both applicable runtime lanes, carries the new sources/spec into the relevant input identities, preserves the old DPA/DVR/DRT targets, and expands retained selection to 26. Its receipt checks require every declared positive name exactly once, complete target summaries with zero failed/ignored/measured/filtered cases, command/capture success and exact subject/run identity (`.github/workflows/rust-desktop-runtime.yml:2473–2585`, `2615–2633`, `4128–4208`, `4410–4415`, `4634–4686`). Actual App selection adds the three exact new names while retaining the six prior DVR/DRT names (`rust-desktop-runtime.yml:5191–5203`, `5235–5248`). This is an authored source assessment, not evidence those selections executed.

The parent supplied the source-only fact that incoming runtime run `41569288081` failed before jobs because the incoming 512,095-byte workflow exceeded the official Actions 500 KB workflow limit; the frozen candidate workflow is 538,006 bytes and the root is integrating the active external repair. I did not treat that inherited blocker as acceptance, did not duplicate its repair, and did not inspect later integration bytes. A repaired/published candidate still needs its own exact-H execution evidence.

The ledger/RTM changes link DTR to the existing responsibility, preserve historical read references and retain pending authority/execution/global status. The new source carrier explicitly preserves partial ranges and distinguishes generated contract review from full semantic reading. No newly authored test, source identity, code path, or source approval is being promoted here to `unit_verified` or a global flag.

## Read extent and limits

All 15 manifest paths were dispositioned and every DTR-introduced hunk was read. The large existing files were reviewed at the changed/adjacent owner boundaries; this is not a claim to have deep-read their entire unrelated contents.

| Manifest path | Read/disposition extent |
| --- | --- |
| `projects/openclaw-rust/docs/specs/device-token-rotation-wire.md` | Complete frozen 249-line contract. |
| `projects/openclaw-rust/evidence/device-token-rotation-wire-source-647e0932-20261103.json` | Complete 617-line carrier, including actual source identities, historical range preservation, dependency/remaining-scope and pending-verification fields. |
| `source/host/src/device_pairing_owner.rs` | All DTR hunks; deep reads of lines 1–510, 575–662, 1000–1803 and dispatch neighbors; preceding DRT/native admission, marker, revision, schema and mutation boundaries examined. Unrelated pairing/bootstrap/URL/test bodies were not fully reread. |
| `source/host/src/gateway_server.rs` | All DTR hunks; trait/hooks, method classification, complete protected ingress/dispatch/report path and retained event/bridge admission/output consumers at lines 150–260, 995–1075, 1320–1470, 1581–2085. Unrelated join/avatar/rate-limit internals were not fully reread. |
| `source/host/src/host_gateway_api.rs` | All changed hunks and registry/owner context, lines 1–130. Unrelated native registry entries and nonce internals were not fully reread. |
| `source/host/app/src/main.rs` | All DTR hunks; actual projection helpers 2706–2830, complete OpenClaw branch 4144–4185, pairing authentication/bound hooks/authorization and normal call boundary 13293–13425, test module registration. The approximately 990 KB App file was not fully reread. |
| `source/host/app/src/tests/device_pairing_revocation_tests.rs` | All DTR visibility changes and reused fixture/helpers; existing token-retirement/projection neighbors inspected. Unchanged older test bodies were context, not a claimed new complete review. |
| `source/host/app/src/tests/device_token_rotation_wire_tests.rs` | Complete lines 1–173. |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | Complete lines 1–1488. |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | Complete lines 1–1067. |
| `source/host/tests/device_pairing_retained_stream_contract.rs` | All DTR changes and complete six-case additions; shared fixture, admission instrumentation, production bridge composition, SQL observations, I/O deadlines, EOF, owned pending-client and shutdown helpers deeply read at lines 1–790. Original DVR/DRT cases unchanged by DTR were not all reread in full. |
| `source/host/tests/gateway_protocol_contract.rs` | Complete changed inventory/owner/case-negative hunk with adjacent extension-registry context. |
| `.github/workflows/rust-desktop-runtime.yml` | All DTR changed hunks; relevant input binding, new/retained exact selections, invocation, full-count/name/exit/subject receipt and actual App execution loops. Unrelated large workflow jobs were not fully reread; no YAML/schema/validator was run. |
| `projects/openclaw-rust/parity-ledger.json` | Complete DTR delta in the existing token-lifecycle responsibility, including preserved historical/current source references and status implications. The whole 1.4 MB ledger was not deep-read. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Complete DTR evidence-link delta and surrounding requirement identities. Unrelated RTM entries were not fully reread. |

Governance/adjacent contracts read: root AGENTS; OCRM SOT current authority/constraints and historical-identity separation (first 115 lines); complete current OCRM main Spec §17; relevant M03/M04/M13/M32 module-map rows; the pairing-authority task; complete frozen DPA-v1, DVR-v1 and DRT-v1 specifications. Older historical SOT discovery narrative and unrelated project/domain specifications were not recursively loaded.

Fixed-U source was inspected from the exact source-copy directory supplied by the parent. My own deep/contract reads were: device token handler `src/gateway/server-methods/devices.ts:1–240`; complete token kernel, management authz, token utility, token policy, shared device-auth and operator-scope compatibility files; device request/result schema 1–78; descriptor rows 250–292; the trimmed-string-list dependency implementation; UI node operation ranges 175–225 and 358–460; CLI runtime ranges 447–504 and 919–941. These source roles support the request/result, result-grant, repair, metadata and delivery analysis above.

The supplied 47 exact upstream identities are not 47 newly completed semantic reads by this reviewer. I did not fully reread the generated megabyte predicate file, full generator implementation, all UI components, source tests, CLI, worker broker/store and wake paths. Historical partial ranges in the source carrier remain partial. Full source/dependency/legal closure, complete generated member-set acceptance and consumer compatibility require their separate scoped authority/evidence work.

**Execution performed: none.** No local or htch tests, builds, linters, generators, schema checks, validators, benchmark or candidate harness ran. Only source/Git read operations and this report write were performed. No product, Git or GitHub state was mutated.

