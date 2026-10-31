# Round 65 — MTProto ping_delay_disconnect wire serialization and TLS template owner review

Date: 2026-11-01
Stage: A — implementation only; executable validation deferred

## Fixed authorities

- Upstream U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`
- `special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88`
- `mtproto_tls_socket.cpp@e7562e3ca1a1da03f5d368085d5e61d887ab3f16`

## TLS adjacency review

The fixed-U `tlsClientHello/tlsBlock*` declarations are consumed as a local typed ClientHello template AST. `PrepareClientHelloRules()` constructs the AST and `Generator::Part::writeBlock` expands it into TLS camouflage bytes; the generator also performs randomness, X25519 public-key generation, scoped-length encoding, permutation, HMAC digest insertion and timestamp injection. This is not an inbound MTProto body parser/trailing-validation responsibility.

Fabushi's current canonical network owner is direct `TcpTransport` over `TcpStream`/MTProto framing and has no precise TLS camouflage socket/generator lifecycle. Creating an inbound `tlsClientHello/tlsBlock*` parser would therefore be a false equivalence. The TLS camouflage lifecycle is recorded fail-closed/open.

## Independent responsibility implemented

Fixed-U `ping_delay_disconnect#f3427b8c ping_id:long disconnect_delay:int = Pong` is independently representable by the existing telegram-protocol wire serializer. `session_private.cpp` uses it with `kPingDelayDisconnect=60` on a main connected session; that session/timer decision is intentionally not moved into the wire builder.

Canonical platform-core:
- spec commit: `c98a668a64a25c516065f0dda938200290712018`
- implementation commit: `cb816c7540ea93717238b3aec96372cdab1c1706`
- api blob: `405b04166bea039051833a38ad0a75c62107f70c`
- lib blob: `ef7fe73335551e303fa71b02f5cf066d778ae8a4`

Desktop provider sync:
- implementation commit: `d38f6bef11a9a4334f8387c73c7b74e7994c0cdb`
- provider api blob: `405b04166bea039051833a38ad0a75c62107f70c`
- provider lib blob: `ef7fe73335551e303fa71b02f5cf066d778ae8a4`

Source inspection confirms `build_ping_delay_disconnect` emits exactly 16 bytes: constructor, signed 64-bit ping id, signed 32-bit disconnect delay, all little-endian. Authored contracts cover the fixed-U delay value and signed minimum/maximum boundaries. They were not run.

## Stage-A state

Ledger responsibility: `TDRP-R9-MTPROTO-PING-DELAY-DISCONNECT-WIRE-001`.

Implementation: implemented.
Validation: not-run.
Accepted: false.

No GitHub Actions/status/checks were queried or triggered. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.

Remaining blockers include TLS camouflage ClientHello generation/socket lifecycle, the main-session ping timer/scheduling policy, SpecialConfigRequest higher lifecycle, gzip, full difference decoding/application, arbitrary historical sent/resend/container state, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT.
