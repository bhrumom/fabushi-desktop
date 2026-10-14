# Corrected deterministic source batch 5623-5633

Authority: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders are exact non-directory/blob ranks in the accepted recursive tree.

- 5623 `Telegram/SourceFiles/settings/settings.style@8b9e56dd...`: Shared Settings design tokens/layout/iconography for canonical controls, security/session/search/business/credits/passkey/power/privacy surfaces. Owner: Canonical design system + Settings.
- 5624 `Telegram/SourceFiles/settings/settings_builder.cpp@20b6e6da...`: Settings composition/search/highlight builder implementation over typed sections and reusable controls. Owner: Canonical Settings shell + UniversalSearch + design system.
- 5625 `Telegram/SourceFiles/settings/settings_builder.h@3ac0a93b...`: Typed Settings builder/search registry and section/control metadata contracts. Owner: Canonical Settings shell + UniversalSearch.
- 5626 `Telegram/SourceFiles/settings/settings_codes.cpp@78c43922...`: Hidden diagnostic/developer command dispatcher for logs, update testing, language/palette/endpoints/test environment and support operations. Owner: Controlled diagnostics/support policy.
- 5627 `Telegram/SourceFiles/settings/settings_codes.h@a9e13582...`: Diagnostic code feed interface. Owner: Controlled diagnostics.
- 5628 `Telegram/SourceFiles/settings/settings_common.cpp@7133758b...`: Reusable Settings search/highlight/navigation/button/layout behavior. Owner: Canonical design system + Settings.
- 5629 `Telegram/SourceFiles/settings/settings_common.h@f42ed946...`: Reusable Settings section/control/search/highlight interfaces. Owner: Canonical design system + Settings shell.
- 5630 `Telegram/SourceFiles/settings/settings_common_session.cpp@dc6ea9f3...`: Session-aware Settings capability helper over typed section identity. Owner: Canonical Settings routing/session capability.
- 5631 `Telegram/SourceFiles/settings/settings_common_session.h@22546d12...`: Session-bound section factory/container contract. Owner: Canonical Settings shell lifecycle/routing.
- 5632 `Telegram/SourceFiles/settings/settings_credits_graphics.cpp@c753c816...`: Credits/wallet/gift commerce orchestration with top-up checkout, balances, withdrawal, receipts/subscriptions and saved/unique gift mutations. Owner: Canonical Wallet/Payments/Credits + Gift/Commerce + Entitlement.
- 5633 `Telegram/SourceFiles/settings/settings_credits_graphics.h@d5b89449...`: Credits/Gift action contracts for checkout/top-up/balance/history/subscription/gift menus and low-balance recovery. Owner: Canonical Wallet/Payments/Credits + Gift/Commerce.

All applicable responsibilities remain mapped-open; reading grants no unknown credit. Current contiguous read-through after the corrected batches is 5,653/16,123; unread 10,470; unknown 15,844; omitted 0. First unread is 5,654 `Telegram/SourceFiles/statistics/chart_lines_filter_controller.cpp@40c3b612...`.
