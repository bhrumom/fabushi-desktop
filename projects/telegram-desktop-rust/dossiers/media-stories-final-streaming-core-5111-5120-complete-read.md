# Exact-source dossier: orders 5111-5120 — final Story surfaces and streaming audio contracts

Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`, tree `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`. Complete direct exact-blob reads. Accounting after batch: read-through 5,120; unread 11,000; unknown 15,841; unknown-closed 279; omitted 0.

## 5111 — sibling preview contract

The header completes the sibling preview responsibility already read in 5110: exact FullStoryId/peer identity, weak lifetime, lazy media loader, userpic/name image caches and hover transition state all belong to the one Story sibling preview owner.

## 5112-5113 — Story sequence progress

The slider clamps index/total, resets progress on Story data replacement, derives responsive per-Story segments, suppresses the ordinary segment paint for video-stream mode, and updates only the active segment from canonical playback progress. This is a Status/progress projection, not a playback owner.

## 5114-5115 — stealth entitlement/action

Stealth mode combines authoritative mode timing, premium eligibility and current time into active/cooldown/available states; updates countdown text; shows activated/already/cooldown feedback; offers premium upgrade where required; and fences activation so repeated clicks do not create duplicate requests. The activation callback occurs through the authoritative mode/action flow.

Canonical owner: **Story privacy/stealth entitlement action** with canonical Dialog/Menu/Toast surfaces.

## 5116-5117 — Story View facade

`View` owns one controller and forwards the Story surface's public queries/actions: layout, playback, sibling navigation, pause, content press, menu, share/delete/report/profile actions, keyboard handling, stealth setup, reaction menu attachment, caption/repost, live video-stream update and comments visibility. Fabushi should keep one `StoryCapabilitySurface` composition root and must not reproduce a second Story controller.

## 5118-5119 — streaming audio track

`AudioTrack` validates the stream and callback contract, decodes packets until an exact first frame can establish state, handles initial seek-frame skipping, initializes the canonical mixer only after valid information exists, emits readiness once, then supports packet enqueue/forced buffering, pause/resume, speed, volume and irreversible stop. Playback position subscribes to the matching mixer identity, distinguishes waiting-for-data, paused, terminal and error states, clamps time to the stream duration and tears down on destruction. The source explicitly separates decode-thread state from main-thread controls.

Canonical owner: **Media streaming audio runtime**, adapted to Fabushi's existing player/runtime rather than copied as a Telegram mixer.

## 5120 — streaming common protocol

The common header defines the behavioral vocabulary: Both/Audio/Video/Inspection modes; seek position, duration override, bounded speed/volume, audio identity, audio/video sync, shown gating, hardware allowance, seekability and looping; typed track/information/preload/playback/waiting/speed/muted/finished/error updates; and bounded frame request/format contracts including ARGB/YUV/NV12/native texture. Unknown time and unavailable duration are distinct sentinels, and frame area is bounded.

## Acceptance

All ten entries are exact tree/path/blob bound and `read_complete=true` only. Streaming implementation, concurrency/performance/fault evidence, Story UI/service closure, same-head Actions and independent release acceptance remain open.
