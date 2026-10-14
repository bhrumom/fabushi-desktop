# Exact-source dossier: orders 5101-5110 — Story reactions, views, reply, share and sibling preview

Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`, tree `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`. Complete direct exact-blob reads. Accounting after batch: read-through 5,110; unread 11,010; unknown 15,841; unknown-closed 279; omitted 0.

## 5101 — reaction action contract

The reaction owner exposes Message-vs-Reaction selection mode, liked identity/value, reply-focus/send-text arbitration, selector/menu attachment, custom liked-icon loading and bounded fly-animation lifetimes. It is a view/action projection over authoritative Story reaction state, not a second reaction store.

## 5102-5103 — recent views and reactions detail

Recent Views projects Story type and permission before exposing viewers/reactions. It maintains userpic strips and counters, opens a detailed menu with bounded incremental population (50-row additions and a bounded initial page budget), keeps placeholders while data is incomplete, and converges delayed userpic loads through explicit lifetimes. Viewer visibility and reaction visibility are authorization concerns and must remain server/canonical-policy driven.

Canonical owner: **Story views/reactions detail Popover**.

## 5104-5105 — Story reply/comment composer

`ReplyArea` is not merely a text box. It reuses compose controls for Story replies/comments and video-stream comments; derives placeholders from stealth mode, reply type and paid-message price; sends text, reactions, voice, existing photos/documents, inline results and prepared file bundles; checks media/chat restrictions, slowmode and sending errors; distinguishes scheduled sends from ordinary slowmode enforcement; performs paid-message balance approval before retrying the exact send; and clears compose/listen/reply state only on the successful send path. Target/attachment chooser and payment callbacks are lifetime guarded.

Fabushi must absorb these semantics into the **canonical Composer + existing Story reply/Conversation send owner**. No Story-specific message transport or draft store is allowed.

## 5106-5107 — repost attribution

The repost view derives source peer/name and original Story/channel-post identity, lazily resolves a missing source Story, distinguishes deleted from still-loading source state, renders bounded quoted attribution, hit-tests only the rendered region and navigates through the existing Story/history owner. Async resolution and ripple callbacks are lifetime guarded.

Canonical owner: **Story repost attribution/link surface**.

## 5108-5109 — share and share-at-time

The source resolves the exact Story/message before opening the flow, supports copy-link only when an authoritative direct link exists, distinguishes call/link-only Stories from media forwarding, filters recipients by the rights required for the content/inline result, carries paid-message counting/approval through the existing send path, and preserves video timestamps for share-at-time. This belongs in the canonical Forward/Share dialog and send owner rather than a second ShareBox runtime.

## 5110 — sibling preview

Adjacent Story preview resolves the currently valid Story id from the live source, lazily upgrades blurred media to a good photo/video thumbnail, uses streamed-video fallback and failure fencing, and fades the settled preview while preserving hover/name geometry. Loader callbacks are lifetime scoped, and zero/invalid geometry is guarded rather than painted into a null image.

Canonical owner: **Story sibling navigation/preview**.

## Acceptance

All ten entries are exact tree/path/blob bound and `read_complete=true` only. Production implementation, service composition, canonical UI/a11y/responsive closure, exact-head test evidence and independent release acceptance remain open. Same-head Source authority must attest the shard and manifest.
