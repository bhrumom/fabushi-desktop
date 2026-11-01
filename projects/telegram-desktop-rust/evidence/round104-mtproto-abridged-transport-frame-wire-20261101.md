# Round 104 MTProto abridged transport frame wire evidence

Stage A only. Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.

Fixed-U source anchor: `Telegram/SourceFiles/mtproto/connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`. Version0 abridged framing uses a one-byte word count for 1..126 words and `0x7f + LE24(words)` for 127+ words; its decoder rejects extended counts below 127. Adjacent `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` has no TL constructor for this transport frame.

Durable platform spec: `6f0d7e3cfc1d8d90fa47360b21ae57900df2d33d` / `a5ff2064fc9386cbfa4dc428b633291ff6d9623f`.
Platform production commit: `97c38a1bde61edc53ed9d888ffb3cb541e4ff68f`.
Canonical transport blob: `7f981b96f2f8a17823ba9c84bf2e2324d408be31`.
Canonical shipping network blob: `54f509474391bb5118ba4678ba5c566ab3e74ec8`.
Canonical focused contract blob: `00eff72062ab8d3285234428bd09be3fccb27993`.
Desktop contract mirror commit: `f2405ce2ea29785fbc218840d3e9bf1ff4048576` / blob `00eff72062ab8d3285234428bd09be3fccb27993`.
Desktop shipping network blob: `54f509474391bb5118ba4678ba5c566ab3e74ec8`.

Production disposition is minimal existing-owner rewrite: add only fixed-U rejection of noncanonical extended abridged word counts below 127 in canonical `decode_abridged`; all other isolated abridged framing and the shipping `TcpTransport` call chain remain in existing owners.

Unique ledger row: `TDRP-R9-MTPROTO-ABRIDGED-TRANSPORT-FRAME-WIRE-001`, implemented / rewritten / validation not-run / accepted false. Coverage after this row: responsibilities_total 105; unknown 16172; unread 16145; omitted 0; unmapped_responsibilities 0.

Focused contracts are authored only and were not run. No CI/status/check query or executable validation was performed.
