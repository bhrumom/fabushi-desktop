# Round 118 — Direct HTTP carrier / staggered reply lifecycle owner audit

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 87  
Stage: A, deferred executable validation

## Fixed source

- `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/details/mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9`
- `Telegram/SourceFiles/mtproto/details/mtproto_domain_resolver.h@cb1979bbbf0454135c8668ed1e5013da85d1dedc`
- `Telegram/SourceFiles/mtproto/special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88`
- `Telegram/SourceFiles/mtproto/special_config_request.h@91901489e778089b03187682a9556e929867b94f`

## Direct dispatch boundary

Both fixed-U owners set their dedicated `QNetworkAccessManager` to `QNetworkProxy::NoProxy` before dispatch. DomainResolver issues GETs only. The currently seeded SpecialConfigRequest attempts are Google, Mozilla and Firestore, so those reachable attempts are GETs as well; the source contains dormant RemoteConfig/Realtime branches and a POST path for non-empty payloads, but the reviewed constructor does not seed those attempt types.

All request descriptors and response normalization remain in the already-implemented pure owners; this round reviewed the point where a real HTTPS carrier must consume them.

## 800 ms stagger and winner semantics

Both sources use `kSendNextTimeout = 800 ms`, but their cancellation scopes differ.

DomainResolver stores attempts per A/AAAA key and schedules the next attempt against `Attempts.guard`. A non-empty parsed DNS response erases both the request collection and the attempt entry; destroying the guard fences a not-yet-fired delayed callback for that key. Empty/invalid normalized DNS response does not settle the race.

SpecialConfigRequest schedules the next attempt against `this`. Its successful response path does not clear `_attempts` or stop remaining scheduled attempts. While the object remains alive, other attempts continue and may also finish/apply.

## ServiceWebRequest lifecycle

`ServiceWebRequest` transfers its reply pointer on move construction. Move assignment calls `destroy()` on an already-held different reply, where `destroy()` disconnects `finished`, calls `abort()`, and schedules `deleteLater()`.

Ordinary destruction does not call `destroy()`; it only schedules `deleteLater()`. Therefore erasing a vector/map of wrappers is not source-equivalent to explicitly aborting every reply. Both request owners read the whole body and remove the finished wrapper even if the reply reports an error.

This distinction is now a required carrier contract rather than being flattened into a generic first-success/abort-all abstraction.

## Existing-owner-first audit

Canonical platform findings:

- `native/telegram-network/Cargo.toml@9ab75b9fa7d8a97f49ef4d627008727826337162`: only telegram-protocol, getrandom, thiserror, zeroize.
- `native/telegram-network/src/lib.rs@3743331b0b1453048a98a21ae4ac2d2b91a2b2a5`: synchronous `std::net::TcpStream` MTProto transport plus the pure request-normalization module; no HTTP/Web-PKI/timer/cancellable-reply owner.
- `native/telegram-network/Cargo.lock@d395652dcdffdafee02b8bcb1cc9aaf057ce1ac9`: no reqwest/hyper/ureq/rustls/native-tls HTTP/TLS stack.
- `native/telegram-runtime/Cargo.toml@a0d5aee63c886b11c3ad8fbf37885e0a3c5b6053`: no HTTP client or task/timer runtime.
- Vendored `third_party/mahayana/codex-rs/http-client/Cargo.toml@46afa51ba5d844e73894c743b7a4025263db0e7b` uses the Codex workspace reqwest/rustls/tokio stack; its README `89e35ac59d2dbf390d056de0aec88b14833962b1` explicitly describes a Codex-shared transport that resolves system/PAC/WPAD/environment/direct proxy routes. It is not promoted into the Telegram owner.

No reusable first-party native HTTP carrier/lifecycle owner exists at the reviewed heads.

## Disposition

Platform spec-first record: `2eade7264ce40c53e18eee75a21aa6c2a214ef05` / spec blob `91c92cadaf5df2db84eacb1111616af942851b67`.

Actual direct HTTPS dispatch requires a Web-PKI client, hard NoProxy mode, response headers/body/error, cancellable in-flight handle, timer/scheduler and shipping runtime integration. Adding that dependency graph would require deliberate native-carrier design and lock inputs; Stage A forbids dependency resolution and lock regeneration. This round therefore makes no production-code change and does not add a fake trait-only facade, unreachable race state machine, thread-sleep surrogate, system curl subprocess, or executable test against a nonexistent carrier.

Unblock condition: establish/approve a first-party reusable direct HTTP carrier in the canonical native networking layer, then implement the durable spec inside existing `telegram-network` and connect it to the shipping runtime.

## Ledger disposition

- `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001`: remains `mapped / pending / validation_status=not-run / accepted=false`.
- Pure request normalization and pure response normalization remain implemented separately.
- Remaining application work after the carrier is global HTTP-time mutation, temporary-DC `help.getConfig`, and `dc_options` replacement/change notification.
- Coverage remains `source_entries_total=16202, responsibilities_total=120, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

## Deferred contracts

The durable spec records the authored carrier contract matrix: hard NoProxy, injected-clock 800 ms stagger, distinct DomainResolver vs SpecialConfigRequest winner semantics, readable body with error metadata, exact per-reply finalization, owner-destruction cleanup, Date exposure separated from clock mutation, and no sensitive logging.

No executable contract file was added because no shipping carrier API exists to test. No contract was executed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
