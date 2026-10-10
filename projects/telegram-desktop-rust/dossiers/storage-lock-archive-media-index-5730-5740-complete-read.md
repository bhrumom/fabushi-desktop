# Storage file-lock, archive/media preparation and sparse indexes — orders 5730-5740

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

5,730–5,732 define exclusive storage locking on POSIX and Windows, including conflict process handling. 5,733–5,734 define folder/multi-file ZIP preparation with symlink exclusion, cancellation, size/disk failure and stale temp cleanup. 5,735–5,736 cover Composer/editor media drag/drop, MIME/limit/decode and async preview preparation. 5,737–5,740 implement source-neutral shared-media categories plus sparse message-id range merge/query/count/invalidation semantics.

All remain mapped-open under canonical Persistence/platform, Attachment/Media Editor and Shared Media/index owners. Accounting: **5,740/16,123 read; 10,383 unread; 15,844 unknown; 0 omitted**. First unread: **5,741 `Telegram/SourceFiles/storage/storage_user_photos.cpp@232befde40c5edea34286105374058fcc66ab658`**.
