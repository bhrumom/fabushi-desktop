# Gram wallet animation resources — complete source read

Accepted upstream: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

The deterministic queue entries `gram.tgs`, `gram_white.tgs`, and `gram_white_fast.tgs` were gzip-decompressed and their complete Lottie JSON structures inspected. All are 512×512 at 60fps. `gram.tgs` is 121 frames (~2s), `gram_white.tgs` is 60 frames (~1s), and `gram_white_fast.tgs` is 121 frames (~2s). They are packaged under `/animations` by `Telegram/Resources/qrc/telegram/animations.qrc`.

Production consumer tracing shows these are not decorative orphan assets. `wallet_amount_field.cpp` and wallet sending/history rows create a Lottie icon named `gram`; wallet balance ink uses `gram_white`. `history_view_gram_transfer.cpp` uses `gram_white` as the transfer-card mark/sending-loop spare and `gram_white_fast` for read/settle transitions. The animation lifecycle respects disabled-animation and power-saving behavior and is tied to transfer-card sending/read state; it does not itself mutate value.

Fabushi disposition is therefore not “copy three files”. The responsibility is visual state feedback for a native value-transfer/wallet surface. The existing canonical owners are PaymentSettlement/value presentation plus `TranscriptEntry`; Telegram-specific TON/Gram branding or wallet product policy must not be imported blindly. Until product applicability is decided and a native equivalent is wired into the canonical owners with visual/temporal tests, these three rows are read/decomposed but remain open and receive no implementation or verification credit.
