# OCRM round20-c: frozen SpawnBroker/native-resource protocol and target Host authorization path

> Audit-only descendant of round20-b. **NOT positive scoped G01/G02, Rust pairing production code, independent reviewer ACCEPT, an Actions receipt, or overall acceptance**.
> Task: 3dd43a37-9f36-4b7a-adac-dfcc87584523; round 20; date 2026-10-23.

## A. Exact source, target, and carrier lease

- Source fixed at openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70, tree 04ccce62f1d74dc8e0c88290ae8ded027b7b0f16. No moving upstream main, unverified dependency replacement or source baseline switch.
- Target bhrumom/fabushi-desktop PR #169 remained OPEN/unmerged, H=b5db35fa272c83833685d70a730b5f8285fa1176, tree=d01637ba8d83cd6c723077555ccf725887f7156e, original PR base=f6eb1758c59d65aba93e5608d7aba445e0bad42b. Live main separately at 7394f6dc7687f9b986ee5589d40aa680ce23b90a, tree fbeb0bb32d944d0c1053880c471fe5c52279f052. Do not use these as interchangeable production targets.
- Parent audit carrier for this incremental note is 9b2a0b8bc60a813238bd276fc6026600cd4229dd, tree e5924cef41413717db0d0c0777d7bb1a98aeafa0; previous audit note blob a7dbfa385b92a051ed90ca5c046cc7aeb2d2d6af and original round20 audit blob 1ddd5959a4ab471d39d01e4feeff3407fa01772e must stay unchanged. This file alone is new in the current continuation.

## B. OpenClaw frozen direct dependency full-text evidence

Every interval below was read via GitHub connector using the exact frozen ref, not an inferred current ref. The entire listed file was retrieved/read; that fact does NOT close its recursive build/test/dynamic import or runtime/process dependency set.

| Frozen path | Exact Git blob | Read interval and meaning |
| --- | --- | --- |
| src/process/spawn-broker/host.ts | d440d37816a58c89f0cbeb41c0e902ee36c07ece | L1-160,161-315,316-480,481-600,601-735/735; owning broker, resource gate, boot, command and native cleanup |
| src/process/spawn-broker/resource-host.ts | a85633e89ddad5d48ce1c7bce58f960e1ed9a2b7 | L1-180,181-346/346; resource claim, sequenced owner messages and close receipt |
| src/process/spawn-broker/child.ts | 8cffbe5daa7a5b59b8949a88a0769b55231c4e6a | L1-190,191-371/371; child proxy, stdin/output/IPC, exact exit and close |
| src/process/spawn-broker/resource-protocol.ts | e13653ca4f25d994aa62690be2ec58b890ec5f66 | L1-69/69; typed resource attach, owner ACK, close-error and closed replies |
| src/process/spawn-broker/protocol.ts | 12dd68ee2c2ea954e6d4fa0ca70e31c39c702510 | L1-82/82; spawn/launch/pipe/output/error/result protocol |
| src/process/spawn-broker/resource-authority.ts | 63781a6c76e7e25aaf1e4428725112abe51c1898 | L1-34/34; private IPC endpoint, 0700 root and 32-byte random secret |
| src/infra/runtime-worker-url.ts | 77701a6697c824f5229b86f634e0c0ca635495bf | L1-79/79; source vs packaged dist child selection and Node/Bun loader |
| src/infra/runtime-process-entrypoints.ts | 18071b162512d6975ba7e281d4b2b152cc19f28b | L1-8/8; entrypoint manifest, SQLite child flag |
| src/infra/runtime-process-manifest.ts | 574271fcb7457427d90fbe9d94a57312f938bafa | L1-139/139; stateRead, workerNativeLifecycle, spawnBroker, sqliteReadOnly, sqliteSnapshotStaging, sqliteReadOnlyNativeResource entry mapping |
| src/infra/runtime-process-url.ts | d3cbb353e8156f53aca34ee1ccd8ddc15604d003 | L1-19/19; sealed deployment entry overrides generic dist resolution |
| src/process/spawn-broker/child-completion.ts | 5069bb0d44fd1130596c432b74d14a37a925f7ae | L1-52/52; native exit + IPC down + pipe close receipts |
| src/process/spawn-broker/child-diagnostics.ts | 215b7903ea022a8ac1c25f7bf91a1bb23bec3eb1 | L1-32/32; runtime error classification and redact stderr diagnostics |
| src/infra/sqlite-readonly-native-resource.types.ts | 00ea775c460558d67a7c15537cdbc7f7b5bdf78f | L1-55/55; allocated/retire owner replies and resource session/copy protocol |

Related directly imported source inspected previously but still not recursively closed: src/infra/sqlite-readonly-native-resource.ts@d9c025f54f23742666cb80fed1930af39cb15029 L1-358, src/infra/runtime-worker-generation.ts@21be874025d0df2e81988d86134199c79724b599 L1-116, src/process/spawn-broker/cleanup.ts@43a4c5888556869d82436a36750970152117ee96 L1-257 fetched/outline only, src/process/spawn-broker/ipc.ts@ef7830b750d31bacc017ee5253a74d7d83047564 L1-160 fetched/outline only, and native worker lifecycle and terminal process entrypoints. Need full read cleanup/ipc and broker server/worker real startup/host admission, then generated build provenance.

## C. New falsifiable oracles: source contracts, NOT executed in Actions

**PAIR-R20C-01 — broker admission vs physical process fact.** host.ts L46-59,L214-308 limits combined requests to 256; failed delivery can mark notStarted only in precise pre-native-initiation path. host.ts L576-610 binds prepare-spawn approval to the original admitted child; `owned` PID may arrive after local child close and still retain native cleanup. child.ts L77-96 makes markNotStarted owner-only; L136-180 delays terminal proxy events until native spawn callbacks. Counterexample: a timed-out or locally dropped prepare-spawn is NOT provably no side effect once a PID has been admitted; do not replay mutation without status reconciliation.

**PAIR-R20C-02 — broker loss and mutation outcome unknown.** host.ts L384-468 classifies IPC/channel failure after broker ready as command outcome unavailable. It rejects request delivery, disconnects the channel, retains native PID/process-group cleanup and refrains from reporting command rollback. child.ts L247-285 separates observed native exit from proxy failure: only previously observed exit stays an exit fact. Counterexample: transport failure following accepted work must be reported outcome-unknown; a local rejected Promise or closed IPC is not a database COMMIT rollback.

**PAIR-R20C-03 — resource claim and close are independent.** resource-host.ts L74-190 snapshots attachment before admission; owner messages have monotonic per-claim sequence and 256 MiB/1024-message IPC capacity inherited from ipc.ts. `release()` fails until `claim.closed`. L208-283 distinguishes `resource-failed` (callback only), `resource-close-error` (specific close attempt failed) and `resource-closed` (true receipt). Counterexample: native resource factory error, run failure or first close failure cannot mark directory disposed or release a claim.

**PAIR-R20C-04 — generation-bound private native authority is not device pairing authorization.** resource-protocol.ts L12-68 requires bootstrap or attachment endpoint, secret, generation, startupDeadline and typed ID; resource-authority.ts L8-34 allocates 0700 private directory, random 32-byte secret and platform pipe/socket. host.ts L470-483 rejects readiness PID or generation mismatch. These capabilities authenticate a **same-host broker native resource**, not an OpenClaw device key, role/scope, pairing request nor revocation. Counterexample: possession of a broker endpoint/secret may not grant pairing scope or impersonate a mobile/companion identity.

**PAIR-R20C-05 — pipes/exit/IPC settle independently.** child.ts L99-134 waits output close per FD; L198-245 distinguishes native `exit`, `closed` and IPC `disconnect`; finishClose L278-285 requires native exit, no IPC channel and no open pipes. child-completion.ts L3-51 joins exit + disconnect + tracked pipes even when parent IPC disconnect suppressed aggregate `ChildProcess.close`. host.ts L678-726 declines broker close with open native resource claims and awaits broker completion plus retained cleanups. Counterexample: kill signal, readable stream end or process error without PID exit does not settle all outputs/resources.

**PAIR-R20C-06 — exact runtime entry graph is environment-dependent.** runtime-process-manifest.ts L13-89 maps stateRead, native lifetime, spawnBroker, SQLite readonly/staging/native resource and other processes; runtime-process-url.ts L6-19 permits sealed bundle entrypoint; runtime-worker-url.ts L7-78 chooses installed dist root vs source sibling/package dist and Node tsx loader vs Bun --no-install. Counterexample: auditing source file path or one Node dev variant does not prove packaged Windows/macOS/Linux/runtime behavior or license/source closure; record the actual executable/entrypoint and artifact digest for each accepted product target.

**PAIR-R20C-07 — no hidden second mutation writer on Coordinator side.** Coordinator target `gateway_request_dispatcher.rs` L63-95 accepts registered handlers and rejects unknown methods; L131-245 validates method name, dispatches HTTP POST with Host bearer headers, supports 15-second default and classifies DNS/network/timeout/HTTP failures. These are transport facts, not operation idempotency or accepted COMMIT receipts. Gateway client targeted L146-210 and L234-385 tracks connection generation, SSE/reconnect and rejects unavailable transport; reconnect is not a pairing mutation reconcile. Counterexample: HTTP write succeeded, response lost: retain outcome-unknown and re-query durable Host status before any authorized retry, not automatically re-send the POST.

**PAIR-R20C-08 — target existing storage candidates are not authorization truth.** Target `agent_db_schema.rs` L7-30 defines only kv, legacy blobs and transcript_entries plus related indexes; L69-183 prepared statements operate those domain rows and no pairing approval/requestId/revision/scope/revoke schema is declared in this file. `connector_secret_store.rs` L39-133 owns agent/platform JSON connector secrets and filename-fenced read/write/delete; L45-69 returns empty record on missing/unreadable/malformed data. An empty record there must not be interpreted as an authoritative absence of previously COMMITted device pairing approval; there is no demonstrated role/revocation/version/generation contract. The future canonical Host security storage candidate requires source-neutral, account-fenced unique writer, schema, scoped authorization and ADR review or extension rationale. Counterexample: malformed connector secrets JSON must not authorize retrying or revoking a pairing record, and session KV must not silently become a second pairing database.

## D. Exact target file read ledger for new producer/consumer facts

| Target file at PR #169 H | Git blob SHA | Exact reviewed interval |
| --- | --- | --- |
| source/node-agent-coordinator/src/gateway/gateway_request_dispatcher.rs | 38ca19db76493f9c2bca8862b484d61b7e74f47c | L1-135 + L136-246/246, full |
| source/host/src/extensions/session/connector_secret_store.rs | d304c7cb153a03d0c07a9f310aa01a81e533edba | L1-148/148, full |
| source/host/src/extensions/session/agent_db_schema.rs | 3d381c12c55d8444caac81f61b64bd81efa5e2bd | L1-184/184, full |
| source/host/src/gateway_protocol.rs | 13b6a5bad55c133ed67e39bc94b7e500808a82a5 | L1-86/86, full |
| source/host/src/storage/agent_paths.rs | 5b023b86317d7f726945522d456ef1c2537c21b4 | L1-34/34, full |
| source/node-agent-coordinator/src/gateway/gateway_client.rs | 67c666d4b3ec716aca60763bb53c594f066d0660 | L90-215 + L216-385, targeted ONLY (1064-line source not fully read) |
| source/host/src/extensions/session/channel_store.rs | c1af306ecc5040132913a4eabe31e7447cd00244 | L1-135, targeted ONLY (339-line source) |

Earlier round20 owner/consumer matrix remains unchanged, including Host gateway method registry, Host bearer server, Host Core credential and Box secret owners, Coordinator main/renderer port, ProductionRenderer, CLI, device-agent and native messaging gateway. The above is a **scoped negative finding about examined files**, not a proof that no companion consumer exists anywhere or that current main has exactly these same methods. The live main's separately advancing FDN network enrollment must be re-audited under any later production integration lease.

## E. Remaining 14-package PostgreSQL provenance and scoped gate

Frozen pnpm-lock.yaml@80fb4d9fa985297a24ac88931ef534e85a40ba47. Versioned matrix and 13 source/tag/license relationships remain in immutable round19 note. Full pinned list: @types/pg@8.23.1, pg@8.23.1, pg-cloudflare@1.4.1, pg-connection-string@2.14.1, pg-int8@1.0.1, pg-pool@3.14.0, pg-protocol@1.16.1, pg-types@2.2.0, pgpass@1.0.5, postgres-array@2.0.0, postgres-bytea@1.0.1, postgres-date@1.0.7, postgres-interval@1.2.0, split2@4.2.0.

All 14 **actual published npm tarballs** remain unverified by byte-wise SHA512, included distributable LICENSE/NOTICE, dist or npm attestation: **0 verified published artifacts in the recorded evidence**. @types/pg exact 8.23.1 published-source origin unresolved despite DefinitelyTyped candidate commits; pg-protocol has tsc/prepublish generated dist outside the reviewed source tree; pg-pool 3.14.0 uses a different Git source commit than pg 8.23.1. This continuation performed GitHub code provenance reads only: no npm downloads, installs, package generation, executable verification, local/container build or release actions. GitHub Actions-only future exact-version tarball verification is still a required independent unit.

## F. Explicit blockers, review and authority accounting

- OCRM-BLOCK-GW-SCOPED-AUTHORITY-001: source/worker/gateway dynamic & transitive import, test/oracle, generated build and packaging closures still OPEN; positive scoped G01/G02 NOT ESTABLISHED. Broker cleanup/ipc/worker-server and source native entrypoint package graph need further exact-source review.
- OCRM-BLOCK-GW-INDEPENDENT-REVIEW-002: real agent tool status returned NOT_FOUND: tunnel-client not seen for >300 seconds; PR #169 submitted reviews query was empty in round20. No actual subagent nor independent reviewer ACCEPT may be claimed. The agent outage does not block further GitHub read-only tasks.
- OCRM-BLOCK-PG-NPM-PROVENANCE-003: 14/14 npm tarballs not byte-proven; typing origin and distributed license unresolved.
- OCRM-BLOCK-PAIRING-PRODUCTION-004: exact PR H lacks an audited unique Rust owner/writer, approved Host schema, current-state reconcile, typed incoming gateway/CLI/UI/companion callers, authenticated operation generation and end-to-end result/side-effect/cleanup contract. Candidate architecture decision is not implemented or accepted.
- Neither previous P0 run 39770034071 / artifact 12420959002 nor the round20 audit-only carrier demonstrate scoped G01/G02, author-independent review, user-facing production wiring, supply-chain provenance or OCRM AC01–24. This new child carrier is an independent target and has no claimed CI receipt.
- Only a new audit dossier file is authorized for this round20 continuation; parent note blobs, PR #169, production Rust/UI, shared manifests, inventory, ledger, RTM, source lock, workflows, admission flags and release remain untouched. Do not promote a source read to implemented, wired, verified or accepted.

**Next independently executable scope:** fully audit frozen spawn-broker/cleanup.ts, ipc.ts and worker.ts, SpawnBroker resource worker/server and native-process entrypoints, packages/normalization-core and fs-safe pertinent imports; trace target Host secure account storage and gateway method registration through actual external/companion callers. In an approved GitHub Actions-only provenance lane reconcile 14 exact tarball bytes and distributed LICENSE, then source-backed scoped G01/G02 + independent reviewer ACCEPT, recheck live branch lease, and only then implement Rust production code.

GitHub source refs: https://github.com/openclaw/openclaw/tree/ad8d12f126d29b0db204187adf5092db80294e70/src/process/spawn-broker and https://github.com/bhrumom/fabushi-desktop/pull/169 .
