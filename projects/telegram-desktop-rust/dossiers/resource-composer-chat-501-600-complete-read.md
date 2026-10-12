# Chat/composer resource responsibilities 501-600 — complete source read

Authority: telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db / tree 94ae09469c816b350f60dc9ada1ff049323be8e7.

GitHub Actions run 37849904823, source-authority job 113560126953, artifact 11580638761, digest `sha256:66e4351888c8ca0cc8019a175447a5b886eeab04863879ffea9f355bb2b01bf9` records exact blob, byte size and file type for every order 501-600. `source-consumer-reachability-501-600.txt` performs accepted-tree grep for input_liked, input_record, input_record_filled and input_video and records no named hits outside the asset directory.

## Responsibility groups

501 AI tone; 502-504 and 506-508 ephemeral/once media; 505 managed Bot code; 509 gift/forge commerce; 510-513 location/request; 514-519 attachment/auto-delete; 520-528 Bot command/keyboard; 529-534 comments; 535-550 edit/forward/gift/like; 554-557 reply/link/paid; 564-575 replace/reply/save; 597-599 emoji. These map to canonical Fabushi owners and do not create Telegram-shaped roots.

551-553 input_liked, 558-563 input_record/input_record_filled, and order 600 input_video stay consumer-reachability open. Exact source bytes are read; semantics are not guessed from filenames.

## Schedule/silent 576-596

Telegram's schedule/scheduled/send/silent affordances bind to the already-existing Fabushi shipping chain rather than a new owner:

`ConversationComposer/Menu -> submission queue -> Coordinator/Host native_messaging -> bhrumom/fabushi D1 direct-message owner`.

Canonical server PR #2746 persists `silent` and `scheduledAtMs`, includes both in idempotent `clientRequestId` payload equality, keeps future messages hidden from the recipient until due, and activates due rows through the single durable Worker+D1 owner. Desktop keeps compare-and-remove draft/stash fencing and the existing submission queue.

Source-read closure does not waive current-head/release evidence requirements.
