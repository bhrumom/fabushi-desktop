# OCRM round 20 - frozen SQLite read pool, snapshot and worker lifecycle

Source audit ONLY. Not Rust implementation, positive scoped G01/G02, independent ACCEPT, or CI receipt. Task 3dd43a37-9f36-4b7a-adac-dfcc87584523, round 20.

## Exact authority

- Frozen upstream openclaw/openclaw ad8d12f126d29b0db204187adf5092db80294e70; tree 04ccce62f1d74dc8e0c88290ae8ded027b7b0f16.
- Target PR #169 open/unmerged at b5db35fa272c83833685d70a730b5f8285fa1176, tree d01637ba8d83cd6c723077555ccf725887f7156e. Its reported base f6eb1758c59d65aba93e5608d7aba445e0bad42b.
- Live main advanced independently to 7394f6dc7687f9b986ee5589d40aa680ce23b90a, tree fbeb0bb32d944d0c1053880c471fe5c52279f052. Production lease must be refreshed.
- Parent independent round19 audit commit 3c6472b8fda83ce17dbbf6c5d4b3b8be37562f93; note blob 8beb98e6dbaa74e20e60dae5616a96e2f982888c remains immutable. This round20 audit carrier does not move PR #169 or modify production, inventory, lock, RTM, ledger, workflows, or flags.

## Frozen source Git identities, actual examined lines

| File | Blob SHA | Examination |
|---|---|---|
| src/state/openclaw-state-read-pool.ts | 1ff025735fb0804bc4152e48e9beca69e9c7fc4d | 1-68 complete |
| src/infra/sqlite-snapshot-source.ts | 14931e709dbccb2daab535c15201dfc671994904 | 1-460 complete |
| src/infra/sqlite-snapshot-single-flight.ts | fdd43625346f66a705e9e71774111369e6f9a6cc | 1-355 complete |
| src/infra/sqlite-snapshot-staging-owner.ts | 251f37366ffac2ed5f9e995c785fc9e30993b245 | 1-382 complete |
| src/infra/sqlite-worker-identity.ts | 9d6fef1eea5eec6901ce9a454e7ab6a7001d7d2d | 1-243 complete |
| src/infra/sqlite-live-snapshot.ts | d5d47ff07eee4e247419c35d108aec99491ec799 | 1-62 complete |
| src/infra/sqlite-private-snapshot-fork.ts | 3ba0c2082b818c846036e5ccee2aec61960a18cb | 1-131 complete |
| packages/worker-runtime/src/retained-operation.ts | 16cb02bbc296f8dd1b7fe308cfdcaf72c168461d | 1-177 complete |
| packages/worker-runtime/src/worker-lifecycle.ts | 2bfb04a02c8cd52c4c3228595f95405d4a073a07 | 1-37 complete |
| src/infra/worker-native-lifecycle.ts | 4afc06188377ae708c7a9db36a272b650f719086 | 1-457 fetched, 40-190 and 270-378 inspected |
| packages/worker-runtime/src/worker-task-pool-core.ts | f73ca6fbfb5cf54a9c69143cce14d02fae8c39f9 | 1-750 fetched, 140-750 inspected |
| src/infra/sqlite-readonly-worker-session.ts | 20f93d5f15a7e58b00d3811f532aacf99b5aac5f | 1-440 fetched, targeted inspection |
| src/infra/sqlite-readonly-worker-pool.ts | b7df53740cacc9b5caef87c60bc9c9152ea2425b | 1-327 fetched, targeted inspection |
| packages/worker-runtime/src/worker-task-pool-retirement.ts | 8f8b1804b1b9538793953231bc1cfb76b1e1fcef | 1-378 fetched, targeted inspection |
| src/state/openclaw-state-read-operation.ts | 2f8740a2800aa5190b00621da5b7f74bdd17e4e2 | 1-318 prior full audit; confirmed |
| src/state/openclaw-state-read-worker.ts | d247c009cf515b8d42623512fd400efc792fbe5e | 1-456 prior full audit; confirmed |

Other exact dependency blobs were fetched/targeted: worker-task-pool-owned.ts 88f6b75cbde72daa17147afc2160e4b56c0d93f2, worker-task-pool-resources.ts e8abbdf4173bd08c59f1a0831560402706463b52, worker-task-pool-completion.ts 131c781f9bc972bf95edcb12e969e6f9d3246580, worker-task-native-sections.ts f4bd0041085d89047b62c916141e4a98d77ee480, worker-task-admission.ts e7703e49a298d2e2b26c8978588d2f3531f1f686, worker-task-capacity.ts 0b5d528a4883aae234efdf2f396dde3cd9e9ed02, sqlite-readonly-worker.ts 42fb30c511907b162f8e418543d9561f13d4592f. Retrieved bytes do NOT equal full transitive semantic closure.

## Falsifiable oracles defined; NOT RUN

- PAIR-R20-01: read-pool L27-67 sets read-only database access, min 2 workers, 512 MiB V8 old-generation configuration, 30s burst idle, 128 pending tasks, 256 MiB queue budget. Writer-release assertion only wrapped in test/development; staging workerClass writer serializes snapshot preparation but databaseAccess remains read-only. Negative: reader or staging worker cannot approve/revoke a pairing grant.
- PAIR-R20-02: SQLite identity verifies physical dev/inode and canonical path, birthtime only where valid. Expected snapshot identity requires artifact preservation; retained main/WAL fingerprints and rooted copy prevent stale or changed source reuse.
- PAIR-R20-03: single-flight shares by physical source+operation+scope, uses per-consumer references and cleanup receipts; cancelling one borrower cannot delete bytes still owned by another. Failed orphan removal is not success.
- PAIR-R20-04: retained worker task result, consumed-input receipt, native resource close and worker retirement are separate. Failed/late host exchange and cancellation do not prove durable mutation rollback.
- PAIR-R20-05: staging owner retains allocated directories and abort ports until task release, all directory cleanup and native owner close. Process generation teardown cannot silently claim cleanup.
- PAIR-R20-06: one-shot rollback-journal retry in sqlite-snapshot-source L413-453 redoes an inspection read only, never a device approve/rotate/revoke mutation.
- PAIR-R20-07: prior PAIR-R19-04/05 shows COMMIT succeeded but reply lost is outcome-unknown. Fresh requestId, device/key, paired/pending, role/scope, persisted revision, bearer/issuer and node generation must be reconciled before any authorized retry. Absent pending row alone is inconclusive.

## Exact target H producer-owner-consumer audit

Host gateway method registry source/host/src/host_gateway_api.rs@e93659285baf4903d3c97cf85629864341ad4126 (1-256) has no OpenClaw pairing method in its actual listed surface; gateway_server.rs@1ce506ae0bc75ee96418e55eb64a935dd7865ac0 (1-1244) is existing HTTP/TLS/Bearer ingress, not proven equivalent WS pairing authorization. host_secret_store.rs@7f20790cf4c29e20de214468a82fd93d3aeb0daa (1-73) stores machineId, not pairing approval. auth_service.rs@e063f2d072d64559544fc1cb8a1050f5999599e3 (1-495 targeted) owns distinct Fabushi Core/inference credentials. secrets_service.rs@d57be254edb21f9e183e72bd971ae6e07fdc1e9e (1-484 targeted) owns Box secrets and applied generations. storage/store_db.rs@f52eddab95fcccec3b9c0e232ddf25b5f0fc27ce (1-140) is generic SQLite infrastructure, not a chosen pairing writer. FDN network_enrollment_owner.rs@dac2ea3dfa439570de57722c23f583b160f15af9 cannot be a second OpenClaw writer.

Coordinator renderer_port_server.rs@ede3e482d6d29847fb56ac1d0f2d77524331f55e (1-191) fences requestId and emits abort on cancel, not COMMIT undo. Coordinator main.rs@c822c07dff3027a22a658d1ece6aefe919691365 (1-2990 fetched/targeted) supervises Host relay. Renderer coordinator-client.ts@f120b9f600861560409d9a9cac6487cb5ec79e63 (1-194) rejects ephemeral pending requests on port loss. ProductionRenderer.tsx@af8a83a8dd7cf1444474889ce3e64fe796a7d458 (1-4457 targeted) has no demonstrated pairing consumer. CLI main.rs@a0e0a98f20e7f24eb75dc13ae9bffdf5c4846b6e (1-2305 targeted) uses separate FeatureHost; CLI mahayana-gateway.rs@660eca96a76f1bd0fb2ae05d7b03f60a579f24ca (1-500 targeted) uses distinct stdio JSON-RPC. device_agent.rs@10029b40dcccd21c920b760b5deb234b93af90e1 (1-661 targeted) connects separate official remote WebSocket. native messaging gateway.rs@2c3427851d21d30d9501a28beaf7c0d1c2d6091b (1-259) is distinct WS service. Dynamically loaded and external companion implementations are not excluded by selected searches.

## External PostgreSQL packages and blockers

Fourteen exact pinned versions (round19 dossier section 5): @types/pg@8.23.1; pg@8.23.1; pg-cloudflare@1.4.1; pg-connection-string@2.14.1; pg-int8@1.0.1; pg-pool@3.14.0; pg-protocol@1.16.1; pg-types@2.2.0; pgpass@1.0.5; postgres-array@2.0.0; postgres-bytea@1.0.1; postgres-date@1.0.7; postgres-interval@1.2.0; split2@4.2.0. **0/14 actual published npm tarball SHA512 bytes, distributed licenses/NOTICE, generated dist or attestations verified.** @types/pg exact publishing source unresolved; pg-protocol dist generated; pg-pool source release differs from pg source commit. Repository Git tags or lock integrity claims cannot replace published tarball verification. Execute package download/verification/unpack only via GitHub Actions.

Open blockers: OCRM-BLOCK-GW-SCOPED-AUTHORITY-001 (recursive source/dynamic/build dependencies, positive scoped G01/G02 absent); OCRM-BLOCK-GW-INDEPENDENT-REVIEW-002 (GitHub PR #169 reviews empty; real agents.status NOT_FOUND tunnel-client unseen >300 seconds, no reviewer spawned); OCRM-BLOCK-PG-NPM-PROVENANCE-003 (14 tarballs and exact typings source); OCRM-BLOCK-PAIRING-PRODUCTION-004 (single current Rust Host pairing writer and production callers not proven). No independent ACCEPT, no scoped CI receipt, no permission for Rust implementation or state promotion. Existing PR #169 P0 run 39770034071/artifact 12420959002 only proves its own limited source-authority/structural inventory, not this new carrier. No new tests/builds run; no duplicate Gmail notice sent.

Ready next: read native worker task server/bootstrap/runtime-generation/SpawnBroker and remaining sqlite readonly session/staging dependencies at frozen SHA; resolve Host security schema, dynamic Gateway/Renderer/CLI/companion pairing consumers; create exact-subject GitHub Actions-only tarball provenance gate and independently reviewed scoped G01/G02 before production Rust edit.

Sources: https://github.com/openclaw/openclaw/tree/ad8d12f126d29b0db204187adf5092db80294e70 and https://github.com/bhrumom/fabushi-desktop/pull/169
