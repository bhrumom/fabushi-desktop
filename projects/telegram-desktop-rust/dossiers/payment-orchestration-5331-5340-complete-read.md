# Payment orchestration 5331-5340 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`).

Orders 5331-5340 were read completely from exact blobs. Applicable payment/form, balance/reaction and provider-tokenization responsibilities map to existing Mahayana payment/payment_provider/wallet/service, canonical auth/external-URL/security, and production reaction owners.

- **5331 — `Telegram/SourceFiles/payments/payments_checkout_process.h` @ `fe10b2c0fab4d91dd70195d57da6602f4bd6e1ee`**: typed checkout modes/results, submit states, panel delegate boundary and session-scoped lifecycle.
- **5332 — `Telegram/SourceFiles/payments/payments_form.cpp` @ `50b22888d6a4c7f92a5158d9bcf1922528690d41`**: invoice/form loading, customer/card validation, saved/new credential selection, provider tokenization, temporary-password/trust gates and authoritative payment submission.
- **5333 — `Telegram/SourceFiles/payments/payments_form.h` @ `1fc35c61a7734291c59c5b1e503a8591ed59e582`**: typed invoice/payment method/order form model and update/error protocol.
- **5334 — `Telegram/SourceFiles/payments/payments_non_panel_process.cpp` @ `e9c8cf62562e68ae8574acff63060b3056551baf`**: balance-gated credits purchase/receipt flow with paid/cancelled/failed settlement and weak-window lifetime fencing.
- **5335 — `Telegram/SourceFiles/payments/payments_non_panel_process.h` @ `3b323d37e976524b73320af11cccbf4adee74b97`**: typed non-panel payment/receipt orchestration boundary.
- **5336 — `Telegram/SourceFiles/payments/payments_reaction_process.cpp` @ `d9a2cd5408dac1521af44601e9a0807d7e6fcc2b`**: paid-reaction balance gating, duplicate-send fence, entity liveness recheck, sender privacy choice and removal-aware UI lifecycle.
- **5337 — `Telegram/SourceFiles/payments/payments_reaction_process.h` @ `1071b0dfbbba814266c31527a8296fdaaa051b43`**: typed paid-reaction action/detail boundary.
- **5338 — `Telegram/SourceFiles/payments/smartglocal/smartglocal_api_client.cpp` @ `0f577374e84bebc53ce8b001bd7713e5b8f97c1b`**: strict tokenize endpoint validation, single in-flight request, old-reply isolation, structured JSON/network error handling and main-thread settlement.
- **5339 — `Telegram/SourceFiles/payments/smartglocal/smartglocal_api_client.h` @ `3166b386dae109a6b216e2fd7085176ff15258f8`**: typed provider tokenization client/configuration lifecycle.
- **5340 — `Telegram/SourceFiles/payments/smartglocal/smartglocal_callbacks.h` @ `53cecceb2d5bfd91b2579ab3b75131aea4563a6d`**: typed tokenization completion result boundary.

Telegram Stars, MTProto invoice wiring and SmartGlocal runtime are source-specific and not introduced. Read-through only: `unknown=15841`, `omitted=0`.
