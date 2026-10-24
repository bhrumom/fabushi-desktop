# OCRM Round79 — BoxStore / GatewayLocalExec daemon-writer census and admission refresh — 2026-10-24

Task `3dd43a37-9f36-4b7a-adac-dfcc87584523`, report round 72. Evidence-only additive dossier. No production Rust/pairing mutation, ledger/RTM/flag edit, GitHub Actions run, build, test, generator, package or deployment is claimed.

## Frozen authority and carrier lease

- PR #169 exact subject remains `b5db35fa272c83833685d70a730b5f8285fa1176`, open, base `main`, exact Host Rust tree inventory obtained from the Git tree without truncation.
- Audit branch `ocrm/round20-snapshot-lifecycle-audit-20261023` was re-read immediately before this write at lease `c86ed6a1de5250efd44bee253bd0c140469065bc`.
- This dossier is a new evidence carrier only. It cannot inherit historical Actions results as an acceptance verdict.

## R79-BOX-STORE-SPAWNS-NOT-DAEMON-WRITERS

`source/host/src/extensions/box_store_sync/production.rs@d37ec70db1d2690ef63ec2081e7ef17af723be57` was re-read at the frozen PR subject.

The only relevant production `.spawn(...)` sites in the inspected lifecycle section are:
- around L370-L382: Rust thread `box-store-db-debounce` running `run_store_db_debounce_loop`;
- around L387-L430: Rust thread `box-store-sync-poller` running periodic object-store/snapshot synchronization.

The service owns `JoinHandle` fields for the Chrome watcher, poller and store-db worker. It does not import or hold `OwnedBoxExecDaemon`, `Child`, `ProductionBoxTransport`, `start_managed_box_exec_daemon_from_process_env`, or a Box exec-daemon process identity. These spawns are synchronization worker threads, not a hidden managed Box child writer/recovery owner.

Therefore BoxStore synchronization cannot legally replace or reinitialize a dead managed Box exec daemon merely because it has background threads.

## R79-GATEWAY-LOCAL-EXEC-NOT-BOX-DAEMON-WRITER

`source/host/src/extensions/local_exec/gateway_local_exec_sand_box.rs@96528614bc745964e7599ebcf8e5b9a1b1d27bec` was read at the frozen PR subject. Exact string inspection found no `start_managed_box_exec_daemon`, `OwnedBoxExecDaemon`, `ProductionBoxTransport`, `Child`, daemon `spawn`, or `recreate` call.

Its `GatewayLocalExecSandBox` owns a `SandLocalExecBridge`, permission gate, optional computer id, file-size limit and failure reporter. `remote_resource_accessor()` builds a typed `RemoteResourceAccessor<GatewayLocalExecManager>` through `PRODUCTION_LOCAL_EXEC_CODEC`; it does not own an OS child.

`source/host/src/extensions/local_exec/production.rs@8a5ca2dae2760db4e093c0cf3fb7630053814bda` confirms that this `RemoteResourceAccessor` is only an `Arc<M>` manager wrapper and protobuf/JSON codec surface. `source/host/src/extensions/local_exec/extension.rs@241bd7b1a6078a204690afec848c442aba77c2f9` composes the bridge/provider channel and telemetry. Neither file owns Box process creation or replacement.

This closes a previously plausible but unverified candidate: Host GatewayLocalExec is not a second managed Box daemon writer.

## R79-TRUE-LOCAL-BOX-CHILD-OWNER-RECONFIRMED

`source/host/src/box/exec_daemon_process.rs@5cf8ed0995ea57e7532ed0d08dd339e0edaa9975` remains the local Box child process owner:
- `OwnedBoxExecDaemon` contains `Option<Child>` plus the parent-lifetime stdin pipe;
- `start_managed_box_exec_daemon_from_process_env` delegates to `start_box_exec_daemon_process`;
- startup executes `Command::new(...).spawn()`, captures PID, verifies authenticated generated Ping through `ProductionBoxTransport`, then returns the owned child;
- after readiness, this file still exposes shutdown/termination helpers and stdout/stderr reader threads, not a respawn watcher.

`source/host/app/src/main.rs@8e19361b3b278d085927a84ea4c8ce8c3fe3de80` shows the production Host composition calling `start_managed_box_exec_daemon_from_process_env(&app_data_dir)` before starting production Host extensions (L11446 vicinity), retaining the returned optional owner for the Host lifetime, and invoking `daemon.close()` during shutdown (L13428 vicinity). No second spawn/recreate call appears in those production startup/shutdown owner ranges.

`source/host/tests/box_exec_daemon_process_contract.rs@06506be3ce37c43ff02dff1da820859cab86eb3b` exercises sibling binary resolution, contaminated-port refusal, and real-daemon spawn/authenticated Ping/explicit close when a test binary is supplied. It still lacks post-readiness child death with surviving Host and recovery/replacement assertions.

## R79-SEPARATE-COORDINATOR-LOCAL-EXEC-SUPERVISOR-IS-DIFFERENT-DAEMON

A second process supervisor does exist outside the Host Box owner, but it is a different product boundary and cannot be counted as Box daemon recovery:

`source/node-agent-coordinator/src/local_exec/supervisor.rs@e84ef80f44a532bbbe640281d12aa0cfdd18437e` defines `LocalExecDaemonAction::{Spawn,Adopt,Replace}`, a separate `LocalExecSupervisor`, `LOCAL_EXEC_DAEMON_RESPAWN_LIMIT`, generation tokens, process identity and a `LocalExecControl::spawn_local_exec_daemon` port.

That identity is explicitly the Coordinator local-exec provider daemon, with its own discovery/generation protocol. It does not hold the Host `OwnedBoxExecDaemon`, Box endpoint/auth token, or Box `ProductionBoxTransport`. Its existence is therefore not evidence of a legal replacement writer for the Host-managed Box exec daemon.

This distinction prevents a false-positive owner election from the generic words “local exec daemon”, “spawn”, “replace”, or “respawn”.

## R79-PRODUCTION-BOX-TRANSPORT-IS-CLIENT-SURFACE-NOT-CHILD-OWNER

`source/host/src/box/generated_production.rs@81f5a74dbf0ebc3700f177639bdd875f8c00eb56` defines `ProductionBoxTransport` and generated request/response transport behavior. Its “spawn” occurrences include protocol concepts such as background shell spawn and are not OS `Child` creation.

`source/host/src/box/production.rs@96aa8c1ccf7ca2c0f8a5085a11279d29c113706e` constructs `ProductionBoxTransport::from_endpoint` and the loopback/shared-desktop composition, then exposes readiness/resource/environment/file operations. It contains no managed child owner or replacement lifecycle.

Thus transport/client creation and shell-spawn RPC vocabulary must not be misclassified as daemon process ownership.

## Scoped G01/G02 and independent owner-oracle verdict refresh

Fresh audit-branch reads remain fail-closed:

- `projects/openclaw-rust/dossiers/gateway-ingress-scoped.md@783fc5da574b13b3e4229d1c08552a63c2be6387` still states `unit_authority_ready=false`, scoped G01 unavailable/not passed, scoped G02 independent verdict unavailable and §11.1 NOT ADMITTED.
- `projects/openclaw-rust/management/tasks/P0-gateway-rpc-authority-and-auth.md@154bdea00d3a991bd9cb1c015fb621ffd1179824` still says NOT READY and scoped G01/G02 not run.
- `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md@9757e8cc1c8968c10f50f240d799854ec47357d5` still has its exit/acceptance conditions pending rather than positive unit admission.
- GitHub PR #169 currently has zero review submissions and zero review threads. No remote reviewer verdict exists that can be construed as an independent Box owner/oracle ACCEPT.

Therefore no Actions lane, pairing mutation, Box/gateway production change, ledger/RTM acceptance upgrade, or completion flag update is authorized by this round.

## Exhaustiveness boundary and connector limitation

The exact PR tree reports 584 Rust blobs under `source/host/src/`. A prior attempt to fetch a large batch through the GitHub connector hit the connector's maximum-per-Code-Mode-call limit; this round did not fabricate an all-584 full-text scan. Instead, it closed the two user-named candidate files, the concrete child owner, Host composition, generated transport, Box production facade, focused process contract, and the separate Coordinator local-exec supervisor.

Accordingly the strong claim is limited to the exact files and owner paths above: no newly inspected candidate supplies a second legal managed Box child writer. A repository-wide every-blob lexical census remains an open read-only evidence task and must be done in connector-safe batches or another approved read-only mechanism without local build/test execution.

## Distinctness and current verdict

Round77 established post-readiness Box-child death with no observed recovery and the Host-gateway misclassification consequence. Round78 excluded ForeverBox/remote RecreateSandBox, major consumers and Host health as hidden replacement owners. This Round79 newly excludes BoxStore worker-thread spawns and GatewayLocalExec/codec/provider surfaces, and separately identifies the Coordinator local-exec respawn supervisor as a different daemon authority rather than a Box recovery owner.

Current verdict remains **NOT ACCEPTED / NOT ADMITTED**. The approved managed-daemon recovery contract, scoped G01/G02 receipt, independent owner/oracle ACCEPT, typed domain-error/application-vs-transport fix, full recursive source closure, OCRM-AC01..24, release evidence and final independent ACCEPT are still open.
