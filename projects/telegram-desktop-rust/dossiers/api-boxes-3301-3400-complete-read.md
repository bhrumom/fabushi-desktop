# Telegram source read: deterministic orders 3301-3400

Authority: accepted `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`; exact object/size/type remains bound to Source authority run `37869950664`, job `113625536646`, artifact `11589926169`, `upstream-recursive-inventory.json`.

This batch was read from exact C++/header/style sources. It includes sticker creation, suggested posts, text entities, todo lists, media toggles, transcription/summarization, unread things, update ingestion, usernames, privacy, views, web authorizations, read/reaction participants and the broad `ApiWrap` façade, followed by product boxes for contacts, auto-download/auto-lock, wallpapers, filters, AI compose, proxies, polls, deletion, dictionaries, download paths, captions, privacy/todo editing, gifts, localization, moderation, music attachments, passcode and peer pickers.

Multi-owner files are not mechanically collapsed: `api_toggling_media` spans Media saved/favorite state and Notifications ringtone persistence; `api_user_names` spans Identity/Profile, Community/channel and Bot username state; `apiwrap` is recorded as a shared façade whose behaviors remain owned by the underlying canonical domains. Box UI maps to the canonical Fabushi design-system controls rather than introducing Telegram-derived specialized components.

All rows remain mapped/open. Source reading does not satisfy product implementation, persistence/restart, fault/security, UI/a11y/visual, exact-head workflow or packaged release evidence.

Accounting: `read_through=3400`, `unread=12720`, `unknown=15841`, `omitted=0`, `unknown_closed=279`.
