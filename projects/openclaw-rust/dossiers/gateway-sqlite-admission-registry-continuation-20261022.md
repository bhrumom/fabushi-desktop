# OCRM Gateway SQLite/StateDB admission + pairing consumers: exact-head source dossier (2026-10-22)

## Scope, status and non-claims

- Project OCRM-001; OCRM-M01/M04, GW-AUTH-01/GW-AUTH-02, OCRM-UNIT-GW-INGRESS-001, upstream device pairing's SQLite transaction, admission, source-of-truth and actual Gateway/UI/CLI consumers.
- **Evidence-only partial source investigation. Not a positive scoped G01/G02 verdict. Not independent ACCEPT. Spec section 11.1 NOT admitted.** Full recursive source/dependency closure, all 121 lazy handler modules, native Rust writer election, positive test oracles and production parity all remain OPEN. No read/mapped/implemented/wired/verified/accepted upgrade is claimed.
- Frozen upstream source: openclaw/openclaw commit ad8d12f126d29b0db204187adf5092db80294e70; tree 04ccce62f1d74dc8e0c88290ae8ded027b7b0f16. No upstream rebaseline.
- Verified target subject H: bhrumom/fabushi-desktop commit b93056fc52264986f6abcad9803ff31bb7edf0aa; tree e61c23e0eaa6330dba1e978b628c94ab35ad48ce. Parent 286b00ece5ad9084e26c0dfb5180c98bd7d21190. PR #169, branch ocrm/p0-source-authority-20261022, PR base main=f6eb1758c59d65aba93e5608d7aba445e0bad42b; separately queried live main=7394f6dc7687f9b986ee5589d40aa680ce23b90a. This *document's own later commit is an evidence carrier and a NEW subject*, not H; it cannot inherit H's test verdict.

## Verified exact-H GitHub Actions receipt (not equivalent to semantic scope acceptance)

- Run 39767296497 attempt 1, pull_request, completed/success, run head_sha exactly H.
- Source-authority job 119785819071 completed/success; persist-verified-inventory job 119786297102 completed/success. Downloaded both decoded complete job logs and verified actual git checkout --force of b93056fc52264986f6abcad9803ff31bb7edf0aa. Not a synthetic merge checkout.
- Canonical tracked source rows: **58,750**. Canonical manifest SHA256: **586cadaa199916ac9593782b59059bfdcf8c72a717e534a72225e7e883977153**. Persist job reconstructed 58 committed shards back into 58,750 rows; no new source-derived shard changes.
- Structural G02 negative probes rejected duplicate source path, forged unread denominator, unbacked accepted verdict and orphan responsibility. This is STRUCTURAL validation, not scope-specific semantic G02 or full closure.
- Immutable artifact ID **12421490304**, name ocrm-p0-authority-b93056fc52264986f6abcad9803ff31bb7edf0aa, 4,479,295 bytes, GitHub metadata digest **sha256:56cfaba051caa80c4a1a7a9719cce5a20a7a000b78924a8db198a2115ba54dba**. Metadata, upload and download-log digest agree; do not assume this alone proves semantic content acceptance.
- At subject H, projects/openclaw-rust/inventory/index.json@6a91d48d718453bf414c819cfcff6c80aa616fb2 truthfully remains unread=58,750, unclassified=58,750, source_closure_ready=false, dependency_closure_enumerated=false, lfs_content_closed=false and baseline_identity_accepted=false. Nothing in this dossier upgrades them.

## Newly full-read frozen source and blob identities

Each listed text was fetched at the frozen upstream ref and examined across the whole declared line range; trailing newline may appear as an empty last line. Full source *file* read is NOT complete recursive source closure.

| Frozen path | Exact Git blob SHA | Read range |
| --- | --- | --- |
| src/infra/sqlite-transaction.ts | eac4144f319f561bc786c99d2dd01223dc3b1337 | 1-717 |
| src/infra/sqlite-database-admission.ts | ea267e5c7abccb15466f5b606b896eca28afa585 | 1-740 |
| src/state/openclaw-state-db.ts | adb2af202d485fc4d98fb405935921da8efd2689 | 1-734 |
| src/infra/sqlite-database-admission-context.ts | 74cbe80d8f81ec0020ff5655a340de7dabca5dcb | 1-68 |
| src/infra/sqlite-native-admission.ts | f70559b1d24e189acdcca62263be72c1ae4c2efc | 1-35 |
| src/infra/database-engine/admission-scope.ts | 843b7dddfc45e81224336c4fdc8f306b5c1e8818 | 1-48 |
| src/state/openclaw-state-db-contract.ts | 674669639559e288389442ebcf3f3f24f484365b | 1-177 |
| src/state/openclaw-state-ownership-preflight.ts | b9513d9293e3e3ef5d8c69a61af65334703e3494 | 1-103 |
| src/state/openclaw-state-ownership.ts | 3910a770050e5d207c6eca47ffa3d5a892fd92af | 1-277 |
| src/infra/gateway-state-owner.ts | f393e7f1b8c3b632ebdb8f12bdef002212de4cca | 1-746 |
| src/infra/sqlite-database-admission-record.ts | a2ee4569ea3b23293fa38058388061236b999d0a | 1-760 |
| src/infra/sqlite-database-write-receipts.ts | c381431762db0501fa63b90bfdf3112fdfab981b | 1-509 |
| src/infra/sqlite-database-admission-read.ts | addb3def599251bdfec5fece08938ee462b44326 | 1-208 |
| src/infra/sqlite-database-admission-seal.ts | 35ae125ccb678e44a54e89d99948ef4dc0c839b9 | 1-312 |
| src/infra/sqlite-busy-timeout.ts | 1441f6d7d9c91a235f8248e8a1ddc5b9c4bbb894 | 1-95 |
| src/state/openclaw-state-db-open.ts | f90ef3094b42fb50d7110b0ee804e74cda4ced96 | 1-345 |
| src/state/openclaw-state-db-readonly.ts | 5bd95bc9cf5aa3e1dc24a2d9c4dbbc03e46a67f8 | 1-745 |
| src/state/openclaw-state-db-cache.ts | adcc28173d50c16683e776dcf6bbd056022dd35a | 1-750 |
| src/gateway/server-methods.ts | 5bebd34bd2b3e5765947fb58a5b71be98588c10a | 1-604 |
| src/gateway/server-methods/devices.ts | ad29c673a9637e230b1ddb96e9b597e0426701fb | 1-578 |
| src/gateway/server-methods/device-management-authz.ts | 6d4e874d6a6cb97ce46e54103533a436ec93c471 | 1-71 |
| src/gateway/server-methods/request-authorization.ts | a6230b631de50d0ad282e108d0af5e62717cfcd2 | 1-661 |
| src/gateway/server-methods/method-authorization.ts | a4dda8be141ad623d9e2e668f51e2e6bf2227cd3 | 1-139 |
| src/gateway/methods/registry.ts | e073cd9d480820c47662a786c90de9558a9d379e | 1-127 |
| src/gateway/methods/descriptor.ts | f36351dfe5d1db8430f3f467d35190344633deb1 | 1-113 |
| src/gateway/methods/core-method-policy.ts | 2b64d1738783b5261d091ef40e7f34606fd1f0f3 | 1-125 |
| src/cli/devices-cli.runtime.ts | 7b9807282c413bf3ed978f72b895c092f1c58855 | 1-966 |
| src/cli/devices-cli-fallback.ts | 819a73733962b2a5c15116e0609dfe45ab3effde | 1-91 |
| src/cli/local-state-owner.ts | aa86219f6c4490c7587eeda1dc088c7f5e30f754 | 1-372 |
| ui/src/lib/nodes/page-operations.ts | 5edeeb4c60c1a521ddf31cab59d9ef31ea859716 | 1-581 |

The policy table src/gateway/methods/core-descriptors.ts@6361c57fbfca7dab4aab55284762f84a385898fc is **targeted** at 278-320/762 (large whole output was truncated); do not treat its entire 762-line semantic review as closed. Earlier dossier gateway-pairing-commit-owner-continuation-20261022.md records prior pairing/store/settlement/postcommit blobs; no duplicated or upgraded claim here.

## Transaction, physical admission and authority boundaries

1. sqlite-transaction.ts: transaction entry is synchronous, mutation callback must not return Promise; nested savepoints stage/discard effects, rollback failure poisons the native handle. Retry of SQLITE_BUSY is limited to BEFORE callback/admission starts. Once COMMIT or native execution may have happened, transport loss and timeout are outcome-unknown, not grounds to replay. Physical COMMIT proceeds before post-commit observers; observer delivery is not a second commit.
2. sqlite-database-admission.ts + admission-record + write-receipts: owner registry binds canonical path to file identity (device/inode/birthtime), native descriptor custody, generation and publication scopes; filename/replaced-file identity alone cannot grant a writer. Worker-local facts/receipts remain transaction-scoped, host-only fact keys require host authority, rollback/dirty DDL invalidates. Failed exit or unknown commit invalidates grant candidates; revision changes are not positive proof of committed authorization.
3. gateway-state-owner.ts + state-db-ownership: Gateway process/maintenance role and live owner lock; stored gateway.supervision in config_machine_state is a provenance/ownership fact, not an independent capability grant. Owner must remain live at admission and physical write; the CLI can only acquire exclusive offline custody or route to the exact live owner with expectedOwnerId. A local cached read or copied WAL snapshot cannot authorize mutation.
4. state-db.ts, state-db-open, readonly and cache: content-version/schema/integrity admission precedes repair or write; file identity checked again on open and after setup. Immutable/disposable read scopes and cached views never become a durable pairing writer; current permission guards reenter committed state instead of inherited discovery snapshot. Quarantine/close/worker settlement retains native custody and must not present a stale cache as live authority.
5. sqlite-database-admission-seal: clean-close restart seal restores compatible **schema/integrity proof only**, NEVER pairing tokens/grants or arbitrary row caches. Invalid/legacy seal/feature changes fail closed; worker cannot re-admit stale proof.
6. Per-machine device_pairing_pending, device_pairing_paired and revision in config_machine_state.devicePairing are persistent owner facts (previous full pairing-store dossier). Post-COMMIT pairing snapshot/cache invalidation can project these; neither React state nor CLI/companion is a second source of authorization.

## Gateway/CLI/UI and Fabushi existing production consumers (verified limits)

- src/gateway/methods/core-descriptors.ts:278-294 + policy builder: node.pair.* and device.pair.list/approve/reject/remove/rename and device.token.rotate/revoke are operator.pairing scoped; setupCode/setupStatus are admin and unadvertised. Actual core handler must have classified descriptor; registry rejects duplicates and plugin scope weakening. GW server-methods/request-authorization checks current authenticated role/scope, profile/session fences and pending startup; async handler preparation and node connection pairing state are rechecked before invoke.
- src/gateway/server-methods/devices.ts: device-token-authenticated non-admin callers cannot manage other devices; non-operator role/role token changes require admin. Paired list redacts secret tokens. Cross-device rotation returns tokenDelivery=withheld-cross-device and no bearer; on rotate/revoke/remove it fences clients and schedules disconnect/reconciliation after durable mutation.
- src/cli/devices-cli.runtime.ts and devices-cli-fallback: normal operations call the remote Gateway RPC, read-only access selected for CLI transport. Only implicit loopback and explicit pre-dispatch closure/pairing-required contract allow local read/fallback; an explicit --url or remotely dispatched/unknown mutation cannot silently switch to local files. Local approval acquires state owner's exclusive offline custody, onForeignOwner=refuse, exact request/proven matching device public key, roles, metadata and scope ceiling. --latest previews a request, never directly approves. local-state-owner.ts routes under expectedOwnerId, distinguishes OWNER_REFUSED and OUTCOME_UNKNOWN after dispatch and forbids local replay.
- ui/src/lib/nodes/page-operations.ts: device.list, approve, reject, rename, remove, rotate/revoke go via RPC, generation-fence visible refresh. User-managed credential persistence/revocation for exact local device survives page reconnect; UI rotation validates explicit tokenDelivery and matching device/role rather than guessing absence means success. Exec approvals native node profile is read-only in this UI.
- Target source/host/src/host_gateway_api.rs@e93659285baf4903d3c97cf85629864341ad4126:1-256 is current Host method-to-domain owner registry, no device.pair.* or device.token.* registration. Target source/host/src/gateway_server.rs@1ce506ae0bc75ee96418e55eb64a935dd7865ac0:38-116,1000-1115 is an HTTP /api allowlisted Bearer/loopback gateway, not proven to implement OpenClaw's WebSocket pairing protocol. Target source/node-agent-coordinator/src/gateway/gateway_request_dispatcher.rs@38ca19db76493f9c2bca8862b484d61b7e74f47c:1-246 is request router/HTTP bridge; transport response/timeout not durable mutation fact.
- Target source/mahayana/mahayana-rs/mahayana-cli/src/device_agent.rs@10029b40dcccd21c920b760b5deb234b93af90e1:395-530 and source/product/fabushi/fabushi-official-device-registry.ts@461ec329caf850c5b494db38c62a2ed88ef5f140:240-330 talk to a separate authenticated official device service. The latter validates session before and after tool calls, and routes through App Agent Surface; it is neither an OpenClaw device.pair writer nor evidence of OpenClaw protocol parity.
- Existing target UI/Electron/Coordinator producer→bridge→Host snippets and exact SHA are documented in gateway-pairing-consumers-followup-20261022.md. That traced transport is **not** an accepted pairing producer→unique Rust writer→consumer circuit. Full target UI, mobile/companion and cli callgraph remains open; no owner election or ADR approval.

## Falsifiable, NOT-RUN continuation oracles

- OCRM-GW-WS-O61: BEGIN IMMEDIATE contention/retry only before entry; a COMMIT-success + lost reply/restart must not replay a mutation or misclassify it as rolled back.
- O62: nested savepoint rollback versus outer COMMIT: coupled paired/token/revision facts become visible atomically, no pre-commit permission/cache publication.
- O63: file path swap, inode/generation/owner lock changes and failed close reject a stale handle; held physical custody and incomplete cleanup cannot re-admit writers.
- O64: native scope receipt size/sequence/revision and host-only fact exchange; malformed/stale/duplicated receipts and foreign worker may neither publish nor grant authority.
- O65: a valid clean-close seal accelerates schema/integrity proof only, never resurrects a revoked token or pairing grant after restart.
- O66: artifact-preserving/snapshot/current reads respect exact owner, cancellation and retained drain; no reader cache/tombstone creates second writer or hides revocation.
- O67: pairing RPC denies role/scope/cross-device violations, and later role, token, device-key and pairing-generation change between authorization and write; approval/rotation must bind the exact committed row.
- O68: CLI fallback refuses explicit URL, dispatched/unknown outcome, foreign offline owner, divergent local request ID, changed public key/roles/scope; --latest remains preview-only.
- O69: UI reconnect/abort, cross-device withheld token, stale response and accepted result with failed refresh preserve correct account-bound persisted outcome and disclose unknown where required.
- O70: Gateway, Host, Coordinator, Renderer, CLI, device agent and companion converge on **one** authenticated Rust pairing writer, while keeping official device enrollment and App Agent Surface as separate explicit contracts.

## Open exact-scoped frontier and immutable evidence discipline

- Still requires full recursively imported/dynamic TS, codegen, tests and package-source dependencies, including sqlite-worker-identity.ts@9d6fef1eea5eec6901ce9a454e7ab6a7001d7d2d, sqlite-schema-facts.ts@6125196b7a8e256066d251b2557cf54716fa3219, sqlite-reader-lifecycle.ts@ff40ee42e0b58f04aee9c24ef7e69e8ebb013112, kysely-sync-cache-state.ts@29f87af9b2d2837db28cb2e1f26c6e8428de3108, sqlite-error-diagnostics.ts@5972bcdbf6d72065b26bd96db90933aa762753b7, core-descriptors.ts@6361c57fbfca7dab4aab55284762f84a385898fc, server-methods/lazy-core-handlers.ts and remaining Gateway authenticated connection, method, plugin and pairing worker dependencies. The noted SHA values are identity only where full read is not explicitly listed.
- Exact target counterpart still needs complete source read and actual producer→owner→storage→consumer trace across Host, Coordinator, Renderer, CLI and companion/FDN/mobile, and proof no parallel mutable pairing owner exists.
- No full dependency/LFS closure, no executable scoped G01/G02 Actions, no independent reviewer ACCEPT, no positive section 11.1 admission. Continue nonproduction research and source-neutral existing-owner review; production Rust, shared protocols, ledger/RTM and all acceptance flags are frozen.
- Every executable test/check/generator/build must run in GitHub Actions only. H's two green P0 jobs prove only their scoped structural receipt. This document itself, once committed, creates a new target H2 that needs its **own** exact-head applicable Actions receipt; the H artifact ID/digest above remains historical, not transferred.
