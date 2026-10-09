# Storage media preparation and durable serialization — deterministic orders 5712-5721 complete read

Accepted upstream: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

Orders 5,712–5,713 cover media-send preparation and task/result lifetime: type detection, image/video/song/voice/round/sticker/archive preparation, thumbnails/large-photo policy, upload-ready metadata and cancellation. Orders 5,714–5,717 cover local settings/theme/background/language/update persistence, legacy encrypted migration and bounded serialization primitives. Orders 5,718–5,721 cover versioned document/media and user/chat/channel cache persistence, legacy location migration and fail-closed reconstruction.

Canonical mapping stays with existing source-neutral messaging Attachment/media-preparation, Persistence/Settings/Local Security/Updater/Theme/Localization, messaging data persistence, and Identity/Profile/Conversation owners. Telegram/MTP/Qt formats are migration inputs only and do not create a second runtime or UI family.

Production closure still requires cancellation/account-switch/teardown fencing; malformed media and encode/archive failure; wrong-key/tamper/corrupt/truncated/future-version persistence; interrupted atomic writes; legacy migration; cache-vs-server authority reconciliation; reload/restart; and canonical UI/a11y evidence.

Accounting: **5,721 / 16,123 read; 10,402 unread; 15,844 unknown; 0 omitted**. Reading alone closes no unknown. First unread: **5,722 `Telegram/SourceFiles/storage/storage_account.cpp@5888ffa1164327b7170a2a3d8df7793a3fa467da`**.
