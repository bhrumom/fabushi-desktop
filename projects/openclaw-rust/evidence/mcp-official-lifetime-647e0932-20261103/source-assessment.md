# Official MCP lifetime assessment

Task `647e0932-bea4-4113-9a61-65ae3db58955`; continuation of the bounded MCP assessment campaign. Exact target base: `78aa967cc9400bb5c207441b2a78dbf7021d596c`, tree `f62fc666ade7ad6f8dbf40e889e5ec100a74e974`. All source came from exact-object `git show`/Git search, not the mutable checkout. No DTR mutable paths or other review reports were read.

**Disposition: one coherent next bridge contract is supported: retire the official MCP edge's captured credential/completion callbacks and refuse a new vault write after its existing service has been disposed.** The credential and completion bypasses are concrete production wiring faults. The final-scope-await/write case is also source-supported, with narrower implications than a cross-account exploit. Implementation remains unstarted pending the root's shared-contract decision. No executable reproduction or verification is claimed.

Actual inherited configuration remains `gpt-6-astra` / xhigh; this assignment crosses asynchronous account access, resource retirement, callbacks and vault admission. No worker or model override was created. Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845` / tree `90c8aab39972da11cfad220361781d49e1bc6700`. No U source was directly read here and no inherited reading is credited as new review. Existing exact-U closing/lifetime evidence may support only that invariant, not official OAuth parity or backend authorization.

## Production chain and existing owners

The desktop or Host user action enters the production MCP adapter. `authenticateHostMcpServer`, account removal and plugin install call the current runtime's `ensureMcpManager()` and then the returned facade (`adapters/mcp-oauth.ts:375–427`). Desktop IPC reaches the same facade through `getManager` (`mcp-desktop.ts:9–12`). The real factory creates `OfficialMcpService` from the production adapter's official ports, delegates official IDs to it, and disposes it with the facade (`desktop-mcp-manager.ts:92–95, 211–249, 277–279`).

The existing runtime exclusively owns manager-attempt currentness. Reset clears `current` synchronously before calling facade cleanup; permanent disposal also sets the runtime latch. Its ordinary credential callback checks currentness at entry, after acquiring the account service, and after the token await. Its ordinary auth-completion observer checks before broadcast and again before Host refresh (`mcp-runtime.ts:69–105, 145–149, 188–205`).

Real account-transition cleanup reaches that reset: `production-root-provider.ts:66–87` supplies `context.requireMcp().resetMcpManager()` to `account-transition-cleanup.ts:48–70`. Account login also resets after login/account settlement, then refreshes (`cursor-auth-wiring.ts:145–153`). The official service therefore has a real retirement path, not merely a test-only disposal method.

Official state is the existing `OFFICIAL_MCP_INSTALLS` value. `official-mcp-vault.ts:10–23` creates a scope-specific `SandUserSecretsStore` at `official-mcp/<scope>.json`, with a fixed `() => scope` account-slot callback. `SandUserSecretsStore.upsert` reads its slot, encrypts/updates the value and persists by temporary-file rename (`user-secrets-store.ts:105–128, 148–190`). No root/global account selector is consulted by that writer after the scope-specific store is selected. Each official service has its own mutation queue; it does not serialize replacement instances.

The completion consumer is also real. The official `changed` callback directly broadcasts and calls `context.mcpHost.refreshMcp()` (`adapters/mcp-oauth.ts:257–260`). Root composition maps that to the current production MCP service's `refreshHostMcp` (`main-production-services.ts:754–756`), which calls its registered desktop refresh handle (`adapters/mcp-oauth.ts:344–345`). That handle forwards to the current Coordinator `refreshMcp` leg (`mcp-desktop.ts:8`; `adapters/mcp-oauth.ts:175–178`). It can therefore refresh the successor's Host/Skills view after the old manager has retired. This read stops at that existing Coordinator port; downstream implementation correctness is not newly claimed.

## Hypothesis decisions

### A. Official OAuth bypasses the existing credential-attempt fence — ACCEPT

The production official OAuth getter obtains `context.requireAccount().getAuthService()` and then calls that service's `getValidAccessToken` directly (`adapters/mcp-oauth.ts:250–256`). The same factory already forwards the ordinary manager token path through `options.getAccessToken` (`262–265`), whose exact-attempt checks are in the runtime. Official OAuth therefore bypasses an existing usable owner boundary.

Every official broker operation acquires a Fabushi token immediately before its broker request: `official-mcp-oauth.ts:24–30, 59–76`. This includes start, poll, acknowledge, cancel, refresh and revoke. A concrete trigger is an old official operation held while awaiting the account service, followed by actual reset and replacement. Its direct getter can then call the token method and return a current credential to the old operation. Another direct trigger is reset itself: runtime detaches the attempt, facade disposal calls official disposal, and official disposal invokes `oauth.cancel` for pending authorizations (`official-mcp-service.ts:368–371`). The raw official getter can newly acquire credentials during that cleanup despite the attempt already being retired.

Consequences proved by source: fresh credential acquisition/return and possible new broker request from an old callback. **Not proved:** remote acceptance of an old attempt/connection under a different account, a successful cross-account revoke, or a provider authorization bypass. Those depend on backend ownership checks not read here.

Minimum correction: use the production factory's existing `options.getAccessToken({backendUrl})` for official OAuth, preserving its null/error behavior. No new account identity, credential cache, authorization policy or token format is required.

Important limit: this establishes the existing MC-05 issuance boundary. A credential already returned to the broker adapter is already handed out; `request` itself has a subsequent await continuation before HTTP fetch. Do not claim that using the guarded getter alone prevents every possible HTTP dispatch after retirement or cancels admitted I/O. A stronger final HTTP-admission contract would require a separately explicit port-boundary check.

### B. Official changed bypasses runtime lifetime checks — ACCEPT

The raw production callback directly calls broadcast then Host refresh, without currentness checks. Concrete producers survive disposal:

- Poll starts while current; its held poll result fails after disposal. The catch path unconditionally calls cancel and then `changed` (`official-mcp-service.ts:345–363`).
- OAuth credentials commit while current; acknowledge is held; disposal happens; acknowledge completion reaches unconditional `changed` (`351–356`).
- Disconnect commits local removal and waits for remote cancellation/revocation. Disposal happens during that await; `Promise.allSettled` completes and `changed` still runs (`88–106`).

Clearing future timers does not cancel these entered continuations. Callback identity fencing belongs to the existing runtime, which already handles stale callbacks and synchronous reset during broadcast. Merely checking `service.disposed` before calling `changed` is insufficient for that second boundary because broadcast can itself retire the runtime before refresh.

Minimum correction: reuse one attempt-bound completion forwarding implementation for the existing facade observer and official change notification. A small callback added to the existing manager options may expose that existing behavior to the production adapter; it must not introduce another registry or lifetime flag.

Preserve payload semantics: official currently broadcasts `{status: "changed", provider: "fabushi-official"}` but calls refresh **without a completion argument**. The legacy observer passes its completion to refresh. A shared helper must preserve both forms explicitly; sending the official marker as a new Host completion payload has not been justified by this investigation. Keep the post-broadcast currentness recheck.

### C. scope returns after disposal and admits a fresh vault write — ACCEPT, narrowly

`OfficialMcpService.scope()` checks `disposed` before `await ports.getScope()` but not afterward (`32–36`). `mutate` awaits a final `scope()` comparison and then immediately calls the write port (`57–69`). The production scope getter has real asynchronous account-service and peek-token boundaries (`adapters/mcp-oauth.ts:245–249`).

A falsifiable scenario is: an install/instructions mutation has read and changed its old state image; its final scope read is held; actual login/reset disposes that service; a replacement for the same account becomes current; the held scope returns the unchanged account hash; the old continuation starts a new write of its old state image. The production vault has no manager-retirement check and uses the captured scope. With a successor write before release, the old write can overwrite the successor's whole official-install value. The old facade may subsequently return a disposed error from `servers()`, despite having written. Same-account login is a real reset path, so this does not require an invented cross-account ordering.

Reject the stronger assertion that this necessarily writes A's state into B's account. A changed hash is rejected by the existing scope comparison, and vault file/slot selection is explicit. The source-supported risk is retired write admission, including stale overwrite when the scope is unchanged.

A lifecycle-only correction can recheck the existing disposed latch after the awaited scope read and **at the actual pre-write admission boundary** in `mutate`, with no await between that final check and invoking `ports.write`. Do not rely solely on a post-await check inside `scope`: its caller resumes through another promise continuation. The current mutation image, revision policy, permission decisions and vault format remain with their existing owners; adding a retirement check does not authorize more TypeScript business logic.

Important limit: a write whose port was already invoked before retirement is already admitted. This report does not establish safe cancellation, rollback or serialization of that writer against a successor. `SandUserSecretsStore` performs later filesystem awaits, and no cross-instance versioned-write protocol was read. Do not label that remaining case solved by a new pre-write check.

## Proposed shared contract and feasible allowlist

The next narrow user-facing behavior is: **after account reset or MCP shutdown, an old official connector must not newly obtain credentials, announce itself as current, refresh the successor through its callback, or start a new local vault write. Current connector operations retain their existing outputs.** Scope this to the explicit credential-return, callback and vault-write admission boundaries above. A completed external effect is not undone by local retirement.

Suggested code allowlist, subject to root contract authorization:

1. `source/electron-main/mcp/mcp-runtime.ts` — reuse/expose the existing attempt-bound completion forwarding; retain one `current` attempt and the post-broadcast check.
2. `source/electron-main/adapters/mcp-oauth.ts` — wire official broker credentials and notifications through those runtime-owned ports; preserve event and refresh arguments.
3. `source/electron-main/mcp/official-mcp-service.ts` — only existing-disposed-latch checks after scope acquisition and immediately before a new vault-write invocation. Do not expand into policy, scheduling, permission or store redesign.
4. `source/electron-main/mcp/mcp-runtime.contract.test.ts` — add production-composed regression/control cases to the already-selected bundle.

Root owns the necessary narrow amendment to `mcp-account-listing-retirement.md` and any exact-subject receipt integration. No new runtime, store, workflow or framework is required. `official-mcp-oauth.ts`, `official-mcp-vault.ts`, `user-secrets-store.ts` and `desktop-mcp-manager.ts` should remain read-only for this minimal contract unless implementation uncovers a concrete prerequisite. In particular, do not solve already-admitted cross-instance persistence by casually adding a second lock/store in the vault bridge.

If root intentionally chooses only the two production callback corrections, C must remain explicitly open; that smaller batch must not claim official service retirement or vault admission complete.

Cleanup semantics must be explicit before coding: after runtime detachment, pending-authorization cancel attempts cannot obtain a fresh credential through the old callback. Failure/refusal of that best-effort cleanup does not prove the broker attempt was cancelled; preserve existing bounded server-side expiry/cleanup responsibility rather than inventing a replacement-account retry or retaining credentials in a new owner. No server expiry implementation was inspected here.

## Meaningful verification on the current selected bundle

Existing authored controls cover generic runtime token-await retirement and observer reset, but the production official wiring does not use those callbacks. `official-mcp.contract.test.ts` exercises a direct service with controlled vault/OAuth ports and account switching; its read cases do not exercise actual runtime reset or the default production adapter bypasses. It also uses real 2.2-second waits for polling, so copying those waits would weaken the new ordering oracle.

Use the actual `createProductionMcpOAuthPorts`/adapter, runtime, desktop facade and `OfficialMcpService` for at least one regression per changed boundary. Controlled account service, network and storage I/O are valid seams; do not replace the production official callback with a test-authored guarded closure. Existing store `reveal`/`upsert` and network calls can be controlled at their I/O boundary to avoid a new production factory framework. Keep the real production port construction in the subject. The factory starts legacy discovery warming (`desktop-mcp-manager.ts:179–191`), so the fixture must explicitly refuse/control unrelated network rather than allowing live I/O.

Required deterministic cases:

- Hold real official broker credential acquisition before account-service return and before token return. Reset/dispose via the real adapter, create a successor where applicable, release A, and assert no new old token request at the former boundary/no token return or broker fetch at the latter. Verify current B requests still forward the original backend URL and expected token. Include old cancel initiated by disposal. Preserve already-entered broker success/failure without replay.
- Enter a real official poll, hold its response, reset, then release failure and ready outcomes. Use a controlled test clock for the real poll callback, not elapsed sleeps. Repeat with acknowledge held after committed credentials and disconnect cleanup held after local removal. Old completions must produce zero new broadcasts and zero current Host refresh calls. Current completion retains the exact official event and empty refresh argument.
- Let a current official completion's real broadcast callback synchronously reset the runtime. The one broadcast may stand; subsequent Host refresh must be suppressed. A subsequent B completion must still refresh normally.
- Hold the last scope acquisition of an actual official mutation before its write. Reset and establish B for the same scope; record B's install image; release A; require zero A write-port invocations and unchanged B state. Include an ordinary current mutation that writes once. Include a separately held write already invoked before reset to prove the contract does not synthesize cancellation or replay after admission.
- Keep the existing MC/MR/MB/MD and official account-scope/revision controls. No provider credential, broker token or full vault payload belongs in diagnostics.

The current build already bundles `mcp-runtime.contract.test.ts` under `--plugin-skills-contracts` (`desktop/scripts/build-electron-runtime.mjs:91–110`), and the OCRM workflow runs that complete target (`rust-desktop-runtime.yml:4728–4746`). A new workflow is unnecessary. Any fixture compatibility issue must be resolved in the sanctioned Actions lane, not by local execution. No test counts, passes, package acceptance or real-account conformance are claimed now.

## Direct-read identities and limits

The parent supplied hypotheses and read locations, but all load-bearing conclusions above were independently read at this target. Ranges below are this assessment's actual read credit; searches/truncated surrounding output are not whole-file reading.

| Path | Blob | Read range |
| --- | --- | --- |
| `source/electron-main/mcp/official-mcp-service.ts` | `3da590f7908870f1c7ac7806326e6ed04093882c` | 1–373 |
| `source/electron-main/mcp/official-mcp-oauth.ts` | `41449071ecdfe32906fa082870e3c763aa0c2042` | 1–79 |
| `source/electron-main/mcp/official-mcp-vault.ts` | `ca1a8beed13059998958030d399278c3c7e9a78d` | 1–26 |
| `source/electron-main/mcp/mcp-runtime.ts` | `4236b468306aee00d727bbd59d27c15537fbf768` | 1–208 |
| `source/electron-main/adapters/mcp-oauth.ts` | `6653252d5f12ad3e345ee9104dd58c93734594df` | 1–465; 477–505 |
| `source/electron-main/mcp/desktop-mcp-manager.ts` | `17af10771dfaea32dec8626f5a9e3ab4d5be6ad8` | 1–282 |
| `source/electron-main/secrets/user-secrets-store.ts` | `ccec290256c6804de357850da6c8d97a2b18d755` | 1–192 |
| `source/electron-main/coordinator/account-transition-cleanup.ts` | `9a5d5701ceefbeb137b407a8891db06737577a8f` | 1–74 |
| `source/electron-main/coordinator/production-root-provider.ts` | `4c5a4f73315994da7258d1d605ba4f5a4d438605` | 55–94 |
| `source/electron-main/main-production-services.ts` | `d194b12a676ca93e992a3638e5d5dde286d03c53` | 740–766 |
| `source/electron-main/mcp/mcp-desktop.ts` | `98d57679b13b215fafe02fa61860edb95c660197` | 1–14 |
| `source/electron-main/account/cursor-auth-wiring.ts` | `21cea3702d46d78002ed550786c0ab0c83ef830a` | 135–158 |
| `source/electron-main/mcp/official-mcp.contract.test.ts` | `6bbf2b015d370e106df0288774c3af43e1b3f702` | 1–256 |
| `source/electron-main/mcp/mcp-runtime.contract.test.ts` | `c611c057f0dc57c35805adae759350d84c420ebe` | 1–182; 515–550; 605–670; 1540–1612 |
| `desktop/scripts/build-electron-runtime.mjs` | `8c5e5f1e35a3388a9032cbc72ef8ee6370119e1b` | 91–110 |
| `.github/workflows/rust-desktop-runtime.yml` | `e705a065dc34f066af6a095bcc7475d36da1f4e2` | 4710–4755 |

`AGENTS.md` was previously read in this campaign and exact-object diff confirms no change from the earlier baseline. Current account-listing-retirement spec clauses 81–144 and 157–190 were reread. Its existing MC/MB contracts explicitly leave official callbacks separate, so the recommended work requires an additive narrow contract rather than falsely declaring those old criteria failed.

Unresolved outside this slice: remote broker authorization/attempt ownership; account-service internal freshness between departure and runtime reset; credentials already handed out; HTTP/vault I/O already entered; cross-instance durable serialization; full official MCP policy migration into Rust; real provider/account/platform/package conformance. No new authority, ledger, source-review, global completion or acceptance credit is asserted.

This report is the only write. No product/spec/ledger/workflow edits, Git/GitHub mutation, dispatch, tests, builds, generators, lint or local/htch executable validation occurred.
