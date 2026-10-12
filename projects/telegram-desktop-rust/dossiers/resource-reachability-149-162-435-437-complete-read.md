# Revision 9 resource reachability closure — orders 149, 162, 435–437

Authority: `telegramdesktop/tdesktop@63a1831728147e941ad2dc8939879cf5afe419c7` (tree `c259b2d6982213b2231266508316cd23f99f5a47`).

This dossier closes only source-disposition uncertainty for five exact resource blobs. It does **not** claim product implementation, does not use a directory-level N/A rule, and does not weaken any reachable Theme, Design System, CallSession, call-history, recording, group-media, packaging, updater, or release responsibility.

## Exact blobs

- 149 `Telegram/Resources/animations/palette.tgs@cc4887f94716074a79894087d1922f6c31fb1629`
- 162 `Telegram/Resources/animations/robot.tgs@0076344f4c799f30199f553d5402e5f8b8c1daf8`
- 435 `Telegram/Resources/icons/calls/dialogs_user.png@ddbd7b2d037e97f92bca5f984de8b5e3ddcb7349`
- 436 `Telegram/Resources/icons/calls/dialogs_user@2x.png@44ebf40e679920cee4d9b1962bc036ea206f7683`
- 437 `Telegram/Resources/icons/calls/dialogs_user@3x.png@2397372594032592da59869b18ee3c657b225d61`

All five rows were already exact-blob read and responsibility-decomposed; this pass resolves only their previously open consumer reachability.

## Manifest and consumer audit

All eight `Telegram/Resources/qrc/telegram/*.qrc` files at the accepted commit were read for exact resource-name references.

- `palette.tgs` appears exactly once, in `Telegram/Resources/qrc/telegram/animations.qrc@6791a54a5e75d894bb7068d4596d80ba0af01f58`, line 25, as `<file alias="palette.tgs">../../animations/palette.tgs</file>`.
- No qrc/telegram manifest contains `robot.tgs` or `dialogs_user`.
- Accepted-tree exact searches for `:/palette.tgs` and `palette.tgs"_q` find no runtime loader. The only `palette.tgs` hit is the manifest alias above.
- Accepted-tree exact searches for `robot.tgs`, `:/robot.tgs`, and `robot.tgs"_q` find no runtime consumer.
- Accepted-tree exact searches for `dialogs_user`, `calls/dialogs_user`, `dialogsUser`, and `dialogs_user@2x` find no code/style/manifest consumer.

The accepted tree does contain reachable call resources such as `icons/calls/hands.lottie` and `icons/calls/voice.lottie`, with explicit qrc aliases and `ui/controls/call_mute_button.cpp` consumers. That contrast is evidence that the three `dialogs_user` PNGs are not being inferred N/A merely because they live under a call-resource directory.

## Disposition

- Order 149: shipping-bundle orphan presentation resource. Bundling alone is not a product behavior or runtime consumer; no separate Fabushi Palette owner or copied artwork is warranted.
- Order 162: source orphan/unreachable presentation resource. The previous “QRC inclusion confirmed” note was unsupported and is corrected.
- Orders 435–437: source orphan/unreachable call glyph scale variants. No icon-specific owner is created; real call behaviors stay mapped to canonical Fabushi CallSession and related owners.

Each row is closed with `unknown_closed=true` and `omitted=false`. The first shard therefore has only order 58 still unknown-open. Global source accounting becomes read-through 16,137; unread 0; unknown 12,025; omitted 0. Fresh descendant GitHub Actions Source authority evidence is still required after the accounting commit.
