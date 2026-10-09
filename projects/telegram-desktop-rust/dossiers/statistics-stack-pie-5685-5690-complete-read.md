# Statistics stack/pie views — deterministic orders 5685-5690 complete read

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

This batch closes the statistics/view subdirectory read-through: stack-chart x geometry and percentage-to-index mapping, pie percentage aggregation/rounding correction, and stack-linear local zoom into animated pie presentation with hover/selection/footer state.

These responsibilities remain in the one canonical Analytics/Chart owner. The source assumes several preconditions that the Fabushi implementation must make explicit and fail closed: at least two x points where width is computed, non-zero percentage denominators, valid zoom/index bounds, aligned line lengths, non-zero sums before normalization, and stable transition dimensions. Zero totals, single visible series, all-disabled filters and rapid hover/filter/zoom changes need deterministic settlement.

Accounting: **5,690 / 16,123 read; 10,433 unread; 15,844 unknown; 0 omitted**. Reading alone closes no unknown. First unread: **5,691 `Telegram/SourceFiles/statistics/widgets/chart_header_widget.cpp@b8754ceb6388ac856183b5bec7bd58cb3d923684`**.
