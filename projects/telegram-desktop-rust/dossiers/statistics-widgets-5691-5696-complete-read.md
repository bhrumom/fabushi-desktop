# Statistics widgets — deterministic orders 5691-5696 complete read

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

This batch closes the statistics subtree read-through: responsive/elided chart headers, animated line-filter chips with a guard against disabling the last visible line, and the localized point-details popup with per-line percentages, Credits/TON/USD value projection, optional zoom affordance, DPR/theme cache and ripple state.

Fabushi must implement these through existing canonical controls and source-neutral Chart/Analytics surfaces: Checkbox/Switch-style filtering, Popover/DetailPanel, typography/design tokens and accessibility primitives. No Telegram-named widget family or second analytics/wallet owner is allowed. Empty/mismatched chart data, invalid x indices, stale filter identities, long/RTL strings, currency precision, account/theme changes and teardown require explicit bounds/fencing.

Accounting: **5,696 / 16,123 read; 10,427 unread; 15,844 unknown; 0 omitted**. Reading alone closes no unknown. The exact path for order 5,697 must be taken from the accepted recursive tree before the next batch is recorded.
