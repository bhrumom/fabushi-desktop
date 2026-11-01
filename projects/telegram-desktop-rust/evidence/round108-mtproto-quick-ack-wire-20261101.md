# Round 108 — MTProto quick-ACK wire boundary

Date: 2026-11-01
Stage: A source-level implementation only; executable validation deferred.
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4
Responsibility: TDRP-R9-MTPROTO-QUICK-ACK-WIRE-001

## Fixed-U findings

- connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381: Version0 and VersionD shipping frame builders emit ordinary lengths only. parsePacket/socketPacket explicitly state the newer quick-ACK form is not supported.
- connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a: secure transport prefix construction is adjacent but does not introduce a quick-ACK marker.
- session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614: sendSecureRequest computes the MTProto 2 SHA-256/message-key material, while arbitrary sent/resend/ACK history lives separately. No fixed-U quick-ACK correlation lifecycle was found.

## Existing-owner disposition

Existing canonical telegram-protocol already owns:
- crypto.rs@32d69573acc6ef4ce7bcc80073f5cdd7e2bea7ad quick_ack_token;
- transport.rs@74189488dc3ec1015edc59989f7aced8ad65f5c3 generic Abridged/Intermediate request-marker and token decoding.

These pure wire primitives do not require arbitrary request history. They remain unchanged.

Shipping telegram-network never requests quick-ACK because send_payload calls ordinary TransportFrameCodec::encode. Before this round receive_payload silently discarded decoded quick_ack_token values. Because no fixed-U correlation owner exists, silent discard was replaced by a typed fail-closed boundary rather than by invented request-history semantics.

## Durable changes

- Spec: platform-core 1a09a5674ec16af4228bbf0aed0fb302cb52fba0 / docs/specs/telegram-mtproto-quick-ack-wire.md@6cbc5fdc140759e66c3c14ac853bdf18c8ce4fba.
- Production: platform-core 605b8928de5fb64bf235196093ef804e350994f2 / native/telegram-network/src/lib.rs@7c442299aedd4aa7444ba53c7ab8212542dfc2d1.
- Authored-only contracts: platform-core d1cec8b3952a4be1b02c6f1dae1ab008991a0342 / native/telegram-network/tests/tcp_transport.rs@9dfb554bae0dac9ea68ebd6b2fe7cab664cc1240.
- Desktop provider mirror: f24b51ec5ba2d959f8c08fe44c6af706f38d7224; production/test blobs are byte-identical to canonical.

## Contract cases authored, not run

- ordinary Abridged TcpTransport send keeps bit 7 of the short length byte clear;
- unsolicited abridged quick-ACK token 0x81234567 is surfaced as NetworkError::UnexpectedQuickAck(0x81234567);
- pre-existing protocol contracts continue to cover generic Abridged/Intermediate marker/token behavior and known quick_ack_token derivation;
- PaddedIntermediate remains quick-ACK unsupported.

## Explicitly open

Quick-ACK request-to-outstanding-request correlation; arbitrary request histories; broader connection scheduling/racing/reconnect/send retry; proxy/Web/TLS camouflage; auth-key binding/persistence/expiry/renewal; gzip; complete difference/differenceSlice decoding/application; broader fixed-U source/legal closure; Stage-B executable evidence; independent release ACCEPT.

No executable validation or GitHub Actions/status/check query was performed.
