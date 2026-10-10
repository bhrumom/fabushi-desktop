# Storage transfer — deterministic orders 5702-5711 complete read

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

This batch covers adaptive download session balancing and priority generations; request/session redirect on timeout removal; file-reference refresh; CDN reupload/hash verification; generic file-loader local/cache/cloud fallback and partial resume; file-open/write cleanup; bounded part loaders; HTTP(S)-only web transfer with redirect downgrade/scheme protection and TLS/auth failure; and multipart uploads with adaptive part/session sizing, async preparation cancellation, removed-session resend, progress/failure projection, pause/cancel, big-file mode and two-stage video-cover completion.

Canonical mapping stays with Fabushi messaging attachment/file-transfer owners and platform download/save adapters. MTProto/DC/CDN specifics are not retained as a parallel runtime. Production closure must prove stale-result fencing, retry/idempotency, partial/disk/network failure, integrity, teardown/account switch, upload preparation cancellation and reload/restart recovery.

Accounting: **5,711 / 16,123 read; 10,412 unread; 15,844 unknown; 0 omitted**. Reading alone closes no unknown. First unread: **5,712 `Telegram/SourceFiles/storage/localimageloader.cpp`**.
