# Story slider/view source-completeness dossier

Authority: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@e1ed57a44e7c14e0cbb91bcf0f7ec3e408786a39`  
Fabushi mapping baseline: `bhrumom/fabushi-desktop@3bc92400826cc4ca7ac665b467708e22261edc61`  
Status: read-complete / provenance-rebound at e1ed57a44e7c14e0cbb91bcf0f7ec3e408786a39 / mapped-open / not implemented / not verified

### Rebaseline provenance

The accepted upstream advanced 15 commits from `72b3b71c3d6e450e5ef94a3112dd750a0168aa0b` to `e1ed57a44e7c14e0cbb91bcf0f7ec3e408786a39`. GitHub exact-file reads at the new commit confirm every source entry in this dossier has the same blob SHA as the prior read. Therefore the semantic read is provenance-rebound rather than inherited blindly; no changed story blob is being treated as read.

## Exact source entries

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/media/stories/media_stories_slider.h` | `37dbca0504c3423a14665c5b692f2dc2699d5469` | complete, 64 lines | public bounded story progress contract | mapped-open |
| `Telegram/SourceFiles/media/stories/media_stories_slider.cpp` | `bdb547186edc53fbfd64dd1434f861a27fb3be0d` | complete, 157 lines | bounded index/total normalization, progress reset/update and responsive segment projection | mapped-open |
| `Telegram/SourceFiles/media/stories/media_stories_view.h` | `0f3e59e99503aa4924937ac715eccdbcd0966fa3` | complete, 149 lines | public Story viewer interaction/lifecycle capability contract | mapped-open |
| `Telegram/SourceFiles/media/stories/media_stories_view.cpp` | `8fe4a5de24fd67abe0b64da0a45ec4e0fd901b30` | complete, 195 lines | delegation of navigation/playback/actions/reactions/comments/caption/repost/stealth lifecycle | mapped-open |

All four files were read in full at the accepted upstream commit. No truncated rendering is credited as complete.

## Recovered responsibilities

### ST-SLIDER-01 — bounded story progress state

Symbols: `SliderData`, `Slider::show`, `Slider::resetProgress`, `Slider::updatePlayback`, `Slider::layout`, `Slider::paint`.

The product responsibility is not the Qt painter. The transferable contract is:

- `total` is normalized to at least one and `index` is clamped to `[0,total-1]`;
- switching Story data resets playback progress before projecting the new active segment;
- playback state updates only the active segment progress;
- segment count is bounded by available width, with undisplayed trailing segments represented as non-visible projection rather than changing canonical Story ordering;
- static/story-image presentation may render local progress while video-stream projection may be owned by the video playback surface.

This must map to the canonical Story/Resource playback truth and a typed Story capability component. It must not create a second Story state store or source-specific slider owner.

### ST-VIEW-01 — Story capability-surface lifecycle and navigation

Symbols: `View::show`, `ready`, `story`, `finalShownGeometry*`, `contentLayout`, `closeByClickAt`, `subjumpAvailable`, `subjumpFor`, `jumpFor`, `lifetime`.

The view is a thin public façade over the Story controller. Transferable behavior is stable Story identity/context, ready/teardown lifecycle, close-hit semantics, bounded same-source and cross-source navigation, and lifetime fencing. Fabushi already has a canonical Rust Story owner in `native/mahayana-messaging/src/story.rs`; any shipping viewer must be a typed ProductShell capability surface/overlay that consumes this truth, not a parallel application or Story store.

### ST-VIEW-02 — playback pause/input/menu continuity

Symbols: `updatePlayback`, `paused`, `togglePaused`, `contentPressed`, `menuShown`, `tryProcessKeyInput`, `ignoreWindowMove`, `updateVideoStream`.

Press/hold, menu visibility, keyboard input and video-stream changes all participate in one playback/pause lifecycle. The eventual Fabushi surface must preserve pause reason/state continuity across interaction and input, while Resource/media playback remains the canonical playback owner. Qt mouse/window mechanics are presentation-replaced, but the interaction state machine is not N/A.

### ST-VIEW-03 — Story object actions and derived projections

Symbols: `shareRequested`, `deleteRequested`, `reportRequested`, `toggleInProfileRequested`, `allowStealthMode`, `setupStealthMode`, `attachReactionsToMenu`, `commentsShownValue`, `fileOrigin`, `captionText`, `skipCaption`, `repost`, `lookupRepostHandler`, `showFullCaption`.

These actions must reuse existing canonical owners: Story identity/privacy/delete/pin/reaction in the messaging Story owner; Share/Forward through canonical Message/Conversation/Search/permission owners; media/file origin through Resource; participant/profile routing through canonical Profile; comments/replies through Conversation/Transcript when applicable. No StoryShare, StoryReactionStore, StoryCommentsStore or Telegram-specific UI root is justified.

## Existing-owner-first audit

Current exact main `3bc92400826cc4ca7ac665b467708e22261edc61` contains `native/mahayana-messaging/src/story.rs` plus protocol/service/engine commands for publish/delete/view/react and canonical Story persistence. Existing Resource/attachment/media owners remain responsible for media bytes/cache/playback; ProductShell already owns capability overlays. A scan of the shipping `frontend/src/production/ProductionRenderer.tsx` did not establish a shipping Story viewer composition. Therefore:

- Story domain truth: existing owner;
- Resource/media playback: existing owner;
- Story viewer UI: mapped to ProductShell capability-surface/overlay slot, but shipping implementation/evidence remains open;
- actions must call existing canonical owners rather than create a source-specific subsystem.

This is a mapping/read-completeness advancement only. It is not evidence that the Story viewer, slider, keyboard/pause lifecycle, stealth/comments/repost actions, or packaged journey are implemented.

## Required production/test evidence before implementation can become verified

At minimum, future implementation must add exact-head GitHub Actions evidence for:

- clamp/reset/progress temporal behavior, including Story switch while playback is active;
- same-source and cross-source navigation bounds and stale/lifetime fencing;
- press/menu/keyboard/video-stream pause continuity;
- delete/report/profile/share/reaction/comments permission and settlement semantics;
- resource/media failure and reconnect/reload behavior;
- canonical ProductShell capability-surface route/back/close behavior, keyboard/a11y, responsive layout and visual timeline;
- packaged temporal acceptance with stable Story identity and no duplicate Story/Resource owner.

No row from these four files is marked implemented or verified by this dossier.

Coverage after these four complete reads: `unread=15,757`, `unknown=15,788`, `omitted=0`, `baseline_ready=false`, `acceptance.accepted=false`.
