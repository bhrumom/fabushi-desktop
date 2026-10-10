# Console-lock test precondition — deterministic order 5772 complete read

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

Exact source: `Telegram/SourceFiles/test/test_console_lock.cpp@0a1fcffab5bdd7a9f90ace3b9c8a69b5806e7808`.

## Responsibility

This is a professional-test precondition/privacy diagnostic responsibility, not production UI. On Windows it resolves the current process session and reads `WTSSessionInfoEx.SessionFlags` to classify the console as locked, unlocked or unknown. A positively locked console produces a bounded gate message so focus/clipboard/input failures are not misreported as product regressions.

The implementation deliberately refuses to overclaim. API errors, malformed/short buffers, unexpected levels/flags and the documented Windows 7 / Server 2008 R2 reversed lock/unlock defect remain `unknown`. Non-Windows builds are explicit `not-applicable` and do not fire the lock gate.

## Fabushi disposition

Map to the canonical cross-platform desktop test/evidence harness. Preserve:
- positive-lock gating without converting unknown to unlocked;
- diagnostic reasons based only on session metadata/errors, never user content;
- explicit non-Windows N/A;
- coverage for locked/unlocked/unknown, malformed responses, legacy Windows and teardown/retry behavior.

Do not create a Telegram-derived runtime or UI.

Accounting after this read: **5,772/16,123 read; 10,351 unread; 15,844 unknown; 0 omitted**. Reading alone closes no unknown.
