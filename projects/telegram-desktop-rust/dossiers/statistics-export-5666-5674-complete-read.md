# Statistics formatting and XLSX export — deterministic orders 5666-5674 complete read

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

This batch covers locale/timezone-aware statistics labels, TON/Credits chart icon projection, authoritative analytics-to-sheet projection, the signed 64-bit chart value contract, and a minimal XLSX/OpenXML serializer.

Production mapping is deliberately source-neutral: canonical Analytics owns authoritative values; canonical localization/time utilities own date/time labels; canonical Wallet/Credits visual tokens own currency graphics; canonical Export owns XLSX generation. No Telegram statistics/export runtime or product UI is introduced.

The XLSX contract includes legal and unique <=31-character sheet names, XML escaping and invalid-control filtering, typed text/header/number/date/datetime cells, Excel serial dates, numeric-header alignment, bounded column widths, frozen header rows, relationships/content-types and compressed output. Text/header cells remain inline strings rather than formulas, so canonical implementation must retain formula-injection safety. Empty sheets, short line series, invalid numeric values, Unicode/RTL names, duplicate/forbidden-only names, ZIP/write failures, account switch, cancellation, retry/reload and cross-platform file export require explicit fail-closed evidence.

Accounting after this batch: **5,674 / 16,123 read; 10,449 unread; 15,844 unknown; 0 omitted**. Reading alone closes no unknown. First unread: **5,675 `Telegram/SourceFiles/statistics/view/abstract_chart_view.cpp@4b5f2929aec01df6bec99ce2a0d1a9aae15b4bdd`**.
