# Payment checkout information, fields, summary and panel 5371-5380 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`).

Orders 5371-5380 were read completely from exact blobs. Portable responsibilities map to existing source-neutral Fabushi owners; Telegram/Stripe/Linux runtimes or UI roots are not copied.

- **5371 — `Telegram/SourceFiles/payments/ui/payments_edit_information.cpp` @ `aa409b72c29041d49cf78081d3a08d4b5284d854`**: requested customer-information validation, focus/error and save-information lifecycle; canonical owner: native payment + canonical form.
- **5372 — `Telegram/SourceFiles/payments/ui/payments_edit_information.h` @ `6755939a01414f1b7d87263e71a6fc3f5ce920ce`**: typed requested-information editor contract; canonical owner: native payment + canonical form.
- **5373 — `Telegram/SourceFiles/payments/ui/payments_field.cpp` @ `25892dae53a71e59e7cd7117548cafc9083da659`**: field normalization, money editing, cursor preservation and validator lifecycle; canonical owner: canonical TextField/payment form.
- **5374 — `Telegram/SourceFiles/payments/ui/payments_field.h` @ `26711ddd935cfb79adc6a053aebbfcd3fad89793`**: typed payment field/validator contract; canonical owner: canonical TextField/payment form.
- **5375 — `Telegram/SourceFiles/payments/ui/payments_form_summary.cpp` @ `bb178a0530e4e53bcaca8f0324b5cec5a5984661`**: checkout/receipt totals, sections, tips and scroll continuity; canonical owner: canonical payment surface + Money/Invoice.
- **5376 — `Telegram/SourceFiles/payments/ui/payments_form_summary.h` @ `ed7ba7373af3010fe8c5a127bcecca2866541156`**: typed checkout summary lifecycle; canonical owner: canonical payment surface.
- **5377 — `Telegram/SourceFiles/payments/ui/payments_panel.cpp` @ `adf27afecbee264df8933d4645fc1c12da84fe31`**: shipping/tips/method/webview/terms/trust/close/progress checkout orchestration; canonical owner: canonical payment surface + external webview/security.
- **5378 — `Telegram/SourceFiles/payments/ui/payments_panel.h` @ `db6f345c6fb03f5978c524c29f4628c74df7d141`**: typed checkout panel lifecycle contract; canonical owner: canonical payment surface.
- **5379 — `Telegram/SourceFiles/payments/ui/payments_panel_data.h` @ `1021e7b56daeb362417b38e0c346d6ff8cf930fe`**: checkout UI data/value contracts; canonical owner: native payment + canonical renderer projection.
- **5380 — `Telegram/SourceFiles/payments/ui/payments_panel_delegate.h` @ `fd73fd1441682bd829c606951992452d67107264`**: typed checkout command/delegate boundary; canonical owner: canonical payment command/event.

Orders 5371-5374 exposed a bounded domain gap: invoice-requested customer data lacked fail-closed requiredness/bounds validation before charging. Commit `39da06252e33ef7f8d53061514f8234108ee609e` implements that source-neutral slice plus a focused Rust contract. Exact-head Actions and the wider checkout/reaction/platform responsibilities remain open.

Read-through does not close unknown. `unknown=15841`, `omitted=0`.
