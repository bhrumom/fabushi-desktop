# Round 109 — MTProto TLS camouflage exact wire closure

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 78
Stage: A; executable validation deferred.

Start authority: desktop `dbf9b26a921de1c10f47bbf549dd4588e6a4d196`, platform-core `b48b9a92c45f7954322946fb7640b53affc5d7bc`.
Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.

Source facts: `mtproto_tls_socket.cpp@e7562e3ca1a1da03f5d368085d5e61d887ab3f16`, `mtproto_tls_socket.h@d4a90083ead17f27fcacc219129935a148132608`, `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`, `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`. Adjacent Web proxy reread: `mtproto_web_proxy_socket.cpp@9aa05e7034031310de4aa9d410fbb15d21c63ee8`, header `02ed27b77d4ff606b3e1cb548a60e4f2c2ca11f0`; it is stream/window/lifecycle work, not this exact TLS wire responsibility.

Spec-first: `7313529ae77d1abb99f678e8105ec4525474fe55` / `ba258901a12ba33fbf1f89b31867a87ef9fd88bc`.
Production: `e4fb391fc4f9f6e151f35469e0ded7326be39749`; TLS codec `d15b24a350cc69960efa6f84c509245178917ba9`, protocol lib `8e0634a0c9508913e508807f94aaaeb6c71e56eb`, network `241cd8ec792070a8de8af9ec6ca5b317e34376d1`, dependency input `a62d99f23d370cd75590c7b3186307c1689554ed`.
Authored-only contracts: `9b43bb1bb0bb557a9c1289370f4fdc30a571ad1c`; protocol test `212728aacd277b38821d110379215f75c2b4b675`, network test `71f9732b1c81d7bf782ff1eccb2852b1401cfa6f`.
Desktop provider sync: `6b914b3089a5d3593c276aa37120f9c2f53db198`; all five mirrored production/test blobs are byte-identical.

The exact implemented boundary is `0xEE || 16-byte key || domain`, fixed ClientHello template interpretation (GREASE/random/domain/X25519/scopes/permutation/M/E/padding/HMAC/timestamp xor), incremental authenticated ServerHello validation, and TLS-looking application record framing around the existing obfuscated padded-intermediate transport. HMAC uses `hmac 0.12.1` and X25519 uses `x25519-dalek 2.0.1`; no bespoke crypto is introduced.

State: `implemented / rewritten / validation_status=not-run / accepted=false`. WebProxySocket/WebProxy::Transport, broader proxy selection, scheduling/racing/reconnect/send retry, arbitrary request history, auth-key binding/persistence, gzip, complete difference decoding/application, remaining unknown/unread, broader source/legal closure, Stage-B executable evidence and release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, packaging, deployment, installation or runtime acceptance was performed.
