# Payment provider 5341-5350 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`).

Orders 5341-5350 were read completely from exact blobs. Provider card/token/error decoding and request lifecycle responsibilities map to existing PaymentProvider/payment/wallet plus canonical external-URL/security/error owners.

- **5341 — `Telegram/SourceFiles/payments/smartglocal/smartglocal_card.cpp` @ `a08b0980dad10f6e1a3e9dc077892bf716eedd32`**: strict decoded masked-card representation and last-four extraction.
- **5342 — `Telegram/SourceFiles/payments/smartglocal/smartglocal_card.h` @ `6ebd8bc0cae33288c35130441254692698f5a8df`**: typed minimal masked-card contract.
- **5343 — `Telegram/SourceFiles/payments/smartglocal/smartglocal_error.cpp` @ `cc764a07f2c17133bc23db9b57c6b4d7cb59eacd`**: structured provider error decoding with malformed-response fallback.
- **5344 — `Telegram/SourceFiles/payments/smartglocal/smartglocal_error.h` @ `e3829ff455e0715507a0446fe69184747983a106`**: typed provider/json/network error classification.
- **5345 — `Telegram/SourceFiles/payments/smartglocal/smartglocal_token.cpp` @ `ee8725ca0c8d837a0726575b5ba76b2f7d78143f`**: strict token-id decode with optional masked-card metadata.
- **5346 — `Telegram/SourceFiles/payments/smartglocal/smartglocal_token.h` @ `43f822da0986700d1b258567169fe93230890f59`**: typed minimal token contract.
- **5347 — `Telegram/SourceFiles/payments/stripe/stripe_address.h` @ `5be06f6cc2d3d3997b0505b827eba2cc6d189981`**: billing-address requirement enum contract.
- **5348 — `Telegram/SourceFiles/payments/stripe/stripe_api_client.cpp` @ `853dc1498c4b8a3089934ded1a033073b0a91028`**: fixed provider endpoint/version headers, single in-flight tokenization, stale-reply isolation, structured parse/network/provider errors and main-thread completion.
- **5349 — `Telegram/SourceFiles/payments/stripe/stripe_api_client.h` @ `8fc26eb822dfd093597796e947e19ed87bd9d36a`**: typed provider tokenization client lifecycle.
- **5350 — `Telegram/SourceFiles/payments/stripe/stripe_callbacks.h` @ `8e0d0a2a5a3b95c45ee0b0390cc89429f22a773b`**: typed provider token/error callback boundary.

Stripe/SmartGlocal SDK and wire formats are source-specific and are not copied as a second runtime. Read-through only: `unknown=15841`, `omitted=0`.
