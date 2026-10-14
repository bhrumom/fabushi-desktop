# Professional task-test harness — deterministic orders 5759-5771 complete read

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

The harness is itself an applicable migration/test responsibility. It requires pure bounded readiness predicates; product assertions in action/assert stages rather than hidden in waits; exact live object/action identity; diagnostic timeout details; N/A only for genuine preconditions; packed scenarios with explicit teardown; protected fixture secrets; deterministic activation/focus/input; animation-clock evidence; capture of the real paint owner with DPR/scale boundaries; and privacy-safe clipboard evidence that classifies foreign data without logging it.

Orders 5,760–5,771 provide those activation, agent/fixture, animation, dialog-shell, capture and clipboard helpers. Qt implementation details are not copied into production; Fabushi's professional GitHub Actions test/evidence system must preserve equivalent fail-closed semantics across supported platforms.

This batch also repairs two current-head authority failures without weakening gates: order 5,739 is rebound to the exact accepted blob `cc0e3ed6b2c763326c2c16635a7671a747ef506b`, and the Grok semantic-adaptation record for `source/electron-main/attachments/attachments.ts` is refreshed to the actual production blob `9fb1ed46b2ae623e7fa781d6d3ab47f465f4a332`.

Accounting: **5,771/16,123 read; 10,352 unread; 15,844 unknown; 0 omitted**. Reading alone closes no unknown. First unread: **5,772 `Telegram/SourceFiles/test/test_console_lock.cpp@0a1fcffab5bdd7a9f90ace3b9c8a69b5806e7808`**.
