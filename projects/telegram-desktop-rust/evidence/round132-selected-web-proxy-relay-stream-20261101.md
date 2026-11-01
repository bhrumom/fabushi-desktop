# Round 132 — selected WebProxy relay stream/window/bootstrap carrier

Date: 2026-11-01
Task: `7457eb1c-11a3-4626-8563-a3e495872536` round 101
Stage: A — implementation, zero executable validation

## Authority

- fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- desktop start: `b5a68f143e9736c023a274a77ea72beb27f0a842`
- platform-core start: `fb44f2b7170558560833a4f2e6cea0533b8e8d56`
- platform spec-first: `c9aafb3332bea046bc73de00e781213ddd064c1b`
- platform initial relay production: `994841d0a4a342f61c5f960da9f55b4d434d1305`
- platform runtime bootstrap publication: `ceff666e3fe7b5b5e48ac00ded76ee263a0e33a4`
- platform final lifecycle correction: `9a8e1c552e3ca9a611e5e0e7b845b845b22854ac`
- desktop production: `8f5e43b02c5cbd81f55f04f218ac7909763fdb6a`
- desktop lifecycle mirror: `324add75e62fef572a68b038ac7d3a5a04a93451`

## Fixed-U reread

The relay/frame/socket/browser chain was manually reread at fixed U:
- `Telegram/SourceFiles/mtproto/web_proxy/web_proxy_frame.h@a5ba491fed67d70aee88e6ec2e3ac47c991e78f3`
- `Telegram/SourceFiles/mtproto/web_proxy/web_proxy_frame.cpp@bd1d7b310f40203f9da914efc8bdd6711035884b`
- `Telegram/SourceFiles/mtproto/web_proxy/web_proxy_transport.h@f58850d2bc2ceabd93113d7725903e1c0d92d71b`
- `Telegram/SourceFiles/mtproto/web_proxy/web_proxy_transport.cpp@e25ad970e84686e9aca061d02c274048793b5ad4`
- `Telegram/SourceFiles/mtproto/web_proxy/web_proxy_webview.cpp@8a2efc8fb15e400366f199cd956effc8ad362da8`
- `Telegram/SourceFiles/mtproto/details/mtproto_web_proxy_socket.cpp@9aa05e7034031310de4aa9d410fbb15d21c63ee8`
- `Telegram/SourceFiles/mtproto/connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`
- `Telegram/SourceFiles/mtproto/mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef`

Fixed-U facts used by the rewrite: exact 8-byte frame header; 24-bit stream ids; 1 MiB payload and 4096-frame batch ceilings; 4 MiB send/receive credit; 64 KiB Data chunks; Window returned only after local reads; 256 KiB/20 ms Window coalescing; Welcome-before-stream traffic; bounded closed-stream tombstones; loopback browser fallback and MessageChannel bridge; 32-byte secure nonce/capability; fail-closed carrier loss.

## Existing-owner-first implementation

No second Telegram runtime/provider/settings/session owner was created.

Platform `telegram-network` owns the transient relay/browser carrier. The final source blobs are:
- `native/telegram-network/src/lib.rs@450d6754908647b3c0e039ce95da94bb1a2fdf8f`
- `native/telegram-network/src/web_proxy_relay.rs@8f6297bace3318c099588e0d6549b65fdeeb3fd2`
- `native/telegram-runtime/src/runtime.rs@3a3da0eb4bf004569a3989e05e55c8b86a199c41`
- spec `docs/specs/telegram-web-proxy-relay-stream-boundary.md@76d0fe82f2d7da37326ad9f350674264af5f8576`

`TransportSocket::Web` presents a logical Read/Write stream beneath existing MTProto obfuscation/framing. `NetworkAttemptControl` can cancel Web logical streams. RuntimeClient prepares/retires relay state from the existing transient selected-proxy projection and returns only a loopback bootstrap URL.

Desktop mirrors the provider blobs and keeps ProductSettingsOwner v2 as the only durable owner. MainEdge `d9ffb976fcdfb7b6741e47e237f6e11e7110628d` strictly validates the returned loopback URL and reuses the existing system-browser shell. The source-neutral Settings IPC/Host contracts are unchanged.

## Manual corrections before bookkeeping

Source-only review found and corrected:
1. partial relay input must survive browser binary-message boundaries;
2. browser carrier writes must be serialized;
3. relay registry/lifecycle must not depend on a blocking accept thread accidentally holding Arc ownership;
4. browser carrier replacement must clear/close the old generation first;
5. disabled->enabled or IPv6-only changes for the same Web endpoint must not retire the newly prepared relay.

No executable validation was used to discover or confirm these corrections.

## Authored contracts — not executed

- exact relay frame vectors, malformed Window/unknown type and incomplete-input retention in `web_proxy_relay.rs`;
- browser MessageChannel/bootstrap secret-boundary contract in `web_proxy_relay.rs`;
- secret-free RuntimeClient Web bootstrap projection contract in `runtime.rs`;
- strict loopback bootstrap URL acceptance/rejection in `source/electron-main/network-proxy-settings.contract.test.ts@d3576631d11ad4375aab3960dafb922886851be1`.

All remain authored-only.

## Known adjacent gap

A persisted Web selection restored during AppHost construction calls the existing `apply_network_proxy` and prepares a fresh loopback relay, but that startup path currently discards the returned one-shot bootstrap URL instead of handing it to the existing Electron shell. Mutation-time Web selection is wired end-to-end. Restart auto-reopening is retained as an explicit adjacent lifecycle follow-up; it is not represented as verified or accepted.

## Stage-A execution statement

No GitHub Actions/check/status/workflow query or trigger was performed. No build, typecheck, lint, schema validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deploy, install or runtime acceptance was executed.

## Disposition

`TDRP-R9-MTPROTO-WEB-PROXY-RELAY-STREAM-001` is `implemented / validation_status=not-run / accepted=false` for the production-reachable selected-Web relay stream/window/browser-bootstrap carrier. Stage B executable validation remains open. The broader fixed-U migration remains incomplete.
