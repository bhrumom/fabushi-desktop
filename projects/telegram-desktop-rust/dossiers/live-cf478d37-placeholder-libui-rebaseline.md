# TDRP Revision 9 live cf478d37 rebaseline

Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@cf478d37c8f57df831cdedb5621e1cf2ef069a0f (root tree fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,793; first unread=5,794 Telegram/SourceFiles/test/test_log.cpp@17c72a8ae1bc672e7c380b855887b859b78fabec; unread=10,332; unknown=15,846; omitted=0. Changed accepted-prefix orders 11/30/31/92/4599/5759/5764/5765 were explicitly re-read; lib_ui is now order 6,589; runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 are historical-only for 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e + b4d9f7f8cf580eefd553c5ce105f1d3e682de87f.

## Direct delta

- channel_earn.style: botEarnInputField replaces permanent placeholderMargins(-23px,...) with placeholderShiftLeft:-23px.
- Telegram/lib_ui 24310a3196c6632f58101a3ec1c553c4565bc9be -> 91ff4463899f23f4227bc4cbbdbe7696d479b9bd: exactly input_field.cpp, masked_input_field.cpp, widgets.style changed. Both field painters interpolate horizontal floating-placeholder movement; RTL mirroring remains after translation. New lib_ui has 432 non-directory blobs and zero nested gitlinks.

## Deterministic reconciliation

Root paths retain identical order. Accepted-prefix changed blobs are re-read at orders 11/30/31/92/4599/5759/5764/5765. Two new test_window_exposure blobs appear at 5,847-5,848, after the read-through candidate. Orders 5,774-5,793 remain identity/order-stable and are formally recorded. Telegram/lib_ui moves to root order 6,589 because of the two new files. Full recursive denominator re-evaluates to 16,125; first unread is 5,794 Telegram/SourceFiles/test/test_log.cpp@17c72a8ae1bc672e7c380b855887b859b78fabec.
