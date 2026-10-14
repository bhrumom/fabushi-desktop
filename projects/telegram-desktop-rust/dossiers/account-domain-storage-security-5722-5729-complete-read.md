# Account/domain storage and local-security keying — deterministic orders 5722-5729 complete read

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

5,722–5,723 cover encrypted account persistence (drafts/cache/settings/trust/payment/webview/bot/wallet records). 5,724–5,725 cover bounded cloud-blob download/ZIP extraction. 5,726–5,727 strengthen Local Passcode/App Lock/Wallet key protection: bounded KDF wraps, worker derivation + secret cleansing, fresh verification tokens, committed-generation crash recovery, legacy migration, App Lock and wallet keyring. 5,728–5,729 expose shared-media/profile-photo cache mutation/query streams.

All remain mapped-open to existing source-neutral owners. Unsupported KDFs, stale verification, corrupt/future data, interrupted passcode changes, wallet corruption, archive failures and account/restart transitions must fail closed.

Accounting: **5,729 / 16,123 read; 10,394 unread; 15,844 unknown; 0 omitted**. Reading alone closes no unknown. First unread: **5,730 `Telegram/SourceFiles/storage/storage_file_lock.h@181b5bc063827458c4e59a0daa167355f835ae0b`**.
