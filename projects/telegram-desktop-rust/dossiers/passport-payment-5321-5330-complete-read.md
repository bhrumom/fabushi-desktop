# Passport / checkout 5321-5330 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`).

Orders 5321-5330 were read completely from exact blobs. Applicable behavior maps to existing canonical account/identity/provider-authorization, attachment/resource, form-validation/design-system and payment/payment-provider owners; no Passport or Telegram payment runtime is introduced.

- **5321 — `Telegram/SourceFiles/passport/passport_panel_edit_scans.h` @ `2d648391b331428f0e30059ab6a2e1a1c12f9200`**: typed secure-scan editor contract for scan classes, add/delete/restore/error state and change detection.
- **5322 — `Telegram/SourceFiles/passport/passport_panel_form.cpp` @ `dda2272b267fabc1e0c662ebf343a64a723274fc`**: secure authorization form rows, completeness/error projection, navigation and edit activation.
- **5323 — `Telegram/SourceFiles/passport/passport_panel_form.h` @ `83ea1b2da8bcb0320d4b45bb1da68bb4c0786142`**: typed secure authorization form presentation contract.
- **5324 — `Telegram/SourceFiles/passport/passport_panel_password.cpp` @ `48a4f404726f96161545d87fdb53b39d719534af`**: password entry/setup/recovery/email-confirmation lifecycle with cancellation and error projection.
- **5325 — `Telegram/SourceFiles/passport/passport_panel_password.h` @ `63df347c6b2bc503b646aea6f5e1153a15b8c03d`**: typed password/recovery authorization presentation contract.
- **5326 — `Telegram/SourceFiles/passport/ui/passport_details_row.cpp` @ `49f63b6cc4028b9c083b064a6950362455cac5e5`**: validated detail row editing for text/postcode/country/date/gender, focus transfer and error clearing.
- **5327 — `Telegram/SourceFiles/passport/ui/passport_details_row.h` @ `d93708fe4423550107f14e7e88e6a94fb83d26d7`**: typed detail-row value/error/focus contract.
- **5328 — `Telegram/SourceFiles/passport/ui/passport_form_row.cpp` @ `ca76b4095675bfcda7e5a4cf15f39bca263fe68f`**: form-row title/description plus ready/error state projection and bounded layout.
- **5329 — `Telegram/SourceFiles/passport/ui/passport_form_row.h` @ `2778df2048e3a01a1fd696b30d914c26d51a7163`**: typed form-row ready/error presentation contract.
- **5330 — `Telegram/SourceFiles/payments/payments_checkout_process.cpp` @ `cfa02cf756fe35be4e17a3545780fcd1a59e7d98`**: checkout singleton/dedupe lifecycle, submit-state fencing, terms/trust gates, structured WebView callback validation, sensitive screenshot policy and paid/pending/failed settlement projection.

Portable invariants include bounded form/resource validation, dirty-state truth, cancellation, exact-scope async fencing, checkout dedupe, terms/trust gates, structured callback validation and authoritative settlement projection. Read-through only: `unknown=15841`, `omitted=0`.
