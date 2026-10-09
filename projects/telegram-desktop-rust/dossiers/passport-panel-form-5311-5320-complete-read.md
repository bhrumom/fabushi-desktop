# Passport panel/form 5311-5320 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`).

Orders 5311-5320 were read completely from exact blobs. Applicable behavior stays source-neutral and must be absorbed by current Fabushi account/identity, provider-authorization, attachment/resource, recovery, form-validation and design-system owners.

- **5311 — `Telegram/SourceFiles/passport/passport_form_view_controller.h` @ `590d7c2109cb6ccd3be0c987dca3a7a1c9eaf2f8`**: typed secure-form scope/view contract, ready/error projection and presentation boundary.
- **5312 — `Telegram/SourceFiles/passport/passport_panel.cpp` @ `3b35fbab700a749acb51bf4e57253f93490d5b4e`**: authorization surface navigation/presentation lifecycle: password/no-password/form/edit/error states, back/close cancellation, modal/toast projection.
- **5313 — `Telegram/SourceFiles/passport/passport_panel.h` @ `ef40205c4ff728b3eaae7dcca067ca2bf4ebc1c4`**: typed authorization panel presentation boundary.
- **5314 — `Telegram/SourceFiles/passport/passport_panel_controller.cpp` @ `68a844e26e64122d09901e642ff972f777183f1d`**: authorization UI orchestration and schema: field constraints, country/language/native-name fallback, contact validation, row completeness/error state, password setup/recovery, scan lifecycle, destructive confirmation, best-document selection, unsaved-change fencing and verification box lifecycle.
- **5315 — `Telegram/SourceFiles/passport/passport_panel_controller.h` @ `c08645056e5b6c5dcf9567b1326ab65b35d59f2f`**: typed panel orchestration contract for secure scopes, scans, errors, save/delete/verification and cancellation.
- **5316 — `Telegram/SourceFiles/passport/passport_panel_edit_contact.cpp` @ `7ac24c301d992307c6e1388cdd3d3e1d048249a6`**: phone/email edit and verification UX: reuse existing value, normalize/validate input, code-length autosubmit, resend/call/error state, fragment URL action and save/delete lifecycle.
- **5317 — `Telegram/SourceFiles/passport/passport_panel_edit_contact.h` @ `a3d4a035d86ef7500944233f5d4f046f16e4cd93`**: typed contact edit/verification scheme contract.
- **5318 — `Telegram/SourceFiles/passport/passport_panel_edit_document.cpp` @ `686d280ad22193d4e3363ac82286433da1a22a1d`**: document/details edit UX: one-of document picker, destructive option, dynamic native-name section, field/scans partition, fallback population, unsaved-change detection, validation and scroll-to-first-error.
- **5319 — `Telegram/SourceFiles/passport/passport_panel_edit_document.h` @ `40415d65e1f3e0d6cfba76bdda77b2c8b9703609`**: typed document form schema and row-level validator/formatter/fallback contract.
- **5320 — `Telegram/SourceFiles/passport/passport_panel_edit_scans.cpp` @ `4b80005d88098b9c4631b0b714fc0324a4ff5a76`**: secure scan UX/resource preprocessing: image readability/dimension/size bounds, deterministic resize/JPEG normalization, multiple/special scan add-delete-restore, error truth, upload-more requirement, serial async processing with UI-lifetime guards and stale-callback prevention.

Key portable invariants include schema legality and bounded text formats, native-name conditional/fallback behavior, truthful ready/error aggregation, unsaved-change confirmation, one-of document selection, scoped destructive confirmation, verification-code state, scan preprocessing bounds (readability/dimensions/10MiB output cap/2048px resize/JPEG normalization), serial multi-file async processing, lifetime-guarded callbacks, add/delete/restore semantics, and scroll/focus to the first validation error.

No Telegram Passport panel/controller is introduced. Read-through only: `unknown=15841`, `omitted=0`.
