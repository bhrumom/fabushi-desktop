# OCRM Round 78 — ForeverBox / Host health and first-party replacement-owner audit

Task: 3dd43a37-9f36-4b7a-adac-dfcc87584523. Date: 2026-10-24. Evidence-only additive dossier. This is not an implementation, test receipt, G01/G02 admission, independent ACCEPT, or release acceptance.

## Authority

- Frozen subject remains bhrumom/fabushi-desktop PR #169, OPEN/unmerged, exact HEAD `b5db35fa272c83833685d70a730b5f8285fa1176`, tree `d01637ba8d83cd6c723077555ccf725887f7156e`.
- Audit carrier parent lease for this dossier is `a44060be53dd8c9d1bd1efc1ddee0177ab5d7987`, tree `2e9365e5936c5f379a2ef5b1a7631afb655985df`.
- No source rebaseline is made. Live Desktop main moved independently and is cited only as current product-direction evidence.
- No GitHub Actions, build, lint, test, generator, package, deploy, pairing mutation, production-code mutation, ledger/RTM mutation, or completion-flag mutation was performed for this audit.

## R78-BOX-REMOTE-RECREATE-NOT-DAEMON-OWNER

Round77 established that the managed local Box exec child has no post-readiness child-exit watcher/respawn path while Host remains alive. This round separately checked the plausible ForeverBox / BoxLifecycle replacement route and found it is a different lifecycle boundary.

At frozen HEAD:
- `source/host/src/extensions/forever_box/forever_box_service.rs` blob `2c549d54d607b567289a21e5c5a370f834bdbe37` owns the shared ForeverBox facade and exposes ensure/reset/update style operations over the existing HostBox/lifecycle boundary.
- `source/host/src/extensions/forever_box/host_box.rs` blob `c841d53cc412e8813ce1904cfb90e329fff9dd30` wraps the existing LoopbackSandBox/accessor path.
- `source/host/src/extensions/box_lifecycle/box_lifecycle_service.rs` and `production.rs` were read at the frozen HEAD. Their recreate operation constructs the remote `RecreateSandBoxRequest { preserve_data, force }` and calls the production Box lifecycle RPC. This path has no local `Child`, daemon spawn, or `OwnedBoxExecDaemon` authority.
- `source/host/tests/extensions_box_lifecycle_contract.rs` blob `dd46a1188776b8dc6d7cf2feda2ed0341eb4a0cb` validates the lifecycle service/extension contract around remote recreate.
- `source/host/tests/extensions_box_lifecycle_production_contract.rs` blob `a1a47caffba2b1acd55997571c3bf863bf6e8889` validates the production Dashboard/GetUserPrivacyMode, GrokBot/GetSandBoxRunState and RecreateSandBox RPC paths. It does not test local managed-daemon replacement.

Therefore ForeverBox reset/update/recreate is a remote sandbox/pod lifecycle path, not a hidden legal replacement owner for the local managed Box exec daemon.

## R78-BOX-ACCESSOR-RELOAD-NOT-REPLACEMENT

The principal production consumers checked this round all consume the same ForeverBox/HostBox authority but do not own the local daemon child lifecycle:

- `source/host/src/host_production_extensions.rs` blob `6294645234707fef8472aff85737c2550252fd0b` is the central production composition for the Box-stage extensions.
- `source/host/tests/host_production_extensions_contract.rs` blob `cfdd72dfc5b4e2960c2da5a443826937f922512b` contains the focused assertion that the Box-stage extensions have one production composition owner.
- `source/host/src/extensions/attachments/extension.rs` blob `fcb0696d8e3559e942411ac594ed5c80ebff096f` injects `Arc<ForeverBoxService>`; `attachments_service.rs` blob `783a78e21ad4c87db1930cadf3ba46dd6bbb806e` implements its Box port through `ForeverBoxService`, reading the current run state and uploading through the existing box.
- `source/host/src/extensions/secrets/extension.rs` blob `f18a1402bc7c1873ff54b8c725bdfdea284280c2` injects the same service and applies environment changes through `apply_environment`; its retry ownership is secrets-application retry, not daemon process spawning.
- `source/host/src/extensions/teach_recording/teach_recording_service.rs` blob `021df34c0d47f61a863959429a10cc614e70817a` holds `Arc<ForeverBoxService>` and calls `ensure` / `ensure_ready` before its shell/recording operations.
- `source/host/src/extensions/forever_box/runner_tools.rs` blob `b72846bba116968d5e1a557d7a7fdcd8897131a5` re-resolves accessors and calls readiness after box recreation where required; that is accessor recovery after the remote lifecycle change, not ownership of a local child respawn.

No checked consumer acquires the local `OwnedBoxExecDaemon`/child spawn authority. The audit does not claim that every one of the 585 Host Rust blobs was individually fetched in this round: an attempted exhaustive connector scan hit the connector's per-call limit, so this dossier intentionally limits its conclusion to the production composition and concrete consumers/read paths named above rather than inventing an exhaustive scan result.

## R78-BOX-HEALTH-BLIND

At frozen HEAD, health/status surfaces do not make managed-daemon death a Host-unhealthy condition:

- `source/host/src/box/loopback_sand_box.rs` blob `ac2fcd557f09eb3c5fce9e8d22001dd6600266d3` returns literal `true` from `is_available()` and literal `"running"` from `run_state()`. Its post-readiness watchdog Pings the daemon and reports telemetry/readiness episodes; it does not own the child process or respawn it.
- `source/host/src/gateway_server.rs` blob `1ce506ae0bc75ee96418e55eb64a935dd7865ac0` serves `GET /health` with `ok:true`, pid, busy state, active agent, start time and last-busy metadata. The health contract has no Box/daemon-readiness field.
- `source/host/tests/gateway_server_contract.rs` blob `63972a0123e2f51288d0615993c0795b155a770b` validates Host HTTP health/security and busy metadata, not managed Box child liveness.
- `source/node-agent-coordinator/src/gateway/host_supervisor.rs` blob `ea7d370c0a79948e7e6a0a179405d3aacd9e3a56` uses Host `/health` to choose UseCached/Probe/Reconnect for the gateway endpoint.
- `source/node-agent-coordinator/tests/host_supervisor_health.rs` blob `31d2ca7fb5aeb833f56a71f93072b798a1fddfd7` validates 200 health, 503 and transport failure leading to gateway reconnect. It has no post-readiness managed-child-death case.
- `source/node-agent-coordinator/src/main.rs` blob `c822c07dff3027a22a658d1ece6aefe919691365` has a real Host-process waiter which invalidates the gateway and can respawn Host after the Host process itself exits. That path indirectly reconstructs Box as part of a fresh Host, but it is not a daemon-only recovery path while the Host survives.

Combined with Round77, no newly discovered legal production path in the inspected owners/consumers replaces or reinitializes the managed local daemon after a post-readiness child crash while the Host process remains alive. The missing falsifier remains: kill the managed daemon after readiness while keeping Host alive and prove the approved owner either replaces the daemon or transitions Host/gateway health into a supervised Host restart without duplicate mutation.

## Admission remains fail-closed

Fresh reads of the audit branch still do not establish scoped admission:
- `projects/openclaw-rust/dossiers/gateway-ingress-scoped.md` keeps scoped G01/G02 prerequisites fail-closed.
- `projects/openclaw-rust/reviews/2026-10-22-spec-review.md` still records the absence of positive scoped G01/G02 and independent owner/oracle acceptance.
- `projects/openclaw-rust/parity-ledger.json` and `requirements-traceability-matrix.json` still have all completion flags false; the ledger carries no accepted responsibility set that can substitute for an independent owner/oracle ACCEPT.

Thus no Actions lane or Box/gateway production change is admitted by this dossier.

## Sand historical receipt and current Fabushi replacement owner

The historical immutable deployment question remains open:
- `bhrumom/fabushi-backend` live main advanced during this audit and was last read at `14c3e08b2387b3546c2d8ed6550ee371eea5ab3b`, tree `59e382bb1a065e7914eff7bb53ef7361782bd0ba`.
- Current-tree GitHub code searches did not locate `/sand/automation-runs/complete`, an automation-runs completion route, or a Sand deployment receipt.
- Historical backend commit search returned automation-related commits, but the inspected candidates were unrelated registration/version-policy automation or a large consolidation whose diff had no `/sand/automation-runs/complete`, `automation-runs`, `sandAgentId`, or `ackRunUuids` match. No immutable historical deployed Sand completion SHA/owner/receipt was recovered.

Current Fabushi product ownership has, however, moved in a first-party direction on live Desktop main; this is drift/current-product evidence, not evidence for the frozen PR subject:
- `ea0689be9ec66e92591c33d57801eb149db27749` (#171) keeps the existing Host `AutomationFireBackendRuntime` as the client owner and removes the local behavior that promoted completion 404/409 errors to Reported. The current success transition occurs only after an actually successful completion POST.
- Desktop main later advanced to `fb1189efd9d85c650864d6987ea8fb68c913558d`, tree `1a1ef09be4c9fd801354439a5b82cc460f0b4b6a` (#172). Its reviewed diff removes the foreign Cursor product-backend fallback, selects approved Fabushi first-party service origins/credentials, and adds an account-bound native automation run-journal transport under the existing Host/service composition.

These changes identify the current Fabushi replacement direction as existing Host Automations plus Fabushi first-party service ownership. They do not manufacture the missing historical Sand deployment receipt, do not prove complete cloud rollout, and do not alter PR #169 admission status.

The existing Gmail Sand support thread was re-read: it has three messages, all SENT by this account and no incoming reply. Its latest sent update already records the product direction away from Cursor/Sand proprietary dependency, so no duplicate notification was sent this round.

## PostgreSQL 14 published-archive provenance

Frozen OpenClaw `pnpm-lock.yaml` blob `80fb4d9fa985297a24ac88931ef534e85a40ba47` was re-read for the fixed fourteen-package PostgreSQL slice:
`pg@8.23.1`, `pg-cloudflare@1.4.1`, `pg-connection-string@2.14.1`, `pg-protocol@1.16.1`, `pg-pool@3.14.0`, `pg-int8@1.0.1`, `pg-types@2.2.0`, `pgpass@1.0.5`, `postgres-array@2.0.0`, `postgres-bytea@1.0.1`, `postgres-date@1.0.7`, `postgres-interval@1.2.0`, `split2@4.2.0`, and `@types/pg@8.23.1`.

The exact lockfile SRI declarations remain present, but they are declarations in the frozen source lock and are not an independent download/rehash of each official published tarball. The official Registry read interface available to this session still did not yield usable exact-version archive metadata/bytes. Therefore the independently verified counts remain:
- official published tarball bytes + independently recomputed SHA512: 0/14;
- archived LICENSE verified from each published tarball: 0/14;
- archived NOTICE verified from each published tarball: 0/14.

The existing PostgreSQL support thread was re-read and still has one SENT message and no reply; no duplicate notification was sent.

## Independent-worker status

A real three-worker independent read-only audit was attempted this round for Box lifecycle, admission, and Sand/PostgreSQL provenance. The worker service returned `Tunnel-client has not been seen for 300 seconds`, so no worker actually started and no independent verdict is claimed. The existing tunnel-client support thread remains SENT with no reply; no duplicate email was sent.

## Disposition

This dossier adds evidence only. No Actions were triggered and no protected production/pairing/ledger/RTM/flag files were changed. Scoped G01/G02, independent owner/oracle ACCEPT, an approved managed-daemon recovery contract, historical Sand deployment receipt, all fourteen published-package archive provenance records, full Rust/native equivalence, OCRM-AC01..AC24, and final release acceptance remain OPEN.
