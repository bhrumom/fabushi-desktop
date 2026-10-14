# PCH and storage security/migration — deterministic orders 5697-5701 complete read

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

Order 5697 is the cross-platform C++ precompiled-header/dependency normalization surface. Fabushi does not copy a PCH, but the applicable compilation/platform dependency contract remains mapped-open until cross-platform shipping evidence proves the canonical Electron/Rust/TS graph.

Orders 5698-5701 are critical local persistence/security responsibilities. The file protocol writes magic/version + payload + MD5 with atomic QSaveFile commit and fallback rename/flush behavior, drains queued same-key writes, reads modern and legacy candidates with version/signature checks, encrypts local records, and derives local keys. Passcode wrapping records KDF family/parameters, supports Argon2id when available with bounded memory/time/lanes, bounded scrypt defaults, rejects invalid/over-cost parameters and short salt, and retains legacy PBKDF2 readers. The settings decoder is a versioned migration boundary for authorization/proxy/cache/power/notifications/download/theme/window/session/input/media/emoji and fallback configuration; malformed streams, invalid values and unknown blocks fail closed.

Canonical owners are Persistence, Local Security/Key Protection, Settings Migration and Account Security. Required evidence includes KDF vectors/cost ceilings, unavailable/OOM behavior, wrong passcode, corrupt/truncated/version/unknown-block fuzzing, interrupted writes and rename/flush failures, async drain/idempotency, legacy migration, account fencing, restart/recovery and secret lifetime/zeroization. This complements rather than closes the existing Local Passcode/App Lock/Wallet key-protection gap.

Accounting: **5,701 / 16,123 read; 10,422 unread; 15,844 unknown; 0 omitted**. Reading alone closes no unknown. First unread: **5,702 `Telegram/SourceFiles/storage/download_manager_mtproto.cpp@d0007eb17fb0bcfbdffd89b86d8efdb900fcd363`**.
