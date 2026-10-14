# Menu resources 1603-1700 - complete read dossier

Authority: telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db / tree 94ae09469c816b350f60dc9ada1ff049323be8e7.

Exact source authority: Fabushi PR #42 HEAD ae11d378f92ce19e42e9ccdf59494461672809f3, GitHub Actions run 37861966662, Source authority job 113599732231, artifact 11585584799, digest sha256:094e1f221b8078f62f6cd9f313409317ea82d0f2733e10acd8d5f1ad4c32be55.

## Responsibility decomposition

- **1603 blockchain verified** — wallet “about chain” / chain-verification presentation. This is wallet/trust metadata, not generic blocked-state UI despite the nearby block family.
- **1604-1606 boosts** — channel/peer boost actions and boost management.
- **1607-1609 bot** — bot identity/profile/open-bot presentation, reused in affiliate flows.
- **1610-1612 bot add** — adding a Bot to Stars affiliate/referral flows.
- **1613-1615 bot commands** — bot command/settings management.
- **1616-1618 calendar** — shared schedule/date action used by message scheduling, add-to-calendar, date sorting and Story/media menus. Consumers are independent.
- **1619-1621 calls receive** — notification setting that enables/disables receiving calls on the current device.
- **1622-1624 cancel** — generic cancel/remove affordance reused by emoji recent removal, media cover removal and attention variants; it carries no single domain responsibility by itself.
- **1625-1627 cancel fee** — peer-menu fee-removal state/action.
- **1628-1633 caption hide/show** — forwarding and send-media caption inclusion state; this changes message payload semantics and cannot be reduced to display.
- **1634-1636 channel** — channel creation/open/navigation and sponsored-source/channel context.
- **1637-1639 chat bubble** — send-message/open-chat affordance plus chat-related settings imagery.
- **1640-1642 chat discuss** — discussion/monoforum/topic navigation and related permission/support surfaces.
- **1643-1645 chats** — chat/direct-message collection and monoforum/direct-message management.
- **1646-1648 clear** — clear-history destructive action.
- **1649-1651 collapse** — persisted archive collapse and media-tab expansion/collapse state.
- **1652-1654 colors** — chat theme/profile color/theme editing and gift-theme presentation.
- **1655-1656 community/community remove** — community navigation/service state and destructive community removal.
- **1657-1659 copy** — copy text/link actions across AI compose, translation, giveaway, media and context menus; clipboard effect belongs to each caller.
- **1660-1662 copyright** — no-forward/copyright restriction/report category presentation.
- **1663-1666 gift craft** — Star Gift craft chance/random traits/start/tools semantics in the real gift-crafting service flow.
- **1667-1669 create poll** — actual Poll creation from peer/media/bot attachment surfaces; also reused in poll-vote notification settings. This is a distinct product capability and directly confirms the earlier Poll preview gap cannot be treated as implemented.
- **1670-1672 customize** — context-specific custom TTL, mute duration, Poll duration and peer privacy/type configuration.
- **1673-1675 delete** — destructive deletion across messages/downloads/media/language/ringtones and other domain objects; each caller owns authorization and persistence.
- **1676-1678 devices** — active sessions, website sessions and URL-auth device metadata under Privacy & Security.
- **1679-1681 disable** — attention/destructive disable/revoke state used for TTL disable and bot/contact revocation.
- **1682-1684 discussion** — forum/topic/discussion navigation plus support/feedback entry.
- **1685-1687 dock bounce** — platform notification attention behavior, especially macOS/Linux alert/bounce settings.
- **1688-1690 download** — real save/download action and Downloads navigation.
- **1691-1693 download locked** — media-view download action in a restricted/locked state; permission/content-policy semantics must remain fail-closed.
- **1694 download off** — disabled/off download state in the shared download menu family.
- **1695-1697 drugs** — moderation/report category presentation; this is a report taxonomy item, not drug functionality.
- **1698-1700 earn** — value/earn/price action reused by fee state, gifts/commerce sorting/offers and Story/media price menus.

## Migration status

All entries are exact-source read and responsibility-decomposed. No implementation or verification credit is granted in this dossier. Each applicable responsibility must be matched against the current canonical Account, Conversation, Identity, Resource, Composer, moderation, notification, wallet/Marketplace and service owners with real protocol/state/persistence evidence.

The deterministic read-through advances from 1602 to 1700. Unknown remains 15,988, omitted remains 0, and unread decreases from 14,518 to 14,420.
