# Stripe validation / error 5351-5360 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`).

Orders 5351-5360 were read completely from exact blobs. Applicable behavior maps to canonical PaymentProvider/payment, form-validation, security and error-projection owners.

- **5351 — `Telegram/SourceFiles/payments/stripe/stripe_card.cpp` @ `ca2c864a9165cbf5d1f42783a865907dca3f5dd3`**: strict provider card-object field presence, brand/funding normalization and minimal card projection.
- **5352 — `Telegram/SourceFiles/payments/stripe/stripe_card.h` @ `30ff47b643781f58c83b7d0e2923ed54100970e2`**: typed provider card metadata contract.
- **5353 — `Telegram/SourceFiles/payments/stripe/stripe_card_params.cpp` @ `81b72c4e0328648616a3212c9fd0927ac076ca25`**: deterministic card parameter field serialization for ephemeral tokenization.
- **5354 — `Telegram/SourceFiles/payments/stripe/stripe_card_params.h` @ `d107dc57f784162ca5656065b6800f29146bae8d`**: typed ephemeral card-input parameter contract.
- **5355 — `Telegram/SourceFiles/payments/stripe/stripe_card_validator.cpp` @ `599a3922df18c67499b091ece51754e904d37acc`**: card input validation state machine: whitespace normalization, numeric-only, BIN/brand lengths, Luhn, expiry threshold, CVC length and formatting groups.
- **5356 — `Telegram/SourceFiles/payments/stripe/stripe_card_validator.h` @ `68fdf5ad13954fa6c799132ddab4fd0c5e4e25ce`**: typed Invalid/Incomplete/Valid payment-field validation contract.
- **5357 — `Telegram/SourceFiles/payments/stripe/stripe_decode.cpp` @ `acd2dac0b0b89c4b10ca10a975fa07926640fedd`**: required-field presence guard for provider JSON decode.
- **5358 — `Telegram/SourceFiles/payments/stripe/stripe_decode.h` @ `5b0b0e152461495d02bfba7600c31ccecf5a51ab`**: typed required-field decode helper contract.
- **5359 — `Telegram/SourceFiles/payments/stripe/stripe_error.cpp` @ `c5be7e010471ae42531b863f2601163f1da02e92`**: structured API/invalid-request/card/unknown error decoding with parameter normalization and card-specific reason mapping.
- **5360 — `Telegram/SourceFiles/payments/stripe/stripe_error.h` @ `36247d3875ff6d66ed59f45158ea6dec32c11472`**: typed provider/network/card/cancellation/checkout error taxonomy.

Stripe SDK/wire details are source-specific; portable validation/error/ephemeral-input semantics remain mapped-open. Read-through only: `unknown=15841`, `omitted=0`.
