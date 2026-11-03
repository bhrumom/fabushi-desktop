# DTR-v1 R2 independent GENERAL / security / lifetime source review

## Verdict and authority

**SOURCE-ONLY ACCEPT — no concrete blocking finding identified in the reviewed frozen scope.** This is an independent code/specification/oracle review. It does not establish compilation, passing tests, a successful GitHub Actions run, scoped authority acceptance, unit completion, or global OCRM acceptance.

- Task: `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, review round 2.
- Reviewer: fresh blind GENERAL/security/lifetime worker `/root/dtr_security_r2`.
- Actual configured model/effort supplied by the parent: `gpt-6-astra` / `xhigh`. Appropriate because the scope changes shared credential authority, selected-role transaction state, reopen/concurrency behavior, and retained transport admission.
- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Frozen candidate tree: `7644b264bf72ef57f7333a7f4f8d7accd2dd7af8`.
- Committed publication base: `a27a3a6daec5377a56f61221b34ac3f3c8c4005e`.
- Previous review tree used for the source fix delta: `5d8dd303c69999f1f8d68d687ab74b779d675464`.
- Original committed base: `497c2c7a7bc5201aead83d3d36e44f52ba164d04`; explicit behavior overlay: `f6aaaf556fe99d53b46060da46691e154bffdfac`.
- Scope manifest: `/workspace/scratch/29e7135d8bfe/reviews/dtr-r2/manifest.json`, all 16 declared paths.
- Parent supplied publication identity `113c32276b99ed668be095e0be6ed164dffdf46d` with the same candidate tree. This reviewer did not independently inspect GitHub or rely on concurrent execution.
- Fixed upstream authority: OpenClaw commit `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`, using the supplied immutable exact-source cache and its identity manifest.

Product reads used exact Git objects. No product source, Git state, GitHub state, or other review artifact was mutated. No local or htch build, test, lint, generator, schema check, candidate harness, or runtime acceptance was executed. No peer/prior review reports, implementation handoffs, or root arbitration/decision artifacts were consulted. This report is the only authored file.

## Findings

**No severity-bearing findings.** I found no reproducible source-level scenario in the changed code that justified a security, lifetime, correctness, or acceptance-gate blocker. The review did not turn broader explicitly deferred upstream obligations or unavailable runtime evidence into a claim of completion.

## R2 fixes and security conclusions

### One credential spelling across producers and decisive readers

`source/host/src/device_pairing_owner.rs:94-164` makes the distinction explicit and preserves it at the call sites. Stored/admitted credential grants use the ECMAScript whitespace set, where NEL is significant and BOM is whitespace. Canonical credential normalization removes empty entries and duplicates without adding permissions. DTR resulting-scope normalization separately expands the specified admin/write implications and sorts by UTF-16 order. The authority policy compares the already adapted requested string against canonical grant strings; it does not apply Rust trimming again and accidentally turn a literal NEL-bearing grant into `operator.admin`.

I traced the shared change into native token production through `scopes_for_role`, fresh bearer proof creation, pending scope-set compatibility, strict live/approved grant reads, bound DPA/DVR/DRT/DTR management, and retained currentness. The legacy DPA explicit request adapter remains separate at `device_pairing_owner.rs:1755` and continues to use native trimming. Thus the R2 shared fix does not silently change that older explicit-input contract. DPA target reads still use `rotation_record` and its existing raw fields; the new decisive SQL selection is confined to credential currentness.

The owner authority tests provide specific source oracles for minted literal NEL grants under both literal and admin approval, actual DPA/DRT/DVR/DTR use, positive admin controls, BOM/native producers, and pending-set behavior (`device_token_rotation_wire_authority_contract.rs:1520-1792`). The new HTTP case exercises the real three bound legacy/new management hooks instead of passing solely because an unimplemented hook refuses. These assertions would distinguish the privilege-confusion failure from an unrelated blanket refusal.

### Selected primary role becomes usable atomically; withdrawal remains authoritative

At `device_pairing_owner.rs:1435-1458`, DTR strictly parses the target roles array. If the selected normalized role is already represented, no rewritten roles value is supplied. If it is absent, only a matching approved primary role permits appending that selected role. Other original array values and their order are preserved; an unselected primary role is not materialized.

The roles change is performed in the same checked `UPDATE` as the selected token at `1519-1531`, followed by the existing checked revision increment and commit at `1532-1536`. It cannot become durable before token issuance or survive a failed revision/commit independently. The unchanged raw roles string is retained when selected membership already exists.

`current_pairing_record` still requires membership in `roles_json` at `1602-1603`; it does not fall back to the primary label. This is essential to preserving established withdrawal behavior. The new owner tests cover existing/missing selected entries, operator and node, reopen, role withdrawal with the primary label still present, unselected-primary non-resurrection, row/revision/commit rollback faults, and two independent managers serializing membership insertion. The new HTTP and retained tests carry the successor through the real bound hook and reconnect after an acknowledged owner shutdown/reopen, then independently remove role-array membership and require refusal while other-device/native controls remain live. These are causal oracles for both successful issuance and later withdrawal.

### Decisive SQL selection preserves strict currentness without reading an unused fallback

`current_pairing_record` now selects `COALESCE(approved_scopes_json, scopes_json)` in SQLite at `1594-1600`. The selected value is decoded by strict credential readers. A non-null malformed approval cannot fall back, and SQL NULL approval requires the fallback to be valid. A valid explicit approval can coexist with an unused malformed or differently typed legacy scopes field without making a newly issued successor unusable. DTR target baseline selection at `1489-1496` applies the same decisive rule, including an explicitly empty baseline.

The method retains issuer-instance binding, exact selected role/token matching, retirement-marker checks and constant-time fingerprint comparison. Public authorization projection fields are not promoted into proof authority. Bound management intersects sealed admitted grants, current live grants, current approval and any explicit header ceiling; it checks the entire live grant against current approval first (`1620-1664`). Retained admission additionally preserves the original admitted grant rather than silently refreshing it from a successor or a broader live row (`1550-1574`). Foreign-owner, stale-token, withdrawn-role, narrowing and malformed-store conditions therefore remain separate from target policy.

## End-to-end path and lifetime assessment

The registered `device.token.rotate` route reaches a dedicated fail-closed `GatewayApi` hook, carrying optional sealed paired proof and the native/header ceiling. The HTTP binding does not fall back to generic dispatch when a provider omits that hook. The actual App implementation forwards to its existing `self.device_pairing` owner and preserves the existing single successful-contact update. Registry and generic native dispatch include the exact method once; suffix/case aliases are not introduced.

Authentication still distinguishes actual native access from explicit invalid credentials. An invalid presented authorization is not reinterpreted as credential-absent loopback authority. A native caller has no paired self identity; its response withholds the new token. Paired non-admin targeting and nonoperator restrictions are checked before target lookup. Admin authority is not inferred from `operator.write`; whole resulting operator scopes must fit every applicable caller ceiling and target approval (`1406-1505`). Node scopes are bounded by the selected role's approval, with admin required for that management target.

The owner takes an immediate SQLite transaction before decisive caller and target reads. A selected token, any required role membership, node-only surface invalidation and the one existing revision change share the transaction. Target update count, revision presence/type/overflow/update count, and commit failure are handled as failure. The response is constructed only after commit. It contains the specified non-secret fields and includes the token only for an admitted paired caller targeting the same device (`1537-1545`). It does not revalidate the now-retired self credential after committing and thereby discard a valid response. No replay mechanism is added for a lost response.

Lifecycle metadata handling preserves unrelated/unknown fields while checking the retained creation and last-use values; a revoked selected object can be repaired through authorized issuance. New token material comes from the existing secure random generator. Only node issuance clears both node surfaces. Operator issuance preserves them. Actual App node projections preserve the paired device and unrelated/local/disconnected state rather than claiming that a token change is a physical node teardown.

I traced retained `/events`, LocalExec and WebAuthn admission through the existing currentness hook. Checks occur at the established subscription/registration/header/retry/frame/heartbeat or decoded-response boundary. The bridge subscription owns its close callback and invokes it once on drop (`gateway_server.rs:296-301`). Checks do not hold the database mutex over transport I/O. A frame or response already admitted before the commit may finish; its next boundary must refuse the predecessor. The event path checks before filtering, so an unselected/internal channel wake cannot prolong a retired proof by bypassing currentness. The ordinary 15-second receive heartbeat remains the quiet-stream retirement mechanism.

The exact-U references support these distinctions: the upstream public broker and transaction kernel bind token replacement and publication; the WS policy-response hold deliberately separates finishing the accepted mutation response from immediately fencing later authority. Browser outcome classification checks the grant and delivery/secret pair, stores in-band material into the captured gateway/device/role scope, and treats withheld cross-device delivery as success. The CLI emits the returned result. Native DTR supplies the stated wire behavior without claiming to implement the entire upstream worker/cache/WS runtime.

## Source-oracle and workflow assessment

| Evidence surface | Reviewed source behavior | Limit of conclusion |
| --- | --- | --- |
| Owner contract, 44 named tests | Full file read, including independent read-only complete SQL snapshots, exact response/metadata checks, now-current successor checks, malformed inputs/authority, self and independent-manager concurrency, fault rollback, R2 Unicode/producer/primary-role cases | Tests have not been executed by this reviewer |
| HTTP contract, 13 named tests | Full file read; real TCP Gateway and owner, default-hook refusal, native/paired positives, target nondisclosure, exact hooks/counters, stale-proof barriers, races, response-loss barrier, successor reopen and withdrawal | Socket close after durable commit proves the intended lost-response scenario in the source oracle; it is not evidence that a particular OS write error occurred |
| Retained contract, 27 required names | Shared real-owner/transport/provider fixtures and all seven DTR cases read; before/after admission barriers, true EOF, gzip, real quiet heartbeat, selected-role isolation, provider close-once, decoded-response effects, R2 reopen/withdrawal | Not all unchanged historical test bodies were deep-read; details below |
| App DTR contract, 3 named tests | Full file read; existing projection helper/owner behavior, reopen, selective surface invalidation, and source-binding assertions for the actual App hook and single contact update | These are helper/owner plus source-binding tests; they do not by themselves instantiate an entire shipping App transport |
| Existing root workflow | DTR additions and their surrounding command, subject, input-hash, parser and App selection gates read | No run/artifact acceptance was inspected or asserted |
| Extracted portable runner | Added complete name lists, command selection, exact counts, parser controls, raw capture/exit handling, subject and input custody read | Source review only; runner was not invoked |

The owner and HTTP fixtures compare independently observed SQL state, including negative cases, rather than merely checking that a production method returns an error. The retained fixture treats a timeout as failure rather than EOF, uses real provider effects for response admission, preserves surviving controls, and acknowledges server/API ownership shutdown before reopen or directory cleanup. Its barrier tests separate prior admission from queued work and require completion within the relevant production heartbeat window.

The root workflow still runs complete locked owner, HTTP and retained targets with one test thread, alongside existing DPA/DVR/DRT and integration gates (`.github/workflows/rust-desktop-runtime.yml:1748-1757`). The required name lists contain owner 44, HTTP 13, retained 27; the acceptance checks require each required pass exactly once and exact positive summaries with zero failed, ignored, measured and filtered tests. App selection retains the existing cases and adds the three explicit DTR cases, with exact selection/count checks.

Capture is separately decisive: root `command_evidence` requires exactly one zero command-exit row and exactly one zero capture-exit row (`572-582`). `run_recovery_check` preserves full raw command output, records command exit independently, then returns nonzero for capture/bookkeeping failure (`779-793`). The extracted runner likewise distinguishes cargo exit, capture failure, missing/duplicate names and summary mismatch; it binds receipts to the exact subject commit/tree, upstream identity, run/attempt and input hashes. The workflow invokes the existing extracted portable runner, rather than creating a second parallel acceptance owner. I found no deleted existing gate or new success shortcut in the declared publication delta.

## Read coverage and explicit limits

All 16 declared paths were examined through their exact candidate changes and applicable surrounding source. The full DTR specification, adjacent DPA/DVR/DRT contracts, applicable P0 device task and module-map M03/M04/M13/M32 rows, current OCRM SOT portion and current main-spec §17 were consulted. Applicable root AGENTS instructions, including review independence, immutable scope and execution policy, were read. Historical SOT sections were not treated as current authority.

Principal native source ranges inspected, with line numbers at the frozen tree:

- `source/host/src/device_pairing_owner.rs`: `1-1845` relevant helpers/production owner, plus `2079-2099` dispatch. Later unrelated setup/pairing utility bodies were not deep-read.
- `source/host/src/gateway_server.rs`: changed hook/registry/dispatch hunks, `295-380`, `1340-1475`, `1570-2065`; not the entire server file.
- `source/host/src/host_gateway_api.rs`: exact method registration/routing changes and surrounding registry context.
- `source/host/app/src/main.rs`: changed routing/hook hunks, actual owner/currentness/native management hooks at `13293-13425`, node projection consumers at `2706-2829`, owner/startup references located; not the whole App file.
- `source/host/tests/device_token_rotation_wire_authority_contract.rs`: full `1-1792`.
- `source/host/tests/device_token_rotation_wire_gateway_contract.rs`: full `1-1244`.
- `source/host/tests/device_pairing_retained_stream_contract.rs`: `1-1105`, all DTR additions at `1705-2181`, and full changed helper hunks. Unchanged `1106-1704` was not deep-read in full; retained requirements, shared fixture behavior, actual transport consumer source, required names and unchanged diff status supply the adjacent context.
- `source/host/app/src/tests/device_token_rotation_wire_tests.rs`: full `1-173`; existing `device_pairing_revocation_tests.rs` helper/projection context and complete visibility-only delta. `gateway_protocol_contract.rs`: complete changed registry oracle.
- `.github/workflows/rust-desktop-runtime.yml`: complete declared changes plus subject/input/capture and DTR/App gate ranges, particularly `1-105`, `271-305`, `550-582`, `750-801`, `1235-1250`, `1670-1770`, `2499-2664`, `3674-3684`, `4180-4250`. This is not a full semantic review of every unrelated workflow job.
- `projects/openclaw-rust/scripts/capture_portable_contracts.py`: relevant identity/input ranges `1-120`, complete added names around `491-580`, command selection around `790-800`, parser/control/capture/final-acceptance ranges `954-1072`, and full declared change.
- DTR spec: full `1-307`. Source-evidence JSON: identity, scope/depth and acceptance-limit sections plus manifest context; not every historical source-record paragraph independently verified. Parity ledger and RTM: complete publication changes and surrounding affected row/status context. The unrelated incoming media evidence archive was not reviewed.

The supplied upstream cache has 47 records; **47 acquired identities is not a claim that this reviewer fully read 47 files**. This review deeply read the selected load-bearing contracts/consumers:

- `src/gateway/server-methods/devices.ts:1-290`, `device-management-authz.ts:1-70`, `validation.ts:1-94`.
- `src/infra/device-pairing-tokens.kernel.ts:1-248`, `device-pairing-identity.ts:1-196`, `device-pairing-token-policy.ts:1-140`, `device-pairing-token-utils.ts:1-62`, `device-pairing-tokens.ts:1-161`, `device-pairing-dispatch.worker.ts:1-101`, `device-pairing-store.ts:1-390`.
- `src/shared/device-auth.ts:1-34`, `src/shared/operator-scope-compat.ts:1-119`; rotation method descriptor row/context around `core-descriptors.ts:260-285`.
- Rotation request/result schema at `packages/gateway-protocol/src/schema/devices.ts:1-75`, closed-object helper `1-11`, non-empty primitive `15-35`; validator binding at `validators/workers.ts:1-12,190-210`, `protocol-validator.ts:1-65`, generated worker reexports `1-55`.
- Generated predicates were a **targeted dependency read only**: `1596-1602`, `1809-1852`, `8338-8352`, `19152-19164`, `22918-22922`, `23088-23092`. No other generated predicates or generator execution receive credit.
- `src/gateway/server/ws-policy-close.ts:1-146`; browser rotation/classification/cache consumers `ui/src/lib/nodes/page-operations.ts:340-455`, `ui/src/lib/nodes/index.ts:1-230`, dialog imports and `reportRotationOutcome` around `180-215`; CLI token-command consumer `src/cli/devices-cli.runtime.ts:919-940`.

Acquisition metadata, historical read-depth strings and generated-source availability were not promoted to new semantic-review credit. Upstream issuer-generation policy, worker publication/cache invalidation, broader WS and external node/device acceptance remain the explicit boundaries recorded by the candidate specification and evidence. This report supports convergence of this frozen source review only; the normal exact-subject execution and acceptance owners retain their gates.
