# Menu resources 1544-1602 - complete read dossier

Authority: telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db / tree 94ae09469c816b350f60dc9ada1ff049323be8e7.

## Boundary and method

Orders 1544-1602 are menu/icon source resources. This batch is read by exact accepted-tree bytes plus exact style consumers, then by following the style symbols into C++ product call sites. It is not treated as a single “menu icons” capability and no row receives implementation credit merely because Fabushi has a visually similar icon.

The `block` scale family begins at order 1600 and continues through orders 1601-1602, so this dossier deliberately crosses the 1600 evidence-file boundary rather than truncating the responsibility at an arbitrary hundred-entry boundary.

Exact binary/consumer evidence comes from the current accepted upstream source-attestation artifacts for ranges 1501-1600 and 1601-1700.

## Responsibility families

- **1544-1549 2SV / privacy-policy presentation.** `menuIcon2SV` is used by Privacy & Security cloud-password settings. The “off” art is also consumed as a bot profile privacy-policy action icon. This is security/account and bot-policy presentation, not one boolean icon state.
- **1550-1552 generic Add.** Reused by info/media and Story-set menus; this is a generic action affordance with multiple product consumers.
- **1553-1555 Add account.** `settings_main.cpp` starts another account/session through the application domain. Fabushi account/session ownership must be used if this responsibility is implemented.
- **1556-1558 Add to folder.** Used by peer menus, moderation flows, dialog suggestions, profile actions and gifts to choose filters/folders/administered groups. This is conversation/filter membership behavior, not icon-only UI.
- **1559-1564 Admin / promote.** Used by participant-role editing and community/profile administrator surfaces. Requires permission-aware participant/admin state.
- **1565-1570 Affiliate descriptors.** `affiliate_simple` and `affiliate_transparent` describe Bot Stars referral/affiliate program properties in the join surface.
- **1571 AI prompt.** `compose_ai_box.cpp` uses the prompt icon as a dedicated AI compose tone/prompt tab. Any Fabushi adaptation belongs to canonical Composer/AI assistance, not a Telegram-specific button.
- **1572-1574 Show all / all media.** Reused by media overview, emoji-pack/media menus, IV slideshow conversion and credits/unique-attribute surfaces; consumers are independent and remain open until separately mapped.
- **1575-1577 Antispam / privacy / trust descriptor.** Used by group anti-spam controls, bot webview privacy and affiliate reliability presentation; a shared raster does not merge those responsibilities.
- **1578-1583 Archive / archive-open.** Used for chats, Stories, sticker sets, archive settings and the main-menu Archived Chats entry. Archive state and persistence are product behavior.
- **1584 Rich article premium choice.** Consumed by the IV editor premium-choice surface. It belongs to the IV/editor responsibility, not generic media.
- **1585-1593 Stars auction explanation.** Carryover/drop/refund icons are feature-list semantics in the Star Gift auction flow.
- **1594-1599 TTL / auto-delete.** `menuIconTTL` and `menuIconTTLAny` are used by global Privacy & Security TTL, send-file TTL, contact/menu TTL and the reusable TTL menu. This requires real expiry state/protocol/persistence.
- **1600-1602 Block family.** Used by blocked-users settings, peer/history actions, sponsored-content reporting, moderation attention variants, translation ignore actions and media-view/report presentation. The family is broader than a single “block user” button.

## Accounting and fail-closed status

Every resource in orders 1544-1602 has been read with its exact source consumer and decomposed into the responsibility families above. The deterministic read-through can therefore advance from 1543 to 1602.

No family in this dossier is promoted to verified merely from source reading. Existing Fabushi Account/Conversation/Identity/Composer/Resource owners must be audited responsibility-by-responsibility and backed by exact-head production/test evidence before `unknown_closed` changes. Accordingly unknown remains 15,991, omitted remains 0, and unread decreases from 14,577 to 14,518.
