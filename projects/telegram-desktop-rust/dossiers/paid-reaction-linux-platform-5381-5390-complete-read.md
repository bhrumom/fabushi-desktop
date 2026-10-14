# Paid reaction and Linux platform integration 5381-5390 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`).

Orders 5381-5390 were read completely from exact blobs. Portable responsibilities map to existing source-neutral Fabushi owners; Telegram/Stripe/Linux runtimes or UI roots are not copied.

- **5381 — `Telegram/SourceFiles/payments/ui/payments_reaction_box.cpp` @ `47c85912697337895114fdef25af7d582079ec97`**: paid-reaction amount, top-reactor identity/anonymity and balance state; canonical owner: canonical Reaction + Wallet/payment + Dialog.
- **5382 — `Telegram/SourceFiles/payments/ui/payments_reaction_box.h` @ `3d3416ebfd10bbb0cd33b2a578563e949440494f`**: typed paid-reaction dialog contract; canonical owner: canonical Reaction + Wallet/payment + Dialog.
- **5383 — `Telegram/SourceFiles/platform/linux/current_geo_location_linux.cpp` @ `fe26b13df9e2991cfcdfb1bf3fdb5c06622b516d`**: fail-closed asynchronous exact-location platform adapter; canonical owner: platform/location permission adapter.
- **5384 — `Telegram/SourceFiles/platform/linux/current_geo_location_linux.h` @ `6bb8195430d9cf504d93db967807da5e11ae73eb`**: typed platform location adapter boundary; canonical owner: platform/location.
- **5385 — `Telegram/SourceFiles/platform/linux/file_utilities_linux.cpp` @ `8c9dc7ae5b59748b5fdc66cbd1ef2bfd432f1b9a`**: XDG portal Open-With fd/activation request lifecycle; canonical owner: Electron shell/file/open-with.
- **5386 — `Telegram/SourceFiles/platform/linux/file_utilities_linux.h` @ `1f65cf3e6350e6e4516c92ed0f9cb5223f34d90a`**: platform file/url/dialog adapter surface; canonical owner: Electron shell/file/dialog.
- **5387 — `Telegram/SourceFiles/platform/linux/integration_linux.cpp` @ `2cfd1f662284cf3a70666fad923d7e74e1d6684e`**: activation/open-file, Wayland token, sleep/lock/theme/event-loop integration; canonical owner: Electron lifecycle/platform adapters.
- **5388 — `Telegram/SourceFiles/platform/linux/integration_linux.h` @ `11b203214a0f6ce2dc248d575a3bbb2fb14fee98`**: typed platform integration factory; canonical owner: Electron platform adapter.
- **5389 — `Telegram/SourceFiles/platform/linux/launcher_linux.cpp` @ `1414bad71e0417da957b9934df3787160be07f49`**: update/relaunch/protected-install validation and secure update-log lifecycle; canonical owner: source/electron-main/update + packaging.
- **5390 — `Telegram/SourceFiles/platform/linux/launcher_linux.h` @ `cc7c6b15466ed13f163bbc1512bcd6447f2c461e`**: typed launcher/update lifecycle boundary; canonical owner: source/electron-main/update + packaging.

Orders 5371-5374 exposed a bounded domain gap: invoice-requested customer data lacked fail-closed requiredness/bounds validation before charging. Commit `39da06252e33ef7f8d53061514f8234108ee609e` implements that source-neutral slice plus a focused Rust contract. Exact-head Actions and the wider checkout/reaction/platform responsibilities remain open.

Read-through does not close unknown. `unknown=15841`, `omitted=0`.
