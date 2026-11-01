# Round 110 — WebProxy relay stream/window/nonce boundary

Date: 2026-11-01
Task round: 79
Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Exact source readback

- `web_proxy_frame.h@a5ba491fed67d70aee88e6ec2e3ac47c991e78f3`
- `web_proxy_frame.cpp@bd1d7b310f40203f9da914efc8bdd6711035884b`
- `web_proxy_transport.h@f58850d2bc2ceabd93113d7725903e1c0d92d71b`
- `web_proxy_transport.cpp@e25ad970e84686e9aca061d02c274048793b5ad4`
- `mtproto_web_proxy_socket.cpp@9aa05e7034031310de4aa9d410fbb15d21c63ee8`
- `mtproto_web_proxy_socket.h@02ed27b77d4ff606b3e1cb548a60e4f2c2ca11f0`
- `web_proxy_webview.cpp@8a2efc8fb15e400366f199cd956effc8ad362da8`
- `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`

## Disposition

The fixed-U relay wire has an independently specifiable frame grammar and window scalar boundary, but no independently shippable Fabushi implementation exists today because the current canonical `telegram-network` call graph has no WebProxy carrier/logical-stream path. The WebView nonce gate is additionally lifecycle-bound. Existing-owner-first therefore maps this responsibility to `telegram-network` and records a blocked/no-production-delta outcome instead of creating dead code.

Durable spec-first platform record: `bde283f60d5413426649737a47912b5ec4c3899a` / `docs/specs/telegram-web-proxy-relay-stream-boundary.md@b7de819a9594f74c6aab7f75112c6876f2b21e87`.

Canonical/desktop network production mirror remains `241cd8ec792070a8de8af9ec6ca5b317e34376d1`; canonical/desktop authored network contract mirror remains `71f9732b1c81d7bf782ff1eccb2852b1401cfa6f`. No provider source delta was manufactured.

The unique responsibility is `TDRP-R9-MTPROTO-WEB-PROXY-RELAY-STREAM-001`: `mapped / pending / validation_status=not-run / accepted=false`. Authored contract vectors live in the platform spec and were not executed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E execution, packaging, deployment, installation or runtime acceptance was performed.
