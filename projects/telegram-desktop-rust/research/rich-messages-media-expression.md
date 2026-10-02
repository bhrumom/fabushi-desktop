# Telegram rich messages, media streaming, and expression research dossier

Status: researched and owner-resolved for polls/rich variants, media playback/streaming, and sticker/GIF/custom-emoji expression; implementation/acceptance remains open
Frozen upstream: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`
Canonical Fabushi architecture snapshot: PR #20 `bbc7b34a5f6dad46e3d4ca88fe21cc4f7932ce09`

## MSG-POLL-RICH

Frozen `Telegram/SourceFiles/data/data_poll.cpp` models a poll as versioned message-attached state: question/options, closed/public/multi-choice/quiz policies, close timers, authoritative result updates, voter/result visibility and optional media. Timer close schedules an authoritative refresh instead of treating the optimistic close as final.

Plausible Fabushi owners inspected: typed Transcript/card registry, generic widget card, Task/Automation, a new poll message store. Selected `existing_owner = typed transcript/message model + transcript-card registry`; a poll is a typed message payload/card, not a second history owner. Exact seams include `frontend/src/recovered/features/conversation/cards/transcript-card/registry.ts`, `resolver.ts`, `views/widget.tsx`, `source/shared/transcript.ts`, and existing Session/Transcript persistence.

Required state/network contract: canonical message identity plus typed poll payload/version; idempotent vote operation IDs; authoritative result revision; timer/close state; privacy/visibility policy; stale result rejection and gap recovery through the same conversation sync infrastructure. Focused tests: create/render, single/multi vote, duplicate vote settlement, stale result, timer close/restart, result visibility, deleted base message. Current blocker: no native poll operation/sync path or dedicated poll card contract.

## MEDIA-STREAM-CACHE

Frozen `Telegram/SourceFiles/data/data_streaming.cpp` shares readers/documents per media object, chooses remote loaders when required, carries alternative video qualities, uses a large-file cache, and keeps recently used streaming documents alive only for a bounded interval. This complements the upload/download behavior already researched in `rooms-search-media.md`.

Plausible Fabushi owners inspected: existing attachments/resources, transcript attachment cards, workspace media viewer/voice surface, Electron media protocol, Host attachment service, a new media product store. Selected `existing_owner = existing attachments/artifacts/resource lifecycle + existing media viewer/transcript cards`, with minimal resumable media transfer/cache infrastructure below them. Exact seams: `source/host/src/extensions/attachments/*`, `source/electron-main/attachments/*`, `source/electron-main/media/media-protocol.ts`, `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx`, `voice.tsx`, and transcript-card attachment views.

Required contract: resource identity remains canonical attachment/artifact identity; resumable ranges/chunks, hash verification, cache eviction, cancellation/backpressure, quality/rendition metadata and crash-safe partial transfer live below it. Streaming must not mint duplicate message/resource truth. Focused tests: range resume, duplicate chunk, hash failure, cancel/restart, cache eviction, quality switch, offline playback where cached, bounded memory/CPU, large media and platform file handling. Current blocker: native remote blob/media service and cross-device media settlement do not exist.

## EXPRESSION-STICKER-GIF-EMOJI

Telegram's chat-helper/data/media expression families distinguish catalog/search/selection from the message that references the selected asset. Fabushi currently has an emoji catalog/picker in the transcript reaction surface but no canonical sticker/GIF/custom-emoji catalog owner.

Plausible owners: Composer, existing reaction/emoji picker, attachments/resources, transcript cards, new expression subsystem. Selected `existing_owner = Composer + existing rich transcript/reaction expression surfaces`, with catalog/cache data treated as supporting resource metadata; no second conversation or message owner. Exact current seams include `frontend/src/recovered/features/conversation/cards/transcript-card/emoji-catalog.ts`, `emoji-picker-content.tsx`, reaction picker/actions, Composer, and attachments/resources for binary assets.

Required contract: stable expression/resource IDs independent of Telegram IDs, search/catalog pagination, entitlement/availability policy where applicable, cache/version invalidation, accessibility labels, and message/reaction references settled through canonical message operations. Focused tests: search/pagination, unavailable/stale asset, cached/restart rendering, custom emoji fallback, GIF autoplay/power policy, keyboard/a11y, reaction-vs-message semantics. Current blocker: sticker/GIF/custom-emoji catalog and native sync/resource distribution are not implemented.
