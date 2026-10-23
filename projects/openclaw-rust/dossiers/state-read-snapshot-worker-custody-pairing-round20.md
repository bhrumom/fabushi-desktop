# OCRM round 20 — state reader, snapshot, worker custody and pairing owner audit

Status: SOURCE RESEARCH ONLY. No scoped G01/G02 approval, independent ACCEPT, production Rust implementation, or verified npm tarball. Nothing here promotes any OCRM flag.

## Fixed subject and actual repository lease
- Frozen OpenClaw `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.
- Fabushi PR #169 remains open/unmerged, head `b5db35fa272c83833685d70a730b5f8285fa1176`, tree `d01637ba8d83cd6c723077555ccf725887f7156e`, branch `ocrm/p0-source-authority-20261022`; PR base SHA `f6eb1758c59d65aba93e5608d7aba445e0bad42b`.
- Live target main was `7394f6dc7687f9b986ee5589d40aa680ce23b90a`, tree `fbeb0bb32d944d0c1053880c471fe5c52279f052` — not silently rebaselined.
- Previous separate audit commit `3c6472b8fda83ce17dbbf6c5d4b3b8be37562f93`, tree `506ae9069cbcbe29f20e1854f5e2e68dbc59bf58` remains independent from PR #169. Current round-20 document is another audit carrier, not a scoped implementation subject.
- Governance read at candidate: `AGENTS.md@e6a1f21ba5206cdc807b6f9738dad2cec2942100`; `projects/openclaw-rust/SOURCE_OF_TRUTH.md@4b9bdfe745da026ca7ebbca9ea82127d42017f0f`; main Spec `docs/specs/openclaw-rust-equivalence-migration.md@52a0202af8e1ea10e42f8ff8f5913e8ec93a4776` §§11.1/12/15.1; `module-map.md@25e4e8258ac45cf72eea704fdceedd0820622a51`; P0 task `P0-source-baseline-and-owner-audit.md@9757e8cc1c8968c10f50f240d799854ec47357d5`.

## Frozen upstream files fully inspected in GitHub
Paths below are relative to the pinned OpenClaw commit. Each entry names exact blob SHA; `full` refers only to that individual file, not its recursive dependency closure.
- `src/state/openclaw-state-read-pool.ts@1ff025735fb0804bc4152e48e9beca69e9c7fc4d` L1–68/68.
- `src/infra/sqlite-snapshot-source.ts@14931e709dbccb2daab535c15201dfc671994904` L1–230 and L231–460/460.
- `packages/worker-runtime/src/lifecycle.ts@2cfdd23befd86661d251d2b55d6299c3d08b8094` L1–14/14.
- `packages/worker-runtime/src/retained-operation.ts@16cb02bbc296f8dd1b7fe308cfdcaf72c168461d` L1–177/177.
- `packages/worker-runtime/src/worker-lifecycle.ts@2bfb04a02c8cd52c4c3228595f95405d4a073a07` L1–37/37.
- `packages/worker-runtime/src/worker-task-pool-core.ts@f73ca6fbfb5cf54a9c69143cce14d02fae8c39f9` L1–210,211–410,411–610,611–750/750.
- `packages/worker-runtime/src/worker-task-pool-owned.ts@88f6b75cbde72daa17147afc2160e4b56c0d93f2` L1–155,156–320/320.
- `packages/worker-runtime/src/worker-task-pool-completion.ts@131c781f9bc972bf95edcb12e969e6f9d3246580` L1–154/154.
- `packages/worker-runtime/src/worker-task-pool-resources.ts@e8abbdf4173bd08c59f1a0831560402706463b52` L1–209/209.
- `packages/worker-runtime/src/worker-task-pool-retirement.ts@8f8b1804b1b9538793953231bc1cfb76b1e1fcef` L1–190,191–378/378.
- `packages/worker-runtime/src/worker-task-pool-worker.ts@940914c7a6153759f4b5f4d70b88d4f44a4a1be3` L1–164/164.
- `packages/worker-runtime/src/worker-task-host.ts@b6c85f2705a77b405d37f5ca4477160e29fe2dd0` L1–80/80.
- `packages/worker-runtime/src/worker-task-admission.ts@e7703e49a298d2e2b26c8978588d2f3531f1f686` L1–95/95.
- `src/infra/worker-task-pool.ts@5a8565c4678f35b029c24921a62e14c7f026755b` L1–80/80.
- `src/infra/worker-native-lifecycle.ts@4afc06188377ae708c7a9db36a272b650f719086` L1–160,161–320,321–457/457.
- `src/infra/worker-pool-sizing.ts@0389f146ef046775fae6d4039f15c1ecda9b056b` L1–44/44.
- `src/infra/worker-task-capacity.ts@4e5ed2cdc6967cee1bb4f50710fc55b923f57afa` L1–16/16.
- `src/infra/sqlite-snapshot-single-flight.ts@fdd43625346f66a705e9e71774111369e6f9a6cc` L1–210,211–355/355.
- `src/infra/sqlite-snapshot-staging-owner.ts@251f37366ffac2ed5f9e995c785fc9e30993b245` L1–190,191–382/382.
- `src/infra/sqlite-live-snapshot.ts@d5d47ff07eee4e247419c35d108aec99491ec799` L1–62/62.
- `src/infra/sqlite-readonly-location.types.ts@3bedb510ef9e6384fa3fa1bd59423222d2114c81` L1–32/32.
- `src/state/openclaw-agent-write-admission-state.ts@4c9b60d65f365858819ddde0c59a125a331774c4` L1–32/32.
- `src/infra/sqlite-handle-lifecycle.ts@7de9b2eff19bb93061ef53bfb1edb2289cd42e12` L1–21/21.

## Falsifiable source-derived behavior (NOT executed as target tests)
- `R20-READ-01` Pool selection: SQLite library selected before worker spawn; state-read worker count at least two, 512-MB old-generation per worker, bounded queue/bytes, surplus idle retirement; test/development rotation asserts no active writer (`openclaw-state-read-pool.ts:24–68`, `worker-pool-sizing.ts:10–19`, `worker-task-admission.ts:7–95`). Counterexample: unbounded/stale retained read monopolizes whole pool or bypasses writer fence.
- `R20-READ-02` Snapshot source authority: private snapshot/WAL family, live-owner `assertCurrent`, expected file identity and artifact-preserving scope must precede borrowed bytes (`sqlite-snapshot-source.ts:54–230`, `sqlite-live-snapshot.ts:19–62`). Counterexample: stale snapshot treated as current approval or orphan WAL removed during another reader's lease.
- `R20-READ-03` Singleflight key uses path identity+operation and optional owner scope; lease reference released once, last withdraw aborts orphan producer and cleanup failures remain failures (`sqlite-snapshot-single-flight.ts:41–355`). Counterexample: new generation shares retired bytes or falsely reports cleanup success.
- `R20-READ-04` Staging owner serializes resource custody with pool `workerClass:'writer'` but explicitly `databaseAccess:'read-only'` (`sqlite-snapshot-staging-owner.ts:51–76`): **NOT a pairing-business writer**. Directory registration, cleanup receipt and native process join stay separate (`:94–382`, `worker-native-lifecycle.ts:109–457`, `worker-task-pool-resources.ts:15–209`). Counterexample: IPC close counted as native COMMIT/stop.
- `R20-READ-05` Retained operation can settle synchronously independently of Promise and mandatory cleanup; owned task error/cancel can retain native slot and admission budget until receipt/retirement; reject close needs explicit recovery (`retained-operation.ts:4–177`, `worker-task-pool-owned.ts:84–320`, `worker-task-pool-completion.ts:19–154`, `worker-task-pool-core.ts:211–750`, `worker-task-pool-retirement.ts:119–299`). Counterexample: error or timeout implies transaction rollback or immediate safe retry.
- `R20-READ-06` Rotation pauses dispatch and joins tasks/native stop/artifact cleanup; late host exchange cannot feed a successor; a limited read-only snapshot retry on rollback journal cannot justify replaying a pairing mutation (`worker-task-pool-core.ts:295–750`, `worker-task-pool-retirement.ts:42–378`, `sqlite-snapshot-source.ts:326–460`).
- `R20-READ-07` From prior full reads: `device-pairing-store-readonly.ts@a2541c87160e7acfffd96374dde710bba7c186ea`, `device-pairing-publication.ts@a8d9e7bcbde564ebf8d08cd00ca7fd8e6cf97666`, `openclaw-state-db-readonly.ts@5bd95bc9cf5aa3e1dc24a2d9c4dbbc03e46a67f8`: after unknown COMMIT, fresh authoritative state must be reconciled by request/device/key/revision/roles/scopes/bearer and node generation. Missing pending row alone proves neither success nor failure.

## Fabushi H owner / consumer investigation
- FULL: `source/host/src/host_gateway_api.rs@e93659285baf4903d3c97cf85629864341ad4126` L1–256. Registry has native Agent/Session/Secrets/MCP etc. **No pairing approval/rotate/revoke method in this registry**; absence here does not prove absence elsewhere.
- FULL: `source/host/src/host_secret_store.rs@7f20790cf4c29e20de214468a82fd93d3aeb0daa` L1–73: machine ID cache/file, not pairing durable authorization owner.
- FULL: `source/host/src/storage/store_db.rs@f52eddab95fcccec3b9c0e232ddf25b5f0fc27ce` L1–140: database-path write generations, live handles/WAL checkpoint, not domain approval writer.
- FULL: `source/node-agent-coordinator/src/renderer_port_server.rs@ede3e482d6d29847fb56ac1d0f2d77524331f55e` L1–191: port Hello, unique IDs, Cancel→Abort, not transaction authority.
- FULL: `frontend/src/production/coordinator-client.ts@f120b9f600861560409d9a9cac6487cb5ec79e63` L1–194: request/reconnect pending Promise rejection, not persistence/revoke owner.
- PARTIAL: `frontend/src/production/ProductionRenderer.tsx@af8a83a8dd7cf1444474889ce3e64fe796a7d458` L643–690/4457: `callCoordinatorWithAbort` local rejection sends no Cancel.
- PARTIAL: `source/mahayana/mahayana-rs/mahayana-secrets/src/lib.rs@9ebde84131d61a5d6386ec9f2c47d511b6ca3423` L1–220/871: encrypted age/keyring namespaces (credential primitive, not pairing authorization). Host auth/Secrets extensions examined by symbols only; no unique pairing row writer established.
- PARTIAL: `source/mahayana/mahayana-rs/mahayana-cli/src/main.rs@a0e0a98f20e7f24eb75dc13ae9bffdf5c4846b6e` L500–730/2305, device_agent `@10029b40dcccd21c920b760b5deb234b93af90e1` L390–520/661: CLI uses local runtime, device agent is separate official `wss://fabushi-mcp.ombhrum.com/agent` with session/bearer/lease/generation. Neither proves OpenClaw Host pairing client.
- Existing Host HTTP/TLS/Bearer `gateway_server.rs@1ce506ae0bc75ee96418e55eb64a935dd7865ac0` and `network_enrollment_owner.rs@dac2ea3dfa439570de57722c23f583b160f15af9` are distinguishable existing owners; no proof of OpenClaw pairing writer. Coordinator `main.rs@c822c07dff3027a22a658d1ece6aefe919691365` only previously targeted; actual external companion location remains unconfirmed. See `gateway-pairing-consumers-followup-20261022.md@cc6d16c0cfe54eaa5c46e9c0112f18db8c6aef46`.

## PostgreSQL 14-package provenance: all published-byte checks remain open
- Prior exact lock and 13 source/tag/commit/licenses matrix: `pairing-unknown-reconcile-postgresql-provenance-round19.md@8beb98e6dbaa74e20e60dae5616a96e2f982888c`, frozen `pnpm-lock.yaml@80fb4d9fa985297a24ac88931ef534e85a40ba47`.
- All locked packages: `@types/pg@8.23.1`, `pg@8.23.1`, `pg-cloudflare@1.4.1`, `pg-connection-string@2.14.1`, `pg-int8@1.0.1`, `pg-pool@3.14.0`, `pg-protocol@1.16.1`, `pg-types@2.2.0`, `pgpass@1.0.5`, `postgres-array@2.0.0`, `postgres-bytea@1.0.1`, `postgres-date@1.0.7`, `postgres-interval@1.2.0`, `split2@4.2.0`.
- Known negative: `pg-pool@3.14.0` source commit `544b1ce8152bc280e398dc1e8a66920abe6a640e` differs from `pg@8.23.1` source `0980cefebe0ae461da8883703be049fe13ca96cf`; `pg-protocol`/`pg-cloudflare` generate `dist`; DefinitelyTyped inspected manifest `types/pg` says `8.23.9999`, not npm `8.23.1`. CDN file listings may show licenses but do not establish exact tarball bytes.
- NO actual npm tarball downloaded/hashed this round. Every lock SHA512-byte comparison, published LICENSE/NOTICE, dist provenance, attestation and recursive distribution dependency confirmation remains unverified. Use a frozen GitHub Actions package-oracle lane.

## Honest verdict / frontier
- Exact H Actions run `39770034071` = completed success for `source-authority` job `119794964992` and `persist-verified-inventory` `119795368378`; artifact `12420959002`, digest `sha256:67ff42bd730af69010500aa43dcb1d40d0991b7b191c0791c6af0e14ebda8d65`. **P0 structural only; not scoped G01/G02 or Rust parity.** PR #169 submitted reviews = zero.
- Real subagent `agents.status` failed `NOT_FOUND` (Tunnel-client not seen for 300 seconds); no spawned worker, independent review or ACCEPT claimed.
- `OCRM-BLOCK-GW-SCOPED-AUTHORITY-001`: remaining direct/dynamic/build graph not fully read/locked, no scoped G01/G02; **no production Rust pairing writer admission**.
- `OCRM-BLOCK-GW-INDEPENDENT-REVIEW-002`: unavailable reviewer, no independent ACCEPT; continue GitHub-only read audit.
- `OCRM-BLOCK-PG-NPM-PROVENANCE-003`: 14 tarball hash/license/provenance checks open.
- Next exact source reads: `src/infra/sqlite-readonly-worker.ts@42fb30c511907b162f8e418543d9561f13d4592f` (579 lines), `bun-sqlite-library.ts@27594ac8a2e2feb74d851669d31d1bd579e37f55` (338), `openclaw-state-read.types.ts@242e3b155e1195a75dc9e2560ccad36d30711223` (322), worker-runtime pool types/bootstrap/native sections and native handle; recurse via actual import/runtime handlers. Target full Host credential domain owner + Coordinator/Renderer/CLI/companion consumers; establish unique write/reconcile path. Only with independently accepted positive scoped G01/G02 and new exact candidate lease consider minimal Rust production wiring plus exact-head GitHub Actions.
- No production code, inventory, ledger, RTM, lock, acceptance flag, workflow, PR #169 or main mutated by this dossier. New audit commit must not inherit old green receipts.
