# Payment provider and card UI 5361-5370 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`).

Orders 5361-5370 were read completely from exact blobs. Provider wire details remain adapter-only; portable responsibilities map to the existing source-neutral `native/mahayana-messaging` PaymentProvider/payment owners plus canonical form-validation, security, error and checkout-presentation owners. Telegram/Stripe presentation or runtime is not copied.

- **5361 — `Telegram/SourceFiles/payments/stripe/stripe_form_encodable.h` @ `7cae5e25071e6385c8cf2011bed2ed411467ed3c`**: provider form-encodable parameter contract and typed adapter wrapper.
- **5362 — `Telegram/SourceFiles/payments/stripe/stripe_form_encoder.cpp` @ `2f8f61f081b224c94ccf6952ae2411d96e50a6fd`**: deterministic ephemeral provider form encoding: drop empty values, sort keys, namespace fields and percent-encode keys/values.
- **5363 — `Telegram/SourceFiles/payments/stripe/stripe_form_encoder.h` @ `79e28471d86567fc68094f58f1ce54bf8de467f4`**: stateless provider form-encoding contract.
- **5364 — `Telegram/SourceFiles/payments/stripe/stripe_payment_configuration.h` @ `a42e7921a81e4702066426e1814db2cc206c2b63`**: provider checkout configuration boundary for publishable configuration and merchant presentation metadata.
- **5365 — `Telegram/SourceFiles/payments/stripe/stripe_pch.h` @ `b7c2e9fee1b35daddac7fe1245728bfd3f55b5df`**: provider module build-time include aggregation with no independent product state machine.
- **5366 — `Telegram/SourceFiles/payments/stripe/stripe_token.cpp` @ `d91e48be38e76943d6c2f3e94036952773d9f0cb`**: strict minimal token response decoding with required id/livemode/created fields and optional card metadata.
- **5367 — `Telegram/SourceFiles/payments/stripe/stripe_token.h` @ `f22aefe60667314bbc096c126d02765b1d842e9a`**: typed minimal provider token contract with empty-state semantics.
- **5368 — `Telegram/SourceFiles/payments/ui/payments.style` @ `e537343e6709b7ed86215306fc2c27a616592952`**: checkout visual hierarchy for summary, fields, tips, shipping, loading, critical errors and provider-webview footer.
- **5369 — `Telegram/SourceFiles/payments/ui/payments_edit_card.cpp` @ `0c2aac434e9ee4ca29e465274dadc087abda60e3`**: card-entry edit state machine with cursor-preserving normalization, card/expiry/CVC validation, conditional billing fields, save-information opt-in, focus/error targeting and submit/cancel flow.
- **5370 — `Telegram/SourceFiles/payments/ui/payments_edit_card.h` @ `ae4cce54b415fead818a50e40143b82e2c09ae4c`**: typed card-edit presentation lifecycle and field ownership contract.

The card editor additionally proves cursor-stable formatting across insertion/backspace/delete, exact field focus/error restoration, conditional country/ZIP/name collection and explicit save-information choice. New UI must reuse canonical `TextField`, `Checkbox`, `Button`, `Status`, `LoadingState`, `ErrorState`, `Toast`, `Dialog` and semantic tokens; no Telegram/Stripe-specific component root is permitted.

Read-through only. Responsibility-level shipping composition and exact-head tests remain open, so `unknown=15841` and `omitted=0` are unchanged.
