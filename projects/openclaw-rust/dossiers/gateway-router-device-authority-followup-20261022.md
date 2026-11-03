# GW-AUTH-01/02 continued: pre-dispatch router, pairing commit and FDN owner (2026-10-22)

**Status: source-only audited subset; scoped G01/G02 = NOT ACCEPTED; `11.1 unit admission = false.** This is an additive OCRM-M01/M04 / `11.1 dossier, not an acceptance ledger change, implementation, or independent verdict. Task `projects/openclaw-rust/management/tasks/P0-gateway-rpc-authority-and-auth.md`; unit `OCRM-UNIT-GW-INGRESS-001`.

Frozen upstream `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. Investigated target PR #169 parent H=`be01835f532bf99da519fe23ec5e0bec903feb9c`, tree `dec351f25339590f10d6fd07aeb73dedcd37cdfd`; target main `f6eb1758c59d65aba93e5608d7aba445e0bad42b`. This document is a **later carrier/subject**, and H's receipt must not be inherited by the subsequent commit.

## A. Full-text reads completed in this continuation

All line intervals below were retrieved from exact frozen GitHub files. This table is a semantic **read frontier**, not a complete recursive dependency set, approved owner decision or evidence of Rust parity.

| Repository path | Blob SHA | Complete range |
| --- | --- | --- |
| `src/gateway/server-methods.ts` | `5bebd34bd2b3e5765947fb58a5b71be98588c10a` | 1–604/604 |
| `src/gateway/server-methods/request-authorization.ts` | `a6230b631de50d0ad282e108d0af5e62717cfcd2` | 1–661/661 |
| `src/gateway/server-methods/core-handlers.ts` | `81acd1271c7184e3354e91ad52824c2a60d2408a` | 1–228/228 |
| `src/gateway/server-methods/lazy-core-handlers.ts` | `4f71b2dbf2d50263b0df580b20dc1a2b919eb7ba` | 1–59/59 |
| `src/gateway/server-methods/method-authorization.ts` | `a4dda8be141ad623d9e2e668f51e2e6bf2227cd3` | 1–139/139 |
| `src/gateway/server-methods/session-mutation-guards.ts` | `6c304c09080c1eee4c10b4c6c4a10f5e381268df` | 1–569/569 |
| `src/gateway/methods/core-method-policy.ts` | `2b64d1738783b5261d091ef40e7f34606fd1f0f3` | 1–125/125 |
| `src/gateway/methods/registry.ts` | `e073cd9d480820c47662a786c90de9558a9d379e` | 1–127/127 |
| `src/gateway/server/ws-connection/handshake-auth-helpers.ts` | `bea19630a0b3a7d9340b7dc4e42d55a6acf1eed5` | 1–305/305 |
| `src/gateway/server/ws-connection/connect-auth.ts` | `708e34da5847a31d4d839b7917eeb65ae53c2496` | 1–608/608 |
| `src/gateway/server/ws-connection/connect-device-pairing.ts` | `57cf297e7d50f725ce657553972c8da17f214712` | 1–624/624 |
| `src/gateway/server/ws-connection/connect-existing-device.ts` | `595010b5c47ea3b16e34dc63ec59890e5a55735b` | 1–184/184 |
| `src/gateway/server/ws-connection/connect-pairing-approval-plan.ts` | `e83274a97c874c18964ff34e0e080ea47bfcf2f7` | 1–281/281 |
| `src/gateway/server/ws-connection/connect-device-tokens.ts` | `443df70b1e67d60b14c3041d4d2bef2e3b28f6d9` | 1–89/89 |
| `src/infra/device-pairing-access.ts` | `53889a9bed1994c1ca009f0df3dca366a774b907` | 1–22/22 |

Additional **target**, not upstream, full read: `bhrumom/fabushi-desktop@H:source/host/src/network_enrollment_owner.rs` blob `dac2ea3dfa439570de57722c23f583b160f15af9`, 1–714/714. The preceding dossier `gateway-dispatch-profile-source-frontier-20261022.md` already records the four requested WS entrypoint full reads, first-hop 84 blob identities and 121 lazy-handler blob-only fanout. Do not double-count those as new full semantic closures.

## B. Precise behavior / authorization contracts from newly read files

1. **Registration, namespace and lazy implementation** — `server-methods.ts:83–116` assembles core, plugin and auxiliary descriptors. Loaded plugin method names outrank local extra handlers, while duplicate descriptor names are rejected (`methods/registry.ts:22–85`) and unclassified core methods are forbidden (`methods/core-method-policy.ts:81–123`). `core-handlers.ts:14–206` maps approved method names to 121 delayed module references and caches both successful and failed imports; `lazy-core-handlers.ts:8–58` resolves forwarding functions while retaining entry lifetime. All 121 handler blobs are identity-only, not read complete.
2. **Pre-dispatch and accepted ownership** — `server-methods.ts:168–259` admits root/continuation work before dispatch and applies control-plane write budgets. `request-authorization.ts:206–288` reauthorizes role/scope, upload policy, selected profile and gateway startup. `request-authorization.ts:308–419` distinguishes startup pending writes from bounded 20-second read discovery and retries authorization after each configuration/projection yield. `request-authorization.ts:423–659` protects session group catalog, target-row identity, synchronous consume, node pairing freshness, and session admission before a handler starts.
3. **No authorization gap during lazy import** — `server-methods.ts:372–403,476–554` rechecks source, operator, selected profile, session and entry guards **after** lazy preparation, before release/`markInvoked` and actual invocation; shared-read reuse is constrained by its own invocation guard. `method-authorization.ts:39–139` implements role, scope, limited synthetic-approval/cron alternatives; node-role acceptance there is **not** proof that node method execution has passed its separate pairing/command authority checks. `session-mutation-guards.ts:212–271,274–362,365–442,478–568` distinguishes worker/native source lifetime and retained accepted-work authority, with commit-time session/target assertions. A disconnected transport does not automatically roll back an accepted mutation.
4. **Authenticated handshake / signatures** — `handshake-auth-helpers.ts:24–60` isolates browser-Origin rate-limit keying. `connect-auth.ts:335–398` requires device ID derived from public key, signed timestamp within 120 seconds, live nonce equality, and a verified v3 or v2 signature. `connect-auth.ts:400–525` then authenticates remote ingress/shared bootstrap/device-token paths, caps ingress scopes **after signature**, and fences shared-auth generation. Missing-device paths clear unbound claimed scopes except specified local paths (`connect-auth.ts:268–325`); remote proxy or same IP is not a free node privilege.
5. **Pairing approval, live revalidation and token issue** — `connect-pairing-approval-plan.ts:86–142,144–279` distinguishes local-grade, constrained trusted-CIDR node and owner-verified proxy/remote ingress/bootstrap lanes, with exact identity/role/scope caps. `connect-device-pairing.ts:162–257` checks same-key live grants and allocates versioned pending requests; `:284–343` rechecks current authorization/config/pending identity at approval commit; `:404–521` fetches the **current** durable paired record before allowing handshake continuation. `connect-existing-device.ts:55–108` pins platform/family, role and approved scope; `:110–165` checks bounded bootstrap handoff. `connect-device-pairing.ts:524–612` distinguishes local/remote pairing, live token-issuance guard and no reusable remote-ingress token mint; `connect-device-tokens.ts:35–85` ensures token issues remain device/role/scopes/issuer-generation bound. Metadata access patches and timestamp coalescing (`device-pairing-access.ts:4–20`) cannot substitute for explicit authorization.
6. **Target owner election remains OPEN** — `source/host/src/network_enrollment_owner.rs:54–136,242–359,362–559,570–714` persists *Fabushi FDN network-enrollment* grant and worker-generation state per user/device, invokes `api.ombhrum.com`/FDN grants, runs an owned tsnet worker and reconciles `OutcomeUnknown`. Its account/device-scoped JSON state, supervision and cleanup do **not** constitute the OpenClaw device-pairing request/approval/token/revocation SQLite/worker owner. The actual existing Host Gateway transport/router and any appropriate canonical Rust auth writer must be traced and selected without inventing a second competing writer.

## C. Newly acquired recursive dependency identities — SHA only unless in A

These are concrete next-hop dependencies from the above imports and frozen commit; **not** asserted full-reading or a complete closure.

| Path | Blob |
| --- | --- |
| `src/gateway/methods/descriptor.ts` | `f36351dfe5d1db8430f3f467d35190344633deb1` |
| `src/gateway/server-methods/gateway-client-identity.ts` | `d7bf828d12d055a669ed9bc6257523559bf1c960` |
| `src/gateway/session-group-catalog.ts` (dynamic `request-authorization.ts:424`) | `ea8d8c61139e1bf435a312fb9c00ae76789377c4` |
| `src/infra/device-pairing-approval.ts` | `708db58a83f4d743054f70e2208a4fc4cc08bb3b` |
| `src/infra/device-pairing-resolution.ts` | `53f26d60aa67be1cc546e81ca244440ca79e4e62` |
| `src/gateway/methods/registry.ts` | `e073cd9d480820c47662a786c90de9558a9d379e` (full A) |

More direct imports (including source `server-methods/shared-types.ts@253c1ac4e95c8e2c5be076fefde2ae20878881ad`, 740 lines), tests, generators, package `@openclaw/normalization-core` exports, plugin runtime registrations, session-authority stores and all lazy-handler transitive paths remain **unread or insufficiently locked**. Names, objects and counts are not closure predicates.

## D. Proposed falsifiable source/Rust differential oracles — NOT RUN

- **OCRM-GW-WS-O30:** Duplicate core/plugin descriptor or unclassified handler must fail before exposing handler/scope; no fallback dispatch through an auxiliary admin method with a protected name.
- **OCRM-GW-WS-O31:** Lazy handler import rejects/caches and crossing async config/identity revocation must fail before actual handler invocation; no optimistic `markInvoked`, no side-effect or stale receipt.
- **OCRM-GW-WS-O32:** Database startup refuses pending writes before execution with retryable/unavailable **not** outcome-unknown; timed-out read discovery does not grant stale session access.
- **OCRM-GW-WS-O33:** Node role may pass method-type gate but pending/revoked/stale node pairing must fail its NodeRegistry and command-capability gates; no node action by operator shared scope alone.
- **OCRM-GW-WS-O34:** Device nonce replay, device/public-key mismatch, timestamp skew beyond ±120 seconds and malformed v2/v3 signing payload all reject; scope capping cannot validate a forged signed scope.
- **OCRM-GW-WS-O35:** Browser-Origin loopback, trusted proxy or CIDR claims without approved role/node command surface never silently grant mutable node permissions; narrow documented local exceptions retain their explicit conditions.
- **OCRM-GW-WS-O36:** Auto-approval plan followed by revoke, key rotation, config/auto-approve toggle, changed pending request or device metadata must be rechecked at committed worker approval and live pairing read; no stale token issuance.
- **OCRM-GW-WS-O37:** Bootstrap handoff and hello delivery remain separate durable outcomes; disconnect/timeout after accepted mutation requires original-owner reconcile, not blind retry or general socket abort.
- **OCRM-GW-WS-O38:** Target FDN enrollment, Host `/api` Gateway, stdio CLI gateway and external device-agent maintain distinct credentials/storage; an OpenClaw pairing mutation must name **one** canonical Rust writer with full UI/CLI/device consumer reachability before G02 acceptance.

## E. Current exact-H evidence, external block and next-ready frontier

H=`be01835f532bf99da519fe23ec5e0bec903feb9c` had GitHub Actions `39760252203` completed/success with `source-authority` job `119762361740` and `persist-verified-inventory` job `119762799413` successful. Logs verified exact checkout H, canonical 58,750 paths, 58 persisted shards, SHA256 `586cadaa199916ac9593782b59059bfdcf8c72a717e534a72225e7e883977153`; negative **structural** G02 probes reject duplicate path, unread denominator tamper, fabricated acceptance and orphan responsibility. Artifact `12418150470` `ocrm-p0-authority-be01835f532bf99da519fe23ec5e0bec903feb9c` metadata digest `sha256:06759fd4b6510583e39ab186aff26985e6862768432efd2a9aecd2930f5b339e`. This proves **P0 tracked/structural** gates on H, not complete dependencies, Gateway scoped G01/G02, Rust differential equivalence or production acceptance. New carrier/target must obtain its own receipt.

Subagent `agents.status` again returns `NOT_FOUND: Tunnel-client has not been seen for 300 seconds`; no independent reviewer or worker started, no user action is established as necessary and no duplicate support mail is sent. Remain `GW-AUTH-01` (transitive source closure), `GW-AUTH-02` (unique pairing Rust writer / full consumer chain), `GW-AUTH-03` (positive scoped Actions oracles and independent review) **OPEN**; `unit_authority_ready=false` and all ledger/RTM/flags unchanged.

**Immediate next ready action:** complete frozen full read of `src/gateway/server-methods/shared-types.ts`, `src/gateway/methods/descriptor.ts`, `src/gateway/server-methods/gateway-client-identity.ts`, `src/gateway/session-group-catalog.ts` and device approval worker chain. Cross-reference actual Fabushi Host Gateway pair credential writer/storage and UI/CLI/companion revoke/reconnect consumer before any production Rust edit. Execute all validators only in GitHub Actions, and preserve exact-H/H2 separation.
