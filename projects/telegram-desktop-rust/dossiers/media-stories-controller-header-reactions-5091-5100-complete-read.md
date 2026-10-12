# Exact-source dossier: orders 5091-5100 — Story controller, caption, header and reactions

Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`, tree `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`. Complete direct exact-blob reads. Accounting after batch: read-through 5,100; unread 11,020; unknown 15,841; unknown-closed 279; omitted 0.

## 5091 — Story stealth presentation contract

The style source defines the Stealth dialog's title/body, feature rows, lock/logo treatment, close control and primary action states. This is presentation evidence only: Fabushi must express the same entitlement/status semantics through canonical Dialog/AlertDialog, Button, IconButton, Badge and semantic tokens, including keyboard/focus, disabled state and light/dark contrast. No source-specific component root is justified.

## 5092-5093 — expanded caption lifecycle

`CaptionFullView` is a controller-bound, scrollable expanded caption overlay. It strips quote entities for the expanded presentation, preserves marked-text/link behavior, updates geometry from the live Story layout, supports repost hit testing, closes on Escape or an outside primary click only when no active/pressed link owns the gesture, tracks focus-chain state, and animates from collapsed to expanded geometry before settling `captionClosed`. Pointer pulling/down/closing state and lifetime disposal are part of the responsibility, not merely styling.

Canonical owner: **Story viewer caption DetailPanel/overlay**, reusing the existing Story/Media viewer state and URL/navigation owners.

## 5094-5095 — Story viewer controller

The controller is the high-density Story lifecycle owner in the source. It binds exact Story/context/list identity; builds the responsive viewer layout; coordinates photo/video playback and unsupported-media fallback; bounds peer/story/media preloading; settles read state only after the configured viewing/progress threshold; handles pause/play permission, power-save-sensitive playback, volume and seek state; composes sibling navigation, caption/repost, reactions, recent views, reply, share, report and stealth surfaces; and retires old media/lifetimes when the active Story or source context changes.

The responsibility must be absorbed into the existing canonical `StoryCapabilitySurface` plus Story viewer/MediaViewer and canonical service owners. It must not become a Telegram Story runtime or duplicate Story store.

## 5096-5097 — delegate boundary

The delegate is a thin viewer-composition callback seam. Its semantics are to hand actions back to existing navigation/action owners for the life of the viewer, not to own Story state. The Fabushi adaptation remains source-neutral and lifetime-bound.

## 5098-5099 — Story header

The header projects peer/avatar/name, repost source, Story position/count, privacy badge, edited/video/silent state and either changing date text or live video-stream viewer count. It owns play/pause and volume presentation, privacy/silent tooltips, responsive volume geometry, hover/disabled states and window-drag exclusion. Date text uses a bounded next-change timer and is suspended while the live-viewer projection owns that slot; subscriptions/timers are lifetime-scoped.

Canonical owner: **Story viewer Toolbar/Header**, using Avatar, Badge, IconButton, Tooltip/Popover and canonical media controls.

## 5100 — Story reactions and interactive areas

The reaction surface projects authoritative Story-area/reaction metadata into bounded geometry and hit testing, reaction counts, custom emoji, suggested reaction bubbles and weather areas. It lazily binds emoji/sticker resources, supports Celsius/Fahrenheit projection, and runs reaction fly/fade effects with explicit stopping/cleanup lifetimes. These are derived effects around the existing canonical Story/reaction service truth; they must not create a second reaction store.

Canonical owner: **Story reaction action/area owner** composed in the existing Story viewer.

## Acceptance

All ten entries are exact tree/path/blob bound and `read_complete=true` only. Production implementation, server/service closure, canonical UI composition, a11y/responsive verification, focused integration/E2E, current-head artifacts and independent release acceptance remain open. The same-head Source authority job must attest the new disposition shard and source-attestation manifest.
