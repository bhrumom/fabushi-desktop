# Console-lock interface contract — deterministic order 5773 complete read

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

Exact source: `Telegram/SourceFiles/test/test_console_lock.h@02a5d6e8ce07d014ebd7577b37f5b32e8d1bbbe9`.

## Responsibility

This header defines the public contract paired with order 5,772's Windows console-lock reader. It is professional test/evidence infrastructure, not product UI.

The state model is deliberately four-valued: `NotApplicable`, `Unlocked`, `Locked`, and `Unknown`. `ConsoleLockReading::locked()` returns true only for `Locked`; therefore inability to read a session, malformed metadata, unsupported legacy behavior, or a non-Windows platform cannot be silently promoted into a safe/unlocked result. `ConsoleLockGate` returns a skip reason only for a positively locked reading and remains empty for every other state.

The contract also requires a fresh read at the moment of each call rather than a cached application lock bit. Diagnostics are restricted to the source/raw session metadata/reason needed to explain the classification; user, domain, and window-station names carried by the underlying WTS structure are explicitly outside the allowed evidence surface.

## Fabushi disposition

Map to the one canonical cross-platform desktop professional test/evidence harness. Preserve the four-state model and `locked()` truth table; no-cache/fresh-read semantics; positive-lock-only gating; Unknown and non-Windows N/A as non-gating states; privacy-safe diagnostics; and unit/contract coverage for state/gate text and cross-platform behavior.

Do not create a Telegram-derived runtime, product component, or source-named UI.

Accounting after this read: **5,773/16,123 read; 10,350 unread; 15,844 unknown; 0 omitted**. Reading alone closes no unknown.

First unread: **5,774 `Telegram/SourceFiles/test/test_corner_patch.cpp@a34785a050cdeea405c3948345704d2e19395843`**.
