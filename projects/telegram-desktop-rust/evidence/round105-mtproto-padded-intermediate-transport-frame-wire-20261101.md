# Round 105 MTProto padded-intermediate transport frame wire evidence

Stage A only. Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.

Fixed-U source anchor: `Telegram/SourceFiles/mtproto/connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`. `TcpConnection::Protocol::VersionD` writes a four-byte little-endian byte length for the following payload plus 0..15 random padding bytes; its receive parser requires total packet length >=8 and <`0x04000000`. The underlying outbound MTProto payload comes from 32-bit words. Connection-start AES-CTR/secret setup and quick-ACK lifecycle are adjacent but excluded.

Platform spec commit: `ed28746a014682767de9866966eb0b4bb2953618` / initial spec blob `2ecb3e305e850ffd67c7dcc4740bb4b283831566`.
Platform production commit: `3a3d850139082e32a783b7d03b6ea2fd31b00ee9`.
Canonical transport blob: `5d6abd61f130b25baae10a6269eebd20fd025b3a`.
Canonical shipping network blob: `54f509474391bb5118ba4678ba5c566ab3e74ec8`.
Canonical contract commit/blob: `cbc042f54b5b2bb7ec035ad68aaad5bd40263b4a` / `cd53d0b1b546aaceaf5ad46aaf6957dab5b32bf9`.
Desktop contract mirror commit/blob: `551d641ba59ae6f2ef7e3c08bfbf884db8f7f2a3` / `cd53d0b1b546aaceaf5ad46aaf6957dab5b32bf9`.
Desktop shipping network blob remains `54f509474391bb5118ba4678ba5c566ab3e74ec8`.
Platform implementation-record commit/spec blob: `440da6a1b0f24907e77f1c1a94761adb9d17ae1d` / `b6ea2c2df755fe632d624e6621e5b89379a47b0e`.

Production disposition is an existing-owner rewrite only: PaddedIntermediate default encode uses OS randomness for 0..15 padding bytes; explicit padding enforces the 0..15 domain and 4-byte unpadded payload alignment; receive framing enforces the fixed-U VersionD floor/strict ceiling; VersionD quick-ACK forms fail closed while generic Intermediate semantics remain separate.

Unique ledger row: `TDRP-R9-MTPROTO-PADDED-INTERMEDIATE-TRANSPORT-FRAME-WIRE-001`, implemented / rewritten / validation not-run / accepted false. Coverage after this row: responsibilities_total 106; unknown 16172; unread 16145; omitted 0; unmapped_responsibilities 0.

Focused contracts are authored only and were not run. No CI/status/check query or executable validation was performed.
