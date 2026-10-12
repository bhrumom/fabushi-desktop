# UI controls delete/download/images/emoji 6002-6011 complete read

Accepted upstream: `telegramdesktop/tdesktop@65e23ba7137ea4129b6bc1b2616104a1f59495ef`.

## 6002-6003 delete ContextMenu action
The delete action may render a destruction deadline without becoming the deletion owner. It formats days, hh:mm:ss, or mm:ss according to remaining time, refreshes at bounded cadence, supports Enter/Return activation, and triggers a separate delayed expiry callback after zero. User delete and timer expiry must converge on the authoritative Message lifecycle with duplicate/idempotency fencing.

## 6004-6005 download bar
The bar projects aggregate ready/total bytes plus count/done items, transitions loading to finished, uses one thumbnail or document fallback, responds to palette changes, elides text to available width, and changes secondary copy from progress to a canonical destination link when complete. Transfer/restart truth belongs to Downloads/Resource; the bar is a status/navigation projection.

## 6006-6007 dynamic image strip
The strip subscribes to dynamic image updates, reveals items with staggered progress, dims nonselected images and enlarges hover, ignores incidental pointer movement until a motion threshold, publishes selected index/global anchor, wraps Left/Up and Right/Down navigation, and activates on Enter/Space. Parent feature owns the actual participant/media choice.

## 6008-6011 emoji button and field composition
The emoji control is a canonical IconButton specialization: ellipse ripple, hover variants, optional semantic overrides, and an infinite loading ring with reduced-motion static fallback. The factory binds the field's existing message handlers and emoji suggestions, positions the panel above or below based on current geometry, follows field/box movement, optionally fades with focus, and recomputes panel geometry before toggle/hover. These map to IconButton + Composer/TextField + Popover/Picker + emoji provider, not a source-specific EmojiButton.

## Accounting
Read-through **6,011/16,125**; unread **10,114**; unknown **15,846**; omitted **0**. First unread is **6,012** `Telegram/SourceFiles/ui/controls/feature_list.cpp@8c3bf36c62b4e9698674c0985f6d98b1156c77dc`. All ten rows remain mapped-open.
