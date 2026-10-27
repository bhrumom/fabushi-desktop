# OCRM Gateway dispatch / hello / profile — pinned source, consumers & oracle audit (2026-10-22)


## 1. 状态、来源和证据边界

- Task: `GW-AUTH-01/02/03`, OCRM-M03/M04, 原始范围依主 Spec；本 dossier 是**非生产只读审计**，不作实现、验收或分母缩减。
- Frozen upstream: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`; Git commit tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`，未移动 baseline。
- Current target parent at audit: PR #169 branch `ocrm/p0-source-authority-20261022`, commit `93ddb09d1e440fc463c9068aac90cfb6633ec078`, tree `8b7a138de8152d6f131af633c43900a9692b7b5f`; baseline main `f6eb1758c59d65aba93e5608d7aba445e0bad42b`. This dossier's future carrier is **not** that parent; creation of a new commit requires independent exact-head evidence.
- Root `AGENTS.md` blob `e6a1f21ba5206cdc807b6f9738dad2cec2942100`, OCRM `SOURCE_OF_TRUTH.md` blob `4b9bdfe745da026ca7ebbca9ea82127d42017f0f`, scoped task blob `9757e8cc1c8968c10f50f240d799854ec47357d5`.
- Direct relative imports are identified from the **four** requested sources, including TypeScript type-only references and dynamic imports. The nine separately fetched frozen source/package subtrees were not truncated; this proves these references' **object identities only**, not their transitive source closure or semantic reading. Bare package imports / Node builtins / runtime-resolved plugin registrations need separate lock, review and provenance.
- Source `src/gateway/server-methods.ts` blob `5bebd34bd2b3e5765947fb58a5b71be98588c10a` (604 lines) and `src/gateway/server-methods/core-handlers.ts` blob `81acd1271c7184e3354e91ad52824c2a60d2408a` (228 lines) have been inspected for their import/dispatch structure **only**, not marked full-read. The listed 121 lazy handler files are **SHA-only**, not considered read or accepted.
- The current code shape remains distinct from accepted Rust parity. This pass performed **no builds, execution, tests or performance assertions** outside GitHub Actions.

## 2. Full semantic reading ranges

Below, “full” means all recorded contiguous ranges for the named source blob have been inspected in this audit or the same source read operation. It does **not** confer transitive readiness.

| Frozen source path | Exact Git blob | Audited ranges / total lines |
| --- | --- | --- |
| `src/gateway/server/ws-connection/authenticated-request-dispatch.ts` | `965068f52f6abd5945d0cd5f0e3ac35ae5c31fc4` | 1–110; 111–285; 286–443; 444–568 / 568 |
| `src/gateway/server/ws-connection/connect-hello.ts` | `45403cf7dac7a9414b05bac43f5d1f0458de8120` | 1–110; 111–273; 274–425; 426–533 / 533 |
| `src/gateway/server/ws-connection/connect-user-profile.ts` | `72bc221d53cecff121e63be212b992d5650e3b78` | 1–145; 146–251 / 251 |
| `src/gateway/server/ws-connection/message-handler-types.ts` | `06ce9ee4bd8c12be664b81de73baff821572a7e1` | 1–207 / 207 |
| `src/gateway/server/ws-connection/authenticated-request-dispatch.server-methods.runtime.ts` | `aa5f3ec7d8a3853fb2ee0964d9feb6415f613837` | 1–3 / 3 |
| `src/gateway/server/ws-policy-close.ts` | `ab125fddfac0f443397cba2512301d728687aaf3` | 1–147 / 147 |
| `src/gateway/expected-profile.ts` | `a5bb84710f3747b4b77e7a256d2b00c2e3e0dbf4` | 1–145 / 145 |
| `src/gateway/server/ws-connection/connect-operator-access.ts` | `e9e777538b720f424872b8e84fd515d8ac44cae7` | 1–112 / 112 |
| `src/gateway/device-pair-setup-completion.ts` | `e3c7b362ac0dab6f4d165c2be9138b5e723c765a` | 1–97 / 97 |
| `src/gateway/server-request-entry.ts` | `9c7ae2894d07fe1ff6fe7e4c8caad5182f3dfa3e` | 1–87 / 87 |
| `src/gateway/server/ws-connection/connect-auth-security.ts` | `a8fedd3c23b6a8c722334b6b44d97cb3ff3c1703` | 1–59 / 59 |
| `src/gateway/auth-policy.ts` | `8fcf9bc3834e4185e69fa530fa68426ac2586b09` | 1–212 / 212 |
| `src/gateway/device-revocation.ts` | `d4b14934c45c5cb63f127846bebc8c49744e36b4` | 1–304 / 304 |
| `src/gateway/server/ws-connection/request-start.ts` | `af5a47cf27a27ac150e1c8eac127034097112608` | 1–217 / 217 |
| `src/gateway/server/connection-transport.ts` | `c96a6a572db7bd9ef830a21f828bfb3263803d0a` | 1–54 / 54 |
| `src/gateway/server/client-registry.ts` | `8ae6ece4190ef5bd432e4f25db9cd9409ce75ab2` | 1–119 / 119 |
| `src/gateway/server/ws-connection/node-lifecycle-dispatch.ts` | `f0f950528175f7e526e79f052339a8f2f562948b` | 1–54 / 54 |
| `src/gateway/server/ws-connection/connect-policy.ts` | `fbcd9f470e079f31d20eef3fc2b332431cb2eea0` | 1–99 / 99 |

## 3. Authenticated request dispatch, authority & recovery semantics

- `authenticated-request-dispatch.ts:53–110,111–183`: validate the inbound request frame before dispatch; bind a committed config grant separately from invalidatable transport policy. Client invalidation can close a socket while keeping a preaccepted source grant when the committed grant has not changed; operator ingress, role access, device revoke and shared-auth session generation must each be fenced.
- `authenticated-request-dispatch.ts:184–285`: captured device/source identity and retained requester client survive response/disconnect until the **real authority owner** releases them. Credential-invalidating method set is exactly `device.pair.remove`, `device.token.rotate`, `device.token.revoke`, `node.pair.remove`. A `Promise.allSettled` barrier reserves their receipt order **before profile preparation yields**, and a later request waits on its predecessor. This is not blanket serialization of all completed handler work.
- `authenticated-request-dispatch.ts:286–443`: the response is either sent, error, unavailable or serialization-recovered, and can be suppressed after authority loss unless a held policy response is being settled. Unexpected role-error bursts close after threshold 10. Selected-account binding guards output independently of the transport; runtime delegated approval is checked again. Disconnect-triggered explicit abort is confined to `sessions.companion.ask`, `sessions.usage`, `plugins.reload(waitForDrain)`, and CLI `node.invoke`; ordinary accepted mutating requests are **not cancelled** by socket closure.
- `authenticated-request-dispatch.ts:444–568`: queue admission is bounded for operator, and role/source authority is checked again **after** queue wait and before handler execution. Handle errors/timeouts with typed retryable/unavailable distinctions; release handler entry, retained source and device-revoke holds in `finally`. Node `node.invoke.progress/result` use a distinct drain tracker; do not infer request completion merely because WebSocket closes.
- `request-start.ts:1–217`: work budget 1,024 queued/50 MiB and per-connection 256; control budget 1,024 queued/1 MiB and per-connection 16; four concurrent special preparations, maximum 30,000 ms queue wait, 64 starts per task turn. Queue start fairness and budget are separate from domain mutation commit.
- `device-revocation.ts:1–304`: exact captured owner + device/role + source identity cohort is retained. Accepted source authority survives ordinary disconnect, **not** explicit committed revoke, Gateway close, lost operator access or generation fence. New socket/ID may never be substituted into stale captured authority.
- `ws-policy-close.ts:1–147`: tentative policy invalidation fences transport; a committed source revoke notifies retained work. `holdGatewayPolicyResponse` must be claimed by the mutation owner; close may wait for held response completion to preserve its wire ordering; this does not create a second business writer.
- `expected-profile.ts:46–145`: selected account must match exact canonical profile and **must not silently follow merged aliases**; mismatch indicates `execution:not_started` or `execution:may_have_executed`, so unknown side effects require reconcile, never blind duplicate.
- `connect-policy.ts:1–99`: only the specified Control UI operator Tailscale device/auth-none cases bypass pairing; the node role is not granted that exception. Never equate trusted address or loopback with requested scopes or node command approval.

## 4. Connect hello, profile, and pairing publication semantics

- `connect-hello.ts:64–273`: authenticated principal recovery scope is a hash of verified principal and device; fallback recovery identities are separately scoped by device token, shared auth generation or device. Sensitive snapshot/capability disclosures are gated by role/scopes/origin; advertised methods, UI plugin surfaces, update details, and browser OAuth methods are not unconditional grants.
- `connect-hello.ts:274–425`: bootstrap setup token consumption and durable uncertain handoff happen **before** transport `hello-ok` write. If write fails, setup-completion marks delivery uncertain; generic bootstrap can be restored under explicit path; unknown delivery must reconcile through the owning durable record rather than replay the credential. `onHelloDelivered` is after transport delivery; `connection-transport.ts` defines transport send completion as **not a remote peer acknowledgement**.
- `connect-hello.ts:426–533`: node last-connection and first-connection effects recheck `pairingGeneration`; a claimed stale pending node cleanup is finalized and then `node.pair.resolved` broadcast. Post-connect health snapshot refresh intentionally avoids live probe on reply-adjacent path; presence commit/publication follows accepted hello.
- `connect-user-profile.ts:1–251`: admission takes canonical owner / verified GitHub / Tailscale / email profile as appropriate, prepares role authority and rechecks it across yields. Deferred avatar/profile sync may enrich identity, not replace admitted access grant. Missing mandatory role-bearing operator profile may reject, with shared-secret paths retaining their separate contract.
- `connect-operator-access.ts:79–112`: token-bearing connect re-reads pairing source identity to detect replacement credentials during async preparation; a mismatch yields `NOT_PAIRED` and 1008 close, not authorization via the replacement generation.
- `device-pair-setup-completion.ts:1–97`: separate durable setup consume, delivery confirmation, and `confirmed`/`uncertain` publication; broadcast loss itself is not a restored credential.
- Pairing durable state: from the preceding dossier's verified worker/source review, SQLite/worker mutation and committed receipts own pairing; in-memory publication is a revision/epoch-guarded projection. This **does not prove** a corresponding Rust authoritative pairing store exists on current Fabushi production.

## 5. Proposed falsifiable oracle frontier — NOT EXECUTED / NOT ACCEPTED

| Oracle ID | Trigger | Required owner-level result and negative assertion |
| --- | --- | --- |
| OCRM-GW-WS-O18 | Device revoke/rotate/remove after request accepted but before commit | Old source capture fails at owner commit, no stale mutation, duplicate replay or new-device recapture; nonmutating response may be suppressed |
| OCRM-GW-WS-O19 | Config candidate vs committed policy, shared-generation rotation and socket close | Candidate transport may close but already-accepted committed work follows **committed** grant; real revocation must fence retained source |
| OCRM-GW-WS-O20 | Two credential mutations with overlapping async profile prep; middle refusal | Receipt order preserves all predecessor barriers; failed middle request releases waiters without fake success |
| OCRM-GW-WS-O21 | CLI `node.invoke`, `sessions.usage`, `sessions.companion.ask`, `plugins.reload(waitForDrain)` disconnect vs ordinary mutation | Only listed request-owned waits abort; ordinary accepted mutation settles/reconciles under own owner, not cancelled or repeated by closed transport |
| OCRM-GW-WS-O22 | Queue saturation, 30s expiry, revoke while parked, restart entry sealed | Bounded rejection/fair scheduling; no post-revoke start; retryable refusal not interpreted as terminal side effect |
| OCRM-GW-WS-O23 | Selected profile switches/merges before handler vs after invocation | Fail closed; `not_started` vs `may_have_executed` remains distinct, latter requires reconcile and prevents blind retry |
| OCRM-GW-WS-O24 | Bootstrap token consumed, hello failed/lost/confirmed; double reconnect | Persist uncertain/confirmed with exact token/device/generation; no wrong-device delivery, replay or fabricated acknowledgement |
| OCRM-GW-WS-O25 | Node reconnection races stale pending pairing cleanup | Finalize only captured cleanup claim; re-check pairing generation and epoch, notify first connection once at authoritative owner |
| OCRM-GW-WS-O26 | Auth-none/Tailscale/trusted-proxy Control UI vs node role and remote ingress expiry | Only documented operator exceptions; node identity/scopes remain checked, proxy CIDR alone grants no node command |
| OCRM-GW-WS-O27 | GitHub/Tailscale profile sync lags role/identity change, avatar update after disconnect | Never change admitted grant, no cross-account alias migration or stale profile publish |
| OCRM-GW-WS-O28 | Held policy response, failed serialization, close while write queued | Handler and response custody settle separately; no unauthorized error or completion claim; response precedes deferred close when held |
| OCRM-GW-WS-O29 | Renderer AbortSignal settles local promise, Host already accepted mutation | UI preserves outcome-unknown and queries owner receipt; no duplicate side effect or second mutation writer |

Each oracle needs frozen baseline inputs, target production caller-to-owner-to-consumer trace, fault injection/positive+negative checks **on GitHub Actions**, receipt and independent verdict before upgrading scoped G01/G02, §11.1 or ledger/RTM.

## 6. Real Fabushi consumers / rejected false owner matches at target parent H

| Current target path (parent H) | Blob SHA | This round's evidence and what is *not* proven |
| --- | --- | --- |
| `frontend/src/main.tsx` | `ccb3e932cb9b017742c7d6a6ecb77c2faf5814d7` | Previous dossier: production renderer mount; not pairing writer |
| `frontend/src/production/ProductionRenderer.tsx` | `af8a83a8dd7cf1444474889ce3e64fe796a7d458` | Previous targeted production coordinator client and `sendPrompt` consumer; no native OpenClaw WS caller proven |
| `frontend/src/production/coordinator-client.ts` | `f120b9f600861560409d9a9cac6487cb5ec79e63` | Preload/IPC typed request; AbortSignal projection requires unknown-result oracle |
| `source/host/src/gateway_server.rs` | `1ce506ae0bc75ee96418e55eb64a935dd7865ac0` | Existing Rust HTTP/TLS `/api`, Bearer/Origin; **not** OpenClaw WS auth/role/device-pair parity |
| `source/host/src/host_gateway_api.rs` | `e93659285baf4903d3c97cf85629864341ad4126` | Existing Host domain method router; no OpenClaw pairing store ownership established |
| `source/host/src/network_enrollment_owner.rs` | `dac2ea3dfa439570de57722c23f583b160f15af9` | Existing Rust account/device network enrollment and reconcile; distinct from OpenClaw pairing canonical writer |
| `source/mahayana/mahayana-rs/mahayana-cli/Cargo.toml` | `5896c0ed152f71bb06cb55d24d213446b86d13ca` | Defines concrete `mahayana` and separate `mahayana-gateway` binaries, not a single unified Host gateway |
| `source/mahayana/mahayana-rs/mahayana-cli/src/main.rs` | `a0e0a98f20e7f24eb75dc13ae9bffdf5c4846b6e` | Targeted callsite survey: clap main line 515, CLI dispatch ~610, FeatureHost ~781, `device` ~683 |
| `source/mahayana/mahayana-rs/mahayana-cli/src/bin/mahayana-gateway.rs` | `660eca96a76f1bd0fb2ae05d7b03f60a579f24ca` | Actual stdio JSON-RPC 2.0 line reader, Rust local `dispatch_request` and peer approvals; **not** OpenClaw WS pairing |
| `source/mahayana/mahayana-rs/mahayana-gateway/src/lib_v2.rs` | `6e8e361312c2d80ca1ac04dc492f22b80b567fa8` | Local Rust replay/state gateway; not permission to bypass Host existing owner |
| `source/mahayana/mahayana-rs/mahayana-gateway-peer/src/lib_v2.rs` | `0371a98be4328c37b05398a1e6785b8daf89dfd5` | Local durable server→client approval registry; not durable device pairings |
| `source/mahayana/mahayana-rs/mahayana-cli/src/device_agent.rs` | `10029b40dcccd21c920b760b5deb234b93af90e1` | Device-agent WebSocket for Fabushi external MCP surface, not Host `/api` or OpenClaw node WS pairings |
| `source/product/fabushi/fabushi-official-device-registry.ts` | `461ec329caf850c5b494db38c62a2ed88ef5f140` | Prior dossier full 330-line separate external app agent surface; no authority overlap assumed |

Observed shipping UI→IPC→Coordinator→Host REST domain pipeline must remain canonical. CLI local FeatureHost/gateway and external agent surfaces have distinct entry/credential/receipt contracts. The unique canonical OpenClaw-equivalent pairing writer/storage migration, cross-surface caller policy, account switch, companion installation and all applicable UI surfaces remain **open**, and no new parallel state owner or Gateway process is authorized without ADR plus positive evidence.

## 7. Existing exact-parent GitHub Actions evidence; never infer scoped success

- PR #169 parent `93ddb09d1e440fc463c9068aac90cfb6633ec078` source-authority run `39756906818` attempt 1 `completed/success`; jobs `119751092991` (source-authority) and `119751555601` (persist-verified-inventory), both `completed/success`; checkout exact H, frozen upstream commit/tree.
- Log: canonical tracked inventory **58,750** rows, full reconstruction from **58** shard candidates and SHA256 `586cadaa199916ac9593782b59059bfdcf8c72a717e534a72225e7e883977153`; G02 negative probes reject duplicated path, unread denominator tamper, unbacked accepted verdict and orphan responsibility. **External dependency closure still open.**
- Artifact `12416755235`, `ocrm-p0-authority-93ddb09d1e440fc463c9068aac90cfb6633ec078`, digest `sha256:c7454b6f4737f6f22f653f30180aee8c144db18f1a4901275eda7faa1a29e1de`, 4,479,295 bytes. Download/reconstruction verification reported in persist job. This is **global P0 tracked-tree/structural** evidence only, NOT Gateway scoped G01/G02 or equivalence.
- This new doc will have its own H2 carrier; H's green run cannot be transferred to H2; a new exact-H2 run/receipt must be inspected before treating it as fresh evidence. No production Rust, shared interfaces, ledger, RTM or acceptance flags changed by this audit.

## 8. Blockers & immediate independent reading frontier

- `GW-AUTH-01` source scoped G01 **OPEN**: first-hop tracked identities now resolved, but real transitive recursive import/external-package/handler closure, related tests/fixtures, and complete code semantics remain unread. In particular `server-methods.ts` → `request-authorization.ts` → `session-group-catalog.ts` dynamic, `core-handlers.ts` → 121 lazy handlers, and `shared-types.ts` type/module imports are not semantically accepted.
- `GW-AUTH-02` owner/wiring scoped G02 **OPEN**: CLI/stdio gateway and external device agent are real and distinct, but actual full Fabushi Host pairing owner, renderer/CLI/companion end-to-end scope and revoke/recover consumers cannot yet be certified.
- `GW-AUTH-03` §11.1 / oracle / independent review **OPEN**: oracles O18–O29 proposed only, no positive Actions result or independent review. Subagent status probe: `NOT_FOUND: Tunnel-client has not been seen for 300 seconds`; no worker launched.
- Next independent read tasks: (a) full segmented `src/gateway/server-methods.ts`, `src/gateway/server-methods/request-authorization.ts`, `src/gateway/server-methods/core-handlers.ts` plus `src/gateway/server-methods/lazy-core-handlers.ts`, `src/gateway/server-methods/shared-types.ts`; (b) pairing mutation handlers `device-pair-setup.ts`, `devices.ts`, `nodes.ts` and declared tests; (c) target Host account/enrollment/binding composition and CLI/companion identity lifecycle; (d) run/verify scoped positive source/owner oracle on Actions after §11.1 admissions; (e) independent read-only reviewer when connection genuinely returns. No implementation or acceptance promotion until then.

## Annex A — four-entrypoint repo-relative first-hop SHA inventory (84 identities)

This is a **first-hop tree identity ledger**, not a semantically completed transitive dependency closure. Git tree sources: `src/config/@2db675eb2d224f5cc2e469d4a62be966185b3164`; `src/gateway/@0a0b92dd355ab0054af724ea469dd6571c0fa30a`; `src/infra/@39b6294cf55be73ddad9c70d349bec35444c287d`; `src/logging/@1a81c0536cc4b12f2b46171706d6dc88d3e618c1`; `src/process/@fbf8abbb2888b578b323f6772fb0e028e4c2867d`; `src/shared/@bba759b8470a8263f59559ae50e615e7a7816533`; `src/state/@7614b5b58285981a9cc0554fd94f4d53a3d17f8c`; `packages/gateway-client/@65dd7f194b8d33bb4cfd5b09d32b617325c57baf`; `packages/gateway-protocol/@765aa057d63a6d5e092990c8dd0383f5b9ac5717`. Every subtree result was `truncated:false`.

| Source path | Frozen Git blob SHA | Direct importing entrypoints |
| --- | --- | --- |
| `packages/gateway-client/src/request-error.ts` | `f9b285663f3e3081e0310df163f923b0a7ad0071` | `authenticated-request-dispatch.ts` |
| `packages/gateway-protocol/src/client-info.ts` | `e672eff7a9af5d20ed1878e5e78d82c4da13d323` | `authenticated-request-dispatch.ts`, `connect-hello.ts` |
| `packages/gateway-protocol/src/index.ts` | `034f25aa1e5e50d9da65ba680941eb17e182a103` | `authenticated-request-dispatch.ts`, `connect-hello.ts`, `message-handler-types.ts` |
| `packages/gateway-protocol/src/system-run-execution-context.ts` | `add971f80215add716b55419678f2a22958d816e` | `connect-hello.ts` |
| `src/config/control-ui-link-base.ts` | `42eb754a50bbe0e2083f9735091b6d4e067a7df0` | `connect-hello.ts` |
| `src/config/types.openclaw.ts` | `c8b1d6c8caa3bf5e7738af4f409820acfe8a8532` | `message-handler-types.ts` |
| `src/gateway/auth-policy.ts` | `8fcf9bc3834e4185e69fa530fa68426ac2586b09` | `authenticated-request-dispatch.ts` |
| `src/gateway/auth-policy.types.ts` | `2339801f196708647b5388953eb8b3d24f2b3fd4` | `message-handler-types.ts` |
| `src/gateway/auth-rate-limit.ts` | `ea61c1d4d0d9eb3d235edd8dbd0a1b695f6930b6` | `message-handler-types.ts` |
| `src/gateway/auth.ts` | `9769ec1697b1fb81812b987ca59a840c07fc0233` | `message-handler-types.ts` |
| `src/gateway/chat-attachment-policy.ts` | `1f2a594179b3ebbef31dbcbe50addaad33c5cb5f` | `connect-hello.ts` |
| `src/gateway/control-ui-identity.ts` | `e4e06535d4a33ec5f7f23512524c9318af3bcc66` | `connect-hello.ts` |
| `src/gateway/control-ui-plugin-tabs.ts` | `1e6e9f2b21b868e4638b194373f8f1545181f177` | `connect-hello.ts` |
| `src/gateway/control-ui-shared.ts` | `a5d6fd4ba02e61205d54ee2bfe46c811232ec23a` | `connect-hello.ts` |
| `src/gateway/device-pair-setup-completion.ts` | `e3c7b362ac0dab6f4d165c2be9138b5e723c765a` | `connect-hello.ts` |
| `src/gateway/device-revocation.ts` | `d4b14934c45c5cb63f127846bebc8c49744e36b4` | `authenticated-request-dispatch.ts` |
| `src/gateway/error-shape.ts` | `bd783c65606964da5f7b50f03133ecdd5598362f` | `authenticated-request-dispatch.ts` |
| `src/gateway/events.ts` | `e744fee3fcc47141b93e62b01d424306284b72ca` | `connect-hello.ts` |
| `src/gateway/expected-profile.ts` | `a5bb84710f3747b4b77e7a256d2b00c2e3e0dbf4` | `authenticated-request-dispatch.ts`, `connect-user-profile.ts` |
| `src/gateway/github-user-identity.ts` | `b9aa63fad520980ac6b79089f76e432465b21bdd` | `connect-user-profile.ts` |
| `src/gateway/health/presentation.ts` | `968f54448988c8c988f9b6a93f74463dd506df68` | `connect-hello.ts` |
| `src/gateway/ingress-attribution.ts` | `ede758a73245e70971750ad3b950af3dfcebd7a4` | `authenticated-request-dispatch.ts`, `connect-hello.ts`, `message-handler-types.ts` |
| `src/gateway/local-user-ingress.ts` | `546a5390eb36f10d65f01153f90d3daf92dcb649` | `connect-user-profile.ts` |
| `src/gateway/method-scopes.ts` | `1e5a8343fe315afa88eeb5ebeb3c96b569a670d8` | `connect-hello.ts` |
| `src/gateway/methods/registry.ts` | `e073cd9d480820c47662a786c90de9558a9d379e` | `message-handler-types.ts` |
| `src/gateway/node-connection-notifications.ts` | `720780457bdb9092f42be2b2b2021bdb4f8439f5` | `connect-hello.ts` |
| `src/gateway/node-pairing-auto-approve.types.ts` | `d84dcc905b1cc6094ac34820e08cf26cfc64e30b` | `message-handler-types.ts` |
| `src/gateway/node-reapproval-coordinator.ts` | `196cae2e1909f861bfc8bbd5e86e02ad86c40c36` | `message-handler-types.ts` |
| `src/gateway/operator-access-policy.ts` | `414a65ad1087491a539112d3807fcbf6db0a0f24` | `authenticated-request-dispatch.ts`, `connect-user-profile.ts` |
| `src/gateway/operator-role-policy.ts` | `5da2445fcab7a509191a8a79b5a415ac744f7ad6` | `authenticated-request-dispatch.ts`, `connect-hello.ts` |
| `src/gateway/payload-limits.ts` | `7be04a01d161292b1a4ef9e80786fd99dff2b781` | `connect-hello.ts` |
| `src/gateway/plugin-node-capability.ts` | `e7cbc33053a04e905c5af5dd829f4801a1bcbfea` | `message-handler-types.ts` |
| `src/gateway/provider-browser-auth.ts` | `36b42dda9d12497f41daab7b123dc65bf9ec7553` | `connect-hello.ts` |
| `src/gateway/role-policy.types.ts` | `5f40b9ac6f6f608d2c0038f60a0c57b3eaae62e2` | `message-handler-types.ts` |
| `src/gateway/server-connection-work.ts` | `6b0445c02fb54e1389e28ba1e147e6aee4898046` | `message-handler-types.ts` |
| `src/gateway/server-constants.ts` | `eec049a491adfef0d8c028352580f0006d741cb6` | `connect-hello.ts`, `connect-user-profile.ts` |
| `src/gateway/server-methods/session-mutation-guards.ts` | `6c304c09080c1eee4c10b4c6c4a10f5e381268df` | `authenticated-request-dispatch.ts` |
| `src/gateway/server-methods/types.ts` | `cbc2821dbc78a735fc3a32254cbb012c9b213a62` | `message-handler-types.ts` |
| `src/gateway/server-request-entry.ts` | `9c7ae2894d07fe1ff6fe7e4c8caad5182f3dfa3e` | `authenticated-request-dispatch.ts` |
| `src/gateway/server-shared-auth-generation.ts` | `9934ff2d22245e144eecb8dd458fa931d7d1267a` | `authenticated-request-dispatch.ts` |
| `src/gateway/server/client-identity-types.ts` | `45e43b40eab706ace5b5243ae49a3b16dcf2ff84` | `message-handler-types.ts` |
| `src/gateway/server/client-registry.ts` | `8ae6ece4190ef5bd432e4f25db9cd9409ce75ab2` | `message-handler-types.ts` |
| `src/gateway/server/connection-transport.ts` | `c96a6a572db7bd9ef830a21f828bfb3263803d0a` | `message-handler-types.ts` |
| `src/gateway/server/health-refresh-admission.ts` | `b22e706600b3a7210ec1feac71ed83c1c9014172` | `connect-hello.ts` |
| `src/gateway/server/health-state.ts` | `bb1456e56fa11fc7ec63a46c2fbf58ce7ddf8536` | `connect-hello.ts` |
| `src/gateway/server/ws-connection/authenticated-request-dispatch.server-methods.runtime.ts` | `aa5f3ec7d8a3853fb2ee0964d9feb6415f613837` | `authenticated-request-dispatch.ts` |
| `src/gateway/server/ws-connection/connect-admission.ts` | `f55361c35e36130c420ffa64c42817bedffd5fc2` | `connect-user-profile.ts` |
| `src/gateway/server/ws-connection/connect-auth-security.ts` | `a8fedd3c23b6a8c722334b6b44d97cb3ff3c1703` | `connect-hello.ts` |
| `src/gateway/server/ws-connection/connect-operator-access.ts` | `e9e777538b720f424872b8e84fd515d8ac44cae7` | `connect-user-profile.ts` |
| `src/gateway/server/ws-connection/connect-policy.ts` | `fbcd9f470e079f31d20eef3fc2b332431cb2eea0` | `message-handler-types.ts` |
| `src/gateway/server/ws-connection/handshake-auth-helpers.ts` | `bea19630a0b3a7d9340b7dc4e42d55a6acf1eed5` | `message-handler-types.ts` |
| `src/gateway/server/ws-connection/message-handler-types.ts` | `06ce9ee4bd8c12be664b81de73baff821572a7e1` | `authenticated-request-dispatch.ts`, `connect-hello.ts`, `connect-user-profile.ts` |
| `src/gateway/server/ws-connection/node-lifecycle-dispatch.ts` | `f0f950528175f7e526e79f052339a8f2f562948b` | `message-handler-types.ts` |
| `src/gateway/server/ws-connection/request-diagnostics.ts` | `abcb3f5a364597b8d6a309a325e34bee060ad1e1` | `authenticated-request-dispatch.ts` |
| `src/gateway/server/ws-connection/request-start.ts` | `af5a47cf27a27ac150e1c8eac127034097112608` | `authenticated-request-dispatch.ts` |
| `src/gateway/server/ws-policy-close.ts` | `ab125fddfac0f443397cba2512301d728687aaf3` | `authenticated-request-dispatch.ts` |
| `src/gateway/server/ws-types.ts` | `bfb206f9a56307615183ff58599c9b93aade2947` | `authenticated-request-dispatch.ts`, `connect-user-profile.ts`, `message-handler-types.ts` |
| `src/gateway/session-row-projection-access.ts` | `2c92f1c9e353f7cca73d989707f9a97188aa5ef5` | `connect-hello.ts` |
| `src/gateway/session-sharing.ts` | `1206c28162860aa50ecac8ab8b527e25c514c728` | `connect-hello.ts` |
| `src/gateway/stale-install.ts` | `7481293d68f4e6737aa68abdfe519f95c0599301` | `authenticated-request-dispatch.ts` |
| `src/gateway/ws-log.ts` | `f9a35ce09ff04dfd0bd088a7b82d4ef2fdc31108` | `authenticated-request-dispatch.ts`, `connect-hello.ts`, `connect-user-profile.ts` |
| `src/infra/abort-signal.ts` | `e19f1948e86007d8d27b9c05262621747ba0090d` | `authenticated-request-dispatch.ts` |
| `src/infra/crypto-digest.ts` | `27a01b719edbce93c317efb321cee570d1086d3f` | `connect-hello.ts` |
| `src/infra/device-bootstrap.ts` | `33bd2f3618c8270a02be8988af06fda9a38483f4` | `connect-hello.ts` |
| `src/infra/device-bootstrap.worker-types.ts` | `73baf4a19a89094fb09be64fea8a6fd14a7b7a4b` | `message-handler-types.ts` |
| `src/infra/device-pairing-access.ts` | `53889a9bed1994c1ca009f0df3dca366a774b907` | `message-handler-types.ts` |
| `src/infra/device-pairing-core.types.ts` | `2cc2a8adfc6b75ad9faf21506e1f8a1dbd07751f` | `message-handler-types.ts` |
| `src/infra/device-pairing-node.ts` | `3e032d10f1b2079a22784e8181d14f2eda8e4946` | `connect-hello.ts`, `message-handler-types.ts` |
| `src/infra/device-pairing.ts` | `ded452595bb22cdd5582d06cc7fe2ff9d4d6b609` | `connect-hello.ts` |
| `src/infra/device-pairing.types.ts` | `5a7413d40f0a96fff15da5588ecee53f427b25f7` | `message-handler-types.ts` |
| `src/infra/diagnostic-trace-context.ts` | `b4e02074bb1fe59d46a95330fe7cb5761408be3a` | `authenticated-request-dispatch.ts` |
| `src/infra/errors.ts` | `6ced4e1031396fe1cd57b79232ef33cad695ecdb` | `connect-hello.ts` |
| `src/infra/host-account-name.ts` | `83ccd70242c140bdf79e5bfd6ad463404bb33de6` | `connect-user-profile.ts` |
| `src/infra/system-presence.ts` | `39d917830ffc4f858e36cd823d12a988d8e527fb` | `connect-hello.ts` |
| `src/logging/subsystem.ts` | `0933267fa875c50c74318242455ccf29836e8d57` | `message-handler-types.ts` |
| `src/process/gateway-work-admission.ts` | `695465ddf131bb2be90d17f5fa22b01f6d3a7678` | `authenticated-request-dispatch.ts`, `connect-hello.ts` |
| `src/shared/deferred.ts` | `1d2a71bd573394a16409cc3a013d34264437a377` | `authenticated-request-dispatch.ts` |
| `src/shared/device-bootstrap-profile.ts` | `bed097a75b02ba623153cd36ca023d44d277b960` | `message-handler-types.ts` |
| `src/shared/lazy-runtime.ts` | `f5c61b538dd947db4cefb6888b949bf111ff9515` | `authenticated-request-dispatch.ts` |
| `src/state/user-channel-identity-operations.ts` | `64c1d7790400298871ace4df953ea6fcbbf8247c` | `connect-user-profile.ts` |
| `src/state/user-profile-list.ts` | `e8d1f2bed73f80dc7b4d200e5c26ebc7262e80ef` | `connect-hello.ts` |
| `src/state/user-profile-writes.ts` | `9a1e6ad80bfb328e317f57147d40bf00742c9dfb` | `connect-user-profile.ts` |
| `src/state/user-profiles.ts` | `55297c24a0187e603617ce528a3f212240df88c3` | `connect-user-profile.ts` |
| `src/version.ts` | `be4415b12657229051bea80f63118cae7fb1650c` | `connect-hello.ts` |

Direct referenced repository paths with no frozen blob: 0 (none). External/bare specifier forms sampled from entrypoints: `@openclaw/normalization-core/record-coerce`, `node:http`. Their pinned package identity and recursive transitive inputs remain open.

## Annex B — lazy Gateway method fanout from `core-handlers.ts` (121 dynamic references)

Source: `src/gateway/server-methods/core-handlers.ts@81acd1271c7184e3354e91ad52824c2a60d2408a`; module semantics **NOT** fully read. Path was normalized with `..` resolution against the frozen tree. `SHA-only; unread` explicitly does not count as read/mapped/implemented.

| Import source line | Dynamically reached frozen handler | Frozen blob SHA | Status |
| ---: | --- | --- | --- |
| 15 | `src/gateway/server-methods/agent.ts` | `ebad3015491b80d26274a79b83f1ab287d9556ae` | SHA-only; unread |
| 17 | `src/gateway/server-methods/agent-identity.ts` | `8de661efbaf6988fce44a8e2484e6e715e9cd133` | SHA-only; unread |
| 18 | `src/gateway/server-methods/agents.ts` | `11adafa77c7d48db3815599fa0094f673edabbe9` | SHA-only; unread |
| 19 | `src/gateway/server-methods/claws.ts` | `c90d86e65318e3833d43199000ab6f0d560ed74f` | SHA-only; unread |
| 20 | `src/gateway/server-methods/claws-drafts.ts` | `eb084252cd96c07ea7da75a423a59a07840afa27` | SHA-only; unread |
| 21 | `src/gateway/server-methods/claws-add.ts` | `49c922b54490edfbe920afcc389f9b6334914509` | SHA-only; unread |
| 23 | `src/gateway/server-methods/agents-workspace.ts` | `fa3eff5b3aeaa82f2e3164df69be6ed8a7367097` | SHA-only; unread |
| 24 | `src/gateway/server-methods/artifacts.ts` | `30fc55e2d88d41f9ef4df52822e42bbd815fb818` | SHA-only; unread |
| 25 | `src/gateway/server-methods/board.ts` | `b438d1feec20cbf25044fdad2c34f13aaa76f3fc` | SHA-only; unread |
| 26 | `src/gateway/server-methods/audit.ts` | `8ea2c5651eadb8af8cdde19a1c37adc510e3d92f` | SHA-only; unread |
| 27 | `src/gateway/server-methods/users.ts` | `4f9022a6638983fe0140bf6432181a9305401e80` | SHA-only; unread |
| 29 | `src/gateway/server-methods/users-mentionable.ts` | `3dd1a5a4a93b7a5b6632a0852aa9369cc477acfc` | SHA-only; unread |
| 30 | `src/gateway/server-methods/attach.ts` | `ab3bab0c6a8fb6032df596c0fb67eb9826755fb2` | SHA-only; unread |
| 31 | `src/gateway/server-methods/channels.ts` | `487f4748b7339c3e5aa00e18f89a5b32b79bdb54` | SHA-only; unread |
| 33 | `src/gateway/server-methods/channel-pairing.ts` | `64c26d2c69bcbf4366a151209efbb840c8b9fafe` | SHA-only; unread |
| 34 | `src/gateway/server-methods/chat.ts` | `809277d3ace5372801b9164e42a080be993f7460` | SHA-only; unread |
| 36 | `src/gateway/server-methods/chat-send-external-entry.ts` | `a5c99f9c7c69e47e32be37aa005fd7c3da9d8d22` | SHA-only; unread |
| 41 | `src/gateway/server-methods/chat-abort-handler.ts` | `a2383031c7c733a77f23a8b4f536bf7de7f2b84f` | SHA-only; unread |
| 44 | `src/gateway/server-methods/commands.ts` | `f93fc4505437cbde822742375057f090b1512139` | SHA-only; unread |
| 45 | `src/gateway/server-methods/computer.ts` | `7b2b4aab37def5106b972ffd943f1a0ae3e43ad6` | SHA-only; unread |
| 46 | `src/gateway/server-methods/config.ts` | `7d6bcfc37e954d3073a40bee3775cedb63c74e03` | SHA-only; unread |
| 47 | `src/gateway/server-methods/conversations.ts` | `e8bf425d6b9f870ae21a5cca635fe8de2d3ec745` | SHA-only; unread |
| 48 | `src/gateway/server-methods/connect.ts` | `3a107452f3ccdfa566d7110e2168ae9066ec3fa7` | SHA-only; unread |
| 49 | `src/gateway/server-methods/control-ui.ts` | `33761eff4c410a257efa6711ece210b58d88a783` | SHA-only; unread |
| 51 | `src/gateway/server-methods/plugins-control-ui.ts` | `e69cf221db1cfe5fbcbe4a2b205695f91cfde2f7` | SHA-only; unread |
| 52 | `src/gateway/server-methods/cron.ts` | `5af61dea384a7c51824228fb3c4fe08f48443b9e` | SHA-only; unread |
| 53 | `src/gateway/server-methods/devices.ts` | `ad29c673a9637e230b1ddb96e9b597e0426701fb` | SHA-only; unread |
| 55 | `src/gateway/server-methods/device-pair-setup.ts` | `9eb69926e1a65dfda7b1b4fd865c0db7e2882c30` | SHA-only; unread |
| 56 | `src/gateway/server-methods/diagnostics.ts` | `3c4eb4674b4116943045c47308647bb366a3d543` | SHA-only; unread |
| 57 | `src/gateway/server-methods/debug-proxy.ts` | `caf9788da883e243899a632606436fb4adb81c73` | SHA-only; unread |
| 58 | `src/gateway/server-methods/doctor.ts` | `16297370f03054c1de16963f1e32994c7d9f8161` | SHA-only; unread |
| 59 | `src/gateway/server-methods/environments.ts` | `6e5740af167b9aae8b6abb0eddb88df8c9353d18` | SHA-only; unread |
| 60 | `src/gateway/server-methods/worktrees.ts` | `523628659c36dbcfb2623d86ece12be7d7b70f4e` | SHA-only; unread |
| 62 | `src/gateway/server-methods/exec-approvals.ts` | `a2e563355a81ac75572293f7ce63d7af05c93f94` | SHA-only; unread |
| 63 | `src/gateway/server-methods/fs.ts` | `cc8605eb28001b8d058cc0e3e72c38b72514df5a` | SHA-only; unread |
| 64 | `src/gateway/server-methods/health.ts` | `93d4211725ccf1bd088c85e73f833b43c5d57cfd` | SHA-only; unread |
| 65 | `src/gateway/server-methods/logs.ts` | `435b1fb54f271deab624103838773b91fe8c0fbb` | SHA-only; unread |
| 66 | `src/gateway/server-methods/memory-search.ts` | `70ba98b5f133139b2d647892687a6ff3cebbe67e` | SHA-only; unread |
| 67 | `src/gateway/server-methods/mentions.ts` | `3573b4736bce55dde46b737a2dbe6adb9fd44975` | SHA-only; unread |
| 68 | `src/gateway/server-methods/terminal.ts` | `e0b49fbd7138ed74cdacdabdf260b5e56afb1ee1` | SHA-only; unread |
| 69 | `src/gateway/server-methods/transcripts.ts` | `ca0f6999b9ac14e5b0838709e5ebc3b91e8a3010` | SHA-only; unread |
| 70 | `src/gateway/server-methods/ui-command.ts` | `467e73c43f55090e367ff08fc5129c2cbb8d185b` | SHA-only; unread |
| 71 | `src/gateway/server-methods/themes.ts` | `fbe981a058b26ce0cf61a09dca1c3eafacb05b72` | SHA-only; unread |
| 73 | `src/gateway/server-methods/models-auth-status.ts` | `9c92aa7176025ffd84c5bb6a78e6eb6a65c8456e` | SHA-only; unread |
| 75 | `src/gateway/server-methods/models-auth-login.ts` | `6908e5edc63d9543a949f46f1ffdd21fbfd64abb` | SHA-only; unread |
| 76 | `src/gateway/server-methods/mcp-events.ts` | `b7ee7cba9c7cdec2a288498ed1033ac7eb80b19d` | SHA-only; unread |
| 78 | `src/gateway/server-methods/mcp-auth-login.ts` | `6299105ccf221429d0a855555feb7222ecfda8dc` | SHA-only; unread |
| 80 | `src/gateway/server-methods/models-auth-order.ts` | `2d7ad31f66f711cfc52c28f360ca29b3fbcae7db` | SHA-only; unread |
| 81 | `src/gateway/server-methods/models.ts` | `83ff81ac53186b6616ad73f6fa304847e6f18faf` | SHA-only; unread |
| 82 | `src/gateway/server-methods/models-probe.ts` | `4fec0851bb99c7caa1a603e9358b40899c15a1d7` | SHA-only; unread |
| 83 | `src/gateway/server-methods/web-search.ts` | `a950e424d31c87248e91813f5b33d5557eec02e0` | SHA-only; unread |
| 85 | `src/gateway/server-methods/native-hook-relay.ts` | `807f0b1a730ecffb7c0c122115284b1cc8917154` | SHA-only; unread |
| 87 | `src/gateway/server-methods/nodes.pending-work.ts` | `9f10c0f73ae6a6f1ea090428d582b947f6a16a22` | SHA-only; unread |
| 88 | `src/gateway/server-methods/nodes.ts` | `1cc6ed07ec84984768942f3bb15f70c401877749` | SHA-only; unread |
| 90 | `src/gateway/server-methods/plugin-host-hooks.ts` | `51762c3202043fb8e7c9d9ffe46cd443712762dd` | SHA-only; unread |
| 91 | `src/gateway/server-methods/plugins.ts` | `3f2d8e2bcd66a6ba5f201c1a4dcbd0ed6c2de49d` | SHA-only; unread |
| 93 | `src/gateway/server-methods/plugins-mutations.ts` | `7c96e50dda03cf836e7a518c93fe0d44bd7c3b33` | SHA-only; unread |
| 94 | `src/gateway/server-methods/projects.ts` | `8ceaac9f75107eae401cc136012d511f12981c62` | SHA-only; unread |
| 95 | `src/gateway/server-methods/portals.ts` | `418ccbce766e8b2006b84350e3f500329568ee90` | SHA-only; unread |
| 96 | `src/gateway/server-methods/progress-card.ts` | `ba8964157f3d636897b7fac4eff755202b2d7682` | SHA-only; unread |
| 97 | `src/gateway/server-methods/migrations.ts` | `db48ec8e55cbbed22a3ccd243453957a6dff4e2f` | SHA-only; unread |
| 98 | `src/gateway/server-methods/backup.ts` | `29a61b70cd7308195218b6a0f7878d54824a39ab` | SHA-only; unread |
| 99 | `src/gateway/server-methods/storage.ts` | `23a3968072368b403bbdce7b5090137fac54f6ef` | SHA-only; unread |
| 100 | `src/gateway/server-methods/push.ts` | `d77e476d4e75ef26beade8620d007b874a868f2e` | SHA-only; unread |
| 101 | `src/gateway/server-methods/restart.ts` | `fe7b9c1381e100012c1fdc0662d12cb8c394e58b` | SHA-only; unread |
| 102 | `src/gateway/server-methods/suspend.ts` | `b9d2dbbebcd055b67d187f15d0e31d5f6747c013` | SHA-only; unread |
| 103 | `src/gateway/server-methods/send.ts` | `2cb0c68a083d8e7913cba03a46ee0d1404c434f4` | SHA-only; unread |
| 105 | `src/gateway/server-methods/sessions-files.ts` | `257688688a2233a47cf053ebbe271debda3f5627` | SHA-only; unread |
| 107 | `src/gateway/server-methods/factory-github-proof.ts` | `f41e48856b907f241d3217199779cd15402fbcab` | SHA-only; unread |
| 109 | `src/gateway/server-methods/sessions-github.ts` | `e19f95ef823edb3a9702ce840b7585eaa5dd7791` | SHA-only; unread |
| 110 | `src/gateway/server-methods/sessions-diff.ts` | `9fc948f04de59972c130b8a73a3c48a7da76639a` | SHA-only; unread |
| 112 | `src/gateway/server-methods/sessions-abort.ts` | `daeb7c0b758d8085dc61debe8a45262c724f534d` | SHA-only; unread |
| 114 | `src/gateway/server-methods/sessions-compact.ts` | `285bf0373b2547ab7e660df375ad78cd9ca13197` | SHA-only; unread |
| 116 | `src/gateway/server-methods/sessions-create.ts` | `7b1438d32f015525eb15d0c69b55de88c3601fe5` | SHA-only; unread |
| 118 | `src/gateway/server-methods/sessions-title.ts` | `3c3eb8f8e6cbe760746c33a115deb8962be1b4d3` | SHA-only; unread |
| 120 | `src/gateway/server-methods/sessions-recover.ts` | `b3fb2b1e49590e666bd9274a661256253923d1e5` | SHA-only; unread |
| 122 | `src/gateway/server-methods/sessions-delete.ts` | `e12aa82b9a33b411d0106184adbaf6a45ef3c7c5` | SHA-only; unread |
| 124 | `src/gateway/server-methods/sessions-dispatch.ts` | `d36257c49bc61b5f3bf1cf59a884ba3fa788b2d4` | SHA-only; unread |
| 126 | `src/gateway/server-methods/sessions-groups.ts` | `09ddbf5c2a083417e7a0b50a416c3a4670597260` | SHA-only; unread |
| 127 | `src/gateway/server-methods/sessions-goal.ts` | `2e1a83a2cce09f869df04df8da92359aba35696b` | SHA-only; unread |
| 129 | `src/gateway/server-methods/sessions-provider-review.ts` | `f43c67b9ed77d495b9535fd4361d4c4eb8abfeb5` | SHA-only; unread |
| 131 | `src/gateway/server-methods/sessions-messaging.ts` | `8af2d9de1fd63aca75b7b14e0f05a32669f51085` | SHA-only; unread |
| 133 | `src/gateway/server-methods/sessions-mutations.ts` | `9fa0e9d74d47ee2687d86a5554eefb9e763cb967` | SHA-only; unread |
| 135 | `src/gateway/server-methods/session-processes.ts` | `b1455a7f54e47e84dd1844424e54057385db2065` | SHA-only; unread |
| 136 | `src/gateway/server-methods/sessions-read.ts` | `d525fb9ac43c3869fd2ef19681ca55feb9671df0` | SHA-only; unread |
| 138 | `src/gateway/server-methods/sessions-rewind.ts` | `4b72754d8499da9c6c2726c61157108d97c7d1aa` | SHA-only; unread |
| 140 | `src/gateway/server-methods/sessions-sharing.ts` | `9409a63bcd073f48de3157586b7a866a79e26897` | SHA-only; unread |
| 142 | `src/gateway/server-methods/sessions-subscriptions.ts` | `3c74496fb21cbb4ca1dfe8b2b7e33ad468d29ced` | SHA-only; unread |
| 144 | `src/gateway/server-methods/sessions-suggestions.ts` | `09cec70271f4a49bb52d3757f2d804dadb29b489` | SHA-only; unread |
| 146 | `src/gateway/server-methods/sessions-reactions.ts` | `1ddbcde0ae35442ab78cf8e4d1283be710f3d4eb` | SHA-only; unread |
| 148 | `src/gateway/server-methods/session-catalog.ts` | `88330393acc726c9df5fe918e1637cc91532bf18` | SHA-only; unread |
| 150 | `src/gateway/server-methods/session-discussion.ts` | `e838d3474bda57ccbf66980b783d5982425d5e13` | SHA-only; unread |
| 152 | `src/gateway/server-methods/session-activity-summary.ts` | `3459e4ddb244d8efa9af00068473f88b05435dcb` | SHA-only; unread |
| 154 | `src/gateway/session-observer-rpc.ts` | `68052ef5c4c8343eda05848268b5e86e369634c0` | SHA-only; unread |
| 156 | `src/gateway/session-companion-rpc.ts` | `f0374874d18dc18964718eab2a10910b09941d8d` | SHA-only; unread |
| 157 | `src/gateway/server-methods/hooks-status.ts` | `93364055c9620d624fdebfce02db150b1979d066` | SHA-only; unread |
| 158 | `src/gateway/server-methods/skills.ts` | `c92884f6e11b4e19cb721025ec7df60923251634` | SHA-only; unread |
| 159 | `src/gateway/server-methods/system.ts` | `b82d81e04211d7ec465f003429bf3bd216502db4` | SHA-only; unread |
| 160 | `src/gateway/server-methods/presence.ts` | `83689efe52f0a736b2434b7ba681906a2231f86d` | SHA-only; unread |
| 161 | `src/gateway/talk/handlers/index.ts` | `a24cb0958bfb93fe75e8bb8050918b1f5817641c` | SHA-only; unread |
| 163 | `src/gateway/talk/handlers/mode.ts` | `5450e38d535ff75b37066614350e2bc5abd36879` | SHA-only; unread |
| 165 | `src/gateway/server-methods/task-suggestions.ts` | `dec1c44dcd9014cf58d26e3af3b4e8fc4106d42b` | SHA-only; unread |
| 166 | `src/gateway/server-methods/tools-catalog.ts` | `1a781281d113715eda1f79369a5f8310c12d4da0` | SHA-only; unread |
| 167 | `src/gateway/server-methods/tools-github.ts` | `80bd28f8f06eaf371c8c57e153bc56f9b5c662e2` | SHA-only; unread |
| 169 | `src/gateway/server-methods/tools-effective.ts` | `ec2b716da65b1cfea44beea5e1daf6f856a80dcb` | SHA-only; unread |
| 170 | `src/gateway/server-methods/tools-invoke.ts` | `46c0f973a441b0246beed2c6b7233be7d35e9f6c` | SHA-only; unread |
| 171 | `src/gateway/server-methods/mcp-app.ts` | `1c18cf91d32b24297b01194c433b3170303f66aa` | SHA-only; unread |
| 173 | `src/gateway/server-methods/mcp-app-onboarding.ts` | `b2e392063baf40111f57d18ea5c3b40f039982de` | SHA-only; unread |
| 175 | `src/gateway/server-methods/mcp-app-extensions.ts` | `0bfa436b6b7a72b07c7cd31ba3a8c8127a1b86ce` | SHA-only; unread |
| 176 | `src/gateway/server-methods/canvas.ts` | `2f7c4fe6bc899d349fef5a9930c42171cc7f8c64` | SHA-only; unread |
| 177 | `src/gateway/server-methods/tts.ts` | `a97e14caa023cb44a953884b191fd714dd457cfb` | SHA-only; unread |
| 178 | `src/gateway/server-methods/update.ts` | `eeb48058b8a49687e0fd3b7ac168576d3471dc64` | SHA-only; unread |
| 180 | `src/gateway/server-methods/update-handoff-sentinel.ts` | `07b8685aad7033cc0b58730ad7b11601f36214d6` | SHA-only; unread |
| 181 | `src/gateway/server-methods/usage.ts` | `ba2520e3cc4e9259e5e80f0652732436af58b491` | SHA-only; unread |
| 183 | `src/gateway/server-methods/voicewake-routing.ts` | `9a18e4df3cc697fc6d993ef81d2d87c0c483746d` | SHA-only; unread |
| 184 | `src/gateway/server-methods/voicewake.ts` | `72ca0e91e59716972db5d74208967dff273d0a92` | SHA-only; unread |
| 185 | `src/gateway/server-methods/web.ts` | `548f7dbaf367b0a6d5649b248be4d3d6451268d4` | SHA-only; unread |
| 186 | `src/gateway/server-methods/system-agent.ts` | `a7bd2902f383e83e6c3bd21c01140cae335942db` | SHA-only; unread |
| 188 | `src/gateway/server-methods/system-agent-approvals.ts` | `1dbf253d343ceaaee8949010571c5d7bfce1615d` | SHA-only; unread |
| 190 | `src/gateway/server-methods/system-changes.ts` | `7b1abbdaa2fce9a1556ada70b91f72f95c5cf881` | SHA-only; unread |
| 191 | `src/gateway/server-methods/wizard.ts` | `9e69714f8acedaae27f63ff553b0300e5b319f45` | SHA-only; unread |

Dynamic targets without frozen blob: 0 (none). More recursion and runtime registry/conditional loaders are still unknown and unaccepted.
