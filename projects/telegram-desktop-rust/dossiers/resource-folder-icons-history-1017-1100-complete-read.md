# Telegram folder/history resources 1017–1100 — complete source read

Upstream: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

Source evidence used for this correction: Fabushi Desktop parent HEAD `9864475c9f3997b60383b68772ab6de65979244b`; run `37856862858`; source-authority job `113583254143`; artifact `11584661832`; digest `sha256:b282792a51f3fb2f6d66cd00bad76c575f07147a1cdbfeffcf669fd9cf4bfc50`.

The accepted source-consumer artifact proves the previously mis-accounted raster folder resources are reachable from `Telegram/SourceFiles/ui/filter_icons.style`:

- orders 1017–1019, `folders_unmuted{,@2x,@3x}.png` → `foldersUnmuted` / `foldersUnmutedActive` at lines 76–77 using the exact `"folders/folders_unmuted"` token;
- orders 1020–1022, `folders_unread{,@2x,@3x}.png` → `foldersUnread` / `foldersUnreadActive` at lines 52–53 using the exact `"folders/folders_unread"` token;
- orders 1023–1025, `folders_work{,@2x,@3x}.png` → `foldersWork` / `foldersWorkActive` at lines 74–75 using the exact `"folders/folders_work"` token.

These nine rows therefore belong to the same source-neutral canonical conversation-folder icon vocabulary as the already-accounted raster family. Their source reachability is proven, but shipping implementation, visual/a11y coverage, artwork licensing/adoption, and release verification remain fail-closed; no Telegram artwork is imported and no implementation/verification status is inferred from the resource name.

The remainder of 1017–1100 covers folder SVG/settings-section vocabularies, the intro Fragment affordance, audio/file transfer controls, message comments, and conversation jump navigation. Consumer closure requires an exact resource token/path reference, not a generic basename match. File-transfer semantics continue across the 1100 boundary into orders 1101–1106.
