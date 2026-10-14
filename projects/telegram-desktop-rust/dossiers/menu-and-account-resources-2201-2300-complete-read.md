# Telegram source read: deterministic orders 2201–2300

Authority: accepted `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db`, tree `94ae09469c816b350f60dc9ada1ff049323be8e7`; binary/generated evidence is run `37866391182`, job `113614324572`, artifact `11588621238` (`sha256:ec0af670db24bc03f3b9f874abf1223e2d3c8b89778dc4499beb57c78070b6a5`).

Real caller audit separates: unblock/moderation and Bot restart state; sticker favorites; unique Gifts; notifications/unmute; pin state across Conversation/Stories/Gifts; mark-unread; Stars rating; Calls/video chat; reply-thread navigation; Violence reporting taxonomy; Wallet; Business welcome messages; Media zoom; PSA/forward/story-reply metadata; gift crafting; notifications monitor/schedule toggle; avatar change; shared-media selection/play/download; Passkey QR rendering; Passport identity flow; and checkout address/card/email/name/phone fields. Payment email/name/phone are treated as payer identity/contact inputs owned by checkout, not inferred as generic payment transports.

Five legacy resource families have no accepted-tree generated consumer and no whole-repository exact resource reference: `menu_settings`, `menu_shadow`, `notification_send`, `overview_links_check`, `overview_links_check_bg`. Their 15 density files are explicitly unreachable/non-applicable and do not create Fabushi features. `zoom_in`, `zoom_out`, `notify_toggle`, and `passkey_qr_center` also lack generated token lines but have direct C++/QRC caller evidence, so they remain applicable.

No Telegram artwork is imported and source reading grants no implemented/verified credit.
