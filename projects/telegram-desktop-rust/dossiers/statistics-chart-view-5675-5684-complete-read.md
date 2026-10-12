# Statistics chart views — deterministic orders 5675-5684 complete read

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

This batch covers the chart-view abstraction, selected-point cache identity, double-line scaling, enabled-series height ranges, bar/stack-bar rendering and aggregate heights, animated rulers with dual-axis/currency captions, closed chart-type dispatch, and linear/double-linear DPR caches plus nearest-X hit testing.

Canonical mapping is one reusable source-neutral Analytics/Chart owner. Rendering/painter implementation may differ by platform, but filter/range/hover/zoom/cache state, keyboard/focus/a11y, theme/high-DPI/responsive/reduced-motion and teardown semantics require parity. The port must explicitly fail closed on empty/one-point data, zero maxima/ranges, all-disabled lines, mismatched series, invalid chart types and pointer endpoint cases rather than inherit unsafe divide-by-zero or iterator assumptions.

Accounting: **5,684 / 16,123 read; 10,439 unread; 15,844 unknown; 0 omitted**. Reading alone closes no unknown. First unread: **5,685 `Telegram/SourceFiles/statistics/view/stack_chart_common.cpp@82a1143fe03883c253b97edcc13e4bc72f82fa82`**.
