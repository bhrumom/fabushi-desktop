# OCRM Round77 — managed Box daemon death has no recovery owner and command transport-down cannot heal it

Task `3dd43a37-9f36-4b7a-adac-dfcc87584523`; user report round 70; 2026-10-24. Exact-source **read-only** evidence only. No GitHub Actions, build, test, generator, package, deployment, pairing mutation, production patch, ledger/RTM mutation, scoped G01/G02 approval or independent ACCEPT is claimed.

## Frozen subject, carrier and admission boundary

- Subject remains `bhrumom/fabushi-desktop` PR #169, open/unmerged at exact HEAD `b5db35fa272c83833685d70a730b5f8285fa1176`, tree `d01637ba8d83cd6c723077555ccf725887f7156e`. This audit does not modify that PR.
- Audit carrier lease before this dossier is `3b318c671dcb4894d4c100719b9e68bc9d6033eb`, tree `7d4c4fed7d3253bbb21ef0fa04be59ccd363add8`. Live Desktop main has independently advanced to `ea0689be9ec66e92591c33d57801eb149db27749` through merged PR #171; that main identity is not substituted for the fixed PR #169 subject.
- `projects/openclaw-rust/management/tasks/P0-gateway-rpc-authority-and-auth.md` blob `154bdea00d3a991bd9cb1c015fb621ffd1179824` still says **NOT READY**, scoped G01/G02 not run and existing Host/Box owners only proposed. `P0-source-baseline-and-owner-audit.md` blob `9757e8cc1c8968c10f50f240d799854ec47357d5` remains not executed; the current ledger has no accepted responsibilities and the RTM rows remain not-started. No independent Box owner/oracle admission was found.
- Therefore this round remains source-only. The production-fix gate requested by the user is not open.

## R77-BOX-DAEMON-DEATH-NO-RECOVERY-51 — new non-duplicate lifecycle evidence

### 1. The default managed child is spawned once and monitored only until initial readiness

`source/host/src/box/exec_daemon_process.rs` blob `5cf8ed0995ea57e7532ed0d08dd339e0edaa9975` selects managed mode unless `SAND_STANDALONE_BOX_EXEC_DAEMON` is set (L73-75). `OwnedBoxExecDaemon` owns the `Child` and parent-lifetime stdin pipe (L95-104). Startup spawns the process (L200-237) and polls `child.try_wait()` only inside the **pre-readiness** loop (L246-275). Once Ping succeeds it returns the child inside `OwnedBoxExecDaemon` (L287-295). The remaining child-status helpers are shutdown/termination helpers (L332 onward); no post-readiness watcher or respawn loop is established in this owner. The only background readers created by startup consume stdout/stderr (L298-309).

The production Host calls `start_managed_box_exec_daemon_from_process_env` once before production extensions in `source/host/app/src/main.rs` blob `8e19361b3b278d085927a84ea4c8ce8c3fe3de80` L11446-11454. It closes the retained daemon during Host shutdown at L13428-13432. No second production spawn site was observed in this inspected lifecycle path.

### 2. The Loopback watchdog detects loss but cannot replace the owned process

`source/host/src/box/loopback_sand_box.rs` blob `ac2fcd557f09eb3c5fce9e8d22001dd6600266d3` reports `is_available() == true` unconditionally (L181-183). Foreground readiness repeatedly Pings the endpoint and eventually returns `SandBoxDaemonUnreachableError` after the configured timeout (L206-357). `start_daemon_watchdog` starts only a telemetry/readiness thread (L360-389). Its loop Pings the endpoint and updates `DaemonWatchdogEpisode` / emits `DaemonPingReport` (L584-655); it has no `OwnedBoxExecDaemon`, `Child`, spawn call or restart authority.

Thus an owned daemon that exits **after** initial readiness can leave the Host process alive while Box readiness remains unavailable. The watchdog can observe that state but cannot heal it.

### 3. The dead Box dependency is converted to Host HTTP 500, reusing the Round76 error bridge

This dossier does not re-count Round76's application-error taxonomy defect. It reuses that established bridge for a different trigger: a real managed-child outage. In `source/host/app/src/main.rs` L9275-9307, Box MCP load/accessor/raw-exec errors are converted to `GatewayCommandError::Internal(error.to_string())`. `source/host/src/gateway_command_error.rs` blob `a7827b29c7f53899b1f34e7c989849816f4090fb` maps `Internal` to HTTP 500. The string conversion means any original `SandBoxDaemonUnreachableError` type is no longer available at the HTTP command boundary.

Coordinator `source/node-agent-coordinator/src/gateway/gateway_request_dispatcher.rs` blob `38ca19db76493f9c2bca8862b484d61b7e74f47c` classifies every HTTP >=500 response as `GatewayDispatchError::Unreachable(Http(status))`. `source/node-agent-coordinator/src/main.rs` blob `c822c07dff3027a22a658d1ece6aefe919691365` then calls `gateway.transport_down` for that command failure (L2669-2672 on the production dispatch worker). This is a Host-gateway state mutation caused by a downstream Box-child outage while the Host can still be alive.

### 4. Command-level transport_down does not restart the dead Box and does not necessarily reconnect SSE

The reconnect consumers refine, rather than duplicate, Round76:

- `GatewayClientState::disconnected` in `source/node-agent-coordinator/src/gateway/gateway_client.rs` blob `67c666d4b3ec716aca60763bb53c594f066d0660` L191-204 sets `connected=false` and `stream_live=false`, records the outcome and computes retry delay for retryable reachability outcomes. It retains the cached `GatewayConnection`.
- `wait_for_gateway_connection` in Coordinator `main.rs` L1597-1643 does **not** require that connected flag. It returns the current Host supervisor connection (L1608-1615), then the cached gateway connection (L1616-1623), or re-reads Host discovery (L1624-1635). Therefore the next command can still target the same live Host even after the Box-triggered command marked gateway state down.
- `run_gateway_event_stream` L1255-1370 keeps the existing SSE loop while the Coordinator is open, Host generation is unchanged and transport is not explicitly suppressed. Its continuation predicate does not consult `GatewayClientState.connected` (L1304-1312). Actual SSE rediscovery/reconnect happens only after the event stream itself returns an error/end (L1315-1368). A command's `transport_down` does not itself close or replace that stream.
- If the Host process itself exits, a separate Host child waiter does clear the generation, mark transport down and respawn the Host with bounded crash backoff (L1540-1585). That Host restart would re-enter Host startup and can recreate its managed Box child. A Box-only child death does not trigger this Host-process waiter.

The inspected default path therefore has two independent problems after a post-readiness Box-child death: (a) no owner respawns the Box child while Host remains alive; (b) the failed Box command temporarily projects the **Host gateway** as disconnected even though cached command routing and the existing SSE socket can still be live. Neither cached Host rediscovery nor SSE reconnect has authority to recreate the owned Box process.

## Existing tests and falsifiable oracle boundary

Selected process coverage does not close this lifecycle:
- `source/host/tests/box_exec_daemon_process_contract.rs` blob `06506be3ce37c43ff02dff1da820859cab86eb3b` verifies sibling resolution, contaminated-port rejection, and—when an exact daemon binary is supplied—spawn/authenticated Ping/explicit close with port release. It does not kill the daemon **after readiness while Host remains alive** and assert recovery.
- Round76 already recorded that selected gateway policy/health tests cover genuine 503/health reconnect, not Box application-error preservation. This dossier adds the missing managed-child death/recovery owner, not another copy of that finding.

**Unrun R77-BOX-LIFETIME-77A** — only after scoped G01/G02 and an independent Box owner/oracle admission: launch the exact staged managed daemon under the production Host, establish healthy SSE and a benign Host command, then terminate the managed daemon after readiness without killing Host. Assert the approved Box owner detects the exact child generation/PID death, exposes a typed Box dependency/lifecycle failure without declaring the Host gateway down, and follows the independently approved bounded recovery contract. If recovery is restart-based, prove a fresh generation/PID, contaminated-port protection, crash backoff/circuit behavior, no stale-child resurrection and the required MCP/environment/state rehydration semantics. A later benign Host command/SSE event must remain transport-live; a true Host crash must still exercise the existing Host restart path. No mock-only provider may substitute for the staged managed process where the gate requires it.

## External provenance checks remain unresolved

### Sand

Read-only GitHub recheck keeps `bhrumom/fabushi-backend` main at `069f69d903e2e358714796fe465a39e7e0468ca3` / tree `719d7adffd7bd8615cb6c073938d72e0de5ceb44`. Repository searches did not recover an immutable deployed SHA/owner/receipt for `/sand/automation-runs/complete`. Desktop main's merged PR #171 is a client ACK-safety correction and explicitly does not prove a deployed Sand service identity. The existing support thread has no external reply; later self-authored project direction changes do not supply the historical deployment receipt. No repeat support email was sent.

### PostgreSQL 14-package slice

The locked 14-package slice remains the exact set recorded in Round33 blob `f530e67d757816aa1082988898f0533c9d40f914`: `pg@8.23.1`, `pg-cloudflare@1.4.1`, `pg-connection-string@2.14.1`, `pg-protocol@1.16.1`, `pg-pool@3.14.0`, `pg-int8@1.0.1`, `pg-types@2.2.0`, `pgpass@1.0.5`, `postgres-array@2.0.0`, `postgres-bytea@1.0.1`, `postgres-date@1.0.7`, `postgres-interval@1.2.0`, `split2@4.2.0`, `@types/pg@8.23.1`. Their lockfile SRI values remain declarations, not an independent rehash of official tarball bytes. The public Registry JSON path remains inaccessible in the available reader and no authoritative archive bytes were acquired, so independent official tarball SHA512 plus archived LICENSE/NOTICE remains **0/14 verified**, not “0 missing”. The existing support thread has no reply; no duplicate email was sent.

### Independent reviewer tooling

The requested real subagent attempt failed again with `Tunnel-client has not been seen for 300 seconds`. An existing unresolved support thread already covers that same connection issue and has no reply; no duplicate notification was sent. No independent worker/reviewer result is invented.

## Distinctness and verdict

Round74 blob `70fe22cd345eff8316990e51dcb31f2474c4622f` records managed `LoadMcpServers` no-op success. Round75 blob `fda076293e430ab9d2558dfd90e25e00799a2783` records missing MCP raw Exec oneof dispatch. Round76 blob `01b64eae2abb9c51d0db559cf45f5dcc66d451f9` records typed application-error loss and Host-gateway reachability misclassification. Round62 blob `9e9730c823a429be0e14ecc37ec9754a05c58ccc` records Coordinator/Host exit fencing. None of those establishes the **post-readiness managed Box child death with Host surviving, the absence of a Box restart owner, and the fact that command-level gateway state-down neither closes SSE nor restores the Box child**.

Scoped G01/G02, independent owner/oracle admission, full source/dependency closure, production Rust parity, OCRM-AC01..24, release evidence and independent final ACCEPT remain pending. No Actions or production mutation is authorized from this dossier alone.
