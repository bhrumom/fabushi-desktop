# Round 106 evidence — MTProto obfuscated connection-start wire

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 / round 75
Stage: A only; executable validation intentionally not run.

## Authority readback

- Desktop checkpoint start: `f5b4906d790253b037940a3e6be2df7006f8b889`.
- Platform-core checkpoint start: `21c433fb6b1a37b91fa0757b018868c21aa511dc`.
- Fixed upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Primary fixed-U source: `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`.
- Adjacent fixed-U definitions: `connection_tcp.h@ce09deab2f722ca22d371c851a7c43d939a9034b`, `mtproto_auth_key.h@31d0e3db369b309b8f2e9ea1d6b83b6a6d51b20c`, `mtproto_auth_key.cpp@cb4a4e5c3b94ed33d126068792536a08800615f2`, `mtproto_tcp_socket.cpp@b3130fd706b7cbfa33f7c9f54ac5e87abe9c3e10`, `mtproto_dc_options.cpp@d8499fc1df2ed7fdb88da687a04b824d243ff49a`, `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`.

## Exact responsibility

`TDRP-R9-MTPROTO-OBFUSCATED-CONNECTION-START-WIRE-001` isolates direct-TCP 64-byte connection-start nonce filtering, Version0/Version1/VersionD secret classification and key derivation, protocol/DC-id insertion, the plain56+cipher8 transmitted prefix, and continued bidirectional AES-256-CTR streams. Framing remains with the existing TransportFrameCodec; socket lifetime remains with TcpTransport.

Fixed-U direct nonce rejection covers first byte 0xEF, first LE32 HEAD/POST/GET /EEEEEEEE/DDDDDDDD/02010316, and second LE32 zero. Version0 uses the nonce key source directly; Version1 and VersionD use SHA256(source || secret16). VersionD DD17 and EE>=21 both use only bytes 1..17 as secret16. Receive key/IV are derived from reversed nonce[8..56]. Protocol id and signed protocol DC id are inserted before the full nonce is CTR-transformed; only encrypted bytes 56..64 replace the transmitted nonce tail, while the send CTR remains advanced by 64 bytes.

## Existing-owner implementation

- Spec commit: `17d0c7e803d710f98cfcc2d7dcd64829819d5241`, blob `15d097dfaac262a8cdbcdaaa2b0fc1c3e8946f35`.
- Production commit: `0f93e212d5151c598f19a67b0a65ad19774dc1c3`.
- Canonical transport: `native/telegram-protocol/src/transport.rs@74189488dc3ec1015edc59989f7aced8ad65f5c3`.
- Canonical protocol exports: `native/telegram-protocol/src/lib.rs@d5727367035cef2adbba0a6a6a2f6b95e2716450`.
- Canonical shipping network: `native/telegram-network/src/lib.rs@54245116969f53ee53ccc3f979bbd9fb7382cc97`.
- Authored-only contract commit: `8b04666c4e1d0c8c986432bcf272a6679f96566d`.
- Protocol contract blob: `4de139a0f98c2d2b784145aa1995a67458af9c97`.
- Network contract blob: `1a94aaa3f79e8108e04bc5418657fd4e641d0a4a`.
- Desktop provider/contract synchronization: `76e467b5a1b3aecceffc4cdb6d34dc12461f12d3`.

The existing `connect_directory` shipping path now constructs `ObfuscatedConnectionStart` from the canonical endpoint secret and Main/test protocol DC id, writes the prefix once, encrypts framed outbound bytes with continued send state and decrypts inbound bytes before frame parsing. Raw `TcpTransport::from_stream` remains un-obfuscated for focused fixtures.

## Authored-only contracts

Protocol contracts cover exact secret shapes, DD/EE secret16 equivalence, all fixed-U nonce reserved forms, prefix partition, DC-id-sensitive encrypted tail, Version0/Version1 divergence, split-call CTR continuity, send/receive direction separation and VersionD suffix exclusion. The network contract covers one obfuscated prefix and contiguous bidirectional CTR around the existing abridged frame codec.

No contract was executed.

## Explicitly open

Proxy/Web/TLS camouflage and TLS ClientHello/TlsBlock, media-cluster negative protocol DC ids, temporary-DC normalization, quick-ACK lifecycle/correlation, connection racing/retry/reconnect, auth-key binding/persistence/expiry/renewal, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


## Final source-level compliance readback

Platform-core final source-level compliance commit is `5b7ed34df4c8063fe3db9627552bc673dda9c92f`, spec blob `65dc6e1d9e611ad5428fc7c9c85551ac6d868081`. The source-reviewed protocol contract correction is `ecf143c6edec0c859e2c19a0f94aa1c9dca215ae`; the corrected desktop mirror is `6f950d056170a232d22ee08c6975cb051592594f`, with byte-identical contract blob `307c973117a96fa901662d021ac79f5ec76b28cf`.

The responsibility remains implemented only, not verified or accepted. Coverage remains responsibilities_total 107, unknown 16172, unread 16145, omitted 0, unmapped_responsibilities 0. No executable validation or GitHub Actions/status/check query was performed.
