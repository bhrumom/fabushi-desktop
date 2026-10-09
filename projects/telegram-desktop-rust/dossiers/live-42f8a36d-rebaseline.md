# Live Telegram rebaseline: 42f8a36d

Authority: telegramdesktop/tdesktop@42f8a36d43b8c805bc821905bea4cfeb3af1d41d / root tree 6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa. Relative to 3a15bf1f this is four commits ahead with exactly nine modified existing paths and no add/delete/gitlink change, so recursive denominator 16,120 is unchanged.

Changed exact blobs:
- Telegram/Resources/uwp/AppX/AppxManifest.xml -> bac5785d2ae4240264577d97444187090e9bbdb7
- Telegram/Resources/winrc/Telegram.rc -> d3ab29c8f583ed0645f4a0de344530180eaa3265
- Telegram/Resources/winrc/Updater.rc -> 5064d9df4282b377272b583071c00de207039abd
- Telegram/SourceFiles/core/version.h -> 84ea0f90963b3a8a8dc3f9595b4e3d37dbf8dedf
- Telegram/SourceFiles/history/view/history_view_message.cpp -> 5152d85e89b6da554095d85a1a16d96d4f969c3d
- Telegram/SourceFiles/ui/chat/attach/attach_bot_webview.cpp -> 540dc2afe3845ca8a0e8db34104f0654e7c522d0
- Telegram/build/build_mac.py -> 881a1c212486976b318777b7f6e5c944da2bd5d3
- Telegram/build/version -> 70a670e4f8e31cebfaae55d32955ee1eb0100c12
- changelog.txt -> bf82383b02717265acac50035c2ce3000afdac7a

Semantic reconciliation: release identity moves 7.2.10 beta to 7.3 stable; build_mac.py adds PkgInfo and localization layout work; history_view_message.cpp makes MarkdownArticleBubbleEdges explicit; attach_bot_webview.cpp removes the prior native-message byte-size early rejection; changelog adds 7.3. build_mac.py adds no acquisition URL/action/clone/ref/pin, so binding sets are semantically unchanged but must be regenerated on the current exact PR HEAD.

Orders 3154/3200/3201/3765/4413 are rebound to live blobs. Prior Actions artifacts are historical-only. baseline_ready and acceptance.accepted stay false.
