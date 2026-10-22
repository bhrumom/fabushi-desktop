# OCRM Gateway SQLite pairing and P0 run continuation — 2026-10-22

**Scope:** OCRM-M01/M04; GW-AUTH-01/02; OCRM-UNIT-GW-INGRESS-001. This is source research ONLY. Scoped G01/G02, independent reviewer ACCEPT and §11.1 admission remain OPEN; no production Rust, ledger, RTM or acceptance flag changes.

## Exact identity and new Actions receipt

Frozen source: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. Subject target H: `bhrumom/fabushi-desktop@286b00ece5ad9084e26c0dfb5180c98bd7d21190`, tree `2e31b77d90c69b7dce6522216d45401545828bdb`; PR #169 source branch `ocrm/p0-source-authority-20261022`. PR base SHA `f6eb1758c59d65aba93e5608d7aba445e0bad42b`; separately fetched live main `7394f6dc7687f9b986ee5589d40aa680ce23b90a`.

Run `39765741273` attempt 1 completed/success; source-authority job `119780613181` and persist-verified-inventory job `119781021291` both completed/success. Downloaded decoded job logs show both performed detached checkout of exact H, not synthetic merge. Source authority checked frozen upstream commit/tree. Canonical tracked inventory = **58,750** records and sha256 `586cadaa199916ac9593782b59059bfdcf8c72a717e534a72225e7e883977153`. Persist job reconstructed **58** shards and 58,750 records with PASS, reporting `No new source-derived shard content`. Structural G02 negative probes rejected duplicate path, tampered unread count, unbacked accepted verdict, orphan responsibility. Full external packages/LFS/source semantics are NOT closed.

Immutable artifact ID **12420850119**, name `ocrm-p0-authority-286b00ece5ad9084e26c0dfb5180c98bd7d21190`, ZIP size **4,479,295** bytes; metadata/upload/download digest unanimously `sha256:a24d72efc75475d55083183ff8ecb12c4f1577f4385afb71b163284835b94dcd`. A later dossier carrier H2 is a new subject and cannot inherit H's proof.

## Frozen upstream: newly complete text review (actual blobs)

| File | Git blob SHA | Reviewed interval |
| --- | --- | --- |
| `src/infra/device-pairing-store.ts` | `b7592120467b67ab19011d3ce3eea055559adf25` | 1–648, trailing empty line 649 |
| `src/infra/sqlite-worker-operation-settlement.ts` | `d1c3b24539d751d33f067e2c7ad83f6485839c26` | 1–203, trailing empty line 204 |
| `src/infra/sqlite-post-commit.ts` | `a203acefc5d28361a24a620fd132c8f264e8f4e9` | 1–258, trailing empty line 259 |
| `src/infra/device-pairing-store-cache.ts` | `82335a9a30db69a0a612ae42e1dace395cf7c486` | 1–63 |
| `src/infra/device-pairing-cloud-worker.ts` | `7d17c6b00992c27ce44f1f0d6ac9c7d6e8c4a6c4` | 1–90 |
| `src/infra/device-pairing-row-mapping.ts` | `8af0a7638a6e2c23300323043935c549cb77739d` | 1–246 |
| `src/infra/device-pairing.types.ts` | `5a7413d40f0a96fff15da5588ecee53f427b25f7` | 1–163 |
| `src/infra/device-pairing-core.types.ts` | `2cc2a8adfc6b75ad9faf21506e1f8a1dbd07751f` | 1–123 |
| `src/infra/sql-connection.ts` | `7e32009a4bb407f5641ce09bf1d408f145c65429` | 1–62 |
| `src/infra/sqlite-worker-request-custody.ts` | `9d97e0addf77673ffc1354405d2c606fe9f6346d` | 1–90 |
| `src/infra/sqlite-lifecycle-errors.ts` | `e95e8412ca4872e9b63ffb4f614be84c4546935d` | 1–99 |
| `src/infra/device-bootstrap.worker-types.ts` | `73baf4a19a89094fb09be64fea8a6fd14a7b7a4b` | 1–40 |
| `src/state/openclaw-state-db-transaction.ts` | `8475c371acfd101d4894d5a1325ffae637846ad9` | 1–51 |
| `src/infra/device-pairing-dispatch.worker.ts` | `26b447c56db8318c92dcc6f47c11ebc7c85d985a` | 1–101 |
| `src/infra/device-pairing-mutation.worker.ts` | `50face6d5f32ef58cbfe23cef7f19ccdb2348118` | 1–25 |

Additional inspected-but-NOT-full ranges: `src/state/openclaw-state-db.ts@adb2af202d485fc4d98fb405935921da8efd2689` 570–694/734, `src/infra/sqlite-transaction.ts@eac4144f319f561bc786c99d2dd01223dc3b1337` 360–635/717; `src/infra/sqlite-worker-contract.ts@f2cffe2e4bc4ecd94b7f8528d706deedcd141bd4` 1–160/316; `src/gateway/server-methods.ts@5bebd34bd2b3e5765947fb58a5b71be98588c10a` 1–135,250–354,535–604/604, `src/gateway/server-methods/devices.ts@ad29c673a9637e230b1ddb96e9b597e0426701fb` 350–578/578; `src/cli/devices-cli.runtime.ts@7b9807282c413bf3ed978f72b895c092f1c58855` 175–325/966; `ui/src/lib/nodes/page-operations.ts@5edeeb4c60c1a521ddf31cab59d9ef31ea859716` 130–275,385–440/581. Earlier dossiers may separately establish other full read intervals; this round does NOT assert full recursive closure.

## Verified contract / state owner insights

- `device-pairing-store.ts:91–146,157–245,300–390`: AsyncLocalStorage binds worker-authorized physical DB transaction. Paired row updates and `config_machine_state.devicePairing.revision` share one transaction; cache and node-snapshot invalidation are staged for real COMMIT. Revision and paired rows are read from one coherent snapshot. `device_pairing_pending` and `device_pairing_paired` are durable rows; read caches are projections, not separate writers.
- `device-pairing-store.ts:420–648`: bootstrap credential/device/expiry is checked against the authoritative row at consumption, after earlier async work; cloud worker binding uses environment owner/provision epoch. Setup completion begins `uncertain` and requires exact setupId/deviceId/retention-valid update to `confirmed`; bearer retirement is in the same transaction. A transport success is not proof of peer delivery.
- `sqlite-worker-operation-settlement.ts:31–64,105–190`: receipts require version 1 and strictly positive safe integer sequence; captured native facts are bounded at 32MiB; one operation owner stages COMMIT receipt under SQL publication anchor, tracks sequence, retains committed facts on message delivery failure; `completed` vs `unknown` comes from executing owner and never independently grants write permission.
- `sqlite-post-commit.ts:41–125,138–258`: commit facts install before projection and best-effort notification; failure is fenced before and after install. Nested savepoint rollback discards staged effects; unknown native outcome invalidates rather than certifies a rollback. Observer/log failure cannot reverse a physically durable COMMIT.
- `device-pairing-row-mapping.ts:129–169` allowlists approval provenance and maps unknown setup access to `limited`, unknown delivery to `uncertain`. Credential/token scopes and separate node capability surface are distinct; reconnect cannot silently mint broader node authority.
- `src/gateway/server-methods.ts:83–125` builds core/plugin registry; `server-methods/devices.ts:350–575` serves pairing approve/reject/list/remove and token rotate/revoke through Gateway authorization + worker mutation. `ui/src/lib/nodes/page-operations.ts:138–275,389–436` calls the RPC and refreshes only current UI generation; `src/cli/devices-cli.runtime.ts:180–325` has a *constrained, owner-verified loopback/local-state fallback* that still needs complete scope security review. Neither is proof of a second remote writer.
- Target H `source/host/src/gateway_server.rs@1ce506ae0bc75ee96418e55eb64a935dd7865ac0:1–112,1037–1200` has explicit HTTP /api method allowlist, Host auth and dispatch; `source/host/src/host_gateway_api.rs@e93659285baf4903d3c97cf85629864341ad4126:1–190` contains method→domain owner registry but no source OpenClaw `device.pair.*` methods in examined scope. `source/node-agent-coordinator/src/gateway/gateway_request_dispatcher.rs@38ca19db76493f9c2bca8862b484d61b7e74f47c:50–100` is registered method transport, not device credential writer. Renderer/Coordinator port path described in `gateway-pairing-consumers-followup-20261022.md`; not an accepted pairing integration.
- Target `source/mahayana/mahayana-rs/mahayana-cli/src/main.rs@a0e0a98f20e7f24eb75dc13ae9bffdf5c4846b6e:1667–1685` operates its own device Agent; `device_agent.rs@10029b40dcccd21c920b760b5deb234b93af90e1:392–525` uses distinct official device gateway signed-in session, generation/lease and heartbeat. `source/product/fabushi/fabushi-official-device-registry.ts@461ec329caf850c5b494db38c62a2ed88ef5f140` is App-Agent Surface. Existing FDN enrollment is another domain. None proves OpenClaw paired-device permission-store equivalence.

**Owner election OPEN:** Host security/durable state and Host Gateway are existing-owner-first candidates. Coordinator, Renderer, network enrollment, CLI device service and App-Agent must not become unauthorized duplicate pairing writers. No production Rust or ADR election without the complete current security/storage/consumer proof.

## New unexecuted falsifiable oracles

- `OCRM-GW-WS-O55`: native COMMIT vs savepoint rollback, mixed coupled writes and revision/row atomicity; no early authorization publication.
- `O56`: oversized/malformed/duplicate/out-of-order native receipts, failed transport, unknown settlement and crash prevent replay or false rollback.
- `O57`: token expiry/device mismatch during async setup, cloud-worker owner change and uncertain-to-confirmed handoff never double-spend credential.
- `O58`: role downgrade/token revoke/pairing generation replacement fences UI, CLI local fallback, companion and retained worker authorities.
- `O59`: failing cache/projection/observer cannot restore revoked grant or suppress independent committed receipt.
- `O60`: a single Rust Host pairing writer serves real Gateway/Renderer/CLI/companion paths and excludes competing FDN/App-Agent writer.

**Outstanding:** dependency and LFS/package integrity closure, `sqlite-transaction.ts`, `sqlite-database-admission.ts`, OpenClaw StateDB generator/schema, Gateway 121 lazy handler modules, source tests, full target Host/Coordinator/UI/CLI/companion callgraph, scoped G01/G02 Actions, independent read-only review and §11.1. All completion flags stay false. No local test/build; no user-support blocker newly demonstrated; no duplicate support email.
