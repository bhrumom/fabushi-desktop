# QR component — deterministic read 6,977–7,028

Authority: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941` -> `nayuki/QR-Code-generator@720f62bddb7226106071d4728c292cb1df519ceb` (MIT), component tree `c0807c6a96a18b9eb3ba8e3113da8e4e3e3c75d3`.

All 52 non-directory entries were read and responsibility-decomposed across C/C++, Java/fast-Java, Python, Rust/no-heap, Rust, TypeScript/JavaScript, tests, demos and package/build inputs. Preserved contracts include versions 1–40, four ECC levels, numeric/alphanumeric/byte/Kanji/ECI segmentation, bounded smallest-fit version selection, optional ECC boost, explicit/automatic masks with penalty scoring, Reed–Solomon ECC/interleave, typed too-long/invalid-input failure, out-of-bounds module-as-light behavior, no-heap caller-buffer safety, and bounded border/scale rendering.

## Existing-owner-first audit

Exact parent HEAD `274e39e8a47635b1587877ba5daa97a40ec72bd1` already has RemoteControl pairing ownership for pairing/manual codes, auth/token refresh, expiry/status, persistence and stale-response fencing; SharedRoom ownership for `shareUrl`, expiry and room lifecycle; and canonical DeepLink parsing/normalization. QR therefore remains a stateless source-neutral representation/codec concern behind those owners plus canonical Resource presentation. No QR/Identity/Conversation/Computer duplicate owner or ADR is created. A reusable codec owner/dependency remains mapped-open until downstream product consumers prove it necessary.

Reading grants no production or verification credit. Focused QR dependency/codec/rendering tests and fresh descendant exact-head GitHub Actions remain required.

Accounting: **7,028 / 16,125 read; 9,097 unread; 15,845 unknown; 0 omitted**. First unread: **7,029** `Telegram/ThirdParty/TooManyCooks::.clang-format@b484c8323870e494c8031a35ddac49aa114052b3`.
