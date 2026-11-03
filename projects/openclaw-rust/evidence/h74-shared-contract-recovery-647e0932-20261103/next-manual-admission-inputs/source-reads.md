# Exact source read record

U = `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` / tree `90c8aab39972da11cfad220361781d49e1bc6700`.
H = `bhrumom/fabushi-desktop@74f11d3d31c6245ac5886dc0181460b2d4349744` / tree `aca91e1cfbedeecb039d442fb3528caac89873a3`.

All listed product text was fetched in full through `GET /repos/<repo>/contents/<path>?ref=<exact commit>`; read extents below describe semantic review, not just acquisition. Blob identities came from exact contents-directory or Git-tree metadata. H metadata also used the integrator's verified exact tree through `GIT_NO_LAZY_FETCH=1 git ls-tree`. Local text search was a locator only. A broad local exact-tree grep reported missing unrelated blobs and is **not** evidence of complete caller absence. The upstream `src/gateway` contents listing stopped at 1,000 entries; exact Git tree `54d72aa9b2cd4f0c798e1ff899bf85be21c9bb5d` was read instead (`truncated=false`, 2,835 direct entries) for the required identities. No repository parser or generator ran.

## Fixed U

| Ref | Path | Exact blob | Semantic read |
| --- | --- | --- | --- |
| U1 | `packages/gateway-protocol/src/schema/cron.ts` | `dc5fdd96254399208eaec405e107219f40b397b2` | Bounded: imports, 114–139 ID-union helper, 611–703 run/history schemas. |
| U1 | `packages/gateway-protocol/src/schema/closed-object.ts` | `2e3cafba66532f38708eabdc6b2ac98b4e8cc236` | Complete, 1–12; `additionalProperties:false`. |
| U2 | `src/gateway/methods/core-descriptors.ts` | `852432d87eecbd5d68bb73b5435f23daa8862377` | Bounded: cron descriptors 299–328; run is admin/control-plane write. |
| U2 | `src/gateway/server-methods/method-authorization.ts` | `ac99d03f94f35de81ff15cfb70bb84bb5fbc911a` | Complete, 1–126; operator role/scope, signed management/runtime exceptions. |
| U2 | `src/gateway/method-scopes.ts` | `a0bd7baa2384890202a25ed146aaa227d2fc11cb` | Bounded: 1–90, 268–344; cron runtime scope, core-scope dispatch. |
| U2 | `src/gateway/operator-scopes.ts` | `46213c1aeb779f618139cf2824d2b6274ee74bdb` | Complete, 1–50; closed scope set. |
| U2 | `src/gateway/methods/core-method-policy.ts` | `2b64d1738783b5261d091ef40e7f34606fd1f0f3` | Bounded descriptor-to-scope lookup, 16–19 and 61–66. |
| U3 | `src/gateway/server-methods/cron-job-access.ts` | `28a75469df97ceaef4cc137510ee8477ee07038a` | Complete, 1–232; exact lookup, scoped handler, read-current guard. |
| U3 | `src/gateway/server-methods/cron-caller-scope.ts` | `1c8997c06e1c7859824cfb5ac84a711d932f1c4b` | Bounded: 40–65, 110–201, 216–294, 389–450; policy, commit guard, authenticated scope, ownership. |
| U3 | `src/gateway/server-methods/cron.ts` | `35cd3e5e36e26ccefba265c4955377fff0bc5ca0` | Bounded: 1–125 imports/entry; 506–669 complete run handler and grant wrapper through its finally. |
| U3 | `src/gateway/server-methods/cron-input-validation.ts` | `2d3671ec2d30b473eda64afdb179508a2b6cbd97` | Bounded: 502–567 complete caller-session wait/deadlock decision. |
| U4 | `src/cli/cron-cli/register.cron-simple.ts` | `ed18eba20a47260fba17a9b7b1b75fba372fb524` | Bounded: 1–120, 245–312; complete wait loop plus run request/result consumption. |
| U5 | `src/cron/service/ops-run.ts` | `475bb8ab9dfe552eeddeefda706d4c7b57f9c41a` | Complete, 1–204; preflight/queue/admission/ack/wait ownership and abort behavior. |
| U5 | `src/cron/service/ops-run-preparation.ts` | `485a4d6a1088215ca7a477ad622e5fc208bcf762` | Complete, 1–222; caller guards, stopped/disabled/spec/active/due decisions. |
| U6 | `src/cron/service/timer-runnable.ts` | `86dac8e47275b5892e5bea91f68631c2222cc94e` | Complete, 1–199; force, due, enabled, backoff and occurrence conditions. |
| U6 | `src/cron/service/state.ts` | `5bf41763e586fd75cb100dcc81aa2a3663871496` | Bounded: queued-manual-run field/initialization, 591–650 mode/result types. |
| U7 | `src/cron/service/run-queue.ts` | `e7d9fe5abfa76d3c6568eb5199e84616de07bb52` | Bounded: 1–210 complete request/adoption boundary. Later drain/launch/stop functions only located, not claimed deeply reviewed. |
| U7 | `src/cron/service/run-queue-requests.ts` | `b8e268be5513a37ea29acb164e51f31b45065ff0` | Complete, 1–76; receipt/run identity and preserveSchedule request projection. |
| U7 | `src/cron/service/run-queue-execution.ts` | `42045bf23ec71b0bd970f03acf113011be4ddd83` | Complete, 1–260; live eligibility, exact receipt, execution, finalization call and preserveCadence. |
| U8 | `src/gateway/server-methods/cron-runs.ts` | `ae36c78a58ee186880861340b8869493494c1d52` | Complete, 1–171; exact run filter, hydration, deletion/history visibility and disclosure guard. |

Supporting entry inspection: `src/gateway/server-methods.ts`, blob `0950c355d79f7ed84fa76f7dd2e831c7c6a4864e`, bounded 386–423 pre-dispatch authority and handler selection. `request-authorization.ts`, blob `3ae39bf1e8c6e4bebec79ec1176a44621bcafa6e`, import/entry locator only; not a completed authorization review. `src/cron/service.ts` full content acquired, bounded 170–197 wrapper reviewed; no complete file claim. Incorrect exploratory paths for old cron schema/ops/run-wait returned 404; the import-directed paths above supplied the actual contract.

## Frozen H74

| Ref | Path | Exact blob | Semantic read |
| --- | --- | --- | --- |
| H1 | `source/host/src/host_gateway_api.rs` | `127b340f120f931af778783aabcf338869fe0bc8` | Complete, 1–358; public registry, Automations ownership and native method. |
| H1 | `source/host/src/gateway_server.rs` | `2cc83cf387ae37c3b1c28663f72d9d0492e84ccd` | Bounded: context/trait 120–257; 990–1073 classifications; HTTP route/auth/dispatch 1717–1770, 1900–2055. Other auth branches were located but are not a complete ingress-security review. |
| H2 | `source/host/app/src/main.rs` | `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df` | Bounded complete relevant functions: 2966–3122 list/trigger; 4436–4475 dispatch; 5417–5433 adapters; 6078–6243 terminal/custody/session admission; 6245–6438 routed execution/fences; 8322–8338 Host-lane call; 13507–13545 paired target check; 13580–13590 entry; 14086–14141 native run; 14961–15019 context dispatch; 15233–15312 box predicate/Host fallback; 15326–15340 Mini App predicate; 18014–18020 lane consumer. |
| H3 | `source/host/src/extensions/transcript/automation_runtime.rs` | `044f53c8eb94924168cc7dd953f3732cd70fa668` | Bounded: 1–130 dispatch; 240–610 owner/store wrappers/lifecycle; 773–981 manual/background attempt; 1261–1311 mutation lock. |
| H3 | `source/host/src/extensions/transcript/automation_run_path.rs` | `a80ca919c1d05c30be29ad60e109f3a54f324e19` | Bounded: 106–240 results/args; 377–759 complete manual construction, duplicate gate, durable admission, execution/settlement; 803–860 exact settlement helpers. |
| H4 | `source/host/src/automations/automation_store.rs` | `57f39e545e525a36de3c7e9830970623862ee1fd` | Bounded: 1–170 checked reads; 173–385 definition/schedule/admission/finish operations; 509–512 format projection. Unread normalization helpers are not claimed complete. |
| H4 | `source/host/src/automations/automation.rs` | `4b27d9f5a7c4174a26e1244c50bef683d2c01482` | Complete, 1–88; run/config/record storage shape and ID producer. |
| H5 | `source/host/src/extensions/session/agent_session.rs` | `0a54da8349bc3edc9ef21c5f1c1862e13a9e3039` | Bounded: 886–919 actual store/admitted-mutation/list facades. ProductionSessionWorkers implementation itself not deeply reviewed in this assignment. |
| H6 | `source/shared/rpc/coordinator.ts` | `933bf07b62ca454af999c323e2591c3213e8194b` | Bounded: 277–303 native void RPC contract. |
| H6 | `frontend/src/recovered/features/automations/routines/controller.ts` | `1cbd18feb408b51589b271e5044f63ac9e9e879f` | Bounded: native source signature plus 245–295 run/refresh/event/reset. |
| H6 | `frontend/src/recovered/features/automations/routines/run-history-provider.ts` | `dd9a99fcceddd36400b1fbf02149601bf087c6df` | Complete, 1–203; history consumer over native snapshots. |
| H7 | `source/host/src/extensions/transcript/run_scheduler.rs` | `eb8994dc45b3722e8bfcc672e9e610c662429650` | Bounded: 1–265 queue types/state/admission/start; not full scheduler recovery review. |
| H7 | `source/host/src/extensions/transcript/send_turn_dispatch.rs` | `3ebae343888e43406d0e1e077f486ebb062d63eb` | Bounded: 207–321 enqueue/ticket/settle API; actual admission persistence needs further reading for cron enqueue design. |
| H7 | `source/host/src/extensions/transcript/sand_pending_wake_store.rs` | `50910a5b438b1eedeb980a23426929dbfa9e1df3` | Bounded: 1–190 kinds, durable marker and parse/error contract. |
| H8 | `source/mahayana/mahayana-rs/mahayana-feature-host/src/implementation.rs` | `98b665d390edfbada2112d902e51c9e38dfc27ea` | Bounded: complete `grok_gateway_call` 6416–6664; no manual-automation method in this match. |
| H8 | `source/mahayana/mahayana-rs/mahayana-unified-app-host/src/lib.rs` | `39966aec08778144f55b0adf17de8f2f0770c8d5` | Bounded: 227–253 delegation, 530–543 JSON dispatch. |
| H8 | `source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs` | `3b87324329177dc0df00b98c644dd893ae207f27` | Bounded: 284–332 complete gateway delegate/dispatch/handle; unknown method fallback. |
| H8 | `source/host/src/extensions/attachments/attachments_service.rs` | `75a49490ee80e7416df66df2431d2f3013f1b17c` | Bounded dispatch negative: 1636–2245 method labels and default-None contract, plus opening/closing arms. No media-body behavior claim. |
| H8 | `source/host/src/box/production.rs` | `96aa8c1ccf7ca2c0f8a5085a11279d29c113706e` | Bounded: 1–26; the only box fallback selector is `box.applyEnvironment`. |
| H9 | `source/host/app/src/tests/cron_job_identity_tests.rs` | `646d24ab3d27dede7bfcb4f5fe421585694c390b` | Bounded: 178–272 real-owner trigger seam; 395–428 supplementary wiring assertion/first control. |
| H9 | `source/host/tests/automation_run_path_contract.rs` | `5db2171d127c375f67b629e5fdfc6fa39556db5d` | Bounded: 284–343 complete barrier-driven duplicate control; other control names located only. |
| H9 | `source/host/tests/automation_runtime_contract.rs` | `d53183110694a71acb48c62bd74865d015e29a51` | Test names located; source acquired, no full behavior review or execution claim. |
| H9 | `source/host/tests/openclaw_chat_cron_node_contract.rs` | `b6a107dda12c867e57e7cd6ff38d245308f723ea` | Bounded: 636–669 complete stale source-anchor test. |

Other full acquisitions used only for locator/negative narrowing, with no whole-file claim: `production_runtime.rs` blob `9f15ddbdebe0ec7fe612975e28e9a7d5d0c3beb6`; `host_production_extensions.rs` blob `36b5dbac27d346f883150df179372034e0bea223`.

Governance identities: root `AGENTS.md` `e4682748087759b2d4bb2b3a47916b6148e8a6f2`; SOT `3748d734b10c9ed5b2d948467fbaa9483f1dd306` (current opening authority/constraints); main spec `69c9667af2328e20a1fba09ebf80c5aab4d5c336` (§17 671–720); module map `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3` (M04/M19 and ownership constraints); CJI spec `988f36d855e3efffc5b5563006dddf926393fcbe` (complete). Current task `P0-source-baseline-and-owner-audit.md` was acquired at H and its relevant automation history 176–280 reviewed; no live status conclusion was inferred from older receipts.

Reconstruct any source through `https://github.com/<repository>/blob/<U-or-H>/<path>` or the exact contents URL above. No dependency closure, platform/provider execution, HTTP/WS interoperability, packaged acceptance, grant lifecycle implementation, queue storage transaction/recovery implementation, or whole-file review beyond the explicitly complete entries is claimed.
