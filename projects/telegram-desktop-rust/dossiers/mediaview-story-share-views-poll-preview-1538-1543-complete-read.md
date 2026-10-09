# Media-view Story share/views and Poll preview 1538-1543 - complete read dossier

Authority: telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db / tree 94ae09469c816b350f60dc9ada1ff049323be8e7.

## Exact source boundary

Orders 1538-1540 are `Telegram/Resources/icons/mediaview/viewer_share{,@2x,@3x}.png`. The exact consumer is `Telegram/SourceFiles/media/view/media_view.style:124` (`mediaviewShare`).

Orders 1541-1543 are `Telegram/Resources/icons/mediaview/views{,@2x,@3x}.png`. This asset family has two distinct exact consumers and therefore cannot be collapsed into one responsibility:

- `Telegram/SourceFiles/boxes/polls.style:167` -> `pollAttachView`.
- `Telegram/SourceFiles/media/view/media_view.style:1213` -> `storiesViewsIcon`.

The exact accepted-tree hashes, byte sizes, file(1) types and consumer trace are retained by PR-head run 37861531152 source-authority job 113598179987 artifact 11585889089, digest sha256:66fd1e23df5d008980e458187af23b2b6500b43218de4ea59b2582791a0124eb. The source-attestation step completed successfully; the job later failed only because the persisted inventory narrative used `unknown is` instead of the validator-required `unknown remains`. This artifact is source evidence, not a passing acceptance run.

## Story share responsibility

Telegram media-view only exposes the share control when the displayed media is a Story and `Story::canShare()` is true. Current upstream `Data::Story::canShare()` requires all of:

- public privacy;
- forwarding is not forbidden;
- the Story is still live or is pinned in the profile.

Fabushi already owns the Story surface in `frontend/src/production/StoryCapabilitySurface.tsx` and the canonical Story state in `native/mahayana-messaging/src/story.rs`. The native model already carries `privacy`, `pinned_to_profile`, `protected_content`, and expiry. The current change extends the Coordinator DTO/parser to retain that source-neutral state and gates the canonical Share action by `privacy.kind === "everyone"`, `!protectedContent`, and `pinnedToProfile || expiresAtMs > nowMs`. The action rechecks against `Date.now()` to avoid an expiry race. No Telegram share icon/artwork is imported.

Status: verified on exact HEAD ae11d378f92ce19e42e9ccdf59494461672809f3. GitHub Actions run 37861966662 renderer job 113599732256 passed CONTRACT-TDRP-STORY-SHARE-VIEWS-001, renderer typecheck/build, and source authority job 113599732231 passed with artifact 11585584799.

## Story views responsibility

The Story model already owns named views plus `anonymous_view_count`. The current change extends the Coordinator DTO/parser to retain `anonymousViewCount` and projects the canonical visible count as `Object.keys(story.views).length + story.anonymousViewCount`. The surface reuses `SandBadge`; it does not import Telegram's eye raster.

Status: the Story views sub-responsibility is verified on exact HEAD ae11d378f92ce19e42e9ccdf59494461672809f3 by the same renderer/source-authority evidence. Orders 1541-1543 nevertheless remain unknown-open because the shared mediaview/views asset also owns the still-unimplemented Poll attachment preview consumer.

## Poll attachment preview responsibility

The second `mediaview/views` consumer is not a view counter. In `Telegram/SourceFiles/poll/poll_media_upload.cpp`, `pollAttachView` is painted over an existing Poll media thumbnail while hovered/pressed and not uploading, providing a preview/view affordance. Current Fabushi shipping frontend has no canonical Poll composer / Poll media attachment preview owner. Therefore this responsibility remains open. A Story views implementation cannot close orders 1541-1543 by itself.

Any future implementation must use the canonical Composer / attachment / Dialog or media-preview owners, real Poll protocol and persistence, keyboard/a11y/touch behavior, and exact-head tests; it must not add a Telegram-specific Poll component solely to consume this icon family.

## Accounting

This dossier makes orders 1538-1543 read-complete and responsibility-decomposed. Deterministic read-through may advance from 1537 to 1543 and unread decreases from 14,583 to 14,577. Story share orders 1538-1540 are now unknown-closed by exact-head evidence. Unknown is therefore 15,988. Orders 1541-1543 remain unknown-open because Poll attachment preview is still unimplemented even though the Story views sub-responsibility is verified. Omitted remains 0.
